/** Models (read-only): only values the server reports; contract reference labelled READ. */
import { useUiStore } from '../state/context';
import { STRINGS } from '../state/strings';
import { PanelTitle, EmptyState } from './StatePack';

export function ModelsPanel() {
  const settings = useUiStore((state) => state.settings);
  return (
    <div className="mx-auto max-w-[760px] p-4">
      <PanelTitle>{STRINGS.panels.modelsTitle}</PanelTitle>
      {settings.provider === null && settings.model === null ? (
        <EmptyState message={STRINGS.models.none} />
      ) : (
        <dl className="surface-panel grid grid-cols-[140px_1fr] gap-y-1 rounded-[var(--radius-md)] p-3">
          <dt className="type-mono-meta text-meta">{STRINGS.models.providerLabel}</dt>
          <dd className="type-mono-code text-ink-900">{settings.provider ?? STRINGS.models.notReported}</dd>
          <dt className="type-mono-meta text-meta">{STRINGS.models.modelLabel}</dt>
          <dd className="type-mono-code text-ink-900">{settings.model ?? STRINGS.models.notReported}</dd>
        </dl>
      )}
      <p className="type-mono-meta mt-3 text-meta">{STRINGS.models.contractNote}</p>
    </div>
  );
}
