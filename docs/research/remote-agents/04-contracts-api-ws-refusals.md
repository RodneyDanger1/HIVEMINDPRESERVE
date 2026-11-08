# 04 — Contract, migrations, API/WS, refusals, crash contract

**Status:** DESIGN (D), current-contract facts V. **Nothing here may be coded before the migration allocation
(D-R8) and contract-delta approval (D-R9) are signed.** Current frozen contract: `CommandName::ALL = 17` (14 shipped
+ 3 decision-gated), `EventName::ALL = 24`, 65 refusal codes
(`backend/crates/hiverouter-contract/tests/schema_roundtrip.rs:158-167`; UI-COMMANDS-v1 §2/§7) **[V]**.

## 1. Baseline that constrains the design (V)

| Item | Current fact |
|---|---|
| Migrations | v1…v10 in `db.rs:32-248`, **v11 `agent_memory_scoped`** at `db.rs:249-274`; forward-only, exercised inverses |
| Run engine | single active run; `submit_prompt` refuses a second (`execution.rs:1058-1074`); tasks require `run_id NOT NULL` (`db.rs:63-74`) |
| Roster | read-only `GET /api/agents/runs/{run_id}/roster` exists (`http.rs:536`; handler `:1026`) returning task `kind/role/label/state/depth/rev` |
| Memory | `GET /api/agents/runs/{run_id}/memory` exists; v11 table keys `(run_id, scope_id, agent_key, key)` (`store.rs:720-900`) |
| Approvals | read `GET /api/approvals`; resolve is the frozen `APPROVAL_RESOLVE` (nonce/TTL, `UI-COMMANDS-v1:37`); no non-task approval type |
| Auth | loopback Host guard (`http.rs:553-600`) + Origin allowlist + per-session token for WS (`ws.rs:362-383`); no inbound remote listener |
| Tools | 9 tool ids incl. `file_write`, `shell_exec`; `PermissionClass::{Read,Mutate,Execute,Network}` (`tool.rs:30-56`, `:108-142`); no device/remote tool |
| Daemons/remotes | **no entity, table, route, event, or command** in the committed tree |

Consequences: remote/device commands are **additive**, cannot reuse `NODE_*` (frozen node vocabulary
`["hyperagent","superagent","worker","tool"]`; `error` client-only), cannot be tasks, and must not require a second
active run. The existing approval lane is run/task/tool-call bound; remote approvals need their own record/event
(frozen `APPROVAL_REQUEST` is not widened beyond its documented additive fields).

## 2. Migration chain — real conflict and proposed reservation

`20`/`21`/`22` planned v11 `a2a_escalation`, v12 `settings`, v13 `memory_banks`, v15 daemons. **The committed tree
already used v11 for `agent_memory_scoped`.** The wave-4 docs are therefore stale on numbering. Proposed
reservation (Lead decision, no code before it):

| Version | Owner/plan | Contents |
|---|---|---|
| **v12** | this design (remote control plane) | `remote_agents`, `remote_grants`, `devices`, `device_actions`, `remote_audit`, `remote_supervisor_state` (one migration; §3) |
| v13 | `21` daemons, renumbered | `daemons`, `daemon_runs`, `daemon_reports` (record-not-task model; `21` §4.1 adapted) |
| v14 | `22` memory banks, renumbered | bank tree + `02` remote/system extension (`memory_grants`) |
| v15+ | A2A/provider/cost re-plan | `20`/`23`/`25` chain adapted by Lead |

Why v12 first: the remote control plane can ship without the bank tree (learned remote memory is gated, `02` §1)
and without daemons (a supervisor is one loop, not a daemon record); it depends only on v11/current routes. If the
Lead instead wants memory/daemons first, remote becomes v14/v15 and the same delta order still applies. A single
migration reviewer owns v12–v15 jointly (`20` §4.1 rule 2). **No duplicate v12, no edited shipped migration.**

## 3. DDL (v12, proposed)

```sql
-- 1. Remote agent: tree-free principal. NO run_id, NO parent_task_id, NO FK to tasks.
CREATE TABLE remote_agents (
  remote_id         TEXT PRIMARY KEY NOT NULL,      -- uuid v7
  display_name      TEXT NOT NULL CHECK (length(display_name) BETWEEN 1 AND 120),
  owner_kind        TEXT NOT NULL CHECK (owner_kind IN ('operator','hive','office')),
  owner_id          TEXT NOT NULL,                  -- operator | hive:<task_id> | office:<office_id>
  transport_kind    TEXT NOT NULL CHECK (transport_kind IN ('in_process','lan_http','mqtt_tls','outbound_ws')),
  adapter_kind      TEXT NOT NULL,                  -- home_assistant_rest | mqtt_tls | fixture | remote_node
  endpoint_ref      TEXT NOT NULL,                  -- non-secret allowlisted endpoint/config ref
  credential_ref    TEXT,                           -- OS credential-store key; never the secret
  capabilities_json TEXT NOT NULL DEFAULT '[]',     -- declared inventory, validated on enrolment
  state             TEXT NOT NULL DEFAULT 'enrolled'
                    CHECK (state IN ('enrolled','online','degraded','offline','quarantined','revoked','retired')),
  generation        INTEGER NOT NULL DEFAULT 1,
  rev               INTEGER NOT NULL DEFAULT 0,     -- revocation/lease CAS
  restart_attempts  INTEGER NOT NULL DEFAULT 0,
  restart_window_at TEXT,
  last_seen_at      TEXT,
  lease_expires_at  TEXT,
  quarantined_at    TEXT,
  revoked_at        TEXT,
  created_by        TEXT NOT NULL,
  created_at        TEXT NOT NULL,
  CHECK (owner_kind = 'operator' OR owner_id LIKE 'hive:%' OR owner_id LIKE 'office:%')
);
CREATE INDEX idx_remote_state ON remote_agents(state, last_seen_at);
CREATE UNIQUE INDEX ux_remote_name ON remote_agents(display_name) WHERE revoked_at IS NULL;

-- 2. Capability grants: fail-closed, per device/action, TTL, attenuable.
CREATE TABLE remote_grants (
  grant_id        TEXT PRIMARY KEY NOT NULL,
  remote_id       TEXT NOT NULL REFERENCES remote_agents(remote_id),
  device_id       TEXT NOT NULL REFERENCES devices(device_id),
  capability_id   TEXT NOT NULL,
  action          TEXT NOT NULL,
  constraints_json TEXT NOT NULL DEFAULT '{}',     -- rate/window/device subset; validated
  granted_by      TEXT NOT NULL,                   -- operator only
  granted_at      TEXT NOT NULL,
  expires_at      TEXT,
  revoked_at      TEXT,
  rev             INTEGER NOT NULL DEFAULT 0
);
CREATE UNIQUE INDEX ux_remote_grant ON remote_grants(remote_id, device_id, capability_id, action)
WHERE revoked_at IS NULL;

-- 3. Device registry: one owning remote, class, normalized state (telemetry, not memory).
CREATE TABLE devices (
  device_id       TEXT PRIMARY KEY NOT NULL,       -- uuid v7
  remote_id       TEXT NOT NULL REFERENCES remote_agents(remote_id),
  class           TEXT NOT NULL CHECK (class IN ('light','switch','camera','cover','lock','sensor','other')),
  name            TEXT NOT NULL CHECK (length(name) BETWEEN 1 AND 120),
  room            TEXT,
  adapter_ref     TEXT NOT NULL,                   -- entity_id/topic/opaque non-secret id
  capabilities_json TEXT NOT NULL,
  state_json      TEXT NOT NULL DEFAULT '{}',      -- normalized, ≤ 8 KiB, schema by class
  state_version   INTEGER NOT NULL DEFAULT 0,
  last_seen_at    TEXT,
  safety_class    TEXT NOT NULL CHECK (safety_class IN ('read_only','reversible','critical')),
  revoked_at      TEXT,
  created_at      TEXT NOT NULL
);
CREATE INDEX idx_devices_remote ON devices(remote_id, class);
CREATE UNIQUE INDEX ux_device_remote_ref ON devices(remote_id, adapter_ref) WHERE revoked_at IS NULL;

-- 4. Device action: one row per request; at-most-once actuation by idempotency key.
CREATE TABLE device_actions (
  action_id       TEXT PRIMARY KEY NOT NULL,
  remote_id       TEXT NOT NULL REFERENCES remote_agents(remote_id),
  device_id       TEXT NOT NULL REFERENCES devices(device_id),
  capability_id   TEXT NOT NULL,
  action          TEXT NOT NULL,
  params_json     TEXT NOT NULL,
  params_sha256   TEXT NOT NULL,
  expected_json   TEXT,
  request_id      TEXT NOT NULL,
  idempotency_key TEXT NOT NULL,
  requested_by_kind TEXT NOT NULL CHECK (requested_by_kind IN ('operator','task','remote')),
  requested_by_id TEXT NOT NULL,
  approval_id     TEXT,                            -- remote approval row id, not the run approval
  approval_nonce  TEXT,
  approval_expires_at TEXT,
  status          TEXT NOT NULL
                  CHECK (status IN ('requested','awaiting_approval','approved','denied','executing',
                                    'confirmed','failed','timeout','unknown','cancelled','interrupted')),
  state_before_hash TEXT,
  state_after_hash  TEXT,
  error_code      TEXT,
  created_at      TEXT NOT NULL,
  started_at      TEXT,
  ended_at        TEXT,
  UNIQUE(remote_id, device_id, idempotency_key)
);
CREATE INDEX idx_device_actions_status ON device_actions(status, created_at);
CREATE INDEX idx_device_actions_device ON device_actions(device_id, created_at);

-- 5. Append-only remote audit; no secrets/payload bodies.
CREATE TABLE remote_audit (
  audit_id        TEXT PRIMARY KEY NOT NULL,
  at              TEXT NOT NULL,
  actor_kind      TEXT NOT NULL CHECK (actor_kind IN ('operator','system','remote')),
  actor_id        TEXT NOT NULL,
  remote_id       TEXT,
  device_id       TEXT,
  action          TEXT NOT NULL,
  request_id      TEXT,
  idempotency_key TEXT,
  params_sha256   TEXT,
  outcome         TEXT NOT NULL,
  refusal_code    TEXT,
  detail_json     TEXT NOT NULL DEFAULT '{}',      -- bounded, redacted
  created_at      TEXT NOT NULL
);
CREATE INDEX idx_remote_audit_remote ON remote_audit(remote_id, at);
CREATE INDEX idx_remote_audit_device ON remote_audit(device_id, at);

-- 6. Supervisor cursor/state (single lane; restart-safe; no per-remote thread state).
CREATE TABLE remote_supervisor_state (
  id              INTEGER PRIMARY KEY CHECK (id = 1),
  lane_started_at TEXT,
  last_tick_at    TEXT,
  next_due_at     TEXT,
  backoff_ms      INTEGER NOT NULL DEFAULT 0,
  last_error_code TEXT,
  updated_at      TEXT NOT NULL
);
```

Ordering note: `remote_grants` references `devices`, so create `devices` before `remote_grants` (the DDL order above
is illustrative; the migration writer must order FKs). Down migration drops in reverse dependency order and must
leave no orphan rows; `migration_remote_roundtrip_keeps_data` exercises it. `credential_ref` is a reference; a
secret is never a column.

## 4. HTTP read routes (Host-guarded, operator session)

Inserted beside the agent routes (`http.rs:517-545` pattern). All are read-only, `cache-control: no-store` where
state is live:

| Method | Route | Response |
|---|---|---|
| GET | `/api/remote` | `{remotes:[RemoteCard], count, supervisor:{state,next_due,backlog}}` |
| GET | `/api/remote/{remote_id}` | `{remote, grants:[…], devices:[DeviceCard], last_audit:[…]}` |
| GET | `/api/remote/{remote_id}/devices` | `{devices:[DeviceCard]}` |
| GET | `/api/devices/{device_id}` | `{device, state, state_version, observed_at, freshness, last_actions:[…]}` |
| GET | `/api/devices/{device_id}/actions?limit=1..100&before=` | paged action ledger |
| GET | `/api/remote/audit?remote_id=&device_id=&limit=1..200&before=` | paged redacted audit |
| GET | `/api/remote/grants?remote_id=` | live + revoked grants (revoked for audit) |
| GET | `/api/supervisor` | lane state/backlog/quarantine list; no credentials |

There is **no** HTTP write for a device action. Device actions are operator WS commands (§5) so ACK/refusal is one
lane and cannot be replayed by a web page. The one exception is credential installation, which must be a
loopback-only, token-required `POST /api/remote/{id}/credential` that writes the OS credential store and never
echoes the value (or preferably an out-of-band OS-store helper; decision U-R1).

## 5. WS commands (operator-only; Delta R1)

All are mutating, operator-only, idempotent by `correlation_id`, and follow the envelope (`UI-COMMANDS-v1` §1).
Every command below gets a `COMMAND_MATRIX` row, payload schema, ACK, and ≥1 named-refusal test before shipping
(`UI-COMMANDS-v1:95-99`).

| Command | Payload (required → optional) | Precondition | Success | Named refusals (minimum) |
|---|---|---|---|---|
| `REMOTE_CREATE` | `display_name, owner_kind, owner_id, adapter_kind, endpoint_ref, credential_ref?` | operator; endpoint allowlisted; `LAN_HTTP` verified or `enrolled` | ACK + `REMOTE_STATE(enrolled)` | `E_REMOTE_TRANSPORT`, `E_REMOTE_AUTH`, `E_REMOTE_ENDPOINT`, `E_SCHEMA_INVALID` |
| `REMOTE_UPDATE` | `remote_id, display_name?, owner_kind?, owner_id?` | exists, not retired | ACK + `REMOTE_STATE` | `E_REMOTE_UNKNOWN`, `E_REMOTE_REVOKED`, `E_SCHEMA_INVALID` |
| `REMOTE_ENABLE` | `remote_id, enabled, switch_rev` | exists; CAS on rev | ACK + `REMOTE_STATE` | `E_REMOTE_UNKNOWN`, `E_REMOTE_STALE_REV`, `E_REMOTE_QUARANTINED` |
| `REMOTE_REVOKE` | `remote_id, reason?` | exists | ACK + `REMOTE_STATE(revoked)` | `E_REMOTE_UNKNOWN`, `E_REMOTE_RETIRED` |
| `REMOTE_GRANT` | `remote_id, device_id, capability_id, action, constraints?, expires_at?` | remote online? no; device belongs to remote; capability declared | ACK + `REMOTE_GRANT_CHANGED` | `E_REMOTE_CAPABILITY`, `E_DEVICE_UNKNOWN`, `E_REMOTE_GRANT_INVALID`, `E_REMOTE_REVOKED` |
| `REMOTE_GRANT_REVOKE` | `grant_id, reason?` | live grant | ACK + `REMOTE_GRANT_CHANGED` | `E_REMOTE_GRANT_UNKNOWN` |
| `DEVICE_ACTION` | `device_id, capability_id, action, params?, expected_state?, idempotency_key?` | grant, device online, state fresh where required | ACK + `DEVICE_ACTION(requested/awaiting_approval)` | `E_REMOTE_OFFLINE`, `E_DEVICE_UNAVAILABLE`, `E_REMOTE_CAPABILITY`, `E_DEVICE_STATE_STALE`, `E_DEVICE_ACTION_IN_FLIGHT`, `E_DEVICE_RATE_LIMIT`, `E_DEVICE_APPROVAL_REQUIRED` (when policy holds) |
| `DEVICE_ACTION_RESOLVE` | `action_id, nonce, decision approve/deny, confirmation_text?` | awaiting approval, fresh nonce, TTL valid | ACK + `DEVICE_ACTION(executing/denied)` | `E_DEVICE_APPROVAL_NOT_FOUND`, `E_DEVICE_APPROVAL_REPLAY`, `E_DEVICE_APPROVAL_TIMEOUT`, `E_DEVICE_CONFIRM_REQUIRED`, `E_DEVICE_STATE_STALE` |
| `DEVICE_ACTION_CANCEL` | `action_id` | non-terminal | ACK + `DEVICE_ACTION(cancelled)` | `E_DEVICE_ACTION_TERMINAL`, `E_DEVICE_ACTION_IN_FLIGHT` (cancel only before dispatch) |

`E_DEVICE_APPROVAL_REQUIRED` is a *named pending state*, not a failure to execute: `DEVICE_ACTION` returns ACK with
`DEVICE_ACTION(status=awaiting_approval)` and emits `DEVICE_APPROVAL_REQUEST`; the UI shows the decision. The
existing run/task `APPROVAL_RESOLVE` is deliberately **not** reused (different scope/nonce table); the approval
mirror can share the UI component, not the wire command.

## 6. Events (Delta R1, +6 → 30 if accepted; version decision U-R2)

| Event | Payload (bounded) | Emitted when |
|---|---|---|
| `REMOTE_STATE` | `remote_id, state, last_seen_at, lease_expires_at?, reason_code?` | lifecycle transition or supervisor state change |
| `REMOTE_GRANT_CHANGED` | `remote_id, grant_id, device_id, capability_id, action, state: granted/revoked, expires_at?` | grant create/revoke |
| `REMOTE_REPORT` | `remote_id, kind: health/audit/drift, severity, summary≤2000, evidence_ref?` | watchdog/supervisor report (no model text, no secrets) |
| `DEVICE_STATE` | `device_id, remote_id, state{normalized}, state_version, observed_at, freshness: fresh/stale/unknown` | probe/state change/confirmation |
| `DEVICE_ACTION` | `action_id, device_id, status, error_code?, state_before_hash?, state_after_hash?, at` | every action transition |
| `DEVICE_APPROVAL_REQUEST` | `action_id, device_id, action, risk, params_preview≤512, expected_state_hash, nonce, expires_at` | policy requires approval |

`DEVICE_APPROVAL_REQUEST` carries a redacted preview only (same `MAX_ARGS_PREVIEW_CHARS=512` pattern as approval
args); never a credential/full payload. `parsed` fields validate against generated schemas; unknown event names are
ignored by old UI, unknown *payload fields* are ignored by additive rules (the existing decoder tolerance).

## 7. Refusal codes (Delta R1; ≈30 new, additive; exact count reconciled with the contract owner)

| Code | Meaning / trigger |
|---|---|
| `E_REMOTE_UNKNOWN` | remote id absent |
| `E_REMOTE_REVOKED` | revoked/quarantined principal |
| `E_REMOTE_RETIRED` | deleted remote |
| `E_REMOTE_OFFLINE` | no live lease at action |
| `E_REMOTE_CAPABILITY` | grant/declared capability missing or action not covered |
| `E_REMOTE_GRANT_UNKNOWN` | grant id absent |
| `E_REMOTE_GRANT_INVALID` | expired/revoked/malformed/over-broad grant |
| `E_REMOTE_SELF_CONTROL` | owner/self target forbidden |
| `E_REMOTE_ENDPOINT` | endpoint not allowlisted/invalid scheme |
| `E_REMOTE_TRANSPORT` | transport failure/version mismatch |
| `E_REMOTE_AUTH` | connector credential rejected/missing (no key echo) |
| `E_REMOTE_PROTOCOL` | malformed/oversize remote frame |
| `E_REMOTE_STALE_REV` | CAS rev mismatch (switch/update) |
| `E_REMOTE_QUARANTINED` | restart/protocol limit exceeded; operator retry needed |
| `E_REMOTE_RESTART_LIMIT` | restart intensity exhausted |
| `E_REMOTE_INTERRUPTED` | action/request was in flight when the server restarted; outcome unknown until reconciled; no auto-retry |
| `E_DEVICE_UNKNOWN` | device id absent |
| `E_DEVICE_UNAVAILABLE` | adapter probe says unreachable |
| `E_DEVICE_CLASS_UNSUPPORTED` | class/adapter not in v1 inventory |
| `E_DEVICE_ACTION_UNSUPPORTED` | action not in capability |
| `E_DEVICE_STATE_UNKNOWN` | pre-state unavailable |
| `E_DEVICE_STATE_STALE` | state older than class freshness window |
| `E_DEVICE_APPROVAL_NOT_FOUND` | action/approval id absent |
| `E_DEVICE_APPROVAL_REPLAY` | nonce already consumed |
| `E_DEVICE_APPROVAL_TIMEOUT` | TTL expired |
| `E_DEVICE_CONFIRM_REQUIRED` | missing/wrong confirmation phrase or second step |
| `E_DEVICE_RATE_LIMIT` | per-device/remote quota |
| `E_DEVICE_IDEMPOTENCY_CONFLICT` | same key, different request |
| `E_DEVICE_ACTION_IN_FLIGHT` | another action on the same device |
| `E_DEVICE_ACTION_TERMINAL` | cancel/resolve after terminal |
| `E_DEVICE_ACTION_TIMEOUT` | postcondition not observed in window |
| `E_MEMORY_SCOPE_DENIED`/`E_MEMORY_WRITE_DENIED` | remote memory isolation (`02`) |
| `E_AUDIT_UNAVAILABLE` | audit write failure; the action must not be ACKed |

Exact code count/new names require the contract owner to reconcile with the `20` delta-2 planned codes and the
65-code mirror; do not duplicate an existing code. Every code needs message + remediation + web mirror + refusal
parity test (`web/tests/refusalParity.test.ts` pattern). Existing `E_ORIGIN_REJECTED`, `E_TOKEN_INVALID`,
`E_DB_BUSY`, `E_INTERNAL` are reused where applicable.

**Contract process:** a `CHANGE-LEDGER` row (UI-COMMANDS preamble) + regenerated `contract-v1.json` golden +
`COMMAND_MATRIX` rows + web `KNOWN_REFUSAL_CODES` mirror + `contract.test.ts` counts. Because 6 events and 9
commands are added, decide whether the schema string stays `hiverouter.v1` (precedent: 1.2 added events) or becomes
`hiverouter.v2`; this is **U-R2**, owned by the Lead/contract seat.

## 8. State enums (separate from 16 `AgentState`)

- `RemoteState`: `enrolled|online|degraded|offline|quarantined|revoked|retired`
- `DeviceAvailability`: `unknown|available|unavailable|stale`
- `DeviceActionStatus`: `requested|awaiting_approval|approved|denied|executing|confirmed|failed|timeout|unknown|cancelled|interrupted`
- `RiskClass`: `read_only|reversible|critical`

The UI must not map these onto agent state glyphs/colours; it uses the remote/device card grammar (`05`). The
server never emits `NODE_*` for a remote/device.

## 9. Crash/recovery contract

On boot, after migrations and before the WS bus accepts clients:

1. `remote_agents.state = online|degraded → unknown` (not offline); clear `lease_expires_at`; `generation += 1`.
2. `device_actions.status ∈ {requested, awaiting_approval, approved, executing}` → `interrupted` with
   `error_code=E_REMOTE_INTERRUPTED` (§7; no auto-retry). The state is reconciled only by a safe device state read;
   if reconciliation cannot determine the outcome, the action stays `unknown`.
3. Expire device approvals whose TTL passed; emit `DEVICE_ACTION(interrupted/expired)` once.
4. Supervisor lane starts with `next_due_at=now`; probes remotes/devices; emits `REMOTE_STATE`/`DEVICE_STATE`.
5. A second boot is idempotent: no action transitioned twice, no duplicate audit rows (test
   `remote_boot_recovery_idempotent`).
6. A safe read may be retried by the supervisor; an actuation never is. An action that was `executing` remains
   `unknown` until a device state read reconciles it; the operator sees "unknown outcome" and may issue a new
   approved action.

## 10. Acceptance tests (contract/API/crash; no device, no paid call)

1. `ui_command_matrix_remote_rows` — 9 new commands have matrix row, payload schema, ACK, ≥1 named refusal.
2. `golden_schemas_remote_delta` — deliberate regeneration; old 17/24/65 fixtures still decode.
3. `schema_roundtrip_remote_fixtures` — every command/event sample serializes/deserializes.
4. `remote_create_ack_and_refusals` — local fixture adapter happy path + `E_REMOTE_ENDPOINT`/`E_REMOTE_AUTH`.
5. `remote_revoke_blocks_actions` — revoke then action ⇒ `E_REMOTE_REVOKED`, zero adapter calls.
6. `grant_covers_exact_action` — missing action/device mismatch refused `E_REMOTE_CAPABILITY`; live grant works.
7. `device_action_idempotent_replay` — same key returns prior result, no second adapter call; conflicting params ⇒
   `E_DEVICE_IDEMPOTENCY_CONFLICT`.
8. `device_action_requires_approval_critical` — garage request returns awaiting_approval + nonce; wrong confirmation
   `E_DEVICE_CONFIRM_REQUIRED`; correct approves one dispatch.
9. `device_action_stale_state_refused` — old `last_seen_at` ⇒ `E_DEVICE_STATE_STALE`, zero calls.
10. `device_action_single_flight` — two concurrent requests ⇒ one `E_DEVICE_ACTION_IN_FLIGHT`.
11. `device_action_no_retry_after_timeout` — adapter 200 but no confirmation ⇒ `unknown`, no automatic second
    dispatch; adapter called once.
12. `remote_routes_host_origin_token` — Host guard rejects foreign Host; read routes require the UI session;
    no credential ref/secret in JSON.
13. `remote_boot_recovery_idempotent` — in-flight action → interrupted once; second boot no duplicate event.
14. `remote_audit_append_only` — every command/refusal writes one audit row with request id; audit write failure
    refuses (no ACK).
15. `remote_no_task_rows_or_events` — `tasks_for_run` never contains a remote id; `NODE_*` decoders never accept a
    remote kind; `MAX_AGENTS` unaffected by N remotes.
16. `remote_memory_scope_refusals` — a remote adapter's `MEMORY_READ` other-bank request ⇒ `E_MEMORY_SCOPE_DENIED`.
17. `supervisor_single_lane_backoff` — 10 remotes, one lane; no parallel probe for one remote; backoff grows and
    caps; quarantine after restart limit.
18. `remote_schema_version_pin` — connector `remote.v1` mismatch refused `E_REMOTE_PROTOCOL`; unknown fields ignored.
19. `version_decision_gate` — if schema becomes v2, old UI fixtures still ignored/mirrored (tested at decision time).
20. `dirty_tree_anchor` — code anchors in this folder remain resolvable after the migration lands (line-reference
    drift test is a docs check, not code).

## 11. Open contract decisions (owned by Lead)

- **U-R1** credential installation path: OS-store helper vs loopback token POST (no secret echo).
- **U-R2** schema string: additive `hiverouter.v1` (1.2 precedent) vs `hiverouter.v2`.
- **U-R3** migration reservation: v12 remote / v13 daemons / v14 memory vs another order.
- **U-R4** whether `DEVICE_ACTION` should be an operator-only command or a tool (`remote_call`) in Delta R2 (this
  design: both phases, command first).
- **U-R5** one more named code vs reusing `E_DEVICE_ACTION_TIMEOUT` for interrupted actions.
