/** Routing table (derived from the graph's parent edges; no invented routes). */
import { useUiStore } from '../state/context';
import { STRINGS } from '../state/strings';
import { NODE_KIND_LABELS, visualFor } from '../state/agentState';
import { PanelTitle, EmptyState } from './StatePack';

export function RoutingPanel() {
  const tasks = useUiStore((state) => state.tasks);
  const edges = Object.values(tasks).filter((task) => task.parentId !== null);
  return (
    <div className="h-full min-h-0 overflow-y-auto p-4">
      <PanelTitle>{STRINGS.panels.routingTitle}</PanelTitle>
      {edges.length === 0 ? <EmptyState message={STRINGS.panels.routingEmpty} /> : null}
      <table className="w-full border-collapse">
        <thead>
          <tr className="type-label text-ink-500">
            <th className="border-b border-hairline px-2 py-1 text-left">from</th>
            <th className="border-b border-hairline px-2 py-1 text-left">to</th>
            <th className="border-b border-hairline px-2 py-1 text-left">kind</th>
            <th className="border-b border-hairline px-2 py-1 text-left">{STRINGS.node.role}</th>
          </tr>
        </thead>
        <tbody>
          {edges.map((task) => {
            const parent = task.parentId ? tasks[task.parentId] : null;
            return (
              <tr key={task.id} className="type-mono-code text-ink-900">
                <td className="border-b border-hairline px-2 py-1">{parent?.label ?? STRINGS.node.noMetrics}</td>
                <td className="border-b border-hairline px-2 py-1">{task.label}</td>
                <td className="border-b border-hairline px-2 py-1">{NODE_KIND_LABELS[task.kind]}</td>
                <td className="border-b border-hairline px-2 py-1">{visualFor(task.state).label}</td>
              </tr>
            );
          })}
        </tbody>
      </table>
    </div>
  );
}
