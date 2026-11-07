# Deep Foresight — Run 2 trace

**Date:** 2026-11-06 · **Starting HEAD:** `90cb7a7` (Run-1 committed; clean) · **Version:** `1.2.0-dev`
**Objective:** implement the approval-gate primitives (WP-01) and the in-process waiter primitive, while four planners produce code-level blueprints for WP-02 (side effects), WP-03 (engine suspension), WP-06 (web replay/retry), WP-07/WP-04 (proof).

## Lead code

1. `approval::{expire, cancel_pending, decision_label, count_for_run}` with one CAS per primitive.
2. Unique partial index `approvals_one_pending_per_call` on `(run_id, tool_call_id) WHERE decision IS NULL`; migration preflight refuses duplicate legacy rows with `E_APPROVAL_REPLAY` and defers the index (no deletes).
3. `ToolsBridge::{with_approval_ttl, expire_approval, cancel_approval, approval_decision, approval_count_for_run, verify_tool_grant}`.
4. New `core/src/approval_wait.rs`: `ApprovalWake`, `ApprovalWaiters::{arm,wake_resolved,wake_cancelled,disarm,len,is_empty}` with 4 tests.
5. Approval tests: `expire_cancel_and_decision_labels_are_cas_guarded`, plus bugcheck `lead_bugcheck_wp01` (duplicate legacy rows, index rejection, retry migration).

## Run-2 planners (read-only)

- `RUN2-SIDE-EFFECT-CODE.md` — split protocol: preflight (read-only) → verify grant → commit; `file_write` first, shell deferred.
- `RUN2-ENGINE-CODE.md` — H1–H12, 14 suspension tests, frame-count reconciliation, blockers C1a/C2.
- `RUN2-WEB-CODE.md` — 13 hunks for typed `CommandOutcome`, approval state machine, replay/retry, removal of the local pending lock.
- `RUN2-PROOF-CODE.md` — 14 suspension tests, 5 wiremock matrix tests, live-tool-turn/approval specs, 3 paid prompts gated by `LIVE_TOOL_TURN=1`.
- `RUN2-COMPILED-LEDGER.md` — 8 work packages, 16 contradictions, 17 UNKNOWNs, six-wave handoff prompt.

## Independent bug-check

| Bug | Evidence | Result |
|---|---|---|
| R2-1 unique-index migration failed on legacy duplicate pending rows | RED: `E_INTERNAL_ERROR` / `UNIQUE constraint failed`, index absent | Fixed: split DDL, duplicate preflight `E_APPROVAL_REPLAY`, no rows deleted, index deferred; GREEN twice |
| R2-2 TTL bulk path duplicated the CAS | One-CAS acceptance check | `expire_pending` delegates to `expire`, counts CAS wins |

## Gate (actual)

| Command | Result |
|---|---|
| `cargo test --workspace --locked` | **404 passed / 0 failed** (2 ignored) |
| `npm.cmd run typecheck` | exit 0 |
| `npm.cmd test -- --run` | **147 passed / 0 failed** |
| `cargo check --workspace --tests --locked` | exit 0 |
| Browser tool/approval specs | 16 passed in Run 1; UI unchanged in Run 2, not re-run |
| Docs/harness/key scan | Run-1 values unchanged; not re-run in Run 2 |

## Current blockers carried to Run 3

- Engine suspension is still absent (`execution.rs` non-read tools refuse `E_TOOL_NOT_ALLOWED`).
- WP-02 executor is absent; concurrency decision C1a (callback `verify_grant` vs preflight-then-verify) must be chosen.
- C2: side effects default-off flag vs always-wired.
- No live tool-turn proof; no 1.2 tag/artifacts.
- Trace remains 1/264/5/49 with empty raw capture.

## Next run

Run 3: WP-02 file_write executor + WP-03 engine suspension primitives per `RUN2-COMPILED-LEDGER.md`, with `core/tests/suspension.rs` and the deterministic matrix first. Do not run paid live tests until the deterministic suite is green.
