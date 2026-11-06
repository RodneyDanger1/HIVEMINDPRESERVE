/**
 * S02 pinned chat core + S03 composer + S27 streaming reply. The core hive is
 * a fixed overlay: panning/zooming the canvas never moves it (PDF:109-115).
 * Submits only SUBMIT_PROMPT; replies stream from REPLY_DELTA events; every
 * refusal is named by ToastRegion, never silently dropped.
 */
import { useEffect, useMemo, useRef, useState, type FormEvent } from 'react';
import { useUiStore } from '../state/context';
import { STRINGS } from '../state/strings';
import { useAppReducedMotion } from '../state/useAppReducedMotion';

export function ChatCore() {
  const messages = useUiStore((state) => state.messages);
  const submitPrompt = useUiStore((state) => state.submitPrompt);
  const pending = useUiStore((state) => state.pendingCommands);
  const connection = useUiStore((state) => state.connection);
  const chat = useUiStore((state) => state.chat);
  const pinChat = useUiStore((state) => state.pinChat);
  const setChatVisible = useUiStore((state) => state.setChatVisible);
  const streamingRunId = useUiStore((state) => state.streamingRunId);
  const reduced = useAppReducedMotion();
  const [draft, setDraft] = useState('');
  const listRef = useRef<HTMLDivElement | null>(null);

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

  const onSubmit = (event: FormEvent<HTMLFormElement>): void => {
    event.preventDefault();
    const text = draft;
    setDraft('');
    void submitPrompt(text);
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
                const text = draft;
                setDraft('');
                void submitPrompt(text);
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
