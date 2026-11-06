/** S21 Settings: provider/model/key_present (SETTINGS_STATUS), health probe, motion, diff gate, dev gallery. */
import { useEffect, useRef } from 'react';
import { useUiStore } from '../state/context';
import { STRINGS } from '../state/strings';
import { PanelTitle, LoadingState, ErrorState } from './StatePack';
import { DevGallery } from './DevGallery';

export function SettingsPanel() {
  const settings = useUiStore((state) => state.settings);
  const probeSettings = useUiStore((state) => state.probeSettings);
  const probeHealth = useUiStore((state) => state.probeHealth);
  const reducedOverride = useUiStore((state) => state.reducedMotionOverride);
  const setReducedMotionOverride = useUiStore((state) => state.setReducedMotionOverride);
  const openDiffModal = useUiStore((state) => state.openDiffModal);
  const probed = useRef(false);

  useEffect(() => {
    if (probed.current) {
      return;
    }
    probed.current = true;
    void probeSettings();
    void probeHealth();
  }, [probeHealth, probeSettings]);

  const keyLabel =
    settings.keyPresent === null
      ? STRINGS.settings.keyUnknown
      : settings.keyPresent
        ? STRINGS.settings.keyPresent
        : STRINGS.settings.keyMissing;

  return (
    <div className="mx-auto max-w-[760px] overflow-y-auto p-4">
      <PanelTitle>{STRINGS.settings.title}</PanelTitle>

      <section className="surface-panel mt-2 rounded-[var(--radius-md)] p-3" aria-label={STRINGS.settings.keyTitle}>
        <div className="flex items-center justify-between gap-2">
          <div className="type-label text-ink-500">{STRINGS.settings.keyTitle}</div>
          <button
            type="button"
            className="type-mono-meta rounded-[var(--radius-sm)] border border-hairline px-2 py-1 text-ink-800 hover:bg-honey-100"
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
            <div
              role="alert"
              className="mt-2 rounded-[var(--radius-sm)] border border-signal bg-panel p-2"
            >
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
          <dd className="type-mono-code text-ink-900">
            {settings.reasoningEffort ?? STRINGS.settings.notReported}
          </dd>
          <dt className="type-mono-meta text-meta">{STRINGS.settings.keyTitle}</dt>
          <dd className="type-mono-code text-ink-900">{keyLabel}</dd>
        </dl>
        <p className="type-mono-meta mt-2 text-meta">{STRINGS.settings.keyUnprobedHint}</p>
      </section>

      <section className="surface-panel mt-3 rounded-[var(--radius-md)] p-3" aria-label={STRINGS.settings.healthLabel}>
        <div className="flex items-center justify-between gap-2">
          <div className="type-label text-ink-500">{STRINGS.settings.healthLabel}</div>
          <button
            type="button"
            className="type-mono-meta rounded-[var(--radius-sm)] border border-hairline px-2 py-1 text-ink-800 hover:bg-honey-100"
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

      <section className="surface-panel mt-3 rounded-[var(--radius-md)] p-3">
        <label className="type-mono-code flex items-center gap-2 text-ink-900">
          <input
            type="checkbox"
            checked={reducedOverride}
            onChange={(event) => setReducedMotionOverride(event.target.checked)}
          />
          {STRINGS.states.replay} · reduced motion override
        </label>
        <button
          type="button"
          className="type-label mt-3 rounded-[var(--radius-button)] border border-hairline px-3 py-1.5 text-ink-900 hover:bg-honey-100"
          onClick={openDiffModal}
        >
          {STRINGS.diff.title}
        </button>
      </section>

      <DevGallery />
    </div>
  );
}
