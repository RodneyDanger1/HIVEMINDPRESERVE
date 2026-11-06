import { describe, expect, it } from 'vitest';
import { WsApiClient, type Transport } from '../src/api/client';
import { RefusalError } from '../src/api/refusals';
import { SCHEMA_VERSION, type Refusal } from '../src/api/contract';

class FakeTransport implements Transport {
  readonly sent: string[] = [];
  private opened = false;
  private readonly openCbs: Array<() => void> = [];
  private readonly closeCbs: Array<(code?: number) => void> = [];
  private readonly messageCbs: Array<(data: unknown) => void> = [];
  private readonly errorCbs: Array<(message?: string) => void> = [];

  send(data: string): void {
    this.sent.push(data);
  }
  close(): void {
    for (const cb of this.closeCbs) cb(1000);
  }
  onOpen(cb: () => void): void {
    this.openCbs.push(cb);
    if (this.opened) cb();
  }
  onClose(cb: (code?: number) => void): void {
    this.closeCbs.push(cb);
  }
  onMessage(cb: (data: unknown) => void): void {
    this.messageCbs.push(cb);
  }
  onError(cb: (message?: string) => void): void {
    this.errorCbs.push(cb);
  }
  triggerOpen(): void {
    this.opened = true;
    for (const cb of this.openCbs) cb();
  }
  emitMessage(payload: unknown): void {
    for (const cb of this.messageCbs) cb(JSON.stringify(payload));
  }
  emitRaw(payload: string): void {
    for (const cb of this.messageCbs) cb(payload);
  }
  triggerError(message: string): void {
    for (const cb of this.errorCbs) cb(message);
  }
}

function eventFrame(name: string, payload: unknown, correlationId: string): unknown {
  return {
    schema: SCHEMA_VERSION,
    type: 'EVENT',
    name,
    id: correlationId,
    correlation_id: correlationId,
    run_id: null,
    task_id: null,
    seq: 1,
    ts: new Date().toISOString(),
    payload,
  };
}

const fetchStub = (async () =>
  new Response(
    JSON.stringify({ token: 'test-token', schema: SCHEMA_VERSION, last_seq: 7, ws: '/ws' }),
    { status: 200, headers: { 'content-type': 'application/json' } },
  )) as unknown as typeof fetch;

async function openedClient(): Promise<{ client: WsApiClient; transport: FakeTransport }> {
  let transport: FakeTransport | null = null;
  const client = new WsApiClient({
    baseUrl: 'http://127.0.0.1:8080',
    fetchImpl: fetchStub,
    transportFactory: (url) => {
      expect(url).toContain('token=test-token');
      expect(url).toContain('last_seq=7');
      transport = new FakeTransport();
      return transport;
    },
    requestTimeoutMs: 500,
    handshakeTimeoutMs: 500,
  });
  const openPromise = client.open();
  await new Promise((resolve) => setTimeout(resolve, 0));
  if (!transport) {
    throw new Error('transport was not created');
  }
  (transport as FakeTransport).triggerOpen();
  await openPromise;
  return { client, transport };
}

describe('WS API client (ACK / refusal / hello)', () => {
  it('opens, sends the hello control frame, and resolves a command only on ACK', async () => {
    const { client, transport } = await openedClient();
    const hello = JSON.parse(transport.sent[0]) as { type: string; last_seq: number };
    expect(hello).toEqual({ type: 'hello', last_seq: 7 });

    const promise = client.send('SETTINGS_STATUS', {}, { correlationId: 'corr-1' });
    expect(transport.sent).toHaveLength(2);
    transport.emitMessage(
      eventFrame('ACK', { command: 'SETTINGS_STATUS', correlation_id: 'corr-1' }, 'corr-1'),
    );
    await expect(promise).resolves.toBeUndefined();
  });

  it('rejects on a typed ERROR frame (never a fake success)', async () => {
    const { client, transport } = await openedClient();
    const promise = client.send('SETTINGS_STATUS', {}, { correlationId: 'corr-2' });
    const refusal: Refusal = {
      code: 'E_KEY_MISSING',
      message: 'missing',
      how_to_fix: 'configure',
      retryable: false,
    };
    transport.emitMessage(eventFrame('ERROR', refusal, 'corr-2'));
    await expect(promise).rejects.toBeInstanceOf(RefusalError);
    await promise.catch((error: unknown) => {
      expect(error).toBeInstanceOf(RefusalError);
      expect((error as RefusalError).refusal.code).toBe('E_KEY_MISSING');
    });
  });

  it('reports malformed frames as E_SCHEMA_INVALID without dropping the socket', async () => {
    const { client, transport } = await openedClient();
    const errors: string[] = [];
    client.subscribeClientError((refusal) => errors.push(refusal.code));
    transport.emitRaw('not-json');
    expect(errors).toEqual(['E_SCHEMA_INVALID']);
  });

  it('rejects pre-send when the socket is closed', async () => {
    const { client } = await openedClient();
    client.close();
    await expect(client.send('SETTINGS_STATUS', {})).rejects.toBeInstanceOf(RefusalError);
  });

  it('rejects jail-escaping FILE_READ paths before sending', async () => {
    const { client, transport } = await openedClient();
    await expect(client.send('FILE_READ', { path: '../secret' })).rejects.toBeInstanceOf(
      RefusalError,
    );
    expect(transport.sent).toHaveLength(1);
  });

  it('bounds GET /api/session with a named timeout refusal (no hang, no transport)', async () => {
    const neverFetch = (() => new Promise<Response>(() => {})) as unknown as typeof fetch;
    let transportCreated = false;
    const client = new WsApiClient({
      baseUrl: 'http://127.0.0.1:8080',
      fetchImpl: neverFetch,
      httpTimeoutMs: 25,
      transportFactory: () => {
        transportCreated = true;
        return new FakeTransport();
      },
    });
    const startedAt = Date.now();
    const outcome = await client.open().then(
      () => null,
      (error: unknown) => error,
    );
    expect(outcome).toBeInstanceOf(RefusalError);
    const refusal = (outcome as RefusalError).refusal;
    expect(refusal.code).toBe('E_WS_DISCONNECTED');
    expect(refusal.message).toContain('GET /api/session');
    expect(refusal.message).toContain('timed out after 25 ms');
    expect(refusal.how_to_fix.length).toBeGreaterThan(0);
    expect(Date.now() - startedAt).toBeLessThan(2000);
    expect(transportCreated).toBe(false);
    expect(client.getStatus()).toBe('error');
  });

  it('names a network failure on GET /api/session instead of throwing a bare error', async () => {
    const failingFetch = (async () => {
      throw new Error('ECONNREFUSED');
    }) as unknown as typeof fetch;
    const client = new WsApiClient({
      baseUrl: 'http://127.0.0.1:8080',
      fetchImpl: failingFetch,
      httpTimeoutMs: 500,
    });
    const outcome = await client.open().then(
      () => null,
      (error: unknown) => error,
    );
    expect(outcome).toBeInstanceOf(RefusalError);
    expect((outcome as RefusalError).refusal.code).toBe('E_WS_DISCONNECTED');
    expect((outcome as RefusalError).refusal.message).toContain('GET /api/session');
    expect((outcome as RefusalError).refusal.message).toContain('ECONNREFUSED');
  });
});
