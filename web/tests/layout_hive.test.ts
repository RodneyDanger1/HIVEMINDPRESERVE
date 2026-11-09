/**
 * Deterministic hive layout invariants.
 *
 * The HyperAgent is the fixed centre anchor; SuperAgent branches alternate
 * outward left/right; every deeper generation is pushed further outward on its
 * own side and vertically packed so sibling subtrees never collide. Filtered
 * projections retain ancestor chains so every edge keeps mounted endpoints.
 */
import { describe, expect, it } from 'vitest';
import nodesSource from '../src/flow/nodes.tsx?raw';
import canvasSource from '../src/flow/Canvas.tsx?raw';
import {
  buildEdges,
  buildNodes,
  filterVisibleTasks,
  HIVE_ANCHOR_ID,
  HIVE_ANCHOR_POSITION,
  layoutTasks,
  nodeSize,
} from '../src/flow/graph';
import type { AgentState, NodeKindLabel } from '../src/state/agentState';
import type { TaskRecord } from '../src/state/store';

const noop = (): void => {};

function task(
  id: string,
  parentId: string | null,
  kind: NodeKindLabel,
  state: AgentState = 'running',
): TaskRecord {
  return {
    id,
    parentId,
    kind,
    label: id,
    role: null,
    state,
    progress: null,
    errorCode: null,
    rev: 1,
  };
}

/** Hive root + two SuperAgent branches with Agents and Subagents. */
function hive(): Record<string, TaskRecord> {
  return {
    H: task('H', null, 'hyperagent'),
    S1: task('S1', 'H', 'superagent'),
    A1: task('A1', 'S1', 'worker'),
    T1: task('T1', 'A1', 'subagent'),
    T2: task('T2', 'A1', 'subagent', 'queued'),
    A2: task('A2', 'S1', 'worker'),
    S2: task('S2', 'H', 'superagent'),
    A3: task('A3', 'S2', 'worker'),
    A4: task('A4', 'S2', 'error', 'failed'),
  };
}

interface Rect {
  readonly id: string;
  readonly x: number;
  readonly y: number;
  readonly w: number;
  readonly h: number;
}

function cardRect(
  id: string,
  tasks: Readonly<Record<string, TaskRecord>>,
  positions: Readonly<Record<string, { readonly x: number; readonly y: number }>>,
): Rect {
  const size = nodeSize(tasks[id].kind);
  const position = positions[id];
  return { id, x: position.x, y: position.y, w: size.width, h: size.height };
}

function overlaps(a: Rect, b: Rect): boolean {
  const x = Math.min(a.x + a.w, b.x + b.w) - Math.max(a.x, b.x);
  const y = Math.min(a.y + a.h, b.y + b.h) - Math.max(a.y, b.y);
  return x > 1 && y > 1;
}

describe('hive layout: fixed HyperAgent centre', () => {
  it('pins the HyperAgent card centre to the world origin', () => {
    const tasks = hive();
    const positions = layoutTasks(tasks);
    const size = nodeSize('hyperagent');
    expect(positions.H).toEqual({ x: -size.width / 2, y: -size.height / 2 });
    expect(positions.H.x + size.width / 2).toBe(0);
    expect(positions.H.y + size.height / 2).toBe(0);
    // The projected React Flow node set excludes the HyperAgent: the fixed
    // ChatCore overlay is the only HyperAgent surface, at the same origin.
    const nodes = buildNodes(tasks, null, noop);
    expect(nodes.some((node) => node.id === 'H')).toBe(false);
    expect(nodes.some((node) => node.id === HIVE_ANCHOR_ID)).toBe(false);
    expect(HIVE_ANCHOR_POSITION).toEqual({ x: -0.5, y: -0.5 });
  });

  it('anchors the HyperAgent at the origin regardless of insertion order', () => {
    const tasks = hive();
    const reversed = Object.fromEntries(Object.entries(tasks).reverse()) as Record<string, TaskRecord>;
    const a = layoutTasks(tasks);
    const b = layoutTasks(reversed);
    expect(b.H).toEqual(a.H);
    expect(b.H).toEqual({ x: -nodeSize('hyperagent').width / 2, y: -nodeSize('hyperagent').height / 2 });
    // Every position is deterministic for a given graph.
    expect(b).toEqual(a);
  });
});

describe('hive layout: outward branches with collision avoidance', () => {
  it('alternates direct SuperAgent branches left and right without stacking', () => {
    const tasks = hive();
    const positions = layoutTasks(tasks);
    const s1 = cardRect('S1', tasks, positions);
    const s2 = cardRect('S2', tasks, positions);
    const s1Center = s1.x + s1.w / 2;
    const s2Center = s2.x + s2.w / 2;
    expect(s1Center).toBeLessThan(0);
    expect(s2Center).toBeGreaterThan(0);
    // The branch must clear the fixed 400px-wide chat overlay at the centre.
    expect(-s1Center).toBeGreaterThan(nodeSize('hyperagent').width / 2 + s1.w / 2);
  });

  it('recursively expands every generation further outward on its own side', () => {
    const tasks = hive();
    const positions = layoutTasks(tasks);
    for (const current of Object.values(tasks)) {
      const parentId = current.parentId;
      if (parentId === null) {
        continue;
      }
      const parent = tasks[parentId];
      const parentCenter = cardRect(parentId, tasks, positions).x + nodeSize(parent.kind).width / 2;
      const childRect = cardRect(current.id, tasks, positions);
      const childCenter = childRect.x + childRect.w / 2;
      // Never directly below the expanded parent card.
      expect(childCenter).not.toBe(parentCenter);
      if (parent.kind === 'hyperagent') {
        // Depth 1 clears the central chat card on both sides.
        expect(Math.abs(childCenter)).toBeGreaterThan(nodeSize('hyperagent').width / 2);
        continue;
      }
      // Same side, strictly further out: Agents/Subagents radiate outward.
      expect(Math.sign(childCenter)).toBe(Math.sign(parentCenter));
      expect(Math.abs(childCenter)).toBeGreaterThan(Math.abs(parentCenter));
    }
  });

  it('vertically separates sibling subtrees so no two cards overlap', () => {
    const tasks = hive();
    const positions = layoutTasks(tasks);
    const rects = Object.values(tasks).map((entry) => cardRect(entry.id, tasks, positions));
    for (let i = 0; i < rects.length; i += 1) {
      for (let j = i + 1; j < rects.length; j += 1) {
        expect(overlaps(rects[i], rects[j])).toBe(false);
      }
    }
  });

  it('keeps every node reachable with a finite, distinct position', () => {
    const tasks = hive();
    const positions = layoutTasks(tasks);
    expect(Object.keys(positions).sort()).toEqual(Object.keys(tasks).sort());
    const seen = new Set<string>();
    for (const [id, position] of Object.entries(positions)) {
      expect(Number.isFinite(position.x)).toBe(true);
      expect(Number.isFinite(position.y)).toBe(true);
      const key = `${position.x},${position.y}`;
      expect(seen.has(key), `duplicate position for ${id}`).toBe(false);
      seen.add(key);
    }
  });

  it('attaches each card to the side it branches toward its parent from', () => {
    const nodes = buildNodes(hive(), null, noop);
    const sideOf = new Map(nodes.map((node) => [node.id, node.data.branchSide]));
    expect(sideOf.get('S1')).toBe('left');
    expect(sideOf.get('S2')).toBe('right');
    // Descendants inherit their branch side, so edges run outward from both
    // parents to children exactly like the SuperAgent -> Agent edge above.
    expect(sideOf.get('A1')).toBe('left');
    expect(sideOf.get('T1')).toBe('left');
    expect(sideOf.get('A3')).toBe('right');
    expect(sideOf.get('A4')).toBe('right');
    expect(sideOf.has('H')).toBe(false);
  });
});

describe('canvas visibility filters keep hierarchy edges mounted', () => {
  const tasks: Record<string, TaskRecord> = {
    H: task('H', null, 'hyperagent', 'completed'),
    S1: task('S1', 'H', 'superagent', 'completed'),
    A1: task('A1', 'S1', 'worker', 'running'),
    A2: task('A2', 'S1', 'worker', 'completed'),
    T1: task('T1', 'A1', 'subagent', 'queued'),
  };

  it('hide-finished retains every visible node ancestor chain and its edges', () => {
    const visible = filterVisibleTasks(tasks, {
      hideFinished: true,
      focusBranch: false,
      selectedTaskId: null,
    });
    // A2 is finished and has no visible descendants => dropped. H and S1 are
    // finished too, but they are ancestors of the running A1 => retained.
    expect(Object.keys(visible).sort()).toEqual(['A1', 'H', 'S1', 'T1']);

    const edges = buildEdges(visible);
    expect(edges).toHaveLength(3);
    const cardIds = new Set(buildNodes(visible, null, noop).map((node) => node.id));
    for (const edge of edges) {
      expect(cardIds.has(edge.target)).toBe(true);
      if (edge.source === HIVE_ANCHOR_ID) {
        // HyperAgent root edges anchor to the always-mounted hive core.
        expect(visible.H?.kind).toBe('hyperagent');
      } else {
        expect(cardIds.has(edge.source)).toBe(true);
      }
    }
  });

  it('focus-branch keeps the selected branch and its ancestor chain only', () => {
    const visible = filterVisibleTasks(tasks, {
      hideFinished: false,
      focusBranch: true,
      selectedTaskId: 'A1',
    });
    expect(Object.keys(visible).sort()).toEqual(['A1', 'H', 'S1', 'T1']);
  });

  it('returns the same projection when no filter is active', () => {
    const visible = filterVisibleTasks(tasks, {
      hideFinished: false,
      focusBranch: false,
      selectedTaskId: null,
    });
    expect(visible).toBe(tasks);
  });
});

describe('interaction invariants (static pins)', () => {
  it('every node card button is nodrag so pointer-down selects instead of dragging', () => {
    const buttons = nodesSource.match(/<button/g) ?? [];
    const nodrag = nodesSource.match(/className="nodrag/g) ?? [];
    expect(buttons.length).toBeGreaterThan(0);
    expect(nodrag.length).toBeGreaterThanOrEqual(buttons.length);
    // Handles follow the branch side so left-branch edges never cross the
    // parent card.
    expect(nodesSource).toContain('position={targetPosition}');
    expect(nodesSource).toContain('position={sourcePosition}');
  });

  it('Canvas mounts the fixed anchor position and never overlays the HyperAgent', () => {
    expect(canvasSource).toContain('HIVE_ANCHOR_POSITION');
    expect(canvasSource).toContain("node.data.task.kind === 'hyperagent'");
    // Drag must persist the live overlay, not just the drop position.
    expect(canvasSource).toContain('setPositionOverrides((previous) => ({');
  });
});
