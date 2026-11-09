/**
 * Hierarchical HMAS topology derived only from task parent links. No layout
 * fiction: a node is indented by its real depth and selects the existing card.
 * Every row can request SPAWN for that exact parent; the button is disabled
 * with the real reason whenever the class matrix or run state forbids it.
 */
import { useMemo } from 'react';
import { NODE_KIND_LABELS } from '../state/agentState';
import { spawnPlanFor } from '../state/spawnMatrix';
import type { TaskRecord } from '../state/store';
import { StateGlyph } from '../flow/nodes';

interface AgentHierarchyProps {
  readonly tasks: Readonly<Record<string, TaskRecord>>;
  readonly selectedTaskId: string | null;
  readonly selectTask: (taskId: string | null) => void;
  /**
   * Sends SPAWN_SUB_HIVE with this task as the exact parent. Absent on a
   * replay/replay-only surface, where spawning is refused by construction.
   */
  readonly spawnTask?: (taskId: string) => void;
  /** Visible reason when this task cannot spawn; null means it can. */
  readonly spawnReason?: (task: TaskRecord) => string | null;
}

interface TreeNode {
  readonly task: TaskRecord;
  readonly depth: number;
}

const REPLAY_REASON = 'Replay is read-only. Exit replay to spawn a child.';

export function AgentHierarchy({
  tasks,
  selectedTaskId,
  selectTask,
  spawnTask,
  spawnReason,
}: AgentHierarchyProps) {
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
        {rows.map(({ task, depth }) => {
          const plan = spawnPlanFor(task);
          const reason =
            spawnTask === undefined
              ? REPLAY_REASON
              : (spawnReason?.(task) ?? (plan === null ? 'This task class cannot spawn.' : null));
          const childKind = plan?.childKind ?? null;
          return (
            <li key={task.id}>
              <div
                className={
                  'surface-panel flex w-full items-center gap-2 rounded-[var(--radius-sm)] px-2 py-1 ' +
                  (task.id === selectedTaskId ? 'border-honey-500' : '')
                }
                style={{ marginLeft: `${depth * 14}px` }}
              >
                <button
                  type="button"
                  aria-pressed={task.id === selectedTaskId}
                  onClick={() => selectTask(task.id)}
                  className="flex min-w-0 flex-1 items-center gap-2 text-left"
                >
                  <span className="type-mono-meta text-meta">{depth === 0 ? '◆' : '└'}</span>
                  <StateGlyph state={task.state} />
                  <span className="type-mono-code min-w-0 flex-1 truncate text-ink-900">{task.label}</span>
                  <span className="type-mono-meta text-meta">{NODE_KIND_LABELS[task.kind]}</span>
                </button>
                <button
                  type="button"
                  data-spawn-parent={task.id}
                  aria-label={`Spawn a ${childKind ?? 'child'} under ${task.label}`}
                  disabled={reason !== null}
                  title={
                    reason ??
                    `Spawn a ${childKind ?? 'child'} under this ${NODE_KIND_LABELS[task.kind]} (SPAWN_SUB_HIVE).`
                  }
                  onClick={() => {
                    if (reason === null) {
                      spawnTask?.(task.id);
                    }
                  }}
                  className="type-mono-meta shrink-0 rounded-[var(--radius-sm)] border border-hairline px-1.5 py-0.5 text-[10px] text-ink-800 hover:bg-honey-100 disabled:cursor-not-allowed disabled:opacity-50"
                >
                  SPAWN
                </button>
              </div>
              {reason === null ? null : (
                <p
                  className="type-mono-meta mt-0.5 text-[10px] text-meta"
                  style={{ marginLeft: `${depth * 14 + 8}px` }}
                >
                  SPAWN disabled: {reason}
                </p>
              )}
            </li>
          );
        })}
      </ul>
    </section>
  );
}
