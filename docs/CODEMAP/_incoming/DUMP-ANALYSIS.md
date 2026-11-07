# DUMP-ANALYSIS — historical Git-ingest snapshot of `rodneydanger1/hivemind`

**Status:** READ-ONLY analysis of an external ingest file. Nothing in this document was produced by
executing product code. No product file was modified to produce it.
**Input:** `C:\Users\Riley\Downloads\Gitingesthivemind.txt` — 5,007,714 bytes, 102,926 physical lines
(`Get-Content` count), 636 textual occurrences of `FILE: `, **635 real FILE sections**.
**Baseline tree:** `C:\Users\Riley\Desktop\hiverouter-v1`, `git rev-parse HEAD` = `aa16e13…`, **823 tracked
files** (`git ls-files` = 823 entries), single squashed commit, date 2026-11-06.
**Analysis date:** 2026-11-07 (America/Chicago).
**Evidence classes used below:** `VERIFIED` = recomputed in this analysis from the dump or from the current
tree with a quoted command/result; `READ` = quoted from a file in the dump; `UNVERIFIED` = asserted but not
proven here.

> **One-line verdict.** The dump is a **635-file snapshot of the post-1.1.0 / `1.2.0-dev` source line**, whose
> `audit/` directory is a **byte-identical carry-over of the `v1.0.0`-era audit** (`HEAD 3988622`, 507 files).
> All 635 paths still exist at current HEAD; 556 of them are byte-identical, 79 have changed, and HEAD adds
> **188 paths the dump never had**. Every audit finding below is therefore **HISTORICAL** — a statement about
> commit `3988622`, not about HEAD `aa16e13`.

---

## 1. Exact snapshot inventory

### 1.1 How the dump is encoded (and the three exactness traps)

| Property | Value | How measured |
|---|---|---|
| File size | 5,007,714 B | `(Get-Item).Length` |
| Physical lines | **102,926** | `(Get-Content -ReadCount 0).Count` — the figure in the task brief |
| `str.splitlines()` boundaries | 102,930 | 4 extra: four control chars (`0x0C`×3, `0x0B`×1) inside one file |
| Section delimiter | `=`×48 / `FILE: <path>` / `=`×48 | first at dump line 770–772 |
| `FILE: ` occurrences | 636 | `raw.count('FILE: ')` |
| **Real FILE sections** | **635** | `^FILE: ` at line start = 635; all 635 unique; 0 duplicates |
| Directory-tree file nodes | 635 | independent parse of lines 2–768 (132 dir nodes) |
| Tree paths ≡ section paths | **yes, exactly** | set equality of 635 vs 635 |
| Payload size | 4,881,819 B over 98,380 lines | sum of section bodies |
| Sub-directory listing depth | full tree, then bodies | lines 2–768 are the tree; bodies start line 773 |

**Trap 1 — the "636th FILE entry" does not exist.** The 636th substring match is
`DEFAULT_DB_FILE: &str = "hiverouter.db"` in `backend/crates/hiverouter-core/src/config.rs` (dump line 13781),
i.e. a `*_FILE:` constant, not a section header. The trustworthy count is **635**, corroborated twice
(tree nodes and unique paths agree exactly). If a future tool reports 636, it is counting substrings.

**Trap 2 — binary/empty bodies are placeholders, not content.** 49 of 635 sections (7.7%) carry a synthetic
body: **28 × `[Empty file]`** (12 B) and **20 × `[Binary file]`** (13 B), plus one genuine 18-byte file.
Consequence: for those 49 paths the dump holds **no content at all** — only the fact that the ingest classified
them. Examples: `runtime-test/data*/hiverouter.db-shm` (actually 32,768 B of SQLite SHM at HEAD),
`docs/audit/evidence/phase-b/cli/*.err` (genuinely 0 B at HEAD — placeholder correct),
`docs/audit/evidence/phase-b/cli/version-human.err` (726 B UTF-16LE at HEAD — correctly "binary"),
and **`plans/waves/W1-CORE-RECOVERY.md`, a 10,001-byte Markdown file at HEAD that the dump lists as
`[Binary file]`** — a false binary classification or the file was empty at snapshot time.

**Trap 3 — four control characters corrupt four identifiers.** Exactly four characters in the whole dump are
stray control codes, all inside `plans/foresight/RUN-3.md` (dump body 75413–75621): `0x0B` (VT) at dump line
75595 replacing the `v` of `verify_tool_grant`, and `0x0C` (FF) ×3 at line 75601 replacing the `f` of
`finish_timeout_wait`, `finish_cancelled_wait`, `final_status`. Any quote or code identifier recovered from
that file must be repaired by hand. This is an ingest-fidelity defect, not repo content.

### 1.2 Top-level counts — dump vs current HEAD

| Area | Dump | HEAD `aa16e13` | Δ |
|---|---:|---:|---:|
| `docs/` | **174** | **278** | +104 |
| `plans/` | 109 | 114 | +5 |
| `backend/` | 106 | 112 | +6 |
| `audit/` | **69** | **93** | +24 |
| `runtime-test/` | 46 | 82 | +36 |
| `web/` | 69 | 71 | +2 |
| `e2e/` | 37 | 39 | +2 |
| `tools/` | 19 | 23 | +4 |
| `dist/` | 0 | 4 | +4 |
| `(root)` | 3 | 4 | +1 |
| `ledgers/` | 2 | 2 | 0 |
| `.cargo/` | 1 | 1 | 0 |
| **Total** | **635** | **823** | **+188** |

Root files in the dump: `README.md`, `CHANGE-LEDGER-IMPL.md`, `rust-toolchain.toml` (`.gitignore` was added
later — it is current-only).

Dump composition by extension (`VERIFIED`): `.md` 221, `.rs` 100, `.json` 91, `.ts` 67, `.err` 23, `.tsx` 22,
`.py` 20, `.txt` 19, `.mjs` 17, `.db-shm` 13, `.mmd` 12, `.csv` 10, `.toml` 7, `.ps1` 4, `.c`/`.html`/`.css` 2
each, `.prompty`/`.zip`/`.jsonl` 1 each = **635**. Note the shape: the corpus is **35 % Markdown** — this is a
documentation-heavy repository, and a naive "code analysis" of the dump would be analysing docs.

Largest sections: `plans/foresight/RUN2-COMPILED-LEDGER.md` (49,752 B), `docs/ui/figcraft/A/scripts/05_brand.json`
(49,110 B), `audit/parts/b-core/FILES.md` (47,633 B), `backend/crates/hiverouter-core/src/run_state.rs`
(47,264 B), `tools/build-release.ps1` (45,273 B).

### 1.3 Coverage of the dump vs the current tracked tree

| Set relationship | Count | Evidence |
|---|---:|---|
| Dump paths | 635 | parsed |
| Current tracked paths | 823 | `git ls-files -z` → 823 NUL-separated records |
| **Common** | **635 (100 %)** | every dump path exists at HEAD |
| **In dump, absent at HEAD** | **0** | no deletions since the snapshot |
| **At HEAD, absent from dump** | **188** | added after the snapshot |
| Byte-identical after normalising CRLF/BOM | **556** | sha256 compare, dump body vs working file |
| Changed (both exist, content differs) | **79** | same compare |
| Placeholder bodies (no content to compare) | **49** | of which all 49 land in the "changed" set |

**So the dump is a strict subset of HEAD, not a divergent branch.** That is the single most useful structural
fact in this analysis: nothing was lost, only added or edited.

The 188 current-only paths, by area:

| Area | Count | Representative |
|---|---:|---|
| `docs/ui` | 62 | FigCraft build/asset-index material |
| `docs/audit` | 21 | `evidence/**` additions |
| `audit/diagrams` | 12 | all six `.png` + six `.svg` exports of diagrams 01–06 |
| `docs/diagrams` | 10 | rendered diagram exports |
| `docs/trace` | 7 | trace captures |
| `runtime-test` | 36 | more DB/WAL sets, `pkg-final/`, `pkg-smoke/`, `s*.out` |
| `audit` (root) | 24 | incl. `audit/QUALITY-MATRIX.csv`, `audit/APPENDIX-FILE-CENSUS.md`, `audit/inventory/FILE-MANIFEST*` |
| `plans` | 5 | foresight/seat reports |
| `backend` | 6 | incl. **`backend/Cargo.lock`** |
| `dist` | 4 | `HiveMind-1.0.0-win64.zip` + `.sha256`, same for 1.1.0 |
| `tools` | 4 | `tools/bin/*`, `tools/cc-test/*` |
| `e2e` | 2 | `e2e/.gitignore`, `e2e/package-lock.json` |
| `web` | 2 | — |
| root | 1 | `.gitignore` |

Notable absences, each of which matters for trusting the dump's audit claims:

- **`audit/inventory/FILE-MANIFEST.csv` / `.json` / `-ENRICHED.*` and `audit/QUALITY-MATRIX.csv` are NOT in the
  dump.** The paper's central claim is "507 tracked files, sha256 per file"; the dump ships *only*
  `audit/inventory/{CODE-STATS,DEEP-LIST,DIRECTORY-CENSUS}.md` (3 of the 8 inventory artifacts). The hash-anchored
  manifest that would let a reader re-derive the audit is absent, so **the dump cannot self-verify its own
  hashes** — it can only be checked against the current tree (which I did: 556/635 identical).
- `backend/Cargo.lock` is absent (tracked later) → every dependency-count claim in the audit (405 packages,
  duplicate-major finding A-F07) is unverifiable *from the dump*, though `backend/Cargo.lock` at HEAD does exist
  and does confirm the duplicate majors (§5.3).
- `dist/*.zip` artifacts are absent → packaging claims cannot be checked from the dump.
- `docs/CODEMAP/` is present with **15 files** (00–07, 09–12, `_data/summary.json`, 2 `_tools`); HEAD adds
  `08-FILE-CATALOG.md`, `_data/inventory.csv`, `_data/inventory.json`. (Relevant to this very document: the
  snapshot already contains an earlier CODEMAP generation.)
- `audit/diagrams` ships only the six `.mmd` sources; the `.png`/`.svg` exports are current-only.

### 1.4 What the 79 changed files are

Changed paths cluster in evidence/log and generated-data files (**30 of 79 are `runtime-test/**`** DB-WAL/SHM and
`.err/.out` captures; **24 more are `docs/audit/**`** CLI/evidence captures — 54 of 79 are captured output), plus
these substantive ones:

`CHANGE-LEDGER-IMPL.md`, `audit/ROADMAP.md`, `audit/diagrams/05-dependency-lineage.mmd`,
`audit/parts/watchdog-1/{DEPENDENCIES.md,QUALITY.csv}`, `audit/parts/watchdog-2/EVIDENCE.md`,
`backend/crates/hiverouter-core/src/{approval_wait.rs,lib.rs}`,
`backend/crates/hiverouter-core/tests/{http_boot.rs,provider_adapter.rs}`,
`backend/crates/hiverouter-tools/src/commands.rs`, `docs/CODEMAP/00-START-HERE.md`, `docs/audit/QA-ENV.md`,
`docs/audit/evidence/gen_controls.py`, `e2e/specs/{a11y-gates,tool-loop}.spec.ts`,
`e2e/{data-probe,live-check2,live-inspect,live-probe,live-shot,live-smoke}.mjs`,
`plans/foresight/RUN-3.md`, `plans/seats/SEAT-04/REPORT.md`, `.cargo/config.toml`.

Note what is **not** changed: `README.md`, `audit/FINDINGS-REGISTER.md`, `audit/HIVEMIND-AUDIT-PAPER.md`,
`audit/VERIFICATION.md`, `audit/WAVE-SHEET.md`, `audit/APPENDIX-IGNORED-DELIVERABLES.md`,
`audit/QUALITY-*.json`, `audit/ROADMAP-TASKS.csv`, `audit/IGNORED-DELIVERABLES.csv`,
`audit/inventory/*.md`, all six `audit/diagrams/*.mmd` except 05, `docs/claims/*`, `web/package.json`.
The audit corpus was **frozen in place** and carried forward unchanged.

### 1.5 Dating the snapshot (and why the exact commit is UNVERIFIED)

`VERIFIED` facts that constrain the date:

1. `README.md` (dump 773–835, 60 lines) is titled *"HiveMind — implementation workspace (sealed release
   1.1.0)"* and says the working tree is *"on an unreleased 1.2 pre-release source line"*.
2. `web/package.json` and all four crate manifests say `1.2.0-dev`; `web/package.json` is byte-identical in
   dump and HEAD.
3. `docs/RELEASE-EVIDENCE.md` (identical at HEAD) records *"HEAD `889b779` … `hivemind-1.1.0` remains
   `87079ce`"* and *"the tree is dirty"*.
4. The audit documents are the `v1.0.0`-era artifacts: `HEAD 3988622`, tag `v1.0.0`, 507 tracked files.
5. The local repository has **exactly one commit** (`git rev-list --count HEAD` = 1, `aa16e13`, dated
   2026-11-06, "docs: add GitHub publication receipt…"). The upstream history was squashed, so
   **`3988622`, `6a2b1c6`, `87079ce`, `889b779` are not objects in this clone** and cannot be inspected.

Conclusion: the dump post-dates the 1.1.0 seal (`87079ce`) and the version bump to `1.2.0-dev`, and is
consistent with the `889b779` working tree. Because the history was squashed and 79 files were edited between
snapshot and HEAD, **the exact source commit of the dump is UNVERIFIED**; it can only be characterised as
"a dirty `1.2.0-dev` working tree with 635 tracked files".

---

## 2. Every audit finding in the two named artifacts — all **HISTORICAL**

Source artifacts (both **byte-identical at HEAD**, so they are still the shipped audit text):

- `audit/FINDINGS-REGISTER.md` — dump section header line 906, body 908–983, 76 lines, 8,637 B.
- `audit/HIVEMIND-AUDIT-PAPER.md` — dump section header line 985, body 987–1235, 249 lines, 25,480 B.

**HISTORICAL banner (verbatim from the paper, dump line 993):**

> "**Post-release supersession (2026-11-06).** This paper is the audit snapshot at `HEAD 3988622` / tagged
> `v1.0.0`. Its headline P0 findings are historical: **A-F01 (provider not wired), A-F02 (replay dead) and
> A-F03 (16-vs-15 state parity) were fixed and live-proven after this audit** at sealed source `6a2b1c6` and
> released under tag `hivemind-1.0.0`. … The body below is retained unchanged as the dated audit evidence, not
> a statement about the current HEAD."

Register header (verbatim, dump line 908):

> "HEAD `3988622` · 507 tracked files · **30 consolidated findings (3 P0 / 10 P1 / 16 P2 / 1 informational)** ·
> seats: b-core (BC), b-provider (BP), watchdog-1 (W1), watchdog-2 (W2), figcraft-a (FA), figcraft-b (FB),
> ui-watchdog (UW), strip-watchdog (SW)."

Register discipline clause (verbatim, dump line 970): *"No finding becomes 'fixed' in this audit; fixes belong
to the roadmap waves."* → the register is a **point-in-time defect census**, and it says so itself.

### 2.1 P0 — correctness / data-integrity (3 findings) — HISTORICAL

| ID | Registered claim (condensed, faithful) | Evidence cited in the audit | Recheck vs HEAD |
|---|---|---|---|
| **A-F01** P0 | **Shipped server cannot perform a live model run.** `hiverouter-core` has no dependency on `hiverouter-provider`; `http.rs` constructs an `UnconfiguredExecutor`, so `SUBMIT_PROMPT` can never reach the provider even with `DEEPSEEK_API_KEY` set. All prior "provider wired" claims were crate-level only. | `core/Cargo.toml` (contract + tools only); `core/src/http.rs:77,117,154`; `core/src/execution.rs`; origin BC-F01 | **SUPERSEDED** — `core/Cargo.toml:21` now declares `hiverouter-provider`; `http.rs:137–146` only falls back to `UnconfiguredExecutor` when the provider is *unconfigured*; `provider_executor.rs:35,64,258` is the adapter |
| **A-F02** P0 | **History/replay rendering is dead.** `Canvas` passes an always-empty `replayNodes` array; `HISTORY_SNAPSHOT` is stored but never rendered, so the time-travel scrubber cannot show a graph. | `web/src/flow/Canvas.tsx:113-127` (`useMemo<Node[]>(() => [], [])`); `web/src/state/store.ts:379-385`; origin W1-F01 | **SUPERSEDED** — `Canvas.tsx:118` builds `replayNodes` from `history.replayGraph`; `:135` `nodes={history.replayGraph ? replayNodes : nodes}` |
| **A-F03** P0 | **State-vocabulary parity defect.** Backend declares **16** `AgentState` values (incl. `interrupted`); frontend declares **15** and omits `interrupted`, while its header calls the contract a "14-state list". A backend `interrupted` event is not representable by the UI decoder. | `contract/state.rs:18-51,55-72`; `web/src/state/agentState.ts:1-24`; origin Lead + W1 | **SUPERSEDED** — `state.rs:146` asserts `AgentState::ALL.len() == 16`; `web/src/state/agentState.ts` lists 16 incl. `'interrupted'`; header now says "16 states: the contract's 16-state list including `interrupted`" |

### 2.2 P1 — contract / verification / reproducibility (10 findings) — HISTORICAL

| ID | Registered claim | Evidence cited | Recheck vs HEAD |
|---|---|---|---|
| **A-F04** P1 | Prompt/persona assets never loaded by the production server; handshake/context-switch code has no live rewire/engine caller. | `http.rs:82` `TemplateRegistry::default()`; `ws.rs:338-353` no caller; BC-F02/F03 | **SUPERSEDED** — `http.rs:70-71` `load_prompt_assets()` → `TemplateRegistry::load_embedded()`; `prompts.rs:38,43` `include_str!` the manifest/prompty. `TemplateRegistry::default()` survives only as a test-only seam (`http.rs:275,290`) |
| **A-F05** P1 | Settings panel renders a refusal from `refusals[0]` instead of the `SETTINGS_STATUS` error. | `web/src/state/store.ts:666`; W1-F02 | **SUPERSEDED** — `refusals\[0\]` no longer appears anywhere in `store.ts` (0 hits); `SETTINGS_STATUS_RESULT` handled at `store.ts:1537` |
| **A-F06** P1 | Money overflow silently yields `Decimal::ZERO` in `price()` — violates the no-silent-zero cost rule. | `provider/src/accounting.rs:330`; BP-H1 | **STILL OPEN** — `accounting.rs:330-335` is unchanged in shape: `checked_mul(...).map(...).unwrap_or(Decimal::ZERO)`. Adjacent call sites *do* refuse (`:267-283` `Refusal::CostMathInvalid`), which sharpens the inconsistency |
| **A-F07** P1 | Duplicate major versions resolved in the lockfile (`thiserror` 1.0.69 + 2.0.21; `tower-http` 0.6.11 + 0.7.1) — `cargo tree -d` gate not clean. Audit verdict for the gate itself: `UNVERIFIED`. | `backend/Cargo.lock`; BP-H6 | **STILL OPEN** — HEAD `backend/Cargo.lock` still lists `thiserror` 1.0.69 **and** 2.0.21, `tower-http` 0.6.11 **and** 0.7.1. (Static grep only; `cargo tree -d` still not run) |
| **A-F08** P1 | Headline verification evidence is **untracked** (`docs/audit/evidence/final-gate-tests.log`, `e2e/test-results/run-live-FINAL-artifact.log`), so the 507-file manifest lacks the receipts used by FINAL-REPORT/FINAL-DELTA. | `git ls-files` vs `FINAL-REPORT.md:36,38`; FB-F05 | **SUPERSEDED** — `git ls-files docs/audit/evidence` = **54** tracked files (dump already contains them); `docs/trace` = 47 tracked. Not every cited receipt is present, but the class is now tracked |
| **A-F09** P1 | **82 tracked `runtime-test/` artifacts** (SQLite DB/WAL/SHM, snapshots, `.out/.err`), including the shipped binary stored **three times** (identical hash), one stale build, a duplicate CLI, nine DB/WAL sets. Generated artifacts in source control. | `git ls-files runtime-test`; W2-S9 | **STILL OPEN and worse** — `git ls-files runtime-test` = **82** at dump *and* **82** at HEAD, while the directory grew 46 → 82 tracked paths (36 added later) |
| **A-F10** P1 | Documentation/claims drift: FINAL-REPORT trace counts (260 UNVERIFIED / 5 MISSING) disagree with `docs/trace/controls.json` (270 PENDING / 6 MISSING); FINAL-DELTA claim-register FAIL is stale after later repair; committed `evidence/final/pw-out/.last-run.json` says `failed` while closeout says passed. | `FINAL-REPORT.md:45`; `docs/trace/controls.json`; FB-F01..F04 | **NOT RECHECKED** — would need a per-claim recount of the trace census |
| **A-F11** P1 | `.cargo/config.toml` hard-codes absolute host paths (`C:/Users/Riley/...`) for linker/CC/AR → builds are machine-specific. | `.cargo/config.toml`; UW-F01 | **SUPERSEDED** — HEAD `.cargo/config.toml` is fully relative: `CC_x86_64_pc_windows_gnu = "gcc"`, `AR_x86_64_pc_windows_gnu = "ar"`, `[target.x86_64-pc-windows-gnu] linker = "gcc"` |
| **A-F12** P1 | **18 primary design deliverables are gitignored and outside the tracked manifest.** `.gitignore:7,9` excludes `*.figcraft` and `ASSETS/*.svg|png`; the manifest omits the pixel products (18 files, 22,009,234 B). | `.gitignore:7,9`; `git check-ignore -v`; `audit/APPENDIX-IGNORED-DELIVERABLES.md`; FA-F1 / UW-F13 | **STILL OPEN** — HEAD `.gitignore:7-9` still ignores `docs/ui/figcraft/*/*.figcraft`, `docs/ui/figcraft/ASSETS/*.png`, `docs/ui/figcraft/ASSETS/*.svg` |
| **FA-F1** P1/HIGH | (as A-F12 — the figcraft seat's original: "the 18 primary deliverables are gitignored, so the inventory/assignments omit them") | `audit/parts/figcraft-a/FINDINGS.md` F1 | **STILL OPEN** — same `.gitignore` lines |

### 2.3 P2 — hygiene, UX, maintainability (16 findings) — HISTORICAL

| ID | Registered claim | Evidence cited | Recheck vs HEAD |
|---|---|---|---|
| **A-F13** P2 | 6 ad-hoc e2e probe scripts (`data-probe`, `live-check2/inspect/probe/shot/smoke`) are not wired to the Playwright runner. | `e2e/*.mjs`; `e2e/package.json` scripts; W2-S1 | **PARTLY OPEN** — all six still present; a new `e2e/run-guarded.mjs` exists but the six probes are still outside the Playwright spec set |
| **A-F14** P2 | Skip-as-green: app-required specs skip when `E2E_APP_READY` is unset, and a skipped run exits 0. | `e2e/helpers/matrix.ts`; `.last-run.json`; W2-S2 | **STILL OPEN (guard present)** — `matrix.ts:44` `return process.env.E2E_APP_READY !== '1';`; whether a skip now exits non-zero was **not** re-verified |
| **A-F15** P2 | Test output is wiped each run (`test-results/`), so run logs are not durable. | `e2e/playwright.config.ts` outputDir; W2-S3 | **STILL OPEN** — `playwright.config.ts:21` `outputDir: './test-results'` |
| **A-F16** P2 | Trace capture is not completeness-enforced; `S-APPROVAL` has no capture; validator uses lower-bound thresholds (≥10/≥12/≥30) instead of exact sets. | `tools/qa/verify-harness.mjs`; `docs/trace/controls.json`; W2-S4/S8 | **STILL OPEN** — `verify-harness.mjs:102,104,105` still `frozenCommands.length >= 10`, `events.length >= 12`, `refusals.length >= 30` |
| **A-F17** P2 | Tests write evidence into `docs/` during runs (generated artifacts mixed into the documentation tree). | `e2e` trace/evidence writers; `docs/audit/evidence/**`; W2-S10 | **STILL OPEN** — `e2e/helpers/evidence.ts:5-6` and `trace.ts:4` resolve output into `../../docs/trace/…` |
| **A-F18** P2 | Playwright default baseURL is `:8080` while the shipped workflow used `:8799`, with no artifact-identity preflight. | `e2e/playwright.config.ts`; runbook; W2-S7 | **STILL OPEN (documented)** — `playwright.config.ts:17` `process.env.E2E_BASE_URL ?? 'http://127.0.0.1:8080'`, with a comment naming `:8799` |
| **A-F19** P2 | Declared Playfair/Inter faces are not bundled as font packages; UI relies on system/fallback fonts (R-U12 unmet). | `web/src/styles/tokens.css:30-32`; `web/package.json` (no font deps); W1-F04 | **STILL OPEN** — no `*font*` dependency exists in `web/package.json` |
| **A-F20** P2 | Dev mock is statically imported in the production entry; runtime-gated but may be bundled. | `web/src/api/index.ts:8`; W1-F05 | **SUPERSEDED** — `index.ts:6-9,67-68` now uses a **dynamic** `import('./mock')` inside `USE_DEV_MOCK` (`import.meta.env.DEV && VITE_UI_MOCK === '1'`) |
| **A-F21** P2 | No HTTP timeout/abort on `/api/session` or `/health` fetch paths. | `web/src/api/client.ts:235`; `web/src/api/health.ts:15-22`; W1-F06 | **SUPERSEDED** — `client.ts:26` imports `DEFAULT_HTTP_TIMEOUT_MS, fetchWithBoundTimeout`; `requestTimeoutMs`/`handshakeTimeoutMs`/`httpTimeoutMs` are first-class options |
| **A-F22** P2 | Token-isolation test scans `src/` only; shipped `index.html:9` contains a hex favicon colour. | `web/index.html:9`; token test; W1-F07 | **STILL OPEN (moved)** — `web/index.html:7` `<meta name="theme-color" content="#F8F4EB" />` |
| **A-F23** P2 | Approval nonce is treated as an optional extension; frozen `APPROVAL_REQUEST` row lacks it. | contract `event.rs` approval payload; UI-COMMANDS-v1; W1-F08 | **PARTLY ADDRESSED** — `event.rs:483-498` documents nonce as *"Single-use nonce required by `APPROVAL_RESOLVE` (1.2 additive)"* plus `args_sha256`, but kept `Option<String>` for wire compatibility ("pre-1.2 frames carry no nonce") |
| **A-F24** P2 | Stale docs: several pages still say the app does not exist / trace is empty after release; claim-register JSON retains stale decision-critical entries. | `docs/trace/live/README.md`; `CLAIM-REGISTER.json` C-026; UW-F02 / FB-F01 | **NOT RECHECKED** |
| **FA-F2** P2/MED | `ASSETS-INDEX.json` carries A hashes (28/28) but **0/27** B export hashes. | `audit/parts/figcraft-a/FINDINGS.md` F2 | **NOT RECHECKED** — file present, 35,998 B, 60 `sha256` mentions; a per-surface audit was out of scope |
| **FA-F3** P2/MED | `ASSETS-INDEX` watchdog block stale ("spinner #D9D9D9 fix in progress") vs the ledger's fixed state. | figcraft-a F3 | **NOT RECHECKED** |
| **FA-F4** P2/MED | B episodes use non-standard `outcome="stage-ok"` (7/8) and omit command ids in `plan.steps`. | figcraft-a F4 | **NOT RECHECKED** |
| **FA-F5** P2/MED | 6 tracked CPython-3.14 `.pyc` files + scratch `B/build/_run.json`; no `*.pyc` ignore rule. | figcraft-a F5 | **PARTLY ADDRESSED** — `.gitignore:14` now has `*.pyc`, but the **6 `.pyc` files are still tracked** (`git ls-files "*.pyc"` = 6, all `docs/ui/figcraft/B/build/__pycache__/*.cpython-314.pyc`) |

### 2.4 Informational (1 finding) — HISTORICAL

| ID | Registered claim | Recheck |
|---|---|---|
| **FA-F6** INFO | 8 tracked patch scripts + intermediate `.figcraft`/QA PNGs in delivered build folders. | **NOT RECHECKED** |

### 2.5 Provenance of the 30: seat-level originals

The register consolidates 30 IDs. The per-seat originals are richer and are **not** fully consolidated — this
is the part of the audit with the most residual value:

| Seat file | Numbered findings | Severity spread |
|---|---:|---|
| `audit/parts/b-core/FINDINGS.md` | 11 (`F-01`…`F-11`) | HIGH 1 / MED 2 / LOW 7 / INFO 1 |
| `audit/parts/b-provider/FINDINGS.md` | 16 (`H-1`…`H-16`) | MED 5 / LOW 9 / INFO 2 |
| `audit/parts/watchdog-1/FINDINGS.md` | 0 numbered (bulleted High/Medium/Low sections) | — |
| `audit/parts/watchdog-2/FINDINGS.md` | 13 (`S1`…`S13`) | MED 4 / LOW 6 / INFO 3 |
| `audit/parts/figcraft-a/FINDINGS.md` | 8 (`F1`…`F8`) | HIGH 1 / MED 4 / LOW 1 / INFO 2 |
| `audit/parts/figcraft-b/FINDINGS.md` | 10 (`F-01`…`F-10`) | HIGH 1 / MED 4 / LOW 3 / INFO 2 |
| `audit/parts/ui-watchdog/FINDINGS.md` | 0 numbered ("Notes on method" only) | — |
| `audit/parts/strip-watchdog/` | no `FINDINGS.md` (verifier writes `VERIFICATION.md`) | — |
| **Total numbered** | **58** | consolidated into 30 register IDs |

Seat findings that survive consolidation only partially include e.g. `H-3` "nonce-less `resolve` remains a
public API", `H-8` "jail refuses every `~` path segment", `H-10` "`cargo.exe` on the default shell allowlist",
`F-04` "migration v1 hardcodes the contract schema version instead of binding the Rust const", `F-05`
"history read model silently degrades a corrupt revision", `S5` "S21 trace under-claims render-back". These are
**HISTORICAL, unrechecked, and not obviously closed** — they are the best candidates for a fresh pass.

---

## 3. Architecture methodology and topology claims (with quotes)

### 3.1 Four-crate topology and the dependency chain

Paper §3 (dump L1051): *"HiveMind v1 is a **four-crate Rust workspace** plus a **React SPA** and an
**agent-native CLI**"*, then one line per crate. The chain claim (§1.2, dump L1014): *"four internal crates with
the dependency chain `contract → provider → tools → core`; 14 frozen commands, 21 events, 65 refusal codes,
16 declared agent states; 405 Cargo-lock and 147 npm-lock packages."*

`audit/diagrams/03-module-dependencies.mmd` (dump 1943–1960) states the edges explicitly:

```
P --> C     T --> C     T --> P     K --> C     K --> T     K --> W     CLI --> P     CLI --> T
```
where `C`=contract, `P`=provider, `T`=tools, `K`=core, `W`=`web/dist (embedded)`.

`READ`: `backend/crates/hiverouter-core/Cargo.toml:20-22` at HEAD confirms `contract`, `provider`, `tools` —
i.e. the "`core` depends on contract + tools only" premise of A-F01 is definitively gone.

### 3.2 Transport and trust boundary

Paper §3 (dump L1053): *"The browser talks to the binary only over loopback: a session token from
`/api/session` authenticates a WebSocket handshake; a foreign `Origin` is refused; every mutating command is
ACK-or-refusal, with events fan-out carrying monotonic sequence numbers. Provider calls are bounded by the
blueprint's budget core; cost math uses `rust_decimal` only."*

Same paragraph, the design rationale worth preserving: *"The design intentionally collapses transport,
execution, storage, UI delivery and diagnostics into one machine-local process — reducing deployment risk while
making the in-process boundaries the critical audit surface."*

`audit/diagrams/01-system-topology.mmd` (dump 1871–1911) is the machine-readable form: a `Browser` subgraph
(React 19 SPA, partysocket `hello{last_seq}`, `fetch /api/session · /health`) and a `Binary` subgraph
("`hiverouter.exe — Rust single binary (loopback only)`") containing axum router, WS handler (Origin allowlist
+ per-session token), rust-embed static UI, `/diagnose`, the frozen
`hiverouter.v1 contract 14 commands · 21 events · 65 refusals`, event bus, `GraphAuthority` (petgraph, single
owner), run lifecycle, provider client, tool registry/approval/jail/shell, SQLite, snapshot store, JSONL audit
ledger; plus `DeepSeek API` and `hiverouter-cli` outside.

### 3.3 Command/data flow and the no-optimism rule

`audit/diagrams/02-data-flow.mmd` (dump 1915–1939) is the canonical interaction contract:

```
U->>UI: gesture (submit / spawn / rewire / cancel / kill / files)
UI->>WS: COMMAND frame {schema hiverouter.v1, correlation_id}
WS->>H: Origin allowlist + session token check
H->>H: schema validation + named refusal path
H->>S: dispatch (ACK only after acceptance)
S->>T: tool execution and/or provider call (bounded)
T->>D: persist ledger/run/approval rows
D-->>S: success or typed refusal
S->>E: ACK + EVENT frames (monotonic seq)
E-->>WS: fan-out to all clients
WS-->>UI: typed result or Error{code,how_to_fix}
UI-->>U: render-back from server events (no optimistic success)
```

Paper §6.2 (dump L1157) enumerates the invariants as *"one graph authority with cycle refusal; ACK-before-mutation;
named refusals (65 codes); nonce-bound single-use approvals with replay/TTL refusal; workspace jail fails closed
on 6 escape vectors; Decimal-only cost math; NULL usage ≠ zero (`INCOMPLETE`, `settled_usd:null`); loopback-only
bind; Origin+token WS boundary"* — and immediately disclaims proof: *"The audit does not re-derive their proofs
here; file-level evidence is in the parts."*

The dump's own (later-generation) `docs/CODEMAP/02-ARCHITECTURE.md` (dump 45959–46017) restates the same
invariants in current terms, e.g. *"No fake success: every failure is a typed `E_*` refusal with
message/how_to_fix/retryable"*, *"Single-use approval: nonce + `args_sha256` + `decision IS NULL` CAS; unique
pending index"*, *"At-most-once side effect: preflight before grant; grant consumed exactly once; no retry after
commit failure"*, *"Unknown is not zero: usage/bytes/limits stay absent when unmeasured"*.

### 3.4 State inventory

`audit/diagrams/04-state-inventory.mmd` (dump 1964–1975) is the authoritative historical statement — and its
own caveat is the part most often dropped when the diagram is quoted:

```
subgraph Declared["Declared AgentState inventory (contract/state.rs:18-51; 16 variants)"]
  persistent: healthy · running · queued · planning · thinking · awaiting_approval · streaming
  terminal (is_terminal=true): failed · completed · cancelled · killed · interrupted
  server/client surface states: reconnecting · replay · empty · loading
note["Transitions are enforced by execution/run code paths,
      not enumerated in state.rs; this diagram lists declarations only."]
```

Paper §3 (dump L1055): *"16 declared `AgentState` variants with five terminal states
(`failed, completed, cancelled, killed, interrupted`)"*.

### 3.5 Audit methodology claims (the reusable part)

Paper §2.1 (dump L1034): *"`git ls-files` enumerated the tracked corpus; each file was read as bytes and hashed
with SHA-256 into `FILE-MANIFEST.csv/.json`. Ignored trees were measured separately (`DIRECTORY-CENSUS.md`).
The manifest is the single source of truth for path, size, hash, language, category, seat and depth."*

Paper §2.2 (dump L1036) — the exact partition, which sums to 507:

> *"b-core 54 files (contract + core), b-provider 33 (provider + tools + manifests), watchdog-1 55 (web),
> watchdog-2 122 (e2e + tools + runtime-test), figcraft-a 103 (docs/ui + ledgers), figcraft-b 91 (docs
> plans/contracts/claims/reports/audit), ui-watchdog 49 (trace/diagrams/root + provenance), strip-watchdog
> independent verifier. Sum = 507."*

Paper §2.3 (dump L1038) — evidence rules, the part worth adopting verbatim:

> *"Each claim must cite `path:line`, a command output, or an official URL with fetch date. Unverifiable claims
> are marked `UNVERIFIED` with the proving action. … The verifier independently samples hashes/claims and checks
> that no product file changed."*

`audit/WAVE-SHEET.md` (dump 1809–1867) adds the operational rules: read-only audit of product code, no installs /
rebuilds / paid calls; *"Deep files (56) get an 8–12 paragraph analysis each"*; *"Never infer a function's
behavior from its filename"*; *"If a file is unreadable/binary: record metadata + class + `UNVERIFIED
(binary/generated)`; do not invent content"* — a rule the dump's own ingest then violated (§1.1 trap 2).

### 3.6 Migration target

`audit/diagrams/06-migration-target.mmd` (dump 2017–2026):

```
C["Current v1.0.0 — 507 tracked files · 188 code · 36,802 source LOC — localhost single binary + React SPA"]
  → W1["Wave 1 — P0 correctness & hygiene"]
  → W2["Wave 2 — contract & observability parity"]
  → W3["Wave 3 — UI/asset truth"]
  → T["Target — auditable, reproducible v1.1 — 100% traced controls · zero generated debt · CI-enforced parity matrix"]
```

Paper §7.1 (dump L1179) frames it as: *"Current: a working, compact, single-binary v1.0.0 with strong contract
discipline and a rich test corpus, but with hygiene debt … Target v1.1: a generated-artifact-clean repository;
backend/frontend protocol parity enforced by CI; a complete evidence-backed trace matrix (or explicitly gated
UNVERIFIED modes); human-approved pixel fidelity; reproducible packaging with CLI parity."*

---

## 4. Quality statistics, roadmap, ignored deliverables, verification posture

### 4.1 Corpus profile (as claimed) — HISTORICAL

Paper §4.1 (dump L1075–1086), `audit/inventory/CODE-STATS.md` (dump 2030–2168) and
`audit/inventory/DIRECTORY-CENSUS.md` (dump 2236–2383) agree on:

| Dimension | Value |
|---|---|
| Tracked files / bytes | **507** / 63.98 MiB (67,087,861 B) |
| Code-classified | **188** (strict `source` category: 158) |
| Source LOC / total text LOC | **36,802** / 106,840 |
| Top directories (507-file census) | docs 236, backend 87, runtime-test 82, web 55, e2e 24, tools 16, root 4, ledgers 2, .cargo 1 |
| Languages | Rust 79/21,669 · TS 40/5,870 · TSX 21/1,708 · Python 27/6,293 · JSON 88/60,049 · MD 57/4,193 · JS 12/798 · CSS 243 · PS1 198 · HTML 17 · C 6 |
| Cargo-lock / npm-lock packages | **405** / **147** |
| Git history | 23 commits, 2 author identities, all 2026-11-06 |
| Generated/ignored trees | `target/` 13,525 files/8,087.2 MB · `web/node_modules` 3,976/110.2 MB · `e2e/node_modules` 194/18.2 MB · `tools/w64devkit` 9,452/716.7 MB · `dist/` 6/20.3 MB · `runtime-test/` 82 tracked/33.6 MB |

Internal arithmetic check (`VERIFIED`): the language table's itemised source LOC sums to 35,540, while
`36,802` is the total — the verifier already flagged this (see §4.5 item 4): the missing 1,262 = JS 798 + CSS 243
+ PS1 198 + HTML 17 + C 6. The total is right; the table is a partial breakdown.

### 4.2 Test posture (claimed, not re-executed) — HISTORICAL

| Gate | Claim | Evidence cited |
|---|---|---|
| Rust contract/core/provider/tools | **233 passed / 0 failed / 1 ignored** | `docs/audit/evidence/final-gate-tests.log` |
| Frontend unit/typecheck/build | 46 tests, typecheck 0, Vite build 0 | `web/`; FRONTEND status |
| Live browser e2e | **79 passed / 0 failed / 14 explicit skips** | `e2e/test-results/run-live-FINAL-artifact.log` |
| Frozen contract integrity | 14 commands / 21 events / 65 refusals / 16 states | contract tests + source |
| Live server | `/health`, `/diagnose`, `/api/session`, `/` all 200; traversal 404; foreign Origin 403 | `docs/audit/PHASE-B-VERIFICATION.md` |
| Packaging | server+CLI zip, clean-extraction smoke | `dist/` + `FINAL-CLOSEOUT.md` |
| Trace ladder | **43 WIRED-AND-WORKS** of 319 controls; 270 PENDING / 6 MISSING | `docs/trace/controls.json` |

### 4.3 Quality scores — and a real inconsistency between the artifacts

`audit/QUALITY-SUMMARY.json` (dump 1381–1479): 507 files scored, dimension means
**correctness 4.24 · maintainability 3.72 · modularity 3.52 · error handling 3.63 · security 4.06 · docs 3.34**;
per-seat (files/means): b-core 54 (4.85/4.76/4.96/4.80/4.87/4.81), b-provider 33 (4.55/4.42/4.36/4.27/4.61/3.58),
watchdog-1 55 (4.47/4.24/4.36/4.29/3.78/3.98), watchdog-2 122 (3.74/3.22/3.22/3.22/3.47/2.51),
figcraft-a 103 (4.50/3.45/3.37/3.67/3.53/3.31), figcraft-b 91 (4.20/3.44/2.57/3.38/5.00/2.97),
ui-watchdog 49 (3.92/3.84/3.24/2.57/3.90/3.69).

`audit/QUALITY-CODE-STATS.json` (dump 1264–1346) and paper §6.3 agree on code-only means over **184 of 188**
scored files: **4.43 / 4.25 / 4.32 / 4.15 / 4.11 / 3.84**, `total_scored: 507, code_scored: 184,
non_code_scored: 323`.

**`audit/FINDINGS-REGISTER.md` disagrees** (dump ~L952-960): it reports a *different* code-only table
"159 files scored before FA partition lands": **4.50 / 4.29 / 4.37 / 4.33 / 4.29 / 3.97**, and claims the FA
partition *"will move the code-file count to 188"*. So the register carries a **pre-merge quality snapshot**
that contradicts the paper and the JSON. Anyone quoting "quality 4.50 correctness" is quoting the stale table.

Second smell worth recording: `QUALITY-CODE-STATS.json` shows figcraft-a's 25 code files scoring **exactly**
4.0/4.0/4.0/3.0/3.0/3.0 (only two distinct values), and figcraft-b's security at **exactly 5.0**. That is
consistent with a uniform placeholder rather than 1–5 evidence-weighted scoring. I did not audit the row-level
evidence, so this is flagged, not asserted.

### 4.4 Roadmap claims — HISTORICAL

`audit/ROADMAP-TASKS.csv` (`VERIFIED` by parsing: 101 rows, 11 columns
`task_id,phase,seat,title,objective,deliverable,acceptance,evidence_required,depends_on,estimate,priority`):

| Phase | Tasks |
|---|---:|
| P1 Reconnaissance & Full Inventory | 15 |
| P2 Micro-File Exhaustive Analysis | 20 |
| P3 Architectural Mapping & Interaction Tracing | 20 |
| P4 Upstream Provenance & Donor Repo Audit | 15 |
| P5 Code Quality, Mathematical Rigor & Tech Debt Scoring | 15 |
| P6 Scientific Research Paper Synthesis & Final Roadmap | 16 |
| **Total** | **101** |

By seat: b-core 12, b-provider 11, watchdog-1 14, watchdog-2 14, figcraft-a 14, figcraft-b 14, ui-watchdog 12,
strip-watchdog 10 = 101. Priority: P1 46 / P2 55. Every row's `evidence_required` is
"command output, path:line, or official URL+date"; every estimate "15-40 min".

Paper §7.3 (dump L1199–1203) gives the four waves; Wave 1 = the three P0s, Wave 2 = the nine P1s + design-artifact
tracking, Wave 3 = probe scripts / fail-closed skips / durable logs / DOM G-UI gates / baseURL / fonts / dev mock
/ timeouts / token-isolation / approval nonce / ASSETS-INDEX, Wave 4 = `.pyc`/scratch/patch cleanup, doc
consolidation, S25 and Tauri/installer decisions.

### 4.5 Ignored deliverables and the verifier's verdict

`audit/APPENDIX-IGNORED-DELIVERABLES.md` (dump 868–904) and `audit/IGNORED-DELIVERABLES.csv` (`VERIFIED`: 18
rows, columns `artifact,owner,kind,surface_id,bytes,sha256,tracked`): **18 gitignored artifacts, 22,009,234 B
(21.0 MiB)**, each with a full sha256, spread across A-design, A exports (8), B exports (8), B-design. The
appendix's own implication: *"the empirical corpus is 507 tracked files **plus** this 18-file ignored deliverable
class; assertions in the paper that say 'every file' refer to tracked files only"*, and its migration advice:
*"decide a policy — commit final design sources+exports (with size budget) or generate them reproducibly from
scripts and store only hashes/manifests."*

`audit/VERIFICATION.md` (dump 1715–1805) is the independent verifier's report. Verdict: **PASS-WITH-NOTES**,
with `507/507` manifest rows/hashes/quality rows, `25/25` spot-checked `path:line` citations resolving, and
"product-file changes = 0". Its own recorded open items:

1. figcraft-a `FILES.md` mid-write → resolved to 103/103 at final poll.
2. `QUALITY-MATRIX.csv` → re-merged to 507 rows (overall mean **3.752**).
3. *"`WAVE-SHEET.md` still says '120 tasks'"* — **stale in the delivered tree**: the dump's WAVE-SHEET (line 56)
   already says *"101 tasks across 8 seats / 6 phases"*, and the Lead addendum claims it was corrected. So the
   verifier's note 3 is itself out of date relative to the dump it ships with.
4. Paper language breakdown incomplete (35,540 of 36,802 itemised) — confirmed arithmetically above.
5. `audit/VERIFICATION.md` unwritten → it exists, plus a Lead addendum (dump 1789–1804).
6. Honest coverage number: *"507/507 files have `FILES.md` records and 507/507 quality rows … Trace stays
   honestly partial (43 WIRED-AND-WORKS hops; 270 PENDING, 6 MISSING of 319) and e2e keeps 14 explicit skips —
   no fabricated passes."*

---

## 5. What the current source supersedes, and what stays useful

### 5.1 Version / release truth chain — the single most important correction

| Layer | Version | Files | Evidence |
|---|---|---:|---|
| Audit paper subject | `v1.0.0`, HEAD `3988622` | **507** | paper L4, register L3 |
| Dump snapshot | sealed `1.1.0` (`87079ce`), tree on `1.2.0-dev` | **635** | README L5-7; `docs/RELEASE-EVIDENCE.md` |
| Current HEAD | `1.2.0-dev` (pre-release, unsealed) | **823** | `web/package.json:4`, four crate manifests, `README.md:1-7` |

The dump's `README.md` is still HEAD's `README.md` (byte-identical) and it is explicit:

> *"**Version truth:** this README documents the sealed **1.1.0** release. The working tree is on an unreleased
> **1.2 pre-release** source line … No 1.2 feature is released, built into an artifact, tagged, or end-to-end
> verified; do not present 1.2 work as current."*

`docs/RELEASE-EVIDENCE.md` (identical at HEAD) adds: sealed 1.1.0 = `87079ce`; recorded working HEAD `889b779`
(7 commits ahead, dirty); *"No 1.2 tag. Existing tags: `hivemind-1.0.0`, `hivemind-1.1.0`, `v1.0.0`"*;
*"**No 1.2 artifact**"*; manifest still the sealed 1.1.0 record.

**Therefore:** any sentence of the form "the project is at 1.0.0 / 507 files / 233 tests" is **wrong about HEAD**
and must be dated. Conversely "1.2 is released" is also wrong — 1.2 is an unsealed dev line with no artifact,
no tag, no release-gate PASS.

### 5.2 Superseded numbers

| Claim from the dump | Historical value | Current reality | Status |
|---|---|---|---|
| Tracked files | 507 (audit) / 635 (dump tree) | **823** (`git ls-files`) | SUPERSEDED |
| Backend tests | 233 passed / 1 ignored | **475 passed / 2 ignored** recorded in `docs/RELEASE-EVIDENCE.md` (1.2 tree); my static count = **477 `#[test]`-ish attrs / 1 `#[ignore]`** | SUPERSEDED |
| Frontend tests | 46 | **179 passed**, recorded; my static `it()/test()` count = **179** (exact match) | SUPERSEDED |
| e2e | 79 passed / 14 skips | **106 passed / 0 failed / 166 skipped** recorded (guarded run) | SUPERSEDED |
| Source LOC | 36,802 | not recomputed | UNVERIFIED |
| Cargo / npm packages | 405 / 147 | lockfiles changed (`backend/Cargo.lock` not even in dump) | UNVERIFIED |
| Git history | 23 commits, 2 authors | local clone = **1 squashed commit**; upstream history unavailable | SUPERSEDED/UNVERIFIABLE |
| CODEMAP docs | 15 files in dump | 18 at HEAD (+`08-FILE-CATALOG.md`, `_data/inventory.csv`, `_data/inventory.json`) | SUPERSEDED |
| Audit diagrams | 6 `.mmd` only in dump | 6 × (`.mmd`+`.png`+`.svg`) at HEAD | SUPERSEDED |
| Trace ladder | 43 W&W / 270 PENDING / 6 MISSING | not recomputed | UNVERIFIED |

### 5.3 Finding-by-finding status at HEAD (evidence in §2.1–2.4)

**Superseded / fixed, verified at HEAD (9):** A-F01, A-F02, A-F03, A-F04, A-F05, A-F08, A-F11, A-F20, A-F21 —
plus partial: A-F23 (nonce now documented as a 1.2 additive field), FA-F5 (`*.pyc` ignore rule added, the 6 old
`.pyc` files still tracked).

**Still open, verified at HEAD (12):** A-F06 (silent `Decimal::ZERO`), A-F07 (duplicate majors), A-F09 (82 tracked
`runtime-test` files), A-F12/FA-F1 (figcraft gitignored), A-F13 (probe scripts), A-F14 (`E2E_APP_READY` guard),
A-F15 (`test-results` wipe), A-F16 (lower-bound thresholds), A-F17 (tests write into `docs/`), A-F18 (baseURL
`:8080` default), A-F19 (no bundled fonts), A-F22 (hex colour in `index.html`).

**Not rechecked (6):** A-F10, A-F24, FA-F2, FA-F3, FA-F4, FA-F6.

Reconciliation check: 9 fixed + 12 still open + 2 partial (A-F23, FA-F5) + 6 not rechecked + 1 (FA-F1, counted
with its duplicate A-F12) = **30**.

**Still useful, unchanged by time:** the invariants list (no-fake-success, single-use approval, at-most-once side
effect, terminal-state guard, one terminal `RUN_STATUS`, unknown-is-not-zero, bounded replay) — these are
contract properties, not statistics; the refusal/ACK-before-mutation discipline; the eight-vector jail fail-closed
claim; the evidence-rules methodology (§3.5); the audit's own honesty markers ("no paid-provider run",
"pixel fidelity needs human review", "43/319 traced"); the 30 finding *titles* as a defect taxonomy to re-run
against HEAD; and the 18-file ignored-deliverables census with full sha256s (the only surviving record of those
assets' hashes, since they are gitignored and absent from the dump's tracked manifest).

### 5.4 What the dump cannot tell you

- It cannot verify its own hashes: `FILE-MANIFEST.csv`, `QUALITY-MATRIX.csv`, `APPENDIX-FILE-CENSUS.md` and
  `backend/Cargo.lock` are **all absent** from the dump. Every hash/package count in the audit is therefore
  `READ`, never `VERIFIED`, from the dump alone.
- It cannot show the 18 ignored deliverables' bytes (they are gitignored) — only their hashes.
- It cannot show 49 bodies at all (placeholders).
- It cannot name its own commit (§1.5).

---

## 6. UNVERIFIED and stale-risk register

| # | Risk | Class | What would close it |
|---|---|---|---|
| 1 | **Exact snapshot commit unknown.** Local history is squashed to one commit; the dump's own docs reference `3988622`, `6a2b1c6`, `87079ce`, `889b779`, none of which exist in this clone | UNVERIFIED | Fetch `rodneydanger1/hivemind` history; identify the commit whose tree = these 635 paths and 79 blobs |
| 2 | **"636 FILE entries" is a miscount** (substring, not sections) | VERIFIED correction | already closed here; use 635 |
| 3 | **49 placeholder bodies** (28 `[Empty file]`, 20 `[Binary file]`) — 7.7 % of the snapshot has no content | VERIFIED limitation | re-ingest those paths from git or from the current tree |
| 4 | **`plans/waves/W1-CORE-RECOVERY.md` is `[Binary file]` in the dump but 10,001 B of Markdown at HEAD** — ingest lost text or the file was empty then | UNVERIFIED | inspect the file at the snapshot commit |
| 5 | **Four control chars (`0x0B`×1, `0x0C`×3) corrupt identifiers** in `plans/foresight/RUN-3.md` (`verify_tool_grant`, `finish_timeout_wait`, `finish_cancelled_wait`, `final_status`) | VERIFIED defect | none needed for reading; repair before quoting code |
| 6 | **Audit corpus is `v1.0.0`-era but ships byte-identical at HEAD** — readers will silently take 507-file / 233-test statements as current | HIGH staleness risk | always date them; §5.1 table |
| 7 | **Two contradictory quality tables** in the shipped audit (register 4.50/4.29/4.37/4.33/4.29/3.97 over 159 files vs paper/JSON 4.43/4.25/4.32/4.15/4.11/3.84 over 184 of 188) | VERIFIED inconsistency | recompute from `QUALITY-MATRIX.csv` (exists only at HEAD) |
| 8 | **Suspect uniform quality scores** (figcraft-a code rows exactly 4.0/4.0/4.0/3.0/3.0/3.0; figcraft-b security exactly 5.0) | SUSPECT | audit the per-row evidence cells in the seat `QUALITY.csv` files |
| 9 | **Verifier's own NOTE 3 is stale** (`WAVE-SHEET` "120 tasks" vs the dump's "101 tasks") | VERIFIED inconsistency | cite ROADMAP-TASKS.csv (101) as authority |
| 10 | **Test counts are recorded, not reproducible here** — no build/run was performed (read-only task) | UNVERIFIED | run `cargo test --workspace --locked` / `npm test` / guarded e2e |
| 11 | **`cargo tree -d` never run** (A-F07 gate) — duplicate majors confirmed only by grepping `Cargo.lock` | PARTIALLY VERIFIED | `cargo tree -d` |
| 12 | **Trace-ladder numbers (43 W&W / 270 PENDING / 6 MISSING of 319)** not recomputed | UNVERIFIED | re-run `tools/qa/verify-harness.mjs` + recount `docs/trace/controls.json` |
| 13 | **Dependency/licence claims** (`405`/`147` packages, "every licence claim not fetched is UNVERIFIED") | UNVERIFIED in the audit's own words | refetch official registries with dates |
| 14 | **6 register findings not rechecked** (A-F10, A-F24, FA-F2/F3/F4/F6) plus **6 seat-level findings** never consolidated (b-provider `H-3`, `H-8`, `H-10`; b-core `F-04`, `F-05`; watchdog-2 `S5`) | UNVERIFIED | targeted pass against HEAD |
| 15 | **`runtime-test/` count coincidence**: 82 tracked paths in the dump *and* 82 at HEAD, but 36 of HEAD's 82 are newer — the same number does **not** mean "unchanged" | VERIFIED trap | compare path sets, not counts |

---

## 7. Top-10 most useful things in this dump

1. **A complete defect taxonomy that was never re-run.** 30 consolidated + 58 seat-level findings, each with
   `path:line` evidence, from a real `v1.0.0` snapshot. Re-running those titles against HEAD is cheap and
   directly actionable — §2.1–2.4 already shows 9 fixed and 11 still open.
2. **The invariants catalogue** (paper §6.2 + `docs/CODEMAP/02-ARCHITECTURE.md`): no-fake-success, single-use
   nonce + `args_sha256` + `decision IS NULL` CAS, at-most-once side effect via preflight→grant CAS, terminal-state
   guard, one terminal `RUN_STATUS` owner, unknown-is-not-zero, bounded replay. Best short statement of what the
   system must not break.
3. **The evidence-rules methodology** (paper §2.3, WAVE-SHEET): every claim needs `path:line`, command output, or
   official URL + fetch date; `UNVERIFIED` must name the proving action; the verifier must confirm no product file
   changed. Directly reusable for any future audit of this repo.
4. **A verified, hash-anchored census of the 18 gitignored design deliverables** (22,009,234 B, full sha256) —
   the only record of those assets' hashes anywhere, since they are excluded from git *and* from the tracked
   manifest.
5. **The exact historical P0 set and its resolution path** — A-F01/A-F02/A-F03 fixed at `6a2b1c6`, giving a
   worked example of "audit finding → wave → fix → live proof" for this codebase.
6. **A concrete, current hygiene backlog with evidence I re-verified at HEAD**: silent `Decimal::ZERO`
   (`accounting.rs:334`), duplicate majors (`thiserror`, `tower-http`), 82 tracked `runtime-test` artifacts,
   figcraft gitignore policy, lower-bound harness thresholds (`verify-harness.mjs:102-105`), tests writing into
   `docs/`, hex colour in `index.html:7`.
7. **The 101-task / 6-phase / 8-seat roadmap as a ready-made work breakdown**, with per-task acceptance and
   evidence requirements — still one of the few complete plans in the repo.
8. **Documented, still-live test-harness traps**: `E2E_APP_READY` skip-as-green, wiped `test-results/`, baseURL
   `:8080` vs `:8799`, no artifact-identity preflight. These silently convert "green" into "not run".
9. **Six architecture diagrams in Mermaid** (topology, data flow, module deps, state inventory, lineage, migration)
   that state *declarations only* and say so — useful as the canonical historical topology, with the caveat that
   diagram 04's note ("transitions are enforced by run code paths, not enumerated in state.rs") is the honest part.
10. **A dated honesty trail** — "no paid-provider run", "43/319 traced", "pixel fidelity needs human review",
    "no fabricated passes" — embedded in the artifacts themselves. It is a good model for how to record limits,
    and it is also why the numbers must be read as of `2026-11-06`, not as of HEAD.

---

## Appendix A — Method and reproduction

Everything above is derived by parsing the dump as text and comparing against the working tree. No product code
was edited; the only file written inside the repository is this document.

1. Parse sections with the delimiter `=`×48 / `FILE: <path>` / `=`×48; each body runs to the next such delimiter.
   Counts: 635 sections, 635 unique, 0 duplicates, 0 empty.
2. Parse the ASCII tree (dump lines 2–768) independently: 132 directory nodes, 635 file nodes, path set identical
   to the section set.
3. Compare path sets against `git ls-files -z` (823 records; NUL-separated to avoid PowerShell redirect encoding
   damage — a naive `git ls-files > file.txt` produced a UTF-16 file that mis-parsed as 1,647 lines).
4. For each of the 635 common paths: extract body, strip trailing blank padding, sha256 it, and sha256 the
   CRLF-normalised, BOM-stripped working-tree file; compare → 556 identical / 79 changed.
5. Parse `ROADMAP-TASKS.csv` (101 rows), `IGNORED-DELIVERABLES.csv` (18 rows), `QUALITY-*.json`,
   `QUALITY-SUMMARY.json`; recount phases/seats/priorities rather than trusting prose.
6. Re-check each finding at HEAD with `Select-String` on the cited files (evidence quoted inline in §2).
7. Static test counts (`Select-String` over `backend/crates/*/{src,tests}/*.rs` and `web/tests/*.ts`) are labelled
   as static approximations, never as executed results.

## Appendix B — Canonical dump line anchors

| Artifact | Dump `FILE:` line | Body range |
|---|---:|---|
| `README.md` | 771 | 773–835 |
| `audit/APPENDIX-IGNORED-DELIVERABLES.md` | 866 | 868–904 |
| `audit/FINDINGS-REGISTER.md` | 906 | 908–983 |
| `audit/HIVEMIND-AUDIT-PAPER.md` | 985 | 987–1235 |
| `audit/IGNORED-DELIVERABLES.csv` | 1237 | 1239–1260 |
| `audit/QUALITY-CODE-STATS.json` | 1262 | 1264–1346 |
| `audit/QUALITY-SUMMARY.json` | 1379 | 1381–1479 |
| `audit/ROADMAP-TASKS.csv` | 1481 | 1483–1587 |
| `audit/ROADMAP.md` | 1589 | 1591–1711 |
| `audit/VERIFICATION.md` | 1713 | 1715–1805 |
| `audit/WAVE-SHEET.md` | 1807 | 1809–1867 |
| `audit/diagrams/01-system-topology.mmd` | 1869 | 1871–1911 |
| `audit/diagrams/02-data-flow.mmd` | 1913 | 1915–1939 |
| `audit/diagrams/03-module-dependencies.mmd` | 1941 | 1943–1960 |
| `audit/diagrams/04-state-inventory.mmd` | 1962 | 1964–1975 |
| `audit/diagrams/05-dependency-lineage.mmd` | 1977 | 1979–2013 |
| `audit/diagrams/06-migration-target.mmd` | 2015 | 2017–2026 |
| `audit/inventory/CODE-STATS.md` | 2028 | 2030–2168 |
| `audit/inventory/DIRECTORY-CENSUS.md` | 2234 | 2236–2383 |
| `docs/CODEMAP/02-ARCHITECTURE.md` | 45957 | 45959–46017 |
| `plans/foresight/RUN-3.md` | 75411 | 75413–75621 (holds the 4 control-char corruptions) |

> **HISTORICAL reminder.** Sections 2, 3, 4 and every quoted number describe commit `3988622` (audit) or the
> 635-file `1.2.0-dev` snapshot. They are **not** statements about HEAD `aa16e13`. Where §2 says "still open",
> that verdict is mine, taken 2026-11-07 against HEAD, and is evidence-cited — but it is a read, not a run.
