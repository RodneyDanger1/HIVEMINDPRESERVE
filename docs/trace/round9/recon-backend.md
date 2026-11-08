# Round 9 — Backend Recon (AUDIT-ONLY, no fixes applied)

Seat: recon, backend. Method: `forgewright-recon` (frame behaviour → entry point → callers → tests → data flow → ledger).

**Nothing was edited except this file.** No product code, tests, config or binary was changed. HiveMind was not
started or stopped; no provider call was made; no browser was used. The app is stopped and keyless, as handed over.

**Nothing was executed.** No `cargo test`, no `cargo check`, no server boot. Every test/code claim below is
*read from source*, not observed. Lines are from the working tree **including uncommitted changes** (see §0).

Status vocabulary used throughout:
- **VERIFIED** = read in source at the cited line.
- **PINNED** = VERIFIED *and* an existing test asserts the current behaviour.
- **UNVERIFIED** = believed but not proven by source or execution (listed in §6).

---

## 0. Working-tree state that the recon must account for

`git log --oneline -5` (read, not modified):

```
8b9ccc8 feat: subagent cards and live-snapshot preservation
a536417 feat: subagent card kind and offline recovery/reconnect fixes
c790aac fix: recovery snapshot scope, session re-bootstrap, and UI truth
4e02e77 fix: fail-closed replay, keyless submit refusal, and UI truth defects
5c9546b fix: scope replay, guard keyless submit, and repair Orca UI defects
```

`git status --short` shows three modified paths plus untracked files. One is **product source with no commit**:

- `backend/crates/hiverouter-core/src/http.rs` — adds `GET /api/agents/runs/{run_id}/roster` (`http.rs:522`,
  `http.rs:942-983`) and a **mangled one-line function signature** at `http.rs:985`:
  `async fn agent_memory_list(...) -> Response {    let mut profiles: Vec<...> = Vec::new();`
  (compiles, but the newline/indent was destroyed — evidence of an in-flight paste, not a reviewed edit).
- `backend/crates/hiverouter-core/tests/snapshot_history.rs` — adds the roster assertions (`:211-223`).
- `tools/desktop-launcher/HiveMind.exe` — binary diff (not inspected, out of scope).

Consequence: the roster endpoint is **new, uncommitted, and covered only by one HTTP test**; it has no consumer in
`web/src` (grep: no `/roster` in `web/`). All roster findings below describe working-tree behaviour.

---

## 1. Recon ledger

### 1.1 LOG-01 hydration data flow (backend half)

1. **Target behaviour** — After a reconnect/resync, the server must let the client rebuild the committed state
   (transcript + run status + run-scoped graph) from HTTP reads, and must resume the live stream exactly once,
   with a watermark the client can trust. Backend owners: `send_hello_and_replay`, the history read endpoints,
   `history_graph_value`.
2. **Entry point** — `ws.rs:495-565` `send_hello_and_replay` (invoked at `ws.rs:416` via
   `send_hello_replay_and_approvals`, and again per hello control frame at `ws.rs:627-631`);
   `http.rs:666-694` `history_state`; `http.rs:772-810` `history_messages`; `http.rs:640-664` `history_revisions`.
3. **Callers** — `ws.rs:416`, `ws.rs:629` (hello pull), `ws.rs:842-858` (`HISTORY_REVISION_SET` → `state_at` →
   `history_graph_value`). HTTP routes mounted at `http.rs:517-521`. Web consumers (read-only evidence of the
   flow): `web/src/state/store.ts:434-453` (paths), `:1060-1091`, `:1305-1318`, `:1362-1391`.
4. **Tests + command** — `backend/crates/hiverouter-core/tests/ws_transport.rs::hello_replay_and_gap_resync`
   (`:108-146`), `invalid_json_frame_keeps_connection_usable` (`:149-171`);
   `tests/snapshot_history.rs::history_http_api` (`:151-224`);
   `tests/http_boot.rs::history_messages_route_returns_persisted_transcript` (`:501-546`).
   Command: `cd backend; cargo test --workspace --locked` (workspace root; evidence convention in
   `docs/evidence/round7-backend-tests.log:2`), single test: `cargo test -p hiverouter-core --test ws_transport hello_replay_and_gap_resync`.
5. **Data flow** —
   *Pass:* client sends `{"type":"hello","last_seq":n}` → `wait_for_hello` (`ws.rs:469-493`) →
   `bus.replay_after(n)` (`bus.rs:172-206`) → either `Current`/`Fresh`/`Events`/`Gap`
   (`bus.rs:33-47`) → `SERVER_HELLO{last_seq: current_seq}` then buffered envelopes (`ws.rs:523-538`) →
   client applies events (`store.ts:1331-1334`) → HTTP `GET /api/history/runs/{id}/state?revision=N`
   → `history.state_at` (`history.rs:109-115`) → `history_graph_value` filters by `tasks_for_run` and adds
   `role/kind/label` (`http.rs:701-741`).
   *Failure path:* reconnect after >256 events → `Replay::Gap` (`bus.rs:194-199`) → `RESYNC_REQUIRED`
   **then** `SERVER_HELLO` (`ws.rs:539-562`) → the client must full-resnapshot because no replay is offered.

   **Defect (VERIFIED, PINNED by test):** `SERVER_HELLO.last_seq` is set to `bus.current_seq()` — the *server's*
   latest — and is sent **before** the replayed events, whose `seq` are *lower* than that value
   (`ws.rs:500`, `ws.rs:523-538`). The contract defines the field as *"Last server sequence number the client had
   seen"* (`hiverouter-contract/src/event.rs:178-179`). The test pins the current framing:
   `assert_eq!(hello["payload"]["last_seq"], 3)` then `replay_first["seq"] == 2`
   (`ws_transport.rs:127-133`). The web store adopts hello's value as its applied cursor
   (`web/src/state/store.ts:1365`), so the watermark jumps ahead of the replay batch it is still receiving.
   Safe fix candidate: echo the *reported* `last_seq` in `SERVER_HELLO` (or emit the replay batch before the
   hello) and keep the client cursor monotonic on applied events only.

### 1.2 Stale CAS / rewire

1. **Target behaviour** — `REWIRE_TASK` must be an optimistic-CAS mutation (`expected_rev`) that moves a task to
   a new parent atomically, refuses cycles, refuses a concurrent rewire, switches the task's scoped context, and
   leaves **one** authoritative parent everywhere the system can read it.
2. **Entry point** — `ws.rs:650-736` (`Command::RewireTask`); authority `graph.rs:141-201` `GraphAuthority::rewire`.
3. **Callers** — `ws.rs:674-676` (only production caller); `graph.rs:114-124` `begin_rewire` is **test-only**
   (`tests/graph_authority.rs:65-75`). CAS sources: `graph.rs:150-155` (node rev), `store.rs:297-354`
   (`UPDATE tasks ... WHERE state NOT IN (terminal)`, no `expected_rev`).
4. **Tests + command** — `tests/graph_authority.rs::stale_expected_revision_requires_resync` (`:48-62`),
   `rewire_in_flight_is_refused_then_cleared` (`:65-75`), `rewire_to_self_is_a_cycle` (`:39-45`);
   `tests/ws_transport.rs::rewire_ack_and_node_event_fan_out_to_both_clients` (`:174-235`),
   `rewire_cycle_is_refused_over_ws_and_graph_is_unchanged` (`:237-277`),
   `thousand_rewires_without_a_dropped_ack` (`:279-330`); `tests/context_switch.rs::rewire_context_switch`.
   Command: `cargo test -p hiverouter-core --test ws_transport --test graph_authority --test context_switch`.
5. **Data flow** —
   *Pass:* read `previous_parent` from graph (`ws.rs:654`) → seed context if missing (`ws.rs:666-673`,
   `ws.rs:304-326`) → `graph.rewire(task, new_parent, expected_rev)` (`ws.rs:674-676`) → read node under one lock
   (`ws.rs:682-692`) → `switch_on_rewire` clears old-parent facts / injects new (`ws.rs:695-721`,
   `context.rs:138-186`) → `ACK` then `NODE_REWIRE` (`ws.rs:722-731`).
   *Failure path:* stale rev → `E_RESYNC_REQUIRED` (`graph.rs:152-154`); cycle → `E_CYCLE` with no mutation
   (`graph.rs:160-170`); **context switch fails after the graph already moved** → `ERROR` is published but the
   graph mutation is *not* rolled back (`ws.rs:707-719`, comment admits "graph mutation is already committed").

   **Defect (VERIFIED):** a successful rewire writes **only** the graph node (`graph.rs:194-197`) and the
   in-memory context. There is **no** `UPDATE tasks SET parent_task_id` anywhere in the crate
   (grep over `backend/crates/hiverouter-core/src`: the only `UPDATE tasks` is `store.rs:300`, state only).
   Everything that reads the *row* keeps the old parent:
   `agent_roster` (`http.rs:961`), the A2A sibling gate (`execution.rs:2560-2561` via `store.get_task`),
   fan-out cap counting (`execution.rs:1075-1080`), the aggregator child selection (`execution.rs:1457-1458`),
   and recovery's `settle_graph_node` re-add (`recovery.rs:403`). Two authorities therefore disagree after every
   rewire and the disagreement survives restart (the snapshot keeps the new parent, the row the old one).
   **CAS asymmetry (VERIFIED):** the graph refuses a stale rev; `store.update_task_state` does not take a rev at
   all (`store.rs:297-334`) — two concurrent state writers both "succeed" (last-write-wins) though each bumps
   `rev`. Nodes also carry *two* revision counters that drift: `tasks.rev` (DB, `store.rs:300`) vs
   graph `node.rev` (`graph.rs:211`), incremented in different places (`execution.rs:931-932`) and desynchronised
   by `reconcile_terminal_tasks` (`execution.rs:1242-1245`) and `repair_graph_state` (`recovery.rs:389-390`).
   `agent_roster` reports the DB rev (`http.rs:967`) while `/history/.../state` and `NODE_STATE` report the graph
   rev (`http.rs:726`, `execution.rs:754`) — two different `rev` values for one node.

### 1.3 Terminal RUN_STATUS ordering

1. **Target behaviour** — A run's terminal `RUN_STATUS` must be the last frame that claims the run is over, and
   must not be observable before the terminal task states, `REPLY_DONE`, and the terminal history revision are
   durable. Otherwise a client that reacts to the terminal frame reads a pre-terminal world.
2. **Entry point** — `execution.rs:1255-1317` `dispatch` (terminal reconciliation + terminal write + snapshot);
   `execution.rs:2335-2398` `finalize_approval_timeout` (second terminal publisher);
   `run_state.rs:204-250` (`Effect` descriptors that name the ordering contract).
3. **Callers** — `dispatch` is spawned from `submit_prompt` (`execution.rs:1050-1052`);
   `finalize_approval_timeout` from `finish_timeout_wait` (`:2284`) and `terminal_approval_expired` (`:2321`).
4. **Tests + command** — `tests/run_lifecycle.rs::planned_parent_chain_is_persisted_not_flattened_to_root`
   (`:408-461`, waits for the first non-`queued/planning/running` RUN_STATUS),
   `streamed_deltas_are_published_before_done_and_not_duplicated`;
   `tests/suspension.rs::timeout_fails_run_with_e_approval_timeout_and_no_side_effect`,
   `cancel_while_awaiting_cancels_run_and_consumes_approval`;
   `tests/tool_loop_matrix.rs:196-216` (drives to the terminal frame).
   Command: `cargo test -p hiverouter-core --test run_lifecycle --test suspension --test tool_loop_matrix`.
5. **Data flow** —
   *Pass:* `dispatch_inner` returns outcome → `reconcile_terminal_tasks` publishes one `NODE_STATE` per task
   (`execution.rs:1276`, `:1218-1253`) → under `with_lifecycle_lock` the terminal status is persisted and
   `RUN_STATUS` published once (`:1282-1303`) → terminal snapshot committed (`:1312`).
   *Failure path (timeout):* `finalize_approval_timeout` writes one timeout `TOOL_RESULT` (`:2344`), one task
   terminal `NODE_STATE` (`:2351-2364`), then the terminal `RUN_STATUS failed E_APPROVAL_TIMEOUT` (`:2367-2396`).

   **Defect (VERIFIED):** the terminal frame is published **before** the terminal snapshot is written, in both
   publishers — `execution.rs:1293-1301` then `:1312`, and `:2386-2394` with *no* snapshot write at all (the
   snapshot only appears later when `dispatch` resumes). The timeout case has the widest window: the terminal
   `RUN_STATUS` is emitted inside `execute_tool_turn`, and `dispatch` cannot commit the terminal revision until
   the error unwinds through `execute_task_inner` → `dispatch_inner` → `dispatch`. A client that reacts to
   `RUN_STATUS failed (E_APPROVAL_TIMEOUT)` by fetching `/api/history/runs/{id}/revisions` can legitimately see
   `latest_revision` still describing `awaiting_approval` (or `E_HISTORY_GAP` at the expected revision), i.e. the
   terminal frame is observable before the terminal truth is durable. Safe fix candidate: commit the terminal
   snapshot before publishing the terminal `RUN_STATUS` (and let the timeout path commit it itself).
   Secondary (VERIFIED): `reconcile_terminal_tasks` can relax a task to `failed` while the run is `cancelled`
   etc. by design, but it publishes `NODE_STATE` frames *after* `RUN_STATUS` for no run — ordering itself is fine,
   only the snapshot is late.

### 1.4 History truncation

1. **Target behaviour** — `GET /api/history/runs/{run_id}/messages` must return a usable transcript when a run
   exceeds the cap, and the `truncated` flag must say *which* end was dropped.
2. **Entry point** — `http.rs:772-810` `history_messages`; ordering owner `store.rs:435-468` `RunStore::messages`.
3. **Callers** — route `http.rs:520`; web `web/src/state/store.ts:448-453`, `:1073-1080`.
4. **Tests + command** — `tests/http_boot.rs::history_messages_route_returns_persisted_transcript` (`:501-546`)
   asserts only the under-cap case (`truncated == false`, 2 messages). **No test covers >1000 messages** (grep
   `MAX_MESSAGES` / `truncated` finds only `http_boot.rs:534`).
   Command: `cargo test -p hiverouter-core --test http_boot history_messages_route_returns_persisted_transcript`.
5. **Data flow** —
   *Pass:* `messages()` → SQL `ORDER BY created_at ASC, rowid ASC` (`store.rs:439-441`) → map to JSON
   (`http.rs:787-797`) → `{run_id, messages, truncated:false}`.
   *Failure path:* >1000 rows → `truncated = messages.len() > 1000` and `visible = messages.into_iter().take(1000)`
   (`http.rs:782-786`).

   **Defect (VERIFIED):** because the ordering is ascending, `take(1000)` keeps the **oldest** 1000 rows and
   silently drops the **newest** ones, while reporting `truncated: true`. On hydration the operator loses the most
   recent assistant/tool replies (the part of the transcript they are actually looking at) and the flag does not
   distinguish head/tail truncation. Safe fix candidate: keep the newest N (`last(N)` / `ORDER BY ... DESC LIMIT N`
   then reverse) or add a server-side `from_seq`/cursor; state the dropped range in the payload.
   Adjacent (VERIFIED, P2): `history.rs:80` swallows a corrupt snapshot read with `.ok()` and reports
   `run_status: null` for that revision instead of surfacing `E_SNAPSHOT_CORRUPT`.

### 1.5 Per-hive / branch memory

1. **Target behaviour** — Durable agent memory must be scoped to the hive/branch (or at least the run + task) that
   wrote it, so a child in branch A never receives branch B's facts, and a rewire to a new parent does not carry
   the old branch's memory. `context.rs:5-7` states exactly this for scoped facts.
2. **Entry point** — write `execution.rs:3027-3035` (`complete_task`) → `store.rs:612-643`
   `record_agent_memory`; read `execution.rs:880-884` (`seed_task_context`) → `store.rs:649-671` `agent_memory`.
3. **Callers** — `execution.rs:880` (seeds every created/seeded task: root `:1344`, planned nodes `:1382`,
   spawned children `:1106`, template inserts via `seed_template_contexts` `:849-854`);
   `http.rs:985-1002` (memory panel); `http.rs:1004-1020` (clear).
4. **Tests + command** — `tests/run_lifecycle.rs::durable_role_memory_is_recalled_into_worker_context`
   (`:505-544`) writes memory under `agent_key = "worker"` with a **different** `RunId`/`TaskId` (`:514-515`) and
   asserts it appears in a worker context; `store.rs:1081-1097` `agent_memory_is_durable_and_role_scoped`;
   `http_boot.rs::agent_memory_route_lists_and_clears_role_memory` (`:366`).
   Command: `cargo test -p hiverouter-core --test run_lifecycle durable_role_memory_is_recalled_into_worker_context`.
5. **Data flow** —
   *Pass:* task completes → text ≤2000 chars stored with `agent_key = task.role` and `key = "task:<task id>"`
   (`execution.rs:3029-3035`) → a future task of the same *role* reads the newest 8 rows
   (`store.rs:656-659`) → inserted into `ScopedContext.facts` as `memory:<key>` (`execution.rs:880-883`) →
   rendered into the prompt by `prompt_with_context` (`execution.rs:53-58`, `:2766`).
   *Failure path:* none — the read cannot fail the task (`if let Ok(memories)` at `execution.rs:880` silently
   drops a DB error, another fail-open).

   **Defect (VERIFIED, PINNED as intended by the test):** the scope is the **role string**, not the hive, branch,
   run or task. Consequences:
   - `agent_memory` filters on `agent_key` only (`store.rs:657`) and never on `run_id`/`task_id`, so memories from
     **any other run** (including a previous, unrelated run) are injected into a new run's prompt. The test above
     codifies exactly this cross-run behaviour.
   - The upsert rewrites provenance: `ON CONFLICT(agent_key, key) DO UPDATE SET ... run_id = excluded.run_id,
     task_id = excluded.task_id` (`store.rs:625-629`) — the stored `run_id`/`task_id` are not the row's origin.
   - **Branch leak:** two workers in different sub-hives share the role `worker`, so branch B's child inherits
     branch A's `memory:task:<id>` facts. The rewire escape hatch does not help: `switch_context` only clears keys
     with the `parent:<id>:` prefix (`context.rs:100-107`, `:148-159`), and memory facts are stored as
     `memory:task:<id>`, so **rewiring a task to a new parent leaves the old branch's memory facts in place** —
     contradicting the module's stated guarantee (`context.rs:5-7`).
   - The HTTP memory panel iterates **persona ids** (`http.rs:986-987`: `planner`, `content-writer`, `superagent`,
     `subagent`, `data-analyst`, `code-reviewer`, `chatbot` — `assets/prompts/personas.json:1-67`) while writes and
     prompt reads use the **task role** from the plan (`worker`/`super`/`agent`/…). Only coincident ids
     (`planner`, `subagent`) can ever appear in the panel, so the panel is mostly blind to stored memory.
   Safe fix candidate: scope the key (`run:<id>`/`hive:<root task>`/`branch:<parent>`) or add
   `run_id`/`parent_task_id` columns to the read filter; clear `memory:` facts on rewire; make the panel and the
   seeding path share one identity function.

### 1.6 HyperAgent aggregate access

1. **Target behaviour** — The run's HyperAgent (planner root) is the orchestrator that receives the aggregate of
   its children (the HMAS "HyperAgent reads the hive's results" requirement) and can address them.
2. **Entry point** — root creation `execution.rs:1331-1345`; execution split `planner.rs:143-193`
   (`leaf_order`, `aggregation_order`); aggregate prompt `execution.rs:1441-1495`; tier allowlist
   `hiverouter-tools/src/tool.rs:391-396`; kind mapping `execution.rs:99-106`.
3. **Callers** — `dispatch_inner` `:1406-1436` (leaves), `:1441-1495` (aggregators), `:1499`
   (`dispatch_spawned_tasks`); `execute_spawn_agent` returns a child's text inline `execution.rs:2691-2702`.
4. **Tests + command** — `tests/run_lifecycle.rs::planned_parent_chain_is_persisted_not_flattened_to_root`
   (`:408-461`) asserts `reply_done == 3` = "leaf + two aggregators" for a 4-level tree, and asserts one prompt
   contains `"Aggregate the completed child results"` (`:457-460`).
   Command: `cargo test -p hiverouter-core --test run_lifecycle planned_parent_chain_is_persisted_not_flattened_to_root`.
5. **Data flow** —
   *Pass:* plan nodes at depth ≥1 are inserted (`execution.rs:1356-1385`) → `leaf_order` (sinks, no dependents)
   executes first (`planner.rs:147-162`, `execution.rs:1406-1436`) → `aggregation_order` (nodes someone depends
   on, deepest first) builds `aggregate_prompt` from `store.task_result(child)` only (`execution.rs:1449-1476`)
   → each aggregator executes and is stored (`:1477-1495`).
   *Failure path:* a child with no result contributes the literal line `"Child <id> has no recorded result."`
   (`execution.rs:1470-1474`) — honest, and the intended failure shape.

   **Defect (VERIFIED):** the root task (`role: "planner"`, `depth: 0`, `state: Running`) is created
   (`execution.rs:1331-1345`) but is **never a plan node**, so it is in neither `leaf_order` nor
   `aggregation_order`; **the HyperAgent therefore never executes, never sees child results, and never produces an
   aggregation of the run.** The test's own arithmetic proves it: 3 plan nodes execute, the depth-0 root does not
   (and `reconcile_terminal_tasks` later stamps it `completed`/`failed`/`cancelled` without it having run,
   `execution.rs:1226-1253`). Even a HyperAgent that *did* run has no aggregate access: its allowlist is
   `[spawn_agent, report_result, finish]` (`tool.rs:391-396`) — no `file_read`, no child-listing tool, no
   `ask_parent`/`send_to_sibling`, and `report_result` only writes. The only way it can ever see a child result is
   the inline return of `spawn_agent` (`execution.rs:2691-2702`), which the root never calls.
   Safe fix candidate: execute the root after all aggregators with an aggregate prompt built from every executed
   task result (or make the root an implicit aggregator), and/or give HyperAgent a read-only children/results tool.

### 1.7 A2A roster / answer / topic bus / mute

1. **Target behaviour** — Agents must be able to discover peers (roster), receive an **answer** to `ask_parent`,
   publish/subscribe on a topic bus, and honour operator mute/blind settings.
2. **Entry point** — roster `http.rs:942-983` (+ route `:522`, uncommitted); messages mailbox
   `http.rs:812-840`; writes `execution.rs:2427-2497` (`ask_parent`), `:2519-2619` (`send_to_sibling`);
   storage `store.rs:526-605`.
3. **Callers** — `execute_tool_turn` `execution.rs:1703-1730`; approval-gated cross-branch path
   `execution.rs:1705-1728` → `resume_and_execute` `:2212-2215`; web `AgentMessagesPanel.tsx:35`
   (`/api/agents/runs/{id}/messages`). **No caller consumes `agent_messages_for_run` for a prompt**
   (grep: only `http.rs:822` and `store.rs` tests) — the mailbox is write-only for the engine.
4. **Tests + command** — `tests/tool_loop_engine.rs::ask_parent_writes_a_durable_message` (`:302-360`);
   `tests/suspension.rs::cross_branch_message_requires_approval_then_writes_once`;
   `tests/http_boot.rs::agent_messages_route_returns_durable_mailbox` (`:468-498`);
   `tests/snapshot_history.rs::history_http_api` (`:211-223`, new roster assertions).
   Command: `cargo test -p hiverouter-core --test tool_loop_engine --test suspension --test http_boot --test snapshot_history`.
5. **Data flow** —
   *Pass:* model calls `ask_parent{text}` → auth (`tool.rs` allowlist) → per-run message budget
   (`execution.rs:2458-2474`) → `record_agent_message` (`store.rs:526-554`) + `AgentMessage` event
   (`execution.rs:2480`) → tool result carries `message_id`/`to_task_id` (`:2481-2489`); a UI reads them via
   `GET /api/agents/runs/{id}/messages`.
   *Failure path:* no parent → `E_TASK_NOT_FOUND` (`execution.rs:2443-2449`); budget → `E_CALL_LIMIT`
   (`:2458-2465`); cross-branch sibling → `E_TOOL_NOT_ALLOWED` unless an approval bridge exists
   (`execution.rs:2560-2570`, `:1708-1728`).

   **Defects (VERIFIED):**
   - **No answer path.** `ask_parent` returns a `message_id` and the parent is never woken, never handed the text
     (nothing reads `agent_messages_for_run` into a prompt), and no tool exists for a parent to reply to a specific
     child except another `send_to_sibling` (which is a *new* message, not an answer, and needs the child's id).
     The tool name promises request/response semantics that the runtime does not implement.
   - **Roster is row-only and partly wrong.** `agent_roster` lists every `tasks` row (including the structural
     planner root and any non-agent row), reports `parent_task_id` and `rev` **from the DB row** (`http.rs:961`,
     `:967`), so after a rewire it reports the *stale* parent/rev (§1.2) while `NODE_REWIRE` and
     `/history/.../state` report the new graph values.
   - **Kind is two different vocabularies.** The roster and the history graph map `subagent` → `"subagent"`
     (`http.rs:743-751`), while `NODE_STATE`, `NODE_REWIRE` and `RUN_METRICS` map the same task through
     `task_agent_tier` → `"worker"` (`execution.rs:151-165`, `:99-106`, `:663-668`, `:730`, `:755`). The same node
     is `subagent` in HTTP and `worker` on the event stream.
   - **Topic bus: absent.** Zero matches for `topic`/`Topic` in `backend/crates/**` (grep).
   - **Mute/blind/invisible: absent.** Zero matches for `mute`/`Mute` in `backend/crates/**`; `SETTINGS_STATUS_RESULT`
     carries only provider/model/key/thinking/effort (`ws.rs:787-804`). There is no field, command, route or
     enforcement point.

### 1.8 Remote / Watchdog agents

1. **Target behaviour** — Operator requirement: distinct Remote-agent and Watchdog-agent kinds with live cards and
   a roster surface (`GET /api/agents/remote`, daemons/heartbeats).
2. **Entry point** — none. Searched `backend/crates/**` for `Remote`, `Watchdog|watchdog`, `Daemon|daemon`,
   `heartbeat`: **zero product matches** (only `health.rs` prose and `desktop-launcher` out of scope). Routes are
   the closed list at `http.rs:513-535`; node kinds are the closed set at `execution.rs:99-106` and
   `http.rs:743-751`.
3. **Callers** — none (no route, no command, no event kind).
4. **Tests + command** — none. `grep -i "remote\|watchdog" backend/crates/*/tests` → no match.
5. **Data flow** — *Pass:* n/a (absent). *Failure path:* an operator/UI asking for the surface gets the router
   fallback (`static_ui`, `http.rs:533`) → 404; an unknown role string on `SPAWN_SUB_HIVE` is refused by
   `validate_spawn_role` (`execution.rs:113-142`) unless it aliases one of the four known tiers, and then maps to
   the historic tier (`execution.rs:151-165`).

   **Classification (VERIFIED):** requirement gap, **not** a false claim — no current UI or doc in this tree shows
   Remote/Watchdog cards as live. Ranked P1 in §2 because the known-area list treats it as outstanding scope, not
   because a shipped surface lies.

### 1.9 Scheduler / scale caps

1. **Target behaviour** — Hard caps enforced *before* a spawn/step: depth 3, fan-out 8, 64 agents/run,
   40 provider calls/run (`caps.rs:8-15`), plus per-run tool-turn/call caps (`tool_loop.rs:13-21`) and the
   provider token budget. Execution is sequential inside one run, with a deferred spawn queue.
2. **Entry point** — `caps.rs:41-72` (`check_spawn`, `check_calls`); enforced at `execution.rs:1091`
   (`spawn_sub_hive`), `execution.rs:1368` (planned nodes), queue `execution.rs:1510-1560`
   (`dispatch_spawned_tasks`), counters `db.rs:657-700`.
3. **Callers** — `spawn_sub_hive` from `SPAWN_SUB_HIVE` (`ws.rs:763-772`), `spawn_agent` (`execution.rs:2665`),
   template insert (`ws.rs:815-841`); `check_calls` at `execution.rs:1323` (planning) and `:2781` (each task).
4. **Tests + command** — `tests/run_lifecycle.rs::caps_refused_by_name`;
   `tests/run_lifecycle.rs::profile_max_children_tightens_the_global_fanout_cap` (`:596-693`);
   `caps.rs:74-95` unit tests; `tests/tool_loop_engine.rs::exhausted_call_budget_fails_with_call_limit`,
   `persisted_turn_cap_fails_honestly_instead_of_fake_completion`;
   `tests/tool_loop_matrix.rs::tool_loop_matrix_turn_cap_is_named_call_refusal`.
   Command: `cargo test -p hiverouter-core --test run_lifecycle --test tool_loop_engine --test tool_loop_matrix`.
5. **Data flow** —
   *Pass:* `spawn_sub_hive` takes `task_state_lock` (`execution.rs:1064-1105`) → counts siblings/agents from rows →
   persona limits (`:1083-1090`) → `caps.check_spawn(depth, siblings, agents)` → insert row + graph node →
   seed context → publish `NODE_SPAWN` → enqueue; the dispatch loop drains the queue after the plan
   (`execution.rs:1499`, `:1510-1560`).
   *Failure path:* over-cap → named refusal `E_SPAWN_DEPTH` / `E_FANOUT_LIMIT` / `E_AGENT_LIMIT`
   (`caps.rs:50-59`), refused **before** any row exists.

   **Defect (VERIFIED):** the **40-provider-calls-per-run cap is not actually a cap.** `caps.check_calls` is only
   ever called as `check_calls(1)` (`execution.rs:1323`, `:2781`) — a constant that can never exceed 40 — and no
   per-run provider-call counter exists (grep `check_calls`: two call sites, both literal `1`). The real
   enforced limits are the *tool* caps (`MAX_TOOL_TURNS` 12, `MAX_CALLS_PER_RUN` 40 tool calls) and the token
   budget, so a run can make 1 planning + N leaf + M aggregator + K continuation provider calls well past 40. This
   contradicts the module guarantee `"hard caps (depth 3, fan-out 8, 64 agents, 40 calls) are enforced before a
   spawn/reply"` (`execution.rs:5-6`).
   Secondary (VERIFIED): the documented "40 super-steps" is also only enforced through `MAX_PLAN_NODES`
   (`planner.rs:10`, `:54-56`) and the tool counters, not through `Caps`.

### 1.10 Provider API support / refusal

1. **Target behaviour** — Every provider failure maps to one frozen `E_*` refusal, secrets never reach logs, absent
   usage never becomes zero, unsupported request shapes are refused before the network, and streaming completes
   honestly (`E_PROVIDER_INCOMPLETE`, never a fabricated success).
2. **Entry point** — `hiverouter-provider/src/client.rs:492-537` (`classify_status`), `:350-443` (`complete`),
   `:549-620` (`stream`), `error.rs:18-110` (`ProviderError`), `dto.rs:537-575` (`ChatRequest::validate`),
   `budget.rs` (token/cost budget), `provider_executor.rs:289-352` (plan/execute), `:362-429`
   (continue_with_tools), `:162-286` (stream pump).
3. **Callers** — `provider_executor.rs:297-318` (planner), `:330-352` (first leaf call), `:374-428`
   (continuation); budget `:361-372`, `:564-572`; refusal origin for the engine `execution.rs:2808-2819`.
4. **Tests + command** — `hiverouter-provider/tests/error_matrix.rs`, `refusal_matrix.rs`, `streaming.rs`,
   `tool_call_transport.rs`, `tool_continuation.rs`, `tool_conversation.rs`, `tool_assembler.rs`;
   `hiverouter-core/tests/provider_streaming.rs::length_finish_reason_is_incomplete_not_success`,
   `usage_absent_after_stream_is_incomplete_not_zero`, `malformed_sse_is_a_named_refusal`;
   `tests/provider_adapter.rs::usage_absent_is_incomplete_not_zero`.
   Command: `cargo test -p hiverouter-provider; cargo test -p hiverouter-core --test provider_streaming --test provider_adapter`.
5. **Data flow** —
   *Pass:* validated `ChatRequest` (`dto.rs:537-575`) → `stream` → `classify_status` (401/402 → `E_PROVIDER_AUTH`
   never retried; 429 → `E_LLM_RATE_LIMIT`; 5xx → retryable) → `pump_stream` accumulates text/reasoning/tool
   fragments (`client.rs:639-747`) → executor settles budget usage (`provider_executor.rs:233-243`) → finish
   reason must be `stop`/`tool_calls` (`:252-261`) → `StreamedReply`.
   *Failure path:* `finish_reason == "length"` → `E_PROVIDER_INCOMPLETE` (`:244-251`); no `[DONE]` →
   `E_PROVIDER_INCOMPLETE` (`:171-177`); assembled tool call missing id/name/args → `E_PROVIDER_INVALID_RESPONSE`
   (`tool_assembler.rs:219-231`, `:553-559`); forced tool choice + thinking → refused pre-send
   (`dto.rs:562-573`).

   **No defect found on the refusal surface (VERIFIED).** Two minor notes: (a) `continue_with_tools` always calls
   `.with_tools(tools)` and `with_tool_choice(Auto)` even when the tier list is empty
   (`provider_executor.rs:414-420`), unlike `execute` which guards on `!tools.is_empty()`
   (`:342-344`) — unreachable today because every tier has ≥3 tools (`tool.rs:391-434`), but it is an
   inconsistency the provider could reject with HTTP 400 if a tier ever gets an empty allowlist; (b) an empty
   `StreamedReply` is refused (`provider_executor.rs:271-279`) so a "silent success" cannot occur.

### 1.11 Keyless submit guard

1. **Target behaviour** — A keyless deployment must refuse `SUBMIT_PROMPT` **before** `create_run`, naming
   `E_KEY_MISSING`, instead of creating a run that dies later.
2. **Entry point** — `ws.rs:743-753` (guard); check producer `checks.rs:140-161` `provider_key_check`;
   boot wiring `main.rs:108-114`; executor fallback `http.rs:231-253`, `execution.rs:288-313`.
3. **Callers** — `handle_client_text` → `Command::SubmitPrompt` (`ws.rs:743-762`) → `engine.submit_prompt`
   (`execution.rs:1016-1054`). `submit_prompt` has exactly one production caller (grep).
4. **Tests + command** — `tests/recovery_gates.rs` pins `provider_key_check` shape/code/secret-free
   (`:63-119`) but **no test exercises the WS guard**: `tests/support/mod.rs:65-75` builds `AppState` with
   `run_checks()` only (no `provider_key_check`), so the guard's `any(...)` never matches in the harness.
   `AppState::with_services` also passes zero personas (`http.rs:432`).
   Command (guard is untested): `cargo test -p hiverouter-core --test recovery_gates`.
5. **Data flow** —
   *Pass:* boot with `HIVEROUTER_PROVIDER` unset → default `enabled = true` (`config.rs:56-68`, `:77-90`) →
   `provider_key_check(true, ApiKey::present_in_env())` pushed into `checks` (`main.rs:108-114`) → key absent →
   `Check{name:"deepseek_api_key", ok:false, code:"E_KEY_MISSING"}` (`checks.rs:154-159`) → WS submit sees the
   failed check → publishes `ERROR E_KEY_MISSING` and returns before `create_run` (`ws.rs:746-753`).
   *Failure path:* `HIVEROUTER_PROVIDER=off` (or any path that omits the check, e.g. the test harness) →
   `provider_key_check` returns `None` (`checks.rs:143-146`) → the guard's `any(...)` finds no key check → submit
   is accepted → `create_run` + `RUN_CREATED` → `UnconfiguredExecutor::plan` returns `E_KEY_MISSING`
   (`execution.rs:293-301`) → the run is created and then fails.
   Also `Config::resolve` (test helper) forces `provider.enabled = false` (`config.rs:303-314`), which is a
   second, independent reason the guard cannot fire from that constructor.

   **Defect (VERIFIED):** the guard is **conditional on the check being present**, not on the actual ability to
   call the provider. It is correct for the current keyless deployment (provider enabled, key absent) but it
   silently disappears when the provider is explicitly disabled, and it has no regression test. Safe fix
   candidate: refuse on `config.provider.enabled && engine.executor is UnconfiguredExecutor` (or expose a
   `provider_ready()` on `AppState` derived from the executor itself), and add a WS test that builds a server with
   a failed `deepseek_api_key` check.

### 1.12 Recovery scoped snapshots

1. **Target behaviour** — At boot, mark active runs `interrupted`, settle their tasks, restore the last committed
   **run-scoped** snapshot, and never let one run's nodes appear in another run's revision or in the restored
   graph image; a corrupt snapshot is reported, never silently reset.
2. **Entry point** — `recovery.rs:85-314` `recover_on_boot(_with_events)`; scoping
   `snapshot.rs:264-285` `capture_scoped`; producers `execution.rs:1189-1216` `commit_run_snapshot`,
   `recovery.rs:263-311`; restore `graph.rs:304-327`.
3. **Callers** — `main.rs:87-106` (boot, no bus), `http.rs:318-326` / `:354` / `:398`
   (`publish_recovery_catch_up`, bus exists), `recovery.rs:328-354` (`publish_recovered_node_states`).
4. **Tests + command** — `tests/kill_restore.rs::kill_restore` (`:47-117`, real `abort()` writer),
   `corrupt_snapshot_at_recovery_is_refused_not_reset`; `tests/snapshot_history.rs::history_api_returns_real_revisions`,
   `history_http_api`, `migration_roundtrip`; `tests/recovery_gates.rs`.
   **`capture_scoped` itself has no direct test** — every test snapshot uses unscoped `capture`
   (`kill_restore.rs:40`, `recovery.rs:546`, `:766`, `:898`, `snapshot_history.rs:110-113`).
   Command: `cargo test -p hiverouter-core --test kill_restore --test snapshot_history --test recovery_gates`.
5. **Data flow** —
   *Pass:* `list_runs` → non-terminal runs persisted `interrupted` (`recovery.rs:113-131`) → snapshots restored
   oldest-first so the newest image wins (`:139-177`) → settle non-terminal task rows + graph + one
   `NODE_STATE` each (`:183-229`) → settle graph-only leftovers (`:231-261`) → write one new **scoped** revision
   per changed run (`:263-311`, `capture_scoped(graph, run_id, revision, status, &allowed)`).
   *Failure path:* corrupt snapshot → recorded in `report.corrupt_snapshots`, boot continues, workspace not
   reset (`:150-173`); snapshot write failure → `report.snapshot_write_failures`, DB+graph still settled
   (`:295-310`); missing parent node → child added at top level with a warning (`:407-417`).

   **Findings (VERIFIED):** the scoping itself is correct (nodes/edges/toposort all filtered,
   `snapshot.rs:271-284`; producers pass `tasks_for_run`, `execution.rs:1193-1202`, `recovery.rs:279-286`), and
   the "one in-memory image, newest wins" limit is documented in the report type (`recovery.rs:44-50`). Two real
   gaps:
   - **Unscoped `capture` is still public and used by every test** (`snapshot.rs:291-305`), so nothing pins the
     cross-run filter; a regression to `capture` in `commit_run_snapshot` would leak other runs' nodes into a
     revision and no test would fail.
   - **Mislabeled error code:** a run that crashed while `awaiting_approval` is marked
     `interrupted` with `error_code = E_APPROVAL_TIMEOUT` (`recovery.rs:123-128`). A restart is not an approval
     timeout; the code contradicts the state (`interrupted`) and will be shown as an approval timeout in the UI.
   - Minor: restore order is by `runs.created_at` string, not by snapshot recency (`recovery.rs:139-147`), so "the
     newest loadable snapshot wins" is only true if runs are resumed in creation order.

---

## 2. P0 / P1 / P2 ranking

Definitions used here: **P0** = wrong truth reaches the operator or model, or a stated hard guarantee is
unenforced in a way that leaks/corrupts state. **P1** = a functional defect or missing committed capability with a
plausible user-visible failure and no test pinning the correct behaviour. **P2** = consistency, contract-drift,
observability or test-coverage defects.

| # | ID | Sev | Area | One-line claim | Evidence |
|---|---|---|---|---|---|
| 1 | R9-B01 | **P0** | per-hive/branch memory | Memory is keyed by role string only; it crosses runs **and** branches, is injected into prompts, and survives rewire (the `parent:` clear never touches `memory:` keys). | `execution.rs:880`, `:3027-3035`; `store.rs:612-671`; `context.rs:100-107`, `:138-186`; `ws.rs:340-357`; test pins cross-run read `run_lifecycle.rs:505-544` |
| 2 | R9-B02 | **P0** | stale CAS / rewire | `REWIRE_TASK` moves the graph + context but never the `tasks` row (no `UPDATE ... parent_task_id` exists), so roster, A2A sibling gate, fan-out counting, aggregator child selection and recovery disagree with the graph — permanently. | `ws.rs:650-736`; `graph.rs:194-197`; `store.rs:300` (state-only update); readers `http.rs:961`, `execution.rs:1075-1080`, `:1457-1458`, `:2560-2561`; `recovery.rs:403` |
| 3 | R9-B03 | **P1** | LOG-01 hydration | `SERVER_HELLO.last_seq` carries the server's *current* seq and is sent before the replay batch, so the client's watermark jumps ahead of events it has not applied; contract says the field is the client's last seen. | `ws.rs:500`, `:523-538`; `event.rs:178-179`; `bus.rs:172-206`; consumer `web/src/state/store.ts:1365`; test pins it `ws_transport.rs:127-133` |
| 4 | R9-B04 | **P1** | terminal RUN_STATUS ordering | Terminal `RUN_STATUS` is published **before** the terminal snapshot; the approval-timeout path publishes it long before `dispatch` can commit the revision, so a client reacting to it reads a pre-terminal history. | `execution.rs:1282-1312`; `:2367-2396` (no snapshot); `:1255-1276` |
| 5 | R9-B05 | **P1** | history truncation | `take(1000)` on an ASC transcript keeps the **oldest** messages and drops the newest, while `truncated:true` gives no direction; the >cap path is untested. | `http.rs:782-786`; `store.rs:439-441`; only under-cap test `http_boot.rs:534` |
| 6 | R9-B06 | **P1** | HyperAgent aggregate access | The run root (HyperAgent) is created but is in neither `leaf_order` nor `aggregation_order`, so it never executes and never aggregates; its tool allowlist also has no way to read child results. | `execution.rs:1331-1345`; `planner.rs:143-193`; `execution.rs:1406-1495`; `tool.rs:391-396` |
| 7 | R9-B07 | **P1** | A2A answer | `ask_parent` has no answer path: the parent is never woken or given the text, nothing reads `agent_messages_for_run` into a prompt, and no reply tool exists — the tool name promises request/response. | `execution.rs:2427-2497`; `store.rs:578-605` (only `http.rs:822` reads it); `tool.rs:391-434` |
| 8 | R9-B08 | **P1** | scheduler/scale caps | The documented 40-provider-calls/run cap is not enforced: `check_calls` is only called with the literal `1`, and no per-run provider-call counter exists. | `caps.rs:8-15`, `:66-71`; `execution.rs:5-6`, `:1323`, `:2781`; real caps `tool_loop.rs:13-21` |
| 9 | R9-B09 | **P1** | keyless submit guard | The guard depends on a `deepseek_api_key` check being present in `state.checks`; with the provider disabled (or in the test harness) no check exists, so submit is accepted and the run fails later. No test exercises the guard. | `ws.rs:743-753`; `checks.rs:143-146`; `main.rs:108-114`; `config.rs:79-90`, `:303-314`; harness `tests/support/mod.rs:65-75` |
| 10 | R9-B10 | **P1** | Remote / Watchdog | Requirement gap, not a false claim: zero code, route, command, event kind or test for Remote/Watchdog/daemon/heartbeat agents. | grep over `backend/crates/**`: zero product matches; route list `http.rs:513-535`; kinds `execution.rs:99-106`, `http.rs:743-751` |
| 11 | R9-B11 | **P1** | recovery scoped snapshots | A crash during `awaiting_approval` is recorded as `interrupted` **with `error_code = E_APPROVAL_TIMEOUT`** — the code contradicts the state and mislabels a restart as an approval timeout. | `recovery.rs:123-128`; `main.rs:87-96` |
| 12 | R9-B12 | **P2** | stale CAS / rewire | Node revision has two counters (DB `tasks.rev` vs graph `node.rev`) that drift; `agent_roster` reports the DB one while `NODE_STATE`/history report the graph one — two `rev` values for one node, and `REWIRE_TASK.expected_rev` compares only the graph value. | `store.rs:300`; `graph.rs:211`; `execution.rs:931-932`, `:1242-1245`; `recovery.rs:389-390`; `http.rs:726` vs `:967`; `execution.rs:754` |
| 13 | R9-B13 | **P2** | state CAS | `store.update_task_state` has no `expected_rev`: concurrent writers both succeed (last-write-wins) while the graph refuses stale revs — the two layers disagree about what "CAS" means. | `store.rs:297-354` vs `graph.rs:150-155` |
| 14 | R9-B14 | **P2** | A2A kind vocab | One task is `kind: "subagent"` over HTTP (`/roster`, `/history/.../state`) and `kind: "worker"` on `NODE_STATE`/`NODE_REWIRE`/`RUN_METRICS`. | `http.rs:743-751` vs `execution.rs:151-165`, `:99-106`, `:663-668`, `:730`, `:755` |
| 15 | R9-B15 | **P2** | A2A roster | New uncommitted endpoint lists every `tasks` row (including the structural planner root), reports DB parent/rev (stale after rewire), has no `muted`/capability/topic fields, and has no web consumer. | `http.rs:942-983`, `:522`; test `snapshot_history.rs:211-223`; grep: no `/roster` in `web/` |
| 16 | R9-B16 | **P2** | memory identity | The memory panel reads by **persona id** while the engine writes by **task role**; only coincident ids ever appear, so stored memory is mostly invisible. | `http.rs:986-987`; `execution.rs:880`, `:3030`; `assets/prompts/personas.json:1-67` |
| 17 | R9-B17 | **P2** | history truthfulness | `HistoryReadModel::revisions` swallows a corrupt snapshot read with `.ok()` and reports `run_status: null` instead of `E_SNAPSHOT_CORRUPT`. | `history.rs:80` |
| 18 | R9-B18 | **P2** | recovery scoped snapshots | `capture_scoped` is untested; every test uses unscoped `capture`, so a regression to the unscoped path would leak other runs' nodes into revisions silently. | `snapshot.rs:264-285` vs `:291-305`; tests `kill_restore.rs:40`, `recovery.rs:546`, `:766`, `:898` |
| 19 | R9-B19 | **P2** | recovery scoped snapshots | Restore order is by `runs.created_at` string, not snapshot recency, so "newest loadable snapshot wins" only holds when runs are created in resume order. | `recovery.rs:139-147`, `:44-50` |
| 20 | R9-B20 | **P2** | provider request shape | `continue_with_tools` sends `tools: []` + `tool_choice: auto` when a tier allowlist is empty (unreachable today, but unlike `execute` which guards). | `provider_executor.rs:414-420` vs `:342-344` |
| 21 | R9-B21 | **P2** | transport surface | `DIFF_PROPOSE`/`DIFF_APPLY`/`DIFF_REJECT` are in the frozen vocabulary and matrix but decode to `E_DECISION_REQUIRED` and have no handler — by design, but it means the frozen command surface has three deliberately dead entries. | `hiverouter-contract/src/command.rs:484-486`; `ui_command_matrix.rs:114` |
| 22 | R9-B22 | **P2** | hygiene | `http.rs:985` is a mangled signature line (uncommitted); `execution.rs:143-145` and `:2502-2503` are orphaned/reattached doc comments (the first line was lost, so `is_cross_branch_message` carries `execute_send_to_sibling`'s doc). | `http.rs:985`; `execution.rs:143-151`, `:2502-2517` |

---

## 3. Existing tests that pin the current (buggy) behaviour — do not "fix" without changing these

| Test | File:line | Pins |
|---|---|---|
| `hello_replay_and_gap_resync` | `tests/ws_transport.rs:127-133` | `SERVER_HELLO.last_seq == 3` **before** replayed `seq 2, 3` (R9-B03) |
| `durable_role_memory_is_recalled_into_worker_context` | `tests/run_lifecycle.rs:505-544` | memory written by a *different* run is recalled by role (R9-B01) |
| `planned_parent_chain_is_persisted_not_flattened_to_root` | `tests/run_lifecycle.rs:408-461` | only 3 plan nodes execute for a 4-level tree; root never runs (R9-B06) |
| `stale_expected_revision_requires_resync` | `tests/graph_authority.rs:47-62` | graph CAS semantics and post-rewire revision advance (R9-B12/R9-B13) |
| `rewire_ack_and_node_event_fan_out_to_both_clients` | `tests/ws_transport.rs:174-235` | rewire ACK/NODE_REWIRE order and `kind` on the wire (R9-B14) |
| `history_http_api` | `tests/snapshot_history.rs:151-224` | roster shape/parent/state from the task row (R9-B02/R9-B15) |
| `history_messages_route_returns_persisted_transcript` | `tests/http_boot.rs:526-538` | `truncated == false` for 2 messages (no >cap case, R9-B05) |
| `agent_memory_route_lists_and_clears_role_memory` | `tests/http_boot.rs` | panel keyed by persona id (R9-B16) |
| `timeout_fails_run_with_e_approval_timeout_and_no_side_effect` | `tests/suspension.rs` | timeout terminal code/ordering (R9-B04/R9-B11) |
| `provider_key_check_*` | `tests/recovery_gates.rs:63-119` | check shape/code/secret-free — **not** the WS guard (R9-B09) |
| `kill_restore` / `corrupt_snapshot_at_recovery_is_refused_not_reset` | `tests/kill_restore.rs:47-117` | recovery uses **unscoped** `capture` (R9-B18) |

**Commands (documented, NOT executed in this recon):**

```powershell
cd backend
cargo test --workspace --locked                                     # full backend gate
cargo test -p hiverouter-core --test ws_transport                    # hydration/replay/rewire
cargo test -p hiverouter-core --test snapshot_history --test http_boot # history + roster + messages
cargo test -p hiverouter-core --test run_lifecycle --test suspension   # lifecycle + approvals
cargo test -p hiverouter-core --test kill_restore --test recovery_gates # boot recovery
cargo test -p hiverouter-core --test tool_loop_engine --test tool_loop_matrix # tool loop caps
cargo test -p hiverouter-provider                                    # refusal/streaming matrices
```

The only recorded run in this tree is `docs/evidence/round7-backend-tests.log` (475 passed / 0 failed / 2 ignored
per `docs/CODEMAP/10-VERIFICATION.md`); it predates the uncommitted roster change, so the roster test has **not**
been shown to pass.

---

## 4. Bugs to add to known areas

### New (not in the LOG-xx list)

1. **LOG-12 (new) — memory scope is a role, not a hive/branch/run.** `execution.rs:880` + `store.rs:612-671` +
   `context.rs:138-186`. Facts leak across runs and branches and survive rewire. Test
   `run_lifecycle.rs:505-544` currently rewards the leak.
2. **LOG-13 (new) — `REWIRE_TASK` is graph-only; the `tasks` row is never updated (no DB writer exists).**
   Divergence is permanent and is read by roster, A2A sibling gate, fan-out counting, aggregator child selection
   and recovery. `ws.rs:650-736`, `store.rs:300`.
3. **LOG-14 (new) — terminal `RUN_STATUS` precedes the terminal snapshot** (and the timeout path never writes one
   itself). `execution.rs:1282-1312`, `:2367-2396`.
4. **LOG-15 (new) — history truncation keeps the oldest 1000 messages and drops the newest.** `http.rs:782-786`,
   `store.rs:439-441`; no >cap test.
5. **LOG-16 (new) — HyperAgent root never executes and has no aggregate read path.** `execution.rs:1331-1345`,
   `planner.rs:143-193`, `tool.rs:391-396`.
6. **LOG-17 (new) — `ask_parent` has no answer; agent messages are write-only for the engine.** `execution.rs:2427-2497`;
   only `http.rs:822` reads the mailbox.
7. **LOG-18 (new) — the 40-provider-call cap is a literal `1` comparison, not a counter.** `execution.rs:1323`,
   `:2781`, `caps.rs:66-71`.
8. **LOG-19 (new) — the keyless guard is derived from a conditional boot check, not from provider readiness**, and
   has no WS regression test. `ws.rs:743-753`, `checks.rs:143-146`, `config.rs:79-90`.
9. **LOG-20 (new) — `SERVER_HELLO.last_seq` semantic + ordering** (server seq, before replay) vs the contract field
   meaning and the client cursor. `ws.rs:500`, `:523-538`, `event.rs:178-179`, `store.ts:1365`.
10. **LOG-21 (new) — two `kind` vocabularies** (`subagent` over HTTP vs `worker` on `NODE_STATE`/metrics) and two
    `rev` counters for the same node. `http.rs:743-751` vs `execution.rs:99-106`, `:663-668`, `:730`, `:755`;
    `store.rs:300` vs `graph.rs:211`.
11. **LOG-22 (new) — recovery mislabels a crash during `awaiting_approval` as `E_APPROVAL_TIMEOUT`.** `recovery.rs:123-128`.

### Existing known areas this recon extends

- **LOG-01 (hydration):** backend half is healthy apart from LOG-20; the LOG-01 "destroy the live stream" symptom
  is client-side (`web/src/state/store.ts:1362-1391`) — do not look for a server-side cause. `RESYNC_REQUIRED`
  ordering (`ws.rs:539-562`) is correct.
- **LOG-03 (recovery):** scoping is correct; add the unscoped-`capture` regression guard (LOG-18/R9-B18) and the
  error-code fix (LOG-22).
- **LOG-07 (rev source):** the "rev source" question is answered here — `agent_roster` uses the DB `tasks.rev`,
  everything else uses the graph `node.rev` (R9-B12). Pick one before adding more consumers.
- **A2A roster/answer/mute:** roster partially exists (uncommitted, row-authoritative); **answer, topic bus and
  mute remain entirely absent** (zero code matches).
- **Remote/Watchdog agents:** still zero code (requirement gap, P1 — not a shipped lie).
- **Scheduler/scale caps:** depth/fan-out/agents/tool caps are enforced; the *provider-call* cap is not (LOG-18).

---

## 5. Files I expect to touch (future fix candidates — none touched now)

| File | Why |
|---|---|
| `backend/crates/hiverouter-core/src/ws.rs` | hello watermark/order (LOG-20); keyless guard (LOG-19); rewire persistence call + rollback (LOG-13) |
| `backend/crates/hiverouter-core/src/execution.rs` | memory scoping (LOG-12); root HyperAgent execution (LOG-16); terminal snapshot-before-status (LOG-14); provider-call counter (LOG-18); kind/rev unification (LOG-21) |
| `backend/crates/hiverouter-core/src/store.rs` | `parent_task_id` writer + `expected_rev` CAS (LOG-13); memory key/columns (LOG-12); message tail window (LOG-15) |
| `backend/crates/hiverouter-core/src/http.rs` | `history_messages` tail window (LOG-15); roster parent/rev source (LOG-13/LOG-21); memory panel identity (LOG-16); fix mangled `:985` |
| `backend/crates/hiverouter-core/src/context.rs` / `snapshot.rs` / `recovery.rs` | rewire memory-fact clearing (LOG-12); scoped-capture test seam (LOG-18); interrupted error code (LOG-22) |
| `backend/crates/hiverouter-core/src/history.rs` | stop swallowing corrupt snapshot reads (R9-B17) |
| `backend/crates/hiverouter-core/tests/*` | new >1000-message test, WS keyless test, cross-run/cross-branch memory isolation test, scoped-capture test, roster-after-rewire test |
| `backend/crates/hiverouter-contract/src/event.rs` + schemas | only if the hello field semantics change (golden schema regeneration) |

---

## 6. Unknowns and evidence still missing (UNVERIFIED)

- **No test was run.** Every "pass/fail" statement about tests is read from source. In particular the *uncommitted*
  roster assertions (`snapshot_history.rs:211-223`) and the whole workspace gate are **not** shown green.
- **The web consumer behaviour is inferred, not executed.** The statement that `store.ts:1365` makes the hello
  watermark skip replayed events is a source reading; whether any live UI path actually drops an event was not
  observed (no browser, per the seat's rules).
- **Live keyless boot not observed.** The guard's production behaviour (provider enabled, key absent) is derived
  from `main.rs:108-114` + `ws.rs:743-753`; no server boot was performed.
- **SQLite ordering assumptions:** `ORDER BY created_at ASC, rowid ASC` uses string timestamps
  (`store.rs:439-441`); same-millisecond ties rely on `rowid`. Not exercised with >1000 rows.
- **`ApiKey::present_in_env()`** (`hiverouter-provider/src/secret.rs`) was not read line by line in this seat; the
  keyless conclusion depends only on its boolean, which `recovery_gates.rs:72-73` does exercise.
- **`budget.rs` token-budget enforcement** is the one cap family I did not trace end-to-end; the *provider-call*
  cap finding (LOG-18) does not depend on it.
- **`tools/desktop-launcher/HiveMind.exe` is modified** in the working tree (binary). Not inspected; it is outside
  this seat's scope and outside `backend/crates/*`.
- **`audit/round9-src.json`** is a UTF-16 `unslop-audit` report over `web/src` (57 files), not a backend artifact;
  it was not used as evidence.

---

## 7. Recon completeness

- Every file named in the seat scope was read: `execution.rs`, `http.rs`, `ws.rs`, `recovery.rs`, `snapshot.rs`,
  `store.rs`, plus `run_state.rs`, `graph.rs`, `context.rs`, `caps.rs`, `planner.rs`, `history.rs`, `checks.rs`,
  `config.rs`, `db.rs`, `bus.rs`, `provider_executor.rs`, `tool_loop.rs`, `main.rs`, `lib.rs`;
  `hiverouter-contract` (`event.rs`, `command.rs`, `state.rs`); `hiverouter-tools` (`tool.rs`);
  `hiverouter-provider` (`client.rs`, `dto.rs`, `error.rs`).
- Tests read: `run_lifecycle`, `snapshot_history`, `ws_transport`, `http_boot`, `kill_restore`, `suspension`,
  `tool_loop_engine`, `tool_loop_matrix`, `recovery_gates`, `graph_authority`, `context_switch`, `support/mod.rs`.
- Every ledger row names an entry point, at least one caller, at least one test (or states that none exists), a
  pass path and a failure path.
- `recon incomplete` is **not** claimed for any area; the residual gaps are enumerated in §6.
