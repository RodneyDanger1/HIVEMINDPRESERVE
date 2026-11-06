import { test, expect } from '@playwright/test';
import { contrastRatio, EXPECTED_CONTRAST, KNOWN_FAILURES } from '../helpers/contrast';

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

/**
 * W9A: the 12 previously-fixme DOM gates now live in `specs/gui-gates.spec.ts` and run
 * deterministically against `web/dist` + mocked WS. The impossible-against-static gates
 * (G-UI-3/10, axe subcheck, residual G-UI-5 substates) are owned exclusions in
 * `e2e/GATE-EXCLUSIONS.md`; they are not skipped rows in this file (no skip-as-green).
 */
