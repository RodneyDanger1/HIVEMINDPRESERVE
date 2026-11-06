/**
 * App shell (milestone 1): chrome only — nav rail (72px), workspace tabs, HUD,
 * canvas field, pinned chat-core placeholder, library (260px), scrubber.
 * The wired 28-surface implementation (canvas/nodes/store/API client) lands
 * next; this shell contains no fabricated data: every unknown metric renders
 * the contract's null glyph (`STRINGS.node.noMetrics`), never 0.
 */
import { STRINGS } from './state/strings';

const NAV_ITEMS = [
  STRINGS.nav.workspace,
  STRINGS.nav.agents,
  STRINGS.nav.data,
  STRINGS.nav.models,
  STRINGS.nav.routing,
  STRINGS.nav.settings,
] as const;

const TAB_ITEMS = [
  STRINGS.tabs.workspace,
  STRINGS.tabs.swarm,
  STRINGS.tabs.routing,
  STRINGS.tabs.history,
] as const;

export function App() {
  return (
    <div className="flex h-full min-h-0 bg-canvas text-ink-900">
      <a className="skip-link" href="#hive-canvas">
        {STRINGS.app.skipToCanvas}
      </a>

      {/* S11 — nav rail: 72px fixed, six destinations */}
      <nav
        aria-label={STRINGS.nav.aria}
        className="flex w-[72px] shrink-0 flex-col items-center gap-2 border-r border-hairline bg-panel py-4"
      >
        <div className="type-mono-meta mb-2 text-ink-500" aria-hidden="true">
          HM
        </div>
        {NAV_ITEMS.map((label, index) => (
          <button
            key={label}
            type="button"
            aria-current={index === 0 ? 'page' : undefined}
            className={
              'flex h-12 w-12 items-center justify-center rounded-[var(--radius-md)] border text-[10px] leading-tight ' +
              (index === 0
                ? 'border-honey-400 bg-honey-100 text-ink-900'
                : 'border-transparent text-ink-500 hover:border-hairline hover:bg-panel-alt')
            }
          >
            <span className="px-1 text-center">{label}</span>
          </button>
        ))}
      </nav>

      <main className="flex min-w-0 flex-1 flex-col">
        {/* workspace tabs (S08) + HUD (S09) + project identity (S28) */}
        <header className="flex flex-wrap items-center gap-3 border-b border-hairline bg-panel-alt px-4 py-2">
          <div role="tablist" aria-label={STRINGS.tabs.aria} className="flex gap-1">
            {TAB_ITEMS.map((label, index) => (
              <button
                key={label}
                type="button"
                role="tab"
                aria-selected={index === 0}
                className={
                  'type-label rounded-[var(--radius-sm)] px-3 py-1 ' +
                  (index === 0 ? 'bg-honey-100 text-ink-900' : 'text-ink-500 hover:bg-panel')
                }
              >
                {label}
              </button>
            ))}
          </div>
          <div className="ml-auto flex items-center gap-4">
            <div
              className="type-mono-meta rounded-[var(--radius-pill)] border border-hairline bg-panel px-3 py-1 text-ink-500"
              role="status"
              aria-label={STRINGS.hud.aria}
            >
              <span>{STRINGS.node.noMetrics}</span> {STRINGS.hud.superagents} ·{' '}
              <span>{STRINGS.node.noMetrics}</span> {STRINGS.hud.workers} ·{' '}
              <span>{STRINGS.node.noMetrics}</span> {STRINGS.hud.green}
            </div>
            <div className="text-right">
              <div className="type-mono-meta text-ink-500">{STRINGS.library.projectLabel}</div>
              <div className="type-mono-code text-ink-900">{STRINGS.library.projectFallback}</div>
            </div>
          </div>
        </header>

        {/* S01 canvas field + S02 pinned chat core placeholder */}
        <section
          id="hive-canvas"
          aria-label={STRINGS.app.title}
          className="relative min-h-0 flex-1 overflow-hidden bg-canvas"
        >
          <svg className="hive-canvas-field" aria-hidden="true" width="100%" height="100%">
            <defs>
              <pattern id="honeycomb" width="48" height="42" patternUnits="userSpaceOnUse">
                <path
                  d="M12 0L36 0L48 21L36 42L12 42L0 21Z"
                  fill="none"
                  stroke="currentColor"
                  strokeWidth="1"
                />
              </pattern>
            </defs>
            <rect width="100%" height="100%" fill="url(#honeycomb)" />
          </svg>
          <div className="absolute left-1/2 top-1/2 w-[min(560px,80%)] -translate-x-1/2 -translate-y-1/2">
            <div className="surface-panel rounded-[var(--radius-card)] p-4">
              <div className="type-heading-h3">{STRINGS.chat.title}</div>
              <p className="type-body-l mt-2 text-ink-500">{STRINGS.chat.empty}</p>
              <div className="mt-3 flex items-center gap-2">
                <input
                  className="type-mono-code min-w-0 flex-1 rounded-[var(--radius-md)] border border-hairline bg-panel-alt px-3 py-2"
                  placeholder={STRINGS.composer.placeholder}
                  aria-label={STRINGS.chat.aria}
                  disabled
                />
                <button
                  type="button"
                  className="type-label rounded-[var(--radius-button)] bg-honey-300 px-4 py-2 text-ink-900"
                  disabled
                >
                  {STRINGS.composer.submit}
                </button>
              </div>
              <p className="type-mono-meta mt-2 text-ink-500">{STRINGS.composer.hint}</p>
            </div>
          </div>
        </section>

        {/* S10 time-travel scrubber placeholder (no run yet) */}
        <footer className="flex items-center gap-3 border-t border-hairline bg-panel-alt px-4 py-2">
          <label className="type-label text-ink-500" htmlFor="scrubber">
            {STRINGS.scrubber.label}
          </label>
          <input
            id="scrubber"
            type="range"
            min={0}
            max={0}
            value={0}
            disabled
            aria-label={STRINGS.scrubber.aria}
            className="h-1 flex-1"
          />
          <span className="type-mono-meta text-ink-500">{STRINGS.scrubber.empty}</span>
        </footer>
      </main>

      {/* S12 library panel — 260px */}
      <aside
        aria-label={STRINGS.library.aria}
        className="hidden w-[260px] shrink-0 flex-col gap-4 border-l border-hairline bg-panel p-3 lg:flex"
      >
        <div>
          <div className="type-label text-ink-500">{STRINGS.library.title}</div>
          <div className="type-mono-code mt-1 text-ink-900">{STRINGS.library.projectFallback}</div>
        </div>
        <div>
          <div className="type-label text-ink-500">{STRINGS.library.activeAgents}</div>
          <p className="type-mono-meta mt-1 text-ink-500">{STRINGS.library.emptyAgents}</p>
        </div>
        <div>
          <div className="type-label text-ink-500">{STRINGS.library.templates}</div>
          <ul className="mt-1 space-y-1">
            {[
              STRINGS.templates.contentWriter,
              STRINGS.templates.dataAnalyst,
              STRINGS.templates.codeReviewer,
              STRINGS.templates.chatbot,
            ].map((template) => (
              <li key={template}>
                <button
                  type="button"
                  className="type-mono-code w-full rounded-[var(--radius-sm)] border border-hairline bg-panel-alt px-2 py-1 text-left hover:bg-honey-100"
                >
                  {template}
                </button>
              </li>
            ))}
          </ul>
        </div>
        <p className="type-mono-meta mt-auto text-ink-500">{STRINGS.library.tip}</p>
      </aside>

      {/* S14 toast/refusal region (empty until the wire layer lands) */}
      <div
        aria-live="assertive"
        aria-label={STRINGS.toast.regionAria}
        className="pointer-events-none fixed right-4 top-4 z-50 flex w-[min(420px,90vw)] flex-col gap-2"
      />
    </div>
  );
}
