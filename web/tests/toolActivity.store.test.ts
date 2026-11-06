import { describe, expect, it } from 'vitest';
import { createUiStore, MAX_BUFFERED_TOOL_RESULTS } from '../src/state/store';
import { clientRefusal, RefusalError } from '../src/api/refusals';
import {
  SCHEMA_VERSION,
  type DecodedEvent,
  type Envelope,
  type Refusal,
  type ServerCommandName,
  type ToolOutcome,
} from '../src/api/contract';
import type { ConnectionStatus, SessionInfo, UiApi } from '../src/api/client';

/**
 * W12 / Wave 1 seat 5 — tool + approval state in the one UI store.
 * Uses the existing FakeUiApi harness shape from tests/store.test.ts; no network,
 * no real socket. Asserts dedupe/order/outcome transitions, 1.2 approval fields,
 * the in-flight resolve guard and the absence of optimistic approve/deny.
 * Wave 3 adds the deterministic out-of-order policy: TOOL_RESULT is buffered until its
 * TOOL_CALL (bounded, deduped, never rendered alone), replayed once; terminal runs drop
 * their buffered orphans.
 */
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
  emitStatus(status: ConnectionStatus, detail?: Refusal): void {
    for (const listener of this.statuses) listener(status, detail);
  }
  resolveNext(): void {
    this.sends.shift()?.resolve();
  }
  rejectNext(refusal: Refusal): void {
    this.sends.shift()?.reject(new RefusalError(refusal));
  }
}

async function connectedStore(): Promise<{ api: FakeUiApi; store: ReturnType<typeof createUiStore> }> {
  const api = new FakeUiApi();
  const store = createUiStore(api);
  await store.getState().init();
  return { api, store };
}

function toolCall(
  toolCallId: string,
  turn: number,
  tool = 'file_read',
  argsSha256 = 'a'.repeat(64),
): DecodedEvent {
  return {
    name: 'TOOL_CALL',
    payload: {
      run_id: 'run-1',
      task_id: 'task-1',
      tool_call_id: toolCallId,
      tool,
      args_sha256: argsSha256,
      turn,
    },
  };
}

function toolResult(
  toolCallId: string,
  outcome: ToolOutcome,
  code?: string,
): DecodedEvent {
  return {
    name: 'TOOL_RESULT',
    payload: {
      run_id: 'run-1',
      task_id: 'task-1',
      tool_call_id: toolCallId,
      outcome,
      ...(code === undefined ? {} : { code }),
    },
  };
}

function approvalRequest(extra: Record<string, unknown> = {}): DecodedEvent {
  return {
    name: 'APPROVAL_REQUEST',
    payload: { approval_id: 'a1', tool: 'file_write', ...extra } as never,
  };
}

describe('W12 tool activity reducer', () => {
  it('creates exactly one pending row from TOOL_CALL (identity, args hash, turn, no text)', async () => {
    const { api, store } = await connectedStore();
    api.emit(toolCall('tc-1', 1));

    const rows = store.getState().toolActivity;
    expect(rows).toHaveLength(1);
    expect(rows[0]?.toolCallId).toBe('tc-1');
    expect(rows[0]?.tool).toBe('file_read');
    expect(rows[0]?.argsSha256).toBe('a'.repeat(64));
    expect(rows[0]?.turn).toBe(1);
    expect(rows[0]?.outcome).toBe('pending');
    expect(rows[0]?.text).toBeUndefined();
  });

  it('dedupes a repeated TOOL_CALL and preserves arrival order', async () => {
    const { api, store } = await connectedStore();
    api.emit(toolCall('tc-1', 1, 'file_read', 'a'.repeat(64)));
    api.emit(toolCall('tc-1', 1, 'shell_exec', 'b'.repeat(64))); // replay/conflict: first identity wins
    api.emit(toolCall('tc-2', 2));

    const rows = store.getState().toolActivity;
    expect(rows.map((row) => row.toolCallId)).toEqual(['tc-1', 'tc-2']);
    expect(rows[0]?.tool).toBe('file_read');
    expect(rows[0]?.argsSha256).toBe('a'.repeat(64));
    expect(rows[0]?.outcome).toBe('pending');
  });

  it('transitions pending -> terminal once and captures the refusal code', async () => {
    const { api, store } = await connectedStore();
    api.emit(toolCall('tc-1', 1));
    api.emit(toolResult('tc-1', 'refused', 'E_PATH_OUTSIDE_JAIL'));

    const row = store.getState().toolActivity[0];
    expect(row?.outcome).toBe('refused');
    expect(row?.code).toBe('E_PATH_OUTSIDE_JAIL');
  });

  it('buffers a result for an unknown call without rendering it; invalid outcome is ignored', async () => {
    const { api, store } = await connectedStore();
    api.emit(toolResult('tc-unknown', 'ok'));
    // Out-of-order results are buffered invisibly: a result alone never fabricates a row.
    expect(store.getState().toolActivity).toEqual([]);

    api.emit(toolCall('tc-1', 1));
    api.emit({
      name: 'TOOL_RESULT',
      payload: {
        run_id: 'run-1',
        task_id: 'task-1',
        tool_call_id: 'tc-1',
        outcome: 'not-a-real-outcome' as unknown as ToolOutcome,
      },
    });
    expect(store.getState().toolActivity[0]?.outcome).toBe('pending');
  });

  it('keeps arrival order when results arrive in reverse order', async () => {
    const { api, store } = await connectedStore();
    api.emit(toolCall('tc-1', 1));
    api.emit(toolCall('tc-2', 2));
    api.emit(toolResult('tc-2', 'denied'));
    api.emit(toolResult('tc-1', 'ok'));

    const rows = store.getState().toolActivity;
    expect(rows.map((row) => row.toolCallId)).toEqual(['tc-1', 'tc-2']);
    expect(rows.map((row) => row.outcome)).toEqual(['ok', 'denied']);
  });

  it('keeps the first terminal outcome (a second result never re-transitions)', async () => {
    const { api, store } = await connectedStore();
    api.emit(toolCall('tc-1', 1));
    api.emit(toolResult('tc-1', 'timeout'));
    api.emit(toolResult('tc-1', 'ok', 'E_SHOULD_NOT_APPLY'));

    const row = store.getState().toolActivity[0];
    expect(row?.outcome).toBe('timeout');
    expect(row?.code).toBeUndefined();
  });
});

describe('W12 approval state', () => {
  it('stores the six 1.2 approval fields and defaults them to null when absent', async () => {
    const { api, store } = await connectedStore();
    api.emit(approvalRequest());
    const bare = store.getState().approvals[0];
    expect(store.getState().approvals).toHaveLength(1);
    expect(bare?.approvalId).toBe('a1');
    expect(bare?.tool).toBe('file_write');
    expect(bare?.nonce).toBeNull();
    expect(bare?.decision).toBeNull();
    expect(bare?.runId).toBeNull();
    expect(bare?.taskId).toBeNull();
    expect(bare?.toolCallId).toBeNull();
    expect(bare?.argsSha256).toBeNull();
    expect(bare?.expiresAt).toBeNull();

    api.emit(
      approvalRequest({
        nonce: 'n-1',
        run_id: 'run-1',
        task_id: 'task-1',
        tool_call_id: 'tc-9',
        args_sha256: 'c'.repeat(64),
        expires_at: '2026-11-06T18:00:00Z',
      }),
    );
    const approval = store.getState().approvals[0];
    expect(store.getState().approvals).toHaveLength(1);
    expect(approval?.nonce).toBe('n-1');
    expect(approval?.runId).toBe('run-1');
    expect(approval?.taskId).toBe('task-1');
    expect(approval?.toolCallId).toBe('tc-9');
    expect(approval?.argsSha256).toBe('c'.repeat(64));
    expect(approval?.expiresAt).toBe('2026-11-06T18:00:00Z');
  });

  it('refuses to resolve without a nonce and sends nothing', async () => {
    const { api, store } = await connectedStore();
    api.emit(approvalRequest());
    await store.getState().resolveApproval('a1', 'approve');

    expect(api.sends).toHaveLength(0);
    expect(store.getState().refusals[0]?.refusal.code).toBe('E_SCHEMA_INVALID');
    expect(store.getState().approvals).toHaveLength(1);
  });

  it('guards duplicate resolves while in flight and never optimistically decides', async () => {
    const { api, store } = await connectedStore();
    api.emit(approvalRequest({ nonce: 'n-1' }));

    const first = store.getState().resolveApproval('a1', 'approve');
    await Promise.resolve();
    expect(api.sends).toHaveLength(1);
    expect(api.sends[0]?.name).toBe('APPROVAL_RESOLVE');
    const sentPayload = api.sends[0]?.payload as
      | { readonly approval_id?: string; readonly nonce?: string; readonly decision?: string }
      | undefined;
    expect(sentPayload?.approval_id).toBe('a1');
    expect(sentPayload?.nonce).toBe('n-1');
    expect(sentPayload?.decision).toBe('approve');
    expect(store.getState().resolvingApprovals).toEqual(['a1']);

    // Second gesture while the first command is pending must be a no-op.
    const second = store.getState().resolveApproval('a1', 'deny');
    await Promise.resolve();
    expect(api.sends).toHaveLength(1);

    // No optimistic approve/deny: the record is unchanged before the server event.
    const pendingApproval = store.getState().approvals[0];
    expect(store.getState().approvals).toHaveLength(1);
    expect(pendingApproval?.decision).toBeNull();

    api.resolveNext();
    await Promise.all([first, second]);
    expect(store.getState().resolvingApprovals).toEqual([]);
    expect(store.getState().approvals).toHaveLength(1);
    expect(store.getState().approvals[0]?.decision).toBeNull();

    api.emit({
      name: 'APPROVAL_RESOLVED',
      payload: { approval_id: 'a1', tool: 'file_write', decision: 'approve' },
    });
    expect(store.getState().approvals).toEqual([]);
  });

  it('clears the in-flight guard after a transport refusal and keeps the approval unresolved', async () => {
    const { api, store } = await connectedStore();
    api.emit(approvalRequest({ nonce: 'n-1' }));

    const resolution = store.getState().resolveApproval('a1', 'deny');
    await Promise.resolve();
    expect(api.sends).toHaveLength(1);
    api.rejectNext(clientRefusal('E_APPROVAL_REPLAY'));
    await resolution;

    expect(store.getState().resolvingApprovals).toEqual([]);
    expect(store.getState().approvals).toHaveLength(1);
    expect(store.getState().approvals[0]?.decision).toBeNull();
  });
});

describe('W12 out-of-order TOOL_RESULT buffering (deterministic policy)', () => {
  it('buffers a result before its call and replays it exactly once on the late call', async () => {
    const { api, store } = await connectedStore();
    api.emit(toolResult('tc-ooo', 'denied', 'E_KILL_DENIED'));
    // No fabricated row: the tool/args/turn are only known from the TOOL_CALL.
    expect(store.getState().toolActivity).toEqual([]);

    api.emit(toolCall('tc-ooo', 1, 'file_list'));
    const rows = store.getState().toolActivity;
    expect(rows).toHaveLength(1);
    expect(rows[0]?.toolCallId).toBe('tc-ooo');
    expect(rows[0]?.tool).toBe('file_list');
    expect(rows[0]?.turn).toBe(1);
    expect(rows[0]?.outcome).toBe('denied');
    expect(rows[0]?.code).toBe('E_KILL_DENIED');

    // Terminal-once still holds after the replay.
    api.emit(toolResult('tc-ooo', 'ok'));
    expect(store.getState().toolActivity).toHaveLength(1);
    expect(store.getState().toolActivity[0]?.outcome).toBe('denied');
  });

  it('dedupes duplicate orphan results: the first terminal outcome wins', async () => {
    const { api, store } = await connectedStore();
    api.emit(toolResult('tc-dup', 'ok'));
    api.emit(toolResult('tc-dup', 'refused', 'E_SHOULD_NOT_APPLY'));

    api.emit(toolCall('tc-dup', 1));
    const row = store.getState().toolActivity[0];
    expect(store.getState().toolActivity).toHaveLength(1);
    expect(row?.outcome).toBe('ok');
    expect(row?.code).toBeUndefined();
  });

  it('never renders an orphan result that never receives a matching call', async () => {
    const { api, store } = await connectedStore();
    api.emit(toolResult('tc-orphan', 'timeout'));
    api.emit(toolCall('tc-other', 1));

    const rows = store.getState().toolActivity;
    expect(rows.map((row) => row.toolCallId)).toEqual(['tc-other']);
    expect(rows.some((row) => row.toolCallId === 'tc-orphan')).toBe(false);
  });

  it('bounds the buffer (FIFO) and drops the oldest orphan deterministically', async () => {
    const { api, store } = await connectedStore();
    for (let index = 0; index < MAX_BUFFERED_TOOL_RESULTS; index += 1) {
      api.emit(toolResult(`buf-${index}`, 'ok'));
    }
    api.emit(toolResult('buf-overflow', 'timeout')); // evicts buf-0 (oldest)

    api.emit(toolCall('buf-0', 1)); // buffer entry dropped -> starts pending
    api.emit(toolCall('buf-1', 2)); // still buffered -> replays ok
    api.emit(toolCall('buf-overflow', 3)); // buffered -> replays timeout

    const rows = store.getState().toolActivity;
    expect(rows.map((row) => row.toolCallId)).toEqual(['buf-0', 'buf-1', 'buf-overflow']);
    expect(rows[0]?.outcome).toBe('pending');
    expect(rows[1]?.outcome).toBe('ok');
    expect(rows[2]?.outcome).toBe('timeout');
  });

  it('drops buffered orphans when their run becomes terminal (never replays a finished run)', async () => {
    const { api, store } = await connectedStore();
    api.emit({ name: 'RUN_CREATED', payload: { run_id: 'run-1', status: 'running' } });
    api.emit(toolResult('tc-late', 'timeout'));
    expect(store.getState().toolActivity).toEqual([]);

    api.emit({
      name: 'RUN_STATUS',
      payload: { run_id: 'run-1', status: 'completed', error_code: null },
    });
    expect(store.getState().run?.status).toBe('completed');

    api.emit(toolCall('tc-late', 1));
    // The buffered terminal result is gone: the late call starts pending, never a replay.
    expect(store.getState().toolActivity).toHaveLength(1);
    expect(store.getState().toolActivity[0]?.outcome).toBe('pending');
  });

  it('drops a late result that arrives after its run is already terminal (stale run)', async () => {
    const { api, store } = await connectedStore();
    api.emit({ name: 'RUN_CREATED', payload: { run_id: 'run-1', status: 'running' } });
    api.emit({
      name: 'RUN_STATUS',
      payload: { run_id: 'run-1', status: 'completed', error_code: null },
    });
    expect(store.getState().run?.status).toBe('completed');

    // Fan-out-late result for the finished run: dropped, never buffered/replayed.
    api.emit(toolResult('tc-stale', 'timeout'));
    expect(store.getState().toolActivity).toEqual([]);

    api.emit(toolCall('tc-stale', 1));
    expect(store.getState().toolActivity).toHaveLength(1);
    expect(store.getState().toolActivity[0]?.outcome).toBe('pending');
  });
});
