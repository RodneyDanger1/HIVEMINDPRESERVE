/**
 * S01 canvas + S13 minimap + S20 zoom controls. @xyflow/react 12.12.1 with
 * explicit zoom bounds 0.1-3.0 (library defaults 0.5/2 overridden, R-U7),
 * Ctrl+0 recentre (Windows; Meta+0 alias), honeycomb vector field, bee edges.
 * Replay mode renders the read-only HISTORY_SNAPSHOT graph via the same
 * components but never mutates live state.
 */
import { useCallback, useEffect, useMemo, useState } from 'react';
import {
  MiniMap,
  ReactFlow,
  ReactFlowProvider,
  useReactFlow,
  type Edge,
  type Node,
  type NodeTypes,
  type EdgeTypes,
} from '@xyflow/react';
import '@xyflow/react/dist/style.css';
import { buildEdges, buildNodes, buildReplayTasks, HIVE_ANCHOR_ID, type CanvasNode } from './graph';
import { NODE_RENDERERS } from './nodes';
import { BeeEdge } from './BeeEdge';
import { handleCanvasShortcut } from '../a11y/keyboard';
import { useUiStore } from '../state/context';
import { HudBadge } from '../panels/HudBadge';
import { LAYOUT } from '../tokens/tokens';
import { isTerminal } from '../state/agentState';
import type { TaskRecord } from '../state/store';
import { useAppReducedMotion } from '../state/useAppReducedMotion';
import { STRINGS } from '../state/strings';

const nodeTypes = NODE_RENDERERS as unknown as NodeTypes;
const edgeTypes: EdgeTypes = { bee: BeeEdge };
const POSITION_STORAGE_PREFIX = 'hivemind.canvas.positions.v1';

type Position = { x: number; y: number };
type PositionMap = Readonly<Record<string, Position>>;

function loadPositions(runId: string | null): PositionMap {
  if (runId === null) {
    return {};
  }
  try {
    const raw = window.localStorage.getItem(`${POSITION_STORAGE_PREFIX}:${runId}`);
    if (raw === null) {
      return {};
    }
    const parsed: unknown = JSON.parse(raw);
    if (typeof parsed !== 'object' || parsed === null || Array.isArray(parsed)) {
      return {};
    }
    const positions: Record<string, Position> = {};
    for (const [taskId, point] of Object.entries(parsed)) {
      if (typeof point !== 'object' || point === null) {
        continue;
      }
      const candidate = point as { x?: unknown; y?: unknown };
      if (typeof candidate.x === 'number' && typeof candidate.y === 'number') {
        positions[taskId] = { x: candidate.x, y: candidate.y };
      }
    }
    return positions;
  } catch {
    return {};
  }
}

function savePositions(runId: string | null, positions: PositionMap): void {
  if (runId === null) {
    return;
  }
  try {
    window.localStorage.setItem(
      `${POSITION_STORAGE_PREFIX}:${runId}`,
      JSON.stringify(positions),
    );
  } catch {
    // Storage may be unavailable/quota-full; the canvas remains usable.
  }
}

function zoomPercent(zoom: number): string {
  return `${Math.round(zoom * 100)}%`;
}

function CanvasInner() {
  const tasks = useUiStore((state) => state.tasks);
  const selectedTaskId = useUiStore((state) => state.selectedTaskId);
  const selectTask = useUiStore((state) => state.selectTask);
  const rewireSelected = useUiStore((state) => state.rewireSelected);
  const runId = useUiStore((state) => state.run?.id ?? null);
  const setViewport = useUiStore((state) => state.setViewport);
  const connection = useUiStore((state) => state.connection);
  const history = useUiStore((state) => state.history);
  const reduced = useAppReducedMotion();
  const { setCenter, getZoom, zoomIn, zoomOut, fitView, getIntersectingNodes } = useReactFlow();
  const [zoomLabel, setZoomLabel] = useState('100%');
  const [positionOverrides, setPositionOverrides] = useState<PositionMap>({});
  const [hideFinished, setHideFinished] = useState(false);
  const [focusBranch, setFocusBranch] = useState(false);

  useEffect(() => {
    setPositionOverrides(loadPositions(runId));
  }, [runId]);

  const handleSelect = useCallback(
    (taskId: string) => {
      selectTask(taskId);
    },
    [selectTask],
  );

  const visibleTasks = useMemo<Readonly<Record<string, TaskRecord>>>(() => {
    if (!hideFinished && !focusBranch) {
      return tasks;
    }
    const selected = selectedTaskId !== null ? tasks[selectedTaskId] : undefined;
    if (focusBranch && selected !== undefined) {
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
    return Object.fromEntries(
      Object.entries(tasks).filter(([, task]) => !hideFinished || !isTerminal(task.state)),
    );
  }, [tasks, hideFinished, focusBranch, selectedTaskId]);

  const nodes = useMemo<CanvasNode[]>(() => {
    if (history.replayGraph) {
      return [];
    }
    const laidOut = buildNodes(visibleTasks, selectedTaskId, handleSelect);
    return laidOut.map((node) => {
      const override = positionOverrides[node.id];
      return override ? { ...node, position: override } : node;
    });
  }, [visibleTasks, selectedTaskId, handleSelect, history.replayGraph, positionOverrides]);

  const hasTasks = Object.keys(visibleTasks).length > 0;
  const liveNodes = useMemo<CanvasNode[]>(() => {
    if (history.replayGraph) {
      return [];
    }
    const hasHiveParent = Object.values(visibleTasks).some((task) => task.kind === 'hyperagent');
    if (!hasHiveParent) {
      return nodes;
    }
    return [
      ...nodes,
      {
        id: HIVE_ANCHOR_ID,
        type: 'default',
        position: { x: -0.5, y: -0.5 },
        style: { width: 1, height: 1, opacity: 0, pointerEvents: 'none' },
        data: {},
        draggable: false,
        selectable: false,
        connectable: false,
        focusable: false,
      } as CanvasNode,
    ];
  }, [nodes, visibleTasks, history.replayGraph]);
  useEffect(() => {
    // First graph: fit the whole swarm so child cards are never born outside
    // the viewport. Ctrl+0 still returns to the hive origin.
    if (history.replayGraph || !hasTasks) {
      return;
    }
    const id = requestAnimationFrame(() => {
      void fitView({ padding: 0.24, duration: 0, minZoom: 0.2, maxZoom: 1 });
    });
    return () => cancelAnimationFrame(id);
  }, [runId, hasTasks, history.replayGraph, fitView]);

  const edges = useMemo<Edge[]>(() => buildEdges(visibleTasks), [visibleTasks]);

  const duration = reduced ? 0 : 250;
  // Ctrl+0/Meta+0 recentres INSTANTLY at document capture level so browser
  // shortcuts, focused controls or React Flow's own key handling cannot swallow
  // it; a rAF re-apply guarantees the transform matrix has changed after the key.
  useEffect(() => {
    const recentre = (): void => {
      setCenter(0, 0, { zoom: getZoom(), duration: 0 });
      requestAnimationFrame(() => {
        setCenter(0, 0, { zoom: getZoom(), duration: 0 });
      });
    };
    const onKeyDown = (event: KeyboardEvent): void => {
      const handled = handleCanvasShortcut(event, {
        recenter: recentre,
        zoomIn: () => {
          void zoomIn({ duration });
        },
        zoomOut: () => {
          void zoomOut({ duration });
        },
        fitView: () => {
          void fitView({ duration, padding: 0.2 });
        },
      });
      if (handled) {
        event.preventDefault();
        event.stopPropagation();
      }
    };
    document.addEventListener('keydown', onKeyDown, true);
    return () => document.removeEventListener('keydown', onKeyDown, true);
  }, [duration, fitView, getZoom, setCenter, zoomIn, zoomOut]);

  // S13: accessible name on the React Flow minimap element (role img).
  useEffect(() => {
    const label = STRINGS.minimap.aria;
    const apply = (): void => {
      for (const selector of ['.react-flow__minimap', '.rf__minimap']) {
        const element = document.querySelector(selector);
        if (element) {
          element.setAttribute('aria-label', label);
          element.setAttribute('role', 'img');
        }
      }
    };
    apply();
    const raf = requestAnimationFrame(apply);
    return () => cancelAnimationFrame(raf);
  }, []);

  const selected = selectedTaskId ? tasks[selectedTaskId] : null;
  const replayTasks = useMemo(
    () => buildReplayTasks(history.replayGraph?.graph),
    [history.replayGraph],
  );
  const noopSelect = useCallback((_taskId: string) => {}, []);
  const replayNodes = useMemo(
    () => buildNodes(replayTasks, null, noopSelect, false),
    [replayTasks, noopSelect],
  );
  const replayEdges = useMemo(() => buildEdges(replayTasks), [replayTasks]);

  return (
    <div id="hive-canvas" className="relative h-full min-h-0 w-full" role="application" aria-label="HiveMind spatial canvas">
      <svg className="hive-canvas-field" aria-hidden="true" width="100%" height="100%">
        <defs>
          <pattern id="honeycomb-field" width="48" height="42" patternUnits="userSpaceOnUse">
            <path d="M12 0L36 0L48 21L36 42L12 42L0 21Z" fill="none" stroke="currentColor" strokeWidth="1" />
          </pattern>
        </defs>
        <rect width="100%" height="100%" fill="url(#honeycomb-field)" />
      </svg>
      <div className="pointer-events-none absolute inset-0 z-0 opacity-40">
        <div className="hive-ambient-glow" />
      </div>
      <div className="pointer-events-none absolute left-1/2 top-1/2 z-0 h-[660px] w-[620px] -translate-x-1/2 -translate-y-1/2">
        <div className="hive-ambient-glow" />
        <svg aria-hidden="true" className="h-full w-full" width="620" height="660" viewBox="0 0 620 660">
          <defs>
            <radialGradient id="hive-lobe" cx="50%" cy="42%" r="62%">
              <stop offset="0%" stopColor="var(--color-honey-200)" stopOpacity="0.95" />
              <stop offset="52%" stopColor="var(--color-honey-400)" stopOpacity="0.88" />
              <stop offset="100%" stopColor="var(--color-honey-700)" stopOpacity="0.9" />
            </radialGradient>
            <radialGradient id="hive-highlight" cx="50%" cy="35%" r="55%">
              <stop offset="0%" stopColor="var(--color-honey-100)" stopOpacity="0.9" />
              <stop offset="100%" stopColor="var(--color-honey-300)" stopOpacity="0" />
            </radialGradient>
          </defs>
          {[95, 215, 335, 455, 575].map((cy) => (
            <ellipse key={cy} cx="310" cy={cy} rx="270" ry="108" fill="url(#hive-lobe)" />
          ))}
          {[95, 215, 335, 455, 575].map((cy) => (
            <ellipse key={`h-${cy}`} cx="310" cy={cy - 18} rx="168" ry="42" fill="url(#hive-highlight)" />
          ))}
        </svg>
        <div className="hive-sheen" />
      </div>
      <HudBadge />
      <div className="absolute left-3 top-16 z-20 flex items-center gap-1">
        <button
          type="button"
          aria-pressed={hideFinished}
          className={
            'type-mono-meta rounded-[var(--radius-pill)] border px-2 py-1 ' +
            (hideFinished
              ? 'border-honey-500 bg-honey-100 text-ink-900'
              : 'border-hairline bg-panel text-ink-500 hover:bg-honey-100')
          }
          onClick={() => setHideFinished((previous) => !previous)}
        >
          Hide finished
        </button>
        <button
          type="button"
          aria-pressed={focusBranch}
          disabled={selectedTaskId === null}
          title={selectedTaskId === null ? 'Select a node to focus its branch' : 'Show only the selected branch'}
          className={
            'type-mono-meta rounded-[var(--radius-pill)] border px-2 py-1 disabled:opacity-50 ' +
            (focusBranch
              ? 'border-honey-500 bg-honey-100 text-ink-900'
              : 'border-hairline bg-panel text-ink-500 hover:bg-honey-100')
          }
          onClick={() => setFocusBranch((previous) => !previous)}
        >
          Focus branch
        </button>
        <span className="type-mono-meta rounded-[var(--radius-pill)] border border-hairline bg-panel px-2 py-1 text-meta">
          {Object.keys(visibleTasks).length}/{Object.keys(tasks).length}
        </span>
      </div>
      <ReactFlow
        nodes={history.replayGraph ? replayNodes : liveNodes}
        edges={history.replayGraph ? replayEdges : edges}
        nodeTypes={nodeTypes}
        edgeTypes={edgeTypes}
        minZoom={0.1}
        maxZoom={3}
        defaultViewport={{ x: 0, y: 0, zoom: 1 }}
        nodesFocusable
        edgesFocusable
        deleteKeyCode={null}
        multiSelectionKeyCode={null}
        onNodeClick={(_, node) => {
          if (!history.replayGraph) {
            handleSelect(node.id);
          }
        }}
        onNodeDragStop={(_, node: Node) => {
          if (history.replayGraph) {
            return;
          }
          setPositionOverrides((previous) => {
            const next = {
              ...previous,
              [node.id]: { x: node.position.x, y: node.position.y },
            };
            savePositions(runId, next);
            return next;
          });
          const task = tasks[node.id];
          if (!task || task.kind !== 'worker') {
            return;
          }
          const target = getIntersectingNodes(node).find((candidate) => {
            const candidateTask = tasks[candidate.id];
            return (
              candidate.id !== node.id &&
              candidateTask !== undefined &&
              (candidateTask.kind === 'superagent' || candidateTask.kind === 'hyperagent') &&
              candidate.id !== task.parentId
            );
          });
          if (target !== undefined) {
            selectTask(node.id);
            void rewireSelected(target.id);
          }
        }}
        onPaneClick={() => selectTask(null)}
        onMoveEnd={(_, viewport) => {
          setZoomLabel(zoomPercent(viewport.zoom));
          setViewport(viewport);
        }}
        fitView={history.replayGraph !== null}
        fitViewOptions={{ padding: 0.2, minZoom: 0.2, maxZoom: 1.4 }}
        className="h-full w-full bg-transparent"
      >
        <MiniMap
          pannable
          zoomable
          ariaLabel={STRINGS.minimap.aria}
          nodeColor={(node: Node) => {
            const kind = (node.data as { task?: { kind?: string } } | undefined)?.task?.kind;
            switch (kind) {
              case 'superagent':
                return 'var(--color-honey-400)';
              case 'worker':
                return 'var(--color-honey-600)';
              case 'tool':
                return 'var(--color-ink-500)';
              case 'error':
                return 'var(--color-signal)';
              default:
                return 'var(--color-honey-300)';
            }
          }}
          style={{
            width: LAYOUT.minimap.width,
            height: LAYOUT.minimap.height,
            background: 'var(--color-panel)',
            border: '1px solid var(--color-hairline)',
            borderRadius: 'var(--radius-md)',
          }}
        />
      </ReactFlow>

      {history.replayGraph ? (
        <div className="absolute left-1/2 top-4 -translate-x-1/2">
          <div className="type-label rounded-[var(--radius-pill)] border border-hairline bg-recessed px-3 py-1 text-ink-800">
            {STRINGS.states.replay} · {STRINGS.scrubber.label} {history.selected}
          </div>
        </div>
      ) : null}

      <div className="absolute bottom-32 left-3 flex flex-col gap-1" aria-label="Zoom controls">
        <button
          type="button"
          className="surface-panel h-8 w-8 rounded-[var(--radius-sm)] text-lg leading-none text-ink-800 hover:bg-honey-100"
          onClick={() => {
            void zoomIn({ duration });
          }}
          aria-label={STRINGS.controls.zoomIn}
        >
          +
        </button>
        <span className="type-mono-meta text-center text-meta">{zoomLabel}</span>
        <button
          type="button"
          className="surface-panel h-8 w-8 rounded-[var(--radius-sm)] text-lg leading-none text-ink-800 hover:bg-honey-100"
          onClick={() => {
            void zoomOut({ duration });
          }}
          aria-label={STRINGS.controls.zoomOut}
        >
          −
        </button>
        <button
          type="button"
          className="surface-panel h-8 w-8 rounded-[var(--radius-sm)] text-[10px] leading-none text-ink-800 hover:bg-honey-100"
          onClick={() => {
            setCenter(0, 0, { zoom: getZoom(), duration: 0 });
          }}
          aria-label={STRINGS.controls.recenter}
          title={STRINGS.controls.recenter}
        >
          0
        </button>
        <button
          type="button"
          className="surface-panel h-8 w-8 rounded-[var(--radius-sm)] text-[10px] leading-none text-ink-800 hover:bg-honey-100"
          onClick={() => {
            void fitView({ duration, padding: 0.2 });
          }}
          aria-label={STRINGS.controls.zoomFit}
        >
          ⤢
        </button>
      </div>

      {nodes.length === 0 && !history.replayGraph ? (
        <div className="pointer-events-none absolute inset-0 flex items-end justify-center pb-40">
          <p className="type-mono-meta text-meta">
            {Object.keys(tasks).length > 0 ? 'No agents match the current filters.' : STRINGS.panels.agentsEmpty}
          </p>
        </div>
      ) : null}

      <div aria-live="polite" className="sr-only">
        {selected ? `${selected.label}: ${selected.state}` : ''}
        {connection === 'reconnecting' ? STRINGS.connection.reconnectBanner : ''}
      </div>
    </div>
  );
}

export function Canvas() {
  return (
    <ReactFlowProvider>
      <CanvasInner />
    </ReactFlowProvider>
  );
}
