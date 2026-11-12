/**
 * S15/S16 tool activity (W12): renders the run's tool calls from TOOL_CALL/TOOL_RESULT.
 *
 * Outcomes rendered: pending | ok | refused | denied | timeout | cancelled, plus an
 * explicit empty state. No fabricated rows: an unknown outcome falls back to a neutral
 * 'Unknown' label and the count summary only counts declared outcomes. Rows are
 * grouped by the wire `task_id`; result `bytes`/`sha256` render only when the
 * TOOL_RESULT frame carried them (unknown is shown as '—', never estimated).
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
  /** Wire event timestamp when the frame carried one; never synthesized. */
  readonly at?: string;
  readonly code?: string;
  readonly text?: string;
  /** Owning task from the wire `task_id`; absent rows are grouped as unknown. */
  readonly taskId?: string | null;
  /** Result byte size from TOOL_RESULT; never estimated. */
  readonly bytes?: number | null;
  /** Result digest from TOOL_RESULT; never invented. */
  readonly sha256?: string | null;
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

/** Byte size is rendered only when the server reported a finite non-negative number. */
export function formatActivityBytes(value: number | null | undefined): string {
  return typeof value === 'number' && Number.isFinite(value) && value >= 0
    ? String(value)
    : UNKNOWN_ACTIVITY_VALUE;
}

export function formatActivitySha256(value: string | null | undefined, head = 12): string {
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

/** One agent's tool rows; `taskId === null` means the wire omitted attribution. */
export interface ToolActivityGroup {
  readonly taskId: string | null;
  readonly entries: readonly ToolActivityEntry[];
}

/**
 * Groups rows by owning task while preserving the deterministic turn order
 * inside each group. Group order follows the first row of each task.
 */
export function groupToolActivity(
  entries: readonly ToolActivityEntry[],
): readonly ToolActivityGroup[] {
  const groups = new Map<string | null, ToolActivityEntry[]>();
  for (const entry of sortToolActivity(entries)) {
    const rawTaskId = entry.taskId ?? null;
    const taskId = rawTaskId === null || rawTaskId.trim().length === 0 ? null : rawTaskId;
    const bucket = groups.get(taskId) ?? [];
    bucket.push(entry);
    groups.set(taskId, bucket);
  }
  return [...groups.entries()].map(([taskId, grouped]) => ({ taskId, entries: grouped }));
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
  const groups = groupToolActivity(activity);
  // A single legacy group needs no heading; mixed/known attribution always shows it.
  const showGroupLabels =
    groups.length > 1 || groups.some((group) => group.taskId !== null);
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
          {groups.map((group) => (
            <li key={group.taskId ?? 'unattributed'} className="list-none">
              {showGroupLabels ? (
                <div className="type-mono-meta mb-1 flex items-center justify-between gap-2 text-meta">
                  <span title={group.taskId ?? undefined}>
                    {labels.taskId}{' '}
                    {group.taskId === null
                      ? STRINGS.toolActivity.unattributedTask
                      : truncateActivityValue(group.taskId)}
                  </span>
                  <span>
                    {group.entries.length} {STRINGS.toolActivity.calls}
                  </span>
                </div>
              ) : null}
              <ul className="m-0 list-none space-y-2 p-0">
                {group.entries.map((entry) => (
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
                      <span title={entry.taskId ?? undefined}>
                        {labels.taskId} {truncateActivityValue(entry.taskId)}
                      </span>
                      <span>
                        {labels.turn} {entry.turn}
                      </span>
                      <span>
                        {labels.bytes} {formatActivityBytes(entry.bytes)}
                      </span>
                      <span title={entry.sha256 ?? undefined}>
                        {labels.sha256} {formatActivitySha256(entry.sha256)}
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
            </li>
          ))}
        </ul>
      )}
    </section>
  );
}
