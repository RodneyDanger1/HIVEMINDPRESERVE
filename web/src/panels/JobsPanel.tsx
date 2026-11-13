/**
 * Background jobs surface (CAP-5).
 *
 * Reads the real `/api/jobs` route, shows the persisted/live output tail, and
 * can kill a running job after an explicit confirmation. It does not invent a
 * job, a PID, or a terminal: starting jobs stays on the CLI/HTTP path until a
 * dedicated start surface is wired.
 */
import { useCallback, useEffect, useState } from 'react';
import { useUiStore } from '../state/context';
import { EmptyState, ErrorState, LoadingState, PanelTitle } from './StatePack';

type JobState = 'running' | 'succeeded' | 'failed' | 'timed_out' | 'killed';

interface JobRow {
  readonly id: string;
  readonly session_id: string;
  readonly program: string;
  readonly args: readonly string[];
  readonly cwd: string;
  readonly state: JobState;
  readonly exit_code: number | null;
  readonly output?: { readonly tail?: string; readonly bytes?: number; readonly truncated?: boolean };
}

async function jobRequest(path: string, init?: RequestInit): Promise<Response> {
  const handshake = await fetch('/api/session', { headers: { accept: 'application/json' } });
  if (!handshake.ok) throw new Error('session unavailable');
  const token = (await handshake.json()) as { token?: string };
  const headers: Record<string, string> = { accept: 'application/json', ...(init?.headers as Record<string, string> | undefined) };
  if (token.token) headers.Authorization = `Bearer ${token.token}`;
  return fetch(path, { ...init, headers });
}

function splitCommand(input: string): string[] | null {
  const tokens: string[] = [];
  const pattern = /"([^"]*)"|'([^']*)'|(\S+)/g;
  let match: RegExpExecArray | null;
  while ((match = pattern.exec(input)) !== null) {
    tokens.push(match[1] ?? match[2] ?? match[3] ?? '');
  }
  return tokens.length > 0 ? tokens : null;
}

export function JobsPanel() {
  const connection = useUiStore((state) => state.connection);
  const activeSessionId = useUiStore((state) => state.activeSessionId);
  const [jobs, setJobs] = useState<readonly JobRow[] | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [selected, setSelected] = useState<string | null>(null);
  const [busy, setBusy] = useState<string | null>(null);
  const [command, setCommand] = useState('');
  const [startError, setStartError] = useState<string | null>(null);
  const [starting, setStarting] = useState(false);

  const load = useCallback(async () => {
    try {
      const response = await jobRequest('/api/jobs');
      if (!response.ok) throw new Error(`GET /api/jobs returned ${response.status}`);
      const body = (await response.json()) as { jobs?: readonly JobRow[] };
      setJobs(body.jobs ?? []);
      setError(null);
    } catch (failure) {
      setError(failure instanceof Error ? failure.message : 'jobs unavailable');
    }
  }, []);

  useEffect(() => {
    if (connection !== 'open') return;
    void load();
    const timer = window.setInterval(() => void load(), 2_000);
    return () => window.clearInterval(timer);
  }, [connection, load]);

  const kill = async (job: JobRow): Promise<void> => {
    if (!window.confirm(`Kill ${job.program} ${job.args.join(' ')}? This terminates its process tree.`)) {
      return;
    }
    setBusy(job.id);
    try {
      const response = await jobRequest(`/api/jobs/${encodeURIComponent(job.id)}/kill`, {
        method: 'POST',
        headers: { 'content-type': 'application/json' },
        body: '{}',
      });
      if (!response.ok) throw new Error(`kill returned ${response.status}`);
      await load();
    } catch (failure) {
      setError(failure instanceof Error ? failure.message : 'kill failed');
    } finally {
      setBusy(null);
    }
  };

  const start = async (): Promise<void> => {
    if (!activeSessionId) {
      setStartError('Select a named session in the composer first.');
      return;
    }
    const tokens = splitCommand(command);
    if (tokens === null) {
      setStartError('Enter an allowlisted command, for example: git.exe status');
      return;
    }
    // The server allowlist is exact (bare *.exe names). Accept the natural
    // spelling for the two shipped tools without widening the server policy.
    const program =
      tokens[0] === 'git' ? 'git.exe' : tokens[0] === 'cargo' ? 'cargo.exe' : tokens[0];
    setStarting(true);
    setStartError(null);
    try {
      const response = await jobRequest('/api/jobs', {
        method: 'POST',
        headers: { 'content-type': 'application/json' },
        body: JSON.stringify({
          session_id: activeSessionId,
          program,
          args: tokens.slice(1),
        }),
      });
      const body = (await response.json().catch(() => null)) as JobRow | { message?: string } | null;
      if (!response.ok) {
        throw new Error(
          body !== null && 'message' in body && body.message
            ? body.message
            : `job start returned ${response.status}`,
        );
      }
      if (body !== null && 'id' in body) setSelected(body.id);
      setCommand('');
      await load();
    } catch (failure) {
      setStartError(failure instanceof Error ? failure.message : 'job start failed');
    } finally {
      setStarting(false);
    }
  };

  const chosen = jobs?.find((job) => job.id === selected) ?? null;
  return (
    <section aria-label="Background jobs" className="surface-panel mt-3 rounded-[var(--radius-card)] p-3">
      <PanelTitle>Background jobs</PanelTitle>
      <form
        className="mt-2 flex flex-wrap items-center gap-1"
        onSubmit={(event) => {
          event.preventDefault();
          void start();
        }}
      >
        <label className="sr-only" htmlFor="job-command">
          Allowlisted command
        </label>
        <input
          id="job-command"
          className="type-mono-code min-w-0 flex-1 rounded-[6px] border border-hairline bg-panel px-2 py-1 text-[11px] text-ink-900"
          placeholder="git.exe status"
          value={command}
          onChange={(event) => setCommand(event.target.value)}
          disabled={starting}
        />
        <button
          type="submit"
          className="rounded-[6px] border border-honey-500 bg-honey-300 px-2 py-1 text-[11px] font-semibold text-ink-900 disabled:opacity-50 max-[640px]:min-h-11 max-[640px]:min-w-11"
          disabled={starting || command.trim().length === 0 || !activeSessionId}
          title={activeSessionId ? 'Run through the allowlisted job registry' : 'Select a named session in the composer first'}
        >
          Run
        </button>
      </form>
      <p className="type-mono-meta mt-1 text-[10px] text-meta">
        Command console — allowlisted programs only (`git`, `cargo`), no PTY/shell. Select a named
        session in the composer to enable it.
      </p>
      {startError !== null ? <ErrorState message={startError} /> : null}
      {connection !== 'open' ? <EmptyState message="Jobs load when the local server is connected." /> : null}
      {connection === 'open' && jobs === null && error === null ? <LoadingState /> : null}
      {error !== null ? <ErrorState message={error} /> : null}
      {connection === 'open' && jobs !== null && jobs.length === 0 ? <EmptyState message="No background jobs yet. Start one from the CLI." /> : null}
      {jobs !== null && jobs.length > 0 ? (
        <ul className="mt-2 space-y-1" aria-label="Job list">
          {jobs.map((job) => (
            <li key={job.id} className="rounded-[8px] border border-hairline bg-panel p-2">
              <div className="flex flex-wrap items-center gap-2">
                <button
                  type="button"
                  className="type-mono-code min-w-0 flex-1 truncate text-left text-[12px] text-ink-900"
                  aria-pressed={selected === job.id}
                  onClick={() => setSelected(selected === job.id ? null : job.id)}
                >
                  {job.program} {job.args.join(' ')}
                </button>
                <span className="type-mono-meta text-[10px] text-meta">{job.state}</span>
                <span className="type-mono-meta text-[10px] text-meta">
                  exit {job.exit_code === null ? 'unknown' : job.exit_code}
                </span>
                {job.state === 'running' ? (
                  <button
                    type="button"
                    className="rounded-[6px] border border-signal px-1.5 py-0.5 text-[10px] text-signal disabled:opacity-50"
                    disabled={busy === job.id}
                    onClick={() => void kill(job)}
                  >
                    Kill
                  </button>
                ) : null}
              </div>
              {selected === job.id ? (
                <div className="mt-1">
                  <div className="type-mono-meta text-[10px] text-meta">
                    session {job.session_id} · cwd {job.cwd} · bytes {job.output?.bytes ?? 0}
                    {job.output?.truncated ? ' · tail truncated' : ''}
                  </div>
                  <pre className="type-mono-code mt-1 max-h-40 overflow-auto whitespace-pre-wrap text-[11px] text-ink-800">
                    {job.output?.tail?.trim() ? job.output.tail : 'No output captured.'}
                  </pre>
                </div>
              ) : null}
            </li>
          ))}
        </ul>
      ) : null}
      {chosen === null && jobs !== null && jobs.length > 0 ? (
        <p className="type-mono-meta mt-1 text-[10px] text-meta">Select a job to inspect its output tail.</p>
      ) : null}
    </section>
  );
}
