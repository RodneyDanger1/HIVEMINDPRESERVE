param(
    [string]$ReceiptPath = ""
)
$ErrorActionPreference = 'Stop'
$root = Split-Path -Parent (Split-Path -Parent $PSScriptRoot)
if (-not $ReceiptPath) {
    $ReceiptPath = Join-Path $root ("docs\evidence\live\scale-drill-{0}.json" -f (Get-Date -Format 'yyyyMMdd-HHmmss'))
}
$env:PATH = (Join-Path $root 'tools\w64devkit\bin') + ';' + $env:USERPROFILE + '\.cargo\bin;' + $env:PATH
Push-Location (Join-Path $root 'backend')
$oldErrorAction = $ErrorActionPreference
$ErrorActionPreference = 'Continue'
try {
    $output = & cargo test -p hiverouter-core --test scale_s0 --test scale_s1 --locked 2>&1
    $exit = $LASTEXITCODE
} finally {
    $ErrorActionPreference = $oldErrorAction
    Pop-Location
}
$text = ($output | Out-String)
$result = [ordered]@{
    receipt = 'hivemind.scale-drill.v1'
    checked_at = (Get-Date).ToString('o')
    command = 'cargo test -p hiverouter-core --test scale_s0 --test scale_s1 --locked'
    exit_code = $exit
    s0_small_fanout_pass = ($text -match 's0_probe_measures_small_fanout \.\.\. ok')
    s1_serial_41_pass = ($text -match 's1_serial_baseline_measures_41_tasks \.\.\. ok')
    s1_parallel_41_pass = ($text -match 's1_parallel_baseline_measures_41_tasks_at_bound_32 \.\.\. ok')
    note = 'These are the repository scale probes: 41-task serial and 41-task bounded-parallel. This is not a live 40-node provider run.'
}
$result['ok'] = ($exit -eq 0) -and $result.s0_small_fanout_pass -and $result.s1_serial_41_pass -and $result.s1_parallel_41_pass
$result | ConvertTo-Json -Depth 5 | Set-Content -LiteralPath $ReceiptPath -Encoding UTF8
if (-not $result.ok) {
    $text | Select-Object -Last 60
    throw "scale drill failed with exit $exit"
}
Write-Output ($result | ConvertTo-Json -Depth 5)
