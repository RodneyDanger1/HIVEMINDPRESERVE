/**
 * Durable HMAS mailbox view: one row per persisted agent-to-agent message.
 * The server is the source; this panel never invents a message or recipient.
 */
import { useEffect, useState } from 'react';
import { useUiStore } from '../state/context';
import { EmptyState } from './StatePack';

interface AgentMessageEntry {
  readonly id: string;
  readonly from_task_id: string;
  readonly to_task_id: string;
  readonly kind: string;
  readonly text: string;
  readonly created_at: string;
}

type Status = 'idle' | 'loading' | 'ok' | 'error';

export function AgentMessagesPanel() {
  const runId = useUiStore((state) => state.run?.id ?? null);
  const messageVersion = useUiStore((state) => state.agentMessageVersion);
  const [status, setStatus] = useState<Status>('idle');
  const [messages, setMessages] = useState<readonly AgentMessageEntry[]>([]);

  useEffect(() => {
    if (runId === null) {
      setMessages([]);
      setStatus('idle');
      return;
    }
    let cancelled = false;
    setStatus('loading');
    fetch(`/api/agents/runs/${encodeURIComponent(runId)}/messages`, {
      headers: { Accept: 'application/json' },
    })
      .then(async (response) => {
        if (!response.ok) {
          throw new Error(`agent messages HTTP ${response.status}`);
        }
        const body = (await response.json()) as { messages?: unknown };
        if (!Array.isArray(body.messages)) {
          throw new Error('agent messages payload has no messages array');
        }
        const decoded: AgentMessageEntry[] = [];
        for (const entry of body.messages) {
          if (typeof entry !== 'object' || entry === null) {
            continue;
          }
          const item = entry as Record<string, unknown>;
          if (
            typeof item.id !== 'string' ||
            typeof item.from_task_id !== 'string' ||
            typeof item.to_task_id !== 'string' ||
            typeof item.kind !== 'string' ||
            typeof item.text !== 'string'
          ) {
            continue;
          }
          decoded.push({
            id: item.id,
            from_task_id: item.from_task_id,
            to_task_id: item.to_task_id,
            kind: item.kind,
            text: item.text,
            created_at: typeof item.created_at === 'string' ? item.created_at : '',
          });
        }
        if (!cancelled) {
          setMessages(decoded);
          setStatus('ok');
        }
      })
      .catch(() => {
        if (!cancelled) {
          setMessages([]);
          setStatus('error');
        }
      });
    return () => {
      cancelled = true;
    };
  }, [runId, messageVersion]);

  return (
    <section aria-label="Agent messages" className="mt-4 border-t border-hairline pt-4">
      <h3 className="type-label text-ink-900">Agent messages</h3>
      {status === 'loading' ? (
        <p className="type-mono-meta mt-1 text-meta">Loading persisted mailbox…</p>
      ) : null}
      {status === 'error' ? (
        <p className="type-mono-meta mt-1 text-signal">Mailbox unavailable; no messages invented.</p>
      ) : null}
      {status === 'ok' && messages.length === 0 ? (
        <EmptyState message="No agent-to-agent messages in this run." />
      ) : null}
      <ul className="mt-2 space-y-1">
        {messages.map((message) => (
          <li key={message.id} className="surface-panel rounded-[var(--radius-sm)] p-2">
            <div className="type-mono-meta text-meta">
              {message.kind} · {message.from_task_id.slice(0, 8)} → {message.to_task_id.slice(0, 8)}
            </div>
            <div className="type-mono-code mt-1 break-words text-ink-900">{message.text}</div>
          </li>
        ))}
      </ul>
    </section>
  );
}
