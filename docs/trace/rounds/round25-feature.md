# Round 25 feature (offline)

- Added `refresh_a2a_inbox` at task execution start: durable messages addressed to the task are re-read, old `a2a:` facts removed, newest 8 inserted at 500 chars.
- A store read failure preserves the previously seeded facts; unsafe secret-shaped messages are skipped.
- Child context inheritance remains filtered (`a2a:`/`roster:` not inherited).
- No runtime/UI/server/provider test.

Claims: source-level only. Compile/tests + live prompt behavior unverified. Tool-turn continuations do not re-read the inbox.
