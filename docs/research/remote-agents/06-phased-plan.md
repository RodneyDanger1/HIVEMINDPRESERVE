# 06 — Phased plan, gates, and rollback

**Status:** DESIGN (D). This plan is deliberately **local-first and operator-first**: a working light switch and
honest state are worth more than a network agent protocol. No phase starts before its gate; no phase adds a second
scheduler/model loop.

## 1. Phase map

| Phase | Outcome | Depends on | Effort | Migration |
|---|---|---|---|---|
| **R0 — Freeze & decisions** | Signed decisions U-R1…U-R9, migration reservation (v12 remote, v13 daemons, v14 memory), contract delta approval, fixture list | task-12 docs + Lead | S | none |
| **R1 — Operator device control (local)** | Remote record + device registry + grants + operator light/switch actions + state/audit + honest UI; Home Assistant REST adapter; no agent invocation | R0 | M | v12 |
| **R2 — Supervisor & watchdog** | Single supervisor lane; leases/probes/quarantine/backoff; daemon/watchdog cards integrated with 21's daemon record (v13); reports/transitions; crash recovery | R1 + v13 daemon migration (renumbered) | M | v13 |
| **R3 — Universal memory isolation** | global/system bank read-only; per-remote private bank; grants; prompt assembly/cache; quarantine/purge | R1 + v14 memory-bank migration (renumbered from 22's v13) | M–L | v14 |
| **R4 — Agent-originated remote calls** | `remote_call` tool (Delta R2) with grant + per-action approval binding a non-task principal/run context; no tree node | R3 + approval bridge extension | M | none |
| **R5 — Network remote + sensitive devices** | outbound WebSocket/mTLS remote node; optional A2A card interop; camera snapshots; garage/lock controls (lock only if security review passes) | R2/R3 + security review | L | none/new fields |
| **R6 — Fleet/hardening** | multi-remote scale, restart drills, audit retention, restore/backup, operator runbook | R5 | M | possible |

Parenthetical: R2/R3 depend on the Lead's migration decision because wave-4 docs planned v11–v17 differently than
the committed tree (current v11 = `agent_memory_scoped`). R1's v12 can ship alone if the control plane is accepted;
R2/R3 are not blockers for a light switch demo but are blockers for "universal memory" and "watchdog" claims.

## 2. Phase detail

### R0 — Freeze & decisions (no product code)

Deliverables: signed decision record for U-R1…U-R9; migration reservation table; contract delta R1 (9 commands, 6
events, 26 codes) accepted by contract owner; fixture list (Home Assistant mock, MQTT mock, remote node fixture,
crash harness); threat review of device classes; a `CHANGE-LEDGER` row plan.
Gate G-R0: all four decisions signed; no open blocker on v12 numbering; security owner accepts R2/R3 device rules.
Rollback: nothing to roll back.

### R1 — Operator device control (local; v12)

Scope: `remote_agents`, `devices`, `remote_grants`, `device_actions`, `remote_audit`, supervisor cursor; Home
Assistant REST adapter for `light/switch` and read-only state; camera device cards disabled/read-only (no snapshot
yet); no memory bank (system facts from settings only); no network remote.
Deliverables: v12 migration + down; 5 WS commands (`REMOTE_CREATE/UPDATE/ENABLE/REVOKE/GRANT`,
`GRANT_REVOKE`, `DEVICE_ACTION/RESOLVE/CANCEL` — 9 total), 3 events (`REMOTE_STATE`, `DEVICE_STATE`,
`DEVICE_ACTION`), approval event variant; 8 read routes; adapter trait + HA adapter; Remote lane UI with light/switch
cards, grants editor, audit drawer, approval variant; docs: operator runbook for adding an HA token.
Tests: `04` §10 tests 1–11, `15`, `17`, `remote_no_task_rows_or_events`, plus HA fixture calls
(`home_assistant_light_on_uses_service_call`, `home_assistant_state_read_normalized`,
`home_assistant_token_missing_named_refusal`).
Gate G-R1: a fake HA fixture toggles a light through the UI; state confirmed; unknown outcome does not move the
switch; crash after dispatch marks interrupted and does not replay; zero task rows; audit complete; Host/Origin/
token tests pass.
Rollback: feature flag `remote_control.enabled=false` hides the lane and refuses WS commands with
`E_COMMAND_UNKNOWN`; v12 rows remain inert; migration down exists for pre-data revert only.

### R2 — Supervisor + watchdog (v13 daemons renumbered)

Scope: supervisor lane (single-flight, leases, restart intensity 3/3600, quarantine, backoff, boot reconciliation);
watchdog daemon kinds `remote_health|device_reachability|device_state_drift|action_backlog`; daemon card/panel
integration; remote reports in UI; audit retention.
Deliverables: v13 daemons migration (21's v15 content adapted; record-not-task); supervisor module; watchdog checks;
`REMOTE_REPORT`/`SUPERVISOR_STATE` events; crash-drill harness; tests 12/13/15/17 + `supervisor_single_lane_backoff`,
`watchdog_transition_only_report`, `remote_crash_no_replay`.
Gate G-R2: kill the server mid-action → on boot exactly one interrupted action, no duplicate event, device state
reconciled; restart intensity quarantines a flapping adapter; watchdog reports only transitions; no second scheduler.
Rollback: disable supervisor lane + daemon feature flag; v13 rows inert; no data loss (append-only audit remains).

### R3 — Universal memory isolation (v14 banks renumbered)

Scope: `global` (system/universal, read-only) + `remote` bank; `memory_grants` point-to-point TTL; prompt assembly
for the remote connector (`global` + own remote bank only); quarantine/trust; retention/purge; UI memory inspector
for remotes; no task-bank access.
Deliverables: v14 migration (22 bank tree + `02` extension); store read/write with explicit bank allowlist; connector
`MEMORY_READ/WRITE` verbs; memory routes; audit `memory_context_snapshots`; tests in `02` §5 + `remote_prompt_uses_only_global_and_own`,
`remote_memory_cannot_poison_run`, `remote_memory_purge_receipt`.
Gate G-R3: a remote LLM/connector prompt contains only global+own blocks; a task cannot read remote memory; a remote
write never becomes authoritative without operator action; prompt hashes/pre-fix measurements recorded (`25` rule:
no cache claim without measurement); revocation purges content with receipt.
Rollback: disable remote memory (remote runs stateless); v14 down migration returns to v13; task banks remain.

### R4 — Agent-originated remote calls (Delta R2, no new migration)

Scope: `remote_call` tool (`PermissionClass::Execute` or new `Device` class; choose via contract review) for a task
to request a granted device action; the approval is bound to the task/tool-call context *and* the remote action
id/nonce; a successful call returns an observation (state/result), never writes remote memory; no tree node.
Deliverables: tool contract + schema; engine routing with approval; action ledger correlation; UI evidence row in
run detail + remote lane link; denial/refusal tests; rate/caps per run and per device.
Gate G-R4: a task cannot call a device without grant+approval; approval spoofing/args_sha256 mismatch refused; the
action appears once in run evidence and once in remote audit; kill/cancel ordering leaves no "executing" row.
Rollback: remove tool id/registration + contract delta; no schema change.

### R5 — Network remote + sensitive devices

Scope: outbound WebSocket/mTLS remote node (`remote.v1`), optional A2A agent-card interop; camera snapshot
(operator-only, consent/privacy); garage control (R2 class); door lock only if the security review signs the
device-class extension.
Deliverables: remote node fixture + transport docs; cert/token provisioning; frame schema/version; camera adapter;
garage adapter with state confirmation; UI camera drawer/garage two-step; tests `remote_protocol_pin`,
`outbound_only_no_listener`, `camera_snapshot_bounded`, `garage_prevents_stale_open`,
`garage_never_auto_retries`, `lock_actuation_disabled_v1`.
Gate G-R5: remote node reconnects after network flap without replay; no inbound port; camera image bounded/metadata
stripped; garage open blocked on stale state and confirmed after approval; audit has no credential; external A2A
interop is optional and cannot alter internal verbs.
Rollback: disable `outbound_ws` transport flag; remote nodes rejected with `E_REMOTE_TRANSPORT`; local adapters
keep working.

### R6 — Fleet/hardening

Scope: multiple remotes/devices, 150-device load profile, backup/restore of remote records/audit, retention
policies, operator runbook, incident response (revoked device, compromised token), UI density/LOD for Remote lane,
a11y audit.
Tests: load with 150 devices/fixture adapters (no device calls beyond fixtures); restore drill; token rotation;
audit growth bound; UI performance (≤120 mounted cards, no unbounded live regions); crash matrix.
Gate G-R6: documented runbook; measured load; no unbounded queues; restore verified; a11y pass; sign-off.

## 3. Acceptance suite index (deterministic; no real device/cloud/paid model)

| # | Test | Phase |
|---|---|---|
| 1 | `ui_command_matrix_remote_rows` | R1 |
| 2 | `golden_schemas_remote_delta` | R1 |
| 3 | `remote_create_ack_and_refusals` | R1 |
| 4 | `remote_revoke_blocks_actions` | R1 |
| 5 | `grant_covers_exact_action` | R1 |
| 6 | `home_assistant_light_turn_on_confirmed` | R1 |
| 7 | `device_action_idempotent_replay` | R1 |
| 8 | `device_action_requires_approval_critical` | R1/R5 |
| 9 | `device_action_stale_state_refused` | R1 |
| 10 | `device_action_single_flight` | R1 |
| 11 | `http_200_without_state_is_unknown` | R1 |
| 12 | `remote_boot_recovery_idempotent` | R2 |
| 13 | `supervisor_single_lane_backoff_quarantine` | R2 |
| 14 | `watchdog_transition_only_report` | R2 |
| 15 | `remote_no_task_rows_or_events` | R1 |
| 16 | `remote_memory_reads_only_global_and_own` | R3 |
| 17 | `task_cannot_read_remote_memory` | R3 |
| 18 | `remote_memory_cannot_poison_run` | R3 |
| 19 | `remote_call_requires_grant_and_approval` | R4 |
| 20 | `outbound_only_no_inbound_listener` | R5 |
| 21 | `garage_prevents_stale_open_and_never_auto_retries` | R5 |
| 22 | `camera_snapshot_bounded_and_consented` | R5 |
| 23 | `remote_audit_no_credentials` | R1…R5 |
| 24 | `no_optimistic_device_state` / `unknown_is_not_success` (UI) | R1/R5 |
| 25 | `remote_reconnect_no_actuation_replay` | R2/R5 |
| 26 | `restore_remote_records_audit` | R6 |

Commands: `cd backend; cargo test -p hiverouter-contract -p hiverouter-core`; `cd web; npm.cmd run typecheck && npm.cmd
test -- --run`; fixture binaries as dev-only features (never in release; no production test-flag branch).

## 4. Risk register and kill criteria

| Risk | Likelihood | Impact | Mitigation | Kill criterion |
|---|---|---|---|---|
| Scope creep into a full agent runtime/network mesh | High | High | R1 local operator only; no model loop; no second scheduler | R1 gate ships without any network/model loop |
| Confused deputy / cross-device control | Medium | High | per-device ACL, ownership check, no cross-remote grant | any test failure in `grant_binds_remote_and_device` blocks |
| Unsafe garage/lock actuation | Low | Critical | two-step approval + state confirmation + no auto-retry; lock disabled | any stale-state reachable ⇒ feature stays disabled |
| Credential leakage | Medium | High | OS store, no DB/UI/log/prompt, redaction tests | any DOM/audit credential test failure blocks |
| Prompt injection through device/memory text | Medium | High | data fence, no authority from text, injection fixtures | injection fixture affects grants/approvals ⇒ block |
| Crash replay/duplicate actuation | Medium | Critical | idempotency + interrupted state + boot reconciliation | replay drill produces two device calls ⇒ block |
| Migration/contract collision | High | Medium | R0 decision; v12 reservation; golden regen | no migration until Lead signs |
| UI lies (stale/online) | Medium | High | honest-state rules + DOM tests | any optimistic/current-stale test fails ⇒ block |
| Availability regression (supervisor blocking engine) | Low | High | separate lane, no locks across engine/DB writes, bounded probes | p95 run latency regression >5 % ⇒ fix/quarantine |

## 5. Rollback and release rules

- Every phase behind a feature flag (`remote_control.enabled`, `remote_supervisor.enabled`,
  `remote_memory.enabled`, `remote_network.enabled`); disabled flags must hide the UI and refuse commands with a
  named refusal, not crash.
- DB migrations: append-only; production rollback is restore-from-backup, not `down` (`20` §4.1 rule 7). `down`
  exists for tests/pre-data revert only.
- Contract: additive changes only; old UI ignores unknown events, refuses unknown commands with `E_COMMAND_UNKNOWN`
  (frozen behavior); no removed/renamed fields.
- Each phase has a **stop rule**: if the gate's crash/authority test fails, revert the phase commit and keep the
  prior phase; never "fix forward" by weakening the gate.

## 6. Effort and sequencing notes

- R1 is the largest single phase (schema + adapter + UI + approvals) but has no dependency on memory/daemon work;
  it is the recommended first demo and the smallest honest slice.
- R2/R3 can run in parallel only after R0 signs the migration split; they touch disjoint tables but share the
  supervisor/roster shape and the contract delta must not double-edit the migration chain.
- R4 should not start before R3; otherwise a task can call a device but the caller's memory context is undefined.
- R5 is the only phase that touches network and media; keep it behind security review and a local-only default.
- R6 is measured, not assumed: 150 devices with fixture adapters, no claims about cloud/production fleet.
