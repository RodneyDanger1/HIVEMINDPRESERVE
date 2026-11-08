# HiveMind desktop launcher (no Tauri / no Visual Studio)

This is the low-friction Windows desktop path chosen after the Tauri toolchain was blocked by free disk space.

- `HiveMindLauncher.cs` is a small WinForms launcher compiled with the .NET Framework `csc` that ships with Windows.
- It starts the existing `hiverouter.exe` on `127.0.0.1:8099` if nothing is listening, waits for `/health`, then opens the **same web UI** in an Edge WebView2 app window (`--app=http://127.0.0.1:8099/ --window-size=1440,900`).
- It never bundles or uses a second UI build. The Rust server still embeds `web/dist`.
- It uses `%LOCALAPPDATA%\HiveMind` for server data and the WebView2 profile.
- When it owns the server process, closing the app window stops the server. If a server is already running, the launcher only opens the window.
- It injects `DEEPSEEK_API_KEY` into the server child from a **DPAPI-encrypted** `%LOCALAPPDATA%\HiveMind\provider.key`. The key is never in source, a command line, a log, or browser JavaScript.

## Provider key

Create or rotate the encrypted key:

```powershell
powershell -ExecutionPolicy Bypass -File tools\set-provider-key.ps1
```

The script prompts with hidden input. It does not echo the key.

## Compile

```powershell
& 'C:\Windows\Microsoft.NET\Framework64\v4.0.30319\csc.exe' /nologo /target:winexe /optimize+ `
  /out:tools\desktop-launcher\HiveMind.exe /r:System.Windows.Forms.dll /r:System.Security.dll `
  tools\desktop-launcher\HiveMindLauncher.cs
```

The compiled launcher is copied to `C:\Users\Riley\Desktop\HiveMind.exe`.

## Why not Tauri now

Tauri v2 needs the MSVC C++ Build Tools plus Windows SDK (multiple GB). The install failed with Windows error `0x80070070` because the C: drive had only **1.99 GB free**. WebView2 runtime is already present.

Tauri remains the eventual native-shell option; this launcher is the working substitute today.
