import { describe, expect, it } from 'vitest';
import { failureOf, failureText, httpFailure } from '../src/api/failure';
import { RefusalError } from '../src/api/refusals';

function jsonResponse(body: unknown, status = 500): Response {
  return new Response(JSON.stringify(body), {
    status,
    headers: { 'content-type': 'application/json' },
  });
}

describe('named HTTP failures (audit item 9)', () => {
  it('keeps the server code, message and how_to_fix', async () => {
    const failure = await httpFailure(
      jsonResponse(
        {
          code: 'E_RUN_NOT_FOUND',
          message: 'No committed run.',
          how_to_fix: 'Submit a prompt first.',
          retryable: false,
        },
        404,
      ),
      'GET /api/history/runs/r1/state',
    );
    expect(failure).toBeInstanceOf(RefusalError);
    expect(failure.refusal).toEqual({
      code: 'E_RUN_NOT_FOUND',
      message: 'No committed run.',
      how_to_fix: 'Submit a prompt first.',
      retryable: false,
    });
  });

  it('fills missing refused fields from the contract copy rather than inventing', async () => {
    const failure = await httpFailure(jsonResponse({ code: 'E_TOKEN_INVALID' }, 401), 'GET /api/session');
    expect(failure.refusal.code).toBe('E_TOKEN_INVALID');
    expect(failure.refusal.message.length).toBeGreaterThan(0);
    expect(failure.refusal.how_to_fix.length).toBeGreaterThan(0);
  });

  it('never returns a bare "HTTP 500" for a bodyless/HTML error route', async () => {
    for (const response of [
      new Response('<html>oops</html>', { status: 500 }),
      new Response('', { status: 500 }),
      jsonResponse({ detail: 'plain object without a code' }),
    ]) {
      const failure = await httpFailure(response, 'GET /api/a2a/pending');
      const text = failureText(failure, 'unreachable');
      expect(text).toContain('E_WS_DISCONNECTED');
      expect(text).toContain('GET /api/a2a/pending');
      expect(text).toContain('HTTP 500');
      expect(text).toContain(' — '); // how_to_fix is always appended
      expect(text.trim()).not.toBe('HTTP 500');
    }
  });

  it('formats RefusalError and plain errors for inline rendering', () => {
    const named = new RefusalError({
      code: 'E_WS_DISCONNECTED',
      message: 'route failed',
      how_to_fix: 'start the server',
      retryable: true,
    });
    expect(failureText(named, 'fallback')).toBe(
      'E_WS_DISCONNECTED: route failed — start the server',
    );
    expect(failureOf(named, 'fallback').code).toBe('E_WS_DISCONNECTED');
    expect(failureText(new Error('boom'), 'fallback')).toBe('boom');
    expect(failureText('not an error', 'fallback')).toBe('fallback');
  });
});
