# Round 36 bug sweep (offline)

No runtime/server/UI/provider test.

## Fixes
- Run-memory fetches are sequence-guarded; a late response from a previous run cannot overwrite the current run's rows.
- A scope filter that no longer exists falls back to All scopes instead of hiding every row.

## Evidence
- Web typecheck clean.
- No Rust/UI live verification.

## Gaps
A2A answer/wake, Remote/Watchdog, scheduler/scale, `/api/*` auth, runtime tests.
