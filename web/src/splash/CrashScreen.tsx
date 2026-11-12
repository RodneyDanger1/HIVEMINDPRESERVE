/**
 * Crash surface: a render error anywhere in the app shell must produce a named,
 * readable screen with a recovery action, never a blank window.
 *
 * The boundary logs only the error message and the React component stack. Run
 * state, prompts, and provider material are never logged or rendered here.
 */
import { Component, type ErrorInfo, type ReactNode } from 'react';
import { STRINGS } from '../state/strings';

export interface CrashScreenProps {
  readonly message: string;
  readonly onReload?: () => void;
}

export function reloadInterface(): void {
  if (typeof window !== 'undefined') {
    window.location.reload();
  }
}

/** The named refusal screen shown when the interface itself fails to render. */
export function CrashScreen({ message, onReload = reloadInterface }: CrashScreenProps) {
  return (
    <div
      className="flex h-full min-h-0 w-full flex-col items-center justify-center overflow-y-auto bg-canvas p-6 text-ink-900"
      role="alert"
      aria-label={STRINGS.crash.title}
      data-testid="ui-crash-screen"
    >
      <div className="surface-panel w-full max-w-[560px] rounded-[var(--radius-lg)] p-5">
        <div className="type-mono-meta text-[11px] tracking-[0.14em] text-signal">UI_CRASH</div>
        <h1 className="mt-1 font-display text-[26px] leading-tight text-ink-900">
          {STRINGS.crash.title}
        </h1>
        <p className="mt-2 text-[12.5px] leading-[18px] text-ink-500">{STRINGS.crash.body}</p>
        <div className="mt-3 rounded-[var(--radius-sm)] border border-hairline bg-panel-alt p-2">
          <div className="type-mono-meta text-[11px] text-meta">{STRINGS.crash.detailLabel}</div>
          <p className="type-mono-code mt-1 break-words text-[12px] text-ink-800">{message}</p>
        </div>
        <button
          type="button"
          className="type-label mt-4 h-9 rounded-[var(--radius-sm)] border border-honey-500 bg-honey-300 px-3 font-semibold text-ink-900 hover:bg-honey-400"
          onClick={onReload}
        >
          {STRINGS.crash.reload}
        </button>
        <p className="type-mono-meta mt-3 text-[11px] leading-[16px] text-meta">{STRINGS.crash.hint}</p>
      </div>
    </div>
  );
}

interface AppErrorBoundaryState {
  readonly message: string | null;
}

/**
 * Top-level boundary. A child render error is caught here so the operator sees
 * the crash screen instead of an empty document.
 */
export class AppErrorBoundary extends Component<{ readonly children: ReactNode }, AppErrorBoundaryState> {
  override state: AppErrorBoundaryState = { message: null };

  static getDerivedStateFromError(error: unknown): AppErrorBoundaryState {
    return { message: describeError(error) };
  }

  override componentDidCatch(error: unknown, info: ErrorInfo): void {
    // Message + component stack only: no state, no props, no provider material.
    console.error('HiveMind UI crashed:', describeError(error), info.componentStack ?? '');
  }

  override render(): ReactNode {
    const message = this.state.message;
    if (message !== null) {
      return (
        <CrashScreen
          message={message}
          onReload={() => {
            this.setState({ message: null });
            reloadInterface();
          }}
        />
      );
    }
    return this.props.children;
  }
}

export function describeError(error: unknown): string {
  if (error instanceof Error && error.message.length > 0) {
    return error.message;
  }
  if (typeof error === 'string' && error.length > 0) {
    return error;
  }
  return 'Unknown render error';
}
