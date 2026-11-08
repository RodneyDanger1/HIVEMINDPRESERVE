# HiveMind Master Finish Plan

Baseline: commit `fa4c373` plus this working wave. Offline only; no server/UI/provider run.
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

## Must not claim yet

- True HMARS or equivalent-to-donor code (ADAPTERS: no donor code copied).
- Remote/daemon agents, universal memory, device control.
- Root execution for flat plans, full A2A answer, bank-tree memory.
- 300–600 total / 50–150 active.
- FigCraft visual fidelity or production browser readiness.
- Provider feature completeness beyond implemented Chat/Tools/Models/Balance.

## Open decisions

Root final turn policy for flat plans; A2A answer deadline/authority; memory bank trust/retention; remote capability classes and operator approval; migration numbers (v11 committed, v12+ reserved); scale stage target; whether FigCraft assets remain 20MB public bundle or narrower HiveMind-only set.
