# GATE-CY-2 — recon → plan → lead (mandatory cycle 2 of 3)

Seat: **S3 UI-State/Replay** · READ-ONLY · HEAD `5c69b66`.

## 1. RECON (product slice read one-by-one, READLOG updated each time)
- `web/src/flow/Canvas.tsx` (256 L): replay branch now present — `buildReplayTasks(history.replayGraph?.graph)` → `replayNodes`/`replayEdges` → ReactFlow `nodes/edges` (lines 113-136); minimap a11y bridge (96-110); zoom bounds 0.1/3 (139-140).
- `web/src/flow/graph.ts` (147 L): `buildReplayTasks` (93-131) projects `graph.nodes[].{task_id,parent_task_id,state,rev}`; drops unknown parents; **forces kind `worker`/label `task <id8>`**; `buildNodes` always sets `draggable:true, selectable:true` (64-72).
- `web/src/state/store.ts` (748 L): `HISTORY_SNAPSHOT` stores `{revision, graph}` (385-393); `SETTINGS_STATUS_RESULT` carries thinking/reasoning_effort (346-361); `probeSettings` single-command guard + refusal attribution by new toast id (662-684); `isRunActive/isTerminal` gate mutating actions (434-441, 550-557).
- `web/src/state/agentState.ts` (134 L): **16 states incl `interrupted`** (9-26); four PDF dots (30); visual shapes incl `interrupted` triangle (73); `isTerminal` = completed/cancelled/killed/interrupted (111-118) — **`failed` not terminal**.
- `web/src/state/strings.ts` (378 L): 16 labels incl `interrupted` (166-183); Settings thinking/effort labels (155-156); mock/seed strings (239-249).
- `web/src/panels/SettingsPanel.tsx` (128 L): provider/model/thinking/effort/key rendered (65-78); typed refusal (51-64); single effect guard (15-24).
- `web/src/chat/ChatCore.tsx` (154 L; assigned `panels/Chat.tsx` does not exist): pinned chat + composer + streaming; draft cleared before knowing refusal (51-56, 131-137).
- `web/src/api/contract.ts` (875 L): 14 commands / 21 events / 45 documented refusal codes; `HistorySnapshotPayload.graph: unknown` (235-239); settings result optional thinking/reasoning_effort (242-248); approval nonce optional (223-229).
- `web/src/api/index.ts` (19 L): real client default; mock only under `import.meta.env.DEV && VITE_UI_MOCK==='1'` (14-19), static import line 8.
- `web/src/api/client.ts` (497 L): session fetch (235-238) **no timeout/abort**; handshake timeout only after transport creation (389-419); command timeout 15 s (443-452); ACK-only resolve / typed ERROR reject (339-379).
- `web/src/api/health.ts` (54 L): `/health` fetch (15-24) no timeout; parses status/checks/version safely.

## 2. PLAN
Verification matrix (all read-only): H1 replay render+read-only via source analysis + shipped-bundle marker probe; H2 parity via enum/labels/terminal set + bundle string count; H3 settings via store/panel/contract field chain; H4 mock exclusion via dist-bundle grep for mock-only markers; H5 timeouts via source inspection + test coverage scan. Also scan `web/tests/*` (all 8) for the discriminating tests each hypothesis needs. Any inability (no browser/rebuild/tests execution) becomes an explicit UNVERIFIED line, not an inferred PASS.

## 3. LEAD DECISION
**PASS-WITH-NOTES — proceed.** Static verification accepted because the seat is read-only (no rebuild/test execution authorised). Runtime e2e replay confirmation is delegated to the e2e seat / later wave and must appear as UNVERIFIED in REPORT. Do not convert absence-of-test into a pass.

Status: **PASS-WITH-NOTES (not blocked)**.
