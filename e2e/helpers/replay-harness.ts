import { createServer, type ServerResponse } from 'node:http';
import { existsSync, readFileSync } from 'node:fs';
import { extname, join, normalize } from 'node:path';
import { fileURLToPath } from 'node:url';
import type { Page, WebSocketRoute } from '@playwright/test';
import {
  TOOL_OUTCOMES,
  type ApprovalRequestPayload,
  type ToolCallPayload,
  type ToolResultPayload,
} from '../../web/src/api/contract';

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

/* ------------------------------------------------------------------ */
/* W12 tool-loop / approval helpers (reusable by every mocked-WS spec) */
/* ------------------------------------------------------------------ */

/** One COMMAND frame captured from the page through the mocked socket. */
export interface MockSentCommand {
  readonly name: string;
  /** Parsed payload when the frame is a well-formed COMMAND with object payload. */
  readonly payload: Record<string, unknown> | null;
  readonly raw: string;
  readonly at: string;
  readonly correlationId: string | null;
}

function isRecord(value: unknown): value is Record<string, unknown> {
  return typeof value === 'object' && value !== null && !Array.isArray(value);
}

function assertNonEmptyString(field: string, value: unknown): asserts value is string {
  if (typeof value !== 'string' || value.trim().length === 0) {
    throw new TypeError(`replay-harness: ${field} must be a non-empty string`);
  }
}

function assertSha256(field: string, value: unknown): asserts value is string {
  assertNonEmptyString(field, value);
  if (!/^[0-9a-f]{64}$/i.test(value)) {
    throw new TypeError(`replay-harness: ${field} must be a 64-char hex SHA-256`);
  }
}

function assertTurn(value: unknown): asserts value is number {
  if (typeof value !== 'number' || !Number.isInteger(value) || value < 1) {
    throw new TypeError('replay-harness: TOOL_CALL.turn must be an integer >= 1');
  }
}

function assertOutcome(value: unknown): void {
  if (typeof value !== 'string' || !(TOOL_OUTCOMES as readonly string[]).includes(value)) {
    throw new TypeError(
      `replay-harness: TOOL_RESULT.outcome must be one of ${TOOL_OUTCOMES.join('|')}`,
    );
  }
}

/**
 * Builds a valid `TOOL_CALL` EVENT frame (W12 contract: run_id, task_id,
 * tool_call_id, tool, args_sha256, turn). Throws on any missing/ill-typed field so a
 * spec cannot accidentally send the production decoder an invalid payload.
 */
export function toolCallFrame(payload: ToolCallPayload, seq: number): string {
  assertNonEmptyString('TOOL_CALL.run_id', payload.run_id);
  assertNonEmptyString('TOOL_CALL.task_id', payload.task_id);
  assertNonEmptyString('TOOL_CALL.tool_call_id', payload.tool_call_id);
  assertNonEmptyString('TOOL_CALL.tool', payload.tool);
  assertSha256('TOOL_CALL.args_sha256', payload.args_sha256);
  assertTurn(payload.turn);
  return eventFrame('TOOL_CALL', payload, seq);
}

/**
 * Builds a valid `TOOL_RESULT` EVENT frame. `outcome` is restricted to the frozen
 * vocabulary; optional `code`/`bytes`/`sha256` are validated when present.
 */
export function toolResultFrame(payload: ToolResultPayload, seq: number): string {
  assertNonEmptyString('TOOL_RESULT.run_id', payload.run_id);
  assertNonEmptyString('TOOL_RESULT.task_id', payload.task_id);
  assertNonEmptyString('TOOL_RESULT.tool_call_id', payload.tool_call_id);
  assertOutcome(payload.outcome);
  if (payload.code !== undefined && payload.code !== null) {
    assertNonEmptyString('TOOL_RESULT.code', payload.code);
  }
  if (payload.bytes !== undefined && payload.bytes !== null) {
    if (!Number.isInteger(payload.bytes) || payload.bytes < 0) {
      throw new TypeError('replay-harness: TOOL_RESULT.bytes must be a non-negative integer');
    }
  }
  if (payload.sha256 !== undefined && payload.sha256 !== null) {
    assertSha256('TOOL_RESULT.sha256', payload.sha256);
  }
  return eventFrame('TOOL_RESULT', payload, seq);
}

/**
 * Builds a valid `APPROVAL_REQUEST` EVENT frame. `approval_id`/`tool` are required;
 * the six 1.2 fields are validated when present and never fabricated when absent.
 */
export function approvalRequestFrame(payload: ApprovalRequestPayload, seq: number): string {
  assertNonEmptyString('APPROVAL_REQUEST.approval_id', payload.approval_id);
  assertNonEmptyString('APPROVAL_REQUEST.tool', payload.tool);
  if (payload.decision !== undefined && payload.decision !== null) {
    assertNonEmptyString('APPROVAL_REQUEST.decision', payload.decision);
  }
  for (const field of ['nonce', 'run_id', 'task_id', 'tool_call_id'] as const) {
    const value = payload[field];
    if (value !== undefined && value !== null) {
      assertNonEmptyString(`APPROVAL_REQUEST.${field}`, value);
    }
  }
  if (payload.args_sha256 !== undefined && payload.args_sha256 !== null) {
    assertSha256('APPROVAL_REQUEST.args_sha256', payload.args_sha256);
  }
  if (payload.expires_at !== undefined && payload.expires_at !== null) {
    assertNonEmptyString('APPROVAL_REQUEST.expires_at', payload.expires_at);
    if (Number.isNaN(Date.parse(payload.expires_at))) {
      throw new TypeError('replay-harness: APPROVAL_REQUEST.expires_at must be an ISO-8601 date');
    }
  }
  return eventFrame('APPROVAL_REQUEST', payload, seq);
}

/** Sends a validated `TOOL_CALL` frame through the mocked socket. */
export function sendToolCall(ws: MockWsHarness, payload: ToolCallPayload, seq: number): void {
  ws.route().send(toolCallFrame(payload, seq));
}

/** Sends a validated `TOOL_RESULT` frame through the mocked socket. */
export function sendToolResult(ws: MockWsHarness, payload: ToolResultPayload, seq: number): void {
  ws.route().send(toolResultFrame(payload, seq));
}

/** Sends a validated `APPROVAL_REQUEST` frame through the mocked socket. */
export function sendApprovalRequest(
  ws: MockWsHarness,
  payload: ApprovalRequestPayload,
  seq: number,
): void {
  ws.route().send(approvalRequestFrame(payload, seq));
}

/** Sends arbitrary raw bytes (invalid JSON, non-EVENT, truncated frame) for refusal tests. */
export function sendMalformedFrame(ws: MockWsHarness, raw: string): void {
  ws.route().send(raw);
}

/**
 * Sends a syntactically valid EVENT envelope with an intentionally invalid payload,
 * for decoder-refusal tests (`E_SCHEMA_INVALID`) without hand-building the envelope.
 */
export function sendMalformedEvent(
  ws: MockWsHarness,
  name: string,
  payload: unknown,
  seq: number,
): void {
  ws.route().send(eventFrame(name, payload, seq));
}

export interface MockWsHarness {
  /** The routed page-side socket; test code pushes events through it. */
  route(): WebSocketRoute;
  /** Every COMMAND name the page sent through the mocked socket (proves pre-mutation/no-send). */
  commands: string[];
  /** Raw frames the page sent (control hello + COMMAND envelopes), for diagnostics only. */
  sent: string[];
  /** Parsed COMMAND frames in arrival order (name/payload/raw/correlation/at). */
  readonly sentCommands: readonly MockSentCommand[];
  /** COMMAND frames with the given name, in arrival order. */
  commandPayloads(name: string): MockSentCommand[];
  /** `APPROVAL_RESOLVE` COMMAND frames captured so far. */
  approvalResolveCommands(): MockSentCommand[];
  /**
   * Waits until a COMMAND with `name` (and optional predicate) is captured.
   * Throws with captured command names on timeout — never a silent pass.
   */
  waitForCommand(
    name: string,
    predicate?: (command: MockSentCommand) => boolean,
    timeoutMs?: number,
  ): Promise<MockSentCommand>;
  /** Sends raw server→client bytes through the mocked socket (malformed-frame tests). */
  sendRaw(raw: string): void;
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
  const sentCommands: MockSentCommand[] = [];

  const requireRoute = (): WebSocketRoute => {
    if (current === null) {
      throw new Error('replay harness: WebSocket route was not established by the app');
    }
    return current;
  };
  const findCommands = (name: string): MockSentCommand[] =>
    sentCommands.filter((command) => command.name === name);

  await page.routeWebSocket(/\/ws(\?.*)?$/, (route) => {
    current = route;
    route.onMessage((message) => {
      if (typeof message !== 'string') {
        return;
      }
      sent.push(message);
      try {
        const parsed = JSON.parse(message) as {
          type?: unknown;
          name?: unknown;
          payload?: unknown;
          correlation_id?: unknown;
        };
        if (parsed.type === 'COMMAND' && typeof parsed.name === 'string') {
          commands.push(parsed.name);
          sentCommands.push({
            name: parsed.name,
            payload: isRecord(parsed.payload) ? parsed.payload : null,
            raw: message,
            at: new Date().toISOString(),
            correlationId:
              typeof parsed.correlation_id === 'string' ? parsed.correlation_id : null,
          });
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
        // Raw control frame (`{type:"hello"}`) or malformed bytes: no ACK is fabricated.
      }
    });
  });

  const harness: MockWsHarness = {
    commands,
    sent,
    sentCommands,
    route: requireRoute,
    commandPayloads: findCommands,
    approvalResolveCommands: () => findCommands('APPROVAL_RESOLVE'),
    async waitForCommand(
      name: string,
      predicate?: (command: MockSentCommand) => boolean,
      timeoutMs = 5_000,
    ): Promise<MockSentCommand> {
      const deadline = Date.now() + timeoutMs;
      for (;;) {
        const match = findCommands(name).find((command) =>
          predicate === undefined ? true : predicate(command),
        );
        if (match) {
          return match;
        }
        if (Date.now() >= deadline) {
          throw new Error(
            `replay harness: no ${name} COMMAND captured within ${timeoutMs}ms; ` +
              `captured=[${sentCommands.map((command) => command.name).join(', ')}]`,
          );
        }
        await new Promise((resolve) => setTimeout(resolve, 25));
      }
    },
    sendRaw(raw: string): void {
      requireRoute().send(raw);
    },
  };
  return harness;
}
