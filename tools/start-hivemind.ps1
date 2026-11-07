# Starts the built HiveMind localhost server and opens its UI.
# The app is a server + browser UI; there is no separate desktop executable.
param(
    [string]$Bind = '127.0.0.1:8099',
    [string]$DataDir = (Join-Path $env:LOCALAPPDATA 'HiveMind')
)

$ErrorActionPreference = 'Stop'
$root = Split-Path -Parent $PSScriptRoot
$exe = Join-Path $root 'target\release\hiverouter.exe'

if (-not (Test-Path $exe)) {
    throw "Missing $exe. Build it first: cd backend; cargo build --release --locked"
}

$base = "http://$Bind"
try {
    $health = Invoke-RestMethod -Uri "$base/health" -TimeoutSec 2
    Write-Host "HiveMind is already running at $base (schema $($health.database.schema_version))."
    Start-Process $base
    exit 0
} catch {
    # No healthy server on this address: start one.
}

New-Item -ItemType Directory -Force -Path $DataDir | Out-Null
$stdout = Join-Path $DataDir 'server.stdout.log'
$stderr = Join-Path $DataDir 'server.stderr.log'
$process = Start-Process -FilePath $exe `
    -ArgumentList '--bind', $Bind, '--data-dir', $DataDir `
    -PassThru -RedirectStandardOutput $stdout -RedirectStandardError $stderr

$deadline = (Get-Date).AddSeconds(20)
do {
    Start-Sleep -Milliseconds 300
    try {
        $health = Invoke-RestMethod -Uri "$base/health" -TimeoutSec 2
        break
    } catch {
        if ($process.HasExited) {
            $tail = Get-Content $stderr -ErrorAction SilentlyContinue | Select-Object -Last 20
            throw "HiveMind exited during boot. stderr:`n$($tail -join [Environment]::NewLine)"
        }
    }
} while ((Get-Date) -lt $deadline)

if ($null -eq $health) {
    throw "HiveMind did not become healthy within 20s. See $stderr"
}

Write-Host "HiveMind running at $base (pid $($process.Id), health $($health.status))."
Write-Host "Data: $DataDir"
Write-Host "Stop: Stop-Process -Id $($process.Id)"
Start-Process $base
