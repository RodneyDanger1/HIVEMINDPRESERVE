/**
 * Exhaustive unit tests for the pure approval contract (`src/state/approval.ts`).
 *
 * Covers: the classifier/retryability matrix for every named refusal, the display
 * status precedence, the availability truth table, the ACK settle window, the
 * replay window/epoch/dedupe rules, the retry cap and the selectors.
 *
 * Pure data only: no React, no zustand, no DOM, no timers, no network, no commands.
 * Command/refusal copy stays in STRINGS; these tests assert codes/statuses only.
 */

import { describe, expect, it } from 'vitest';
import { KNOWN_REFUSAL_CODES } from '../src/api/contract';
import type { ApprovalRequestPayload, Refusal } from '../src/api/contract';
import type { ConnectionStatus } from '../src/api/client';
import type { ApprovalRecord } from '../src/state/store';
import {
  APPROVAL_ACK_SETTLE_CODE,
  APPROVAL_ACK_SETTLE_MS,
  APPROVAL_REPLAY_SETTLE_MS,
  APPROVAL_RESOLVE_TIMEOUT_MS,
  MAX_APPROVAL_REPLAY_EVENTS,
  MAX_APPROVAL_RESOLVE_ATTEMPTS,
  ackSettleDue,
  ackSettleTransition,
  approvalActionsEnabled,
  approvalDisplayStatus,
  approvalStatusAfterRequest,
  beginReplayWindow,
  canRetryApproval,
  classifyApprovalRefusal,
  dedupeReplayApprovals,
  initialApprovalReplayState,
  initialApprovalResolveState,
  isApprovalExpired,
  isReplayWindowOpen,
  markApprovalsStale,
  selectApprovalResolve,
  selectApprovalResolveMap,
  selectApprovals,
  selectApprovalsStale,
  settleReplay,
  settleReplayStatus,
  stampReplayEpoch,
} from '../src/state/approval';
import type {
  ApprovalDisplayStatus,
  ApprovalFailureKind,
  ApprovalReplayState,
  ApprovalResolveState,
  ApprovalStateSlice,
} from '../src/state/approval';

const NOW = 1_800_000_000_000;

function iso(offsetMs: number): string {
  return new Date(NOW + offsetMs).toISOString();
}

function record(overrides: Partial<ApprovalRecord> = {}): ApprovalRecord {
  const base: ApprovalRecord = {
    approvalId: 'approval-1',
    tool: 'file_write',
    nonce: 'nonce-1',
    decision: null,
    requestedAt: NOW - 1_000,
    runId: null,
    taskId: null,
    toolCallId: null,
    argsSha256: null,
    expiresAt: iso(60_000),
  };
  return { ...base, ...overrides } as ApprovalRecord;
}

function refusal(code: string): Refusal {
  return { code, message: `message:${code}`, how_to_fix: `fix:${code}`, retryable: false };
}

function resolveState(overrides: Partial<ApprovalResolveState> = {}): ApprovalResolveState {
  return { ...initialApprovalResolveState(), ...overrides } as ApprovalResolveState;
}

function payload(overrides: Partial<ApprovalRequestPayload> = {}): ApprovalRequestPayload {
  return { approval_id: 'approval-1', tool: 'file_write', ...overrides } as ApprovalRequestPayload;
}

function replayState(overrides: Partial<ApprovalReplayState> = {}): ApprovalReplayState {
  return { ...initialApprovalReplayState(), ...overrides } as ApprovalReplayState;
}

/* ------------------------------------------------------------------ */
/* constants                                                           */
/* ------------------------------------------------------------------ */

describe('approval contract constants', () => {
  it('matches the frozen timing, retry and replay bounds', () => {
    expect(APPROVAL_REPLAY_SETTLE_MS).toBe(400);
    expect(MAX_APPROVAL_REPLAY_EVENTS).toBe(20);
    expect(MAX_APPROVAL_RESOLVE_ATTEMPTS).toBe(3);
    expect(APPROVAL_ACK_SETTLE_MS).toBe(2_000);
    expect(APPROVAL_RESOLVE_TIMEOUT_MS).toBe(15_000);
    expect(APPROVAL_ACK_SETTLE_CODE).toBe('E_WS_DISCONNECTED');
  });
});

/* ------------------------------------------------------------------ */
/* initial resolve state                                               */
/* ------------------------------------------------------------------ */

describe('initialApprovalResolveState', () => {
  it('is the neutral pending state with no attempts and no refusal', () => {
    expect(initialApprovalResolveState()).toEqual({
      status: 'pending',
      attempts: 0,
      lastDecision: null,
      refusal: null,
      lastAttemptAt: null,
    });
  });

  it('returns a fresh mutable copy each call', () => {
    const first = initialApprovalResolveState();
    const second = initialApprovalResolveState();
    expect(first).not.toBe(second);
    expect(first).toEqual(second);
  });
});

/* ------------------------------------------------------------------ */
/* classifier / retryability matrix                                    */
/* ------------------------------------------------------------------ */

const CLASSIFIER_MATRIX: ReadonlyArray<
  readonly [string, ApprovalFailureKind, ApprovalDisplayStatus, boolean]
> = [
  ['E_WS_DISCONNECTED', 'transport', 'retryable', true],
  ['E_RESYNC_REQUIRED', 'transport', 'retryable', true],
  ['E_DB_BUSY', 'transport', 'retryable', true],
  ['E_INTERNAL_ERROR', 'transport', 'retryable', true],
  ['E_APPROVAL_DECISION_INVALID', 'operator', 'retryable', true],
  ['E_SCHEMA_INVALID', 'operator', 'retryable', true],
  ['E_APPROVAL_REPLAY', 'stale', 'stale', false],
  ['E_APPROVAL_NOT_FOUND', 'stale', 'stale', false],
  ['E_APPROVAL_TIMEOUT', 'expired', 'expired', false],
  ['E_APPROVAL_REQUIRED', 'expired', 'expired', false],
];

describe('classifyApprovalRefusal', () => {
  it('matches every row of the named-refusal matrix', () => {
    for (const [code, kind, status, retryable] of CLASSIFIER_MATRIX) {
      expect(classifyApprovalRefusal(refusal(code)), code).toEqual({
        kind,
        status,
        retryable,
      });
    }
  });

  it('defaults unknown named codes to bounded operator retry', () => {
    expect(classifyApprovalRefusal(refusal('E_ORIGIN_REJECTED'))).toEqual({
      kind: 'operator',
      status: 'retryable',
      retryable: true,
    });
    expect(classifyApprovalRefusal(refusal('E_TOOL_TIMEOUT'))).toEqual({
      kind: 'operator',
      status: 'retryable',
      retryable: true,
    });
  });

  it('classifies every frozen contract refusal into a valid kind/status', () => {
    const kinds: readonly ApprovalFailureKind[] = [
      'transport',
      'operator',
      'stale',
      'expired',
      'terminal',
      'ignored',
    ];
    const statuses: readonly ApprovalDisplayStatus[] = [
      'pending',
      'resolving',
      'awaiting_server',
      'retryable',
      'expired',
      'stale',
      'unconfirmed',
    ];
    expect(KNOWN_REFUSAL_CODES.length).toBeGreaterThanOrEqual(65);
    for (const code of KNOWN_REFUSAL_CODES) {
      const classified = classifyApprovalRefusal(refusal(code));
      expect(kinds, code).toContain(classified.kind);
      expect(statuses, code).toContain(classified.status);
      expect(typeof classified.retryable, code).toBe('boolean');
      if (classified.kind === 'stale' || classified.kind === 'expired') {
        expect(classified.retryable, code).toBe(false);
      } else {
        expect(classified.retryable, code).toBe(true);
      }
    }
  });

  it('never retries stale or expired classes and always retries transport/operator', () => {
    expect(classifyApprovalRefusal(refusal('E_APPROVAL_REPLAY')).retryable).toBe(false);
    expect(classifyApprovalRefusal(refusal('E_APPROVAL_NOT_FOUND')).retryable).toBe(false);
    expect(classifyApprovalRefusal(refusal('E_APPROVAL_TIMEOUT')).retryable).toBe(false);
    expect(classifyApprovalRefusal(refusal('E_APPROVAL_REQUIRED')).retryable).toBe(false);
    expect(classifyApprovalRefusal(refusal('E_WS_DISCONNECTED')).retryable).toBe(true);
    expect(classifyApprovalRefusal(refusal('E_DECISION_REQUIRED')).retryable).toBe(true);
  });
});

/* ------------------------------------------------------------------ */
/* client-clock expiry                                                 */
/* ------------------------------------------------------------------ */

describe('isApprovalExpired', () => {
  it('treats missing or invalid expiry as unknown, never expired', () => {
    expect(isApprovalExpired(null, NOW)).toBe(false);
    expect(isApprovalExpired(undefined, NOW)).toBe(false);
    expect(isApprovalExpired('', NOW)).toBe(false);
    expect(isApprovalExpired('not-a-date', NOW)).toBe(false);
  });

  it('is expired at or before now and live strictly after now', () => {
    expect(isApprovalExpired(iso(-1), NOW)).toBe(true);
    expect(isApprovalExpired(iso(0), NOW)).toBe(true);
    expect(isApprovalExpired(iso(1), NOW)).toBe(false);
    expect(isApprovalExpired(iso(60_000), NOW)).toBe(false);
  });
});

/* ------------------------------------------------------------------ */
/* display status precedence                                           */
/* ------------------------------------------------------------------ */

describe('approvalDisplayStatus', () => {
  it('prefers an active resolve status over an expired client clock', () => {
    const expiredRecord = record({ expiresAt: iso(-1_000) });
    for (const status of ['resolving', 'awaiting_server', 'retryable', 'stale', 'unconfirmed'] as const) {
      expect(approvalDisplayStatus(expiredRecord, resolveState({ status }), NOW)).toBe(status);
    }
    expect(approvalDisplayStatus(expiredRecord, resolveState({ status: 'expired' }), NOW)).toBe(
      'expired',
    );
  });

  it('falls back to the client-clock expiry and then pending', () => {
    expect(approvalDisplayStatus(record({ expiresAt: iso(-1) }), undefined, NOW)).toBe('expired');
    expect(approvalDisplayStatus(record({ expiresAt: iso(1) }), undefined, NOW)).toBe('pending');
    expect(approvalDisplayStatus(record({ expiresAt: null }), undefined, NOW)).toBe('pending');
    expect(approvalDisplayStatus(record({ expiresAt: 'bad' }), undefined, NOW)).toBe('pending');
    expect(approvalDisplayStatus(record(), initialApprovalResolveState(), NOW)).toBe('pending');
  });
});

/* ------------------------------------------------------------------ */
/* availability truth table                                            */
/* ------------------------------------------------------------------ */

interface AvailabilityRow {
  readonly label: string;
  readonly record: ApprovalRecord;
  readonly resolve: ApprovalResolveState | undefined;
  readonly connection: ConnectionStatus;
  readonly expected: {
    readonly approve: boolean;
    readonly deny: boolean;
    readonly retry: boolean;
    readonly dismiss: boolean;
  };
}

const AVAILABILITY_ROWS: readonly AvailabilityRow[] = [
  {
    label: 'pending + nonce + open',
    record: record(),
    resolve: resolveState(),
    connection: 'open',
    expected: { approve: true, deny: true, retry: false, dismiss: false },
  },
  {
    label: 'pending + nonce + reconnecting',
    record: record(),
    resolve: undefined,
    connection: 'reconnecting',
    expected: { approve: false, deny: false, retry: false, dismiss: false },
  },
  {
    label: 'pending + nonce-less',
    record: record({ nonce: null }),
    resolve: undefined,
    connection: 'open',
    expected: { approve: false, deny: false, retry: false, dismiss: false },
  },
  {
    label: 'pending + client-clock expired',
    record: record({ expiresAt: iso(-1) }),
    resolve: undefined,
    connection: 'open',
    expected: { approve: false, deny: false, retry: false, dismiss: true },
  },
  {
    label: 'resolving',
    record: record(),
    resolve: resolveState({ status: 'resolving', attempts: 1 }),
    connection: 'open',
    expected: { approve: false, deny: false, retry: false, dismiss: false },
  },
  {
    label: 'awaiting_server',
    record: record(),
    resolve: resolveState({ status: 'awaiting_server', attempts: 1 }),
    connection: 'open',
    expected: { approve: false, deny: false, retry: false, dismiss: false },
  },
  {
    label: 'retryable below cap + nonce + open',
    record: record(),
    resolve: resolveState({ status: 'retryable', attempts: 1 }),
    connection: 'open',
    expected: { approve: false, deny: false, retry: true, dismiss: false },
  },
  {
    label: 'retryable + connection not open',
    record: record(),
    resolve: resolveState({ status: 'retryable', attempts: 1 }),
    connection: 'reconnecting',
    expected: { approve: false, deny: false, retry: false, dismiss: false },
  },
  {
    label: 'retryable + nonce-less',
    record: record({ nonce: null }),
    resolve: resolveState({ status: 'retryable', attempts: 1 }),
    connection: 'open',
    expected: { approve: false, deny: false, retry: false, dismiss: false },
  },
  {
    label: 'retryable + retry cap reached',
    record: record(),
    resolve: resolveState({ status: 'retryable', attempts: MAX_APPROVAL_RESOLVE_ATTEMPTS }),
    connection: 'open',
    expected: { approve: false, deny: false, retry: false, dismiss: false },
  },
  {
    label: 'expired (resolve status)',
    record: record(),
    resolve: resolveState({ status: 'expired', attempts: 1 }),
    connection: 'open',
    expected: { approve: false, deny: false, retry: false, dismiss: true },
  },
  {
    label: 'stale',
    record: record(),
    resolve: resolveState({ status: 'stale', attempts: 1 }),
    connection: 'open',
    expected: { approve: false, deny: false, retry: false, dismiss: true },
  },
  {
    label: 'unconfirmed',
    record: record(),
    resolve: resolveState({ status: 'unconfirmed', attempts: 0 }),
    connection: 'open',
    expected: { approve: false, deny: false, retry: false, dismiss: true },
  },
  {
    label: 'expired resolve state without nonce',
    record: record({ nonce: null }),
    resolve: resolveState({ status: 'expired' }),
    connection: 'open',
    expected: { approve: false, deny: false, retry: false, dismiss: true },
  },
];

describe('approvalActionsEnabled', () => {
  it('matches the full availability truth table', () => {
    for (const row of AVAILABILITY_ROWS) {
      expect(
        approvalActionsEnabled({
          record: row.record,
          resolve: row.resolve,
          nowMs: NOW,
          connection: row.connection,
        }),
        row.label,
      ).toEqual(row.expected);
    }
  });

  it('stays disabled for every non-open connection status while pending', () => {
    const nonOpen: readonly ConnectionStatus[] = ['idle', 'connecting', 'reconnecting', 'closed', 'error'];
    for (const connection of nonOpen) {
      expect(approvalActionsEnabled({ record: record(), resolve: undefined, nowMs: NOW, connection }), connection).toEqual({
        approve: false,
        deny: false,
        retry: false,
        dismiss: false,
      });
    }
  });
});

describe('canRetryApproval', () => {
  it('allows retry only for a fresh retryable card below the cap on an open connection', () => {
    expect(canRetryApproval({ record: record(), resolve: resolveState({ status: 'retryable', attempts: 2 }), nowMs: NOW, connection: 'open' })).toBe(true);
    expect(canRetryApproval({ record: record(), resolve: resolveState({ status: 'retryable', attempts: MAX_APPROVAL_RESOLVE_ATTEMPTS }), nowMs: NOW, connection: 'open' })).toBe(false);
    expect(canRetryApproval({ record: record(), resolve: resolveState({ status: 'retryable', attempts: 1 }), nowMs: NOW, connection: 'reconnecting' })).toBe(false);
    expect(canRetryApproval({ record: record({ expiresAt: iso(-1) }), resolve: resolveState({ status: 'retryable', attempts: 1 }), nowMs: NOW, connection: 'open' })).toBe(false);
    expect(canRetryApproval({ record: record({ nonce: null }), resolve: resolveState({ status: 'retryable', attempts: 1 }), nowMs: NOW, connection: 'open' })).toBe(false);
    expect(canRetryApproval({ record: record(), resolve: undefined, nowMs: NOW, connection: 'open' })).toBe(false);
  });
});

/* ------------------------------------------------------------------ */
/* replay window / epoch / settle                                      */
/* ------------------------------------------------------------------ */

describe('replay window state', () => {
  it('starts closed, fresh and unstamped', () => {
    expect(initialApprovalReplayState()).toEqual({
      stale: false,
      epoch: 0,
      window: false,
      lastReplayAt: null,
      source: null,
    });
    expect(isReplayWindowOpen(initialApprovalReplayState())).toBe(false);
  });

  it('marks stale by advancing the epoch and closing the window', () => {
    const initial = replayState({ lastReplayAt: 123, source: 'hello' });
    const stale = markApprovalsStale(initial);
    expect(stale).toEqual({
      stale: true,
      epoch: 1,
      window: false,
      lastReplayAt: 123,
      source: 'hello',
    });
    expect(initial.stale).toBe(false);
    expect(initial.epoch).toBe(0);
    expect(stale).not.toBe(initial);
    expect(markApprovalsStale(stale).epoch).toBe(2);
  });

  it('opens a bounded window on SERVER_HELLO and preserves epoch/stale', () => {
    const stale = replayState({ stale: true, epoch: 3 });
    const window = beginReplayWindow(stale, 'hello', NOW);
    expect(window).toEqual({ stale: true, epoch: 3, window: true, lastReplayAt: NOW, source: 'hello' });
    expect(isReplayWindowOpen(window)).toBe(true);
    expect(beginReplayWindow(replayState(), 'session', NOW + 1).source).toBe('session');
  });

  it('settles the window and clears the stale marker', () => {
    const open = beginReplayWindow(replayState({ stale: true, epoch: 2 }), 'hello', NOW);
    expect(settleReplay(open)).toEqual({
      stale: false,
      epoch: 2,
      window: false,
      lastReplayAt: NOW,
      source: 'hello',
    });
  });

  it('stamps seen epochs immutably', () => {
    const seen: Readonly<Record<string, number>> = { 'approval-1': 1 };
    const stamped = stampReplayEpoch(seen, 'approval-2', 2);
    expect(stamped).toEqual({ 'approval-1': 1, 'approval-2': 2 });
    expect(seen).toEqual({ 'approval-1': 1 });
    expect(stampReplayEpoch(stamped, 'approval-1', 5)).toEqual({
      'approval-1': 5,
      'approval-2': 2,
    });
  });
});

describe('settleReplayStatus', () => {
  it('confirms records seen in the current epoch', () => {
    expect(settleReplayStatus('unconfirmed', 2, 2)).toBe('pending');
    expect(settleReplayStatus('pending', 2, 2)).toBe('pending');
    expect(settleReplayStatus('retryable', 2, 2)).toBe('retryable');
    expect(settleReplayStatus('resolving', 2, 2)).toBe('resolving');
    expect(settleReplayStatus('awaiting_server', 2, 2)).toBe('awaiting_server');
  });

  it('unconfirms actionable records not seen in the current epoch', () => {
    expect(settleReplayStatus('pending', undefined, 2)).toBe('unconfirmed');
    expect(settleReplayStatus('pending', 1, 2)).toBe('unconfirmed');
    expect(settleReplayStatus('retryable', 1, 2)).toBe('unconfirmed');
  });

  it('preserves in-flight and terminal records not seen in the current epoch', () => {
    expect(settleReplayStatus('resolving', undefined, 2)).toBe('resolving');
    expect(settleReplayStatus('awaiting_server', 1, 2)).toBe('awaiting_server');
    expect(settleReplayStatus('expired', undefined, 2)).toBe('expired');
    expect(settleReplayStatus('stale', 1, 2)).toBe('stale');
    expect(settleReplayStatus('unconfirmed', 1, 2)).toBe('unconfirmed');
  });
});

describe('approvalStatusAfterRequest', () => {
  it('preserves in-flight states even when the payload expiry is past', () => {
    expect(approvalStatusAfterRequest('resolving', payload({ expires_at: iso(-1) }), NOW)).toBe('resolving');
    expect(approvalStatusAfterRequest('awaiting_server', payload({ expires_at: null }), NOW)).toBe('awaiting_server');
  });

  it('confirms read-only terminal states only with a future expiry', () => {
    expect(approvalStatusAfterRequest('unconfirmed', payload({ expires_at: iso(60_000) }), NOW)).toBe('pending');
    expect(approvalStatusAfterRequest('stale', payload({ expires_at: iso(1) }), NOW)).toBe('pending');
    expect(approvalStatusAfterRequest('expired', payload({ expires_at: iso(1) }), NOW)).toBe('pending');
    expect(approvalStatusAfterRequest('unconfirmed', payload({ expires_at: iso(-1) }), NOW)).toBe('unconfirmed');
    expect(approvalStatusAfterRequest('stale', payload({ expires_at: null }), NOW)).toBe('stale');
    expect(approvalStatusAfterRequest('expired', payload({ expires_at: null }), NOW)).toBe('expired');
  });

  it('resets every other arrival to pending', () => {
    expect(approvalStatusAfterRequest(undefined, payload(), NOW)).toBe('pending');
    expect(approvalStatusAfterRequest('pending', payload(), NOW)).toBe('pending');
    expect(approvalStatusAfterRequest('retryable', payload(), NOW)).toBe('pending');
    expect(approvalStatusAfterRequest('pending', payload({ expires_at: iso(-1) }), NOW)).toBe('pending');
  });
});

describe('dedupeReplayApprovals', () => {
  it('keeps one payload per approval id, last wins, first-seen order preserved', () => {
    const deduped = dedupeReplayApprovals([
      payload({ approval_id: 'a', tool: 'file_write' }),
      payload({ approval_id: 'b', tool: 'shell_exec' }),
      payload({ approval_id: 'a', tool: 'file_list' }),
    ]);
    expect(deduped.map((item) => item.approval_id)).toEqual(['a', 'b']);
    expect(deduped[0]?.tool).toBe('file_list');
    expect(deduped).toHaveLength(2);
  });

  it('bounds the batch at MAX_APPROVAL_REPLAY_EVENTS by default', () => {
    const many = Array.from({ length: MAX_APPROVAL_REPLAY_EVENTS + 2 }, (_value, index) =>
      payload({ approval_id: `approval-${String(index)}` }),
    );
    const deduped = dedupeReplayApprovals(many);
    expect(deduped).toHaveLength(MAX_APPROVAL_REPLAY_EVENTS);
    expect(deduped[0]?.approval_id).toBe('approval-0');
    expect(deduped.at(-1)?.approval_id).toBe(`approval-${String(MAX_APPROVAL_REPLAY_EVENTS - 1)}`);
  });

  it('never mutates the input and handles empty/zero bounds', () => {
    const input = [payload({ approval_id: 'a' })];
    const snapshot = [...input];
    dedupeReplayApprovals(input);
    expect(input).toEqual(snapshot);
    expect(dedupeReplayApprovals([])).toEqual([]);
    expect(dedupeReplayApprovals(input, 0)).toEqual([]);
  });
});

/* ------------------------------------------------------------------ */
/* ACK settle window                                                   */
/* ------------------------------------------------------------------ */

describe('ACK settle window', () => {
  it('is due only for an awaiting_server card past the settle budget', () => {
    const atThreshold = resolveState({
      status: 'awaiting_server',
      attempts: 1,
      lastAttemptAt: NOW - APPROVAL_ACK_SETTLE_MS,
    });
    const beforeThreshold = resolveState({
      status: 'awaiting_server',
      attempts: 1,
      lastAttemptAt: NOW - APPROVAL_ACK_SETTLE_MS + 1,
    });
    expect(ackSettleDue(undefined, NOW)).toBe(false);
    expect(ackSettleDue(resolveState({ status: 'pending' }), NOW)).toBe(false);
    expect(ackSettleDue(resolveState({ status: 'resolving' }), NOW)).toBe(false);
    expect(ackSettleDue(resolveState({ status: 'retryable' }), NOW)).toBe(false);
    expect(ackSettleDue(resolveState({ status: 'awaiting_server', lastAttemptAt: null }), NOW)).toBe(false);
    expect(ackSettleDue(beforeThreshold, NOW)).toBe(false);
    expect(ackSettleDue(atThreshold, NOW)).toBe(true);
  });

  it('offers the retryable transition only when the card is still waiting', () => {
    expect(ackSettleTransition(undefined)).toBeNull();
    expect(ackSettleTransition(resolveState({ status: 'pending' }))).toBeNull();
    expect(ackSettleTransition(resolveState({ status: 'retryable' }))).toBeNull();
    expect(ackSettleTransition(resolveState({ status: 'resolving' }))).toBeNull();
    expect(ackSettleTransition(resolveState({ status: 'awaiting_server', attempts: 2 }))).toEqual({
      status: 'retryable',
      code: 'E_WS_DISCONNECTED',
    });
    expect(APPROVAL_ACK_SETTLE_CODE).toBe('E_WS_DISCONNECTED');
  });
});

/* ------------------------------------------------------------------ */
/* selectors                                                           */
/* ------------------------------------------------------------------ */

describe('approval selectors', () => {
  const slice: ApprovalStateSlice = {
    approvals: [record()],
    approvalsStale: true,
    approvalResolve: {
      'approval-1': resolveState({ status: 'retryable', attempts: 1 }),
    },
  };

  it('reads the approval list and stale flag directly', () => {
    expect(selectApprovals(slice)).toBe(slice.approvals);
    expect(selectApprovalsStale(slice)).toBe(true);
  });

  it('selects one resolve state or the whole map', () => {
    expect(selectApprovalResolve('approval-1')(slice)?.status).toBe('retryable');
    expect(selectApprovalResolve('missing')(slice)).toBeUndefined();
    expect(selectApprovalResolveMap(slice)).toBe(slice.approvalResolve);
  });
});
