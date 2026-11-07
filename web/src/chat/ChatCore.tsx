/**
 * S02 pinned chat core + S03 composer + S27 streaming reply. The core hive is
 * a fixed overlay: panning/zooming the canvas never moves it (PDF:109-115).
 * Submits only SUBMIT_PROMPT; replies stream from REPLY_DELTA events; every
 * refusal is named by ToastRegion, never silently dropped.
 */
import { useEffect, useMemo, useRef, useState, type FormEvent } from 'react';
import { useUiStore, useUiStoreApi } from '../state/context';
import { STRINGS } from '../state/strings';
import { useAppReducedMotion } from '../state/useAppReducedMotion';
import { validatePromptText } from '../api/contract';
import type { ChatMessage, ToastRefusal } from '../state/store';

/** Baselines captured immediately before SUBMIT_PROMPT is dispatched. */
export interface PromptSubmissionBaseline {
  readonly messageIds: ReadonlySet<string>;
  readonly refusalIds: ReadonlySet<string>;
  readonly startedAt: number;
}

export function promptSubmissionBaseline(
  messages: readonly ChatMessage[],
  refusals: readonly ToastRefusal[],
  startedAt: number,
): PromptSubmissionBaseline {
  return {
    messageIds: new Set(messages.map((message) => message.id)),
    refusalIds: new Set(refusals.map((entry) => entry.id)),
    startedAt,
  };
}

/** The refusal the store published while this submission was in flight, if any. */
export function findSubmissionRefusal(
  baseline: PromptSubmissionBaseline,
  refusals: readonly ToastRefusal[],
): ToastRefusal | null {
  return (
    refusals.find(
      (entry) => !baseline.refusalIds.has(entry.id) && entry.at >= baseline.startedAt,
    ) ?? null
  );
}

/** The optimistic user row this submission appended, if it is still present. */
export function findOptimisticPrompt(
  baseline: PromptSubmissionBaseline,
  messages: readonly ChatMessage[],
  text: string,
): ChatMessage | null {
  return (
    messages.find(
      (message) =>
        message.role === 'user' && !baseline.messageIds.has(message.id) && message.text === text,
    ) ?? null
  );
}

/** ACK: clear only the sent text, never text typed while the command was in flight. */
export function draftAfterAck(current: string, submitted: string): string {
  return current === submitted ? '' : current;
}

/** Refusal: restore the submitted text unless the operator already typed a replacement. */
export function draftAfterRefusal(current: string, submitted: string): string {
  return current.trim().length === 0 ? submitted : current;
}

export function ChatCore() {
  const messages = useUiStore((state) => state.messages);
  const submitPrompt = useUiStore((state) => state.submitPrompt);
  const pending = useUiStore((state) => state.pendingCommands);
  const connection = useUiStore((state) => state.connection);
  const chat = useUiStore((state) => state.chat);
  const pinChat = useUiStore((state) => state.pinChat);
  const setChatVisible = useUiStore((state) => state.setChatVisible);
  const streamingRunId = useUiStore((state) => state.streamingRunId);
  const runErrorCode = useUiStore((state) => state.run?.errorCode ?? null);
  const reduced = useAppReducedMotion();
  const store = useUiStoreApi();
  const [draft, setDraft] = useState('');
  const listRef = useRef<HTMLDivElement | null>(null);
  // Synchronous double-submit guard: the derived `submitting` updates on render,
  // so two fast Enter presses could otherwise both pass the disabled check.
  const inFlightRef = useRef(false);

  const submitting = useMemo(
    () => Object.values(pending).some((entry) => entry.name === 'SUBMIT_PROMPT'),
    [pending],
  );

  useEffect(() => {
    const node = listRef.current;
    if (node) {
      node.scrollTop = node.scrollHeight;
    }
  }, [messages]);

  if (!chat.visible) {
    return (
      <div className="pointer-events-none absolute inset-x-0 top-4 flex justify-center">
        <button
          type="button"
          className="pointer-events-auto surface-panel type-label rounded-[var(--radius-pill)] px-4 py-2 text-ink-800"
          onClick={() => setChatVisible(true)}
        >
          {STRINGS.chat.reopen}
        </button>
      </div>
    );
  }

  /**
   * Validate -> submit -> ACK/refusal settlement. The draft is never cleared
   * before the command is acknowledged; on a named refusal the optimistic user
   * message the store appended is retracted and the typed prompt stays in the
   * composer (M-018: no silent loss).
   */
  const submit = async (): Promise<void> => {
    if (inFlightRef.current) {
      return;
    }
    const text = draft;
    if (validatePromptText(text) !== null) {
      // The store validates before appending and raises the named refusal
      // toast; this call therefore sends nothing and renders no user message.
      await submitPrompt(text);
      return;
    }
    const before = store.getState();
    const baseline = promptSubmissionBaseline(before.messages, before.refusals, Date.now());
    inFlightRef.current = true;
    try {
      await submitPrompt(text);
    } finally {
      inFlightRef.current = false;
    }
    const after = store.getState();
    // submitPrompt returns void (store-owned API), so the ACK outcome is read
    // from the refusal the store publishes for this command: server ERROR
    // frames land before the send promise settles, and local refusals are
    // pushed by the same path. A store API that returns the ACK outcome would
    // replace this observation without changing the rollback below.
    const refusal = findSubmissionRefusal(baseline, after.refusals);
    if (refusal === null) {
      setDraft((current) => draftAfterAck(current, text));
      return;
    }
    // Refused: keep/restore the draft and retract exactly the optimistic row.
    setDraft((current) => draftAfterRefusal(current, text));
    const optimistic = findOptimisticPrompt(baseline, after.messages, text);
    if (optimistic) {
      store.setState({
        messages: store.getState().messages.filter((message) => message.id !== optimistic.id),
      });
    }
  };

  const onSubmit = (event: FormEvent<HTMLFormElement>): void => {
    event.preventDefault();
    void submit();
  };

  return (
    <div className="pointer-events-none absolute inset-0 flex items-center justify-center p-4">
      <section
        aria-label={STRINGS.chat.aria}
        className="pointer-events-auto surface-panel w-[min(600px,92%)] rounded-[var(--radius-card)] p-3"
        style={{ boxShadow: 'var(--shadow-e3)', transition: reduced ? 'none' : 'box-shadow 160ms ease' }}
      >
        <header className="flex items-center justify-between gap-2">
          <div>
            <div className="type-heading-h3 text-ink-900">{STRINGS.chat.title}</div>
            <div className="type-mono-meta text-meta">
              {chat.pinned ? STRINGS.chat.pinned : STRINGS.chat.pin} ·{' '}
              {connection === 'open' ? STRINGS.connection.open : STRINGS.connection[connection]}
            </div>
          </div>
          <div className="flex items-center gap-1">
            <button
              type="button"
              className="type-mono-meta rounded-[var(--radius-sm)] border border-hairline px-2 py-1 text-ink-800 hover:bg-honey-100"
              onClick={() => {
                void pinChat(!chat.pinned);
              }}
              aria-pressed={chat.pinned}
            >
              {chat.pinned ? STRINGS.chat.unpin : STRINGS.chat.pin}
            </button>
            <button
              type="button"
              className="type-mono-meta rounded-[var(--radius-sm)] border border-hairline px-2 py-1 text-ink-800 hover:bg-honey-100"
              onClick={() => setChatVisible(false)}
              aria-label={STRINGS.chat.closeAria}
            >
              {STRINGS.controls.close}
            </button>
          </div>
        </header>

        {runErrorCode === null ? null : (
          <p
            role="alert"
            className="type-mono-meta mt-2 rounded-[var(--radius-sm)] border border-signal bg-recessed px-2 py-1 text-signal"
          >
            Run refused: {runErrorCode}
          </p>
        )}

        <div
          ref={listRef}
          className="mt-2 max-h-[42vh] min-h-[72px] overflow-y-auto rounded-[var(--radius-sm)] bg-panel-alt px-2 py-2"
          aria-live="polite"
          aria-label={STRINGS.chat.aria}
        >
          {messages.length === 0 ? (
            <p className="type-body-l text-ink-500">{STRINGS.chat.empty}</p>
          ) : (
            messages.map((message) => (
              <article key={message.id} className="mb-2 last:mb-0">
                <div className="type-mono-meta text-meta">
                  {message.role === 'assistant' ? STRINGS.chat.assistantName : STRINGS.chat.userLabel}
                  {message.streaming ? ` · ${STRINGS.states.loading}` : ''}
                </div>
                <div className="type-body-l whitespace-pre-wrap text-ink-900">{message.text}</div>
              </article>
            ))
          )}
          {streamingRunId ? (
            <div className="type-mono-meta text-meta" role="status">
              {STRINGS.agentStates.streaming}
            </div>
          ) : null}
        </div>

        <form className="mt-2 flex items-start gap-2" onSubmit={onSubmit}>
          <label className="sr-only" htmlFor="composer">
            {STRINGS.composer.placeholder}
          </label>
          <textarea
            id="composer"
            className="type-mono-code min-h-[44px] min-w-0 flex-1 resize-y rounded-[var(--radius-md)] border border-hairline bg-panel-alt px-3 py-2 text-ink-900"
            placeholder={STRINGS.composer.placeholder}
            value={draft}
            onChange={(event) => setDraft(event.target.value)}
            onKeyDown={(event) => {
              if (event.key === 'Enter' && !event.shiftKey) {
                event.preventDefault();
                void submit();
              }
            }}
            rows={2}
            disabled={connection !== 'open'}
          />
          <button
            type="submit"
            className="type-label rounded-[var(--radius-button)] bg-honey-300 px-4 py-2 text-ink-900 hover:bg-honey-400 disabled:cursor-not-allowed disabled:opacity-60"
            disabled={connection !== 'open' || submitting || draft.trim().length === 0}
          >
            {submitting ? STRINGS.states.loading : STRINGS.composer.submit}
          </button>
        </form>
        <p className="type-mono-meta mt-1 text-meta">{STRINGS.composer.hint}</p>
      </section>
    </div>
  );
}
