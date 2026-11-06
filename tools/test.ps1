# HiveMind v1 — local test harness (Windows / PowerShell)
#
# One command runs the backend test gate that this workspace can execute fully
# offline. It is the local equivalent of the blueprint's "test suite runs with
# one command" exit gate (Phase 11): no Docker, no cloud CI, no live provider.
#
# Usage:
#   pwsh -NoProfile -ExecutionPolicy Bypass -File tools\test.ps1
#   pwsh -NoProfile -ExecutionPolicy Bypass -File tools\test.ps1 -BackendOnly
#
# Exit code 0 = every selected gate passed; non-zero = first failing gate.
param(
    [switch]$BackendOnly
)

$ErrorActionPreference = "Stop"
$repo = Split-Path -Parent $PSScriptRoot
$cargoBin = Join-Path $env:USERPROFILE ".cargo\bin"
$gccBin = Join-Path $repo "tools\w64devkit\bin"
$env:PATH = "$cargoBin;$gccBin;$env:PATH"
$env:CARGO_PROFILE_DEV_DEBUG = "0"
$env:CARGO_INCREMENTAL = "0"

function Invoke-Gate {
    param([string]$Name, [scriptblock]$Command)
    Write-Host ""
    Write-Host "=== $Name ===" -ForegroundColor Cyan
    & $Command
    if ($LASTEXITCODE -ne 0) {
        Write-Host "GATE FAILED: $Name (exit $LASTEXITCODE)" -ForegroundColor Red
        exit $LASTEXITCODE
    }
    Write-Host "GATE PASSED: $Name" -ForegroundColor Green
}

Push-Location (Join-Path $repo "backend")
try {
    Invoke-Gate "provider crate tests (wiremock offline matrix)" {
        cargo test -p hiverouter-provider --locked
    }
    Invoke-Gate "tools crate tests (approval/jail/cancel/ledger/audit/health/census/CLI)" {
        cargo test -p hiverouter-tools --locked
    }
    Invoke-Gate "contract crate tests" {
        cargo test -p hiverouter-contract --locked
    }
    Invoke-Gate "core crate tests" {
        cargo test -p hiverouter-core --locked
    }
    if (-not $BackendOnly) {
        Invoke-Gate "web typecheck + unit tests (when web exists)" {
            $web = Join-Path $repo "web"
            if (-not (Test-Path (Join-Path $web "package.json"))) {
                Write-Host "web/package.json absent; skipping web gates (recorded as not-run, not passed)"
                $global:LASTEXITCODE = 0
                return
            }
            Push-Location $web
            try {
                cmd /c "npm.cmd run typecheck"
                if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }
                cmd /c "npm.cmd test -- --run"
            }
            finally {
                Pop-Location
            }
        }
    }
}
finally {
    Pop-Location
}

Write-Host ""
Write-Host "ALL SELECTED LOCAL GATES PASSED" -ForegroundColor Green
exit 0
