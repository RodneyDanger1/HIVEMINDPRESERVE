/** Floating throughput HUD (S09), pinned above the hive without stealing the canvas. */
import { useMemo } from 'react';
import { useUiStore } from '../state/context';
import { STRINGS, formatCount, formatDelta, formatPercent } from '../state/strings';
import { deriveHud } from '../state/metrics';
import { LAYOUT } from '../tokens/tokens';

export function HudBadge() {
  const tasks = useUiStore((state) => state.tasks);
  const metrics = useUiStore((state) => state.metrics);
  const replaying = useUiStore((state) => state.history.replayGraph !== null);
  const hud = useMemo(() => deriveHud(tasks, metrics), [tasks, metrics]);

  return (
    <div className="pointer-events-none absolute left-1/2 top-3 z-20 -translate-x-1/2">
      <div
        className="type-mono-meta flex items-center gap-1.5 whitespace-nowrap rounded-[var(--radius-pill)] border bg-panel px-4 text-[8.5px] text-ink-800 shadow-[var(--shadow-e2)]"
        style={{
          width: LAYOUT.hud.width,
          height: LAYOUT.hud.height,
          borderColor: 'var(--color-border-gold)',
        }}
        role="status"
        aria-label={STRINGS.hud.aria}
      >
        <span
          className={
            'h-[6px] w-[6px] rounded-full ' +
            (hud.source === 'server' && hud.greenPct !== null
              ? 'bg-verify'
              : hud.source === 'derived'
                ? 'bg-honey-500'
                : 'bg-ink-500')
          }
          aria-hidden="true"
        />
        <span className="font-semibold">{STRINGS.node.hyperAgent}</span>
        <span aria-hidden="true">·</span>
        {replaying ? (
          <span className="text-ink-500">REPLAY · LIVE METRICS HIDDEN</span>
        ) : hud.source === 'none' ? (
          <span className="text-ink-500">{STRINGS.hud.noMetrics}</span>
        ) : (
          <>
            {hud.source === 'derived' ? (
              <span className="rounded-[var(--radius-pill)] bg-panel-alt px-1.5 text-[7.5px] text-ink-500">
                DERIVED
              </span>
            ) : null}
            <span>{formatCount(hud.superagents)} {STRINGS.hud.superagents}</span>
            <span aria-hidden="true">·</span>
            <span>{formatCount(hud.workers)} {STRINGS.hud.workers}</span>
            <span aria-hidden="true">·</span>
            <span>{formatPercent(hud.greenPct)} {STRINGS.hud.green}</span>
          </>
        )}
        {hud.delta === null ? null : (
          <span className="text-ink-800">{formatDelta(hud.delta)}</span>
        )}
      </div>
    </div>
  );
}
