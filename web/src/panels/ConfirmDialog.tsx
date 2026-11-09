/**
 * HiveMind confirmation surface. Replaces browser confirm() for destructive
 * operator actions; the surrounding focus and Escape behavior stay local and
 * no business decision is made here.
 */
import { useEffect, useRef } from 'react';

export interface ConfirmDialogProps {
  readonly open: boolean;
  readonly title: string;
  readonly body: string;
  readonly confirmLabel: string;
  readonly danger?: boolean;
  readonly onConfirm: () => void;
  readonly onCancel: () => void;
}

export function ConfirmDialog({
  open,
  title,
  body,
  confirmLabel,
  danger = false,
  onConfirm,
  onCancel,
}: ConfirmDialogProps) {
  const confirmRef = useRef<HTMLButtonElement | null>(null);
  const previousFocus = useRef<Element | null>(null);

  useEffect(() => {
    if (!open) {
      return undefined;
    }
    previousFocus.current = document.activeElement;
    confirmRef.current?.focus();
    const onKeyDown = (event: KeyboardEvent): void => {
      if (event.key === 'Escape') {
        event.preventDefault();
        onCancel();
      }
    };
    window.addEventListener('keydown', onKeyDown);
    return () => {
      window.removeEventListener('keydown', onKeyDown);
      const previous = previousFocus.current;
      if (previous instanceof HTMLElement) {
        previous.focus();
      }
    };
  }, [open, onCancel]);

  if (!open) {
    return null;
  }

  return (
    <div
      className="fixed inset-0 z-50 flex items-center justify-center bg-ink-900/20 p-4"
      onMouseDown={(event) => {
        if (event.target === event.currentTarget) {
          onCancel();
        }
      }}
    >
      <div
        role="dialog"
        aria-modal="true"
        aria-labelledby="confirm-dialog-title"
        aria-describedby="confirm-dialog-body"
        className="chat-message surface-panel w-[min(420px,92vw)] rounded-[var(--radius-lg)] p-4 shadow-[var(--shadow-e3)]"
      >
        <div className="type-label text-ink-900" id="confirm-dialog-title">
          {title}
        </div>
        <p className="type-body-l mt-2 text-ink-700" id="confirm-dialog-body">
          {body}
        </p>
        <div className="mt-4 flex justify-end gap-2">
          <button
            type="button"
            className="type-label h-9 rounded-[var(--radius-button)] border border-hairline px-3 text-ink-800 transition-colors hover:bg-honey-100"
            onClick={onCancel}
          >
            Cancel
          </button>
          <button
            ref={confirmRef}
            type="button"
            className={
              'type-label h-9 rounded-[var(--radius-button)] border px-3 font-semibold transition-colors ' +
              (danger
                ? 'border-signal bg-recessed text-signal hover:bg-panel-alt'
                : 'border-honey-500 bg-honey-300 text-ink-900 hover:bg-honey-400')
            }
            onClick={onConfirm}
          >
            {confirmLabel}
          </button>
        </div>
      </div>
    </div>
  );
}
