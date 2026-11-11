# Remove the per-user HiveMind app and shortcuts. User data is preserved unless -PurgeData.
#
# Usage:
#   powershell -ExecutionPolicy Bypass -File tools\uninstall-desktop.ps1
#   powershell -ExecutionPolicy Bypass -File tools\uninstall-desktop.ps1 -PurgeData
#
# Idempotent: running it when HiveMind is not installed reports that and exits 0.
[CmdletBinding()]
param(
    [string]$InstallRoot = (Join-Path $env:LOCALAPPDATA 'Programs\HiveMind\app'),
    [string]$DataRoot = (Join-Path $env:LOCALAPPDATA 'HiveMind'),
    # Leave the Desktop/Start Menu alone (staging and automation).
    [switch]$NoShortcuts,
    [switch]$PurgeData
)
$ErrorActionPreference = 'Stop'

# install.json (written by install-desktop.ps1) records the real roots, so a
# custom -InstallRoot/-DataRoot install uninstalls itself correctly.
$installRecord = Join-Path $InstallRoot 'install.json'
if (Test-Path -LiteralPath $installRecord) {
    try {
        $record = Get-Content -LiteralPath $installRecord -Raw | ConvertFrom-Json
        if (-not $PSBoundParameters.ContainsKey('InstallRoot') -and -not [string]::IsNullOrWhiteSpace($record.install_root)) {
            $InstallRoot = $record.install_root
        }
        if (-not $PSBoundParameters.ContainsKey('DataRoot') -and -not [string]::IsNullOrWhiteSpace($record.data_root)) {
            $DataRoot = $record.data_root
        }
    } catch {
        Write-Warning "install.json could not be parsed; using the default paths."
    }
}

# Stop only a launcher/server we own from this install root; never kill unrelated processes.
$installedExe = Join-Path $InstallRoot 'hiverouter.exe'
$installedLauncher = Join-Path $InstallRoot 'HiveMind.exe'
foreach ($name in @('hiverouter', 'HiveMind')) {
    $ownedPath = if ($name -eq 'hiverouter') { $installedExe } else { $installedLauncher }
    foreach ($process in @(Get-Process $name -ErrorAction SilentlyContinue)) {
        try {
            if ($process.Path -and $process.Path -eq $ownedPath) {
                Stop-Process -Id $process.Id -Force
                $process.WaitForExit(5000) | Out-Null
            }
        } catch {
            Write-Warning "Could not stop $name.exe (pid $($process.Id)): $($_.Exception.Message)"
        }
    }
}

$removed = $false
if (Test-Path -LiteralPath $InstallRoot) {
    Remove-Item -LiteralPath $InstallRoot -Recurse -Force
    $removed = $true
}
if (-not $NoShortcuts) {
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
}

# Add/Remove Programs entry written by install-desktop.ps1.
$uninstallKey = 'HKCU:\Software\Microsoft\Windows\CurrentVersion\Uninstall\HiveMind'
if (Test-Path -LiteralPath $uninstallKey) {
    Remove-Item -LiteralPath $uninstallKey -Recurse -Force
}

if ($PurgeData -and (Test-Path -LiteralPath $DataRoot)) {
    $leftover = @(Get-Process hiverouter -ErrorAction SilentlyContinue | Where-Object { $_.Path -eq $installedExe })
    if ($leftover.Count -gt 0) {
        throw "Refusing to purge $DataRoot while HiveMind's server is still running (pid $($leftover[0].Id))."
    }
    Remove-Item -LiteralPath $DataRoot -Recurse -Force
}

if (-not $removed) {
    Write-Host "HiveMind was not installed at $InstallRoot; shortcuts and registry entries were checked anyway." -ForegroundColor Yellow
} elseif ($NoShortcuts) {
    Write-Host 'HiveMind application files and uninstall entry removed; shortcuts were left in place (-NoShortcuts).' -ForegroundColor Green
} else {
    Write-Host 'HiveMind app, shortcuts and uninstall entry removed.' -ForegroundColor Green
}
if (-not $PurgeData) {
    Write-Host "User data preserved at $DataRoot (use -PurgeData to delete)." -ForegroundColor Yellow
}
