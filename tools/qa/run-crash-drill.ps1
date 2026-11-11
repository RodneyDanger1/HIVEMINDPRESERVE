param(
    [string]$Server = "",
    [int]$Port = 8117,
    [string]$ReceiptPath = ""
)
$ErrorActionPreference = 'Stop'
$root = Split-Path -Parent (Split-Path -Parent $PSScriptRoot)
if (-not $Server) { $Server = Join-Path $root 'target\release\hiverouter.exe' }
if (-not (Test-Path -LiteralPath $Server)) { throw "server binary not found: $Server" }
if (-not $ReceiptPath) {
    $ReceiptPath = Join-Path $root ("docs\evidence\live\crash-drill-{0}.json" -f (Get-Date -Format 'yyyyMMdd-HHmmss'))
}
$data = Join-Path ([System.IO.Path]::GetTempPath()) ("hivemind-crash-{0}" -f ([guid]::NewGuid()))
$workspace = Join-Path $data 'workspace'
New-Item -ItemType Directory -Path $workspace -Force | Out-Null

function Wait-Health([int]$TimeoutSeconds) {
    $deadline = (Get-Date).AddSeconds($TimeoutSeconds)
    while ((Get-Date) -lt $deadline) {
        try {
            return Invoke-RestMethod -Uri "http://127.0.0.1:$Port/health" -TimeoutSec 1
        } catch { Start-Sleep -Milliseconds 200 }
    }
    throw "server did not become healthy on port $Port"
}

function Start-Server {
    return Start-Process -FilePath $Server -ArgumentList @(
        '--bind', "127.0.0.1:$Port",
        '--data-dir', $data,
        '--workspace-root', $workspace
    ) -WorkingDirectory $root -PassThru
}

$first = Start-Server
$firstHealth = Wait-Health 20
$firstPid = $first.Id
$firstBoot = [string]$firstHealth.boot_id
Stop-Process -Id $firstPid -Force
for ($i = 0; $i -lt 50; $i++) {
    try { Invoke-RestMethod -Uri "http://127.0.0.1:$Port/health" -TimeoutSec 1 | Out-Null; Start-Sleep -Milliseconds 200 }
    catch { break }
}
$second = Start-Server
$secondHealth = Wait-Health 20
$secondPid = $second.Id
$secondBoot = [string]$secondHealth.boot_id
Stop-Process -Id $secondPid -Force

$checkedAt = (Get-Date).ToString('o')
$result = [ordered]@{
    receipt = 'hivemind.crash-drill.v1'
    checked_at = $checkedAt
    server = $Server
    port = $Port
    data_dir = $data
    first_boot_id = $firstBoot
    second_boot_id = $secondBoot
    first_pid = $firstPid
    second_pid = $secondPid
    schema_version = [int]$secondHealth.database.schema_version
    version = [string]$secondHealth.version
    data_dir_preserved = ([string]$secondHealth.data_dir -eq $data)
    boot_id_changed = ($firstBoot -ne $secondBoot)
    first_process_exited = -not (Get-Process -Id $firstPid -ErrorAction SilentlyContinue)
    second_process_exited = -not (Get-Process -Id $secondPid -ErrorAction SilentlyContinue)
}
$result['ok'] = $result.data_dir_preserved -and $result.boot_id_changed -and
    $result.first_process_exited -and $result.second_process_exited -and
    ($result.schema_version -eq 24)
$result | ConvertTo-Json -Depth 5 | Set-Content -LiteralPath $ReceiptPath -Encoding UTF8
if (-not $result.ok) { throw "crash drill failed: $($result | ConvertTo-Json -Compress)" }
Write-Output ($result | ConvertTo-Json -Depth 5)
