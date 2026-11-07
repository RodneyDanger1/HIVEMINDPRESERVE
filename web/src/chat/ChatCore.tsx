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
import { clientRefusal } from '../api/refusals';
import type { ChatMessage, ToastRefusal } from '../state/store';
import { MessageBody, TypingDots, WorkingBee } from './ChatMotion';

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

const STARTERS = [
  { label: 'Map this repository', prompt: 'Map the architecture of this repository and identify the highest-risk module.' },
  { label: 'Review an API change', prompt: 'Review the current API change for regressions and missing tests.' },
  { label: 'Explain a failure', prompt: 'Explain the last failure and propose the smallest safe fix.' },
] as const;

export function ChatCore() {
  const messages = useUiStore((state) => state.messages);
  const submitPrompt = useUiStore((state) => state.submitPrompt);
  const pending = useUiStore((state) => state.pendingCommands);
  const connection = useUiStore((state) => state.connection);
  const chat = useUiStore((state) => state.chat);
  const setChatVisible = useUiStore((state) => state.setChatVisible);
  const streamingRunId = useUiStore((state) => state.streamingRunId);
  const runStatus = useUiStore((state) => state.run?.status ?? null);
  const replaying = useUiStore((state) => state.history.replayGraph !== null);
  const runErrorCode = useUiStore((state) => state.run?.errorCode ?? null);
  const runRefusal = runErrorCode === null ? null : clientRefusal(runErrorCode);
  const reduced = useAppReducedMotion();
  const store = useUiStoreApi();
  const [draft, setDraft] = useState('');
  const listRef = useRef<HTMLDivElement | null>(null);
  const composerRef = useRef<HTMLTextAreaElement | null>(null);
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
        className="pointer-events-auto surface-panel flex h-[480px] w-[min(400px,92%)] -translate-y-8 flex-col rounded-[18px] p-3"
        style={{ boxShadow: 'var(--shadow-e2)', border: 'none', transition: reduced ? 'none' : 'box-shadow 160ms ease' }}
      >
        <header
          className="-mx-3 -mt-3 mb-2 flex h-11 items-center justify-between gap-2 rounded-t-[18px] px-3 text-white"
          style={{ background: 'linear-gradient(135deg, var(--color-ink-800), var(--color-honey-800))' }}
        >
          <div className="flex min-w-0 flex-1 items-center gap-2">
            <WorkingBee
              running={
                streamingRunId !== null ||
                runStatus === 'running' ||
                runStatus === 'thinking' ||
                runStatus === 'planning'
              }
            />
            <div className="truncate text-[14px] font-semibold leading-[18px]">{STRINGS.chat.title}</div>
          </div>
          <div className="flex shrink-0 items-center gap-1">
            <button
              type="button"
              className="type-mono-meta whitespace-nowrap rounded-[var(--radius-sm)] border border-white/25 px-2 py-0.5 text-[16px] leading-none text-white/90 hover:bg-white/10"
              onClick={() => setChatVisible(false)}
              aria-label={STRINGS.chat.closeAria}
            >
              ×
            </button>
          </div>
        </header>

        {runRefusal === null ? null : (
          <div
            role="alert"
            className="mt-2 rounded-[var(--radius-sm)] border border-signal bg-ink-900 px-2 py-1.5 text-canvas"
          >
            <div className="type-label text-honey-200">Run refused</div>
            <div className="type-mono-meta mt-0.5 text-honey-200">{runRefusal.code}</div>
            <div className="mt-1 text-[12.5px] leading-[17px]">{runRefusal.message}</div>
            <div className="type-mono-meta mt-1 text-honey-100/85">how_to_fix: {runRefusal.how_to_fix}</div>
          </div>
        )}

        <div
          ref={listRef}
          className="mt-2 min-h-0 flex-1 overflow-y-auto rounded-[var(--radius-sm)] bg-panel-alt px-2 py-2"
          aria-live="polite"
          aria-label={STRINGS.chat.aria}
        >
          {messages.length === 0 ? (
            <div>
              <p className="text-[12.5px] leading-[18px] text-ink-500">{STRINGS.chat.empty}</p>
              <div className="mt-3 flex flex-col gap-1.5">
                {STARTERS.map((starter) => (
                  <button
                    key={starter.label}
                    type="button"
                    className="type-label rounded-[8px] border border-hairline bg-panel px-2.5 py-1.5 text-left text-[10.5px] text-ink-800 hover:border-honey-400 hover:bg-honey-100"
                    onClick={() => {
                      setDraft(starter.prompt);
                      composerRef.current?.focus();
                    }}
                  >
                    {starter.label}
                  </button>
                ))}
              </div>
            </div>
          ) : (
            messages.map((message) => (
              <article
                key={message.id}
                className={
                  'chat-message mb-2 rounded-[10px] px-2 py-1.5 last:mb-0 ' +
                  (message.role === 'assistant' ? 'bg-honey-100' : 'bg-panel-alt')
                }
              >
                <div className="type-mono-meta text-meta">
                  {message.role === 'assistant' ? STRINGS.chat.assistantName : STRINGS.chat.userLabel}
                  {message.streaming ? ` · ${STRINGS.states.loading}` : ''}
                </div>
                <MessageBody text={message.text} />
                {message.streaming ? <span className="ml-0.5 inline-block h-[12px] w-[2px] animate-pulse bg-honey-500 align-middle" aria-hidden="true" /> : null}
              </article>
            ))
          )}
          {streamingRunId ? (
            <div className="type-mono-meta flex items-center gap-2 text-meta" role="status">
              {messages.some((message) => message.streaming && message.text.length > 0) ? null : (
                <TypingDots />
              )}
              {STRINGS.agentStates.streaming}
            </div>
          ) : null}
        </div>

        <form className="mt-2 flex h-14 items-center gap-2 rounded-[12px] border border-hairline bg-panel px-3" onSubmit={onSubmit}>
          <label className="sr-only" htmlFor="composer">
            {STRINGS.composer.placeholder}
          </label>
          <textarea
            id="composer"
            ref={composerRef}
            className="h-full min-w-0 flex-1 resize-none border-0 bg-transparent py-4 text-[13px] leading-[18px] text-ink-900 outline-none placeholder:text-ink-500 disabled:opacity-60"
            placeholder={STRINGS.composer.placeholder}
            value={draft}
            onChange={(event) => setDraft(event.target.value)}
            onKeyDown={(event) => {
              if (event.key === 'Enter' && !event.shiftKey) {
                event.preventDefault();
                void submit();
              }
            }}
            rows={1}
            disabled={connection !== 'open' || replaying}
          />
          <button
            type="submit"
            className="flex h-8 w-8 shrink-0 items-center justify-center rounded-full bg-honey-300 text-ink-900 hover:bg-honey-400 disabled:cursor-not-allowed disabled:opacity-50"
            aria-label={STRINGS.composer.submit}
            disabled={connection !== 'open' || replaying || submitting || draft.trim().length === 0}
          >
            <svg width="15" height="15" viewBox="0 0 20 20" aria-hidden="true">
              <path d="M3 10h13M11 4l6 6-6 6" fill="none" stroke="currentColor" strokeWidth="1.8" strokeLinecap="round" strokeLinejoin="round" />
            </svg>
          </button>
        </form>
        <p className="type-mono-meta mt-1 text-meta">{STRINGS.composer.hint}</p>
      </section>
    </div>
  );
}
