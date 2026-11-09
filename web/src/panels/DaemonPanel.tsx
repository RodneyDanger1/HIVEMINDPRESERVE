/**
 * Daemon/watchdog records are tree-free offices: they are not task nodes and
 * never invoke a model. This panel only projects server state and sends the
 * explicit operator switch; no card is invented client-side.
 */
import { useCallback, useEffect, useState } from 'react';
import { apiBaseUrl } from '../api/client';
import { useUiStoreApi } from '../state/context';
import { EmptyState } from './StatePack';

interface DaemonRow {
  readonly daemon_id: string;
  readonly display_name: string;
  readonly owner_kind: string;
  readonly owner_id: string;
  readonly kind: string;
  readonly state: string;
  readonly interval_secs: number;
  readonly last_tick_at: string | null;
  readonly last_report_at: string | null;
  readonly last_error_code: string | null;
}

interface SupervisorRow {
  readonly last_tick_at: string | null;
  readonly next_due_at: string | null;
  readonly backoff_ms: number;
  readonly last_error_code: string | null;
}

type Status = 'idle' | 'loading' | 'ok' | 'error';

export function DaemonPanel() {
  const storeApi = useUiStoreApi();
  const [status, setStatus] = useState<Status>('idle');
  const [daemons, setDaemons] = useState<readonly DaemonRow[]>([]);
  const [supervisor, setSupervisor] = useState<SupervisorRow | null>(null);
  const [name, setName] = useState('Hive watchdog');
  const [kind, setKind] = useState('watchdog');
  const [interval, setInterval] = useState('60');
  const [notice, setNotice] = useState<string | null>(null);
  const [busy, setBusy] = useState(false);
  const [reports, setReports] = useState<Readonly<Record<string, readonly { outcome: string; created_at: string }[]>>>({});

  const load = useCallback(async (): Promise<void> => {
    setStatus('loading');
    try {
      const [daemonResponse, supervisorResponse] = await Promise.all([
        fetch(`${apiBaseUrl()}/api/daemons`, { headers: { Accept: 'application/json' } }),
        fetch(`${apiBaseUrl()}/api/supervisor`, { headers: { Accept: 'application/json' } }),
      ]);
      if (!daemonResponse.ok) throw new Error(`daemons HTTP ${daemonResponse.status}`);
      const body = (await daemonResponse.json()) as { daemons?: unknown };
      const rows = Array.isArray(body.daemons)
        ? body.daemons.filter((raw): raw is DaemonRow => {
            if (typeof raw !== 'object' || raw === null) return false;
            const item = raw as Record<string, unknown>;
            return (
              typeof item.daemon_id === 'string' &&
              typeof item.display_name === 'string' &&
              typeof item.state === 'string' &&
              typeof item.kind === 'string'
            );
          })
        : [];
      setDaemons(rows);
      if (supervisorResponse.ok) {
        const projection = (await supervisorResponse.json()) as SupervisorRow;
        setSupervisor(projection);
      } else {
        setSupervisor(null);
      }
      setStatus('ok');
    } catch {
      setStatus('error');
    }
  }, []);

  useEffect(() => {
    void load();
  }, [load]);

  const write = async (path: string, body: unknown): Promise<Record<string, unknown>> => {
    const token = storeApi.getState().session?.token;
    if (!token) throw new Error('Session token unavailable');
    const response = await fetch(`${apiBaseUrl()}${path}`, {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
        Accept: 'application/json',
        'x-hivemind-token': token,
      },
      body: JSON.stringify(body),
    });
    const payload = (await response.json().catch(() => ({}))) as Record<string, unknown>;
    if (!response.ok) throw new Error(typeof payload.code === 'string' ? payload.code : `HTTP ${response.status}`);
    return payload;
  };

  const create = async (): Promise<void> => {
    if (busy || name.trim().length === 0) return;
    setBusy(true);
    try {
      await write('/api/daemons', {
        display_name: name.trim(),
        kind,
        interval_secs: Number(interval),
      });
      setNotice('Daemon enrolled; it is not running until switched on.');
      await load();
    } catch (error: unknown) {
      setNotice(error instanceof Error ? error.message : 'daemon refused');
    } finally {
      setBusy(false);
    }
  };

  const toggle = async (daemon: DaemonRow): Promise<void> => {
    if (busy) return;
    setBusy(true);
    try {
      await write(`/api/daemons/${encodeURIComponent(daemon.daemon_id)}/switch`, {
        enabled: daemon.state !== 'running',
      });
      await load();
    } catch (error: unknown) {
      setNotice(error instanceof Error ? error.message : 'switch refused');
    } finally {
      setBusy(false);
    }
  };

  const retire = async (daemon: DaemonRow): Promise<void> => {
    if (busy) return;
    setBusy(true);
    try {
      await write(`/api/daemons/${encodeURIComponent(daemon.daemon_id)}/retire`, {});
      await load();
    } catch (error: unknown) {
      setNotice(error instanceof Error ? error.message : 'retire refused');
    } finally {
      setBusy(false);
    }
  };

  const loadReports = async (daemon: DaemonRow): Promise<void> => {
    if (reports[daemon.daemon_id]) {
      setReports((current) => {
        const next = { ...current };
        delete next[daemon.daemon_id];
        return next;
      });
      return;
    }
    try {
      const response = await fetch(
        `${apiBaseUrl()}/api/daemons/${encodeURIComponent(daemon.daemon_id)}/reports`,
        { headers: { Accept: 'application/json' } },
      );
      if (!response.ok) throw new Error(`reports HTTP ${response.status}`);
      const body = (await response.json()) as { reports?: unknown };
      const rows = Array.isArray(body.reports)
        ? body.reports.filter((raw): raw is { outcome: string; created_at: string } => {
            if (typeof raw !== 'object' || raw === null) return false;
            const item = raw as Record<string, unknown>;
            return typeof item.outcome === 'string' && typeof item.created_at === 'string';
          })
        : [];
      setReports((current) => ({ ...current, [daemon.daemon_id]: rows }));
    } catch (error: unknown) {
      setNotice(error instanceof Error ? error.message : 'reports unavailable');
    }
  };

  return (
    <section aria-label="Daemons and supervisor" className="surface-panel mt-3 rounded-[var(--radius-card)] p-3">
      <div className="flex items-center justify-between gap-2">
        <h2 className="type-label text-ink-900">Daemons · {daemons.length}</h2>
        <span className="type-mono-meta text-[9px] text-meta">
          {supervisor?.last_tick_at ? `tick ${supervisor.last_tick_at}` : 'supervisor idle'}
          {supervisor && supervisor.backoff_ms > 0 ? ` · backoff ${supervisor.backoff_ms}ms` : ''}
          {supervisor?.last_error_code ? ` · ${supervisor.last_error_code}` : ''}
        </span>
      </div>
      <p className="type-mono-meta mt-1 text-[9px] text-meta">
        Tree-free offices. No task row, no model call, no run cap.
      </p>
      {notice ? <p role="status" className="type-mono-meta mt-1 text-meta">{notice}</p> : null}
      <div className="mt-2 flex flex-wrap gap-2">
        <input
          className="type-mono-code h-8 min-w-[150px] flex-1 rounded-[var(--radius-sm)] border border-hairline bg-panel px-2 text-ink-900"
          value={name}
          maxLength={120}
          onChange={(event) => setName(event.target.value)}
          aria-label="Daemon display name"
        />
        <select
          className="type-mono-code h-8 rounded-[var(--radius-sm)] border border-hairline bg-panel px-2 text-ink-900"
          value={kind}
          onChange={(event) => setKind(event.target.value)}
          aria-label="Daemon kind"
        >
          <option value="watchdog">watchdog</option>
          <option value="relay">relay</option>
          <option value="switch">switch</option>
          <option value="satellite">satellite</option>
        </select>
        <input
          className="type-mono-code h-8 w-[76px] rounded-[var(--radius-sm)] border border-hairline bg-panel px-2 text-ink-900"
          value={interval}
          inputMode="numeric"
          onChange={(event) => setInterval(event.target.value.replace(/[^0-9]/g, ''))}
          aria-label="Daemon interval seconds"
        />
        <button
          type="button"
          className="type-label h-8 rounded-[var(--radius-button)] border border-honey-500 bg-honey-300 px-3 font-semibold text-ink-900 hover:bg-honey-400 disabled:opacity-50"
          disabled={busy}
          onClick={() => void create()}
        >
          Enroll daemon
        </button>
      </div>
      {status === 'loading' ? <p className="type-mono-meta mt-2 text-meta">Loading daemons…</p> : null}
      {status === 'error' ? <p role="alert" className="type-mono-meta mt-2 text-signal">Daemon lane unavailable; no card invented.</p> : null}
      {status === 'ok' && daemons.length === 0 ? <EmptyState message="No daemons enrolled." /> : null}
      <ul className="mt-2 space-y-1">
        {daemons.map((daemon) => (
          <li key={daemon.daemon_id} className="rounded-[var(--radius-md)] border border-hairline bg-panel-alt p-2">
            <div className="flex items-center justify-between gap-2">
              <span className="type-mono-code text-ink-900">{daemon.display_name}</span>
              <span className="type-mono-meta text-meta">{daemon.kind} · {daemon.state}</span>
            </div>
            <div className="type-mono-meta mt-1 text-meta">
              every {daemon.interval_secs}s · {daemon.owner_kind}:{daemon.owner_id}
              {daemon.last_report_at ? ` · last report ${daemon.last_report_at}` : ''}
              {daemon.last_error_code ? ` · ${daemon.last_error_code}` : ''}
            </div>
            <div className="mt-2 flex gap-1">
              <button
                type="button"
                className="type-label h-8 rounded-[var(--radius-button)] border border-hairline px-3 text-ink-800 hover:bg-honey-100 disabled:opacity-50"
                disabled={busy}
                onClick={() => void toggle(daemon)}
              >
                {daemon.state === 'running' ? 'Stop' : 'Start'}
              </button>
              <button
                type="button"
                className="type-label h-8 rounded-[var(--radius-button)] border border-hairline px-3 text-ink-800 hover:bg-honey-100 disabled:opacity-50"
                disabled={busy}
                onClick={() => void loadReports(daemon)}
              >
                Reports
              </button>
              <button
                type="button"
                className="type-label h-8 rounded-[var(--radius-button)] border border-signal px-3 text-signal hover:bg-recessed disabled:opacity-50"
                disabled={busy}
                onClick={() => void retire(daemon)}
              >
                Retire
              </button>
            </div>
            {reports[daemon.daemon_id] ? (
              <ul className="mt-1 space-y-0.5">
                {reports[daemon.daemon_id].slice(0, 5).map((report, index) => (
                  <li key={`${report.created_at}-${index}`} className="type-mono-meta text-[9px] text-meta">
                    {report.created_at} · {report.outcome}
                  </li>
                ))}
              </ul>
            ) : null}
          </li>
        ))}
      </ul>
    </section>
  );
}
