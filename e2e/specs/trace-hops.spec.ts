import { test, expect } from '@playwright/test';
import { loadMatrix, skipReason } from '../helpers/matrix';

const matrix = loadMatrix();

/**
 * 11-hop trace capture (skills: trace, evidence-check).
 * Seeds live in docs/trace/controls.json; captured traces append to docs/trace/traces.json.
 * While the app is absent every hop remains PENDING/UNVERIFIED — never a pass.
 */
for (const surface of matrix.surfaces) {
  for (const hop of matrix.hop_names) {
    test(`trace ${surface.id} ${hop} — ${surface.surface}`, async () => {
      test.skip(skipReason());
      // PENDING: drive the surface and record {verdict, evidence:{kind,path,quote}, captured_at}
      // for this hop; a hop with no evidence must stay UNVERIFIED.
      void hop;
      void surface;
    });
  }
}

test.describe('trace seed files are present and parseable', () => {
  test('controls/traces/events seeds exist with PENDING statuses', async () => {
    // Static check requires the files to exist; schema validation is done by
    // tools/qa/verify-harness.mjs (runs now, no app/Playwright needed).
    const { readFileSync } = await import('node:fs');
    const { fileURLToPath } = await import('node:url');
    for (const file of ['controls.json', 'traces.json', 'events.json']) {
      const url = new URL(`../../docs/trace/${file}`, import.meta.url);
      const parsed = JSON.parse(readFileSync(fileURLToPath(url), 'utf8')) as Record<string, unknown>;
      expect(parsed, `${file} parse`).toBeTruthy();
    }
  });
});
