# BASELINE VERIFICATION — independent verifier seat (Phase A)

- **Seat:** independent verifier (read-only on product code; writes only `docs/audit/**`, `docs/claims/VERIFIER-*.md`, `docs/trace/**`, `docs/diagrams/**`).
- **Date:** 2026-11-06, America/Chicago (session window 04:05–04:30).
- **Workspace:** `C:\Users\Riley\Desktop\hiverouter-v1` (build tree) reading `C:\Users\Riley\Desktop\hivemind-blueprint` (reference, READ-ONLY).
- **Method:** every figure below was re-derived in this session by running the command shown; no teammate receipt was accepted. No installs, no live paid API call, no product-code edits.
- **Contract:** `EXECUTION-PROMPT.md` §0/§7; `docs/plans/{HASH-VERIFICATION,TOOLCHAIN,SESSION-PLAN}.md`; `docs/contracts/UI-COMMANDS-v1.md`; blueprint §5–§9; `UI-WIRING-GUIDE.md`; `reports/ui/UI-REQUIREMENTS.md`; `reports/verify/PRUNED-PLAN-CLAIMS.*`.

## 1. Source-of-truth hash gate — PASS (9/9)

Command: `Get-FileHash -Algorithm SHA256` on each artifact under `C:\Users\Riley\Desktop\hivemind-blueprint`.

| Artifact | Expected (EXECUTION-PROMPT §0) | Observed | Bytes | Status |
|---|---|---|---|---|
| `blueprint/HIVEMIND-BLUEPRINT-v1.md` | `4921ABE45746D5932FE5EF681D86C0D8F6217E9607F89D6AEBA9F8E9BBBD716B` | identical | 126400 | **MATCH** |
| `blueprint/UI-WIRING-GUIDE.md` | `3911C3E5DED6ED4728E00811126E1639C10BDEA281587997837A302A5F1E8104` | identical | 14838 | **MATCH** |
| `blueprint/APPENDIX-B-SOURCE-ARCHIVE.md` | `D28B9939E62CE51F86CD9F174396C26E1B1854967CBE4319A403111B881B69F3` | identical | 49551 | **MATCH** |
| `blueprint/PHASE-MAP.md` | `9BE9B0C1EEBACE57DB75AB776DB185E203EAA466C122E6A9F91736AFBFD00422` | identical | 11536 | **MATCH** |
| `blueprint/CHANGE-LEDGER.md` | `A94303A7C43ABCE3D7126569C5CBE05975E521FB808C3DBC7AF8A07E0620795F` | identical | 55039 | **MATCH** |
| `blueprint/FACT-REGISTER.md` | `AA8804B6544899FE55B5FE2DB1E2E3DCA7EE31890496B553AF2C90E0C9740B2A` | identical | 13655 | **MATCH** |
| `blueprint/CANONICAL-ARTIFACT-REGISTER.md` | `231825B6DCF0B0E90448D8C3EDB62C73133AA5CFA60D2F7D77521FCB851F680F` | identical | 7569 | **MATCH** |
| `blueprint/EXTERNAL-DEPENDENCIES.md` | `0C933FBFDD84B68AF89CCD34C2FC27DFC3BC3878A23CD50551870235FBF10308` | identical | 33498 | **MATCH** |
| `source/HIVEMIND POSSIBLITY.pruned.txt` | `7B85760E7662E7F8C33B410DFBECD6551AB569DAAA630A4F45DFB0B15E84D713` | identical | 146433 | **MATCH** |

Pruned-source custody re-derived: **146433 bytes**, **2613 CRLF** and **2613 LF** (CRLF-only), **no BOM**; hash above. This independently closes the `PRUNED-PLAN-CLAIMS.json` caveat ("hash not recomputed by that seat"): the source is byte-identical to the locked hash.

## 2. Toolchain re-verified — PASS

Quoted command output from this session (paths are absolute; no PATH mutation relied on):

| Item | Command | Observed output | Verdict |
|---|---|---|---|
| GCC | `tools\w64devkit\bin\gcc.exe --version` | `gcc.exe (GCC) 16.2.0` / `Copyright (C) 2026 Free Software Foundation, Inc.` | PASS |
| Cargo | `%USERPROFILE%\.cargo\bin\cargo.exe --version` | `cargo 1.99.1 (5f94df478 2026-08-27)` | PASS |
| rustc | `%USERPROFILE%\.cargo\bin\rustc.exe --version` | `rustc 1.99.1 (58cf8f9c3 2026-10-13)` | PASS |
| Rust toolchain | `rustc --print sysroot` | `C:\Users\Riley\.rustup\toolchains\stable-x86_64-pc-windows-gnu` | PASS |
| MSRV gate | installed 1.99.1 vs blueprint MSRV 1.89.0 | 1.99.1 >= 1.89.0 | PASS |
| C compile | `gcc hello.c -o hello.exe` (evidence dir) | `compile_exit=0`; run → `c-compile-ok`, `run_exit=0`; exe 49876 B sha256 `8F186FF8B9B159DD6107C4D2F5C57362B0679733EB1E688BB4E995B3B257F119` | PASS |
| Rust GNU link | `rustc hello.rs --target x86_64-pc-windows-gnu -C linker=<w64devkit gcc> -o hello_rs.exe` | `compile_exit=0`; run → `rust-gnu-link-ok`, `run_exit=0`; exe 5088192 B sha256 `8DF34E8A4D2C71099027DDD263FE52F040969F3F4E02CFAA26D252A173230ED3` | PASS |
| mermaid CLI | `mmdc.cmd --version` | `11.17.0` | PASS |
| Headless renderer | `C:\Program Files\Google\Chrome\Application\chrome.exe` present; `PUPPETEER_EXECUTABLE_PATH` smoke export exit 0, PNG 1110 B | PASS |
| Node | `node --version` | `v26.7.0` (floor >=22.12.0) | PASS |
| git | `git --version` | `git version 2.54.0.windows.1` | PASS |
| Disk free (C:) | `(Get-PSDrive C).Free` across run | 8.29–8.37 GB (see D-2) | RECORDED |

Toolchain evidence kept at `docs/audit/evidence/toolchain/` (`hello.c`, `hello.rs`, `hello.exe`, `hello_rs.exe`).

## 3. Claim-register validation (`docs/claims/CLAIM-REGISTER.csv/.json`) — PASS with two stale-status annotations

Register at baseline: 8 claims. `.json` declares `total 8, verified 6, read 2, unverified 0, contradicted 0`; row-level recount matches exactly (C-001..C-006 VERIFIED, C-007/C-008 READ).

| ID | Claim (as registered) | Independent re-check | Status at baseline |
|---|---|---|---|
| C-001 | All nine source-of-truth hashes match | Re-computed all nine: **9/9 MATCH** (§1) | **VERIFIED (confirmed)** |
| C-002 | FigCraft headless CLI exposes 597 commands | Re-ran `figcraft-cli.exe commands --json`: `json_bytes=164601`, `command_count=597`; binary 81,590,128 B at `C:\Program Files\FigCraft\figcraft-cli.exe` | **VERIFIED (confirmed)** |
| C-003 | Rust toolchain is not installed at this host at W0 | `cargo 1.99.1` / `rustc 1.99.1` now installed per-user; `TOOLCHAIN.md` records the install | **STALE — superseded** (was true at W0; must not be read as current) |
| C-004 | MSVC cl.exe and Windows SDK absent; GNU route viable | `cl.exe_count=0` under VS/Kits trees (recursive search); `Windows Kits\10` contains only `UnionMetadata` → no usable SDK; w64devkit gcc compiles+links (§2) | **VERIFIED (confirmed; refine wording to "no usable MSVC compiler/SDK")** |
| C-005 | Node v26.7.0 satisfies floor >=22.12.0 | `node --version` → `v26.7.0` | **VERIFIED (confirmed)** |
| C-006 | Disk free 6.05 GB at W0 | Observed 8.29–8.37 GB free now (`Get-PSDrive C`) | **STALE snapshot** (historical W0 value; current figure re-recorded) |
| C-007 | FigCraft GUI attach needs `--control`; headless CLI path used | READ-only claim; cited `FIGCRAFT-COMMANDREPORT.txt` exists; not exercised by verifier | READ (accepted as written, not re-fetched) |
| C-008 | Blueprint is a NEW synthesis, not the current app | Blueprint line 10: "Subject: the PROPOSED HiveMind/HiveRouter … **not** the current application" | READ (confirmed by citation) |

**Register verdict:** internally consistent and correctly labelled; no false `VERIFIED` found against fetched/run evidence. Two rows are time-stamped W0 environment snapshots that no longer describe the tree (`C-003`, `C-006`) — required action: append a note/supersession date, do not re-verify them as current. `C-004` wording should be tightened to "no usable MSVC compiler/SDK". These are register-hygiene items, not gate failures.

## 4. `reports/verify/PRUNED-PLAN-CLAIMS.*` validation — PASS

Independent parse of `PRUNED-PLAN-CLAIMS.csv` (49 rows) against the JSON:

| Check | Observed | Expected (JSON + blueprint §9) | Verdict |
|---|---|---|---|
| Row count | 49 | 49 | PASS |
| Status counts | `VERIFIED 1, READ 15, UNVERIFIED 19, CONTRADICTED 14` | identical | PASS |
| Contradiction IDs | `PPC-004,005,006,008,010,011,012,015,016,017,031,041,043,049` | identical set | PASS |
| Blueprint §9 statement ("49 claims: 1 VERIFIED / 15 READ / 19 UNVERIFIED / 14 CONTRADICTED") | matches parsed register | PASS |
| Decision-critical UNVERIFIED list | 10 rows present (60fps, thousands-of-agents, 890B KV, 552B/8B, prefix caching, exact token counting, live spend HUD, Tailwind @theme, 50-node burst, rewire-instantly) | carried as non-gates in blueprint §6 | PASS |

Spot-checks of the carried contradictions against the blueprint: PPC-010/011/012 (Python/Docker/seccomp/cloud) → resolved by D1 fence §6; PPC-015/016/017 (model ids, `/v1`) → corrected in §3/§5; PPC-031 (`reactflow` → `@xyflow/react`) → corrected in pins §3; PPC-041 (non-compiling `.chno` scheduler) → Phase 4 rejects it; PPC-043 (blanket WCAG) → corrected contrast table §5/§9. No unresolved contradiction is presented as a gate.

## 5. Baseline workspace state (snapshot, not a build verdict)

- `git HEAD d1638f2` ("W0/W1 checkpoint: verified blueprint, GNU toolchain …, frozen UI contract, contract crate, FigCraft A …"); working tree dirty because W1 backend work is in flight (expected, not a defect at baseline).
- Backend: `backend/crates/hiverouter-contract` (contract/envelope/event/refusal/state/usage + golden/schema/command-matrix tests), `hiverouter-core` (bus/checks/config/db/graph/http/main/ws), `hiverouter-provider` (budget/client/dto/accounting/pricing/secret + refusal-matrix test), `hiverouter-tools` stub. Cargo metadata present; **no build/test result was reported or re-run by this seat yet**.
- Frontend: `web/` contains only empty `src/` and `tests/` directories (**0 files**) → UI surfaces are not implemented at baseline; `docs/trace/controls.json` is therefore a planned inventory with every surface `UNVERIFIED`.
- `docs/audit/`, `docs/diagrams/` were empty before this seat's Phase-A writes. `docs/trace/controls.json` was concurrently authored by the **trace seat** (schema `hiverouter.trace.controls/v1`, generated `2026-11-06T10:24:50Z` = 04:24:50 local): 28 controls S01-S28 in guide order, every hop `PENDING`, source `e2e/data/control-matrix.json` (28 rows). This verifier independently re-checked it (count 28, ordered ids S01..S28, all status `PENDING_APP_NOT_BUILT`, all ids present in UI-WIRING-GUIDE §1) and **did not re-write the file** (one-writer rule); the verifier's own Phase-A 28-surface skeleton is archived at `docs/audit/evidence/controls-verifier-skeleton.json`.

## 6. Contradictions / discrepancies found (recorded, not hidden)

| # | Finding | Classification | Required handling |
|---|---|---|---|
| D-1 | `HASH-VERIFICATION.md` environment probe says rust/cargo "NOT FOUND"; `TOOLCHAIN.md` records rust 1.99.1 installed | Temporal supersession (W0 vs later), not a false claim if dated | Read W0 probe as historical; register C-003 must carry a superseded note |
| D-2 | Disk free: W0 note 6.05 GB; Lead brief said ~4.1 GB; observed **8.29–8.37 GB** | Snapshot drift | Do not budget from stale figures; re-measure before cargo/node_modules-heavy steps; record observed value |
| D-3 | C-004 says "Windows SDK absent"; `C:\Program Files (x86)\Windows Kits\10` directory exists (only `UnionMetadata`) | Wording precision | Claim should read "no usable MSVC compiler/SDK"; conclusion (GNU route) stands |
| D-4 | No product build/test evidence exists yet; backend edits in flight | Gate status: **UNVERIFIED**, not a defect | Phase B must re-run builds/tests on a quiescent tree; no product claim accepted until then |
| D-5 | Blueprint §6 open user decisions remain open: CF-02 prune fence confirmation, CF-03 Windows-vs-Ubuntu deployment, CF-09 deferred phases (plugins/multimodal/pausable debug/DR), CF-10 Obsidian, CF-05 desktop shell, CF-UI-06 streaming; S25 `DIFF_*` decision-gated | Decision-owned, not assumed | Do not implement/claim S25 `DIFF_*`; carry decisions into final report exactly as open |
| D-6 | UI pixel-level fidelity has **no board raster** (PDF is text-layer; contact sheets are shadow sprites) → `UNVERIFIED/needs_human` by design | Accepted limitation | Never present visual fidelity as verified |
| D-7 | `PRUNED-PLAN-CLAIMS.json` caveat: that seat did not recompute the pruned source hash | Custody caveat | Closed by this seat: hash recomputed, MATCH (§1) |
| D-8 | `docs/contracts/UI-COMMANDS-v1.md` (declared FROZEN Phase 1) has **uncommitted modifications** at baseline: adds commands `APPROVAL_RESOLVE`, `SETTINGS_STATUS`, `FILE_TREE_LIST`, `FILE_READ`, events `SETTINGS_STATUS_RESULT` / `FILE_TREE_RESULT` / `FILE_CONTENT`, and refusal codes `E_APPROVAL_NOT_FOUND`, `E_APPROVAL_DECISION_INVALID`, `E_PATH_OUTSIDE_JAIL`, `E_PATH_NOT_FOUND`, `E_FILE_TOO_LARGE` (current sha256 `FE6EE89E…CD7B`, 6833 B; `git diff` vs HEAD = +12/−4 lines, git index `9a45db1..5435170`). Matching Rust enum variants exist in `backend/crates/hiverouter-contract/src/command.rs` (lines 37-43, 94-95), i.e. doc+code move together. But the contract's own rule ("changing a name here requires a `CHANGE-LEDGER-IMPL.md` row and re-running the command matrix test") is **not evidenced**: no `CHANGE-LEDGER-IMPL.md` exists anywhere in the workspace yet. | Process deviation to close, not silent | Lead must add the `CHANGE-LEDGER-IMPL.md` row for the frozen-contract extension and re-run `ui_command_matrix` (+ serialized new fixtures) before Phase B trace/capability gates; verifier will re-check in Phase B |

## 7. Verdict

**BASELINE PASS** for the three Phase-A gates: (1) all nine source-of-truth hashes match; (2) GNU toolchain compiles C and links Rust end-to-end with quoted output at the pinned MSRV; (3) both claim registers are internally consistent and their contradiction/UNVERIFIED sets reconcile with the blueprint. Baseline **product** gates remain **UNVERIFIED/PENDING** because builds and tests are still in flight (D-4) — Phase B must re-run them on a quiet tree and may not accept receipts.

## 8. Report shape

| Done | Evidence |
|---|---|
| Re-verified 9/9 blueprint/source hashes | §1 table; `Get-FileHash` output; pruned source 146433 B / 2613 lines re-derived |
| Re-ran toolchain gates (gcc, cargo, C compile, Rust GNU link) | §2 quoted commands/outputs; binaries retained under `docs/audit/evidence/toolchain/` |
| Validated hiverouter-v1 claim register (8 claims) | §3 row-by-row re-check; C-003/C-006 stale annotations; C-004 wording |
| Validated PRUNED-PLAN-CLAIMS register (49 claims) | §4 counts/IDs exact match; spot-checks against blueprint |
| Recorded baseline workspace state + 7 discrepancies | §5, §6 |
| Created 28-surface controls inventory + 5 Mermaid diagrams | `docs/trace/controls.json` re-validated (28 ordered S01-S28, all PENDING); verifier skeleton archived at `docs/audit/evidence/controls-verifier-skeleton.json`; `docs/diagrams/{architecture,agent-lifecycle-state,run-sequence,ui-control-event-sequence,er-schema}.mmd` + `.png` + `.svg` (mmdc 11.17.0, Chrome headless, exit 0) |

| Not done | Reason |
|---|---|
| Product build/test/census/trace/UI-PDF gates | Builds in flight at baseline (D-4); Phase B scope, run only on a quiescent tree |
| Claim-register edits (C-003/C-006 annotations, C-004 wording) | Verifier is read-only on `docs/claims/CLAIM-REGISTER.*` except `VERIFIER-*.md`; Lead owns the register |

| Unverified | What would prove it |
|---|---|
| Backend compiles/tests pass, frontend builds, boot smoke, 28 surfaces render | Phase B: `cargo test`, `npm run typecheck && npm run build`, boot probe, Playwright/e2e receipts with quoted output |
| UI/PDF conformance and FigCraft asset acceptance | Phase B with UI-watchdog ledger + asset hashes; visual fidelity remains `needs_human` |
| Live provider behavior, cost math vs real usage | Requires user-approved key/budget; otherwise stays UNVERIFIED per D1 |
