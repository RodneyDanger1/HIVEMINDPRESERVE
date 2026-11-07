# HiveMind desktop launcher (no Tauri / no Visual Studio)

This is the low-friction Windows desktop path chosen after the Tauri toolchain was blocked by free disk space.

- `HiveMindLauncher.cs` is a 7 KB WinForms launcher compiled with the .NET Framework `csc` that ships with Windows.
- It starts the existing `target\release\hiverouter.exe` on `127.0.0.1:8099` if nothing is listening, waits for `/health`, then opens the **same web UI** in an Edge WebView2 app window (`--app=http://127.0.0.1:8099/ --window-size=1440,900`).
- It never bundles or uses a second UI build. The Rust server still embeds `web/dist`.
- It uses `%LOCALAPPDATA%\HiveMind` for server data and the WebView2 profile.
- When it owns the server process, closing the app window stops the server. If a server is already running, the launcher only opens the window.

## Compile

```powershell
& 'C:\Windows\Microsoft.NET\Framework64\v4.0.30319\csc.exe' /nologo /target:winexe /optimize+ `
  /out:tools\desktop-launcher\HiveMind.exe /r:System.Windows.Forms.dll `
  tools\desktop-launcher\HiveMindLauncher.cs
```

The compiled launcher is copied to `C:\Users\Riley\Desktop\HiveMind.exe`.

## Why not Tauri now

Tauri v2 needs the MSVC C++ Build Tools plus Windows SDK (multiple GB). The install failed with Windows error `0x80070070` because the C: drive had only **1.99 GB free**. WebView2 runtime is already present.

Tauri remains the eventual native-shell option; this launcher is the working substitute today.
