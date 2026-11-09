/**
 * Durable HMAS mailbox view: one row per persisted agent-to-agent message.
 * The server is the source; this panel never invents a message or recipient.
 */
import { useEffect, useState } from 'react';
import { apiBaseUrl } from '../api/client';
import { failureText, httpFailure , schemaFailure} from '../api/failure';
import { useUiStore, useUiStoreApi } from '../state/context';
import { EmptyState } from './StatePack';

interface AgentMessageEntry {
  readonly id: string;
  readonly from_task_id: string;
  readonly to_task_id: string;
  readonly kind: string;
  readonly text: string;
  readonly status?: string;
  readonly answer_text?: string | null;
  readonly answer_actor?: string | null;
  readonly created_at: string;
}

type Status = 'idle' | 'loading' | 'ok' | 'error';

export function AgentMessagesPanel() {
  const storeApi = useUiStoreApi();
  const runId = useUiStore((state) => state.run?.id ?? null);
  const messageVersion = useUiStore((state) => state.agentMessageVersion);
  const [status, setStatus] = useState<Status>('idle');
  const [messages, setMessages] = useState<readonly AgentMessageEntry[]>([]);
  const [kindFilter, setKindFilter] = useState('all');
  const [answeringId, setAnsweringId] = useState<string | null>(null);
  const [answerDraft, setAnswerDraft] = useState('');
  const [answerError, setAnswerError] = useState<string | null>(null);
  const [notice, setNotice] = useState<string | null>(null);
  const [version, setVersion] = useState(0);
  const visibleMessages =
    kindFilter === 'all' ? messages : messages.filter((message) => message.kind === kindFilter);

  useEffect(() => {
    setKindFilter('all');
  }, [runId]);

  useEffect(() => {
    if (runId === null) {
      setMessages([]);
      setStatus('idle');
      return;
    }
    let cancelled = false;
    setStatus('loading');
    fetch(`${apiBaseUrl()}/api/agents/runs/${encodeURIComponent(runId)}/messages`, {
      headers: { Accept: 'application/json' },
    })
      .then(async (response) => {
        if (!response.ok) {
          throw await httpFailure(response, 'GET agent messages');
        }
        const body = (await response.json()) as { messages?: unknown };
        if (!Array.isArray(body.messages)) {
          throw schemaFailure('agent messages payload has no messages array');
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
            status: typeof item.status === 'string' ? item.status : 'open',
            answer_text: typeof item.answer_text === 'string' ? item.answer_text : null,
            answer_actor: typeof item.answer_actor === 'string' ? item.answer_actor : null,
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
  }, [runId, messageVersion, version]);

  const answer = async (messageId: string): Promise<void> => {
    const text = answerDraft.trim();
    if (text.length === 0) {
      setAnswerError('Answer text is empty.');
      return;
    }
    const token = storeApi.getState().session?.token;
    setAnswerError(null);
    setAnsweringId(messageId);
    try {
      if (!runId || !token) {
        throw new Error('session token unavailable');
      }
      const response = await fetch(
        `${apiBaseUrl()}/api/agents/runs/${encodeURIComponent(runId)}/messages/${encodeURIComponent(messageId)}/answer`,
        {
          method: 'POST',
          headers: {
            'Content-Type': 'application/json',
            Accept: 'application/json',
            'x-hivemind-token': token,
          },
          body: JSON.stringify({ answer: text }),
        },
      );
      if (!response.ok) {
        const body = (await response.json().catch(() => ({}))) as { code?: string };
        throw new Error(body.code ?? `answer HTTP ${response.status}`);
      }
      const ack = (await response.json().catch(() => null)) as { status?: unknown } | null;
      if (ack?.status !== 'answered') {
        throw new Error('answer ACK did not report status=answered');
      }
      // An OK POST is not proof the mailbox projection changed. Confirm against
      // the durable GET before claiming success; never patch the row locally.
      let confirmed = false;
      try {
        const projection = await fetch(
          `${apiBaseUrl()}/api/agents/runs/${encodeURIComponent(runId)}/messages`,
          { headers: { Accept: 'application/json' } },
        );
        if (projection.ok) {
          const body = (await projection.json()) as { messages?: unknown };
          if (Array.isArray(body.messages)) {
            const row = body.messages.find(
              (raw): raw is Record<string, unknown> =>
                typeof raw === 'object' && raw !== null && (raw as Record<string, unknown>).id === messageId,
            );
            confirmed =
              row !== undefined && typeof row.status === 'string' && row.status !== 'open';
          }
        }
      } catch {
        confirmed = false;
      }
      if (confirmed) {
        setNotice('Answer accepted (confirmed by the server projection).');
        setAnswerDraft('');
      } else {
        setNotice(
          'Answer submitted, but the server projection has not confirmed it yet; refresh before assuming success.',
        );
      }
      setVersion((value) => value + 1);
    } catch (error: unknown) {
      setAnswerError(failureText(error, 'answer refused'));
    } finally {
      setAnsweringId(null);
    }
  };

  return (
    <section aria-label="Agent messages" className="surface-panel mt-3 rounded-[var(--radius-card)] p-3">
      <h3 className="type-label text-ink-900">Agent messages</h3>
      {notice ? (
        <p role="status" className="type-mono-meta mt-1 text-meta">
          {notice}
        </p>
      ) : null}
      {status === 'ok' && messages.length > 0 ? (
        <label className="mt-2 block">
          <span className="type-mono-meta block text-meta">Kind</span>
          <select
            className="mt-1 h-8 w-full rounded-[var(--radius-sm)] border border-hairline bg-panel px-2 type-mono-code text-ink-900"
            value={kindFilter}
            onChange={(event) => setKindFilter(event.target.value)}
          >
            <option value="all">All messages</option>
            <option value="ask_parent">ask_parent</option>
            <option value="send_to_sibling">send_to_sibling</option>
          </select>
        </label>
      ) : null}
      {status === 'loading' ? (
        <p className="type-mono-meta mt-1 text-meta">Loading persisted mailbox…</p>
      ) : null}
      {status === 'error' ? (
        <p className="type-mono-meta mt-1 text-signal">Mailbox unavailable; no messages invented.</p>
      ) : null}
      {status === 'ok' && messages.length === 0 ? (
        <EmptyState message="No agent-to-agent messages in this run." />
      ) : null}
      {status === 'ok' && messages.length > 0 && visibleMessages.length === 0 ? (
        <EmptyState message="No messages of this kind in this run." />
      ) : null}
      <ul className="mt-2 space-y-1">
        {visibleMessages.map((message) => (
          <li key={message.id} className="surface-panel rounded-[var(--radius-sm)] p-2">
            <div className="type-mono-meta text-meta">
              {message.kind} · {message.from_task_id.slice(0, 8)} → {message.to_task_id.slice(0, 8)} ·{' '}
              {message.status ?? 'open'}
            </div>
            <div className="type-mono-code mt-1 break-words text-ink-900">{message.text}</div>
            {message.answer_text ? (
              <div className="type-mono-code mt-1 break-words text-signal">
                answer ({message.answer_actor ?? 'operator'}): {message.answer_text}
              </div>
            ) : null}
            {(message.status ?? 'open') === 'open' ? (
              <div className="mt-1 flex gap-1">
                <input
                  className="h-8 min-w-0 flex-1 rounded-[var(--radius-sm)] border border-hairline bg-panel px-2 type-mono-code text-ink-900"
                  aria-label={`Answer ${message.id}`}
                  value={answeringId === message.id ? answerDraft : ''}
                  placeholder="Operator answer"
                  onChange={(event) => {
                    setAnsweringId(message.id);
                    setAnswerDraft(event.target.value);
                  }}
                />
                <button
                  type="button"
                  className="h-8 rounded-[var(--radius-sm)] border border-hairline px-2 type-mono-meta"
                  disabled={answeringId !== null}
                  aria-busy={answeringId === message.id}
                  onClick={() => void answer(message.id)}
                >
                  {answeringId === message.id ? 'Sending…' : 'Answer'}
                </button>
              </div>
            ) : null}
            {answerError && answeringId === message.id ? (
              <div className="type-mono-meta mt-1 text-signal">{answerError}</div>
            ) : null}
          </li>
        ))}
      </ul>
    </section>
  );
}
