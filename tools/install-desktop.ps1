# Install HiveMind per-user (no administrator rights).
#
# Usage:
#   powershell -ExecutionPolicy Bypass -File tools\install-desktop.ps1
#   powershell -ExecutionPolicy Bypass -File tools\install-desktop.ps1 -SkipProvider -NoLaunch
#   powershell -ExecutionPolicy Bypass -File install-desktop.ps1        (from the portable folder)
#
# Prerequisites are checked BEFORE anything is written, so a refused install
# leaves the machine untouched. Every refusal is named and names the fix.
#
# The installer never receives a provider key on its command line. With provider
# setup enabled it prompts with Read-Host -AsSecureString and writes only the
# DPAPI-encrypted provider.key under the data directory.
[CmdletBinding()]
param(
    [string]$InstallRoot = (Join-Path $env:LOCALAPPDATA 'Programs\HiveMind\app'),
    [string]$DataRoot = (Join-Path $env:LOCALAPPDATA 'HiveMind'),
    [string]$WorkspaceRoot = (Join-Path $env:LOCALAPPDATA 'HiveMind\workspace'),
    [ValidateSet('deepseek', 'tokenlab', 'custom', 'none')]
    [string]$Provider = 'deepseek',
    [string]$BaseUrl,
    [string]$WireModel,
    [switch]$SkipProvider,
    [switch]$NoLaunch,
    # Install without touching the Desktop/Start Menu (automation and staging).
    [switch]$NoShortcuts,
    [switch]$Force,
    # Allow installing an older package over a newer installation.
    [switch]$Downgrade,
    # Accept a package whose CHECKSUMS.txt does not match (never do this for a
    # downloaded package unless the source is trusted).
    [switch]$SkipIntegrityCheck
)

$ErrorActionPreference = 'Stop'

function Write-Step([string]$Text) {
    Write-Host "[install] $Text" -ForegroundColor Cyan
}

function Stop-Install([string]$Code, [string]$Message, [string]$Fix) {
    # A named refusal, mirroring the product's E_* envelope shape.
    $lines = @(
        "HiveMind install refused ($Code).",
        "",
        $Message
    )
    if (-not [string]::IsNullOrWhiteSpace($Fix)) {
        $lines += @('', "How to fix: $Fix")
    }
    throw ($lines -join [Environment]::NewLine)
}

# ---------------------------------------------------------------------------
# 0. Locate the portable package. The same script ships at tools\ and inside
#    dist\HiveMind\, so resolve by content instead of by one fixed layout.
# ---------------------------------------------------------------------------
$repoRoot = Split-Path -Parent $PSScriptRoot
$packageCandidates = @(
    (Join-Path $repoRoot 'dist\HiveMind'),
    $PSScriptRoot
)
$source = $null
foreach ($candidate in $packageCandidates) {
    if ((Test-Path -LiteralPath (Join-Path $candidate 'HiveMind.exe')) -and
        (Test-Path -LiteralPath (Join-Path $candidate 'hiverouter.exe'))) {
        $source = $candidate
        break
    }
}
if ($null -eq $source) {
    Stop-Install 'E_PACKAGE_MISSING' `
        "No portable package was found. Looked in: $($packageCandidates -join ', ')." `
        'Run tools\build-desktop.ps1 first, or run install-desktop.ps1 from inside dist\HiveMind.'
}
$launcher = Join-Path $source 'HiveMind.exe'
$server = Join-Path $source 'hiverouter.exe'
Write-Step "package: $source"

# ---------------------------------------------------------------------------
# 1. Prerequisites. Nothing is created or copied before every hard check passes.
# ---------------------------------------------------------------------------
$os = [Environment]::OSVersion.Version
if ($PSVersionTable.PSVersion -lt [version]'5.1') {
    Stop-Install 'E_OS_UNSUPPORTED' "PowerShell $($PSVersionTable.PSVersion) is too old." 'Install Windows Management Framework 5.1 or newer.'
}
if (-not [Environment]::Is64BitOperatingSystem) {
    Stop-Install 'E_OS_UNSUPPORTED' 'HiveMind ships an x86_64 server binary and cannot run on a 32-bit Windows installation.' 'Install the 64-bit Windows build.'
}

# .NET Framework 4.x runs HiveMind.exe (the launcher). Windows 10 1903+ and
# Windows 11 include it; older or stripped images may not.
$netKey = 'HKLM:\SOFTWARE\Microsoft\NET Framework Setup\NDP\v4\Full'
if (-not (Test-Path -LiteralPath $netKey)) {
    Stop-Install 'E_RUNTIME_MISSING' 'The .NET Framework 4.x runtime (needed by HiveMind.exe) is not installed.' 'Install .NET Framework 4.8 from Microsoft, then run this installer again.'
}
$netRelease = (Get-ItemProperty -LiteralPath $netKey -ErrorAction SilentlyContinue).Release
if ($null -eq $netRelease -or [int]$netRelease -lt 500000) {
    Write-Warning "The installed .NET Framework 4.x release ($netRelease) is older than 4.8; HiveMind.exe may fail to start. Consider updating .NET Framework."
}

$browsers = @(
    'C:\Program Files (x86)\Microsoft\Edge\Application\msedge.exe',
    'C:\Program Files\Google\Chrome\Application\chrome.exe',
    'C:\Program Files (x86)\Google\Chrome\Application\chrome.exe'
)
if (-not ($browsers | Where-Object { Test-Path -LiteralPath $_ })) {
    Write-Warning 'No Edge or Chrome installation was found. HiveMind falls back to your default browser, which will not get the app-window UI.'
}

# The shipped binaries are built for x86_64-pc-windows-gnu and import only
# OS-provided DLLs (kernel32/msvcrt/ws2_32/...); no Visual C++ redistributable
# and no MinGW runtime is required next to hiverouter.exe.
Write-Step 'prerequisite check passed (64-bit Windows, PowerShell 5.1+, .NET Framework 4.x)'

# Package integrity: CHECKSUMS.txt is written by tools\build-desktop.ps1.
$checksumFile = Join-Path $source 'CHECKSUMS.txt'
$integrity = 'not-verified'
if (Test-Path -LiteralPath $checksumFile) {
    if ($SkipIntegrityCheck) {
        $integrity = 'skipped'
        Write-Warning 'Package integrity check skipped by request (-SkipIntegrityCheck).'
    } else {
        $mismatches = @()
        foreach ($line in Get-Content -LiteralPath $checksumFile) {
            if ($line -notmatch '^([0-9a-fA-F]{64})\s+(.+)$') { continue }
            $expected = $matches[1].ToLowerInvariant()
            $name = $matches[2].Trim()
            # A manifest cannot contain its own hash; older builds wrote a stale
            # self-entry, so it is ignored here.
            if ($name -eq 'CHECKSUMS.txt') { continue }
            $target = Join-Path $source $name
            if (-not (Test-Path -LiteralPath $target)) { continue }
            $actual = (Get-FileHash -Algorithm SHA256 -LiteralPath $target).Hash.ToLowerInvariant()
            if ($actual -ne $expected) { $mismatches += $name }
        }
        if ($mismatches.Count -gt 0) {
            Stop-Install 'E_PACKAGE_CORRUPT' `
                ("These package files do not match CHECKSUMS.txt: " + ($mismatches -join ', ') + '.') `
                'Re-download or rebuild the package. Use -SkipIntegrityCheck only for a build you produced yourself.'
        }
        $integrity = 'verified'
        Write-Step 'package checksums verified'
    }
} else {
    Write-Warning 'CHECKSUMS.txt is missing from the package; integrity was not verified.'
}

# Version metadata (BUILD.json is written by tools\build-desktop.ps1).
$packageVersion = $null
$packageCommit = $null
$buildFile = Join-Path $source 'BUILD.json'
if (Test-Path -LiteralPath $buildFile) {
    try {
        $build = Get-Content -LiteralPath $buildFile -Raw | ConvertFrom-Json
        $packageVersion = [string]$build.version
        $packageCommit = [string]$build.commit
    } catch {
        Write-Warning "BUILD.json could not be parsed: $($_.Exception.Message)"
    }
}
if ([string]::IsNullOrWhiteSpace($packageVersion)) {
    $packageVersion = '0.0.0-unknown'
    Write-Warning 'The package has no version metadata; upgrade/downgrade checks are limited.'
}

function Compare-AppVersion([string]$Left, [string]$Right) {
    # Semver-ish comparison that tolerates the "-dev" suffix used by this repo:
    # the numeric core decides first; with an equal core a version WITH a
    # prerelease suffix is older than one without.
    $leftCore = $Left
    $leftPre = ''
    $leftDash = $Left.IndexOf('-')
    if ($leftDash -ge 0) {
        $leftCore = $Left.Substring(0, $leftDash)
        $leftPre = $Left.Substring($leftDash + 1)
    }
    $rightCore = $Right
    $rightPre = ''
    $rightDash = $Right.IndexOf('-')
    if ($rightDash -ge 0) {
        $rightCore = $Right.Substring(0, $rightDash)
        $rightPre = $Right.Substring($rightDash + 1)
    }
    $leftVersion = $null
    $rightVersion = $null
    $parsed = [version]::TryParse($leftCore, [ref]$leftVersion)
    if (-not $parsed) { return [string]::Compare($Left, $Right, [StringComparison]::Ordinal) }
    $parsed = [version]::TryParse($rightCore, [ref]$rightVersion)
    if (-not $parsed) { return [string]::Compare($Left, $Right, [StringComparison]::Ordinal) }
    $core = $leftVersion.CompareTo($rightVersion)
    if ($core -ne 0) { return $core }
    if ($leftPre -eq $rightPre) { return 0 }
    if ($leftPre -eq '') { return 1 }
    if ($rightPre -eq '') { return -1 }
    return [string]::Compare($leftPre, $rightPre, [StringComparison]::Ordinal)
}

# Existing installation: upgrade in place, refuse an unintended downgrade.
$installRecord = Join-Path $InstallRoot 'install.json'
$existingVersion = $null
if (Test-Path -LiteralPath $installRecord) {
    try {
        $existing = Get-Content -LiteralPath $installRecord -Raw | ConvertFrom-Json
        $existingVersion = [string]$existing.version
    } catch {
        Write-Warning "The existing install.json could not be parsed: $($_.Exception.Message)"
    }
}
$isUpgrade = $false
if (-not [string]::IsNullOrWhiteSpace($existingVersion)) {
    $comparison = Compare-AppVersion $packageVersion $existingVersion
    if ($comparison -lt 0 -and -not $Downgrade) {
        Stop-Install 'E_VERSION_DOWNGRADE' `
            "The package is version $packageVersion but $existingVersion is already installed." `
            'Install the newer package, or pass -Downgrade to replace it on purpose.'
    }
    $isUpgrade = $true
    Write-Step "existing installation $existingVersion -> $packageVersion"
}

$installRootFiles = @(Get-ChildItem -LiteralPath $InstallRoot -Force -ErrorAction SilentlyContinue)
if ($installRootFiles.Count -gt 0 -and -not $Force -and -not $isUpgrade) {
    Stop-Install 'E_INSTALL_EXISTS' `
        "The install root already exists and has files: $InstallRoot" `
        'Use -Force to replace the app files (user data in the data directory is never touched).'
}

# Write access + free space. Create the directories here so a permission refusal
# happens before any file is copied.
foreach ($dir in @($InstallRoot, $DataRoot, $WorkspaceRoot)) {
    try {
        New-Item -ItemType Directory -Force -Path $dir | Out-Null
        $probe = Join-Path $dir ('.hivemind-write-probe-' + [guid]::NewGuid().ToString('N'))
        [IO.File]::WriteAllText($probe, 'ok')
        Remove-Item -LiteralPath $probe -Force
    } catch {
        Stop-Install 'E_PERMISSION_DENIED' "Cannot write to $dir : $($_.Exception.Message)" 'Choose a writable -InstallRoot/-DataRoot, or fix the folder permissions.'
    }
}
$freeBytes = $null
$driveName = $null
try {
    $drive = New-Object IO.DriveInfo([IO.Path]::GetPathRoot($DataRoot))
    $freeBytes = $drive.AvailableFreeSpace
    $driveName = $drive.Name
} catch {
    Write-Verbose "Free-space check unavailable: $($_.Exception.Message)"
}
if ($null -ne $freeBytes -and $freeBytes -lt 250MB) {
    Stop-Install 'E_DISK_FULL' ("Only " + [math]::Round($freeBytes / 1MB) + ' MB is free on ' + $driveName + '.') 'Free at least 250 MB and run the installer again.'
}

# ---------------------------------------------------------------------------
# 2. Copy the application files.
# ---------------------------------------------------------------------------
Write-Step "installing to $InstallRoot"
Copy-Item -LiteralPath $launcher -Destination $InstallRoot -Force
Copy-Item -LiteralPath $server -Destination $InstallRoot -Force
foreach ($extra in @('README.txt', 'set-provider-key.ps1', 'set-provider-profile.ps1', 'CHECKSUMS.txt', 'BUILD.json', 'install-desktop.ps1', 'uninstall-desktop.ps1')) {
    $candidate = Join-Path $repoRoot "tools\$extra"
    if (-not (Test-Path -LiteralPath $candidate)) { $candidate = Join-Path $source $extra }
    if (Test-Path -LiteralPath $candidate) {
        Copy-Item -LiteralPath $candidate -Destination (Join-Path $InstallRoot $extra) -Force
    }
}

# Shortcuts: Desktop and Start Menu, current user only.
function New-Shortcut([string]$Path) {
    $dir = Split-Path -Parent $Path
    New-Item -ItemType Directory -Force -Path $dir | Out-Null
    $shell = New-Object -ComObject WScript.Shell
    $shortcut = $shell.CreateShortcut($Path)
    $shortcut.TargetPath = Join-Path $InstallRoot 'HiveMind.exe'
    $shortcut.WorkingDirectory = $InstallRoot
    $shortcut.Description = 'HiveMind local HMARS'
    $shortcut.IconLocation = (Join-Path $InstallRoot 'HiveMind.exe') + ',0'
    $shortcut.Save()
}
if (-not $NoShortcuts) {
    New-Shortcut (Join-Path ([Environment]::GetFolderPath('Desktop')) 'HiveMind.lnk')
    New-Shortcut (Join-Path ([Environment]::GetFolderPath('StartMenu')) 'Programs\HiveMind\HiveMind.lnk')
    Write-Host 'Shortcuts: Desktop\HiveMind.lnk and Start Menu\Programs\HiveMind\HiveMind.lnk' -ForegroundColor Green
} else {
    Write-Host 'Shortcuts skipped (-NoShortcuts).' -ForegroundColor Yellow
}

# Add/Remove Programs entry, so the uninstall path is discoverable.
$uninstallKey = 'HKCU:\Software\Microsoft\Windows\CurrentVersion\Uninstall\HiveMind'
New-Item -Path $uninstallKey -Force | Out-Null
$uninstallScript = Join-Path $InstallRoot 'uninstall-desktop.ps1'
if (Test-Path -LiteralPath $uninstallScript) {
    $uninstallCommand = 'powershell.exe -NoProfile -ExecutionPolicy Bypass -File "' + $uninstallScript + '"'
} else {
    $uninstallCommand = ''
}
$appSizeKb = 0
Get-ChildItem -LiteralPath $InstallRoot -File -Recurse -ErrorAction SilentlyContinue | ForEach-Object { $appSizeKb += [int][math]::Ceiling($_.Length / 1KB) }
$registryValues = @{
    DisplayName = 'HiveMind'
    DisplayVersion = $packageVersion
    Publisher = 'HiveMind'
    InstallLocation = $InstallRoot
    DisplayIcon = (Join-Path $InstallRoot 'HiveMind.exe')
    NoModify = 1
    NoRepair = 1
    EstimatedSize = $appSizeKb
}
if (-not [string]::IsNullOrWhiteSpace($uninstallCommand)) {
    $registryValues.UninstallString = $uninstallCommand
    $registryValues.QuietUninstallString = $uninstallCommand
}
foreach ($name in $registryValues.Keys) {
    $propertyType = 'String'
    if ($registryValues[$name] -is [int]) { $propertyType = 'DWord' }
    New-ItemProperty -Path $uninstallKey -Name $name -Value $registryValues[$name] -PropertyType $propertyType -Force | Out-Null
}

if (-not $SkipProvider) {
    $providerProfile = @{
        name = $Provider
        api_type = $(if ($Provider -eq 'deepseek') { 'deepseek' } else { 'tokenlabs' })
        model = 'deepseek-flash'
    }
    if ($Provider -eq 'tokenlab') {
        if ([string]::IsNullOrWhiteSpace($BaseUrl)) { throw 'TokenLab requires -BaseUrl.' }
        if ([string]::IsNullOrWhiteSpace($WireModel)) { throw 'TokenLab requires -WireModel.' }
        $providerProfile.base_url = $BaseUrl.TrimEnd('/')
        $providerProfile.wire_model = $WireModel
    } elseif ($Provider -eq 'custom') {
        if ([string]::IsNullOrWhiteSpace($BaseUrl) -or [string]::IsNullOrWhiteSpace($WireModel)) {
            throw 'Custom provider requires -BaseUrl and -WireModel.'
        }
        $providerProfile.base_url = $BaseUrl.TrimEnd('/')
        $providerProfile.wire_model = $WireModel
    }
    if ($Provider -ne 'none') {
        $json = $providerProfile | ConvertTo-Json -Depth 4
        [IO.File]::WriteAllText((Join-Path $DataRoot 'provider.json'), $json, (New-Object System.Text.UTF8Encoding($false)))
        $secure = Read-Host -AsSecureString 'Provider API key (input hidden; Enter to skip)'
        $bstr = [IntPtr]::Zero
        try {
            $bstr = [Runtime.InteropServices.Marshal]::SecureStringToBSTR($secure)
            $plain = [Runtime.InteropServices.Marshal]::PtrToStringBSTR($bstr)
            if (-not [string]::IsNullOrWhiteSpace($plain)) {
                Add-Type -AssemblyName System.Security
                $entropy = [Text.Encoding]::UTF8.GetBytes('HiveMind.ProviderKey.v1')
                $bytes = [Text.Encoding]::UTF8.GetBytes($plain.Trim())
                $cipher = [Security.Cryptography.ProtectedData]::Protect(
                    $bytes,
                    $entropy,
                    [Security.Cryptography.DataProtectionScope]::CurrentUser
                )
                [IO.File]::WriteAllBytes((Join-Path $DataRoot 'provider.key'), $cipher)
                [Array]::Clear($bytes, 0, $bytes.Length)
            }
        } finally {
            if ($bstr -ne [IntPtr]::Zero) { [Runtime.InteropServices.Marshal]::ZeroFreeBSTR($bstr) }
            $plain = $null
        }
    }
}

[IO.File]::WriteAllText(
    $installRecord,
    (@{
        version = $packageVersion
        commit = $packageCommit
        installed_at = (Get-Date).ToString('o')
        install_root = $InstallRoot
        data_root = $DataRoot
        workspace_root = $WorkspaceRoot
        source = $source
        integrity = $integrity
        upgrade = $isUpgrade
    } | ConvertTo-Json -Depth 3),
    (New-Object System.Text.UTF8Encoding($false))
)

Write-Host "Installed HiveMind $packageVersion to $InstallRoot" -ForegroundColor Green
Write-Host "Data: $DataRoot" -ForegroundColor Green
Write-Host "Workspace: $WorkspaceRoot" -ForegroundColor Green

if (-not $NoLaunch) {
    Start-Process -FilePath (Join-Path $InstallRoot 'HiveMind.exe') -WorkingDirectory $InstallRoot | Out-Null
    # The launcher picks the first free port in 8099..8199 and records it, so the
    # health probe never assumes 8099.
    $runtimeFile = Join-Path $DataRoot 'runtime.json'
    $port = 0
    $deadline = (Get-Date).AddSeconds(30)
    while ((Get-Date) -lt $deadline) {
        if (Test-Path -LiteralPath $runtimeFile) {
            try {
                $runtime = Get-Content -LiteralPath $runtimeFile -Raw | ConvertFrom-Json
                if ($null -ne $runtime.port) { $port = [int]$runtime.port }
            } catch {
                $port = 0
            }
            if ($port -gt 0) { break }
        }
        Start-Sleep -Milliseconds 250
    }
    if ($port -eq 0) {
        # Fall back to a scan, matching the launcher's port range.
        for ($candidate = 8099; $candidate -le 8199 -and $port -eq 0; $candidate++) {
            try {
                $probe = Invoke-WebRequest -Uri "http://127.0.0.1:$candidate/health" -UseBasicParsing -TimeoutSec 1
                if ($probe.Content -like '*"schema":"hiverouter*') { $port = $candidate }
            } catch {
                # No HiveMind server on this port.
            }
        }
    }
    if ($port -eq 0) {
        Write-Warning "HiveMind did not report a health endpoint. Launch it from the shortcut and inspect $DataRoot."
    } else {
        Write-Host "HiveMind is running at http://127.0.0.1:$port" -ForegroundColor Green
    }
}
