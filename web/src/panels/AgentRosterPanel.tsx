/**
 * Read-only roster projection: class, rank, profile and ownership come from the
 * server roster endpoint. No card state is invented client-side.
 */
import { useEffect, useState } from 'react';
import { apiBaseUrl } from '../api/client';
import { useUiStore } from '../state/context';
import { EmptyState } from './StatePack';

interface RosterRow {
  readonly task_id: string;
  readonly role: string;
  readonly class: string | null;
  readonly rank: number | null;
  readonly profile_id: string | null;
  readonly ownership_mode: string | null;
  readonly ownership_scope: string | null;
}

type Status = 'idle' | 'loading' | 'ok' | 'error';

export function AgentRosterPanel() {
  const runId = useUiStore((state) => state.run?.id ?? null);
  const [status, setStatus] = useState<Status>('idle');
  const [rows, setRows] = useState<readonly RosterRow[]>([]);

  useEffect(() => {
    if (runId === null) {
      setRows([]);
      setStatus('idle');
      return;
    }
    let cancelled = false;
    setStatus('loading');
    fetch(`${apiBaseUrl()}/api/agents/runs/${encodeURIComponent(runId)}/roster`, {
      headers: { Accept: 'application/json' },
    })
      .then(async (response) => {
        if (!response.ok) {
          throw new Error(`roster HTTP ${response.status}`);
        }
        const body = (await response.json()) as { agents?: unknown };
        if (!Array.isArray(body.agents)) {
          throw new Error('roster payload has no tasks');
        }
        const decoded: RosterRow[] = [];
        for (const raw of body.agents) {
          if (typeof raw !== 'object' || raw === null) {
            continue;
          }
          const item = raw as Record<string, unknown>;
          if (typeof item.task_id !== 'string' || typeof item.role !== 'string') {
            continue;
          }
          decoded.push({
            task_id: item.task_id,
            role: item.role,
            class: typeof item.class === 'string' ? item.class : null,
            rank: typeof item.rank === 'number' ? item.rank : null,
            profile_id: typeof item.profile_id === 'string' ? item.profile_id : null,
            ownership_mode: typeof item.ownership_mode === 'string' ? item.ownership_mode : null,
            ownership_scope: typeof item.ownership_scope === 'string' ? item.ownership_scope : null,
          });
        }
        if (!cancelled) {
          setRows(decoded);
          setStatus('ok');
        }
      })
      .catch(() => {
        if (!cancelled) {
          setRows([]);
          setStatus('error');
        }
      });
    return () => {
      cancelled = true;
    };
  }, [runId]);

  return (
    <section aria-label="Agent roster" className="surface-panel mt-3 rounded-[14px] p-3">
      <h3 className="type-label text-ink-900">Roster · {rows.length}</h3>
      {status === 'loading' ? <p className="type-mono-meta mt-1 text-meta">Loading roster…</p> : null}
      {status === 'error' ? (
        <p className="type-mono-meta mt-1 text-signal">Roster unavailable; no rows invented.</p>
      ) : null}
      {status === 'ok' && rows.length === 0 ? <EmptyState message="No roster rows." /> : null}
      {rows.length > 0 ? (
        <ul className="mt-1 space-y-0.5">
          {rows.map((row) => (
            <li key={row.task_id} className="type-mono-meta text-meta">
              {row.task_id.slice(0, 8)} · {row.class ?? 'unreported'} · rank{' '}
              {row.rank ?? '—'} · {row.role} · {row.ownership_mode ?? 'no ownership'}
              {row.ownership_scope ? ` (${row.ownership_scope})` : ''}
            </li>
          ))}
        </ul>
      ) : null}
    </section>
  );
}
