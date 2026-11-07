/**
 * S10 time-travel scrubber: selects a committed revision (the store's exact
 * revision list, not a max-index guess), renders replay, and exposes the
 * store's explicit live exit so replay is never a one-way state.
 */
import { useEffect, useState } from 'react';
import { useUiStore } from '../state/context';
import { STRINGS } from '../state/strings';

export function Scrubber() {
  const history = useUiStore((state) => state.history);
  const run = useUiStore((state) => state.run);
  const setHistoryRevision = useUiStore((state) => state.setHistoryRevision);
  const exitHistoryReplay = useUiStore((state) => state.exitHistoryReplay);

  const revisions = history.revisions;
  const count = revisions.length > 0 ? revisions.length : Math.max(history.count, 0);
  const selectedAt = revisions.indexOf(history.selected);
  const selectedIndex =
    selectedAt >= 0 ? selectedAt : Math.max(0, Math.min(count - 1, history.selected));
  const [value, setValue] = useState<number>(selectedIndex);

  useEffect(() => {
    setValue(selectedIndex);
  }, [selectedIndex]);

  // A committed revision is availability: `latest` is -1 only when the run has
  // none, and revision 0 is a real committed state. The old `latest === 0`
  // heuristic must never disable a legitimately replayable run.
  const available = run !== null && count > 0;
  const inReplay = history.replayGraph !== null;
  const maxIndex = Math.max(count - 1, 0);
  const position = `${value + 1} ${STRINGS.scrubber.of} ${count}`;
  const commit = (next: number): void => {
    const revision = revisions[next];
    if (revision !== undefined) {
      void setHistoryRevision(revision);
    }
  };

  return (
    <footer className="flex items-center gap-3 border-t border-hairline bg-panel-alt px-4 py-2">
      <label className="type-label text-ink-500" htmlFor="scrubber">
        {STRINGS.scrubber.label}
      </label>
      <input
        id="scrubber"
        type="range"
        min={0}
        max={maxIndex}
        step={1}
        value={value}
        disabled={!available}
        aria-label={STRINGS.scrubber.aria}
        aria-valuetext={available ? position : STRINGS.scrubber.empty}
        className="h-1 flex-1 accent-[var(--color-honey-500)]"
        onChange={(event) => setValue(Number(event.target.value))}
        onPointerUp={(event) => commit(Number(event.currentTarget.value))}
        onKeyUp={(event) => commit(Number(event.currentTarget.value))}
      />
      <span className="type-mono-meta text-meta" aria-live="polite">
        {!available
          ? STRINGS.scrubber.empty
          : `${inReplay ? `${STRINGS.states.replay} · ` : ''}${position}`}
      </span>
      <button
        type="button"
        className="type-mono-meta rounded-[var(--radius-sm)] border border-hairline px-2 py-1 text-ink-800 hover:bg-honey-100 disabled:opacity-50"
        onClick={() => exitHistoryReplay()}
        disabled={!available || !inReplay}
      >
        {STRINGS.scrubber.live}
      </button>
    </footer>
  );
}
