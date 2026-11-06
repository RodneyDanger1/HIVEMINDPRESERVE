import { test, expect } from '@playwright/test';
import { probeApp } from '../helpers/app';
import { appMissing, skipReason } from '../helpers/matrix';

/**
 * WS transport contract — live-server binding (W9A scope decision).
 *
 * The two former `test.fixme` stubs (foreign-Origin refusal; hello/ACK duplicate) are now
 * tracked product exclusions W1/W2 in `e2e/GATE-EXCLUSIONS.md` with owner + acceptance.
 * Keeping them as skipped tests would be skip-as-green, so they are removed here; the
 * deterministic connect/hello path is exercised by the dist+mockWS harness in
 * `specs/replay-state.spec.ts` / `specs/gui-gates.spec.ts`, and real ACK-before-mutation is
 * covered by `specs/live-app.spec.ts` when a live server is attached (E2E_APP_READY=1).
 */
test.describe('WS transport contract (live server required)', () => {
  test('health + session token are served on loopback', async ({ request }) => {
    test.skip(appMissing(), skipReason());
    const probe = await probeApp(request);
    expect(probe.ready, probe.evidence.join('; ')).toBe(true);
    expect(probe.token, 'session token').toBeTruthy();
  });
});
