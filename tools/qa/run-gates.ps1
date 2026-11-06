<#
.SYNOPSIS
  HiveMind one-command gate harness (no paid calls by default).

.DESCRIPTION
  Runs the release gates in order and fails closed:
    G0 preflight (clean tree, version truth, tag/manifest freshness, toolchain)
    G1 docs-truth checker          node tools/qa/check-docs-truth.mjs
    G2 cargo workspace tests       cargo test --workspace --locked
    G3 web typecheck / tests / build
    G4 guarded Playwright e2e      e2e/run-guarded.mjs (SKIPPED by name when no app is ready)
    G5 static QA harness           node tools/qa/verify-harness.mjs
    G6 strict key scan             sk-[A-Za-z0-9]{32,} over git-tracked text files
    G7 manifest + artifact hashes  tools/release-manifest.json vs disk/zip/web dist

  Flags:
    -DryRun     resolve + print the plan and preflight only; execute nothing, write nothing.
    -Rehearsal  allow a dirty tree / version-mismatched manifest (printed as REHEARSAL; real defects still fail).
    -Live       explicit paid-provider opt-in: requires E2E_APP_READY=1, E2E_BASE_URL and a present
                DEEPSEEK_API_KEY (never printed); sets LIVE_ONE_PROMPT=1 for the guarded runner.
                Without -Live, LIVE_ONE_PROMPT is forced to 0 so paid specs skip by name.
    -SkipE2E    skip the guarded e2e step by name (recorded SKIPPED, not PASS).
    -RequireE2E the guarded e2e gate must execute and PASS; a skip (-SkipE2E, missing app/server env) is a FAIL.
    -Release    release mode: zero skipped gates. On a sealed tree the manifest must describe exactly HEAD
                (manifest.source_commit == HEAD). A documented pre-release tree (current version != manifest
                version, e.g. 1.2.0-dev while the manifest/tag still record 1.1.0) is accepted only when the
                manifest tag target equals its source_commit, that source is an ancestor of HEAD, and
                docs/RELEASE-EVIDENCE.md + plans/waves/README.md document the pre-release line; the run then
                reports PRE-RELEASE and exits 3 - never a release PASS (no false green).

  Exit codes: 0 = all gates pass (named skips are reported, never counted as PASS; zero skips in -Release mode),
  1 = one or more gates failed or a release-mode skip was recorded, 2 = setup/file error,
  3 = -Release (or its -DryRun plan) ran on a documented pre-release tree (gates not necessarily executed; this is not a
      release certification).
  No paid provider call is made by this script itself; it never prints the key or absolute workspace paths.
  It writes no tracked files (cargo/web/e2e write only ignored build/test dirs).
#>
[CmdletBinding()]
param(
    [switch]$DryRun,
    [switch]$Rehearsal,
    [switch]$Live,
    [switch]$SkipE2E,
    [switch]$RequireE2E,
    [switch]$Release
)

$ErrorActionPreference = 'Stop'
# Windows PowerShell 5.1 does not auto-load these BCL assemblies.
try { Add-Type -AssemblyName System.IO.Compression -ErrorAction Stop } catch { }
try { Add-Type -AssemblyName System.IO.Compression.FileSystem -ErrorAction Stop } catch { }

$script:Repo = (Resolve-Path (Join-Path $PSScriptRoot '..\..')).Path
$script:Steps = New-Object System.Collections.ArrayList
$script:SkipReason = $null

function Rel([string]$Path) {
    $full = [System.IO.Path]::GetFullPath($Path)
    $base = [System.IO.Path]::GetFullPath($script:Repo)
    if ($full.StartsWith($base, [System.StringComparison]::OrdinalIgnoreCase)) {
        return (($full.Substring($base.Length).TrimStart('\', '/')) -replace '\\', '/')
    }
    return ($full -replace '\\', '/')
}

function Sha256([string]$Path) {
    return (Get-FileHash -Algorithm SHA256 -LiteralPath $Path).Hash.ToLowerInvariant()
}

function Setup-Fail([string]$Code, [string]$Message) {
    Write-Host ""
    Write-Host ("[{0}] {1}" -f $Code, $Message) -ForegroundColor Red
    Write-Host "GATES: SETUP ERROR - no gate was claimed as passed." -ForegroundColor Red
    exit 2
}

function Invoke-Native([string]$Exe, [string[]]$Arguments, [string]$WorkingDirectory, [string]$Label) {
    Write-Host ("`$ " + $Label) -ForegroundColor DarkGray
    Push-Location $WorkingDirectory
    try {
        & $Exe @Arguments
        $code = $LASTEXITCODE
    }
    finally { Pop-Location }
    if ($null -ne $code -and $code -ne 0) { throw ("$Label exited $code") }
}

function Add-Step([string]$Name, [string]$Status, [int]$ExitCode, [double]$Seconds, [string]$Detail) {
    [void]$script:Steps.Add([pscustomobject]@{
        name = $Name; status = $Status; exit = $ExitCode; seconds = $Seconds; detail = $Detail
    })
    $color = 'Green'
    if ($Status -eq 'FAIL') { $color = 'Red' }
    elseif ($Status -eq 'SKIPPED') { $color = 'Yellow' }
    elseif ($Status -eq 'REHEARSAL') { $color = 'Yellow' }
    Write-Host ("STEP {0}  exit={1}  {2:0.0}s  {3}" -f $Status, $ExitCode, $Seconds, $Name) -ForegroundColor $color
}

function Invoke-Gate([string]$Name, [scriptblock]$Body) {
    $sw = [System.Diagnostics.Stopwatch]::StartNew()
    Write-Host ""
    Write-Host ("=== " + $Name + " ===") -ForegroundColor Cyan
    $status = 'PASS'
    $code = 0
    $detail = ''
    try {
        & $Body
        if ($null -ne $LASTEXITCODE) { $code = [int]$LASTEXITCODE }
    }
    catch {
        $status = 'FAIL'
        $code = 1
        $detail = $_.Exception.Message
        Write-Host ("GATE FAILED: " + $Name + " :: " + $detail) -ForegroundColor Red
    }
    $sw.Stop()
    Add-Step $Name $status $code $sw.Elapsed.TotalSeconds $detail
    return $status
}

function Add-Skip([string]$Name, [string]$Reason) {
    Add-Step $Name 'SKIPPED' 0 0.0 $Reason
}

function Add-E2E-Skip([string]$Reason) {
    if ($RequireE2E) {
        Add-Step 'G4 guarded e2e' 'FAIL' 1 0.0 ('-RequireE2E: ' + $Reason + ' (a skip cannot satisfy the release gate)')
    }
    else {
        Add-Skip 'G4 guarded e2e' ($Reason + ' - recorded SKIPPED, never PASS')
    }
}

# ------------------------------------------------------------------ paths / toolchain
$backend = Join-Path $script:Repo 'backend'
$web = Join-Path $script:Repo 'web'
$e2e = Join-Path $script:Repo 'e2e'
$docsChecker = Join-Path $script:Repo 'tools/qa/check-docs-truth.mjs'
$harness = Join-Path $script:Repo 'tools/qa/verify-harness.mjs'
$manifestPath = Join-Path $script:Repo 'tools/release-manifest.json'
$cargoBin = Join-Path $env:USERPROFILE '.cargo\bin'
$gccBin = Join-Path $script:Repo 'tools/w64devkit/bin'

function Resolve-Exe([string[]]$Candidates, [string]$Name) {
    foreach ($c in $Candidates) {
        if ($c -and (Test-Path -LiteralPath $c)) { return $c }
    }
    $cmd = Get-Command $Name -ErrorAction SilentlyContinue
    if ($cmd) { return $cmd.Source }
    return $null
}

$cargo = Resolve-Exe @((Join-Path $cargoBin 'cargo.exe')) 'cargo.exe'
$rustc = Resolve-Exe @((Join-Path $cargoBin 'rustc.exe')) 'rustc.exe'
$node = Resolve-Exe @() 'node.exe'
$npm = $null
if ($node) {
    $npm = Resolve-Exe @((Join-Path (Split-Path -Parent ([string]$node)) 'npm.cmd')) 'npm.cmd'
}

# ------------------------------------------------------------------ G0 preflight
$head = (& git -C $script:Repo rev-parse HEAD).Trim()
$describe = (& git -C $script:Repo describe --tags --always).Trim()
$dirty = @(& git -C $script:Repo status --porcelain)
$webPkg = Get-Content -LiteralPath (Join-Path $web 'package.json') -Raw | ConvertFrom-Json
$version = [string]$webPkg.version
$crateVersions = @()
foreach ($dir in (Get-ChildItem -LiteralPath (Join-Path $backend 'crates') -Directory)) {
    $toml = Join-Path $dir.FullName 'Cargo.toml'
    if (Test-Path -LiteralPath $toml) {
        $m = [regex]::Match((Get-Content -LiteralPath $toml -Raw), '(?m)^\s*version\s*=\s*"([^"]+)"')
        if ($m.Success) { $crateVersions += [pscustomobject]@{ crate = $dir.Name; version = $m.Groups[1].Value } }
    }
}
if (-not $cargo) { Setup-Fail 'E_GATE_TOOLCHAIN' 'cargo.exe not found (looked in %USERPROFILE%\.cargo\bin and PATH)' }
if (-not $node) { Setup-Fail 'E_GATE_TOOLCHAIN' 'node.exe not found on PATH' }
if (-not $npm) { Setup-Fail 'E_GATE_TOOLCHAIN' 'npm.cmd not found next to node.exe' }
if (-not (Test-Path -LiteralPath $docsChecker)) { Setup-Fail 'E_GATE_MISSING_FILE' 'tools/qa/check-docs-truth.mjs missing' }
if (-not (Test-Path -LiteralPath $harness)) { Setup-Fail 'E_GATE_MISSING_FILE' 'tools/qa/verify-harness.mjs missing' }

$cargoVersion = (& $cargo --version).Trim()
$rustcVersion = (& $rustc --version).Trim()
$nodeVersion = (& $node --version).Trim()
$npmVersion = (& cmd.exe /c "npm.cmd --version").Trim()

$manifest = $null
$manifestError = $null
if (Test-Path -LiteralPath $manifestPath) {
    try { $manifest = Get-Content -LiteralPath $manifestPath -Raw | ConvertFrom-Json }
    catch { $manifestError = $_.Exception.Message }
}

$preReleaseCandidate = $false
if ($manifest -and [string]$manifest.version) {
    $preReleaseCandidate = [bool](($version -match '-') -and ([string]$manifest.version -ne $version))
}

$preflightProblems = @()
$preflightWarnings = @()
if ($dirty.Count -gt 0) {
    $msg = ("working tree is not clean (" + $dirty.Count + " path(s))")
    if ($Rehearsal) { $preflightWarnings += ($msg + ' - allowed by -Rehearsal') }
    else { $preflightProblems += ("E_GATE_DIRTY_TREE: " + $msg + '; commit/stash or pass -Rehearsal') }
}
$badCrateVersions = @($crateVersions | Where-Object { $_.version -ne $version })
if ($crateVersions.Count -eq 0) { $preflightProblems += 'E_GATE_VERSION: no backend crate versions parsed' }
if ($badCrateVersions.Count -gt 0) {
    $preflightProblems += ('E_GATE_VERSION: web version ' + $version + ' != ' + (($badCrateVersions | ForEach-Object { $_.crate + ':' + $_.version }) -join ', '))
}
if (-not $manifest) {
    $msg = 'tools/release-manifest.json missing or unparsable'
    if ($manifestError) { $msg += (' (' + $manifestError + ')') }
    if ($Rehearsal) { $preflightWarnings += ($msg + ' - allowed by -Rehearsal') }
    else { $preflightProblems += ('E_GATE_MANIFEST_STALE: ' + $msg) }
}
else {
    if ([string]$manifest.version -ne $version) {
        $msg = ('manifest version ' + [string]$manifest.version + ' != current version ' + $version)
        if ($preReleaseCandidate) {
            $preflightWarnings += ('PRE-RELEASE CANDIDATE: ' + $msg + ' - manifest remains the sealed release record; -Release requires documented tag/source')
        }
        elseif ($Rehearsal) { $preflightWarnings += ($msg + ' - allowed by -Rehearsal') }
        else { $preflightProblems += ('E_GATE_MANIFEST_STALE: ' + $msg) }
    }
    if (-not $manifest.tag) { $preflightProblems += 'E_GATE_MANIFEST_STALE: manifest has no tag' }
    else {
        $tagTarget = (& git -C $script:Repo rev-list -n1 ([string]$manifest.tag) 2>$null)
        if ($LASTEXITCODE -ne 0 -or -not $tagTarget) {
            $msg = ('manifest tag ' + [string]$manifest.tag + ' does not exist')
            if ($Rehearsal) { $preflightWarnings += ($msg + ' - allowed by -Rehearsal') }
            else { $preflightProblems += ('E_GATE_MANIFEST_STALE: ' + $msg) }
        }
        elseif ([string]$manifest.source_commit -and $tagTarget.Trim() -ne [string]$manifest.source_commit) {
            $msg = ('manifest source_commit ' + ([string]$manifest.source_commit).Substring(0, 8) + ' != tag target ' + $tagTarget.Trim().Substring(0, 8))
            if ($Rehearsal) { $preflightWarnings += ($msg + ' - allowed by -Rehearsal') }
            else { $preflightProblems += ('E_GATE_MANIFEST_STALE: ' + $msg) }
        }
    }
    $behind = 0
    if ($manifest.source_commit) {
        & git -C $script:Repo merge-base --is-ancestor ([string]$manifest.source_commit) HEAD 2>$null
        if ($LASTEXITCODE -eq 0) { $behind = [int]((& git -C $script:Repo rev-list --count ([string]$manifest.source_commit + '..HEAD')).Trim()) }
        else { $preflightProblems += 'E_GATE_MANIFEST_STALE: manifest source_commit is not an ancestor of HEAD' }
    }
    if ($behind -gt 0) { $preflightWarnings += ("HEAD is $behind commit(s) ahead of the sealed source (post-release commits)") }
}

# Release mode: zero skipped gates. Sealed trees must match HEAD exactly; a documented pre-release tree is
# accepted with an explicit PRE-RELEASE result (exit 3) and can never produce a green release PASS.
$script:PreReleaseAccepted = $false
if ($Release) {
    if ($Rehearsal) { $preflightProblems += 'E_GATE_RELEASE: -Release cannot be combined with -Rehearsal' }
    if (-not $manifest) {
        $preflightProblems += 'E_GATE_RELEASE: a parsed tools/release-manifest.json is required for release mode'
    }
    elseif (-not [string]$manifest.source_commit) {
        $preflightProblems += 'E_GATE_RELEASE: manifest has no source_commit'
    }
    elseif (-not $preReleaseCandidate) {
        if ([string]$manifest.source_commit.Trim() -ne $head) {
            $preflightProblems += ('E_GATE_RELEASE: manifest source_commit ' + ([string]$manifest.source_commit).Substring(0, 8) + ' != HEAD ' + $head.Substring(0, 8) + ' (rebuild/refresh manifest or do not release)')
        }
    }
    else {
        # Documented pre-release tree: manifest must still be the sealed 1.x record; tag target == source_commit;
        # sealed source must be an ancestor of HEAD; docs must state the pre-release line and sealed record.
        $manifestSource = ([string]$manifest.source_commit).Trim()
        $manifestTag = [string]$manifest.tag
        if (-not $manifestTag) { $preflightProblems += 'E_GATE_PRERELEASE: manifest has no sealed tag' }
        else {
            $tagTargetNow = (& git -C $script:Repo rev-list -n1 $manifestTag 2>$null)
            if ($LASTEXITCODE -ne 0 -or -not $tagTargetNow) { $preflightProblems += ('E_GATE_PRERELEASE: sealed tag ' + $manifestTag + ' does not exist') }
            elseif ($tagTargetNow.Trim() -ne $manifestSource) {
                $preflightProblems += ('E_GATE_PRERELEASE: tag ' + $manifestTag + ' target ' + $tagTargetNow.Trim().Substring(0, 8) + ' != manifest source_commit ' + $manifestSource.Substring(0, 8))
            }
        }
        if ($manifestSource -eq $head) {
            $preflightProblems += ('E_GATE_PRERELEASE: current version ' + $version + ' != manifest ' + [string]$manifest.version + ' but HEAD equals the manifest source_commit (inconsistent tree)')
        }
        else {
            & git -C $script:Repo merge-base --is-ancestor $manifestSource HEAD 2>$null
            if ($LASTEXITCODE -ne 0) { $preflightProblems += 'E_GATE_PRERELEASE: sealed source_commit is not an ancestor of HEAD' }
        }
        $evidencePath = Join-Path $script:Repo 'docs/RELEASE-EVIDENCE.md'
        if (-not (Test-Path -LiteralPath $evidencePath)) {
            $preflightProblems += 'E_GATE_PRERELEASE: docs/RELEASE-EVIDENCE.md missing'
        }
        else {
            $evidence = Get-Content -LiteralPath $evidencePath -Raw
            if (-not $evidence.Contains($version)) { $preflightProblems += ('E_GATE_PRERELEASE: RELEASE-EVIDENCE does not document current source version ' + $version) }
            if (-not $evidence.Contains([string]$manifest.version)) { $preflightProblems += ('E_GATE_PRERELEASE: RELEASE-EVIDENCE does not document sealed version ' + [string]$manifest.version) }
            if (-not $evidence.Contains($manifestTag)) { $preflightProblems += ('E_GATE_PRERELEASE: RELEASE-EVIDENCE does not document sealed tag ' + $manifestTag) }
            if (-not $evidence.Contains($manifestSource.Substring(0, [Math]::Min(8, $manifestSource.Length)))) { $preflightProblems += 'E_GATE_PRERELEASE: RELEASE-EVIDENCE does not document sealed source commit' }
            if (-not ($evidence -match 'pre[- ]release')) { $preflightProblems += 'E_GATE_PRERELEASE: RELEASE-EVIDENCE does not mark the tree pre-release' }
        }
        $waveIndexPath = Join-Path $script:Repo 'plans/waves/README.md'
        if (-not (Test-Path -LiteralPath $waveIndexPath)) {
            $preflightProblems += 'E_GATE_PRERELEASE: plans/waves/README.md missing'
        }
        else {
            $waveIndex = Get-Content -LiteralPath $waveIndexPath -Raw
            if (-not $waveIndex.Contains($version)) { $preflightProblems += ('E_GATE_PRERELEASE: wave index does not document ' + $version) }
            if (-not ($waveIndex -match 'pre[- ]release')) { $preflightProblems += 'E_GATE_PRERELEASE: wave index does not mark the tree pre-release' }
        }
        $preReleaseProblems = @($preflightProblems | Where-Object { $_ -like 'E_GATE_PRERELEASE:*' })
        if ($preReleaseProblems.Count -eq 0 -and -not $Rehearsal) {
            $script:PreReleaseAccepted = $true
            $preflightWarnings += ('PRE-RELEASE TREE ACCEPTED: working tree ' + $version + ' is pre-release; sealed ' + [string]$manifest.version + ' (' + $manifestTag + ' @ ' + $manifestSource.Substring(0, 8) + ') remains the release record; -Release exits 3, never PASS')
        }
    }
}
# -RequireE2E: the guarded e2e gate cannot be skipped or silently absent.
if ($RequireE2E) {
    if ($SkipE2E) { $preflightProblems += 'E_GATE_E2E_REQUIRED: -RequireE2E conflicts with -SkipE2E' }
    if ($env:E2E_APP_READY -ne '1') { $preflightProblems += 'E_GATE_E2E_REQUIRED: -RequireE2E requires a running app (E2E_APP_READY=1)' }
    if (-not $env:E2E_BASE_URL) { $preflightProblems += 'E_GATE_E2E_REQUIRED: -RequireE2E requires E2E_BASE_URL' }
}

Write-Host ""
Write-Host "HiveMind gates - one command, no paid calls by default" -ForegroundColor Cyan
Write-Host ("HEAD       " + $head + " (" + $describe + ")")
Write-Host ("version    " + $version + "  crates: " + (($crateVersions | ForEach-Object { $_.crate + ':' + $_.version }) -join ', '))
Write-Host ("tree       " + $(if ($dirty.Count -eq 0) { 'clean' } else { 'dirty (' + $dirty.Count + ' path(s))' }))
Write-Host ("manifest   " + $(if ($manifest) { 'version ' + [string]$manifest.version + ' tag ' + [string]$manifest.tag } else { 'MISSING/UNREADABLE' }))
Write-Host ("toolchain  " + $cargoVersion + ' | ' + $rustcVersion + ' | node ' + $nodeVersion + ' | npm ' + $npmVersion)
Write-Host ("mode       " + $(if ($Rehearsal) { 'REHEARSAL' } else { 'STRICT' }) + $(if ($Live) { ' + LIVE(paid opt-in)' } else { ' + no-paid' }) + $(if ($RequireE2E) { ' + REQUIRE-E2E' } else { '' }) + $(if ($Release) { ' + RELEASE' } else { '' }) + $(if ($script:PreReleaseAccepted) { ' + PRE-RELEASE(accepted)' } else { '' }) + $(if ($DryRun) { ' + DRY-RUN' } else { '' }))
foreach ($w in $preflightWarnings) { Write-Host ("WARN  " + $w) -ForegroundColor Yellow }
foreach ($p in $preflightProblems) { Write-Host ("FAIL  " + $p) -ForegroundColor Red }

if ($DryRun) {
    Write-Host ""
    Write-Host "DRY RUN plan (nothing executed, nothing written):" -ForegroundColor Cyan
    Write-Host "  G1 docs-truth        node tools/qa/check-docs-truth.mjs"
    Write-Host "  G2 cargo tests       cargo test --workspace --locked   (cwd backend/)"
    Write-Host "  G3a web typecheck    npm.cmd run typecheck             (cwd web/)"
    Write-Host "  G3b web tests        npm.cmd test -- --run             (cwd web/)"
    Write-Host "  G3c web build        npm.cmd run build                 (cwd web/)"
    if ($SkipE2E) { Write-Host "  G4  guarded e2e      SKIPPED (-SkipE2E)" }
    elseif (-not $Live) { Write-Host "  G4  guarded e2e      node run-guarded.mjs if E2E_APP_READY=1 + E2E_BASE_URL; else SKIPPED (no paid calls; LIVE_ONE_PROMPT=0)" }
    else { Write-Host "  G4  guarded e2e      node run-guarded.mjs with LIVE_ONE_PROMPT=1 (PAID opt-in; needs app + key)" }
    Write-Host "  G5 static harness    node tools/qa/verify-harness.mjs"
    Write-Host "  G6 key scan          sk-[A-Za-z0-9]{32,} over git-tracked text files"
    Write-Host "  G7 manifest/artifacts tools/release-manifest.json vs target/release, dist zip/sidecar/checksums, web dist"
    Write-Host ""
    if ($preflightProblems.Count -gt 0) {
        Write-Host "DRY RUN: FAIL (preflight would refuse)" -ForegroundColor Red
        exit 1
    }
    if ($script:PreReleaseAccepted) {
        Write-Host 'DRY RUN: PRE-RELEASE (documented pre-release tree; gates not executed; not a release PASS)' -ForegroundColor Yellow
        exit 3
    }
    Write-Host "DRY RUN: OK (preflight satisfied; no gate executed)" -ForegroundColor Green
    exit 0
}

if ($preflightProblems.Count -gt 0) {
    Write-Host ""
    Write-Host "GATES: FAIL (preflight refused; no gate executed)" -ForegroundColor Red
    exit 1
}

# paid-gate environment: never let a paid spec run without -Live
if ($Live) {
    $keyPresent = [bool]($env:DEEPSEEK_API_KEY -and $env:DEEPSEEK_API_KEY.Length -gt 0)
    if ($env:E2E_APP_READY -ne '1' -or -not $env:E2E_BASE_URL) {
        Setup-Fail 'E_GATE_LIVE_SETUP' '-Live requires E2E_APP_READY=1 and E2E_BASE_URL for a running app (browser tests never start the app)'
    }
    if (-not $keyPresent) { Setup-Fail 'E_GATE_LIVE_SETUP' '-Live requires DEEPSEEK_API_KEY to be present (value never printed)' }
    $env:LIVE_ONE_PROMPT = '1'
    Write-Host "LIVE MODE: paid provider specs authorized (LIVE_ONE_PROMPT=1); one short prompt per paid gate." -ForegroundColor Yellow
}
else {
    $env:LIVE_ONE_PROMPT = '0'
}

$env:PATH = "$cargoBin;$gccBin;$env:PATH"
$env:CARGO_PROFILE_DEV_DEBUG = '0'
$env:CARGO_INCREMENTAL = '0'
$env:PLAYWRIGHT_SKIP_BROWSER_DOWNLOAD = '1'

# ------------------------------------------------------------------ G1 docs truth
Invoke-Gate 'G1 docs-truth checker' {
    Invoke-Native $node @((Rel $docsChecker)) $script:Repo 'node tools/qa/check-docs-truth.mjs'
} | Out-Null

# ------------------------------------------------------------------ G2 cargo workspace
Invoke-Gate 'G2 cargo workspace tests (--locked)' {
    $prev = $env:PATH
    try {
        $env:PATH = "$cargoBin;$gccBin;$prev"
        Invoke-Native $cargo @('test', '--workspace', '--locked') $backend 'cargo test --workspace --locked'
    }
    finally { $env:PATH = $prev }
} | Out-Null

# ------------------------------------------------------------------ G3 web
Invoke-Gate 'G3a web typecheck' {
    Invoke-Native 'cmd.exe' @('/c', 'npm.cmd run typecheck') $web 'npm.cmd run typecheck'
} | Out-Null
Invoke-Gate 'G3b web unit tests' {
    Invoke-Native 'cmd.exe' @('/c', 'npm.cmd test -- --run') $web 'npm.cmd test -- --run'
} | Out-Null
Invoke-Gate 'G3c web production build' {
    Invoke-Native 'cmd.exe' @('/c', 'npm.cmd run build') $web 'npm.cmd run build'
} | Out-Null

# ------------------------------------------------------------------ G4 guarded e2e
if ($SkipE2E) {
    Add-E2E-Skip '-SkipE2E supplied'
}
elseif ($env:E2E_APP_READY -ne '1') {
    Add-E2E-Skip 'E2E_APP_READY!=1 (no app server); guarded suite not run'
}
elseif (-not $env:E2E_BASE_URL) {
    Add-E2E-Skip 'E2E_BASE_URL not set; guarded suite not run'
}
else {
    Invoke-Gate 'G4 guarded e2e (Playwright, installed browser only)' {
        Invoke-Native $node @('run-guarded.mjs') $e2e 'node e2e/run-guarded.mjs'
    } | Out-Null
}

# ------------------------------------------------------------------ G5 static harness
Invoke-Gate 'G5 static QA harness' {
    Invoke-Native $node @((Rel $harness)) $script:Repo 'node tools/qa/verify-harness.mjs'
} | Out-Null

# ------------------------------------------------------------------ G6 strict key scan
Invoke-Gate 'G6 strict tracked-tree key scan' {
    $tracked = @(& git -C $script:Repo ls-files)
    if ($LASTEXITCODE -ne 0) { throw 'git ls-files failed' }
    $binaryExt = @('.exe', '.dll', '.pdb', '.zip', '.png', '.jpg', '.jpeg', '.gif', '.ico', '.webp', '.woff', '.woff2', '.ttf', '.otf', '.eot', '.pdf', '.mp4', '.wasm', '.rlib', '.d')
    $strict = [regex]'sk-[A-Za-z0-9]{32,}'
    $near = [regex]'sk-[A-Za-z0-9]{16,31}'
    $hits = @()
    $nearHits = @()
    $scanned = 0
    foreach ($rel in $tracked) {
        if (-not $rel) { continue }
        $ext = [System.IO.Path]::GetExtension($rel).ToLowerInvariant()
        if ($binaryExt -contains $ext) { continue }
        $full = Join-Path $script:Repo ($rel -replace '/', '\')
        if (-not (Test-Path -LiteralPath $full)) { continue }
        try { $bytes = [System.IO.File]::ReadAllBytes($full) } catch { continue }
        $isBinary = $false
        foreach ($b in $bytes) { if ($b -eq 0) { $isBinary = $true; break } }
        if ($isBinary) { continue }
        $text = [System.Text.Encoding]::UTF8.GetString($bytes)
        $scanned = $scanned + 1
        $lineNo = 0
        foreach ($line in ($text -split "`n")) {
            $lineNo = $lineNo + 1
            if ($strict.IsMatch($line)) { $hits += ($rel + ':' + $lineNo) }
            elseif ($near.IsMatch($line)) { $nearHits += ($rel + ':' + $lineNo) }
        }
    }
    Write-Host ("scanned " + $scanned + " tracked text file(s)")
    if ($nearHits.Count -gt 0) {
        Write-Host ("WARN near-miss sk- tokens (16-31 chars, not failing): " + (($nearHits | Select-Object -First 5) -join ', ')) -ForegroundColor Yellow
    }
    if ($hits.Count -gt 0) {
        throw ("strict sk-[A-Za-z0-9]{32,} credential pattern found at " + (($hits | Select-Object -First 10) -join ', '))
    }
} | Out-Null

# ------------------------------------------------------------------ G7 manifest + artifacts
Invoke-Gate 'G7 manifest / artifact hash verification' {
    if (-not $manifest) { throw 'tools/release-manifest.json missing or unparsable' }
    $problems = @()
    $preReleaseMode = [bool]$script:PreReleaseAccepted
    if ([string]$manifest.schema -ne 'hivemind.release-manifest.v1') { $problems += 'unexpected schema: ' + [string]$manifest.schema }
    if ([string]$manifest.version -ne $version) {
        if ($script:PreReleaseAccepted) {
            Write-Host ("pre-release tree: manifest version " + [string]$manifest.version + " is the sealed release record; current source is " + $version)
        }
        else { $problems += 'manifest version ' + [string]$manifest.version + ' != ' + $version }
    }
    $tagTarget = (& git -C $script:Repo rev-list -n1 ([string]$manifest.tag) 2>$null)
    if ($LASTEXITCODE -ne 0 -or -not $tagTarget) { $problems += ('tag ' + [string]$manifest.tag + ' missing') }
    elseif ([string]$manifest.source_commit -and $tagTarget.Trim() -ne [string]$manifest.source_commit) { $problems += 'tag target != manifest source_commit' }

    foreach ($artifact in @($manifest.artifacts)) {
        $p = Join-Path $script:Repo (([string]$artifact.path) -replace '/', '\')
        if (-not (Test-Path -LiteralPath $p)) {
            if ($preReleaseMode) { Write-Host ("pre-release drift (not a failure): artifact not built in this tree: " + [string]$artifact.path) -ForegroundColor Yellow }
            else { $problems += ('artifact missing: ' + [string]$artifact.path) }
            continue
        }
        $sha = Sha256 $p
        $bytes = (Get-Item -LiteralPath $p).Length
        if ($sha -ne [string]$artifact.sha256) {
            if ($preReleaseMode) { Write-Host ("pre-release drift (not a failure): artifact differs from sealed manifest: " + [string]$artifact.path) -ForegroundColor Yellow }
            else { $problems += ('artifact sha mismatch: ' + [string]$artifact.path) }
        }
        if ($bytes -ne [int]$artifact.bytes) {
            if ($preReleaseMode) { Write-Host ("pre-release drift (not a failure): artifact size differs from sealed manifest: " + [string]$artifact.path) -ForegroundColor Yellow }
            else { $problems += ('artifact size mismatch: ' + [string]$artifact.path) }
        }
    }

    if ($manifest.package) {
        $zipRel = [string]$manifest.package.zip
        $zipPath = Join-Path $script:Repo ($zipRel -replace '/', '\')
        if (-not (Test-Path -LiteralPath $zipPath)) { $problems += ('package zip missing: ' + $zipRel) }
        else {
            if ((Sha256 $zipPath) -ne [string]$manifest.package.zip_sha256) { $problems += ('package zip sha mismatch: ' + $zipRel) }
            if ((Get-Item -LiteralPath $zipPath).Length -ne [int]$manifest.package.zip_bytes) { $problems += ('package zip bytes mismatch: ' + $zipRel) }
            if ($manifest.package.sidecar) {
                $sidecar = Join-Path $script:Repo (([string]$manifest.package.sidecar) -replace '/', '\')
                if (-not (Test-Path -LiteralPath $sidecar)) { $problems += 'sidecar missing' }
                elseif ((Sha256 $sidecar) -ne [string]$manifest.package.sidecar_sha256) { $problems += 'sidecar sha mismatch' }
            }
            if ($manifest.package.checksums) {
                $checksums = Join-Path $script:Repo (([string]$manifest.package.checksums) -replace '/', '\')
                if (-not (Test-Path -LiteralPath $checksums)) { $problems += 'package CHECKSUMS.txt missing' }
                elseif ((Sha256 $checksums) -ne [string]$manifest.package.checksums_sha256) { $problems += 'package CHECKSUMS.txt sha mismatch' }
            }
            $stream = [System.IO.File]::OpenRead($zipPath)
            try {
                $zip = New-Object System.IO.Compression.ZipArchive($stream, [System.IO.Compression.ZipArchiveMode]::Read, $false)
                try {
                    $expected = @($manifest.package.contents | ForEach-Object { [string]$_ })
                    $actual = @($zip.Entries | ForEach-Object { $_.FullName })
                    foreach ($name in $expected) { if ($actual -notcontains $name) { $problems += ('zip missing member: ' + $name) } }
                    foreach ($artifact in @($manifest.artifacts)) {
                        $name = [System.IO.Path]::GetFileName([string]$artifact.path)
                        $entry = $zip.Entries | Where-Object { $_.FullName -eq $name } | Select-Object -First 1
                        if (-not $entry) { $problems += ('zip missing packaged artifact: ' + $name); continue }
                        $es = $entry.Open()
                        try {
                            $ms = New-Object System.IO.MemoryStream
                            try { $es.CopyTo($ms); $hashBytes = $ms.ToArray() } finally { $ms.Dispose() }
                        }
                        finally { $es.Dispose() }
                        $sha = [System.Security.Cryptography.SHA256]::Create()
                        $hash = ([System.BitConverter]::ToString($sha.ComputeHash($hashBytes)) -replace '-', '').ToLowerInvariant()
                        $sha.Dispose()
                        if ($hash -ne [string]$artifact.sha256) { $problems += ('zip member sha mismatch: ' + $name) }
                    }
                }
                finally { $zip.Dispose() }
            }
            finally { $stream.Dispose() }
        }
    }
    else { $problems += 'manifest has no package block' }

    if ($manifest.web -and $manifest.web.files) {
        $treeLines = @()
        foreach ($wf in @($manifest.web.files)) {
            $p = Join-Path $script:Repo (([string]$wf.path) -replace '/', '\')
            if (-not (Test-Path -LiteralPath $p)) {
                if ($preReleaseMode) { Write-Host ("pre-release drift (not a failure): web dist file missing after dev rebuild: " + [string]$wf.path) -ForegroundColor Yellow }
                else { $problems += ('web dist file missing: ' + [string]$wf.path) }
                continue
            }
            $sha = Sha256 $p
            $bytes = (Get-Item -LiteralPath $p).Length
            if ($sha -ne [string]$wf.sha256) {
                if ($preReleaseMode) { Write-Host ("pre-release drift (not a failure): web dist differs from sealed manifest: " + [string]$wf.path) -ForegroundColor Yellow }
                else { $problems += ('web dist sha mismatch: ' + [string]$wf.path) }
            }
            if ($bytes -ne [int]$wf.bytes) {
                if ($preReleaseMode) { Write-Host ("pre-release drift (not a failure): web dist size differs from sealed manifest: " + [string]$wf.path) -ForegroundColor Yellow }
                else { $problems += ('web dist bytes mismatch: ' + [string]$wf.path) }
            }
            $treeLines += ([string]$wf.path + "`t" + $sha + "`t" + $bytes)
        }
        if ($manifest.web.tree_sha256) {
            $sorted = @($treeLines | Sort-Object)
            $joined = [string]::Join("`n", $sorted) + "`n"
            $sha = [System.Security.Cryptography.SHA256]::Create()
            $tree = ([System.BitConverter]::ToString($sha.ComputeHash([System.Text.Encoding]::UTF8.GetBytes($joined))) -replace '-', '').ToLowerInvariant()
            $sha.Dispose()
            if ($tree -ne [string]$manifest.web.tree_sha256) {
                if ($preReleaseMode) { Write-Host "pre-release drift (not a failure): web dist tree_sha256 differs from sealed manifest (dev build expected)" -ForegroundColor Yellow }
                else { $problems += 'web dist tree_sha256 mismatch (rebuild changed dist?)' }
            }
        }
    }
    else {
        if ($preReleaseMode) { Write-Host "pre-release drift (not a failure): manifest has no web dist block" -ForegroundColor Yellow }
        else { $problems += 'manifest has no web dist block' }
    }

    if ($preReleaseMode) {
        Write-Host "pre-release tree: sealed tag/source and package zip/sidecar/CHECKSUMS verified strictly; workspace artifact/web-dist drift is warned (dev build), not a release claim" -ForegroundColor Yellow
    }
    if ($problems.Count -gt 0) { throw ('manifest/artifact problems: ' + (($problems | Select-Object -Unique) -join '; ')) }
    Write-Host "manifest, artifacts, package zip/sidecar/checksums and web dist hashes verified"
} | Out-Null

# ------------------------------------------------------------------ summary
$headEnd = (& git -C $script:Repo rev-parse HEAD).Trim()
$pass = @($script:Steps | Where-Object { $_.status -eq 'PASS' }).Count
$fail = @($script:Steps | Where-Object { $_.status -eq 'FAIL' }).Count
$skip = @($script:Steps | Where-Object { $_.status -eq 'SKIPPED' }).Count
$releaseSkipFail = [bool]($Release -and $skip -gt 0)
$preReleaseAccepted = [bool]$script:PreReleaseAccepted

Write-Host ""
Write-Host "================================================================" -ForegroundColor Cyan
Write-Host ("HEAD start  " + $head)
Write-Host ("HEAD end    " + $headEnd + $(if ($headEnd -ne $head) { '  (CHANGED - investigate)' } else { '' }))
if ($Rehearsal) { Write-Host 'MODE        REHEARSAL (dirty tree / stale manifest allowed)' -ForegroundColor Yellow }
if ($Release -and $preReleaseAccepted) { Write-Host ('MODE        PRE-RELEASE (working tree ' + $version + '; sealed ' + [string]$manifest.version + ' remains the release record; NOT a release PASS)') -ForegroundColor Yellow }
elseif ($Release) { Write-Host 'MODE        RELEASE (clean tree, manifest.source_commit == HEAD, zero skipped gates)' -ForegroundColor Yellow }
Write-Host ("passed " + $pass + " | failed " + $fail + " | skipped " + $skip + " (skips are never PASS)")
foreach ($step in $script:Steps) {
    Write-Host ("  " + $step.status.PadRight(9) + " exit=" + $step.exit + "  " + ("{0:0.0}s" -f $step.seconds).PadLeft(7) + "  " + $step.name + $(if ($step.detail) { '  :: ' + $step.detail } else { '' }))
}
if ($fail -gt 0 -or $headEnd -ne $head -or $releaseSkipFail) {
    Write-Host ""
    if ($releaseSkipFail) { Write-Host ("RELEASE MODE: " + $skip + " skipped gate(s) cannot PASS the release gate") -ForegroundColor Red }
    Write-Host "GATES: FAIL" -ForegroundColor Red
    exit 1
}
if ($Release -and $preReleaseAccepted) {
    Write-Host ""
    Write-Host ("GATES: PRE-RELEASE (not a release) - working tree " + $version + "; sealed " + [string]$manifest.version + " remains authoritative") -ForegroundColor Yellow
    exit 3
}
Write-Host ""
if ($skip -gt 0) {
    Write-Host ("GATES: PASS (with " + $skip + " named SKIP(s); skips are not PASS)") -ForegroundColor Yellow
}
else {
    Write-Host "GATES: PASS" -ForegroundColor Green
}
exit 0
