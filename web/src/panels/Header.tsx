/** Global top bar (S08/S09/S28): brand, connection and operator. */
import { useUiStore } from '../state/context';
import { STRINGS } from '../state/strings';

export function Header() {
  const connection = useUiStore((state) => state.connection);
  const project = useUiStore((state) => state.project);

  return (
    <header className="flex h-[52px] shrink-0 items-center gap-3 border-b border-hairline bg-panel px-3">
      <div className="flex min-w-0 items-center gap-2">
        <svg width="28" height="28" viewBox="0 0 28 28" aria-hidden="true" className="shrink-0">
          <path
            d="M14 2.2 24 8v12L14 25.8 4 20V8Z"
            fill="var(--color-ink-900)"
            stroke="var(--color-honey-500)"
            strokeWidth="1"
          />
          <path
            d="M14 8.4 19 11.3v5.4L14 19.6 9 16.7v-5.4Z"
            fill="var(--color-honey-400)"
          />
        </svg>
        <span className="truncate text-[17px] font-bold leading-none text-ink-900">HiveMind</span>
      </div>

      <div className="ml-auto flex shrink-0 items-center gap-3">
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
        <div className="flex items-center gap-2 border-l border-hairline pl-3">
          <span className="flex h-8 w-8 items-center justify-center rounded-full bg-honey-200 text-[11px] font-semibold text-ink-800">
            {project.owner === 'You' ? 'HM' : project.owner.slice(0, 2).toUpperCase()}
          </span>
          <span className="type-mono-meta hidden text-ink-800 sm:block">{project.owner}</span>
        </div>
      </div>
    </header>
  );
}
