# HiveMind Master Finish Plan

Baseline: commit `66aff50` plus the agent-definitions wave. The prior wave commits were `fa4c373`, `679bd6b`, `bb95a44`, `aa15ec9`, `66aff50`. Offline only; no server/UI/provider run.
This plan is the merge point for the team reports in `docs/research/`, `docs/audit/`, `docs/ui/figcraft/`, and the independent analysis seats.

## Current verdict

HiveMind is a credible **single-run hierarchical HMAS**: typed planner DAG, spawn tiers, personas, tool loop, durable A2A rows, scoped memory rows, crash snapshots, DeepSeek client + refusal paths, CLI, and a spatial React UI. It is **not yet true HMARS**: the root synthesis turn is new and hierarchical-only; A2A is one-way; memory is rows, not banks; Remote/daemon agents do not exist; authority/security/scaling are incomplete; FigCraft is now mounted as static evidence but pixel/runtime fidelity is unverified.

## Delivered in this wave

| Change | Evidence |
|---|---|
| HyperAgent root final synthesis turn for hierarchical plans | `execution.rs` dispatch Phase 3; `execute_root_task_with_prompt`; synthesis system prompt in `provider_executor.rs`; no tools; bounded prompt; no late spawn |
| Snapshot carries `kind`/`role`/`label` | `graph.rs TaskNode` + `set_metadata`; execution/recovery populate; web replay already read them |
| Memory write/read isolation | each task writes its own bank id; children read parent bank; root prompt aggregates run memory; A2A/roster/memory prompt values sanitized |
| FigCraft representation surface | `web/src/figcraft/`, `FigCraftPanel.tsx`, `web/public/figcraft/` (21 accepted SVG boards, 13 HiveMind); mounted in Settings; Vite build copies `/figcraft/*` |
| UI graph/UX corrections | HiveField in viewport portal; replay fit; broken-parent edges; bee direction/dash/reduced motion; active-unknown capacity; skip target; hash routes; no duplicate owner spokes |
| Audits | memory/A2A findings + remote/daemon design + UI graph audit + goal gap plan + FigCraft coverage |

## Truth ledger

| Area | Status | Blocker |
|---|---|---|
| HyperAgent execution | PARTIAL | Root turn only when plan has aggregators; no flat-plan final turn; cap not reserved |
| SuperAgent sub-orchestration | PARTIAL | Tier/persona/spawn exists; no owned branch sub-DAG or ownership switch |
| Worker/Subagent | PARTIAL | Spawn policy exists; no distinct Subagent tier/bank; role/allowlist gaps |
| Memory banks | PARTIAL | Run+task scopes, parent recall, root aggregate; no bank tree/trust/retention/promotion |
| A2A | PARTIAL | Durable ask/send, roster, inbox; no answer/wake/deadline/status, no continuation refresh |
| Remote/daemon | MISSING | Design only under `docs/research/remote-agents/`; no code/schema/API/UI |
| UI/FigCraft | PARTIAL | Gallery mounted; run detail/answer/remote/daemon/authority surfaces missing; pixel fidelity unverified |
| Provider | PARTIAL | Chat/stream/tools/models/balance; no context preflight, 402 identity gap, no Files/Search/etc |
| Recovery/logs | PARTIAL | Snapshots/recovery/audit; no durable event journal/resume/daemon watchdogs |
| Authority/security | PARTIAL | Host guard + approvals; no `/api/*` token auth, spawn approval, autonomy settings |
| Scale | PARTIAL | One active run, serial leaves, caps 64/40, in-flight 8; no scheduler/admission |

## Top gaps, ordered

1. **A2A answer/wake.** Ask has no answer, wake, deadline, status, or continuation inbox refresh; terminal recipients are not rejected.
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
- A2A M-04/M-07/M-09 remain live: no answer/status/wake, no continuation inbox refresh, terminal recipients accepted, check-then-insert budget, approved cross-branch send skips args/grant binding.
- Flat planner DAGs still complete without a root provider answer because root synthesis is gated on `has_aggregators`. This is an open product decision, not a bug.

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

## Production completion checklist

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
