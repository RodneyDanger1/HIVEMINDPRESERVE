# Save the non-secret provider profile used by HiveRouter/HiveMind at boot.
#
# The API key is NOT stored here. Store it separately with:
#   powershell -ExecutionPolicy Bypass -File tools\set-provider-key.ps1
# The desktop launcher injects that DPAPI-protected key into the child only.
#
# Examples:
#   powershell -ExecutionPolicy Bypass -File tools\set-provider-profile.ps1 `
#     -Name tokenlab -BaseUrl https://api.tokenlab.amd.com/v1 `
#     -WireModel DeepSeek-V4.1-Flash
#
#   powershell -ExecutionPolicy Bypass -File tools\set-provider-profile.ps1 `
#     -Name deepseek
#
# Restart the server/desktop app after changing the profile. Environment
# variables still override the saved file.

[CmdletBinding()]
param(
    [Parameter(Mandatory = $true)]
    [ValidateSet('deepseek', 'tokenlab', 'custom')]
    [string]$Name,

    [string]$BaseUrl,

    [string]$WireModel,

    [ValidateSet('deepseek-flash', 'deepseek-v4-pro')]
    [string]$Model = 'deepseek-flash'
)

$ErrorActionPreference = 'Stop'

$profile = switch ($Name) {
    'deepseek' {
        @{ name = 'deepseek'; base_url = 'https://api.deepseek.com'; model = $Model }
    }
    'tokenlab' {
        if ([string]::IsNullOrWhiteSpace($BaseUrl)) {
            throw 'Token Lab requires -BaseUrl (for example https://api.tokenlab.amd.com/v1).'
        }
        if ([string]::IsNullOrWhiteSpace($WireModel)) {
            throw 'Token Lab requires -WireModel (for example DeepSeek-V4.1-Flash).'
        }
        @{ name = 'tokenlab'; base_url = $BaseUrl.TrimEnd('/'); model = $Model; wire_model = $WireModel }
    }
    'custom' {
        if ([string]::IsNullOrWhiteSpace($BaseUrl)) {
            throw 'A custom OpenAI-compatible provider requires -BaseUrl.'
        }
        if ([string]::IsNullOrWhiteSpace($WireModel)) {
            throw 'A custom OpenAI-compatible provider requires -WireModel.'
        }
        @{ name = 'custom'; base_url = $BaseUrl.TrimEnd('/'); model = $Model; wire_model = $WireModel }
    }
}

$dir = Join-Path $env:LOCALAPPDATA 'HiveMind'
New-Item -ItemType Directory -Force -Path $dir | Out-Null
$file = Join-Path $dir 'provider.json'
$json = $profile | ConvertTo-Json -Depth 4
[IO.File]::WriteAllText($file, $json, (New-Object System.Text.UTF8Encoding($false)))
Write-Host "Provider profile saved at $file" -ForegroundColor Green
Write-Host 'The API key is stored separately with tools\set-provider-key.ps1; this file contains no secret.'
Write-Host 'Restart the server/desktop app to apply the profile.'
