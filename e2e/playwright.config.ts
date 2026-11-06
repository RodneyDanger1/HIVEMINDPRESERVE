// Playwright config for the HiveMind v1 QA harness.
//
// BROWSER POLICY: no browser download. This config launches the *installed*
// Chrome (Channel "chrome") or Edge (Channel "msedge") on this machine:
//   C:\Program Files\Google\Chrome\Application\chrome.exe       (156.0.8078.75)
//   C:\Program Files (x86)\Microsoft\Edge\Application\msedge.exe (156.0.4314.58)
// Do NOT run `npx playwright install`. If a channel is missing, the run must
// fail loudly with a named refusal; it must not fall back to a downloaded browser.
//
// Current status (2026-11-06 08:0x): the release binary serves the embedded UI at
// http://127.0.0.1:8799; run with E2E_BASE_URL=http://127.0.0.1:8799 and
// E2E_APP_READY=1. `webServer` is still intentionally not configured (the server is
// started by the build/Lead, not by Playwright), and browser tests must never start a
// paid provider run.
import { defineConfig } from '@playwright/test';

const BASE_URL = process.env.E2E_BASE_URL ?? 'http://127.0.0.1:8080';

export default defineConfig({
  testDir: './specs',
  outputDir: './test-results',
  timeout: 30_000,
  expect: { timeout: 5_000 },
  fullyParallel: false,
  workers: 1, // one worker keeps WS/event ordering deterministic for trace evidence
  retries: 0,
  forbidOnly: true,
  reporter: [
    ['list'],
    ['html', { outputFolder: 'playwright-report', open: 'never' }],
    // Machine-readable results consumed by the skip-guard teardown.
    ['json', { outputFile: 'test-results/results.json' }],
  ],
  use: {
    baseURL: BASE_URL,
    headless: true,
    trace: 'retain-on-failure',
    screenshot: 'only-on-failure',
    video: 'off',
    acceptDownloads: false,
    // No storageState / no auth bypass: the app's own /api/session token flow is under test.
  },
  projects: [
    // Default project: installed Google Chrome channel (no download).
    { name: 'chrome', use: { channel: 'chrome' } },
    // Optional project: installed Microsoft Edge channel (no download).
    { name: 'msedge', use: { channel: 'msedge' } },
  ],
});

