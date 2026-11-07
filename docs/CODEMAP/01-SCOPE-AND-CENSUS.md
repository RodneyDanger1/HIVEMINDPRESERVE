# 01 — Scope and Census

## Freeze

- Commit: `e021b12e5bb7e5334c71012bcf7a8810d91b9e58`
- `git status --porcelain`: `?? docs/CODEMAP/`
- Source version: `1.2.0-dev` (`web/package.json`, four crate manifests)
- Census generated from a filesystem walk excluding `.git` and excluding the new `docs/CODEMAP/` output, so the numbers describe the repository **before** this encyclopedia was added.
- `.git` is counted separately (226 files) and explained as VCS state, not product source.

## Exact partition

| Layer | Files | Bytes |
|---|---:|---:|
| Tracked | 805 | 85.61 MiB |
| Ignored | 35,675 | 18.60 GiB |
| Working tree | 36,480 | 18.68 GiB |

## Per-directory census

| Area | Files | Tracked | Ignored | Bytes |
|---|---:|---:|---:|---:|
| `target` | 17,244 | 0 | 17,244 | 15.41 GiB |
| `tools` | 9,475 | 23 | 9,452 | 735.79 MiB |
| `backend` | 4,754 | 112 | 4,642 | 2.30 GiB |
| `web` | 4,141 | 71 | 4,070 | 117.68 MiB |
| `docs` | 314 | 260 | 54 | 33.02 MiB |
| `e2e` | 237 | 39 | 198 | 19.60 MiB |
| `plans` | 114 | 114 | 0 | 1.60 MiB |
| `audit` | 93 | 93 | 0 | 2.37 MiB |
| `runtime-test` | 82 | 82 | 0 | 33.60 MiB |
| `dist` | 19 | 4 | 15 | 54.69 MiB |
| `(root)` | 4 | 4 | 0 | 6.92 KiB |
| `ledgers` | 2 | 2 | 0 | 14.62 KiB |
| `.cargo` | 1 | 1 | 0 | 229 B |
| **Total** | **36,480** | **805** | **35,675** | **18.68 GiB** |

Notable: `backend/` is 4,754 total because `backend/target/` is a second Cargo target directory (4,633 ignored build files) in addition to the root `target/` (17,244). `tools/` is 9,475 because it contains the ignored `tools/w64devkit/` compiler toolchain (9,452 files), not 9,475 authored tools.

## Extension census — tracked

| Extension | Files | Tracked | Ignored | Bytes |
|---|---:|---:|---:|---:|
| `.md` | 358 | 219 | 139 | 3.60 MiB |
| `.json` | 3,361 | 109 | 3,252 | 6.34 MiB |
| `.rs` | 145 | 104 | 41 | 4.29 MiB |
| `.ts` | 1,078 | 68 | 1,010 | 11.05 MiB |
| `.py` | 46 | 43 | 3 | 450.70 KiB |
| `.out` | 24 | 24 | 0 | 40.51 KiB |
| `.err` | 23 | 23 | 0 | 726 B |
| `.tsx` | 22 | 22 | 0 | 91.29 KiB |
| `.png` | 42 | 20 | 22 | 9.88 MiB |
| `.txt` | 228 | 19 | 209 | 9.57 MiB |
| `.figcraft` | 20 | 18 | 2 | 2.05 MiB |
| `.mjs` | 604 | 17 | 587 | 3.56 MiB |
| `.csv` | 13 | 13 | 0 | 347.90 KiB |
| `.db` | 13 | 13 | 0 | 52.00 KiB |
| `.db-shm` | 13 | 13 | 0 | 416.00 KiB |
| `.db-wal` | 13 | 13 | 0 | 1.53 MiB |
| `.mmd` | 12 | 12 | 0 | 16.40 KiB |
| `.svg` | 26 | 11 | 15 | 12.90 MiB |
| `.exe` | 960 | 10 | 950 | 11.92 GiB |
| `.toml` | 7 | 7 | 0 | 5.57 KiB |
| `.pyc` | 6 | 6 | 0 | 114.97 KiB |
| `.ps1` | 20 | 4 | 16 | 105.62 KiB |
| `.zip` | 4 | 3 | 1 | 22.55 MiB |
| `(none)` | 7,051 | 2 | 7,049 | 13.02 MiB |
| `.c` | 54 | 2 | 52 | 817.09 KiB |
| `.css` | 19 | 2 | 17 | 628.18 KiB |
| `.html` | 14 | 2 | 12 | 707.87 KiB |
| `.sha256` | 2 | 2 | 0 | 131 B |
| `.log` | 46 | 1 | 45 | 1.72 MiB |
| `.lock` | 1 | 1 | 0 | 97.83 KiB |

## Extension census — all files

| Extension | Files | Tracked | Ignored | Bytes |
|---|---:|---:|---:|---:|
| `(none)` | 7,051 | 2 | 7,049 | 13.02 MiB |
| `.json` | 3,361 | 109 | 3,252 | 6.34 MiB |
| `.timestamp` | 3,088 | 0 | 3,088 | 144.75 KiB |
| `.d` | 2,946 | 0 | 2,946 | 15.75 MiB |
| `.h` | 2,515 | 0 | 2,515 | 98.78 MiB |
| `.rmeta` | 2,366 | 0 | 2,366 | 2.95 GiB |
| `.cmake` | 1,525 | 0 | 1,525 | 4.87 MiB |
| `.a` | 1,519 | 0 | 1,519 | 317.04 MiB |
| `.vim` | 1,516 | 0 | 1,516 | 10.76 MiB |
| `.rlib` | 1,493 | 0 | 1,493 | 2.76 GiB |
| `.map` | 1,118 | 0 | 1,118 | 10.09 MiB |
| `.ts` | 1,078 | 68 | 1,010 | 11.05 MiB |
| `.o` | 1,021 | 0 | 1,021 | 33.39 MiB |
| `.exe` | 960 | 10 | 950 | 11.92 GiB |
| `.js` | 745 | 0 | 745 | 38.70 MiB |
| `.mjs` | 604 | 17 | 587 | 3.56 MiB |
| `.md` | 358 | 219 | 139 | 3.60 MiB |
| `.idl` | 339 | 0 | 339 | 10.68 MiB |
| `.1` | 299 | 0 | 299 | 4.52 MiB |
| `.hpp` | 243 | 0 | 243 | 1.06 MiB |
| `.txt` | 228 | 19 | 209 | 9.57 MiB |
| `.rs` | 145 | 104 | 41 | 4.29 MiB |
| `.in` | 122 | 0 | 122 | 394.86 KiB |
| `.dll` | 115 | 0 | 115 | 360.27 MiB |
| `.nsh` | 112 | 0 | 112 | 1.13 MiB |
| `.mts` | 96 | 0 | 96 | 540.13 KiB |
| `.def` | 90 | 0 | 90 | 118.99 KiB |
| `.lib` | 90 | 0 | 90 | 4.58 MiB |
| `.obj` | 78 | 0 | 78 | 1.02 MiB |
| `.nlf` | 67 | 0 | 67 | 402.95 KiB |

## Largest files

| Path | Bytes | Tracked |
|---|---:|---|
| `target\release\deps\libwindows-f5aa2ce57b50dcc4.rlib` | 130,326,680 | no |
| `target\release\deps\rustcDaT8qo\lib.rmeta` | 121,869,903 | no |
| `target\release\deps\libwindows-bbd8c36c9970abac.rmeta` | 121,869,821 | no |
| `target\release\deps\libwindows-f5aa2ce57b50dcc4.rmeta` | 121,711,079 | no |
| `backend\target\debug\deps\libwindows-e73ba2882c19977b.rlib` | 118,642,158 | no |
| `target\debug\deps\libwindows-e73ba2882c19977b.rlib` | 118,590,774 | no |
| `backend\target\debug\deps\libwindows-e73ba2882c19977b.rmeta` | 109,295,438 | no |
| `target\debug\deps\libwindows-e73ba2882c19977b.rmeta` | 109,295,438 | no |
| `backend\target\debug\deps\libwindows-5d53e323556248be.rmeta` | 84,937,557 | no |
| `target\debug\deps\libwindows-5d53e323556248be.rmeta` | 84,937,557 | no |
| `target\debug\deps\tool_loop_matrix-1365363346e2c33c.exe` | 47,139,810 | no |
| `target\debug\deps\tool_loop_matrix-6a5554c67ee5ddc6.exe` | 47,138,195 | no |
| `backend\target\debug\deps\tool_loop_matrix-6a5554c67ee5ddc6.exe` | 47,130,537 | no |
| `target\debug\deps\provider_streaming-39026aa1c2fed08c.exe` | 47,064,189 | no |
| `target\debug\deps\provider_streaming-38103349d86ea22d.exe` | 47,059,502 | no |
| `backend\target\debug\deps\provider_streaming-38103349d86ea22d.exe` | 47,052,936 | no |
| `target\debug\deps\provider_adapter-7e7bc9e041272be2.exe` | 46,997,870 | no |
| `target\debug\deps\provider_adapter-7c2072a6badc104f.exe` | 46,994,553 | no |
| `backend\target\debug\deps\provider_adapter-7c2072a6badc104f.exe` | 46,987,919 | no |
| `target\debug\deps\ws_transport-73ff99a05ffb5a93.exe` | 46,963,420 | no |
| `target\debug\deps\ws_transport-85e6529add42bae7.exe` | 46,957,949 | no |
| `backend\target\debug\deps\ws_transport-85e6529add42bae7.exe` | 46,951,315 | no |
| `target\debug\deps\approval_ledger-e4d25dea2a0cf0b7.exe` | 46,941,555 | no |
| `target\debug\deps\approval_ledger-448e5014c44068ef.exe` | 46,938,040 | no |
| `backend\target\debug\deps\approval_ledger-448e5014c44068ef.exe` | 46,931,406 | no |

## Text vs binary rule

There are two honest rules and they disagree:

- **Git content rule:** 687 tracked files contain no NUL in the first 8,000 bytes → Git treats them as text; 118 as binary.
- **Format rule used by the catalog:** 662 “text-like” tracked files (`.rs/.ts/.tsx/.js/.mjs/.py/.ps1/.json/.toml/.md/.css/.html/.svg/.mmd/.prompty/.txt/.csv/.sha256/.log/.jsonl/.lock/.c/.gitignore`) and 143 “binary/asset” files.

The difference is **33 files**: four NUL-containing `.txt/.json` files are misclassified as text, and 29 plain-text `.out/.err` files are misclassified as binary by the format rule. The catalog uses the format rule for grouping but records the actual encoding per file; `.out/.err` evidence is treated as text evidence. Lockfiles are generated, not authored prose.

## Encoding and line endings

Tracked encoding census:

| Encoding | Files | Bytes |
|---|---:|---:|
| UTF-8 | 655 | 8,751,138 |
| NUL-containing | 96 | 80,718,456 |
| UTF-8 BOM | 32 | 236,091 |
| UTF-16LE BOM | 22 | 65,626 |

`core.autocrlf=true`; there is no `.gitattributes`. Therefore:

- Working-tree bytes and Git blob bytes differ for text files (CRLF vs LF); the tracked-side SHA-256 in this catalog hashes **working-tree bytes**.
- Do not compare it directly with GitHub blob IDs. For commit-stable identity, use `git hash-object`/`git ls-tree`; for content identity on this checkout, use the recorded SHA-256.
- 87 tracked files are CRLF, 24 mixed, 551 LF, 12 no trailing newline (auditor recount); the catalog records encoding, not a normalized line count.
- “177,861 lines” from the first pass = PowerShell non-empty lines; physical newlines ≈189,444. This encyclopedia does not use a single line-count claim.

## Ignored partition (35,675)

| Family | Files | Bytes | Why |
|---|---:|---:|---|
| Root `target/` | 17,244 | 15.78 GB | Cargo build output |
| `backend/target/` | 4,633 | 2.35 GB | second Cargo target dir |
| `tools/w64devkit/` | 9,452 | 716.7 MB | vendored MinGW toolchain |
| `web/node_modules/` | 4,067 | 116.5 MB | npm deps |
| `e2e/node_modules/` | 194 | 18.2 MB | npm deps |
| `web/dist/` | 3 | 0.7 MB | Vite build embedded by rust-embed |
| `dist/` ignored | 15 | ~38.7 MB | release archives/manifests |
| `docs/` ignored | 54 | ~7.4 MB | evidence logs + FigCraft sources/assets |
| `e2e` reports | 4 | ~1.2 MB | Playwright results/report |
| backend logs | 9 | small | command logs |
| **Sum** | **35,675** | **19.97 GB** | exact |

## Depth labels

| Label | Meaning |
|---|---|
| `DEEP` | Architecture-critical file read and explained in the area docs |
| `INDEXED` | Every path is in the catalog with identity/encoding/role; semantics covered by its area doc |
| `STRUCTURAL` | Binary/asset: format, hash, size, provenance, consumer; not line-by-line |
| `FAMILY` | Generated/vendor file: exact path + family row; not semantically re-explained |

No file is omitted from `_data/inventory.csv`.
