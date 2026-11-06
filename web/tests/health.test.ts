import { describe, expect, it } from 'vitest';
import { fetchHealth } from '../src/api/health';
import { RefusalError } from '../src/api/refusals';

const jsonResponse = (body: unknown, status = 200): Response =>
  new Response(JSON.stringify(body), {
    status,
    headers: { 'content-type': 'application/json' },
  });

const neverFetch = (() => new Promise<Response>(() => {})) as unknown as typeof fetch;

async function failureOf(promise: Promise<unknown>): Promise<RefusalError> {
  const outcome = await promise.then(
    () => null,
    (error: unknown) => error,
  );
  expect(outcome).toBeInstanceOf(RefusalError);
  return outcome as RefusalError;
}

describe('health probe (bounded, named refusals)', () => {
  it('parses a healthy snapshot', async () => {
    const fetchImpl = (async () =>
      jsonResponse({
        status: 'ok',
        checks: [{ name: 'db', ok: true }, { name: 'ws', ok: false }],
        version: '1.0.0',
      })) as unknown as typeof fetch;
    const snapshot = await fetchHealth({ fetchImpl });
    expect(snapshot).toEqual({ status: 'ok', checks: ['db:ok', 'ws:failed'], version: '1.0.0' });
  });

  it('times out with a named client refusal instead of hanging', async () => {
    const startedAt = Date.now();
    const failure = await failureOf(fetchHealth({ fetchImpl: neverFetch, timeoutMs: 25 }));
    expect(failure.refusal.code).toBe('E_WS_DISCONNECTED');
    expect(failure.refusal.message).toContain('GET /health');
    expect(failure.refusal.message).toContain('timed out after 25 ms');
    expect(failure.refusal.how_to_fix.length).toBeGreaterThan(0);
    expect(Date.now() - startedAt).toBeLessThan(2000);
  });

  it('surfaces a network failure as a named refusal (never a bare error)', async () => {
    const fetchImpl = (async () => {
      throw new Error('ECONNREFUSED');
    }) as unknown as typeof fetch;
    const failure = await failureOf(fetchHealth({ fetchImpl, timeoutMs: 500 }));
    expect(failure.refusal.code).toBe('E_WS_DISCONNECTED');
    expect(failure.refusal.message).toContain('ECONNREFUSED');
    expect(failure.refusal.message).toContain('GET /health');
  });

  it('honours a caller abort signal with a named refusal', async () => {
    const controller = new AbortController();
    const pending = fetchHealth({ fetchImpl: neverFetch, timeoutMs: 5000, signal: controller.signal });
    controller.abort();
    const failure = await failureOf(pending);
    expect(failure.refusal.code).toBe('E_WS_DISCONNECTED');
    expect(failure.refusal.message).toContain('aborted by the caller');
  });

  it('rejects a non-JSON body as E_SCHEMA_INVALID', async () => {
    const fetchImpl = (async () =>
      new Response('not-json', {
        status: 200,
        headers: { 'content-type': 'application/json' },
      })) as unknown as typeof fetch;
    const failure = await failureOf(fetchHealth({ fetchImpl, timeoutMs: 500 }));
    expect(failure.refusal.code).toBe('E_SCHEMA_INVALID');
  });
});
