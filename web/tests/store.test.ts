import { describe, expect, it } from 'vitest';
import { createUiStore } from '../src/state/store';
import { clientRefusal, RefusalError } from '../src/api/refusals';
import { SCHEMA_VERSION, type DecodedEvent, type Envelope, type Refusal, type ServerCommandName } from '../src/api/contract';
import type { ConnectionStatus, SessionInfo, UiApi } from '../src/api/client';

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
    for (const listener of this.events) listener(event, envelope);
  }
  resolveNext(): void {
    this.sends.shift()?.resolve();
  }
  rejectNext(refusal: Refusal): void {
    this.sends.shift()?.reject(new RefusalError(refusal));
  }
  emitStatus(status: ConnectionStatus, detail?: Refusal): void {
    for (const listener of this.statuses) listener(status, detail);
  }
  emitClientError(refusal: Refusal): void {
    for (const listener of this.clientErrors) listener(refusal);
  }
}

async function connectedStore(): Promise<{ api: FakeUiApi; store: ReturnType<typeof createUiStore> }> {
  const api = new FakeUiApi();
  const store = createUiStore(api);
  await store.getState().init();
  return { api, store };
}

describe('UI store command lifecycle', () => {
  it('connects and reports open', async () => {
    const { store } = await connectedStore();
    expect(store.getState().connection).toBe('open');
  });

  it('does not mutate run state before ACK/RUN_CREATED', async () => {
    const { api, store } = await connectedStore();
    const submission = store.getState().submitPrompt('hello hive');
    await Promise.resolve();
    expect(api.sends).toHaveLength(1);
    expect(api.sends[0]?.name).toBe('SUBMIT_PROMPT');
    expect(store.getState().run).toBeNull();
    expect(store.getState().messages.some((message) => message.role === 'user')).toBe(true);

    api.resolveNext();
    await submission;
    expect(store.getState().pendingCommands).toEqual({});

    api.emit({ name: 'RUN_CREATED', payload: { run_id: 'run-1', status: 'running' } });
    expect(store.getState().run?.id).toBe('run-1');
  });

  it('refuses empty prompts before sending', async () => {
    const { api, store } = await connectedStore();
    await store.getState().submitPrompt('');
    expect(api.sends).toHaveLength(0);
    expect(store.getState().refusals[0]?.refusal.code).toBe('E_PROMPT_EMPTY');
  });

  it('surfaces a named ERROR as a toast and keeps run state null', async () => {
    const { api, store } = await connectedStore();
    const submission = store.getState().submitPrompt('hello?');
    await Promise.resolve();
    api.emit({
      name: 'ERROR',
      payload: {
        code: 'E_KEY_MISSING',
        message: 'missing key',
        how_to_fix: 'configure key',
        retryable: false,
      },
    });
    api.rejectNext(clientRefusal('E_KEY_MISSING'));
    await submission;
    expect(store.getState().refusals[0]?.refusal.code).toBe('E_KEY_MISSING');
    expect(store.getState().run).toBeNull();
  });

  it('updates settings from SETTINGS_STATUS_RESULT', async () => {
    const { api, store } = await connectedStore();
    api.emit({
      name: 'SETTINGS_STATUS_RESULT',
      payload: { provider: 'deepseek', model: 'deepseek-flash', key_present: false },
    });
    expect(store.getState().settings.keyPresent).toBe(false);
    expect(store.getState().settings.model).toBe('deepseek-flash');
  });

  it('blocks approval resolution without a nonce and sends with one', async () => {
    const { api, store } = await connectedStore();
    api.emit({ name: 'APPROVAL_REQUEST', payload: { approval_id: 'a1', tool: 'file.patch' } });
    await store.getState().resolveApproval('a1', 'approve');
    expect(api.sends).toHaveLength(0);
    expect(store.getState().refusals[0]?.refusal.code).toBe('E_SCHEMA_INVALID');

    api.emit({ name: 'APPROVAL_REQUEST', payload: { approval_id: 'a2', tool: 'file.patch', nonce: 'n-1' } });
    const resolution = store.getState().resolveApproval('a2', 'deny', 'operator');
    await Promise.resolve();
    expect(api.sends).toHaveLength(1);
    expect(api.sends[0]?.name).toBe('APPROVAL_RESOLVE');
    api.resolveNext();
    await resolution;
    expect(store.getState().refusals).toHaveLength(1);
  });

  it('sends exactly one SETTINGS_STATUS per Settings open (pending guard)', async () => {
    const { api, store } = await connectedStore();
    const first = store.getState().probeSettings();
    const second = store.getState().probeSettings();
    await Promise.resolve();
    expect(api.sends).toHaveLength(1);
    expect(api.sends[0]?.name).toBe('SETTINGS_STATUS');
    api.resolveNext();
    await Promise.all([first, second]);
    expect(store.getState().settings.status).toBe('pending');
    api.emit({
      name: 'SETTINGS_STATUS_RESULT',
      payload: { provider: 'deepseek', model: 'deepseek-flash', key_present: true },
    });
    expect(store.getState().settings.status).toBe('ok');
    expect(store.getState().settings.keyPresent).toBe(true);
  });

  it('keeps graph nodes but distinguishes metadata-less tasks honestly', async () => {
    const { api, store } = await connectedStore();
    api.emit({ name: 'NODE_SPAWN', payload: { task_id: 't-1', state: 'running' } });
    const task = store.getState().tasks['t-1'];
    expect(task.kind).toBe('worker');
    expect(task.label).toContain('Task');
    api.emit({
      name: 'NODE_SPAWN',
      payload: { task_id: 't-2', state: 'running', kind: 'superagent', label: 'Backend', parent_task_id: null },
    });
    expect(store.getState().tasks['t-2']?.kind).toBe('superagent');
  });
});
