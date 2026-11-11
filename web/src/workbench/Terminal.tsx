/**
 * CAP-4 workbench — terminal output pane and command line.
 *
 * The pane renders the exact transcript lines the host supplies. It is
 * line-oriented on purpose: without the pinned xterm dependency it does not
 * claim ANSI/PTY emulation. When `@xterm/xterm` is installed the host can keep
 * this component for the accessible transcript and attach the terminal engine
 * for interactive keystrokes (see WORKBENCH_OPTIONAL_DEPENDENCIES).
 *
 * Commands are never executed here: `onSubmit` is the caller's execution path;
 * interrupt/clear are caller callbacks; the status chip is the supplied state.
 */
import { useEffect, useId, useRef, useState } from 'react';
import type { KeyboardEvent } from 'react';

export type TerminalStatus = 'idle' | 'connecting' | 'running' | 'closed' | 'error';
export type TerminalLineKind = 'stdout' | 'stderr' | 'system' | 'input' | 'exit';
export type TerminalApprovalState = 'required' | 'pending' | 'approved' | 'denied';

export interface TerminalLine {
  readonly id?: string;
  readonly kind?: TerminalLineKind;
  readonly text: string;
  /** ISO-8601 timestamp from the runtime; absent stays absent. */
  readonly at?: string | null;
}

export interface TerminalApproval {
  readonly state: TerminalApprovalState;
  readonly tool: string;
  readonly message?: string | null;
}

export interface TerminalProps {
  readonly lines: readonly TerminalLine[];
  readonly status?: TerminalStatus;
  readonly cwd?: string | null;
  readonly shell?: string | null;
  readonly errorMessage?: string | null;
  readonly approval?: TerminalApproval | null;
  readonly disabled?: boolean;
  readonly onSubmit?: (command: string) => void;
  readonly onInterrupt?: () => void;
  readonly onClear?: () => void;
  readonly onRequestApproval?: () => void;
  readonly onRetry?: () => void;
  readonly className?: string;
  readonly inputLabel?: string;
  readonly placeholder?: string;
}

export const TERMINAL_COPY = {
  section: 'Terminal',
  output: 'Terminal output',
  empty: 'No output yet.',
  emptyHint: 'Commands and process output appear here as the runtime reports them.',
  connecting: 'Connecting to the shell…',
  running: 'Running',
  idle: 'Ready',
  closed: 'Closed',
  error: 'Terminal error',
  retry: 'Retry',
  stop: 'Stop',
  clear: 'Clear',
  command: 'Command',
  run: 'Run',
  placeholder: 'Type a command and press Enter',
  readOnly: 'Terminal is read-only; no command channel is connected.',
  approvalRequired: 'Approval required before running a command.',
  approvalPending: 'Waiting for approval before running a command.',
  approvalApproved: 'Command execution approved.',
  approvalDenied: 'Command execution was denied.',
  requestApproval: 'Request approval',
  interruptHint: 'Ctrl+C interrupts a running command when the input has no selection.',
  stdout: 'stdout',
  stderr: 'stderr',
  system: 'system',
  input: 'input',
  exit: 'exit',
} as const;

export function terminalStatusLabel(status: TerminalStatus): string {
  switch (status) {
    case 'idle':
      return TERMINAL_COPY.idle;
    case 'connecting':
      return TERMINAL_COPY.connecting;
    case 'running':
      return TERMINAL_COPY.running;
    case 'closed':
      return TERMINAL_COPY.closed;
    case 'error':
      return TERMINAL_COPY.error;
  }
}

export function terminalStatusTone(status: TerminalStatus): string {
  switch (status) {
    case 'running':
      return 'border-honey-500 bg-honey-100 text-ink-900';
    case 'error':
      return 'border-signal bg-error-surface text-signal';
    case 'closed':
      return 'border-hairline bg-recessed text-ink-500';
    case 'connecting':
      return 'border-amber bg-panel-alt text-ink-800';
    case 'idle':
      return 'border-hairline bg-panel text-ink-700';
  }
}

export function terminalLineTone(kind: TerminalLineKind | undefined): string {
  switch (kind) {
    case 'stderr':
      return 'text-signal';
    case 'system':
      return 'text-ink-500 italic';
    case 'input':
      return 'text-honey-700';
    case 'exit':
      return 'text-amber';
    case 'stdout':
    case undefined:
      return 'text-ink-900';
  }
}

export function terminalLineLabel(kind: TerminalLineKind | undefined): string {
  switch (kind) {
    case 'stderr':
      return TERMINAL_COPY.stderr;
    case 'system':
      return TERMINAL_COPY.system;
    case 'input':
      return TERMINAL_COPY.input;
    case 'exit':
      return TERMINAL_COPY.exit;
    case 'stdout':
    case undefined:
      return TERMINAL_COPY.stdout;
  }
}

/** UTC HH:MM:SS for a real timestamp; an unparseable value stays unknown. */
export function formatTerminalTime(value: string | null | undefined): string {
  if (value === null || value === undefined || value.length === 0) {
    return '—';
  }
  const parsed = Date.parse(value);
  if (!Number.isFinite(parsed)) {
    return '—';
  }
  return new Date(parsed).toISOString().slice(11, 19);
}

/** The prompt is a UI label, not fabricated shell output. */
export function terminalPrompt(cwd: string | null | undefined): string {
  return cwd === null || cwd === undefined || cwd.length === 0 ? '›' : `${cwd} ›`;
}

export interface TerminalInputAvailability {
  readonly status: TerminalStatus;
  readonly hasSubmit: boolean;
  readonly disabled?: boolean;
  readonly approval?: TerminalApproval | null;
}

/**
 * Input is enabled only for a live/idle transcript with a submit callback and no
 * unresolved or denied approval gate. Closed/error/connecting cannot submit.
 */
export function canSubmitTerminal(input: TerminalInputAvailability): boolean {
  if (input.disabled === true || !input.hasSubmit) {
    return false;
  }
  if (input.status !== 'idle' && input.status !== 'running') {
    return false;
  }
  const approval = input.approval;
  if (approval === null || approval === undefined) {
    return true;
  }
  return approval.state === 'approved';
}

function TerminalApprovalBanner({
  approval,
  onRequestApproval,
}: {
  approval: TerminalApproval;
  onRequestApproval?: () => void;
}) {
  if (approval.state === 'approved') {
    return (
      <p
        role="status"
        data-approval-state={approval.state}
        className="type-mono-meta border-b border-verify bg-panel-alt px-3 py-1.5 text-verify"
      >
        {TERMINAL_COPY.approvalApproved} · {approval.tool}
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
        {TERMINAL_COPY.approvalPending} · {approval.tool}
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
        {denied ? TERMINAL_COPY.approvalDenied : TERMINAL_COPY.approvalRequired} · {approval.tool}
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
          {TERMINAL_COPY.requestApproval}
        </button>
      )}
    </div>
  );
}

export function Terminal({
  lines,
  status = 'idle',
  cwd = null,
  shell = null,
  errorMessage = null,
  approval = null,
  disabled = false,
  onSubmit,
  onInterrupt,
  onClear,
  onRequestApproval,
  onRetry,
  className = '',
  inputLabel = TERMINAL_COPY.command,
  placeholder = TERMINAL_COPY.placeholder,
}: TerminalProps) {
  const [command, setCommand] = useState('');
  const outputRef = useRef<HTMLDivElement | null>(null);
  const commandId = useId().replace(/[^a-zA-Z0-9_-]/g, '');
  const showTimes = lines.some((line) => line.at !== undefined && line.at !== null && line.at.length > 0);
  const canSubmit = canSubmitTerminal({
    status,
    hasSubmit: onSubmit !== undefined,
    disabled,
    approval,
  });

  useEffect(() => {
    const output = outputRef.current;
    if (output !== null) {
      output.scrollTop = output.scrollHeight;
    }
  }, [lines]);

  const submit = (): void => {
    if (!canSubmit || command.trim().length === 0) {
      return;
    }
    onSubmit?.(command);
    setCommand('');
  };

  const onInputKeyDown = (event: KeyboardEvent<HTMLInputElement>): void => {
    if (event.ctrlKey && event.key.toLowerCase() === 'c') {
      const input = event.currentTarget;
      const hasSelection = input.selectionStart !== input.selectionEnd;
      if (!hasSelection && status === 'running' && onInterrupt !== undefined) {
        event.preventDefault();
        onInterrupt();
      }
      return;
    }
    if (event.ctrlKey && event.key.toLowerCase() === 'l' && onClear !== undefined) {
      event.preventDefault();
      onClear();
    }
  };

  return (
    <section
      aria-label={TERMINAL_COPY.section}
      aria-busy={status === 'connecting'}
      className={`flex min-h-0 flex-col overflow-hidden rounded-[var(--radius-card)] border border-hairline bg-ink-900 shadow-[var(--shadow-e2)] ${className}`}
    >
      <header className="flex flex-wrap items-center gap-2 border-b border-ink-700 bg-ink-800 px-3 py-2 text-canvas">
        <div className="flex min-w-0 flex-1 items-baseline gap-2">
          <h2 className="type-label shrink-0 text-honey-300">{TERMINAL_COPY.section}</h2>
          <span className="type-mono-meta min-w-0 truncate text-canvas/70" title={cwd ?? undefined}>
            {terminalPrompt(cwd)}
          </span>
          {shell === null ? null : (
            <span className="type-mono-meta hidden shrink-0 text-canvas/60 sm:inline">{shell}</span>
          )}
        </div>
        <span
          className={`type-mono-meta shrink-0 rounded-[var(--radius-pill)] border px-2 py-0.5 ${terminalStatusTone(status)}`}
          data-terminal-status={status}
        >
          {terminalStatusLabel(status)}
        </span>
        {status === 'running' && onInterrupt !== undefined ? (
          <button
            type="button"
            className="type-label rounded-[var(--radius-sm)] border border-signal px-2 py-1 text-canvas hover:bg-signal/20"
            onClick={() => onInterrupt()}
          >
            {TERMINAL_COPY.stop}
          </button>
        ) : null}
        {onClear !== undefined && lines.length > 0 ? (
          <button
            type="button"
            className="type-label rounded-[var(--radius-sm)] border border-ink-700 px-2 py-1 text-canvas/80 hover:bg-ink-700"
            onClick={() => onClear()}
          >
            {TERMINAL_COPY.clear}
          </button>
        ) : null}
      </header>

      {approval === null ? null : (
        <TerminalApprovalBanner approval={approval} onRequestApproval={onRequestApproval} />
      )}

      {status === 'error' ? (
        <div role="alert" className="m-3 rounded-[var(--radius-md)] border border-signal bg-error-surface p-3">
          <p className="type-label text-signal">{TERMINAL_COPY.error}</p>
          <p className="type-body-l mt-1 text-ink-900">
            {errorMessage ?? 'The terminal reported an error.'}
          </p>
          {onRetry === undefined ? null : (
            <button
              type="button"
              className="type-label mt-2 rounded-[var(--radius-button)] border border-signal px-3 py-1.5 text-signal"
              onClick={() => onRetry()}
            >
              {TERMINAL_COPY.retry}
            </button>
          )}
        </div>
      ) : null}

      {status === 'connecting' ? (
        <p role="status" className="type-mono-meta p-3 text-canvas/70">
          {TERMINAL_COPY.connecting}
        </p>
      ) : null}

      <div
        ref={outputRef}
        role="log"
        aria-label={TERMINAL_COPY.output}
        aria-live="polite"
        aria-relevant="additions text"
        className="min-h-[12rem] flex-1 overflow-auto bg-ink-900 p-3"
      >
        {lines.length === 0 ? (
          <div className="type-mono-meta text-canvas/60">
            <p>{TERMINAL_COPY.empty}</p>
            <p className="mt-1 text-canvas/40">{TERMINAL_COPY.emptyHint}</p>
          </div>
        ) : (
          <ol className="m-0 list-none p-0">
            {lines.map((line, index) => (
              <li
                key={line.id ?? index}
                data-kind={line.kind ?? 'stdout'}
                aria-label={`${terminalLineLabel(line.kind)}: ${line.text}`}
                className={`type-mono-code flex gap-2 whitespace-pre-wrap break-words ${terminalLineTone(line.kind)}`}
              >
                {showTimes ? (
                  <span className="type-mono-meta w-16 shrink-0 select-none pt-0.5 text-right text-canvas/40">
                    {formatTerminalTime(line.at)}
                  </span>
                ) : null}
                <span className="min-w-0 flex-1">{line.text.length === 0 ? '\u00a0' : line.text}</span>
              </li>
            ))}
          </ol>
        )}
      </div>

      {onSubmit === undefined ? (
        <p className="type-mono-meta border-t border-ink-700 bg-ink-800 px-3 py-1.5 text-canvas/60">
          {TERMINAL_COPY.readOnly}
        </p>
      ) : (
        <form
          className="flex flex-wrap items-center gap-2 border-t border-ink-700 bg-ink-800 px-2 py-2"
          onSubmit={(event) => {
            event.preventDefault();
            submit();
          }}
        >
          <label className="sr-only" htmlFor={commandId}>
            {inputLabel}
          </label>
          <input
            id={commandId}
            aria-label={inputLabel}
            className="type-mono-code min-w-0 flex-1 rounded-[var(--radius-sm)] border border-ink-700 bg-ink-900 px-2 py-1 text-canvas placeholder:text-canvas/40 disabled:cursor-not-allowed disabled:opacity-50"
            placeholder={placeholder}
            value={command}
            disabled={!canSubmit}
            onChange={(event) => setCommand(event.target.value)}
            onKeyDown={onInputKeyDown}
          />
          <button
            type="submit"
            className="type-label rounded-[var(--radius-sm)] border border-honey-600 bg-honey-300 px-3 py-1 text-ink-900 hover:bg-honey-400 disabled:cursor-not-allowed disabled:opacity-50"
            disabled={!canSubmit || command.trim().length === 0}
          >
            {TERMINAL_COPY.run}
          </button>
          <span className="type-mono-meta hidden w-full text-canvas/40 lg:block">
            {TERMINAL_COPY.interruptHint}
          </span>
        </form>
      )}
    </section>
  );
}
