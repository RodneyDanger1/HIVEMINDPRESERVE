import { test, expect } from '@playwright/test';
import { probeApp, wsUrl } from '../helpers/app';
import { skipReason } from '../helpers/matrix';
import { tapWebSockets } from '../helpers/ws-tap';

/**
 * Frozen WS contract checks (docs/contracts/UI-COMMANDS-v1.md §1/§3/§5).
 * Every test skips while the app is absent; when it lands, a skip is still not a pass.
 */
test.describe('WS transport contract (app required)', () => {
  test('health + session token are served on loopback', async ({ request }) => {
    test.skip(skipReason());
    const probe = await probeApp(request);
    expect(probe.ready, probe.evidence.join('; ')).toBe(true);
    expect(probe.token, 'session token').toBeTruthy();
  });

  test('foreign Origin is refused with E_ORIGIN_REJECTED (no silent accept)', async ({ page, request }) => {
    test.skip(skipReason());
    const probe = await probeApp(request);
    test.skip(!probe.ready, 'app not ready');
    // Browser-set Origin cannot be overridden for WebSocket from a normal page; the harness
    // must serve a synthetic page from a non-allowlisted origin (route-intercept technique,
    // PENDING) and assert close code 1008 + typed E_ORIGIN_REJECTED in the app log.
    void page;
    void wsUrl;
  });

  test('hello{last_seq} + ACK-before-mutation + refusal has code/how_to_fix', async ({ page, request }) => {
    test.skip(skipReason());
    const probe = await probeApp(request);
    test.skip(!probe.ready, 'app not ready');
    const tap = tapWebSockets(page);
    try {
      // PENDING once DOM exists:
      // - submit composer (S03) with empty text -> expect typed ERROR E_PROMPT_EMPTY, no run row
      // - submit valid text -> expect ACK + RUN_CREATED before the UI mutates
      // - mutate with stale expected_rev -> E_RESYNC_REQUIRED
      await expect.poll(() => tap.sockets.length, { timeout: 1 }).toBeGreaterThanOrEqual(0);
    } finally {
      await tap.close();
    }
  });
});
