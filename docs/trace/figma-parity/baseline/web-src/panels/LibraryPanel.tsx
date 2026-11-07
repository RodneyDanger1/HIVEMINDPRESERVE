/** Left project/library rail: identity, agents, templates, state legend, settings, tip. */
import { useUiStore } from '../state/context';
import { STRINGS } from '../state/strings';
import { DOT_STATES, visualFor } from '../state/agentState';
import { StateGlyph } from '../flow/nodes';
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
  const settings = useUiStore((state) => state.settings);
  const insertTemplate = useUiStore((state) => state.insertTemplate);
  const active = Object.values(tasks);

  return (
    <aside
      aria-label={STRINGS.library.aria}
      className="hidden w-[300px] shrink-0 flex-col gap-5 overflow-y-auto border-r border-hairline bg-panel p-4 xl:flex"
    >
      <div>
        <div className="type-mono-meta tracking-wide text-meta">PROJECT</div>
        <div className="type-heading-h3 mt-1 truncate text-ink-900">{project.name}</div>
        <div className="type-mono-meta mt-1 text-meta">PROPOSED · v1 · {project.id.slice(0, 8)}</div>
      </div>

      <div className="border-t border-hairline pt-3">
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

      <div className="border-t border-hairline pt-3">
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
        <p className="type-mono-meta mt-1 text-meta">TEMPLATE_INSERT · licence-checked manifest</p>
      </div>

      <div className="border-t border-hairline pt-3">
        <div className="type-label text-ink-500">STATE VOCABULARY</div>
        <ul className="mt-1 space-y-1">
          {DOT_STATES.map((state) => (
            <li key={state} className="type-mono-meta flex items-center gap-2 text-meta">
              <StateGlyph state={state} />
              <span>{visualFor(state).label}</span>
            </li>
          ))}
        </ul>
        <p className="type-mono-meta mt-1 text-meta">colour is never the only channel</p>
      </div>

      <div className="border-t border-hairline pt-3">
        <div className="type-label text-ink-500">SETTINGS</div>
        <dl className="type-mono-meta mt-1 space-y-0.5 text-meta">
          <div className="flex justify-between gap-2">
            <dt>provider</dt>
            <dd className="text-ink-800">{settings.provider ?? '—'}</dd>
          </div>
          <div className="flex justify-between gap-2">
            <dt>model</dt>
            <dd className="truncate text-ink-800">{settings.model ?? '—'}</dd>
          </div>
          <div className="flex justify-between gap-2">
            <dt>key status</dt>
            <dd className="text-ink-800">
              {settings.keyPresent === null ? '—' : settings.keyPresent ? 'present' : 'missing'}
            </dd>
          </div>
        </dl>
      </div>

      <div className="mt-auto border-t border-hairline pt-3">
        <p className="type-mono-meta mb-1 text-meta" title={FIGCRAFT_GATE.notes.join(' | ')}>
          {FIGCRAFT_GATE.allowFrontend ? STRINGS.assets.accepted : STRINGS.assets.pending}
        </p>
        <p className="type-mono-meta text-meta">{STRINGS.library.tip}</p>
      </div>
    </aside>
  );
}
