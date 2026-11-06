# READLOG — Seat S5 (Release/KeyOps)

Mandatory order held: (1) audit paper in full → (2) four official DeepSeek docs in full → (3) assigned audit
files one at a time → (4) product files one at a time. Line counts below are from the `read` tool (authoritative);
`Measure-Object -Line` undercounted on CRLF files (note recorded in EVIDENCE §E2). No skipping; binary classes
read as metadata/hash only. Read-only seat: no key access, no paid calls, no builds, no product/git writes.

Baseline at read time: HEAD `5c69b66e363e9f1d0801ea5c2e66a1fc12c8943c` = `v1.0.0-3-g5c69b66`; tag v1.0.0 =
`3988622d…`; tree **dirty, 8 files** (README.md, backend/Cargo.lock, 4 crate Cargo.toml, tools/runbook.md,
web/package.json; +46/−26).

## Phase 1 — audit paper (mandatory step 1)

| # | path | bytes | lines | sha256 | read at (CST) | depth | verified facts |
|---|---|---:|---:|---|---|---|---|
| 1 | `audit/HIVEMIND-AUDIT-PAPER.md` | 25,141 | 244 | `3baad198337564e1bd92ed23580a926426f92e2d4a9d64f260dbaaf4d6713d495` | 11:00 | FULL | subject HEAD 3988622 / tag v1.0.0; 507 tracked files; 8 seats; 405 Cargo-lock / 147 npm pkgs; 14 cmds/21 events/65 refusals/16 states; P0 A-F01 provider-not-wired, A-F02 dead replay, A-F03 16v15 states; P1 A-F07..A-F12; §5.3 dist "tracked" claim (refuted later); App. B reproducibility; App. C limits (no paid run, partial trace, licence UNVERIFIED) |

## Phase 2 — four official DeepSeek docs (mandatory step 2)

| # | URL | read at | depth | verified facts used by S5 |
|---|---|---|---|---|
| 2 | `https://api-docs.deepseek.com/` | 11:02 | FULL | `base_url` `https://api.deepseek.com` (+`/anthropic`); models `deepseek-flash` (V4.1-Flash-1017) / `deepseek-v4-pro`; `POST /chat/completions`; `DEEPSEEK_API_KEY`; thinking/reasoning_effort/stream params |
| 3 | `https://api-docs.deepseek.com/quick_start/pricing/` | 11:02 | FULL | 1M context / **384K max output**; Flash 0.003/0.006 hit, 0.15/0.3 miss, 0.6/1.2 out; Pro 0.022/0.044, 0.66/1.32, 1.98/3.96; peak 01–04 & 06–10 UTC Mon–Fri excl. CN holidays; concurrency 2500/500 |
| 4 | `https://api-docs.deepseek.com/guides/thinking_mode/` | 11:03 | PARTIAL — fetch truncated at tool-call sample | thinking default enabled/high; effort mapping (minimal→low, medium/xhigh→high, max→max); temp/presence/frequency ignored in thinking; reasoning_content pass-back required when tools present or 400 |
| 5 | `https://api-docs.deepseek.com/api/create-chat-completion/` | 11:03 | PARTIAL — fetch truncated at streaming response schema | messages min 1; model enum 2; max_tokens 1–393216 (defaults 8K/64K/128K-max); stream+stream_options.include_usage; tools function-only; tool_choice none/auto/required/named (required/named 400 in thinking); finish_reason enum; usage hit+miss+reasoning_tokens; user_id isolation |

## Phase 3 — assigned audit files (one at a time)

| # | path | bytes | lines | sha256 | read at | depth | verified facts |
|---|---|---:|---:|---|---|---|---|
| 6 | `audit/diagrams/06-migration-target.mmd` | 662 | 7 | `94a127fa46de065ea1f3dfeca6ebe619c8b62d737d11ea78056ddeb2cf64328c` | 11:08 | FULL | current v1.0.0 → W1 P0 correctness/hygiene → W2 contract/observability parity (16v15 states, event/trace merge, DOM G-UI) → W3 UI/asset truth (pixel review, S25) → target v1.1 100% traced/zero generated debt/CI parity. **No release/packaging wave** (gap) |
| 7 | `audit/tools/build_inventory.py` | 10,004 | 209 | `e7d3a9e53def9f2a51ed4c5b8739769195420eb8a4341ff0e99d00f500e092a5` | 11:08 | FULL | manifest from `git ls-files` only (line 140); sha256 upper; seat fn A–H; deep list 54; category()/ignored-class census hardcoded (dist/ 6 files/20.3 MB at lines 194-202) → release artefacts never hashed by audit inventory |
| 8 | `audit/tools/enrich_manifest.py` | 3,465 | 68 | `e8647e0d224b28b8aec18e3e0c93ae13b06fd10ca56cf6543fa6ea7c781b2a49` | 11:08 | FULL | adds `loc` (count newlines+1, text ext <5 MB) and `is_code`; writes ENRICHED + CODE-STATS.md; duplicate `/tests/` condition at line 64 |

## Phase 4 — product slice (one at a time; analysis in REPORT §3-§5)

| # | path | bytes | lines | sha256 | read at | depth | verified facts |
|---|---|---:|---:|---|---|---|---|
| 9 | `backend/Cargo.toml` | 1,765 | 54 | `323a5865a3ed81f1d2bbd00131195e7cc5ead96ace998606d640461afc258f10` | 11:14 | FULL | workspace 4 crates; MSRV 1.89.0; `MIT OR Apache-2.0`; exact `=x.y.z` pins lines 17-43; release `lto=thin, codegen-units=1, strip, panic=abort` |
| 10 | `backend/crates/hiverouter-contract/Cargo.toml` | 522 | 20 | `a926c4e1eac076e16e5a9df1886b0a6685a7f3eeb3f4dd53c4802a592723dd7d` | 11:14 | FULL | v1.0.0; serde/serde_json/uuid/thiserror/schemars(uuid1)/jsonschema; dev tempfile; no internal deps |
| 11 | `backend/crates/hiverouter-provider/Cargo.toml` | 757 | 30 | `c8b8601294a1f74dff6340ec9fdea8f3d5b1924c82cfa989b5f73ce09c17b2c6` | 11:14 | FULL | v1.0.0; contract + reqwest + rust_decimal + zeroize + async-trait; dev wiremock/tokio/tracing-subscriber |
| 12 | `backend/crates/hiverouter-tools/Cargo.toml` | 1,203 | 51 | `074ede6fa256df200ea5b51720d4d86f84553468d432355b7377abd11c9de53d` | 11:14 | FULL | v1.0.0; lib + `hiverouter-cli` bin + criterion bench; contract+provider+axum+win32job(win); dev criterion 0.8.2 |
| 13 | `backend/crates/hiverouter-core/Cargo.toml` | 1,083 | 41 | `a3f86ea4183c2ba3029fce40bf9d8d893eaa133c2fc77ad48d2a4edd74a66c56` | 11:14 | FULL | v1.0.0; bin `hiverouter` + lib + build.rs; deps contract **and provider** (line 21) + tools + rust-embed (embedded web/dist) — provider wired in current tree |
| 14 | `backend/Cargo.lock` | 100,148 | 4,047 | `a2060244f12243d82074e60993fbfdb98188fee6bcbb7d73a3a63506767583f4` | 11:16 | FULL (2 passes) | `version = 4`; 4 workspace pkgs at 1.0.0; duplicate majors: thiserror 1.0.69+2.0.21, tower-http 0.6.11+0.7.1, multiple getrandom/rand/windows majors (A-F07 persists); exact resolution of tokio 1.54.1/axum 0.8.9/reqwest 0.13.5/rusqlite 0.41.1/rust_decimal 1.43.0/zeroize 1.9.1/win32job 2.0.3 |
| 15 | `.gitignore` | 202 | 15 | `4562ca0b1e8a0857f535075477a1801b58fa28425932dc01a6f6e161a9976a94` | 11:18 | FULL | ignores target/, node_modules/, **dist/ (line 3)**, w64devkit/, *.pdb, *.tmp, figcraft *.figcraft/png/svg, **.env & .env.* (10-11)**, __pycache__, *.pyc, *.log — no release-manifest exception |
| 16 | `tools/runbook.md` | 4,396 | 82 | `e7fba10f3332ca65fd3f81873aa2916916ada14ea7f889be8172c805c5b72cfc` | 11:18 | FULL | truthful packaging ("localhost server + browser UI; no installer/Tauri/service"); prereqs Node>=22.12/Rust>=1.89/gcc/`DEEPSEEK_API_KEY`; key only present:true/false; build cmd `cargo build --release --locked`; **line 40 promises tools/release-manifest.json**; named refusal table incl E_KEY_MISSING/E_USAGE_INCOMPLETE |
| 17 | `tools/build-release.ps1` | 2,051 | 51 | `2b465b0d05f1c7b3f6689c2625f43b35aa747ebd379f6baa5f83d5395ec375e5` | 11:18 | FULL | PATH inject repo w64devkit; `cargo build --release --locked` (line 21); collects target/release exe hashes+bytes; writes `tools/release-manifest.json` (line 45); **does not build web, copy dist, write CHECKSUMS, or check tag/version** |
| 18 | `tools/release-manifest.json` | — | — | — | 11:18 | **MISSING at 11:18** | Test-Path false then; `git ls-files` empty. **11:41 re-check: file now exists untracked** (12 lines) — read in EVIDENCE §E7; its hashes contradict shipped dist (S5-12). See REPORT §9 |
| 19 | `web/package.json` | 939 | 37 | `29dc97b13da96cd82bb8822a39903fd4422cb98e6027ea92ac78f1abfeb52f4b` | 11:19 | FULL | name hiverouter-web, version 1.0.0, private, engines node>=22.12.0; exact pins react 19.3.1, vite 8.3.6, TS 7.0.2, tailwind 4.3.4, @xyflow/react 12.12.1, dagre 3.1.3, zustand 5.0.16, motion 14.9.1, partysocket 1.3.0; scripts dev/typecheck/build/preview/test |
| 20 | `dist/HiveMind-v1-win64.zip` | 6,571,928 | binary | `bc7a2bad98fab1c849e3340d195c26f165581350b630f7ba37d89ee7cb27fee7` | 11:20 | METADATA | zip = 5 members (CHECKSUMS.txt, cli exe, server exe, README.md, RUNBOOK.md); no nested dir; mtime 09:33:17 |
| 21 | `dist/HiveMind-v1-win64/CHECKSUMS.txt` | 712 | 8 | `842e44c7439268b5114ee8a5a7f0e556dbb24d4ab958fd61ecd780d5f3380cb4` | 11:20 | FULL | server `184D51A7…`, cli `51E1ABA6…` (both verified against files); build line claims `cargo build --release --locked`; run/open/cli usage lines; no zip hash, no toolchain/commit metadata |
| 22 | `dist/HiveMind-v1-win64/hiverouter-cli.exe` | 5,364,224 | binary | `51e1aba6dc7349bfb8904014d7007015e13642ea9710f04d380525dccbd9b9ad` | 11:20 | METADATA | PE binary; hash matches CHECKSUMS; mtime 08:52:56; version string UNVERIFIED (not executed) |
| 23 | `dist/HiveMind-v1-win64/hiverouter.exe` | 9,329,152 | binary | `184d51a7c3cca7b9d50954b677c5ad8a14472700a651fdf0d270441be3eac9a9` | 11:20 | METADATA | PE binary; hash matches CHECKSUMS; mtime 09:10:59; embeds UI built before current `web/dist` (10:58:47) |
| 24 | `dist/HiveMind-v1-win64/README.md` | 1,276 | 24 | `cfa567329edc9e6c52ef02c8620e2fd27c5917bc92c3efc1794ea26dbf0beba3` | 11:21 | FULL | workspace layout + toolchain (gnu, w64devkit) + build commands; references blueprint by **absolute host path** line 3; status points to PHASE evidence + ASSETS-INDEX + watchdog ledger |
| 25 | `dist/HiveMind-v1-win64/RUNBOOK.md` | 4,396 | = `tools/runbook.md` | `e7fba10f…` identical | 11:21 | READ-BY-HASH (identical bytes to #16; not re-read separately) | shipped runbook mirrors source runbook exactly |
| 26 | `docs/evidence/key-validation.md` | 1,038 | 15 | `71ea686846cc5055576c6fc86c388c296a81ab003b5d027aa556178d8323a2a6` | 11:22 | FULL | key custody: env-only, never repo/logs/UI/DB; endpoint+model; one validation call with thinking disabled/max_tokens 16/stream false; HTTP 200, response id `8d5af788…`, content `HIVE-KEY-OK`, usage 14/6/20; no key substring present; docs read list |
| 27 | `docs/evidence/live-provider-smoke.md` | 1,312 | 13 | `9ed04f4581ba7500fd5398e046ceaada9887c88aa8906e61f1145b1263e18eeb` | 11:22 | FULL | live server smoke debug binary 127.0.0.1:8799, fresh data dir; health/session/WS OK; SUBMIT_PROMPT → RUN_CREATED→ACK→NODE_SPAWN×2→NODE_STATE→REPLY_DELTA→REPLY_DONE→NODE_STATE→RUN_STATUS; completed; usage 89/6/95; honest open gaps: single delta non-streaming, SSE delta publishing + replay UI still being completed |

### Derived facts used in REPORT (not separate files)

- `git rev-list -n1 v1.0.0` = `3988622d…` vs HEAD `5c69b66e…`; HEAD message = provider wiring + replay arm +
  16-state parity + settings ⇒ P0 fixes are **post-tag**; tree dirty (8 files, +46/−26).
- `git ls-files dist` = empty and `.gitignore:3 dist/` ⇒ dist is ignored/untracked (paper §5.3 claim refuted).
- `CHECKSUMS.txt` hashes == recomputed hashes for both binaries; zip members listed read-only.
- Key scan: only `<your-key>` placeholder at `runbook.md:45` / shipped `RUNBOOK.md:45`; no key value.
