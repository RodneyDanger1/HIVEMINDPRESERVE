import { describe, expect, it } from 'vitest';
import { createUiStore } from '../src/state/store';
import { SCHEMA_VERSION } from '../src/api/contract';
import type {
  DecodedEvent,
  Envelope,
  Refusal,
  ServerCommandName,
  Usage,
} from '../src/api/contract';
import type { ConnectionStatus, SessionInfo, UiApi } from '../src/api/client';
import { RefusalError } from '../src/api/refusals';

/**
 * Streaming render-back tests: ordered REPLY_DELTA* → REPLY_DONE must produce
 * exactly one assistant message whose text is the concatenation of the deltas,
 * finalized once — no duplicated text, no ghost message, no cross-run mixing.
 * Uses the same local FakeUiApi pattern as web/tests/store.test.ts (existing
 * helpers are not exported; existing tests are not modified).
 */
class FakeUiApi implements UiApi {
  readonly kind = 'ws' as const;
  readonly sends: Array<{
    readonly name: ServerCommandName;
    readonly correlationId: string | undefined;
    readonly resolve: () => void;
    readonly reject: (error: Error) => void;
  }> = [];
  private readonly events = new Set<(event: DecodedEvent, envelope: Envelope<unknown>) => void>();
  private readonly statuses = new Set<(status: ConnectionStatus, detail?: Refusal) => void>();
  private readonly clientErrors = new Set<(refusal: Refusal) => void>();

  async open(): Promise<SessionInfo> {
    return { token: 'fake', schema: SCHEMA_VERSION, last_seq: 0, ws: '/ws' };
  }
  close(): void {
    return;
  }
  send<N extends ServerCommandName>(
    name: N,
    _payload: unknown,
    options?: { readonly correlationId?: string },
  ): Promise<void> {
    return new Promise<void>((resolve, reject) => {
      this.sends.push({ name, correlationId: options?.correlationId, resolve, reject });
    });
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
    for (const listener of this.events) {
      listener(event, envelope);
    }
  }
  resolveNext(): void {
    this.sends.shift()?.resolve();
  }
  rejectNext(refusal: Refusal): void {
    this.sends.shift()?.reject(new RefusalError(refusal));
  }
}

const usage = (): Usage => ({
  prompt_tokens: 10,
  completion_tokens: 5,
  total_tokens: 15,
  prompt_cache_hit_tokens: 0,
  prompt_cache_miss_tokens: 10,
  reasoning_tokens: null,
});

async function connectedStore(): Promise<{
  api: FakeUiApi;
  store: ReturnType<typeof createUiStore>;
}> {
  const api = new FakeUiApi();
  const store = createUiStore(api);
  await store.getState().init();
  return { api, store };
}

function assistantMessages(store: ReturnType<typeof createUiStore>) {
  return store.getState().messages.filter((message) => message.role === 'assistant');
}

async function runningStore(runId: string): Promise<{
  api: FakeUiApi;
  store: ReturnType<typeof createUiStore>;
}> {
  const connected = await connectedStore();
  const submission = connected.store.getState().submitPrompt('hello hive');
  await Promise.resolve();
  connected.api.resolveNext();
  await submission;
  connected.api.emit({ name: 'RUN_CREATED', payload: { run_id: runId, status: 'running' } });
  return connected;
}

describe('streaming render-back (REPLY_DELTA → REPLY_DONE)', () => {
  it('assembles ordered deltas into one assistant message, then finalizes with no duplicate text', async () => {
    const { api, store } = await runningStore('run-1');

    for (const delta of ['Sure', ', here', ' is the code.']) {
      api.emit({ name: 'REPLY_DELTA', payload: { run_id: 'run-1', text_delta: delta } });
      expect(store.getState().streamingRunId).toBe('run-1');
    }

    const mid = assistantMessages(store);
    expect(mid).toHaveLength(1);
    expect(mid[0]?.text).toBe('Sure, here is the code.');
    expect(mid[0]?.runId).toBe('run-1');
    expect(mid[0]?.streaming).toBe(true);

    api.emit({ name: 'REPLY_DONE', payload: { run_id: 'run-1', usage: usage() } });

    const final = store.getState();
    const done = assistantMessages(store);
    expect(done).toHaveLength(1);
    expect(done[0]?.text).toBe('Sure, here is the code.');
    expect(done[0]?.streaming).toBe(false);
    expect(final.streamingRunId).toBeNull();
    expect(final.messages).toHaveLength(2); // user + assistant only
    const transcript = final.messages.map((message) => message.text).join('\n');
    expect(transcript.split('Sure, here is the code.').length - 1).toBe(1);
  });

  it('REPLY_DONE before any delta leaves no ghost assistant message', async () => {
    const { api, store } = await runningStore('run-2');
    const before = store.getState().messages;
    api.emit({ name: 'REPLY_DONE', payload: { run_id: 'run-2', usage: usage() } });
    expect(assistantMessages(store)).toHaveLength(0);
    expect(store.getState().streamingRunId).toBeNull();
    expect(store.getState().messages).toHaveLength(1); // the user message only
    // Nothing streamed: finalization returns the same array (no whole-transcript copy).
    expect(store.getState().messages).toBe(before);
  });

  it('an empty delta is a no-op: no copy and no empty assistant row', async () => {
    const { api, store } = await runningStore('run-empty');
    const before = store.getState().messages;
    api.emit({ name: 'REPLY_DELTA', payload: { run_id: 'run-empty', text_delta: '' } });
    expect(store.getState().messages).toBe(before);
    expect(assistantMessages(store)).toHaveLength(0);
    expect(store.getState().streamingRunId).toBeNull();
  });

  it('keeps a second run’s deltas out of the first run’s finalized text', async () => {
    const { api, store } = await runningStore('run-3');
    api.emit({ name: 'REPLY_DELTA', payload: { run_id: 'run-3', text_delta: 'first run text' } });
    api.emit({ name: 'REPLY_DONE', payload: { run_id: 'run-3', usage: usage() } });

    api.emit({ name: 'RUN_CREATED', payload: { run_id: 'run-4', status: 'running' } });
    api.emit({ name: 'REPLY_DELTA', payload: { run_id: 'run-4', text_delta: 'second run text' } });
    api.emit({ name: 'REPLY_DONE', payload: { run_id: 'run-4', usage: usage() } });

    const assistants = assistantMessages(store);
    expect(assistants).toHaveLength(2);
    expect(assistants[0]?.text).toBe('first run text');
    expect(assistants[0]?.streaming).toBe(false);
    expect(assistants[1]?.text).toBe('second run text');
    expect(assistants[1]?.runId).toBe('run-4');
    expect(store.getState().streamingRunId).toBeNull();
  });

  it('ignores a foreign run’s deltas without a run switch', async () => {
    const { api, store } = await runningStore('run-foreign-host');
    api.emit({
      name: 'REPLY_DELTA',
      payload: { run_id: 'run-other', text_delta: 'must not appear' },
    });
    expect(assistantMessages(store)).toHaveLength(0);
    expect(store.getState().streamingRunId).not.toBe('run-other');
  });

  it('duplicate REPLY_DONE is idempotent (text and message count unchanged)', async () => {
    const { api, store } = await runningStore('run-5');
    api.emit({ name: 'REPLY_DELTA', payload: { run_id: 'run-5', text_delta: 'done once' } });
    api.emit({ name: 'REPLY_DONE', payload: { run_id: 'run-5', usage: usage() } });
    const before = JSON.stringify(store.getState().messages);
    const beforeRef = store.getState().messages;

    api.emit({ name: 'REPLY_DONE', payload: { run_id: 'run-5', usage: usage() } });
    expect(JSON.stringify(store.getState().messages)).toBe(before);
    expect(store.getState().messages).toBe(beforeRef);
    expect(assistantMessages(store)).toHaveLength(1);
    expect(store.getState().streamingRunId).toBeNull();
  });

  it('an empty final delta does not duplicate or reset accumulated text', async () => {
    const { api, store } = await runningStore('run-6');
    api.emit({ name: 'REPLY_DELTA', payload: { run_id: 'run-6', text_delta: 'abc' } });
    api.emit({ name: 'REPLY_DELTA', payload: { run_id: 'run-6', text_delta: '' } });
    expect(assistantMessages(store)[0]?.text).toBe('abc');

    api.emit({ name: 'REPLY_DONE', payload: { run_id: 'run-6', usage: usage() } });
    expect(assistantMessages(store)).toHaveLength(1);
    expect(assistantMessages(store)[0]?.text).toBe('abc');
    expect(assistantMessages(store)[0]?.streaming).toBe(false);
  });
});
