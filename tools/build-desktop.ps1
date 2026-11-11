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
    [switch]$NoZip
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
$commit = (& git -C $root rev-parse HEAD).Trim()
$dirty = [bool](& git -C $root status --porcelain)

Write-Host "[1/5] web build" -ForegroundColor Cyan
Push-Location $web
try {
    & cmd.exe /c 'npm.cmd run build'
    if ($LASTEXITCODE -ne 0) { throw "web build failed: $LASTEXITCODE" }
} finally {
    Pop-Location
}

Write-Host "[2/5] server build ($Configuration)" -ForegroundColor Cyan
Push-Location $backend
try {
    $env:PATH = "$env:USERPROFILE\.cargo\bin;$root\tools\w64devkit\bin;$env:PATH"
    & cargo @profileArgs
    if ($LASTEXITCODE -ne 0) { throw "server build failed: $LASTEXITCODE (stop any running hiverouter.exe first)" }
} finally {
    Pop-Location
}
if (-not (Test-Path -LiteralPath $server)) { throw "server exe missing: $server" }

Write-Host "[3/5] launcher compile" -ForegroundColor Cyan
if (-not (Test-Path $csc)) { throw "csc.exe not found at $csc" }
$launcherExe = Join-Path $launcherDir 'HiveMind.exe'
& $csc /nologo /target:winexe /optimize+ /out:$launcherExe /r:System.Windows.Forms.dll /r:System.Security.dll (Join-Path $launcherDir 'HiveMindLauncher.cs')
if ($LASTEXITCODE -ne 0) { throw "launcher compile failed: $LASTEXITCODE" }

Write-Host "[4/5] assemble portable folder" -ForegroundColor Cyan
New-Item -ItemType Directory -Force -Path $outDir | Out-Null
Copy-Item -LiteralPath $launcherExe -Destination $outDir -Force
Copy-Item -LiteralPath $server -Destination $outDir -Force
foreach ($script in @('set-provider-key.ps1', 'set-provider-profile.ps1')) {
    $source = Join-Path $root "tools\$script"
    if (Test-Path -LiteralPath $source) { Copy-Item -LiteralPath $source -Destination $outDir -Force }
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

Install for this user (shortcuts, first-run setup):
  powershell -ExecutionPolicy Bypass -File install-desktop.ps1

No Tauri, no Visual Studio, no MSVC Build Tools are required.
The UI is the same embedded web/dist served by the Rust binary.
"@ | Set-Content -LiteralPath (Join-Path $outDir 'README.txt') -Encoding ASCII

$launcherHash = (Get-FileHash -Algorithm SHA256 -LiteralPath (Join-Path $outDir 'HiveMind.exe')).Hash.ToLowerInvariant()
$serverHash = (Get-FileHash -Algorithm SHA256 -LiteralPath (Join-Path $outDir 'hiverouter.exe')).Hash.ToLowerInvariant()
@{
    version = $version
    commit = $commit
    dirty = $dirty
    built_at = (Get-Date).ToString('o')
    server_sha256 = $serverHash
    launcher_sha256 = $launcherHash
} | ConvertTo-Json -Depth 4 | Set-Content -LiteralPath (Join-Path $outDir 'BUILD.json') -Encoding ASCII
$lines = foreach ($file in Get-ChildItem -LiteralPath $outDir -File | Sort-Object Name) {
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
