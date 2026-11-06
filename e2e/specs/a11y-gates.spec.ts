import { test, expect } from '@playwright/test';
import { contrastRatio, EXPECTED_CONTRAST, KNOWN_FAILURES } from '../helpers/contrast';
import { appMissing, skipReason } from '../helpers/matrix';

/**
 * G-UI-1..12 plan (reports/ui/UI-REQUIREMENTS.md §4, UI-WIRING-GUIDE.md §4).
 * Static contrast math is proven now; DOM gates skip until the web app exists.
 */
test.describe('a11y/contrast — static math (runs now; no app required)', () => {
  test('G-UI-1: six printed rows reproduce within ±0.01', () => {
    for (const row of EXPECTED_CONTRAST) {
      const actual = contrastRatio(row.fg, row.bg);
      expect(Math.abs(actual - row.expected), `${row.pair} expected ${row.expected} got ${actual.toFixed(3)}`).toBeLessThanOrEqual(0.01);
    }
  });

  test('G-UI-1: known failing pairs fail exactly as recorded (meta/focus ring)', () => {
    for (const row of KNOWN_FAILURES) {
      const actual = contrastRatio(row.fg, row.bg);
      expect(Math.abs(actual - row.failing), `${row.pair} recorded ${row.failing} got ${actual.toFixed(3)}`).toBeLessThanOrEqual(0.02);
      expect(actual, `${row.pair} must be below ${row.threshold} to be a recorded failure`).toBeLessThan(row.threshold);
    }
  });
});

test.describe('G-UI-1..12 — DOM gates (app required)', () => {
  const gates = [
    ['G-UI-1', 'contrast table on live token pairs'],
    ['G-UI-2', 'Ctrl+0 recentre (Meta+0 macOS alias); text-field focus not hijacked'],
    ['G-UI-3', 'foreign Origin -> E_ORIGIN_REJECTED; token accepted in first frame'],
    ['G-UI-4', 'one >=13-state enum, enum<->token<->label total'],
    ['G-UI-5', 'missing-state coverage: loading/empty/error/refused/streaming/replay/offline/missing-key/thinking'],
    ['G-UI-6', 'reduced motion: 0 animations, state still updates'],
    ['G-UI-7', 'zoom clamps at 0.1/3.0, both reachable'],
    ['G-UI-8', 'focus ring >=3:1; axe serious = 0'],
    ['G-UI-9', 'refusal toast: code + how_to_fix + aria-live, pre-mutation'],
    ['G-UI-10', 'kill socket -> hello/resnapshot/Reconnecting; stale mutation E_RESYNC_REQUIRED'],
    ['G-UI-11', 'no hex outside token module; every token has PDF page:line'],
    ['G-UI-12', 'P95 frame time measured or claim UNVERIFIED'],
  ] as const;

  for (const [gate, check] of gates) {
    test(`${gate}: ${check}`, async ({ page }) => {
      test.skip(appMissing(), skipReason());
      void page;
      // DOM gate implementation is pending; the live token/contrast, zoom/Ctrl+0, refusal,
      // WS and render-back evidence currently lives in specs/live-app.spec.ts. Marking this
      // fixme (not passed) so the suite cannot claim a gate it did not execute.
      test.fixme(true, 'DOM gate not implemented yet — see specs/live-app.spec.ts and docs/trace/live/');
    });
  }
});
