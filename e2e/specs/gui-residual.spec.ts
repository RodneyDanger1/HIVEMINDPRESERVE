import { test, expect } from '@playwright/test';
import { readFileSync } from 'node:fs';
import { fileURLToPath } from 'node:url';
import path from 'node:path';
import {
  eventFrame,
  installMockWs,
  startDistHarness,
  type DistHarness,
  type MockWsHarness,
} from '../helpers/replay-harness';

/**
 * W11 — G-UI-5 residual substates, deterministically executed against `web/dist`
 * with a mocked WebSocket transport. No live server, no key, no provider call.
 *
 * Covers the four substates previously excluded as "needs a modelled lifecycle":
 *   loading      — SETTINGS_STATUS probing leaves SettingsState.status='pending'
 *   thinking     — NODE_SPAWN state='thinking' renders the enum label on the agent card
 *   streaming    — RUN_CREATED → RUN_STATUS(streaming) → REPLY_DELTA → REPLY_DONE
 *   reconnecting — RESYNC_REQUIRED then SERVER_HELLO restores the connection
 *
 * No test here may skip/fixme; the guarded runner (`e2e/run-guarded.mjs`) fails an
 * all-skip green run. Evidence registry: `e2e/GATE-EXCLUSIONS.md`.
 */
const ROOT = fileURLToPath(new URL('../../', import.meta.url));
const REJOIN_BANNER = /Reconnecting to the local server/;

let harness: DistHarness;

test.beforeAll(async () => {
  harness = await startDistHarness();
});

test.afterAll(async () => {
  await harness.close();
});

async function openConnected(page: import('@playwright/test').Page): Promise<MockWsHarness> {
  const ws = await installMockWs(page);
  await page.goto(harness.url);
  await expect(page.getByText(/Connected/).first()).toBeVisible({ timeout: 15_000 });
  return ws;
}

test.describe('G-UI-5 residual substates — deterministic mocked lifecycle', () => {
  test('loading: pending SETTINGS_STATUS probe renders the loading state (not fake data)', async ({ page }) => {
    await openConnected(page);
    await page.getByRole('button', { name: 'Settings', exact: true }).first().click();
    await expect(page.getByText('Provider key status').first()).toBeVisible();
    // The mock auto-ACKs SETTINGS_STATUS but never sends SETTINGS_STATUS_RESULT, so the
    // settings probe stays pending and the UI must show its loading primitive.
    const loading = page.locator('main').getByRole('status').filter({ hasText: 'Loading' });
    await expect(loading).toBeVisible({ timeout: 10_000 });
    await expect(page.getByText('Key configured')).toHaveCount(0);
    await expect(page.getByText('Key missing')).toHaveCount(0);
  });

  test('thinking: NODE_SPAWN thinking renders the enum label on agent + canvas cards', async ({ page }) => {
    const ws = await openConnected(page);
    ws.route().send(
      eventFrame(
        'NODE_SPAWN',
        { task_id: 'task-w11-think', state: 'thinking', kind: 'superagent', label: 'Thinker W11', parent_task_id: null },
        1,
      ),
    );
    const nav = page.locator('nav[aria-label="Primary navigation"]');
    await nav.getByRole('button', { name: 'Agents', exact: true }).click();
    await expect(page.getByText('Thinker W11', { exact: true }).first()).toBeVisible({ timeout: 10_000 });
    await expect(page.getByText('Thinking', { exact: true }).first()).toBeVisible();

    await nav.getByRole('button', { name: 'Workspace', exact: true }).click();
    const canvasCard = page.locator('.react-flow__node').filter({ hasText: 'Thinker W11' });
    await expect(canvasCard).toBeVisible({ timeout: 10_000 });
    await expect(canvasCard.getByRole('button').first()).toHaveAttribute('aria-label', /Thinking/);
  });

  test('streaming: deltas render progressively and clear on REPLY_DONE without losing text', async ({ page }) => {
    const ws = await openConnected(page);
    const route = ws.route();
    route.send(eventFrame('RUN_CREATED', { run_id: 'run-w11-stream', status: 'running' }, 1));
    route.send(eventFrame('RUN_STATUS', { run_id: 'run-w11-stream', status: 'streaming' }, 2));
    route.send(eventFrame('REPLY_DELTA', { run_id: 'run-w11-stream', text_delta: 'Deterministic ' }, 3));
    route.send(eventFrame('REPLY_DELTA', { run_id: 'run-w11-stream', text_delta: 'stream text' }, 4));

    await expect(page.getByText('Deterministic stream text')).toBeVisible({ timeout: 10_000 });
    const streamingStatus = page.getByRole('status').filter({ hasText: 'Streaming' });
    await expect(streamingStatus).toBeVisible();
    // No terminal event yet: the composer stays enabled (no stuck/fake terminal state).
    await expect(page.locator('#composer')).toBeEnabled();

    route.send(
      eventFrame(
        'REPLY_DONE',
        { run_id: 'run-w11-stream', usage: { prompt_tokens: 10, completion_tokens: 5, total_tokens: 15 } },
        5,
      ),
    );
    await expect(streamingStatus).toHaveCount(0, { timeout: 10_000 });
    await expect(page.getByText('Deterministic stream text')).toBeVisible();
  });

  test('reconnecting: RESYNC_REQUIRED blocks mutations, SERVER_HELLO restores without losing the run', async ({ page }) => {
    const ws = await openConnected(page);
    const route = ws.route();
    route.send(eventFrame('RUN_CREATED', { run_id: 'run-w11-reconnect', status: 'running' }, 1));
    const controlStatus = page.getByText(/run-w11-.*running/).first();
    await expect(controlStatus).toBeVisible({ timeout: 10_000 });

    route.send(eventFrame('RESYNC_REQUIRED', { expected_seq: 1, last_seq: 0 }, 2));
    await expect(page.getByRole('alert').first()).toContainText(REJOIN_BANNER, { timeout: 10_000 });
    await expect(page.locator('#composer')).toBeDisabled();

    route.send(eventFrame('SERVER_HELLO', { last_seq: 0, schema: 'hiverouter.v1' }, 3));
    await expect(page.getByText(/Reconnecting to the local server/)).toHaveCount(0, { timeout: 10_000 });
    await expect(page.locator('#composer')).toBeEnabled();
    await expect(page.getByText(/run-w11-.*running/).first()).toBeVisible();
  });

  test('registry hygiene: residual states are covered and no skip-as-green rows remain', async () => {
    const spec = readFileSync(fileURLToPath(new URL('./gui-residual.spec.ts', import.meta.url)), 'utf8');
    for (const state of ['loading', 'thinking', 'streaming', 'reconnecting']) {
      expect(spec.toLowerCase(), `residual spec names ${state}`).toContain(state);
    }
    const fixmeOrSkip = /test\.(?:fixme|skip)\s*\(/;
    expect(fixmeOrSkip.test(spec), 'gui-residual.spec.ts must not skip/fixme').toBe(false);

    const exclusions = readFileSync(path.join(ROOT, 'e2e', 'GATE-EXCLUSIONS.md'), 'utf8');
    expect(exclusions, 'G-UI-5 residual exclusion must be closed').not.toMatch(/G-UI-5 \(residual substates\)/);
    expect(exclusions, 'closed-exclusion evidence recorded').toMatch(/gui-residual\.spec\.ts/);
    expect(exclusions, 'axe decision recorded as explicit exclusion').toMatch(/explicit exclusion, no install/i);
    for (const stillExcluded of ['G-UI-3', 'G-UI-10', 'W1', 'W2']) {
      expect(exclusions, `${stillExcluded} remains a named exclusion`).toContain(stillExcluded);
    }
    expect(exclusions, 'no unapproved axe install claim').not.toMatch(/installed axe/i);
  });
});
