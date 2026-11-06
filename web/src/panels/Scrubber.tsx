/** S10 time-travel scrubber: commits HISTORY_REVISION_SET on release, renders replay. */
import { useEffect, useState } from 'react';
import { useUiStore } from '../state/context';
import { STRINGS } from '../state/strings';

export function Scrubber() {
  const history = useUiStore((state) => state.history);
  const setHistoryRevision = useUiStore((state) => state.setHistoryRevision);
  const [value, setValue] = useState<number>(history.selected);

  useEffect(() => {
    setValue(history.selected);
  }, [history.selected]);

  const max = Math.max(history.latest, 0);
  const commit = (next: number): void => {
    void setHistoryRevision(next);
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
        max={max}
        step={1}
        value={value}
        disabled={max === 0}
        aria-label={STRINGS.scrubber.aria}
        className="h-1 flex-1 accent-[var(--color-honey-500)]"
        onChange={(event) => setValue(Number(event.target.value))}
        onPointerUp={() => commit(value)}
        onKeyUp={() => commit(value)}
      />
      <span className="type-mono-meta text-meta" aria-live="polite">
        {max === 0 ? STRINGS.scrubber.empty : `${value} ${STRINGS.scrubber.of} ${max}`}
      </span>
      <button
        type="button"
        className="type-mono-meta rounded-[var(--radius-sm)] border border-hairline px-2 py-1 text-ink-800 hover:bg-honey-100"
        onClick={() => commit(max)}
        disabled={max === 0}
      >
        {STRINGS.scrubber.live}
      </button>
    </footer>
  );
}
