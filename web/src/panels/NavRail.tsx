/** S11 nav rail: 72px, six destinations, roving arrow-key focus. */
import { useRef } from 'react';
import { useUiStore } from '../state/context';
import { STRINGS } from '../state/strings';
import type { NavSection } from '../state/store';
import { nextRovingIndex } from '../a11y/keyboard';
import { NavIcon } from './NavIcon';

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
      className="flex w-[72px] shrink-0 flex-col items-center gap-[20px] border-r border-hairline bg-panel py-3"
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
          className="flex w-full flex-col items-center gap-1.5 text-center"
        >
          <span
            className={
              'flex h-12 w-12 items-center justify-center rounded-[11px] border ' +
              (nav === item.id
                ? 'border-ink-900 bg-ink-900'
                : 'border-transparent hover:border-hairline hover:bg-panel-alt')
            }
          >
            <NavIcon id={item.id} active={nav === item.id} />
          </span>
          <span
            className={
              'text-[8px] font-medium leading-tight ' +
              (nav === item.id ? 'text-ink-900' : 'text-ink-500')
            }
          >
            {item.label}
          </span>
        </button>
      ))}
    </nav>
  );
}
