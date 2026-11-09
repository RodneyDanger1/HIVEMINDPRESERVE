/**
 * Operator A2A inbox: reads the durable open-question projection and answers
 * through the same CAS endpoint the mailbox uses. No row is invented client-side
 * and an answer is never optimistically marked as accepted.
 */
import { useEffect, useState } from 'react';
import { apiBaseUrl } from '../api/client';
import { useUiStore, useUiStoreApi } from '../state/context';
import { EmptyState } from './StatePack';

interface PendingQuestion {
  readonly run_id: string;
  readonly id: string;
  readonly from_task_id: string;
  readonly to_task_id: string;
  readonly kind: string;
  readonly text: string;
  readonly deadline_at: string | null;
  readonly created_at: string;
}

type Status = 'idle' | 'loading' | 'ok' | 'error';

export function A2AInboxPanel() {
  const storeApi = useUiStoreApi();
  const runId = useUiStore((state) => state.run?.id ?? null);
  const [status, setStatus] = useState<Status>('idle');
  const [questions, setQuestions] = useState<readonly PendingQuestion[]>([]);
  const [drafts, setDrafts] = useState<Readonly<Record<string, string>>>({});
  const [notice, setNotice] = useState<string | null>(null);
  const [version, setVersion] = useState(0);
  const [sendingId, setSendingId] = useState<string | null>(null);

  useEffect(() => {
    let cancelled = false;
    setStatus('loading');
    fetch(`${apiBaseUrl()}/api/a2a/pending`, { headers: { Accept: 'application/json' } })
      .then(async (response) => {
        if (!response.ok) {
          throw new Error(`A2A pending HTTP ${response.status}`);
        }
        const body = (await response.json()) as { messages?: unknown };
        if (!Array.isArray(body.messages)) {
          throw new Error('A2A pending payload has no messages');
        }
        const decoded: PendingQuestion[] = [];
        for (const raw of body.messages) {
          if (typeof raw !== 'object' || raw === null) {
            continue;
          }
          const item = raw as Record<string, unknown>;
          if (
            typeof item.run_id !== 'string' ||
            typeof item.id !== 'string' ||
            typeof item.from_task_id !== 'string' ||
            typeof item.to_task_id !== 'string' ||
            typeof item.kind !== 'string' ||
            typeof item.text !== 'string'
          ) {
            continue;
          }
          decoded.push({
            run_id: item.run_id,
            id: item.id,
            from_task_id: item.from_task_id,
            to_task_id: item.to_task_id,
            kind: item.kind,
            text: item.text,
            deadline_at: typeof item.deadline_at === 'string' ? item.deadline_at : null,
            created_at: typeof item.created_at === 'string' ? item.created_at : '',
          });
        }
        if (!cancelled) {
          setQuestions(decoded);
          setStatus('ok');
        }
      })
      .catch(() => {
        if (!cancelled) {
          setQuestions([]);
          setStatus('error');
        }
      });
    return () => {
      cancelled = true;
    };
  }, [version, runId]);

  const answer = async (question: PendingQuestion): Promise<void> => {
    if (sendingId !== null) {
      return;
    }
    const draft = (drafts[question.id] ?? '').trim();
    if (draft.length === 0 || draft.length > 2_000) {
      setNotice('Answer must be 1–2000 characters.');
      return;
    }
    const token = storeApi.getState().session?.token;
    if (!token) {
      setNotice('Session token unavailable; answer not sent.');
      return;
    }
    setSendingId(question.id);
    try {
      const response = await fetch(
        `${apiBaseUrl()}/api/agents/runs/${encodeURIComponent(question.run_id)}/messages/${encodeURIComponent(question.id)}/answer`,
        {
          method: 'POST',
          headers: {
            'Content-Type': 'application/json',
            Accept: 'application/json',
            'x-hivemind-token': token,
          },
          body: JSON.stringify({ answer: draft, actor: 'operator' }),
        },
      );
      if (!response.ok) {
        const body = (await response.json().catch(() => ({}))) as { code?: string };
        throw new Error(body.code ?? `answer HTTP ${response.status}`);
      }
      setNotice('Answer accepted.');
      setDrafts((current) => ({ ...current, [question.id]: '' }));
      setQuestions((current) => current.filter((row) => row.id !== question.id));
      setVersion((value) => value + 1);
    } catch (error: unknown) {
      setNotice(error instanceof Error ? error.message : 'answer refused');
    } finally {
      setSendingId(null);
    }
  };

  return (
    <section aria-label="A2A inbox" className="surface-panel mt-3 rounded-[var(--radius-card)] p-3">
      <div className="flex items-center justify-between gap-2">
        <h3 className="type-label text-ink-900">A2A inbox · {questions.length}</h3>
        <button
          type="button"
          className="type-label h-8 rounded-[var(--radius-button)] border border-hairline px-3 text-ink-800 hover:bg-honey-100 transition-colors"
          onClick={() => setVersion((value) => value + 1)}
        >
          Refresh
        </button>
      </div>
      {notice ? (
        <p role="status" className="type-mono-meta mt-1 text-meta">
          {notice}
        </p>
      ) : null}
      {status === 'loading' ? <p className="type-mono-meta mt-1 text-meta">Loading inbox…</p> : null}
      {status === 'error' ? (
        <p role="alert" className="type-mono-meta mt-1 text-signal">
          Inbox unavailable; no question invented.
        </p>
      ) : null}
      {status === 'ok' && questions.length === 0 ? (
        <EmptyState message="No open A2A questions." />
      ) : null}
      <ul className="mt-2 space-y-2">
        {questions.map((question) => (
          <li
            key={question.id}
            className="chat-message rounded-[var(--radius-md)] border border-hairline bg-panel-alt p-2"
          >
            <div className="type-mono-meta text-meta">
              {question.kind} · {question.from_task_id.slice(0, 8)} → {question.to_task_id.slice(0, 8)}
              {question.deadline_at ? ` · deadline ${question.deadline_at}` : ''}
            </div>
            <div className="type-body-l mt-1 break-words text-ink-900">{question.text}</div>
            <input
              className="type-mono-code mt-2 h-8 w-full rounded-[var(--radius-sm)] border border-hairline bg-panel px-2 text-ink-900"
              maxLength={2_000}
              placeholder="Operator answer"
              value={drafts[question.id] ?? ''}
              onChange={(event) =>
                setDrafts((current) => ({ ...current, [question.id]: event.target.value }))
              }
              onKeyDown={(event) => {
                if (event.key === 'Enter' && !event.shiftKey) {
                  event.preventDefault();
                  void answer(question);
                }
              }}
            />
            <button
              type="button"
              aria-busy={sendingId === question.id}
              disabled={sendingId === question.id}
              className="type-label mt-1 h-8 rounded-[var(--radius-button)] border border-honey-500 bg-honey-300 px-3 font-semibold text-ink-900 transition-colors hover:bg-honey-400 disabled:opacity-50"
              onClick={() => void answer(question)}
            >
              {sendingId === question.id ? 'Sending…' : 'Answer'}
            </button>
          </li>
        ))}
      </ul>
    </section>
  );
}
