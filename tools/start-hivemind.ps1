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
# Select the provider key environment from the saved non-secret profile.
$apiType = 'deepseek'
if (-not [string]::IsNullOrWhiteSpace($env:HIVEROUTER_API_TYPE)) {
    $apiType = $env:HIVEROUTER_API_TYPE
}
$profileFile = Join-Path $DataDir 'provider.json'
if (Test-Path $profileFile) {
    try {
        $providerProfile = Get-Content -LiteralPath $profileFile -Raw | ConvertFrom-Json
        if (-not [string]::IsNullOrWhiteSpace($providerProfile.api_type)) {
            $apiType = $providerProfile.api_type
        } elseif ($providerProfile.name -match '^tokenlab') {
            $apiType = 'tokenlabs'
        }
    } catch {
        Write-Warning 'The provider profile could not be parsed; defaulting to DeepSeek.'
    }
}
$keyEnv = if ($apiType -eq 'tokenlabs') { 'HIVEROUTER_API_KEY' } else { 'DEEPSEEK_API_KEY' }
# Prefer the DPAPI-encrypted key written by tools\set-provider-key.ps1. The
# decrypted value is put into the child process environment only; it is never
# printed, logged, or embedded in a command line.
$keyFile = Join-Path $DataDir 'provider.key'
if (Test-Path $keyFile) {
    try {
        Add-Type -AssemblyName System.Security
        $entropy = [Text.Encoding]::UTF8.GetBytes('HiveMind.ProviderKey.v1')
        $encrypted = [IO.File]::ReadAllBytes($keyFile)
        $plainBytes = [Security.Cryptography.ProtectedData]::Unprotect(
            $encrypted,
            $entropy,
            [Security.Cryptography.DataProtectionScope]::CurrentUser
        )
        $plain = [Text.Encoding]::UTF8.GetString($plainBytes).Trim()
        Set-Item -Path "Env:$keyEnv" -Value $plain
        $plain = $null
        [Array]::Clear($plainBytes, 0, $plainBytes.Length)
    } catch {
        Write-Warning 'The provider key file could not be decrypted; prompts will refuse E_KEY_MISSING.'
    }
}
$stdout = Join-Path $DataDir 'server.stdout.log'
$stderr = Join-Path $DataDir 'server.stderr.log'
$process = Start-Process -FilePath $exe `
    -ArgumentList '--bind', $Bind, '--data-dir', $DataDir, '--workspace-root', $root `
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
