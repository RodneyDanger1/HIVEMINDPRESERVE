/**
 * Operator-visible session todos. This is a UI over the existing session-todo
 * API, not an agent tool and not a fake plan. Todos are also injected into agent
 * prompts by the engine; additions here are immediately visible to the next run.
 */
import { useCallback, useEffect, useState } from 'react';
import { useUiStore } from '../state/context';
import { EmptyState, ErrorState, LoadingState, PanelTitle } from './StatePack';

interface Todo {
  readonly id: string;
  readonly seq: number;
  readonly text: string;
  readonly state: 'open' | 'done' | 'cancelled';
}

async function todoRequest(path: string, init?: RequestInit): Promise<Response> {
  const handshake = await fetch('/api/session', { headers: { accept: 'application/json' } });
  if (!handshake.ok) throw new Error('session unavailable');
  const token = (await handshake.json()) as { token?: string };
  const headers: Record<string, string> = {
    accept: 'application/json',
    ...(init?.headers as Record<string, string> | undefined),
  };
  if (token.token) headers.Authorization = `Bearer ${token.token}`;
  return fetch(path, { ...init, headers });
}

export function SessionTodos() {
  const connection = useUiStore((state) => state.connection);
  const sessionId = useUiStore((state) => state.activeSessionId);
  const [todos, setTodos] = useState<readonly Todo[] | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [draft, setDraft] = useState('');
  const [busy, setBusy] = useState(false);

  const load = useCallback(async () => {
    if (!sessionId) {
      setTodos([]);
      return;
    }
    try {
      const response = await todoRequest(
        `/api/sessions/${encodeURIComponent(sessionId)}/todos`,
      );
      if (!response.ok) throw new Error(`GET todos returned ${response.status}`);
      const body = (await response.json()) as { todos?: readonly Todo[] };
      setTodos(body.todos ?? []);
      setError(null);
    } catch (failure) {
      setError(failure instanceof Error ? failure.message : 'todos unavailable');
    }
  }, [sessionId]);

  useEffect(() => {
    if (connection !== 'open') return;
    void load();
  }, [connection, load]);

  const setState = async (todo: Todo, state: 'open' | 'done'): Promise<void> => {
    if (!sessionId || busy) return;
    setBusy(true);
    try {
      const response = await todoRequest(
        `/api/sessions/${encodeURIComponent(sessionId)}/todos/${encodeURIComponent(todo.id)}/state`,
        {
          method: 'POST',
          headers: { 'content-type': 'application/json' },
          body: JSON.stringify({ state }),
        },
      );
      if (!response.ok) throw new Error(`todo state returned ${response.status}`);
      await load();
    } catch (failure) {
      setError(failure instanceof Error ? failure.message : 'todo update failed');
    } finally {
      setBusy(false);
    }
  };

  const add = async (): Promise<void> => {
    const text = draft.trim();
    if (!text || !sessionId || busy || text.length > 4_000) return;
    setBusy(true);
    try {
      const response = await todoRequest(
        `/api/sessions/${encodeURIComponent(sessionId)}/todos`,
        {
          method: 'POST',
          headers: { 'content-type': 'application/json' },
          body: JSON.stringify({ text }),
        },
      );
      if (!response.ok) throw new Error(`POST todo returned ${response.status}`);
      setDraft('');
      await load();
    } catch (failure) {
      setError(failure instanceof Error ? failure.message : 'todo add failed');
    } finally {
      setBusy(false);
    }
  };

  return (
    <section aria-label="Session todos" className="surface-panel mt-3 rounded-[var(--radius-card)] p-3">
      <PanelTitle>Session todos</PanelTitle>
      {!sessionId ? (
        <EmptyState message="Select a named session in the composer to see or add todos." />
      ) : null}
      {sessionId && todos === null && error === null ? <LoadingState /> : null}
      {error !== null ? <ErrorState message={error} /> : null}
      {sessionId && todos !== null && todos.length === 0 ? (
        <EmptyState message="No todos in this session yet." />
      ) : null}
      {todos !== null && todos.length > 0 ? (
        <ul className="mt-2 space-y-1" aria-label="Todo list">
          {todos.map((todo) => (
            <li
              key={todo.id}
              className="flex items-start gap-2 rounded-[8px] border border-hairline bg-panel p-2"
            >
              <span className="type-mono-meta shrink-0 text-[10px] text-meta">{todo.seq}</span>
              <span className="min-w-0 flex-1 text-[12px] text-ink-900">{todo.text}</span>
              <span className="type-mono-meta shrink-0 text-[10px] text-meta">{todo.state}</span>
              <button
                type="button"
                className="type-mono-meta shrink-0 rounded-[4px] border border-hairline px-1 py-0.5 text-[10px] text-ink-700 disabled:opacity-50"
                disabled={busy}
                onClick={() => void setState(todo, todo.state === 'done' ? 'open' : 'done')}
              >
                {todo.state === 'done' ? 'Reopen' : 'Done'}
              </button>
            </li>
          ))}
        </ul>
      ) : null}
      {sessionId ? (
        <form
          className="mt-2 flex items-center gap-1"
          onSubmit={(event) => {
            event.preventDefault();
            void add();
          }}
        >
          <label className="sr-only" htmlFor="new-todo">
            New todo
          </label>
          <input
            id="new-todo"
            className="type-mono-code min-w-0 flex-1 rounded-[6px] border border-hairline bg-panel px-2 py-1 text-[11px] text-ink-900"
            placeholder="Add a todo for this session"
            value={draft}
            disabled={busy}
            onChange={(event) => setDraft(event.target.value)}
          />
          <button
            type="submit"
            className="rounded-[6px] border border-hairline bg-panel px-2 py-1 text-[11px] text-ink-800 disabled:opacity-50"
            disabled={busy || draft.trim().length === 0}
          >
            Add
          </button>
        </form>
      ) : null}
    </section>
  );
}
