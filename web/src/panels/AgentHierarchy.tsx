/**
 * Hierarchical HMAS topology derived only from task parent links. No layout
 * fiction: a node is indented by its real depth and selects the existing card.
 */
import { useMemo } from 'react';
import { NODE_KIND_LABELS } from '../state/agentState';
import type { TaskRecord } from '../state/store';
import { StateGlyph } from '../flow/nodes';

interface AgentHierarchyProps {
  readonly tasks: Readonly<Record<string, TaskRecord>>;
  readonly selectedTaskId: string | null;
  readonly selectTask: (taskId: string | null) => void;
}

interface TreeNode {
  readonly task: TaskRecord;
  readonly depth: number;
}

export function AgentHierarchy({ tasks, selectedTaskId, selectTask }: AgentHierarchyProps) {
  const rows = useMemo<readonly TreeNode[]>(() => {
    const byParent = new Map<string | null, TaskRecord[]>();
    for (const task of Object.values(tasks)) {
      const parent = task.parentId ?? null;
      const bucket = byParent.get(parent) ?? [];
      bucket.push(task);
      byParent.set(parent, bucket);
    }
    for (const bucket of byParent.values()) {
      bucket.sort((left, right) => left.label.localeCompare(right.label) || left.id.localeCompare(right.id));
    }
    const rows: TreeNode[] = [];
    const visited = new Set<string>();
    const walk = (parentId: string | null, depth: number): void => {
      for (const task of byParent.get(parentId) ?? []) {
        if (visited.has(task.id)) {
          continue;
        }
        visited.add(task.id);
        rows.push({ task, depth });
        walk(task.id, depth + 1);
      }
    };
    walk(null, 0);
    // A cycle or orphan would otherwise be invisible; show it as a root row.
    for (const task of Object.values(tasks)) {
      if (!visited.has(task.id)) {
        rows.push({ task, depth: 0 });
      }
    }
    return rows;
  }, [tasks]);

  return (
    <section aria-label="Agent hierarchy" className="mb-4">
      <h3 className="type-label text-ink-900">Hierarchy</h3>
      <ul className="mt-2 space-y-1">
        {rows.map(({ task, depth }) => (
          <li key={task.id}>
            <button
              type="button"
              aria-pressed={task.id === selectedTaskId}
              onClick={() => selectTask(task.id)}
              className={
                'surface-panel flex w-full items-center gap-2 rounded-[var(--radius-sm)] px-2 py-1 text-left ' +
                (task.id === selectedTaskId ? 'border-honey-500' : '')
              }
              style={{ marginLeft: `${depth * 14}px` }}
            >
              <span className="type-mono-meta text-meta">{depth === 0 ? '◆' : '└'}</span>
              <StateGlyph state={task.state} />
              <span className="type-mono-code min-w-0 flex-1 truncate text-ink-900">{task.label}</span>
              <span className="type-mono-meta text-meta">{NODE_KIND_LABELS[task.kind]}</span>
            </button>
          </li>
        ))}
      </ul>
    </section>
  );
}
