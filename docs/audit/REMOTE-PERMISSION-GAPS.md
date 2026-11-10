# REMOTE-PERMISSION-GAPS — remote/daemon authority, approval, idempotency, and gate audit

- **Task:** task-19
- **Repo:** `C:\Users\Riley\Desktop\hiverouter-v1`
- **Requested snapshot:** HEAD `919c339` (captured at audit start; `git status` was clean)
- **Mode:** read-only logic audit. No source edit, no commit, no provider key, no TokenLab call. The only written file is this document.
- **Result:** **GAPS CONFIRMED.** The only agent path that can create a detached remote or daemon is the HyperAgent root-control turn. SuperAgent/Agent cannot request one, and the root path has no durable approval, no idempotency key, and only success-biased audit. The exact-two live run shows seven tasks, but it does not durably evidence the root gate decision.

## 0. Provenance and evidence labels

| Label | Meaning in this audit |
|---|---|
| `SRC` | Read from tracked source at the requested snapshot. |
| `TEST-SRC` | Read from a test file; the test was **not executed** by this audit. |
| `LIVE-DOC` | Read from the live-run summary/narrative; no raw DB/event journal was available. |
| `INFER` | Follows from the cited code; not separately exercised. |
| `ABSENT` | The requested capability or record is not present; it is not a claim that it works. |

Snapshot note: during the audit another writer advanced the workspace to HEAD `197ee28` (docs-only) and left uncommitted source changes in `execution.rs` (two prompt-label lines), `remote.rs` (remote-created/grant attribution: hyper-created remote audits as `system`, grant `granted_by` becomes the created-by value; device creation remains operator-attributed), and `tests/run_lifecycle.rs` (+20 lines of grant/remote actor assertions). The `remote.rs` patch is a partial response to G2 but is **not reviewed or tested here**, does not add approval/idempotency/daemon audit/refusal audit, and is outside the requested `919c339` snapshot. All source findings and line numbers below are for `919c339`; current `remote.rs` line numbers after 351 shift by roughly ten lines, and current `run_lifecycle.rs` line numbers after 591 shift by +20. The live evidence files discussed below were added in `197ee28`; they were not present at `919c339`.

## 1. Verdict

1. **Only HyperAgent root control can create a detached remote or daemon.** `spawn_remote_agent` and `spawn_daemon` are not `ToolId` variants, are not in any tier allowlist, and are not offered to leaf-model turns. A SuperAgent/Agent that forges one gets `E_TOOL_NOT_FOUND` from the registry, not a request, approval, or audited refusal.
2. **Lower-level request → HyperAgent approval is not implemented.** `ask_parent` is a free-text mailbox question; it has no request kind, no target/effect, no decision, and no path from a parent answer to a remote/daemon creation. The live test document says this explicitly (`docs/audit/AGENTIC-LIVE-TESTS.md:172`).
3. **The root path replaces adjudication with direct execution.** At the requested snapshot, the root control turn executes `spawn_remote_agent`/`spawn_daemon` immediately, with no approval row/event, no request id, no idempotency key, and no durable root-tool record. Device/grant writes are audited as **operator** actions and grants are stored with `granted_by = 'operator'`, even when created by the HyperAgent. A later uncommitted patch partially changes remote/grant attribution (see §0) but not the approval, idempotency, or daemon gaps.
4. **Duplicate and late request semantics are not safe.** A duplicate remote display name becomes `E_INTERNAL_ERROR` (retryable) and can fail the run after the first remote was already committed. A duplicate daemon is accepted. A duplicate A2A open question is refused as `E_CALL_LIMIT` without an `a2a_audit` row. A device-action idempotency key returns the prior result even when the new payload conflicts.
5. **Revocation is real for the remote row but incomplete.** `revoke_remote` is revalidated inside the action/resolve transactions and is tested, but there is no public device/grant revoke, pending actions are not cancelled, and refused late/replayed approvals are not audited.
6. **Exact-two is observed but weakly evidenced.** Seven tasks and the documented role counts are present; the root gate is real in code but is an **unconditional refusal** (every validated plan has a depth-1 node, so the allow path is dead). The live summary contains no root-tool request/refusal row, no per-task export, and only one run, so the live run does not record the gate; its outcome is inferred from the task count and narrative rather than directly proven.

## 2. The intended model, and the requirement conflict

### 2.1 The audit brief's intended model

The brief asks for all of the following:

```text
lower-level agent requests detached remote/daemon
  -> HyperAgent decides yes/no using:
       conflict, collision, crash/error risk, authority, resource limits
  -> approval is durable, audited, idempotent
  -> duplicate/late requests have defined, recorded, retryable semantics
```

### 2.2 The repository's own design model

The repository does **not** describe lower-level creation requests. It is stricter:

| Source | Intended authority |
|---|---|
| `docs/research/hmars/12-role-capability-model.md:239-242` | HyperAgent may **request** a daemon (`daemon.request`, approval-gated); only the operator creates one. |
| `docs/research/hmars/12-role-capability-model.md:288` | Agent cannot spawn a daemon; daemon creation is operator/Hyper-only. |
| `docs/research/hmars/12-role-capability-model.md:1105` | HyperAgent may only request a daemon; a SuperAgent may only own an existing one. |
| `docs/research/hmars/12-role-capability-model.md:1469` | The operator always creates a daemon; a SuperAgent never creates one. |
| `docs/research/hmars/21-roles-daemons-implementation.md:990-1000` | Operator creates any daemon; HyperAgent may request one; SuperAgent/Agent/Subagent cannot create. |
| `docs/research/remote-agents/01-remote-agent-model.md:66-81` | The operator is the only authority that creates grants or approves actions; no remote self-grant, no transitive authority. |
| `docs/research/remote-agents/01-remote-agent-model.md:164-169` | Every action/refusal writes an audit row; every action has a request row and a result row. |

### 2.3 Conflict that must be resolved before implementation

**The brief and the repository disagree about who may initiate a detached record.** The brief says lower-level agents may request and HyperAgent decides. The design docs say only the operator creates, while HyperAgent may request and SuperAgent may not.

The current code implements **neither** model:

- it has no lower-level request protocol;
- it has no HyperAgent request → operator approval protocol;
- it has a direct HyperAgent root-control creation path, which is a third model.

Therefore, any fix must first choose the authority model. A patch that merely adds a root "approval" without choosing who is allowed to request will repeat the ambiguity.

## 3. What the code actually enforces

### 3.1 Agent-facing tool surface: absence, not a permission check

```text
INTENDED (brief)
  lower_agent.request_detached(kind, reason)
    -> request row pending
    -> HyperAgent policy decision
    -> approval + idempotency
    -> create only after decision

ACTUAL (code)
  ToolId = { SpawnAgent, AskParent, SendToSibling, ReportResult, Finish,
             FileRead, FileList, FileWrite, ShellExec }        tool.rs:111-130
  registry has no remote/daemon/root-control tool               tool.rs:335-392
  tier allowlists contain only registry tools                   tool.rs:394-445
  leaf model gets only allowlist specs                          tool_loop.rs:200-219
  forged root tool -> registry.resolve()
      -> E_TOOL_NOT_FOUND (ToolNotFound)                        tool.rs:481-493
      -> execute_tool_turn records refusal + tool_turns row      execution.rs:2439-2457
```

`spawn_remote_agent`, `spawn_daemon`, and `spawn_superagent` are deliberately **not** in the registry. The root-control comment says so directly: root tools "never enter the global tool registry, never become WS commands, and cannot be requested by leaf tasks" (`execution.rs:3854-3856`). The enforcement is therefore structural absence. It is not a `tier` check that would survive a future registry addition.

**What a lower agent can do:** call `ask_parent` with free text (`tool.rs:226-232`). That is a durable question, not a structured request. The parent's answer can be text only; it has no tool/effect authority and no path back into root control.

**Consequence:** a SuperAgent cannot even get `E_TOOL_NOT_ALLOWED` for a root-only action; it gets `E_TOOL_NOT_FOUND`, which is the wrong semantic and does not record a request attempt in any control-plane audit.

### 3.2 HyperAgent root-control path: direct execution

```text
INTENDED (brief)
  root decides yes/no, records approval, executes atomically, idempotently

ACTUAL (code)
  root_checkpoint (only root task; production enabled)             execution.rs:3857
    if !root_orchestrator: return                                  :3864-3866
    one provider call, max 4 advertised tool calls                 :3890, :3901
    for each of up to 4 calls:                                     :3914
      spawn_superagent -> refused for every validated plan (see §3.3)  :3916-3927
      spawn_remote_agent -> create_remote + create_device + grant  :3929-3930, :4005-4086
      spawn_daemon -> daemons.create(...)                          :3933-3971
      read_memory_summary -> memory_hyper_summary(run, 8)          :3973-3987
      finish -> return                                             :3989
    _result_text is built and discarded; no event/row                :3913
```

Production wires the stores and enables the root turn (`http.rs:329-337`). The root turn runs before the `Planning -> Running` transition (`execution.rs:1899-1914`).

There is no approval object for any of these operations. The only approval machinery in the tree is for registered tool ids (`backend/crates/hiverouter-tools/src/approval.rs:984-1040`); `request_tool_approval` refuses an unknown tool, so `spawn_remote_agent`/`spawn_daemon` cannot use it. The remote/daemon root calls bypass `ToolsBridge`, the `approvals` table, `APPROVAL_REQUEST`, and `APPROVAL_RESOLVED` entirely. The existing tool gate is not the defect: it already has nonce/TTL/args binding/CAS/audit semantics; the defect is the bypass.

The root can therefore perform an authority-changing operation with:

- no durable request id;
- no decision record;
- no approval nonce/TTL;
- no idempotency key;
- no `TOOL_CALL`/`TOOL_RESULT` event for the root turn;
- no run-scoped audit row that says the HyperAgent requested it.

The remote/device/grant rows are the only trace, and their audit attribution is wrong (see §4.2).

### 3.3 Root branch gate: the allow branch is dead code

```text
INTENDED (comment in code)
  "may add one branch lead only when the planner produced a flat plan"
                                                          execution.rs:3881-3884

PLAN VALIDATION (before the gate)
  Plan::validate rejects an empty plan                              planner.rs:50-53
  a node with no parent must have depth == 1                        planner.rs:96-101
  every parent chain therefore terminates at a depth-1 node
  plan.validate() is called before the root task exists              execution.rs:1777

ACTUAL (code)
  has_planned_branch = any planned node has depth == 1             :3885
  if has_planned_branch:
      spawn_superagent result = "refused" text only                :3917-3920
  else:
      spawn_sub_hive(root, "super")                                :3922
  no counter, no per-run cap, no audit row
  MAX_ROOT_TOOL_CALLS = 4                                          :3890
  every call in the response is iterated up to 4                   :3914
  comment at :3886-3889 says "root adds a few extra branches"
```

**The `else` branch is unreachable for every validated plan.** `Plan::validate` requires a non-empty node set and requires every parentless node to be depth 1 (`planner.rs:50-53,96-101`). Any valid plan must therefore contain at least one depth-1 node, so `has_planned_branch` is always true and `spawn_superagent` is always refused. The "flat plan" state named by the comment cannot be produced by the current validator. The earlier ten-task attempt reported in the live document is a pre-root-cap observation, not evidence that the current allow path works; a plan itself can still contain many nodes.

Consequences:

1. Root `spawn_superagent` is a dead capability under the current plan contract. It cannot add even one branch. The comment and the `spawn_sub_hive` call at `execution.rs:3922` are misleading. Branch creation still exists through the planner, the operator `SPAWN_SUB_HIVE` command, and model `spawn_agent` under the caps; only the root-model path is dead.
2. The gate only guards `spawn_superagent`. It does not guard `spawn_remote_agent` or `spawn_daemon`, which still execute up to the four-call cap.
3. A gate refusal is appended to `_result_text`, which is never sent back to the model and never persisted or published; the code comment at `execution.rs:3996-3999` says the result text is intentionally not fed back. The model and operator cannot tell from durable state that the request was refused.
4. The gate is not an approval, not a policy decision record, and not idempotent.
5. Because the refusal is unconditional, the exact-two live result (no extra branch) is consistent with the gate but does **not** prove a conditional decision. It proves the planner's branches were used and the root's add was refused. The intended "add only when the planner supplied none" branch was never exercised.

The focused unit test covers one `spawn_superagent` call against a plan that already has one depth-1 node (`run_lifecycle.rs:599-630`; executor at `:305-329`). It proves the branch is not added. It does **not** exercise the allow path, because no valid plan can reach it.

If the intended behavior is "root adds one branch when the planner supplied none", the plan contract must first define a valid plan shape with no depth-1 node (currently `E_SCHEMA_INVALID`), or the root tool must be removed. Adding a counter or approval to an unreachable branch would not fix the logic.

### 3.4 Remote create/device/grant path

```text
ACTUAL (root path)
  execute_root_remote_call(task, call)                             execution.rs:4005
    call.name must be spawn_remote_agent                           :4010-4012
    display_name non-empty (store also enforces 1..120)            :4013-4019
    kind defaults to daemon_relay; only light|garage|daemon_relay  :4020-4027
    remote.create_remote(owner_kind="hive", owner_id=root.id,
                         created_by="hyper:<root>")                :4029-4039
    if light|garage|daemon_relay:
      create_device(...)                                           :4064-4072
      create_grant(... capability/action narrow ...)                :4073-4080
    return text                                                    :4082-4085
```

`create_remote`, `create_device`, and `create_grant` are **three separate store calls with three separate transactions** (`remote.rs:312-370`, `:452-508`, `:552-605`). If device/grant creation fails, the remote record is already committed. If a later root call fails, the run is failed by `dispatch_inner` (`execution.rs:1899`, `:1676-1681`) but the partial records remain.

The store writes these audit rows:

| Operation | Audit actor | Problem |
|---|---|---|
| `create_remote` | `actor_kind="operator"`, `actor_id=created_by` (`remote.rs:352-365`) | `actor_id` may be `hyper:<root>` while `actor_kind` says operator; `remote_audit.actor_kind` has no agent kind (`db.rs:750-754`). |
| `create_device` | `actor_kind="operator"`, `actor_id="operator"` (`remote.rs:489-502`) | Root-created device is indistinguishable from an operator device. |
| `create_grant` | `granted_by='operator'` in SQL (`remote.rs:572`); audit `actor_kind="operator"` (`remote.rs:584-597`) | An agent-created grant claims operator consent; `NewGrant` has no actor field (`remote.rs:141-149`). |

This is the strongest audit defect in the remote plane: the trace says operator consent where the actual actor was the HyperAgent model turn.

The HTTP operator routes are a separate surface (`http.rs:626-649`, `:1745-1925`). They are token-gated as operator mutations (`http.rs:704-736`), but they use the same store methods and the same operator attribution, so they cannot distinguish operator from root either.

The supervisor tick is also narrower than the design: it expires leases, expires approvals, and ticks daemons (`remote_supervisor.rs:96-100`); it does not probe a remote or transition it to `degraded`/`quarantined`. A remote becomes `online` only through an explicit heartbeat (`remote.rs:633-674`), and the only shipped transport is `in_process` (`remote.rs:312-315`).

### 3.5 Daemon enrollment

```text
ACTUAL (root path)
  spawn_daemon:
    display_name required, kind default "watchdog", interval default 60
    daemons.create(owner_kind="hive", owner_id=root.id,
                   created_by="hyper:<root>")                      execution.rs:3933-3971
    job = {"check":"db"} for watchdog, "{}" otherwise              :3960-3964
    state starts "enrolled"                                        daemon.rs:169
    no audit row, no approval, no request row, no idempotency key
```

`DaemonStore::create` validates the owner/kind/interval and inserts (`daemon.rs:146-187`). It writes no `remote_audit`/`a2a_audit`/approval record. `set_enabled` and `retire` also write no audit (`daemon.rs:219-265`). Only runtime reports are durable (`daemon.rs:500-548`).

There is no unique daemon name, so repeated root calls create multiple daemons. `MAX_ROOT_TOOL_CALLS=4` is the only bound; there is no global daemon count cap.

The live Test C documents exactly this path: root control creates `Porch` (light remote + device + grant) and `Pulse` (watchdog daemon), run completed, no approval (`docs/audit/AGENTIC-LIVE-TESTS.md:99-137`; `docs/evidence/live/live-tokenlab-remote-daemon.json:1-30`).

### 3.6 A2A ask_parent attempts and audit

```text
ACTUAL
  execute_ask_parent:
    parent exists and is not terminal                                execution.rs:3225-3233
    text non-empty and <= 2,000 chars                                :3218-3223
    duplicate open identical question -> refused E_CALL_LIMIT        :3244-3258
    insert message under 32/run budget                               :3259-3282
    set 120 s deadline                                               :3283-3297
    record_agent_message_attempt(parent_answer, max 2)               :3302-3315
    parent answer -> CAS answer_agent_message -> audit               :3330-3355
    else wait on in-process waiter / operator answer                 :3367-3402
    timeout -> expire if still open                                  :3399
    status + answer -> ok/expired/timeout result                     :3405-3438
```

What works:

- One durable message row, one deadline, one in-process waiter.
- Parent answer is CAS-transitioned and audited (`store.rs:983-1043`).
- Attempts are persisted and audited atomically; cap is 2 (`store.rs:919-976`; `execution.rs:97`).
- A terminal parent is now refused before insert (`execution.rs:3225-3233`).
- The live A2A run shows `status=answered`, `attempts=1`, `answer_actor=parent:<superagent>` (`docs/evidence/live/live-tokenlab-a2a.json:1-24`).

What is incomplete:

1. **Duplicate open question is classified as `E_CALL_LIMIT`.** That code means "run exceeded the 40-call cap" (`refusal.rs:161-163`), and it is non-retryable. The duplicate is refused before `record_agent_message_attempt`, so it produces no `a2a_audit` row and no attempt increment. It is recorded only as a `tool_turns` refusal (`execution.rs:2691-2719`) with the wrong code.
2. **Operator answer does not check the message deadline.** `answer_agent_message` CASes `status='open'` only (`store.rs:983-1043`). The HTTP route checks run existence, not run/task terminality (`http.rs:1020-1075`). If the in-process timer has not yet expired the row, an answer after `deadline_at` can still win; if the run is terminal but the row is still open, the answer can still be accepted. Only a restart calls `expire_open_agent_messages` (`recovery.rs:139-142`).
3. **Expiry is not audited.** `expire_agent_message` updates status only (`store.rs:1093-1104`); boot expiry updates all open rows without an audit row (`store.rs:1848-1860`).
4. **Late/replayed answers are refused but not audited.** After expiry, `answer_agent_message` returns `E_TASK_TERMINAL`; after a successful answer it also returns `E_TASK_TERMINAL`; neither path writes an `a2a_audit` row (`store.rs:1028-1042`; HTTP 409 at `http.rs:1070-1072`).
5. **Operator answers do not increment attempts.** Attempts count parent-answer attempts only (`execution.rs:3302-3315`); the live UI shows the persisted parent count (`http.rs:2198-2200`). This is a truthful distinction only if the UI says so; it currently does not.

### 3.7 Memory scoping

```text
ACTUAL
  task read set:
    run bank + own branch + parent branch + nearest ancestor hive     store.rs:2180-2228
    sibling and foreign-run banks not in the IN list                  :2180-2228
    nearest_hive_bank walks up to 8 ancestors, stops at hyper         :2753-2780
    test proves sibling exclusion                                     :3501-3524
  root summary:
    run/hive entries only, max 8                                      execution.rs:3973-3987
    store query joins memory_banks by run and kind IN ('run','hive')  store.rs:2784-2815
  remote/daemon:
    no memory columns in remote_agents/daemons                        db.rs:663-687, :791-814
    no RemoteStore/DaemonStore memory methods
    no bank scope id, no memory read/write path
```

What this means:

- Task memory read scoping is implemented and has a sibling-exclusion test.
- The HyperAgent summary is bounded to run/hive, not raw branch banks.
- **Remote/daemon memory scoping is absent, not implemented.** The remote design's "system bank read-only + private remote bank" (`docs/research/remote-agents/02-memory-isolation.md:12-15`) has no table, API, or read path in the product. Because a remote/daemon has no model/tool, it also has no route by which to leak a bank. This is a missing feature, not a demonstrated leak. Do not claim remote memory isolation.
- `memory_entries_for_bank` is an unscoped read API (`store.rs:2250-2287`) with test callers only in this snapshot; the HTTP bank route uses the run-scoped `memory_entries_for_bank_scoped` (`http.rs:2323-2345`). Neither is reachable by an agent tool. Their existence does not create a remote/daemon scope.

### 3.8 Revocation and late decisions

```text
REMOTE REVOKE (works, but incomplete)
  revoke_remote: state='revoked', revoked_at, rev+1, audit          remote.rs:414-446
  heartbeat requires revoked_at IS NULL                             :633-674
  request_action rechecks remote/device/grant/lease in tx           :945-1040
  resolve_action rechecks remote/device/grant/lease in tx           :1158-1266
  tests prove revoke/lease/device races are refused                 remote_control_plane.rs:82-188

GAPS
  no public device revoke / grant revoke (fields exist)             db.rs:689-721
  revoke_remote does not cancel pending device_actions               remote.rs:414-446
  a revoked remote leaves the action awaiting_approval              remote_control_plane.rs:100-114
  refusal paths return before audit
    - precheck expiry/nonce                                          remote.rs:1144-1156
    - in-tx state/nonce/expiry/grant                                 :1174-1239
    - no audit on any Err path                                       :1275-1288 is success-only
  request_action refusals also return before audit                   :909-1021
  remote_audit refusal_code column is never written                  remote.rs:259-296
```

`expire_approvals` is the exception: the sweep writes one supervisor audit row per expired approval (`remote.rs:678-725`; test `remote_control_plane.rs:190-262`). That proves the sweep, not the late operator decision.

### 3.9 Resource limits

- `MAX_ROOT_TOOL_CALLS=4` bounds one root response (`execution.rs:3890`).
- `spawn_sub_hive` enforces depth/fan-out/agent caps and persona limits (`execution.rs:1440-1513`).
- No cap exists on the number of remotes or daemons. The root control turn can create up to four remotes/daemons per run; across runs the tables grow without a policy limit.
- Remotes/daemons are intentionally tree-free and do not consume `MAX_AGENTS`/`MAX_CALLS` (`remote.rs:1-6`; `daemon.rs:1-10`). That is a design choice, but it means the root's only resource check is the per-turn tool cap.
- The daemon supervisor ticks at most 64 daemons per pass (`remote_supervisor.rs:96-100`), which is a throughput bound, not an admission cap.

## 4. Side-by-side: intended decision inputs vs actual behavior

| Decision input from the brief | Actual root behavior | Evidence |
|---|---|---|
| **Conflict** — two requests compete for the same target/branch | No conflict detection. Same remote display name hits a DB unique index and becomes `E_INTERNAL_ERROR`; the first remote is already committed. Same daemon name is accepted repeatedly. | `db.rs:687`; `remote.rs:327-368`; `daemon.rs:146-187` |
| **Collision** — two creates/actions collide on identity or idempotency key | Remote/device/grant have DB uniqueness; device action key returns the prior row even if the payload differs; no conflict-specific refusal code exists. | `db.rs:705,720,745`; `remote.rs:928-935`; no `E_DEVICE_IDEMPOTENCY_CONFLICT` in `refusal.rs` |
| **Crash/error risk** | Remote+device+grant are three transactions. A failure after the first leaves a partial remote. A root-call error fails the run; no compensating delete. | `remote.rs:312-370,452-508,552-605`; `execution.rs:1899,1676-1681` |
| **Authority** | Root-only by code location; no explicit authority check, no approval, no nonce. A root-created grant is stored `granted_by='operator'`. | `execution.rs:3854-4086`; `remote.rs:572` |
| **Resource limits** | Four root tool calls per turn; no remote/daemon count cap. `spawn_superagent` is unreachable for a validated plan, so its `spawn_sub_hive` caps are not a usable branch-admission path. | `execution.rs:3890`; `planner.rs:50-53,96-101`; `remote_supervisor.rs:96-100` |

## 5. Duplicate and late request matrix

Legend: **recorded** means a durable row/event that a later operator can inspect. A tracing log or a `tool_turns` row is not an `a2a_audit`/`remote_audit` row.

| Request | Duplicate behavior | Durable record | Retryable? | Late behavior |
|---|---|---|---|---|
| `spawn_remote_agent` same display name, root turn | Second call gets `E_INTERNAL_ERROR` (DB unique), first remote/device/grant remain; run fails | First `remote_created`/`device_created`/`grant_created` rows only; no root-tool or failure row | `E_INTERNAL_ERROR` is `retryable=true` (`refusal.rs:314`), so a retry is encouraged without a new key | No late concept; root turn is one pass |
| `spawn_daemon` same display name, root turn | Accepted repeatedly; duplicate daemons created | Daemon row(s) only; no audit | N/A | No late concept |
| `spawn_superagent` with any **validated** plan | Always refused: every valid plan has a depth-1 node, so `has_planned_branch` is always true; no new task | No row/event; `_result_text` discarded (`execution.rs:3913`) | No named refusal/retry | N/A |
| `spawn_superagent` with a plan that has no depth-1 node | Cannot occur: `Plan::validate` rejects an empty plan and parentless depth>1 (`planner.rs:50-53,96-101`) | N/A (run fails `E_SCHEMA_INVALID` before root) | N/A | N/A |
| Device action same idempotency key, same payload | Returns prior `DeviceAction` (`remote.rs:928-935`) | Prior action row; no new audit | Idempotent success | N/A |
| Device action same key, **different** capability/action/params | Returns prior `DeviceAction` as if it were the requested action | Prior action row; conflicting request is not recorded | Appears successful; no conflict refusal | N/A |
| Critical device approval, second resolve | `E_DEVICE_APPROVAL_INVALID` before lock; 409 | No audit row | `retryable=false` (`refusal.rs:306`) | Replay is refused |
| Critical device approval after expiry | `E_DEVICE_APPROVAL_INVALID` before/inside tx; 409 | Sweep writes `device_action_approval_expired` only if the sweep ran; the late attempt itself is not audited | `false` | Refused; action remains until sweep |
| A2A duplicate open question | `E_CALL_LIMIT` (`refusal.rs:161-163`) | `tool_turns` refusal; no `a2a_audit`; no attempt increment | `false`; misleading "40-call cap" text | N/A |
| A2A second answer after answered | `E_TASK_TERMINAL`; HTTP 409 | No audit row | `false` | Refused |
| A2A answer after `deadline_at` but before expiry sweep | Accepted if row is still `open` | `answered` audit row; no lateness check | Already answered | **Can be accepted late** |
| A2A answer to a terminal run with an open row | Accepted if row is still `open` | `answered` audit row | N/A | **Late/terminal acceptance** |
| `create_remote` HTTP duplicate name | 503 `E_INTERNAL_ERROR` (fallback mapping), first row committed | First audit row only | `true` | N/A |
| `create_daemon` HTTP duplicate name | Accepted; multiple daemons | Daemon row(s) only; no audit | N/A | N/A |

### 5.1 Failure modes at a glance

| Trigger | Observed result | Recorded? | Severity |
|---|---|---|---|
| SuperAgent/Agent tries a root-only tool name | `E_TOOL_NOT_FOUND` tool refusal; no request row | `tool_turns` only; no control-plane audit | High (G1) |
| Root creates remote/daemon | Side effect occurs with no approval/request key | Remote success rows only, operator-attributed; daemon row only | High (G2) |
| Root requests the same remote name twice | `E_INTERNAL_ERROR`; run failed; first remote/device/grant remain | First create audits only; no failure audit | High (G3) |
| Root requests `spawn_superagent` with any valid plan | Always refused; branch not added | Nothing durable | High (G4) |
| Root requests a flat-plan branch | Impossible: plan validation rejects the plan | `E_SCHEMA_INVALID` before root | High (G4) |
| Root requests four unique remotes/daemons | Up to four side effects in one turn | Success rows only; no root gate record | Medium (G9) |
| Duplicate A2A open question | `E_CALL_LIMIT`; no new row | `tool_turns` refusal; no `a2a_audit` | Medium (G5) |
| Operator answers after `deadline_at` but before expiry | Accepted if still `open` | `answered` audit; no lateness marker | Medium (G5) |
| Operator answers a terminal run's open message | Accepted if still `open` | `answered` audit; no terminal check | Medium (G5) |
| Late/replayed device approval | `E_DEVICE_APPROVAL_INVALID`; 409 | No audit row for the attempt | Medium (G6) |
| Resolve after remote revoke | `E_REMOTE_UNKNOWN`; 404 | No audit row; action remains `awaiting_approval` | Medium (G6) |
| Same device idempotency key, conflicting payload | Prior action returned as success | Prior action only; no conflict row | Medium (G7) |
| Remote create fails after remote insert | Remote remains without device/grant | `remote_created` only; no failure/compensation | High (G2/G3) |

## 6. Exact-two live evidence and root branch gate

### 6.1 Observed evidence

`docs/evidence/live/live-tokenlab-hierarchy.json:1-25` records:

- `tasks: 7`
- `hierarchy: { agents: 2, superagents: 2, subagents: 2, hyper: 1 }`
- `status: completed`
- usage 14,407 prompt / 1,144 completion / 15,551 total
- run id `01a1c402-3dc8-7341-ba38-3c22cdb80edf`

`docs/audit/AGENTIC-LIVE-TESTS.md:11-52` adds the role/class matrix and the pseudocode: the planner supplied branch leads; the root requested `spawn_superagent`; the request was refused; the run completed with seven tasks. `docs/evidence/HMARS-BUGFIX-REPORT.md:216-218` states the root-cap fix is what made the exact-two run produce seven tasks rather than ten.

### 6.2 What the gate code actually proves

- `Plan::validate` rejects an empty plan and requires every parentless node to be depth 1 (`planner.rs:50-53,96-101`); it runs before the root task is inserted (`execution.rs:1777`).
- Therefore every valid plan has a depth-1 node, `has_planned_branch` is always true (`execution.rs:3885`), and `spawn_superagent` is always refused. The refusal is a local string (`execution.rs:3916-3920`).
- The focused test `hyperagent_checkpoint_refuses_an_extra_branch_when_the_planner_has_one` uses a one-branch plan and one root `spawn_superagent`, then asserts two total tasks (`run_lifecycle.rs:599-630`). It proves the refusal; it cannot exercise the allow path because no valid plan reaches it.
- The `MAX_ROOT_TOOL_CALLS=4` cap is still meaningful for `spawn_remote_agent`/`spawn_daemon`, but not for adding branches.

### 6.3 Evidence classification

| Claim | Status | Why |
|---|---|---|
| The exact-two run completed with 7 tasks and the stated role counts | `LIVE-DOC` / `VERIFIED` as a document read | Summary JSON plus narrative; no raw DB export. |
| The root gate fired during that live run | `INFER` / `WEAK` | The gate is structurally guaranteed to refuse; there is no root-tool request/refusal row, no `TOOL_CALL`/`TOOL_RESULT` for the root turn, and no gate audit. The count is consistent, but the live document does not prove the refusal was recorded or that a conditional decision occurred. |
| The code refuses an extra branch when any valid plan exists | `TEST-SRC` / `SRC` | Test code plus the validator proof; not executed by this audit. |
| The code can add one branch when the planner supplied none | **ABSENT / UNREACHABLE** | `Plan::validate` rejects the only shape that would make `has_planned_branch` false; the allow branch (`execution.rs:3921-3927`) is dead. |
| The exact-two result is stable across runs | **NOT PROVEN** | One observed run; `AGENTIC-LIVE-TESTS.md:174-182` still lists "repeat exact-two three times" as a next test. |

### 6.4 Why this matters for the permission model

The exact-two evidence is being used as proof of root governance, but the gate is an **unconditional refusal**, not a decision. It does not prove:

- who requested the extra branch (no request row);
- that the HyperAgent made a durable decision (no decision row);
- that the refused request is auditable (no row);
- that a root branch can ever be added when the planner supplied none (it cannot in this snapshot);
- that the same gate applies to remotes/daemons (it does not).

## 7. Top gaps and severity

### G1 — No lower-level request/admission path (High, requirements)

**Intended:** SuperAgent/Agent can request; HyperAgent decides.

**Actual:** `spawn_remote_agent`/`spawn_daemon` are absent from `ToolId` and all allowlists. `ask_parent` is free text with no structured request, no target/effect, and no parent decision that can create a record. A forged root tool is `E_TOOL_NOT_FOUND`.

**Failure:** the product cannot satisfy the brief's approval flow; a lower-level agent has no way to ask for a satellite/daemon except prose that no code interprets. Operators may believe the capability exists because the root can create these records.

**Smallest fix:** define one `detached_request` contract and one admission function before adding any new root behavior. The request must carry a request id/idempotency key, requesting task, kind, target/capability, reason, and desired scope. Do not expose creation until the request can be decided and recorded.

### G2 — Root creation bypasses approval and misattributes authority (High, authority/audit)

**Intended:** approval is durable, audited, idempotent; operator-only grant creation per repo design.

**Actual:** at `919c339`, root executes immediately. There is no approval row/event, no request id, no nonce/TTL, no idempotency key. Device/grant audit rows use `actor_kind='operator'`; `granted_by` is hardcoded `'operator'`; `remote_audit.actor_kind` cannot represent a HyperAgent. Daemon creation has no audit at all. Denied remote actions and refused resolutions return before any audit row, and `remote_audit.refusal_code` is never written. The uncommitted working-tree patch noted in §0 changes hyper-created remote/grant attribution to `system` and `granted_by=<created_by>`, but device creation remains operator-attributed and the approval/idempotency/daemon/refusal gaps remain.

**Failure:** an agent-originated authority grant is indistinguishable from operator consent; operators cannot audit who approved a device grant or why. A refused/denied action leaves no durable record, violating the remote design's R-4 invariant.

**Smallest fix:** use one admission/decision record; add actor kinds `hyper`/`agent` (or a separate `actor` reference) to the control-plane audit; write refusal rows for create/action/resolve; never hardcode `granted_by='operator'` for a model-originated grant.

### G3 — Duplicate root requests are unsafe and non-idempotent (High, duplicate/late)

**Intended:** duplicate requests return the prior decision/result or a named duplicate refusal; no partial record; retry is safe.

**Actual:** duplicate remote name → `E_INTERNAL_ERROR` (retryable) after the first remote/device/grant are committed; run fails. Duplicate daemon → accepted. There is no request id/key. A root burst of four unique names creates four principals. A partial failure leaves a remote without device/grant.

**Failure:** a provider retry or a repeated root tool call can fail the run while leaving side effects; a retryable internal error invites a retry that creates a second principal under a new name.

**Smallest fix:** make the create operation a single transaction keyed by request id; `INSERT ... ON CONFLICT` or a pre-read under the same transaction; duplicate returns the prior result; name collision is a named `E_DUPLICATE_*` refusal, not `E_INTERNAL_ERROR`.

### G4 — Root branch gate is an unaudited, unreachable allow path (High, gate)

**Intended:** root adds at most one branch when the planner did not; refuses extra branches; decision is recorded.

**Actual:** `Plan::validate` guarantees at least one depth-1 node for every valid plan (`planner.rs:50-53,96-101`; called at `execution.rs:1777`). `has_planned_branch` is therefore always true (`execution.rs:3885`), and `spawn_superagent` is always refused. The allow branch (`execution.rs:3921-3927`) is dead. The refusal is discarded; there is no gate audit row; the focused test only proves the refusal. `spawn_remote_agent`/`spawn_daemon` are not gated at all.

**Failure:** the root cannot exercise its documented branch-add capability; the exact-two result is an unconditional refusal, not a conditional policy decision; the operator cannot see the gate; the gate's stated "flat plan" case cannot be produced. A future change that makes a flat plan valid would suddenly open the unreviewed four-call burst path.

**Smallest fix:** decide the intended topology first. If root may add a branch when the planner supplied none, define a valid plan shape with no branch lead and test the allow path. If root may not add a branch, remove `spawn_superagent` from the root prompt/tool path. In either case persist a `root_gate` decision row (requested/allowed/refused, reason, run/task) and apply the same admission to remotes/daemons.

### G5 — A2A duplicate/late semantics are incomplete and partly untruthful (Medium, A2A audit)

**Intended:** duplicate questions/answers are named and recorded; a late answer is refused by deadline/terminal state; attempts are visible.

**Actual:** duplicate open question → `E_CALL_LIMIT` (wrong meaning) with no `a2a_audit`; operator answer does not check `deadline_at` or run/task terminality; expiry has no audit; late/replayed answers return `E_TASK_TERMINAL` with no audit; operator answers do not increment attempts.

**Failure:** the model sees a false "40-call cap" message; an operator can answer a question after the deadline or after the run is terminal; the audit cannot distinguish an expired/replayed/late answer from a valid one.

**Smallest fix:** add a named duplicate code; record duplicate/expired/late attempts in `a2a_audit`; check `deadline_at` and run/task state inside the answer CAS; optionally count operator answers as a distinct `operator_answer` attempt or expose the distinction in the UI.

### G6 — Revocation is row-level only; pending work and denials are not handled (Medium, revocation)

**Intended:** revocation is immediate, pending actions are cancelled or flagged, every refusal is audited; device/grant can be revoked.

**Actual:** `revoke_remote` revalidated in action/resolve transactions, tested. But no public device/grant revoke; pending actions remain `awaiting_approval`; refusal paths return before audit; `remote_audit.refusal_code` unused.

**Failure:** a revoked remote can leave a device blocked by a stale approval until the TTL sweep; an operator cannot revoke one compromised device/grant without revoking the whole remote; denied/late attempts have no audit.

**Smallest fix:** on `revoke_remote`, transition non-terminal `device_actions` to `cancelled`/`denied` with an audit row (or explicitly document the TTL-only policy); add device/grant revoke methods/routes; audit every refusal with `refusal_code`.

### G7 — Device-action idempotency ignores conflicting payloads (Medium, duplicate)

**Intended:** same key + same payload returns prior result; same key + conflicting payload returns `E_DEVICE_IDEMPOTENCY_CONFLICT`; different key while in flight returns `E_DEVICE_ACTION_IN_FLIGHT` (`docs/research/remote-agents/03-device-capabilities.md:105-107`).

**Actual:** `find_action(device_id, idempotency_key)` returns the prior row without comparing capability/action/params (`remote.rs:928-935`). No `E_DEVICE_IDEMPOTENCY_CONFLICT` code exists. The prior result is returned as success.

**Failure:** a caller can believe a different action was performed; an idempotency key collision silently maps two distinct requests to one effect.

**Smallest fix:** compare `(capability_id, action, params_sha256, expected_json)` before replay; return a named conflict on mismatch; keep the unique index.

### G8 — Remote/daemon memory scope is absent (Low, capability absence)

**Intended (remote design):** remote gets system-bank read-only + private remote bank, isolated.

**Actual:** no memory columns in `remote_agents`/`daemons`, no store methods, no model/tool for a remote/daemon to read or write memory. Task memory scoping exists and excludes siblings (`store.rs:3501-3524`).

**Failure:** none demonstrated; the feature is absent. This is recorded so future docs do not claim remote memory isolation.

**Smallest fix:** do not add a bank until the remote/daemon has a model/tool path; if the feature is required, add an explicit `remote_id`/`daemon_id` bank scope, read-only system access, and a test proving no run/hive/branch read.

### G9 — No global remote/daemon admission cap (Low/Medium, resource limits)

**Intended:** HyperAgent decides using resource limits.

**Actual:** only four root tool calls per turn; no max remotes/daemons; no run-level or global count check; root tool calls are not counted in `runs.tool_calls`/`run_tool_counters`; daemons start enrolled and can be created repeatedly.

**Failure:** a model can create up to four remote/daemon records per run, and repeated runs grow the roster without a policy limit. The root prompt says "at most once" only in the final-answer prompt that offers no tools; the control prompt says "one or more."

**Smallest fix:** add an admission cap (per run and global) to the same admission function as G1/G2; return a named `E_REMOTE_LIMIT`/`E_DAEMON_LIMIT`; do not rely on model prose.

### G10 — Root final prompt advertises a tool it cannot call (Low, dead path/lying prompt)

`ROOT_SYSTEM` says the HyperAgent "may call spawn_remote_agent at most once" (`provider_executor.rs:46-52`), but `finalize_root` builds a request with no `.with_tools(...)` (`provider_executor.rs:615-641`). The final turn cannot call it. The only actual call site is the earlier `orchestrate_root` turn (`provider_executor.rs:544-586`), which has no "at most once" enforcement.

**Failure:** the model is told an affordance exists that the wire does not; prompt-based behavior may be inconsistent. This is a truth defect, not an authority bypass.

**Smallest fix:** either remove the tool claim from the final prompt or give the final turn no tool claim; keep the actual affordance in `orchestrate_root` and enforce its cap there.

## 8. Smallest logical fix order

The order is chosen so each step can be tested without depending on a later one. Do not start with UI or with a new remote adapter; the authority/audit boundary must exist first.

| Order | Fix | Depends on | Acceptance |
|---|---|---|---|
| 1 | **Define one request/decision contract** for detached creation: request id, idempotency key, requesting task, kind, target/capability, reason, decision, actor, timestamps. Decide the authority model (brief's lower-level request vs repo's operator-only) before coding. | none | A contract test can insert/read a pending request and a decision; no side effect occurs without a decision row. |
| 2 | **Route all creation through one admission function** used by the root turn and operator HTTP; atomic remote+device+grant; one transaction; actor `hyper`/`operator` preserved. | 1 | Root and HTTP calls produce the same request/decision shape; no partial remote after a forced device/grant failure; audit actor is truthful. |
| 3 | **Make duplicates and late decisions explicit**: same key returns prior decision/result; conflicting payload returns a named conflict; expired/terminal decision returns a named refusal and an audit row. | 1, 2 | Duplicate root call does not fail the run or create a second principal; late decision is refused and recorded; `E_INTERNAL_ERROR` is not used for duplicate names. |
| 4 | **Resolve the root branch gate**: decide whether root may add a branch. If yes, define a valid no-branch-lead plan shape and test the allow path; if no, remove `spawn_superagent` from the root path. Persist a gate decision row; apply the same admission to remote/daemon; count admitted root side effects in the run tool counters. | 1-3 | The allow path is either reachable with a valid plan and tested, or the tool is absent; the planned-branch case adds zero; a durable gate row exists; remote/daemon use the same admission and are counted. |
| 5 | **Tighten A2A**: named duplicate code; audit duplicate/expired/late attempts; check `deadline_at` and run/task status in the answer CAS. | none (independent) | Duplicate question returns a named duplicate and an `a2a_audit` row; late/terminal answer 409 with an audit row; attempts and answers are distinguishable. |
| 6 | **Tighten remote revocation and device idempotency**: cancel/flag pending actions on revoke; add device/grant revoke or document the absence; compare params on idempotent replay; audit every refusal. | 2 (for audit actor) | Revoked remote leaves no stale `awaiting_approval`; conflicting key returns a named conflict; denied/expired/replayed resolutions have audit rows. |
| 7 | **Evidence**: repeat exact-two three times; capture root gate/admission row if the gate is retained, task roles, and DB export; run Test C and a lower-level request test once the protocol exists. | 1-6 | 3/3 stable counts; a gate/admission row exists if the gate is retained; Test C shows request→decision→create with the correct actor; no claim beyond observed runs. |

Do not combine steps 2 and 4: the admission function must exist before the gate can record a decision. Do not add a lower-level request tool before step 1; it would create a second, undocumented path.

## 9. Concrete test proposals

These are proposals; none was executed by this audit. Commands assume the repo root. Provider-key tests must remain operator-authorized and must not be run by an audit.

### 9.1 Structural/permission tests

| Test | Assertion |
|---|---|
| `remote_permission.rs::leaf_cannot_request_detached_record` | A SuperAgent/Worker task calling `spawn_remote_agent`/`spawn_daemon` gets a named refusal, and no request/remote/daemon row exists. (Today the expected code is `E_TOOL_NOT_FOUND`; after G1 it should be `E_PERMISSION_REQUEST_REQUIRED` or the chosen admission refusal.) |
| `remote_permission.rs::root_request_requires_decision` | A root request without a decision row and approval cannot create a remote/daemon; no partial entries. |
| `remote_permission.rs::root_request_is_idempotent` | Same request key twice returns the same decision/opaque result; exactly one remote/device/grant; no second audit `created` row. |
| `remote_permission.rs::conflicting_request_key_is_refused` | Same key, different kind/name/capability returns a named conflict; no second row. |

### 9.2 Root gate tests

| Test | Assertion |
|---|---|
| `run_lifecycle.rs::root_gate_allow_path_is_reachable_or_tool_removed` | If `spawn_superagent` is retained, a valid plan shape must reach the allow branch and create one branch; if no such valid shape is defined, assert the tool is absent from the root prompt/checkpoint. A "flat plan" fixture that fails `Plan::validate` is not acceptable. |
| `run_lifecycle.rs::root_planned_branch_adds_zero` | Existing depth-1 plan + `spawn_superagent` → zero new tasks; one gate-refusal row. |
| `run_lifecycle.rs::root_gate_decision_is_durable` | The gate row exists with run/task/requested/allowed/refused; the refusal is not only `_result_text`. |
| `run_lifecycle.rs::root_remote_daemon_have_same_admission` | Remote and daemon creations also produce a gate/admission row; a four-call burst is capped. |

### 9.3 Remote control-plane tests

| Test | Assertion |
|---|---|
| `remote_control_plane.rs::duplicate_remote_name_is_named_and_does_not_fail_run` | Second create returns `E_DUPLICATE_REMOTE`/chosen code; first row remains; run not failed; no partial records. |
| `remote_control_plane.rs::remote_create_is_atomic` | Force device/grant failure; assert no remote row without its device/grant (or a documented partial state with an audit row). |
| `remote_control_plane.rs::hyper_created_grant_is_not_attributed_to_operator` | `granted_by`/audit actor is `hyper:<task>`; `remote_audit.actor_kind` can represent it. |
| `remote_control_plane.rs::request_action_refusal_is_audited` | Missing grant/offline/revoked/in-flight each produces a `remote_audit` row with `outcome='refused'` and `refusal_code`. |
| `remote_control_plane.rs::late_resolve_is_refused_and_audited` | Expire approval, resolve with valid nonce; assert refusal, no state change, and one audit row. |
| `remote_control_plane.rs::revoke_cancels_pending_action` | Create awaiting approval, revoke remote; assert action is cancelled/denied and audited before TTL. |
| `remote_control_plane.rs::duplicate_key_with_different_params_is_conflict` | Same device/key, different action/params; assert named conflict and no second effect. |

### 9.4 A2A tests

| Test | Assertion |
|---|---|
| `tool_loop_engine.rs::duplicate_open_question_is_audited` | Duplicate text while open returns a named duplicate code; one `a2a_audit` duplicate row; attempts unchanged or explicitly recorded. |
| `store.rs::late_operator_answer_after_deadline_is_refused_and_audited` | Deadline passed but row open; answer returns a named deadline/late refusal; no `answered` state; audit row. |
| `store.rs::terminal_run_cannot_answer_open_question` | Run cancelled/completed with an open message; answer returns `E_RUN_TERMINAL`/`E_TASK_TERMINAL`; no `answered`; audit row. |
| `store.rs::expiry_is_audited` | `expire_agent_message`/boot expiry writes an `expired` audit row. |
| `store.rs::operator_answer_is_not_a_parent_attempt` | If attempts are intended to count all answers, assert the chosen distinct semantics; otherwise assert the UI labels parent-only attempts. |

### 9.5 Memory tests

| Test | Assertion |
|---|---|
| `store.rs::remote_has_no_memory_scope` | There is no remote/daemon bank or read API; if one is added, a remote cannot read run/hive/branch/role banks. |
| `store.rs::memory_read_set_excludes_sibling_branch` (existing) | Keep it; it is the current proof for task scoping. |
| `store.rs::root_summary_excludes_raw_branch` | Root summary contains only run/hive entries; no sibling branch value. |

### 9.6 Live evidence (operator-authorized only)

1. Repeat exact-two three times with a fresh data directory; capture the run row, task rows, root gate/admission row if the gate is retained, and event journal. Acceptance: 3/3 seven tasks; no extra branch; the gate decision is either durably recorded or the root branch tool is absent by design.
2. Drive Test C (root-created remote/daemon) after G1/G2; capture request/decision/audit rows and actor IDs. Acceptance: the request is pending before creation, the decision is durable, `granted_by` is truthful, and no `E_INTERNAL_ERROR` appears.
3. Drive a lower-level request after G1; approve then refuse. Acceptance: approve creates exactly one record; refuse creates none; both are audited and idempotent on replay.
4. A2A late-answer test with a short deadline (no provider spend required if using a test executor); assert 409 + audit, not `answered`.

## 10. Non-claims and evidence limits

The following are **not implemented** in the audited snapshot and must not be claimed as present:

- lower-level remote/daemon request protocol;
- HyperAgent request → operator approval for a detached record;
- approval/nonce for root-created remote/daemon;
- idempotency key for remote/daemon creation;
- durable audit for daemon create/enable/retire;
- device/grant revoke API;
- cancellation of pending actions on remote revoke;
- `E_DEVICE_IDEMPOTENCY_CONFLICT`;
- remote/daemon memory banks or isolation tests;
- network remote transport (the only adapter is `in_process`; `remote.rs:1-6`);
- root branch add when the planner supplied no branch lead (the allow path is unreachable under the current plan validator);
- root gate audit row;
- a root gate allow-path test or a defined no-branch-lead plan shape;
- 3-run stability for exact-two.

Evidence limits:

- The exact-two live evidence is a summary; it has no per-task export, no event journal, and no gate row. The role counts are in the narrative, not the JSON.
- The live Test C evidence is a summary; it does not include the request/approval/audit chain. It proves the records exist, not that an approval authorized them.
- The tests cited in this document were read, not executed. No claim of `cargo test` pass/fail is made.
- The audit made no provider call and used no key. The live TokenLab artifacts are read from the workspace only.

## 11. Audit receipt

| Item | Value |
|---|---|
| Task id | `task-19` |
| Requested HEAD | `919c3398c5bf47b4deb9ed9fd2004d2a0916cb9a` |
| Final observed HEAD | `197ee28f0ec392018549c36e09826d073a003cbf` (docs-only commit; working tree also has concurrent uncommitted `execution.rs`, `remote.rs`, and `tests/run_lifecycle.rs` changes, none reviewed here) |
| Files read | `execution.rs`, `remote.rs`, `daemon.rs`, `remote_supervisor.rs`, `provider_executor.rs`, `store.rs`, `db.rs`, `tool.rs`, `approval.rs`, `refusal.rs`, `http.rs`, `recovery.rs`, `run_lifecycle.rs`, `remote_control_plane.rs`, plus design/live docs |
| Source edits | none |
| Tests executed | none |
| Provider calls | none |
| Written | this file only |
| Top gaps | G1 no lower-level request; G2 root bypasses approval/audit attribution; G3 duplicate root requests unsafe; G4 root gate allow path unreachable/unaudited; G5 A2A duplicate/late; G6 revocation incomplete; G7 idempotency conflict silent |
| Minimal fix order | 1 request/decision contract → 2 one admission + atomic create → 3 duplicate/late semantics → 4 root gate decision + record → 5 A2A deadline/audit → 6 revocation/idempotency → 7 evidence |
