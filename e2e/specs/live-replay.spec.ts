import { test, expect } from '@playwright/test';
import { mkdirSync, writeFileSync } from 'node:fs';
import { dirname, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';
import { randomUUID } from 'node:crypto';
import {
  openWsProbe,
  paidGate,
  redactSecrets,
  serverSkipReason,
  startLiveServer,
  type LiveServer,
  type ProbeEnvelope,
} from '../helpers/live-server';

const CAPTURE_DIR = resolve(
  dirname(fileURLToPath(import.meta.url)),
  '../../docs/trace/live/captures/w11',
);

function writeCapture(name: string, value: unknown): void {
  mkdirSync(CAPTURE_DIR, { recursive: true });
  writeFileSync(resolve(CAPTURE_DIR, name), `${JSON.stringify(value, null, 2)}\n`, 'utf8');
}

/**
 * W4 live replay proof — real Rust server, real WebSocket, real snapshot store.
 *
 * Test 1 runs with NO provider key: HISTORY_REVISION_SET for an unknown run must return
 * a typed refusal and no HISTORY_SNAPSHOT (fail-closed, no fabricated replay).
 *
 * Test 2 is the only paid path (requires LIVE_ONE_PROMPT=1 + key): one short prompt ->
 * completed run -> iterate real revisions via /api/history/runs/{id}/revisions ->
 * HISTORY_REVISION_SET -> HISTORY_SNAPSHOT with >=2 nodes -> the UI (same server) enters
 * read-only replay (banner, nodes, 0 draggable, 0 selectable). Otherwise named skip.
 */
test.describe('live replay (real server, ephemeral process)', () => {
  test.setTimeout(360_000);

  let live: LiveServer | null = null;

  test.afterEach(async () => {
    if (live !== null) {
      const stopped = await live.stop();
      live = null;
      expect(redactSecrets(stopped.logs.join('\n'))).not.toMatch(/sk-[A-Za-z0-9_-]{8,}/);
    }
  });

  test('HISTORY_REVISION_SET for an unknown run refuses by name, no snapshot (no paid call)', async ({
    page,
  }) => {
    const missing = serverSkipReason();
    test.skip(missing !== null, missing ?? '');
    if (missing !== null) return;

    live = await startLiveServer({ withProviderKey: false });
    await page.goto(live.baseURL);
    await expect(page.getByText(/Connected/).first()).toBeVisible({ timeout: 15_000 });

    const probe = await openWsProbe(page, live.wsUrl(0));
    await probe.sendCommand('HISTORY_REVISION_SET', { run_id: randomUUID(), revision: 0 });

    const failure = await probe.waitFor((frame) => frame.name === 'ERROR', 15_000, 'typed history refusal');
    const code = String(failure.payload?.code ?? '');
    expect(['E_HISTORY_GAP', 'E_RUN_NOT_FOUND']).toContain(code);
    const frames = await probe.frames();
    expect(frames.filter((frame) => frame.name === 'HISTORY_SNAPSHOT')).toEqual([]);
    await probe.close();
  });

  test('live replay: HISTORY_REVISION_SET -> HISTORY_SNAPSHOT >=2 nodes, UI read-only (paid gate)', async ({
    page,
  }) => {
    const gate = paidGate();
    test.skip(!gate.enabled, gate.reason);
    if (!gate.enabled) return;

    live = await startLiveServer();
    await page.goto(live.baseURL);
    await expect(page.getByText(/Connected/).first()).toBeVisible({ timeout: 15_000 });

    const probe = await openWsProbe(page, live.wsUrl(0));
    await probe.sendCommand('SUBMIT_PROMPT', {
      text: 'Reply with the single word: OK',
      project_id: randomUUID(),
    });
    const created = await probe.waitFor((frame) => frame.name === 'RUN_CREATED', 60_000, 'RUN_CREATED');
    const runId = String(created.payload?.run_id ?? '');
    expect(runId.length).toBeGreaterThan(0);
    await probe.waitFor(
      (frame) =>
        frame.name === 'RUN_STATUS' &&
        frame.payload?.run_id === runId &&
        frame.payload?.status === 'completed',
      300_000,
      'RUN_STATUS completed for the run',
    );

    const history = await live.fetchJson<{
      latest_revision: number | null;
      revision_count: number;
      revisions: Array<{ revision: number }>;
    }>(`/api/history/runs/${runId}/revisions`);
    expect(history.revision_count, 'the run must have at least one readable revision').toBeGreaterThan(0);
    const revisions = history.revisions
      .map((entry) => entry.revision)
      .sort((a, b) => b - a)
      .slice(0, 12);

    let chosen: { revision: number; nodes: number } | null = null;
    const observed: Array<{ revision: number; nodes: number }> = [];
    for (const revision of revisions) {
      await probe.sendCommand('HISTORY_REVISION_SET', { run_id: runId, revision });
      const snapshot = await probe.waitFor(
        (frame: ProbeEnvelope) =>
          frame.name === 'HISTORY_SNAPSHOT' &&
          frame.payload?.run_id === runId &&
          frame.payload?.revision === revision,
        20_000,
        `HISTORY_SNAPSHOT revision ${revision}`,
      );
      const graph = snapshot.payload?.graph as { nodes?: unknown[] } | undefined;
      const nodes = Array.isArray(graph?.nodes) ? graph.nodes.length : 0;
      observed.push({ revision, nodes });
      if (nodes >= 2) {
        chosen = { revision, nodes };
        break;
      }
    }
    expect(observed.length, 'at least one revision was requested').toBeGreaterThan(0);
    expect(chosen, `no revision with >=2 graph nodes; observed=${JSON.stringify(observed)}`).not.toBeNull();

    // The same HISTORY_SNAPSHOT is fanned out to the app's WS: the canvas must now be in
    // read-only replay mode (banner + projected nodes + non-draggable/non-selectable).
    await expect(page.getByText(/Replay \(read-only\)/)).toBeVisible({ timeout: 15_000 });
    await expect(page.locator('.react-flow__node')).toHaveCount(chosen?.nodes ?? 0, { timeout: 15_000 });
    await expect(page.locator('.react-flow__node.draggable')).toHaveCount(0);
    await expect(page.locator('.react-flow__node.selectable')).toHaveCount(0);
    writeCapture('live-replay.json', {
      captured_at: new Date().toISOString(),
      release_binary: process.env.E2E_SERVER_BIN ?? null,
      run_id: runId,
      observed,
      chosen,
      revision_count: history.revision_count,
      frames: JSON.parse(redactSecrets(JSON.stringify(await probe.frames()))) as unknown,
    });
    await probe.close();
  });
});
