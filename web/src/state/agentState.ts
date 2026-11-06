/**
 * ONE agent-state vocabulary (R-U4 / UI-COMMANDS-v1 §4). 15 states: the
 * contract's 14-state list plus `empty` (surface state). A superset of 13 is
 * allowed by the frozen contract; this module is the only vocabulary the UI
 * renders.
 */
import { STRINGS } from './strings';

export const AGENT_STATES = [
  'healthy',
  'running',
  'queued',
  'failed',
  'completed',
  'cancelled',
  'killed',
  'planning',
  'thinking',
  'awaiting_approval',
  'streaming',
  'reconnecting',
  'replay',
  'empty',
  'loading',
] as const;

export type AgentState = (typeof AGENT_STATES)[number];

export const DOT_STATES = ['healthy', 'running', 'failed', 'queued'] as const;
export type DotState = (typeof DOT_STATES)[number];

export const MIN_AGENT_STATE_COUNT = 13;

export function isAgentState(value: unknown): value is AgentState {
  return typeof value === 'string' && (AGENT_STATES as readonly string[]).includes(value);
}

export function isDotState(value: unknown): value is DotState {
  return typeof value === 'string' && (DOT_STATES as readonly string[]).includes(value);
}

/** The four PDF dots map 1:1; every other state renders a shape/text cue too. */
export function dotFor(state: AgentState): DotState | null {
  return isDotState(state) ? state : null;
}

export type StateShape = 'dot' | 'ring' | 'diamond' | 'square' | 'triangle' | 'hex';

export interface StateVisual {
  readonly state: AgentState;
  readonly label: string;
  readonly dot: DotState | null;
  readonly shape: StateShape;
  readonly toneToken: string;
}

const NON_DOT_VISUAL: Record<
  Exclude<AgentState, DotState>,
  { shape: StateShape; toneToken: string }
> = {
  completed: { shape: 'ring', toneToken: 'verify' },
  cancelled: { shape: 'square', toneToken: 'ink-500' },
  killed: { shape: 'square', toneToken: 'signal' },
  planning: { shape: 'diamond', toneToken: 'honey-500' },
  thinking: { shape: 'ring', toneToken: 'honey-500' },
  awaiting_approval: { shape: 'triangle', toneToken: 'amber' },
  streaming: { shape: 'ring', toneToken: 'honey-500' },
  reconnecting: { shape: 'triangle', toneToken: 'ink-500' },
  replay: { shape: 'hex', toneToken: 'ink-800' },
  empty: { shape: 'square', toneToken: 'hairline' },
  loading: { shape: 'ring', toneToken: 'ink-500' },
};

export function visualFor(state: AgentState): StateVisual {
  const label = STRINGS.agentStates[state];
  if (isDotState(state)) {
    return { state, label, dot: state, shape: 'dot', toneToken: state };
  }
  // `failed` is both a dot state and listed above; TypeScript narrows safely here.
  const visual = NON_DOT_VISUAL[state as Exclude<AgentState, DotState>];
  return { state, label, dot: null, shape: visual.shape, toneToken: visual.toneToken };
}

/** Transition matrix from UI-WIRING-GUIDE §3 (single enum, no second vocabulary). */
export const STATE_TRANSITIONS: Readonly<Record<AgentState, readonly AgentState[]>> = {
  healthy: ['running'],
  running: ['completed', 'failed', 'cancelled', 'killed', 'thinking', 'streaming', 'queued'],
  queued: ['running', 'cancelled'],
  failed: ['queued', 'running', 'completed'],
  completed: [],
  cancelled: [],
  killed: [],
  planning: ['running', 'thinking', 'queued', 'failed'],
  thinking: ['streaming', 'running', 'completed', 'failed', 'cancelled'],
  awaiting_approval: ['running', 'failed', 'cancelled', 'killed'],
  streaming: ['completed', 'failed', 'cancelled', 'killed', 'running'],
  reconnecting: ['healthy', 'failed'],
  replay: [],
  empty: ['loading', 'healthy'],
  loading: ['healthy', 'empty', 'failed'],
};

export function canTransition(from: AgentState, to: AgentState): boolean {
  return STATE_TRANSITIONS[from].includes(to);
}

/** States that block mutating controls until resolved. */
export function isTerminal(state: AgentState): boolean {
  return state === 'completed' || state === 'cancelled' || state === 'killed';
}

export function isRunActive(state: AgentState): boolean {
  return !isTerminal(state) && state !== 'empty' && state !== 'loading';
}

/** Node type vocabulary (4 node types + 1 broken-route error state, R-U5). */
export const NODE_KIND_KEYS = ['hyperagent', 'superagent', 'worker', 'tool', 'error'] as const;
export type NodeKindLabel = (typeof NODE_KIND_KEYS)[number];

export const NODE_KIND_LABELS: Readonly<Record<NodeKindLabel, string>> = {
  hyperagent: STRINGS.node.hyperAgent,
  superagent: STRINGS.node.superAgent,
  worker: STRINGS.node.workerAgent,
  tool: STRINGS.node.toolAgent,
  error: STRINGS.node.errorState,
};
