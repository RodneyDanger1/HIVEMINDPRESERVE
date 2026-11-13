/** S11 nav rail: 72px, six destinations, roving arrow-key focus. */
import { useRef } from 'react';
import { useUiStore } from '../state/context';
import { STRINGS } from '../state/strings';
import type { NavSection } from '../state/store';
import { nextRovingIndex } from '../a11y/keyboard';
import { NavIcon } from './NavIcon';

const ITEMS: ReadonlyArray<{ id: NavSection; label: string; hint: string }> = [
  { id: 'workspace', label: STRINGS.nav.workspace, hint: 'The live hive graph and the chat dock.' },
  { id: 'agents', label: STRINGS.nav.agents, hint: 'Agent roster, jobs, todos, memory, and proposals.' },
  { id: 'data', label: STRINGS.nav.data, hint: 'Runs, commits, evidence, and exported data.' },
  { id: 'models', label: STRINGS.nav.models, hint: 'Provider model catalogue, usage, and balance.' },
  { id: 'routing', label: STRINGS.nav.routing, hint: 'Model and task routing rules.' },
  { id: 'remote', label: STRINGS.nav.remote, hint: 'Remote principals, devices, grants, and daemons.' },
  { id: 'settings', label: STRINGS.nav.settings, hint: 'Provider, key, diagnostics, and FigCraft.' },
  { id: 'help', label: STRINGS.nav.help, hint: 'Tutorials and plain-English explanations of every feature.' },
];

export function NavRail() {
  const nav = useUiStore((state) => state.view.nav);
  const setNav = useUiStore((state) => state.setNav);
  const refs = useRef<Array<HTMLButtonElement | null>>([]);

  return (
    <nav
      aria-label={STRINGS.nav.aria}
      className="flex w-[72px] shrink-0 flex-col items-center gap-[20px] border-r border-hairline bg-panel py-3 max-[640px]:fixed max-[640px]:inset-x-0 max-[640px]:bottom-0 max-[640px]:z-40 max-[640px]:h-14 max-[640px]:w-full max-[640px]:flex-row max-[640px]:items-center max-[640px]:justify-around max-[640px]:gap-0 max-[640px]:border-r-0 max-[640px]:border-t max-[640px]:py-0"
      onKeyDown={(event) => {
        const forward = event.key === 'ArrowDown' || event.key === 'ArrowRight';
        const backward = event.key === 'ArrowUp' || event.key === 'ArrowLeft';
        if (!forward && !backward) {
          return;
        }
        event.preventDefault();
        const index = ITEMS.findIndex((item) => item.id === nav);
        const next = nextRovingIndex(index, forward ? 1 : -1, ITEMS.length);
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
          aria-label={item.label}
          title={`${item.label}: ${item.hint}`}
          aria-current={nav === item.id ? 'page' : undefined}
          tabIndex={nav === item.id ? 0 : -1}
          onClick={() => setNav(item.id)}
          className="flex w-full flex-col items-center gap-1.5 text-center max-[640px]:gap-0"
        >
          <span
            className={
              'flex h-12 w-12 items-center justify-center rounded-[var(--radius-button)] border ' +
              (nav === item.id
                ? 'border-ink-900 bg-ink-900'
                : 'border-transparent hover:border-hairline hover:bg-panel-alt')
            }
          >
            <NavIcon id={item.id} active={nav === item.id} />
          </span>
          <span
            className={
              'text-[11px] font-medium leading-tight max-[640px]:hidden ' +
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
