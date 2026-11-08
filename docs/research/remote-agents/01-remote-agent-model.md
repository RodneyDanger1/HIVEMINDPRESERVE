# 01 — Remote agent model, authority, lifecycle, and supervision

**Status:** DESIGN (D), with current-code facts marked V. Companion: `02-memory-isolation.md`,
`03-device-capabilities.md`, `04-contracts-api-ws-refusals.md`.
**Decisions:** D-R1, D-R2, D-R5, D-R7, D-R10 from `00-README.md`.

## 1. Definitions — five objects that must not be confused

| Object | What it is | Persistence | Task row? | Model? | Memory | Authority |
|---|---|---|---|---|---|---|
| **Task/agent** | A unit in a run's DAG (`tasks`, `run_id NOT NULL`, `parent_task_id`, depth 0–3) | `tasks` (v2; `db.rs:63-74`) | yes | worker/leaf provider call | run/hive/branch/role banks (22) | tier/profile/ownership/grant |
| **Daemon/watchdog** | One immutable job + switch + cadence + reports; deterministic check; no model/tools/budget | `daemons`/`daemon_runs`/`daemon_reports` (21 §3.7/§4.1) | **no** | **no** | **none** | operator switch only; never spawned into the tree |
| **Remote agent** | A tree-free capability principal outside any run; owns devices/capabilities; reports health; can be invoked by an operator or (later) an approved task tool | new `remote_agents` (this design) | **no** | connector-defined (may call a model locally, outside HiveMind's budget) | **system bank read + one private remote bank** (§02) | operator-enrolled; capability grants; per-action approval |
| **Device** | A physical/virtual endpoint with a class and closed actions/states (light, camera, garage cover, lock, sensor) | new `devices` | no | no (adapter only) | none; state is telemetry | remote's capability grant + operator approval |
| **Supervisor** | The single internal lane that leases, probes, reconnects, quarantines and reconciles remotes/devices | new `remote_supervisor_state`/`leases`; plus audit | no | no | none | system; cannot approve actions |

Current tree has only the first object. The daemon object is designed in `21-roles-daemons-implementation.md`
(record not task; §3.7.1 rejects a scheduled Task); the remote/device/supervisor objects are new in this folder.
The frozen node vocabulary is `["hyperagent","superagent","worker","tool"]` and `error` is client-only
(`backend/crates/hiverouter-contract/src/event.rs` `NODE_KIND_VOCABULARY`; `NODE_SPAWN` never carries a remote or
daemon kind) **[V]**. Remote/device/daemon cards therefore use **separate card kinds and events**, not
`NODE_*` (`21` §3.8.2/§4.6; `04`).

## 2. Tree-free, explained against the operator's sentence

Operator: "Remote agents may sit under HyperAgent or a SuperAgent" (`OPERATOR-DECISIONS.md:39-40`). Task
constraint: remote agents are **tree-free**. Both are satisfied by separating *ownership* from *tree position*:

- `remote_agents.owner_kind ∈ {operator, hive, office}`; `owner_id` is a reference (`hive:<super_task_id>`,
  `office:<office_id>`, or `operator`). It grants authority scope and UI grouping.
- There is **no** `parent_task_id`/`parent_remote_id`, **no** edge in `GraphAuthority`, **no** `tasks` row, and
  **no** `NODE_SPAWN`/`NODE_STATE` for a remote. `MAX_AGENTS=64`, `MAX_FANOUT=8`, `MAX_SPAWN_DEPTH=3`,
  `MAX_CALLS=40` (`backend/crates/hiverouter-core/src/caps.rs:9-15`) are not consumed by remotes **[V]**.
- The canvas/hierarchy ("Hive") shows only tasks. The **Remote** lane/roster shows remotes, devices, and their
  owner; selecting an owner in the tree filters the Remote lane, not the reverse.
- If an operator literally wants a visible tree child, that is a **UI grouping** in the roster projection — never a
  synthetic task row. A test asserts `tasks_for_run` never contains a remote id and `NODE_*` never emits a remote
  kind.

Rationale: a remote is not part of a run's success/failure; it outlives runs; it has no depth/fan-out/no task
state machine; attaching it as a task would consume caps, pollute history/replay, and violate the current
`tasks.run_id NOT NULL`/single-active-run engine (`execution.rs:1058-1074`; `db.rs:63-74`) **[V]**. This mirrors
`21`'s rejection of "daemon = scheduled Task" for the same structural reasons (21 §3.7.1).

## 3. Authority model

```
operator (sole root authority)
  ├─ owns/enrols/revokes remote agents
  ├─ grants capability ACLs (remote × device × capability × action × constraints)
  ├─ approves every risk-gated device action (or explicitly delegates a narrow auto policy)
  └─ owns the global/system memory bank (read-only to agents/remotes)
        │
        ▼
owner reference (hive/office/operator) ──► remote agent ──► devices/capabilities
        │                                      │
        │                                      ├─ reads: system bank (RO) + private remote bank
        │                                      ├─ writes: private remote bank (quarantined)
        │                                      └─ executes: only granted device actions, after approval policy
        ▼
supervisor lane (system) — lease/probe/reconcile/quarantine; never approves, never actuates
```

Rules (all testable):

1. **Operator is the only authority that can create a grant or approve an action.** A remote cannot self-grant,
   cannot grant another remote, cannot spawn tasks, cannot control its owner, and cannot read another remote's
   memory or devices. `E_REMOTE_SELF_CONTROL`/`E_REMOTE_CAPABILITY` refusals.
2. **Capability is a grant, not a class.** Effective capacity = `remote's enrolled capability set ∩ operator grant
   ∩ device's declared capability ∩ policy mode`, minus revocations. Empty/unknown ⇒ refuse (fail-closed).
3. **Attenuation only.** A grant cannot exceed the remote's enrolled capabilities or the device class's declared
   actions; re-granting cannot widen an existing grant without an explicit operator edit + audit row.
4. **No transitive authority.** Remote A cannot proxy through Remote B or through a task; a task can call a remote
   only via the approved `remote_call` tool (Delta R2) and the action still carries its own approval.
5. **Authority expires.** Grants have `expires_at` and `rev(:revocation)`; revocation is immediate at the next
   action/lease check; in-flight actions are cancelled if the operator revokes (or complete if already at the
   device and confirmation is pending — recorded, never silently).
6. **The supervisor is a system actor, not an authority.** It may mark a remote quarantined/offline, cancel
   connection leases, and refuse actions, but it cannot approve, grant, or write memory content.
7. **Humans-only boundary.** Retention/visibility/blindness settings (operator decisions) are never writable by
   remotes or agents; system bank writes are operator-only and audited.

## 4. Lifecycle

### 4.1 Remote lifecycle states (separate from `AgentState` = 16)

| State | Meaning | Entered by | UI |
|---|---|---|---|
| `enrolled` | record exists; transport not yet verified | operator create | grey, "not connected" |
| `online` | latest heartbeat/state within lease | supervisor probe/handshake | verified green + `last_seen` |
| `degraded` | reachable but failing checks or stale device state | supervisor check | amber + failing check count |
| `offline` | lease expired / transport failed threshold | supervisor | grey "offline since T" |
| `quarantined` | repeated failures/restart limit or unsafe protocol | supervisor | red, actions refused |
| `revoked` | operator revoked; identity may not reconnect | operator | red, terminal; card retained for audit |
| `retired` | operator deleted after revoke; no further events | operator | hidden by default |

Transition rules: `enrolled → online|offline|revoked`; `online ↔ degraded`; `online|degraded → offline` on lease/
probe failure; `offline → online` on valid heartbeat; any non-terminal → `quarantined` on restart/protocol limit;
any → `revoked` by operator; `revoked → retired` after retention. No transition to a task state; the UI must not
paint these on the agent-state shape.

### 4.2 Enrolment (operator, local-first)

1. Operator chooses `transport_kind`:
   - `in_process` — a built-in adapter (e.g., a test/fixture adapter, or a bundled connector) — Phase R1.
   - `lan_http` — adapter runs locally and calls a known endpoint (Home Assistant REST, MQTT/TLS, explicit HTTP)
     — Phase R1–R2. The HiveMind server remains loopback-only.
   - `outbound_ws` — a remote node connects **out** to HiveMind with mTLS/token; no inbound listener — Phase R5.
2. Operator supplies display name, owner, endpoint reference, credential **reference** (OS credential store key),
   adapter kind, and the declared capability/device inventory. Credentials are never entered into the DB, the
   browser's local storage, prompts, logs or `SETTINGS_STATUS`.
3. Supervisor verifies: TLS/identity (or loopback policy), protocol/card version, device inventory shape, and a
   read-only probe. Failure ⇒ `enrolled` + named refusal; success ⇒ `online` with `lease_expires_at`.
4. Operator grants capabilities (least privilege), with optional constraints (rate, time window, device subset,
   max duration). No grant is auto-created.
5. Any change bumps `remote_agents.rev` and writes `remote_audit` (actor, action, old/new hashes). A listing route
   exposes the record but never the credential.

### 4.3 Transport contract (no inbound exposure in v1)

- Browser/UI ↔ HiveMind: existing loopback HTTP + WS with Origin allowlist + per-session token
  (`http.rs:562-600` host guard; `ws.rs:362-383` Origin + token) **[V]**. Remote transports must **not** reuse
  `state.session.token()`.
- Local adapters: outbound HTTP only to an operator-configured base URL with an allowlist (`https` required except
  explicit loopback `http://127.0.0.1`/localhost with a warning), bounded timeouts, no redirect following to a
  different origin, no proxy inheritance, and no user-supplied arbitrary URL from model text.
- Remote node (R5): outbound WebSocket/HTTP with mTLS client cert or a per-remote bearer; rotating
  `remote_agents.rev`-bound token; replay-protected nonce/seq; declared protocol version (`remote.v1`); no raw
  browser token, no shared operator token. A2A external interop is optional and only through this transport
  (A2A's agent-card discovery/task lifecycle are the relevant external precedent; internal verbs remain HiveMind's
  own — see `SOURCES.md`).
- Protocol messages are bounded JSON (device inventory/state/action/ack/report); size caps and schema validation
  on both ends. Unknown fields ignored; unknown types refused by name.

## 5. Supervisor, watchdog, log, crash map

### 5.1 Three lanes (not a scheduler fork)

| Lane | Owns | Tick/scope | Cannot |
|---|---|---|---|
| **Run engine** (existing) | tasks, plan, provider calls, approvals inside a run | event/stream driven | touch remotes/devices |
| **Daemon scheduler** (21, v13+ after re-plan) | deterministic daemon jobs (watch/control checks), reports | timer (heartbeat/report cadence, grace) | run a model, use tools, call a device directly |
| **Remote supervisor** (this design) | leases, probes, reconnects, quarantine, action dispatch to adapters | one single-flight loop, interval-derived due times, backoff | approve, grant, spawn tasks, model calls |

One process-wide supervisor task with a durable `next_due_at`; no per-remote OS thread/actor. The bus is used only
for UI events; the supervisor never blocks the run engine (separate locks, no `await` under a store lock).

### 5.2 Watchdog checks (extend 21's daemon kinds; deterministic only)

| `check.kind` | Target | Observation | Report/transition |
|---|---|---|---|
| `remote_health` | remote | last heartbeat/lease, transport errors, quarantine flag | `online/degraded/offline/quarantined` |
| `device_reachability` | device | adapter probe/state read freshness | `available/unavailable/stale` |
| `device_state_drift` | device | declared vs expected postcondition for a completed action | drift report; does not retry actuation |
| `action_backlog` | remote/device | queued/awaiting approvals/in-flight age | stuck report; operator notice |
| `audit_health` | server | append failures/disk bound | report; no silent truncation |

Watchdog daemons never call a device directly: they observe the supervisor/adapters' recorded state. A daemon that
needs to act is out of v1 and requires the action path + approvals (21 §3.7.1's rule; `27`'s "monitor-and-act"
agents are a P3 donor pattern only).

### 5.3 Logging

- Append-only `remote_audit` (DB) + the existing local JSONL audit sink (`tools/src/audit.rs` §V) for
  process-level `RemoteSupervisor` events. Fields: `ts, actor_kind/actor_id, remote_id, device_id?, action,
  capability, request_id, idempotency_key, outcome, refusal_code, params_sha256, state_before/after_hash`.
- Never store credentials, tokens, raw device URLs with embedded secrets, or full device payloads (bounded
  summaries/hashes). UI shows an audit drawer with copyable request ids, not secrets.
- Every refusal has a code + one-line remediation; every action has one audit row at request and one at result.

### 5.4 Crash / restart map (must be deterministic)

| Failure | Detection | Recovery on boot | Persistent evidence | UI |
|---|---|---|---|---|
| Server dies with remote `online` | supervisor lease expiry after restart | `online→unknown`, re-probe; never assume still online | `remote_audit`, `remote_supervisor_state` | "Reconnecting / last seen T" |
| In-flight device action | action row `executing` with started_at | mark `interrupted`; do **not** auto-retry; reconcile via state read only | `device_actions`, audit | "unknown outcome; check device" |
| Pending device approval | approval TTL/`expires_at` | expire; notify; no execution | `device_actions`/approval row | expired card, no action |
| Supervisor crash mid-probe | lease/next_due persisted | recompute due from persisted cursor; at-most-once probe per interval | supervisor state | no duplicate reports |
| Adapter process crash | probe failure | restart intensity 3/3600; after limit quarantine; no actuation retry | audit | degraded→quarantined |
| DB busy/lock | write error | retry bounded; action stays pending; never ACK without write | audit | named refusal `E_DB_BUSY` |
| Duplicate request after reconnect | same idempotency key | return recorded outcome/refusal; never re-actuate | `device_actions` unique key | same result shown |
| Revocation during action | rev check before dispatch/after response | cancel before dispatch; after dispatch record outcome + flag; no further actions | audit | revoked, "action may have reached device" |

**Invariant:** a write/actuation is **never replayed automatically**. Safe reads (state/probe) may be retried with
backoff; an operator-originated act requires a new request + approval. This is stricter than "retry at the
transport layer" and is the only acceptable rule for garage/lock classes.

### 5.5 Interaction with the run engine (current facts)

- The engine is single-active-run and serial dispatch (`submit_prompt` refuses a second active run,
  `execution.rs:1058-1074`) **[V]**; remotes must not create runs or call `submit_prompt`.
- `GraphAuthority` is the task-tree authority; remote ownership is a separate table, not a graph node
  (`graph.rs`; no remote in `GraphAuthority`) **[V]**.
- Recovery (`recovery.rs:74-315`) settles non-terminal runs/tasks and emits catch-up frames; it must also call the
  remote reconciliation hook (`recover_remote_on_boot`) after the DB is available, before the bus serves UI. The
  hook is additive and idempotent; a second boot finds no in-flight action to transition (test).
- The current `execution.rs` dirty diff adds a HyperAgent root turn; it does not change the remote model, but the
  roster/capacity surfaces now expose more task kinds. Remote cards remain separate.

## 6. Invariants (R-series)

| # | Invariant |
|---|---|
| R-1 | A remote has no `tasks` row, no `parent_task_id`, no graph edge, and consumes no run/agent/call cap. |
| R-2 | A remote action is executable only if a live grant covers `(remote, device, capability, action)` and the policy layer approves it. |
| R-3 | No remote can read another remote's memory/device list, spawn a task, or control its owner; refusal is named. |
| R-4 | Every state transition/action attempt writes an audit row; no silent success, no silent failure. |
| R-5 | Device values have a provenance timestamp; stale/unknown never renders as current. |
| R-6 | Actuation is at-most-once per idempotency key; crash/reconnect never auto-replays it. |
| R-7 | Daemons/watchdogs are records, not tasks; they cannot call a device directly. |
| R-8 | One supervisor lane; no per-remote thread/actor; single-flight and bounded backoff. |
| R-9 | Credentials exist only in the OS credential store and are referenced by id; never in DB/UI/log/prompt. |
| R-10 | Remote memory is isolated by bank scope; system/global is read-only; grants are capped/TTL and audited. |
| R-11 | Local-first: no inbound remote listener in v1; remote transport is outbound and uses its own credential. |
| R-12 | Every new command/event/refusal has a matrix row, schema, ACK path and named-refusal test before it ships (UI-COMMANDS §7). |

## 7. Non-goals (v1)

- A remote agent running a HiveMind task/provider loop, consuming `MAX_CALLS`/run budget, or appearing in the
  canvas hierarchy.
- Model-in-the-daemon check loop; event-triggered daemon kinds; arbitrary script/flow execution (`27`: Node-RED
  function nodes are a rejected anti-pattern).
- Inbound network exposure of the HiveMind server; browser-token reuse for remote transport; cloud discovery.
- Automatic actuation of garage/lock classes; bulk/batch device control; auto-retry of actuation.
- External A2A wire interop or MCP server hosting (patterns only; internal contract first).
