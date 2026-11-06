# SEAT-05 (Release/KeyOps) — EVIDENCE.md

Read-only evidence log. Every command below was executed in this session; outputs quoted or summarised
verbatim. No builds, no installs, no network fetches except the four official DeepSeek doc pages; no key
access; no paid calls; no product/git/config writes.

## E1 — Baseline (HEAD, tag, tree state)

```
git -C <repo> rev-parse HEAD            -> 5c69b66e363e9f1d0801ea5c2e66a1fc12c8943c
git -C <repo> describe --tags --always  -> v1.0.0-3-g5c69b66
git -C <repo> rev-list -n1 v1.0.0       -> 3988622d6ef65ff39a0c8bb126862a8e8405ffd9
git -C <repo> log --oneline -4
  -> 5c69b66 feat: wire DeepSeek provider into core; live smoke, replay arm, 16-state parity, settings
     f4c780d audit: close verifier doc-sync notes (LOC breakdown, roadmap wording already 101)
     cf70b22 audit: 507-file exhaustive audit -- paper, 30 findings, quality matrix, 6 diagrams, 101-task roadmap, verification
     3988622 closeout: sync FINAL-REPORT with packaged CLI + zip hash; process cleanup
git status --porcelain -> M README.md, M backend/Cargo.lock, M backend/crates/hiverouter-{contract,core,provider,tools}/Cargo.toml,
                          M tools/runbook.md, M web/package.json   (8 modified files)
git diff --stat -> 8 files changed, 46 insertions(+), 26 deletions(-)
```
Note: the first `git status` preview was truncated at 5 lines; the full porcelain query shows **8** dirty files.

## E2 — Assigned file metadata (bytes / lines-from-read-tool / sha256 lower)

| file | bytes | lines | sha256 |
|---|---:|---:|---|
| `backend/Cargo.toml` | 1,765 | 54 | `323a5865a3ed81f1d2bbd00131195e7cc5ead96ace998606d640461afc258f10` |
| `backend/crates/hiverouter-contract/Cargo.toml` | 522 | 20 | `a926c4e1eac076e16e5a9df1886b0a6685a7f3eeb3f4dd53c4802a592723dd7d` |
| `backend/crates/hiverouter-provider/Cargo.toml` | 757 | 30 | `c8b8601294a1f74dff6340ec9fdea8f3d5b1924c82cfa989b5f73ce09c17b2c6` |
| `backend/crates/hiverouter-tools/Cargo.toml` | 1,203 | 51 | `074ede6fa256df200ea5b51720d4d86f84553468d432355b7377abd11c9de53d` |
| `backend/crates/hiverouter-core/Cargo.toml` | 1,083 | 41 | `a3f86ea4183c2ba3029fce40bf9d8d893eaa133c2fc77ad48d2a4edd74a66c56` |
| `backend/Cargo.lock` | 100,148 | 4,047 | `a2060244f12243d82074e60993fbfdb98188fee6bcbb7d73a3a63506767583f4` |
| `.gitignore` | 202 | 15 | `4562ca0b1e8a0857f535075477a1801b58fa28425932dc01a6f6e161a9976a94` |
| `tools/runbook.md` | 4,396 | 82 | `e7fba10f3332ca65fd3f81873aa2916916ada14ea7f889be8172c805c5b72cfc` |
| `tools/build-release.ps1` | 2,051 | 51 | `2b465b0d05f1c7b3f6689c2625f43b35aa747ebd379f6baa5f83d5395ec375e5` |
| `tools/release-manifest.json` | **MISSING** | — | — (Test-Path false; `git ls-files` empty) |
| `web/package.json` | 939 | 37 | `29dc97b13da96cd82bb8822a39903fd4422cb98e6027ea92ac78f1abfeb52f4b` |
| `dist/HiveMind-v1-win64.zip` | 6,571,928 | binary | `bc7a2bad98fab1c849e3340d195c26f165581350b630f7ba37d89ee7cb27fee7` |
| `dist/HiveMind-v1-win64/CHECKSUMS.txt` | 712 | 8 | `842e44c7439268b5114ee8a5a7f0e556dbb24d4ab958fd61ecd780d5f3380cb4` |
| `dist/HiveMind-v1-win64/hiverouter-cli.exe` | 5,364,224 | binary | `51e1aba6dc7349bfb8904014d7007015e13642ea9710f04d380525dccbd9b9ad` |
| `dist/HiveMind-v1-win64/hiverouter.exe` | 9,329,152 | binary | `184d51a7c3cca7b9d50954b677c5ad8a14472700a651fdf0d270441be3eac9a9` |
| `dist/HiveMind-v1-win64/README.md` | 1,276 | 24 | `cfa567329edc9e6c52ef02c8620e2fd27c5917bc92c3efc1794ea26dbf0beba3` |
| `dist/HiveMind-v1-win64/RUNBOOK.md` | 4,396 | 82* | `e7fba10f…` == `tools/runbook.md` (identical bytes, not re-read) |
| `docs/evidence/key-validation.md` | 1,038 | 15 | `71ea686846cc5055576c6fc86c388c296a81ab003b5d027aa556178d8323a2a6` |
| `docs/evidence/live-provider-smoke.md` | 1,312 | 13 | `9ed04f4581ba7500fd5398e046ceaada9887c88aa8906e61f1145b1263e18eeb` |

`*` READLOG's earlier `Measure-Object -Line` counts were systematically lower (e.g. Cargo.lock 3,642 vs
4,047; package.json 36 vs 37) — the `read`-tool totals are authoritative here; discrepancy recorded as an
evidence-method note, not a file defect.

## E3 — Release-artefact checks (read-only, no extraction)

```
git ls-files dist                         -> (empty)        # dist is NOT tracked
git ls-files tools/release-manifest.json  -> (empty)
.gitignore:3                              -> dist/           # explains V5
Get-FileHash dist/.../hiverouter.exe     -> 184D51A7…        == CHECKSUMS.txt line 2
Get-FileHash dist/.../hiverouter-cli.exe -> 51E1ABA6…        == CHECKSUMS.txt line 3
ZipFile.OpenRead(dist/HiveMind-v1-win64.zip).Entries
  -> CHECKSUMS.txt 712 B ; hiverouter-cli.exe 5,364,224 B ; hiverouter.exe 9,329,152 B ;
     README.md 1,276 B ; RUNBOOK.md 4,396 B            (5 members, no nested dir)
LastWriteTime: hiverouter-cli.exe 08:52:56 ; hiverouter.exe 09:10:59 ; zip 09:33:17 ;
               web/dist/index.html + assets 10:58:47    # packaged server predates web build
Test-Path web/package-lock.json -> True
```

## E4 — Key-custody scan (read-only, no key material touched)

Set scanned: `tools/runbook.md`, `tools/build-release.ps1`, `docs/evidence/key-validation.md`,
`docs/evidence/live-provider-smoke.md`, `dist/…/README.md`, `dist/…/RUNBOOK.md`, `dist/…/CHECKSUMS.txt`,
`web/package.json`, `.gitignore`.
Pattern: `sk-[A-Za-z0-9]|DEEPSEEK_API_KEY\s*=|api[_-]?key\s*[=:]`.
Result: 2 hits, both the same documentation placeholder:
`runbook.md:45` and `RUNBOOK.md:45` → `` `$env:DEEPSEEK_API_KEY = "<your-key>"` (never commit it; never paste it into the UI) ``.
No key value, token, or secret present. `.gitignore:10-11` excludes `.env` / `.env.*`.

## E5 — Official docs fetched (HTTP 200) and facts used

1. `https://api-docs.deepseek.com/` — full; base URLs, model names, `DEEPSEEK_API_KEY`, chat/completions POST,
   thinking/reasoning_effort/stream example.
2. `https://api-docs.deepseek.com/quick_start/pricing/` — full; 1M context, 384K max output, off-peak/peak
   prices for `deepseek-flash` and `deepseek-v4-pro`, concurrency 2500/500, peak windows UTC.
3. `https://api-docs.deepseek.com/guides/thinking_mode/` — read, **truncated** at tool-call sample;
   thinking default enabled/high; effort mapping; reasoning_content pass-back rule with tools.
4. `https://api-docs.deepseek.com/api/create-chat-completion/` — read, **truncated** at streaming response
   schema; request/response schema incl. max_tokens 1–393216, stream_options, tool_choice rules, usage fields.
Cross-check against repo: `docs/evidence/key-validation.md` uses model `deepseek-flash`, endpoint
`https://api.deepseek.com/chat/completions`, `thinking.type=disabled`, `max_tokens=16`, `stream=false` — all
consistent with the official docs; usage block shape matches the documented usage object.

## E6 — Files that could not be read / limitations

- `dist/HiveMind-v1-win64.zip`, `hiverouter.exe`, `hiverouter-cli.exe`: binary class; read as metadata/hashes
  only (no decompilation, no execution). Version strings inside binaries **UNVERIFIED**.
- `tools/release-manifest.json`: does not exist (this is the finding, not an unreadable file).
- Individual hunks of the 8 dirty files: diffstat captured; line-level hunks not audited (timebox) — UNVERIFIED.
- Live provider behaviour: requires key + paid call — prohibited; evidence docs treated as READ/UNVERIFIED.

## E7 — Quiescence re-check (11:41 CST; tree moved during the audit)

```
git status --porcelain  (11:41) -> 15 modified files:
  README.md, backend/Cargo.lock, 4 crate Cargo.toml,
  hiverouter-core/src/{execution.rs,provider_executor.rs},
  hiverouter-tools/src/{cli_support.rs,shell.rs},
  tools/runbook.md, web/package.json, web/src/flow/{Canvas.tsx,graph.ts},
  web/src/state/agentState.ts
  + untracked: ?? docs/RELEASE-EVIDENCE.md  ?? docs/RELEASE.md  ?? plans/seats/
               ?? tools/release-manifest.json
git rev-parse --short HEAD -> 5c69b66 (unchanged)
```
`tools/release-manifest.json` now exists (untracked) and was read in full:
```
[ { "path": "C:\\Users\\Riley\\Desktop\\hiverouter-v1\\target\\release\\hiverouter.exe",
    "sha256": "6b6f1d0ba3f12f556bb897429e2c78e7db4ad39060b2bc325f4fe660da7280b3", "bytes": 13251584 },
  { "path": "...\\target\\release\\hiverouter-cli.exe",
    "sha256": "d76bc64a944153c7d7e9821adb86daf1b7ffb819329c7a3f0d8a0bf82befeec0", "bytes": 5364224 } ]
```
Contradiction vs shipped dist: server manifest `6b6f1d0b…`/13,251,584 B vs dist `184D51A7…`/9,329,152 B;
cli manifest `d76bc64a…` vs dist `51E1ABA6…` (same 5,364,224 B). Manifest uses absolute host paths and has no
version/tag/toolchain/web-hash metadata. Recorded as S5-12 (HIGH) in REPORT §9; S5-02 ("manifest missing") was
true at 11:18 and is superseded. Tree was **not quiescent** during this audit (S5-13) — re-verify after seal.
