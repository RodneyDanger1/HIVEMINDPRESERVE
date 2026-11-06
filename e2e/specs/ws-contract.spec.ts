import { test, expect } from '@playwright/test';
import { probeApp, wsUrl } from '../helpers/app';
import { appMissing, skipReason } from '../helpers/matrix';
import { tapWebSockets } from '../helpers/ws-tap';

/**
 * Frozen WS contract checks (docs/contracts/UI-COMMANDS-v1.md §1/§3/§5).
 * Every test skips while the app is absent; when it lands, a skip is still not a pass.
 */
test.describe('WS transport contract (app required)', () => {
  test('health + session token are served on loopback', async ({ request }) => {
    test.skip(appMissing(), skipReason());
    const probe = await probeApp(request);
    expect(probe.ready, probe.evidence.join('; ')).toBe(true);
    expect(probe.token, 'session token').toBeTruthy();
  });

  test('foreign Origin is refused with E_ORIGIN_REJECTED (no silent accept)', async ({ page, request }) => {
    test.skip(appMissing(), skipReason());
    const probe = await probeApp(request);
    test.skip(!probe.ready, 'app not ready');
    // Browser-set Origin cannot be overridden for WebSocket from a normal page; the harness
    // must serve a synthetic page from a non-allowlisted origin (route-intercept technique,
    // PENDING) and assert close code 1008 + typed E_ORIGIN_REJECTED in the app log.
    void page;
    void wsUrl;
    test.fixme(true, 'synthetic non-allowlisted-origin page not implemented; origin/token evidence lives in live-app.spec.ts WS handshake test');
  });

  test('hello{last_seq} + ACK-before-mutation + refusal has code/how_to_fix', async ({ page, request }) => {
    test.skip(appMissing(), skipReason());
    const probe = await probeApp(request);
    test.skip(!probe.ready, 'app not ready');
    const tap = tapWebSockets(page);
    try {
      // hello{last_seq} + token + ACK-before-mutation are proven against the live server in
      // specs/live-app.spec.ts (WS handshake + CHAT_PIN ACK + E_PROMPT_EMPTY refusal tests).
      await expect.poll(() => tap.sockets.length, { timeout: 1 }).toBeGreaterThanOrEqual(0);
      test.fixme(true, 'duplicate of live-app.spec.ts WS handshake/ACK tests; keep as fixme until this spec owns the scenario');
    } finally {
      await tap.close();
    }
  });
});
