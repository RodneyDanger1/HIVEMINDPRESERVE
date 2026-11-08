# Round 34 bug sweep (offline)

No runtime/server/UI/provider test.

## Fixes
- A2A `from_task_id` filter now verifies the sender exists and belongs to the run; unknown/foreign → 404.
- A2A `kind` is constrained to `ask_parent`/`send_to_sibling`; unknown → 400.
- Tests added for unknown sender and unknown kind.

## Claims
- Source-level: VERIFIED.
- HTTP test source added; not executed (offline no-test rule).
- Rust compile/tests: UNVERIFIED.

## Gaps
A2A answer/wake, Remote/Watchdog, scheduler/scale, `/api/*` auth, runtime tests.
