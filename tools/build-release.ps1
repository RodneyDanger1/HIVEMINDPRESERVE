<#
.SYNOPSIS
  HiveMind deterministic release builder.

.DESCRIPTION
  Turns one clean checkout at one commit into a sealed release:
    preflight (git clean/HEAD/tag, toolchain, version, drift policy)
    -> web typecheck/tests/build (+ per-file dist hashes + tree hash)
    -> cargo build --release --locked
    -> drift policy (preserve recorded 1.0 hashes; optional new staging version)
    -> stage README.md/RUNBOOK.md/RELEASE.md + both binaries + CHECKSUMS.txt
    -> deterministic zip (+ .sha256 sidecar)
    -> extract-and-verify every hash
    -> write tools/release-manifest.json  (repo-relative paths only)
    -> absolute-path scan -> summary (old -> new)

  -DryRun performs every read-only check and prints the plan; it executes NO build and writes NO files.

  Seal topology (1.2, CODE-PROOF §6): source commit C holds code/tests/docs; the
  seal commit D = C + artifacts + manifest + release evidence/docs. The release
  tag points at D. -SourceOnly records a v2 manifest with sealed=false and
  tag_target=null at C; -RecordSeal -SealCommit <D> re-verifies the allowlisted
  C..D diff and recorded hashes, then writes sealed=true + seal_diff (manifest
  only, no rebuild). tools/qa/check-seal-topology.mjs is the pure predicate used
  by tools/qa/run-gates.ps1 for the sealed release acceptance.

  Exit codes: 0 ok | 2 precondition | 3 web | 4 cargo | 5 hash drift | 6 verify | 7 path policy.
  No tag is created, moved, or deleted by this script. No key is read. No absolute paths are emitted.
#>
[CmdletBinding()]
param(
    [switch]$DryRun,
    [switch]$AllowDirty,
    [switch]$AllowTagAncestor,
    [string]$Version,
    [string]$Tag,
    [string]$NewStagingVersion,
    [switch]$SkipWebTests,
    [switch]$KeepVerifyDir,
    [switch]$SourceOnly,
    [switch]$RecordSeal,
    [string]$SealCommit
)

$ErrorActionPreference = 'Stop'
$script:Repo = $null
# Windows PowerShell 5.1 does not auto-load these BCL assemblies; Add-Type works on PS 5.1 and PS 7.
try { Add-Type -AssemblyName System.IO.Compression -ErrorAction Stop } catch { }
try { Add-Type -AssemblyName System.IO.Compression.FileSystem -ErrorAction Stop } catch { }

# ---------------------------------------------------------------- named refusals
function Fail([string]$Code, [string]$Message, [int]$ExitCode) {
    Write-Host ("[{0}] {1}" -f $Code, $Message) -ForegroundColor Red
    Write-Host "RELEASE REFUSED - nothing was published." -ForegroundColor Red
    exit $ExitCode
}

function Step([string]$Message) { Write-Host ("=== " + $Message + " ===") -ForegroundColor Cyan }

# ---------------------------------------------------------------- helpers
function Rel([string]$Path) {
    $full = [System.IO.Path]::GetFullPath($Path)
    $base = [System.IO.Path]::GetFullPath($script:Repo)
    if ($full.StartsWith($base, [System.StringComparison]::OrdinalIgnoreCase)) {
        $r = $full.Substring($base.Length).TrimStart('\', '/')
        return ($r -replace '\\', '/')
    }
    return ($full -replace '\\', '/')
}

function Sha256([string]$Path) {
    return (Get-FileHash -Algorithm SHA256 -LiteralPath $Path).Hash.ToLowerInvariant()
}

function Write-Utf8NoBom([string]$Path, [string]$Text) {
    $enc = New-Object System.Text.UTF8Encoding($false)
    [System.IO.File]::WriteAllText($Path, $Text, $enc)
}

function TreeHash([object[]]$Files) {
    $lines = New-Object System.Collections.Generic.List[string]
    foreach ($f in ($Files | Sort-Object { $_.path })) {
        $lines.Add(("{0}`t{1}`t{2}" -f $f.path, $f.sha256, $f.bytes))
    }
    $joined = [string]::Join("`n", $lines) + "`n"
    $sha = [System.Security.Cryptography.SHA256]::Create()
    $hash = $sha.ComputeHash([System.Text.Encoding]::UTF8.GetBytes($joined))
    $sha.Dispose()
    return ([System.BitConverter]::ToString($hash) -replace '-', '').ToLowerInvariant()
}

function HashFileList([string[]]$Paths) {
    $out = @()
    foreach ($p in $Paths) {
        if (-not (Test-Path -LiteralPath $p)) { return $null }
        $out += [pscustomobject]@{
            name = [System.IO.Path]::GetFileName($p)
            path = (Rel $p)
            sha256 = (Sha256 $p)
            bytes = (Get-Item -LiteralPath $p).Length
        }
    }
    return ,$out
}

function Sha256Text([string]$Text) {
    $sha = [System.Security.Cryptography.SHA256]::Create()
    try {
        $hash = $sha.ComputeHash([System.Text.Encoding]::UTF8.GetBytes($Text))
        return ([System.BitConverter]::ToString($hash) -replace '-', '').ToLowerInvariant()
    }
    finally { $sha.Dispose() }
}

# Exact 1.2 seal allowlist (CODE-PROOF §6). Must stay identical to
# tools/qa/check-seal-topology.mjs SEAL_ALLOWLIST.
$script:SealAllowlist = @(
    'tools/release-manifest.json',
    'dist/**',
    'docs/RELEASE-EVIDENCE.md',
    'plans/waves/README.md',
    'docs/RELEASE.md',
    'README.md'
)

function Test-SealAllowlisted([string]$Path, [string[]]$Allowlist) {
    $p = (($Path -replace '\\', '/')).Trim()
    if (-not $p) { return $false }
    if ($p.StartsWith('/') -or $p -match '^[A-Za-z]:' -or $p -match '(^|/)\.\.(/|$)') { return $false }
    foreach ($entry in $Allowlist) {
        $e = (([string]$entry) -replace '\\', '/').Trim()
        if (-not $e) { continue }
        if ($e.EndsWith('/**')) {
            $prefix = $e.Substring(0, $e.Length - 3).TrimEnd('/')
            if ($p -eq $prefix -or $p.StartsWith($prefix + '/')) { return $true }
        }
        elseif ($p -eq $e) { return $true }
    }
    return $false
}

function Test-ManifestFileRow([string]$RelPath, [string]$ExpectedSha, [object]$ExpectedBytes, [string]$Label) {
    if (-not $RelPath) { return ($Label + ': manifest path missing') }
    $p = Join-Path $script:Repo ($RelPath -replace '/', '\')
    if (-not (Test-Path -LiteralPath $p)) { return ($Label + ' missing: ' + $RelPath) }
    $sha = Sha256 $p
    if ($ExpectedSha -and $sha -ne $ExpectedSha) { return ($Label + ' sha mismatch: ' + $RelPath) }
    if ($null -ne $ExpectedBytes -and [string]$ExpectedBytes -ne '') {
        $bytes = (Get-Item -LiteralPath $p).Length
        if ([int64]$ExpectedBytes -ne [int64]$bytes) { return ($Label + ' bytes mismatch: ' + $RelPath) }
    }
    return $null
}

function Get-ManifestArtifactDrift([object]$Manifest) {
    $mismatches = @()
    if (-not $Manifest.artifacts -or @($Manifest.artifacts).Count -eq 0) {
        return @('manifest has no artifacts[] to verify')
    }
    foreach ($a in @($Manifest.artifacts)) {
        $m = Test-ManifestFileRow ([string]$a.path) ([string]$a.sha256) $a.bytes 'artifact'
        if ($m) { $mismatches += $m }
    }
    if (-not $Manifest.package) {
        $mismatches += 'manifest has no package block'
    }
    else {
        $m = Test-ManifestFileRow ([string]$Manifest.package.zip) ([string]$Manifest.package.zip_sha256) $Manifest.package.zip_bytes 'package zip'
        if ($m) { $mismatches += $m }
        if ($Manifest.package.sidecar) {
            $m = Test-ManifestFileRow ([string]$Manifest.package.sidecar) ([string]$Manifest.package.sidecar_sha256) $null 'package sidecar'
            if ($m) { $mismatches += $m }
        }
        if ($Manifest.package.checksums) {
            $m = Test-ManifestFileRow ([string]$Manifest.package.checksums) ([string]$Manifest.package.checksums_sha256) $null 'package CHECKSUMS'
            if ($m) { $mismatches += $m }
        }
    }
    if ($Manifest.web -and $Manifest.web.files) {
        foreach ($wf in @($Manifest.web.files)) {
            $m = Test-ManifestFileRow ([string]$wf.path) ([string]$wf.sha256) $wf.bytes 'web dist file'
            if ($m) { $mismatches += $m }
        }
    }
    else { $mismatches += 'manifest has no web.files[] to verify' }
    return @($mismatches)
}

function New-DeterministicZip([string]$ZipPath, [object[]]$Files) {
    $stream = [System.IO.File]::Open($ZipPath, [System.IO.FileMode]::Create, [System.IO.FileAccess]::Write)
    try {
        $zip = New-Object System.IO.Compression.ZipArchive($stream, [System.IO.Compression.ZipArchiveMode]::Create, $false)
        try {
            $fixed = New-Object System.DateTimeOffset(1980, 1, 1, 0, 0, 0, [System.TimeSpan]::Zero)
            foreach ($f in ($Files | Sort-Object { $_.name })) {
                $entry = $zip.CreateEntry($f.name, [System.IO.Compression.CompressionLevel]::Optimal)
                $entry.LastWriteTime = $fixed
                $es = $entry.Open()
                try {
                    $fs = [System.IO.File]::OpenRead($f.fullpath)
                    try { $fs.CopyTo($es) } finally { $fs.Dispose() }
                }
                finally { $es.Dispose() }
            }
        }
        finally { $zip.Dispose() }
    }
    finally { $stream.Dispose() }
}

function Get-ZipEntries([string]$ZipPath) {
    $result = @()
    $stream = [System.IO.File]::OpenRead($ZipPath)
    try {
        $zip = New-Object System.IO.Compression.ZipArchive($stream, [System.IO.Compression.ZipArchiveMode]::Read, $false)
        try {
            foreach ($e in $zip.Entries) {
                $esh = [System.Security.Cryptography.SHA256]::Create()
                $s = $e.Open()
                try {
                    $ms = New-Object System.IO.MemoryStream
                    try {
                        $s.CopyTo($ms)
                        $bytes = $ms.ToArray()
                    }
                    finally { $ms.Dispose() }
                }
                finally { $s.Dispose() }
                $hash = $esh.ComputeHash($bytes)
                $esh.Dispose()
                $result += [pscustomobject]@{
                    name = $e.FullName
                    bytes = $e.Length
                    sha256 = ([System.BitConverter]::ToString($hash) -replace '-', '').ToLowerInvariant()
                }
            }
        }
        finally { $zip.Dispose() }
    }
    finally { $stream.Dispose() }
    return ,$result
}

function Test-AbsolutePaths([string]$Text, [string]$Where) {
    $matches = [regex]::Matches($Text, '(?<![A-Za-z0-9])[A-Za-z]:[\\/]|\\\\[A-Za-z0-9]')
    if ($matches.Count -gt 0) {
        $samples = @()
        foreach ($m in $matches) { $samples += $m.Value }
        Fail "E_RELEASE_ABSOLUTE_PATH" ("absolute path(s) found in " + $Where + ": " + (($samples | Select-Object -Unique) -join ', ')) 7
    }
}

function Invoke-Step([string]$ExePath, [string[]]$Arguments, [string]$WorkingDirectory, [string]$FailCode, [int]$FailExit) {
    Write-Host ("`$ " + $ExePath + " " + ($Arguments -join ' ')) -ForegroundColor DarkGray
    Push-Location $WorkingDirectory
    try {
        & $ExePath @Arguments
        $code = $LASTEXITCODE
    }
    finally { Pop-Location }
    if ($code -ne 0) { Fail $FailCode ("command failed (exit " + $code + "): " + $ExePath + " " + ($Arguments -join ' ')) $FailExit }
}

# ---------------------------------------------------------------- preflight
$script:Repo = (Resolve-Path (Join-Path $PSScriptRoot '..')).Path
Push-Location $script:Repo
try {
    Step "preflight: git state"
    $git = 'git'
    $head = (& $git rev-parse HEAD).Trim()
    if ($LASTEXITCODE -ne 0) { Fail "E_RELEASE_GIT" "git rev-parse HEAD failed" 2 }
    $describe = (& $git describe --tags --always).Trim()
    $status = @(& $git status --porcelain)
    $commitDate = (& $git show -s --format=%cI HEAD).Trim()

    if ($status.Count -gt 0 -and -not $AllowDirty) {
        Fail "E_RELEASE_DIRTY_TREE" ("working tree is not clean (" + $status.Count + " path(s)). Commit/stash first; -AllowDirty is for rehearsal only.") 2
    }
    if ($status.Count -eq 0) { $dirty = $false } else { $dirty = $true }

    # version: crate version (single source), overridable
    $coreToml = Join-Path $script:Repo 'backend/crates/hiverouter-core/Cargo.toml'
    if (-not (Test-Path -LiteralPath $coreToml)) { Fail "E_RELEASE_VERSION" "backend/crates/hiverouter-core/Cargo.toml not found" 2 }
    if (-not $Version) {
        $m = [regex]::Match((Get-Content -LiteralPath $coreToml -Raw), '(?m)^\s*version\s*=\s*"([0-9]+\.[0-9]+\.[0-9]+)"')
        if (-not $m.Success) { Fail "E_RELEASE_VERSION" "could not parse crate version from hiverouter-core/Cargo.toml" 2 }
        $Version = $m.Groups[1].Value
    }
    [void]([version]$Version)
    if (-not $Tag) { $Tag = "hivemind-$Version" }

    # existing manifest (preserve recorded 1.0 hashes / drive drift policy)
    $manifestPath = Join-Path $script:Repo 'tools/release-manifest.json'
    $existingManifest = $null
    if (Test-Path -LiteralPath $manifestPath) {
        try { $existingManifest = Get-Content -LiteralPath $manifestPath -Raw | ConvertFrom-Json }
        catch { Fail "E_RELEASE_MANIFEST" ("existing tools/release-manifest.json does not parse: " + $_.Exception.Message) 2 }
    }
    $manifestVersion = if ($existingManifest -and $existingManifest.version) { [string]$existingManifest.version } else { '0.0.0' }

    # ---------------------------------------------------------------- seal mode prechecks (1.2)
    if ($SourceOnly -and $RecordSeal) {
        Fail "E_RELEASE_FLAGS" "-SourceOnly and -RecordSeal are mutually exclusive" 2
    }
    if ($RecordSeal -and -not $SealCommit) {
        Fail "E_RELEASE_FLAGS" "-RecordSeal requires -SealCommit <full 40-hex seal commit D>" 2
    }
    if ($SealCommit -and -not $RecordSeal) {
        Fail "E_RELEASE_FLAGS" "-SealCommit is only valid with -RecordSeal" 2
    }
    if ($SealCommit -and ($SealCommit.Trim() -notmatch '^[0-9a-fA-F]{40}$')) {
        Fail "E_RELEASE_FLAGS" "-SealCommit must be a full 40-hex git commit id" 2
    }
    if ($RecordSeal -and $NewStagingVersion) {
        Fail "E_RELEASE_FLAGS" "-RecordSeal cannot be combined with -NewStagingVersion (seal records the existing manifest; no rebuild)" 2
    }

    if ($RecordSeal) {
        Step "record seal: source C -> seal D"
        if ($dirty) {
            Fail "E_RELEASE_SEAL_STATE" "-RecordSeal requires a clean working tree (the seal evidence is the committed D tree); commit the seal commit D first." 2
        }
        $sealCommitId = $SealCommit.Trim().ToLowerInvariant()
        if (-not $existingManifest) {
            Fail "E_RELEASE_SEAL_STATE" "no tools/release-manifest.json; run -SourceOnly at the source commit first" 2
        }
        if ([string]$existingManifest.schema -ne 'hivemind.release-manifest.v2') {
            Fail "E_RELEASE_SEAL_STATE" ("manifest schema '" + [string]$existingManifest.schema + "' is not hivemind.release-manifest.v2; run -SourceOnly first") 2
        }
        if ([bool]$existingManifest.sealed) {
            Fail "E_RELEASE_SEAL_STATE" "manifest is already sealed; a seal commit is recorded exactly once" 2
        }
        $manifestSource = ([string]$existingManifest.source_commit).Trim()
        if ($manifestSource -notmatch '^[0-9a-fA-F]{40}$') {
            Fail "E_RELEASE_SEAL_STATE" "manifest.source_commit is not a full commit id; rebuild the source manifest" 2
        }
        if ([string]$existingManifest.version -ne $Version) {
            Fail "E_RELEASE_SEAL_STATE" ("manifest version " + [string]$existingManifest.version + " != current version " + $Version) 2
        }
        if ([string]$existingManifest.tag -ne $Tag) {
            Fail "E_RELEASE_SEAL_STATE" ("manifest tag " + [string]$existingManifest.tag + " != " + $Tag) 2
        }
        $sealHead = $head.Trim()
        if ($sealHead -ne $sealCommitId) {
            Fail "E_RELEASE_SEAL_COMMIT" ("HEAD " + $sealHead.Substring(0, 8) + " != -SealCommit " + $sealCommitId.Substring(0, 8) + "; check out the seal commit D before recording") 2
        }
        & $git cat-file -e ($sealCommitId + '^{commit}') 2>$null | Out-Null
        if ($LASTEXITCODE -ne 0) {
            Fail "E_RELEASE_SEAL_COMMIT" ("-SealCommit " + $SealCommit + " is not a commit in this repository") 2
        }
        if ($manifestSource -eq $sealCommitId) {
            Fail "E_RELEASE_SEAL_COMMIT" "seal commit D must be a child commit (artifacts/manifest/evidence), not the source commit C" 2
        }
        & $git merge-base --is-ancestor $manifestSource $sealCommitId 2>$null | Out-Null
        if ($LASTEXITCODE -ne 0) {
            Fail "E_RELEASE_SEAL_COMMIT" "seal commit D is not a descendant of manifest source_commit C" 2
        }
        $sealDiffNames = @(& $git diff --name-only ($manifestSource + '..' + $sealCommitId) 2>$null)
        if ($LASTEXITCODE -ne 0) {
            Fail "E_RELEASE_SEAL_COMMIT" "git diff between manifest source_commit C and -SealCommit D failed" 2
        }
        $manifestAllow = @()
        if ($existingManifest.seal_allowlist) { $manifestAllow = @($existingManifest.seal_allowlist | ForEach-Object { [string]$_ }) }
        $expectedAllow = @($script:SealAllowlist)
        if ((($manifestAllow | Sort-Object) -join '|') -ne (($expectedAllow | Sort-Object) -join '|')) {
            Fail "E_RELEASE_SEAL_STATE" "manifest seal_allowlist is missing or differs from the exact 1.2 allowlist; rebuild the source manifest with tools/build-release.ps1 -SourceOnly" 2
        }
        $sealOffenders = @()
        foreach ($p in $sealDiffNames) {
            $rel = (([string]$p) -replace '\\', '/').Trim()
            if (-not (Test-SealAllowlisted $rel $expectedAllow)) { $sealOffenders += $rel }
        }
        if ($sealOffenders.Count -gt 0) {
            Fail "E_SEAL_DIFF" ("seal diff " + $manifestSource.Substring(0, 8) + ".." + $sealCommitId.Substring(0, 8) + " contains non-allowlisted path(s): " + (($sealOffenders | Select-Object -First 10) -join ', ')) 2
        }
        $artifactDrift = @(Get-ManifestArtifactDrift $existingManifest)
        if ($artifactDrift.Count -gt 0) {
            Fail "E_SEAL_ARTIFACT" ("recorded artifact hashes do not match the seal tree: " + (($artifactDrift | Select-Object -First 6) -join '; ')) 5
        }
        $sortedSealNames = @($sealDiffNames | ForEach-Object { (([string]$_) -replace '\\', '/').Trim() } | Where-Object { $_ } | Sort-Object)
        $sealDiffSha = Sha256Text ((($sortedSealNames | Sort-Object) -join "`n") + "`n")
        Step ("seal diff: " + $sortedSealNames.Count + " allowlisted path(s) sha256=" + $sealDiffSha)
        if ($DryRun) {
            Write-Host ""
            Step "DRY RUN - RecordSeal plan (no writes)"
            Write-Host ("source C: " + $manifestSource + "  seal D: " + $sealCommitId)
            Write-Host ("allowlist: " + ($expectedAllow -join ', '))
            Write-Host ("diff files: " + ($sortedSealNames -join ', '))
            Write-Host "would write tools/release-manifest.json with sealed=true, seal_diff.from/to/sha256; no rebuild."
            Write-Host "DRY RUN OK - no files were written." -ForegroundColor Green
            exit 0
        }
        $sealedManifest = $existingManifest
        $sealedManifest | Add-Member -NotePropertyName schema -NotePropertyValue 'hivemind.release-manifest.v2' -Force
        $sealedManifest | Add-Member -NotePropertyName sealed -NotePropertyValue $true -Force
        $sealedManifest | Add-Member -NotePropertyName sealed_at -NotePropertyValue ((Get-Date).ToString('yyyy-MM-ddTHH:mm:sszzz')) -Force
        $sealedManifest | Add-Member -NotePropertyName tag_target -NotePropertyValue $null -Force
        $sealedManifest | Add-Member -NotePropertyName seal_diff -NotePropertyValue ([ordered]@{ from = $manifestSource; to = $sealCommitId; sha256 = $sealDiffSha; files = @($sortedSealNames) }) -Force
        $sealedManifest | Add-Member -NotePropertyName note -NotePropertyValue ('Sealed by tools/build-release.ps1 -RecordSeal: source ' + $manifestSource + '; seal ' + $sealCommitId + '; only allowlisted evidence/artifact paths differ. Tag target is resolved by tools/qa/run-gates.ps1; no rebuild, no tag move.') -Force
        if (Test-Path -LiteralPath $manifestPath) {
            $sealBackup = Join-Path $script:Repo ("dist/.release-manifest-seal-" + (Get-Date -Format 'yyyyMMddHHmmss') + ".json")
            Copy-Item -LiteralPath $manifestPath -Destination $sealBackup -Force
        }
        $sealedJson = ($sealedManifest | ConvertTo-Json -Depth 10)
        Write-Utf8NoBom $manifestPath ($sealedJson + "`n")
        Test-AbsolutePaths (Get-Content -LiteralPath $manifestPath -Raw) 'tools/release-manifest.json'
        Step "DONE - seal recorded (manifest only; no rebuild)"
        Write-Host ("source C: " + $manifestSource)
        Write-Host ("seal D:   " + $sealCommitId)
        Write-Host ("diff:     " + $sortedSealNames.Count + " allowlisted path(s) sha256=" + $sealDiffSha)
        exit 0
    }

    # effective staging version (proactive supersede); drift-driven bump is refused post-build
    $effectiveVersion = $Version
    $supersedeReason = $null
    if ($NewStagingVersion) {
        [void]([version]$NewStagingVersion)
        if ([version]$NewStagingVersion -le [version]$Version) {
            Fail "E_RELEASE_VERSION" ("-NewStagingVersion " + $NewStagingVersion + " must be greater than current version " + $Version) 2
        }
        $effectiveVersion = $NewStagingVersion
        $supersedeReason = "operator-requested staging version"
        if (-not $Tag -or $Tag -eq ("hivemind-" + $Version)) { $Tag = "hivemind-" + $effectiveVersion }
    }
    # tag gate
    $tagExists = $false
    $tagTarget = $null
    $behind = $null
    if ($Tag) {
        $tagList = @(& $git tag --list $Tag)
        if ($tagList.Count -gt 0) {
            $tagExists = $true
            $tagTarget = (& $git rev-list -n1 $Tag).Trim()
            $behind = [int]((& $git rev-list --count ($Tag + '..HEAD')).Trim())
        }
    }
    if ($SourceOnly) {
        # Source-only build at C: the release tag is created later at seal commit D.
        # An existing tag remains immutable: rebuilding a source commit for a tag
        # that points elsewhere is refused (no silent tag/source mismatch).
        if ($tagExists -and $tagTarget -ne $head) {
            Fail "E_RELEASE_TAG_MISMATCH" ("tag '" + $Tag + "' points at " + $tagTarget.Substring(0, 8) + ", not source HEAD " + $head.Substring(0, 8) + "; an existing release tag is immutable") 2
        }
        if ($tagExists) {
            Write-Host ("source-only build at a tagged commit; tag '" + $Tag + "' stays put; manifest records sealed=false/tag_target=null.") -ForegroundColor Yellow
        }
        else {
            Write-Host ("source-only build: tag '" + $Tag + "' is expected to be created at the seal commit after evidence; manifest records sealed=false/tag_target=null.") -ForegroundColor Yellow
        }
        $tagTarget = $null
        $behind = $null
    }
    elseif (-not $tagExists) {
        if (-not $NewStagingVersion) { Fail "E_RELEASE_TAG_MISSING" ("tag '" + $Tag + "' does not exist. Create the annotated tag first (owner action), or use -NewStagingVersion for a staged superseding release.") 2 }
        Write-Host ("WARN: tag '" + $Tag + "' missing; -NewStagingVersion was supplied, continuing as staged release.") -ForegroundColor Yellow
    }
    else {
        $isAncestor = $false
        & $git merge-base --is-ancestor $tagTarget $head | Out-Null
        if ($LASTEXITCODE -eq 0) { $isAncestor = $true }
        if ($tagTarget -ne $head) {
            if (-not ($AllowTagAncestor -and $isAncestor)) {
                Fail "E_RELEASE_TAG_MISMATCH" ("HEAD " + $head.Substring(0,8) + " is not the tagged commit " + $tagTarget.Substring(0,8) + " for '" + $Tag + "' (HEAD is " + $behind + " commit(s) ahead). Decide: retag at HEAD (owner), or build the tagged commit, or -AllowTagAncestor for a rehearsal.") 2
            }
            Write-Host ("WARN: rehearsal build from HEAD ahead of tag '" + $Tag + "' by " + $behind + " commit(s); manifest records tag_behind_commits.") -ForegroundColor Yellow
        }
    }

    # release paths (single definition; must never be empty)
    $stagingRel = "dist/HiveMind-" + $effectiveVersion + "-win64"
    $zipRel = $stagingRel + ".zip"

    Step "preflight: toolchain"
    $cargoCandidates = @()
    if ($env:USERPROFILE) { $cargoCandidates += (Join-Path $env:USERPROFILE '.cargo\bin\cargo.exe') }
    $cmd = Get-Command cargo.exe -ErrorAction SilentlyContinue
    if ($cmd) { $cargoCandidates += $cmd.Source }
    $cargo = $cargoCandidates | Where-Object { Test-Path -LiteralPath $_ } | Select-Object -First 1
    if (-not $cargo) { Fail "E_TOOLCHAIN_MISSING" "cargo.exe not found (looked in %USERPROFILE%\.cargo\bin and PATH)" 2 }
    $rustc = (Join-Path (Split-Path -Parent $cargo) 'rustc.exe')
    if (-not (Test-Path -LiteralPath $rustc)) { Fail "E_TOOLCHAIN_MISSING" ("rustc.exe not found next to cargo: " + $rustc) 2 }
    $cargoVersion = (& $cargo --version).Trim()
    $rustcVersion = (& $rustc --version).Trim()
    $nodeCmd = Get-Command node.exe -ErrorAction SilentlyContinue
    if (-not $nodeCmd) { Fail "E_TOOLCHAIN_MISSING" "node.exe not found on PATH" 2 }
    $nodeVersion = (& $nodeCmd.Source --version).Trim()
    $npmCmd = Get-Command npm.cmd -ErrorAction SilentlyContinue
    if (-not $npmCmd) {
        $npmP = Join-Path (Split-Path -Parent $nodeCmd.Source) 'npm.cmd'
        if (Test-Path -LiteralPath $npmP) { $npmPath = $npmP } else { Fail "E_TOOLCHAIN_MISSING" "npm.cmd not found (npm.ps1 is blocked by execution policy; use npm.cmd)" 2 }
    } else { $npmPath = $npmCmd.Source }
    $npmVersion = (& $npmPath --version).Trim()
    $msrv = [version]'1.89.0'
    $rustParsed = [regex]::Match($rustcVersion, 'rustc\s+([0-9]+\.[0-9]+\.[0-9]+)')
    if (-not $rustParsed.Success) { Fail "E_TOOLCHAIN_MSRV" ("cannot parse rustc version: " + $rustcVersion) 2 }
    if ([version]$rustParsed.Groups[1].Value -lt $msrv) { Fail "E_TOOLCHAIN_MSRV" ("rustc " + $rustParsed.Groups[1].Value + " < MSRV 1.89.0") 2 }
    $nodeParsed = [regex]::Match($nodeVersion, 'v?([0-9]+\.[0-9]+\.[0-9]+)')
    if (-not $nodeParsed.Success -or [version]$nodeParsed.Groups[1].Value -lt [version]'22.12.0') {
        Fail "E_TOOLCHAIN_MSRV" ("node " + $nodeVersion + " < required 22.12.0") 2
    }
    $gccPath = Join-Path $script:Repo 'tools/w64devkit/bin/gcc.exe'
    if (-not (Test-Path -LiteralPath $gccPath)) {
        Write-Host "WARN: tools/w64devkit/bin/gcc.exe missing (required for rusqlite bundled build)." -ForegroundColor Yellow
        $gccVersion = "missing"
    } else { $gccVersion = (& $gccPath --version | Select-Object -First 1).Trim() }

    Step "preflight: provider contract facts"
    $provider = [ordered]@{
        endpoint = 'https://api.deepseek.com/chat/completions'
        models = @('deepseek-flash', 'deepseek-v4-pro')
        max_output_tokens = 393216
        key_env = 'DEEPSEEK_API_KEY'
        source = 'official DeepSeek docs (api-docs.deepseek.com) + backend/crates/hiverouter-provider/src/dto.rs'
    }
    $dtoPath = Join-Path $script:Repo 'backend/crates/hiverouter-provider/src/dto.rs'
    if (Test-Path -LiteralPath $dtoPath) {
        $dtoText = Get-Content -LiteralPath $dtoPath -Raw
        foreach ($modelId in $provider.models) {
            if ($dtoText -notmatch [regex]::Escape('"' + $modelId + '"')) {
                Fail "E_RELEASE_PROVIDER_CONTRACT" ("provider model id '" + $modelId + "' not found in dto.rs; release manifest would be stale") 2
            }
        }
    } else {
        Write-Host "WARN: provider dto.rs not found; using documented provider facts without source cross-check." -ForegroundColor Yellow
    }

    Step "preflight: recorded 1.0 artifacts vs disk (drift preview)"
    $artifactNames = @('hiverouter.exe', 'hiverouter-cli.exe')
    $targetDir = Join-Path $script:Repo 'target/release'
    $diskArtifacts = @()
    foreach ($n in $artifactNames) {
        $p = Join-Path $targetDir $n
        if (Test-Path -LiteralPath $p) {
            $diskArtifacts += [pscustomobject]@{ name = $n; path = (Rel $p); sha256 = (Sha256 $p); bytes = (Get-Item -LiteralPath $p).Length }
        }
    }
    $recordedArtifacts = @()
    if ($existingManifest -and $existingManifest.artifacts) { $recordedArtifacts = @($existingManifest.artifacts) }
    $drift = $false
    $driftRows = @()
    foreach ($d in $diskArtifacts) {
        $rec = $recordedArtifacts | Where-Object { $_.path -like ('*' + $d.name) } | Select-Object -First 1
        if ($rec) {
            if ($rec.sha256 -ne $d.sha256) {
                $drift = $true
                $driftRows += [pscustomobject]@{ name = $d.name; recorded = $rec.sha256; on_disk = $d.sha256 }
            }
        }
    }
    $recZip = $null
    if ($existingManifest -and $existingManifest.package) { $recZip = $existingManifest.package }
    if (-not $recZip -and $existingManifest -and $existingManifest.artifacts) {
        foreach ($a in $existingManifest.artifacts) { if ($a.path -like '*HiveMind-*.zip') { $recZip = $a } }
    }
    $diskZipPath = $null
    $recordedZipPath = $null
    if ($recZip) { $recordedZipPath = $recZip.path }
    if ($recordedZipPath) { $diskZipPath = Join-Path $script:Repo ($recordedZipPath -replace '/', '\') }
    $recordedZipSha = $null
    if ($recZip -and $recZip.sha256) { $recordedZipSha = $recZip.sha256 }
    if ($diskZipPath -and (Test-Path -LiteralPath $diskZipPath)) {
        $currentZipSha = Sha256 $diskZipPath
        Write-Host ("recorded zip: " + $recordedZipPath + " sha " + $recordedZipSha)
        Write-Host ("on-disk zip: " + $currentZipSha + " (match=" + ($currentZipSha -eq $recordedZipSha) + ")")
    }

    if ($drift -and $effectiveVersion -eq $manifestVersion) {
        Fail "E_RELEASE_HASH_DRIFT" ("a rebuild would change recorded 1.0 artifact bytes. Preserve the 1.0 hashes: pass -NewStagingVersion 1.1.0 to stage a superseding release (old hashes kept in supersedes[]).") 5
    }
    $sidecarRel = $zipRel + ".sha256"

    Step "preflight: summary"
    Write-Host ("repo:              " + $script:Repo)
    Write-Host ("HEAD:              " + $head + " (" + $describe + ")")
    Write-Host ("dirty:             " + $dirty)
    Write-Host ("version:           " + $effectiveVersion + "  staging: " + $stagingRel)
    Write-Host ("tag:               " + $Tag + "  exists=" + $tagExists + "  behind=" + $behind)
    Write-Host ("cargo/rustc:       " + $cargoVersion + " / " + $rustcVersion)
    Write-Host ("node/npm:          " + $nodeVersion + " / " + $npmVersion + "  gcc: " + $gccVersion)
    Write-Host ("drift detected:    " + $drift)
    foreach ($r in $driftRows) { Write-Host ("  drift " + $r.name + ": recorded " + $r.recorded + " -> on-disk " + $r.on_disk) }

    if ($DryRun) {
        $webDistDir = Join-Path $script:Repo 'web/dist'
        $webFiles = @()
        if (Test-Path -LiteralPath $webDistDir) {
            $webFiles = @(Get-ChildItem -LiteralPath $webDistDir -Recurse -File | ForEach-Object {
                [pscustomobject]@{ path = (Rel $_.FullName); sha256 = (Sha256 $_.FullName); bytes = $_.Length }
            })
        }
        Write-Host ""
        Step "DRY RUN plan (no build, no writes)"
        Write-Host "1. web: npm ci --no-audit --no-fund -> run typecheck -> run test -> run build"
        Write-Host ("   current web/dist files: " + $webFiles.Count + "  tree= " + (TreeHash $webFiles))
        Write-Host "2. cargo build --release --locked (backend/)"
        Write-Host ("3. drift policy: effective version " + $effectiveVersion + ($(if ($supersedeReason) { " (" + $supersedeReason + ")" } else { "" })))
        Write-Host ("4. stage " + $stagingRel + "/ {README.md, RUNBOOK.md, RELEASE.md, hiverouter.exe, hiverouter-cli.exe, CHECKSUMS.txt}")
        Write-Host ("5. zip " + $zipRel + " (deterministic) + sidecar " + $sidecarRel)
        Write-Host ("6. extract+verify every member hash against staging")
        Write-Host "7. write tools/release-manifest.json (schema hivemind.release-manifest.v2; sealed=false at source build)"
        if ($SourceOnly) {
            Write-Host ("   SOURCE-ONLY: tag_target=null; tag '" + $Tag + "' is created at the later seal commit D (owner action); sealed=false.")
        }
        else {
            Write-Host ("   tag_target=" + $(if ($tagTarget) { $tagTarget } else { 'null' }) + "; sealed=false; seal_allowlist + seal_diff=null.")
        }
        Write-Host "8. absolute-path scan -> summary old->new"
        if ($diskZipPath -and (Test-Path -LiteralPath $diskZipPath)) {
            $entries = Get-ZipEntries $diskZipPath
            Write-Host ("existing zip members: " + (($entries | ForEach-Object { $_.name }) -join ', '))
        }
        Write-Host ""
        Write-Host "DRY RUN OK - no files were written." -ForegroundColor Green
        exit 0
    }

    # ---------------------------------------------------------------- web build
    Step "web: install -> typecheck -> tests -> build"
    $webDir = Join-Path $script:Repo 'web'
    Invoke-Step $npmPath @('ci', '--no-audit', '--no-fund') $webDir 'E_WEB_BUILD_FAILED' 3
    Invoke-Step $npmPath @('run', 'typecheck') $webDir 'E_WEB_BUILD_FAILED' 3
    if (-not $SkipWebTests) { Invoke-Step $npmPath @('run', 'test', '--', '--run') $webDir 'E_WEB_BUILD_FAILED' 3 }
    Invoke-Step $npmPath @('run', 'build') $webDir 'E_WEB_BUILD_FAILED' 3
    $webDistDir = Join-Path $script:Repo 'web/dist'
    if (-not (Test-Path -LiteralPath $webDistDir)) { Fail "E_WEB_BUILD_FAILED" "web/dist missing after build" 3 }
    $webFiles = @(Get-ChildItem -LiteralPath $webDistDir -Recurse -File | ForEach-Object {
        [pscustomobject]@{ path = (Rel $_.FullName); sha256 = (Sha256 $_.FullName); bytes = $_.Length }
    } | Sort-Object { $_.path })
    $webTree = TreeHash $webFiles
    Write-Host ("web/dist files: " + $webFiles.Count + "  tree sha256: " + $webTree)

    # ---------------------------------------------------------------- cargo build
    Step "cargo: build --release --locked"
    $env:PATH = (Split-Path -Parent $cargo) + ';' + (Join-Path $script:Repo 'tools/w64devkit/bin') + ';' + $env:PATH
    $backendDir = Join-Path $script:Repo 'backend'
    Invoke-Step $cargo @('build', '--release', '--locked') $backendDir 'E_CARGO_BUILD_FAILED' 4
    $builtArtifacts = @()
    foreach ($n in $artifactNames) {
        $p = Join-Path $targetDir $n
        if (-not (Test-Path -LiteralPath $p)) { Fail "E_CARGO_BUILD_FAILED" ("expected build artifact missing: " + (Rel $p)) 4 }
        $builtArtifacts += [pscustomobject]@{ name = $n; path = (Rel $p); sha256 = (Sha256 $p); bytes = (Get-Item -LiteralPath $p).Length }
    }
    # drift policy now uses real build output
    $realDrift = $false
    $oldToNew = @()
    foreach ($d in $builtArtifacts) {
        $rec = $recordedArtifacts | Where-Object { $_.path -like ('*' + $d.name) } | Select-Object -First 1
        if ($rec -and $rec.sha256 -ne $d.sha256) {
            $realDrift = $true
            $oldToNew += [pscustomobject]@{ artifact = $d.name; old_sha256 = $rec.sha256; new_sha256 = $d.sha256; old_bytes = $rec.bytes; new_bytes = $d.bytes }
        }
    }
    if ($realDrift -and $effectiveVersion -eq $manifestVersion) {
        Fail "E_RELEASE_HASH_DRIFT" "build changed artifact bytes but staging version was not bumped; recorded 1.0 hashes must be preserved (-NewStagingVersion required)." 5
    }
    if ($realDrift -and $effectiveVersion -ne $manifestVersion) {
        Write-Host ("superseding release " + $manifestVersion + " -> " + $effectiveVersion + " (old -> new):") -ForegroundColor Yellow
        foreach ($r in $oldToNew) { Write-Host ("  " + $r.artifact + ": " + $r.old_sha256 + " -> " + $r.new_sha256) }
    }

    # backup existing manifest inside ignored dist/ (never overwrite history silently)
    if (Test-Path -LiteralPath $manifestPath) {
        $backup = Join-Path $script:Repo ("dist/.release-manifest-" + (Get-Date -Format 'yyyyMMddHHmmss') + ".json")
        Copy-Item -LiteralPath $manifestPath -Destination $backup -Force
    }
    $manifestVersion = if ($existingManifest -and $existingManifest.version) { [string]$existingManifest.version } else { '0.0.0' }

    # ---------------------------------------------------------------- staging
    Step ("staging: " + $stagingRel)
    $stagingDir = Join-Path $script:Repo ($stagingRel -replace '/', '\')
    if (Test-Path -LiteralPath $stagingDir) { Remove-Item -LiteralPath $stagingDir -Recurse -Force }
    New-Item -ItemType Directory -Force -Path $stagingDir | Out-Null
    $sourceDocs = @(
        @{ src = (Join-Path $script:Repo 'README.md'); name = 'README.md' },
        @{ src = (Join-Path $script:Repo 'tools/runbook.md'); name = 'RUNBOOK.md' },
        @{ src = (Join-Path $script:Repo 'docs/RELEASE.md'); name = 'RELEASE.md' }
    )
    foreach ($d in $sourceDocs) {
        if (-not (Test-Path -LiteralPath $d.src)) { Fail "E_RELEASE_STAGE_MISSING" ("staging source missing: " + (Rel $d.src)) 2 }
        Copy-Item -LiteralPath $d.src -Destination (Join-Path $stagingDir $d.name) -Force
    }
    foreach ($a in $builtArtifacts) {
        Copy-Item -LiteralPath (Join-Path $script:Repo ($a.path -replace '/', '\')) -Destination (Join-Path $stagingDir $a.name) -Force
    }
    $staged = @()
    foreach ($n in @('README.md', 'RUNBOOK.md', 'RELEASE.md', 'hiverouter.exe', 'hiverouter-cli.exe')) {
        $p = Join-Path $stagingDir $n
        $staged += [pscustomobject]@{ name = $n; path = (Rel $p); sha256 = (Sha256 $p); bytes = (Get-Item -LiteralPath $p).Length }
    }
    $checksumLines = @()
    foreach ($s in ($staged | Sort-Object { $_.name })) { $checksumLines += (($s.sha256 + '  ' + $s.name)) }
    $checksumsPath = Join-Path $stagingDir 'CHECKSUMS.txt'
    Write-Utf8NoBom $checksumsPath (([string]::Join("`n", $checksumLines)) + "`n")
    $staged += [pscustomobject]@{ name = 'CHECKSUMS.txt'; path = (Rel $checksumsPath); sha256 = (Sha256 $checksumsPath); bytes = (Get-Item -LiteralPath $checksumsPath).Length }

    # ---------------------------------------------------------------- zip + sidecar
    Step ("zip: " + $zipRel)
    $zipPath = Join-Path $script:Repo ($zipRel -replace '/', '\')
    if (Test-Path -LiteralPath $zipPath) { Remove-Item -LiteralPath $zipPath -Force }
    $zipInputs = @()
    foreach ($s in $staged) { $zipInputs += [pscustomobject]@{ name = $s.name; fullpath = (Join-Path $stagingDir $s.name) } }
    New-DeterministicZip $zipPath $zipInputs
    $zipSha = Sha256 $zipPath
    $zipBytes = (Get-Item -LiteralPath $zipPath).Length
    Write-Utf8NoBom (Join-Path $script:Repo ($sidecarRel -replace '/', '\')) ($zipSha + "`n")

    # ---------------------------------------------------------------- verify
    Step "verify: extract zip and compare every hash"
    $verifyDir = Join-Path $env:TEMP ('hivemind-release-verify-' + $PID)
    if (Test-Path -LiteralPath $verifyDir) { Remove-Item -LiteralPath $verifyDir -Recurse -Force }
    try {
        New-Item -ItemType Directory -Force -Path $verifyDir | Out-Null
        $vstream = [System.IO.File]::OpenRead($zipPath)
        try {
            $vzip = New-Object System.IO.Compression.ZipArchive($vstream, [System.IO.Compression.ZipArchiveMode]::Read, $false)
            try {
                foreach ($ve in $vzip.Entries) {
                    $target = Join-Path $verifyDir $ve.FullName
                    $vsrc = $ve.Open()
                    try {
                        $vdst = [System.IO.File]::Open($target, [System.IO.FileMode]::Create, [System.IO.FileAccess]::Write)
                        try { $vsrc.CopyTo($vdst) } finally { $vdst.Dispose() }
                    }
                    finally { $vsrc.Dispose() }
                }
            }
            finally { $vzip.Dispose() }
        }
        finally { $vstream.Dispose() }
        $memberNames = @(Get-ChildItem -LiteralPath $verifyDir -File | ForEach-Object { $_.Name } | Sort-Object)
        $expectedNames = @($staged | ForEach-Object { $_.name } | Sort-Object)
        if (($memberNames -join ',') -ne ($expectedNames -join ',')) {
            Fail "E_RELEASE_VERIFY_FAILED" ("zip member set mismatch: " + ($memberNames -join ', ') + " != " + ($expectedNames -join ', ')) 6
        }
        foreach ($s in $staged) {
            $vp = Join-Path $verifyDir $s.name
            if (-not (Test-Path -LiteralPath $vp)) { Fail "E_RELEASE_VERIFY_FAILED" ("zip missing member " + $s.name) 6 }
            $vh = Sha256 $vp
            if ($vh -ne $s.sha256) { Fail "E_RELEASE_VERIFY_FAILED" ("zip member hash mismatch for " + $s.name + ": " + $vh + " != " + $s.sha256) 6 }
        }
        $checksumEntries = @(Get-Content -LiteralPath (Join-Path $verifyDir 'CHECKSUMS.txt'))
        foreach ($line in $checksumEntries) {
            if ($line.Trim()) {
                $parts = $line.Trim() -split '\s+'
                if ($parts.Count -lt 2) { Fail "E_RELEASE_VERIFY_FAILED" ("malformed CHECKSUMS line: " + $line) 6 }
                $expectedHash = $parts[0]; $member = $parts[1]
                $s = $staged | Where-Object { $_.name -eq $member } | Select-Object -First 1
                if (-not $s) { Fail "E_RELEASE_VERIFY_FAILED" ("CHECKSUMS references unknown member " + $member) 6 }
                if ($s.sha256 -ne $expectedHash) { Fail "E_RELEASE_VERIFY_FAILED" ("CHECKSUMS hash mismatch for " + $member) 6 }
            }
        }
        Write-Host ("verified " + $staged.Count + " zip members + CHECKSUMS (" + $zipPath + ")")
    }
    finally {
        if (-not $KeepVerifyDir -and (Test-Path -LiteralPath $verifyDir)) { Remove-Item -LiteralPath $verifyDir -Recurse -Force }
    }

    # ---------------------------------------------------------------- manifest
    Step "manifest: tools/release-manifest.json"
    $supersedes = @()
    if ($existingManifest -and $existingManifest.artifacts -and $effectiveVersion -ne $manifestVersion) {
        foreach ($a in $existingManifest.artifacts) {
            $supersedes += [pscustomobject]@{ path = $a.path; sha256 = $a.sha256; bytes = $a.bytes; note = 'SUPERSEDED by ' + $effectiveVersion }
        }
    }
    $manifest = [ordered]@{
        schema = 'hivemind.release-manifest.v2'
        product = 'HiveMind'
        version = $effectiveVersion
        tag = $Tag
        tag_target = $tagTarget
        sealed = $false
        seal_allowlist = @($script:SealAllowlist)
        seal_diff = $null
        tag_behind_commits = $behind
        source_commit = $head
        source_commit_date = $commitDate
        source_dirty = $dirty
        source_describe = $describe
        generated_at = (Get-Date).ToString('yyyy-MM-ddTHH:mm:sszzz')
        toolchain = [ordered]@{
            rustc = $rustcVersion
            cargo = $cargoVersion
            node = $nodeVersion
            npm = $npmVersion
            gcc = $gccVersion
        }
        toolchain_line = $rustcVersion
        provider = $provider
        web = [ordered]@{
            dir = 'web/dist'
            algorithm = 'sha256 per file; tree = sha256 over sorted "relpath<TAB>sha256<TAB>bytes" lines'
            tree_sha256 = $webTree
            files = $webFiles
        }
        web_dist_algorithm = 'sha256 per file; path+sha256+bytes array'
        web_dist_files = $webFiles
        staging_dir = $stagingRel
        artifacts = @($builtArtifacts)
        package = [ordered]@{
            zip = $zipRel
            zip_sha256 = $zipSha
            zip_bytes = $zipBytes
            sidecar = $sidecarRel
            sidecar_sha256 = (Sha256 (Join-Path $script:Repo ($sidecarRel -replace '/', '\')))
            checksums = ($stagingRel + '/CHECKSUMS.txt')
            checksums_sha256 = (Sha256 $checksumsPath)
            contents = @($staged | ForEach-Object { $_.name } | Sort-Object)
        }
        supersedes = $supersedes
        old_to_new = $oldToNew
        note = 'Paths are repo-relative. Built by tools/build-release.ps1; verified by zip extract-and-hash. No tag was moved by this script.'
    }
    $manifestJson = ($manifest | ConvertTo-Json -Depth 10)
    Write-Utf8NoBom $manifestPath ($manifestJson + "`n")

    # ---------------------------------------------------------------- policy scan + summary
    Step "policy: absolute-path scan"
    Test-AbsolutePaths (Get-Content -LiteralPath $manifestPath -Raw) 'tools/release-manifest.json'
    Test-AbsolutePaths (Get-Content -LiteralPath $checksumsPath -Raw) ($stagingRel + '/CHECKSUMS.txt')
    $stagedDocHostPaths = 0
    foreach ($docName in @('README.md', 'RUNBOOK.md', 'RELEASE.md')) {
        $docText = Get-Content -LiteralPath (Join-Path $stagingDir $docName) -Raw
        if ($docText -match '[A-Za-z]:\\Users\\') { $stagedDocHostPaths = $stagedDocHostPaths + 1 }
    }
    if ($stagedDocHostPaths -gt 0) {
        Write-Host ("WARN: " + $stagedDocHostPaths + " staged doc(s) contain a host path; docs are read-only for this seat.") -ForegroundColor Yellow
    }
    Write-Host "no absolute paths in manifest/CHECKSUMS"

    Step "DONE - release build complete (manifest sealed=false; -RecordSeal at D records the seal diff)"
    Write-Host ("version:  " + $effectiveVersion + "  tag: " + $Tag + " (behind=" + $behind + ")")
    Write-Host ("source:   " + $head)
    Write-Host ("web tree: " + $webTree)
    foreach ($a in $staged) { Write-Host ("  " + $a.name + "  " + $a.sha256 + "  " + $a.bytes + " B") }
    Write-Host ("zip:      " + $zipRel + "  " + $zipSha + "  " + $zipBytes + " B")
    Write-Host ("manifest: tools/release-manifest.json (schema hivemind.release-manifest.v2; sealed=false until -RecordSeal)")
    exit 0
}
finally {
    Pop-Location
}




