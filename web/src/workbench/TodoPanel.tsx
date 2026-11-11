/**
 * CAP-4 workbench — plan / todo panel.
 *
 * The list is caller-owned: checking an item invokes `onToggle` and never flips
 * local state, so a refused or unpersisted write cannot look completed. Progress
 * is derived only from the supplied items. Approval-gated items expose the
 * approval state and cannot be completed until approved.
 */
import { useId } from 'react';
import type { ReactNode } from 'react';

export type TodoStatus = 'pending' | 'in_progress' | 'completed' | 'blocked' | 'cancelled';
export type TodoApprovalState = 'required' | 'pending' | 'approved' | 'denied';
export type TodoPanelStatus = 'idle' | 'loading' | 'ready' | 'error';

export interface TodoItem {
  readonly id: string;
  readonly text: string;
  readonly status: TodoStatus;
  readonly detail?: string | null;
  readonly acceptance?: string | null;
  readonly ownerAgentId?: string | null;
  readonly ownerLabel?: string | null;
  readonly approval?: TodoApprovalState | null;
  readonly approvalTool?: string | null;
}

export interface TodoPanelProps {
  readonly items: readonly TodoItem[];
  readonly title?: string;
  readonly status?: TodoPanelStatus;
  readonly errorMessage?: string | null;
  readonly readOnly?: boolean;
  readonly onToggle?: (id: string, next: 'completed' | 'pending') => void;
  readonly onRequestApproval?: (id: string) => void;
  readonly onRetry?: () => void;
  readonly className?: string;
}

export const TODO_COPY = {
  section: 'Plan & todos',
  loading: 'Loading plan…',
  error: 'Plan unavailable',
  retry: 'Retry',
  empty: 'No plan yet.',
  emptyHint: 'Plan steps appear here after the runtime records them.',
  done: 'done',
  blocked: 'blocked',
  readOnly: 'Read-only',
  approval: 'Approval',
  approvalRequired: 'Approval required',
  approvalPending: 'Awaiting approval',
  approvalApproved: 'Approved',
  approvalDenied: 'Approval denied',
  requestApproval: 'Request',
  acceptance: 'acceptance',
  owner: 'owner',
  pending: 'Pending',
  inProgress: 'In progress',
  completed: 'Completed',
  cancelled: 'Cancelled',
} as const;

const STATUS_LABELS: Readonly<Record<TodoStatus, string>> = {
  pending: TODO_COPY.pending,
  in_progress: TODO_COPY.inProgress,
  completed: TODO_COPY.completed,
  blocked: TODO_COPY.blocked,
  cancelled: TODO_COPY.cancelled,
};

const STATUS_TONES: Readonly<Record<TodoStatus, string>> = {
  pending: 'border-hairline bg-panel text-ink-500',
  in_progress: 'border-honey-500 bg-honey-100 text-ink-900',
  completed: 'border-verify bg-panel text-verify',
  blocked: 'border-amber bg-panel-alt text-ink-800',
  cancelled: 'border-hairline bg-recessed text-ink-500',
};

const APPROVAL_TONES: Readonly<Record<TodoApprovalState, string>> = {
  required: 'border-amber bg-panel-alt text-ink-800',
  pending: 'border-amber bg-panel-alt text-ink-800',
  approved: 'border-verify bg-panel text-verify',
  denied: 'border-signal bg-error-surface text-signal',
};

export function todoStatusLabel(status: TodoStatus | string): string {
  return Object.prototype.hasOwnProperty.call(STATUS_LABELS, status)
    ? STATUS_LABELS[status as TodoStatus]
    : 'Unknown';
}

export function todoStatusTone(status: TodoStatus | string): string {
  return Object.prototype.hasOwnProperty.call(STATUS_TONES, status)
    ? STATUS_TONES[status as TodoStatus]
    : STATUS_TONES.pending;
}

export function todoApprovalLabel(state: TodoApprovalState): string {
  switch (state) {
    case 'required':
      return TODO_COPY.approvalRequired;
    case 'pending':
      return TODO_COPY.approvalPending;
    case 'approved':
      return TODO_COPY.approvalApproved;
    case 'denied':
      return TODO_COPY.approvalDenied;
  }
}

export interface TodoSummary {
  readonly total: number;
  readonly completed: number;
  readonly blocked: number;
  readonly inProgress: number;
  /** Whole-number 0-100; 0 when there are no items. */
  readonly percent: number;
}

export function todoSummary(items: readonly TodoItem[]): TodoSummary {
  let completed = 0;
  let blocked = 0;
  let inProgress = 0;
  for (const item of items) {
    if (item.status === 'completed') {
      completed += 1;
    } else if (item.status === 'blocked') {
      blocked += 1;
    } else if (item.status === 'in_progress') {
      inProgress += 1;
    }
  }
  return {
    total: items.length,
    completed,
    blocked,
    inProgress,
    percent: items.length === 0 ? 0 : Math.round((completed / items.length) * 100),
  };
}

export function isTodoComplete(status: TodoStatus): boolean {
  return status === 'completed';
}

/** A cancelled item has no toggle target; every other status does. */
export function todoToggleTarget(status: TodoStatus): 'completed' | 'pending' | null {
  if (status === 'cancelled') {
    return null;
  }
  return status === 'completed' ? 'pending' : 'completed';
}

export function todoProgressLabel(summary: TodoSummary): string {
  return `${summary.completed}/${summary.total} ${TODO_COPY.done}`;
}

function ApprovalBadge({
  item,
  onRequestApproval,
}: {
  item: TodoItem;
  onRequestApproval?: (id: string) => void;
}): ReactNode {
  if (item.approval === null || item.approval === undefined) {
    return null;
  }
  return (
    <span className="inline-flex flex-wrap items-center gap-1">
      <span
        className={`type-mono-meta rounded-[var(--radius-pill)] border px-1.5 py-0.5 ${APPROVAL_TONES[item.approval]}`}
        data-todo-approval={item.approval}
        title={item.approvalTool ?? undefined}
      >
        {todoApprovalLabel(item.approval)}
      </span>
      {item.approval === 'required' || item.approval === 'denied' ? (
        onRequestApproval === undefined ? null : (
          <button
            type="button"
            className="type-mono-meta rounded-[var(--radius-sm)] border border-honey-500 bg-honey-300 px-1.5 py-0.5 text-ink-900 hover:bg-honey-400"
            onClick={() => onRequestApproval(item.id)}
          >
            {TODO_COPY.requestApproval}
          </button>
        )
      ) : null}
    </span>
  );
}

export function TodoPanel({
  items,
  title = TODO_COPY.section,
  status = 'ready',
  errorMessage = null,
  readOnly = false,
  onToggle,
  onRequestApproval,
  onRetry,
  className = '',
}: TodoPanelProps) {
  const summary = todoSummary(items);
  const interactive = status === 'ready' && !readOnly && onToggle !== undefined;
  const readOnlyView = status === 'ready' && !interactive;
  const baseId = useId().replace(/[^a-zA-Z0-9_-]/g, '');

  return (
    <section
      aria-label={title}
      aria-busy={status === 'loading'}
      className={`flex min-h-0 flex-col overflow-hidden rounded-[var(--radius-card)] border border-hairline bg-panel shadow-[var(--shadow-e1)] ${className}`}
    >
      <header className="flex flex-wrap items-center gap-2 border-b border-hairline bg-panel-alt px-3 py-2">
        <h2 className="type-label min-w-0 flex-1 text-ink-900">{title}</h2>
        <span className="type-mono-meta shrink-0 text-meta" aria-live="polite">
          {todoProgressLabel(summary)}
        </span>
        {summary.blocked > 0 ? (
          <span className="type-mono-meta shrink-0 rounded-[var(--radius-pill)] border border-amber bg-panel-alt px-1.5 py-0.5 text-ink-800">
            {summary.blocked} {TODO_COPY.blocked}
          </span>
        ) : null}
        {readOnlyView ? (
          <span className="type-mono-meta shrink-0 rounded-[var(--radius-pill)] border border-hairline bg-recessed px-1.5 py-0.5 text-ink-800">
            {TODO_COPY.readOnly}
          </span>
        ) : null}
      </header>

      {status === 'ready' && summary.total > 0 ? (
        <div className="border-b border-hairline bg-panel px-3 py-2">
          <div
            role="progressbar"
            aria-label={todoProgressLabel(summary)}
            aria-valuemin={0}
            aria-valuemax={100}
            aria-valuenow={summary.percent}
            aria-valuetext={todoProgressLabel(summary)}
            className="h-1.5 w-full overflow-hidden rounded-[var(--radius-pill)] bg-recessed"
          >
            <div
              className="h-full rounded-[var(--radius-pill)] bg-honey-500"
              style={{ width: `${summary.percent}%` }}
            />
          </div>
        </div>
      ) : null}

      {status === 'idle' ? (
        <div className="p-3">
          <p className="type-body-l text-ink-500">{TODO_COPY.empty}</p>
          <p className="type-mono-meta mt-1 text-meta">{TODO_COPY.emptyHint}</p>
        </div>
      ) : null}

      {status === 'loading' ? (
        <p role="status" className="type-mono-meta p-3 text-meta">
          {TODO_COPY.loading}
        </p>
      ) : null}

      {status === 'error' ? (
        <div role="alert" className="m-3 rounded-[var(--radius-md)] border border-signal bg-error-surface p-3">
          <p className="type-label text-signal">{TODO_COPY.error}</p>
          <p className="type-body-l mt-1 text-ink-900">
            {errorMessage ?? 'The plan could not be loaded.'}
          </p>
          {onRetry === undefined ? null : (
            <button
              type="button"
              className="type-label mt-2 rounded-[var(--radius-button)] border border-signal px-3 py-1.5 text-signal"
              onClick={() => onRetry()}
            >
              {TODO_COPY.retry}
            </button>
          )}
        </div>
      ) : null}

      {status === 'ready' && items.length === 0 ? (
        <div className="p-3">
          <p className="type-body-l text-ink-500">{TODO_COPY.empty}</p>
          <p className="type-mono-meta mt-1 text-meta">{TODO_COPY.emptyHint}</p>
        </div>
      ) : null}

      {status === 'ready' && items.length > 0 ? (
        <ul aria-label={title} className="m-0 min-h-0 flex-1 list-none space-y-1 overflow-y-auto p-2">
          {items.map((item, index) => {
            const toggleTarget = todoToggleTarget(item.status);
            const approvalBlocked =
              item.approval === 'required' ||
              item.approval === 'pending' ||
              item.approval === 'denied';
            const checkboxDisabled =
              !interactive || toggleTarget === null || item.status === 'cancelled' || approvalBlocked;
            const checkboxId = `${baseId}-todo-${index}`;
            return (
              <li
                key={item.id}
                className="rounded-[var(--radius-sm)] border border-hairline bg-panel px-2 py-1.5"
                data-todo-status={item.status}
              >
                <div className="flex items-start gap-2">
                  <input
                    id={checkboxId}
                    type="checkbox"
                    className="mt-0.5 h-3.5 w-3.5 shrink-0 accent-honey-500 disabled:cursor-not-allowed disabled:opacity-50"
                    checked={isTodoComplete(item.status)}
                    disabled={checkboxDisabled}
                    aria-label={`${item.text}, ${todoStatusLabel(item.status)}`}
                    onChange={() => {
                      if (toggleTarget === null) {
                        return;
                      }
                      onToggle?.(item.id, toggleTarget);
                    }}
                  />
                  <div className="min-w-0 flex-1">
                    <label
                      htmlFor={checkboxId}
                      className={`type-body-l block cursor-pointer break-words ${
                        item.status === 'completed'
                          ? 'text-ink-500 line-through'
                          : item.status === 'cancelled'
                            ? 'text-ink-500 line-through'
                            : 'text-ink-900'
                      }`}
                    >
                      {item.text}
                    </label>
                    <div className="mt-1 flex flex-wrap items-center gap-1.5">
                      <span
                        className={`type-mono-meta rounded-[var(--radius-pill)] border px-1.5 py-0.5 ${todoStatusTone(item.status)}`}
                      >
                        {todoStatusLabel(item.status)}
                      </span>
                      <ApprovalBadge item={item} onRequestApproval={onRequestApproval} />
                      {item.ownerLabel === undefined || item.ownerLabel === null ? null : (
                        <span className="type-mono-meta text-meta">
                          {TODO_COPY.owner}: {item.ownerLabel}
                        </span>
                      )}
                      {item.ownerLabel === undefined &&
                      item.ownerAgentId !== undefined &&
                      item.ownerAgentId !== null ? (
                        <span className="type-mono-meta text-meta" title={item.ownerAgentId}>
                          {TODO_COPY.owner}: {item.ownerAgentId}
                        </span>
                      ) : null}
                    </div>
                    {item.detail === undefined || item.detail === null || item.detail.length === 0 ? null : (
                      <p className="type-mono-meta mt-1 break-words text-ink-700">{item.detail}</p>
                    )}
                    {item.acceptance === undefined ||
                    item.acceptance === null ||
                    item.acceptance.length === 0 ? null : (
                      <p className="type-mono-meta mt-1 break-words text-meta">
                        {TODO_COPY.acceptance}: {item.acceptance}
                      </p>
                    )}
                  </div>
                </div>
              </li>
            );
          })}
        </ul>
      ) : null}
    </section>
  );
}
