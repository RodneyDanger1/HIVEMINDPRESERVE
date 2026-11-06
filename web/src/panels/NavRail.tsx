/** S11 nav rail: 72px, six destinations, roving arrow-key focus. */
import { useRef } from 'react';
import { useUiStore } from '../state/context';
import { STRINGS } from '../state/strings';
import type { NavSection } from '../state/store';
import { nextRovingIndex } from '../a11y/keyboard';

const ITEMS: ReadonlyArray<{ id: NavSection; label: string }> = [
  { id: 'workspace', label: STRINGS.nav.workspace },
  { id: 'agents', label: STRINGS.nav.agents },
  { id: 'data', label: STRINGS.nav.data },
  { id: 'models', label: STRINGS.nav.models },
  { id: 'routing', label: STRINGS.nav.routing },
  { id: 'settings', label: STRINGS.nav.settings },
];

export function NavRail() {
  const nav = useUiStore((state) => state.view.nav);
  const setNav = useUiStore((state) => state.setNav);
  const refs = useRef<Array<HTMLButtonElement | null>>([]);

  return (
    <nav
      aria-label={STRINGS.nav.aria}
      className="flex w-[72px] shrink-0 flex-col items-center gap-1 border-r border-hairline bg-panel py-3"
      onKeyDown={(event) => {
        if (event.key !== 'ArrowDown' && event.key !== 'ArrowUp') {
          return;
        }
        event.preventDefault();
        const index = ITEMS.findIndex((item) => item.id === nav);
        const next = nextRovingIndex(index, event.key === 'ArrowDown' ? 1 : -1, ITEMS.length);
        refs.current[next]?.focus();
      }}
    >
      <div className="type-mono-meta mb-1 text-meta" aria-hidden="true">
        HM
      </div>
      {ITEMS.map((item, index) => (
        <button
          key={item.id}
          ref={(element) => {
            refs.current[index] = element;
          }}
          type="button"
          aria-current={nav === item.id ? 'page' : undefined}
          tabIndex={nav === item.id ? 0 : -1}
          onClick={() => setNav(item.id)}
          className={
            'flex h-11 w-14 items-center justify-center rounded-[var(--radius-md)] border px-1 text-center text-[10px] leading-tight ' +
            (nav === item.id
              ? 'border-honey-400 bg-honey-100 text-ink-900'
              : 'border-transparent text-ink-500 hover:border-hairline hover:bg-panel-alt')
          }
        >
          {item.label}
        </button>
      ))}
    </nav>
  );
}
