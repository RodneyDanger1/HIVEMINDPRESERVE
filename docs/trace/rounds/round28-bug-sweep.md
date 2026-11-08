# Round 28 bug sweep (offline)

No runtime/server/UI/provider test.

## Fixes
- `/api/runtime` no longer silently hides DB errors for latest-run/task-count; returns 503.
- Runtime status is `no-store`.
- History run switch clears stale approvals/resync state, not just metrics/tool/stream.

## Claims
- Source-level: VERIFIED.
- Web typecheck: VERIFIED.
- Rust compile/tests: UNVERIFIED (not run).
- Live runtime: UNVERIFIED.

## Gaps
A2A answer/wake, per-tool-turn inbox, Remote/Watchdog, scheduler/scale, `/api/*` auth, runtime tests.
