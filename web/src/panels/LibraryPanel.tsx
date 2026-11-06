/** S12 library, 260px: project identity, derived active agents, template inserts, tip. */
import { useUiStore } from '../state/context';
import { STRINGS } from '../state/strings';
import { visualFor } from '../state/agentState';
import { FIGCRAFT_GATE } from '../assets/manifest';

const TEMPLATES: ReadonlyArray<{ id: string; label: string }> = [
  { id: 'content-writer', label: STRINGS.templates.contentWriter },
  { id: 'data-analyst', label: STRINGS.templates.dataAnalyst },
  { id: 'code-reviewer', label: STRINGS.templates.codeReviewer },
  { id: 'chatbot', label: STRINGS.templates.chatbot },
];

export function LibraryPanel() {
  const tasks = useUiStore((state) => state.tasks);
  const project = useUiStore((state) => state.project);
  const insertTemplate = useUiStore((state) => state.insertTemplate);
  const active = Object.values(tasks);

  return (
    <aside
      aria-label={STRINGS.library.aria}
      className="hidden w-[260px] shrink-0 flex-col gap-4 border-l border-hairline bg-panel p-3 xl:flex"
    >
      <div>
        <div className="type-label text-ink-500">{STRINGS.library.projectLabel}</div>
        <div className="type-mono-code mt-1 truncate text-ink-900">{project.name}</div>
        <div className="type-mono-meta text-meta">{project.id.slice(0, 8)}</div>
      </div>
      <div>
        <div className="type-label text-ink-500">
          {STRINGS.library.activeAgents} · {active.length}
        </div>
        {active.length === 0 ? (
          <p className="type-mono-meta mt-1 text-meta">{STRINGS.library.emptyAgents}</p>
        ) : (
          <ul className="mt-1 space-y-1">
            {active.map((task) => (
              <li key={task.id} className="type-mono-code flex items-center justify-between gap-2 truncate">
                <span className="truncate">{task.label}</span>
                <span className="type-mono-meta text-meta">{visualFor(task.state).label}</span>
              </li>
            ))}
          </ul>
        )}
      </div>
      <div>
        <div className="type-label text-ink-500">{STRINGS.library.templates}</div>
        <ul className="mt-1 space-y-1">
          {TEMPLATES.map((template) => (
            <li key={template.id}>
              <button
                type="button"
                className="type-mono-code w-full rounded-[var(--radius-sm)] border border-hairline bg-panel-alt px-2 py-1 text-left hover:bg-honey-100"
                onClick={() => {
                  void insertTemplate(template.id);
                }}
              >
                {template.label}
              </button>
            </li>
          ))}
        </ul>
      </div>
      <div className="mt-auto">
        {FIGCRAFT_GATE.allowFrontend ? null : (
          <p className="type-mono-meta mb-2 text-meta" title={FIGCRAFT_GATE.evidence.join(' | ')}>
            {STRINGS.assets.pending}
          </p>
        )}
        <p className="type-mono-meta text-meta">{STRINGS.library.tip}</p>
      </div>
    </aside>
  );
}
