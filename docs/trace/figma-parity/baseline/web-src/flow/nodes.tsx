/**
 * Node cards (S15-S19): 4 node types + 1 broken-route error state (R-U5).
 * Every card renders the single state vocabulary with shape + text cue, never
 * colour alone. Metrics the wire contract does not deliver render "—".
 */
import { Handle, Position } from '@xyflow/react';
import type { CSSProperties, ReactNode } from 'react';
import { STRINGS, formatCount, formatCtx, formatPercent } from '../state/strings';
import { NODE_KIND_LABELS, visualFor, type AgentState } from '../state/agentState';
import type { CanvasNodeData } from './graph';

/** Local prop shape: the cards only consume the task payload (NodeTypes cast in Canvas). */
type HiveNodeProps = { readonly data: CanvasNodeData };

function tokenColor(token: string): string {
  return token.startsWith('honey-') || token.startsWith('ink-') || token === 'verify' || token === 'signal' || token === 'amber' || token === 'hairline'
    ? `var(--color-${token})`
    : 'var(--color-ink-500)';
}

export function StateGlyph({ state, size = 10 }: { state: AgentState; size?: number }) {
  const visual = visualFor(state);
  const color = tokenColor(visual.toneToken);
  const shape = visual.dot ?? visual.shape;
  return (
    <span className="inline-flex items-center gap-1" title={`${visual.label}`}>
      <svg width={size} height={size} viewBox="0 0 12 12" aria-hidden="true" style={{ color }}>
        {shape === 'dot' || shape === 'ring' ? (
          <circle cx="6" cy="6" r="4" fill={visual.dot ? 'currentColor' : 'none'} stroke="currentColor" strokeWidth={visual.dot ? 0 : 1.75} />
        ) : null}
        {shape === 'diamond' ? <polygon points="6,1 11,6 6,11 1,6" fill="currentColor" /> : null}
        {shape === 'square' ? <rect x="2" y="2" width="8" height="8" fill="currentColor" /> : null}
        {shape === 'triangle' ? <polygon points="6,1 11,10.5 1,10.5" fill="currentColor" /> : null}
        {shape === 'hex' ? <polygon points="6,1 10.5,3.5 10.5,8.5 6,11 1.5,8.5 1.5,3.5" fill="currentColor" /> : null}
      </svg>
      <span className="type-mono-meta" style={{ color }}>
        {visual.label}
      </span>
    </span>
  );
}

function Shell(props: {
  readonly data: CanvasNodeData;
  readonly title: string;
  readonly kindLabel: string;
  readonly caption: string;
  readonly children?: ReactNode;
  readonly error?: boolean;
}) {
  const { data } = props;
  const selected = data.selected;
  const style: CSSProperties = {
    borderColor: selected ? 'var(--color-honey-500)' : 'var(--color-hairline)',
    boxShadow: selected ? 'var(--shadow-e2)' : 'var(--shadow-e1)',
  };
  return (
    <div
      className={
        'surface-panel rounded-[var(--radius-card)] px-3 py-2 text-left transition-colors ' +
        (props.error ? 'border-signal bg-panel' : 'bg-panel')
      }
      style={style}
      data-node-kind={props.kindLabel.toLowerCase()}
    >
      <Handle type="target" position={Position.Left} className="!h-2 !w-2 !border-0 !bg-honey-500" />
      <button
        type="button"
        className="block w-full cursor-pointer text-left"
        onClick={() => data.onSelect(data.task.id)}
        aria-label={`${props.title} — ${props.kindLabel}, ${visualFor(data.task.state).label}`}
        aria-pressed={selected}
      >
        <div className="flex items-center justify-between gap-2">
          <span className="type-label text-ink-500">{props.kindLabel}</span>
          <StateGlyph state={data.task.state} />
        </div>
        <div className="type-mono-code mt-0.5 truncate text-ink-900">{props.title}</div>
        <div className="type-mono-meta mt-0.5 truncate text-meta">{props.caption}</div>
        {props.children}
      </button>
      <Handle type="source" position={Position.Right} className="!h-2 !w-2 !border-0 !bg-honey-500" />
    </div>
  );
}

function Progress({ value }: { value: number | null }) {
  return (
    <div className="mt-1.5">
      <div className="h-1 w-full overflow-hidden rounded-[var(--radius-pill)] bg-panel-alt">
        {value === null ? null : (
          <div
            className="h-full rounded-[var(--radius-pill)] bg-honey-500"
            style={{ width: `${Math.max(0, Math.min(100, value))}%` }}
          />
        )}
      </div>
      <div className="type-mono-meta mt-0.5 flex justify-between text-meta">
        <span>{STRINGS.node.progress}</span>
        <span>{value === null ? STRINGS.node.noMetrics : formatPercent(value, 0)}</span>
      </div>
    </div>
  );
}

export function HyperAgentNode({ data }: HiveNodeProps) {
  return (
    <Shell
      data={data}
      title={data.task.label}
      kindLabel={NODE_KIND_LABELS.hyperagent}
      caption={STRINGS.chat.title}
    >
      <div className="type-mono-meta mt-1 text-meta">{STRINGS.chat.pinned}</div>
    </Shell>
  );
}

export function SuperAgentNode({ data }: HiveNodeProps) {
  return (
    <Shell data={data} title={data.task.label} kindLabel={NODE_KIND_LABELS.superagent} caption={STRINGS.node.role}>
      <Progress value={data.task.progress} />
    </Shell>
  );
}

export function WorkerAgentNode({ data }: HiveNodeProps) {
  return (
    <Shell data={data} title={data.task.label} kindLabel={NODE_KIND_LABELS.worker} caption={STRINGS.node.workerLabel}>
      <div className="type-mono-meta mt-1 flex justify-between text-meta">
        <span>{formatCtx(null)}</span>
        <span>{STRINGS.node.progress}</span>
      </div>
      <Progress value={data.task.progress} />
    </Shell>
  );
}

export function ToolAgentNode({ data }: HiveNodeProps) {
  return (
    <Shell data={data} title={data.task.label} kindLabel={NODE_KIND_LABELS.tool} caption={STRINGS.node.toolLabel}>
      <div className="type-mono-meta mt-1 flex justify-between text-meta">
        <span>{STRINGS.node.score}</span>
        <span>{STRINGS.node.noMetrics}</span>
      </div>
    </Shell>
  );
}

export function BrokenRouteNode({ data }: HiveNodeProps) {
  return (
    <Shell
      data={data}
      title={data.task.label}
      kindLabel={NODE_KIND_LABELS.error}
      caption={data.task.errorCode ?? STRINGS.node.failed}
      error
    >
      <div className="type-mono-meta mt-1 flex justify-between text-meta">
        <span>{STRINGS.node.attempt}</span>
        <span>{formatCount(null)}</span>
      </div>
    </Shell>
  );
}

export const NODE_RENDERERS = {
  hyperagent: HyperAgentNode,
  superagent: SuperAgentNode,
  worker: WorkerAgentNode,
  tool: ToolAgentNode,
  error: BrokenRouteNode,
} as const;
