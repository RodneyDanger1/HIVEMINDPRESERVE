/**
 * Node cards (S15-S19), matching the HiveRouter spatial style system:
 * SuperAgent 420x150 r14 with a honey header, Worker 320x96 r12 with a state
 * dot, Tool 420x240 r14 with a dark result box, Broken route 320x110 crimson.
 * Metrics the wire contract does not deliver render "—", never a fabricated 0.
 */
import { Handle, Position } from '@xyflow/react';
import type { CSSProperties, ReactNode } from 'react';
import { STRINGS, formatCount, formatCtx, formatPercent } from '../state/strings';
import { NODE_KIND_LABELS, visualFor, type AgentState } from '../state/agentState';
import type { CanvasNodeData } from './graph';

type HiveNodeProps = { readonly data: CanvasNodeData };

function tokenColor(token: string): string {
  return token.startsWith('honey-') || token.startsWith('ink-') || token === 'verify' || token === 'signal' || token === 'amber' || token === 'hairline'
    ? `var(--color-${token})`
    : 'var(--color-ink-500)';
}

function stateTone(state: AgentState): string {
  switch (state) {
    case 'healthy':
      return 'var(--color-honey-400)';
    case 'running':
      return 'var(--color-honey-600)';
    case 'failed':
    case 'killed':
    case 'interrupted':
      return 'var(--color-signal)';
    case 'queued':
      return 'var(--color-hairline)';
    case 'completed':
      return 'var(--color-verify)';
    default:
      return 'var(--color-ink-500)';
  }
}

/** A state marker with no text label, for 40px chips and card headers. */
export function StateDot({ state, size = 8 }: { state: AgentState; size?: number }) {
  const active = state === 'running' || state === 'thinking' || state === 'streaming';
  return (
    <svg
      width={size}
      height={size}
      viewBox="0 0 12 12"
      aria-hidden="true"
      className={active ? 'state-pulse' : undefined}
      style={{ color: stateTone(state) }}
    >
      <circle cx="6" cy="6" r="4" fill="currentColor" />
    </svg>
  );
}

export function StateGlyph({ state, size = 10 }: { state: AgentState; size?: number }) {
  const visual = visualFor(state);
  const color = visual.dot === null ? tokenColor(visual.toneToken) : stateTone(state);
  const shape = visual.dot ?? visual.shape;
  return (
    <span className="inline-flex items-center gap-1" title={visual.label}>
      <svg width={size} height={size} viewBox="0 0 12 12" aria-hidden="true" style={{ color }}>
        {shape === 'dot' || shape === 'ring' ? (
          <circle cx="6" cy="6" r="4" fill={visual.dot ? 'currentColor' : 'none'} stroke="currentColor" strokeWidth={visual.dot ? 0 : 1.75} />
        ) : null}
        {shape === 'diamond' ? <polygon points="6,1 11,6 6,11 1,6" fill="currentColor" /> : null}
        {shape === 'square' ? <rect x="2" y="2" width="8" height="8" fill="currentColor" /> : null}
        {shape === 'triangle' ? <polygon points="6,1 11,10.5 1,10.5" fill="currentColor" /> : null}
        {shape === 'hex' ? <polygon points="6,1 10.5,3.5 10.5,8.5 6,11 1.5,8.5 1.5,3.5" fill="currentColor" /> : null}
      </svg>
      <span className="type-mono-meta" style={{ color }}>{visual.label}</span>
    </span>
  );
}

function Frame(props: {
  readonly data: CanvasNodeData;
  readonly children: ReactNode;
  readonly error?: boolean;
  readonly gold?: boolean;
  readonly radius?: number;
}) {
  const selected = props.data.selected;
  const style: CSSProperties = {
    borderColor: selected
      ? 'var(--color-ink-700)'
      : props.error
        ? 'var(--color-signal)'
        : props.gold
          ? 'var(--color-border-gold)'
          : 'var(--color-hairline)',
    boxShadow: selected ? 'var(--shadow-e2)' : 'var(--shadow-e1)',
    borderRadius: props.radius ?? 12,
  };
  return (
    <div
      className="hive-node relative flex h-full w-full flex-col overflow-hidden border-[1.5px] bg-panel text-left"
      style={style}
      data-node-kind={props.data.task.kind}
    >
      <Handle type="target" position={Position.Left} className="!h-2 !w-2 !border-0 !bg-honey-500" />
      {props.children}
      <Handle type="source" position={Position.Right} className="!h-2 !w-2 !border-0 !bg-honey-500" />
    </div>
  );
}

function Progress({ value, height = 5 }: { value: number | null; height?: number }) {
  return (
    <div
      className="w-full overflow-hidden rounded-[var(--radius-pill)] bg-recessed"
      style={{ height }}
    >
      {value === null ? null : (
        <div
          className="h-full rounded-[var(--radius-pill)] bg-honey-500"
          style={{ width: `${Math.max(0, Math.min(100, value))}%` }}
        />
      )}
    </div>
  );
}

export function SuperAgentNode({ data }: HiveNodeProps) {
  return (
    <Frame data={data} radius={14} gold>
      <div className="flex h-10 items-center justify-between gap-2 bg-honey-100 px-4">
        <button
          type="button"
          className="min-w-0 truncate text-left text-[14.5px] font-semibold leading-[18.5px] text-ink-900"
          onClick={() => data.onSelect(data.task.id)}
          aria-label={`${data.task.label} — ${NODE_KIND_LABELS.superagent}, ${visualFor(data.task.state).label}`}
          aria-pressed={data.selected}
        >
          {data.task.label}
        </button>
        <span className="flex shrink-0 items-center rounded-[var(--radius-pill)] bg-honey-300 px-2 py-0.5">
          <span className="type-mono-meta text-[8.5px] text-ink-800">{visualFor(data.task.state).label}</span>
        </span>
      </div>
      <button type="button" className="flex min-h-0 flex-1 flex-col px-4 py-2 text-left" onClick={() => data.onSelect(data.task.id)}>
        <span className="type-mono-meta text-meta">{NODE_KIND_LABELS.superagent}</span>
        <span className="mt-0.5 line-clamp-1 text-[11.5px] leading-[16px] text-ink-700">
          {STRINGS.node.roleCaption}
        </span>
        <div className="mt-auto">
          <Progress value={data.task.progress} />
          <div className="type-mono-meta mt-1 flex justify-between text-meta">
            <span>{STRINGS.node.workersMetric}</span>
            <span>{data.task.progress === null ? STRINGS.node.noMetrics : formatPercent(data.task.progress, 0)}</span>
          </div>
        </div>
      </button>
    </Frame>
  );
}

export function WorkerAgentNode({ data }: HiveNodeProps) {
  return (
    <Frame data={data} radius={12}>
      <button type="button" className="flex min-h-0 flex-1 flex-col px-4 py-3 text-left" onClick={() => data.onSelect(data.task.id)}>
        <div className="flex items-center gap-2">
          <StateDot state={data.task.state} size={9} />
          <span className="min-w-0 flex-1 truncate text-[13px] font-medium leading-[18px] text-ink-900">{data.task.label}</span>
          <span className="type-mono-meta text-meta">
            {data.task.progress === null ? STRINGS.node.noMetrics : formatPercent(data.task.progress, 0)}
          </span>
        </div>
        <span className="type-mono-meta mt-1 text-meta">{STRINGS.node.workerLabel}</span>
        <div className="mt-auto">
          <Progress value={data.task.progress} height={3} />
          <span className="type-mono-meta mt-1 block text-meta">{formatCtx(null)}</span>
        </div>
      </button>
    </Frame>
  );
}

export function ToolAgentNode({ data }: HiveNodeProps) {
  return (
    <Frame data={data} radius={14}>
      <button type="button" className="flex min-h-0 flex-1 flex-col px-4 py-3 text-left" onClick={() => data.onSelect(data.task.id)}>
        <div className="flex items-center justify-between gap-2">
          <span className="truncate text-[13.5px] font-semibold text-ink-900">{data.task.label}</span>
          <span className="type-mono-meta rounded-[var(--radius-pill)] bg-recessed px-2 py-0.5 text-ink-700">
            {NODE_KIND_LABELS.tool}
          </span>
        </div>
        <div className="mt-2 flex min-h-0 flex-1 items-center rounded-[8px] bg-ink-800 px-3 py-2">
          <span className="type-mono-meta text-honey-100/80">{STRINGS.node.noToolOutput}</span>
        </div>
        <div className="mt-2">
          <Progress value={data.task.progress} height={6} />
          <div className="type-mono-meta mt-1 flex justify-between text-meta">
            <span>{STRINGS.node.score}</span>
            <span>{STRINGS.node.noMetrics}</span>
          </div>
        </div>
      </button>
    </Frame>
  );
}

export function BrokenRouteNode({ data }: HiveNodeProps) {
  return (
    <Frame data={data} error radius={12}>
      <button type="button" className="flex min-h-0 flex-1 flex-col px-4 py-3 text-left" onClick={() => data.onSelect(data.task.id)}>
        <div className="flex items-center gap-2 text-signal">
          <span aria-hidden="true">▲</span>
          <span className="truncate text-[13px] font-semibold">{data.task.label}</span>
        </div>
        <span className="type-mono-meta mt-1 text-meta">{data.task.errorCode ?? STRINGS.node.errorState}</span>
        <span className="type-mono-meta mt-2 text-ink-700">{STRINGS.node.brokenRoute}</span>
        <div className="mt-auto type-mono-meta flex justify-between text-meta">
          <span>{STRINGS.node.attempt}</span>
          <span>{formatCount(null)}</span>
        </div>
      </button>
    </Frame>
  );
}

export function HyperAgentNode({ data }: HiveNodeProps) {
  return (
    <div className="relative flex h-full w-full items-center justify-center" data-node-kind="hyperagent">
      <Handle type="target" position={Position.Left} className="!h-2 !w-2 !border-0 !bg-honey-500" />
      <div
        aria-hidden="true"
        className="absolute inset-3 border-[1.5px] border-honey-400/70 bg-honey-100/20"
        style={{ clipPath: 'polygon(25% 6.7%, 75% 6.7%, 100% 50%, 75% 93.3%, 25% 93.3%, 0 50%)' }}
      />
      <button
        type="button"
        className="relative z-10 rounded-[var(--radius-pill)] border border-honey-500 bg-panel/90 px-3 py-1 text-center"
        onClick={() => data.onSelect(data.task.id)}
        aria-label={`${data.task.label} — ${NODE_KIND_LABELS.hyperagent}, ${visualFor(data.task.state).label}`}
        aria-pressed={data.selected}
      >
        <span className="type-mono-meta block text-meta">{NODE_KIND_LABELS.hyperagent}</span>
        <span className="block text-[14.5px] font-semibold leading-[18.5px] text-ink-900">{data.task.label}</span>
      </button>
      <Handle type="source" position={Position.Right} className="!h-2 !w-2 !border-0 !bg-honey-500" />
    </div>
  );
}

export const NODE_RENDERERS = {
  hyperagent: HyperAgentNode,
  superagent: SuperAgentNode,
  worker: WorkerAgentNode,
  tool: ToolAgentNode,
  error: BrokenRouteNode,
} as const;
