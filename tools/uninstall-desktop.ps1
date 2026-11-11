# Remove the per-user HiveMind app and shortcuts. User data is preserved unless -PurgeData.
[CmdletBinding()]
param(
    [string]$InstallRoot = (Join-Path $env:LOCALAPPDATA 'Programs\HiveMind\app'),
    [string]$DataRoot = (Join-Path $env:LOCALAPPDATA 'HiveMind'),
    [switch]$PurgeData
)
$ErrorActionPreference = 'Stop'
# Stop only a launcher/server we own from this install root; never kill unrelated processes.
$installedExe = Join-Path $InstallRoot 'hiverouter.exe'
foreach ($process in Get-Process hiverouter -ErrorAction SilentlyContinue) {
    try {
        if ($process.Path -and $process.Path -eq $installedExe) { Stop-Process -Id $process.Id -Force }
    } catch { }
}
if (Test-Path -LiteralPath $InstallRoot) {
    Remove-Item -LiteralPath $InstallRoot -Recurse -Force
}
$shortcuts = @(
    (Join-Path ([Environment]::GetFolderPath('Desktop')) 'HiveMind.lnk'),
    (Join-Path ([Environment]::GetFolderPath('StartMenu')) 'Programs\HiveMind\HiveMind.lnk')
)
foreach ($shortcut in $shortcuts) {
    if (Test-Path -LiteralPath $shortcut) { Remove-Item -LiteralPath $shortcut -Force }
}
$startFolder = Join-Path ([Environment]::GetFolderPath('StartMenu')) 'Programs\HiveMind'
if ((Test-Path -LiteralPath $startFolder) -and @(Get-ChildItem -LiteralPath $startFolder -Force).Count -eq 0) {
    Remove-Item -LiteralPath $startFolder -Force
}
if ($PurgeData -and (Test-Path -LiteralPath $DataRoot)) {
    Remove-Item -LiteralPath $DataRoot -Recurse -Force
}
Write-Host 'HiveMind app and shortcuts removed.' -ForegroundColor Green
if (-not $PurgeData) {
    Write-Host "User data preserved at $DataRoot (use -PurgeData to delete)." -ForegroundColor Yellow
}
