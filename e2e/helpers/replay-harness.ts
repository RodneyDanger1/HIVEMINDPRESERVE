import { createServer, type ServerResponse } from 'node:http';
import { existsSync, readFileSync } from 'node:fs';
import { extname, join, normalize } from 'node:path';
import { fileURLToPath } from 'node:url';
import type { Page, WebSocketRoute } from '@playwright/test';

/**
 * W4 deterministic replay harness.
 *
 * Serves the current production build (`web/dist`) from a loopback static server and
 * mocks ONLY the WebSocket transport with Playwright's `page.routeWebSocket`. This is a
 * test double at the transport boundary: no real server, no provider call, no key.
 * A test drives the exact event sequence it needs (RUN_CREATED, NODE_SPAWN, and a
 * HISTORY_SNAPSHOT) so replay state is deterministic instead of depending on a live run.
 */
export const SCHEMA_VERSION = 'hiverouter.v1';

const DIST_ROOT = fileURLToPath(new URL('../../web/dist/', import.meta.url));

const MIME: Readonly<Record<string, string>> = {
  '.html': 'text/html; charset=utf-8',
  '.js': 'text/javascript; charset=utf-8',
  '.mjs': 'text/javascript; charset=utf-8',
  '.css': 'text/css; charset=utf-8',
  '.json': 'application/json; charset=utf-8',
  '.svg': 'image/svg+xml',
  '.png': 'image/png',
  '.jpg': 'image/jpeg',
  '.ico': 'image/x-icon',
  '.woff': 'font/woff',
  '.woff2': 'font/woff2',
};

export interface DistHarness {
  readonly url: string;
  close(): Promise<void>;
}

function writeJson(res: ServerResponse, body: unknown): void {
  res.statusCode = 200;
  res.setHeader('content-type', 'application/json; charset=utf-8');
  res.end(JSON.stringify(body));
}

/** Serves `web/dist` plus the two read-only HTTP endpoints the UI boot needs. */
export async function startDistHarness(): Promise<DistHarness> {
  if (!existsSync(join(DIST_ROOT, 'index.html'))) {
    throw new Error(`replay harness: ${DIST_ROOT} has no index.html (build the web app first)`);
  }
  const server = createServer((req, res) => {
    try {
      const pathOnly = (req.url ?? '/').split('?')[0] ?? '/';
      if (pathOnly === '/api/session') {
        writeJson(res, {
          token: 'e2e-session-token',
          schema: SCHEMA_VERSION,
          last_seq: 0,
          ws: '/ws',
        });
        return;
      }
      if (pathOnly === '/health') {
        writeJson(res, {
          status: 'ok',
          schema: SCHEMA_VERSION,
          version: '1.1.0-e2e',
          checks: [{ name: 'replay-harness', ok: true }],
        });
        return;
      }
      const relative = decodeURIComponent(pathOnly === '/' ? '/index.html' : pathOnly).replace(
        /^\/+/,
        '',
      );
      const absolute = normalize(join(normalize(DIST_ROOT), relative));
      if (!absolute.startsWith(normalize(DIST_ROOT))) {
        res.statusCode = 403;
        res.end('forbidden');
        return;
      }
      if (!existsSync(absolute)) {
        res.statusCode = 404;
        res.end('not found');
        return;
      }
      res.setHeader('content-type', MIME[extname(absolute).toLowerCase()] ?? 'application/octet-stream');
      res.setHeader('cache-control', 'no-store');
      res.end(readFileSync(absolute));
    } catch (error) {
      res.statusCode = 500;
      res.end(`replay harness error: ${String(error)}`);
    }
  });
  await new Promise<void>((resolve) => {
    server.listen(0, '127.0.0.1', resolve);
  });
  const address = server.address();
  if (address === null || typeof address === 'string') {
    throw new Error('replay harness: failed to bind a loopback port');
  }
  return {
    url: `http://127.0.0.1:${address.port}`,
    close: () =>
      new Promise<void>((resolve, reject) => {
        server.close((error) => (error ? reject(error) : resolve()));
      }),
  };
}

/** Builds one server→client EVENT envelope exactly as the frozen contract requires. */
export function eventFrame(name: string, payload: unknown, seq: number): string {
  const id = `e2e-${seq}`;
  return JSON.stringify({
    schema: SCHEMA_VERSION,
    type: 'EVENT',
    name,
    id,
    correlation_id: id,
    run_id: null,
    task_id: null,
    seq,
    ts: new Date().toISOString(),
    payload,
  });
}

export interface MockWsHarness {
  /** The routed page-side socket; test code pushes events through it. */
  route(): WebSocketRoute;
  /** Every COMMAND name the page sent through the mocked socket (proves pre-mutation/no-send). */
  commands: string[];
  /** Raw frames the page sent (control hello + COMMAND envelopes), for diagnostics only. */
  sent: string[];
}

/**
 * Installs a deterministic WebSocket mock before navigation. The UI's own client code
 * (partysocket + WsApiClient) still runs unmodified; only the bytes on the wire are
 * test-controlled. COMMAND frames are acknowledged so ACK-gated send() promises settle.
 */
export async function installMockWs(page: Page): Promise<MockWsHarness> {
  let current: WebSocketRoute | null = null;
  const commands: string[] = [];
  const sent: string[] = [];
  await page.routeWebSocket(/\/ws(\?.*)?$/, (route) => {
    current = route;
    route.onMessage((message) => {
      if (typeof message !== 'string') {
        return;
      }
      sent.push(message);
      try {
        const parsed = JSON.parse(message) as { type?: string; name?: string };
        if (parsed.type === 'COMMAND' && typeof parsed.name === 'string') {
          commands.push(parsed.name);
        }
      } catch {
        // Raw control frame (hello): not a COMMAND.
      }
      try {
        const frame = JSON.parse(message) as {
          type?: string;
          name?: string;
          id?: string;
          correlation_id?: string;
        };
        if (frame.type === 'COMMAND') {
          route.send(
            JSON.stringify({
              schema: SCHEMA_VERSION,
              type: 'EVENT',
              name: 'ACK',
              id: `ack-${frame.id ?? 'unknown'}`,
              correlation_id: frame.correlation_id ?? frame.id ?? 'unknown',
              run_id: null,
              task_id: null,
              seq: null,
              ts: new Date().toISOString(),
              payload: {
                command: frame.name,
                correlation_id: frame.correlation_id ?? frame.id ?? 'unknown',
              },
            }),
          );
        }
      } catch {
        // Raw control frame (`{type:"hello"}`) or unknown bytes: not needed by this spec.
      }
    });
  });
  return {
    commands,
    sent,
    route(): WebSocketRoute {
      if (current === null) {
        throw new Error('replay harness: WebSocket route was not established by the app');
      }
      return current;
    },
  };
}
