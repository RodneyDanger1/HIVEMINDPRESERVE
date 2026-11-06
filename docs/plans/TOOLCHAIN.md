# TOOLCHAIN — verified evidence (2026-11-06)

## Rust

| Item | Value | Evidence |
|---|---|---|
| Rust | stable-x86_64-pc-windows-gnu, rustc/cargo 1.99.1 | `rustc --version` → `rustc 1.99.1 (58cf8f9c3 2026-10-13)`; `cargo --version` → `cargo 1.99.1` |
| Install | per-user, no admin, `%USERPROFILE%\.cargo` + `.rustup` | `rustup-init -y --profile minimal --default-toolchain stable-x86_64-pc-windows-gnu --no-modify-path` exit 0 |
| MSRV | installed 1.99.1 >= blueprint MSRV 1.89.0 | OK |

## C compiler / linker

| Item | Value | Evidence |
|---|---|---|
| GCC | 16.2.0 at `tools\w64devkit\bin\gcc.exe` | `gcc --version` |
| ar | `tools\w64devkit\bin\ar.exe` | present |
| C compile | exit 0, binary ran (`gcc-c-ok`) | hello-world compile + execute |
| Rust link | exit 0, binary ran (`rust-gcc-link-ok`) | `rustc hello.rs --target x86_64-pc-windows-gnu -C linker=...gcc.exe` |
| Cargo config | `.cargo/config.toml` pins linker + `CC_x86_64_pc_windows_gnu` / `AR_...` | file added |

## Provenance / security note

- `w64devkit` is the official portable MinGW-w64 distribution from `github.com/skeeto/w64devkit`
  (README/COPYING/VERSION present). Windows Defender previously flagged a downloaded w64devkit zip as
  `Trojan:Win32/Vigorf.A` (ThreatID 2147714384) — this is a known false positive for the bundled
  busybox/binutils (`skeeto/w64devkit` issue #185 and Microsoft Q&A reports). The extracted GCC runs and
  compiles only local hello-world code; it is the only C toolchain on the host.
- A WinLibs download was also attempted (official `brechtsanders/winlibs_mingw` release, sha256
  `3c693050…6b6a1`) but the transfer was incomplete (135,516,600 / 276,414,382 bytes, digest mismatch),
  so it was deleted and never extracted.
- Zig 0.17.1 was downloaded from official `ziglang.org` with matching sha256
  `b03f477a…562e`; it hung (>90 s) on a hello-world compile on this host, so it was deleted.
- No untrusted binary is on `PATH`; compiler is referenced by absolute path in `.cargo/config.toml`.
