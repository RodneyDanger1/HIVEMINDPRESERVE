/**
 * Approval overlay (W12 / Wave 3): APPROVAL_RESOLVE with server nonce.
 *
 * Evidence shown: tool, args_sha256 (truncated, full in title), run/task/tool_call ids,
 * expiry countdown. Approve/deny are disabled while a resolve is pending **until the
 * server's APPROVAL_RESOLVED removes the approval** (an ACK is not a resolution), and
 * when the TTL has expired or the nonce is missing (fail-closed, no optimistic data).
 *
 * Store note: `ApprovalRecord` is owned by the store seat; this panel reads the optional
 * W12 fields through a widened view type. `resolvingApprovals` is the store-side in-flight
 * guard, but the store clears it when the send promise settles on ACK, so the panel keeps
 * a pending lock pruned only when the approval disappears (i.e. on APPROVAL_RESOLVED).
 * Missing fields render the honest unknown placeholder, never invented values.
 */
import { useEffect, useRef, useState } from 'react';
import { useUiStore } from '../state/context';
import { STRINGS } from '../state/strings';
import { approvalActionsEnabled } from '../state/approval';
import { ApprovalPreview } from './ApprovalPreview';

export const APPROVAL_DECISIONS = { approve: 'approve', deny: 'deny' } as const;
export type ApprovalDecision = (typeof APPROVAL_DECISIONS)[keyof typeof APPROVAL_DECISIONS];

/** Widened view of the store's ApprovalRecord with the optional W12 approval fields. */
export interface ApprovalView {
  readonly approvalId: string;
  readonly tool: string;
  readonly nonce: string | null;
  readonly argsSha256?: string | null;
  readonly argsPreview?: string | null;
  readonly expiresAt?: string | null;
  readonly toolCallId?: string | null;
  readonly runId?: string | null;
  readonly taskId?: string | null;
  readonly requestedAt?: number;
}

export const UNKNOWN_VALUE = STRINGS.approval.unknownValue;

/** Truncates an id/hash for display; null/empty becomes the honest unknown placeholder. */
export function truncateId(value: string | null | undefined, head = 12): string {
  if (value === null || value === undefined || value.length === 0) {
    return UNKNOWN_VALUE;
  }
  return value.length <= head ? value : `${value.slice(0, head)}…`;
}

/** Display form for args_sha256; full value belongs in the element's title attribute. */
export function formatArgsSha256(value: string | null | undefined, head = 12): string {
  return truncateId(value, head);
}

/** Word labels for the two honest expiry sentinels (never a fabricated time). */
export interface ExpiryLabels {
  readonly unknown: string;
  readonly expired: string;
}

const EXPIRY_LABELS: ExpiryLabels = {
  unknown: STRINGS.approval.expiry.unknown,
  expired: STRINGS.approval.expiry.expired,
};

/**
 * Deterministic expiry text for a countdown. Never invents a time:
 * missing/unparseable -> labels.unknown; past -> labels.expired; otherwise s/m/h.
 * Labels resolve from `STRINGS.approval.expiry`.
 */
export function formatExpiry(
  expiresAt: string | null | undefined,
  nowMs: number,
  labels: ExpiryLabels = EXPIRY_LABELS,
): string {
  if (!expiresAt) {
    return labels.unknown;
  }
  const expiryMs = Date.parse(expiresAt);
  if (!Number.isFinite(expiryMs)) {
    return labels.unknown;
  }
  const remainingMs = expiryMs - nowMs;
  if (remainingMs <= 0) {
    return labels.expired;
  }
  const totalSeconds = Math.ceil(remainingMs / 1000);
  if (totalSeconds < 60) {
    return `${totalSeconds}s`;
  }
  const minutes = Math.floor(totalSeconds / 60);
  const seconds = totalSeconds % 60;
  if (minutes < 60) {
    return `${minutes}m ${String(seconds).padStart(2, '0')}s`;
  }
  const hours = Math.floor(minutes / 60);
  return `${hours}h ${String(minutes % 60).padStart(2, '0')}m`;
}

/**
 * True only when a valid expires_at is at or before now. Missing/invalid TTL is NOT
 * expired (unknown is not a fake clock); the display stays unknown.
 */
export function isApprovalExpired(expiresAt: string | null | undefined, nowMs: number): boolean {
  if (!expiresAt) {
    return false;
  }
  const expiryMs = Date.parse(expiresAt);
  if (!Number.isFinite(expiryMs)) {
    return false;
  }
  return expiryMs <= nowMs;
}

/**
 * Fail-closed resolution gate: fresh nonce, no in-flight/pending resolve, not expired.
 * `expired` defaults to false so older call sites remain valid.
 */
export function canResolveApproval(record: {
  nonce: string | null;
  resolving: boolean;
  expired?: boolean;
}): boolean {
  return record.nonce !== null && !record.resolving && record.expired !== true;
}

/**
 * Prunes the panel-side pending lock. A pending id survives only while its approval is
 * still present; APPROVAL_RESOLVED removes the approval, the only event that may
 * re-enable resolution (an ACK does not remove the approval).
 */
export function prunePendingResolutions(
  pending: Readonly<Record<string, boolean>>,
  approvalIds: readonly string[],
): Readonly<Record<string, boolean>> {
  const live = new Set(approvalIds);
  let changed = false;
  const next: Record<string, boolean> = {};
  for (const [id, value] of Object.entries(pending)) {
    if (live.has(id)) {
      next[id] = value;
    } else {
      changed = true;
    }
  }
  return changed ? next : pending;
}

/**
 * A network approval without a replayed target is not safe to approve: the
 * server persists only the args digest, so a replay can omit args_preview.
 * Non-network tools are unaffected by this rule.
 */
export function networkApprovalTargetKnown(
  tool: string,
  argsPreview: string | null | undefined,
): boolean {
  if (tool !== 'web_fetch') {
    return true;
  }
  return typeof argsPreview === 'string' && argsPreview.trim().length > 0;
}

/** Accessible name for the dialog; includes the tool so screen readers get context. */
export function approvalAriaLabel(tool: string): string {
  return `${STRINGS.approval.title}: ${tool}`;
}

/** Accessible names for the two decisions (both include the tool under action). */
export function approvalActionAriaLabel(action: ApprovalDecision, tool: string): string {
  const label = action === APPROVAL_DECISIONS.approve ? STRINGS.controls.approve : STRINGS.controls.deny;
  return `${label} tool call for ${tool}`;
}

/** Clamp the queue cursor after approvals are resolved/removed (never out of range). */
export function clampApprovalIndex(index: number, total: number): number {
  if (!Number.isInteger(index) || index < 0 || total <= 0) {
    return 0;
  }
  return Math.min(index, total - 1);
}

/** "Approval 2 of 3" — the queue position of the single visible dialog. */
export function approvalQueueLabel(index: number, total: number): string {
  const safeTotal = Math.max(0, Math.trunc(total));
  const safeIndex = clampApprovalIndex(index, safeTotal);
  return `${STRINGS.approval.queue.label} ${safeIndex + 1} ${STRINGS.approval.queue.of} ${safeTotal}`;
}

function asApprovalView(record: {
  approvalId: string;
  tool: string;
  nonce: string | null;
}): ApprovalView {
  return record as unknown as ApprovalView;
}

export function ApprovalOverlay() {
  const approvals = useUiStore((state) => state.approvals);
  const approvalsStale = useUiStore((state) => state.approvalsStale);
  const resolvingApprovals = useUiStore((state) => state.resolvingApprovals);
  const approveResolveStates = useUiStore((state) => state.approvalResolve);
  const connection = useUiStore((state) => state.connection);
  const chatVisible = useUiStore((state) => state.chat.visible);
  const resolveApproval = useUiStore((state) => state.resolveApproval);
  const retryApproval = useUiStore((state) => state.retryApproval);
  const dismissApproval = useUiStore((state) => state.dismissApproval);
  const [now, setNow] = useState<number>(() => Date.now());
  const [activeIndex, setActiveIndex] = useState(0);
  const dialogRef = useRef<HTMLElement | null>(null);
  const previousFocusRef = useRef<Element | null>(null);

  const activeIndexOf = clampApprovalIndex(activeIndex, approvals.length);
  const activeApproval = approvals[activeIndexOf] ?? null;

  const hasExpiry = approvals.some((approval) => Boolean(asApprovalView(approval).expiresAt));
  useEffect(() => {
    if (!hasExpiry) {
      return undefined;
    }
    const timer = setInterval(() => {
      setNow(Date.now());
    }, 1_000);
    return () => clearInterval(timer);
  }, [hasExpiry]);

  // Keep the cursor legal after a resolve removes the active record.
  useEffect(() => {
    setActiveIndex((current) => clampApprovalIndex(current, approvals.length));
  }, [approvals.length]);

  const activeApprovalId = activeApproval?.approvalId ?? null;
  useEffect(() => {
    if (activeApprovalId === null || typeof document === 'undefined') {
      return undefined;
    }
    previousFocusRef.current = document.activeElement;
    const first = dialogRef.current?.querySelector<HTMLButtonElement>('button:not([disabled])');
    first?.focus();
    return () => {
      if (previousFocusRef.current instanceof HTMLElement) {
        previousFocusRef.current.focus();
      }
    };
  }, [activeApprovalId]);

  if (approvals.length === 0) {
    return null;
  }
  // When the chat is actually mounted, its inline check/deny card owns the
  // decision. The modal remains the fallback when the chat is hidden and in
  // isolated render tests, so no approval becomes unreachable.
  const inlineApprovalHost =
    typeof document !== 'undefined' && document.getElementById('composer') !== null;
  if (chatVisible && inlineApprovalHost) {
    return null;
  }

  const submit = async (view: ApprovalView, decision: ApprovalDecision): Promise<void> => {
    const resolveState = approveResolveStates[view.approvalId];
    const actions = approvalActionsEnabled({
      record: view as unknown as Parameters<typeof approvalActionsEnabled>[0]['record'],
      resolve: resolveState,
      nowMs: Date.now(),
      connection,
    });
    const enabled = decision === APPROVAL_DECISIONS.approve ? actions.approve : actions.deny;
    if (!enabled) {
      return;
    }
    if (
      decision === APPROVAL_DECISIONS.approve &&
      !networkApprovalTargetKnown(view.tool, view.argsPreview)
    ) {
      return;
    }
    await resolveApproval(
      view.approvalId,
      decision,
      decision === APPROVAL_DECISIONS.deny ? 'operator denied' : undefined,
    );
  };

  return (
    <>
      {approvalsStale ? (
        <div
          role="status"
          data-testid="approval-stale-banner"
          className="fixed inset-x-0 top-0 z-50 border-b border-amber bg-panel-alt px-4 py-2 text-center type-label text-ink-800"
        >
          {STRINGS.approval.staleBanner}
        </div>
      ) : null}
      {(activeApproval === null ? [] : [activeApproval]).map((record) => {
        const view = asApprovalView(record);
        const resolveState = approveResolveStates[view.approvalId];
        const actions = approvalActionsEnabled({
          record: view as unknown as Parameters<typeof approvalActionsEnabled>[0]['record'],
          resolve: resolveState,
          nowMs: now,
          connection,
        });
        const inFlight =
          resolvingApprovals.includes(view.approvalId) ||
          resolveState?.status === 'resolving' ||
          resolveState?.status === 'awaiting_server';
        const remaining = formatExpiry(view.expiresAt, now);
        const argsHash = formatArgsSha256(view.argsSha256);
        // Honest TTL display: unknown stays unknown, past stays expired; never a fake time.
        const expiryText =
          remaining === STRINGS.approval.expiry.unknown
            ? STRINGS.approval.expiry.unknown
            : remaining === STRINGS.approval.expiry.expired
              ? STRINGS.approval.expiry.expired
              : `${STRINGS.approval.expiry.expiresIn} ${remaining}`;
        // Stable key: navigation updates the same dialog in place, so the
        // focus-restore target survives Previous/Next (closing the queue
        // returns focus to the operator's original control).
        return (
          <div key="approval-dialog" className="fixed inset-0 z-40 flex items-center justify-center bg-ink-900/20 p-4">
            <section
              ref={dialogRef}
              role="dialog"
              aria-modal="true"
              aria-label={approvalAriaLabel(view.tool)}
              className="surface-panel w-[min(560px,94%)] rounded-[var(--radius-card)] p-4"
              style={{ boxShadow: 'var(--shadow-e3)' }}
              tabIndex={-1}
              onKeyDown={(event) => {
                if (event.key === 'Escape' && actions.dismiss) {
                  event.preventDefault();
                  dismissApproval(view.approvalId);
                  return;
                }
                if (event.key !== 'Tab') {
                  return;
                }
                const focusable = Array.from(
                  event.currentTarget.querySelectorAll<HTMLElement>(
                    'button:not([disabled]), [href], input:not([disabled]), select:not([disabled]), textarea:not([disabled]), [tabindex]:not([tabindex="-1"])',
                  ),
                );
                const first = focusable[0];
                const last = focusable[focusable.length - 1];
                if (first === undefined || last === undefined) {
                  event.preventDefault();
                  return;
                }
                if (event.shiftKey && document.activeElement === first) {
                  event.preventDefault();
                  last.focus();
                } else if (!event.shiftKey && document.activeElement === last) {
                  event.preventDefault();
                  first.focus();
                }
              }}
            >
              <h2 className="type-heading-h3 text-ink-900">{STRINGS.approval.title}</h2>
              {approvals.length > 1 ? (
                <p className="type-mono-meta mt-1 text-meta">
                  {approvals.length - 1} more approval{approvals.length - 1 === 1 ? '' : 's'} pending
                </p>
              ) : null}
              {approvals.length > 1 ? (
                <div
                  data-testid="approval-queue"
                  className="mt-2 flex items-center justify-between gap-2 rounded-[var(--radius-sm)] border border-hairline bg-panel-alt px-2 py-1"
                >
                  <button
                    type="button"
                    aria-label={STRINGS.approval.queue.previous}
                    disabled={activeIndexOf === 0}
                    onClick={() => setActiveIndex(clampApprovalIndex(activeIndexOf - 1, approvals.length))}
                    className="type-label rounded-[var(--radius-sm)] border border-hairline px-2 py-0.5 text-ink-800 hover:bg-honey-100 disabled:cursor-not-allowed disabled:opacity-50"
                  >
                    {STRINGS.approval.queue.previousShort}
                  </button>
                  <span className="type-mono-meta text-meta" aria-live="polite">
                    {approvalQueueLabel(activeIndexOf, approvals.length)}
                  </span>
                  <button
                    type="button"
                    aria-label={STRINGS.approval.queue.next}
                    disabled={activeIndexOf === approvals.length - 1}
                    onClick={() => setActiveIndex(clampApprovalIndex(activeIndexOf + 1, approvals.length))}
                    className="type-label rounded-[var(--radius-sm)] border border-hairline px-2 py-0.5 text-ink-800 hover:bg-honey-100 disabled:cursor-not-allowed disabled:opacity-50"
                  >
                    {STRINGS.approval.queue.nextShort}
                  </button>
                </div>
              ) : null}
              <dl className="mt-2 grid grid-cols-[120px_1fr] gap-y-1">
                <dt className="type-mono-meta text-meta">{STRINGS.approval.toolLabel}</dt>
                <dd className="type-mono-code text-ink-900">{view.tool}</dd>

                <dt className="type-mono-meta text-meta">{STRINGS.approval.fields.argsSha256}</dt>
                <dd className="type-mono-code text-ink-900" title={view.argsSha256 ?? UNKNOWN_VALUE}>
                  {argsHash}
                </dd>

                <dt className="type-mono-meta text-meta">{STRINGS.approval.fields.argsPreview}</dt>
                <dd className="min-w-0">
                  {/*
                   * Structured, display-only review. A missing preview stays the
                   * honest "preview unavailable" state; this component never
                   * touches the approve/deny policy gating below.
                   */}
                  <ApprovalPreview tool={view.tool} argsPreview={view.argsPreview} />
                </dd>

                <dt className="type-mono-meta text-meta">{STRINGS.approval.fields.runId}</dt>
                <dd className="type-mono-code text-ink-900" title={view.runId ?? UNKNOWN_VALUE}>
                  {truncateId(view.runId)}
                </dd>

                <dt className="type-mono-meta text-meta">{STRINGS.approval.fields.taskId}</dt>
                <dd className="type-mono-code text-ink-900" title={view.taskId ?? UNKNOWN_VALUE}>
                  {truncateId(view.taskId)}
                </dd>

                <dt className="type-mono-meta text-meta">{STRINGS.approval.fields.toolCallId}</dt>
                <dd className="type-mono-code text-ink-900" title={view.toolCallId ?? UNKNOWN_VALUE}>
                  {truncateId(view.toolCallId)}
                </dd>

                <dt className="type-mono-meta text-meta">{STRINGS.approval.fields.approvalId}</dt>
                <dd className="type-mono-code text-ink-900" title={view.approvalId}>
                  {truncateId(view.approvalId)}
                </dd>

                <dt className="type-mono-meta text-meta">{STRINGS.approval.fields.expires}</dt>
                <dd className="type-mono-code text-ink-900" aria-live="polite">
                  {expiryText}
                </dd>
              </dl>

              {view.nonce === null ? (
                <p className="type-mono-meta mt-2 rounded-[var(--radius-sm)] border border-amber bg-panel-alt p-2 text-ink-800">
                  {STRINGS.approval.nonceMissing}
                </p>
              ) : (
                <p className="type-mono-meta mt-2 text-meta">
                  {STRINGS.approval.title}: {STRINGS.approval.resolveHint} · {view.tool}
                </p>
              )}

              {inFlight ? (
                <p role="status" aria-live="polite" className="type-mono-meta mt-2 text-meta">
                  {STRINGS.approval.resolving}
                </p>
              ) : null}

              {resolveState?.refusal ? (
                <div className="mt-2 rounded-[var(--radius-sm)] border border-amber bg-panel-alt p-2">
                  <p className="type-mono-meta text-ink-800">
                    {STRINGS.approval.resolveFailed}: {resolveState.refusal.code} —{' '}
                    {resolveState.refusal.message}
                  </p>
                  <div className="mt-2 flex justify-end gap-2">
                    {actions.retry ? (
                      <button
                        type="button"
                        className="type-label rounded-[var(--radius-button)] border border-signal px-3 py-1.5 text-signal"
                        onClick={() => {
                          void retryApproval(view.approvalId);
                        }}
                      >
                        {STRINGS.approval.retry}
                      </button>
                    ) : null}
                    {actions.dismiss ? (
                      <button
                        type="button"
                        className="type-label rounded-[var(--radius-button)] border border-hairline px-3 py-1.5 text-ink-700"
                        onClick={() => dismissApproval(view.approvalId)}
                      >
                        {STRINGS.approval.dismiss}
                      </button>
                    ) : null}
                  </div>
                </div>
              ) : null}

              {resolveState?.refusal ? null : actions.dismiss ? (
                <div className="mt-2 flex justify-end">
                  <button
                    type="button"
                    className="type-label rounded-[var(--radius-button)] border border-hairline px-3 py-1.5 text-ink-700"
                    onClick={() => dismissApproval(view.approvalId)}
                  >
                    {STRINGS.approval.dismiss}
                  </button>
                </div>
              ) : null}

              {!networkApprovalTargetKnown(view.tool, view.argsPreview) ? (
                <p role="status" className="mt-2 type-label text-signal">
                  {STRINGS.approval.networkTargetMissing}
                </p>
              ) : null}

              <div className="mt-3 flex justify-end gap-2">
                <button
                  type="button"
                  className="type-label rounded-[var(--radius-button)] border border-signal px-3 py-1.5 text-signal disabled:opacity-50"
                  disabled={!actions.deny}
                  aria-label={approvalActionAriaLabel(APPROVAL_DECISIONS.deny, view.tool)}
                  onClick={() => {
                    void submit(view, APPROVAL_DECISIONS.deny);
                  }}
                >
                  {STRINGS.controls.deny}
                </button>
                <button
                  type="button"
                  className="type-label rounded-[var(--radius-button)] bg-honey-300 px-3 py-1.5 text-ink-900 disabled:opacity-50"
                  disabled={!actions.approve || !networkApprovalTargetKnown(view.tool, view.argsPreview)}
                  aria-label={approvalActionAriaLabel(APPROVAL_DECISIONS.approve, view.tool)}
                  onClick={() => {
                    void submit(view, APPROVAL_DECISIONS.approve);
                  }}
                >
                  {STRINGS.controls.approve}
                </button>
              </div>
            </section>
          </div>
        );
      })}
    </>
  );
}
