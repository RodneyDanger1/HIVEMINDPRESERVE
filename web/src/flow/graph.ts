/**
 * Pure graph projection: TaskRecord map -> React Flow nodes (dagre layout) and
 * edges (parent -> child). Positions are deterministic for a given graph; the
 * canvas owns only drag offsets. No capability is invented: nodes without
 * server metadata keep the generic kind/label from the store.
 */
import dagre from '@dagrejs/dagre';
import type { Edge, Node } from '@xyflow/react';
import type { TaskRecord } from '../state/store';
import { isAgentState, type NodeKindLabel } from '../state/agentState';

export interface CanvasNodeData extends Record<string, unknown> {
  readonly task: TaskRecord;
  readonly selected: boolean;
  readonly onSelect: (taskId: string) => void;
}

export type CanvasNode = Node<CanvasNodeData>;

export interface Point2 {
  readonly x: number;
  readonly y: number;
}

const NODE_SIZE: Record<NodeKindLabel, { width: number; height: number }> = {
  hyperagent: { width: 280, height: 150 },
  superagent: { width: 232, height: 132 },
  worker: { width: 208, height: 112 },
  tool: { width: 190, height: 104 },
  error: { width: 232, height: 120 },
};

export function layoutTasks(tasks: Readonly<Record<string, TaskRecord>>): Record<string, Point2> {
  const graph = new dagre.graphlib.Graph();
  graph.setGraph({ rankdir: 'LR', nodesep: 40, ranksep: 90, marginx: 40, marginy: 40 });
  graph.setDefaultEdgeLabel(() => ({}));
  const ids = Object.keys(tasks);
  for (const id of ids) {
    const task = tasks[id];
    const size = NODE_SIZE[task.kind];
    // Dagre mutates the label it is given (adds x/y). Passing the shared size
    // object makes every same-kind node inherit the last computed x/y.
    graph.setNode(id, { width: size.width, height: size.height });
  }
  for (const id of ids) {
    const parentId = tasks[id].parentId;
    if (parentId && tasks[parentId]) {
      graph.setEdge(parentId, id);
    }
  }
  dagre.layout(graph);
  const positions: Record<string, Point2> = {};
  for (const id of ids) {
    const node = graph.node(id) as { x: number; y: number; width: number; height: number };
    positions[id] = { x: node.x - node.width / 2, y: node.y - node.height / 2 };
  }
  return positions;
}

export function buildNodes(
  tasks: Readonly<Record<string, TaskRecord>>,
  selectedTaskId: string | null,
  onSelect: (taskId: string) => void,
  interactive = true,
): CanvasNode[] {
  const positions = layoutTasks(tasks);
  return Object.values(tasks).map((task) => ({
    id: task.id,
    type: task.kind,
    position: positions[task.id] ?? { x: 0, y: 0 },
    data: { task, selected: task.id === selectedTaskId, onSelect },
    ariaLabel: `${task.label} (${task.kind}, ${task.state})`,
    draggable: interactive,
    selectable: interactive,
  }));
}

export function buildEdges(tasks: Readonly<Record<string, TaskRecord>>): Edge[] {
  return Object.values(tasks)
    .filter((task) => task.parentId !== null && tasks[task.parentId] !== undefined)
    .map((task) => ({
      id: `edge-${task.parentId}-${task.id}`,
      source: task.parentId ?? '',
      target: task.id,
      type: 'bee',
      animated: false,
    }));
}

/**
 * Projects a stored `HISTORY_SNAPSHOT` graph into read-only task records.
 * The snapshot records only id/parent/state/rev; no role or progress is
 * invented. Unknown/malformed nodes are skipped rather than crashing the
 * canvas or showing a fabricated state.
 */
export function buildReplayTasks(graph: unknown): Record<string, TaskRecord> {
  if (!graph || typeof graph !== 'object') {
    return {};
  }
  const rawNodes = (graph as { nodes?: unknown }).nodes;
  if (!Array.isArray(rawNodes)) {
    return {};
  }
  const tasks: Record<string, TaskRecord> = {};
  for (const raw of rawNodes) {
    if (!raw || typeof raw !== 'object') continue;
    const node = raw as {
      task_id?: unknown;
      parent_task_id?: unknown;
      state?: unknown;
      rev?: unknown;
    };
    if (typeof node.task_id !== 'string' || !isAgentState(node.state)) continue;
    const parentId = typeof node.parent_task_id === 'string' ? node.parent_task_id : null;
    tasks[node.task_id] = {
      id: node.task_id,
      parentId,
      kind: 'worker',
      label: `task ${node.task_id.slice(0, 8)}`,
      state: node.state,
      progress: null,
      errorCode: null,
      rev: typeof node.rev === 'number' ? node.rev : 0,
    };
  }
  // Drop parent references that are not present in this snapshot; otherwise
  // the graph would point at a node the reader cannot see.
  for (const task of Object.values(tasks)) {
    if (task.parentId !== null && tasks[task.parentId] === undefined) {
      tasks[task.id] = { ...task, parentId: null };
    }
  }
  return tasks;
}

export function taskCounts(tasks: Readonly<Record<string, TaskRecord>>): {
  readonly byKind: Readonly<Record<NodeKindLabel, number>>;
} {
  const byKind: Record<NodeKindLabel, number> = {
    hyperagent: 0,
    superagent: 0,
    worker: 0,
    tool: 0,
    error: 0,
  };
  for (const task of Object.values(tasks)) {
    byKind[task.kind] += 1;
  }
  return { byKind };
}
