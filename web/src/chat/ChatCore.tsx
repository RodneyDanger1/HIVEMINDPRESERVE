/**
 * S02 pinned chat core + S03 composer + S27 streaming reply. The core hive is
 * a fixed overlay: panning/zooming the canvas never moves it (PDF:109-115).
 * Submits only SUBMIT_PROMPT; replies stream from REPLY_DELTA events; every
 * refusal is named by ToastRegion, never silently dropped.
 */
import { useEffect, useMemo, useRef, useState, type FormEvent } from 'react';
import { useUiStore, useUiStoreApi } from '../state/context';
import { STRINGS } from '../state/strings';
import { isTerminal } from '../state/agentState';
import { useAppReducedMotion } from '../state/useAppReducedMotion';
import { validatePromptText } from '../api/contract';
import { clientRefusal } from '../api/refusals';
import { CONTROL_INTENTS } from '../state/chatIntents';
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
  const cancelRun = useUiStore((state) => state.cancelRun);
  const replaying = useUiStore((state) => state.history.replayGraph !== null);
  const runErrorCode = useUiStore((state) => state.run?.errorCode ?? null);
  const keyPresent = useUiStore((state) => state.settings.keyPresent);
  const toolActivity = useUiStore((state) => state.toolActivity);
  const tasks = useUiStore((state) => state.tasks);
  const activeAgents = Object.values(tasks).filter((task) => !isTerminal(task.state)).length;
  const liveLabel =
    connection !== 'open'
      ? 'offline'
      : runStatus === 'planning'
        ? 'planning'
        : streamingRunId !== null || runStatus === 'running' || runStatus === 'thinking'
          ? `${activeAgents} active`
          : 'idle';
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
      // An empty thread must show its starters from the top. Scrolling an
      // empty log to its measured height hid the first starter under the
      // header once the root-control block was added.
      node.scrollTop = messages.length === 0 ? 0 : node.scrollHeight;
    }
  }, [messages]);

  if (!chat.visible) {
    return (
      <div className="pointer-events-none absolute inset-x-0 top-4 z-30 flex justify-center">
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
    if (keyPresent === false) {
      // Do not create a doomed run just to have the server fail E_KEY_MISSING.
      // Keep the typed draft and surface the same named refusal the server uses.
      const refusal = clientRefusal('E_KEY_MISSING');
      const current = store.getState();
      store.setState({
        refusals: [
          ...current.refusals,
          { id: crypto.randomUUID(), refusal, at: Date.now() },
        ],
      });
      return;
    }
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
        className="chat-core pointer-events-auto surface-panel flex h-[560px] max-h-[calc(100vh-96px)] w-[min(400px,92%)] -translate-y-8 flex-col rounded-[18px] p-3"
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
            <span
              className="type-mono-meta flex items-center gap-1 whitespace-nowrap px-1 text-[11px] text-honey-100/80"
              role="status"
              aria-live="polite"
            >
              <span
                className={
                  'h-1.5 w-1.5 rounded-full ' +
                  (connection !== 'open'
                    ? 'bg-signal'
                    : streamingRunId !== null || runStatus === 'running'
                      ? 'motion-hive-live-dot bg-honey-300'
                      : 'bg-honey-100/70')
                }
              />
              {liveLabel}
            </span>
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
              <div className="mt-6 flex flex-col items-center gap-1.5 opacity-70" aria-hidden="true">
                <svg width="46" height="46" viewBox="0 0 46 46">
                  <path d="M23 4 39 13v18L23 40 7 31V13Z" fill="none" stroke="var(--color-honey-400)" strokeWidth="1.2" />
                  <path d="M23 13 32 18v10l-9 5-9-5V18Z" fill="var(--color-honey-100)" stroke="var(--color-honey-500)" strokeWidth="1" />
                  <circle cx="23" cy="22" r="3" fill="var(--color-honey-500)" />
                </svg>
                <span className="type-mono-meta text-[11px] tracking-[0.14em] text-meta">READY · NO RUN YET</span>
              </div>
            </div>
          ) : (
            messages.map((message) => (
              <article
                key={message.id}
                className={
                  'chat-message mb-2 rounded-[10px] px-2 py-1.5 last:mb-0 ' +
                  (message.role === 'assistant' ? 'bg-honey-100' : 'bg-panel-alt') +
                  (message.streaming ? ' chat-streaming' : '')
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
          {toolActivity.length > 0 ? (
            <section
              aria-label="Tool activity in chat"
              className="mt-2 rounded-[8px] border border-hairline bg-panel px-2 py-1.5"
            >
              <div className="type-mono-meta flex items-center justify-between text-[11px] text-meta">
                <span>ACTIONS</span>
                <span>{toolActivity.length}</span>
              </div>
              <ul className="mt-1 space-y-0.5">
                {[...toolActivity]
                  .sort((a, b) => a.turn - b.turn || a.toolCallId.localeCompare(b.toolCallId))
                  .slice(-4)
                  .map((entry) => (
                    <li key={entry.toolCallId} className="type-mono-meta flex items-center gap-1.5 text-[11px]">
                      <span className="text-ink-800">{entry.tool}</span>
                      <span
                        className={
                          entry.outcome === 'ok'
                            ? 'text-verify'
                            : entry.outcome === 'pending'
                              ? 'text-meta'
                              : 'text-signal'
                        }
                      >
                        {entry.outcome}
                      </span>
                      {entry.text === undefined || entry.text.length === 0 ? null : (
                        <span className="min-w-0 flex-1 truncate text-meta">{entry.text}</span>
                      )}
                    </li>
                  ))}
              </ul>
            </section>
          ) : null}
          {streamingRunId ? (
            <div className="type-mono-meta flex items-center gap-2 text-meta" role="status">
              {messages.some((message) => message.streaming && message.text.length > 0) ? null : (
                <TypingDots />
              )}
              {STRINGS.agentStates.streaming}
            </div>
          ) : null}
        </div>

        <div className="mt-1 flex flex-wrap items-center gap-1">
          <span className="type-mono-meta text-[10px] tracking-[0.12em] text-meta">ROOT CONTROL</span>
          {CONTROL_INTENTS.map((intent) => (
            <button
              key={intent.id}
              type="button"
              className="type-label rounded-[8px] border border-hairline bg-panel px-1.5 py-0.5 text-[10px] text-ink-800 hover:border-honey-400 hover:bg-honey-100 disabled:opacity-50"
              data-control-intent={intent.id}
              disabled={connection !== 'open' || replaying}
              title={`Fill the composer with a SUBMIT_PROMPT asking the HyperAgent root control for ${intent.label.toLowerCase()}. The server decides; no spawn is claimed here.`}
              onClick={() => {
                setDraft(intent.prompt);
                composerRef.current?.focus();
              }}
            >
              {intent.label}
            </button>
          ))}
        </div>
        <p className="type-mono-meta mt-0.5 text-[10px] leading-[14px] text-meta">
          These fill a prompt, not a command. The server&apos;s bounded HyperAgent root-control
          turn (when enabled) decides whether to call spawn_remote_agent / spawn_daemon; success
          is shown only by the server projection.
        </p>

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
            type="button"
            className="flex h-8 w-8 shrink-0 items-center justify-center rounded-full border border-hairline text-ink-800 hover:bg-honey-100 disabled:cursor-not-allowed disabled:opacity-50"
            aria-label="Stop the active run"
            disabled={connection !== 'open' || replaying || (streamingRunId === null && runStatus !== 'running')}
            onClick={() => {
              void cancelRun();
            }}
          >
            <svg width="11" height="11" viewBox="0 0 12 12" aria-hidden="true">
              <rect x="2.5" y="2.5" width="7" height="7" rx="1" fill="currentColor" />
            </svg>
          </button>
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
