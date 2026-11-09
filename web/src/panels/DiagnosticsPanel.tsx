/**
 * Read-only diagnostics surface for `/diagnose`. The endpoint is secret-free by
 * contract; this panel shows a compact summary and then the exact JSON so an
 * operator never has to guess which values were derived.
 */
import { useEffect, useState } from 'react';
import { apiBaseUrl } from '../api/client';

type Status = 'idle' | 'loading' | 'ok' | 'error';

interface DiagnoseSummary {
  readonly schema: string | null;
  readonly version: string | null;
  readonly db_ok: boolean | null;
  readonly provider_key_present: boolean | null;
  readonly tool_count: number | null;
  readonly refusal_count: number | null;
  readonly token_budget: number | null;
  readonly call_cap: number | null;
}

const EMPTY: DiagnoseSummary = {
  schema: null,
  version: null,
  db_ok: null,
  provider_key_present: null,
  tool_count: null,
  refusal_count: null,
  token_budget: null,
  call_cap: null,
};

export function DiagnosticsPanel() {
  const [status, setStatus] = useState<Status>('idle');
  const [summary, setSummary] = useState<DiagnoseSummary>(EMPTY);
  const [raw, setRaw] = useState<string | null>(null);
  const [workspace, setWorkspace] = useState<{
    readonly dataDir: string | null;
    readonly workspaceRoot: string | null;
    readonly dbPath: string | null;
  } | null>(null);
  const [activeRun, setActiveRun] = useState<string | null>(null);

  useEffect(() => {
    let cancelled = false;
    setStatus('loading');
    Promise.all([
      fetch(`${apiBaseUrl()}/diagnose`, { headers: { Accept: 'application/json' } }),
      fetch(`${apiBaseUrl()}/api/workspace`, { headers: { Accept: 'application/json' } }),
      fetch(`${apiBaseUrl()}/api/runs/active`, { headers: { Accept: 'application/json' } }),
    ])
      .then(async ([response, workspaceResponse, activeResponse]) => {
        if (!response.ok) {
          throw new Error(`diagnose HTTP ${response.status}`);
        }
        const body = (await response.json()) as Record<string, unknown>;
        const workspaceBody = workspaceResponse.ok
          ? ((await workspaceResponse.json()) as Record<string, unknown>)
          : null;
        const activeBody = activeResponse.ok
          ? ((await activeResponse.json()) as Record<string, unknown>)
          : null;
        const latest =
          typeof activeBody?.latest_active_run === 'object' &&
          activeBody.latest_active_run !== null
            ? (activeBody.latest_active_run as Record<string, unknown>)
            : null;
        const activeLabel =
          typeof activeBody?.active_run_id === 'string'
            ? `live ${activeBody.active_run_id.slice(0, 8)}`
            : latest && typeof latest.run_id === 'string'
              ? `${latest.run_id.slice(0, 8)} · ${typeof latest.status === 'string' ? latest.status : 'unknown'}`
              : 'none';
        const provider =
          typeof body.provider === 'object' && body.provider !== null
            ? (body.provider as Record<string, unknown>)
            : null;
        const tools =
          typeof body.tools === 'object' && body.tools !== null
            ? (body.tools as Record<string, unknown>)
            : null;
        const refusals =
          typeof body.refusals === 'object' && body.refusals !== null
            ? (body.refusals as Record<string, unknown>)
            : null;
        if (!cancelled) {
          setSummary({
            schema: typeof body.schema === 'string' ? body.schema : null,
            version: typeof body.version === 'string' ? body.version : null,
            db_ok: typeof body.db_ok === 'boolean' ? body.db_ok : null,
            provider_key_present:
              typeof body.provider_key_present === 'boolean'
                ? body.provider_key_present
                : null,
            tool_count: typeof tools?.count === 'number' ? tools.count : null,
            refusal_count: typeof refusals?.count === 'number' ? refusals.count : null,
            token_budget:
              typeof provider?.token_budget_combined === 'number'
                ? provider.token_budget_combined
                : null,
            call_cap: typeof provider?.call_cap === 'number' ? provider.call_cap : null,
          });
          setRaw(JSON.stringify(body, null, 2));
          setWorkspace(
            workspaceBody === null
              ? null
              : {
                  dataDir:
                    typeof workspaceBody.data_dir === 'string' ? workspaceBody.data_dir : null,
                  workspaceRoot:
                    typeof workspaceBody.workspace_root === 'string'
                      ? workspaceBody.workspace_root
                      : null,
                  dbPath: typeof workspaceBody.db_path === 'string' ? workspaceBody.db_path : null,
                },
          );
          setActiveRun(activeLabel);
          setStatus('ok');
        }
      })
      .catch(() => {
        if (!cancelled) {
          setSummary(EMPTY);
          setRaw(null);
          setWorkspace(null);
          setActiveRun(null);
          setStatus('error');
        }
      });
    return () => {
      cancelled = true;
    };
  }, []);

  return (
    <section className="surface-panel mt-3 rounded-[var(--radius-md)] p-3" aria-label="Diagnostics">
      <h3 className="type-label text-ink-900">Diagnostics</h3>
      {status === 'loading' ? (
        <p className="type-mono-meta mt-1 text-meta">Loading diagnostics…</p>
      ) : null}
      {status === 'error' ? (
        <p className="type-mono-meta mt-1 text-signal">
          Diagnostics unavailable; no values invented.
        </p>
      ) : null}
      {status === 'ok' ? (
        <>
          <dl className="mt-1 grid grid-cols-[150px_1fr] gap-y-0.5">
            <dt className="type-mono-meta text-meta">schema</dt>
            <dd className="type-mono-code text-ink-900">{summary.schema ?? 'not reported'}</dd>
            <dt className="type-mono-meta text-meta">version</dt>
            <dd className="type-mono-code text-ink-900">{summary.version ?? 'not reported'}</dd>
            <dt className="type-mono-meta text-meta">database</dt>
            <dd className="type-mono-code text-ink-900">
              {summary.db_ok === null ? 'unknown' : summary.db_ok ? 'ok' : 'unhealthy'}
            </dd>
            <dt className="type-mono-meta text-meta">provider key</dt>
            <dd className="type-mono-code text-ink-900">
              {summary.provider_key_present === null
                ? 'unknown'
                : summary.provider_key_present
                  ? 'present'
                  : 'absent'}
            </dd>
            <dt className="type-mono-meta text-meta">tools / refusals</dt>
            <dd className="type-mono-code text-ink-900">
              {summary.tool_count ?? '—'} / {summary.refusal_count ?? '—'}
            </dd>
            <dt className="type-mono-meta text-meta">token budget / call cap</dt>
            <dd className="type-mono-code text-ink-900">
              {summary.token_budget ?? '—'} / {summary.call_cap ?? '—'}
            </dd>
            <dt className="type-mono-meta text-meta">active run</dt>
            <dd className="type-mono-code text-ink-900">{activeRun ?? 'unknown'}</dd>
            {workspace ? (
              <>
                <dt className="type-mono-meta text-meta">workspace root</dt>
                <dd className="type-mono-code break-all text-ink-900">
                  {workspace.workspaceRoot ?? 'not reported'}
                </dd>
                <dt className="type-mono-meta text-meta">data dir</dt>
                <dd className="type-mono-code break-all text-ink-900">
                  {workspace.dataDir ?? 'not reported'}
                </dd>
                <dt className="type-mono-meta text-meta">database</dt>
                <dd className="type-mono-code break-all text-ink-900">
                  {workspace.dbPath ?? 'not reported'}
                </dd>
              </>
            ) : null}
          </dl>
          {raw ? (
            <details className="mt-2">
              <summary className="type-mono-meta cursor-pointer text-meta">Raw diagnosis</summary>
              <pre className="type-mono-meta mt-1 max-h-64 overflow-auto whitespace-pre-wrap text-meta">
                {raw}
              </pre>
            </details>
          ) : null}
        </>
      ) : null}
    </section>
  );
}
