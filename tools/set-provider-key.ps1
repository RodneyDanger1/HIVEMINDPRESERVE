# Store the DeepSeek API key for the desktop app.
#
# The key is read as a SecureString prompt, encrypted with Windows DPAPI for the
# current user, and written to:
#   %LOCALAPPDATA%\HiveMind\provider.key
#
# HiveMind.exe decrypts this file and injects DEEPSEEK_API_KEY into the
# hiverouter.exe child only. The key is never written to source, the repo, a
# command line, a log, or stdout.
#
# Run:
#   powershell -ExecutionPolicy Bypass -File tools\set-provider-key.ps1
#
# Rotate by running it again with the new key. Delete the key file and no key is
# supplied (the app will honestly refuse prompts with E_KEY_MISSING).

$ErrorActionPreference = 'Stop'
Add-Type -AssemblyName System.Security

$secure = Read-Host -AsSecureString 'DeepSeek API key (input hidden)'
$bstr = [IntPtr]::Zero
try {
    $bstr = [Runtime.InteropServices.Marshal]::SecureStringToBSTR($secure)
    $plain = [Runtime.InteropServices.Marshal]::PtrToStringBSTR($bstr)
    if ([string]::IsNullOrWhiteSpace($plain)) {
        throw 'The key cannot be empty.'
    }
    $entropy = [Text.Encoding]::UTF8.GetBytes('HiveMind.ProviderKey.v1')
    $bytes = [Text.Encoding]::UTF8.GetBytes($plain.Trim())
    $cipher = [Security.Cryptography.ProtectedData]::Protect(
        $bytes,
        $entropy,
        [Security.Cryptography.DataProtectionScope]::CurrentUser
    )
    $dir = Join-Path $env:LOCALAPPDATA 'HiveMind'
    New-Item -ItemType Directory -Force -Path $dir | Out-Null
    $file = Join-Path $dir 'provider.key'
    [IO.File]::WriteAllBytes($file, $cipher)
    Write-Host "Encrypted provider key stored at $file" -ForegroundColor Green
} finally {
    if ($bstr -ne [IntPtr]::Zero) {
        [Runtime.InteropServices.Marshal]::ZeroFreeBSTR($bstr)
    }
    if ($null -ne $plain) {
        $plain = $null
    }
    if ($null -ne $bytes) {
        [Array]::Clear($bytes, 0, $bytes.Length)
    }
}
