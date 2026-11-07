/**
 * HUD aggregate derivation (S09, MP-15, UI-R-50/R-U13): server RUN_METRICS is
 * authoritative when present; otherwise values are DERIVED from the graph and
 * labelled as derived. Absent values are null and render "—", never 0.
 */
import type { RunMetricsPayload } from '../api/contract';
import type { AgentState } from './agentState';

export interface TaskLike {
  readonly kind: 'hyperagent' | 'superagent' | 'worker' | 'tool' | 'error';
  readonly state: AgentState;
}

export interface HudMetrics {
  readonly superagents: number | null;
  readonly workers: number | null;
  readonly greenPct: number | null;
  readonly delta: number | null;
  readonly source: 'server' | 'derived' | 'none';
}

const GREEN_STATES: readonly AgentState[] = ['healthy', 'completed'];

export function deriveHud(
  tasks: Readonly<Record<string, TaskLike>>,
  server: RunMetricsPayload | null,
): HudMetrics {
  if (server) {
    return {
      superagents: server.superagents,
      workers: server.workers,
      greenPct: server.green_pct,
      delta: server.delta,
      source: 'server',
    };
  }
  const list = Object.values(tasks);
  if (list.length === 0) {
    return { superagents: null, workers: null, greenPct: null, delta: null, source: 'none' };
  }
  const superagents = list.filter(
    (task) => task.kind === 'superagent' || task.kind === 'hyperagent',
  ).length;
  const workers = list.filter((task) => task.kind === 'worker' || task.kind === 'tool').length;
  const green = list.filter((task) => GREEN_STATES.includes(task.state)).length;
  const greenPct = list.length > 0 ? (green / list.length) * 100 : null;
  return { superagents, workers, greenPct, delta: null, source: 'derived' };
}
