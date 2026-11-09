/** Settings: tabbed surface for provider health, appearance, diagnostics and FigCraft evidence. */
import { useEffect, useRef, useState } from 'react';
import { useUiStore } from '../state/context';
import { STRINGS } from '../state/strings';
import { SWATCHES } from '../tokens/tokens';
import { PanelTitle, LoadingState, ErrorState } from './StatePack';
import { DevGallery } from './DevGallery';
import { DiagnosticsPanel } from './DiagnosticsPanel';
import { FigCraftPanel } from './FigCraftPanel';

type SettingsTab = 'overview' | 'provider' | 'appearance' | 'diagnostics' | 'figcraft';

const TABS: readonly { id: SettingsTab; label: string; icon: SettingsIconName }[] = [
  { id: 'overview', label: 'Overview', icon: 'hive' },
  { id: 'provider', label: 'Provider', icon: 'signal' },
  { id: 'appearance', label: 'Appearance', icon: 'sliders' },
  { id: 'diagnostics', label: 'Diagnostics', icon: 'pulse' },
  { id: 'figcraft', label: 'FigCraft', icon: 'grid' },
];

type SettingsIconName = 'hive' | 'signal' | 'sliders' | 'pulse' | 'grid';

const SHORTCUTS: readonly { readonly keys: readonly string[]; readonly action: string }[] = [
  { keys: ['Ctrl', '0'], action: 'Recentre the hive' },
  { keys: ['Ctrl', '='], action: 'Zoom in' },
  { keys: ['Ctrl', '-'], action: 'Zoom out' },
  { keys: ['Ctrl', '1'], action: 'Fit the swarm' },
  { keys: ['Enter'], action: 'Submit the chat prompt' },
  { keys: ['Shift', 'Enter'], action: 'New line in the composer' },
  { keys: ['Esc'], action: 'Close a confirmation or approval' },
  { keys: ['←', '→'], action: 'Move between tabs' },
];

function ShortcutCard() {
  return (
    <section className="surface-panel mt-3 rounded-[var(--radius-md)] p-3">
      <div className="type-label text-ink-500">Keyboard</div>
      <p className="type-body-l mt-1 text-ink-500">
        Shortcuts are active only when focus is outside an input, except in the composer.
      </p>
      <dl className="mt-2 grid grid-cols-[1fr_auto] gap-x-3 gap-y-1">
        {SHORTCUTS.map((shortcut) => (
          <div key={shortcut.action} className="contents">
            <dt className="type-mono-meta text-meta">{shortcut.action}</dt>
            <dd className="flex gap-1">
              {shortcut.keys.map((key) => (
                <kbd
                  key={key}
                  className="type-mono-meta rounded-[var(--radius-xs)] border border-hairline bg-panel-alt px-1.5 py-0.5 text-[11px] text-ink-800"
                >
                  {key}
                </kbd>
              ))}
            </dd>
          </div>
        ))}
      </dl>
    </section>
  );
}

function SettingsIcon({ name }: { name: SettingsIconName }) {  return (
    <svg width="14" height="14" viewBox="0 0 16 16" aria-hidden="true" focusable="false">
      {name === 'hive' ? (
        <path d="M8 1.5 14 5v6l-6 3.5L2 11V5Z M8 4.5 11.5 6.4v3.2L8 11.5 4.5 9.6V6.4Z" fill="none" stroke="currentColor" strokeWidth="1.1" />
      ) : null}
      {name === 'signal' ? (
        <>
          <path d="M2 13V9M6 13V6M10 13V3M14 13V1" stroke="currentColor" strokeWidth="1.2" strokeLinecap="round" />
          <path d="M2 15h12" stroke="currentColor" strokeWidth="1" opacity="0.45" />
        </>
      ) : null}
      {name === 'sliders' ? (
        <>
          <path d="M2 4h12M2 8h12M2 12h12" stroke="currentColor" strokeWidth="1.1" />
          <circle cx="5" cy="4" r="1.5" fill="var(--color-panel)" stroke="currentColor" strokeWidth="1" />
          <circle cx="10" cy="8" r="1.5" fill="var(--color-panel)" stroke="currentColor" strokeWidth="1" />
          <circle cx="7" cy="12" r="1.5" fill="var(--color-panel)" stroke="currentColor" strokeWidth="1" />
        </>
      ) : null}
      {name === 'pulse' ? (
        <path d="M1.5 8h3l1.4-3.2L8.5 11l1.5-3H14" fill="none" stroke="currentColor" strokeWidth="1.2" strokeLinejoin="round" />
      ) : null}
      {name === 'grid' ? (
        <>
          <rect x="2" y="2" width="4.5" height="4.5" rx="0.8" fill="none" stroke="currentColor" strokeWidth="1" />
          <rect x="9.5" y="2" width="4.5" height="4.5" rx="0.8" fill="none" stroke="currentColor" strokeWidth="1" />
          <rect x="2" y="9.5" width="4.5" height="4.5" rx="0.8" fill="none" stroke="currentColor" strokeWidth="1" />
          <rect x="9.5" y="9.5" width="4.5" height="4.5" rx="0.8" fill="none" stroke="currentColor" strokeWidth="1" />
        </>
      ) : null}
    </svg>
  );
}

function ambientLabel(value: number): string {
  if (value === 0) return 'still';
  if (value < 55) return 'calm';
  return 'alive';
}

function readAmbientPreference(): number {
  try {
    const raw = window.localStorage.getItem('hivemind.ambient-motion');
    const parsed = raw === null ? 100 : Number(raw);
    return Number.isFinite(parsed) ? Math.min(100, Math.max(0, parsed)) : 100;
  } catch {
    return 100;
  }
}

export function SettingsPanel() {
  const settings = useUiStore((state) => state.settings);
  const probeSettings = useUiStore((state) => state.probeSettings);
  const probeHealth = useUiStore((state) => state.probeHealth);
  const reducedOverride = useUiStore((state) => state.reducedMotionOverride);
  const setReducedMotionOverride = useUiStore((state) => state.setReducedMotionOverride);
  const probed = useRef(false);
  const [tab, setTab] = useState<SettingsTab>('overview');
  const [ambient, setAmbient] = useState<number>(readAmbientPreference);

  useEffect(() => {
    if (probed.current) {
      return;
    }
    probed.current = true;
    void probeSettings();
    void probeHealth();
  }, [probeHealth, probeSettings]);

  useEffect(() => {
    const level = ambient === 0 ? 'off' : ambient < 55 ? 'calm' : 'full';
    document.documentElement.dataset.ambientMotion = level;
    document.documentElement.style.setProperty('--ambient-strength', String(ambient / 100));
    try {
      window.localStorage.setItem('hivemind.ambient-motion', String(ambient));
    } catch {
      // A privacy shell may reject localStorage; the in-memory control still works.
    }
  }, [ambient]);

  const keyLabel =
    settings.keyPresent === null
      ? STRINGS.settings.keyUnknown
      : settings.keyPresent
        ? STRINGS.settings.keyPresent
        : STRINGS.settings.keyMissing;

  const selectTab = (next: SettingsTab): void => {
    setTab(next);
    requestAnimationFrame(() => document.getElementById(`settings-tab-${next}`)?.focus());
  };

  const onTabKeyDown = (event: React.KeyboardEvent<HTMLDivElement>): void => {
    if (!['ArrowLeft', 'ArrowRight', 'Home', 'End'].includes(event.key)) {
      return;
    }
    event.preventDefault();
    const current = TABS.findIndex((entry) => entry.id === tab);
    const next =
      event.key === 'Home'
        ? 0
        : event.key === 'End'
          ? TABS.length - 1
          : (current + (event.key === 'ArrowRight' ? 1 : -1) + TABS.length) % TABS.length;
    const target = TABS[next];
    if (target !== undefined) {
      selectTab(target.id);
    }
  };

  const keySection = (
    <section className="surface-panel rounded-[var(--radius-md)] p-3" aria-label={STRINGS.settings.keyTitle}>
      <div className="flex items-center justify-between gap-2">
        <div className="type-label text-ink-500">{STRINGS.settings.keyTitle}</div>
        <button
          type="button"
          className="type-label h-8 rounded-[var(--radius-button)] border border-hairline px-3 text-ink-800 transition-colors hover:bg-honey-100"
          onClick={() => {
            void probeSettings();
          }}
        >
          {STRINGS.controls.refresh}
        </button>
      </div>
      {settings.status === 'pending' ? <LoadingState /> : null}
      {settings.status === 'error' ? (
        settings.refusal ? (
          <div role="alert" className="mt-2 rounded-[var(--radius-sm)] border border-signal bg-panel p-2">
            <div className="type-mono-meta text-signal">{settings.refusal.code}</div>
            <p className="type-body-l mt-1 text-ink-900">{settings.refusal.message}</p>
            <p className="type-mono-meta mt-1 text-meta">{settings.refusal.how_to_fix}</p>
          </div>
        ) : (
          <ErrorState message={STRINGS.settings.keyUnknown} />
        )
      ) : null}
      <dl className="mt-2 grid grid-cols-[140px_1fr] gap-y-1">
        <dt className="type-mono-meta text-meta">{STRINGS.settings.providerLabel}</dt>
        <dd className="type-mono-code text-ink-900">{settings.provider ?? STRINGS.settings.keyUnknown}</dd>
        <dt className="type-mono-meta text-meta">{STRINGS.settings.modelLabel}</dt>
        <dd className="type-mono-code text-ink-900">{settings.model ?? STRINGS.settings.keyUnknown}</dd>
        <dt className="type-mono-meta text-meta">{STRINGS.settings.thinkingLabel}</dt>
        <dd className="type-mono-code text-ink-900">{settings.thinking ?? STRINGS.settings.notReported}</dd>
        <dt className="type-mono-meta text-meta">{STRINGS.settings.effortLabel}</dt>
        <dd className="type-mono-code text-ink-900">{settings.reasoningEffort ?? STRINGS.settings.notReported}</dd>
        <dt className="type-mono-meta text-meta">{STRINGS.settings.keyTitle}</dt>
        <dd className="type-mono-code text-ink-900">{keyLabel}</dd>
      </dl>
      <p className="type-mono-meta mt-2 text-meta">{STRINGS.settings.keyUnprobedHint}</p>
    </section>
  );

  const healthSection = (
    <section className="surface-panel mt-3 rounded-[var(--radius-md)] p-3" aria-label={STRINGS.settings.healthLabel}>
      <div className="flex items-center justify-between gap-2">
        <div className="type-label text-ink-500">{STRINGS.settings.healthLabel}</div>
        <button
          type="button"
          className="type-label h-8 rounded-[var(--radius-button)] border border-hairline px-3 text-ink-800 transition-colors hover:bg-honey-100"
          onClick={() => {
            void probeHealth();
          }}
        >
          {STRINGS.settings.refresh}
        </button>
      </div>
      <div className="type-mono-code mt-1 text-ink-900">
        {settings.health === 'unknown' ? STRINGS.settings.healthUnknown : settings.health}
      </div>
      {settings.checks.length > 0 ? (
        <ul className="type-mono-meta mt-1 list-inside list-disc text-meta">
          {settings.checks.map((check) => (
            <li key={check}>{check}</li>
          ))}
        </ul>
      ) : null}
    </section>
  );

  return (
    <div className="relative h-full overflow-y-auto p-4">
      <div className="hive-canvas-field opacity-30" aria-hidden="true" />
      <div className="relative mx-auto max-w-[920px]">
        <PanelTitle>{STRINGS.settings.title}</PanelTitle>
        <div
          role="tablist"
          aria-label="Settings sections"
          className="flex flex-wrap gap-1 border-b border-hairline pb-2"
          onKeyDown={onTabKeyDown}
        >
          {TABS.map((entry) => {
            const selected = tab === entry.id;
            return (
              <button
                key={entry.id}
                id={`settings-tab-${entry.id}`}
                type="button"
                role="tab"
                aria-selected={selected}
                aria-controls={`settings-panel-${entry.id}`}
                tabIndex={selected ? 0 : -1}
                onClick={() => setTab(entry.id)}
                className={
                  'type-label flex h-9 items-center gap-1.5 rounded-[var(--radius-button)] border px-3 transition-colors ' +
                  (selected
                    ? 'border-honey-500 bg-honey-100 text-ink-900'
                    : 'border-hairline bg-panel text-ink-500 hover:bg-panel-alt hover:text-ink-800')
                }
              >
                <SettingsIcon name={entry.icon} />
                {entry.label}
              </button>
            );
          })}
        </div>

        <div
          id={`settings-panel-${tab}`}
          role="tabpanel"
          aria-labelledby={`settings-tab-${tab}`}
          className="mt-3 min-h-0"
        >
          {tab === 'overview' ? (
            <>
              <p className="type-body-l mb-3 max-w-[640px] text-ink-500">
                One local hive: provider state is read from the server, diagnostics are derived, and
                every control names the value it changes.
              </p>
              {keySection}
              {healthSection}
              <ShortcutCard />
            </>
          ) : null}

          {tab === 'provider' ? (
            <>
              {keySection}
              <section className="surface-panel mt-3 rounded-[var(--radius-md)] p-3">
                <div className="type-label text-ink-500">Configured provider boundary</div>
                <p className="type-body-l mt-1 max-w-[640px] text-ink-500">
                  The server reports the configured provider; this panel never relabels it. An
                  external OpenAI-compatible endpoint is shown as the provider the server names
                  (for example a lab endpoint), never as DeepSeek, and never claims a balance or
                  capability the endpoint did not report.
                </p>
                <dl className="mt-2 grid grid-cols-[140px_1fr] gap-y-1">
                  <dt className="type-mono-meta text-meta">provider</dt>
                  <dd className="type-mono-code text-ink-900">
                    {settings.provider ?? STRINGS.settings.notReported}
                  </dd>
                  <dt className="type-mono-meta text-meta">model</dt>
                  <dd className="type-mono-code text-ink-900">{settings.model ?? STRINGS.settings.notReported}</dd>
                  <dt className="type-mono-meta text-meta">key</dt>
                  <dd className="type-mono-code text-ink-900">{keyLabel}</dd>
                </dl>
              </section>
            </>
          ) : null}

          {tab === 'appearance' ? (
            <>
              <section className="surface-panel rounded-[var(--radius-md)] p-3">
                <div className="type-label text-ink-500">Motion</div>
                <p className="type-body-l mt-1 max-w-[640px] text-ink-500">
                  Ambient motion governs the hive glow, sheen, and traveling bees. It is decorative
                  only; state and counts never depend on it.
                </p>
                <label className="mt-3 block">
                  <span className="type-mono-meta flex justify-between text-meta">
                    <span>ambient motion</span>
                    <span>{ambientLabel(ambient)} · {ambient}%</span>
                  </span>
                  <input
                    type="range"
                    min={0}
                    max={100}
                    step={5}
                    value={ambient}
                    onChange={(event) => setAmbient(Number(event.target.value))}
                    className="mt-1 w-full accent-honey-500"
                    aria-label="Ambient motion intensity"
                  />
                </label>
                <label className="type-mono-code mt-3 flex items-center gap-2 text-ink-900">
                  <input
                    type="checkbox"
                    checked={reducedOverride}
                    onChange={(event) => setReducedMotionOverride(event.target.checked)}
                  />
                  Reduced motion override
                </label>
              </section>
              <section className="surface-panel mt-3 rounded-[var(--radius-md)] p-3">
                <div className="type-label text-ink-500">Palette</div>
                <p className="type-body-l mt-1 text-ink-500">
                  The twenty printed HiveMind swatches, in token order.
                </p>
                <ul className="mt-2 grid grid-cols-4 gap-1 sm:grid-cols-5">
                  {SWATCHES.map((swatch) => (
                    <li key={swatch.id} className="min-w-0">
                      <span
                        className="block h-5 w-full rounded-[var(--radius-xs)] border border-hairline"
                        style={{ background: swatch.hex }}
                        aria-hidden="true"
                      />
                      <span className="type-mono-meta mt-0.5 block truncate text-[11px] text-meta">
                        {swatch.id}
                      </span>
                    </li>
                  ))}
                </ul>
              </section>
            </>
          ) : null}

          {tab === 'diagnostics' ? <DiagnosticsPanel /> : null}

          {tab === 'figcraft' ? (
            <div className="flex min-h-[70vh] flex-col">
              <DevGallery />
              <div className="mt-3 min-h-0 flex-1">
                <FigCraftPanel />
              </div>
            </div>
          ) : null}
        </div>
      </div>
    </div>
  );
}
