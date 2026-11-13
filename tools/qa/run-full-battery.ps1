# HiveMind full offline battery. No provider key is read, printed, or required.
#
# Usage:
#   powershell -ExecutionPolicy Bypass -File tools\qa\run-full-battery.ps1
#   ... -SkipBackend   # reuse a just-run backend gate; still runs web/docs/QA/health
#
# Writes a JSON receipt under docs/evidence/live/ and exits non-zero if any
# executed step fails. A skipped step is recorded as SKIPPED, never PASS.
[CmdletBinding()]
param(
    [switch]$SkipBackend,
    [switch]$SkipWeb,
    [switch]$SkipDocs,
    [switch]$SkipHealth
)
$ErrorActionPreference = 'Stop'
$root = Split-Path -Parent (Split-Path -Parent $PSScriptRoot)
if (-not (Test-Path (Join-Path $root 'backend\Cargo.toml'))) { throw "not a HiveMind repo: $root" }
$env:PATH = "$env:USERPROFILE\.cargo\bin;$root\tools\w64devkit\bin;$env:PATH"
$steps = New-Object System.Collections.Generic.List[object]
$started = Get-Date
function Invoke-Step([string]$Name, [scriptblock]$Body, [string]$Workdir) {
    $sw = [System.Diagnostics.Stopwatch]::StartNew()
    Write-Host "=== $Name ===" -ForegroundColor Cyan
    Push-Location $Workdir
    $global:LASTEXITCODE = 0
    try {
        & $Body
        $code = if ($null -eq $LASTEXITCODE) { 0 } else { [int]$LASTEXITCODE }
    } catch {
        Write-Host $_.Exception.Message -ForegroundColor Red
        $code = 1
    } finally {
        Pop-Location
        $sw.Stop()
    }
    $status = if ($code -eq 0) { 'PASS' } else { 'FAIL' }
    $steps.Add([pscustomobject]@{ name = $Name; status = $status; exit_code = $code; seconds = [math]::Round($sw.Elapsed.TotalSeconds, 2) })
    if ($code -ne 0) { Write-Host "STEP FAILED: $Name" -ForegroundColor Red }
    return $code
}
function Add-Skip([string]$Name) {
    $steps.Add([pscustomobject]@{ name = $Name; status = 'SKIPPED'; exit_code = $null; seconds = 0 })
}

$commit = (& git -C $root rev-parse HEAD).Trim()
if (-not $SkipBackend) {
    Invoke-Step 'backend cargo test --workspace --locked' { & cargo test --workspace --locked } (Join-Path $root 'backend') | Out-Null
} else { Add-Skip 'backend cargo test --workspace --locked' }

if (-not $SkipWeb) {
    Invoke-Step 'web typecheck' { & cmd.exe /c 'npm.cmd run typecheck' } (Join-Path $root 'web') | Out-Null
    Invoke-Step 'web unit tests' { & cmd.exe /c 'npm.cmd test -- --run' } (Join-Path $root 'web') | Out-Null
    Invoke-Step 'web production build' { & cmd.exe /c 'npm.cmd run build' } (Join-Path $root 'web') | Out-Null
} else {
    Add-Skip 'web typecheck'; Add-Skip 'web unit tests'; Add-Skip 'web production build'
}

if (-not $SkipDocs) {
    Invoke-Step 'docs truth checker' { & node tools/qa/check-docs-truth.mjs } $root | Out-Null
    Invoke-Step 'static QA harness' { & node tools/qa/verify-harness.mjs } $root | Out-Null
    $qaTests = @(Get-ChildItem -LiteralPath (Join-Path $root 'tools\qa\tests') -Filter '*.test.mjs' |
        Sort-Object Name | ForEach-Object { 'tools/qa/tests/' + $_.Name })
    Invoke-Step 'QA unit tests' { & node @('--test') + $qaTests } $root | Out-Null
} else {
    Add-Skip 'docs truth checker'; Add-Skip 'static QA harness'; Add-Skip 'QA unit tests'
}

if (-not $SkipHealth) {
    Invoke-Step 'installed server /health' {
        $health = Invoke-RestMethod -Uri 'http://127.0.0.1:8099/health' -TimeoutSec 3
        if ($health.status -notin @('ok', 'degraded')) { throw "unexpected health status: $($health.status)" }
        Write-Host ("health=" + $health.status + " version=" + $health.version + " schema=" + $health.schema + " key_missing=" + (-not ($health.checks | Where-Object { $_.name -eq 'deepseek_api_key' -and $_.ok })))
    } $root | Out-Null
} else { Add-Skip 'installed server /health' }

$failed = @($steps | Where-Object status -eq 'FAIL')
$receipt = [pscustomobject]@{
    receipt = 'hivemind.full-battery.v1'
    observed_at = (Get-Date).ToString('o')
    started_at = $started.ToString('o')
    source_commit = $commit
    working_tree_dirty = [bool](& git -C $root status --porcelain)
    executed = @($steps | Where-Object status -ne 'SKIPPED').Count
    skipped = @($steps | Where-Object status -eq 'SKIPPED').Count
    failed = $failed.Count
    steps = $steps
    verdict = if ($failed.Count -gt 0) { 'FAIL' } elseif (@($steps | Where-Object status -eq 'SKIPPED').Count -gt 0) { 'PARTIAL' } else { 'PASS' }
    note = 'Offline battery; no provider call or key was used. SKIPPED is not PASS.'
}
New-Item -ItemType Directory -Force -Path (Join-Path $root 'docs\evidence\live') | Out-Null
$stamp = Get-Date -Format 'yyyyMMdd-HHmmss'
$out = Join-Path $root "docs\evidence\live\full-battery-$stamp.json"
$receipt | ConvertTo-Json -Depth 6 | Set-Content -LiteralPath $out -Encoding UTF8
Write-Host ""
$steps | Format-Table -AutoSize
Write-Host "receipt: $out" -ForegroundColor Green
if ($failed.Count -gt 0) { exit 1 }
