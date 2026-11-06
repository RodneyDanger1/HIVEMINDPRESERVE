import { beforeEach, describe, expect, it, vi } from 'vitest';
import { createUiApi } from '../src/api';
import { DEFAULT_HTTP_TIMEOUT_MS, fetchWithBoundTimeout } from '../src/api/http';
import { RefusalError } from '../src/api/refusals';

/**
 * Lifecycle-focused tests for the UI API boundary. The WsApiClient module is
 * replaced by a recorder so the lazy DevAwareApi ordering can be observed
 * without a network or a real WebSocket.
 */
const mocks = vi.hoisted(() => {
  const lifecycle: string[] = [];
  class FakeWsClient {
    readonly kind = 'ws' as const;
    constructor() {
      lifecycle.push('construct');
    }
    async open(): Promise<{ token: string; schema: string; last_seq: number; ws: string }> {
      lifecycle.push('open');
      return { token: 'stub-token', schema: 'hiverouter.v1', last_seq: 0, ws: '/ws' };
    }
    close(): void {
      lifecycle.push('close');
    }
    send(): Promise<void> {
      lifecycle.push('send');
      return Promise.resolve();
    }
    subscribe(_listener: unknown): () => void {
      lifecycle.push('subscribe');
      return () => {
        lifecycle.push('unsubscribe');
      };
    }
    subscribeStatus(_listener: unknown): () => void {
      lifecycle.push('subscribe-status');
      return () => {
        lifecycle.push('unsubscribe-status');
      };
    }
    subscribeClientError(_listener: unknown): () => void {
      lifecycle.push('subscribe-client-error');
      return () => {
        lifecycle.push('unsubscribe-client-error');
      };
    }
  }
  return { lifecycle, FakeWsClient };
});

vi.mock('../src/api/client', async () => {
  const actual = await vi.importActual<typeof import('../src/api/client')>('../src/api/client');
  return {
    ...actual,
    WsApiClient: mocks.FakeWsClient as unknown,
    PartyTransport: class FakePartyTransport {} as unknown,
  };
});

const RAW_SOURCES = import.meta.glob('../src/**/*.{ts,tsx}', {
  query: '?raw',
  import: 'default',
  eager: true,
}) as Record<string, string>;

const neverFetch = (() => new Promise<Response>(() => {})) as unknown as typeof fetch;

async function failureOf(promise: Promise<unknown>): Promise<RefusalError> {
  const outcome = await promise.then(
    () => null,
    (error: unknown) => error,
  );
  expect(outcome).toBeInstanceOf(RefusalError);
  return outcome as RefusalError;
}

beforeEach(() => {
  mocks.lifecycle.length = 0;
});

describe('DevAwareApi lifecycle (close / open ordering)', () => {
  it('close() before the first open() defers close until the delegate exists, before opening it', async () => {
    const api = createUiApi();
    expect(mocks.lifecycle).toEqual([]); // lazy: nothing constructed at factory time
    const unsubscribe = api.subscribe(() => undefined);
    expect(mocks.lifecycle).toEqual([]); // listener queued, no delegate yet

    api.close();
    await expect(api.open()).resolves.toMatchObject({ token: 'stub-token' });

    expect(mocks.lifecycle).toEqual(['construct', 'subscribe', 'close', 'open']);
    unsubscribe();
    expect(mocks.lifecycle).toContain('unsubscribe');
  });

  it('open() without close() never emits a spurious close and attaches the queued listener once', async () => {
    const api = createUiApi();
    expect(api.kind).toBe('ws');
    const unsubscribe = api.subscribe(() => undefined);
    await api.open();
    expect(mocks.lifecycle).toEqual(['construct', 'subscribe', 'open']);
    expect(mocks.lifecycle.filter((entry) => entry === 'subscribe')).toHaveLength(1);
    expect(mocks.lifecycle).not.toContain('close');
    unsubscribe();
  });

  it('reports kind "ws" before resolution and forwards commands after resolution', async () => {
    const api = createUiApi();
    expect(api.kind).toBe('ws');
    await api.open();
    await api.send('CHAT_PIN', { pinned: true });
    expect(mocks.lifecycle).toContain('send');
    expect(api.kind).toBe('ws');
  });
});

describe('bounded HTTP timeout / abort refusals', () => {
  it('rejects a hung request with a named timeout refusal (bounded wall time)', async () => {
    const startedAt = Date.now();
    const failure = await failureOf(
      fetchWithBoundTimeout('http://127.0.0.1:1/health', {
        label: 'GET /health',
        fetchImpl: neverFetch,
        timeoutMs: 20,
      }),
    );
    expect(failure.refusal.code).toBe('E_WS_DISCONNECTED');
    expect(failure.refusal.message).toContain('GET /health');
    expect(failure.refusal.message).toContain('timed out after 20 ms');
    expect(failure.refusal.how_to_fix.length).toBeGreaterThan(0);
    expect(failure.refusal.retryable).toBe(true);
    expect(Date.now() - startedAt).toBeLessThan(2000);
  });

  it('rejects an in-flight caller abort with a named refusal even if fetch ignores the signal', async () => {
    const controller = new AbortController();
    const pending = fetchWithBoundTimeout('http://127.0.0.1:1/api/session', {
      label: 'GET /api/session',
      fetchImpl: neverFetch,
      timeoutMs: 5000,
      signal: controller.signal,
    });
    controller.abort();
    const failure = await failureOf(pending);
    expect(failure.refusal.code).toBe('E_WS_DISCONNECTED');
    expect(failure.refusal.message).toContain('GET /api/session');
    expect(failure.refusal.message).toContain('aborted by the caller');
  });

  it('rejects a signal that was already aborted before the call', async () => {
    const controller = new AbortController();
    controller.abort();
    const failure = await failureOf(
      fetchWithBoundTimeout('http://127.0.0.1:1/health', {
        label: 'GET /health',
        fetchImpl: neverFetch,
        signal: controller.signal,
      }),
    );
    expect(failure.refusal.code).toBe('E_WS_DISCONNECTED');
    expect(failure.refusal.message).toContain('aborted by the caller');
  });

  it('passes an AbortSignal to fetch and propagates the raw Response on success', async () => {
    let seenSignal: AbortSignal | null = null;
    const fetchImpl = (async (_url: string | URL | Request, init?: RequestInit) => {
      seenSignal = init?.signal ?? null;
      return new Response('{"status":"ok"}', {
        status: 200,
        headers: { 'content-type': 'application/json' },
      });
    }) as unknown as typeof fetch;
    const response = await fetchWithBoundTimeout('http://127.0.0.1:1/health', {
      label: 'GET /health',
      fetchImpl,
      timeoutMs: 500,
    });
    expect(response.status).toBe(200);
    expect(seenSignal).toBeInstanceOf(AbortSignal);
  });

  it('names a network failure instead of surfacing a bare Error', async () => {
    const fetchImpl = (async () => {
      throw new Error('ECONNREFUSED');
    }) as unknown as typeof fetch;
    const failure = await failureOf(
      fetchWithBoundTimeout('http://127.0.0.1:1/health', {
        label: 'GET /health',
        fetchImpl,
        timeoutMs: 500,
      }),
    );
    expect(failure.refusal.code).toBe('E_WS_DISCONNECTED');
    expect(failure.refusal.message).toContain('GET /health');
    expect(failure.refusal.message).toContain('ECONNREFUSED');
  });

  it('exposes a bounded default (8s) without requiring an explicit timeout', () => {
    expect(DEFAULT_HTTP_TIMEOUT_MS).toBe(8000);
  });
});

describe('dev-mock module boundary', () => {
  it('has no static import or re-export of ./mock anywhere in src/api (`createUiApi` only)', () => {
    const apiSources = Object.entries(RAW_SOURCES).filter(([path]) =>
      path.startsWith('../src/api/'),
    );
    expect(apiSources.length).toBeGreaterThanOrEqual(5);
    for (const [path, source] of apiSources) {
      expect({ path, staticImport: /import\s+[^;]*\bfrom\s+['"]\.\/mock['"]/m.test(source) }).toEqual({
        path,
        staticImport: false,
      });
      expect({ path, staticReexport: /export\s+[^;]*\bfrom\s+['"]\.\/mock['"]/m.test(source) }).toEqual({
        path,
        staticReexport: false,
      });
    }
  });

  it('loads the dev mock exactly once through a dynamic import gated on the dev flags', () => {
    const entry = RAW_SOURCES['../src/api/index.ts'] ?? '';
    expect(entry.length).toBeGreaterThan(0);
    const dynamicMatches = entry.match(/import\(\s*['"]\.\/mock['"]\s*\)/g) ?? [];
    expect(dynamicMatches).toHaveLength(1);
    const flagIndex = entry.indexOf('USE_DEV_MOCK');
    const dynamicIndex = entry.indexOf("import('./mock')");
    expect(flagIndex).toBeGreaterThan(-1);
    expect(dynamicIndex).toBeGreaterThan(flagIndex);
    expect(entry).toContain('import.meta.env.DEV');
    expect(entry).toContain('VITE_UI_MOCK');
  });

  it('keeps the app entry chain free of direct mock imports', () => {
    for (const entryPath of ['../src/main.tsx', '../src/App.tsx']) {
      const source = RAW_SOURCES[entryPath] ?? '';
      expect(source.length).toBeGreaterThan(0);
      expect(source).not.toMatch(/from\s+['"][^'"]*mock['"]/);
    }
    const appSource = RAW_SOURCES['../src/App.tsx'] ?? '';
    expect(appSource).toContain('createUiApi');
  });

  it('refuses to construct the dev mock outside DEV && VITE_UI_MOCK=1', async () => {
    const { MockApiClient } = await import('../src/api/mock');
    expect(() => new MockApiClient()).toThrow(/dev-only/);
  });
});
