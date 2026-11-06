import { describe, expect, it } from 'vitest';
import { deriveHud } from '../src/state/metrics';
import type { TaskRecord } from '../src/state/store';

function task(id: string, kind: TaskRecord['kind'], state: TaskRecord['state']): TaskRecord {
  return { id, parentId: null, kind, label: id, state, progress: null, errorCode: null, rev: 1 };
}

describe('HUD derivation (S09 / UI-R-50)', () => {
  it('returns nulls (never 0) when nothing exists', () => {
    const hud = deriveHud({}, null);
    expect(hud).toEqual({
      superagents: null,
      workers: null,
      greenPct: null,
      delta: null,
      source: 'none',
    });
  });

  it('uses server RUN_METRICS when present', () => {
    const hud = deriveHud(
      { a: task('a', 'worker', 'running') },
      { superagents: 4, workers: 12, green_pct: 98.2, delta: -3.4 },
    );
    expect(hud.source).toBe('server');
    expect(hud.superagents).toBe(4);
    expect(hud.workers).toBe(12);
    expect(hud.greenPct).toBe(98.2);
  });

  it('derives from the graph when metrics are absent, labelled derived', () => {
    const hud = deriveHud(
      {
        s: task('s', 'superagent', 'running'),
        w: task('w', 'worker', 'healthy'),
        t: task('t', 'tool', 'completed'),
      },
      null,
    );
    expect(hud.source).toBe('derived');
    expect(hud.superagents).toBe(1);
    expect(hud.workers).toBe(2);
    expect(hud.greenPct).toBeCloseTo((2 / 3) * 100);
    expect(hud.delta).toBeNull();
  });
});
