# Round 24 bug sweep (offline)

No runtime/server/UI/provider test.

## Phase 1
- State: run history + live engine + approvals/metrics/tool activity; A2A rows.
- Flow: task_id query -> recipient filter; selectHistoryRun -> run/tasks/messages/metrics/approvals.
- Root: query accepted arbitrary task ids; historical run switch retained live-run derived state.

## Fixes
| Bug | Fix | Evidence |
|---|---|---|
| `?task_id=` returned empty inbox for unknown/foreign task | validate task exists and belongs to the run; 404 otherwise | `http.rs`; test added |
| Switching history run kept prior live metrics/tool activity/approvals/stream id | reset derived state on selectHistoryRun | `store.ts`; typecheck clean |

## Claims
- Query validation: VERIFIED source.
- Derived-state reset: VERIFIED source + typecheck.
- Runtime behavior: UNVERIFIED (no test authorized).
- Rust compile/tests: UNVERIFIED (not run).

## Gaps
- No live verification; no UI render.
- No A2A answer/wake; no token auth; no pagination.
