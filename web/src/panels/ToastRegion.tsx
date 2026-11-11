/** S14 refusal/error toasts: "REFUSED BY NAME · NOT SILENT" (name + how_to_fix). */
import { AnimatePresence, motion } from 'motion/react';
import { useUiStore } from '../state/context';
import { STRINGS } from '../state/strings';
import { useAppReducedMotion } from '../state/useAppReducedMotion';

/**
 * Approval dialogs sit at z-40 and are blocking decisions. Refusal toasts are
 * z-50 by default, so a long toast stack could cover the dialog header; while
 * any approval is pending the stack drops below the modal but stays above the
 * canvas. Nothing else changes (same position, copy and dismissal).
 */
export function toastStackZClass(approvalCount: number): 'z-30' | 'z-50' {
  return approvalCount > 0 ? 'z-30' : 'z-50';
}

export function ToastRegion() {
  const refusals = useUiStore((state) => state.refusals);
  const approvals = useUiStore((state) => state.approvals);
  const dismiss = useUiStore((state) => state.dismissRefusal);
  const reduced = useAppReducedMotion();

  return (
    <div
      aria-live="assertive"
      aria-label={STRINGS.toast.regionAria}
      data-approval-priority={approvals.length > 0 ? 'modal' : 'normal'}
      className={`pointer-events-none fixed right-4 top-16 flex w-[min(440px,92vw)] flex-col gap-2 ${toastStackZClass(approvals.length)}`}
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
