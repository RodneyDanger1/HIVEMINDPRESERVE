/** Header: S08 tabs, S09 HUD, S28 project identity, connection state. */
import { useMemo } from 'react';
import { useUiStore } from '../state/context';
import { STRINGS, formatCount, formatDelta, formatPercent } from '../state/strings';
import { deriveHud } from '../state/metrics';
import { DerivedBadge } from './StatePack';
import type { TabId } from '../state/store';

const TABS: ReadonlyArray<{ id: TabId; label: string }> = [
  { id: 'workspace', label: STRINGS.tabs.workspace },
  { id: 'swarm', label: STRINGS.tabs.swarm },
  { id: 'routing', label: STRINGS.tabs.routing },
  { id: 'history', label: STRINGS.tabs.history },
];

export function Header() {
  const tab = useUiStore((state) => state.view.tab);
  const setTab = useUiStore((state) => state.setTab);
  const tasks = useUiStore((state) => state.tasks);
  const metrics = useUiStore((state) => state.metrics);
  const connection = useUiStore((state) => state.connection);
  const project = useUiStore((state) => state.project);
  const hud = useMemo(() => deriveHud(tasks, metrics), [tasks, metrics]);

  return (
    <header className="flex flex-wrap items-center gap-3 border-b border-hairline bg-panel-alt px-4 py-2">
      <div role="tablist" aria-label={STRINGS.tabs.aria} className="flex gap-1">
        {TABS.map((entry) => (
          <button
            key={entry.id}
            type="button"
            role="tab"
            aria-selected={tab === entry.id}
            onClick={() => setTab(entry.id)}
            className={
              'type-label rounded-[var(--radius-sm)] px-3 py-1 ' +
              (tab === entry.id ? 'bg-honey-100 text-ink-900' : 'text-ink-500 hover:bg-panel')
            }
          >
            {entry.label}
          </button>
        ))}
      </div>
      <div className="ml-auto flex flex-wrap items-center gap-3">
        <div
          className="type-mono-meta flex items-center gap-2 rounded-[var(--radius-pill)] border border-hairline bg-panel px-3 py-1 text-ink-500"
          role="status"
          aria-label={STRINGS.hud.aria}
        >
          <span>{formatCount(hud.superagents)}</span> {STRINGS.hud.superagents} ·{' '}
          <span>{formatCount(hud.workers)}</span> {STRINGS.hud.workers} ·{' '}
          <span>{formatPercent(hud.greenPct)}</span> {STRINGS.hud.green}
          {hud.delta === null ? null : (
            <span className="text-ink-800">({formatDelta(hud.delta)})</span>
          )}
          <DerivedBadge source={hud.source} />
        </div>
        <div className="text-right">
          <div className="type-mono-meta text-meta">{STRINGS.library.projectLabel}</div>
          <div className="type-mono-code text-ink-900">
            {project.name} · {project.owner}
          </div>
        </div>
        <div
          className={
            'type-mono-meta rounded-[var(--radius-pill)] border px-2 py-1 ' +
            (connection === 'open'
              ? 'border-verify text-verify'
              : connection === 'reconnecting' || connection === 'connecting'
                ? 'border-amber text-ink-800'
                : 'border-signal text-signal')
          }
          aria-live="polite"
        >
          {STRINGS.connection[connection]}
        </div>
      </div>
    </header>
  );
}
