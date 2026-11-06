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
 * W4 live streaming proof â€” boots the REAL Rust server on an ephemeral loopback port +
 * temp data dir (see helpers/live-server.ts), then drives frozen COMMAND envelopes over
 * a real WebSocket.
 *
 * Test 1 + 2 run today with NO provider key (child env is stripped): they prove boot,
 * embedded UI, server WS hello, and the fail-closed E_KEY_MISSING refusal with zero reply
 * deltas â€” no paid call is possible.
 *
 * Test 3 is the only paid path: it requires BOTH `LIVE_ONE_PROMPT=1` AND a key; otherwise
 * it is a named skip (never a fake pass). One short prompt is sent.
 */
test.describe('live streaming (real server, ephemeral process)', () => {
  test.setTimeout(300_000);

  let live: LiveServer | null = null;

  test.afterEach(async () => {
    if (live !== null) {
      const stopped = await live.stop();
      live = null;
      // Evidence hygiene: redacted child logs must never contain a provider key.
      expect(redactSecrets(stopped.logs.join('\n'))).not.toMatch(/sk-[A-Za-z0-9_-]{8,}/);
    }
  });

  test('boots real server: /health, /api/session, embedded UI, WS hello (no key, no paid call)', async ({
    page,
  }) => {
    const missing = serverSkipReason();
    test.skip(missing !== null, missing ?? '');
    if (missing !== null) return;

    live = await startLiveServer({ withProviderKey: false });

    expect(live.health.schema).toBe('hiverouter.v1');
    expect(['ok', 'degraded']).toContain(live.health.status);
    expect(live.session.token.length).toBeGreaterThan(0);
    expect(live.session.schema).toBe('hiverouter.v1');
    expect(live.session.ws).toBe('/ws');
    expect(live.logs().some((line) => line.includes('listening on http://127.0.0.1:'))).toBe(true);

    const response = await page.goto(live.baseURL);
    expect(response?.status()).toBe(200);
    await expect(page.locator('#root')).toBeVisible({ timeout: 15_000 });
    await expect(page.getByText(/Connected/).first()).toBeVisible({ timeout: 15_000 });

    const probe = await openWsProbe(page, live.wsUrl(0));
    const hello = await probe.waitFor((frame) => frame.name === 'SERVER_HELLO', 8_000, 'SERVER_HELLO');
    expect(hello.schema).toBe('hiverouter.v1');
    expect(hello.payload?.schema).toBe('hiverouter.v1');
    await probe.close();
  });

  test('SUBMIT_PROMPT without a key fails closed: typed E_KEY_MISSING, no REPLY_DELTA (no paid call)', async ({
    page,
  }) => {
    const missing = serverSkipReason();
    test.skip(missing !== null, missing ?? '');
    if (missing !== null) return;

    live = await startLiveServer({ withProviderKey: false });
    await page.goto(live.baseURL);
    await expect(page.getByText(/Connected/).first()).toBeVisible({ timeout: 15_000 });

    const probe = await openWsProbe(page, live.wsUrl(0));
    await probe.sendCommand('SUBMIT_PROMPT', { text: 'keyless refusal probe', project_id: randomUUID() });

    const failure = await probe.waitFor(
      (frame: ProbeEnvelope) =>
        frame.name === 'ERROR' || (frame.name === 'RUN_STATUS' && frame.payload?.status === 'failed'),
      15_000,
      'typed E_KEY_MISSING failure',
    );
    const code =
      failure.name === 'ERROR'
        ? String(failure.payload?.code ?? '')
        : String(failure.payload?.error_code ?? '');
    expect(code, `failure frame=${JSON.stringify(failure)}`).toBe('E_KEY_MISSING');

    const frames = await probe.frames();
    expect(frames.filter((frame) => frame.name === 'REPLY_DELTA')).toEqual([]);
    expect(frames.filter((frame) => frame.name === 'REPLY_DONE')).toEqual([]);
    await probe.close();
  });

  test('live streaming: ordered REPLY_DELTA -> REPLY_DONE -> RUN_STATUS completed (paid gate)', async ({
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

    const created = await probe.waitFor(
      (frame) => frame.name === 'RUN_CREATED',
      60_000,
      'RUN_CREATED',
    );
    const runId = String(created.payload?.run_id ?? '');
    expect(runId.length).toBeGreaterThan(0);

    const done = await probe.waitFor(
      (frame) => frame.name === 'REPLY_DONE' && frame.payload?.run_id === runId,
      240_000,
      'REPLY_DONE for the run',
    );
    const completed = await probe.waitFor(
      (frame) =>
        frame.name === 'RUN_STATUS' &&
        frame.payload?.run_id === runId &&
        frame.payload?.status === 'completed',
      60_000,
      'RUN_STATUS completed for the run',
    );

    const frames = await probe.frames();
    const indexOf = (target: ProbeEnvelope): number =>
      frames.findIndex(
        (candidate) => candidate.name === target.name && candidate.seq === target.seq,
      );
    const deltaIndexes = frames
      .map((frame, index) => ({ frame, index }))
      .filter(({ frame }) => frame.name === 'REPLY_DELTA' && frame.payload?.run_id === runId)
      .map(({ index }) => index);
    const doneIndex = indexOf(done);
    const completedIndex = indexOf(completed);

    expect(deltaIndexes.length, 'at least one REPLY_DELTA before REPLY_DONE').toBeGreaterThan(0);
    expect(Math.max(...deltaIndexes)).toBeLessThan(doneIndex);
    expect(doneIndex).toBeLessThan(completedIndex);

    const deltas = frames.filter(
      (frame) => frame.name === 'REPLY_DELTA' && frame.payload?.run_id === runId,
    );
    for (const delta of deltas) {
      expect(typeof delta.payload?.text_delta).toBe('string');
      expect(String(delta.payload?.text_delta).length).toBeGreaterThan(0);
    }
    expect(deltas.map((delta) => String(delta.payload?.text_delta)).join('').trim().length).toBeGreaterThan(0);
    expect(done.payload?.usage).toBeTruthy();
    expect(typeof done.payload?.usage).toBe('object');

    const failed = frames.filter(
      (frame) => frame.name === 'RUN_STATUS' && frame.payload?.run_id === runId && frame.payload?.status === 'failed',
    );
    expect(failed).toEqual([]);
    writeCapture('live-streaming.json', {
      captured_at: new Date().toISOString(),
      release_binary: process.env.E2E_SERVER_BIN ?? null,
      run_id: runId,
      reply_text: deltas.map((delta) => String(delta.payload?.text_delta)).join(''),
      usage: done.payload?.usage ?? null,
      frames: JSON.parse(redactSecrets(JSON.stringify(frames))) as unknown,
    });
    await probe.close();
  });
});

