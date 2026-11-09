/**
 * Projects card: reads the real `/api/projects` registry. It is read-only on
 * purpose — the server exposes no create/rename/switch route in v1, so this
 * surface never presents a dead control.
 */
import { useEffect, useState } from 'react';
import { apiBaseUrl } from '../api/client';

export interface ProjectEntry {
  readonly id: string;
  readonly name: string;
  readonly created_at: string;
}

type State =
  | { readonly kind: 'loading' }
  | { readonly kind: 'ok'; readonly projects: readonly ProjectEntry[] }
  | { readonly kind: 'error'; readonly message: string };

export function ProjectsPanel({ currentProjectId }: { currentProjectId: string }) {
  const [state, setState] = useState<State>({ kind: 'loading' });

  useEffect(() => {
    const controller = new AbortController();
    (async () => {
      try {
        const response = await fetch(`${apiBaseUrl()}/api/projects`, {
          signal: controller.signal,
        });
        if (!response.ok) {
          setState({ kind: 'error', message: `Project registry refused (HTTP ${response.status}).` });
          return;
        }
        const payload = (await response.json()) as { projects?: ProjectEntry[] };
        setState({ kind: 'ok', projects: payload.projects ?? [] });
      } catch (error) {
        if (!controller.signal.aborted) {
          setState({
            kind: 'error',
            message: error instanceof Error ? error.message : 'Project registry unreachable.',
          });
        }
      }
    })();
    return () => controller.abort();
  }, []);

  return (
    <section className="surface-panel rounded-[var(--radius-card)] p-3" aria-label="Projects">
      <div className="type-label mb-2 text-ink-500">Projects</div>
      {state.kind === 'loading' ? (
        <p className="type-mono-meta text-meta">Reading project registry…</p>
      ) : state.kind === 'error' ? (
        <p className="type-mono-meta text-signal">{state.message}</p>
      ) : state.projects.length === 0 ? (
        <p className="type-mono-meta text-meta">
          No projects committed yet. The first run creates one.
        </p>
      ) : (
        <ul className="space-y-1">
          {state.projects.slice(0, 8).map((project) => (
            <li
              key={project.id}
              className={
                'flex items-center justify-between gap-2 rounded-[var(--radius-sm)] border px-2 py-1 ' +
                (project.id === currentProjectId
                  ? 'border-honey-400 bg-honey-100'
                  : 'border-hairline bg-panel')
              }
            >
              <span className="type-mono-meta truncate text-[10px] text-ink-800">{project.name || project.id}</span>
              <span className="type-mono-meta shrink-0 text-[11px] text-meta">
                {project.id === currentProjectId ? 'CURRENT' : 'committed'}
              </span>
            </li>
          ))}
        </ul>
      )}
    </section>
  );
}
