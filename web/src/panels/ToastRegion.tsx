/** S14 refusal/error toasts: "REFUSED BY NAME · NOT SILENT" (name + how_to_fix). */
import { AnimatePresence, motion } from 'motion/react';
import { useUiStore } from '../state/context';
import { STRINGS } from '../state/strings';
import { useAppReducedMotion } from '../state/useAppReducedMotion';

export function ToastRegion() {
  const refusals = useUiStore((state) => state.refusals);
  const dismiss = useUiStore((state) => state.dismissRefusal);
  const reduced = useAppReducedMotion();

  return (
    <div
      aria-live="assertive"
      aria-label={STRINGS.toast.regionAria}
      className="pointer-events-none fixed right-4 top-4 z-50 flex w-[min(440px,92vw)] flex-col gap-2"
    >
      <AnimatePresence initial={false}>
        {refusals.map((entry) => (
          <motion.div
            key={entry.id}
            initial={{ opacity: 0, y: reduced ? 0 : -6 }}
            animate={{ opacity: 1, y: 0 }}
            exit={{ opacity: 0, y: reduced ? 0 : -6 }}
            transition={{ duration: reduced ? 0 : 0.16 }}
            className="pointer-events-auto rounded-[var(--radius-md)] border border-signal bg-panel p-3"
            style={{ boxShadow: 'var(--shadow-e2)' }}
            role="alert"
          >
            <div className="type-mono-meta text-signal">
              {STRINGS.toast.prefix} · {STRINGS.toast.suffix}
            </div>
            <div className="type-mono-code mt-1 text-ink-900">{entry.refusal.code}</div>
            <p className="type-body-l mt-1 text-ink-900">{entry.refusal.message}</p>
            <p className="type-mono-meta mt-1 text-meta">{entry.refusal.how_to_fix}</p>
            <button
              type="button"
              className="type-mono-meta mt-2 rounded-[var(--radius-sm)] border border-hairline px-2 py-1 text-ink-800 hover:bg-honey-100"
              onClick={() => dismiss(entry.id)}
              aria-label={STRINGS.toast.dismiss}
            >
              {STRINGS.controls.dismiss}
            </button>
          </motion.div>
        ))}
      </AnimatePresence>
    </div>
  );
}
