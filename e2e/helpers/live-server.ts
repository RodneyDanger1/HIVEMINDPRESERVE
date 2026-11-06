import { spawn, type ChildProcess } from 'node:child_process';
import { existsSync, mkdirSync, mkdtempSync, rmSync } from 'node:fs';
import { createServer } from 'node:net';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import { fileURLToPath } from 'node:url';
import type { Page } from '@playwright/test';

/**
 * W4 live-server harness (Phase 1: process + teardown).
 *
 * Boots the real `hiverouter` binary on an ephemeral loopback port with a fresh temp
 * data dir, waits for `/health` + `/api/session`, exposes the base URL, and guarantees
 * teardown (child killed, temp dir removed). The provider key is never logged: child
 * output is captured through `redactSecrets()` and only surfaced in failures.
 *
 * Paid-provider calls are gated: only when `LIVE_ONE_PROMPT=1` AND `DEEPSEEK_API_KEY`
 * is present does a spec start the server with the key, and only the explicitly gated
 * tests submit a prompt. No-key tests start the child with the key removed.
 */
export const CONTRACT_SCHEMA = 'hiverouter.v1';

const REPO_ROOT = fileURLToPath(new URL('../../', import.meta.url));
const BINARY_CANDIDATES = [
  process.env.E2E_SERVER_BIN,
  join(REPO_ROOT, 'target', 'debug', 'hiverouter.exe'),
  join(REPO_ROOT, 'target', 'release', 'hiverouter.exe'),
].filter((candidate): candidate is string => typeof candidate === 'string' && candidate.length > 0);

const HELLO_TIMEOUT_MS = 8_000;

export interface BinaryInfo {
  readonly path: string;
  readonly variant: 'debug' | 'release' | 'env';
}

export interface LiveGate {
  readonly enabled: boolean;
  readonly reason: string;
}

export interface LiveHealth {
  readonly status: 'ok' | 'degraded' | 'failing';
  readonly schema: string;
  readonly version: string;
  readonly transport?: { readonly last_seq?: number; readonly graph_nodes?: number | null };
}

export interface LiveSession {
  readonly token: string;
  readonly schema: string;
  readonly last_seq: number;
  readonly ws: string;
}

export interface LiveServer {
  readonly baseURL: string;
  readonly port: number;
  readonly dataDir: string;
  readonly binary: BinaryInfo;
  readonly health: LiveHealth;
  readonly session: LiveSession;
  /** Redacted child output collected so far (never contains the provider key). */
  logs(): readonly string[];
  wsUrl(lastSeq?: number): string;
  /** GET probe against this live server (relative path, absolute base). */
  fetchJson<T>(path: string): Promise<T>;
  stop(): Promise<{ readonly exitCode: number | null; readonly logs: readonly string[] }>;
}

/** Replaces the provider key (and sk-like tokens) before anything is logged/thrown. */
export function redactSecrets(text: string): string {
  let redacted = text;
  const secrets = [process.env.DEEPSEEK_API_KEY].filter(
    (value): value is string => typeof value === 'string' && value.length > 0,
  );
  for (const secret of secrets) {
    redacted = redacted.split(secret).join('[redacted]');
  }
  return redacted.replace(/sk-[A-Za-z0-9_-]{8,}/g, '[redacted]');
}

/** Discovers the server binary without spawning it. */
export function findServerBinary(): BinaryInfo | null {
  for (const candidate of BINARY_CANDIDATES) {
    if (existsSync(candidate)) {
      const variant = candidate === process.env.E2E_SERVER_BIN
        ? 'env'
        : candidate.includes(`${join('target', 'debug')}`)
          ? 'debug'
          : candidate.includes(`${join('target', 'release')}`)
            ? 'release'
            : 'env';
      return { path: candidate, variant };
    }
  }
  return null;
}

/** Named skip reason when the server binary is unavailable (used by every live spec). */
export function serverSkipReason(): string | null {
  if (findServerBinary() !== null) {
    return null;
  }
  return (
    'LIVE_SERVER_MISSING: no hiverouter binary found; expected target/debug/hiverouter.exe, ' +
    'target/release/hiverouter.exe or E2E_SERVER_BIN. The paid/live specs must not fake a pass.'
  );
}

/**
 * Named skip reason for tests that would make a paid provider call.
 * A paid call is possible only with BOTH `LIVE_ONE_PROMPT=1` and a key present.
 */
export function paidGate(): LiveGate {
  const binary = findServerBinary();
  if (binary === null) {
    return { enabled: false, reason: serverSkipReason() ?? 'LIVE_SERVER_MISSING' };
  }
  if (process.env.LIVE_ONE_PROMPT !== '1') {
    return {
      enabled: false,
      reason:
        'LIVE_ONE_PROMPT is not 1: paid provider call disabled. Set LIVE_ONE_PROMPT=1 AND ' +
        'DEEPSEEK_API_KEY explicitly to run the live streaming/replay proof (one short prompt each).',
    };
  }
  if (typeof process.env.DEEPSEEK_API_KEY !== 'string' || process.env.DEEPSEEK_API_KEY.length === 0) {
    return {
      enabled: false,
      reason:
        'DEEPSEEK_API_KEY is not set: cannot run the paid provider proof; refusing to fake a pass. ' +
        'The no-key fail-closed tests still run and prove E_KEY_MISSING refusal.',
    };
  }
  return { enabled: true, reason: '' };
}

function sleep(ms: number): Promise<void> {
  return new Promise((resolve) => {
    setTimeout(resolve, ms);
  });
}

async function reserveLoopbackPort(): Promise<number> {
  return new Promise<number>((resolve, reject) => {
    const probe = createServer();
    probe.once('error', reject);
    probe.listen(0, '127.0.0.1', () => {
      const address = probe.address();
      if (address === null || typeof address === 'string') {
        probe.close();
        reject(new Error('LIVE_PORT_RESERVE_FAILED: could not read the reserved port'));
        return;
      }
      const { port } = address;
      probe.close((error) => {
        if (error) {
          reject(error);
          return;
        }
        resolve(port);
      });
    });
  });
}

interface SpawnedServer {
  readonly child: ChildProcess;
  readonly port: number;
  readonly logs: string[];
}

function collectLogs(child: ChildProcess, logs: string[]): void {
  const append = (chunk: Buffer | string): void => {
    const text = redactSecrets(String(chunk));
    for (const line of text.split(/\r?\n/)) {
      const trimmed = line.trim();
      if (trimmed.length > 0 && logs.length < 200) {
        logs.push(trimmed);
      }
    }
  };
  child.stdout?.on('data', append);
  child.stderr?.on('data', append);
}

function describeFailure(what: string, logs: readonly string[], binary: BinaryInfo): Error {
  const tail = logs.slice(-12).join('\n');
  return new Error(`${what}; binary=${binary.path} variant=${binary.variant}; recent redacted output:\n${tail}`);
}

async function waitForHealth(
  baseURL: string,
  child: ChildProcess,
  logs: string[],
  binary: BinaryInfo,
  timeoutMs: number,
): Promise<LiveHealth> {
  const deadline = Date.now() + timeoutMs;
  let last = '';
  while (Date.now() < deadline) {
    if (child.exitCode !== null) {
      throw describeFailure(`LIVE_SERVER_EXITED: child exited with code ${child.exitCode} during boot`, logs, binary);
    }
    try {
      const response = await fetch(`${baseURL}/health`, {
        signal: AbortSignal.timeout(1_500),
        headers: { accept: 'application/json' },
      });
      const body = (await response.json()) as {
        status?: string;
        schema?: string;
        version?: string;
        transport?: { last_seq?: number; graph_nodes?: number | null };
      };
      last = JSON.stringify(body);
      if (response.ok && (body.status === 'ok' || body.status === 'degraded')) {
        if (body.schema !== CONTRACT_SCHEMA) {
          throw describeFailure(`LIVE_SCHEMA_MISMATCH: /health schema=${String(body.schema)}`, logs, binary);
        }
        return body as LiveHealth;
      }
      if (body.status === 'failing') {
        throw describeFailure(`LIVE_HEALTH_FAILING: /health status=failing body=${last}`, logs, binary);
      }
    } catch (error) {
      if (error instanceof Error && error.message.startsWith('LIVE_')) {
        throw error;
      }
      last = error instanceof Error ? error.message : String(error);
    }
    await sleep(200);
  }
  throw describeFailure(`LIVE_HEALTH_TIMEOUT after ${timeoutMs}ms; last /health probe=${last}`, logs, binary);
}

async function fetchSession(baseURL: string, logs: string[], binary: BinaryInfo): Promise<LiveSession> {
  let response: Response;
  try {
    response = await fetch(`${baseURL}/api/session`, {
      signal: AbortSignal.timeout(3_000),
      headers: { accept: 'application/json' },
    });
  } catch (error) {
    throw describeFailure(
      `LIVE_SESSION_FAILED: ${error instanceof Error ? error.message : String(error)}`,
      logs,
      binary,
    );
  }
  if (!response.ok) {
    throw describeFailure(`LIVE_SESSION_FAILED: HTTP ${response.status}`, logs, binary);
  }
  const body = (await response.json()) as Partial<LiveSession>;
  if (
    typeof body.token !== 'string' ||
    typeof body.schema !== 'string' ||
    typeof body.last_seq !== 'number' ||
    typeof body.ws !== 'string'
  ) {
    throw describeFailure('LIVE_SESSION_SHAPE: /api/session did not match {token,schema,last_seq,ws}', logs, binary);
  }
  if (body.schema !== CONTRACT_SCHEMA) {
    throw describeFailure(`LIVE_SCHEMA_MISMATCH: session schema=${body.schema}`, logs, binary);
  }
  return { token: body.token, schema: body.schema, last_seq: body.last_seq, ws: body.ws };
}

async function waitForEmbeddedUi(liveBaseURL: string, logs: string[], binary: BinaryInfo): Promise<void> {
  let response: Response;
  try {
    response = await fetch(`${liveBaseURL}/`, {
      signal: AbortSignal.timeout(3_000),
      headers: { accept: 'text/html' },
    });
  } catch (error) {
    throw describeFailure(
      `LIVE_UI_FAILED: root request failed: ${error instanceof Error ? error.message : String(error)}`,
      logs,
      binary,
    );
  }
  const body = await response.text();
  if (!response.ok || !body.includes('<div id="root">')) {
    throw describeFailure(
      `LIVE_UI_MISSING: embedded UI not served at ${liveBaseURL}/ (HTTP ${response.status}, body ${body.length} B)`,
      logs,
      binary,
    );
  }
}

async function killAndWait(child: ChildProcess, timeoutMs: number): Promise<number | null> {
  if (child.exitCode !== null) {
    return child.exitCode;
  }
  const exited = new Promise<number | null>((resolve) => {
    child.once('exit', (code) => resolve(code));
  });
  child.kill();
  const timer = sleep(timeoutMs).then(() => 'timeout' as const);
  const result = await Promise.race([exited, timer]);
  if (result === 'timeout') {
    child.kill('SIGKILL');
    const forced = await Promise.race([exited, sleep(2_000).then(() => null)]);
    return forced;
  }
  return result;
}

export interface StartLiveServerOptions {
  /** When false, the child env is stripped of DEEPSEEK_API_KEY (no paid call possible). */
  readonly withProviderKey?: boolean;
  readonly bootTimeoutMs?: number;
}

/**
 * Starts the real server on an ephemeral loopback port + fresh temp data dir.
 * Bounded port retry; every failure is named and includes redacted logs only.
 */
export async function startLiveServer(options: StartLiveServerOptions = {}): Promise<LiveServer> {
  const binary = findServerBinary();
  if (binary === null) {
    throw new Error(serverSkipReason() ?? 'LIVE_SERVER_MISSING');
  }
  const bootTimeoutMs = options.bootTimeoutMs ?? 20_000;
  const tempRoot = mkdtempSync(join(tmpdir(), 'hiverouter-live-'));
  const dataDir = join(tempRoot, 'data');
  const workspaceRoot = join(tempRoot, 'workspace');
  mkdirSync(dataDir, { recursive: true });
  mkdirSync(workspaceRoot, { recursive: true });

  let spawned: SpawnedServer | null = null;
  let lastError: Error | null = null;
  for (let attempt = 1; attempt <= 3; attempt += 1) {
    const port = await reserveLoopbackPort();
    const child = spawn(
      binary.path,
      ['--bind', `127.0.0.1:${port}`, '--data-dir', dataDir, '--workspace-root', workspaceRoot],
      {
        env: buildChildEnv(options.withProviderKey !== false),
        stdio: ['ignore', 'pipe', 'pipe'],
        windowsHide: true,
      },
    );
    const logs: string[] = [];
    collectLogs(child, logs);
    const spawnError = new Promise<Error>((resolve) => {
      child.once('error', (error) => resolve(error));
    });
    spawned = { child, port, logs };
    const baseURL = `http://127.0.0.1:${port}`;
    try {
      const health = await Promise.race([
        waitForHealth(baseURL, child, logs, binary, bootTimeoutMs),
        spawnError.then((error) => {
          throw error;
        }),
      ]);
      const session = await fetchSession(baseURL, logs, binary);
      await waitForEmbeddedUi(baseURL, logs, binary);
      const live: LiveServer = {
        baseURL,
        port,
        dataDir,
        binary,
        health,
        session,
        logs: () => [...logs],
        wsUrl: (lastSeq = 0) =>
          `ws://127.0.0.1:${port}${session.ws}?token=${encodeURIComponent(session.token)}&last_seq=${lastSeq}`,
        fetchJson: async <T>(path: string): Promise<T> => {
          const response = await fetch(`${baseURL}${path}`, {
            signal: AbortSignal.timeout(5_000),
            headers: { accept: 'application/json' },
          });
          if (!response.ok) {
            throw new Error(`LIVE_HTTP_${response.status}: GET ${path}`);
          }
          return (await response.json()) as T;
        },
        stop: async () => {
          const exitCode = await killAndWait(child, 5_000);
          let cleanupError: string | null = null;
          for (let tryIndex = 0; tryIndex < 3; tryIndex += 1) {
            try {
              rmSync(tempRoot, { recursive: true, force: true });
              cleanupError = null;
              break;
            } catch (error) {
              cleanupError = redactSecrets(error instanceof Error ? error.message : String(error));
              await sleep(250);
            }
          }
          return {
            exitCode,
            logs: cleanupError === null ? [...logs] : [...logs, `LIVE_CLEANUP_WARNING: ${cleanupError}`],
          };
        },
      };
      return live;
    } catch (error) {
      lastError = error instanceof Error ? error : new Error(String(error));
      await killAndWait(child, 3_000);
      rmSync(tempRoot, { recursive: true, force: true });
      if (attempt === 3) {
        throw lastError;
      }
    }
  }
  throw lastError ?? new Error('LIVE_SPAWN_FAILED: could not start the server');
}

function buildChildEnv(withProviderKey: boolean): NodeJS.ProcessEnv {
  const env: NodeJS.ProcessEnv = { ...process.env };
  if (!withProviderKey) {
    delete env.DEEPSEEK_API_KEY;
  }
  // The harness owns bind/data/workspace; never inherit stale overrides from the shell.
  delete env.HIVEROUTER_BIND;
  delete env.HIVEROUTER_DATA_DIR;
  delete env.HIVEROUTER_WORKSPACE_ROOT;
  delete env.HIVEROUTER_ALLOWED_ORIGINS;
  return env;
}

/* ------------------------------------------------------------------ */
/* In-page WebSocket protocol probe                                    */
/* ------------------------------------------------------------------ */

export interface ProbeEnvelope {
  readonly schema?: string;
  readonly type?: string;
  readonly name?: string;
  readonly correlation_id?: string;
  readonly run_id?: string | null;
  readonly seq?: number | null;
  readonly payload?: Record<string, unknown>;
  readonly [key: string]: unknown;
}

export interface WsProbe {
  sendCommand(name: string, payload: Record<string, unknown>): Promise<string>;
  waitFor(
    predicate: (envelope: ProbeEnvelope) => boolean,
    timeoutMs?: number,
    label?: string,
  ): Promise<ProbeEnvelope>;
  frames(): Promise<ProbeEnvelope[]>;
  close(): Promise<void>;
}

/**
 * Opens a second WebSocket from the page context (so the browser sends the allowlisted
 * `Origin`) against the live server, performs the raw hello handshake, and lets a test
 * send frozen COMMAND envelopes and wait for EVENT envelopes.
 */
export async function openWsProbe(page: Page, url: string): Promise<WsProbe> {
  await page.evaluate((target: string) => {
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
      socket.send(JSON.stringify({ type: 'hello', last_seq: 0 }));
    });
    socket.addEventListener('close', () => {
      w.__w4Closed = true;
    });
  }, url);

  const readFrames = async (): Promise<ProbeEnvelope[]> =>
    page.evaluate(() => {
      const w = window as unknown as { __w4Frames?: unknown[] };
      return (w.__w4Frames ?? []) as ProbeEnvelope[];
    });

  const probe: WsProbe = {
    async sendCommand(name: string, payload: Record<string, unknown>): Promise<string> {
      return page.evaluate(
        ({ command, body }: { command: string; body: Record<string, unknown> }) => {
          const w = window as unknown as { __w4Ws?: WebSocket };
          if (!w.__w4Ws) {
            throw new Error('WS_PROBE_NOT_OPEN');
          }
          const id = crypto.randomUUID();
          const envelope = {
            schema: 'hiverouter.v1',
            type: 'COMMAND',
            name: command,
            id,
            correlation_id: id,
            run_id: null,
            task_id: null,
            seq: null,
            ts: new Date().toISOString(),
            payload: body,
          };
          w.__w4Ws.send(JSON.stringify(envelope));
          return id;
        },
        { command: name, body: payload },
      );
    },
    async waitFor(
      predicate: (envelope: ProbeEnvelope) => boolean,
      timeoutMs = HELLO_TIMEOUT_MS,
      label = 'matching envelope',
    ): Promise<ProbeEnvelope> {
      const deadline = Date.now() + timeoutMs;
      let seen: string[] = [];
      while (Date.now() < deadline) {
        const frames = await readFrames();
        const hit = frames.find((frame) => typeof frame === 'object' && frame !== null && predicate(frame));
        if (hit) {
          return hit;
        }
        seen = frames.map((frame) => String(frame?.name ?? 'unknown')).slice(-12);
        await sleep(50);
      }
      throw new Error(
        `LIVE_WS_TIMEOUT: no ${label} within ${timeoutMs}ms; recent event names=${JSON.stringify(seen)}`,
      );
    },
    async frames(): Promise<ProbeEnvelope[]> {
      return readFrames();
    },
    async close(): Promise<void> {
      await page.evaluate(() => {
        const w = window as unknown as { __w4Ws?: WebSocket };
        try {
          w.__w4Ws?.close();
        } catch {
          // already closed; teardown is not evidence
        }
      });
    },
  };

  await probe.waitFor((envelope) => envelope.name === 'SERVER_HELLO', HELLO_TIMEOUT_MS, 'SERVER_HELLO');
  return probe;
}
