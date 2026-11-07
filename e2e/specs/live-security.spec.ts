import { test, expect, type Page } from '@playwright/test';
import { randomBytes } from 'node:crypto';
import { createServer, type Server } from 'node:http';
import { connect } from 'node:net';
import {
  redactSecrets,
  serverSkipReason,
  startLiveServer,
  type LiveServer,
} from '../helpers/live-server';
import { shot, writeEvidence } from '../helpers/evidence';

/**
 * RUN-3+ G-UI-3 / W1 origin + token boundary proof (real Rust server, keyless).
 *
 * This spec never makes a paid provider call: every test starts the server with
 * `withProviderKey: false` (the child env is stripped of DEEPSEEK_API_KEY) and only
 * performs WebSocket handshakes / connects a foreign-origin page.
 *
 * Observable truth pinned by the server code (`core/src/ws.rs`):
 * - `GET /ws` checks loopback peer, then the exact Origin allowlist, then the
 *   per-boot token, all BEFORE the upgrade: a foreign Origin gets HTTP 403 with
 *   JSON `{ code: "E_ORIGIN_REJECTED" }`; a wrong token gets HTTP 401 with
 *   `{ code: "E_TOKEN_INVALID" }`. Therefore the spec asserts the pre-upgrade HTTP
 *   refusal (a browser WebSocket hides the status and only fires `error`); the
 *   browser test asserts the observable "cannot open" side of the same boundary.
 * - Default allowlist is derived from the bind port (`http://127.0.0.1:<port>` and
 *   `http://localhost:<port>`); `HIVEROUTER_ALLOWED_ORIGINS` appends extras. The
 *   `startLiveServer({ allowedOrigins })` option is additive and is proven by the
 *   option test (foreign page origin becomes allowed only when passed).
 * - "No mutation" is asserted with `/api/history/runs` before/after (runs array
 *   unchanged) and with `/health` staying `ok`/`degraded` with an unchanged
 *   allowlist size: a refused handshake creates no run/session state.
 */

interface RawHandshake {
  readonly statusLine: string;
  readonly status: number;
  readonly headers: string;
  readonly body: string;
}

interface AttackerPage {
  readonly url: string;
  readonly origin: string;
  close(): Promise<void>;
}

interface HistoryRuns {
  readonly runs: unknown[];
}

interface HealthProbe {
  readonly status?: string;
  readonly transport?: { readonly allowed_origins?: number; readonly last_seq?: number };
}

function parseHttpResponse(buffer: Buffer): RawHandshake {
  const text = buffer.toString('latin1');
  const split = text.indexOf('\r\n\r\n');
  const header = split >= 0 ? text.slice(0, split) : text;
  const statusLine = header.split('\r\n')[0] ?? '';
  const status = Number(statusLine.split(' ')[1] ?? '0');
  const body = split >= 0 ? text.slice(split + 4) : '';
  return { statusLine, status, headers: header, body };
}

function refusalCode(handshake: RawHandshake): string {
  try {
    const parsed = JSON.parse(handshake.body) as { code?: unknown };
    return typeof parsed.code === 'string' ? parsed.code : '';
  } catch {
    return '';
  }
}

/**
 * Raw HTTP/1.1 WebSocket handshake from the test process (not the browser), so the
 * pre-upgrade HTTP status and JSON refusal body are observable. `Origin` is emitted
 * only when provided (missing-Origin case).
 */
function rawWsHandshake(
  port: number,
  options: { readonly token?: string; readonly origin?: string },
): Promise<RawHandshake> {
  const path = options.token === undefined ? '/ws' : `/ws?token=${encodeURIComponent(options.token)}`;
  const lines = [
    `GET ${path} HTTP/1.1`,
    `Host: 127.0.0.1:${port}`,
    'Upgrade: websocket',
    'Connection: Upgrade',
    `Sec-WebSocket-Key: ${randomBytes(16).toString('base64')}`,
    'Sec-WebSocket-Version: 13',
    ...(options.origin === undefined ? [] : [`Origin: ${options.origin}`]),
    'Accept: */*',
    '',
    '',
  ];
  const request = lines.join('\r\n');
  return new Promise<RawHandshake>((resolve, reject) => {
    const socket = connect({ host: '127.0.0.1', port });
    let received = Buffer.alloc(0);
    let settled = false;
    let timer: NodeJS.Timeout | null = null;
    const finish = (): void => {
      if (settled) return;
      settled = true;
      if (timer !== null) clearTimeout(timer);
      socket.destroy();
      resolve(parseHttpResponse(received));
    };
    timer = setTimeout(finish, 5_000);
    socket.on('connect', () => socket.write(request));
    socket.on('data', (chunk: Buffer) => {
      received = Buffer.concat([received, chunk]);
      const parsed = parseHttpResponse(received);
      const split = received.toString('latin1').indexOf('\r\n\r\n');
      if (split < 0) return;
      if (parsed.status === 101) {
        finish();
        return;
      }
      const contentLength = /content-length:\s*(\d+)/i.exec(parsed.headers);
      const expected = contentLength === null ? null : Number(contentLength[1]);
      if (expected !== null ? parsed.body.length >= expected : parsed.body.length > 0) finish();
    });
    socket.on('error', (error: Error) => {
      if (settled) return;
      settled = true;
      if (timer !== null) clearTimeout(timer);
      reject(error);
    });
    socket.on('close', finish);
  });
}

async function startAttackerPage(): Promise<AttackerPage> {
  const server: Server = createServer((_request, response) => {
    response.writeHead(200, {
      'content-type': 'text/html; charset=utf-8',
      'cache-control': 'no-store',
    });
    response.end(
      '<!doctype html><html><head><title>foreign origin</title></head>' +
        '<body><div id="root">foreign loopback origin page</div></body></html>',
    );
  });
  await new Promise<void>((resolve, reject) => {
    server.once('error', reject);
    server.listen(0, '127.0.0.1', () => resolve());
  });
  const address = server.address();
  if (address === null || typeof address === 'string') {
    await new Promise<void>((resolve) => server.close(() => resolve()));
    throw new Error('FOREIGN_PAGE_LISTEN_FAILED: could not read the loopback port');
  }
  const origin = `http://127.0.0.1:${address.port}`;
  return {
    origin,
    url: `${origin}/`,
    close: async () => {
      await new Promise<void>((resolve, reject) => {
        server.close((error) => (error === undefined ? resolve() : reject(error)));
      });
    },
  };
}

interface BrowserSocketOutcome {
  readonly opened: boolean;
  readonly errored: boolean;
  readonly closed: boolean;
}

/** Opens a real browser WebSocket from the page's current origin; reports only observable events. */
async function browserSocketAttempt(page: Page, url: string): Promise<BrowserSocketOutcome> {
  return page.evaluate(
    (target: string) =>
      new Promise<{ opened: boolean; errored: boolean; closed: boolean }>((resolve) => {
        const state = { opened: false, errored: false, closed: false };
        let socket: WebSocket | null = null;
        const finish = (): void => {
          try {
            socket?.close();
          } catch {
            // close is teardown, not evidence
          }
          resolve({ ...state });
        };
        try {
          socket = new WebSocket(target);
        } catch {
          state.errored = true;
          finish();
          return;
        }
        if (socket === null) {
          state.errored = true;
          finish();
          return;
        }
        const opened: WebSocket = socket;
        opened.addEventListener('open', () => {
          state.opened = true;
          finish();
        });
        opened.addEventListener('error', () => {
          state.errored = true;
          finish();
        });
        opened.addEventListener('close', () => {
          state.closed = true;
          finish();
        });
        setTimeout(finish, 5_000);
      }),
    url,
  );
}

async function historyRuns(live: LiveServer): Promise<HistoryRuns> {
  return live.fetchJson<HistoryRuns>('/api/history/runs');
}

async function healthProbe(live: LiveServer): Promise<HealthProbe> {
  return live.fetchJson<HealthProbe>('/health');
}

test.describe('G-UI-3/W1 origin + token boundaries (live server, keyless)', () => {
  test.setTimeout(180_000);

  let live: LiveServer | null = null;
  let attacker: AttackerPage | null = null;

  test.afterEach(async () => {
    if (attacker !== null) {
      await attacker.close();
      attacker = null;
    }
    if (live !== null) {
      const stopped = await live.stop();
      live = null;
      expect(redactSecrets(stopped.logs.join('\n'))).not.toMatch(/sk-[A-Za-z0-9_-]{8,}/);
    }
  });

  function skipWithoutServer(): string | null {
    return serverSkipReason();
  }

  test('positive control: allowlisted loopback Origin + valid token upgrades (HTTP 101, no paid call)', async () => {
    const missing = skipWithoutServer();
    test.skip(missing !== null, missing ?? '');
    if (missing !== null) return;
    live = await startLiveServer({ withProviderKey: false });
    const response = await rawWsHandshake(live.port, {
      token: live.session.token,
      origin: `http://127.0.0.1:${live.port}`,
    });
    expect(response.status, `statusLine=${response.statusLine}`).toBe(101);
    writeEvidence('live-security-positive.json', {
      captured_at: new Date().toISOString(),
      release_binary: process.env.E2E_SERVER_BIN ?? null,
      binary: live.binary,
      status: response.status,
      status_line: response.statusLine,
      origin: `http://127.0.0.1:${live.port}`,
    });
  });

  test('foreign Origin is refused before upgrade as HTTP 403 E_ORIGIN_REJECTED and mutates nothing', async () => {
    const missing = skipWithoutServer();
    test.skip(missing !== null, missing ?? '');
    if (missing !== null) return;
    const foreign = await startAttackerPage();
    attacker = foreign;
    live = await startLiveServer({ withProviderKey: false });
    const before = await historyRuns(live);
    const healthBefore = await healthProbe(live);
    const response = await rawWsHandshake(live.port, {
      token: live.session.token,
      origin: foreign.origin,
    });
    expect(response.status, `statusLine=${response.statusLine}`).toBe(403);
    expect(refusalCode(response), `body=${response.body}`).toBe('E_ORIGIN_REJECTED');
    const after = await historyRuns(live);
    expect(after.runs, 'a refused handshake must not create run/session state').toEqual(before.runs);
    const healthAfter = await healthProbe(live);
    expect(['ok', 'degraded']).toContain(healthAfter.status);
    expect(healthAfter.transport?.allowed_origins).toBe(healthBefore.transport?.allowed_origins);
    writeEvidence('live-security-foreign-origin.json', {
      captured_at: new Date().toISOString(),
      release_binary: process.env.E2E_SERVER_BIN ?? null,
      binary: live.binary,
      attacker_origin: foreign.origin,
      status: response.status,
      status_line: response.statusLine,
      body: JSON.parse(redactSecrets(response.body)) as unknown,
      runs_before: before.runs.length,
      runs_after: after.runs.length,
      allowed_origins_before: healthBefore.transport?.allowed_origins ?? null,
      allowed_origins_after: healthAfter.transport?.allowed_origins ?? null,
    });
  });

  test('missing Origin is refused as HTTP 403 E_ORIGIN_REJECTED', async () => {
    const missing = skipWithoutServer();
    test.skip(missing !== null, missing ?? '');
    if (missing !== null) return;
    live = await startLiveServer({ withProviderKey: false });
    const response = await rawWsHandshake(live.port, { token: live.session.token });
    expect(response.status, `statusLine=${response.statusLine}`).toBe(403);
    expect(refusalCode(response), `body=${response.body}`).toBe('E_ORIGIN_REJECTED');
    writeEvidence('live-security-missing-origin.json', {
      captured_at: new Date().toISOString(),
      release_binary: process.env.E2E_SERVER_BIN ?? null,
      status: response.status,
      status_line: response.statusLine,
      body: JSON.parse(redactSecrets(response.body)) as unknown,
    });
  });

  test('wrong token is refused as HTTP 401 E_TOKEN_INVALID with an allowlisted Origin', async () => {
    const missing = skipWithoutServer();
    test.skip(missing !== null, missing ?? '');
    if (missing !== null) return;
    live = await startLiveServer({ withProviderKey: false });
    const response = await rawWsHandshake(live.port, {
      token: `${live.session.token}-wrong`,
      origin: `http://127.0.0.1:${live.port}`,
    });
    expect(response.status, `statusLine=${response.statusLine}`).toBe(401);
    expect(refusalCode(response), `body=${response.body}`).toBe('E_TOKEN_INVALID');
    const after = await historyRuns(live);
    expect(after.runs, 'a bad token must not create run/session state').toEqual([]);
    writeEvidence('live-security-bad-token.json', {
      captured_at: new Date().toISOString(),
      release_binary: process.env.E2E_SERVER_BIN ?? null,
      status: response.status,
      status_line: response.statusLine,
      body: JSON.parse(redactSecrets(response.body)) as unknown,
      runs_after: after.runs.length,
    });
  });

  test('a foreign-origin browser page cannot open a WebSocket (open=false, error=true)', async ({
    page,
  }, testInfo) => {
    const missing = skipWithoutServer();
    test.skip(missing !== null, missing ?? '');
    if (missing !== null) return;
    const foreign = await startAttackerPage();
    attacker = foreign;
    live = await startLiveServer({ withProviderKey: false });
    await page.goto(foreign.url);
    expect(new URL(page.url()).origin).toBe(foreign.origin);
    const outcome = await browserSocketAttempt(page, live.wsUrl(0));
    expect(outcome.opened, 'foreign Origin must not upgrade in a real browser').toBe(false);
    expect(outcome.errored, 'the browser observes a failed handshake').toBe(true);
    const after = await historyRuns(live);
    expect(after.runs).toEqual([]);
    const screenshot = await shot(page, 'live-security-foreign-browser', testInfo);
    writeEvidence('live-security-foreign-browser.json', {
      captured_at: new Date().toISOString(),
      release_binary: process.env.E2E_SERVER_BIN ?? null,
      page_origin: foreign.origin,
      server_origin_allowlist: [`http://127.0.0.1:${live.port}`, `http://localhost:${live.port}`],
      outcome,
      runs_after: after.runs.length,
      screenshot,
    });
  });

  test('allowedOrigins option appends a foreign loopback origin (HTTP 101 + browser open)', async ({
    page,
  }) => {
    const missing = skipWithoutServer();
    test.skip(missing !== null, missing ?? '');
    if (missing !== null) return;
    const foreign = await startAttackerPage();
    attacker = foreign;
    live = await startLiveServer({ withProviderKey: false, allowedOrigins: [foreign.origin] });

    const health = await healthProbe(live);
    expect(health.transport?.allowed_origins, 'bind defaults (2) + one additive origin').toBe(3);

    const response = await rawWsHandshake(live.port, {
      token: live.session.token,
      origin: foreign.origin,
    });
    expect(response.status, `statusLine=${response.statusLine}`).toBe(101);

    await page.goto(foreign.url);
    const outcome = await browserSocketAttempt(page, live.wsUrl(0));
    expect(outcome.opened, 'an explicitly appended origin is accepted by the real browser').toBe(true);

    writeEvidence('live-security-allowed-origin-option.json', {
      captured_at: new Date().toISOString(),
      release_binary: process.env.E2E_SERVER_BIN ?? null,
      binary: live.binary,
      appended_origin: foreign.origin,
      handshake_status: response.status,
      handshake_status_line: response.statusLine,
      allowlist_size: health.transport?.allowed_origins ?? null,
      browser_outcome: outcome,
    });
  });
});
