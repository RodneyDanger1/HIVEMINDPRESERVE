# HiveMind v1 — release build (Windows / PowerShell)
#
# Produces the v1 artifact truthfully: ONE localhost server binary (`hiverouter`,
# built by the `hiverouter-core` crate) and ONE local CLI binary (`hiverouter-cli`
# from `hiverouter-tools`). There is NO installer and NO desktop shell in v1
# (blueprint D3/CF-04); this script prints the exact artifact paths + SHA-256
# hashes so the evidence record can quote them.
#
# Usage: pwsh -NoProfile -ExecutionPolicy Bypass -File tools\build-release.ps1
param()

$ErrorActionPreference = "Stop"
$repo = Split-Path -Parent $PSScriptRoot
$env:PATH = "$($env:USERPROFILE)\.cargo\bin;$(Join-Path $repo 'tools\w64devkit\bin');$env:PATH"
$env:CARGO_PROFILE_DEV_DEBUG = "0"
$env:CARGO_INCREMENTAL = "0"

Push-Location (Join-Path $repo "backend")
try {
    Write-Host "=== cargo build --release --locked ===" -ForegroundColor Cyan
    cargo build --release --locked
    if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }

    $artifacts = @(
        (Join-Path $repo "target\release\hiverouter.exe"),
        (Join-Path $repo "target\release\hiverouter-cli.exe")
    )
    Write-Host ""
    Write-Host "=== release artifacts (path + SHA-256) ===" -ForegroundColor Cyan
    $manifest = @()
    foreach ($artifact in $artifacts) {
        if (Test-Path $artifact) {
            $hash = (Get-FileHash -Algorithm SHA256 $artifact).Hash.ToLowerInvariant()
            Write-Host ("{0}  {1}" -f $hash, $artifact)
            $manifest += [pscustomobject]@{
                path = $artifact
                sha256 = $hash
                bytes = (Get-Item $artifact).Length
            }
        }
        else {
            Write-Host "NOT BUILT (recorded as missing, not as passed): $artifact" -ForegroundColor Yellow
        }
    }
    $manifest | ConvertTo-Json -Depth 4 | Set-Content -Encoding utf8 (Join-Path $PSScriptRoot "release-manifest.json")
    Write-Host ""
    Write-Host "Packaging statement: v1 = localhost server + browser UI. No installer/desktop shell is claimed."
}
finally {
    Pop-Location
}
