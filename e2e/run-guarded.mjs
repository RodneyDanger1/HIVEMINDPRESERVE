// Guarded Playwright runner: Playwright's reporter/globalTeardown errors do
// not reliably change the process exit, so this wrapper owns the exit code.
// It runs the normal suite, then fails the run if nothing passed and every
// executed test was skipped (unless E2E_ALLOW_ALL_SKIPS=1).
import { spawnSync } from 'node:child_process';
import { existsSync, readFileSync } from 'node:fs';
import { join } from 'node:path';

const cli = join(import.meta.dirname, 'node_modules', '@playwright', 'test', 'cli.js');
const result = spawnSync(process.execPath, [cli, 'test', ...process.argv.slice(2)], {
  stdio: 'inherit',
  env: { ...process.env, PLAYWRIGHT_JSON_OUTPUT_NAME: 'test-results/results.json' },
});
const exit = result.status ?? 1;

let report;
try {
  report = existsSync('test-results/results.json')
    ? JSON.parse(readFileSync('test-results/results.json', 'utf8'))
    : null;
} catch {
  report = null;
}
const stats = report?.stats;
if (stats && process.env.E2E_ALLOW_ALL_SKIPS !== '1') {
  const passed = Number(stats.expected ?? 0);
  const failed = Number(stats.unexpected ?? 0) + Number(stats.flaky ?? 0);
  const skipped = Number(stats.skipped ?? 0);
  if (passed === 0 && failed === 0 && skipped > 0) {
    console.error(
      `[skip-guard] all ${skipped} tests were skipped and none passed; set E2E_ALLOW_ALL_SKIPS=1 only with a written reason`,
    );
    process.exit(1);
  }
}
process.exit(exit);
