/**
 * DEV-ONLY mock API (TEST DOUBLE, §7 boundaries). It implements the same
 * UiApi interface as the real WS client so the UI is exercised end-to-end
 * without a server. It is constructed ONLY by `createUiApi()` when
 * `import.meta.env.DEV && VITE_UI_MOCK === '1'`, and it throws in a production
 * build, so no production path can return fabricated data.
 *
 * Demo scenarios are explicit `!` prefixes in the prompt (never silent):
 *   !refuse:E_KEY_MISSING  -> typed ERROR frame (no ACK, no fake success)
 *   !approval              -> approval request with nonce after the run starts
 */
import {
  EVENT_NAMES,
  SCHEMA_VERSION,
  type CommandPayloads,
  type DecodedEvent,
  type Envelope,
  type EventName,
  type EventPayloads,
  type Refusal,
  type ServerCommandName,
} from './contract';
import { isAgentState } from '../state/agentState';
import { nowIso, uuidv7 } from './ids';
import { RefusalError, clientRefusal } from './refusals';
import type {
  ConnectionStatus,
  SessionInfo,
  UiApi,
} from './client';

const MOCK_SESSION: SessionInfo = {
  token: 'dev-mock-session-token',
  schema: SCHEMA_VERSION,
  last_seq: 0,
  ws: '/ws',
};

const MOCK_TREE_ENTRIES = [
  { path: 'src/router.ts', name: 'router.ts', kind: 'file' as const, size_bytes: 4210 },
  { path: 'src/schema.prisma', name: 'schema.prisma', kind: 'file' as const, size_bytes: 1180 },
  { path: 'src/agent.spec.ts', name: 'agent.spec.ts', kind: 'file' as const, size_bytes: 2040 },
  { path: 'src/main.tsx', name: 'main.tsx', kind: 'file' as const, size_bytes: 860 },
  { path: 'src/beam.rs', name: 'beam.rs', kind: 'file' as const, size_bytes: 3300 },
] as const;

const MOCK_FILE_TEXT = [
  '// DEV MOCK FILE — no real workspace is read in mock mode.',
  'export const route = (worker: string, parent: string) => ({ worker, parent });',
].join('\n');

export class MockApiClient implements UiApi {
  readonly kind = 'mock' as const;
  private seq = 0;
  private readonly listeners = new Set<
    (event: DecodedEvent, envelope: Envelope<unknown>) => void
  >();
  private readonly statusListeners = new Set<
    (status: ConnectionStatus, detail?: Refusal) => void
  >();
  private readonly clientErrorListeners = new Set<(refusal: Refusal) => void>();
  private runId: string | null = null;
  private runActive = false;
  private taskCounter = 0;
  private readonly tasks = new Map<string, { label: string; parentId: string | null }>();

  constructor() {
    if (import.meta.env.PROD) {
      throw new Error('MockApiClient is dev-only and must never be constructed in production');
    }
  }

  async open(): Promise<SessionInfo> {
    this.emitStatus('open');
    return MOCK_SESSION;
  }

  close(): void {
    this.emitStatus('closed');
  }

  private emitStatus(status: ConnectionStatus, detail?: Refusal): void {
    for (const listener of this.statusListeners) {
      listener(status, detail);
    }
  }

  private nextSeq(): number {
    this.seq += 1;
    return this.seq;
  }

  private emit<N extends EventName>(
    name: N,
    payload: EventPayloads[N],
    correlationId?: string,
    runId?: string | null,
    taskId?: string | null,
  ): void {
    const envelope: Envelope<unknown> = {
      schema: SCHEMA_VERSION,
      type: 'EVENT',
      name,
      id: uuidv7(),
      correlation_id: correlationId ?? uuidv7(),
      run_id: runId ?? null,
      task_id: taskId ?? null,
      seq: this.nextSeq(),
      ts: nowIso(),
      payload,
    };
    const event = { name, payload } as DecodedEvent;
    for (const listener of this.listeners) {
      listener(event, envelope);
    }
  }

  private spawnNode(
    label: string,
    kind: 'superagent' | 'worker' | 'tool' | 'error',
    parentId: string | null,
    state: 'queued' | 'running' | 'healthy' | 'failed' = 'queued',
    progress: number | null = null,
    errorCode: string | null = null,
  ): string {
    this.taskCounter += 1;
    const id = `mock-task-${String(this.taskCounter).padStart(3, '0')}`;
    this.tasks.set(id, { label, parentId });
    this.emit('NODE_SPAWN', {
      task_id: id,
      state,
      kind,
      label,
      parent_task_id: parentId,
      progress,
      error_code: errorCode,
    });
    return id;
  }

  private startScenario(text: string, correlationId: string): void {
    const runId = uuidv7();
    this.runId = runId;
    this.runActive = true;
    const superId = this.spawnNode('Backend Architecture', 'superagent', null, 'running');
    const workerA = this.spawnNode('tailwind-builder', 'worker', superId, 'running', 55);
    this.spawnNode('db-schema', 'worker', superId, 'running', 20);
    const toolId = this.spawnNode('Prisma Generator', 'tool', superId, 'healthy', 100);
    this.emit('RUN_CREATED', { run_id: runId, status: 'running' });
    this.emit('RUN_METRICS', {
      superagents: 1,
      workers: 3,
      green_pct: 66.7,
      delta: 0,
    });
    const chunks = ['Certainly! ', 'Here is ', 'the code', '...'];
    chunks.forEach((chunk, index) => {
      globalThis.setTimeout(() => {
        this.emit('REPLY_DELTA', { run_id: runId, text_delta: chunk });
        if (index === chunks.length - 1) {
          this.emit('REPLY_DONE', {
            run_id: runId,
            usage: {
              prompt_tokens: 42,
              completion_tokens: 17,
              total_tokens: 59,
              prompt_cache_hit_tokens: 0,
              prompt_cache_miss_tokens: 42,
              reasoning_tokens: null,
            },
          });
          this.emit('RUN_STATUS', { run_id: runId, status: 'completed' });
          this.runActive = false;
        }
      }, 40 + index * 40);
    });
    if (text.startsWith('!approval')) {
      globalThis.setTimeout(() => {
        this.emit('APPROVAL_REQUEST', {
          approval_id: uuidv7(),
          tool: 'file.patch',
          decision: null,
          nonce: uuidv7(),
        });
        this.emit('NODE_STATE', {
          task_id: workerA,
          state: 'awaiting_approval',
          kind: 'worker',
          label: 'tailwind-builder',
          parent_task_id: superId,
          progress: 55,
          error_code: null,
        });
        this.emit('NODE_STATE', {
          task_id: toolId,
          state: 'awaiting_approval',
          kind: 'tool',
          label: 'Prisma Generator',
          parent_task_id: superId,
          progress: 100,
          error_code: null,
        });
      }, 30);
    }
    this.emit('ACK', { command: 'SUBMIT_PROMPT', correlation_id: correlationId });
  }

  async send<N extends ServerCommandName>(
    name: N,
    payload: CommandPayloads[N],
    options?: { readonly correlationId?: string },
  ): Promise<void> {
    const correlationId = options?.correlationId ?? uuidv7();
    await Promise.resolve();
    if (name === 'SUBMIT_PROMPT') {
      const prompt = (payload as CommandPayloads['SUBMIT_PROMPT']).text;
      const refusalCode = prompt.startsWith('!refuse:') ? prompt.slice('!refuse:'.length).trim() : null;
      if (refusalCode) {
        const refusal = clientRefusal(refusalCode);
        this.emit('ERROR', {
          code: refusal.code,
          message: refusal.message,
          how_to_fix: refusal.how_to_fix,
          retryable: refusal.retryable,
        }, correlationId);
        throw new RefusalError(refusal);
      }
      if (this.runActive) {
        const refusal = clientRefusal('E_RUN_ACTIVE');
        this.emit('ERROR', {
          code: refusal.code,
          message: refusal.message,
          how_to_fix: refusal.how_to_fix,
          retryable: refusal.retryable,
        }, correlationId);
        throw new RefusalError(refusal);
      }
      this.startScenario(prompt, correlationId);
      return;
    }
    if (name === 'SETTINGS_STATUS') {
      this.emit('ACK', { command: 'SETTINGS_STATUS', correlation_id: correlationId });
      this.emit('SETTINGS_STATUS_RESULT', {
        provider: 'deepseek',
        model: 'deepseek-flash',
        key_present: false,
      });
      return;
    }
    if (name === 'FILE_TREE_LIST') {
      this.emit('ACK', { command: 'FILE_TREE_LIST', correlation_id: correlationId });
      this.emit('FILE_TREE_RESULT', { root: '.', entries: [...MOCK_TREE_ENTRIES] });
      return;
    }
    if (name === 'FILE_READ') {
      const path = (payload as CommandPayloads['FILE_READ']).path;
      this.emit('ACK', { command: 'FILE_READ', correlation_id: correlationId });
      this.emit('FILE_CONTENT', {
        path,
        sha256: '0'.repeat(64),
        bytes: MOCK_FILE_TEXT.length,
        text: MOCK_FILE_TEXT,
      });
      return;
    }
    if (name === 'HISTORY_REVISION_SET') {
      this.emit('ACK', { command: 'HISTORY_REVISION_SET', correlation_id: correlationId });
      const requested = (payload as CommandPayloads['HISTORY_REVISION_SET']).revision;
      this.emit('HISTORY_SNAPSHOT', {
        run_id: this.runId ?? uuidv7(),
        revision: requested,
        graph: { nodes: [...this.tasks.keys()], edges: [] },
      });
      return;
    }
    if (name === 'APPROVAL_RESOLVE') {
      const resolve = payload as CommandPayloads['APPROVAL_RESOLVE'];
      this.emit('ACK', { command: 'APPROVAL_RESOLVE', correlation_id: correlationId });
      this.emit('APPROVAL_RESOLVED', {
        approval_id: resolve.approval_id,
        tool: 'file.patch',
        decision: resolve.decision === 'approve' ? 'approved' : 'rejected',
      });
      return;
    }
    if (name === 'TEMPLATE_INSERT') {
      this.emit('ACK', { command: 'TEMPLATE_INSERT', correlation_id: correlationId });
      const parent = [...this.tasks.keys()][0] ?? null;
      this.spawnNode('Content Writer', 'worker', parent, 'queued');
      this.spawnNode('Data Analyst', 'worker', parent, 'queued');
      return;
    }
    if (name === 'SPAWN_SUB_HIVE') {
      this.emit('ACK', { command: 'SPAWN_SUB_HIVE', correlation_id: correlationId });
      const parentId = (payload as CommandPayloads['SPAWN_SUB_HIVE']).parent_task_id;
      this.spawnNode('Sub-hive agent', 'superagent', parentId, 'queued');
      return;
    }
    if (name === 'REWIRE_TASK') {
      this.emit('ACK', { command: 'REWIRE_TASK', correlation_id: correlationId });
      const rewire = payload as CommandPayloads['REWIRE_TASK'];
      this.emit('NODE_REWIRE', {
        task_id: rewire.task_id,
        state: 'running',
        parent_task_id: rewire.new_parent_id,
        kind: 'worker',
        label: 'tailwind-builder',
        progress: 55,
        error_code: null,
      });
      return;
    }
    if (name === 'CANCEL_RUN') {
      this.emit('ACK', { command: 'CANCEL_RUN', correlation_id: correlationId });
      if (this.runId) {
        this.emit('RUN_STATUS', { run_id: this.runId, status: 'cancelled' });
      }
      this.runActive = false;
      return;
    }
    if (name === 'KILL_WORKER') {
      this.emit('ACK', { command: 'KILL_WORKER', correlation_id: correlationId });
      const kill = payload as CommandPayloads['KILL_WORKER'];
      this.emit('NODE_STATE', { task_id: kill.task_id, state: 'killed', kind: 'worker', label: 'worker', parent_task_id: null, progress: null, error_code: null });
      return;
    }
    if (name === 'CHAT_PIN') {
      this.emit('ACK', { command: 'CHAT_PIN', correlation_id: correlationId });
      return;
    }
    this.emit('ERROR', {
      code: 'E_COMMAND_UNKNOWN',
      message: `DEV mock has no scenario for ${name}`,
      how_to_fix: 'Use the real server for this command.',
      retryable: false,
    }, correlationId);
    throw new RefusalError(clientRefusal('E_COMMAND_UNKNOWN'));
  }

  subscribe(listener: (event: DecodedEvent, envelope: Envelope<unknown>) => void): () => void {
    this.listeners.add(listener);
    return () => this.listeners.delete(listener);
  }

  subscribeStatus(listener: (status: ConnectionStatus, detail?: Refusal) => void): () => void {
    this.statusListeners.add(listener);
    return () => this.statusListeners.delete(listener);
  }

  subscribeClientError(listener: (refusal: Refusal) => void): () => void {
    this.clientErrorListeners.add(listener);
    return () => this.clientErrorListeners.delete(listener);
  }
}

/** Assert helper used by tests: mock events are always declared event names. */
export function isMockEventName(name: string): boolean {
  return (EVENT_NAMES as readonly string[]).includes(name);
}

export function isMockAgentState(value: unknown): boolean {
  return isAgentState(value);
}
