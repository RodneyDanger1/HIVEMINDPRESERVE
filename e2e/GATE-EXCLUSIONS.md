# GATE-EXCLUSIONS â€” named product exclusions (W9A)

Every G-UI gate and WS stub that **cannot be proven against the static `web/dist` fixture** (loopback static server + mocked WebSocket, no key, no live Rust server) is listed here with an owner and an executable acceptance. Nothing in this file is a pass: it is a tracked exclusion, and the registry test in `specs/gui-gates.spec.ts` asserts every G-UI-1..12 id is either implemented or listed here.

Implemented gates: G-UI-1, 2, 4, 5 (empty/offline/refused/missing-key/replay in `specs/gui-gates.spec.ts`; loading/thinking/streaming/reconnecting in `specs/gui-residual.spec.ts`, W11), 6, 7, 8 (focus ring), 9, 11, 12.
Static math already executed: G-UI-1 printed rows â†’ `specs/a11y-gates.spec.ts`.

| ID | Gate / stub | Why the static fixture cannot prove it | Owner | Acceptance (what would close it) |
|---|---|---|---|---|
| G-UI-3 | foreign Origin â†’ `E_ORIGIN_REJECTED`; token accepted in first frame | Origin/allowlist and token handshake are enforced by the Rust WS server; `page.routeWebSocket` is a client-side mock and cannot make the server refuse an Origin | b-core / Lead (server owner) | Live-server run with a synthetic non-allowlisted-origin page: WS closes 1008, typed `E_ORIGIN_REJECTED` observed, no state mutation, zero rows written; tracked evidence file recorded |
| G-UI-10 | kill socket â†’ hello/resnapshot/`Reconnecting`; stale mutation â†’ `E_RESYNC_REQUIRED` | Requires real server-side seq/resync/snapshot semantics; a mocked socket cannot prove the server resnapshot contract | b-core / Lead | Live-server test that kills the socket mid-stream, reconnects, asserts `hello{last_seq}` â†’ snapshot/diff adoption without duplicated text and stale mutation refused with `E_RESYNC_REQUIRED`; evidence recorded |
| G-UI-8 (axe subcheck) | axe serious = 0 | **W11 decision: explicit exclusion, no install.** `axe-core` and `@axe-core/playwright` are absent (`MODULE_NOT_FOUND` — checked from `e2e/`); adding either needs Lead approval and a version pinned from the official npm registry (no version is fabricated here). The focus-ring/contrast portion of G-UI-8 is implemented in `specs/gui-gates.spec.ts`. | Lead / tooling owner | Lead approves a pinned axe dependency (exact version + integrity from official npm), the serious-violation=0 assertion is added to G-UI-8 and runs green on `web/dist`; then this row is deleted |
| W1 | foreign Origin refused with `E_ORIGIN_REJECTED` (ws-contract stub removed) | Same reason as G-UI-3 â€” needs the real server | b-core / Lead | Same acceptance as G-UI-3 |
| W2 | hello{last_seq} + ACK-before-mutation + refusal code/how_to_fix (ws-contract stub removed) | `hello`/connect is already exercised by the replay harness (`replay-state.spec.ts` asserts Connected after mocked WS); real ACK-before-mutation and refusal payloads are covered by `live-app.spec.ts` when a live server is available â€” keeping a duplicate fixme stub would be a skip-as-green | QA harness owner | Live-server run green for `live-app.spec.ts` WS handshake/CHAT_PIN/E_PROMPT_EMPTY tests; this exclusion is closed by that evidence, not by a skipped local test |

**Closed exclusion (W11):** G-UI-5 residual substates (`loading`, `thinking`, `streaming`, `reconnecting`) are now executed deterministically in `specs/gui-residual.spec.ts` against `web/dist` + mocked WS (no live server, no key). Evidence command: `node run-guarded.mjs --project=chrome specs/gui-gates.spec.ts specs/gui-residual.spec.ts`. No skip-as-green path remains for these states.

## Suite truth guard

`run-guarded.mjs` wraps Playwright, reads the JSON report and exits non-zero when **only skipped tests** ran (0 passed, 0 failed, skipped > 0) unless `E2E_ALLOW_ALL_SKIPS=1`. Use `npm run e2e:guarded -- --project=chrome`. This prevents a green run that executed nothing. Verified with `npx playwright test specs/ws-contract.spec.ts --project=chrome` (all app-bound tests skip â†’ exit 1) and with the full run (passes present â†’ exit 0).

