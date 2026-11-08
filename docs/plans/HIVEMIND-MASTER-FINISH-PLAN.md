# HiveMind Master Finish Plan

Baseline: HEAD `8a19d0b` (clean tree after local exclude decision). Offline gate at this SHA: `cargo test --workspace --locked` 516 passed / 0 failed / 2 ignored, web typecheck 0, web 189/189, build 0, CLI 0. Evidence: `docs/evidence/gate-8a19d0b5-offline.json`. Prior commits include `7ce2716`, `f482143`, `e8e67a4`. Offline only; no server/UI/provider run.
This plan is the merge point for the team reports in `docs/research/`, `docs/audit/`, `docs/ui/figcraft/`, and the independent analysis seats.

## Current verdict

HiveMind is a credible **single-run hierarchical HMAS**: typed planner DAG, spawn tiers, personas, tool loop, durable A2A rows, scoped memory rows, crash snapshots, DeepSeek client + refusal paths, CLI, and a spatial React UI. It is **not yet true HMARS**: the root synthesis turn is new and hierarchical-only; A2A is one-way; memory is rows, not banks; Remote/daemon agents do not exist; authority/security/scaling are incomplete; FigCraft is now mounted as static evidence but pixel/runtime fidelity is unverified.

## Delivered in this wave

| Change | Evidence |
|---|---|
| HyperAgent root final synthesis turn for hierarchical plans | `execution.rs` dispatch Phase 3; `execute_root_task_with_prompt`; synthesis system prompt in `provider_executor.rs`; no tools; bounded prompt; no late spawn |
| Snapshot carries `kind`/`role`/`label` | `graph.rs TaskNode` + `set_metadata`; execution/recovery populate; web replay already read them |
| Memory write/read isolation | each task writes its own bank id; children read parent bank; root prompt aggregates run memory; A2A/roster/memory prompt values sanitized |
| FigCraft representation surface | `web/src/figcraft/`, `FigCraftPanel.tsx`, `web/public/figcraft/` (21 SVG boards: 13 HiveMind are PROPOSED-ADDITION, 20 A/B artifacts watchdog-passed); mounted in Settings; Vite copies `/figcraft/*`; no built-artifact serving proof |
| UI graph/UX corrections | HiveField in viewport portal; replay fit; broken-parent edges; bee direction/dash/reduced motion; active-unknown capacity; skip target; hash routes; no duplicate owner spokes |
| Audits | memory/A2A findings + remote/daemon design + UI graph audit + goal gap plan + FigCraft coverage |

## Truth ledger

| Area | Status | Blocker |
|---|---|---|
| HyperAgent execution | PARTIAL | Root answer runs for multi-node hierarchical and flat plans (`has_aggregators \|\| nodes > 1`); single-node still skips. No root orchestration tools; one call is not reserved. |
| SuperAgent sub-orchestration | PARTIAL | Tier/persona/spawn exists; no owned branch sub-DAG or ownership switch |
| Worker/Subagent | PARTIAL | Subagent is a 6-tool profile on Worker authority; depth floor landed in `e8e67a4` (untested); no persisted class or wire kind/card/bank. |
| Memory banks | PARTIAL | Run+task scopes, parent recall, root aggregate; no bank tree/trust/retention/promotion |
| A2A | PARTIAL | Durable ask/send, roster, inbox; terminal recipients refused in `8a19d0b` (untested). No answer/wake/deadline/status, no continuation refresh, check-then-insert budget. |
| Remote/daemon | MISSING | Design only under `docs/research/remote-agents/`; no code/schema/API/UI |
| UI/FigCraft | PARTIAL | Gallery mounted; run detail/answer/remote/daemon/authority surfaces missing; pixel fidelity unverified |
| Provider | PARTIAL | Chat/stream/tools/models/balance; no context preflight, 402 identity gap, no Files/Search/etc |
| Recovery/logs | PARTIAL | Snapshots/recovery/audit; no durable event journal/resume/daemon watchdogs |
| Authority/security | PARTIAL | Host guard + approvals; no `/api/*` token auth, spawn approval, autonomy settings |
| Scale | MISSING | No measured scale, scheduler, admission, DB writer, or event coalescing; one active run with serial leaves and caps 64/40. |

## Top gaps, ordered

1. **A2A answer/wake.** Ask has no answer, wake, deadline, status, or continuation inbox refresh. Terminal-recipient refusal exists in `8a19d0b` (ask and send), but has no regression test; do not re-implement it.
2. **Bank-tree memory.** Current rows are parent-scoped; need explicit project/run/hive/branch/role banks, trust/retention, gated promotion, HyperAgent summary reads.
3. **Remote/daemon runtime.** Contract-first: tree-free principal, private+global memory, device ACL, supervisor/watchdog/log/crash map.
4. **Role truth.** Define Subagent tier, correct Super/Worker allowlists, ownership model, and UI labels; remove any overclaim.
5. **Authority/security.** Token-authenticated mutations, spawn approval, autonomy/approval settings, scoped clears, message ACL.
6. **Durability.** Durable event journal + crash-resume policy; daemon logs/watchdogs.
7. **Provider truth.** Context-window preflight, auth identity mapping, feature support matrix/refusals.
8. **Scale.** Admission scheduler, DB writer, event coalescing/LOD; honest capacity claims.
9. **UI completion.** Run detail/failure ledger, kill confirmation, diff gate, A2A answers, remote/daemon cards, memory bank inspector.
10. **Verification.** Root/answer/memory tests, snapshot kinds, live UI DOM gates, crash drills, scale benchmark.

## Phases and gates

**R1 — Freeze root.** Root tests: hierarchical final answer, flat-plan decision, kill/cancel, cap, crash snapshot, no tools. Gate: core compile + test run, no stale tree.

**R2 — A2A answer.** Add answer contract, status/wake/deadline, continuation inbox refresh, terminal-recipient refusal, operator reply route, UI mailbox status. Gate: ask→answer→resume + no cross-branch leak.

**R3 — Roles/authority/remote.** Subagent tier + ownership switch; Remote/daemon schema/API/lifecycle; spawn approval; `/api/*` token auth. Gate: role trace, remote no-task proof, approval matrix.

**R4 — Memory banks.** Bank tree, scopes, trust/retention, promotion, HyperAgent aggregate reads, scoped clear with receipt. Gate: bank invariants and no leakage.

**R5 — UI/Provider/Scale.** FigCraft mounted with run-detail/kill/diff/answer/remote surfaces; provider context preflight; admission scheduler stage. Gate: DOM/a11y, FigCraft asset serving, provider refusal tests, scale stage.

**R6 — Audit/release.** Independent audit, crash/load/security evidence, claim register, docs. Gate: all prior pass with quoted evidence; no scale/HMARS-complete claim before.

## Interim triage corrections

- M-01 header claim is fixed at render (`context.rs` trust label + newline neutralisation), but store/write guard and raw seed inbox are still open.
- M-02 sibling-share is fixed by own-task write + parent read; ancestor/role/hive read-set and trust labels remain open.
- M-03 `take(32)` is gone; root uses a 24k budget/omission marker and run-memory reads. It currently injects up to 32 raw rows labelled shared memory without trust labels.
- M-12 rowid tie-break claim is stale: rowid ordering is present in current store queries.
- A2A M-04/M-07/M-09 remain live in part: no answer/status/wake, no continuation inbox refresh, check-then-insert budget, approved cross-branch send skips args/grant binding. M-07 terminal-recipient half is fixed in `8a19d0b`; keep M-07 open until an atomic-budget and terminal regression test land.
- Flat multi-node planner DAGs DO get the root answer (gate is `has_aggregators || nodes > 1`); only single-node plans skip. Single-node policy remains an explicit decision (D4).

## Independent distance assessment

Weighted code maturity ~5.2/10; verified-evidence maturity ~3.5/10. One solid vertical slice short of a v1 product; three subsystems (bank tree, A2A answer, remote/daemon) short of the full target.

| Area | Grade | Blocker |
|---|---:|---|
| HyperAgent execution | 6 | hierarchical-only root, serial, single plan |
| Hierarchy/roles | 5.5 | Subagent alias not tier; no daemon tier; autonomy modes absent |
| Memory banks | 4 | flat scope table; no bank tree/trust/provenance |
| A2A | 4.5 | one-way; no answer/wake/status/atomic budget |
| Remote/daemon | 1 | design only |
| UI/FigCraft | 6.5 | no pixel/live gate; gallery static |
| Provider completeness | 7 | unwired Files/Search/etc; no context preflight |
| Durability/crash | 7.5 | no resume; no fanout-8 crash drill |
| Security/authority | 6.5 | no single trust boundary; API auth absent |
| Scale/performance | 2.5 | serial dispatch, 1 active run, no scheduler |
| Docs/audit | 6 | evidence drift; stale release registry |

Highest-leverage sequence: **S0 prove current SHA** (freeze writer, compile/test/typecheck/build, one authorized smoke, regenerate status/trace), **S1 one agent-text trust boundary**, **S2 real role enum/spawn matrix/autonomy**, **S3 bank tree + A2A answer**, **S4 measured concurrency/scheduler**, **S5 daemon/remote R0-R2**, **S6 seal**. Remote/daemon is decision-gated and should not block v1 at 50-150 active.


## Wave synthesis

### Agent classes: truth
HiveMind currently has **4 enforced authority tiers** (Hyper, Super, Worker, Tool). "Subagent" is a role string mapped to Worker; it has no tier, contract kind, distinct allowlist, card, or bank. HyperAgent is a root row plus one answer-only synthesis turn; planning is a separate provider call and the root never uses its spawn/report tools. ToolAgent has no validated creator and cannot finish/report. Remote is a proposed tree-free principal/connector, not a tier or NODE kind; daemon/watchdog/supervisor are non-agent records/lanes. Required fix: one closed role/persona/tier/kind table, plan-path spawn validation, Subagent tier decision, root always-or-named-refusal policy, and Remote kept outside AgentTier.

### Memory/A2A Venn
- Write: own task branch (now), engine run bank (now); read: own + parent + run bank (now), originally seeded before writes and role-filtered.
- Still open: no bank tree/trust/retention; root prints raw run rows; rewire can retain old-parent facts; roster is stale/truncated; no answer/wake/status; terminal recipients accepted; TOCTOU 32 budget; approved cross-branch send lacks args/grant binding; secret filtering is partial and inconsistent.
- Clean target: run digest + hive summaries + branch raw; read set computed per turn; A2A status open/answered/expired and answers never grant authority; roster is discovery-only.

### Security/quality
Open P0/P1: unauthenticated `/api/*` mutations and loopback token issuance; persona allowlist now enforced but import validation/tool advertisement still loose; `cmd.exe` allowlist is not containment; approval preview truncation; fail-open security gate/absent CI; provider routes accept missing Origin; weak session token; missing FKs and transactions; raw tool-result/A2A injection labels; no durable event journal. Snapshot write now fsyncs.

### CLI/agent surface
Fixed: dry-run no-side-effect for lifecycle, `--y` alias conflict for viewport, fractional viewport numbers, submit run-id capture, stale CLI state cleanup, persona scope. Open: `--timeout-ms` not global, some exit codes wrong, stop identity/pid reuse, failed-start cleanup, bind/IPv6 validation, CLI does not decrypt DPAPI key, release packaging omits Node CLI, no CLI tests/gates, await-result commands not correlation-scoped.

### Target architecture
Five planes: control (commands/orchestrator/scheduler/policy/budget), agent-host (actor loop/supervisor/mailbox), state (event log/projections/memory/audit), interop (A2A/MCP/remote transport/devices), experience (UI projections/controls). Existing contract, run state machine, graph, provider gateway, tool/approval/jail, snapshots, and UI are reusable primitives. The run engine becomes one component; it is not the runtime.

## Offline gate evidence (HEAD `8a19d0b`, 2026-11-08)

- `cargo test --workspace --locked` with bundled `tools/w64devkit/bin` on PATH: **516 passed, 0 failed, 2 ignored, exit 0** across 42 suites.
- `npm.cmd run typecheck`: clean. `npm.cmd test`: **189/189 passed**. `npm.cmd run build`: exit 0. FigCraft public assets copied to `web/dist/figcraft`.
- CLI `node --check`: exit 0. No e2e/live run; guarded e2e remains unexecuted. JSON receipt: `docs/evidence/gate-8a19d0b5-offline.json`.
- No server/browser/provider call; key absent; no live run.

These are offline unit/build gates, not e2e/live/UI/scale evidence.

## Baseline note
Current tested baseline: `8a19d0b`, clean worktree, full offline gate green as recorded above. Guarded e2e and one authorized keyless refuse-path run remain outstanding; then release packaging/database upgrade/rollback drill.



### P0 - blocks any completion claim
- Freeze one SHA and run full backend/web/build/e2e gates; skipped security specs are not passes.
- Release build must run cargo test/clippy/fmt/dependency checks; a failing test must refuse release.
- Choose one shipped artifact set and record hashes/ports/data dirs; remove contradictory artifacts.
- DB newer-version boot refusal, pre-migration backup, upgrade/rollback drill, backup/restore commands.
- One agent-text trust boundary on write/store/publish/render (seed path now bounded; write redaction still partial).
- Token-authenticated /api/* mutations and /api/session; scoped memory clear with confirmation/receipt.
- Rewrite RUNBOOK/README/release evidence to current HEAD and actual features.

### P1 - product-ready operator experience
- A2A answer/status/wake/atomic budget; memory bank tree/trust/summaries/scoped clear; run-detail/failure ledger, roster, mailbox status/filters, kill confirmation (added).
- FigCraft board-to-surface mapping or explicitly narrow the claim; 20MB gallery bundle decision.
- In-app version/build stamp, support bundle, first-run key/launcher path, axe or documented exclusion, perf budget, clippy/audit/SBOM/secret scanning.

### Honestly deferrable
Remote/daemon runtime, unmeasured scale, Tauri shell, provider long-tail, pixel-perfect fidelity, Windows service/auto-update, multi-user.

## Requirement-matrix synthesis

| Area | Status | Remaining blocker |
|---|---|---|
| HyperAgent execution | PARTIAL | Root turn answer-only and hierarchical-only; flat policy undecided |
| SuperAgent | PARTIAL | Tier/persona/spawn/aggregation; no owned branch/hive bank/budget |
| Workers/subagents | PARTIAL | Personas exist; planner emits worker/superagent only; Subagent not a tier/card/bank |
| Memory banks | PARTIAL | Run bank + own branch now; no bank tree/trust/summaries/scoped clear |
| A2A | PARTIAL | Durable one-way; no status/answer/wake/continuation refresh/atomic budget |
| UI hierarchy | PARTIAL | Canvas/hierarchy; missing run-detail, treegrid, answer, roster, remote lane; gallery static |
| Provider | PARTIAL | DeepSeek-only chat/tools/models/balance; no context preflight/feature matrix |
| Crash/daemon | PARTIAL | Snapshots/recovery/audit; no resume/event journal/daemon/watchdog |
| Scale | MISSING | One active run, serial leaves, 8 in-flight, 64/40 caps |

Current honest distance: strong single-run HMAS skeleton; ~0 of 10 production gates complete; code maturity ~5/10, verified evidence ~3.5/10. Next gate is P0 truth freeze + tests at one SHA.

## Security/quality safeguards still open

### Highest-leverage next changes
1. P0 truth freeze + tests at one SHA.
2. One agent-text/trust boundary on every write/render path.
3. Real role/persona/tier/kind resolution and plan-path spawn validation.
4. A2A answer/status/wake and atomic budget.
5. Run-bank/bank-tree memory read sets and root summaries.
6. API token auth + stop/start identity + gate fixes.


- `/api/*` profile/memory mutations are unauthenticated; `/api/session` hands the WS token to any loopback peer; provider routes accept missing Origin.
- Persona `allowed_tool_ids` is now enforced against the scoped context, but import validation and model tool advertisement still need hardening.
- `cmd.exe` is an allowlisted shell program with cwd/PATH resolution; this is not containment.
- Approval preview truncation can consent to a prefix; args hash binds full payload.
- e2e/security gate wrapper can skip on missing results; no CI.
- Snapshot fsync is now added; missing FKs, duplicate cap authority, non-transactional multi-row writes, raw tool-result/A2A injection labels remain.
- Event journal/recovery-resume and daemon watchdogs are absent.

## Derived P0-P9 gate plan

| Gate | Outcome | Key work | Exit |
|---|---|---|---|
| P0 | truth freeze | pin commit, re-triage findings, sign D1-D9, census/current truth | clean HEAD, test/typecheck/build pass, decisions signed |
| P1 | memory banks | v12 bank tree, read sets, one agent_text guard, trust/provenance, scoped clear | leak matrix + migration/clear tests |
| P2 | A2A answer | open/answered/expired, atomic budget, recipient validation, continuation refresh, approval binding | ask-answer-resume + negatives |
| P3 | orchestration | root always or named flat refusal, SuperAgent hive, Subagent tier/decision, queue skeleton | root/flat/cancel/cap tests + hive summary |
| P4 | UI parity | bank inspector, A2A status/answer, run detail, cards, a11y; gallery remains provenance | DOM/e2e/a11y + asset serving |
| P5 | provider closure | support/refusal matrix, context preflight, 402/auth mapping, model controls | refusal matrix + one-key/model tests |
| P6 | remote light-switch | v13 tree-free principal, device ACL, HA fixture, honest UI | no task rows; grant/idempotency/unknown-state tests |
| P7 | supervisor/daemons | single lane, daemon records, watchdogs, crash reconciliation | one-job daemon/crash drill |
| P8 | remote memory/remote_call | remote bank, grants, approval bridge | isolation + grant/approval tests |
| P9 | scale/hardening | measured 50/150, network/sensitive devices only after sign-off | measured envelope, no unbounded queues |

P0 is the only next executable gate. No HMARS/remote/universal-memory/300-600 claim before its gate.

## Must not claim yet

- True HMARS or equivalent-to-donor code (ADAPTERS: no donor code copied).
- Remote/daemon agents, universal memory, device control.
- Root execution for flat plans, full A2A answer, bank-tree memory.
- 300–600 total / 50–150 active.
- FigCraft visual fidelity or production browser readiness.
- Provider feature completeness beyond implemented Chat/Tools/Models/Balance.

## Open decisions

Root final turn policy for flat plans; A2A answer deadline/authority; memory bank trust/retention; remote capability classes and operator approval; migration numbers (v11 committed, v12+ reserved); scale stage target; whether FigCraft assets remain 20MB public bundle or narrower HiveMind-only set.

## Authoritative ordering and migration ledger

**P0-P9 is the only executable ordering.** R1-R6 are workstream labels: R1→P0/P3, R2→P2, R3→P3 (roles/authority only; remote split to P6), R4→P4, R5→P5/P8/P9, R6→P0/P9. P0 precedes every feature.

| Version | Owner | Contents |
|---|---|---|
| v11 | shipped | `agent_memory_scoped` flat rows |
| v12 | A2A | additive `a2a_messages` + answer attempts/status; dual-write with legacy `agent_messages` |
| v13 | identity | task class/rank/profile/hive/ownership; `settings` + `settings_audit` |
| v14 | memory | banks/entries/journal/context snapshots; backfill flat rows as unverified |
| v15 | durability | append-only event journal/resume checkpoints |
| v16 | remote | tree-free principals/devices/grants/actions/audit |
| v17 | daemons | daemon records/leases/reports; supervisor state |

No lane may claim another's version. A2A status columns must land in v12, never a silent second v12.
Each migration needs: newer-schema refusal, pre-migration backup + integrity check, up/down fixture, and upgrade/rollback drill.

## Audit coverage M-01..M-12

| Finding | Current status |
|---|---|
| M-01 prompt header | render fixed; write/store guard and raw seed path remain open |
| M-02 sibling leak | own-task write/parent read fixed; ancestor/hive/role read set and trust remain open |
| M-03 raw `take(32)` | task results bounded/omitted marker; up to 32 raw memory rows remain without trust |
| M-04 inbox refresh | open: only seed/start refresh, never before continuation |
| M-05 scoped clear | open: only global role delete, unauthenticated, no receipt |
| M-06 legacy rows | open: `legacy` rows unreachable by recall |
| M-07 message budget | terminal-recipient half fixed in `8a19d0b`; check-then-insert TOCTOU and tests open |
| M-08 trust/promotion | open: every completed task's model text auto-written behind a blocklist |
| M-09 approval binding | open: approved cross-branch send bypasses args/grant verification |
| M-10 read errors | open: memory/inbox read errors swallowed; no omitted/truncation counts |
| M-11 clear parity | open: API/CLI/UI clear semantics diverge; sensitive reads lack `no-store` |
| M-12 rowid ordering | fixed: current store orders by `created_at, rowid`; audit stale |

## Claim corrections that must stay visible

- Terminal-recipient refusal: code-present at `8a19d0b`, no regression test yet; answer/status/wake remains absent.
- Subagent depth floor: code-present at `e8e67a4`, no negative test; verified docs still say absent.
- Subagent allowlist: 6-tool persona grant exists; missing pieces are persisted class, wire kind, card, bank.
- Root: multi-node flat plans are answered; single-node is the only skip; no root tools yet.
- FigCraft: 13 HiveMind boards are proposed additions; 20 A/B artifacts are watchdog-passed; no built-binary serving proof.
- Scale: no measurement; do not claim 50-150 active or 300-600.
- Evidence: only `docs/evidence/gate-8a19d0b5-offline.json` is SHA-bound; no e2e/live evidence at this SHA.
## P1 safety progress

- DB: newer-schema boot refusal (`E_DB_SCHEMA_NEWER`) and verified pre-migration `VACUUM INTO` backup are implemented with tests; backup/restore CLI and full v10→v11 drill remain.
- Admission: a keyless submit is refused in `RunEngine::submit_prompt` before any run/user-message/RUN_CREATED row; `ProviderExecutor::key_available` is `Some(false)` only for the unconfigured executor.
- HTTP: every non-GET `/api/*` route requires `Authorization: Bearer <session token>` or `x-hivemind-token`; UI profile import/delete and memory clear attach it. Reads and `/api/session` hardening remain.
- Evidence at `f53087a`: cargo workspace 521 passed / 2 ignored, 42 suites; web typecheck 0; web 189/189; build 0; receipt `docs/evidence/gate-f53087ae-offline.json`. P0 receipt at `8a19d0b5` remains historical.
### P1 correctness additions

- A2A message budget is now one atomic `INSERT ... SELECT ... WHERE count < 32`; the tool returns `E_CALL_LIMIT` with a message-budget detail and never inserts a 33rd row. Store test: `agent_message_budget_is_atomic_and_named`.
- Tool-turn caps are counted per task (`task_tool_turns`) while the call budget remains per run; DB test `task_turn_count_is_per_task_not_per_run`.
- HTTP 402 is `E_PROVIDER_BALANCE` (not `E_PROVIDER_AUTH`); 401 remains auth. Provider/refusal/adapter tests updated; web parity is 67 codes.
- No A2A status/answer/wake yet: the atomic budget is not the answer path.
### P1 DB operator slice

- `hiverouter-tools/src/db_admin.rs`: `backup` uses `VACUUM INTO` + integrity check; `restore` verifies, refuses schema > 11, copies the existing DB aside, removes stale `-wal`/`-shm`, and verifies the restored file.
- CLI: `hiverouter-cli db backup --db=<file> --out=<file>` and `db restore --db=<file> --from=<file> --force=true`; both support `--dry-run` and return JSON receipts.
- Tests: round-trip preserves rows; newer backup schema refused before copy; existing target requires force.
### P2 interim A2A status/operator-answer slice

Delivered:
- v12 additive columns on `agent_messages`; existing rows default `status='open'`.
- `POST /api/agents/runs/{run_id}/messages/{message_id}/answer` (token-gated) performs a CAS `open -> answered`, stores bounded operator text/actor/time, and refuses missing/closed messages by name.\n- `ask_parent` now registers an in-process waiter, sets a 120-second deadline, and blocks the tool turn until the operator answer wakes it; expiry marks the message `expired` and returns a named timeout. The child then continues with the answer in its tool result.
- Mailbox JSON exposes status/deadline/answer fields; UI shows status and an answer field for open messages.
- Tests: store CAS/duplicate; HTTP route success/duplicate; migration/backup schema updated to 12.

Not delivered and must not be claimed:
- no `A2A_ANSWER` command/event, no parent-model answer, no continuation inbox refresh, no answer-attempt cap, no separate answer row/table; answer authority is operator-only. The blocking/wake/deadline path is in-process only and a crash loses the waiter (the open row remains).
- The v12 columns deviate from the ledger's preferred `a2a_messages` dual-write table; reconcile this in the next P2 slice or amend the ledger explicitly.
### P3 preparation

- Subagent aliases (`subagent`, `sub_agent`, `sub-agent`, case variants) now share one admission policy; a worker can spawn any alias and cannot spawn a worker.
- A too-shallow Subagent now returns `E_SPAWN_DEPTH` instead of the misleading `E_TOOL_NOT_ALLOWED` in plan, command, and model/template paths.
- Generic `agent` persona/template added (8 embedded profiles) with a 9-tool worker grant; persisted class/rank/profile and the full identity resolver remain unimplemented.
### P3 v13 identity persistence (partial)

- v13 adds `tasks.class/rank/profile_id`; an `AFTER INSERT` trigger derives them from the role using the same alias table as the resolver work, and existing rows are backfilled. Unknown roles remain NULL.
- `RunStore::task_identity` reads the triple; `/api/agents/runs/{id}/roster` exposes class/rank/profile alongside role/kind. The wire kind still comes from `AgentTier`, so Subagent still renders as worker until the additive kind delta lands.
- New test: subagent alias resolves to class `subagent`, rank 4, profile `subagent`; unknown role remains unreported.
- Not delivered: `TaskRecord` fields, engine authorization by persisted class, ownership columns, strict unknown-role admission, Subagent wire kind/card.
### P3 additive subagent wire kind

- `NODE_KIND_VOCABULARY` now includes `subagent` (server + web mirror already had it). `task_node_kind` derives `subagent` from the role for graph, roster, history, NODE_STATE/NODE_SPAWN, and recovery.
- Authority remains the existing four tiers; Subagent is still Worker authority. `TaskRecord` does not yet carry class/profile fields.
- New test proves a `sub-agent` task emits `subagent`, while a `worker` still emits `worker`.
### P3 class authority on spawn

- `role_class` and `class_may_spawn` now enforce the persisted class matrix in `spawn_sub_hive`: a Subagent parent cannot spawn a Subagent, Agent can spawn only Subagent, Super can spawn Agent/Subagent, Hyper can spawn Super/Agent/Subagent.
- The check reads `tasks.class` first and falls back to the role only for pre-v13/unresolved rows; unknown roles are not laundered into Agent.
- New unit test covers the matrix and unknown-role rejection. Template/plan paths still use tier validation until they are migrated.