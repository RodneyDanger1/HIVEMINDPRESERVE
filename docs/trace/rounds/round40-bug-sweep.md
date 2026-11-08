# Round 40 bug sweep (offline)

No runtime/server/UI/provider test.

## Fix
- Swarm runtime capacity strip was fetched once; now loads on run id/status change and refreshes every 15 s. Stale capacity can no longer describe a later run state.

## Evidence
- Web typecheck clean.
- No runtime/UI verification.

## Gaps
A2A answer/wake, Remote/Watchdog, scheduler/scale, `/api/*` auth; no 30-round post-feature bug cadence yet.
