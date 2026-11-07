/** Global top bar (S08/S09/S28): brand, quick agent search, connection and operator. */
import { useState } from 'react';
import { useUiStore } from '../state/context';
import { STRINGS } from '../state/strings';

export function Header() {
  const connection = useUiStore((state) => state.connection);
  const project = useUiStore((state) => state.project);
  const tasks = useUiStore((state) => state.tasks);
  const selectTask = useUiStore((state) => state.selectTask);
  const [searchOpen, setSearchOpen] = useState(false);
  const [query, setQuery] = useState('');

  const results = Object.values(tasks)
    .filter((task) => task.label.toLowerCase().includes(query.trim().toLowerCase()))
    .slice(0, 6);

  return (
    <header className="relative flex h-[52px] shrink-0 items-center gap-3 border-b border-hairline bg-panel px-3">
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
        <button
          type="button"
          aria-label="Search agents"
          aria-expanded={searchOpen}
          className="flex h-8 w-8 items-center justify-center rounded-full text-ink-500 hover:bg-panel-alt hover:text-ink-900"
          onClick={() => setSearchOpen((open) => !open)}
        >
          <svg width="15" height="15" viewBox="0 0 20 20" aria-hidden="true">
            <circle cx="8.5" cy="8.5" r="5.2" fill="none" stroke="currentColor" strokeWidth="1.5" />
            <path d="m12.5 12.5 4 4" stroke="currentColor" strokeWidth="1.5" strokeLinecap="round" />
          </svg>
        </button>
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
      {searchOpen ? (
        <div className="surface-panel absolute right-3 top-[48px] z-50 w-[320px] rounded-[12px] border border-hairline p-2 shadow-[var(--shadow-e2)]">
          <label className="sr-only" htmlFor="agent-search">
            Search agents
          </label>
          <input
            id="agent-search"
            autoFocus
            value={query}
            onChange={(event) => setQuery(event.target.value)}
            onKeyDown={(event) => {
              if (event.key === 'Escape') {
                setSearchOpen(false);
              }
            }}
            placeholder="Find an agent…"
            className="w-full rounded-[8px] border border-hairline bg-panel-alt px-2 py-1.5 text-[12px] text-ink-900 outline-none focus-visible:ring-2 focus-visible:ring-[var(--color-ink-700)]"
          />
          <ul className="mt-1 max-h-[240px] overflow-y-auto">
            {results.length === 0 ? (
              <li className="type-mono-meta px-2 py-2 text-meta">No agents in this run.</li>
            ) : (
              results.map((task) => (
                <li key={task.id}>
                  <button
                    type="button"
                    className="flex w-full items-center justify-between gap-2 rounded-[6px] px-2 py-1.5 text-left hover:bg-honey-100"
                    onClick={() => {
                      selectTask(task.id);
                      setSearchOpen(false);
                    }}
                  >
                    <span className="truncate text-[12px] text-ink-900">{task.label}</span>
                    <span className="type-mono-meta shrink-0 text-meta">{task.state}</span>
                  </button>
                </li>
              ))
            )}
          </ul>
        </div>
      ) : null}
    </header>
  );
}
