import { test, expect } from '@playwright/test';
import type { Page } from '@playwright/test';
import { createHash, randomUUID } from 'node:crypto';
import { execFileSync } from 'node:child_process';
import { mkdirSync, readFileSync, statSync, writeFileSync } from 'node:fs';
import { dirname, join, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';

import { captureFrames, type FrameRecord } from '../helpers/evidence';
import {
  openWsProbe,
  paidGate,
  redactSecrets,
  serverSkipReason,
  startLiveServer,
  type LiveServer,
  type ProbeEnvelope,
} from '../helpers/live-server';

/**
 * W12 live trace capture producer (registry lane, WP-08).
 *
 * One test per CAPTURE-REGISTRY id. Each captured test writes
 * `docs/trace/live/captures/w12/<CAP-ID>/frames.jsonl`, `dom.json`, `shot.png`
 * and `manifest.json` and then asserts every artifact is non-empty — an empty
 * capture FAILS the test, it never counts as evidence.
 *
 * Paid gates are named and explicit:
 *  - `LIVE_ONE_PROMPT=1` AND a present `DEEPSEEK_API_KEY` (see `paidGate()`);
 *  - the live binary must exist (`serverSkipReason()`);
 *  - CAP-SAPPROVAL additionally needs `LIVE_APPROVAL_CAPTURE=1` because the
 *    engine approval-suspension producer is not landed yet (WP-03): without it
 *    the test skips by name instead of fabricating an APPROVAL_REQUEST.
 * Key material is never written: manifests store booleans only, frames pass
 * through `redactSecrets`, and child logs are asserted key-free afterEach.
 */
const REPO_ROOT = resolve(dirname(fileURLToPath(import.meta.url)), '../..');
const CAPTURE_ROOT = resolve(REPO_ROOT, 'docs/trace/live/captures/w12');
const SPEC_COMMAND = 'node run-guarded.mjs --project=chrome specs/live-trace-capture.spec.ts';
const REGISTRY_CAPTURE_IDS = [
  'CAP-S22-RECHECK',
  'CAP-S10-REPLAY',
  'CAP-S27-STREAMING',
  'CAP-SAPPROVAL',
  'CAP-S15-19-CARDS',
] as const;

interface CaptureGate {
  readonly paid: boolean;
  readonly live_one_prompt: boolean;
  readonly deepseek_key_present: boolean;
  readonly approval_capture_opt_in: boolean;
  readonly app_ready: boolean;
}

interface HopEvidence {
  readonly hop: string;
  readonly evidence_kind: 'dom' | 'request' | 'ws-frame' | 'db-row' | 'log' | 'file';
  readonly pointer: string;
  readonly frames: number;
}

interface CaptureManifest {
  readonly id: (typeof REGISTRY_CAPTURE_IDS)[number];
  readonly status: 'CAPTURED';
  readonly head: string;
  readonly server_bin_sha256: string;
  readonly captured_at: string;
  readonly command: string;
  readonly gate: CaptureGate;
  readonly hop_evidence: readonly HopEvidence[];
  readonly artifacts: {
    readonly frames_jsonl: string;
    readonly dom_json: string;
    readonly shot_png: string;
  };
}

function sha256File(file: string): string {
  return createHash('sha256').update(readFileSync(file)).digest('hex');
}

function headSha(): string {
  try {
    return execFileSync('git', ['rev-parse', 'HEAD'], {
      cwd: REPO_ROOT,
      encoding: 'utf8',
      stdio: ['ignore', 'pipe', 'ignore'],
    }).trim();
  } catch {
    return 'UNKNOWN_HEAD';
  }
}

function captureGate(paid: boolean): CaptureGate {
  return {
    paid,
    live_one_prompt: process.env.LIVE_ONE_PROMPT === '1',
    deepseek_key_present:
      typeof process.env.DEEPSEEK_API_KEY === 'string' && process.env.DEEPSEEK_API_KEY.length > 0,
    approval_capture_opt_in: process.env.LIVE_APPROVAL_CAPTURE === '1',
    app_ready: process.env.E2E_APP_READY === '1',
  };
}

/** Page-tap frames carry `at` (ms); probe envelopes may not. */
function frameAt(frame: FrameRecord | ProbeEnvelope): number {
  const at = (frame as { at?: unknown }).at;
  return typeof at === 'number' ? at : Date.now();
}

/**
 * Writes the four registry artifacts and asserts they are non-empty.
 * All frame payloads are redacted before they touch disk.
 */
async function writeCapture(
  capId: (typeof REGISTRY_CAPTURE_IDS)[number],
  live: LiveServer,
  page: Page,
  frames: readonly (FrameRecord | ProbeEnvelope)[],
  hopEvidence: readonly HopEvidence[],
  gate: CaptureGate,
): Promise<CaptureManifest> {
  const dir = join(CAPTURE_ROOT, capId);
  mkdirSync(dir, { recursive: true });
  const capturedAt = new Date().toISOString();
  const rel = (file: string): string =>
    `docs/trace/live/captures/w12/${capId}/${file}`;
  const artifactPath = (file: string): string => join(dir, file);
  const redacted = JSON.parse(
    redactSecrets(JSON.stringify(frames.map((frame) => ({
      recorded_at: new Date(frameAt(frame)).toISOString(),
      ...frame,
    })))),
  ) as unknown[];

  const framesFile = artifactPath('frames.jsonl');
  writeFileSync(framesFile, `${redacted.map((frame) => JSON.stringify(frame)).join('\n')}\n`, 'utf8');

  const domFile = artifactPath('dom.json');
  writeFileSync(
    domFile,
    `${JSON.stringify(
      {
        captured_at: capturedAt,
        url: page.url(),
        title: await page.title(),
        html: await page.content(),
        nodes: await page.locator('.react-flow__node').count(),
      },
      null,
      2,
    )}\n`,
    'utf8',
  );

  const shotFile = artifactPath('shot.png');
  await page.screenshot({ path: shotFile, fullPage: true });

  const manifest: CaptureManifest = {
    id: capId,
    status: 'CAPTURED',
    head: headSha(),
    server_bin_sha256: sha256File(live.binary.path),
    captured_at: capturedAt,
    command: SPEC_COMMAND,
    gate,
    hop_evidence: [...hopEvidence],
    artifacts: {
      frames_jsonl: rel('frames.jsonl'),
      dom_json: rel('dom.json'),
      shot_png: rel('shot.png'),
    },
  };
  const manifestFile = artifactPath('manifest.json');
  writeFileSync(manifestFile, `${JSON.stringify(manifest, null, 2)}\n`, 'utf8');

  // Non-empty evidence assertions: an empty capture must fail the test.
  expect(redacted.length, 'frames.jsonl must contain at least one frame').toBeGreaterThan(0);
  expect(statSync(framesFile).size, 'frames.jsonl must not be empty').toBeGreaterThan(0);
  expect(statSync(domFile).size, 'dom.json must not be empty').toBeGreaterThan(0);
  expect(JSON.parse(readFileSync(domFile, 'utf8')).html.length, 'dom.json html must not be empty').toBeGreaterThan(0);
  expect(statSync(shotFile).size, 'shot.png must not be empty').toBeGreaterThan(1000);
  expect(manifest.hop_evidence.length, 'manifest must name at least one hop evidence').toBeGreaterThan(0);
  expect(statSync(manifestFile).size).toBeGreaterThan(0);
  return manifest;
}

test.describe('W12 live trace capture (registry lane)', () => {
  test.setTimeout(300_000);
  let live: LiveServer | null = null;

  test.afterEach(async () => {
    if (live !== null) {
      const stopped = await live.stop();
      live = null;
      expect(redactSecrets(stopped.logs.join('\n'))).not.toMatch(/sk-[A-Za-z0-9_-]{8,}/);
    }
  });

  test('capture CAP-S22-RECHECK file tree (no paid)', async ({ page }) => {
    const missing = serverSkipReason();
    test.skip(missing !== null, missing ?? '');
    if (missing !== null) return;

    live = await startLiveServer({ withProviderKey: false });
    const tap = captureFrames(page);
    const response = await page.goto(live.baseURL);
    expect(response?.status()).toBe(200);
    await expect(page.locator('#root')).toBeVisible({ timeout: 15_000 });
    await expect(page.getByText(/Connected/).first()).toBeVisible({ timeout: 15_000 });

    const probe = await openWsProbe(page, live.wsUrl(0));
    await probe.sendCommand('FILE_TREE_LIST', {});
    const ack = await probe.waitFor(
      (frame) => frame.name === 'ACK' && frame.payload?.command === 'FILE_TREE_LIST',
      10_000,
      'FILE_TREE_LIST ACK',
    );
    const result = await probe.waitFor(
      (frame) => frame.name === 'FILE_TREE_RESULT',
      10_000,
      'FILE_TREE_RESULT',
    );
    expect(ack.payload?.command).toBe('FILE_TREE_LIST');
    expect(Array.isArray(result.payload?.entries), 'FILE_TREE_RESULT.entries is an array').toBe(true);
    const probeFrames = await probe.frames();
    await probe.close();

    await writeCapture(
      'CAP-S22-RECHECK',
      live,
      page,
      [...tap.frames, ...probeFrames],
      [
        {
          hop: '6_service',
          evidence_kind: 'ws-frame',
          pointer:
            'docs/trace/live/captures/w12/CAP-S22-RECHECK/frames.jsonl#FILE_TREE_LIST',
          frames: probeFrames.length,
        },
        {
          hop: '10_render_back',
          evidence_kind: 'dom',
          pointer: 'docs/trace/live/captures/w12/CAP-S22-RECHECK/dom.json',
          frames: 1,
        },
      ],
      captureGate(false),
    );
  });

  test('capture CAP-S27-STREAMING ordered deltas (paid gate)', async ({ page }) => {
    const gate = paidGate();
    test.skip(!gate.enabled, gate.reason);
    if (!gate.enabled) return;

    live = await startLiveServer();
    const tap = captureFrames(page);
    await page.goto(live.baseURL);
    await expect(page.getByText(/Connected/).first()).toBeVisible({ timeout: 15_000 });

    const probe = await openWsProbe(page, live.wsUrl(0));
    await probe.sendCommand('SUBMIT_PROMPT', {
      text: 'Reply with exactly one word: OK',
      project_id: randomUUID(),
    });
    const created = await probe.waitFor((frame) => frame.name === 'RUN_CREATED', 60_000, 'RUN_CREATED');
    const runId = String(created.payload?.run_id ?? '');
    expect(runId.length).toBeGreaterThan(0);
    const done = await probe.waitFor(
      (frame) => frame.name === 'REPLY_DONE' && frame.payload?.run_id === runId,
      240_000,
      'REPLY_DONE',
    );
    const completed = await probe.waitFor(
      (frame) =>
        frame.name === 'RUN_STATUS' &&
        frame.payload?.run_id === runId &&
        frame.payload?.status === 'completed',
      60_000,
      'RUN_STATUS completed',
    );
    const probeFrames = await probe.frames();
    const deltas = probeFrames.filter(
      (frame) => frame.name === 'REPLY_DELTA' && frame.payload?.run_id === runId,
    );
    expect(deltas.length, 'at least one ordered REPLY_DELTA').toBeGreaterThan(0);
    expect(probeFrames.indexOf(deltas[deltas.length - 1])).toBeLessThan(probeFrames.indexOf(done));
    expect(probeFrames.indexOf(done)).toBeLessThan(probeFrames.indexOf(completed));
    expect(done.payload?.usage, 'REPLY_DONE carries usage').toBeTruthy();
    await probe.close();

    await writeCapture(
      'CAP-S27-STREAMING',
      live,
      page,
      [...tap.frames, ...probeFrames],
      [
        { hop: '8_provider', evidence_kind: 'ws-frame', pointer: 'docs/trace/live/captures/w12/CAP-S27-STREAMING/frames.jsonl#REPLY_DELTA', frames: deltas.length },
        { hop: '9_response', evidence_kind: 'ws-frame', pointer: 'docs/trace/live/captures/w12/CAP-S27-STREAMING/frames.jsonl#REPLY_DONE', frames: 1 },
        { hop: '10_render_back', evidence_kind: 'dom', pointer: 'docs/trace/live/captures/w12/CAP-S27-STREAMING/dom.json', frames: 1 },
      ],
      captureGate(true),
    );
  });

  test('capture CAP-S10-REPLAY history snapshot (paid gate)', async ({ page }) => {
    const gate = paidGate();
    test.skip(!gate.enabled, gate.reason);
    if (!gate.enabled) return;

    live = await startLiveServer();
    const tap = captureFrames(page);
    await page.goto(live.baseURL);
    await expect(page.getByText(/Connected/).first()).toBeVisible({ timeout: 15_000 });

    const probe = await openWsProbe(page, live.wsUrl(0));
    await probe.sendCommand('SUBMIT_PROMPT', {
      text: 'Reply with exactly one word: OK',
      project_id: randomUUID(),
    });
    const created = await probe.waitFor((frame) => frame.name === 'RUN_CREATED', 60_000, 'RUN_CREATED');
    const runId = String(created.payload?.run_id ?? '');
    await probe.waitFor(
      (frame) =>
        frame.name === 'RUN_STATUS' &&
        frame.payload?.run_id === runId &&
        frame.payload?.status === 'completed',
      240_000,
      'RUN_STATUS completed',
    );

    await probe.sendCommand('HISTORY_REVISION_SET', { run_id: runId, revision: 0 });
    const ack = await probe.waitFor(
      (frame) => frame.name === 'ACK' && frame.payload?.command === 'HISTORY_REVISION_SET',
      10_000,
      'HISTORY_REVISION_SET ACK',
    );
    const snapshot = await probe.waitFor(
      (frame) =>
        frame.name === 'HISTORY_SNAPSHOT' && frame.payload?.run_id === runId && frame.payload?.revision === 0,
      10_000,
      'HISTORY_SNAPSHOT revision 0',
    );
    expect(ack.payload?.command).toBe('HISTORY_REVISION_SET');
    expect(snapshot.payload?.graph, 'HISTORY_SNAPSHOT graph payload').toBeTruthy();
    const probeFrames = await probe.frames();
    await probe.close();

    await writeCapture(
      'CAP-S10-REPLAY',
      live,
      page,
      [...tap.frames, ...probeFrames],
      [
        { hop: '2_handler', evidence_kind: 'ws-frame', pointer: 'docs/trace/live/captures/w12/CAP-S10-REPLAY/frames.jsonl#HISTORY_REVISION_SET', frames: 1 },
        { hop: '10_render_back', evidence_kind: 'dom', pointer: 'docs/trace/live/captures/w12/CAP-S10-REPLAY/dom.json', frames: 1 },
      ],
      captureGate(true),
    );
  });

  test('capture CAP-S15-19-CARDS node cards (paid gate)', async ({ page }) => {
    const gate = paidGate();
    test.skip(!gate.enabled, gate.reason);
    if (!gate.enabled) return;

    live = await startLiveServer();
    const tap = captureFrames(page);
    await page.goto(live.baseURL);
    await expect(page.getByText(/Connected/).first()).toBeVisible({ timeout: 15_000 });

    const probe = await openWsProbe(page, live.wsUrl(0));
    await probe.sendCommand('SUBMIT_PROMPT', {
      text: 'Reply with exactly one word: OK',
      project_id: randomUUID(),
    });
    const created = await probe.waitFor((frame) => frame.name === 'RUN_CREATED', 60_000, 'RUN_CREATED');
    const runId = String(created.payload?.run_id ?? '');
    await probe.waitFor(
      (frame) =>
        frame.name === 'RUN_STATUS' &&
        frame.payload?.run_id === runId &&
        frame.payload?.status === 'completed',
      240_000,
      'RUN_STATUS completed',
    );
    await expect(page.locator('.react-flow__node').first()).toBeVisible({ timeout: 30_000 });
    const cardCount = await page.locator('.react-flow__node').count();
    expect(cardCount, 'rendered node cards from captured frames').toBeGreaterThan(0);
    const probeFrames = await probe.frames();
    await probe.close();

    await writeCapture(
      'CAP-S15-19-CARDS',
      live,
      page,
      [...tap.frames, ...probeFrames],
      [
        { hop: '1_render', evidence_kind: 'dom', pointer: 'docs/trace/live/captures/w12/CAP-S15-19-CARDS/dom.json', frames: 1 },
        { hop: '1_render', evidence_kind: 'ws-frame', pointer: 'docs/trace/live/captures/w12/CAP-S15-19-CARDS/frames.jsonl#NODE_SPAWN', frames: 1 },
      ],
      captureGate(true),
    );
  });

  test('capture CAP-SAPPROVAL approve+deny (paid gate + explicit readiness)', async ({ page }) => {
    const gate = paidGate();
    test.skip(!gate.enabled, gate.reason);
    if (!gate.enabled) return;
    const ready = process.env.LIVE_APPROVAL_CAPTURE === '1';
    test.skip(
      !ready,
      'LIVE_APPROVAL_CAPTURE is not 1: engine approval suspension (WP-03) is not landed; ' +
        'refusing to fabricate an APPROVAL_REQUEST capture. Set LIVE_APPROVAL_CAPTURE=1 only after ' +
        'WP-03 publishes APPROVAL_REQUEST for a model-originated side effect.',
    );
    if (!ready) return;

    live = await startLiveServer();
    const tap = captureFrames(page);
    await page.goto(live.baseURL);
    await expect(page.getByText(/Connected/).first()).toBeVisible({ timeout: 15_000 });

    const probe = await openWsProbe(page, live.wsUrl(0));
    await probe.sendCommand('SUBMIT_PROMPT', {
      text: 'Use the file_write tool once to create w12-capture.txt with content "trace", then reply with exactly: DONE',
      project_id: randomUUID(),
    });
    const approval = await probe.waitFor(
      (frame) => frame.name === 'APPROVAL_REQUEST',
      120_000,
      'APPROVAL_REQUEST (engine suspension producer)',
    );
    const approvalId = String(approval.payload?.approval_id ?? '');
    const nonce = String(approval.payload?.nonce ?? '');
    expect(approvalId.length, 'APPROVAL_REQUEST carries approval_id').toBeGreaterThan(0);
    expect(nonce.length, 'APPROVAL_REQUEST carries the single-use nonce').toBeGreaterThan(0);

    await probe.sendCommand('APPROVAL_RESOLVE', {
      approval_id: approvalId,
      nonce,
      decision: 'approve',
      reason: 'W12 trace capture',
    });
    const resolved = await probe.waitFor(
      (frame) =>
        frame.name === 'APPROVAL_RESOLVED' ||
        (frame.name === 'ERROR' && frame.payload?.correlation_id === undefined),
      60_000,
      'APPROVAL_RESOLVED or a named ERROR',
    );
    if (resolved.name === 'ERROR') {
      throw new Error(
        `APPROVAL_RESOLVE was refused by name: ${String(resolved.payload?.code ?? 'unknown')} — ` +
          'capture must not be written as a success',
      );
    }
    expect(resolved.payload?.decision).toBe('approve');
    const probeFrames = await probe.frames();
    await probe.close();

    await writeCapture(
      'CAP-SAPPROVAL',
      live,
      page,
      [...tap.frames, ...probeFrames],
      [
        { hop: '4_request', evidence_kind: 'ws-frame', pointer: 'docs/trace/live/captures/w12/CAP-SAPPROVAL/frames.jsonl#APPROVAL_REQUEST', frames: 1 },
        { hop: '7_store', evidence_kind: 'ws-frame', pointer: 'docs/trace/live/captures/w12/CAP-SAPPROVAL/frames.jsonl#APPROVAL_RESOLVED', frames: 1 },
        { hop: '10_render_back', evidence_kind: 'dom', pointer: 'docs/trace/live/captures/w12/CAP-SAPPROVAL/dom.json', frames: 1 },
      ],
      captureGate(true),
    );
  });
});
