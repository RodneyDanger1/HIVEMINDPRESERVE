/** S25 code-diff modal: decision-gated (UI-COMMANDS-v1 §2 says do not implement DIFF_*). */
import { useUiStore } from '../state/context';
import { STRINGS } from '../state/strings';

export function DiffDecisionModal() {
  const open = useUiStore((state) => state.diffOpen);
  const close = useUiStore((state) => state.closeDiffModal);
  if (!open) {
    return null;
  }
  return (
    <div className="fixed inset-0 z-40 flex items-center justify-center bg-ink-900/20 p-4">
      <section
        role="dialog"
        aria-modal="true"
        aria-label={STRINGS.diff.title}
        className="surface-panel w-[min(560px,94%)] rounded-[var(--radius-card)] p-4"
        style={{ boxShadow: 'var(--shadow-e3)' }}
      >
        <h2 className="type-heading-h3 text-ink-900">{STRINGS.diff.title}</h2>
        <p className="type-body-l mt-2 text-ink-900">{STRINGS.diff.decisionRequired}</p>
        <div className="mt-3 flex justify-end">
          <button
            type="button"
            className="type-label rounded-[var(--radius-button)] border border-hairline px-3 py-1.5 text-ink-900 hover:bg-honey-100"
            onClick={close}
          >
            {STRINGS.controls.close}
          </button>
        </div>
      </section>
    </div>
  );
}
