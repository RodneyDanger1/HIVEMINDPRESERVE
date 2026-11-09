/**
 * Read-only persisted transcript for the selected/live run. History hydration
 * already replaces `messages` with server rows; this surface renders them and
 * never invents a missing message.
 */
import { useUiStore } from '../state/context';
import { EmptyState } from './StatePack';

export function RunTranscriptPanel() {
  const messages = useUiStore((state) => state.messages);
  const runId = useUiStore((state) => state.run?.id ?? null);

  return (
    <section aria-label="Run transcript" className="mt-3 border-t border-hairline pt-3">
      <h3 className="type-label text-ink-900">Transcript · {messages.length}</h3>
      {runId === null ? <EmptyState message="No run selected." /> : null}
      {runId !== null && messages.length === 0 ? (
        <EmptyState message="No persisted transcript for this run." />
      ) : null}
      <ol className="mt-1 max-h-56 space-y-1 overflow-y-auto">
        {messages.map((message) => (
          <li
            key={message.id}
            className="chat-message rounded-[var(--radius-md)] border border-hairline bg-panel-alt p-2"
          >
            <div className="type-mono-meta text-meta">
              {message.role} · {message.runId?.slice(0, 8) ?? 'no run'}
              {message.streaming ? ' · streaming' : ''}
            </div>
            <pre className="type-mono-code mt-1 whitespace-pre-wrap break-words text-ink-900">
              {message.text}
            </pre>
          </li>
        ))}
      </ol>
    </section>
  );
}
