/**
 * CAP-4 workbench — session rail.
 *
 * The rail is a navigation list, not a local session store: selection is a
 * caller callback and the current session is highlighted from
 * `currentSessionId`. Ordering is the host order with the current session
 * pinned first; no session row is synthesized. Server-reported cost is shown
 * only when it is a finite, non-negative number.
 */
import { useEffect, useMemo, useRef, useState } from 'react';
import type { KeyboardEvent } from 'react';

export type SessionStatus = 'active' | 'idle' | 'closed' | 'error';
export type SessionRailStatus = 'idle' | 'loading' | 'ready' | 'error';

export interface SessionSummary {
  readonly id: string;
  readonly label: string;
  readonly status?: SessionStatus;
  /** ISO-8601 last activity; absent stays unknown. */
  readonly updatedAt?: string | null;
  readonly runId?: string | null;
  readonly model?: string | null;
  readonly branch?: string | null;
  readonly costUsd?: number | null;
  /** True when this session has a pending approval that the operator must see. */
  readonly waitingApproval?: boolean;
}

export interface SessionRailProps {
  readonly sessions: readonly SessionSummary[];
  readonly currentSessionId?: string | null;
  readonly status?: SessionRailStatus;
  readonly errorMessage?: string | null;
  readonly onSelect?: (sessionId: string) => void;
  readonly onNew?: () => void;
  readonly onRetry?: () => void;
  readonly title?: string;
  readonly className?: string;
}

export const SESSION_COPY = {
  section: 'Sessions',
  newSession: 'New session',
  loading: 'Loading sessions…',
  idleState: 'Sessions have not loaded yet.',
  error: 'Sessions unavailable',
  retry: 'Retry',
  empty: 'No sessions yet.',
  emptyHint: 'Sessions appear here once the runtime records one.',
  current: 'Current',
  approval: 'Approval',
  active: 'Active',
  idle: 'Idle',
  closed: 'Closed',
  failed: 'Error',
  unknown: 'Unknown',
  updated: 'updated',
  model: 'model',
  branch: 'branch',
  run: 'run',
} as const;

export function sessionStatusLabel(status: SessionStatus | undefined): string {
  switch (status) {
    case 'active':
      return SESSION_COPY.active;
    case 'idle':
      return SESSION_COPY.idle;
    case 'closed':
      return SESSION_COPY.closed;
    case 'error':
      return SESSION_COPY.failed;
    default:
      return SESSION_COPY.unknown;
  }
}

export function sessionStatusTone(status: SessionStatus | undefined): string {
  switch (status) {
    case 'active':
      return 'border-honey-500 bg-honey-100 text-ink-900';
    case 'error':
      return 'border-signal bg-error-surface text-signal';
    case 'closed':
      return 'border-hairline bg-recessed text-ink-500';
    case 'idle':
    default:
      return 'border-hairline bg-panel text-ink-500';
  }
}

/** UTC YYYY-MM-DD HH:MM (deterministic across hosts); invalid stays unknown. */
export function formatSessionUpdated(value: string | null | undefined): string {
  if (value === null || value === undefined || value.length === 0) {
    return '—';
  }
  const parsed = Date.parse(value);
  if (!Number.isFinite(parsed)) {
    return '—';
  }
  return new Date(parsed).toISOString().slice(0, 16).replace('T', ' ');
}

export function formatSessionCost(costUsd: number | null | undefined): string {
  if (typeof costUsd !== 'number' || !Number.isFinite(costUsd) || costUsd < 0) {
    return '—';
  }
  const digits = costUsd > 0 && costUsd < 0.01 ? 4 : 2;
  return `$${costUsd.toFixed(digits)}`;
}

/** The current session is pinned first; every other row keeps host order. */
export function orderSessions(
  sessions: readonly SessionSummary[],
  currentSessionId: string | null | undefined,
): readonly SessionSummary[] {
  if (currentSessionId === null || currentSessionId === undefined) {
    return [...sessions];
  }
  const current = sessions.find((session) => session.id === currentSessionId);
  if (current === undefined) {
    return [...sessions];
  }
  return [current, ...sessions.filter((session) => session.id !== currentSessionId)];
}

export function truncateSessionId(value: string | null | undefined, head = 10): string {
  if (value === null || value === undefined || value.length === 0) {
    return '—';
  }
  return value.length <= head ? value : `${value.slice(0, head)}…`;
}

export function sessionNavigationTarget(index: number, count: number, key: string): number | null {
  if (count <= 0) {
    return null;
  }
  const current = Math.min(Math.max(index, 0), count - 1);
  switch (key) {
    case 'ArrowDown':
    case 'ArrowRight':
      return (current + 1) % count;
    case 'ArrowUp':
    case 'ArrowLeft':
      return (current - 1 + count) % count;
    case 'Home':
      return 0;
    case 'End':
      return count - 1;
    default:
      return null;
  }
}

export function sessionAriaLabel(session: SessionSummary): string {
  const status = sessionStatusLabel(session.status);
  return `${session.label}, ${status}${session.waitingApproval === true ? ', approval pending' : ''}`;
}

export function SessionRail({
  sessions,
  currentSessionId = null,
  status = 'ready',
  errorMessage = null,
  onSelect,
  onNew,
  onRetry,
  title = SESSION_COPY.section,
  className = '',
}: SessionRailProps) {
  const ordered = useMemo(
    () => orderSessions(sessions, currentSessionId),
    [sessions, currentSessionId],
  );
  const currentIndex = ordered.findIndex((session) => session.id === currentSessionId);
  const [activeIndex, setActiveIndex] = useState(currentIndex >= 0 ? currentIndex : 0);
  const [focusRequest, setFocusRequest] = useState<number | null>(null);
  const buttonRefs = useRef<Map<number, HTMLButtonElement>>(new Map());
  const safeIndex = ordered.length === 0 ? 0 : Math.min(Math.max(activeIndex, 0), ordered.length - 1);

  useEffect(() => {
    setActiveIndex(currentIndex >= 0 ? currentIndex : 0);
  }, [currentIndex, currentSessionId, ordered.length]);

  useEffect(() => {
    if (focusRequest === null || typeof document === 'undefined') {
      return;
    }
    buttonRefs.current.get(focusRequest)?.focus();
    setFocusRequest(null);
  }, [focusRequest]);

  const onRailKeyDown = (event: KeyboardEvent<HTMLUListElement>): void => {
    const target = sessionNavigationTarget(safeIndex, ordered.length, event.key);
    if (target === null) {
      return;
    }
    event.preventDefault();
    setActiveIndex(target);
    setFocusRequest(target);
  };

  return (
    <nav
      aria-label={title}
      aria-busy={status === 'loading'}
      className={`flex min-h-0 w-full flex-col overflow-hidden rounded-[var(--radius-card)] border border-hairline bg-panel shadow-[var(--shadow-e1)] lg:w-[248px] ${className}`}
    >
      <header className="flex flex-wrap items-center gap-2 border-b border-hairline bg-panel-alt px-3 py-2">
        <h2 className="type-label min-w-0 flex-1 text-ink-900">{title}</h2>
        <span className="type-mono-meta shrink-0 text-meta">{ordered.length}</span>
        {onNew === undefined ? null : (
          <button
            type="button"
            className="type-label rounded-[var(--radius-button)] border border-honey-600 bg-honey-300 px-2 py-1 text-ink-900 hover:bg-honey-400"
            onClick={() => onNew()}
          >
            {SESSION_COPY.newSession}
          </button>
        )}
      </header>

      {status === 'loading' ? (
        <p role="status" className="type-mono-meta p-3 text-meta">
          {SESSION_COPY.loading}
        </p>
      ) : null}

      {status === 'idle' ? (
        <p role="status" className="type-mono-meta p-3 text-meta">
          {SESSION_COPY.idleState}
        </p>
      ) : null}

      {status === 'error' ? (
        <div role="alert" className="m-3 rounded-[var(--radius-md)] border border-signal bg-error-surface p-3">
          <p className="type-label text-signal">{SESSION_COPY.error}</p>
          <p className="type-body-l mt-1 text-ink-900">
            {errorMessage ?? 'Sessions could not be loaded.'}
          </p>
          {onRetry === undefined ? null : (
            <button
              type="button"
              className="type-label mt-2 rounded-[var(--radius-button)] border border-signal px-3 py-1.5 text-signal"
              onClick={() => onRetry()}
            >
              {SESSION_COPY.retry}
            </button>
          )}
        </div>
      ) : null}

      {status === 'ready' && ordered.length === 0 ? (
        <div className="p-3">
          <p className="type-body-l text-ink-500">{SESSION_COPY.empty}</p>
          <p className="type-mono-meta mt-1 text-meta">{SESSION_COPY.emptyHint}</p>
        </div>
      ) : null}

      {status === 'ready' && ordered.length > 0 ? (
        <ul
          aria-label={title}
          className="m-0 flex min-h-0 list-none gap-2 overflow-x-auto p-2 lg:flex-col lg:overflow-x-hidden lg:overflow-y-auto"
          onKeyDown={onRailKeyDown}
        >
          {ordered.map((session, index) => {
            const isCurrent = session.id === currentSessionId;
            const isActive = index === safeIndex;
            return (
              <li key={session.id} className="m-0 min-w-[190px] lg:min-w-0">
                <button
                  ref={(element) => {
                    if (element === null) {
                      buttonRefs.current.delete(index);
                    } else {
                      buttonRefs.current.set(index, element);
                    }
                  }}
                  type="button"
                  tabIndex={isActive ? 0 : -1}
                  disabled={onSelect === undefined}
                  aria-disabled={onSelect === undefined}
                  aria-current={isCurrent ? 'true' : undefined}
                  aria-label={sessionAriaLabel(session)}
                  data-session-id={session.id}
                  className={`w-full rounded-[var(--radius-sm)] border px-2 py-1.5 text-left ${
                    isCurrent
                      ? 'border-honey-400 bg-honey-100'
                      : 'border-hairline bg-panel hover:bg-panel-alt'
                  } disabled:cursor-not-allowed disabled:opacity-70`}
                  onClick={() => {
                    setActiveIndex(index);
                    onSelect?.(session.id);
                  }}
                >
                  <div className="flex items-baseline gap-2">
                    <span className="type-body-l min-w-0 flex-1 truncate text-ink-900" title={session.label}>
                      {session.label}
                    </span>
                    <span
                      className={`type-mono-meta shrink-0 rounded-[var(--radius-pill)] border px-1.5 py-0.5 ${sessionStatusTone(session.status)}`}
                    >
                      {sessionStatusLabel(session.status)}
                    </span>
                  </div>
                  <div className="type-mono-meta mt-0.5 flex flex-wrap gap-x-2 gap-y-0.5 text-meta">
                    <span>
                      {SESSION_COPY.updated} {formatSessionUpdated(session.updatedAt)}
                    </span>
                    {session.model === undefined || session.model === null ? null : (
                      <span title={session.model}>
                        {SESSION_COPY.model} {session.model}
                      </span>
                    )}
                    {session.branch === undefined || session.branch === null ? null : (
                      <span title={session.branch}>
                        {SESSION_COPY.branch} {session.branch}
                      </span>
                    )}
                    {session.runId === undefined || session.runId === null ? null : (
                      <span title={session.runId}>
                        {SESSION_COPY.run} {truncateSessionId(session.runId)}
                      </span>
                    )}
                    {session.costUsd === undefined || session.costUsd === null ? null : (
                      <span>{formatSessionCost(session.costUsd)}</span>
                    )}
                  </div>
                  {session.waitingApproval === true ? (
                    <span className="type-mono-meta mt-1 inline-block rounded-[var(--radius-pill)] border border-amber bg-panel-alt px-1.5 py-0.5 text-ink-800">
                      {SESSION_COPY.approval}
                    </span>
                  ) : null}
                  {isCurrent ? (
                    <span className="type-mono-meta mt-1 block text-meta">{SESSION_COPY.current}</span>
                  ) : null}
                </button>
              </li>
            );
          })}
        </ul>
      ) : null}
    </nav>
  );
}
