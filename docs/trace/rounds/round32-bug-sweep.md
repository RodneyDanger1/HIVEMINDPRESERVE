# Round 32 bug sweep (offline)

No runtime/server/UI/provider test.

## Fixes
- `memory --scope` without `--run` is now a named usage refusal, not silent broadening.
- `--scope` empty/oversized rejected before HTTP.
- History run switch resets approval resolve/replay/seen state, not just approvals list.

## Evidence
- Web typecheck clean; CLI syntax 0.
- Source only; no live verification.

## Gaps
A2A answer/wake, Remote/Watchdog, scheduler/scale, `/api/*` auth, runtime tests.
