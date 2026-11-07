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
import { LAYOUT } from '../tokens/tokens';

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

/** Invisible edge anchor at the hive core. The backend hyperagent task renders
 * as the fixed ChatCore overlay, not as a second 420x500 canvas card. */
export const HIVE_ANCHOR_ID = 'hive-core-anchor';

const NODE_SIZE: Record<NodeKindLabel, { width: number; height: number }> = {
  hyperagent: { width: LAYOUT.chat.width, height: LAYOUT.chat.height },
  superagent: LAYOUT.cards.superagent,
  worker: LAYOUT.cards.worker,
  tool: LAYOUT.cards.tool,
  error: LAYOUT.cards.error,
};

export function nodeSize(kind: NodeKindLabel): { readonly width: number; readonly height: number } {
  return NODE_SIZE[kind];
}

export function layoutTasks(tasks: Readonly<Record<string, TaskRecord>>): Record<string, Point2> {
  const graph = new dagre.graphlib.Graph();
  graph.setGraph({ rankdir: 'TB', nodesep: 40, ranksep: 90, marginx: 40, marginy: 40 });
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
  // World origin is the centre of the hyper/hive node (spatial-canvas spec).
  // Shift every dagre coordinate so the root's centre is exactly (0,0); the
  // fixed chat overlay and Ctrl+0 recentre then share the same anchor.
  const root = ids.find((id) => tasks[id]?.parentId === null);
  if (root !== undefined) {
    const rootTask = tasks[root];
    const size = NODE_SIZE[rootTask.kind];
    const rootPosition = positions[root];
    const dx = rootPosition.x + size.width / 2;
    const dy = rootPosition.y + size.height / 2;
    for (const id of ids) {
      positions[id] = { x: positions[id].x - dx, y: positions[id].y - dy };
    }
  }

  // The chat core is a fixed screen overlay centred on the hive. Dagre's tree
  // can otherwise place child cards directly underneath it. The reserve is in
  // world units but deliberately larger than the chat's world footprint at a
  // typical 0.6x fit zoom, so cards also clear it when the viewport zooms out.
  const reserve = {
    halfW: 360,
    halfH: 400,
  };
  const pushed: Array<{
    readonly id: string;
    readonly side: -1 | 1;
    readonly width: number;
    readonly height: number;
    readonly desiredY: number;
  }> = [];
  let sideIndex = 0;
  for (const id of ids) {
    const task = tasks[id];
    if (task.kind === 'hyperagent') {
      continue;
    }
    const size = NODE_SIZE[task.kind];
    const position = positions[id];
    if (position === undefined) {
      continue;
    }
    const centerX = position.x + size.width / 2;
    const centerY = position.y + size.height / 2;
    const intersectsChat =
      Math.abs(centerX) < reserve.halfW + size.width / 2 &&
      Math.abs(centerY) < reserve.halfH + size.height / 2;
    if (!intersectsChat) {
      continue;
    }
    pushed.push({
      id,
      side: sideIndex % 2 === 0 ? -1 : 1,
      width: size.width,
      height: size.height,
      desiredY: position.y,
    });
    sideIndex += 1;
  }

  for (const side of [-1, 1] as const) {
    const column = pushed
      .filter((entry) => entry.side === side)
      .sort((a, b) => a.desiredY - b.desiredY || a.id.localeCompare(b.id));
    let cursor = Number.NEGATIVE_INFINITY;
    for (const entry of column) {
      const y = Math.max(entry.desiredY, cursor);
      positions[entry.id] = {
        x: side * (reserve.halfW + entry.width / 2 + 42) - entry.width / 2,
        y,
      };
      cursor = y + entry.height + 18;
    }
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
  return Object.values(tasks)
    .filter((task) => task.kind !== 'hyperagent')
    .map((task) => {
      const size = nodeSize(task.kind);
      return {
        id: task.id,
        type: task.kind,
        position: positions[task.id] ?? { x: 0, y: 0 },
        style: { width: size.width, height: size.height },
        data: { task, selected: task.id === selectedTaskId, onSelect },
        ariaLabel: `${task.label} (${task.kind}, ${task.state})`,
        draggable: interactive,
        selectable: interactive,
      };
    });
}

export function buildEdges(tasks: Readonly<Record<string, TaskRecord>>): Edge[] {
  return Object.values(tasks)
    .filter((task) => task.parentId !== null && tasks[task.parentId] !== undefined)
    .map((task) => {
      const parent = task.parentId === null ? undefined : tasks[task.parentId];
      const source = parent?.kind === 'hyperagent' ? HIVE_ANCHOR_ID : (task.parentId ?? '');
      return {
        id: `edge-${task.parentId}-${task.id}`,
        source,
        target: task.id,
        type: 'bee',
        animated: false,
        data: {
          broken:
            task.kind === 'error' ||
            task.state === 'failed' ||
            task.state === 'killed' ||
            task.state === 'interrupted',
        },
      };
    });
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
