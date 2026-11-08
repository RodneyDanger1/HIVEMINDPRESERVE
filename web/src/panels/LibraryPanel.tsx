/** Left project/library rail, matching the HiveMind spatial style system. */
import { useUiStore } from '../state/context';
import { STRINGS } from '../state/strings';
import { DOT_STATES, NODE_KIND_LABELS, isTerminal, visualFor } from '../state/agentState';
import { StateDot } from '../flow/nodes';
import { LAYOUT } from '../tokens/tokens';

const TEMPLATES: ReadonlyArray<{ id: string; label: string }> = [
  { id: 'content-writer', label: STRINGS.templates.contentWriter },
  { id: 'data-analyst', label: STRINGS.templates.dataAnalyst },
  { id: 'code-reviewer', label: STRINGS.templates.codeReviewer },
  { id: 'chatbot', label: STRINGS.templates.chatbot },
];

const TABS = [
  { id: 'workspace' as const, label: STRINGS.tabs.workspace },
  { id: 'swarm' as const, label: STRINGS.tabs.swarm },
  { id: 'routing' as const, label: STRINGS.tabs.routing },
  { id: 'history' as const, label: STRINGS.tabs.history },
];

/** The backend does not send labels; show a short id rather than an opaque
 * repeated "Task …" chip when the store's fallback label is kind + id. */
function agentChipLabel(task: { readonly id: string; readonly label: string; readonly kind: string }): string {
  const kindLabel = NODE_KIND_LABELS[task.kind as keyof typeof NODE_KIND_LABELS];
  const generated =
    task.label.startsWith('Agent ') ||
    (kindLabel !== undefined && task.label.startsWith(`${kindLabel} `));
  return generated ? `#${task.id.slice(-8)}` : task.label;
}

export function LibraryPanel() {
  const tasks = useUiStore((state) => state.tasks);
  const project = useUiStore((state) => state.project);
  const settings = useUiStore((state) => state.settings);
  const insertTemplate = useUiStore((state) => state.insertTemplate);
  const selectTask = useUiStore((state) => state.selectTask);
  const tab = useUiStore((state) => state.view.tab);
  const setTab = useUiStore((state) => state.setTab);
  const active = Object.values(tasks).filter(
    (task) => task.kind !== 'hyperagent' && !isTerminal(task.state),
  );

  return (
    <aside
      aria-label={STRINGS.library.aria}
      className="hidden shrink-0 flex-col gap-4 overflow-y-auto border-r border-hairline bg-panel-alt p-3 lg:flex"
      style={{ width: LAYOUT.library }}
    >
      <div className="flex h-8 items-center rounded-[8px] border border-hairline bg-panel px-2">
        <span className="truncate text-[11.5px] font-medium text-ink-800">
          {STRINGS.library.projectLabel}: {project.name}
        </span>
      </div>

      <div
        role="tablist"
        aria-label={STRINGS.tabs.aria}
        className="grid grid-cols-2 gap-1"
        onKeyDown={(event) => {
          if (!['ArrowLeft', 'ArrowRight', 'Home', 'End'].includes(event.key)) {
            return;
          }
          event.preventDefault();
          const current = TABS.findIndex((entry) => entry.id === tab);
          const next =
            event.key === 'Home'
              ? 0
              : event.key === 'End'
                ? TABS.length - 1
                : (current + (event.key === 'ArrowRight' ? 1 : -1) + TABS.length) % TABS.length;
          const target = TABS[next];
          if (target !== undefined) {
            setTab(target.id);
            requestAnimationFrame(() => document.getElementById(`library-tab-${target.id}`)?.focus());
          }
        }}
      >
        {TABS.map((entry) => (
          <button
            key={entry.id}
            id={`library-tab-${entry.id}`}
            type="button"
            role="tab"
            aria-selected={tab === entry.id}
            tabIndex={tab === entry.id ? 0 : -1}
            onClick={() => setTab(entry.id)}
            className={
              'type-label rounded-[7px] border px-1 py-1 text-[9px] ' +
              (tab === entry.id
                ? 'border-honey-400 bg-honey-100 text-ink-900'
                : 'border-transparent text-ink-500 hover:bg-panel-alt')
            }
          >
            {entry.label}
          </button>
        ))}
      </div>

      <div>
        <div className="text-[11.5px] font-semibold text-ink-900">{STRINGS.library.activeAgents}</div>
        {active.length === 0 ? (
          <p className="type-mono-meta mt-1 text-meta">{STRINGS.library.emptyAgents}</p>
        ) : (
          <ul className="mt-2 flex flex-wrap gap-2">
            {active.slice(0, 8).map((task) => (
              <li key={task.id} className="w-12">
                <button
                  type="button"
                  className="flex w-12 flex-col items-center gap-1 text-center"
                  onClick={() => selectTask(task.id)}
                  aria-label={`${task.label} (${visualFor(task.state).label})`}
                  title={task.label}
                >
                  <span
                    className="flex h-10 w-10 items-center justify-center bg-ink-900 text-honey-300"
                    style={{
                      clipPath: 'polygon(50% 0%, 100% 25%, 100% 75%, 50% 100%, 0% 75%, 0% 25%)',
                      boxShadow: 'inset 0 0 0 1.2px var(--color-border-gold)',
                    }}
                  >
                    <StateDot state={task.state} size={9} />
                  </span>
                  <span className="type-mono-meta block w-12 truncate text-[8px] text-ink-700">
                    {agentChipLabel(task)}
                  </span>
                </button>
              </li>
            ))}
          </ul>
        )}
      </div>

      <div>
        <div className="text-[11.5px] font-semibold text-ink-900">{STRINGS.library.templates}</div>
        <ul className="mt-2 space-y-1.5">
          {TEMPLATES.map((template) => (
            <li key={template.id}>
              <button
                type="button"
                className="flex h-[30px] w-full items-center gap-2 rounded-[7px] border border-hairline bg-panel px-2 text-left text-[11px] text-ink-800 hover:bg-honey-100"
                onClick={() => {
                  void insertTemplate(template.id);
                }}
              >
                <svg width="12" height="12" viewBox="0 0 16 16" aria-hidden="true">
                  <path d="m8 2 6 3.2-6 3.2-6-3.2Z M2 10.8 8 14l6-3.2" fill="none" stroke="var(--color-ink-500)" strokeWidth="1.2" />
                </svg>
                <span className="truncate">{template.label}</span>
              </button>
            </li>
          ))}
        </ul>
      </div>

      <div className="rounded-[8px] bg-honey-100 p-2">
        <div className="type-mono-meta text-meta">TIP</div>
        <p className="mt-1 text-[10.5px] leading-[15px] text-ink-700">{STRINGS.library.tip}</p>
      </div>

      <details className="mt-auto border-t border-hairline pt-2">
        <summary className="type-mono-meta cursor-pointer text-meta">State &amp; settings</summary>
        <ul className="mt-2 space-y-1">
          {DOT_STATES.map((state) => (
            <li key={state} className="type-mono-meta flex items-center gap-2 text-meta">
              <StateDot state={state} size={8} />
              <span>{visualFor(state).label}</span>
            </li>
          ))}
        </ul>
        <dl className="type-mono-meta mt-2 space-y-0.5 text-meta">
          <div className="flex justify-between gap-2">
            <dt>provider</dt>
            <dd className="truncate text-ink-800">{settings.provider ?? '—'}</dd>
          </div>
          <div className="flex justify-between gap-2">
            <dt>model</dt>
            <dd className="truncate text-ink-800">{settings.model ?? '—'}</dd>
          </div>
          <div className="flex justify-between gap-2">
            <dt>key</dt>
            <dd className="text-ink-800">
              {settings.keyPresent === null ? '—' : settings.keyPresent ? 'present' : 'missing'}
            </dd>
          </div>
        </dl>
        <p className="type-mono-meta mt-2 text-meta">
          HiveMind v1.2-dev
        </p>
      </details>
    </aside>
  );
}
