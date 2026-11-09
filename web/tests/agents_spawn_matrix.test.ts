/**
 * Agent command wiring: the strict spawn class matrix, its visible disabled
 * reasons, and the exact SPAWN_SUB_HIVE payload the store sends.
 */
import { describe, expect, it } from 'vitest';
import { createUiStore } from '../src/state/store';
import { spawnGate, spawnPlanFor, spawnMatrixReason } from '../src/state/spawnMatrix';
import { CONTROL_INTENTS } from '../src/state/chatIntents';
import { SCHEMA_VERSION } from '../src/api/contract';
import type { DecodedEvent, Envelope, Refusal, ServerCommandName } from '../src/api/contract';
import type { ConnectionStatus, SessionInfo, UiApi } from '../src/api/client';

class FakeUiApi implements UiApi {
  readonly kind = 'ws' as const;
  readonly sends: Array<{
    readonly name: ServerCommandName;
    readonly payload: unknown;
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
    payload: unknown,
    options?: { readonly correlationId?: string },
  ): Promise<void> {
    return new Promise<void>((resolve, reject) => {
      this.sends.push({ name, payload, correlationId: options?.correlationId, resolve, reject });
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
}

async function connectedStore(): Promise<{ api: FakeUiApi; store: ReturnType<typeof createUiStore> }> {
  const api = new FakeUiApi();
  const store = createUiStore(api);
  await store.getState().init();
  return { api, store };
}

function seedRun(api: FakeUiApi): void {
  api.emit({ name: 'RUN_CREATED', payload: { run_id: 'run-1', status: 'running' } });
  api.emit({
    name: 'NODE_SPAWN',
    payload: {
      task_id: 'hyper-1',
      state: 'running',
      kind: 'hyperagent',
      label: 'HyperAgent',
      parent_task_id: null,
      rev: 1,
    },
  });
  api.emit({
    name: 'NODE_SPAWN',
    payload: {
      task_id: 'super-1',
      state: 'running',
      kind: 'superagent',
      label: 'SuperAgent',
      parent_task_id: 'hyper-1',
      rev: 1,
    },
  });
  api.emit({
    name: 'NODE_SPAWN',
    payload: {
      task_id: 'agent-1',
      state: 'running',
      kind: 'worker',
      label: 'Agent',
      parent_task_id: 'super-1',
      rev: 1,
    },
  });
  api.emit({
    name: 'NODE_SPAWN',
    payload: {
      task_id: 'sub-1',
      state: 'running',
      kind: 'subagent',
      label: 'Subagent',
      parent_task_id: 'agent-1',
      rev: 1,
    },
  });
}

describe('spawn class matrix', () => {
  it('maps HyperAgent -> superagent, SuperAgent -> worker (Agent), Agent -> subagent', () => {
    expect(spawnPlanFor({ id: 'h', kind: 'hyperagent' })).toMatchObject({ role: 'superagent', childKind: 'superagent' });
    expect(spawnPlanFor({ id: 's', kind: 'superagent' })).toMatchObject({ role: 'worker', childKind: 'worker' });
    expect(spawnPlanFor({ id: 'a', kind: 'worker' })).toMatchObject({ role: 'subagent', childKind: 'subagent' });
  });

  it('refuses Subagent, tool and error parents', () => {
    expect(spawnPlanFor({ id: 's', kind: 'subagent' })).toBeNull();
    expect(spawnPlanFor({ id: 't', kind: 'tool' })).toBeNull();
    expect(spawnPlanFor({ id: 'e', kind: 'error' })).toBeNull();
    expect(spawnMatrixReason({ id: 's', kind: 'subagent' })).toContain('class matrix ends');
  });

  it('never enables SPAWN without an open connection, active run, and no in-flight command', () => {
    const parent = { id: 's', kind: 'superagent' as const };
    expect(spawnGate({ parent, runStatus: 'running', connection: 'closed', replaying: false, pending: false }).plan).toBeNull();
    expect(spawnGate({ parent, runStatus: null, connection: 'open', replaying: false, pending: false }).reason).toContain('No run');
    expect(spawnGate({ parent, runStatus: 'completed', connection: 'open', replaying: false, pending: false }).reason).toContain('completed');
    expect(spawnGate({ parent, runStatus: 'running', connection: 'open', replaying: false, pending: true }).reason).toContain('ACK');
    expect(spawnGate({ parent, runStatus: 'running', connection: 'open', replaying: true, pending: false }).reason).toContain('Replay');
    expect(spawnGate({ parent, runStatus: 'running', connection: 'open', replaying: false, pending: false }).plan).toMatchObject({ parentId: 's', role: 'worker' });
  });
});

describe('SPAWN_SUB_HIVE wiring', () => {
  it('sends the exact parent and the matrix role for a SuperAgent card', async () => {
    const { api, store } = await connectedStore();
    seedRun(api);
    const sent = store.getState().spawnSubHive('super-1');
    await Promise.resolve();
    expect(api.sends).toHaveLength(1);
    expect(api.sends[0]?.name).toBe('SPAWN_SUB_HIVE');
    expect(api.sends[0]?.payload).toEqual({ parent_task_id: 'super-1', role: 'worker' });
    api.resolveNext();
    await sent;
  });

  it('sends superagent for the HyperAgent and subagent for an Agent', async () => {
    const { api, store } = await connectedStore();
    seedRun(api);
    const hyper = store.getState().spawnSubHive('hyper-1');
    await Promise.resolve();
    expect(api.sends[0]?.payload).toEqual({ parent_task_id: 'hyper-1', role: 'superagent' });
    api.resolveNext();
    await hyper;

    const agent = store.getState().spawnSubHive('agent-1');
    await Promise.resolve();
    // `resolveNext()` shifted the first send; the Agent spawn is now head.
    expect(api.sends[0]?.payload).toEqual({ parent_task_id: 'agent-1', role: 'subagent' });
    api.resolveNext();
    await agent;
  });

  it('sends nothing and surfaces E_TOOL_NOT_ALLOWED for a Subagent parent', async () => {
    const { api, store } = await connectedStore();
    seedRun(api);
    await store.getState().spawnSubHive('sub-1');
    expect(api.sends).toHaveLength(0);
    expect(store.getState().refusals[0]?.refusal.code).toBe('E_TOOL_NOT_ALLOWED');
    expect(store.getState().refusals[0]?.refusal.message).toContain('class matrix ends');
  });

  it('refuses a terminal run with E_RUN_TERMINAL and sends nothing', async () => {
    const { api, store } = await connectedStore();
    seedRun(api);
    api.emit({
      name: 'RUN_STATUS',
      payload: { run_id: 'run-1', status: 'completed' },
    });
    await store.getState().spawnSubHive('super-1');
    expect(api.sends).toHaveLength(0);
    expect(store.getState().refusals[0]?.refusal.code).toBe('E_RUN_TERMINAL');
  });
});

describe('root-control chat intents', () => {
  it('maps spawn remote/daemon to prompt text, never a fabricated command', () => {
    const ids = CONTROL_INTENTS.map((intent) => intent.id);
    expect(ids).toEqual(['spawn-remote-agent', 'spawn-daemon']);
    for (const intent of CONTROL_INTENTS) {
      expect(intent.prompt).toContain('root-control');
      expect(intent.prompt.length).toBeGreaterThan(20);
    }
  });
});
