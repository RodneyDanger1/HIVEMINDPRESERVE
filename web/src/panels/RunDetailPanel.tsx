/**
 * Read-only run detail projection. Server data is authoritative; this panel
 * never invents task counts, usage, or failure codes.
 */
import { useEffect, useState } from 'react';
import { apiBaseUrl } from '../api/client';
import { useUiStore } from '../state/context';
import { EmptyState } from './StatePack';

interface TaskRow {
  readonly task_id: string;
  readonly role: string;
  readonly class: string | null;
  readonly state: string;
  readonly depth: number;
}

interface Detail {
  readonly status: string;
  readonly error_code: string | null;
  readonly task_count: number;
  readonly state_counts: Readonly<Record<string, number>>;
  readonly tasks: readonly TaskRow[];
  readonly memory_banks: number;
  readonly usage: { readonly total_tokens: number | null } | null;
}

type Status = 'idle' | 'loading' | 'ok' | 'error';

export function RunDetailPanel() {
  const runId = useUiStore((state) => state.run?.id ?? null);
  const [status, setStatus] = useState<Status>('idle');
  const [detail, setDetail] = useState<Detail | null>(null);

  useEffect(() => {
    if (runId === null) {
      setDetail(null);
      setStatus('idle');
      return;
    }
    let cancelled = false;
    setStatus('loading');
    fetch(`${apiBaseUrl()}/api/runs/${encodeURIComponent(runId)}/detail`, {
      headers: { Accept: 'application/json' },
    })
      .then(async (response) => {
        if (!response.ok) {
          throw new Error(`run detail HTTP ${response.status}`);
        }
        const body = (await response.json()) as Detail;
        if (!cancelled) {
          setDetail(body);
          setStatus('ok');
        }
      })
      .catch(() => {
        if (!cancelled) {
          setDetail(null);
          setStatus('error');
        }
      });
    return () => {
      cancelled = true;
    };
  }, [runId]);

  return (
    <section aria-label="Run detail" className="mt-3 border-t border-hairline pt-3">
      <h3 className="type-label text-ink-900">Run detail</h3>
      {runId === null ? <EmptyState message="No run selected." /> : null}
      {status === 'loading' ? <p className="type-mono-meta mt-1 text-meta">Loading run detail…</p> : null}
      {status === 'error' ? (
        <p className="type-mono-meta mt-1 text-signal">Run detail unavailable; no fields invented.</p>
      ) : null}
      {status === 'ok' && detail ? (
        <div className="mt-1 space-y-1">
          <div className="type-mono-meta text-meta">
            {detail.status} · {detail.task_count} tasks · {detail.memory_banks} banks
            {detail.error_code ? ` · ${detail.error_code}` : ''}
          </div>
          <div className="type-mono-meta text-meta">
            {Object.entries(detail.state_counts)
              .map(([state, count]) => `${state} ${count}`)
              .join(' · ')}
          </div>
          <div className="type-mono-meta text-meta">
            usage: {detail.usage?.total_tokens ?? 'unreported'}
          </div>
          <ul className="max-h-48 space-y-0.5 overflow-y-auto">
            {detail.tasks.map((task) => (
              <li key={task.task_id} className="type-mono-meta text-meta">
                {task.task_id.slice(0, 8)} · {task.class ?? 'unreported'} · {task.role} · {task.state} · d{task.depth}
              </li>
            ))}
          </ul>
        </div>
      ) : null}
    </section>
  );
}
