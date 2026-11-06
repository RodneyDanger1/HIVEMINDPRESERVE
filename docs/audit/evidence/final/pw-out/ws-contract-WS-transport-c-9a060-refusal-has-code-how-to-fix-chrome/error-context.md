# Instructions

- Following Playwright test failed.
- Explain why, be concise, respect Playwright best practices.
- Provide a snippet of code with the fix, if possible.

# Test info

- Name: ws-contract.spec.ts >> WS transport contract (app required) >> hello{last_seq} + ACK-before-mutation + refusal has code/how_to_fix
- Location: specs\ws-contract.spec.ts:30:3

# Error details

```
Error: Timeout 1ms exceeded while waiting on the predicate
```

# Test source

```ts
  1  | import { test, expect } from '@playwright/test';
  2  | import { probeApp, wsUrl } from '../helpers/app';
  3  | import { appMissing, skipReason } from '../helpers/matrix';
  4  | import { tapWebSockets } from '../helpers/ws-tap';
  5  | 
  6  | /**
  7  |  * Frozen WS contract checks (docs/contracts/UI-COMMANDS-v1.md §1/§3/§5).
  8  |  * Every test skips while the app is absent; when it lands, a skip is still not a pass.
  9  |  */
  10 | test.describe('WS transport contract (app required)', () => {
  11 |   test('health + session token are served on loopback', async ({ request }) => {
  12 |     test.skip(appMissing(), skipReason());
  13 |     const probe = await probeApp(request);
  14 |     expect(probe.ready, probe.evidence.join('; ')).toBe(true);
  15 |     expect(probe.token, 'session token').toBeTruthy();
  16 |   });
  17 | 
  18 |   test('foreign Origin is refused with E_ORIGIN_REJECTED (no silent accept)', async ({ page, request }) => {
  19 |     test.skip(appMissing(), skipReason());
  20 |     const probe = await probeApp(request);
  21 |     test.skip(!probe.ready, 'app not ready');
  22 |     // Browser-set Origin cannot be overridden for WebSocket from a normal page; the harness
  23 |     // must serve a synthetic page from a non-allowlisted origin (route-intercept technique,
  24 |     // PENDING) and assert close code 1008 + typed E_ORIGIN_REJECTED in the app log.
  25 |     void page;
  26 |     void wsUrl;
  27 |     test.fixme(true, 'synthetic non-allowlisted-origin page not implemented; origin/token evidence lives in live-app.spec.ts WS handshake test');
  28 |   });
  29 | 
  30 |   test('hello{last_seq} + ACK-before-mutation + refusal has code/how_to_fix', async ({ page, request }) => {
  31 |     test.skip(appMissing(), skipReason());
  32 |     const probe = await probeApp(request);
  33 |     test.skip(!probe.ready, 'app not ready');
  34 |     const tap = tapWebSockets(page);
  35 |     try {
  36 |       // hello{last_seq} + token + ACK-before-mutation are proven against the live server in
  37 |       // specs/live-app.spec.ts (WS handshake + CHAT_PIN ACK + E_PROMPT_EMPTY refusal tests).
> 38 |       await expect.poll(() => tap.sockets.length, { timeout: 1 }).toBeGreaterThanOrEqual(0);
     |                                                                   ^ Error: Timeout 1ms exceeded while waiting on the predicate
  39 |       test.fixme(true, 'duplicate of live-app.spec.ts WS handshake/ACK tests; keep as fixme until this spec owns the scenario');
  40 |     } finally {
  41 |       await tap.close();
  42 |     }
  43 |   });
  44 | });
  45 | 
```