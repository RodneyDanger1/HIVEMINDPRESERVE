/**
 * Pure graph projection: TaskRecord map -> React Flow nodes (dagre layout) and
 * edges (parent -> child). Positions are deterministic for a given graph; the
 * canvas owns only drag offsets. No capability is invented: nodes without
 * server metadata keep the generic kind/label from the store.
 */
import dagre from '@dagrejs/dagre';
import type { Edge, Node } from '@xyflow/react';
import type { TaskRecord } from '../state/store';
import { isNodeKind } from '../state/store';
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
  subagent: LAYOUT.cards.worker,
  worker: LAYOUT.cards.worker,
  tool: LAYOUT.cards.tool,
  error: LAYOUT.cards.error,
};

export function nodeSize(kind: NodeKindLabel): { readonly width: number; readonly height: number } {
  return NODE_SIZE[kind];
}

function columnHiveLayout(
  tasks: Readonly<Record<string, TaskRecord>>,
  rootId: string,
): Record<string, Point2> {
  const positions: Record<string, Point2> = {};
  const childrenByParent: Record<string, string[]> = {};
  for (const task of Object.values(tasks)) {
    if (task.parentId === null) {
      continue;
    }
    const bucket = childrenByParent[task.parentId] ?? [];
    bucket.push(task.id);
    childrenByParent[task.parentId] = bucket;
  }
  for (const bucket of Object.values(childrenByParent)) {
    bucket.sort();
  }

  const sizeOf = (id: string): { readonly width: number; readonly height: number } => {
    const task = tasks[id];
    return task === undefined ? { width: 0, height: 0 } : NODE_SIZE[task.kind];
  };
  const measure = (id: string): number => {
    const size = sizeOf(id);
    const children = childrenByParent[id] ?? [];
    if (children.length === 0) {
      return size.height;
    }
    return size.height + 16 + children.reduce((total, child) => total + measure(child), 0) + (children.length - 1) * 14;
  };

  const placeColumn = (id: string, side: -1 | 1, top: number): number => {
    const size = sizeOf(id);
    positions[id] = { x: side * 476 - size.width / 2, y: top };
    let cursor = top + size.height;
    for (const child of childrenByParent[id] ?? []) {
      cursor += 16;
      cursor = placeColumn(child, side, cursor);
    }
    return cursor;
  };

  const direct = childrenByParent[rootId] ?? [];
  const left = direct.filter((_, index) => index % 2 === 0);
  const right = direct.filter((_, index) => index % 2 === 1);
  for (const [side, group] of [
    [-1, left],
    [1, right],
  ] as const) {
    if (group.length === 0) {
      continue;
    }
    const total = group.reduce((sum, id) => sum + measure(id), 0) + (group.length - 1) * 24;
    let cursor = -total / 2;
    for (const id of group) {
      cursor = placeColumn(id, side, cursor) + 24;
    }
  }
  positions[rootId] = { x: -NODE_SIZE.hyperagent.width / 2, y: -NODE_SIZE.hyperagent.height / 2 };
  // Orphans/parentless tasks must never pile at the world origin behind the
  // chat. Give each unplaced node a visible slot below the hive.
  let orphanIndex = 0;
  for (const task of Object.values(tasks)) {
    if (positions[task.id] !== undefined) {
      continue;
    }
    const size = NODE_SIZE[task.kind];
    positions[task.id] = {
      x: (orphanIndex % 4) * 360 - 540 - size.width / 2,
      y: 560 + Math.floor(orphanIndex / 4) * 150,
    };
    orphanIndex += 1;
  }
  return positions;
}

export function layoutTasks(tasks: Readonly<Record<string, TaskRecord>>): Record<string, Point2> {
  const hiveRoot = Object.values(tasks).find(
    (task) => task.kind === 'hyperagent' && task.parentId === null,
  );
  if (hiveRoot !== undefined) {
    return columnHiveLayout(tasks, hiveRoot.id);
  }
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
      kind?: unknown;
      role?: unknown;
      label?: unknown;
    };
    if (typeof node.task_id !== 'string' || !isAgentState(node.state)) continue;
    const parentId = typeof node.parent_task_id === 'string' ? node.parent_task_id : null;
    const kind = isNodeKind(node.kind) ? node.kind : 'worker';
    const role = typeof node.role === 'string' && node.role.length > 0 ? node.role : null;
    tasks[node.task_id] = {
      id: node.task_id,
      parentId,
      kind,
      label:
        typeof node.label === 'string' && node.label.trim().length > 0
          ? node.label
          : kind === 'worker'
            ? `Agent ${node.task_id.slice(-8)}`
            : `${kind} ${node.task_id.slice(-8)}`,
      role,
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
    subagent: 0,
    worker: 0,
    tool: 0,
    error: 0,
  };
  for (const task of Object.values(tasks)) {
    byKind[task.kind] += 1;
  }
  return { byKind };
}
