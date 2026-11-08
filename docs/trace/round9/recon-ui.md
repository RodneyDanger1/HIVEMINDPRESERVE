# Round 9 — UI recon (`forgewright-recon`) — HiveMind frontend

**Seat:** UI recon (AUDIT ONLY). **Skill:** `forgewright-recon`.
**Status:** report only. No product, test, config, or binary file was edited. No server was
started, stopped, or signalled; no provider call; no browser; the provider key was not read.
The only file this seat wrote is this one.
**Working tree read:** `master` @ `8b9ccc8` (`git rev-parse`), plus pre-existing uncommitted
changes in `backend/crates/hiverouter-core/src/http.rs`,
`backend/crates/hiverouter-core/tests/snapshot_history.rs`, `tools/desktop-launcher/HiveMind.exe`.
`web/**` is clean, so every UI line number below is the committed source.
**Scope read in full:** `web/src/flow/Canvas.tsx`, `flow/graph.ts`, `flow/nodes.tsx`,
`chat/ChatCore.tsx`, `panels/{Header,HudBadge,LibraryPanel,AgentHierarchy,SwarmPanel,AgentsPanel,HistoryPanel,Scrubber,ControlBar,ApprovalOverlay,ToastRegion,ToolActivityPanel,AgentMemoryPanel,AgentMessagesPanel,AgentProfilesPanel}.tsx`,
`state/{store,agentState,approval,metrics,context,strings}.ts`,
`api/{contract,client,index}.ts`, `a11y/keyboard.ts`, `App.tsx`,
`web/tests/*` (all 23 test files enumerated; the relevant ones read).
**Read outside the UI seat only where the UI contract requires it (quote, not edit):**
`backend/crates/hiverouter-core/src/{http,ws,bus,graph,execution,store,db}.rs`,
`backend/crates/hiverouter-contract/src/event.rs`.

### Verification legend used in this report

| Tag | Meaning |
|---|---|
| **VERIFIED-SRC** | Read in the current source; `path:line` quoted. No runtime execution. |
| **VERIFIED-ABSENCE** | A `grep`/`Select-String` over the stated tree returned zero hits. |
| **INFERRED** | Follows from source + a documented contract/constant, but the runtime path was not executed. |
| **UNVERIFIED** | Not proven this round. App is stopped; no browser, no server, no provider. |
| **NOT RUN** | Test command named but deliberately not executed by this seat. |

Baseline test counts quoted in this report come from `docs/trace/round9/lead-plan.md:52-54`
(web 187/187, `snapshot_history` 4/4) — **NOT RUN** by this seat.

---

## Recon

The skill asks for one ledger per target. Ten ledgers follow, one per known area in the brief.
Each ledger answers: target behavior, entry point, callers, tests + command, data flow with one
failure path, unknowns, expected files.

### L1 — Hydration of run/tasks/history (LOG-01 residual)

- **Target behavior:** a committed snapshot rehydrate at boot `/ reconnect / resync / refusal /
  rewire / manual` must restore the server-authoritative run, graph and transcript without
  destroying newer live state, and must leave *no* stale claim on screen.
- **Entry point:** `web/src/state/store.ts:1038` `performSnapshotRefresh(source)`;
  single state write at `store.ts:1181-1224`.
- **Callers:** `store.ts:2221` `refreshRunSnapshot` (only caller today: `setHistoryRevision`
  `store.ts:2191`); `SERVER_HELLO` gate `store.ts:1381-1391`; `RESYNC_REQUIRED`
  `store.ts:1393-1402`; `ERROR E_RESYNC_REQUIRED` `store.ts:1355-1359`; foreign `NODE_REWIRE`
  without a parent `store.ts:1540-1548`; boot `store.ts:1857`.
  **No UI caller exists for a manual retry** — `refreshRunSnapshot` is not referenced by any
  `.tsx` (VERIFIED-ABSENCE: grep `refreshRunSnapshot` over `web/src/**` hits only `store.ts`).
- **Tests + command:** none. `web/tests/` has no snapshot/hydration fixture
  (VERIFIED-ABSENCE for `performSnapshotRefresh`, `refreshRunSnapshot`, `snapshot` in
  `web/tests/*.test.ts` outside `health.test.ts`). Command that would run one:
  `cd web; npx vitest run tests/<new>.test.ts`. Existing regression neighbours:
  `web/tests/streamingRender.test.ts` (5 tests) and `web/tests/store.test.ts` (8 tests) —
  `cd web; npx vitest run tests/streamingRender.test.ts tests/store.test.ts` (NOT RUN).
- **Data flow (success):** `GET /api/history/runs` -> `chooseHistoryRun` (`store.ts:662`) ->
  `GET .../revisions` -> (`GET .../messages`) -> `GET .../state?revision=latest` ->
  `recordsFromSnapshotGraph` (`store.ts:582`) -> one `set()` that replaces
  `run/tasks/graphRev/history/snapshot/resyncGap/connection`.
- **Data flow (failure path, real):** boot hydrates while a run is active (`tasks:{}, snapshot:
  initial`), then the socket opens. Later the server restarts; the transport re-bootstraps
  (`client.ts:391-413`) and re-reads `/api/session`, which returns `last_seq = bus.current_seq()`
  (`http.rs:1219`). The client adopts that value as its own applied cursor
  (`client.ts:309`) and hellos with it (`client.ts:319`), so the server answers `Replay::Current`
  (`bus.rs:172-181`) and never asks for a resync. The store's hello handler only refreshes when
  `snapshot.status` is `idle|degraded|failed` (`store.ts:1381-1391`); it is `ready`, so **no
  rehydrate happens** and the canvas/ControlBar keep the pre-restart graph and run status.
- **Unknowns:** (a) whether the launcher ever restarts the Rust core under an open tab in normal
  operation (the skill seat observed the production port occupied by a foreign process); (b) the
  exact interleaving of partysocket auto-reconnect (`client.ts:325-330`, which *does* hello with
  the client's own cursor) versus the 1500 ms session refresh (`client.ts:377-385`).
- **Files a fix would touch:** `web/src/api/client.ts` (do not adopt the server cursor as the
  applied cursor; expose "session re-established"), `web/src/state/store.ts` (refresh trigger on a
  session change), `web/tests/apiLifecycle.test.ts`, new `web/tests/*.hydration.test.ts`.

### L2 — Stale CAS rewire

- **Target behavior:** `REWIRE_TASK.expected_rev` must be the newest server node revision the
  client can prove; a refused CAS must surface one named refusal and one resnapshot, never a
  guessed retry.
- **Entry point:** `store.ts:1938` `rewireSelected(newParentId)`; precondition
  `store.ts:704` `rewireBlocker`; on-demand rev read `store.ts:1301` `fetchNodeServerRev`.
- **Callers:** `web/src/panels/ControlBar.tsx:90` (button), `web/src/flow/Canvas.tsx:291`
  (`confirmReparent` after drag-drop). Drag path entry: `Canvas.tsx:265-282`, drop-target search
  `Canvas.tsx:152-172`.
- **Tests + command:** none for `rewireBlocker`/`rewireSelected`/`fetchNodeServerRev`
  (VERIFIED-ABSENCE: `rewire` appears in `web/tests/*` only inside `apiBoundary`/docs strings —
  no test constructs a task with a `rev`). Command: `cd web; npx vitest run tests/store.test.ts`
  does not cover it. Backend contract under test: `cd backend; cargo test -p hiverouter-core
  --test snapshot_history` (NOT RUN).
- **Data flow (success):** selection -> `rewireBlocker` -> (if `unknown-server-rev`)
  `fetchNodeServerRev` -> `REWIRE_TASK{task_id,new_parent_id,expected_rev}` -> `ACK` ->
  `NODE_REWIRE` (server always sends `rev: Some(node.rev)` and `parent_task_id`,
  `ws.rs:957-984`, `ws.rs:723-731`) -> store applies parent+rev (`store.ts:1511-1539`).
- **Data flow (failure path 1, verified):** `fetchNodeServerRev` writes the fetched revision
  **unconditionally**: `set({ tasks: { ...get().tasks, [taskId]: { ...existing, rev: record.rev } } })`
  (`store.ts:1323-1326`). A live `NODE_STATE` carrying a *higher* rev that arrives while the two
  history reads are in flight is overwritten with the older committed rev; `rewireSelected` then
  sends that older `expected_rev`, which `GraphAuthority::rewire` rejects
  (`graph.rs:141-155`, `node.rev != expected_rev -> ResyncRequired`) -> `E_RESYNC_REQUIRED`
  -> `raiseResyncGap` (`store.ts:1270-1283`) -> another full snapshot read.
- **Data flow (failure path 2, verified):** an *omitted* `parent_task_id` is indistinguishable
  from "no parent" on the wire (`event.rs:264-270`, `skip_serializing_if = "Option::is_none"`).
  The store resolves the parent as
  `payloadParent ?? intended?.newParentId ?? existing?.parentId ?? null` (`store.ts:1519`), so a
  re-rooted node can never be de-parented from a live frame; only a snapshot can.
- **Unknowns:** whether the engine ever re-roots a node outside recovery (recovery's
  `add_task(id, None)` at `recovery.rs:389-414` is the known producer). Whether a second client's
  rewire can interleave with ours in practice.
- **Files a fix would touch:** `web/src/state/store.ts` (`Math.max`-style rev merge or re-verify
  after fetch; correlation-aware `pendingRewire`), `web/tests/store.test.ts` or a new
  `rewire.cas.test.ts`.

### L3 — Error UI / terminal ordering / truncation

- **Target behavior:** every failure that matters is a visible, named state; a terminal run is
  consistent everywhere; displayed evidence is truncated with the full value recoverable.
- **Entry points:** store error writes `store.ts:1234-1248` (snapshot failure), `store.ts:1020-1026`
  (`history.refreshStatus='error'` + `refreshRefusal`), `store.ts:1460-1465` (terminal
  `RUN_STATUS`), `store.ts:1442-1462` (run error codes), `ChatCore.tsx:88-91` (run refusal card);
  truncation helpers `ApprovalOverlay.tsx:40-50`, `ToolActivityPanel.tsx:77-86,175`,
  `contract.ts:26,762-769`.
- **Callers (rendering):** `App.tsx:43-48` (connection banner only), `ToastRegion.tsx` (refusals),
  `ChatCore.tsx:225-235` (run refusal), `ApprovalOverlay.tsx` (args hash/preview),
  `ToolActivityPanel.tsx` (tool rows), `ChatCore.tsx:297-320` (chat tool rows).
- **Tests + command:** rendering of the run-refusal card and tool truncation is partly pinned by
  `web/tests/agentsToolActivityMount.test.ts` (static render) and
  `web/tests/toolActivityPanels.test.ts`. **No test asserts that `snapshot.status/refusal/
  incomplete` or `history.refreshRefusal` ever render.** Command:
  `cd web; npx vitest run tests/agentsToolActivityMount.test.ts tests/toolActivityPanels.test.ts`
  (NOT RUN).
- **Data flow (failure path, verified):** `GET .../messages` fails ->
  `messagesUnavailable = true` -> `incomplete: ['messages-unavailable']`, `status: 'ready'`
  (`store.ts:1146-1158`) -> nothing in `web/src/**/*.tsx` reads `snapshot.incomplete` or
  `snapshot.status` (VERIFIED-ABSENCE) -> the chat renders `STRINGS.chat.empty`
  ("Type a prompt to create the first run...", `ChatCore.tsx:243-269`) as if no transcript ever
  existed. The committed transcript is silently absent, not reported.
- **Terminal-ordering failure path (verified):** `RUN_STATUS` terminal does not touch
  `approvals` (`store.ts:1426-1467`), and the server publishes no `APPROVAL_RESOLVED` when a run
  is cancelled (`execution.rs:1114-1131`; resolution events are only published from
  `ws.rs:883-892` on an operator decision). A cancelled/failed run therefore leaves a
  full-screen `aria-modal` approval over the whole app referencing an approval that no longer
  exists; its Approve/Deny stay enabled (status `pending`, nonce present, connection open,
  `approvalActionsEnabled` `approval.ts:254-267`) and the click is refused server-side, after
  which Dismiss becomes available (`approval.ts:126-140`).
- **Truncation facts:** approval ids/args hash truncate to 12 chars with the full value in
  `title` (`ApprovalOverlay.tsx:40-50,258,271-287`) — correct; tool text truncates to 160 chars
  with `title` (`ToolActivityPanel.tsx:173-177`) — correct; the chat tool row truncates text to
  nothing at all with **no `title`** (`ChatCore.tsx:315-317`) — the only unrecoverable
  truncation; `args_preview` over 512 UTF-16 units is dropped to `—`, never truncated
  (`contract.ts:762-769`) — correct and documented.
- **Unknowns:** whether any run status beyond `running/streaming` ever reaches `RUN_STATUS` from
  the engine (would change the ChatCore Stop gate, see F-UI-16).
- **Files a fix would touch:** `App.tsx` (or a new `SnapshotBanner`), `HistoryPanel.tsx`,
  `ChatCore.tsx`, `web/src/state/store.ts` (terminal approval sweep), tests under `web/tests/`.

### L4 — Remote / Watchdog cards

- **Target behavior (operator requirement, not implemented):** every agent, including
  Remote/Watchdog, has a UI card.
- **Entry points present:** the node-kind vocabulary only —
  `web/src/api/contract.ts:213` `NODE_KINDS = ['hyperagent','superagent','subagent','worker',
  'tool','error']`; `web/src/state/agentState.ts:126` `NODE_KIND_KEYS` (same six);
  renderer registry `web/src/flow/nodes.tsx:302-310`; sizes `web/src/flow/graph.ts:31-38`;
  minimap colour switch `Canvas.tsx:698-713`.
- **Callers:** `graph.ts:234-248` `buildNodes` (`type: task.kind`), `Canvas.tsx:654`
  `nodeTypes`.
- **Tests + command:** `web/tests/agentState.test.ts` and `web/tests/graph.replay.test.ts:160-177`
  pin that `subagent` is a distinct card kind; **no test mentions Remote/Watchdog**
  (VERIFIED-ABSENCE across `web/tests`).
- **Data flow (absence):** backend has no producer: `grep -E
  'Watchdog|watchdog|RemoteAgent|remote_agent|heartbeat|Daemon' backend/**/*.rs` -> **0 hits**
  (VERIFIED-ABSENCE); no route (`http.rs:514-532` lists every route); no kind in
  `event.rs:236-271`. The only "watchdog" strings in the UI are the FigCraft design-ledger copy
  (`strings.ts:60,62`, `assets/manifest.ts:2-4,33,109`).
- **Unknowns:** whether the requirement is still in force for this round (it is recorded in
  `docs/trace/orca-wave/04-capability.md:204-205,505` and `lead-plan.md:58` T8).
- **Files a fix would touch (if the requirement is accepted):** contract kind list, agentState
  keys+labels, nodes renderer, graph sizes, minimap switch, backend kind/role producer and a
  route. This is a feature slice, not a bug fix.

### L5 — Ownership switching / Subagent tier

- **Target behavior:** SuperAgent owns a hive/branch, ownership is switchable, and the Subagent
  tier is real (narrow tools, no spawn, persona limits).
- **Entry points:** UI spawn `store.ts:1914-1936` `spawnSubHive` (role chosen from
  `STRINGS.node.*`); wire command `contract.ts:49-51` `SPAWN_SUB_HIVE{parent_task_id, role}`;
  ownership display `web/src/flow/nodes.tsx:159-162` + `strings.ts:281`
  (`roleCaption = 'decomposes intent, owns the sub-hive'`).
- **Callers:** `ControlBar.tsx:63-65`, `LibraryPanel`/`SwarmPanel` copy, `Canvas` spawn via the
  bar only.
- **Tests + command:** `web/tests/store.test.ts:176-188` pins only the kind/label fallback; no
  test asserts the wire `role` value. Command `cd web; npx vitest run tests/store.test.ts`
  (NOT RUN).
- **Data flow (failure path, verified):** the UI sends display labels as roles —
  `'SuperAgent'` / `'WorkerAgent'` / `'Subagent'` (`store.ts:1929-1935` via
  `strings.ts:275-278`). Downstream: `task_agent_tier` lowercases, so the tier is right, but
  `seed_task_context` looks up the persona map with `&task.role` verbatim
  (`execution.rs:866-880`) and `persona_spawn_limits` likewise (`execution.rs:550-558`,
  call site `execution.rs:1084`), while the map is keyed lowercase (`personas.json`
  ids `planner/content-writer/superagent/subagent/...`, confirmed by
  `AgentProfilesPanel.tsx:23-31`). So a UI-spawned `SuperAgent`/`Subagent` gets **no persona
  `allowed_tool_ids` narrowing and no `max_children`/`max_depth`**; only the prompt lookup
  lowercases first (`provider_executor.rs:82-92`).
- **Ownership data model:** absent on both sides. `TaskRecord` has
  `id/parentId/kind/label/role?/state/progress/errorCode/rev` (`store.ts:67-87`); no owner, hive,
  scope, or ownership-mode field; no ownership command in `contract.ts:100-160`. The caption is
  therefore a display-only claim (VERIFIED-SRC).
- **Unknowns:** whether "ownership switching" remains in scope for this round.
- **Files a fix would touch:** `web/src/state/store.ts` (send canonical lowercase roles) and a
  backend test; ownership requires a designed slice (contract + graph + UI).

### L6 — Per-hive / branch memory UI

- **Target behavior:** durable memory is inspectable per hive/branch (run/task scope), not only
  per persona, and the display is truthful about scope.
- **Entry point:** `web/src/panels/AgentMemoryPanel.tsx:24-84` (fetch `/api/agents/memory`,
  DELETE `/api/agents/memory/{id}`); mounted from `AgentsPanel.tsx:74`.
- **Callers:** `AgentsPanel` only. The panel has no prop, no store read, and no dependency on the
  current run/task (VERIFIED-SRC: the file imports only `apiBaseUrl` and `EmptyState`).
- **Tests + command:** none. `web/tests/*` never mentions `/api/agents/memory` or
  `AgentMemoryPanel` (VERIFIED-ABSENCE).
- **Data flow (success):** `GET /api/agents/memory` -> `agent_memory_list` iterates **personas**
  and calls `store.agent_memory(&persona.id, 8)` (`http.rs:985-1001`) -> rows are persona-global.
  **Failure path (verified):** memories are written under `&task.role` verbatim
  (`execution.rs:3029-3036`) and recalled by `agent_memory(agent_key)` with
  `SELECT key,value FROM agent_memory WHERE agent_key = ?1` — `run_id`/`task_id` are stored but
  never filtered (`store.rs:649-668`; table + PK at `db.rs:183-190`). Two consequences: (i) there is no per-hive/branch read on any
  route; (ii) for UI-spawned tasks (`role='SuperAgent'`, L5) the persona-keyed enumeration
  (`persona.id='superagent'`) cannot see the rows, so the panel reports
  "No durable memories for the boot profiles" while `agent_memory` holds rows under
  `SuperAgent`.
- **Unknowns:** whether `personas.json` ids are guaranteed lowercase (they are today per
  `AgentProfilesPanel.tsx:23-31`; the file itself was not opened).
- **Files a fix would touch:** `web/src/panels/AgentMemoryPanel.tsx` (scope selector, empty-state
  copy), `store.rs`/`http.rs` (scoped recall route), tests.

### L7 — A2A answer / roster / topic / mute

- **Target behavior:** agents discover peers (roster), exchange a question *and* an answer, and
  the operator can group (topic) or mute A2A traffic.
- **Entry points:** mailbox `web/src/panels/AgentMessagesPanel.tsx:27-84`
  (`GET /api/agents/runs/{id}/messages`); store signal `store.ts:1551-1554` `AGENT_MESSAGE`
  -> `agentMessageVersion`; wire payload `contract.ts:315-321`, `event.rs:609-624`.
- **Callers:** `AgentsPanel.tsx:75`.
- **Tests + command:** none for these panels (VERIFIED-ABSENCE); backend live test covers roster
  only — `backend/crates/hiverouter-core/tests/snapshot_history.rs:211-223`
  (`cd backend; cargo test -p hiverouter-core --test snapshot_history`, NOT RUN).
- **Data flow (verified absences):**
  - **roster:** route exists — `GET /api/agents/runs/{run_id}/roster` (`http.rs:522`,
    handler `http.rs:942-983`, returns `task_id/parent_task_id/role/kind/label/state/depth/rev`)
    — but **no `web/src` file references `/roster`** (VERIFIED-ABSENCE). A2A discovery is unused
    by the UI, so an operator cannot see who is addressable.
  - **answer:** only two kinds are ever produced: `ask_parent` (`execution.rs:2477`) and
    `send_to_sibling` (`execution.rs:2599`); `ask_parent` is fire-and-forget (it returns
    `{message_id,to_task_id,kind}` and does not wait, `execution.rs:2479-2496`). There is no
    answer/reply row, no answer kind, and no answer field in the schema
    (`db.rs:163-172`: `id,run_id,from_task_id,to_task_id,kind,text,created_at`).
  - **topic / mute:** absent in the UI and in the schema/commands
    (VERIFIED-ABSENCE: `topic`/`mute` in `web/src` -> 0 hits; no such column in `db.rs:163-172`;
    no such command in `contract.ts:100-160`).
- **Unknowns:** whether mute is meant to be an operator preference (client-side) or server policy
  (`event.rs` has no field for it).
- **Files a fix would touch:** `AgentMessagesPanel.tsx` (roster fetch + peer labels + refresh
  control), possibly a `settings`-level mute in `state/store.ts` + `SettingsPanel.tsx`, and a
  backend answer path if "answer" is a real requirement.

### L8 — Replay counters / minimap / run picker

- **Target behavior:** in replay (`history.replayGraph !== null`) every counter, roster and map
  describes the replayed revision; the operator can choose which run's history is shown.
- **Entry points:** replay state `store.ts:1728-1749` `HISTORY_SNAPSHOT`; replay projection
  `graph.ts:280-329` `buildReplayTasks` + `Canvas.tsx:431-497`; counters
  `Canvas.tsx:639-641`, `Canvas.tsx:696`, `HudBadge.tsx:8-63`, `LibraryPanel.tsx:32-137`,
  `HistoryPanel.tsx:9-69`, `Scrubber.tsx:10-76`; run selection `store.ts:662-674`
  `chooseHistoryRun`.
- **Callers:** `App.tsx:50-87` (history tab renders `HistoryPanel` + `Canvas`).
- **Tests + command:** `web/tests/graph.replay.test.ts` (7 tests) pins the projection and
  read-only nodes; e2e `e2e/specs/replay-state.spec.ts` pins inert replay nodes. **No test pins
  the replay *counters*.** Command `cd web; npx vitest run tests/graph.replay.test.ts` (NOT RUN);
  e2e requires a browser (not permitted this seat).
- **Data flow (success):** `HISTORY_REVISION_SET` -> `ACK` -> `HISTORY_SNAPSHOT`
  (`ws.rs:842-857`) -> `buildReplayTasks` -> `replayNodes/replayEdges` -> read-only canvas.
- **Data flow (failure path, verified):** the same `set` that installs `replayGraph` leaves every
  other surface on live data: the canvas filter pill prints live counts during replay
  (`Canvas.tsx:640`), the minimap empty-state class is keyed on live `visibleTasks`
  (`Canvas.tsx:696`), the HUD still renders the live delta next to "REPLAY · LIVE METRICS
  HIDDEN" (`HudBadge.tsx:39-40` vs `:57-59`), the library rail lists live agents and ships a
  `selectTask` that writes live selection during replay (`LibraryPanel.tsx:105-128`, no
  `history` read in the file), and the header quick-search also calls `selectTask`
  (`Header.tsx:126-136`, no replay guard). Additionally `HISTORY_SNAPSHOT` is applied from **any**
  client's request: the bus broadcasts (`bus.rs:92-120`), the handler never compares
  `envelope.correlation_id` with a pending `HISTORY_REVISION_SET` (`store.ts:1728-1749`), so a
  second tab selecting a revision forces this tab into read-only replay.
- **Run picker (verified absence):** `/api/history/runs` may return several runs, but the only
  chooser is `chooseHistoryRun` (`store.ts:662-674`: current run if listed, else the first active,
  else `runs[0]`). There is no selector in `HistoryPanel.tsx` and no store action to change the
  chosen run; `HistoryState` has no `runId` field (`store.ts:149-162`) although `snapshot.runId`
  records it (`store.ts:190`). `HistoryPanel.tsx:27` therefore prints the **live** run id beside
  revisions hydrated for `snapshot.runId` (reachability of a mismatch: UNVERIFIED; the live run is
  normally present in the runs list).
- **Unknowns:** whether replay should freeze selection at all (store currently allows it), and
  whether cross-tab replay hijack is acceptable (it is not documented either way).
- **Files a fix would touch:** `Canvas.tsx`, `HudBadge.tsx`, `LibraryPanel.tsx`,
  `Header.tsx`, `HistoryPanel.tsx`, `store.ts` (`HistoryState.runId`, correlation-gated
  `HISTORY_SNAPSHOT`), tests.

### L9 — Approval focus / Escape

- **Target behavior:** the approval dialog traps focus, takes initial focus, closes/denies on
  Escape (per the product's own blocking-decision copy), and returns focus on close.
- **Entry point:** `web/src/panels/ApprovalOverlay.tsx:165-385` (`ApprovalOverlay`).
- **Callers:** `App.tsx:139` (`<ApprovalOverlay />` outside every view; always mounted).
- **Tests + command:** `web/tests/agentsToolActivityMount.test.ts:176-315` renders the overlay
  with `react-dom/server` `renderToStaticMarkup` only. No test asserts focus, a focus trap, or
  Escape (VERIFIED-ABSENCE: `Escape` in `web/tests` -> 0 hits; the only Escape handler in
  `web/src` is the header search input, `Header.tsx:112-116`).
- **Data flow (failure path, verified):** `approvals.map(...)` (`ApprovalOverlay.tsx:221`) emits
  one `role="dialog" aria-modal="true"` + `fixed inset-0` backplate **per approval**
  (`:244-249`), all simultaneously in the DOM; there is no `useRef`, no `focus()`, no keydown
  listener, no focus trap, and no focus restore anywhere in the file. Keyboard users land on
  whatever Tab order reaches; screen readers see N nested modal dialogs (an invalid modal state)
  and the background is never made inert. With zero approvals the component returns `null`
  (`:187-189`) — that part is correct.
- **Unknowns:** none material. This is purely additive work (effect + refs + keydown), but it
  interacts with `aria-modal` count > 1, which needs a design decision (queue approvals, or one
  dialog with a list).
- **Files a fix would touch:** `web/src/panels/ApprovalOverlay.tsx`, a new DOM-level test
  (the repo has no DOM env today: `web/vite.config.ts:27` `environment: 'node'`, so a DOM test
  needs a config decision — see U-UI-7), plus `web/tests/approval.integration.test.ts` if it is
  promoted.

### L10 — ControlBar

- **Target behavior:** every mutating control is enabled only when its true precondition holds,
  the target of each command is visible, and a refused command is never a silent no-op.
- **Entry point:** `web/src/panels/ControlBar.tsx:11-127`.
- **Callers:** `App.tsx:125-130` (workspace+workspace tab only; **not mounted** in the history
  tab, where only the Scrubber renders, `App.tsx:131-135`).
- **Tests + command:** `web/tests/keyboard.test.ts`, `web/tests/store.test.ts` do not cover the
  bar; e2e `e2e/specs/control-matrix.spec.ts` / `gui-residual.spec.ts` reference its behaviour
  (browser, not permitted). Command: `cd web; npx vitest run tests/store.test.ts` (NOT RUN).
- **Data flow (success):** selection -> `selectTask`; spawn -> `spawnSubHive` (`store.ts:1914`);
  re-route -> `rewireSelected(target)`; cancel -> `CANCEL_RUN`; kill -> `KILL_WORKER`.
- **Data flow (failure paths, verified):**
  1. **Stale re-route target.** `target` is local state that is never reset when the selection
     changes (`ControlBar.tsx:23,73-75`). The target list excludes only the *current* selection
     (`:29-31`). Sequence: select worker A -> choose target T -> select T (a worker) -> `canRewire`
     is true (`selected.kind==='worker'`, `target!==''`, `:34-35`) and sends
     `REWIRE_TASK{task_id:T,new_parent_id:T}` -> always refused `E_CYCLE` (`graph.rs:160-171`).
     The `<select value={target}>` also renders a value that is not among its options.
  2. **Kill scope.** `canKill` requires `selected.kind === 'worker'` (`:38`), but the store allows
     `worker` **or** `tool` (`store.ts:2010`). A selected ToolAgent card cannot be killed from the
     bar even though the store would send the command.
  3. **Re-route scope.** The bar only enables re-route for a `worker` source (`:35`); the canvas
     drag path accepts any non-hyperagent source (`Canvas.tsx:255,273`) and any
     superagent/hyperagent/**worker** target (`Canvas.tsx:158-168`). Two different rules for the
     same mutation.
  4. **Global pending lock.** `anyPending` (`:27`) counts **every** command (including
     `SETTINGS_STATUS`, `FILE_READ`, `CHAT_PIN`), and `canSpawn/canCancel/canKill` all require
     `!anyPending` (`:33-38`). A pending file read (15 s client timeout, `client.ts:225`) disables
     Cancel on a live run.
  5. **Status text.** For a terminal run the bar prints `isTerminal(run.status) ?
     STRINGS.states.unavailable : run.status` (`:122`) — i.e. "completed"/"failed" are replaced
     by the literal word "Unavailable" (`strings.ts:193`). The run's real terminal status is never
     shown in the bar.
- **Unknowns:** whether the ToolAgent card is expected to be killable (the store says yes, the
  brief's kill copy says worker).
- **Files a fix would touch:** `web/src/panels/ControlBar.tsx` only (plus a unit test if the
  precondition helpers are extracted).

---

## 2. Findings, ranked

Severity convention borrowed from the round-9 lead plan: **P0** = the UI can state something
false about live authority or lose operator input; **P1** = a reachable wrong/blocked behavior on
a shipped surface; **P2** = inconsistency/robustness with a bounded impact.

### P0

#### F-UI-01 — P0 — Transport recovery after a server restart does not rehydrate; the canvas keeps a run the server no longer reports

- **Evidence (VERIFIED-SRC):** `client.ts:309` adopts `GET /api/session`'s
  `last_seq` (= `bus.current_seq()`, `http.rs:1219`) as the client's own applied cursor;
  `client.ts:319` hellos with it; `bus.rs:172-186` therefore answers `Replay::Current` and no
  `RESYNC_REQUIRED` is emitted; `store.ts:1381-1391` refreshes only from
  `snapshot.status ∈ {idle,degraded,failed}`, and a previous boot leaves it `ready`
  (`store.ts:1211-1221`).
- **Behavior:** after the Rust core restarts under an open tab, the socket re-opens, `connection`
  becomes `open`, and the run/tasks/`selectedTaskId` are whatever the dead process left. ControlBar
  offers Cancel/Spawn/Kill for that run; commands are refused (or worse, act on the recovered
  run's different state).
- **Why wrong:** the UI's stated contract is "mutating state is applied only from server events,
  and every failure is a named refusal" (`store.ts:1-6`), and the hydration path exists precisely
  to make a reload truthful. Here it is skipped because the *client* erased the only signal
  (its own applied seq) that would have triggered it.
- **Existing tests:** none (L1). **Blast radius:** store hydration + every surface that trusts
  `run`/`tasks`; the fix must not re-trigger a full read on every ordinary reconnect
  (`live-resync.spec.ts` pins "a small replay gap must not force a resnapshot").
- **Safe fix candidate:** keep the store's applied `lastSeq` as the hello cursor (client must not
  overwrite it from `/api/session`; the session value is the *server's* position, not ours), and/or
  add a distinct "session re-established" signal so `store.init`-time hydration can run with
  source `reconnect`. One new test: restart fixture where `/api/session` returns a smaller
  `last_seq`; assert a snapshot read is issued and the new tasks replace the stale ones.
- **Status of the runtime claim:** code path verified; no live reproduction this round (app
  stopped) -> the *user-visible* consequence is INFERRED.

### P1

#### F-UI-02 — P1 — Hydration merge direction is "live wins" with no revision comparison, so a resync can preserve unverified node state over the committed snapshot

- **Evidence (VERIFIED-SRC):** `store.ts:1175-1180` computes `preserveLiveTasks` from run identity
  only; `store.ts:1191-1194` applies `{ ...projection.tasks, ...liveTasks }`. Live records carry
  the revision the client last saw (`ws.rs:963` `rev: Some(node.rev)`), and the projection carries
  the committed revision (`recordsFromSnapshotGraph(store.ts:582-645)` *requires* `rev`), yet the
  merge never compares them.
- **Behavior:** a resync issued because the client missed events (the exact case in which its own
  records are known-stale) keeps its own records for every node the snapshot also knows. The
  snapshot's newer state/rev is discarded silently, while `snapshot.revision` reports the
  committed revision as restored.
- **Why wrong:** the read exists to make the client authoritative again; a "ready" snapshot whose
  contents were partly overridden by unverified client records is a false claim. It also feeds
  L2's stale `expected_rev`.
- **Existing tests:** none (L1). **Blast radius:** `tasks` consumers, `rewireBlocker`,
  `snapshot.revision` display if added.
- **Safe fix candidate:** merge per node by revision: committed record wins when
  `projection.rev >= live.rev` (or when `live.rev === null`); keep live-only nodes; record a new
  `SnapshotIncomplete` value (`'live-gap'`) when live nodes are kept so the (new) UI can say
  "committed state + later events". Do not change the boot path (no live run).

#### F-UI-03 — P1 — Snapshot/history error state is written and never rendered; the UI shows an empty history/chat instead

- **Evidence (VERIFIED-SRC + VERIFIED-ABSENCE):** `snapshot.status/refusal/incomplete` written at
  `store.ts:1114-1157,1234-1248`; `history.refreshStatus/refreshRefusal` written at
  `store.ts:1004-1026,1201-1210`; `isSnapshotDegraded` exported at `store.ts:750` and **never
  called**; no `.tsx` reads `snapshot.*` or `refreshRefusal` (grep over `web/src/**/*.tsx`).
- **Behavior:** a failed `/messages` read or an unreadable history route produces
  "No run history yet" / "No revisions yet." / the chat's first-run empty state, indistinguishable
  from a genuinely empty run. Losing a transcript looks like never having had one.
- **Why wrong:** the plan's own rule is "no fabricated empty state" (`lead-plan.md:41`), and the
  store deliberately records the exact refusal + incomplete reasons for this purpose.
- **Existing tests:** none. **Blast radius:** read-only additions if surfaced in a banner; no
  data-path change.
- **Safe fix candidate:** one `SnapshotBanner` reading `snapshot.status/refusal/incomplete`
  (plus `history.refreshRefusal`) with a Retry that calls the existing
  `refreshRunSnapshot('manual')`; HistoryPanel and ChatCore suppress their empty states while
  `incomplete` includes `messages-unavailable` / `no-committed-revision`.

#### F-UI-04 — P1 — Approval dialog has no initial focus, no focus trap, no Escape, and one `aria-modal` dialog per pending approval

- **Evidence (VERIFIED-SRC):** `ApprovalOverlay.tsx:221` maps approvals to `fixed inset-0`
  dialogs with `role="dialog" aria-modal="true"` (`:244-249`); the file contains no `useRef`,
  `focus(`, or `keydown` (VERIFIED-ABSENCE inside the file); the only Escape handler in `web/src`
  is `Header.tsx:112-116`.
- **Behavior:** keyboard focus stays behind the modal backplate; Escape does nothing; two pending
  approvals produce two simultaneous aria-modal dialogs. The dialog is the product's blocking
  decision surface and it cannot be operated or dismissed from the keyboard alone.
- **Why wrong:** contradicts the surface's own "blocking decision" copy
  (`SwarmPanel.tsx:114-119`) and the a11y gates the repo claims (G-UI-8 focus ring only,
  `e2e/GATE-EXCLUSIONS.md:5`).
- **Existing tests:** static-render only (`agentsToolActivityMount.test.ts`), which cannot see
  focus/keyboard. **Blast radius:** `ApprovalOverlay.tsx`; adding a DOM test needs an environment
  decision (`web/vite.config.ts:27` is `environment: 'node'`, no jsdom/happy-dom dependency
  installed).
- **Safe fix candidate:** focus the dialog on mount, trap Tab within the top-most dialog, map
  Escape to deny-or-dismiss only when the action is enabled (never a silent close on a live
  approval), restore focus on close, and render a queue (one dialog + "1 of N") instead of N
  modals.

#### F-UI-05 — P1 — A terminal run leaves blocking approval cards for approvals the server will never resolve

- **Evidence (VERIFIED-SRC):** `RUN_STATUS` terminal handling touches run/messages/tool buffers
  but not `approvals` (`store.ts:1426-1467`); the server publishes `APPROVAL_RESOLVED` only from an
  operator decision (`ws.rs:883-892`); `cancel_run` cancels tokens and lets the wait path finish
  (`execution.rs:1114-1131`) without emitting `APPROVAL_RESOLVED`.
- **Behavior:** cancel/fail a run while an approval is open -> the full-screen dialog stays,
  Approve/Deny remain enabled (`approval.ts:254-267`), the eventual click is refused, and the operator
  must click Dismiss to recover the UI.
- **Why wrong:** the card claims a live decision that can no longer happen; the modal blocks the
  whole app in the meantime.
- **Existing tests:** `approval.state.test.ts` covers the pure machine; no test covers
  run-terminal × approvals. **Blast radius:** store only (+ one unit test).
- **Safe fix candidate:** on terminal `RUN_STATUS`, mark approvals whose `runId` matches as
  read-only (`status:'expired'`-class) with a reason, or drop them; either way keep
  `approvalResolve` consistent and never auto-send.

#### F-UI-06 — P1 — Per-hive/branch memory is not representable, and UI-spawned roles' memories are invisible to the memory panel

- **Evidence (VERIFIED-SRC):** write key is `&task.role` verbatim
  (`execution.rs:3029-3036`); recall is `WHERE agent_key = ?1` with `run_id`/`task_id` stored but
  never filtered (`store.rs:649-668`; table + PK at `db.rs:183-190`); the route enumerates
  **persona ids** (`http.rs:985-1001`); the panel has no scope control and no run dependency
  (`AgentMemoryPanel.tsx:20-84`).
- **Behavior:** the panel cannot show per-hive/per-branch memory at all; for tasks the UI spawned
  (`role='SuperAgent'`/`'Subagent'`, L5) the persona-keyed enumeration misses the rows, so a
  non-empty memory bank renders as "No durable memories for the boot profiles".
- **Why wrong:** the operator requirement is per-hive/branch memory (K-series in
  `docs/trace/orca-wave/04-capability.md:204-205,344`), and the "no rows invented" copy
  (`AgentMemoryPanel.tsx:93`) is honest only about the rows it looked for.
- **Existing tests:** none. **Blast radius:** panel + one store method + a backend scope filter.
- **Safe fix candidate:** (a) send canonical lowercase roles from the UI (fixes the key skew),
  (b) add an optional `run_id`/`task_id` filter to the recall query and a scope selector in the
  panel, (c) label the panel with the scope it is showing.

#### F-UI-07 — P1 — A2A has no answer, no roster, no topic, no mute; the roster route is unused

- **Evidence (VERIFIED-SRC / VERIFIED-ABSENCE):** only `ask_parent`/`send_to_sibling` are produced
  (`execution.rs:2477,2599`), fire-and-forget (`:2479-2496`); schema has no topic/mute/answer
  columns (`db.rs:163-172`); `GET /api/agents/runs/{run_id}/roster` exists (`http.rs:522,942-983`)
  but no `web/src` file requests it; `web/src` has zero `topic`/`mute` hits.
- **Behavior:** the "mailbox" is a one-way log of two message kinds with raw 8-char task ids; the
  operator cannot see who is addressable, cannot group, cannot mute, and there is no answer row to
  read.
- **Why wrong:** the panel's own copy says "one row per persisted agent-to-agent message" and the
  Swarm copy promises "the cross-branch mailbox" (`SwarmPanel.tsx:116-119`); a question with no
  answer record is half a protocol. This is a requirement gap, not a lie.
- **Existing tests:** roster backend test only (`snapshot_history.rs:211-223`). **Blast radius:**
  panel; mute if client-side touches `settings`; answer needs a backend slice.
- **Safe fix candidate:** fetch the roster to label `from/to` and list addressable peers; add a
  manual refresh + honest empty/error states; treat answer/topic/mute as a requirement decision
  (do not fake them).

#### F-UI-08 — P1 — `fetchNodeServerRev` can downgrade the node revision it then sends as the CAS precondition

- **Evidence (VERIFIED-SRC):** `store.ts:1323-1326` overwrites `rev` with the fetched committed
  value regardless of what arrived in between; `rewireSelected` reads it back at `store.ts:1976`
  and sends it at `:1984-1988`; the server refuses on mismatch (`graph.rs:148-155`) and the client
  turns that into a resync gap (`store.ts:1355-1359,1270-1283`).
- **Behavior:** a re-route initiated during a busy run can guarantee its own refusal and a full
  resnapshot; repeated clicks can loop request -> refusal -> resnapshot.
- **Why wrong:** the command has a real CAS, so the client must never manufacture a stale one when
  a newer value is already known locally.
- **Existing tests:** none. **Blast radius:** store only.
- **Safe fix candidate:** merge with `Math.max(existing.rev ?? -1, record.rev)` and, after a
  fetch, re-run `rewireBlocker` before sending; if a newer live rev arrived, use it.

#### F-UI-09 — P1 — Replay mode leaves live counters/rosters on screen (and a second tab can force replay)

- **Evidence (VERIFIED-SRC):** `Canvas.tsx:640` (live counts during replay), `:696` (minimap
  empty class keyed on live tasks), `HudBadge.tsx:57-59` (delta renders in the replay branch),
  `LibraryPanel.tsx:105-128` (live chips + live `selectTask`), `Header.tsx:126-136` (live search
  selection), `store.ts:1728-1749` (`HISTORY_SNAPSHOT` applied without a correlation check, bus is
  a broadcast).
- **Behavior:** the canvas shows revision N while the chrome reports the live graph; clicking a
  library chip or a search result mutates live selection during replay; a second tab's revision
  click drops this tab into read-only replay.
- **Why wrong:** replay is presented as "read-only" (`strings.ts:191`) and the filter pill
  (`Canvas.tsx:640`) is a count of the visible graph — it is wrong by construction when the graph
  is the replayed one.
- **Existing tests:** projection/read-only nodes pinned (`graph.replay.test.ts`), counters not.
  **Blast radius:** presentational + one store gate.
- **Safe fix candidate:** derive every replay counter from `replayTasks` (already computed at
  `Canvas.tsx:431-434`) and expose it from one selector; disable/ignore `selectTask` while
  replaying; gate `HISTORY_SNAPSHOT` on a pending `HISTORY_REVISION_SET` (the store already
  correlates commands by id in `pendingCommands`, but the ACK is deleted before the snapshot
  arrives — use an explicit `pendingReplayRevision` field set before the send).

### P2

Row-level test status and fix candidates are the ones in the matching ledger (§Recon) and in §8;
where no test exists today the ledger says so. None of these rows is a P0/P1 on its own.

| ID | Finding | Evidence | Why wrong / impact |
|---|---|---|---|
| F-UI-10 | Server-restart hydration is not the only hydration gap: `performSnapshotRefresh` clears `history.replayGraph` on success, silently ejecting the operator from replay, while `refreshCommittedRevisions` preserves it | `store.ts:1207` vs `store.ts:989-992` | two merge policies for the same cursor; user is yanked to live with no notice |
| F-UI-11 | Re-rooted nodes can never lose their parent from a live frame | `event.rs:264-270` (omitted when `None`) + `store.ts:1519` (`?? existing?.parentId`) | canvas keeps an edge the server no longer has; only a snapshot fixes it |
| F-UI-12 | The drop target accepts the hyperagent kind, but no hyperagent node exists in the React Flow set | `Canvas.tsx:158-168` (`candidateTask.kind === 'hyperagent'`) vs `graph.ts:234-235` (filters `hyperagent`; only an invisible `hiveanchor` node exists) | drag-a-card-onto-the-hive is unreachable while the bar offers hyperagent as a re-route parent; the branch is dead |
| F-UI-13 | No ancestry/cycle pre-check before a rewire | `Canvas.tsx:152-172`, `ControlBar.tsx:29-31` | a drag onto a descendant highlights as a valid target and is always refused `E_CYCLE` (`graph.rs:160-171`) |
| F-UI-14 | `nodes` memo omits `interactiveGraph`, so `draggable/selectable` never turn off when the run ends | `Canvas.tsx:202-221` (deps at `:221`), `Canvas.tsx:206` | after a terminal run a card can still be dragged and the "Connect ...?" prompt (`Canvas.tsx:579-608`) offers a rewire the store will refuse (`E_RUN_TERMINAL`) |
| F-UI-15 | "Focus branch" silently disables "Hide finished" | `Canvas.tsx:179-196` (early return ignores `hideFinished`) | the filter pill still shows both pressed; results do not match the two active filters |
| F-UI-16 | ChatCore's Stop control cannot stop `planning`/`thinking`/`queued` runs that ControlBar can cancel | `ChatCore.tsx:357` (`streamingRunId === null && runStatus !== 'running'`) vs `ControlBar.tsx:36` (`isRunActive`) | two cancel affordances disagree on the same run |
| F-UI-17 | Keyless refusal bypasses the store's refusal path (no dedupe, no cap) | `ChatCore.tsx:138-150` builds `refusals` inline; the store caps at 4 and dedupes (`store.ts:803-814`) | repeated Enter with no key grows the toast stack unbounded and duplicates copy |
| F-UI-18 | ControlBar prints "Unavailable" instead of the terminal run status | `ControlBar.tsx:122` + `strings.ts:193` | the bar hides whether the run completed or failed |
| F-UI-19 | Any pending command blocks Cancel/Spawn/Kill | `ControlBar.tsx:27,33-38` | a slow `FILE_READ`/`SETTINGS_STATUS` (15 s timeout, `client.ts:225`) disables Cancel on a live run |
| F-UI-20 | ControlBar's "no target" option is the no-metrics placeholder, and the select's aria-label says "WORKER: Re-route" while it lists every task | `ControlBar.tsx:43-56` (`STRINGS.node.noMetrics` `'—'`; label `${STRINGS.node.workerLabel}: ${reroute}`) | mislabeled control; screen-reader name does not match the option set |
| F-UI-21 | Kill is enabled for `worker` only although the store accepts `worker` or `tool` | `ControlBar.tsx:38` vs `store.ts:2010` | a ToolAgent card cannot be killed from the bar |
| F-UI-22 | Swarm census has no SUBAGENTS bucket, so subagent cards vanish from the counts | `SwarmPanel.tsx:21-27` (5 buckets, none `subagent`) | a run of subagents shows `ACTIVE: n` with all four kind buckets at 0; contradicts "the live branch census" (`SwarmPanel.tsx:1-5`) |
| F-UI-23 | Duplicate nested landmark `aria-label="Agent hierarchy"` | `SwarmPanel.tsx:110` + `AgentHierarchy.tsx:56` | two nested sections with the same accessible name |
| F-UI-24 | AgentMessagesPanel refetches the whole mailbox on every `AGENT_MESSAGE` frame | `AgentMessagesPanel.tsx:84` (`[runId, messageVersion]`) + `store.ts:1551-1554` (increments per event) | N events -> N full reads (bounded at 32 messages per run server-side, but still O(n^2) transfer and no debounce) |
| F-UI-25 | The three raw-fetch panels have no named-refusal surface and no abort | `AgentMemoryPanel.tsx:65-68,83`, `AgentMessagesPanel.tsx:75-79`, `AgentProfilesPanel.tsx:86-89` | every failure collapses to one sentence, bypassing the product's "refused by name" rule; `setState` after unmount is possible in the memory panel (no `AbortController`, unlike `store.ts:944-970`) |
| F-UI-26 | No run picker and no `runId` on `HistoryState` | `store.ts:149-162`, `store.ts:662-674`, `HistoryPanel.tsx:26-29` | with several committed runs only the auto-chosen one is reachable; the header prints the live run id beside possibly another run's revisions |
| F-UI-27 | Scrubber's fallback index mixes a revision number with an array index | `Scrubber.tsx:18-21` (`Math.min(count - 1, history.selected)`) | if `selected` is ever not in `revisions`, the thumb lands on an unrelated revision (today `applyHistoryRevisionList` keeps it a member, so this is latent) |
| F-UI-28 | `columnHiveLayout` has no cycle guard (unlike `AgentHierarchy`) | `graph.ts:44-118` (`measure`/`placeColumn` recurse), vs `AgentHierarchy.tsx:34-52` (visited set) | a cyclic graph payload would recurse to stack overflow and blank the canvas; the server rejects cycles (`graph.rs:160-171`) so this is robustness only |
| F-UI-29 | Library rail disappears below `lg` with no replacement for its tabs | `LibraryPanel.tsx:45` (`hidden ... lg:flex`) | below 1024 px the workspace/swarm/routing/history tab control is unreachable (feature loss; the skill seat's B17) |
| F-UI-30 | Header quick-search has no outside-click/Escape-on-close, no `aria-controls`, and mutates selection during replay | `Header.tsx:68-79,102-142` | popover can only be closed by re-clicking the icon or focusing the input and pressing Escape |

---

## 3. Bugs to add to known areas (falsifiable statements)

Each statement names the observation that would prove it false. None of these is a claim that a
fix has landed.

1. **Hydration after a server restart.** *"After the Rust core restarts under an open tab, the UI
   rehydrates run/tasks/history within one reconnect cycle."* — **False today:** the client hellos
   with the server's own `last_seq` (`client.ts:309,319`), so `bus.replay_after` returns
   `Replay::Current` and the store's hello gate (`store.ts:1381-1391`) sees `snapshot.status ===
   'ready'` and reads nothing. Disproof: a test in which `/api/session` returns `last_seq = 0`
   after the client has applied seq 57 — today no `GET /api/history/runs` is issued at all.
2. **Hydration does not overwrite newer server state with older client state.** — **False today:**
   the merge is `{ ...projection.tasks, ...liveTasks }` with no rev comparison (`store.ts:1191-1194`).
   Disproof: a fixture where a live node has `rev: 5` and the snapshot has `rev: 9`; assert the
   store ends at `rev: 9` (today it ends at 5).
3. **Hydration leaves no stale claim on screen.** — **False today:** `snapshot.status/refusal/
   incomplete` and `history.refreshRefusal` have zero renderers (grep over `web/src/**/*.tsx`).
   Disproof: force `GET .../messages` to 500 and observe any UI text other than an empty chat.
4. **A terminal run leaves no actionable approval.** — **False today:** terminal `RUN_STATUS`
   does not touch `approvals` (`store.ts:1426-1467`) and the server publishes no
   `APPROVAL_RESOLVED` on cancel (`execution.rs:1114-1131`). Disproof: cancel a run with an open
   approval and observe a Go/No-Go control still enabled.
5. **The approval dialog is keyboard-operable.** — **False today:** no initial focus, no trap, no
   Escape (`ApprovalOverlay.tsx` has no ref/keydown). Disproof: Tab into the page with an approval
   open and observe focus reaching background controls.
6. **Exactly one modal dialog is in the DOM for N approvals.** — **False today:**
   `ApprovalOverlay.tsx:221` maps N approvals to N `aria-modal` dialogs. Disproof: seed two
   approvals and count `[aria-modal="true"]` (expect 2).
7. **Rewrite (re-route) always sends the newest known node revision.** — **False today:**
   `fetchNodeServerRev` overwrites with the fetched value (`store.ts:1323-1326`). Disproof: emit a
   higher-rev `NODE_STATE` while the fetch is in flight; assert the sent `expected_rev`.
8. **A missing `parent_task_id` cannot leave a stale edge.** — **False today:** the decoder
   yields `null` for both "omitted" and "explicit null" and the store falls back to
   `existing.parentId` (`store.ts:1519`). Disproof: send `NODE_STATE` for a node with a known
   parent and no `parent_task_id`; observe the edge is still drawn.
9. **Every replay counter describes the replayed revision.** — **False today:** the canvas count
   (`Canvas.tsx:640`), minimap empty class (`:696`) and HUD delta (`HudBadge.tsx:57-59`) are live.
   Disproof: replay revision 1 of a 9-node live run and observe `9/9` or a delta.
10. **Replay is not reachable by another client's gesture.** — **False today:** `HISTORY_SNAPSHOT`
    is applied without a correlation check (`store.ts:1728-1749`) on a broadcast bus.
    Disproof: two store instances on one bus; one sends `HISTORY_REVISION_SET`; the other's
    `replayGraph` becomes non-null.
11. **The library rail and header search do not write selection during replay.** — **False today:**
    `LibraryPanel.tsx:110` and `Header.tsx:130` call `selectTask` with no replay guard.
    Disproof: enter replay, click a rail chip, observe `selectedTaskId` change.
12. **A rewire control cannot target its own selection.** — **False today:** `target` is not reset
    on selection change (`ControlBar.tsx:23,73-75`), and `canRewire` only compares to `''`.
    Disproof: the A -> T -> T sequence above; observe `REWIRE_TASK{task_id===new_parent_id}`.
13. **Kill is offered for every task the store can kill.** — **False today:** the bar requires
    `worker` (`ControlBar.tsx:38`); the store accepts `worker|tool` (`store.ts:2010`).
    Disproof: select a `tool` task; observe Kill disabled.
14. **A pending command cannot disable Cancel on a live run.** — **False today:**
    `anyPending` gates `canCancel` (`ControlBar.tsx:27,36`). Disproof: send `FILE_READ` with a slow
    transport; observe Cancel disabled for the timeout.
15. **The bar shows the run's real terminal status.** — **False today:** it prints
    `STRINGS.states.unavailable` (`ControlBar.tsx:122`, `strings.ts:193`). Disproof: complete a
    run; observe "Unavailable".
16. **Swarm counts partition the graph.** — **False today:** the census has no `subagent` bucket
    (`SwarmPanel.tsx:21-27`). Disproof: a run with 2 subagents; the five buckets omit them.
17. **UI-spawned agents get their persona's tool narrowing and spawn limits.** — **False today:**
    the UI sends `'SuperAgent'`/`'WorkerAgent'`/`'Subagent'` (`store.ts:1929-1935`) and the backend
    looks those up case-sensitively (`execution.rs:550-558,868-880,1084`). Disproof: spawn from the
    bar, then read the task's `ScopedContext.allowed_tool_ids` — it is the tier default, not the
    persona list.
18. **Memory is visible per hive/branch.** — **False today:** the only read route is persona-scoped
    (`http.rs:985-1001`) and the query ignores `run_id`/`task_id` (`store.rs:649-668`).
    Disproof: two runs in one DB, one memory write each; no route returns them separately.
19. **A2A discovery, answer, topic and mute exist.** — **False today:** no roster consumer in
    `web/src`; only two message kinds (`execution.rs:2477,2599`); no topic/mute column
    (`db.rs:163-172`). Disproof: any call to `/roster`, any message row with another kind, any
    `mute` setting.
20. **Remote/Watchdog agents can be displayed.** — **False today:** six node kinds
    (`contract.ts:213`), zero producer hits for `Watchdog|RemoteAgent|heartbeat|Daemon` in
    `backend/**/*.rs`, no route (`http.rs:514-532`).
21. **Chrome never claims a capability the data lacks.** — **False today:** `nodes.tsx:159-162` +
    `strings.ts:281` print "owns the sub-hive" on every SuperAgent, while no owner/hive field
    exists (`store.ts:67-87`); `SwarmPanel.tsx:34-36` states "Re-parenting is only possible while a
    branch is not streaming", while rewire is gated only by run-active (`store.ts:704-721`).
    Disproof: any `owner`/`hive` field in a task payload, or a streaming-check in the rewire path.

---

## 4. SOLID / recon facts (no edits proposed this round)

| Principle | `path:line` | Fact | Consequence |
|---|---|---|---|
| SRP | `web/src/state/store.ts:758-2353` (2358 lines) | one factory owns transport events, HTTP hydration, commands, chat stream, approvals, settings, files, project identity | any hydration test must construct the whole store; the round-9 plan already defers a split (`lead-plan.md:26`) |
| OCP | adding one node kind needs **8** edits: `contract.ts:213`, `agentState.ts:126`, `:129-135`, `nodes.tsx:302-310`, `graph.ts:31-38`, `metrics.ts:9-12`, `Canvas.tsx:698-713`, `SwarmPanel.tsx:21-27` | kind registry is open-coded in every consumer | Remote/Watchdog/`subagent`-tier work repeats the shotgun; a descriptor registry is the obvious seam (`lead-plan.md:28`) |
| DIP | `store.ts:944-970` (`historyFetch`), `:1059-1100` | the store builds URLs + fetch + decoders inline; `UiStoreOptions` (`:210-213`) injects only `fetchImpl`/timeout | hydration cannot be tested without stubbing global `fetch`/URLs; boot tests (`store.test.ts:75-80`) issue real fetches and swallow the failure |
| ISP | `api/client.ts:49-61` | `UiApi` is 6 members and adequate | no change |
| LSP | `panels/ApprovalOverlay.tsx:157-163,193-194,224-225` | the panel widens the store's `ApprovalRecord` with `as unknown as ApprovalView` twice | if the store drops a field, the panel silently renders `—` instead of failing typecheck; the cast hides the contract |
| Dead abstraction | `api/client.ts:80-85,587-600`; `api/index.ts:105-116` | `pullPendingApprovals`/`supportsPendingApprovalPull` exist end-to-end but **no caller** | dead code that claims an approval-hydration capability the store never invokes (reconnect replay happens via hello instead) |
| Dead state | `store.ts:174-178,750-756,160-161` | `SnapshotIncomplete`, `isSnapshotDegraded`, `history.refreshRefusal` are written/exported and never consumed | the error-UI gap is a wiring gap, not a modeling gap |
| State model gap | `store.ts:149-162` vs `:190` | `HistoryState` has no `runId`; `snapshot.runId` holds it | a run picker cannot be added without deciding which object owns "whose history is this" |
| Wire drift (harmless) | `contract.ts:232-242` `label?: string` | the decoder accepts `label`, but the server `NodeEvent` has no `label` field (`event.rs:248-270`) | `payload.label` is always `null`; live labels come from `fallbackTaskLabel` (`store.ts:331-334`), i.e. `"WorkerAgent <8>"` style, while the committed projection has `node_label_for` labels (`http.rs:753-765`). Live and replayed labels for the same node can differ |
| Test boundary | `web/vite.config.ts:26-30` | vitest is `environment: 'node'`, `include: tests/**/*.test.ts`; component coverage is `react-dom/server` static markup | keyboard/focus/DOM bugs (F-UI-04) are structurally invisible to the current suite; adding a DOM test is a tooling decision, not a test-only edit |

---

## 5. Cross-seat de-duplication (do not fix twice)

The `/deepseek-unslop` seat (`docs/trace/round9/skill-unslop.md`) already owns these
overlapping observations, with its own severity labels. This report adds the recon dimensions
(callers, tests, blast radius, failure path) and does not argue them again:

| This report | skill-unslop id | Overlap |
|---|---|---|
| F-UI-09 (HUD delta in replay) | `C8` / `B1` | identical evidence (`HudBadge.tsx:39-40` vs `:57-59`) |
| F-UI-09 (library live during replay) | `C13` / `B2` | identical |
| F-UI-09 (canvas count + minimap during replay) | `B16` | identical |
| F-UI-04 (approval focus/Escape) | `C17` | identical; this report adds "no DOM test exists" + the multi-modal count |
| approvals during replay | `B5` (`ApprovalOverlay.tsx:221`) | same surface as F-UI-04/F-UI-05; treat as one fix |

---

## 6. Unverified register

| ID | Claim | Why unverified | What would prove it |
|---|---|---|---|
| U-UI-1 | The restart-hydration gap (F-UI-01) is user-visible in the shipped launcher | app stopped; no restart performed; port 8080 currently held by a foreign Vite process (skill seat §0) | run the core, open the UI, kill/restart the core, observe the run rail without a reload |
| U-UI-2 | F-UI-02's stale-state outcome occurs on the real resync path | no server, no event replay driven | a fake-bus fixture: live node rev 5 -> `RESYNC_REQUIRED` -> snapshot rev 9 |
| U-UI-3 | F-UI-05 needs a second click to clear (not one) | not driven against a server | cancel a run with an open approval and count gestures to recover the UI |
| U-UI-4 | F-UI-13's drag-onto-descendant is reachable with real data (needs a superagent child) | planner layout not observed this round | live run whose tree has a nested superagent |
| U-UI-5 | F-UI-30's header popover behavior | no browser | click-outside test |
| U-UI-6 | F-UI-22 vs the real census (subagent counts) | no live multi-kind run | a run with subagent tasks |
| U-UI-7 | Any keyboard/focus fix is testable in this repo | no DOM environment; no `@testing-library/*`, `jsdom`, or `happy-dom` in `web/package.json` | add a DOM test env (tooling decision) or verify manually |
| U-UI-8 | Backend-version-dependent claims (rev always present, parent omitted when null) | read from source, not from a live frame capture this round | one captured `ws` frame set from a live run |
| U-UI-9 | `personas.json` persona ids are lowercase | inferred from `AgentProfilesPanel.tsx:23-31` and the case-sensitive lookups; the JSON was not opened | read `personas.json` |
| U-UI-10 | e2e specs still match the current Canvas (`fitView={history.replayGraph !== null}`, `Canvas.tsx:689`) while `e2e/specs/replay-state.spec.ts:114-115` documents the opposite | e2e not run (browser) | run the replay e2e |

---

## 7. Test commands (named, not run by this seat)

| Purpose | Command |
|---|---|
| Web unit/integration (baseline 187 per `lead-plan.md:52`) | `cd web; npm test` |
| Replay projection | `cd web; npx vitest run tests/graph.replay.test.ts` |
| Operator surfaces (static render) | `cd web; npx vitest run tests/agentsToolActivityMount.test.ts tests/toolActivityPanels.test.ts` |
| Approval state machine | `cd web; npx vitest run tests/approval.state.test.ts tests/approval.integration.test.ts` |
| Streaming/terminal ordering | `cd web; npx vitest run tests/streamingRender.test.ts` |
| Store commands | `cd web; npx vitest run tests/store.test.ts` |
| Types | `cd web; npx tsc --noEmit` |
| History/roster contract (backend, baseline 4/4) | `cd backend; cargo test -p hiverouter-core --test snapshot_history` |
| Live replay (needs a browser + server: **not permitted**) | `cd e2e; npx playwright test specs/replay-state.spec.ts` |

---

## 8. Files a fix would touch (expectation only; nothing was edited)

| Fix candidate | Files |
|---|---|
| F-UI-01 restart hydration | `web/src/api/client.ts`, `web/src/state/store.ts`, `web/tests/apiLifecycle.test.ts`, new hydration test |
| F-UI-02 rev-aware merge | `web/src/state/store.ts`, new hydration test |
| F-UI-03 error UI | `App.tsx` (new banner component), `panels/HistoryPanel.tsx`, `chat/ChatCore.tsx`, test |
| F-UI-04/05 approvals | `panels/ApprovalOverlay.tsx`, `state/store.ts` (terminal sweep), DOM-test decision |
| F-UI-06 memory | `panels/AgentMemoryPanel.tsx`, `state/store.ts` (canonical role), `store.rs`/`http.rs` (scope filter), tests |
| F-UI-07 A2A | `panels/AgentMessagesPanel.tsx`, `panels/AgentProfilesPanel.tsx` (labels), `state/store.ts` (mute, if client-side), backend answer slice if required |
| F-UI-08/11 rewire | `web/src/state/store.ts`, `web/tests/store.test.ts` |
| F-UI-09 replay | `flow/Canvas.tsx`, `panels/HudBadge.tsx`, `panels/LibraryPanel.tsx`, `panels/Header.tsx`, `state/store.ts` |
| F-UI-12/13/14/15 canvas | `flow/Canvas.tsx`, `flow/graph.ts` |
| F-UI-16/17 chat | `chat/ChatCore.tsx`, `state/store.ts` |
| F-UI-18..21 ControlBar | `panels/ControlBar.tsx` (+ maybe `state/store.ts` for a shared precondition helper) |
| F-UI-22/23 swarm | `panels/SwarmPanel.tsx`, `panels/AgentHierarchy.tsx` |
| F-UI-26/27 history | `panels/HistoryPanel.tsx`, `panels/Scrubber.tsx`, `state/store.ts` |
| F-UI-29/30 chrome | `panels/LibraryPanel.tsx`, `panels/Header.tsx` |

---

## 9. Compliance statement

- Edited: this file only.
- Tests, config, product code, binary: untouched.
- HiveMind process: not started, stopped, or signalled; no socket opened; no `fetch` to the app.
- Provider key: not read, not used; no provider call.
- Browser: not used.

**Recon completeness:** each of the ten ledgers names an entry point, at least one caller, a test
command, a failure path and the unknowns. Where a ledger could not name a test (L1, L2, L3, L6,
L7, L9), that absence is itself reported and is the reason those rows are findings rather than
assertions.
