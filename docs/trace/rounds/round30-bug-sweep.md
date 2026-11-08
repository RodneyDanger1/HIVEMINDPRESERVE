# Round 30 bug sweep (offline)

No runtime/server/UI/provider test.

## Fix
- SwarmPanel rendered live tasks during History replay, contradicting canvas. It now derives tasks/hierarchy/census from `buildReplayTasks(replayGraph)` when replaying; live otherwise.
- Census includes root/subagent buckets (from round 29).

## Evidence
- `npm run typecheck` clean.
- Source only; no live UI/render.

## Not done
Runtime/UI verification; A2A answer; Remote/Watchdog; scheduler/scale; `/api/*` auth.
