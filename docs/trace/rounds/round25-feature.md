# Round 25 feature (offline)

- Added `refresh_a2a_inbox` before each task provider turn: durable messages addressed to the task are re-read, old `a2a:` facts removed, newest 8 inserted at 500 chars.
- Child context inheritance remains filtered (`a2a:`/`roster:` not inherited).
- No runtime/UI/server/provider test.

Claims: source-level only. Compile/tests + live prompt behavior unverified.
