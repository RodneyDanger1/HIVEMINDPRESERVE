# REPORT — SEAT S3: UI State & Replay

**Program:** HiveMind 1.0 · **Seat:** S3 (UI-State/Replay) · **Mode:** READ-ONLY audit/verification
**Repo:** `C:\Users\Riley\Desktop\hiverouter-v1` · **Observed HEAD:** `5c69b66` (“…replay arm, 16-state parity, settings”)
**Slice:** `web/src/flow/{Canvas,graph}.ts(x)`, `web/src/state/{store,agentState,strings}.ts`, `web/src/panels/SettingsPanel.tsx`, `web/src/panels/Chat.tsx` (missing — see D8), `web/src/chat/ChatCore.tsx`, `web/src/api/{contract,index,client}.ts`, `web/tests/*` (8 files).
**Inputs:** `audit/HIVEMIND-AUDIT-PAPER.md` (244 L) + 4 official DeepSeek docs pages + `audit/diagrams/04-state-inventory.mmd` (9 L). Read fully in the mandated order; see `READLOG.md`.
**Gate cycles:** `GATE-CY-1/2/3.md` (recon→plan→lead each; no cycle failed → not blocked).

## 1. VERIFIED FACTS (source-level, path:line)

| # | Fact | Evidence |
|---:|---|---|
| V1 | **16-state vocabulary incl `interrupted` is implemented** (paper A-F03 parity fixed at this HEAD). Four PDF dot states preserved. | `web/src/state/agentState.ts:9-26` (array), `:30` (dots), `:166-183` `strings.ts` labels incl `interrupted:182`; shipped bundle contains `interrupted` ×5 |
| V2 | **Replay arm is implemented**: `HISTORY_SNAPSHOT` is stored and projected through `buildReplayTasks`→`buildNodes`/`buildEdges`→ReactFlow, with a replay banner. Paper A-F02 (dead replay) is stale at this HEAD. | `store.ts:385-393`; `Canvas.tsx:113-136, 183-189`; `graph.ts:93-131`; bundle contains `HISTORY_SNAPSHOT` ×4 |
| V3 | **Replay is pre-validated for malformed/unknown nodes** (dropped, orphan parents detached), so a bad snapshot cannot crash the canvas. | `graph.ts:93-131` (`isAgentState` guard line 110, orphan fix lines 123-129) |
| V4 | **Settings contract carries model + thinking + effort + key**: result payload required provider/model/key_present, optional `thinking`/`reasoning_effort`; store maps them null-safe; panel renders all five. | `contract.ts:242-248`; `store.ts:78-80, 346-361`; `SettingsPanel.tsx:65-78`; optional absent → “Not reported” (`strings.ts:157`, panel 71/74) |
| V5 | **Exactly one SETTINGS_STATUS per Settings open**, with failure refusal attributed to the probe's own new toast, not `refusals[0]`. Paper A-F05 fixed. | `store.ts:662-684`; test `web/tests/store.test.ts:112-137` |
| V6 | **Production bundle excludes the dev mock**: no `dev-mock-session-token`, no `MockApiClient is dev-only`; guard is DEV+VITE_UI_MOCK. | `api/index.ts:14-19`; dist `index-DzPE48Wh.js` sha `1A0BBFF3…CBDCFF`, markers 0 |
| V7 | **Wire client: ACK-only resolve, typed ERROR rejection, command timeout 15 s, hello handshake timeout 8 s, hello replay on reopen** — no fake success path. | `client.ts:283, 289-294, 339-379, 389-419, 443-452`; tests `apiClient.test.ts` 5/5 |
| V8 | **Contract sizes pinned**: 14 commands / 21 events / 45 documented refusal codes; NODE metadata optional (no invention); approval nonce optional. | `contract.ts:102-117, 294-316, 332-378, 208-229`; `contract.test.ts` |
| V9 | **Frontend tests: 46 tests in 8 files** (source `it(` count = 46; cited log says passed). Not re-run by this seat (read-only). | `web/tests/*`; `docs/reports/FINAL-REPORT.md:37` |
| V10 | **Paper-vs-code staleness**: the audit paper (checkpoint 3988622) still lists A-F02/A-F03 as P0 defects; current HEAD 5c69b66 resolves them (V1/V2). Residual defects are D1-D3 below. | paper §4.2 vs files above |

## 2. DEFECTS (severity, location, consequence)

| ID | Sev | Defect | Evidence | Consequence / fix |
|---|---|---|---|---|
| **D1** | **P1** | **`failed` is not treated as terminal** although the contract declares five terminal states incl `failed`. | `agentState.ts:111-118`; contract terminal set per `04-state-inventory.mmd:5`; effects: `store.ts:434-441, 550-557`; `store.ts:254` streaming clear | Failed run still considered active → spawn/rewire/cancel local gates pass, server must refuse; stale streaming indicator. Fix: add `'failed'` to `isTerminal`; pin with tests (`isTerminal('failed') === true`, interrupted too). |
| **D2** | **P1** | **Replay is not read-only**: replay nodes are `draggable/selectable` and clicks mutate live selection. | `Canvas.tsx:5-6` claims read-only; `Canvas.tsx:118-122` uses `buildNodes`; `graph.ts:64-72` sets `draggable/selectable`; `Canvas.tsx:146` live `onNodeClick` | User can drag replay nodes and change live `selectedTaskId` during replay → contradiction of S10 “read-only replay”, potential confusion. Fix: replay-specific projection with `draggable:false, selectable:false`, `onNodeClick` noop; test the flags. |
| **D3** | **P1** | **Replay payload shape is not pinned by the contract or tests**: `graph: unknown`; parser silently degrades to empty canvas; no unit/integration test for replay. | `contract.ts:235-239`; `graph.ts:93-100` (silent `{}`); `Canvas.tsx:183-189` (banner still shown); no replay test in `web/tests/*` | Any backend snapshot-shape change silently renders an empty replay with no named refusal. Fix: define/validate `HistorySnapshotGraph` (nodes array with required fields), return a named client refusal (banner) when malformed, add tests (valid/empty/malformed/unknown-state/orphan-parent). |
| **D4** | P2 | Settings provider/model fallback uses `keyUnknown` (“Not probed by the server”) instead of `notReported`. | `SettingsPanel.tsx:67,69` vs `strings.ts:157` (thinking/effort use `notReported` at 71/74) | Misleading copy only when server returns empty values (decoder requires non-empty so low impact). Fix copy. |
| **D5** | P2 | **HTTP requests have no timeout/abort**: `/api/session` fetch can hang forever before handshake timeout starts; `/health` likewise. | `client.ts:235-238` (fetch), `:282` (waitForOpen starts later), `:389-419`; `health.ts:18-21` | UI can sit in “Connecting”/health pending indefinitely; no named refusal. Fix: `AbortSignal.timeout` + named `E_WS_DISCONNECTED`/`E_SCHEMA_INVALID`, tests with never-resolving fetch + fake timers. |
| **D6** | P2 | **Mock exclusion relies on tree-shaking** of a dead DEV branch; static import + re-export remain. Current bundle excludes it, but no automated guard. | `api/index.ts:8,12,14-19`; bundle probe (V6) | A future non-conditional use can silently ship the mock. Fix: `await import('./mock')` in a DEV-only branch, or add a build assertion/test. |
| **D7** | P2 | **Test gaps around this seat's invariants**: no tests for replay, `failed`/`interrupted` terminality, thinking/effort propagation, mock exclusion, HTTP timeout. | `web/tests/*` (all 8 read); `agentState.test.ts:14-17` asserts `>=13` not 16; `store.test.ts:112-137` settings test omits thinking/effort | Regressions in D1-D6 undetected. Fix: add the discriminating tests listed in §4. |
| **D8** | P3 | **Assigned path `web/src/panels/Chat.tsx` does not exist**; real chat component is `web/src/chat/ChatCore.tsx`. | `Get-Item web/src/panels/Chat.tsx` → MISSING (READLOG row 13) | Seat-assignment/README defect, not product. Fix assignment in wave sheet. |
| **D9** | P3 | Composer clears the draft before the send/refusal outcome is known → refused prompt loses typed text. | `ChatCore.tsx:51-56, 131-137` | UX honesty: toast explains refusal but text is gone. Fix: clear only after accepted (submitPrompt returns accepted/refused). |
| **D10** | P3 | Demo/`seed` strings ship in the production bundle (label only; no fake data path). | `strings.ts:239-249, 54`; bundle probe `DEV MOCK` ×1 | Hygiene only; no fabricated run/state. Fix: move seed copy into the dev mock module. |
| **D11** | P3 | Store imports from `../api/contract` three times; minor duplication/ordering. | `store.ts:8-15, 22, 27` | Maintainability. Fix consolidation. |
| **D12** | P3 | `interrupted` has no inbound transition in the declared matrix (running/streaming→interrupted), despite being a terminal contract state; `canTransition` is unused in product paths. | `agentState.ts:87-104` | Matrix/documentation contradicts possible live sequence; no runtime enforcement. Fix matrix + document enforcement ownership (server). |

## 3. FILE-BY-FILE PLAN (all `forgewright-plan` required sections)

### 3.1 Outcome
A UI where (a) a failed run is treated as terminal by every local gate; (b) HISTORY_SNAPSHOT replay renders read-only (no drag/selection mutation) and reports a named state when a snapshot is malformed; (c) Settings surface always shows model/thinking/effort/key with correct unknown-vs-missing copy; (d) dev mock can never leak into the production bundle; (e) session/health HTTP calls fail by name within a bounded time. Users can trust replay and sees no misleading active-run controls.

### 3.2 Current behavior (verified)
- 16 states exist (V1) but terminality omits `failed` (D1) → gates treat failed as active.
- Replay renders (V2/V3) but nodes are interactive and the graph shape is unpinned/untested (D2/D3).
- Settings fields all render (V4/V5); copy fallback bug (D4).
- Mock excluded in the current artifact (V6) but by tree-shaking only (D6).
- Command/handshake timeouts exist; HTTP fetch timeouts do not (V7/D5).
- 46 tests pass per cited log; none cover D1-D6 (D7).

### 3.3 Decisions (with rejected alternatives)
1. **`failed` is terminal locally.** Rejected: keep failed “active” for retry affordance — there is no retry command in the frozen contract, so an active failed run only enables commands the server will refuse (fake affordance). Migration: pure UI predicate change; no wire change.
2. **Replay read-only projection, not a mode toggle on live nodes.** Rejected: hiding handles via CSS — drag would still mutate internal positions and clicks still select. Ownership: web only.
3. **Pin the snapshot shape client-side rather than keep `unknown`.** Rejected: trusting backend blindly — replay silently empties on drift, hiding a real failure. Additive validation; malformed → named refusal/banner.
4. **HTTP timeout with `AbortSignal.timeout`** (browser-native, no dependency). Rejected: manual `setTimeout`+race — more code, same result.
5. **Mock via dynamic import in DEV branch** (or build-time assertion). Rejected: relying on tree-shaking — already load-bearing today with no guard.
6. **No contract/wire change in this seat’s remediation.** All fixes are client-side; backend remains authoritative (jail, terminality, graph). If the contract owner wants `failed` terminality clarified, that is a doc/contract task, not a UI hack.

### 3.4 File-by-file change (exact paths, functions, edge cases, error paths)
- `web/src/state/agentState.ts` — add `'failed'` to `isTerminal` (line 111-118); decide/document `interrupted` inbound transitions in `STATE_TRANSITIONS`; keep `isRunActive` semantics; tests for failed/interrupted terminal.
- `web/src/flow/graph.ts` — export `buildReplayNodes`/`buildReplayEdges` (or `buildReplayGraph(graph): {tasks,nodes,edges,malformed}`); set `draggable:false, selectable:false` for replay nodes; keep worker/label fallback (no invention) but emit a `malformed` flag when `graph` is present but unparsable; keep unknown-state skip.
- `web/src/flow/Canvas.tsx` — replay branch uses read-only projection + `onNodeClick` noop when `history.replayGraph`; show `STRINGS` banner/error when replay tasks empty but graph present; do not call live `selectTask` in replay.
- `web/src/state/store.ts` — no API change needed for D1 (predicate), ensure `RUN_STATUS` `streamingRunId` clears on `failed` via corrected predicate; optionally store replay parse status; consolidate contract imports (D11).
- `web/src/api/contract.ts` — add exported `HistorySnapshotGraph` typing + `parseReplayGraph(unknown)` validating `{nodes:[{task_id,state,...}]}`; `HistorySnapshotPayload.graph` stays `unknown` on wire but parsed once. Edge: unknown future state must not throw (skip + report).
- `web/src/api/client.ts` — session fetch: `signal: AbortSignal.timeout(...)`, map `AbortError`→`E_WS_DISCONNECTED` with named message; keep handshake timeout.
- `web/src/api/health.ts` — same timeout + named refusal; keep unknown-not-healthy semantics.
- `web/src/api/index.ts` — dynamic DEV mock import, or add a build-time guard test (D6).
- `web/src/state/strings.ts` — provider/model fallback uses `settings.notReported`; move `seed` strings into mock module (D10); add replay-malformed copy if new banner.
- `web/src/panels/SettingsPanel.tsx` — use `notReported` for provider/model; thinking/effort unchanged.
- `web/src/chat/ChatCore.tsx` — clear draft only on accepted submit (requires `submitPrompt` return accepted/refused; store API additive) (D9).
- `web/tests/*` — add: `graph.replay.test.ts` (valid snapshot→nodes/edges, malformed→`{}`+flag, unknown state skipped, orphan parent detached, replay nodes non-draggable); `agentState.test.ts` additions (16 exact, `isTerminal('failed')`, `isTerminal('interrupted')`, transitions); `store.test.ts` additions (thinking/effort mapping, HISTORY_SNAPSHOT stored, failed run gates refuse); `mock-exclusion` assertion (prod build or static guard test); HTTP-timeout tests with fake fetch + fake timers.

### 3.5 Failure modes
- Malformed/未来 snapshot: replay shows a named banner (not silent empty canvas) and keeps live graph untouched.
- `failed` event with stale streaming message: streaming cleared; controls show disabled reasons (`STRINGS.controls.disabledNoRun`).
- Session fetch hangs/DNS error: timeout yields `E_WS_DISCONNECTED` toast + retry; no spinner-forever.
- Health hangs: Settings shows unknown + toast, never “healthy”.
- Mock accidentally imported in prod: build/test assertion fails (D6 guard).
- Concurrent replay selection: replay clicks no-op; nothing races live state.

### 3.6 Tests (discriminating, nameable)
Command (when write/run authority exists): `cd web; npm.cmd run typecheck; npm.cmd test; npm.cmd run build`.
New tests: `graph.replay.test.ts` (5 cases above), `agentState.test.ts` additions (failed/interrupted/16), `store.test.ts` additions (thinking/effort, replay store, failed gates), `client.timeout.test.ts`/`health.timeout.test.ts` (never-resolving fetch rejects by name), bundle guard (prod build grep for `dev-mock-session-token`). Current cited gate: 46 tests / 8 files pass (`FINAL-REPORT.md:37`), typecheck+build exit 0 — **not re-run here**.

### 3.7 Rollback
Read-only seat: no rollback needed. For remediation wave: every change is additive/reversible — D1 one-line predicate (revert line + tests), D2 projection flag (revert), D3 parser (remove), D5 timeout signals (remove), D6 dynamic import (restore static import). No DB/migration/wire changes; rollback is `git revert` of the remediation commit and the old tests still pass except the new invariants (which would fail — that is the intended RED).

### 3.8 Out of scope / blockers
- Out of scope: backend provider wiring (A-F01), e2e execution/browser, Rust rebuilds, keys/paid calls, S25 diff surface, FigCraft assets, runtime DB.
- Blockers for full verification (recorded, not hidden): read-only mandate forbids running tests/build/browser; runtime replay e2e claim in commit `5c69b66` needs the e2e seat; tree not fully quiescent (`M web/package.json` outside slice); assigned `panels/Chat.tsx` missing.

## 4. TESTS/COMMANDS (what was and was not run)
- **Not run** (read-only): `npm.cmd run typecheck`, `npm.cmd test`, `npm.cmd run build`, Playwright/e2e, live server probes, paid model calls.
- **Cited evidence:** `docs/reports/FINAL-REPORT.md:37` (46 tests / 8 files, typecheck+build exit 0); source `it(` count = 46; bundle probe outputs in `EVIDENCE.md §2`.
- **Commands recommended for the remediation wave** (after the freeze lifts): the command block in §3.6 + `cd web; npm.cmd run build` then grep dist JS for mock markers; e2e replay scenario: open history tab, scrub, assert replay banner + non-empty `.react-flow__node`, drag attempt does not move node / click does not change live selection.

## 5. READ-ONLY CONSTRAINT RECORD
No product/git/config edits; writes only under `plans/seats/SEAT-03/` (uncommitted, as required). No keys accessed, no paid calls. Incidental note: `web/package.json` was already modified in the worktree before this seat started (not touched by this seat).

## 6. UNVERIFIED (must not be treated as PASS)
1. **Runtime replay rendering** (browser): source says it renders; no browser run in this seat → `UNVERIFIED` (e2e seat / Playwright).
2. **Replay read-only behaviour under real pointer events** (D2) — source-analysis only.
3. **16-state parity under real server events**: enum/labels/bundle verified, but no live RUN_STATUS stream observed; `interrupted` inbound transition/`failed` terminality at runtime `UNVERIFIED`.
4. **Settings against a live key**: model/thinking/effort rendering verified in source/store; actual server values with a configured key `UNVERIFIED` (no key access).
5. **HTTP timeout absence consequence**: source-verified missing timeouts; live hang behaviour `UNVERIFIED`.
6. **Mock exclusion for future builds**: verified for the current dist artifact; whether every future build excludes it (tree-shaking) `UNVERIFIED` → propose build guard (D6).
7. **E2E replay claim in commit message** (`5c69b66`): not reproduced by this seat.
8. **Paper-vs-code reconciliation**: paper findings A-F02/A-F03 are stale vs current code; final paper update is Lead's task (`UNVERIFIED` whether other seats reconcile it).

---
Receipt to Lead: see final chat message (≤20 lines).
