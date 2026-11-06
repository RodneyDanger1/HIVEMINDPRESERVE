# W4 — REPLAY / STATE BROWSER PROOF (plan)

Seat: **W4 (Replay/State browser proof)** · WRITE-AUTHORIZED, disjoint scope: `web/tests/**`, `e2e/**` + this plan doc.
No backend / `web/src` product / tools / docs / audit edits; no key access, no paid calls, no commits (reserved to Lead).
Checkpoint observed: HEAD `0dfe23b` (2026-11-06 13:45:20 −06:00); `web/dist` rebuilt 13:45:09 (current source build, markers `HISTORY_SNAPSHOT`×4, `interrupted`×5, `read-only`×1).

---

## Recon (forgewright-recon)

- **Target behavior:** a `HISTORY_SNAPSHOT` event must project the stored graph into React Flow nodes/edges (valid nodes only; malformed/unknown nodes skipped; orphan parents become roots); replay must be non-draggable, non-selectable and must not mutate live selection; the agent vocabulary must be the exact 16 states and `isTerminal('failed')` must be true.
- **Entry points:** `web/src/flow/graph.ts` (`buildReplayTasks` 94-132, `buildNodes` 58-74 interactive flag, `buildEdges` 76-86), `web/src/flow/Canvas.tsx` (replay branch 113-122, `nodeTypes`/handlers 134-158, guarded `onNodeClick` 146-150), `web/src/state/agentState.ts` (16-state array 9-26, `isTerminal` 111-119).
- **Callers:** `Canvas.tsx:113-122` calls `buildReplayTasks(history.replayGraph?.graph)` then `buildNodes(replayTasks, null, noopSelect, false)` + `buildEdges(replayTasks)`; `store.ts:385-393` stores `HISTORY_SNAPSHOT {revision, graph}`.
- **Tests + commands:** `web/tests/*` (Vitest, node env; 46+ tests, incl. `refusalParity.test.ts`); `e2e/playwright.config.ts` (installed Chrome/Edge channels only, no browser download, baseURL from `E2E_BASE_URL`, no `webServer`); `e2e/helpers/{app,matrix,ws-tap,evidence}.ts`; Playwright 1.65.0 + Chrome present at `C:\Program Files\Google\Chrome\Application\chrome.exe`.
- **Data flow (happy):** server `HISTORY_SNAPSHOT` envelope → `parseEventFrame` → store `history.replayGraph` → `buildReplayTasks` → nodes/edges → ReactFlow replay view + banner. **Failure path:** malformed `graph` → `buildReplayTasks` returns `{}` → replay renders empty (no crash, no fabricated state).
- **Current behavior verified in source (before writing tests):**
  - `buildNodes(tasks, selected, onSelect, interactive = true)` sets `draggable: interactive, selectable: interactive` (`graph.ts:58-74`); replay call passes `false` (`Canvas.tsx:119`) ⇒ non-draggable/non-selectable.
  - `onNodeClick` is guarded by `if (!history.replayGraph)` (`Canvas.tsx:146-150`) ⇒ clicks during replay do not call `selectTask`.
  - `isTerminal` includes `'failed'` (`agentState.ts:111-119`); `AGENT_STATES` is exactly 16 values incl `interrupted` (`agentState.ts:9-26`).
  - **Not covered by any existing test** (all 9 `web/tests/*` read): replay projection, replay interactivity, `failed`/`interrupted` terminality, 16-state exactness. Existing `agentState.test.ts` only asserts `>= MIN_AGENT_STATE_COUNT` (13).
  - App fixture: **no live server is running** on `:8799`, `:8080`, `:5173`; a current `web/dist` exists. Playwright can serve that build deterministically itself and spoof the transport with `page.routeWebSocket` (Playwright 1.65 supports it), so browser execution is possible without touching product/backend/keys.
- **Unknowns / open evidence:** live server WS + real replay events (no server/key; owner: Lead/backend), and whether `web/dist` stays current for later runs (build freshness recorded).
- **Files to touch (this task only):** new `web/tests/graph.replay.test.ts`, `web/tests/state.parity.test.ts`; new `e2e/helpers/replay-harness.ts`, `e2e/specs/replay-state.spec.ts`; this plan doc.

## 1. Outcome
Replay/state invariants are executable knowledge: `npm.cmd test -- --run` fails if `HISTORY_SNAPSHOT` projection, replay read-only behavior, the 16-state list, or `isTerminal('failed')` regress, and a browser spec proves replay nodes are non-draggable/non-selectable and clicks do not mutate live selection.

## 2. Current behavior (evidence)
See Recon: `graph.ts:58-74/94-132`, `Canvas.tsx:113-122/146-150`, `agentState.ts:9-26/111-119`. Vitest currently has no replay test; browser replay was previously only claimed via the commit message. `web/dist` build exists but no server; the e2e config has no `webServer` and requires external server env (`E2E_BASE_URL`, `E2E_APP_READY`) for the existing live specs.

## 3. Decisions
1. **Vitest for the pure projection** (no DOM needed): `buildReplayTasks`/`buildNodes`/`buildEdges` are pure functions; test topology, filtering, orphan handling, interactive flag directly. Rejected: jsdom component test — jsdom is not installed and the pure functions carry the logic.
2. **Playwright for UI behavior** with a self-contained deterministic harness: serve the current `web/dist` from a Node static server in the spec and mock only the transport with `page.routeWebSocket`, pushing synthetic `HISTORY_SNAPSHOT`/`NODE_*` envelopes. Rejected: (a) requiring a live Rust server — not running, needs key/provider for real runs, non-deterministic; (b) fake global `WebSocket` injection — more invasive and unnecessary since Playwright's route API is transport-level; (c) skipping the browser test — browser execution is possible.
3. **No product edits.** The invariants are tested as shipped; a failing test would be reported RED to the Lead, not patched here (scope says web/src is out of bounds).
4. **Assert interactivity via React Flow DOM classes + behavior.** React Flow renders `selectable`/`draggable` classes on `.react-flow__node` from node flags (`@xyflow/react` dist index.js ~2345-2360, style.css:225-233). Replay: zero `.draggable`/`.selectable` classes; live: present. Behavior: clicking a replay node leaves the `aria-live` selection text and `.selected` count untouched.
5. **Pin the exact 16-state array** (order included) and the five terminal states; `>=13` assertions stay in the old test but are now backed by exactness.

## 4. File-by-file change
- **NEW `web/tests/graph.replay.test.ts`**
  - `buildReplayTasks` projects `{nodes:[{task_id,parent_task_id,state,rev}]}` → `TaskRecord` map (3 valid nodes from a snapshot containing null/string/missing-id/unknown-state entries, which are skipped).
  - Orphan parent (`parent_task_id` not present) becomes root (`parentId:null`) and yields no edge; valid parent yields exactly one `bee` edge.
  - `buildNodes(replayTasks, null, noop, false)` → every node `draggable:false, selectable:false`; same call with default `true` keeps `draggable/selectable` (regression pin not to over-restrict live canvas).
  - Defensive types: `null`/`undefined`/number/string/`{}`/`{nodes:null}` → `{}`.
- **NEW `web/tests/state.parity.test.ts`**
  - `AGENT_STATES` exactly equals the 16-state array (order pinned), `DOT_STATES` exactly 4, unique values.
  - `isTerminal('failed') === true`, plus completed/cancelled/killed/interrupted; false for the other 10; `isRunActive('failed') === false`, `isRunActive('interrupted') === false`.
  - Every state has a non-empty `STRINGS.agentStates` label; `STATE_TRANSITIONS` has a key for every state (matrix completeness).
- **NEW `e2e/helpers/replay-harness.ts`** — Node static server for `web/dist` (`/`, `/assets/*`, `/api/session`, `/health`), `eventFrame(name,payload,seq)` envelope builder, `installMockWs(page)` using `page.routeWebSocket(/\/ws/)` (acknowledges COMMANDs so ACK-gated code is not exercised; test pushes events explicitly). Test-only double, documented; never touches a real provider.
- **NEW `e2e/specs/replay-state.spec.ts`** — one serial test: boot UI → push RUN_CREATED + 2 NODE_SPAWN (live nodes have `draggable/selectable`; click selects, `aria-live` shows label) → push HISTORY_SNAPSHOT with 3 valid nodes (1 orphan, 2 linked) + malformed entries → replay banner visible, 3 nodes / 1 edge, 0 `.draggable`, 0 `.selectable` → click replay node: selection unchanged, no `.selected`; node bounding box unchanged → banner still replay.
- This plan doc `plans/waves/W4-REPLAY-PROOF.md`.

## 5. Failure modes
- Dist missing/stale → harness serves 404/old bundle; spec fails loudly (no fake pass). Record `dist` mtime/hash in evidence.
- Route timing: WS mocked before `page.goto`; events pushed only after app reports Connected. If the socket route is not established, `route()` throws and the test fails with a named error.
- Malformed replay graph: projection returns `{}` (tested in Vitest); UI stays empty replay, no crash.
- React Flow class names change in a future version → spec fails and forces a re-check (test pins current 12.12.1 behaviour).
- Browser channel missing → config already fails loudly by design (no download fallback).
- No live server/key → intentionally out of scope; real-event replay remains UNVERIFIED (delegated).

## 6. Tests (discriminating, error-path included)
- Commands: `cd web; npm.cmd test -- --run`; `cd web; npm.cmd run typecheck`; `cd e2e; .\node_modules\.bin\playwright.cmd test specs/replay-state.spec.ts --project=chrome`.
- Error path: malformed snapshot entries are skipped (Vitest) and malformed top-level graphs return `{}`; replay click must NOT select (browser).
- Acceptance: all three commands green; quoted output in the final report; no product edits (`git status` limited to the four new test files + this plan).

## 7. Rollback
Delete the four new test files + this plan doc; no product code, config or git history touched. Existing suites are unaffected (new files only; no edits to existing tests, `playwright.config.ts`, or `web/src`).

## 8. Out of scope / blockers
- Out of scope: editing `web/src` (even if RED), backend/WS server, keys/paid runs, docs/audit, `tools/**`, commits.
- Blockers (recorded, not faked): live-server replay with real `HISTORY_SNAPSHOT` frames is **UNVERIFIED** — no server running (`:8799/:8080/:5173` all refused connections) and real runs require a provider key; the browser spec therefore proves UI behaviour against a deterministic transport double + current production bundle. Build freshness is asserted only by observed mtime/hash, not by rebuilding (Lead owns builds).

## 9. Discovered during proof — W4-F1 (HIGH, RED; fix is in web/src, outside this seat's write scope)

**Symptom.** Every same-kind node collapses onto one position. In the browser proof, all three
replay nodes rendered with the *identical* transform `translate(40px, 192px)`; the first run
screenshotted one huge node at zoom 300 % (fitView over a single position). The replay DOM count
(3 nodes / 1 edge) is correct, but the replay *layout* is visually stacked.

**Root cause (read from source).** `web/src/flow/graph.ts:33-56` (`layoutTasks`) calls
`graph.setNode(id, NODE_SIZE[task.kind])` with the **shared module-level size object**; dagre mutates
that label in place while assigning `x/y`, so every node of the same kind ends up sharing the last
computed coordinates. `buildReplayTasks` forces every replay node to `kind: 'worker'`
(`graph.ts:115-116`), so a replay snapshot always collapses to one spot. Live graphs collapse any
two nodes of the same kind the same way.

**Evidence captured by W4.**
- Vitest: `web/tests/graph.replay.test.ts` contains an explicit expected-failure test
  ("lays out same-kind replay nodes at distinct positions (W4-F1)") — currently `1 expected fail`.
- Browser: Playwright debug run (removed from the final spec) showed
  `{id:"snap-1", transform:"translate(40px, 192px)"}` = `snap-2` = `snap-orphan` with viewport
  `translate(56.1px, 119.7px) scale(1.4)`. The shipped spec therefore uses `force:true` for the
  replay click; the inert-selection assertion remains valid because *any* replay-node click must be inert.

**Fix (not applied — web/src is out of scope).** In `layoutTasks`, pass a **copy** of the size label:
`graph.setNode(id, { width: size.width, height: size.height })` (or `{...size}`). Then flip the
expected-failure test back to `it(...)` and re-run `npm.cmd test -- --run` + the Playwright spec.
Owner: web/src product seat. Verification: W4 tests + browser proof green with distinct node boxes.
