# Round 26 bug sweep (offline)

No runtime/server/UI/provider test.

## Fixes
- A2A inbox refresh no longer erases previously seeded facts when the store read fails.
- Secret-shaped A2A message text is skipped before entering a prompt.
- Documentation corrected: inbox refresh occurs at task execution start, not before every provider tool turn.
- History-run switch resets streaming/metrics/tool/approval state; transcript/read failure marks incomplete + degraded.

## Claims
- Source-level behavior: VERIFIED.
- Typecheck (web): VERIFIED later in this pass.
- Rust compile/tests: UNVERIFIED (not run).
- Live behavior: UNVERIFIED.

## Gaps
- No inbox refresh between tool turns.
- No A2A answer/wake/deadline.
- `/api/*` still Host-only; no runtime verification.
