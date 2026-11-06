/** S15-S19 agent registry (read/derived): one card per task, single state vocabulary. */
import { useUiStore } from '../state/context';
import { NODE_KIND_LABELS } from '../state/agentState';
import { STRINGS, formatPercent } from '../state/strings';
import { StateGlyph } from '../flow/nodes';
import { PanelTitle, EmptyState } from './StatePack';

export function AgentsPanel() {
  const tasks = useUiStore((state) => state.tasks);
  const selectTask = useUiStore((state) => state.selectTask);
  const selectedTaskId = useUiStore((state) => state.selectedTaskId);
  const list = Object.values(tasks);

  return (
    <div className="h-full min-h-0 overflow-y-auto p-4">
      <PanelTitle>
        {STRINGS.panels.agentsTitle} · {list.length}
      </PanelTitle>
      {list.length === 0 ? <EmptyState message={STRINGS.panels.agentsEmpty} /> : null}
      <ul className="grid grid-cols-1 gap-2 sm:grid-cols-2 lg:grid-cols-3">
        {list.map((task) => (
          <li key={task.id}>
            <button
              type="button"
              aria-pressed={task.id === selectedTaskId}
              onClick={() => selectTask(task.id)}
              className={
                'surface-panel block w-full rounded-[var(--radius-card)] p-2 text-left ' +
                (task.id === selectedTaskId ? 'border-honey-500' : '')
              }
            >
              <div className="flex items-center justify-between gap-2">
                <span className="type-label text-ink-500">{NODE_KIND_LABELS[task.kind]}</span>
                <StateGlyph state={task.state} />
              </div>
              <div className="type-mono-code mt-1 truncate text-ink-900">{task.label}</div>
              <div className="type-mono-meta mt-1 flex justify-between text-meta">
                <span>{task.id.slice(0, 8)}</span>
                <span>{task.progress === null ? STRINGS.node.noMetrics : formatPercent(task.progress, 0)}</span>
              </div>
            </button>
          </li>
        ))}
      </ul>
    </div>
  );
}
