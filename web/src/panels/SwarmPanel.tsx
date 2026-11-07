/**
 * Swarm view: the HyperAgent tree and the live branch census are the same
 * object, so this surface renders the store's actual hierarchy rather than a
 * second graph implementation. It is read-only; selection is owned by the store.
 */
import { useUiStore } from '../state/context';
import { NODE_KIND_LABELS, isTerminal } from '../state/agentState';
import { STRINGS } from '../state/strings';
import { AgentHierarchy } from './AgentHierarchy';
import { PanelTitle } from './StatePack';
import { ProjectsPanel } from './ProjectsPanel';

export function SwarmPanel() {
  const tasks = useUiStore((state) => state.tasks);
  const selectedTaskId = useUiStore((state) => state.selectedTaskId);
  const selectTask = useUiStore((state) => state.selectTask);
  const setNav = useUiStore((state) => state.setNav);
  const project = useUiStore((state) => state.project);
  const setTab = useUiStore((state) => state.setTab);
  const list = Object.values(tasks);
  const counts = {
    superagents: list.filter((task) => task.kind === 'superagent').length,
    workers: list.filter((task) => task.kind === 'worker').length,
    tools: list.filter((task) => task.kind === 'tool').length,
    errors: list.filter((task) => task.kind === 'error').length,
    active: list.filter((task) => !isTerminal(task.state)).length,
  };

  return (
    <div className="h-full min-h-0 overflow-y-auto bg-canvas p-4">
      <div className="mx-auto flex w-full max-w-[1120px] flex-col gap-3">
        <PanelTitle>{STRINGS.tabs.swarm}</PanelTitle>
        <p className="type-body-l max-w-[720px] text-ink-500">
          The HyperAgent tree and the live swarm graph are one object. Re-parenting is only
          possible while a branch is not streaming; every edge is a bee line, never decoration.
        </p>

        <div className="grid grid-cols-2 gap-2 sm:grid-cols-5">
          {[
            ['SUB-HIVES', counts.superagents],
            ['WORKERS', counts.workers],
            ['TOOLS', counts.tools],
            ['ERRORS', counts.errors],
            ['ACTIVE', counts.active],
          ].map(([label, value]) => (
            <div key={label} className="surface-panel rounded-[12px] p-3">
              <div className="type-mono-meta text-[8.5px] tracking-[0.12em] text-meta">{label}</div>
              <div className="mt-1 font-display text-[26px] leading-none text-ink-900">{value}</div>
            </div>
          ))}
        </div>

        {list.length === 0 ? (
          <section className="relative overflow-hidden rounded-[16px] border border-hairline bg-panel p-6">
            <div
              aria-hidden="true"
              className="pointer-events-none absolute inset-0 opacity-50"
              style={{
                backgroundImage:
                  'radial-gradient(circle at 50% 45%, color-mix(in srgb, var(--color-honey-200) 70%, transparent), transparent 58%)',
              }}
            />
            <div className="relative flex flex-col items-start gap-4 sm:flex-row sm:items-center">
              <div className="flex h-16 w-16 shrink-0 items-center justify-center rounded-[18px] bg-honey-100 ring-1 ring-[var(--color-border-gold)]">
                <svg width="34" height="34" viewBox="0 0 34 34" aria-hidden="true">
                  <path d="M17 3 29 10v14L17 31 5 24V10Z" fill="var(--color-ink-900)" />
                  <path d="M17 10 23 13.5v7L17 24l-6-3.5v-7Z" fill="var(--color-honey-300)" />
                </svg>
              </div>
              <div className="min-w-0 flex-1">
                <h3 className="font-display text-[28px] leading-tight text-ink-900">The hive is idle</h3>
                <p className="mt-1 max-w-[620px] text-[12.5px] leading-[18px] text-ink-500">
                  There is no live branch to map yet. Start with a prompt in Workspace; the HyperAgent
                  will decompose it, then the swarm tree appears here with its real hierarchy.
                </p>
                <div className="mt-3 flex flex-wrap gap-2">
                  <button
                    type="button"
                    className="type-label h-9 rounded-[8px] border border-honey-500 bg-honey-300 px-3 font-semibold text-ink-900 hover:bg-honey-400"
                    onClick={() => setTab('workspace')}
                  >
                    Start in Workspace
                  </button>
                  <button
                    type="button"
                    className="type-label h-9 rounded-[8px] border border-hairline px-3 text-ink-900 hover:bg-honey-100"
                    onClick={() => setNav('agents')}
                  >
                    Open profiles &amp; memory
                  </button>
                </div>
              </div>
            </div>
            <div className="relative mt-5 grid gap-2 sm:grid-cols-3">
              {[
                ['01', 'Describe', 'Submit a prompt; no branch is fabricated locally.'],
                ['02', 'Map', 'The HyperAgent names the work and spawns sub-hives.'],
                ['03', 'Inspect', 'This view mirrors the live hierarchy and census.'],
              ].map(([step, title, body]) => (
                <div key={step} className="rounded-[10px] border border-hairline bg-panel-alt p-3">
                  <div className="type-mono-meta text-[8px] text-meta">STEP {step}</div>
                  <div className="mt-1 text-[12px] font-semibold text-ink-900">{title}</div>
                  <p className="mt-1 text-[11px] leading-[16px] text-ink-500">{body}</p>
                </div>
              ))}
            </div>
          </section>
        ) : (
          <div className="grid gap-3 lg:grid-cols-[minmax(0,1.35fr)_minmax(0,1fr)]">
            <section className="surface-panel rounded-[14px] p-3" aria-label="Agent hierarchy">
              <div className="type-label mb-2 text-ink-500">Hierarchy / workspace-jailed</div>
              <AgentHierarchy tasks={tasks} selectedTaskId={selectedTaskId} selectTask={selectTask} />
            </section>
            <section className="surface-panel rounded-[14px] p-3" aria-label="Swarm controls">
              <div className="type-label mb-2 text-ink-500">Surfaces</div>
              <p className="text-[12.5px] leading-[18px] text-ink-500">
                Profiles, memory and the cross-branch mailbox live in the Agents surface.
                Approvals are nonce-bound and appear as a blocking decision when a branch requests them.
              </p>
              <div className="mt-3 flex flex-wrap gap-2">
                <button
                  type="button"
                  className="type-label h-9 rounded-[8px] border border-hairline px-3 text-ink-900 hover:bg-honey-100"
                  onClick={() => setNav('agents')}
                >
                  Open agents
                </button>
                <button
                  type="button"
                  className="type-label h-9 rounded-[8px] border border-hairline px-3 text-ink-900 hover:bg-honey-100"
                  onClick={() => setNav('data')}
                >
                  Open files
                </button>
              </div>
              <dl className="mt-4 grid grid-cols-2 gap-2">
                {list.slice(0, 6).map((task) => (
                  <div key={task.id} className="rounded-[8px] border border-hairline bg-panel-alt p-2">
                    <dt className="type-mono-meta text-[8px] text-meta">{NODE_KIND_LABELS[task.kind]}</dt>
                    <dd className="mt-0.5 truncate text-[11.5px] font-medium text-ink-900">{task.label}</dd>
                  </div>
                ))}
              </dl>
            </section>
          </div>
        )}
        <ProjectsPanel currentProjectId={project.id} />
      </div>
    </div>
  );
}
