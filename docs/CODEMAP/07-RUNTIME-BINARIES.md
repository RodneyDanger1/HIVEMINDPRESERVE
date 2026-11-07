# 07 — Runtime, Artifacts, Binaries, and Assets

## Tracked binaries/assets

| Family | Count | Tracked bytes | Treatment |
|---|---:|---:|---|
| `.exe` | 10 | 58.1 MB | PE executable; producer/build commit, hash, version, consumer; three `hiverouter.exe` copies are byte-identical |
| `.zip` | 3 | 17.1 MB | Release archives; member manifest + hashes; sealed 1.0/1.1 only |
| `.db` / `.db-wal` / `.db-shm` | 13 each | 2.05 MB | SQLite runtime/fixture residue; schema from `db.rs`, not payload dumps; redaction/reset semantics |
| `.figcraft` | 18 | 1.79 MB | ZIP design build artifacts (`document.json` + `thumbnail.png`); ignored canonical sources are separate |
| `.png` | 20 | 1.49 MB | UI/evidence images; dimensions/IHDR, producer, consumer |
| `.pyc` | 6 | 0.12 MB | CPython 3.14 bytecode under `__pycache__` despite ignore; tracked generated residue |
| `.svg` | 11 | 0.58 MB | Vector assets; XML parse, dimensions, source |
| `.out` / `.err` | 47 | 42 KB | Command evidence streams; plain text despite "binary" grouping; 22 `.err` are empty |
| `.log` | 1 | 1.6 KB | tracked exception (`docs/evidence/live-1.1-paid.log`) |

## Ignored runtime/dist

- Root `target/` 17,244 files/15.78 GB; `backend/target/` 4,633/2.35 GB — Cargo debug/release outputs, `.rlib/.rmeta/.o/.d`, test binaries.
- `web/node_modules/` 4,067, `e2e/node_modules/` 194 — dependencies.
- `tools/w64devkit/` 9,452 — vendored MinGW toolchain (gcc/dlltool); required to compile on this host.
- `web/dist/` 3 files — Vite bundle embedded into the server binary (index/CSS/JS); build input, not arbitrary generated output.
- `dist/` ignored 15 — release archives, hidden manifests, unpacked dirs; 1.0/1.1 only.
- `docs/` ignored 54 — 29 evidence logs, 7 audit logs, 18 FigCraft canonical sources/assets.
- `e2e` generated 4 — test-results/playwright-report.
- `runtime-test/` 82 tracked — SQLite/WAL fixtures, packaged binaries, workspace fixtures; privacy/redaction risk because DB/WAL can contain prompts/approvals.

## Provenance and risk

- Tracked binaries are 86% of tracked bytes (≈77 MiB of 85.6 MiB).
- Three identical `hiverouter.exe` copies and 13 identical 4 KB DBs are redundancy; hashes group them.
- `.pyc` tracked despite `__pycache__/` ignore; source files exist, but staleness should be checked.
- No first-party LICENSE/NOTICE; dependency licenses are inside node_modules/w64devkit, not redistributed in a clean clone.
- No `.gitattributes`; CRLF/LF identity caveat applies to text artifacts.
