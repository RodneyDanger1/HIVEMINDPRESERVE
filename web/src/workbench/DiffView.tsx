/**
 * CAP-4 workbench — code-diff review with an explicit decision gate.
 *
 * The diff is computed locally from the caller's real `before`/`after` text by a
 * deterministic LCS diff (coarse whole-file replacement above a bounded cell
 * budget, labelled as such). Nothing here applies a change: accept/reject are
 * caller callbacks, and the button state is derived only from the supplied
 * `approvalState`, `busy` and `decision` props.
 */
import { useMemo } from 'react';
import type { KeyboardEvent } from 'react';

export type DiffStatus = 'ready' | 'loading' | 'error';
export type DiffApprovalState = 'not_required' | 'required' | 'pending' | 'approved' | 'denied' | 'error';
export type DiffDecision = 'accepted' | 'rejected';
export type DiffLineKind = 'context' | 'add' | 'remove';

export interface DiffLine {
  readonly kind: DiffLineKind;
  readonly text: string;
  /** 1-based line in the old file, or null for an added line. */
  readonly oldLine: number | null;
  /** 1-based line in the new file, or null for a removed line. */
  readonly newLine: number | null;
}

export interface DiffHunk {
  readonly index: number;
  readonly oldStart: number;
  readonly oldCount: number;
  readonly newStart: number;
  readonly newCount: number;
  readonly lines: readonly DiffLine[];
}

export interface DiffResult {
  readonly hunks: readonly DiffHunk[];
  readonly additions: number;
  readonly deletions: number;
  /** True when only the presence of a final newline differs between the sides. */
  readonly eolChange: boolean;
  /** True when the input exceeded the LCS budget and a whole-file replace is shown. */
  readonly coarse: boolean;
}

export interface DiffViewProps {
  readonly filePath: string;
  readonly before: string;
  readonly after: string;
  readonly status?: DiffStatus;
  readonly errorMessage?: string | null;
  readonly language?: string | null;
  readonly approvalState?: DiffApprovalState;
  readonly approvalMessage?: string | null;
  readonly busy?: boolean;
  readonly decision?: DiffDecision | null;
  readonly onRequestApproval?: () => void;
  readonly onAccept?: () => void;
  readonly onReject?: () => void;
  readonly onRetry?: () => void;
  readonly className?: string;
}

export const DIFF_COPY = {
  section: 'Code diff',
  loading: 'Computing diff…',
  error: 'Diff unavailable',
  retry: 'Reload diff',
  noChanges: 'No changes: the proposed content matches the current file.',
  coarse:
    'Large file: showing a whole-file replacement diff because the line-by-line budget was exceeded.',
  truncated: 'Diff output is truncated after 2,000 lines; the decision still applies to the full file.',
  eolChange: 'Line endings or the end-of-file newline differ (line content is unchanged).',
  approvalRequired: 'Approval required before this diff can be applied.',
  approvalPending: 'Waiting for approval before this diff can be applied.',
  approvalApproved: 'Approval received; the diff can be applied.',
  approvalDenied: 'Approval was denied; the diff cannot be applied.',
  approvalError: 'The approval request failed.',
  requestApproval: 'Request approval',
  accept: 'Accept',
  reject: 'Reject',
  accepting: 'Applying decision…',
  accepted: 'Accepted',
  rejected: 'Rejected',
  shortcut: 'Ctrl/Cmd+Enter accepts when enabled',
  additions: 'additions',
  deletions: 'deletions',
  hunks: 'hunks',
} as const;

/** Bounded LCS cell budget: above this a coarse replacement is shown instead. */
export const MAX_DIFF_CELLS = 1_000_000;
export const MAX_DIFF_LINES = 5_000;
export const DIFF_CONTEXT_LINES = 3;
export const MAX_RENDERED_DIFF_LINES = 2_000;

function splitFileLines(text: string): readonly string[] {
  if (text === '') {
    return [];
  }
  const lines = text.replace(/\r\n?/g, '\n').split('\n');
  // A final newline terminates the last line rather than creating a phantom
  // extra blank line. EOL-only changes are reported through `eolChange`.
  if (lines[lines.length - 1] === '') {
    lines.pop();
  }
  return lines;
}

function contextLine(text: string, oldLine: number, newLine: number): DiffLine {
  return { kind: 'context', text, oldLine, newLine };
}
function addedLine(text: string, newLine: number): DiffLine {
  return { kind: 'add', text, oldLine: null, newLine };
}
function removedLine(text: string, oldLine: number): DiffLine {
  return { kind: 'remove', text, oldLine, newLine: null };
}

function coarseDiff(oldLines: readonly string[], newLines: readonly string[]): readonly DiffLine[] {
  const lines: DiffLine[] = [];
  oldLines.forEach((text, index) => lines.push(removedLine(text, index + 1)));
  newLines.forEach((text, index) => lines.push(addedLine(text, index + 1)));
  return lines;
}

function lcsDiff(oldLines: readonly string[], newLines: readonly string[]): readonly DiffLine[] {
  const oldCount = oldLines.length;
  const newCount = newLines.length;
  const width = newCount + 1;
  const table: Uint32Array[] = Array.from({ length: oldCount + 1 }, () => new Uint32Array(width));
  for (let i = oldCount - 1; i >= 0; i -= 1) {
    const row = table[i];
    const next = table[i + 1];
    if (row === undefined || next === undefined) {
      continue;
    }
    for (let j = newCount - 1; j >= 0; j -= 1) {
      row[j] =
        oldLines[i] === newLines[j]
          ? (next[j + 1] ?? 0) + 1
          : Math.max(next[j] ?? 0, row[j + 1] ?? 0);
    }
  }

  const lines: DiffLine[] = [];
  let i = 0;
  let j = 0;
  let oldLine = 1;
  let newLine = 1;
  while (i < oldCount && j < newCount) {
    if (oldLines[i] === newLines[j]) {
      lines.push(contextLine(oldLines[i] ?? '', oldLine, newLine));
      i += 1;
      j += 1;
      oldLine += 1;
      newLine += 1;
      continue;
    }
    const row = table[i];
    const next = table[i + 1];
    const down = next?.[j] ?? 0;
    const right = row?.[j + 1] ?? 0;
    if (down >= right) {
      lines.push(removedLine(oldLines[i] ?? '', oldLine));
      i += 1;
      oldLine += 1;
    } else {
      lines.push(addedLine(newLines[j] ?? '', newLine));
      j += 1;
      newLine += 1;
    }
  }
  while (i < oldCount) {
    lines.push(removedLine(oldLines[i] ?? '', oldLine));
    i += 1;
    oldLine += 1;
  }
  while (j < newCount) {
    lines.push(addedLine(newLines[j] ?? '', newLine));
    j += 1;
    newLine += 1;
  }
  return lines;
}

function buildHunks(lines: readonly DiffLine[]): readonly DiffHunk[] {
  const changeIndices: number[] = [];
  lines.forEach((line, index) => {
    if (line.kind !== 'context') {
      changeIndices.push(index);
    }
  });
  if (changeIndices.length === 0) {
    return [];
  }

  const ranges: { start: number; end: number }[] = [];
  for (const index of changeIndices) {
    const start = Math.max(0, index - DIFF_CONTEXT_LINES);
    const end = Math.min(lines.length - 1, index + DIFF_CONTEXT_LINES);
    const last = ranges[ranges.length - 1];
    if (last !== undefined && start <= last.end + 1) {
      last.end = Math.max(last.end, end);
    } else {
      ranges.push({ start, end });
    }
  }

  const oldBefore = new Array<number>(lines.length);
  const newBefore = new Array<number>(lines.length);
  let oldSeen = 0;
  let newSeen = 0;
  lines.forEach((line, index) => {
    oldBefore[index] = oldSeen;
    newBefore[index] = newSeen;
    if (line.oldLine !== null) {
      oldSeen += 1;
    }
    if (line.newLine !== null) {
      newSeen += 1;
    }
  });

  return ranges.map((range, index) => {
    const slice = lines.slice(range.start, range.end + 1);
    const oldNumbers = slice.flatMap((line) => (line.oldLine === null ? [] : [line.oldLine]));
    const newNumbers = slice.flatMap((line) => (line.newLine === null ? [] : [line.newLine]));
    const oldCount = oldNumbers.length;
    const newCount = newNumbers.length;
    return {
      index,
      oldStart: oldCount > 0 ? (oldNumbers[0] ?? 0) : (oldBefore[range.start] ?? 0),
      oldCount,
      newStart: newCount > 0 ? (newNumbers[0] ?? 0) : (newBefore[range.start] ?? 0),
      newCount,
      lines: slice,
    };
  });
}

/** Deterministic line diff. Never mutates its inputs. */
export function diffLines(before: string, after: string): DiffResult {
  const oldLines = splitFileLines(before);
  const newLines = splitFileLines(after);
  const cells = (oldLines.length + 1) * (newLines.length + 1);
  const coarse =
    oldLines.length > MAX_DIFF_LINES ||
    newLines.length > MAX_DIFF_LINES ||
    cells > MAX_DIFF_CELLS;
  const lines = coarse ? coarseDiff(oldLines, newLines) : lcsDiff(oldLines, newLines);
  const normalizedBefore = before.replace(/\r\n?/g, '\n');
  const normalizedAfter = after.replace(/\r\n?/g, '\n');
  const eolChange =
    before !== after &&
    (before.endsWith('\n') !== after.endsWith('\n') ||
      normalizedBefore === normalizedAfter);
  return {
    hunks: buildHunks(lines),
    additions: lines.filter((line) => line.kind === 'add').length,
    deletions: lines.filter((line) => line.kind === 'remove').length,
    eolChange,
    coarse,
  };
}

export function formatDiffHunkHeader(hunk: DiffHunk): string {
  return `@@ -${hunk.oldStart},${hunk.oldCount} +${hunk.newStart},${hunk.newCount} @@`;
}

export function diffLineAriaLabel(line: DiffLine): string {
  const text = line.text.length === 0 ? 'blank line' : line.text;
  if (line.kind === 'add') {
    return `Added line ${line.newLine ?? '?'}: ${text}`;
  }
  if (line.kind === 'remove') {
    return `Removed line ${line.oldLine ?? '?'}: ${text}`;
  }
  return `Line ${line.newLine ?? line.oldLine ?? '?'}: ${text}`;
}

export interface LimitedDiff {
  readonly hunks: readonly DiffHunk[];
  readonly truncated: boolean;
}

/** Caps rendered rows without changing the decision target. */
export function limitDiffHunks(
  result: DiffResult,
  maxLines: number = MAX_RENDERED_DIFF_LINES,
): LimitedDiff {
  const limit = Math.max(0, Math.trunc(maxLines));
  const hunks: DiffHunk[] = [];
  let remaining = limit;
  let truncated = false;
  for (const hunk of result.hunks) {
    if (remaining <= 0) {
      truncated = true;
      break;
    }
    if (hunk.lines.length <= remaining) {
      hunks.push(hunk);
      remaining -= hunk.lines.length;
      continue;
    }
    const lines = hunk.lines.slice(0, remaining);
    const oldCount = lines.filter((line) => line.oldLine !== null).length;
    const newCount = lines.filter((line) => line.newLine !== null).length;
    hunks.push({
      ...hunk,
      lines,
      oldCount,
      newCount,
    });
    remaining = 0;
    truncated = true;
    break;
  }
  return { hunks, truncated };
}

function ApprovalBanner({
  state,
  message,
  busy,
  onRequestApproval,
}: {
  state: DiffApprovalState;
  message: string | null;
  busy: boolean;
  onRequestApproval?: () => void;
}) {
  if (state === 'not_required' || state === 'approved') {
    if (state === 'not_required') {
      return null;
    }
    return (
      <p
        role="status"
        data-approval-state={state}
        className="type-mono-meta border-b border-verify bg-panel-alt px-3 py-1.5 text-verify"
      >
        {DIFF_COPY.approvalApproved}
      </p>
    );
  }
  const denied = state === 'denied';
  const failed = state === 'error';
  return (
    <div
      role={denied || failed ? 'alert' : 'status'}
      data-approval-state={state}
      className={`border-b px-3 py-1.5 ${
        denied ? 'border-signal bg-error-surface' : 'border-amber bg-panel-alt'
      }`}
    >
      <p className={`type-mono-meta ${denied ? 'text-signal' : 'text-ink-800'}`}>
        {state === 'required'
          ? DIFF_COPY.approvalRequired
          : state === 'pending'
            ? DIFF_COPY.approvalPending
            : state === 'denied'
              ? DIFF_COPY.approvalDenied
              : DIFF_COPY.approvalError}
      </p>
      {message ? <p className="type-body-l mt-0.5 text-ink-900">{message}</p> : null}
      {onRequestApproval === undefined ? null : (
        <button
          type="button"
          className="type-label mt-1 rounded-[var(--radius-button)] border border-honey-500 bg-honey-300 px-2 py-1 text-ink-900 hover:bg-honey-400 disabled:cursor-not-allowed disabled:opacity-50"
          disabled={busy || state === 'pending'}
          onClick={() => onRequestApproval()}
        >
          {DIFF_COPY.requestApproval}
        </button>
      )}
    </div>
  );
}

export function DiffView({
  filePath,
  before,
  after,
  status = 'ready',
  errorMessage = null,
  language = null,
  approvalState = 'not_required',
  approvalMessage = null,
  busy = false,
  decision = null,
  onRequestApproval,
  onAccept,
  onReject,
  onRetry,
  className = '',
}: DiffViewProps) {
  const result = useMemo(() => diffLines(before, after), [before, after]);
  const limited = useMemo(() => limitDiffHunks(result), [result]);
  const hasChanges = result.additions > 0 || result.deletions > 0 || result.eolChange;
  const approved = approvalState === 'not_required' || approvalState === 'approved';
  const canAccept =
    status === 'ready' &&
    hasChanges &&
    !busy &&
    decision === null &&
    approved &&
    onAccept !== undefined;
  const canReject =
    status === 'ready' && hasChanges && !busy && decision === null && onReject !== undefined;

  const onKeyDown = (event: KeyboardEvent<HTMLElement>): void => {
    if (event.key !== 'Enter' || (!event.ctrlKey && !event.metaKey)) {
      return;
    }
    event.preventDefault();
    if (canAccept) {
      onAccept?.();
    }
  };

  return (
    <section
      aria-label={`${DIFF_COPY.section}: ${filePath}`}
      aria-busy={busy || status === 'loading'}
      data-approval-state={approvalState}
      data-decision={decision ?? 'none'}
      className={`flex min-h-0 flex-col overflow-hidden rounded-[var(--radius-card)] border border-hairline bg-panel shadow-[var(--shadow-e1)] ${className}`}
      onKeyDown={onKeyDown}
    >
      <header className="flex flex-wrap items-center gap-2 border-b border-hairline bg-panel-alt px-3 py-2">
        <div className="flex min-w-0 flex-1 items-baseline gap-2">
          <h2 className="type-label shrink-0 text-ink-900">{DIFF_COPY.section}</h2>
          <span className="type-mono-code min-w-0 truncate text-ink-800" title={filePath}>
            {filePath}
          </span>
        </div>
        {language === null ? null : (
          <span className="type-mono-meta hidden shrink-0 text-meta sm:inline">{language}</span>
        )}
        {status === 'ready' ? (
          <>
            <span className="type-mono-meta shrink-0 text-verify">+{result.additions}</span>
            <span className="type-mono-meta shrink-0 text-signal">−{result.deletions}</span>
            <span className="type-mono-meta shrink-0 text-meta">
              {result.hunks.length} {DIFF_COPY.hunks}
            </span>
          </>
        ) : null}
      </header>

      <ApprovalBanner
        state={approvalState}
        message={approvalMessage}
        busy={busy}
        onRequestApproval={onRequestApproval}
      />

      {status === 'ready' && result.coarse ? (
        <p className="type-mono-meta border-b border-amber bg-recessed px-3 py-1.5 text-ink-800">
          {DIFF_COPY.coarse}
        </p>
      ) : null}

      {status === 'loading' ? (
        <p role="status" className="type-mono-meta p-3 text-meta">
          {DIFF_COPY.loading}
        </p>
      ) : null}

      {status === 'error' ? (
        <div role="alert" className="m-3 rounded-[var(--radius-md)] border border-signal bg-error-surface p-3">
          <p className="type-label text-signal">{DIFF_COPY.error}</p>
          <p className="type-body-l mt-1 text-ink-900">
            {errorMessage ?? 'The diff could not be computed.'}
          </p>
          {onRetry === undefined ? null : (
            <button
              type="button"
              className="type-label mt-2 rounded-[var(--radius-button)] border border-signal px-3 py-1.5 text-signal"
              onClick={() => onRetry()}
            >
              {DIFF_COPY.retry}
            </button>
          )}
        </div>
      ) : null}

      {status === 'ready' && !hasChanges ? (
        <div className="p-3">
          <p className="type-body-l text-ink-500">{DIFF_COPY.noChanges}</p>
        </div>
      ) : null}

      {status === 'ready' && hasChanges ? (
        <>
          {result.eolChange ? (
            <p className="type-mono-meta border-b border-amber bg-recessed px-3 py-1.5 text-ink-800">
              {DIFF_COPY.eolChange}
            </p>
          ) : null}
          <div className="min-h-0 flex-1 overflow-auto">
            <ol aria-label={DIFF_COPY.section} className="m-0 list-none p-0">
              {limited.hunks.map((hunk) => (
                <li key={hunk.index} className="m-0 border-b border-hairline last:border-b-0">
                  <div className="type-mono-meta sticky top-0 z-10 border-b border-hairline bg-recessed px-3 py-1 text-ink-800">
                    {formatDiffHunkHeader(hunk)}
                  </div>
                  <ol className="m-0 list-none p-0">
                    {hunk.lines.map((line) => (
                      <li
                        key={`${hunk.index}-${line.kind}-${line.oldLine ?? 'n'}-${line.newLine ?? 'n'}`}
                        aria-label={diffLineAriaLabel(line)}
                        data-kind={line.kind}
                        className={`type-mono-code grid grid-cols-[2.5rem_2.5rem_1.25rem_minmax(0,1fr)] ${
                          line.kind === 'add'
                            ? 'bg-honey-100 text-ink-900'
                            : line.kind === 'remove'
                              ? 'bg-error-surface text-ink-900'
                              : 'bg-panel text-ink-800'
                        }`}
                      >
                        <span className="select-none border-r border-hairline px-1 text-right text-meta">
                          {line.oldLine ?? ''}
                        </span>
                        <span className="select-none border-r border-hairline px-1 text-right text-meta">
                          {line.newLine ?? ''}
                        </span>
                        <span
                          aria-hidden="true"
                          className={`select-none text-center ${
                            line.kind === 'add'
                              ? 'text-verify'
                              : line.kind === 'remove'
                                ? 'text-signal'
                                : 'text-ink-500'
                          }`}
                        >
                          {line.kind === 'add' ? '+' : line.kind === 'remove' ? '−' : ' '}
                        </span>
                        <span className="whitespace-pre-wrap break-words px-2">
                          {line.text.length === 0 ? '\u00a0' : line.text}
                        </span>
                      </li>
                    ))}
                  </ol>
                </li>
              ))}
            </ol>
          </div>
          {limited.truncated ? (
            <p className="type-mono-meta border-t border-amber bg-recessed px-3 py-1.5 text-ink-800">
              {DIFF_COPY.truncated}
            </p>
          ) : null}
        </>
      ) : null}

      {status === 'ready' && hasChanges ? (
        <footer className="flex flex-col-reverse gap-2 border-t border-hairline bg-panel-alt px-3 py-2 sm:flex-row sm:items-center sm:justify-between">
          <div className="type-mono-meta flex flex-wrap items-center gap-2 text-meta">
            {decision === null ? (
              <span className="hidden sm:inline">{DIFF_COPY.shortcut}</span>
            ) : (
              <span
                role="status"
                className={decision === 'accepted' ? 'text-verify' : 'text-signal'}
              >
                {decision === 'accepted' ? DIFF_COPY.accepted : DIFF_COPY.rejected}
              </span>
            )}
            {busy ? (
              <span role="status" aria-live="polite">
                {DIFF_COPY.accepting}
              </span>
            ) : null}
          </div>
          <div className="flex flex-col-reverse gap-2 sm:flex-row">
            <button
              type="button"
              className="type-label rounded-[var(--radius-button)] border border-signal px-3 py-1.5 text-signal disabled:cursor-not-allowed disabled:opacity-50"
              disabled={!canReject}
              aria-label={`${DIFF_COPY.reject} changes to ${filePath}`}
              onClick={() => onReject?.()}
            >
              {DIFF_COPY.reject}
            </button>
            <button
              type="button"
              className="type-label rounded-[var(--radius-button)] border border-honey-600 bg-honey-300 px-3 py-1.5 text-ink-900 hover:bg-honey-400 disabled:cursor-not-allowed disabled:opacity-50"
              disabled={!canAccept}
              aria-label={`${DIFF_COPY.accept} changes to ${filePath}`}
              onClick={() => onAccept?.()}
            >
              {DIFF_COPY.accept}
            </button>
          </div>
        </footer>
      ) : null}
    </section>
  );
}
