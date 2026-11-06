# UI-COMMANDS-v1 — FROZEN CONTRACT (Phase 1 freeze)

Freeze date: 2026-11-06. Source: `blueprint/HIVEMIND-BLUEPRINT-v1.md` Phase 1 (MP-01/MP-02/MP-07, UI-ADD-14) +
`blueprint/UI-WIRING-GUIDE.md` S01-S28 / §2 wiring ladder. This document is the single vocabulary for backend, frontend and
FigCraft surface work. Changing a name here requires a `CHANGE-LEDGER-IMPL.md` row and re-running the command matrix test.

## 1. Envelope

```json
{"schema":"hiverouter.v1","type":"<COMMAND|EVENT>","name":"<STRING>","id":"<uuid-v7>","correlation_id":"<uuid-v7>",
 "run_id":"<uuid|null>","task_id":"<uuid|null>","seq":<u64|null>,"ts":"<RFC3339>","payload":{...}}
```

Rules: `id` unique; `correlation_id` stable across request/ACK/Error/event chain; mutating commands are idempotent by
`correlation_id` for 24 h; `seq` monotonic per connection (server events only); unknown `name` → `E_COMMAND_UNKNOWN`;
schema mismatch → `E_SCHEMA_INVALID`; refusal is a typed `Error` event, never a silent drop.

## 2. Commands (client → server) — frozen

| Name | Payload (required → optional) | Precondition | Success | Named refusals |
|---|---|---|---|---|
| `SUBMIT_PROMPT` | `text`, `project_id` | text 1..8000 chars, run not active | `ACK` + `RUN_CREATED` | `E_PROMPT_EMPTY`, `E_PROMPT_TOO_LONG`, `E_RUN_ACTIVE`, `E_KEY_MISSING` |
| `SPAWN_SUB_HIVE` | `parent_task_id`, `role` | depth < 3, fan-out < 8 | `ACK` + `NODE_SPAWN` | `E_SPAWN_DEPTH`, `E_FANOUT_LIMIT`, `E_TASK_NOT_FOUND`, `E_RUN_TERMINAL` |
| `REWIRE_TASK` | `task_id`, `new_parent_id`, `expected_rev` | graph acyclic, worker not streaming | `ACK` + `NODE_REWIRE` | `E_CYCLE`, `E_RESYNC_REQUIRED`, `E_REWIRE_IN_FLIGHT`, `E_TASK_NOT_FOUND` |
| `CANCEL_RUN` | `run_id` | run active | `ACK` + `RUN_STATUS(CANCELLED)` | `E_RUN_TERMINAL`, `E_RUN_NOT_FOUND` |
| `KILL_WORKER` | `task_id` | task running | `ACK` + `NODE_STATE(KILLED)` | `E_TASK_NOT_FOUND`, `E_TASK_TERMINAL`, `E_KILL_DENIED` |
| `CHAT_PIN` | `pinned` (bool) | client surface | `ACK` | `E_COMMAND_UNKNOWN` only |
| `TEMPLATE_INSERT` | `template_id`, `project_id` | template manifest row present, licence allowed | `ACK` + `NODE_SPAWN` set | `E_TEMPLATE_UNKNOWN`, `E_TEMPLATE_LICENSE_MISSING` (`E_TEMPLATE_LICENSE_MISSING` is emitted when the licence allowlist rejects the row), `E_TEMPLATE_REF_FORBIDDEN` |
| `HISTORY_REVISION_SET` | `run_id`, `revision` | run exists, revision <= latest | `ACK` + `HISTORY_SNAPSHOT` | `E_HISTORY_GAP`, `E_RUN_NOT_FOUND` |
| `VIEWPORT_SET` / `VIEWPORT_RECENTER` | `{x,y,zoom}` / `{}` | client-only, no server effect | n/a (client state) | `E_ZOOM_RANGE` (zoom outside 0.1..3.0) |
| `DIFF_PROPOSE` / `DIFF_APPLY` / `DIFF_REJECT` | decision-gated | code-diff spec not approved | **NOT FROZEN** — do not implement | `E_DECISION_REQUIRED` |

## 3. Events (server → client) — frozen

`ACK` (`command`, `correlation_id`) · `ERROR` (`code`, `message`, `how_to_fix`, `retryable`) ·
`SERVER_HELLO` (`last_seq`, `schema`) · `RESYNC_REQUIRED` (`expected_seq`, `last_seq`) ·
`RUN_CREATED` (`run_id`, `status`) · `RUN_STATUS` (`run_id`, `status`, `error_code?`) ·
`REPLY_DELTA` (`run_id`, `text_delta`) · `REPLY_DONE` (`run_id`, `usage`) ·
`NODE_SPAWN` / `NODE_DRAG` / `NODE_REWIRE` / `NODE_STATE` (`task_id`, `state`) ·
`RUN_METRICS` (`superagents`, `workers`, `green_pct`, `delta`) · `APPROVAL_REQUEST` / `APPROVAL_RESOLVED`
(`approval_id`, `tool`, `decision`) · `HISTORY_SNAPSHOT` (`run_id`, `revision`, `graph`) · `PING` / `PONG`.

## 4. Single agent-state enum (>=13; superset allowed)

`healthy, running, queued, failed, completed, cancelled, killed, planning, thinking, awaiting_approval, streaming,
reconnecting, replay, empty, loading`.
The PDF's four dots map: healthy→healthy, running→running, failed→failed, queued→queued; every other value renders with
shape/text cue as well as colour (`UI-WIRING-GUIDE R-U4`).

## 5. Refusal codes (Phase 1 table; UI must display code + how_to_fix)

Transport/UI: `E_ORIGIN_REJECTED`, `E_TOKEN_INVALID`, `E_RESYNC_REQUIRED`, `E_ZOOM_RANGE`, `E_HISTORY_GAP`, `E_DB_BUSY`,
`E_WS_DISCONNECTED`; command/schema: `E_COMMAND_UNKNOWN`, `E_SCHEMA_INVALID`, `E_PROMPT_EMPTY`, `E_PROMPT_TOO_LONG`,
`E_RUN_ACTIVE`, `E_KEY_MISSING`, `E_SPAWN_DEPTH`, `E_FANOUT_LIMIT`, `E_CYCLE`, `E_REWIRE_IN_FLIGHT`, `E_RUN_TERMINAL`,
`E_RUN_NOT_FOUND`, `E_TASK_NOT_FOUND`, `E_TASK_TERMINAL`, `E_KILL_DENIED`, `E_BUDGET_EXCEEDED`, `E_APPROVAL_REPLAY`,
`E_APPROVAL_TIMEOUT`, `E_DIFF_DECISION_REQUIRED`, `E_TEMPLATE_UNKNOWN`, `E_TEMPLATE_LICENSE_MISSING`,
`E_TEMPLATE_REF_FORBIDDEN`; tool/safety: `E_TOOL_NOT_FOUND`, `E_TOOL_SCHEMA`, `E_JAIL_ESCAPE`, `E_TOOL_TIMEOUT`,
`E_TOOL_CANCELLED`; provider: `E_PROVIDER_AUTH`, `E_PROVIDER_RATE_LIMIT`, `E_PROVIDER_TIMEOUT`, `E_PROVIDER_INCOMPLETE`;
build/toolchain: `E_TOOLCHAIN_MSRV`, `E_NODE_MISSING`. Codes are stable identifiers; messages may change.

## 6. Commands ↔ surface matrix (FigCraft assets must label these)

S03 composer→`SUBMIT_PROMPT`; S04→`SPAWN_SUB_HIVE`; S05→`REWIRE_TASK`; S06→`CANCEL_RUN`; S07→`KILL_WORKER`;
S10→`HISTORY_REVISION_SET`; S12→`TEMPLATE_INSERT`; S20→`VIEWPORT_SET`/`VIEWPORT_RECENTER`; S25→DIFF (decision-gated).
All other surfaces are read/derived. Every mutating control must show ACK or named refusal before mutating UI state.

## 7. Acceptance (Phase 1 gate)

- `ui_command_matrix` test: every frozen name has (a) payload schema, (b) ACK, (c) at least one named refusal path test.
- `schema_roundtrip`: serialize/deserialize every command/event fixture.
- No command ships without a matrix row; no surface ships without a row in `UI-WIRING-GUIDE` §1.
