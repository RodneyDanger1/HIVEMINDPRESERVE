# Build HiveMind as a portable Windows app folder.
#
# Output:
#   dist\HiveMind\HiveMind.exe      launcher
#   dist\HiveMind\hiverouter.exe    embedded-UI server
#   dist\HiveMind\BUILD.json        commit/version/hashes/dirty
#   dist\HiveMind\CHECKSUMS.txt
#   dist\HiveMind\README.txt
#   dist\HiveMind.zip               optional portable archive
#
# Run from any directory:
#   powershell -ExecutionPolicy Bypass -File tools\build-desktop.ps1

param(
    [ValidateSet('release', 'debug')]
    [string]$Configuration = 'release',
    [switch]$NoZip,
    # Repackage with the existing server binary instead of compiling it. The
    # reused binary still embeds the UI it was compiled with, so the package is
    # only UI-current after a full build; BUILD.json records server_reused.
    [switch]$SkipServerBuild
)

$ErrorActionPreference = 'Stop'
$root = Split-Path -Parent $PSScriptRoot
$web = Join-Path $root 'web'
$backend = Join-Path $root 'backend'
$launcherDir = Join-Path $root 'tools\desktop-launcher'
$outDir = Join-Path $root 'dist\HiveMind'
$csc = Join-Path $env:WINDIR 'Microsoft.NET\Framework64\v4.0.30319\csc.exe'
$profileArgs = if ($Configuration -eq 'debug') { @('build', '--profile', 'debug', '--locked') } else { @('build', '--release', '--locked') }
$server = Join-Path $root "target\$Configuration\hiverouter.exe"
$version = (Get-Content (Join-Path $web 'package.json') -Raw | ConvertFrom-Json).version
# Freeze identity at the start and refuse to publish a package when the tree
# moved while it was being built (otherwise BUILD.json names a different commit
# than the bytes actually shipped).
$startCommit = (& git -C $root rev-parse HEAD).Trim()
$startDirty = [bool](& git -C $root status --porcelain)
$commit = $startCommit
$dirty = $startDirty

Write-Host "[1/5] web build" -ForegroundColor Cyan
# A developer shell that still exports these would bake a fixed API origin or the
# dev mock into the shipped bundle. Clear them for the build instead of trusting
# the caller's environment.
foreach ($leaked in @('VITE_API_BASE', 'VITE_UI_MOCK')) {
    if (Test-Path "Env:$leaked") {
        Write-Warning "ignoring inherited $leaked=$((Get-Item "Env:$leaked").Value) for the shipped web build"
        Remove-Item "Env:$leaked"
    }
}
Push-Location $web
try {
    & cmd.exe /c 'npm.cmd run build'
    if ($LASTEXITCODE -ne 0) { throw "web build failed: $LASTEXITCODE" }
} finally {
    Pop-Location
}
$webDist = Join-Path $web 'dist'
if (-not (Test-Path -LiteralPath (Join-Path $webDist 'index.html'))) {
    throw "web build produced no $webDist\index.html; the embedded UI would be the previous build"
}
# Fingerprint the exact UI tree that rust-embed compiled into the server, so
# BUILD.json names what shipped instead of only the repository version.
$uiFiles = @(Get-ChildItem -LiteralPath $webDist -Recurse -File | Sort-Object FullName)
$uiManifestLines = foreach ($file in $uiFiles) {
    $relative = $file.FullName.Substring($webDist.Length + 1).Replace('\', '/')
    '{0}  {1}' -f (Get-FileHash -Algorithm SHA256 -LiteralPath $file.FullName).Hash.ToLowerInvariant(), $relative
}
$sha256 = [Security.Cryptography.SHA256]::Create()
try {
    $uiHash = -join ($sha256.ComputeHash([Text.Encoding]::UTF8.GetBytes(($uiManifestLines -join "`n"))) | ForEach-Object { $_.ToString('x2') })
} finally {
    $sha256.Dispose()
}
$uiFileCount = $uiFiles.Count

Write-Host "[2/5] server build ($Configuration)" -ForegroundColor Cyan
if ($SkipServerBuild) {
    if (-not (Test-Path -LiteralPath $server)) { throw "-SkipServerBuild was requested but $server does not exist" }
    Write-Warning "reusing $server; it embeds the web/dist it was compiled with, so re-run without -SkipServerBuild to ship this UI build"
} else {
    Push-Location $backend
    try {
        $env:PATH = "$env:USERPROFILE\.cargo\bin;$root\tools\w64devkit\bin;$env:PATH"
        & cargo @profileArgs
        if ($LASTEXITCODE -ne 0) { throw "server build failed: $LASTEXITCODE (stop any running hiverouter.exe first)" }
    } finally {
        Pop-Location
    }
}
if (-not (Test-Path -LiteralPath $server)) { throw "server exe missing: $server" }

Write-Host "[3/5] launcher compile" -ForegroundColor Cyan
if (-not (Test-Path $csc)) { throw "csc.exe not found at $csc" }
$launcherExe = Join-Path $env:TEMP ("HiveMindLauncher-" + [guid]::NewGuid().ToString('N') + '.exe')
& $csc /nologo /target:winexe /optimize+ /out:$launcherExe /r:System.Windows.Forms.dll /r:System.Security.dll (Join-Path $launcherDir 'HiveMindLauncher.cs')
if ($LASTEXITCODE -ne 0) { throw "launcher compile failed: $LASTEXITCODE" }

Write-Host "[4/5] assemble portable folder" -ForegroundColor Cyan
New-Item -ItemType Directory -Force -Path $outDir | Out-Null
# Remove stale temp-named launchers from an interrupted/older build.
Get-ChildItem -LiteralPath $outDir -Filter 'HiveMindLauncher-*.exe' -File -ErrorAction SilentlyContinue |
    Remove-Item -Force -ErrorAction SilentlyContinue
Copy-Item -LiteralPath $launcherExe -Destination (Join-Path $outDir 'HiveMind.exe') -Force
Remove-Item -LiteralPath $launcherExe -Force -ErrorAction SilentlyContinue
Copy-Item -LiteralPath $server -Destination $outDir -Force
# Ship the installer/uninstaller inside the package: README.txt tells the user to
# run install-desktop.ps1 from this folder, and the uninstall registry entry
# points at the copy installed next to the app.
foreach ($script in @('set-provider-key.ps1', 'set-provider-profile.ps1', 'install-desktop.ps1', 'uninstall-desktop.ps1')) {
    $source = Join-Path $root "tools\$script"
    if (-not (Test-Path -LiteralPath $source)) { throw "packaging script missing: $source" }
    Copy-Item -LiteralPath $source -Destination $outDir -Force
}
@"
HiveMind portable desktop build
================================
Version: $version
Commit:  $commit
Built:   $((Get-Date).ToString('o'))

Run HiveMind.exe. It starts hiverouter.exe (beside this file) on the first free
loopback port in 8099..8199, waits for /health, then opens the UI in an Edge or
Chrome app window.

Data and the workspace live in:
  %LOCALAPPDATA%\HiveMind\provider.key      DPAPI-encrypted provider key
  %LOCALAPPDATA%\HiveMind\provider.json     non-secret provider profile
  %LOCALAPPDATA%\HiveMind\workspace         agent workspace (created on launch)

The launcher decrypts the provider key and injects it into the server child only.
Create or rotate it with the adjacent script:
  powershell -ExecutionPolicy Bypass -File set-provider-key.ps1
The key is never written into this folder, source, command lines, or logs.

Install for this user (shortcuts, first-run setup, uninstall entry):
  powershell -ExecutionPolicy Bypass -File install-desktop.ps1

Upgrade: run install-desktop.ps1 again over the existing install. User data is
never touched, and an older package is refused unless -Downgrade is passed.
Uninstall (user data preserved):
  powershell -ExecutionPolicy Bypass -File uninstall-desktop.ps1

No Tauri, no Visual Studio, no MSVC Build Tools and no Visual C++ redistributable
are required: the server binary imports only Windows system DLLs, and the UI is
the embedded web/dist served by that binary.
"@ | Set-Content -LiteralPath (Join-Path $outDir 'README.txt') -Encoding ASCII

$endCommit = (& git -C $root rev-parse HEAD).Trim()
$endDirty = [bool](& git -C $root status --porcelain)
if ($endCommit -ne $startCommit -or $endDirty -ne $startDirty) {
    throw "source identity changed during the build (start=$startCommit dirty=$startDirty end=$endCommit dirty=$endDirty); refusing to stamp ambiguous BUILD.json"
}
$launcherHash = (Get-FileHash -Algorithm SHA256 -LiteralPath (Join-Path $outDir 'HiveMind.exe')).Hash.ToLowerInvariant()
$serverHash = (Get-FileHash -Algorithm SHA256 -LiteralPath (Join-Path $outDir 'hiverouter.exe')).Hash.ToLowerInvariant()
@{
    version = $version
    commit = $commit
    dirty = $dirty
    built_at = (Get-Date).ToString('o')
    server_sha256 = $serverHash
    launcher_sha256 = $launcherHash
    ui_sha256 = $uiHash
    ui_files = $uiFileCount
    server_reused = [bool]$SkipServerBuild
    contract_schema = 'hiverouter.v1'
} | ConvertTo-Json -Depth 4 | Set-Content -LiteralPath (Join-Path $outDir 'BUILD.json') -Encoding ASCII
# CHECKSUMS.txt must not list itself: a manifest cannot contain its own hash.
$lines = foreach ($file in Get-ChildItem -LiteralPath $outDir -File | Where-Object { $_.Name -ne 'CHECKSUMS.txt' } | Sort-Object Name) {
    '{0}  {1}' -f (Get-FileHash -Algorithm SHA256 -LiteralPath $file.FullName).Hash.ToLowerInvariant(), $file.Name
}
$lines | Set-Content -LiteralPath (Join-Path $outDir 'CHECKSUMS.txt') -Encoding ASCII

Write-Host "[5/5] archive" -ForegroundColor Cyan
if (-not $NoZip) {
    $zip = Join-Path $root 'dist\HiveMind.zip'
    if (Test-Path $zip) { Remove-Item -LiteralPath $zip -Force }
    Compress-Archive -Path (Join-Path $outDir '*') -DestinationPath $zip
    $zipHash = (Get-FileHash -Algorithm SHA256 -LiteralPath $zip).Hash.ToLowerInvariant()
    $zipHash | Set-Content -LiteralPath ($zip + '.sha256') -Encoding ASCII
    Write-Host "portable zip: $zip ($zipHash)"
}

Get-ChildItem $outDir | Select-Object Name, Length
Write-Host "done: $outDir" -ForegroundColor Green
