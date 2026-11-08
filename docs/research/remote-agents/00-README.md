# Remote agents and daemon control plane — production design

**Folder:** `docs/research/remote-agents/`
**Task:** shared task-12 (wave 4). **Scope:** design only — no product code, no runtime, no server.
**Repository baseline:** `C:\Users\Riley\Desktop\hiverouter-v1`, HEAD `7610b86` ("feat: fix bee edges, add canvas
legend and hash routing") + one uncommitted `execution.rs` root-turn change (unverified). Frozen contract today:
`CommandName::ALL = 17` (14 frozen + 3 decision-gated), `EventName::ALL = 24`, 65 refusal codes
(`backend/crates/hiverouter-contract/tests/schema_roundtrip.rs:158-167`;
`web/tests/approval.state.test.ts:191`). DB chain ends at **v11 `agent_memory_scoped`**
(`backend/crates/hiverouter-core/src/db.rs:249-274`). **No remote-agent, device, or daemon entity exists in the
code** (grep over core/tools/contracts finds only `in-flight`, file `watcher.rs`; the only roster is the read-only
`GET /api/agents/runs/{run_id}/roster` at `backend/crates/hiverouter-core/src/http.rs:536`, handler `:1026`).

**Claim labels used in this folder**

| Tag | Meaning |
|---|---|
| **V** | Verified by reading the cited code/doc line in this pass. |
| **R** | Read in an official external page fetched in this pass (URL listed in `SOURCES.md`). |
| **D** | Design decision of this lane; not implemented. |
| **U** | Unverified / decision still open. |

## One-page verdict

1. **Remote agents are tree-free principals.** They are not tasks, not ranks, not daemon checks, and they never get a
   `parent_task_id`. `owner_kind`/`owner_id` (operator / hive / department) is an ownership reference for authority
   and UI grouping only — no canvas edge, no `tasks` row, no `MAX_AGENTS`/`MAX_CALLS` consumption. This is how the
   operator's "may sit under HyperAgent or SuperAgent" is honored without violating tree-free (D-R1).
2. **Daemons stay records (21's model).** A daemon is one immutable job + switch + cadence + reports; no task, no
   model, no tools, no memory bank, no budget. A remote agent is a capability principal. A watchdog daemon monitors a
   remote/device; a supervisor lane supervises connections/leases/actions. They are three different objects (D-R2).
3. **Universal memory is a policy, not a shared bucket.** The global/system bank is operator-curated and read-only
   to every remote and agent; each remote owns one private bank; task-tree banks (run/hive/branch/role) are invisible
   to remotes; an explicit, capped, TTL grant can open one entry. Remote writes are quarantined and never
   authoritative; system writes are operator-only and audited (D-R3; §`02-memory-isolation.md`).
4. **Devices are declared capabilities, not URLs.** A device is registered per remote with a class
   (`light|switch|camera|cover/garage|lock|sensor|other`) and a closed set of capabilities/actions. Every action is
   ACL-checked (`remote × device × capability × action`), risk-classed, approval-gated, idempotent, and confirmed by
   a state read where the class requires it. Light switches are low-risk reversible; cameras are read-only but
   privacy-sensitive; garage doors and locks are high-risk, always approval + state confirmation, never auto and
   never retried blind (D-R4; §`03-device-capabilities.md`).
5. **Local-first, outbound-only.** HiveMind keeps its loopback-only server; v1 connectors run in-process or on the
   local machine against known endpoints (Home Assistant REST, MQTT/TLS, explicit HTTP adapter). A network remote
   node (mTLS or token, outbound WebSocket) is Phase R5, not v1. The browser session token is never reused for a
   remote transport; credentials live in the OS credential store and never in DB, prompts, UI, or logs (D-R5).
6. **The UI is honest by construction.** Remote/device/watchdog cards live in a tree-free **Remote** lane, never on
   the canvas hierarchy. `unknown`, `offline`, `stale`, `revoked`, `quarantined` are first-class display states; a
   last-known value is labeled with its timestamp, never drawn as current; no optimistic switch movement (D-R6).
7. **Crash recovery never replays actuation.** Boot marks `online→unknown`, clears leases, expires pending device
   approvals, marks in-flight actions `interrupted/unknown`, then re-probes reachable remotes/devices. Only safe
   reads are retried automatically; a write/actuation requires a new operator-approved request with a new
   idempotency key (D-R7; §`01`, §`04`).
8. **Numbering is blocked by a real conflict.** 20/21/22 planned v11 `a2a_escalation`, v12 `settings`, v13 memory,
   v15 daemons; the committed tree already shipped **v11 `agent_memory_scoped`**. This design reserves **v12** for
   the remote control plane and treats daemon/memory versions as a Lead re-plan (D-R8; §`04` migration table).
   Nothing in this folder may be coded before that allocation is signed.
9. **Two contract deltas, one at a time.** Delta R1 (remote control plane): migration, 9 operator commands, 6
   events, ≈30 refusal codes (exact count reconciled with the contract owner), 8 read routes. Delta R2 (device
   classes/capabilities, agent-originated `remote_call` tool) only after R1 is accepted and the approval bridge can
   bind a non-task principal (D-R9).
10. **The design is testable without a device.** Wiremock/Home-Assistant fixture adapters, a fake remote node, a
    fake MQTT broker, and boot/crash drills give deterministic acceptance; no cloud account, no physical switch,
    no paid model call is needed for any acceptance test in this folder.

## Top decisions (short)

| ID | Decision |
|---|---|
| D-R1 | Tree-free remote principal: `owner_kind/owner_id`, no task row/edge; appears in roster/cards only |
| D-R2 | Three objects, not one: remote agent (principal), daemon/watchdog (record), supervisor (system lane) |
| D-R3 | Universal memory = read-only global/system bank + one private remote bank; no task-bank access; grants capped/TTL |
| D-R4 | Device ACL + risk class + approval + state confirmation + idempotency; garage/lock always human-approved |
| D-R5 | Local-first, outbound-only transport; credentials in OS store; no inbound exposure in v1 |
| D-R6 | Tree-free Remote UI lane; honest unknown/offline/stale states; no optimistic actuation |
| D-R7 | Crash recovery reconciles state, never replays actuation; safe reads only |
| D-R8 | Migration v12 for the remote control plane; daemon/memory chain re-planned by Lead (current v11 conflict) |
| D-R9 | Operator-only remote commands in Delta R1; agent-originated `remote_call` + approvals in Delta R2 |
| D-R10 | Watchdog/supervisor integrate with 21's daemon record/scheduler; no second scheduler, no model loop |

## Top risks (short)

| ID | Risk | Mitigation / test |
|---|---|---|
| X-R1 | Confused deputy: remote A acts on device B or a task's memory | Per-device ACL, `remote_id` binding, no wildcard, actor in audit; tests `remote_cannot_call_other_device`, `task_cannot_read_remote_memory` |
| X-R2 | Prompt injection through device/memory text | Device/memory text is data, fenced in prompt; no tool/approval authority from text; injection fixture |
| X-R3 | Unsafe garage/door action from stale state | State precondition + post-confirmation + two-step approval + timeout; no auto-retry; tests |
| X-R4 | Credential/SSRF leakage via connector URL/token | OS credential store, endpoint allowlist, TLS policy, no raw-HTTP proxy, redacted logs; tests |
| X-R5 | Duplicate/replayed actuation after crash/retry | Idempotency key + unique action row + state check; crash drill |
| X-R6 | Second scheduler/agent model creep | Daemon record model (21); supervisor single lane; no model/tools in daemon; tests |
| X-R7 | Memory cross-leak or authority injection | Bank isolation matrix, quarantine, trust, grants; tests |
| X-R8 | Contract/migration collision with wave-3 chain | v12 reservation + reproducible migration test; Lead decision gate |
| X-R9 | UI lies about online/on state | Freshness labels, unknown/offline states, no optimistic control; DOM tests |
| X-R10 | Scope creep to A2A/MCP wire interop | Design internal verbs first; external A2A card/transport optional Phase R5 only |

## File map

| File | Answers |
|---|---|
| `01-remote-agent-model.md` | Authority, tree-free entity, lifecycle/enrolment/transport, supervisor/watchdog/log/crash map, invariants |
| `02-memory-isolation.md` | Universal system/global + private remote memory, actor matrix, schema, prompt/cache, revocation, tests |
| `03-device-capabilities.md` | Device classes/capability grammar/ACL, action contract, light/camera/garage, adapters, security, tests |
| `04-contracts-api-ws-refusals.md` | Migrations (current v11 conflict), DDL, HTTP routes, WS commands/events, refusal codes, auth, crash contract |
| `05-ui-representation.md` | Tree-free Remote lane, cards, controls, approvals, watchdog panel, honest-state rules, a11y, tests |
| `06-phased-plan.md` | R0–R6 phases, deliverables, gates, acceptance suite, rollback, open decisions |
| `SOURCES.md` | Official external citations fetched in this pass + internal docs/code anchors |

## Read-first inputs actually read in this pass

`docs/contracts/ADAPTERS.md`; `docs/contracts/UI-COMMANDS-v1.md`; `docs/research/hmars/20-implementation-
architecture.md` (migration chain, P0/P1 packages); `21-roles-daemons-implementation.md` (ranks, daemon record,
supervisor/watchdog, refusals); `22-memory-bank-implementation.md` (bank scopes/read policy); `23-a2a-
implementation.md` (envelope/verbs/roster); `24-ui-implementation.md` (cards/roster surfaces); `27-open-source-
daemon-catalog.md` (supervision/watchdog/TV-remote donors, anti-patterns); `OPERATOR-DECISIONS.md`;
`09-synthesis-and-goal.md`; current `backend/crates/hiverouter-core/src/{execution,tool_loop,recovery,http,ws,store,
db}.rs` and `backend/crates/hiverouter-tools/src/tool.rs`; `backend/crates/hiverouter-contract/src/{command,event,
refusal}.rs`; `docs/ui/figcraft/HiveMind/SPEC/HIVEMIND-TARGET-SPEC.md`; and the official web pages in `SOURCES.md`.
