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
import { buildEdges, buildNodes, buildReplayTasks, type CanvasNode } from './graph';
import { NODE_RENDERERS } from './nodes';
import { BeeEdge } from './BeeEdge';
import { handleCanvasShortcut } from '../a11y/keyboard';
import { useUiStore } from '../state/context';
import { useAppReducedMotion } from '../state/useAppReducedMotion';
import { STRINGS } from '../state/strings';

const nodeTypes = NODE_RENDERERS as unknown as NodeTypes;
const edgeTypes: EdgeTypes = { bee: BeeEdge };

function zoomPercent(zoom: number): string {
  return `${Math.round(zoom * 100)}%`;
}

function CanvasInner() {
  const tasks = useUiStore((state) => state.tasks);
  const selectedTaskId = useUiStore((state) => state.selectedTaskId);
  const selectTask = useUiStore((state) => state.selectTask);
  const setViewport = useUiStore((state) => state.setViewport);
  const connection = useUiStore((state) => state.connection);
  const history = useUiStore((state) => state.history);
  const reduced = useAppReducedMotion();
  const { setCenter, getZoom, zoomIn, zoomOut, fitView } = useReactFlow();
  const [zoomLabel, setZoomLabel] = useState('100%');

  const handleSelect = useCallback(
    (taskId: string) => {
      selectTask(taskId);
    },
    [selectTask],
  );

  const nodes = useMemo<CanvasNode[]>(() => {
    if (history.replayGraph) {
      return [];
    }
    return buildNodes(tasks, selectedTaskId, handleSelect);
  }, [tasks, selectedTaskId, handleSelect, history.replayGraph]);

  const edges = useMemo<Edge[]>(() => buildEdges(tasks), [tasks]);

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
    () => buildNodes(replayTasks, null, noopSelect),
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
      <ReactFlow
        nodes={history.replayGraph ? replayNodes : nodes}
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
        onNodeClick={(_, node) => handleSelect(node.id)}
        onPaneClick={() => selectTask(null)}
        onMoveEnd={(_, viewport) => {
          setZoomLabel(zoomPercent(viewport.zoom));
          setViewport(viewport);
        }}
        fitView={!history.replayGraph}
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

      <div className="absolute bottom-3 left-3 flex flex-col gap-1" aria-label="Zoom controls">
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
        <div className="pointer-events-none absolute inset-0 flex items-end justify-center pb-24">
          <p className="type-mono-meta text-meta">{STRINGS.panels.agentsEmpty}</p>
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
