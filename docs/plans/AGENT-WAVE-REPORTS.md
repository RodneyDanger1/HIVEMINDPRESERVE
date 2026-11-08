# HiveMind Agent-Wave Report Index

Baseline: HEAD `0ac12aa`; offline evidence at `0ac12aac` (`cargo test --workspace` 533 passed/2 ignored, web 189/189, build 0); no e2e/live run; provider key disabled.
This file compiles the read-only seats and points to the detailed reports. Findings are source-read unless a test/build is quoted.

## Detailed reports

| Report | Seat | Verdict |
|---|---|---|
| [agent-definitions/DRAFT.md](../research/agent-definitions/DRAFT.md) | taxonomy author | five classes defined with code mapping; one free-form role currently does label/rank/grant/memory-key work |
| [agent-definitions/VERIFIED.md](../research/agent-definitions/VERIFIED.md) | independent verifier | ACCEPT WITH CORRECTIONS; Subagent is a profile-only class, Remote has zero code; draft evidence patched to S4 |
| [memory/AUDIT.md](../audit/hivemind-memory-a2a/AUDIT.md) | memory/A2A audit | flat scopes, no bank tree, one-way A2A; some findings stale after fixes |
| [remote-agents/00-README.md](../research/remote-agents/00-README.md) | remote/daemon design | tree-free principal, device ACL, supervisor/watchdog records; design only |
| [HIVEMIND-MASTER-FINISH-PLAN.md](HIVEMIND-MASTER-FINISH-PLAN.md) | merged master plan | current truth, P0-P9 gates, must-not-claim list |
| FigCraft coverage | independent UI seat | gallery mounted; boards not fully mapped to live surfaces; pixel fidelity unverified |
| Goal matrix | independent matrix seat | full requirement-to-code matrix; strongest single-run HMAS skeleton, not HMARS |
| Security/quality seat | independent audit | P0 auth/shell/approval/gate defects; many controls genuinely strong |
| CLI/agent-surface seat | independent audit | 50 commands; dry-run/viewport/run-id/exit defects; release does not ship CLI |
| Product completion seat | independent audit | lifecycle/release/ops, not features, are the remaining blockers |
| Runtime completion seat | independent audit | deterministic DAG executor; A2A answer first vertical slice recommended |

## Verified taxonomy (short)

- **HyperAgent**: single root task, depth 0, no parent, tier HyperAgent; answer-only synthesis for multi-node plans; narrowest tool list; not a continuous conversation loop.
- **SuperAgent**: depth-1 branch lead; 7-tool allowlist, max children/depth from profile; no owned sub-DAG/bank/ownership switch yet.
- **Agent (worker)**: leaf with 9-tool allowlist behind approval/jail; may spawn Subagent only; **no built-in `agent` profile**, unknown role falls back here (critical).
- **Subagent**: profile-only role on Worker tier with a 6-tool allowlist; depth floor enforced in plan/spawn/template paths (`e8e67a4`) but untested; no persisted class, wire kind, card, or bank.
- **Remote tiny agent**: no code. Intended tree-free capability principal, not a task/tier/NODE kind; needs its own tables, grants, supervisor and global bank.
- **Non-classes**: ToolAgent tier has no producer; cards/kinds/profiles are projections/policy, not authority.

## What changed in this wave

- Readiness DAG loop now runs dependencies before dependents and gives each node its dependency/child results.
- Multi-node plans get a root synthesis answer; single-node plans do not (open policy decision).
- Root synthesis uses a synthesis system prompt, no tools, bounded results, run/agent memory summary; tool calls are rejected.
- Memory: engine run bank written before seeding; own-task branch writes; parent/run any-role recall; root aggregate; seed/refresh A2A sanitized.
- Role safety: planned roles validated against parent tier; template ids mapped to spawn classes; compact persona aliases; persona tool scope enforced; HTTP kinds use the tier classifier; Subagent depth floor enforced in plan/spawn/template paths.
- Cancelled streams no longer poison the run budget.
- Snapshot file write fsyncs before rename; snapshot carries kind/role/label.
- CLI: dry-run no longer executes lifecycle commands; `--y` alias removed for viewport; fractional viewport values; submit captures RUN_ID; stale state removed; timeout exits refusal; stop checks health pid; doctor no longer requires local state.
- UI: kill confirmation; viewport-anchored hive field; replay fit; broken-parent edges; bee dash/bob/flip/reduced motion; hash history pushes; FigCraft gallery mounted in Settings.

## Completion blockers (P0/P1)

1. Freeze one SHA and run all gates/builds; no CI, no current release evidence.
2. Release build must run backend tests, clippy/fmt, dependency checks; current release script does not.
3. One shipped artifact/port/data-dir story; current zip, portable launcher, and bare exe disagree.
4. Upgrade/rollback/backup/restore + newer-schema boot refusal.
5. One write-side agent-text trust boundary; seed/refresh sanitization is partial.
6. Token-authenticated `/api/*` mutations and `/api/session`; scoped memory clear with confirmation.
7. A2A answer/status/wake and atomic budget; continuation inbox refresh; terminal-recipient refusal is code-present in `8a19d0b` and needs regression tests.
8. Memory bank tree/trust/retention/summaries; root summaries not raw rows; scoped clear.
9. Role truth: persisted class/rank/profile, closed role enum, `agent` profile, Subagent depth floor, Template insert context.
10. Run detail/failure ledger/roster/mailbox status; FigCraft board-to-surface map or narrowed claim.
11. Provider context preflight, distinct 402 balance code, feature support/refusal matrix.
12. Crash resume/event journal/daemon supervisor; measured scale only after these.

## Current honest claim

HiveMind is a credible single-run hierarchical HMAS with a working planner/DAG executor, personas, tool loop, approvals, scoped memory rows, durable A2A rows, snapshots, CLI, and spatial UI. It is **not** true HMARS, does not have remote/daemon agents, bank-tree memory, A2A answers, measured 50-150 active scale, or a shipped/release-verified 1.2 UI.
