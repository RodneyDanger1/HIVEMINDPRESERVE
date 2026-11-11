/**
 * CAP-4 workbench — per-agent run timeline.
 *
 * Steps are rendered exactly as supplied and attributed to the agent named on
 * the step (`agentId`). Agents can be filtered with a keyboard-operable ARIA
 * tablist; an agent that appears in a step but not in `agents` is surfaced with
 * its id as the label rather than being silently dropped. Approval steps are a
 * first-class status, not a colour-only hint.
 */
import { useEffect, useId, useMemo, useRef, useState } from 'react';
import type { KeyboardEvent } from 'react';

export type TimelineStatus = 'idle' | 'loading' | 'ready' | 'error';
export type TimelineStepStatus = 'pending' | 'running' | 'ok' | 'failed' | 'skipped' | 'awaiting_approval';

export interface TimelineAgent {
  readonly id: string;
  readonly label: string;
  readonly kind?: string | null;
}

export interface TimelineStep {
  readonly id: string;
  /** ISO-8601 event time; unparseable values sort last and render as '—'. */
  readonly at: string;
  readonly agentId: string;
  /** Label used when the agent is not present in the `agents` list. */
  readonly agentLabel?: string | null;
  readonly title: string;
  readonly detail?: string | null;
  readonly evidence?: string | null;
  readonly status: TimelineStepStatus;
  readonly kind?: string | null;
}

export interface TimelineProps {
  readonly agents: readonly TimelineAgent[];
  readonly steps: readonly TimelineStep[];
  readonly selectedAgentId?: string | null;
  readonly onSelectAgent?: (agentId: string | null) => void;
  readonly status?: TimelineStatus;
  readonly errorMessage?: string | null;
  readonly emptyMessage?: string | null;
  readonly onRetry?: () => void;
  readonly className?: string;
}

export const TIMELINE_COPY = {
  section: 'Run timeline',
  allAgents: 'All agents',
  loading: 'Loading timeline…',
  idle: 'Timeline has not loaded yet.',
  error: 'Timeline unavailable',
  retry: 'Retry',
  empty: 'No timeline events yet.',
  emptyForAgent: 'No timeline events for this agent.',
  utc: 'UTC',
  events: 'events',
  agent: 'agent',
  approval: 'Approval needed',
  pending: 'Pending',
  running: 'Running',
  ok: 'Complete',
  failed: 'Failed',
  skipped: 'Skipped',
  awaitingApproval: 'Awaiting approval',
  evidence: 'evidence',
} as const;

const STATUS_LABELS: Readonly<Record<TimelineStepStatus, string>> = {
  pending: TIMELINE_COPY.pending,
  running: TIMELINE_COPY.running,
  ok: TIMELINE_COPY.ok,
  failed: TIMELINE_COPY.failed,
  skipped: TIMELINE_COPY.skipped,
  awaiting_approval: TIMELINE_COPY.awaitingApproval,
};

const STATUS_TONES: Readonly<Record<TimelineStepStatus, string>> = {
  pending: 'border-hairline bg-panel-alt text-ink-500',
  running: 'border-honey-500 bg-honey-100 text-ink-900',
  ok: 'border-verify bg-panel text-verify',
  failed: 'border-signal bg-error-surface text-signal',
  skipped: 'border-hairline bg-recessed text-ink-500',
  awaiting_approval: 'border-amber bg-panel-alt text-ink-800',
};

export function timelineStatusLabel(status: TimelineStepStatus | string): string {
  return Object.prototype.hasOwnProperty.call(STATUS_LABELS, status)
    ? STATUS_LABELS[status as TimelineStepStatus]
    : 'Unknown';
}

export function timelineStatusTone(status: TimelineStepStatus | string): string {
  return Object.prototype.hasOwnProperty.call(STATUS_TONES, status)
    ? STATUS_TONES[status as TimelineStepStatus]
    : STATUS_TONES.pending;
}

/** UTC HH:MM:SS; invalid/absent timestamps are not invented. */
export function formatTimelineTime(value: string | null | undefined): string {
  if (value === null || value === undefined || value.length === 0) {
    return '—';
  }
  const parsed = Date.parse(value);
  if (!Number.isFinite(parsed)) {
    return '—';
  }
  return new Date(parsed).toISOString().slice(11, 19);
}

function stepTime(value: string): number {
  const parsed = Date.parse(value);
  return Number.isFinite(parsed) ? parsed : Number.POSITIVE_INFINITY;
}

/** Ascending event time, then id; invalid times sort last deterministically. */
export function sortTimelineSteps(steps: readonly TimelineStep[]): readonly TimelineStep[] {
  return [...steps].sort((left, right) => {
    const leftTime = stepTime(left.at);
    const rightTime = stepTime(right.at);
    if (leftTime !== rightTime) {
      return leftTime < rightTime ? -1 : 1;
    }
    return left.id.localeCompare(right.id);
  });
}

/**
 * Agents declared by the host plus any agent only seen on a step. Order is the
 * host order first, then first appearance in the step list.
 */
export function timelineAgents(
  agents: readonly TimelineAgent[],
  steps: readonly TimelineStep[],
): readonly TimelineAgent[] {
  const byId = new Map<string, TimelineAgent>();
  for (const agent of agents) {
    byId.set(agent.id, agent);
  }
  for (const step of steps) {
    if (!byId.has(step.agentId)) {
      byId.set(step.agentId, {
        id: step.agentId,
        label: step.agentLabel ?? step.agentId,
        kind: null,
      });
    }
  }
  return [...byId.values()];
}

export interface TimelineGroup {
  readonly agent: TimelineAgent | null;
  readonly steps: readonly TimelineStep[];
}

/** Groups by agent in the declared order, then by first step appearance. */
export function groupTimelineSteps(
  steps: readonly TimelineStep[],
  agents: readonly TimelineAgent[],
): readonly TimelineGroup[] {
  const ordered = sortTimelineSteps(steps);
  const known = timelineAgents(agents, ordered);
  return known
    .map((agent) => ({
      agent,
      steps: ordered.filter((step) => step.agentId === agent.id),
    }))
    .filter((group) => group.steps.length > 0);
}

export type TimelineTabKey = 'ArrowLeft' | 'ArrowRight' | 'Home' | 'End';

/** Arrow/Home/End index mapping for the roving tablist (count includes "All"). */
export function timelineTabTarget(index: number, count: number, key: TimelineTabKey): number {
  if (count <= 0) {
    return 0;
  }
  const current = Math.min(Math.max(index, 0), count - 1);
  switch (key) {
    case 'ArrowLeft':
      return (current - 1 + count) % count;
    case 'ArrowRight':
      return (current + 1) % count;
    case 'Home':
      return 0;
    case 'End':
      return count - 1;
    default:
      return current;
  }
}

export function Timeline({
  agents,
  steps,
  selectedAgentId,
  onSelectAgent,
  status = 'ready',
  errorMessage = null,
  emptyMessage = null,
  onRetry,
  className = '',
}: TimelineProps) {
  const [internalAgentId, setInternalAgentId] = useState<string | null>(null);
  const [focusRequest, setFocusRequest] = useState<number | null>(null);
  const tabRefs = useRef<Map<number, HTMLButtonElement>>(new Map());
  const baseId = useId().replace(/[^a-zA-Z0-9_-]/g, '');
  const availableAgents = useMemo(() => timelineAgents(agents, steps), [agents, steps]);
  const controlled = selectedAgentId !== undefined;
  const requestedAgentId = controlled ? selectedAgentId : internalAgentId;
  const activeAgentId =
    requestedAgentId !== null && availableAgents.some((agent) => agent.id === requestedAgentId)
      ? requestedAgentId
      : null;
  const tabIds = [null, ...availableAgents.map((agent) => agent.id)];
  const activeIndex = Math.max(0, tabIds.indexOf(activeAgentId));
  const orderedSteps = sortTimelineSteps(steps);
  const visibleSteps =
    activeAgentId === null
      ? orderedSteps
      : orderedSteps.filter((step) => step.agentId === activeAgentId);

  useEffect(() => {
    if (focusRequest === null || typeof document === 'undefined') {
      return;
    }
    tabRefs.current.get(focusRequest)?.focus();
    setFocusRequest(null);
  }, [focusRequest]);

  const selectAgent = (agentId: string | null): void => {
    if (!controlled) {
      setInternalAgentId(agentId);
    }
    onSelectAgent?.(agentId);
  };

  const onTabKeyDown = (event: KeyboardEvent<HTMLDivElement>): void => {
    if (!['ArrowLeft', 'ArrowRight', 'Home', 'End'].includes(event.key)) {
      return;
    }
    event.preventDefault();
    const next = timelineTabTarget(activeIndex, tabIds.length, event.key as TimelineTabKey);
    const agentId = tabIds[next] ?? null;
    selectAgent(agentId);
    setFocusRequest(next);
  };

  return (
    <section
      aria-label={TIMELINE_COPY.section}
      aria-busy={status === 'loading'}
      className={`flex min-h-0 flex-col overflow-hidden rounded-[var(--radius-card)] border border-hairline bg-panel shadow-[var(--shadow-e1)] ${className}`}
    >
      <header className="flex flex-wrap items-center gap-2 border-b border-hairline bg-panel-alt px-3 py-2">
        <div className="flex min-w-0 flex-1 items-baseline gap-2">
          <h2 className="type-label text-ink-900">{TIMELINE_COPY.section}</h2>
          <span className="type-mono-meta text-meta">{TIMELINE_COPY.utc}</span>
        </div>
        <span className="type-mono-meta shrink-0 text-meta">
          {visibleSteps.length} {TIMELINE_COPY.events}
        </span>
        {status === 'ready' ? (
          <div
            role="tablist"
            aria-label={TIMELINE_COPY.section}
            className="flex max-w-full gap-1 overflow-x-auto pb-0.5"
            onKeyDown={onTabKeyDown}
          >
            {[null, ...availableAgents].map((agent, index) => {
              const id = agent === null ? null : agent.id;
              const selected = activeAgentId === id;
              return (
                <button
                  key={agent === null ? '__all__' : agent.id}
                  ref={(element) => {
                    if (element === null) {
                      tabRefs.current.delete(index);
                    } else {
                      tabRefs.current.set(index, element);
                    }
                  }}
                  type="button"
                  role="tab"
                  id={`${baseId}-tab-${index}`}
                  aria-selected={selected}
                  aria-controls={`${baseId}-panel`}
                  tabIndex={selected ? 0 : -1}
                  className={`type-mono-meta shrink-0 rounded-[var(--radius-pill)] border px-2 py-0.5 ${
                    selected
                      ? 'border-honey-400 bg-honey-100 text-ink-900'
                      : 'border-hairline bg-panel text-ink-500 hover:bg-panel-alt'
                  }`}
                  onClick={() => selectAgent(id)}
                >
                  {agent === null ? TIMELINE_COPY.allAgents : agent.label}
                </button>
              );
            })}
          </div>
        ) : null}
      </header>

      {status === 'loading' ? (
        <p role="status" className="type-mono-meta p-3 text-meta">
          {TIMELINE_COPY.loading}
        </p>
      ) : null}

      {status === 'idle' ? (
        <p role="status" className="type-mono-meta p-3 text-meta">
          {TIMELINE_COPY.idle}
        </p>
      ) : null}

      {status === 'error' ? (
        <div role="alert" className="m-3 rounded-[var(--radius-md)] border border-signal bg-error-surface p-3">
          <p className="type-label text-signal">{TIMELINE_COPY.error}</p>
          <p className="type-body-l mt-1 text-ink-900">
            {errorMessage ?? 'The timeline could not be loaded.'}
          </p>
          {onRetry === undefined ? null : (
            <button
              type="button"
              className="type-label mt-2 rounded-[var(--radius-button)] border border-signal px-3 py-1.5 text-signal"
              onClick={() => onRetry()}
            >
              {TIMELINE_COPY.retry}
            </button>
          )}
        </div>
      ) : null}

      {status === 'ready' ? (
        <div
          role="tabpanel"
          id={`${baseId}-panel`}
          aria-labelledby={`${baseId}-tab-${activeIndex}`}
          className="min-h-0 flex-1 overflow-y-auto p-2"
        >
          {visibleSteps.length === 0 ? (
            <p className="type-body-l p-2 text-ink-500">
              {emptyMessage ??
                (activeAgentId === null ? TIMELINE_COPY.empty : TIMELINE_COPY.emptyForAgent)}
            </p>
          ) : (
            <ol aria-label={TIMELINE_COPY.section} className="m-0 list-none space-y-2 p-0">
              {visibleSteps.map((step) => (
                <li
                  key={step.id}
                  aria-label={`${formatTimelineTime(step.at)} ${step.title}, ${timelineStatusLabel(step.status)}`}
                  className={`relative rounded-[var(--radius-sm)] border bg-panel p-2 pl-3 ${
                    step.status === 'awaiting_approval' ? 'border-amber' : 'border-hairline'
                  }`}
                >
                  <span
                    aria-hidden="true"
                    className={`absolute inset-y-0 left-0 w-1 rounded-l-[var(--radius-sm)] ${
                      step.status === 'ok'
                        ? 'bg-verify'
                        : step.status === 'failed'
                          ? 'bg-signal'
                          : step.status === 'running'
                            ? 'bg-honey-400'
                            : step.status === 'awaiting_approval'
                              ? 'bg-amber'
                              : 'bg-hairline'
                    }`}
                  />
                  <div className="flex flex-wrap items-baseline gap-x-2 gap-y-1">
                    <time className="type-mono-meta shrink-0 text-meta" dateTime={step.at}>
                      {formatTimelineTime(step.at)}
                    </time>
                    <span
                      className={`type-mono-meta shrink-0 rounded-[var(--radius-pill)] border px-1.5 py-0.5 ${timelineStatusTone(step.status)}`}
                      data-status={step.status}
                    >
                      {timelineStatusLabel(step.status)}
                    </span>
                    <span className="type-mono-code min-w-0 flex-1 text-ink-900">{step.title}</span>
                  </div>
                  <div className="type-mono-meta mt-1 flex flex-wrap gap-x-2 gap-y-1 text-meta">
                    <span title={step.agentId}>
                      {TIMELINE_COPY.agent}:{' '}
                      {availableAgents.find((agent) => agent.id === step.agentId)?.label ?? step.agentId}
                    </span>
                    {step.kind === undefined || step.kind === null ? null : <span>{step.kind}</span>}
                  </div>
                  {step.detail === undefined || step.detail === null || step.detail.length === 0 ? null : (
                    <p className="type-body-l mt-1 text-ink-700">{step.detail}</p>
                  )}
                  {step.evidence === undefined || step.evidence === null || step.evidence.length === 0 ? null : (
                    <p className="type-mono-code mt-1 break-words rounded-[var(--radius-xs)] bg-panel-alt px-2 py-1 text-ink-800">
                      {TIMELINE_COPY.evidence}: {step.evidence}
                    </p>
                  )}
                </li>
              ))}
            </ol>
          )}
        </div>
      ) : null}
    </section>
  );
}
