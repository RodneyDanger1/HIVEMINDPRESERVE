# 03 — Device capabilities: light switches, cameras, garage doors

**Status:** DESIGN (D), external facts R with citations in `SOURCES.md`. Depends on `04` (contracts) and
`01` (authority/state machine). Decisions D-R4, D-R5, D-R6.

## 1. Model: device → capability → action

A device is a registered endpoint with a **class** and a **closed, versioned capability inventory**. A capability
declares its actions, parameters, state properties, risk class, confirmation requirement and rate limit. An action
is a validated invocation of one capability; it is never an arbitrary URL, shell string, or free-form HTTP call.

The abstraction follows W3C WoT's interaction model — a Thing Description exposes **properties** (state),
**actions** (invocations) and **events**, with security configuration — which is exactly the split a remote/device
registry needs ([W3C-WOT] in `SOURCES.md`). HiveMind does not adopt the WoT wire format in v1; it adopts that
conceptual split and keeps its own bounded JSON schema.

```
device                       (registry row, operator-owned)
  class: light | switch | camera | cover/garage | lock | sensor | other
  remote_id                  (one owning remote)
  capabilities: [            (declared by adapter, validated on enrolment)
    {capability_id, actions[], properties[], risk, confirmation, rate_limit}
  ]
capability                    (stable id)
  e.g. "device.light.switch" / "device.camera.view" / "device.garage.door"
action                        (closed verb within a capability)
  e.g. turn_on | turn_off | toggle | snapshot | open | close
```

### 1.1 Capability grammar (v1)

`device.<class>.<verb>` where `<class>` is a stable class id and `<verb>` is a closed action family. Capability ids
are declared in code/registry, not invented by an adapter string. V1 inventory:

| Capability | Actions | Properties (state) | Risk | Confirmation |
|---|---|---|---|---|
| `device.light.switch` | `turn_on`, `turn_off`, `toggle` | `power: bool`, `brightness: int 0..100 or null`, `reachable: bool` | **low** (reversible, local) | state read (power matches expected; brightness optional) |
| `device.switch.toggle` | `turn_on`, `turn_off`, `toggle` | `power: bool`, `reachable: bool` | low | state read |
| `device.camera.view` | `snapshot` (read-only; no stream/PTZ in v1) | `reachable: bool`, `last_snapshot_at`, `privacy_mode: bool` | **high privacy** | no state write; authorization + audit; consent setting |
| `device.cover.garage` | `open`, `close` | `position: open|closed|opening|closing|unknown`, `obstructed: bool`, `reachable: bool` | **critical** | pre-state must be known; post-state confirmation; operator phrase + approval |
| `device.lock.door` | `lock`, `unlock` | `locked: bool`, `reachable: bool` | **critical** (not v1 unless explicitly approved) | approval + state confirmation; two-step |
| `device.sensor.read` | none (read-only) | class-specific numeric/enum | none | freshness only |

No generic `device.http`/`device.raw` action exists. An adapter that cannot express a device as one of the closed
capabilities is out of v1 (`E_DEVICE_CLASS_UNSUPPORTED`); this is the anti-pattern guard against a remote becoming
an open HTTP proxy (a "generic webhook" is a confused deputy waiting to happen).

### 1.2 Device inventory / class facts

Each device row has `device_id, remote_id, class, name, room, adapter_kind, adapter_ref (non-secret),
state_json, state_version, last_seen_at, capabilities_json, safety_class, created_at, revoked_at, rev`.
`state_json` is adapter-normalized, bounded (≤ 8 KiB) and schema-checked per class; raw vendor payloads are not
stored. State is **telemetry**, not memory; it does not enter the memory bank tree and is never rendered as an
instruction.

## 2. Capability ACL (fail-closed)

```
grant = (remote_id, device_id, capability_id, action, constraints, granted_by, granted_at, expires_at, revoked_at)
```

- **Default deny.** No grant ⇒ `E_REMOTE_CAPABILITY`; an empty/unknown capability list is never "all".
- **Per-device, per-action.** `toggle` and `turn_off` are different actions; a garage `open` grant does not imply
  `close` unless both are granted (and even then each call is separately approved/confirmed).
- **Constraints:** rate limit (e.g., ≤ 6 actions/min/device), allowed time window (optional), max `open` duration
  for cover/lock (auto-close is **not** executed in v1; the alert reports "left open" instead), device subset.
- **Attenuation:** operator may narrow but not widen; re-grant requires operator edit + audit; revocation is
  immediate.
- **Ownership:** grant must be for a device owned by the granted remote (`devices.remote_id`); no cross-remote
  device grants in v1. If a genuine shared device is needed later, a distinct `device_shares` design is required —
  do not bolt it onto this table.
- **Scope:** grants are installation-scoped, not run-scoped; a run/task can never widen them. A task can call a
  device only through the coordinated `remote_call` tool (Delta R2), and the per-action approval still applies.

## 3. Action contract

```json
{
  "schema": "hiverouter.remote.v1",
  "request_id": "uuid-v7",
  "remote_id": "uuid",
  "device_id": "uuid",
  "capability_id": "device.light.switch",
  "action": "turn_on",
  "params": {"brightness": 60},
  "expected_state": {"power": true},
  "idempotency_key": "uuid-v7-or-stable-op-id",
  "requested_by": {"kind": "operator|task|remote", "id": "..."},
  "approval_id": "uuid-or-null",
  "deadline_at": "RFC3339"
}
```

Server-side validation order (all before any adapter I/O):

1. Schema/limits: bounded ids, known capability/action, params schema (`additionalProperties:false`), request body
   ≤ 16 KiB.
2. Principal: remote exists, not `revoked|quarantined`, owner/actor allowed.
3. Grant: live, unexpired, covers remote × device × capability × action, constraints satisfied.
4. Device: exists, belongs to remote, not `revoked`; class compatible.
5. Risk/approval: according to §4; missing approval ⇒ `E_DEVICE_APPROVAL_REQUIRED` with a pending action id and
   nonce/TTL; never a silent hold.
6. Precondition: for classes requiring state confirmation, current state must be **fresh** (`now - last_seen_at ≤
   freshness window`) and in an allowed pre-state; unknown/stale ⇒ `E_DEVICE_STATE_UNKNOWN`/`E_DEVICE_STATE_STALE`.
7. Idempotency: unique `(remote_id, device_id, idempotency_key)`; same key returns the prior result, a conflicting
   payload returns `E_DEVICE_IDEMPOTENCY_CONFLICT`; a different key while an action is in flight ⇒
   `E_DEVICE_ACTION_IN_FLIGHT` (no parallel actuation of one device).
8. Rate limit/backlog: per-device and per-remote counters; `E_DEVICE_RATE_LIMIT`.
9. Dispatch: adapter call with bounded timeout; one attempt only for actuation (safe state read may poll).
10. Confirm: read state until expected postcondition or timeout; write `state_before/after` hashes, outcome,
    `E_DEVICE_ACTION_TIMEOUT`/`E_DEVICE_CONFIRM_REQUIRED`; no blind retry.

Statuses: `requested → awaiting_approval → approved|denied → executing → confirmed|failed|timeout|unknown|
cancelled|interrupted`. A `confirmed` action requires an observed postcondition; a transport 200 without state
confirmation is `unknown`, never `succeeded`.

## 4. Risk classes and approval policy

| Risk | Examples | Default policy | Confirm | Auto allowed? |
|---|---|---|---|---|
| **R0 read-only** | sensor read, camera snapshot (privacy) | grant + audit; camera view may require operator approval per consent setting | freshness | state reads yes; camera snapshot only per setting |
| **R1 reversible low** | light/switch turn_on/off/toggle | operator approval by default; explicit narrow grant may allow auto **only in autonomy modes ≥2 and with a light-switch allowlist** | state read | yes, opt-in per device/action, never bulk |
| **R2 critical** | garage open/close, door lock/unlock | **always operator approval**, independent of autonomy mode; confirm phrase for garage `open`; second step (typed phrase or hold-to-confirm) in UI | pre + post state | **never** in v1 |
| **R3 forbidden** | arbitrary HTTP/shell/URL, batch garage, disable safety, unlock-all | refuse `E_DEVICE_CLASS_UNSUPPORTED`/`E_REMOTE_CAPABILITY` | — | never |

Approval record binds: action id, request hash (`params_sha256`), device/remote, capability/action, pre-state hash,
expiry (≤ 120 s for light, ≤ 300 s for garage/lock), nonce, single-use. A decision does not execute by itself; the
supervisor performs one confirmed attempt after approval. Deny-first, Escape closes without deciding, no optimistic
switch movement. Reuse the existing approval UX pattern (`web/src/panels/ApprovalOverlay.tsx`) with a
`device_action` variant; do not overload the frozen run/task approval semantics (`04` §6).

## 5. Class playbooks

### 5.1 Light switch (the canonical R1)

- Grant: `device.light.switch` actions `turn_on|turn_off|toggle` on one light; optional rate 6/min.
- Read: `GET state` via adapter; UI switch is enabled only if `online` + grant + fresh state; switch shows
  `on/off` and `last seen`.
- Action: operator clicks; if approval policy requires, overlay shows device/room, action, current state,
  consequence ("turns the study light on"), TTL; approve → dispatch → poll state ≤ 10 s → updated switch with
  "confirmed at T" or "unknown outcome; check the light".
- If the state was stale/unknown, the UI disables the switch and offers **Refresh**; no "assume off" toggle.
- Auto (optional): only with an explicit grant constraint + autonomy mode ≥2 + a per-device allowlist; every auto
  action is still audited and rate-limited. Default off.

### 5.2 Security camera (read-only, privacy high)

- Capability `device.camera.view` action `snapshot`; no stream/PTZ/record in v1.
- The adapter fetches a single snapshot (Home Assistant camera proxy or equivalent documented endpoint), returns a
  bounded image (≤ 2 MiB, content-type checked, metadata stripped). The UI shows the image with capture time, never
  a live feed that could imply continuous surveillance.
- Operator approval/consent is a per-remote setting (privacy), audited; no agent can request a snapshot in v1 (only
  operator UI). Remote itself may check `reachable`/`privacy_mode` without image content.
- Store only hash/metadata, not image bytes, unless the operator explicitly saves an artifact; no face/analysis
  pipeline.

### 5.3 Garage door (R2, never auto)

Flow: `open` request with `pre_state ∈ {closed}` and fresh state; approval with typed/hold confirmation; dispatch;
poll `opening→open`; if `obstructed` or timeout, stop and report, never retry; if still `open` after the configured
max, emit a warning report (no auto-close actuation in v1); audit pre/post. `close` symmetric (`pre_state ∈
{open}`, confirm `closed`). Unknown pre-state ⇒ refuse (`E_DEVICE_STATE_UNKNOWN`); a stale state ⇒ refuse and ask
for refresh. Two garage actions cannot be in flight concurrently (per-device lock). Lock class follows the same
pattern, and is **out of v1** until the security review signs the lock threat model.

## 6. Adapters (v1)

| Adapter | Use | Contract | Security |
|---|---|---|---|
| `home_assistant_rest` | local HA instance (port 8123) | `GET /api/states/<entity_id>`, `POST /api/services/<domain>/<service>` with `{entity_id, ...}`, `Authorization: Bearer <long-lived token>`; documented status codes 200/201/400/401/404/405 ([HA-REST] in `SOURCES.md`) | endpoint allowlist; TLS for non-loopback; token from OS credential store; no redirects/proxy inheritance; bounded timeout |
| `mqtt_tls` | lights/sensors via a broker | TLS + per-device topic map; pub/sub with correlation id; no arbitrary topic strings from model | broker credentials in OS store; topic allowlist; QoS 1 + idempotency |
| `outbound_node` (R5) | another HiveMind/native remote node | outbound WebSocket, `remote.v1` frames, mTLS or per-remote bearer, seq/nonce | no inbound port; no browser token; protocol pin + schema validation |
| `fixture` | tests/dev only | deterministic fake device | never in release build; gate on a test-only feature, no production branch keying off a test flag |

Adapters implement one trait (design):

```rust
trait DeviceAdapter: Send + Sync {
    fn inventory(&self) -> Result<Vec<DeviceDescriptor>, RemoteError>;
    fn read_state(&self, device: &DeviceRef) -> Result<DeviceState, RemoteError>;
    fn execute(&self, action: &ValidatedAction) -> Result<AdapterAck, RemoteError>;
    fn probe(&self) -> Result<AdapterHealth, RemoteError>;
}
```

The trait returns **normalized** state/ack; the supervisor owns validation/approval/idempotency. An adapter cannot
approve, write memory, spawn tasks, or log secrets. A failed adapter is quarantined, not retried blindly.

## 7. Threats and refusals

| Threat | Control | Test |
|---|---|---|
| Confused deputy (remote A acts on B's device) | grant binds remote+device; device ownership checked; no cross-remote grants | `grant_binds_remote_and_device` |
| Command injection / arbitrary URL | closed capability/action/params; no raw HTTP action; endpoint allowlist | `unknown_action_and_url_refused` |
| Credential leakage | OS credential store reference only; redacted audit; no prompt/UI/log | `credentials_never_serialized` |
| SSRF/redirect | allowlist + no redirects + loopback policy + TLS | `adapter_refuses_redirect_and_private_scheme` |
| Stale state → unsafe actuation | freshness window + pre-state + post-confirmation | `garage_open_requires_fresh_closed_state` |
| Replay/duplicate | idempotency unique key + action status + nonce single-use | `duplicate_action_returns_prior_result` |
| Prompt injection via device name/state | data fence; no authority from text | injection fixture |
| Denial of service/rate | quotas + per-device lock + supervisor backoff | `rate_limit_and_single_flight` |
| Silent success | state confirmation required for R1/R2; transport 200 alone is `unknown` | `http_200_without_state_is_unknown` |
| Batch dangerous ops | no batch action; one device/action per request | `batch_garage_open_refused` |

Refusal codes (final naming in `04`): `E_DEVICE_UNKNOWN`, `E_DEVICE_UNAVAILABLE`, `E_DEVICE_CLASS_UNSUPPORTED`,
`E_DEVICE_ACTION_UNSUPPORTED`, `E_DEVICE_STATE_UNKNOWN`, `E_DEVICE_STATE_STALE`,
`E_DEVICE_APPROVAL_REQUIRED`, `E_DEVICE_CONFIRM_REQUIRED`, `E_DEVICE_RATE_LIMIT`,
`E_DEVICE_IDEMPOTENCY_CONFLICT`, `E_DEVICE_ACTION_IN_FLIGHT`, `E_DEVICE_ACTION_TIMEOUT`,
`E_REMOTE_OFFLINE`, `E_REMOTE_REVOKED`, `E_REMOTE_CAPABILITY`, `E_REMOTE_SELF_CONTROL`.

## 8. Non-goals

- Live camera streaming/recording, PTZ, facial analysis, or media storage in v1.
- Locks as a v1 control (read-only lock state is allowed; actuation waits for the security review).
- Autonomous garage/lock actions, batch actions, or auto-retry of any actuation.
- Arbitrary HTTP/webhook adapters, user scripts, shell actions, or generic "remote command" capability.
- Device control by an unapproved model tool: agent-originated device calls arrive only in Delta R2 through
  `remote_call` + per-action approval.
