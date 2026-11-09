/**
 * Real WsApiClient StrictMode race pin (audit item 1).
 *
 * The store shares one API instance across a StrictMode mount/cleanup/mount.
 * If the first open() is still handshaking when cleanup closes it, the second
 * open() must start its own session and the stale open must not publish a
 * session/status or close the live transport afterwards.
 */
import { describe, expect, it } from 'vitest';
import { WsApiClient, type Transport } from '../src/api/client';
import { SCHEMA_VERSION } from '../src/api/contract';
import { until } from './helpers/fakeUiApi';

class FakeTransport implements Transport {
  closed = false;
  readonly sent: string[] = [];
  private readonly opens: Array<() => void> = [];
  private readonly closes: Array<(code?: number) => void> = [];

  send(data: string): void {
    this.sent.push(data);
  }
  close(): void {
    if (this.closed) {
      return;
    }
    this.closed = true;
    for (const listener of this.closes) listener();
  }
  open(): void {
    for (const listener of this.opens) listener();
  }
  onOpen(listener: () => void): void {
    this.opens.push(listener);
  }
  onClose(listener: (code?: number) => void): void {
    this.closes.push(listener);
  }
  onMessage(): void {
    return;
  }
  onError(): void {
    return;
  }
}

const sessionFetch = (async () =>
  new Response(
    JSON.stringify({ token: 't', schema: SCHEMA_VERSION, last_seq: 0, ws: '/ws' }),
    { status: 200, headers: { 'content-type': 'application/json' } },
  )) as unknown as typeof fetch;

describe('WsApiClient close/open generation', () => {
  it('starts a fresh session after close and a stale open cannot clobber it', async () => {
    const transports: FakeTransport[] = [];
    const client = new WsApiClient({
      baseUrl: 'http://127.0.0.1:1',
      fetchImpl: sessionFetch,
      transportFactory: () => {
        const transport = new FakeTransport();
        transports.push(transport);
        return transport;
      },
      handshakeTimeoutMs: 1_000,
    });

    const first = client.open();
    // The stale open is expected to reject on close; handle it from the start
    // so the intentional rejection is not reported as an unhandled error.
    const staleOutcome = first.then(
      () => null,
      (error: unknown) => error,
    );
    await until(() => transports.length === 1);
    client.close();
    expect(client.getStatus()).toBe('closed');

    const second = client.open();
    await until(() => transports.length === 2);
    transports[1]?.open();
    await second;
    expect(client.getStatus()).toBe('open');

    // Let the stale handshake settle; it must not touch the live generation.
    await staleOutcome;
    await until(() => transports.length === 2);
    expect(client.getStatus()).toBe('open');
    expect(transports[1]?.closed).toBe(false);
    expect(transports[1]?.sent).toHaveLength(1); // hello frame from the live open
  });
});
