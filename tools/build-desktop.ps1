# Build HiveMind as a portable Windows app folder.
#
# Output:
#   dist\HiveMind\HiveMind.exe      launcher (no Tauri / no MSVC)
#   dist\HiveMind\hiverouter.exe    embedded-UI server
#   dist\HiveMind\README.txt
#   dist\HiveMind.zip               optional portable archive
#
# Run from any directory:
#   powershell -ExecutionPolicy Bypass -File tools\build-desktop.ps1
#
# The launcher prefers hiverouter.exe next to itself, so the folder is portable.

param(
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

Write-Host "[1/5] web build" -ForegroundColor Cyan
Push-Location $web
try {
    & npm.cmd run build
    if ($LASTEXITCODE -ne 0) { throw "web build failed: $LASTEXITCODE" }
} finally {
    Pop-Location
}

Write-Host "[2/5] server build ($Configuration)" -ForegroundColor Cyan
Push-Location $backend
try {
    $env:PATH = "$env:USERPROFILE\.cargo\bin;$root\tools\w64devkit\bin;$env:PATH"
    & cargo build --release --locked
    if ($LASTEXITCODE -ne 0) { throw "server build failed: $LASTEXITCODE (stop any running hiverouter.exe first)" }
} finally {
    Pop-Location
}

Write-Host "[3/5] launcher compile" -ForegroundColor Cyan
if (-not (Test-Path $csc)) {
    throw "csc.exe not found at $csc"
}
$launcherExe = Join-Path $launcherDir 'HiveMind.exe'
& $csc /nologo /target:winexe /optimize+ /out:$launcherExe /r:System.Windows.Forms.dll (Join-Path $launcherDir 'HiveMindLauncher.cs')
if ($LASTEXITCODE -ne 0) {
    throw "launcher compile failed: $LASTEXITCODE"
}

Write-Host "[4/5] assemble portable folder" -ForegroundColor Cyan
New-Item -ItemType Directory -Force -Path $outDir | Out-Null
Copy-Item -LiteralPath $launcherExe -Destination $outDir -Force
$server = Join-Path $root "target\$Configuration\hiverouter.exe"
if (-not (Test-Path $server)) {
    throw "server exe missing: $server"
}
Copy-Item -LiteralPath $server -Destination $outDir -Force
@'
HiveMind portable desktop build
================================
Run HiveMind.exe.

It starts hiverouter.exe (kept beside this file) on a free loopback port,
waits for /health, then opens the HiveMind UI in an Edge WebView2 app window.

Data and the WebView profile live in:
  %LOCALAPPDATA%\HiveMind

The launcher keeps the server windowless. Closing the app window stops the
server process it owns. If a compatible HiveMind server is already running on
127.0.0.1:8099, the launcher reuses it instead of starting another one.

No Tauri, no Visual Studio, no MSVC Build Tools are required for this build.
The UI is the same embedded web/dist served by the Rust binary.
'@ | Set-Content -LiteralPath (Join-Path $outDir 'README.txt') -Encoding ASCII

Write-Host "[5/5] archive" -ForegroundColor Cyan
if (-not $NoZip) {
    $zip = Join-Path $root 'dist\HiveMind.zip'
    if (Test-Path $zip) {
        Remove-Item -LiteralPath $zip -Force
    }
    Compress-Archive -Path (Join-Path $outDir '*') -DestinationPath $zip
    Write-Host "portable zip: $zip"
}

Get-ChildItem $outDir | Select-Object Name, Length
Write-Host "done: $outDir" -ForegroundColor Green
