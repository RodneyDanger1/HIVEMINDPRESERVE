import { test, expect, type Page } from '@playwright/test';
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
import { captureFrames, shot, writeEvidence, type FrameRecord } from '../helpers/evidence';

/**
 * RUN-3+ G-UI-10 / W2 reconnect + replay proof (real Rust server).
 *
 * Truth pinned from the server code:
 * - `hello{last_seq}` is answered with `SERVER_HELLO` and then exactly the buffered
 *   events with `seq > last_seq` (`core/src/bus.rs::replay_after`,
 *   `core/src/ws.rs::send_hello_and_replay`), oldest first; replay capacity is 256.
 * - A `last_seq` older than the retained buffer (or a future `last_seq`) yields one
 *   `RESYNC_REQUIRED { expected_seq, last_seq }` followed by `SERVER_HELLO`; the
 *   stream then resumes from the current sequence. No state is mutated by a replays.
 * - A reconnect therefore must never duplicate an event/sequence: the spec asserts
 *   strictly increasing unique `seq` and, on the paid path, that the assembled
 *   `REPLY_DELTA` text contains the expected token exactly once across the two sockets.
 *
 * Keyless tests run today (no provider key, no paid call). The mid-stream test is
 * paid: it is gated by `paidGate('prompt')` (`LIVE_ONE_PROMPT=1` + key) BEFORE any
 * server start, otherwise it is a named skip; the gate-negative test starts nothing.
 */

interface HealthProbe {
  readonly status?: string;
  readonly transport?: { readonly last_seq?: number; readonly allowed_origins?: number };
}

interface HistoryRuns {
  readonly runs: unknown[];
}

interface RawProbe {
  frames(): Promise<ProbeEnvelope[]>;
  waitFor(
    predicate: (envelope: ProbeEnvelope) => boolean,
    timeoutMs?: number,
    label?: string,
  ): Promise<ProbeEnvelope>;
  close(): Promise<void>;
}

/**
 * Opens a second page WebSocket with an explicit `hello{last_seq}` (the shared
 * `openWsProbe` helper always reports 0, which is the fresh-connect case). This is
 * spec-local on purpose: the helper additions for this wave are limited to
 * `paidGate(mode)` and `startLiveServer({ allowedOrigins })`.
 */
async function openRawProbe(page: Page, url: string, lastSeq: number): Promise<RawProbe> {
  await page.evaluate(
    ({ target, reported }: { target: string; reported: number }) => {
      const w = window as unknown as {
        __w4Frames: unknown[];
        __w4Ws?: WebSocket;
        __w4Closed?: boolean;
      };
      w.__w4Frames = [];
      w.__w4Closed = false;
      const socket = new WebSocket(target);
      w.__w4Ws = socket;
      socket.addEventListener('message', (event) => {
        try {
          w.__w4Frames.push(JSON.parse(String(event.data)) as unknown);
        } catch {
          w.__w4Frames.push({ raw: String(event.data) });
        }
      });
      socket.addEventListener('open', () => {
        socket.send(JSON.stringify({ type: 'hello', last_seq: reported }));
      });
      socket.addEventListener('close', () => {
        w.__w4Closed = true;
      });
    },
    { target: url, reported: lastSeq },
  );

  const readFrames = async (): Promise<ProbeEnvelope[]> =>
    page.evaluate(() => {
      const w = window as unknown as { __w4Frames?: unknown[] };
      return (w.__w4Frames ?? []) as ProbeEnvelope[];
    });

  return {
    frames: readFrames,
    async waitFor(
      predicate: (envelope: ProbeEnvelope) => boolean,
      timeoutMs = 15_000,
      label = 'matching envelope',
    ): Promise<ProbeEnvelope> {
      const deadline = Date.now() + timeoutMs;
      let seen: string[] = [];
      while (Date.now() < deadline) {
        const frames = await readFrames();
        const hit = frames.find(
          (frame) => typeof frame === 'object' && frame !== null && predicate(frame),
        );
        if (hit !== undefined) return hit;
        seen = frames.map((frame) => String(frame.name ?? 'unknown')).slice(-12);
        await new Promise((resolve) => setTimeout(resolve, 50));
      }
      throw new Error(
        `LIVE_WS_TIMEOUT: no ${label} within ${timeoutMs}ms; recent event names=${JSON.stringify(seen)}`,
      );
    },
    async close(): Promise<void> {
      await page.evaluate(() => {
        const w = window as unknown as { __w4Ws?: WebSocket };
        try {
          w.__w4Ws?.close();
        } catch {
          // already closed: teardown is not evidence
        }
      });
    },
  };
}

function seqOf(frame: ProbeEnvelope): number {
  return typeof frame.seq === 'number' ? frame.seq : -1;
}

function numbered(frames: ProbeEnvelope[]): ProbeEnvelope[] {
  return frames.filter((frame) => typeof frame.seq === 'number').sort((a, b) => seqOf(a) - seqOf(b));
}

function stringField(value: unknown): string {
  return typeof value === 'string' ? value : '';
}

function runIdOf(frame: ProbeEnvelope): string {
  return stringField(frame.payload?.run_id) || stringField(frame.run_id);
}

async function historyRunCount(live: LiveServer): Promise<number> {
  const runs = await live.fetchJson<HistoryRuns>('/api/history/runs');
  return runs.runs.length;
}

/**
 * Installs a page-init spy around the native WebSocket constructor so the test can
 * close the app's *own* socket mid-stream and count reconnects. The app uses
 * `partysocket` (reconnecting wrapper), so a forced close must produce a second
 * socket that re-sends `hello{last_seq}` — exactly the UI reconnect under test.
 */
async function installAppSocketSpy(page: Page): Promise<void> {
  await page.addInitScript(() => {
    const sockets: WebSocket[] = [];
    class SpiedWebSocket extends WebSocket {
      constructor(url: string | URL, protocols?: string | string[]) {
        super(url, protocols);
        sockets.push(this);
        (window as unknown as { __hiveSockets?: WebSocket[] }).__hiveSockets = sockets;
      }
    }
    (window as unknown as { WebSocket: typeof WebSocket }).WebSocket = SpiedWebSocket;
  });
}

async function appSocketCount(page: Page): Promise<number> {
  return page.evaluate(() => {
    const w = window as unknown as { __hiveSockets?: unknown[] };
    return Array.isArray(w.__hiveSockets) ? w.__hiveSockets.length : 0;
  });
}

/** `hello{last_seq}` values sent by the app sockets (captured by `captureFrames`). */
function helloSequences(frames: FrameRecord[]): number[] {
  const sequences: number[] = [];
  for (const frame of frames) {
    if (frame.dir !== 'sent') continue;
    try {
      const parsed = JSON.parse(frame.payload) as { type?: unknown; last_seq?: unknown };
      if (parsed.type === 'hello' && typeof parsed.last_seq === 'number') {
        sequences.push(parsed.last_seq);
      }
    } catch {
      // ignore non-JSON frames
    }
  }
  return sequences;
}

async function chatCoreMarkerCount(page: Page, marker: string): Promise<number> {
  const text = await page.locator('section[aria-label="HyperAgent hive chat core"]').innerText();
  return text.split(marker).length - 1;
}

test.describe('G-UI-10/W2 reconnect + replay (live server)', () => {
  test.setTimeout(420_000);

  let live: LiveServer | null = null;

  test.afterEach(async () => {
    if (live !== null) {
      const stopped = await live.stop();
      live = null;
      expect(redactSecrets(stopped.logs.join('\n'))).not.toMatch(/sk-[A-Za-z0-9_-]{8,}/);
    }
  });

  test('gate negative: resync paid lane refuses without LIVE_ONE_PROMPT=1 (no server, no spend)', async () => {
    const missingServer = serverSkipReason();
    const previousPrompt = process.env.LIVE_ONE_PROMPT;
    const previousToolTurn = process.env.LIVE_TOOL_TURN;
    delete process.env.LIVE_ONE_PROMPT;
    delete process.env.LIVE_TOOL_TURN;
    try {
      const gate = paidGate('prompt');
      expect(gate.enabled, 'resync paid gate must fail closed').toBe(false);
      expect(gate.reason.length).toBeGreaterThan(0);
      if (missingServer === null) {
        expect(gate.reason).toContain('LIVE_ONE_PROMPT');
      }
    } finally {
      if (previousPrompt === undefined) delete process.env.LIVE_ONE_PROMPT;
      else process.env.LIVE_ONE_PROMPT = previousPrompt;
      if (previousToolTurn === undefined) delete process.env.LIVE_TOOL_TURN;
      else process.env.LIVE_TOOL_TURN = previousToolTurn;
    }
    // No startLiveServer()/submit: the gate check is the whole proof.
  });

  test('keyless reconnect: hello{last_seq} replays exactly the later events (strictly increasing, unique seq)', async ({
    page,
  }) => {
    const missing = serverSkipReason();
    test.skip(missing !== null, missing ?? '');
    if (missing !== null) return;
    live = await startLiveServer({ withProviderKey: false });
    await page.goto(live.baseURL);
    await expect(page.getByText(/Connected/).first()).toBeVisible({ timeout: 15_000 });

    const probe = await openWsProbe(page, live.wsUrl(0));
    await probe.sendCommand('SUBMIT_PROMPT', { text: 'keyless replay probe', project_id: randomUUID() });
    await probe.waitFor(
      (frame) => frame.name === 'RUN_STATUS' && frame.payload?.status === 'failed',
      30_000,
      'keyless RUN_STATUS failed terminal',
    );
    const original = await probe.frames();
    const originalNumbered = numbered(original);
    expect(originalNumbered.length, 'a keyless run still publishes sequenced events').toBeGreaterThanOrEqual(2);
    const firstSeq = seqOf(originalNumbered[0]!);
    const maxSeq = seqOf(originalNumbered[originalNumbered.length - 1]!);
    expect(firstSeq).toBeGreaterThan(0);
    expect(maxSeq).toBeGreaterThanOrEqual(firstSeq);
    await probe.close();
    const runsBefore = await historyRunCount(live);

    // Reconnect from a middle sequence: replay must be exactly the original seqs > reported.
    const reported = firstSeq + Math.floor((maxSeq - firstSeq) / 2);
    const replay = await openRawProbe(page, live.wsUrl(0), reported);
    await replay.waitFor((frame) => frame.name === 'SERVER_HELLO', 10_000, 'SERVER_HELLO');
    await expect.poll(async () => numbered(await replay.frames()).length, { timeout: 10_000 }).toBeGreaterThan(0);
    const replayed = numbered(await replay.frames());
    const replayedSeqs = replayed.map(seqOf);
    const expected = originalNumbered.map(seqOf).filter((seq) => seq > reported);
    expect(replayedSeqs, 'replay = exactly the buffered events after last_seq').toEqual(expected);
    expect(new Set(replayedSeqs).size, 'no duplicate sequence in a reconnect').toBe(replayedSeqs.length);
    for (let index = 1; index < replayedSeqs.length; index += 1) {
      expect(replayedSeqs[index], 'replay is strictly increasing').toBeGreaterThan(replayedSeqs[index - 1]!);
    }
    await replay.close();

    // A reconnect at the current sequence is a no-op replay: SERVER_HELLO only, zero numbered events.
    const current = await openRawProbe(page, live.wsUrl(0), maxSeq);
    await current.waitFor((frame) => frame.name === 'SERVER_HELLO', 10_000, 'SERVER_HELLO at current seq');
    await new Promise((resolve) => setTimeout(resolve, 300));
    const currentNumbered = numbered(await current.frames());
    expect(currentNumbered, 'no events are replayed when the client is current').toEqual([]);
    await current.close();

    const runsAfter = await historyRunCount(live);
    expect(runsAfter, 'replay must not mutate run history').toBe(runsBefore);
    writeEvidence('live-resync-keyless.json', {
      captured_at: new Date().toISOString(),
      release_binary: process.env.E2E_SERVER_BIN ?? null,
      binary: live.binary,
      reported,
      replayed_seqs: replayedSeqs,
      original_seqs: originalNumbered.map(seqOf),
      runs_before: runsBefore,
      runs_after: runsAfter,
    });
  });

  test('future last_seq produces one RESYNC_REQUIRED then SERVER_HELLO, with no mutation (keyless)', async ({
    page,
  }) => {
    const missing = serverSkipReason();
    test.skip(missing !== null, missing ?? '');
    if (missing !== null) return;
    live = await startLiveServer({ withProviderKey: false });
    await page.goto(live.baseURL);
    await expect(page.getByText(/Connected/).first()).toBeVisible({ timeout: 15_000 });
    const health = await live.fetchJson<HealthProbe>('/health');
    const currentSeq = health.transport?.last_seq ?? 0;
    const futureSeq = currentSeq + 50;
    const runsBefore = await historyRunCount(live);

    const probe = await openRawProbe(page, live.wsUrl(0), futureSeq);
    const resync = await probe.waitFor((frame) => frame.name === 'RESYNC_REQUIRED', 15_000, 'RESYNC_REQUIRED');
    expect(resync.payload?.expected_seq).toBe(currentSeq + 1);
    expect(resync.payload?.last_seq).toBe(futureSeq);
    const hello = await probe.waitFor(
      (frame) => frame.name === 'SERVER_HELLO',
      15_000,
      'SERVER_HELLO after RESYNC_REQUIRED',
    );
    expect(hello.payload?.schema).toBe('hiverouter.v1');
    const numberedAfterGap = numbered(await probe.frames());
    expect(numberedAfterGap, 'a gap asks for a resnapshot instead of replaying events').toEqual([]);
    await probe.close();

    const runsAfter = await historyRunCount(live);
    expect(runsAfter, 'a gap refusal must not mutate run history').toBe(runsBefore);
    await expect(page.locator('#composer')).toBeVisible();
    writeEvidence('live-resync-gap.json', {
      captured_at: new Date().toISOString(),
      release_binary: process.env.E2E_SERVER_BIN ?? null,
      binary: live.binary,
      current_seq: currentSeq,
      reported_future_seq: futureSeq,
      expected_seq: resync.payload?.expected_seq ?? null,
      resync_last_seq: resync.payload?.last_seq ?? null,
      runs_before: runsBefore,
      runs_after: runsAfter,
    });
  });

  test('paid: app socket drop mid-stream + hello{last_seq} reconnect, no duplicated reply text (paid gate)', async ({
    page,
  }, testInfo) => {
    const missing = serverSkipReason();
    test.skip(missing !== null, missing ?? '');
    if (missing !== null) return;
    const gate = paidGate('prompt');
    test.skip(!gate.enabled, gate.reason);
    if (!gate.enabled) return;

    const pageFrames = captureFrames(page);
    await installAppSocketSpy(page);
    live = await startLiveServer({ withProviderKey: true });
    await page.goto(live.baseURL);
    await expect(page.getByText(/Connected/).first()).toBeVisible({ timeout: 15_000 });
    await expect.poll(async () => appSocketCount(page), { timeout: 30_000 }).toBeGreaterThanOrEqual(1);

    const probe = await openWsProbe(page, live.wsUrl(0));
    await probe.sendCommand('SUBMIT_PROMPT', {
      // The exact marker appears only in the assistant reply (the prompt asks for a
      // join), so a UI replay bug would make the marker render twice in the chat core.
      text:
        'Output exactly the token formed by joining "HIVE-" and "RESYNC-OK" with no separator, ' +
        'and nothing else.',
      project_id: randomUUID(),
    });
    const created = await probe.waitFor((frame) => frame.name === 'RUN_CREATED', 60_000, 'RUN_CREATED');
    const runId = stringField(created.payload?.run_id);
    expect(runId.length).toBeGreaterThan(0);
    await probe.waitFor(
      (frame) =>
        frame.name === 'REPLY_DELTA' &&
        runIdOf(frame) === runId &&
        typeof frame.payload?.text_delta === 'string',
      240_000,
      'first REPLY_DELTA',
    );

    // Force the app's own socket closed mid-stream. partysocket must reconnect and
    // re-send hello{last_seq}; the server then replays only the missing events.
    const socketsBefore = await appSocketCount(page);
    await page.evaluate(() => {
      const w = window as unknown as { __hiveSockets?: WebSocket[] };
      const sockets = w.__hiveSockets ?? [];
      sockets[sockets.length - 1]?.close(4000, 'e2e forced reconnect');
    });
    await expect
      .poll(async () => appSocketCount(page), { timeout: 60_000 })
      .toBeGreaterThanOrEqual(socketsBefore + 1);
    await expect(page.getByText(/Connected/).first()).toBeVisible({ timeout: 30_000 });
    // The reconnected app socket must send a non-zero hello{last_seq} so the server
    // replays only the missed events. Poll the captured sent frames for it.
    await expect
      .poll(async () => helloSequences(pageFrames.frames).filter((lastSeq) => lastSeq > 0).length, {
        timeout: 30_000,
      })
      .toBeGreaterThanOrEqual(1);
    const hellos = helloSequences(pageFrames.frames);
    expect(hellos.some((lastSeq) => lastSeq > 0), `hello last_seq history=${JSON.stringify(hellos)}`).toBe(true);

    await probe.waitFor(
      (frame) =>
        frame.name === 'RUN_STATUS' && runIdOf(frame) === runId && frame.payload?.status === 'completed',
      240_000,
      'RUN_STATUS completed after the app reconnect',
    );
    await probe.waitFor(
      (frame) => frame.name === 'REPLY_DONE' && runIdOf(frame) === runId,
      60_000,
      'REPLY_DONE for the run',
    );
    const probeFrames = await probe.frames();
    expect(
      probeFrames.filter(
        (frame) => frame.name === 'RUN_STATUS' && runIdOf(frame) === runId && frame.payload?.status === 'failed',
      ),
      'no failed terminal after reconnect',
    ).toEqual([]);
    expect(
      probeFrames.filter((frame) => frame.name === 'RESYNC_REQUIRED'),
      'a small replay gap must not force a resnapshot',
    ).toEqual([]);
    const completedEvents = probeFrames.filter(
      (frame) =>
        frame.name === 'RUN_STATUS' && runIdOf(frame) === runId && frame.payload?.status === 'completed',
    );
    expect(completedEvents, 'exactly one terminal completion on the live socket').toHaveLength(1);

    const marker = 'HIVE-RESYNC-OK';
    await expect
      .poll(async () => chatCoreMarkerCount(page, marker), { timeout: 30_000 })
      .toBe(1);
    await expect(page.locator('#composer')).toBeEnabled();

    const appSocketsAfter = await appSocketCount(page);
    const screenshot = await shot(page, 'live-resync', testInfo);
    writeEvidence(
      'live-resync.json',
      JSON.parse(
        redactSecrets(
          JSON.stringify({
            captured_at: new Date().toISOString(),
            release_binary: process.env.E2E_SERVER_BIN ?? null,
            binary: live.binary,
            run_id: runId,
            app_sockets_before: socketsBefore,
            app_sockets_after: appSocketsAfter,
            hello_last_seq_history: hellos,
            marker,
            marker_occurrences_in_chat: 1,
            completed_events: completedEvents.length,
            resync_required_frames: 0,
            probe_frames: probeFrames,
            app_socket_frames: pageFrames.frames,
            screenshot,
          }),
        ),
      ) as unknown,
    );
    await probe.close();
  });
});
