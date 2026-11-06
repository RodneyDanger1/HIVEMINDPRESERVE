/**
 * S15-S19 agent registry (read/derived): one card per task, single state vocabulary.
 *
 * W12: the agents surface also hosts the tool-activity region. The store owns
 * `toolActivity`/`approvals` (server frames only); this panel passes the entries
 * to the panel owner's `ToolActivityPanel` and exposes a labelled region so
 * keyboard/screen-reader users find the activity list. No optimistic rows are
 * created here and this panel performs no store writes.
 */
import { useUiStore } from '../state/context';
import { NODE_KIND_LABELS } from '../state/agentState';
import { STRINGS, formatPercent } from '../state/strings';
import { StateGlyph } from '../flow/nodes';
import { PanelTitle, EmptyState } from './StatePack';
import { ToolActivityPanel, type ToolActivityEntry } from './ToolActivityPanel';

export function AgentsPanel() {
  const tasks = useUiStore((state) => state.tasks);
  const selectTask = useUiStore((state) => state.selectTask);
  const selectedTaskId = useUiStore((state) => state.selectedTaskId);
  // Structural reconciliation: the store's ToolActivityRecord is assignable to
  // the panel's ToolActivityEntry (same id/tool/hash/turn/outcome fields); the
  // store stays the single writer and this panel only reads.
  const toolActivity: readonly ToolActivityEntry[] = useUiStore((state) => state.toolActivity);
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
      <div
        role="region"
        aria-label={STRINGS.toolActivity.regionAria}
        data-testid="agents-tool-activity"
        className="mt-4 border-t border-hairline pt-4"
      >
        <ToolActivityPanel entries={toolActivity} />
      </div>
    </div>
  );
}
