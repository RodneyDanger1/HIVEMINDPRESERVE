/**
 * CAP-4 workbench — file editor surface.
 *
 * This is a real controlled editor over a plain textarea: line numbers, cursor
 * line, dirty/save state, read-only mode and an explicit approval gate. It does
 * not pretend to have syntax highlighting or CodeMirror features; those are
 * listed in `WORKBENCH_OPTIONAL_DEPENDENCIES` (index.ts) and are never claimed
 * until the dependency is pinned and built.
 *
 * No save succeeds locally: `onSave` is the caller's write path, and the
 * component only ever renders the `dirty`/`status`/`approval` state it is given.
 */
import { useEffect, useRef, useState } from 'react';
import type { KeyboardEvent } from 'react';

export type EditorStatus = 'idle' | 'loading' | 'ready' | 'error';
export type EditorApprovalState = 'required' | 'pending' | 'approved' | 'denied';

export interface EditorApproval {
  readonly id: string;
  readonly tool: string;
  readonly state: EditorApprovalState;
  readonly message?: string | null;
}

export interface EditorProps {
  readonly path: string;
  readonly value: string;
  readonly onChange?: (value: string) => void;
  readonly onSave?: (value: string) => void;
  readonly onRetry?: () => void;
  readonly onRequestApproval?: () => void;
  readonly status?: EditorStatus;
  readonly errorMessage?: string | null;
  readonly readOnly?: boolean;
  readonly readOnlyReason?: string | null;
  readonly language?: string | null;
  readonly dirty?: boolean;
  readonly approval?: EditorApproval | null;
  readonly ariaLabel?: string;
  readonly className?: string;
}

export const EDITOR_COPY = {
  section: 'Editor',
  emptyTitle: 'No file open',
  emptyBody: 'Select a file in the workspace tree to inspect it here.',
  loading: 'Opening file…',
  error: 'File could not be opened',
  retry: 'Retry',
  save: 'Save',
  saved: 'Saved',
  modified: 'Modified',
  readOnly: 'Read-only',
  approvalRequired: 'Approval required before saving',
  approvalPending: 'Waiting for approval before saving',
  approvalApproved: 'Write approved',
  approvalDenied: 'Write approval denied',
  requestApproval: 'Request approval',
  characters: 'chars',
  lines: 'lines',
  lineLimit: 'Line numbers are limited to the first 2,000 lines.',
  cursor: 'Ln',
} as const;

/** Hard cap so a multi-megabyte file cannot create tens of thousands of gutter nodes. */
export const MAX_RENDERED_LINE_NUMBERS = 2_000;

export function countEditorLines(value: string): number {
  return value.length === 0 ? 1 : value.split('\n').length;
}

export function editorLineForOffset(value: string, offset: number): number {
  const safeOffset = Math.min(Math.max(Number.isFinite(offset) ? Math.trunc(offset) : 0, 0), value.length);
  let line = 1;
  for (let index = 0; index < safeOffset; index += 1) {
    if (value.charCodeAt(index) === 10) {
      line += 1;
    }
  }
  return line;
}

export interface EditorLineNumbers {
  readonly numbers: readonly number[];
  readonly truncated: boolean;
}

export function buildEditorLineNumbers(value: string): EditorLineNumbers {
  const total = countEditorLines(value);
  const visible = Math.min(total, MAX_RENDERED_LINE_NUMBERS);
  const numbers = Array.from({ length: visible }, (_, index) => index + 1);
  return { numbers, truncated: total > MAX_RENDERED_LINE_NUMBERS };
}

export type EditorShortcut = 'save' | null;

export function editorShortcut(
  key: string,
  modifiers: { readonly ctrlKey?: boolean; readonly metaKey?: boolean; readonly altKey?: boolean },
): EditorShortcut {
  if (key.toLowerCase() !== 's' || modifiers.altKey === true) {
    return null;
  }
  return modifiers.ctrlKey === true || modifiers.metaKey === true ? 'save' : null;
}

export function approvalBlocksSave(approval: EditorApproval | null | undefined): boolean {
  if (approval === null || approval === undefined) {
    return false;
  }
  return approval.state !== 'approved';
}

export function editorStatusLabel(status: EditorStatus): string {
  switch (status) {
    case 'idle':
      return EDITOR_COPY.emptyTitle;
    case 'loading':
      return EDITOR_COPY.loading;
    case 'error':
      return EDITOR_COPY.error;
    case 'ready':
      return EDITOR_COPY.section;
  }
}

function ApprovalBanner({
  approval,
  onRequestApproval,
}: {
  approval: EditorApproval;
  onRequestApproval?: () => void;
}) {
  if (approval.state === 'approved') {
    return (
      <p
        role="status"
        data-approval-state={approval.state}
        className="type-mono-meta border-b border-verify bg-panel-alt px-3 py-1.5 text-verify"
      >
        {EDITOR_COPY.approvalApproved} · {approval.tool}
      </p>
    );
  }
  if (approval.state === 'pending') {
    return (
      <p
        role="status"
        data-approval-state={approval.state}
        className="type-mono-meta border-b border-amber bg-panel-alt px-3 py-1.5 text-ink-800"
      >
        {EDITOR_COPY.approvalPending} · {approval.tool}
      </p>
    );
  }
  const denied = approval.state === 'denied';
  return (
    <div
      role={denied ? 'alert' : 'status'}
      data-approval-state={approval.state}
      className={`border-b px-3 py-1.5 ${
        denied ? 'border-signal bg-error-surface' : 'border-amber bg-panel-alt'
      }`}
    >
      <p className={`type-mono-meta ${denied ? 'text-signal' : 'text-ink-800'}`}>
        {denied ? EDITOR_COPY.approvalDenied : EDITOR_COPY.approvalRequired} · {approval.tool}
      </p>
      {approval.message ? (
        <p className="type-body-l mt-0.5 text-ink-900">{approval.message}</p>
      ) : null}
      {onRequestApproval === undefined ? null : (
        <button
          type="button"
          className="type-label mt-1 rounded-[var(--radius-button)] border border-honey-500 bg-honey-300 px-2 py-1 text-ink-900 hover:bg-honey-400"
          onClick={() => onRequestApproval()}
        >
          {EDITOR_COPY.requestApproval}
        </button>
      )}
    </div>
  );
}

export function Editor({
  path,
  value,
  onChange,
  onSave,
  onRetry,
  onRequestApproval,
  status = 'ready',
  errorMessage = null,
  readOnly = false,
  readOnlyReason = null,
  language = null,
  dirty = false,
  approval = null,
  ariaLabel,
  className = '',
}: EditorProps) {
  const [cursorLine, setCursorLine] = useState(1);
  const gutterRef = useRef<HTMLDivElement | null>(null);
  const textareaRef = useRef<HTMLTextAreaElement | null>(null);
  const lineInfo = buildEditorLineNumbers(value);
  const totalLines = countEditorLines(value);
  const approvalLocked = approvalBlocksSave(approval);
  const effectiveReadOnly = readOnly || onChange === undefined;
  const canSave =
    onSave !== undefined &&
    status === 'ready' &&
    !effectiveReadOnly &&
    dirty &&
    !approvalLocked;
  const save = (): void => {
    if (!canSave) {
      return;
    }
    onSave?.(value);
  };
  const updateCursorLine = (element: HTMLTextAreaElement): void => {
    setCursorLine(editorLineForOffset(value, element.selectionStart ?? 0));
  };

  useEffect(() => {
    if (status !== 'ready' || typeof document === 'undefined') {
      return;
    }
    const gutter = gutterRef.current;
    const textarea = textareaRef.current;
    if (gutter !== null && textarea !== null) {
      gutter.scrollTop = textarea.scrollTop;
    }
  }, [status, value]);

  const onEditorKeyDown = (event: KeyboardEvent<HTMLTextAreaElement>): void => {
    if (editorShortcut(event.key, event) === 'save') {
      event.preventDefault();
      save();
    }
  };

  return (
    <section
      aria-label={`${EDITOR_COPY.section}: ${path}`}
      aria-busy={status === 'loading'}
      className={`flex min-h-0 flex-col overflow-hidden rounded-[var(--radius-card)] border border-hairline bg-panel shadow-[var(--shadow-e1)] ${className}`}
    >
      <header className="flex flex-wrap items-center gap-2 border-b border-hairline bg-panel-alt px-3 py-2">
        <div className="flex min-w-0 flex-1 items-baseline gap-2">
          <h2 className="type-label shrink-0 text-ink-900">{EDITOR_COPY.section}</h2>
          <span className="type-mono-code min-w-0 truncate text-ink-800" title={path}>
            {path}
          </span>
        </div>
        {language === null ? null : (
          <span className="type-mono-meta hidden shrink-0 text-meta sm:inline">{language}</span>
        )}
        {status === 'ready' ? (
          <span
            className={`type-mono-meta shrink-0 rounded-[var(--radius-pill)] border px-2 py-0.5 ${
              dirty ? 'border-amber bg-honey-100 text-ink-800' : 'border-hairline bg-panel text-ink-500'
            }`}
            data-dirty={dirty ? 'true' : 'false'}
          >
            {dirty ? EDITOR_COPY.modified : EDITOR_COPY.saved}
          </span>
        ) : null}
        {effectiveReadOnly ? (
          <span className="type-mono-meta shrink-0 rounded-[var(--radius-pill)] border border-hairline bg-recessed px-2 py-0.5 text-ink-800">
            {EDITOR_COPY.readOnly}
          </span>
        ) : null}
        <button
          type="button"
          className="type-label rounded-[var(--radius-button)] border border-honey-600 bg-honey-300 px-3 py-1 text-ink-900 hover:bg-honey-400 disabled:cursor-not-allowed disabled:opacity-50"
          disabled={!canSave}
          aria-label={`${EDITOR_COPY.save} ${path}`}
          onClick={() => save()}
        >
          {EDITOR_COPY.save}
        </button>
      </header>

      {approval === null ? null : (
        <ApprovalBanner approval={approval} onRequestApproval={onRequestApproval} />
      )}

      {status === 'idle' ? (
        <div className="flex flex-1 items-center justify-center p-6">
          <div className="max-w-[440px] rounded-[var(--radius-md)] border border-dashed border-hairline bg-panel-alt p-4">
            <p className="type-heading-h3 text-ink-900">{EDITOR_COPY.emptyTitle}</p>
            <p className="type-body-l mt-1 text-ink-500">{EDITOR_COPY.emptyBody}</p>
          </div>
        </div>
      ) : null}

      {status === 'loading' ? (
        <p role="status" className="type-mono-meta p-3 text-meta">
          {EDITOR_COPY.loading}
        </p>
      ) : null}

      {status === 'error' ? (
        <div role="alert" className="m-3 rounded-[var(--radius-md)] border border-signal bg-error-surface p-3">
          <p className="type-label text-signal">{EDITOR_COPY.error}</p>
          <p className="type-body-l mt-1 text-ink-900">
            {errorMessage ?? 'The file could not be read.'}
          </p>
          {onRetry === undefined ? null : (
            <button
              type="button"
              className="type-label mt-2 rounded-[var(--radius-button)] border border-signal px-3 py-1.5 text-signal"
              onClick={() => onRetry()}
            >
              {EDITOR_COPY.retry}
            </button>
          )}
        </div>
      ) : null}

      {status === 'ready' ? (
        <>
          <div className="grid min-h-[18rem] flex-1 grid-cols-[3rem_minmax(0,1fr)] overflow-hidden">
            <div
              ref={gutterRef}
              aria-hidden="true"
              className="type-mono-code select-none overflow-hidden border-r border-hairline bg-panel-alt py-2 text-right text-meta"
            >
              {lineInfo.numbers.map((line) => (
                <div
                  key={line}
                  className={`px-2 ${line === cursorLine ? 'bg-honey-100 text-ink-900' : 'text-ink-500'}`}
                >
                  {line}
                </div>
              ))}
            </div>
            <textarea
              ref={textareaRef}
              aria-label={ariaLabel ?? `${EDITOR_COPY.section}: ${path}`}
              className="type-mono-code min-h-[18rem] w-full resize-none overflow-auto bg-panel p-2 text-ink-900 outline-none"
              spellCheck={false}
              wrap="off"
              value={value}
              readOnly={effectiveReadOnly}
              onChange={(event) => onChange?.(event.target.value)}
              onScroll={(event) => {
                if (gutterRef.current !== null) {
                  gutterRef.current.scrollTop = event.currentTarget.scrollTop;
                }
              }}
              onClick={(event) => updateCursorLine(event.currentTarget)}
              onKeyUp={(event) => updateCursorLine(event.currentTarget)}
              onSelect={(event) => updateCursorLine(event.currentTarget)}
              onKeyDown={onEditorKeyDown}
            />
          </div>
          {effectiveReadOnly ? (
            <p className="type-mono-meta border-t border-hairline bg-recessed px-3 py-1 text-ink-800">
              {readOnlyReason ??
                (readOnly ? EDITOR_COPY.readOnly : 'No edit channel is connected.')}
            </p>
          ) : null}
          <footer className="flex flex-wrap items-center gap-x-3 gap-y-1 border-t border-hairline bg-panel-alt px-3 py-1.5 text-meta">
            <span className="type-mono-meta">
              {EDITOR_COPY.cursor} {cursorLine}
            </span>
            <span className="type-mono-meta">
              {totalLines} {EDITOR_COPY.lines}
            </span>
            <span className="type-mono-meta">
              {value.length} {EDITOR_COPY.characters}
            </span>
            {lineInfo.truncated ? (
              <span className="type-mono-meta text-amber">{EDITOR_COPY.lineLimit}</span>
            ) : null}
          </footer>
        </>
      ) : null}
    </section>
  );
}
