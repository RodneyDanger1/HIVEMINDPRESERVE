# Round 38 bug sweep (offline)

No runtime/server/UI/provider test.

## Fix
- A2A mailbox kind filter reset on run switch; a new run cannot appear empty under a previous filter.

## Evidence
- Web typecheck clean.
- GitNexus `impact` cannot resolve `AgentMessagesPanel` (`UNKNOWN`); `detect-changes` produced no matched summary.
- No runtime/UI verification.

## Gaps
A2A answer/wake, Remote/Watchdog, scheduler/scale, `/api/*` auth, runtime tests.
