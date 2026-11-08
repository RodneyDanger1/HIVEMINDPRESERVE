# 03 — FORGE root-cause audit (SEAT C)

Status: **AUDIT ONLY — no product file was edited, no process was restarted, no provider was called, no key was read.**
Owner: SEAT C (`/forge`).
Date: 2026-11-07T23:22-06:00.
Workspace: `C:\Users\Riley\Desktop\hiverouter-v1`.
Report path: `docs/trace/orca-wave/03-forge.md` (the only file written by this seat).

---

## 0. Method, pinned artifacts, environment

### 0.1 What was read (current content, hashes pinned)

| File | SHA-256 (at analysis) |
|---|---|
| `web/src/flow/Canvas.tsx` | `ACEE54C8630E7A3995B52262AE7012C49311A15B73C82C4072ADF1CAB05B0242` (805 lines, 23:25:20) |
| `web/src/flow/graph.ts` | `600CF896E1E969C41DD16587D71D8A34F7F77E1A613FE5BE9FEC4FC4DD39F5B8` |
| `web/src/flow/nodes.tsx` | `1EA92BB104178E27D68D997B8FABE9E4249D6FA47F31A4A2678D9D42A3C1DEFB` |
| `web/src/state/store.ts` | `E4F163176B6E44D44831A8C5426F1F7A6AE26BE4922F40F75E43C4450A634C2E` |
| `web/src/state/agentState.ts` | read; `isTerminal` at `111-119` |
| `backend/.../hiverouter-core/src/ws.rs` | `F036F47123ACAA942CC399A913484626741997B7CFFB86F589B5F652F1553AA2` |
| `backend/.../hiverouter-core/src/graph.rs` | `8B3A24A29E425EE41ABD2F5DF7CBEA46E3326797185C8D2EDA5620E9FC1680C3` |
| `backend/.../hiverouter-core/src/snapshot.rs` | `2954FF374BE2DBC6E4E42D0D396795DF35864D8FFDA2F57E97413EC29C9F3398` |
| `backend/.../hiverouter-core/src/recovery.rs` | `B35493C23123EAB3656689566B9002B6A44A52186DF3F209AC60D72E00F6D1AE` |
| `backend/.../hiverouter-core/src/execution.rs` | `F02F032A99CAD91E593CCB0140423AB990112515C86F7F417D9345E945A53AC0` |
| `backend/.../hiverouter-core/src/context.rs` | `3839628D1C38E8EE1796DABF75538AC74DB064296461432EF937724CB6B0A2C2` |
| `backend/.../hiverouter-core/src/http.rs` | `AC5F9B96AAC81990DFF96B6D6884CC5FAF812D7CD274ABFC3DCA60A2565F77A4` |
| `backend/.../hiverouter-contract/src/event.rs` | `939FDECADBD7F00329DD292F3BC01CC210042E0C773ABA1F685A2893FEF2DEE0` |

**Concurrent-edit warning (material).** `web/src/flow/graph.ts` (23:11:44) and `web/src/flow/Canvas.tsx` (23:11:51,
then again 23:25:20) were edited by another seat *while this audit was reading them*. Three observed changes:
the radial hive layout (`radialHiveLayout`, commit `ff4f4ea`) was replaced by `columnHiveLayout`; the auto-fit zoom
was re-pinned from `minZoom 0.5 / maxZoom 1` to `minZoom 0.75 / maxZoom 0.75`; and the replay path gained its own
`HIVE_ANCHOR_ID` node (`Canvas.tsx:434-454`) plus a duplicated depth-1 `replayOwnerLinks` (`Canvas.tsx:456-494`,
selected at `496`). Every line number below is the **Canvas hash above / graph hash above** content. Where the
replaced radial code is quoted it is marked *superseded*. Re-pin the hashes before applying any fix; two of the
findings below (D9, P2-1) were partly touched by these concurrent edits and are worded against the newest content.
The defect **classes** (no chat-core reserve in the hive layout; unplaced nodes; clamped fit; depth-1 ownership)
survive the rewrite — see D8/D9/D10.

### 0.2 Live runtime inspected

- Binary: `dist\HiveMind\hiverouter.exe` (pid 15484 at first probe), `GET /health` →
  `{"status":"degraded","version":"1.2.0-dev","schema":"hiverouter.v1",...,"database":{"schema_version":10},"transport":{"last_seq":0,"graph_nodes":35,"allowed_origins":2}}`
  with `deepseek_api_key` **absent** (`E_KEY_MISSING`, keyless app — no provider call was made by this seat).
- **The server process exited on its own during the audit** (after the last probe; not killed or restarted by this seat).
  It was not restarted. All live evidence below was captured before it exited and is quoted verbatim.
- The served bundle is **older than the working tree**: the live canvas booted with `hide finished` *unpressed*
  (`aria-pressed=false`, 4/4 nodes), while the current source defaults `hideFinished` to `true`
  (`Canvas.tsx:112`). Runtime and source are therefore quoted separately and never conflated.

### 0.3 Probe scripts (written outside the repo, read-only)

`%TEMP%\orca-forge-probe.mjs` (Node WS — refused: origin allowlist), `%TEMP%\orca-forge-browser-probe.mjs`
(Playwright `page.evaluate` WS round-trips + DOM measurement, i.e. a real `Origin` header). Neither script mutates
server state: `HISTORY_REVISION_SET` is a read; `REWIRE_TASK` was sent with a stale `expected_rev` (refused before
mutation at `graph.rs:150-155`); `SPAWN_SUB_HIVE` targeted a terminal run (refused before insert at
`execution.rs:1071-1073`).

---

## 1. Symptom → defect map

| Reported symptom | Defect(s) that produce it | Primary hop |
|---|---|---|
| agents detached / flattened | D1, D7, D8, P2-1 | kind is not in the replay payload → no hive root → dagre fallback |
| SuperAgent ownership | D10, D7 | ownership is depth-1 screen-space lines; no owned-child data exists |
| radial layout clipping / overlap | D8, D9 | hive layout has no chat reserve; fit is clamped to 0.75 |
| drag reparent only preview | D3, P2-2 | rewire refused for terminal runs; drag offset persisted before confirm |
| subtree / memory / tool scope not changing | D4 | rewire swaps facts but keeps the task's own tool/schema/memory scope |
| run-scoped snapshot / recovery | D5, D6, D7 | recovery captures unscoped; WS replay is unscoped; client ignores `envelope.run_id` |
| role metadata | D7, P2-3 | `NODE_*` has no role/label; `role` is dropped when a live frame rebuilds the record |
| all-agent cards absent | D2 | `hideFinished=true` + terminal reconcile ⇒ 0 visible cards, no message |

---

### 1.1 Top 10 index (one line each; details in §2)

| # | ID | Sev | Claim | Primary cause | Symptom |
|---|---|---|---|---|---|
| 1 | D1 | P0 | `HISTORY_SNAPSHOT` carries no `kind`; every replay node becomes a Worker and the hive layout degrades to dagre | `ws.rs:832-849` vs `http.rs:692-712` | flattened |
| 2 | D2 | P0 | A finished run paints zero cards by default and says nothing | `Canvas.tsx:112,196,783-789` | cards absent |
| 3 | D3 | P0 | "Connect" is refused with `E_RUN_NOT_FOUND` on every terminal run; the drag offset is persisted before the dialog | `store.ts:1748-1755`, `Canvas.tsx:667-680` | reparent = preview |
| 4 | D4 | P1 | A committed rewire leaves tool ids/schema/tier/memory untouched and clears only the immediate parent's facts | `ws.rs:350-356`, `execution.rs:880`, `context.rs:148-159` | scope never changes |
| 5 | D5 | P1 | Recovery loads one run's image globally, persists it unscoped for every changed run, and re-adds parentless tasks at top level | `recovery.rs:151,279,408` | run-scoped recovery |
| 6 | D6 | P1 | WS replay is unscoped while the HTTP read is scoped (live: 35 vs 4 nodes) | `ws.rs:832-849` vs `http.rs:678-691` | run-scoped snapshot |
| 7 | D7 | P1 | `NODE_*` frames are applied without a run check and the rebuilt record drops `role` | `store.ts:1461-1513`, `event.rs:248-271` | role metadata |
| 8 | D8 | P1 | Nodes unreachable from the hive root get no position and pile up at `(0,0)` under the chat | `graph.ts:43-103,226` | detached/overlap |
| 9 | D9 | P1 | The hive layout has no chat reserve and both `fitView` calls are pinned to 0.75 (a fit that cannot fit) | `graph.ts:43-103`, `Canvas.tsx:365,688` | clipping |
| 10 | D10 | P1 | Ownership is depth-1 only and no card receives owned-child data | `Canvas.tsx:312-351`, `nodes.tsx:141-173` | SuperAgent ownership |

---

## 2. Ranked register — top 10

Legend: **RED** = the check that must fail before the fix; every RED below is stated as a command/assertion that is
runnable in this repo's existing harnesses (`web`: `vitest`; `backend`: `cargo test`).

### P0

---

#### D1 — Replay projection has no `kind`, so every historical agent renders as a Worker and the hierarchy flattens

**Falsifiable sentence.** For a committed revision of a run whose root task role is `planner`,
`HISTORY_SNAPSHOT` carries nodes with **no** `kind`/`role`/`label`, and the client therefore lays the graph out with
`dagre` instead of the hive layout and renders the root as a `worker` card.

**Reproduction (live, captured before the server exited).**
`HISTORY_REVISION_SET {run_id: 01a1b931-941a-7171-a209-1c05777354f8, revision: 7}` over WS returned:

```
frame_names: ["ACK","HISTORY_SNAPSHOT"]
node_count: 35, edge_count: 26, roots: 9
first_node_fields: ["task_id","parent_task_id","state","rev"]
kinds: ["<absent>"]   roles: ["<absent>"]
```

while the HTTP read of the *same* revision (`GET /api/history/runs/{id}/state?revision=7`) returned
`nodes=4` with `kind: "hyperagent"`, `role: "planner"`. Two authorities, one revision, different projections.

**Proven cause.**
- `backend/crates/hiverouter-core/src/ws.rs:832-849` — `Command::HistoryRevisionSet` serialises the persisted
  `GraphSnapshot` verbatim: `let graph = match serde_json::to_value(envelope.graph)`. `GraphSnapshot`/`TaskNode`
  (`graph.rs:18-27`) carry only `task_id`, `parent_task_id`, `state`, `rev` — the immutable snapshot **has no
  kind/role/label by construction**.
- The enrichment that makes the HTTP route correct exists only on the HTTP route:
  `http.rs:692-712` — *"The immutable graph snapshot stores only topology. The task rows are the authority for
  role/kind, so enrich the read projection rather than flattening every historical node into `worker`."*
  followed by `node_kind_for_role` (`http.rs:739-746`) and `node_label_for` (`http.rs:748-765`).
- Client fallback: `web/src/flow/graph.ts:287` — `const kind = isNodeKind(node.kind) ? node.kind : 'worker';`
  and `graph.ts:106-111` — the hive layout is chosen only when a `hyperagent` task with `parentId === null`
  exists; otherwise `dagre` (`graph.ts:112`).

**Minimal fix.** Make the WS path use the same projection as the HTTP path: add one server-side function
`history_state_projection(store, graph, run_id) -> serde_json::Value` (the body of `http.rs:678-712`) and call it
from `ws.rs:832` instead of `serde_json::to_value(envelope.graph)`. Do not duplicate the role table.

**Regression test.** `backend/crates/hiverouter-core/tests/snapshot_history.rs` —
`history_snapshot_event_carries_role_kind_and_label`: insert a task row with `role = "planner"`, commit a revision,
send `HISTORY_REVISION_SET`, assert the emitted `HISTORY_SNAPSHOT.graph.nodes[0].kind == "hyperagent"` and
`label` is non-empty. RED on today's code: `kind` is absent ⇒ `null != "hyperagent"`.

**Residual risk.** The enriched projection derives `kind` from the *free-form* role string; a role that drifts from
the vocabulary silently maps to `worker` (fails closed, but silently). Once D7 is fixed the wire carries a real role
and this derivation can be replaced, not extended.

---

#### D2 — A finished run's canvas is empty by default, with no explanatory message

**Falsifiable sentence.** With the current source, after any run reaches a terminal state (every task settled by
`reconcile_terminal_tasks`), the workspace canvas contains **zero** agent cards on first paint, because
`hideFinished` defaults to `true`, `isTerminal` includes `completed`, and the "no agents match" message is gated on
the chat being hidden.

**Reproduction.**
1. Source: `Canvas.tsx:112` `useState(true)`; `Canvas.tsx:195-197`
   `Object.entries(tasks).filter(([, task]) => !hideFinished || !isTerminal(task.state))`;
   `agentState.ts:111-119` (`completed` is terminal).
2. Live (bundle with the filter *off* at boot): DOM `.react-flow__node` = **4** (3 × `node-worker` + 1 `hiveanchor`),
   chip text `4/4`. After one click on `Hide finished`: `rfNodes = 0`, `kinds = {}`, minimap nodes `0`,
   chip text `0/4`, canvas text contains **no** empty-state copy
   (`emptyFilterMsg: false`, `agentsEmpty: false`).
3. Why no message: `Canvas.tsx:783-789` renders the "No agents match the current filters." pill only when
   `!chatVisible`; the chat is visible by default, so a filtered-to-nothing canvas is silent.
4. Why every card is terminal: `execution.rs:1226-1253` `reconcile_terminal_tasks` settles **every** non-terminal
   task of a terminal run (including the structural root/superagent rows) and publishes `NODE_STATE` for each
   (`execution.rs:1250`).

**Proven cause.** `web/src/flow/Canvas.tsx:112` (default) × `Canvas.tsx:196` (filter) × `agentState.ts:111-119`
(terminal set) × `Canvas.tsx:783-789` (message gate). No single line is wrong in isolation; the composition makes
the default state of the flagship surface empty and silent.

**Minimal fix.** Default `hideFinished` to `false` (one token) **and** un-gate the filter message from `chatVisible`
(`Canvas.tsx:783`, drop `&& !chatVisible`) so a filtered-empty canvas always says why. Do not add new UI.

**Regression test.** `web/tests/canvas.filters.test.ts` — `terminal_run_keeps_cards_visible_by_default`:
build a task map with one `hyperagent` + two `completed` workers, render `Canvas` inside `ReactFlowProvider`
(or assert on the pure filter by exporting it), assert `document.querySelectorAll('.react-flow__node').length === 3`.
RED on today's default ⇒ `0`.

**Residual risk.** Showing finished cards by default makes a 35-node legacy graph denser; that is D1's problem, not
this one, and is contained by the existing filter control.

---

#### D3 — "Connect" on a dropped card can never reparent a finished run, and the drop itself is persisted regardless

**Falsifiable sentence.** On every run a user can actually see (terminal), confirming the drag dialog calls
`rewireSelected`, which immediately returns with `E_RUN_NOT_FOUND` *for a run that exists*, while the card's new
position has already been written to `localStorage` — so the only durable effect of the gesture is a moved card.

**Reproduction.**
1. Complete any run (or load the current app: the chosen run is terminal).
2. Drag a worker card onto another card → the drop-target highlight and dashed preview line appear
   (`Canvas.tsx:247-261`, `292-310`); on drop the confirm dialog opens (`Canvas.tsx:263-280`).
3. Press **Connect** → `confirmReparent` (`Canvas.tsx:282-290`) → `rewireSelected` (`store.ts:1895`) →
   `guardActiveRun()` (`store.ts:1748-1755`) → `pushRefusal(clientRefusal('E_RUN_NOT_FOUND'))`, `return`.
4. The card stays where it was dropped, because the position override was saved **before** the dialog existed:
   `Canvas.tsx:667-677` `setPositionOverrides(...); savePositions(runId, next);` runs on every `onNodeDragStop`,
   and `Canvas.tsx:680` only *then* calls `handleNodeDragStop`; the override is re-applied forever from
   `loadPositions(runId)` (`Canvas.tsx:51-78`, `125-127`, `206-217`).
5. Press **Cancel** instead: the moved card stays moved too. Cancel only clears `pendingReparent`
   (`Canvas.tsx:591-593`).

**Proven cause.** Two independent hops:
- `store.ts:1748-1755` refuses with `E_RUN_NOT_FOUND` while `store.ts:704-721` (`rewireBlocker`) would have
  returned `run-terminal` ⇒ the canvas path never reaches the honest refusal code (`E_RUN_TERMINAL`) the
  ControlBar path would produce.
- `Canvas.tsx:667-680` persists the position before, and independently of, the graph mutation (persistence at
  `677`, confirm dispatch at `680`). There is no producer for `NODE_DRAG` on the server (see P2-4), so the move
  can only ever be client-local until a rewire succeeds.

**Minimal fix.** In `store.rewireSelected`, replace `guardActiveRun()` with the same `rewireBlocker` funnel used
below it (returning `E_RUN_TERMINAL` for a terminal run and `E_TASK_NOT_FOUND` only for a missing run), and move
`savePositions` out of `onNodeDragStop` into the path that runs only when `run('REWIRE_TASK', …)` returned `true`
(`store.ts:1940-1947`) — i.e. persist a drag only after the operator confirmed *and* the server accepted. Do not add
UI.

**Regression test.** `web/tests/store.test.ts` —
`rewire_on_terminal_run_refuses_with_terminal_code_not_run_not_found`: seed `run.status = 'completed'`, call
`rewireSelected(parent)`, assert `refusals[0].code === 'E_RUN_TERMINAL'`. Second test in
`web/tests/canvas.drag.test.ts`: a drag-stop with no confirm leaves `localStorage` without a position entry.

**Residual risk.** A successful rewire still leaves the card at its dropped pixel (the override wins over the new
layout) — that is a *typing* problem (layout vs. override precedence), tracked as part of D8's fix, not hidden here.

---

### P1

---

#### D4 — A committed rewire changes the graph edge but not tool, memory, or subtree scope

**Falsifiable sentence.** After `REWIRE_TASK` is ACKed, the task's `ScopedContext.allowed_tool_ids` and
`output_schema_id` are byte-identical to their pre-rewire values, its memory facts are still those of its original
role namespace, and facts inherited from the old branch's *grandparent* remain in the context.

**Reproduction.**
1. Code: `ws.rs:340-357` — `apply_rewire_context` passes `current.allowed_tool_ids` and `current.output_schema_id`
   into `switch_on_rewire` unchanged; the doc comment says so explicitly (*"keep the task's own values: they are the
   task's persona capability set"*).
2. Memory is keyed by the immutable role, not by the branch: `execution.rs:880`
   `if let Ok(memories) = self.store.agent_memory(&task.role, 8)`. Nothing re-seeds it on rewire.
3. Tier (which drives `validate_spawn_role`) is derived from the same immutable role: `execution.rs:151-163`.
4. Subtree facts: `context.rs:148-159` clears only keys with the prefix of `current.parent_task_id`
   (`parent_prefix(old_parent)`); keys scoped to the old parent's ancestors (`parent:<grandparent>:…`) are left in
   `facts` and stay visible to the agent (`context.rs:66-95` renders every non-secret fact).
5. The UI already admits the gap it cannot close: `Canvas.tsx:584-585` —
   *"The card moves with this connection. Memory, tool and message scope changes are not applied yet; the server
   rewire is the first step."* There is no second step anywhere in the tree (grep for `rewire` in
   `backend/crates/hiverouter-core/src`: only `graph.rewire`, `begin/finish_rewire`, `apply_rewire_context`).

**Proven cause.** `ws.rs:350-356` (scope carried through unchanged) + `execution.rs:880` and `execution.rs:151-163`
(scope keyed to an immutable role) + `context.rs:148-159` (clear depth = 1).

**Minimal fix.** Make the rewire re-derive authority: after `graph.rewire` succeeds, recompute the task's tier from
its *new parent* (`task_agent_tier(parent)`, bounded by `min(child_tier, parent_tier)` so a rewire can only narrow),
set `allowed_tool_ids`/`output_schema_id` from that tier's persona, and re-seed memory for the new tier key. Extend
the fact clear to every `parent:` key not present in the new parent's context (replace the prefix delete with a
"keep only root facts + new parent's facts" rebuild). One new test each.

**Regression test.** `backend/crates/hiverouter-core/tests/graph_authority.rs` (or a new
`tests/rewire_scope.rs`): `rewire_narrows_tool_scope_and_drops_stale_ancestor_facts` — build root→A→task and
root→B, give A a `parent:<root>:style` fact, rewire `task` under B, assert (a) `allowed_tool_ids` equals B's tier
set, (b) `contains_parent_facts(root_of_A)` is false. RED today: both assertions fail.

**Residual risk.** Narrowing on rewire can strand a task mid-run with a tool it has already called; the engine's
approval gate (`tools_bridge`) must refuse the *next* call, not the committed one. This is why the fix is
"min(old, new)", not "new".

---

#### D5 — Recovery restores one run's image globally and persists it as every changed run's snapshot

**Falsifiable sentence.** When two runs have snapshots on disk, boot recovery loads them in turn into one in-memory
graph (last wins), then writes that single image as the *new revision of each changed run* using the unscoped
`capture`, and any task row whose node is not in the loaded image is re-added at the top level with no parent — so a
recovered revision can contain another run's nodes and a flattened tree.

**Reproduction.**
1. `recovery.rs:139-177` — `for (_, run_id) in restore_order { … graph.restore_from_snapshot(&envelope.graph) … }`.
   `restore_from_snapshot` (`graph.rs:304-327`) replaces `state.graph/index/nodes` wholesale, so after the loop the
   graph holds exactly the last restored run.
2. `recovery.rs:273-303` — for each run in `changed_runs`, `snapshot::capture(graph, run_id, revision, status)`
   (line 279). `capture` (`snapshot.rs:291-305`) is the **unscoped** capture; the scoped variant
   `capture_scoped` (`snapshot.rs:264-285`) exists and is the one the run-commit path uses
   (`execution.rs:1203`), so recovery is the outlier, not a missing capability.
3. `recovery.rs:389-414` — `settle_graph_node` fallback: when the parent is absent,
   `graph.add_task(task.id, None)` (line 408) with the warning *"parent node missing at recovery; adding the
   settled task without a parent edge."*
4. Validation of the class: the repository's own raw snapshot files show the unscoped capture in production — the
   newest run's on-disk revisions contain **35 nodes / 26 edges** while its authoritative state is 4 nodes
   (`C:\Users\Riley\AppData\Local\HiveMind\snapshots\01a1b931-941a-…\0000000007.json` → `nodes=35`).
   (Those files were written by a pre-`678748e` binary, which is exactly why the HTTP read boundary filters —
   `http.rs:678-691` — and why the WS path's missing filter (D6) is not theoretical.)

**Proven cause.** `recovery.rs:151` (single global graph) + `recovery.rs:279` (`capture` instead of
`capture_scoped`) + `recovery.rs:408` (`add_task(id, None)`).

**Minimal fix.** Three one-line-class changes: (a) capture with `capture_scoped(..., &allowed)` where `allowed` is
`store.tasks_for_run(run_id)`; (b) for a task whose parent is absent, do not invent a top-level edge — either skip
the graph write and record it in `report.graph_settle_failures` (already exists) or, better, restore per-run images
into a per-run graph; (c) restore only the newest run and report the others as not-restored instead of silently
overwriting.

**Regression test.** `backend/crates/hiverouter-core/tests/recovery_gates.rs` —
`recovery_snapshot_is_run_scoped`: seed two runs with two nodes each, force one interrupted, run
`recover_on_boot`, read the newest snapshot for run A and assert every node id is in `tasks_for_run(A)`.
RED today: run B's nodes are present.

**Residual risk.** Fixing (c) changes which run is visible after a crash; the scrubber/history already exposes other
runs, so this is a report change, not a data loss — but it must be stated in the release note.

---

#### D6 — Two authorities for the same revision: the WS replay read is unscoped, the HTTP read is scoped

**Falsifiable sentence.** For one revision of one run, `HISTORY_SNAPSHOT` and
`GET /api/history/runs/{id}/state` disagree on the node set (35 vs 4 on live data), and only the HTTP one is
run-scoped.

**Reproduction.** The D1 probe above (`node_count 35, roots 9` vs `nodes 4`). Repeating it is read-only.

**Proven cause.** `http.rs:678-691` filters `graph.nodes/edges/toposort` by `tasks_for_run(run_id)` at the read
boundary (*"Older snapshots predate run-scoped capture and contain the whole process graph"*); `ws.rs:832-849` has
no such filter.

**Minimal fix.** Same fix as D1: route both through one projection function that (a) filters by the run's task set
and (b) enriches kind/role/label. One function, two call sites.

**Regression test.** `backend/crates/hiverouter-core/tests/snapshot_history.rs` —
`ws_replay_and_http_state_agree`: write a snapshot containing two runs' nodes, then compare the node-id sets of the
WS event and the HTTP body; assert equality and that only the target run's ids remain. RED today.

**Residual risk.** The filter depends on `tasks_for_run` (DB), so a snapshot whose task rows were pruned becomes
empty rather than wrong — an honest empty state, which the client already renders (`buildReplayTasks` returns `{}`).

---

#### D7 — The client applies `NODE_*` frames from any run, and every live frame erases the task's role

**Falsifiable sentence.** `applyEvent` never compares `envelope.run_id` with the current run for `NODE_SPAWN`/
`NODE_STATE`/`NODE_REWIRE`, and the record it rebuilds omits `role`, so a frame from run B inserts a task into run A's
graph and a single live frame wipes a role that the snapshot had established.

**Reproduction.**
1. `store.ts:1312-1317` uses only `envelope.seq`; no other read of `envelope.run_id` exists in `store.ts`
   (grep: the only `run_id` matches are payload reads and command sends).
2. `store.ts:1374-1396` — `RUN_CREATED` for a *new* run clears `tasks: {}`, so any in-flight frame of the previous
   run lands in an empty graph.
3. `store.ts:1480-1498` — the rebuilt `TaskRecord` has `id/parentId/kind/label/state/progress/errorCode/rev`;
   `role` is absent although `TaskRecord.role` exists (`store.ts:73`) and snapshots populate it
   (`store.ts:637`).
4. The wire cannot even carry it: `event.rs:248-271` `NodeEvent` = `{task_id, state, rev?, kind?, parent_task_id?}`.
   No `role`, no `label`, no `progress`.

**Proven cause.** `store.ts:1461-1513` (no run gate, no `role` merge) + `event.rs:248-271` (no role on the wire).

**Minimal fix.** Two lines of behaviour: (a) `if (envelope.run_id !== null && envelope.run_id !== get().run?.id) return;`
at the top of the `NODE_*` arm; (b) `...(existing?.role === undefined ? {} : { role: existing.role })` in the rebuilt
record so a live frame preserves a snapshot-established role. Adding `role` to the wire is the durable fix and is
listed as the follow-on in D1's residual risk.

**Regression test.** `web/tests/store.test.ts` —
`node_state_from_another_run_is_ignored` and `node_state_preserves_snapshot_role`. RED today: the first asserts
`tasks` is unchanged and fails (a foreign task appears); the second asserts `tasks[id].role === 'planner'` and fails
(`undefined`).

**Residual risk.** Gating on `envelope.run_id` means a producer that omits the envelope run id (older) is accepted
unchanged — deliberate, and matches the additive-field policy in `event.rs:242-246`.

---

#### D8 — Nodes that are not reachable from the hive root get no position and collapse onto the world origin

**Falsifiable sentence.** `layoutTasks` assigns positions only to nodes reachable by walking `childrenByParent` from
the single hyperagent root; every other task (a re-rooted orphan, a second root, a child whose parent record is
absent) falls through `positions[task.id] ?? { x: 0, y: 0 }` and is painted under the fixed chat overlay at world
origin, stacked on its peers.

**Reproduction (unit, decisive).**
```ts
buildNodes({
  root:   { id:'root', parentId:null, kind:'hyperagent', … },
  orphan: { id:'orphan', parentId:null, kind:'worker', … },     // re-rooted orphan
  lost:   { id:'lost', parentId:'absent', kind:'worker', … },   // parent not in map
}, null, () => {})
```
positions: `root` is excluded from nodes; `orphan` and `lost` both get `{x:0,y:0}`.
Provenance of both shapes: `store.ts:634` re-roots a node whose parent is not in the snapshot
(`parentId: node.parentId !== null && ids.has(node.parentId) ? node.parentId : null`) and
`graph.ts:308-312` does the same for replay. Reachability of the third: server recovery creates exactly this shape
(`recovery.rs:408`).

**Proven cause.** `graph.ts:43-103` (`columnHiveLayout` walks only `childrenByParent[rootId]`; no fallback pass) +
`graph.ts:226` (`positions[task.id] ?? { x: 0, y: 0 }`). The same absence of a fallback pass exists in the dagre
branch: a node whose parent is absent is not `setEdge`-ed (`graph.ts:125-128`) yet dagre still places it, so only the
hive branch **stacks** them.

**Minimal fix.** After the column pass, place every still-unpositioned task (sorted by id) in its own column
offset beyond the outermost used column, i.e. a deterministic fallback row. No UI change.

**Regression test.** `web/tests/graph.replay.test.ts` — `unreachable_nodes_get_distinct_fallback_positions`:
assert all `buildNodes` positions are pairwise distinct for the three-node map above. RED today: `orphan` and `lost`
share `0,0`.

**Residual risk.** A fallback row can place a card off-screen for a very large orphan set; it is strictly better than
an invisible overlap, and it makes the orphan *visible* so the upstream data bug can be diagnosed.

---

#### D9 — The hive layout has no chat-core reserve, and the fit is clamped to a fixed 0.75 zoom

**Falsifiable sentence.** At the pinned fit zoom `0.75`, a 420-wide card in the first column (inner edge at world
x = 266) clears the 400-wide chat panel by 0.7 world units; at any zoom below `0.752` that card is painted under the
chat panel, and because `fitView` is called with `minZoom === maxZoom === 0.75` it cannot zoom out to fit a tall
column at all.

**Reproduction.**
1. Arithmetic, current code: `columnHiveLayout` (`graph.ts:74-83`) sets `x = side * 476 - size.width / 2`;
   a superagent or tool card (420 wide, `tokens.ts:174,176`) has its inner edge at `|x| = 476 - 210 = 266`.
   The chat panel is 400 wide, fixed and centred (`ChatCore.tsx:178-181`), so its half-width in world units is
   `200 / zoom`; overlap when `200/z > 266` ⇒ `z < 0.752`.
2. The fit is pinned: `Canvas.tsx:365` `fitView({ padding: chatVisible ? 0.08 : 0.18, duration: 0, minZoom: 0.75, maxZoom: 0.75 })`
   and the replay fit is pinned identically at `Canvas.tsx:688`
   (`fitViewOptions={{ padding: 0.08, minZoom: 0.75, maxZoom: 0.75 }}`), while the canvas itself allows
   `minZoom 0.1 / maxZoom 3` (`Canvas.tsx:596-597`). A `fitView` whose min equals max cannot fit; it only centres.
3. The dagre branch knows about exactly this hazard and reserves for it — `graph.ts:150-157`
   (*"The chat core is a fixed screen overlay … the reserve is … deliberately larger than the chat's world footprint
   at a typical 0.6x fit zoom"*, `halfW: 360`) — while the hive branch (`graph.ts:43-103`) applies no reserve at all.
4. Live corroboration of the pinned zoom: the canvas reported `React Flow 57%` under the *older* bundle whose clamp
   was `minZoom 0.5 / maxZoom 1`; the current source pins 0.75, i.e. a string of fits that can no longer fit.

**Proven cause.** `graph.ts:43-103` (no chat reserve in the hive layout) and `Canvas.tsx:365,688` (fit clamped to a
single zoom). *Superseded:* the radial predecessor had the same missing reserve plus a `Math.max(0.52, …)` sector
floor that guaranteed sibling overlap for ≥2 children of one parent; it was replaced at 23:11:44 and is not a
finding against the current file.

**Minimal fix.** One function: after `columnHiveLayout` produces positions, push any card whose rect intersects the
chat footprint (`|centreX| < 200/zoom + w/2`, `|centreY| < 240/zoom + h/2`) outward to the first non-intersecting
column, reusing the dagre branch's existing push loop (`graph.ts:154-207`) instead of keeping two implementations.
Separately, drop `minZoom/maxZoom` from the two `fitView` calls (or set `minZoom: 0.1`) so the fit can actually fit.

**Regression test.** `web/tests/graph.layout.test.ts` — `hive_layout_clears_the_chat_footprint_at_fit_zoom`:
for a 3-level fixture, assert no card rect intersects `|x| ≤ 200/0.75, |y| ≤ 240/0.75` for the chat panel, and
`fit_view_can_zoom_below_0_75` asserting the exported fit options' `minZoom < maxZoom`. RED today on both.

**Residual risk.** A reserve at a fixed 0.75 zoom is wrong at other zooms (the chat is screen-fixed, the reserve is
world-fixed). The correct contract is a reserve computed at the *current* zoom; that needs a viewport subscription
in the layout and is out of scope for a minimal repair — state it, do not hide it.

---

#### D10 — SuperAgent ownership is not represented: the ownership layer is depth-1 only and no card knows what it owns

**Falsifiable sentence.** The canvas draws an ownership line only from the hive core to the hyperagent's **direct**
children, and `SuperAgentNode` receives no descendant data, so no surface can state which agents a given SuperAgent
owns.

**Reproduction.**
1. `Canvas.tsx:312-351` — `ownerLinks` iterates `visibleTasks` and `continue`s on
   `task.parentId !== root.id` (`Canvas.tsx:324`), i.e. depth 1 only; a worker under a SuperAgent gets no ownership
   line, only the ordinary `bee` edge from `buildEdges` (`graph.ts:236-257`). The 23:25:20 edit duplicated the same
   depth-1 logic for replay (`Canvas.tsx:456-494`, selected at `496`) instead of generalising it, so the limitation is
   now implemented twice.
2. `nodes.tsx:141-173` — `SuperAgentNode` renders `data.task` only: a static caption (`STRINGS.node.roleCaption`,
   `strings.ts:281`) and a `WORKERS / <percent>` footer whose percent is `data.task.progress`, which is **always**
   `null` on the wire (`event.rs:248-271` has no progress field; `store.ts:1486-1489` keeps it `null`), so the
   footer always renders `—` (`nodes.tsx:167`, `nodes.tsx:183`).
3. The aggregate that would fix it already exists and is unused: `graph.ts:316-329` `taskCounts`.
4. Consequence on the live surface: the HUD shows `0 SUPERAGENTS · 3 WORKERS` while the tree has a root + 3 workers,
   because `deriveHud` counts `kind === 'superagent'` and `kind === 'worker'` and the root is neither
   (`metrics.ts:41-42`).

**Proven cause.** `Canvas.tsx:316-329` (root-only ownership; duplicated at `456-494`) + `nodes.tsx:141-173`
(no owned-child input) + `event.rs:248-271` (no progress/role on the wire).

**Minimal fix.** Extend the node data with `ownedCount` computed once from `tasks` in `buildNodes`
(`graph.ts:212-234`) and draw ownership lines for **every** parent→child pair whose parent kind is `superagent` or
`hyperagent` (one predicate change in `Canvas.tsx:324`, duplicated logic at `468` deleted). No new components, no
decoration.

**Regression test.** `web/tests/graph.layout.test.ts` — `superagent_ownership_is_not_depth_one_only`: fixture
root→super→worker, assert `ownerLinks`/edge derivation yields an ownership entry for the superagent→worker pair.
RED today (only root→super exists). Second: `buildNodes` data carries `ownedCount === 1` for the superagent.

**Residual risk.** Drawing an ownership line *and* a bee edge for the same pair doubles the ink; the fix must keep
one of them per pair (recommended: ownership line replaces the bee edge for superagent parents).

---

### P2 (appendix — recorded, not in the top 10)

| ID | Defect | Cause (file:line) | Minimal fix | Test |
|---|---|---|---|---|
| P2-1 | `buildEdges` emits `source: HIVE_ANCHOR_ID` for root children, but `buildNodes` never yields that node; only the two `Canvas` call sites compensate, so the `buildNodes`+`buildEdges` pair is not a closed contract and the replay test's "every edge endpoint exists" invariant is false for any hyperagent-rooted fixture. (Concurrent edit at 23:25:20 added the anchor to the replay path — `Canvas.tsx:434-454` — which removes the production symptom but leaves the contract split and duplicated.) | `graph.ts:236-257` (emits the anchor source) vs `graph.ts:212-234` (never emits it); compensation in `Canvas.tsx:223-245` and now `434-454`; the gate `Canvas.tsx:227-229` (live) and `436-439` (replay) still silently omits the anchor when the root is filtered out of `visibleTasks` | Add the anchor **inside** `buildNodes` when a hive root is present (one owner, both paths, one gate); delete both `Canvas` copies | `graph.replay.test.ts`: assert every edge endpoint exists in the node set **for a hyperagent-rooted fixture** (the existing test only covers a worker-only fixture); a second assertion that `buildNodes` includes `HIVE_ANCHOR_ID` iff a `hyperagent` task exists |
| P2-2 | Drag reparent can create a hierarchy shape the spawn policy forbids: `findDropTarget` accepts `worker` as a parent and any non-hyperagent as a source, and the server's rewire never re-checks tiers (only `spawn_sub_hive` calls `validate_spawn_role`) | `Canvas.tsx:156-168`, `Canvas.tsx:253` vs `execution.rs:113-142` and `ws.rs:650-736` | Call `validate_spawn_role(task_agent_tier(new_parent), &task.role)` before `graph.rewire`; refuse `E_TOOL_NOT_ALLOWED` | `graph_authority.rs`: `rewire_of_super_under_worker_is_refused` |
| P2-3 | The client contract declares `label`, `progress`, `error_code` on `NODE_*`; no producer exists, so three frontend branches are permanently dead | `web/src/api/contract.ts:237-243` + `:710-733` vs `event.rs:248-271` | Delete the dead fields from the decoder (the fail-closed path already handles absence) or add them to `NodeEvent` — pick one and make the schemas agree | `web/tests/contract.test.ts`: a NODE_SPAWN payload without `label` keeps `null`; add a golden-schema assertion that the decoder and `event.rs` field sets are identical |
| P2-4 | `NODE_DRAG` is in the frozen vocabulary and handled by the client, but nothing ever publishes it and no command carries a drag; the "drag" half of the rewire gesture is client-local | `event.rs:38-39`; `ws.rs:969` is the only occurrence (a match arm, no producer) | Remove the handling or document it as reserved; do not let the confirm dialog imply the drag reached the server | grep-level guard test in `contract.test.ts` (`NODE_DRAG` must be listed in the "no producer" allowlist) |
| P2-5 | The HUD total can exceed the sum of the two counts it displays: the hyperagent root is counted by neither `superagents` nor `workers` | `metrics.ts:41-42` | Count `hyperagent` in the HUD's own hyper slot or state the root separately | `web/tests/metrics.test.ts`: 1 hyperagent + 3 workers ⇒ displayed counts sum to the task count |

---

## 3. Causal chain (why one fix does not fix the symptom)

```
commit_run_snapshot ──► SnapshotEnvelope { GraphSnapshot: task_id,parent,state,rev }   (kind/role/label absent)
                                    │
             ┌──────────────────────┴────────────────────────┐
   HTTP /state: enriches via task rows              WS HISTORY_SNAPSHOT: verbatim
   (kind=hyperagent, role=planner)  ✔               (kind absent)  ✘ ──► buildReplayTasks → kind='worker'
             │                                                        └──► layoutTasks: no hyperagent root
             │                                                              └──► dagre (flat) + worker cards
             │
   NODE_* frames: {task_id,state,rev?,kind?,parent?}  ← no role/label/progress
             └──► store rebuilds TaskRecord without `role` ──► role lost on every live frame
             └──► store ignores envelope.run_id ──► foreign nodes enter the current graph
                                                        └──► parent absent in map
                                                              ├──► buildEdges drops the edge (detached)
                                                              └──► layoutTasks: no position → (0,0) under chat
```

Two independent roots feed every reported canvas symptom:
**(R1)** the durable graph format is topology-only and only one of two read paths re-hydrates identity from task rows
(D1, D6, D7, D10's data half);
**(R2)** the client side has no total function from "task map" to "position + edge + label" — unreachable nodes,
absent anchors and missing parents all fall through to `0,0` / dropped edge / `worker`
(D8, P2-1, D1's rendering half).

---

## 4. Evidence log (verbatim, for re-checking)

1. `GET /health` → `status degraded`, `version 1.2.0-dev`, `schema_version 10`, `graph_nodes 35`,
   `deepseek_api_key ok:false code:E_KEY_MISSING`.
2. `GET /api/history/runs` → 5 runs, all `completed`/`failed`; newest `01a1b931-941a-…` revisions `0..7`, latest `7`,
   run_status `completed`.
3. `GET …/state?revision=7` → `nodes=4` (`kind:"hyperagent" role:"planner" label:"HyperAgent 3d23430f"` +
   3 × `worker`), `graph.revision 89`.
4. On-disk snapshot listing: newest run's 8 revisions each contain `nodes=35 edges=26` (pre-`678748e` capture),
   which is why `http.rs:678-691` filters and why D6 matters.
5. Browser DOM (older bundle, refs. §0.2): boot `rfNodes=4` (`3 × node-worker + 1 × hiveanchor`), chip `4/4`,
   `[data-testid=owner-links]` present, minimap nodes `0`; after `Hide finished`: `rfNodes=0`, chip `0/4`,
   no empty-state copy.
6. WS round-trips from a real browser origin: `SERVER_HELLO` received; `HISTORY_SNAPSHOT` node fields
   `["task_id","parent_task_id","state","rev"]`, `node_count 35`, `roots 9`;
   `REWIRE_TASK` (stale rev) → `ERROR E_RESYNC_REQUIRED`; `SPAWN_SUB_HIVE` on a terminal run →
   `ERROR E_RUN_TERMINAL`; `tokenInDom: false`; `localStorage` keys `["hivemind.project-id"]`.

---

## 5. Hypotheses considered and killed (forge §3)

| Hypothesis | Cheapest decisive probe | Verdict |
|---|---|---|
| The canvas is empty because the store never loads tasks | `/api/history/runs` + `/state` returned 4 tasks; DOM rendered 4 nodes | **REFUTED** — data arrives |
| The ownership lines are missing because the SVG is hidden behind the pane | DOM query counted the `owner-links` svg element (1); the per-line child count was **not** measured (probe measured the wrong node) | **REFUTED for presence / UNVERIFIED for completeness** — the depth-1 predicate at `Canvas.tsx:324` is the decisive evidence, not the DOM count |
| The radial layout was the only problem | `graph.ts` rewritten to `columnHiveLayout` at 23:11:44; the same absent chat reserve remains | **REFUTED (superseded)** — defect class survives, arithmetic changed |
| Rewire is refused because the graph rejects the cycle | `graph.rs:161-170` cycle check is correct; a legitimate rewire of a chain member succeeds in `graph_authority.rs` tests | **REFUTED** — the refusal is the client's run gate (D3) |
| The rewire is not durable because the task row is updated but the snapshot isn't | No SQL statement in the repo updates `tasks.parent_task_id` (only `graph.rs:195` writes a parent); durability depends entirely on the *next* `commit_run_snapshot` (`execution.rs:1312,1403,1418,1480,1537,2067,2793`) | **CONFIRMED-as-open**: reachable, not the top failure; recorded under D3's residual risk |
| Memory scope is per task | `store.agent_memory(&task.role, 8)` | **REFUTED** — role-scoped, shared by all tasks of that role (D4) |

---

## 6. Receipt

### Done

| Item | Evidence |
|---|---|
| Read every named module (Canvas/graph/nodes/store/http/snapshot/execution/recovery, plus graph.rs, context.rs, event.rs, metrics.ts, ControlBar, AgentsPanel) | hashes in §0.1; line citations throughout |
| Inspected the live API and the live UI | §4 items 1-6, quoted verbatim |
| Produced 10 ranked P0/P1 defects, each with falsifiable sentence, reproduction, cause file:line, minimal fix, regression test, residual risk | §2 |
| Recorded 5 P2 defects in a condensed table (cause, minimal fix, test) | §2 appendix |
| Kept `git status` clean for product files | No product file in this repo was written by this seat. `git status --porcelain` shows the concurrent seats' work (` M web/src/flow/Canvas.tsx`, ` M web/src/flow/graph.ts`, ` M tools/desktop-launcher/HiveMind.exe`, `?? .claude/ AGENTS.md CLAUDE.md docs/trace/orca-wave/`) — this seat's only write is `docs/trace/orca-wave/03-forge.md` |
| Swept this report for stub markers (§23) | `Select-String` over the report for `TODO|FIXME|XXX|HACK|placeholder|NotImplemented|not implemented|mock|stub|dummy|lorem|simplified for|for illustration` → see §6.1 |

### Not done

| Item | Reason |
|---|---|
| No RED leg was executed (no failing test run, no planted defect) | This seat is audit-only: "No edits/restarts/providers", only `03-forge.md` may be written. Every regression test above is specified for the implementing seat to run RED→GREEN. |
| No product fix applied | Same constraint. |
| No restart of the (now exited) server | Same constraint; the live evidence was captured before exit. |
| No keyed/provider run | Key deliberately absent (`E_KEY_MISSING`); no provider work in scope. |

### Unverified

| Claim | What would prove it |
|---|---|
| D1/D6 fix produces a kind-carrying `HISTORY_SNAPSHOT` | Run the specified `snapshot_history.rs` test RED then GREEN |
| D2's empty canvas on a **current** bundle (source default `true`) | Build and serve the working tree, load a completed run, assert `.react-flow__node` count = 0 before any click |
| D3's `E_RUN_NOT_FOUND` toast on Connect for a terminal run | Browser run of the current bundle: click Connect, read `ToastRegion` text |
| D5's cross-run/flattened recovery snapshot | `recovery_gates.rs` fixture with two runs on disk and an interrupted run |
| D7's cross-run frame leak | `store.test.ts` RED run with a foreign `envelope.run_id` |
| D8/D9/D10 arithmetic against a **rendered** current bundle | Build current `web/`, then measure card rects vs the chat rect at 0.75 and at 0.5 |
| The live bundle equals the working tree | It does **not** (§0.2); the rebuilt artifact was never produced by this seat |

### 6.1 Stub sweep (§23)

```
Select-String -Path "docs\trace\orca-wave\03-forge.md" `
  -Pattern 'TODO|FIXME|XXX|HACK|placeholder|NotImplemented|not implemented|mock|stub|dummy|lorem|simplified for|for illustration'
```

Result: **3 hits, all self-references** — the sweep row in the Done table (contains the pattern), this heading
("Stub sweep"), and the pattern line itself. Classification: **permitted** (documentation self-reference, not code).
Product files were not modified by this seat, so no product file is in sweep scope: **zero product-code hits**.

---

## 7. One-line answers for the integrator

- **Flattened/detached:** the durable graph format has no identity (kind/role/label) and the WS replay path does not
  enrich it; the client then has no total placement function (unreachable nodes → `0,0`, missing parents → no edge).
- **SuperAgent ownership:** ownership is drawn depth-1 only and no card receives owned-child data; the wire cannot
  carry role or progress.
- **Clipping/overlap:** the hive layout has no chat-core reserve (the dagre fallback does), and both `fitView` calls
  are pinned to a single 0.75 zoom so the "fit" cannot fit.
- **Drag reparent:** it is a client-local gesture whose position is persisted before the dialog; the confirm is
  refused with `E_RUN_NOT_FOUND` on every terminal run, and the server would accept a policy-invalid rewire anyway.
- **Scope:** a rewire swaps facts but keeps the task's own tool ids/schema/tier and its role-keyed memory, and clears
  only the immediate parent's facts.
- **Snapshot/recovery:** recovery restores one image globally, writes it with the unscoped `capture` for every changed
  run, and re-adds parentless tasks at the top level; the WS replay read is unscoped while the HTTP read is scoped.
- **All-agent cards absent:** `hideFinished` defaults to `true` and the terminal reconciler makes every task terminal,
  so a finished run paints zero cards — with no message while the chat is visible.
