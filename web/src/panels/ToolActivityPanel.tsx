/**
 * S15/S16 tool activity (W12): renders the run's tool calls from TOOL_CALL/TOOL_RESULT.
 *
 * Outcomes rendered: pending | ok | refused | denied | timeout | cancelled, plus an
 * explicit empty state. No fabricated rows: an unknown outcome falls back to a neutral
 * 'Unknown' label and the count summary only counts declared outcomes.
 *
 * W12 wave 3: all user-facing copy (heading, empty state, section/list labels,
 * outcome labels, evidence labels) resolves from `STRINGS.toolActivity`; the
 * rendered strings/selectors are unchanged for the e2e contract
 * (`section[aria-label="Tool activity"]`, `ul[aria-label="Tool calls"]`,
 * `li[data-outcome=…]`).
 *
 * Store note: `toolActivity` is owned by the store seat (W12). This panel accepts
 * `entries` as props (App/Lead wiring) and falls back to a defensive store read so it
 * works before and after the store change; it never writes to the store.
 */
import type { ToolOutcome } from '../api/contract';
import { useUiStore } from '../state/context';
import { STRINGS } from '../state/strings';
import { EmptyState, PanelTitle } from './StatePack';

export type ToolActivityOutcome = 'pending' | ToolOutcome;

export interface ToolActivityEntry {
  readonly toolCallId: string;
  readonly tool: string;
  readonly argsSha256: string;
  readonly turn: number;
  readonly outcome: ToolActivityOutcome;
  readonly code?: string;
  readonly text?: string;
}

export const TOOL_ACTIVITY_OUTCOMES: readonly ToolActivityOutcome[] = [
  'pending',
  'ok',
  'refused',
  'denied',
  'timeout',
  'cancelled',
] as const;

const OUTCOME_LABELS: Readonly<Record<ToolActivityOutcome, string>> = {
  pending: STRINGS.toolActivity.outcomes.pending,
  ok: STRINGS.toolActivity.outcomes.ok,
  refused: STRINGS.toolActivity.outcomes.refused,
  denied: STRINGS.toolActivity.outcomes.denied,
  timeout: STRINGS.toolActivity.outcomes.timeout,
  cancelled: STRINGS.toolActivity.outcomes.cancelled,
};

const OUTCOME_TONES: Readonly<Record<ToolActivityOutcome, string>> = {
  pending: 'border-hairline bg-panel-alt text-ink-500',
  ok: 'border-verify bg-panel text-verify',
  refused: 'border-signal bg-panel text-signal',
  denied: 'border-signal bg-panel text-signal',
  timeout: 'border-amber bg-panel text-amber',
  cancelled: 'border-hairline bg-panel text-ink-500',
};

export const UNKNOWN_ACTIVITY_VALUE = STRINGS.toolActivity.unknownValue;

export function isToolActivityOutcome(value: unknown): value is ToolActivityOutcome {
  return typeof value === 'string' && (TOOL_ACTIVITY_OUTCOMES as readonly string[]).includes(value);
}

export function toolActivityOutcomeLabel(outcome: ToolActivityOutcome | string): string {
  return isToolActivityOutcome(outcome) ? OUTCOME_LABELS[outcome] : STRINGS.toolActivity.outcomes.unknown;
}

export function toolActivityOutcomeTone(outcome: ToolActivityOutcome | string): string {
  return isToolActivityOutcome(outcome) ? OUTCOME_TONES[outcome] : OUTCOME_TONES.pending;
}

/** Truncates ids/hashes for display; missing values render the honest '—'. */
export function truncateActivityValue(value: string | null | undefined, head = 12): string {
  if (value === null || value === undefined || value.length === 0) {
    return UNKNOWN_ACTIVITY_VALUE;
  }
  return value.length <= head ? value : `${value.slice(0, head)}…`;
}

export function formatActivityArgsSha256(value: string | null | undefined, head = 12): string {
  return truncateActivityValue(value, head);
}

/** Deterministic ordering: turn ascending, then tool_call_id so captures are stable. */
export function sortToolActivity(entries: readonly ToolActivityEntry[]): readonly ToolActivityEntry[] {
  return [...entries].sort((a, b) => {
    if (a.turn !== b.turn) {
      return a.turn - b.turn;
    }
    return a.toolCallId.localeCompare(b.toolCallId);
  });
}

export interface ToolActivitySummary {
  readonly total: number;
  readonly counts: Readonly<Record<ToolActivityOutcome, number>>;
}

export function toolActivitySummary(entries: readonly ToolActivityEntry[]): ToolActivitySummary {
  const counts: Record<ToolActivityOutcome, number> = {
    pending: 0,
    ok: 0,
    refused: 0,
    denied: 0,
    timeout: 0,
    cancelled: 0,
  };
  for (const entry of entries) {
    if (isToolActivityOutcome(entry.outcome)) {
      counts[entry.outcome] += 1;
    }
  }
  return { total: entries.length, counts };
}

export function toolActivityAriaLabel(entry: ToolActivityEntry): string {
  return `${entry.tool} tool call ${truncateActivityValue(entry.toolCallId)}, ${toolActivityOutcomeLabel(entry.outcome)}`;
}

function useStoredToolActivity(): readonly ToolActivityEntry[] {
  return useUiStore((state) => {
    const candidate = (state as unknown as { toolActivity?: readonly ToolActivityEntry[] }).toolActivity;
    return candidate ?? [];
  });
}

export function ToolActivityPanel({ entries }: { entries?: readonly ToolActivityEntry[] } = {}) {
  const stored = useStoredToolActivity();
  const activity = sortToolActivity(entries ?? stored);
  const summary = toolActivitySummary(activity);
  const labels = STRINGS.toolActivity.labels;

  return (
    <section aria-label={STRINGS.toolActivity.sectionAria} className="h-full min-h-0 overflow-y-auto p-4">
      <PanelTitle>
        {STRINGS.toolActivity.heading} · {summary.total}
      </PanelTitle>
      {activity.length === 0 ? (
        <EmptyState message={STRINGS.toolActivity.empty} />
      ) : (
        <ul aria-label={STRINGS.toolActivity.listAria} aria-live="polite" className="mt-2 space-y-2">
          {activity.map((entry) => (
            <li
              key={entry.toolCallId}
              data-outcome={entry.outcome}
              aria-label={toolActivityAriaLabel(entry)}
              className={`surface-panel rounded-[var(--radius-card)] border px-3 py-2 ${toolActivityOutcomeTone(entry.outcome)}`}
            >
              <div className="type-mono-meta flex items-center justify-between gap-2">
                <span className="type-mono-code text-ink-900">{entry.tool}</span>
                <span className="type-label">{toolActivityOutcomeLabel(entry.outcome)}</span>
              </div>
              <div className="type-mono-meta mt-1 flex flex-wrap gap-x-3 gap-y-1 text-meta">
                <span title={entry.argsSha256}>
                  {labels.argsSha256} {formatActivityArgsSha256(entry.argsSha256)}
                </span>
                <span>
                  {labels.turn} {entry.turn}
                </span>
                <span title={entry.toolCallId}>
                  {labels.toolCallId} {truncateActivityValue(entry.toolCallId)}
                </span>
              </div>
              {entry.code === undefined ? null : (
                <div className="type-mono-code mt-1 text-signal">
                  {labels.code} {entry.code}
                </div>
              )}
              {entry.text === undefined ? null : (
                <div className="type-mono-meta mt-1 truncate text-meta" title={entry.text}>
                  {entry.text.length > 160 ? `${entry.text.slice(0, 160)}…` : entry.text}
                </div>
              )}
            </li>
          ))}
        </ul>
      )}
    </section>
  );
}
