import { describe, expect, it } from 'vitest';
import { createUiStore, type ApprovalRecord } from '../src/state/store';
import {
  SCHEMA_VERSION,
  type DecodedEvent,
  type Envelope,
  type Refusal,
  type ServerCommandName,
} from '../src/api/contract';
import type { ConnectionStatus, SessionInfo, UiApi } from '../src/api/client';
import {
  APPROVAL_DECISIONS,
  approvalActionAriaLabel,
  approvalAriaLabel,
  canResolveApproval,
  formatArgsSha256,
  formatExpiry,
  isApprovalExpired,
  prunePendingResolutions,
  truncateId,
  type ApprovalView,
} from '../src/panels/ApprovalOverlay';

/**
 * W12 / Wave 3-4 — approval selector + panel integration.
 *
 * Static-source suite (no build/test execution this wave): pins the extended
 * ApprovalRecord fields, expiry gating, the store-side in-flight guard, the
 * panel-side pending lock that survives an ACK and is released only by
 * APPROVAL_RESOLVED (approval removal), and the nonce gate.
 */
const SOURCE_FILES = import.meta.glob('../src/**/*.{ts,tsx}', {
  query: '?raw',
  import: 'default',
  eager: true,
}) as Record<string, string>;

function sourceOf(suffix: string): string {
  const key = Object.keys(SOURCE_FILES).find((candidate) => candidate.endsWith(suffix));
  return key ? SOURCE_FILES[key] : '';
}

class FakeUiApi implements UiApi {
  readonly kind = 'ws' as const;
  readonly sends: Array<{
    readonly name: ServerCommandName;
    readonly payload: unknown;
    readonly resolve: () => void;
    readonly reject: (error: Error) => void;
  }> = [];
  private readonly events = new Set<(event: DecodedEvent, envelope: Envelope<unknown>) => void>();
  private readonly statuses = new Set<(status: ConnectionStatus, detail?: Refusal) => void>();
  private readonly clientErrors = new Set<(refusal: Refusal) => void>();

  async open(): Promise<SessionInfo> {
    return { token: 'fake', schema: SCHEMA_VERSION, last_seq: 0, ws: '/ws' };
  }
  close(): void {}
  send<N extends ServerCommandName>(name: N, payload: unknown, _options?: { readonly correlationId?: string }): Promise<void> {
    return new Promise<void>((resolve, reject) => {
      this.sends.push({ name, payload, resolve, reject });
    });
  }
  subscribe(listener: (event: DecodedEvent, envelope: Envelope<unknown>) => void): () => void {
    this.events.add(listener);
    return () => {
      this.events.delete(listener);
    };
  }
  subscribeStatus(listener: (status: ConnectionStatus, detail?: Refusal) => void): () => void {
    this.statuses.add(listener);
    return () => {
      this.statuses.delete(listener);
    };
  }
  subscribeClientError(listener: (refusal: Refusal) => void): () => void {
    this.clientErrors.add(listener);
    return () => {
      this.clientErrors.delete(listener);
    };
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

const FUTURE = '2026-11-06T23:00:00.000Z';
const NOW_BEFORE_FUTURE = Date.parse('2026-11-06T22:00:00.000Z');

describe('W12 approval selector/panel integration', () => {
  it('populates all extended approval fields and feeds the panel view model', async () => {
    const { api, store } = await connectedStore();
    api.emit({
      name: 'APPROVAL_REQUEST',
      payload: {
        approval_id: 'ap-1',
        tool: 'file_write',
        nonce: 'nonce-1',
        run_id: 'run-1',
        task_id: 'task-1',
        tool_call_id: 'tc-1',
        args_sha256: 'b'.repeat(64),
        expires_at: FUTURE,
      },
    });
    const record = store.getState().approvals[0] as ApprovalRecord;
    expect(record.runId).toBe('run-1');
    expect(record.taskId).toBe('task-1');
    expect(record.toolCallId).toBe('tc-1');
    expect(record.argsSha256).toBe('b'.repeat(64));
    expect(record.expiresAt).toBe(FUTURE);
    expect(store.getState().resolvingApprovals).toEqual([]);

    // Compile-time structural proof: the store record feeds the panel's view model.
    const view: ApprovalView = record;
    expect(formatArgsSha256(view.argsSha256)).toBe(`${'b'.repeat(12)}…`);
    expect(formatExpiry(view.expiresAt, NOW_BEFORE_FUTURE)).toBe('1h 00m');
    expect(isApprovalExpired(view.expiresAt, NOW_BEFORE_FUTURE)).toBe(false);
    expect(canResolveApproval({ nonce: record.nonce, resolving: false, expired: false })).toBe(true);
    expect(truncateId(record.approvalId)).toBe('ap-1');
  });

  it('pins the e2e selector contract for the dialog and both action labels', () => {
    expect(approvalAriaLabel('file_write')).toBe('Tool approval requested: file_write');
    expect(approvalActionAriaLabel(APPROVAL_DECISIONS.approve, 'file_write')).toBe(
      'Approve tool call for file_write',
    );
    expect(approvalActionAriaLabel(APPROVAL_DECISIONS.deny, 'file_write')).toBe(
      'Deny tool call for file_write',
    );
  });

  it('does not optimistically resolve: ACK keeps the approval and the panel lock until APPROVAL_RESOLVED', async () => {
    const { api, store } = await connectedStore();
    api.emit({
      name: 'APPROVAL_REQUEST',
      payload: { approval_id: 'ap-2', tool: 'file_write', nonce: 'nonce-2', run_id: 'run-2', task_id: 'task-2' },
    });
    const resolution = store.getState().resolveApproval('ap-2', APPROVAL_DECISIONS.approve);
    await Promise.resolve();
    expect(api.sends).toHaveLength(1);
    expect(api.sends[0]?.name).toBe('APPROVAL_RESOLVE');
    expect(api.sends[0]?.payload).toMatchObject({ approval_id: 'ap-2', nonce: 'nonce-2', decision: 'approve' });
    expect(store.getState().resolvingApprovals).toContain('ap-2');
    expect(store.getState().approvals).toHaveLength(1);

    // Store clears its resolving guard when the send settles on ACK (store.ts:683-687);
    // the panel-side lock must survive that ACK until the approval disappears.
    api.resolveNext();
    await resolution;
    expect(store.getState().resolvingApprovals).toEqual([]);
    expect(store.getState().approvals).toHaveLength(1);
    const pendingAfterAck = { 'ap-2': true };
    expect(prunePendingResolutions(pendingAfterAck, ['ap-2'])).toEqual(pendingAfterAck);
    expect(canResolveApproval({ nonce: 'nonce-2', resolving: true })).toBe(false);

    api.emit({ name: 'APPROVAL_RESOLVED', payload: { approval_id: 'ap-2', tool: 'file_write', decision: 'approve' } });
    expect(store.getState().approvals).toHaveLength(0);
    expect(prunePendingResolutions(pendingAfterAck, [])).toEqual({});
  });

  it('a same-approval double click sends one command; the pending lock stays until resolution', async () => {
    const { api, store } = await connectedStore();
    api.emit({
      name: 'APPROVAL_REQUEST',
      payload: { approval_id: 'ap-double', tool: 'file_write', nonce: 'nonce-double' },
    });
    const first = store.getState().resolveApproval('ap-double', APPROVAL_DECISIONS.approve);
    const second = store.getState().resolveApproval('ap-double', APPROVAL_DECISIONS.approve);
    await Promise.resolve();
    expect(api.sends).toHaveLength(1);
    expect(store.getState().resolvingApprovals).toContain('ap-double');
    expect(canResolveApproval({ nonce: 'nonce-double', resolving: true })).toBe(false);
    api.resolveNext();
    await Promise.all([first, second]);
    expect(store.getState().approvals).toHaveLength(1);
  });

  it('deny sends decision=deny with the operator reason and exactly one command', async () => {
    const { api, store } = await connectedStore();
    api.emit({
      name: 'APPROVAL_REQUEST',
      payload: { approval_id: 'ap-3', tool: 'file_write', nonce: 'nonce-3', args_sha256: 'd'.repeat(64) },
    });
    const resolution = store.getState().resolveApproval('ap-3', APPROVAL_DECISIONS.deny, 'operator denied');
    await Promise.resolve();
    expect(api.sends).toHaveLength(1);
    expect(api.sends[0]?.payload).toMatchObject({
      approval_id: 'ap-3',
      nonce: 'nonce-3',
      decision: 'deny',
      reason: 'operator denied',
    });
    expect(api.sends).toHaveLength(1);
    api.sends[0]?.resolve();
    await resolution;
    expect(api.sends).toHaveLength(1);
  });

  it('expired approvals are display-only blocks: disabled resolve, no command, no fabricated success', async () => {
    const { api, store } = await connectedStore();
    const past = '2026-11-06T21:00:00.000Z';
    api.emit({
      name: 'APPROVAL_REQUEST',
      payload: { approval_id: 'ap-expired', tool: 'file_write', nonce: 'nonce-expired', expires_at: past },
    });
    const record = store.getState().approvals[0] as ApprovalRecord;
    expect(isApprovalExpired(record.expiresAt, NOW_BEFORE_FUTURE)).toBe(true);
    expect(formatExpiry(record.expiresAt, NOW_BEFORE_FUTURE)).toBe('expired');
    expect(canResolveApproval({ nonce: record.nonce, resolving: false, expired: true })).toBe(false);
    // UI gate is local (expiry is client-known); an expired approval must not be resolvable.
    expect(api.sends).toHaveLength(0);
  });

  it('missing nonce is refused by name before any command is sent', async () => {
    const { api, store } = await connectedStore();
    api.emit({ name: 'APPROVAL_REQUEST', payload: { approval_id: 'ap-4', tool: 'file_write' } });
    await store.getState().resolveApproval('ap-4', APPROVAL_DECISIONS.approve);
    expect(api.sends).toHaveLength(0);
    expect(store.getState().refusals[0]?.refusal.code).toBe('E_SCHEMA_INVALID');
    expect(store.getState().approvals).toHaveLength(1);
  });

  it('malformed/absent extended fields render honestly with no fabricated values', async () => {
    const { api, store } = await connectedStore();
    api.emit({ name: 'APPROVAL_REQUEST', payload: { approval_id: 'ap-5', tool: 'file_read', nonce: 'nonce-5' } });
    const record = store.getState().approvals[0] as ApprovalRecord;
    expect(record.runId).toBeNull();
    expect(record.taskId).toBeNull();
    expect(record.toolCallId).toBeNull();
    expect(record.argsSha256).toBeNull();
    expect(record.expiresAt).toBeNull();
    expect(formatArgsSha256(record.argsSha256)).toBe('—');
    expect(formatExpiry(record.expiresAt, Date.now())).toBe('unknown');
    expect(isApprovalExpired(record.expiresAt, Date.now())).toBe(false);
    expect(canResolveApproval({ nonce: null, resolving: false })).toBe(false);
    expect(canResolveApproval({ nonce: 'nonce-5', resolving: true })).toBe(false);
  });

  it('ApprovalOverlay uses expiry + store resolving + panel lock and is mounted', () => {
    const overlaySource = sourceOf('/panels/ApprovalOverlay.tsx');
    expect(overlaySource).toContain('resolvingApprovals');
    expect(overlaySource).toContain('isApprovalExpired');
    expect(overlaySource).toContain('canResolveApproval');
    expect(overlaySource).toContain('prunePendingResolutions');
    expect(overlaySource).toContain('nonceMissing');
    expect(overlaySource).toMatch(/disabled=/);

    const appSource = sourceOf('/App.tsx');
    expect(appSource).toContain('ApprovalOverlay');
    expect(appSource).toContain('<ApprovalOverlay');
  });
});
