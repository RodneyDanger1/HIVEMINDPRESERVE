/**
 * Read-only projection of authorized evolution lessons. This panel never
 * applies or removes a lesson; it reads the server's fixed lesson ledger.
 */
import { useEffect, useState } from 'react';
import { apiBaseUrl } from '../api/client';
import { failureText, httpFailure } from '../api/failure';
import { EmptyState, ErrorState, LoadingState } from './StatePack';
import { fetchWithBoundTimeout } from '../api/http';

export interface LessonRow {
  readonly proposalId: string;
  readonly area: string;
  readonly problem: string;
  readonly proposal: string;
}

type Status = 'idle' | 'loading' | 'ok' | 'error';

function text(value: unknown): string | null {
  return typeof value === 'string' ? value : null;
}

export function decodeLessons(body: unknown): LessonRow[] {
  if (typeof body !== 'object' || body === null) {
    return [];
  }
  const raw = (body as Record<string, unknown>).lessons;
  if (!Array.isArray(raw)) {
    return [];
  }
  const rows: LessonRow[] = [];
  for (const item of raw) {
    if (typeof item !== 'object' || item === null) {
      continue;
    }
    const record = item as Record<string, unknown>;
    const proposalId = text(record.proposal_id);
    const area = text(record.area);
    const problem = text(record.problem);
    const proposal = text(record.proposal);
    if (proposalId === null || area === null || problem === null || proposal === null) {
      continue;
    }
    rows.push({ proposalId, area, problem, proposal });
  }
  return rows;
}

export function LessonsPanel() {
  const [status, setStatus] = useState<Status>('loading');
  const [rows, setRows] = useState<readonly LessonRow[]>([]);
  const [failure, setFailure] = useState<string | null>(null);
  const [reloadToken, setReloadToken] = useState(0);

  useEffect(() => {
    const refresh = (): void => setReloadToken((value) => value + 1);
    window.addEventListener('hivemind:evolution-changed', refresh);
    return () => window.removeEventListener('hivemind:evolution-changed', refresh);
  }, []);

  useEffect(() => {
    let cancelled = false;
    const controller = new AbortController();
    setStatus('loading');
    fetchWithBoundTimeout(`${apiBaseUrl()}/api/evolution/lessons`, {
      label: 'GET evolution lessons',
      signal: controller.signal,
      init: { headers: { Accept: 'application/json' } },
    })
      .then(async (response) => {
        if (!response.ok) {
          throw await httpFailure(response, 'GET evolution lessons');
        }
        const body = (await response.json()) as Record<string, unknown>;
        if (!Array.isArray(body.lessons)) {
          throw new Error('lesson route returned no lessons array');
        }
        if (!cancelled) {
          setRows(decodeLessons(body));
          setFailure(null);
          setStatus('ok');
        }
      })
      .catch((error: unknown) => {
        if (!cancelled) {
          setRows([]);
          setFailure(failureText(error, 'Evolution lessons are unavailable.'));
          setStatus('error');
        }
      });
    return () => {
      cancelled = true;
      controller.abort();
    };
  }, [reloadToken]);

  return (
    <section
      aria-label="Authorized evolution lessons"
      className="surface-panel mt-3 rounded-[var(--radius-card)] p-3"
    >
      <div className="flex items-baseline justify-between gap-2">
        <h3 className="type-heading-h4 text-ink-900">Authorized lessons</h3>
        <span className="type-mono-meta text-meta">read-only</span>
      </div>
      {status === 'loading' ? <LoadingState label="Loading lessons..." /> : null}
      {status === 'error' ? <ErrorState message={failure ?? 'Evolution lessons are unavailable.'} /> : null}
      {status === 'ok' && rows.length === 0 ? (
        <EmptyState message="No authorized evolution lessons recorded." />
      ) : null}
      {rows.length > 0 ? (
        <ul className="mt-2 space-y-2">
          {rows.map((row) => (
            <li key={row.proposalId} className="rounded-[var(--radius-card)] border border-hairline p-2">
              <span className="type-label text-ink-900">{row.area}</span>
              <p className="type-mono-meta mt-1 text-ink-700">{row.problem}</p>
              <p className="type-body-sm mt-1 text-ink-900">{row.proposal}</p>
            </li>
          ))}
        </ul>
      ) : null}
      <p className="type-mono-meta mt-2 text-meta">
        Lesson ledger only; it never changes code, prompts, tools, personas, or gates.
      </p>
    </section>
  );
}
