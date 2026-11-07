/**
 * Pure approval state contract for the web client.
 *
 * Extracted from `plans/foresight/CODE-APPROVAL-UX.md` §3.3/§5 (REF) and
 * `plans/foresight/RUN2-WEB-CODE.md` H1/H8 so the store, the cards and the
 * tests share exactly one classifier/display/availability/replay vocabulary.
 *
 * This module is intentionally pure: no React, no zustand, no DOM, no timers,
 * no command sending, no nonce consumption. Type-only imports keep it free of
 * runtime cycles. Timers and the single-use APPROVAL_RESOLVE command stay in
 * `state/store.ts` (a later store hunk); this module only decides *what* the
 * state is and *what is available*.
 *
 * Wire/refusal vocabulary is frozen: codes come from `api/contract.ts`
 * (`KNOWN_REFUSAL_CODES`, 65 codes) and no new refusal is invented here.
 */

import type { ApprovalRequestPayload, Refusal } from '../api/contract';
import type { ConnectionStatus } from '../api/client';
import type { ApprovalRecord } from './store';

/** Time window after SERVER_HELLO during which APPROVAL_REQUEST frames are attributed to replay. */
export const APPROVAL_REPLAY_SETTLE_MS = 400;
/** Bounded replay batch mirror of the server constant (diagnostics/tests only). */
export const MAX_APPROVAL_REPLAY_EVENTS = 20;
/** Manual retry attempts per approval (nonce stays single-use; retry reuses it). */
export const MAX_APPROVAL_RESOLVE_ATTEMPTS = 3;
/** ACK→APPROVAL_RESOLVED settlement budget before a retry is offered. */
export const APPROVAL_ACK_SETTLE_MS = 2_000;
/** Explicit command timeout for APPROVAL_RESOLVE (matches WsApiClient default, named). */
export const APPROVAL_RESOLVE_TIMEOUT_MS = 15_000;
/**
 * Synthetic, named transport refusal used when an ACK is not followed by
 * APPROVAL_RESOLVED inside {@link APPROVAL_ACK_SETTLE_MS}. The store builds the
 * full `Refusal` via `clientRefusal(APPROVAL_ACK_SETTLE_CODE, { message })` so
 * copy stays in `STRINGS`; this module only names the code.
 */
export const APPROVAL_ACK_SETTLE_CODE = 'E_WS_DISCONNECTED';

/** Where a replay batch came from (`hello` = after SERVER_HELLO, `session` = live frame). */
export type ApprovalReplaySource = 'hello' | 'session';

export type ApprovalDisplayStatus =
  | 'pending'
  | 'resolving'
  | 'awaiting_server'
  | 'retryable'
  | 'expired'
  | 'stale'
  | 'unconfirmed';

export type ApprovalFailureKind =
  | 'transport'
  | 'operator'
  | 'stale'
  | 'expired'
  | 'terminal'
  | 'ignored';

/** Per-approval resolve machine state (store-owned; pure data shape). */
export interface ApprovalResolveState {
  /** Display status; `pending` is the neutral initial value (no `idle` status exists). */
  readonly status: ApprovalDisplayStatus;
  /** Manual gestures only, `0..MAX_APPROVAL_RESOLVE_ATTEMPTS`. */
  readonly attempts: number;
  readonly lastDecision: 'approve' | 'deny' | null;
  /** Exact named refusal from the server or the synthetic ACK-settle refusal. */
  readonly refusal: Refusal | null;
  readonly lastAttemptAt: number | null;
}

export type ApprovalResolveOutcome =
  | { readonly ok: true }
  | { readonly ok: false; readonly kind: ApprovalFailureKind; readonly refusal: Refusal };

/** Pure replay-window state (timers live in the store; this is the data it carries). */
export interface ApprovalReplayState {
  readonly stale: boolean;
  readonly epoch: number;
  readonly window: boolean;
  readonly lastReplayAt: number | null;
  readonly source: ApprovalReplaySource | null;
}

/** Args accepted by {@link approvalActionsEnabled} / {@link canRetryApproval}. */
export interface ApprovalAvailabilityArgs {
  readonly record: ApprovalRecord;
  readonly resolve: ApprovalResolveState | undefined;
  readonly nowMs: number;
  readonly connection: ConnectionStatus;
}

/** Availability projection used by the card to enable/disable its three actions. */
export interface ApprovalActions {
  readonly approve: boolean;
  readonly deny: boolean;
  readonly retry: boolean;
  readonly dismiss: boolean;
}

/** ACK-settle transition hint (the store attaches the full Refusal copy). */
export interface AckSettleTransition {
  readonly status: 'retryable';
  readonly code: typeof APPROVAL_ACK_SETTLE_CODE;
}

/** Structural slice of the store that the approval selectors read. */
export interface ApprovalStateSlice {
  readonly approvals: readonly ApprovalRecord[];
  readonly approvalsStale: boolean;
  readonly approvalResolve: Readonly<Record<string, ApprovalResolveState>>;
}

const TRANSPORT_REFUSAL_CODES: ReadonlySet<string> = new Set([
  'E_WS_DISCONNECTED',
  'E_RESYNC_REQUIRED',
  'E_DB_BUSY',
  'E_INTERNAL_ERROR',
]);

const OPERATOR_REFUSAL_CODES: ReadonlySet<string> = new Set([
  'E_APPROVAL_DECISION_INVALID',
  'E_SCHEMA_INVALID',
]);

const STALE_REFUSAL_CODES: ReadonlySet<string> = new Set([
  'E_APPROVAL_REPLAY',
  'E_APPROVAL_NOT_FOUND',
]);

const EXPIRED_REFUSAL_CODES: ReadonlySet<string> = new Set([
  'E_APPROVAL_TIMEOUT',
  'E_APPROVAL_REQUIRED',
]);

const TERMINAL_STATUSES: ReadonlySet<ApprovalDisplayStatus> = new Set<ApprovalDisplayStatus>([
  'expired',
  'stale',
  'unconfirmed',
]);

/**
 * Classifies a named refusal into the retryability/display matrix
 * (`CODE-APPROVAL-UX.md:278-288`).
 *
 * A client command timeout is constructed by the client as
 * `E_WS_DISCONNECTED`, so the transport row covers it.
 */
export function classifyApprovalRefusal(refusal: Refusal): {
  readonly kind: ApprovalFailureKind;
  readonly status: ApprovalDisplayStatus;
  readonly retryable: boolean;
} {
  const code = refusal.code;
  if (STALE_REFUSAL_CODES.has(code)) {
    return { kind: 'stale', status: 'stale', retryable: false };
  }
  if (EXPIRED_REFUSAL_CODES.has(code)) {
    return { kind: 'expired', status: 'expired', retryable: false };
  }
  if (TRANSPORT_REFUSAL_CODES.has(code)) {
    return { kind: 'transport', status: 'retryable', retryable: true };
  }
  if (OPERATOR_REFUSAL_CODES.has(code)) {
    return { kind: 'operator', status: 'retryable', retryable: true };
  }
  // Unknown/other named refusal: keep the code visible, bounded operator retry.
  return { kind: 'operator', status: 'retryable', retryable: true };
}

/** Neutral per-approval resolve state; `approvalResolve[id]` is absent until a gesture. */
export function initialApprovalResolveState(): ApprovalResolveState {
  return {
    status: 'pending',
    attempts: 0,
    lastDecision: null,
    refusal: null,
    lastAttemptAt: null,
  };
}

/** True only when a valid `expires_at` is at or before now (unknown is not expired). */
export function isApprovalExpired(
  expiresAt: string | null | undefined,
  nowMs: number,
): boolean {
  if (!expiresAt) {
    return false;
  }
  const expiryMs = Date.parse(expiresAt);
  if (!Number.isFinite(expiryMs)) {
    return false;
  }
  return expiryMs <= nowMs;
}

function hasFutureExpiry(
  expiresAt: string | null | undefined,
  nowMs: number,
): boolean {
  if (!expiresAt) {
    return false;
  }
  const expiryMs = Date.parse(expiresAt);
  return Number.isFinite(expiryMs) && expiryMs > nowMs;
}

/**
 * Display-status precedence: an active resolve status wins, then the client-clock
 * expiry, then `pending` (`CODE-APPROVAL-UX.md:304`).
 */
export function approvalDisplayStatus(
  record: ApprovalRecord,
  resolve: ApprovalResolveState | undefined,
  nowMs: number,
): ApprovalDisplayStatus {
  const resolveStatus = resolve?.status;
  if (resolveStatus === 'resolving' || resolveStatus === 'awaiting_server') {
    return resolveStatus;
  }
  if (resolveStatus === 'retryable' || resolveStatus === 'stale') {
    return resolveStatus;
  }
  if (resolveStatus === 'unconfirmed' || resolveStatus === 'expired') {
    return resolveStatus;
  }
  if (isApprovalExpired(record.expiresAt, nowMs)) {
    return 'expired';
  }
  return 'pending';
}

/**
 * Retry gate: only a retryable card, with a fresh nonce, an open connection, a
 * live (not client-expired) TTL and attempts below the manual cap
 * (`CODE-APPROVAL-UX.md:290-304`).
 */
export function canRetryApproval(args: ApprovalAvailabilityArgs): boolean {
  const status = approvalDisplayStatus(args.record, args.resolve, args.nowMs);
  return (
    status === 'retryable' &&
    args.record.nonce !== null &&
    args.connection === 'open' &&
    !isApprovalExpired(args.record.expiresAt, args.nowMs) &&
    (args.resolve?.attempts ?? 0) < MAX_APPROVAL_RESOLVE_ATTEMPTS
  );
}

/**
 * Truth-table projection of the card availability (`CODE-APPROVAL-UX.md:290-304`).
 * Everything unknown/terminal stays disabled; only `pending` enables approve/deny,
 * only `retryable` enables retry, only terminal read-only states enable dismiss.
 */
export function approvalActionsEnabled(args: ApprovalAvailabilityArgs): ApprovalActions {
  const status = approvalDisplayStatus(args.record, args.resolve, args.nowMs);
  const pendingActionsEnabled =
    status === 'pending' &&
    args.record.nonce !== null &&
    args.connection === 'open' &&
    !isApprovalExpired(args.record.expiresAt, args.nowMs);
  return {
    approve: pendingActionsEnabled,
    deny: pendingActionsEnabled,
    retry: canRetryApproval(args),
    dismiss: TERMINAL_STATUSES.has(status),
  };
}

/** Neutral replay-window state. */
export function initialApprovalReplayState(): ApprovalReplayState {
  return {
    stale: false,
    epoch: 0,
    window: false,
    lastReplayAt: null,
    source: null,
  };
}

/**
 * Reconnect/resync transition: records are kept but marked stale, the replay
 * epoch advances and any open window closes (`CODE-APPROVAL-UX.md:343`).
 */
export function markApprovalsStale(state: ApprovalReplayState): ApprovalReplayState {
  return {
    ...state,
    stale: true,
    epoch: state.epoch + 1,
    window: false,
  };
}

/**
 * SERVER_HELLO transition: open the bounded replay window. The store arms the
 * `APPROVAL_REPLAY_SETTLE_MS` timer after calling this.
 */
export function beginReplayWindow(
  state: ApprovalReplayState,
  source: ApprovalReplaySource,
  nowMs: number,
): ApprovalReplayState {
  return {
    ...state,
    window: true,
    lastReplayAt: nowMs,
    source,
  };
}

/** Settle timer transition: close the window and clear the stale marker. */
export function settleReplay(state: ApprovalReplayState): ApprovalReplayState {
  return {
    ...state,
    window: false,
    stale: false,
  };
}

/** True while frames are attributed to the just-replayed batch. */
export function isReplayWindowOpen(state: ApprovalReplayState): boolean {
  return state.window;
}

/** Stamps one approval as seen in the given replay epoch (immutable copy). */
export function stampReplayEpoch(
  seenEpoch: Readonly<Record<string, number>>,
  approvalId: string,
  epoch: number,
): Readonly<Record<string, number>> {
  return { ...seenEpoch, [approvalId]: epoch };
}

/**
 * Settle rule per record (`CODE-APPROVAL-UX.md:348`): a record seen in the
 * current epoch is confirmed (an `unconfirmed` marker is cleared); a record not
 * seen in this epoch becomes read-only `unconfirmed` only if it was still
 * actionable (`pending`/`retryable`). In-flight/terminal states are preserved.
 */
export function settleReplayStatus(
  status: ApprovalDisplayStatus,
  seenEpoch: number | undefined,
  currentEpoch: number,
): ApprovalDisplayStatus {
  if (seenEpoch === currentEpoch) {
    return status === 'unconfirmed' ? 'pending' : status;
  }
  if (status === 'pending' || status === 'retryable') {
    return 'unconfirmed';
  }
  return status;
}

/**
 * Replay/live upsert rule (`CODE-APPROVAL-UX.md:345-346`): in-flight states are
 * preserved; a terminal/read-only record is confirmed back to `pending` only by
 * a freshly expired-in-the-future request; every other arrival is `pending`.
 */
export function approvalStatusAfterRequest(
  previousStatus: ApprovalDisplayStatus | undefined,
  payload: ApprovalRequestPayload,
  nowMs: number,
): ApprovalDisplayStatus {
  if (previousStatus === 'resolving' || previousStatus === 'awaiting_server') {
    return previousStatus;
  }
  if (
    (previousStatus === 'unconfirmed' ||
      previousStatus === 'stale' ||
      previousStatus === 'expired') &&
    hasFutureExpiry(payload.expires_at, nowMs)
  ) {
    return 'pending';
  }
  if (previousStatus === 'unconfirmed' || previousStatus === 'stale') {
    return previousStatus;
  }
  if (previousStatus === 'expired') {
    return 'expired';
  }
  return 'pending';
}

/**
 * Bounded, deterministic replay dedupe: one payload per `approval_id`, the last
 * occurrence wins, first-seen order is preserved, and the batch is capped at
 * `MAX_APPROVAL_REPLAY_EVENTS` (mirrors the server bound).
 */
export function dedupeReplayApprovals(
  payloads: readonly ApprovalRequestPayload[],
  maxEvents: number = MAX_APPROVAL_REPLAY_EVENTS,
): readonly ApprovalRequestPayload[] {
  const limit = Number.isFinite(maxEvents) ? Math.max(0, Math.floor(maxEvents)) : 0;
  if (limit === 0) {
    return [];
  }
  const order: string[] = [];
  const byId = new Map<string, ApprovalRequestPayload>();
  for (const payload of payloads) {
    if (!byId.has(payload.approval_id)) {
      order.push(payload.approval_id);
    }
    byId.set(payload.approval_id, payload);
  }
  const deduped: ApprovalRequestPayload[] = [];
  for (const approvalId of order.slice(0, limit)) {
    const payload = byId.get(approvalId);
    if (payload !== undefined) {
      deduped.push(payload);
    }
  }
  return deduped;
}

/** Whether the ACK settle budget elapsed while the card waits for APPROVAL_RESOLVED. */
export function ackSettleDue(
  state: ApprovalResolveState | undefined,
  nowMs: number,
): boolean {
  if (state === undefined || state.status !== 'awaiting_server') {
    return false;
  }
  if (state.lastAttemptAt === null) {
    return false;
  }
  return nowMs - state.lastAttemptAt >= APPROVAL_ACK_SETTLE_MS;
}

/**
 * ACK-settle transition hint: when the card is still `awaiting_server`, the
 * store attaches `clientRefusal(APPROVAL_ACK_SETTLE_CODE, { message })` and sets
 * `retryable`; otherwise `null` (nothing to settle).
 */
export function ackSettleTransition(
  state: ApprovalResolveState | undefined,
): AckSettleTransition | null {
  if (state === undefined || state.status !== 'awaiting_server') {
    return null;
  }
  return { status: 'retryable', code: APPROVAL_ACK_SETTLE_CODE };
}

/* Selectors: read single fields only (zustand-friendly); structurally typed so
 * the module stays free of a runtime store import. `UiState` will satisfy
 * `ApprovalStateSlice` when the store hunk adds `approvalResolve`/`approvalsStale`. */

export const selectApprovals = (state: ApprovalStateSlice): readonly ApprovalRecord[] =>
  state.approvals;

export const selectApprovalsStale = (state: ApprovalStateSlice): boolean =>
  state.approvalsStale;

export const selectApprovalResolve =
  (approvalId: string) =>
  (state: ApprovalStateSlice): ApprovalResolveState | undefined =>
    state.approvalResolve[approvalId];

export const selectApprovalResolveMap = (
  state: ApprovalStateSlice,
): Readonly<Record<string, ApprovalResolveState>> => state.approvalResolve;
