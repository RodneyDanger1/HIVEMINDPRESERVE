/**
 * Controllable UiApi test double. It records open/close calls and lets a test
 * resolve each open() independently, which is what a React StrictMode
 * dispose/re-init sequence needs to be deterministic.
 */
import { SCHEMA_VERSION, type DecodedEvent, type Envelope, type Refusal, type ServerCommandName } from '../../src/api/contract';
import type { ConnectionStatus, SessionInfo, UiApi } from '../../src/api/client';

interface OpenCall {
  readonly resolve: (session: SessionInfo) => void;
  readonly reject: (error: Error) => void;
}

export class FakeUiApi implements UiApi {
  readonly kind = 'ws' as const;
  readonly openCalls: OpenCall[] = [];
  readonly sends: Array<{ readonly name: ServerCommandName; readonly payload: unknown }> = [];
  opens = 0;
  closes = 0;
  private readonly events = new Set<(event: DecodedEvent, envelope: Envelope<unknown>) => void>();
  private readonly statuses = new Set<(status: ConnectionStatus, detail?: Refusal) => void>();
  private readonly clientErrors = new Set<(refusal: Refusal) => void>();

  open(): Promise<SessionInfo> {
    this.opens += 1;
    return new Promise<SessionInfo>((resolve, reject) => {
      this.openCalls.push({ resolve, reject });
    });
  }

  resolveOpen(index: number): void {
    this.openCalls[index]?.resolve({
      token: 'fake',
      schema: SCHEMA_VERSION,
      last_seq: 0,
      ws: '/ws',
    });
  }

  close(): void {
    this.closes += 1;
  }

  send<N extends ServerCommandName>(name: N, payload: unknown): Promise<void> {
    this.sends.push({ name, payload });
    return Promise.resolve();
  }

  subscribe(listener: (event: DecodedEvent, envelope: Envelope<unknown>) => void): () => void {
    this.events.add(listener);
    return () => this.events.delete(listener);
  }

  subscribeStatus(listener: (status: ConnectionStatus, detail?: Refusal) => void): () => void {
    this.statuses.add(listener);
    return () => this.statuses.delete(listener);
  }

  subscribeClientError(listener: (refusal: Refusal) => void): () => void {
    this.clientErrors.add(listener);
    return () => this.clientErrors.delete(listener);
  }

  emit(event: DecodedEvent): void {
    const envelope: Envelope<unknown> = {
      schema: SCHEMA_VERSION,
      type: 'EVENT',
      name: event.name,
      id: 'e',
      correlation_id: 'e',
      run_id: null,
      task_id: null,
      seq: 1,
      ts: new Date().toISOString(),
      payload: {},
    };
    for (const listener of this.events) listener(event, envelope);
  }
}

export function emptyRunsResponse(): Response {
  return new Response(JSON.stringify({ runs: [] }), {
    status: 200,
    headers: { 'content-type': 'application/json' },
  });
}

/** Bounded real-timer wait for a predicate, so tests fail on a hang, not on timing. */
export async function until(predicate: () => boolean): Promise<void> {
  for (let attempt = 0; attempt < 500; attempt += 1) {
    if (predicate()) {
      return;
    }
    await new Promise((resolve) => setTimeout(resolve, 0));
  }
  throw new Error('condition not met within 500 event-loop turns');
}
