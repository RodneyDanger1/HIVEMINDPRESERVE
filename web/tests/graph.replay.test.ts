import { describe, expect, it } from 'vitest';
import { buildEdges, buildNodes, buildReplayTasks } from '../src/flow/graph';
import type { TaskRecord } from '../src/state/store';

const noop = (): void => {};

function snapshot(nodes: unknown[]): unknown {
  return { nodes, edges: [] };
}

describe('HISTORY_SNAPSHOT replay projection (S10)', () => {
  it('projects valid snapshot nodes into TaskRecords (id/parent/state/rev preserved)', () => {
    const replay = buildReplayTasks(
      snapshot([
        { task_id: 'snap-1', state: 'running', rev: 3 },
        { task_id: 'snap-2', parent_task_id: 'snap-1', state: 'completed', rev: 4 },
        { task_id: 'snap-orphan', parent_task_id: 'missing-parent', state: 'healthy', rev: 5 },
      ]),
    );

    expect(Object.keys(replay).sort()).toEqual(['snap-1', 'snap-2', 'snap-orphan']);
    expect(replay['snap-1']).toMatchObject({ id: 'snap-1', parentId: null, state: 'running', rev: 3 });
    expect(replay['snap-2']).toMatchObject({ id: 'snap-2', parentId: 'snap-1', state: 'completed', rev: 4 });
    // Orphan parent is not in the snapshot: re-rooted, never a dangling reference.
    expect(replay['snap-orphan']).toMatchObject({ id: 'snap-orphan', parentId: null, state: 'healthy', rev: 5 });
    // No role/kind is invented when the snapshot carries neither.
    for (const task of Object.values(replay)) {
      expect(task.kind).toBe('worker');
      expect(task.role).toBeNull();
      expect(task.progress).toBeNull();
      expect(task.errorCode).toBeNull();
      expect(task.label.startsWith('Agent ')).toBe(true);
    }
  });

  it('skips malformed and unknown-state nodes instead of fabricating state', () => {
    const replay = buildReplayTasks(
      snapshot([
        null,
        'not-a-node',
        42,
        { task_id: 'snap-ok', state: 'running', rev: 1 },
        { state: 'running' }, // missing task_id
        { task_id: 'snap-bogus', state: 'not-a-real-state' }, // unknown vocabulary
        { task_id: 'snap-empty-state', state: '' },
      ]),
    );

    expect(Object.keys(replay)).toEqual(['snap-ok']);
    expect(replay['snap-ok']?.state).toBe('running');
  });

  it('returns an empty projection for malformed top-level graph payloads', () => {
    for (const graph of [
      null,
      undefined,
      42,
      'graph',
      {},
      { nodes: null },
      { nodes: 'nope' },
      { nodes: {} },
      [],
    ]) {
      expect(buildReplayTasks(graph)).toEqual({});
    }
  });

  it('projects the replay graph to nodes and exactly the valid parent→child edges', () => {
    const replay = buildReplayTasks(
      snapshot([
        { task_id: 'snap-1', state: 'running', rev: 1 },
        { task_id: 'snap-2', parent_task_id: 'snap-1', state: 'completed', rev: 2 },
        { task_id: 'snap-orphan', parent_task_id: 'missing-parent', state: 'healthy', rev: 3 },
      ]),
    );

    const nodes = buildNodes(replay, null, noop, false);
    const edges = buildEdges(replay);

    expect(nodes.map((node) => node.id).sort()).toEqual(['snap-1', 'snap-2', 'snap-orphan']);
    expect(edges).toHaveLength(1);
    expect(edges[0]).toMatchObject({ source: 'snap-1', target: 'snap-2', type: 'bee' });
    // Every edge endpoint exists in the projected node set.
    const ids = new Set(nodes.map((node) => node.id));
    for (const edge of edges) {
      expect(ids.has(edge.source)).toBe(true);
      expect(ids.has(edge.target)).toBe(true);
    }
  });

  it('replay nodes are non-draggable and non-selectable; live nodes stay interactive', () => {
    const replay = buildReplayTasks(
      snapshot([
        { task_id: 'snap-1', state: 'running', rev: 1 },
        { task_id: 'snap-2', parent_task_id: 'snap-1', state: 'completed', rev: 2 },
      ]),
    );

    const replayNodes = buildNodes(replay, null, noop, false);
    expect(replayNodes).toHaveLength(2);
    for (const node of replayNodes) {
      expect(node.draggable).toBe(false);
      expect(node.selectable).toBe(false);
      expect(node.ariaLabel).toContain(`(${node.data.task.kind}, ${node.data.task.state})`);
    }

    const live = buildNodes(
      {
        live: {
          id: 'live',
          parentId: null,
          kind: 'worker',
          label: 'live',
          state: 'running',
          progress: null,
          errorCode: null,
          rev: 1,
        } satisfies TaskRecord,
      },
      'live',
      noop,
    );
    expect(live[0]?.draggable).toBe(true);
    expect(live[0]?.selectable).toBe(true);
    expect(live[0]?.data.selected).toBe(true);
  });

  it('buildEdges ignores children whose parent is absent from the task map', () => {
    const tasks: Record<string, TaskRecord> = {
      child: {
        id: 'child',
        parentId: 'absent',
        kind: 'worker',
        label: 'child',
        state: 'running',
        progress: null,
        errorCode: null,
        rev: 1,
      },
    };
    expect(buildEdges(tasks)).toEqual([]);
  });

  // Regression pin for W4-F1: dagre mutates its label with x/y; the layout must
  // pass a fresh size object or same-kind nodes collapse onto one position.
  it('lays out same-kind replay nodes at distinct positions (W4-F1 fixed)', () => {
    const replay = buildReplayTasks(
      snapshot([
        { task_id: 'snap-1', state: 'running', rev: 1 },
        { task_id: 'snap-2', parent_task_id: 'snap-1', state: 'completed', rev: 2 },
        { task_id: 'snap-orphan', state: 'healthy', rev: 3 },
      ]),
    );
    const nodes = buildNodes(replay, null, noop, false);
    const distinct = new Set(nodes.map((node) => `${node.position.x},${node.position.y}`));
    expect(distinct.size).toBe(nodes.length);
  });
});
