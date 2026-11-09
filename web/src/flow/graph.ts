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
import { isAgentState, isTerminal, type NodeKindLabel } from '../state/agentState';
import { LAYOUT } from '../tokens/tokens';

export interface CanvasNodeData extends Record<string, unknown> {
  readonly task: TaskRecord;
  readonly selected: boolean;
  readonly onSelect: (taskId: string) => void;
  /** Per-card spawn action; the server still enforces the class matrix. */
  readonly onSpawn: (taskId: string) => void;
  /**
   * Which side of its parent this card sits on. Left branches attach their
   * target to the right handle and their source to the left handle, so the
   * parent->child edge never crosses the parent card.
   */
  readonly branchSide: 'left' | 'right' | 'center';
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

/** World position of the invisible hive anchor node. The HyperAgent is the
 * fixed centre of the graph: it never participates in layout, drag or fit. */
export const HIVE_ANCHOR_POSITION: Point2 = { x: -0.5, y: -0.5 };

/** Fixed world origin every position is measured from. */
export const HIVE_ORIGIN: Point2 = { x: 0, y: 0 };

/** Horizontal gap between a parent card edge and its child card edge. */
const BRANCH_GAP_X = 88;
/** Vertical gap between sibling subtrees in the same branch column. */
const BRANCH_GAP_Y = 36;
/** Gap between stacked orphan cards below the hive. */
const ORPHAN_GAP_Y = 150;

/**
 * Reads the child buckets once, sorted by id so the projection is
 * deterministic for a given task map regardless of insertion order.
 */
function childrenIndex(
  tasks: Readonly<Record<string, TaskRecord>>,
): Readonly<Record<string, readonly string[]>> {
  const index: Record<string, string[]> = {};
  for (const id of Object.keys(tasks).sort()) {
    const parentId = tasks[id].parentId;
    if (parentId === null) {
      continue;
    }
    (index[parentId] ??= []).push(id);
  }
  return index;
}

/**
 * Lays the hive out as a branch tree radiating from the fixed HyperAgent
 * centre. Direct SuperAgents alternate left/right; every deeper generation is
 * pushed further OUTWARD on its own side, with its subtree vertically centred
 * and packed so sibling subtrees never collide. No node is ever placed
 * directly below its parent card, and the root is always at the origin.
 */
function columnHiveLayout(
  tasks: Readonly<Record<string, TaskRecord>>,
  rootId: string,
): Record<string, Point2> {
  const positions: Record<string, Point2> = {};
  const childrenByParent = childrenIndex(tasks);

  const sizeOf = (id: string): { readonly width: number; readonly height: number } => {
    const task = tasks[id];
    return task === undefined ? { width: 0, height: 0 } : NODE_SIZE[task.kind];
  };

  // Subtree height = how much vertical room this whole branch needs. Cached so
  // a wide tree is measured once, in O(n), not exponentially.
  const heightMemo = new Map<string, number>();
  const measure = (id: string): number => {
    const cached = heightMemo.get(id);
    if (cached !== undefined) {
      return cached;
    }
    const size = sizeOf(id);
    const children = childrenByParent[id] ?? [];
    let total = size.height;
    if (children.length > 0) {
      const childTotal =
        children.reduce((sum, child) => sum + measure(child), 0) +
        (children.length - 1) * BRANCH_GAP_Y;
      total = Math.max(size.height, childTotal);
    }
    heightMemo.set(id, total);
    return total;
  };

  const place = (id: string, centerX: number, side: -1 | 1, top: number, visiting: ReadonlySet<string>): void => {
    if (positions[id] !== undefined || visiting.has(id)) {
      return;
    }
    const size = sizeOf(id);
    const height = measure(id);
    // Centre the card inside its own subtree band so parent->child edges stay
    // short and readable instead of leaving the card above its children.
    positions[id] = { x: centerX - size.width / 2, y: top + (height - size.height) / 2 };
    const children = (childrenByParent[id] ?? []).filter((child) => positions[child] === undefined);
    if (children.length === 0) {
      return;
    }
    const childTotal =
      children.reduce((sum, child) => sum + measure(child), 0) +
      (children.length - 1) * BRANCH_GAP_Y;
    // Children of a visually tall card start after its lower edge; otherwise
    // they are vertically centred in the leftover space.
    let cursor = top + Math.max(size.height, (height - childTotal) / 2);
    const nextVisiting = new Set(visiting);
    nextVisiting.add(id);
    for (const child of children) {
      const childWidth = sizeOf(child).width;
      // Every generation moves outward by the parent edge + gap + child half
      // width. Same-side sign is preserved: left stays left, right stays right.
      const childCenterX = centerX + side * (size.width / 2 + BRANCH_GAP_X + childWidth / 2);
      place(child, childCenterX, side, cursor, nextVisiting);
      cursor += measure(child) + BRANCH_GAP_Y;
    }
  };

  const direct = childrenByParent[rootId] ?? [];
  const groups: ReadonlyArray<readonly [(-1 | 1), readonly string[]]> = [
    [-1, direct.filter((_, index) => index % 2 === 0)],
    [1, direct.filter((_, index) => index % 2 === 1)],
  ];
  for (const [side, group] of groups) {
    if (group.length === 0) {
      continue;
    }
    const total =
      group.reduce((sum, id) => sum + measure(id), 0) + (group.length - 1) * BRANCH_GAP_Y;
    let cursor = -total / 2;
    for (const id of group) {
      const width = sizeOf(id).width;
      // Direct branches clear the fixed chat overlay by half the chat width
      // plus a readable gap, then the card's own half width.
      const centerX = side * (NODE_SIZE.hyperagent.width / 2 + BRANCH_GAP_X + width / 2);
      place(id, centerX, side, cursor, new Set([rootId]));
      cursor += measure(id) + BRANCH_GAP_Y;
    }
  }

  // The HyperAgent is the fixed centre anchor. Layout must never move it:
  // its centre is exactly (0,0), independent of card size or children.
  positions[rootId] = {
    x: HIVE_ORIGIN.x - NODE_SIZE.hyperagent.width / 2,
    y: HIVE_ORIGIN.y - NODE_SIZE.hyperagent.height / 2,
  };

  // Orphans/parentless tasks must never pile at the world origin behind the
  // chat. Place them deterministically below the lowest branch card.
  let bottom = HIVE_ORIGIN.y;
  for (const [id, position] of Object.entries(positions)) {
    bottom = Math.max(bottom, position.y + sizeOf(id).height);
  }
  let orphanIndex = 0;
  for (const id of Object.keys(tasks).sort()) {
    if (positions[id] !== undefined) {
      continue;
    }
    const size = NODE_SIZE[tasks[id].kind];
    positions[id] = {
      x: (orphanIndex % 4) * 360 - 540 - size.width / 2,
      y: bottom + 120 + Math.floor(orphanIndex / 4) * ORPHAN_GAP_Y,
    };
    orphanIndex += 1;
  }
  return positions;
}

export function layoutTasks(tasks: Readonly<Record<string, TaskRecord>>): Record<string, Point2> {
  const hiveRoots = Object.values(tasks)
    .filter((task) => task.kind === 'hyperagent' && task.parentId === null)
    .sort((a, b) => a.id.localeCompare(b.id));
  if (hiveRoots.length > 0) {
    return columnHiveLayout(tasks, hiveRoots[0].id);
  }
  const graph = new dagre.graphlib.Graph();
  graph.setGraph({ rankdir: 'TB', nodesep: 40, ranksep: 90, marginx: 40, marginy: 40 });
  graph.setDefaultEdgeLabel(() => ({}));
  const ids = Object.keys(tasks).sort();
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
  // World origin is the centre of the root card (spatial-canvas spec). Shift
  // every dagre coordinate so the root's centre is exactly (0,0); the fixed
  // chat overlay and Ctrl+0 recentre then share the same anchor.
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

/**
 * Pure visibility projection for the canvas filters. `hideFinished` removes
 * terminal cards but always retains every visible node's ancestor chain, so a
 * parent->child edge never loses its source node (the HyperAgent anchor stays
 * mounted separately in Canvas). `focusBranch` keeps the selected node, its
 * ancestors and its full descendant branch.
 */
export function filterVisibleTasks(
  tasks: Readonly<Record<string, TaskRecord>>,
  options: {
    readonly hideFinished: boolean;
    readonly focusBranch: boolean;
    readonly selectedTaskId: string | null;
  },
): Readonly<Record<string, TaskRecord>> {
  const { hideFinished, focusBranch, selectedTaskId } = options;
  if (focusBranch && selectedTaskId !== null) {
    const selected = tasks[selectedTaskId];
    if (selected !== undefined) {
      const keep = new Set<string>();
      let cursor: TaskRecord | undefined = selected;
      while (cursor !== undefined) {
        keep.add(cursor.id);
        cursor = cursor.parentId !== null ? tasks[cursor.parentId] : undefined;
      }
      const walk = (parentId: string): void => {
        for (const task of Object.values(tasks)) {
          if (task.parentId === parentId && !keep.has(task.id)) {
            keep.add(task.id);
            walk(task.id);
          }
        }
      };
      walk(selected.id);
      return Object.fromEntries(Object.entries(tasks).filter(([id]) => keep.has(id)));
    }
  }
  if (!hideFinished) {
    return tasks;
  }
  const keep = new Set<string>();
  for (const task of Object.values(tasks)) {
    if (!isTerminal(task.state)) {
      keep.add(task.id);
    }
  }
  for (const id of [...keep]) {
    let parentId = tasks[id]?.parentId ?? null;
    while (parentId !== null && !keep.has(parentId)) {
      keep.add(parentId);
      parentId = tasks[parentId]?.parentId ?? null;
    }
  }
  return Object.fromEntries(Object.entries(tasks).filter(([id]) => keep.has(id)));
}

export function buildNodes(
  tasks: Readonly<Record<string, TaskRecord>>,
  selectedTaskId: string | null,
  onSelect: (taskId: string) => void,
  interactive = true,
  onSpawn: (taskId: string) => void = () => {},
): CanvasNode[] {
  const positions = layoutTasks(tasks);
  const centerX = (id: string): number => {
    const position = positions[id];
    return (position?.x ?? 0) + nodeSize(tasks[id].kind).width / 2;
  };
  const branchSideOf = (task: TaskRecord): 'left' | 'right' | 'center' => {
    if (task.parentId === null) {
      return 'center';
    }
    const parent = tasks[task.parentId];
    if (parent === undefined) {
      return 'center';
    }
    const parentCenter = parent.kind === 'hyperagent' ? HIVE_ORIGIN.x : centerX(parent.id);
    const childCenter = centerX(task.id);
    if (childCenter < parentCenter) {
      return 'left';
    }
    return childCenter > parentCenter ? 'right' : 'center';
  };
  return Object.values(tasks)
    .filter((task) => task.kind !== 'hyperagent')
    .map((task) => {
      const size = nodeSize(task.kind);
      return {
        id: task.id,
        type: task.kind,
        position: positions[task.id] ?? { x: 0, y: 0 },
        style: { width: size.width, height: size.height },
        data: {
          task,
          selected: task.id === selectedTaskId,
          onSelect,
          onSpawn,
          branchSide: branchSideOf(task),
        },
        ariaLabel: `${task.label} (${task.kind}, ${task.state})`,
        draggable: interactive,
        selectable: interactive,
      };
    });
}

function isBrokenState(state: TaskRecord['state']): boolean {
  return state === 'failed' || state === 'killed' || state === 'interrupted';
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
            isBrokenState(task.state) ||
            (parent !== undefined &&
              (isBrokenState(parent.state) || parent.kind === 'error')),
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
