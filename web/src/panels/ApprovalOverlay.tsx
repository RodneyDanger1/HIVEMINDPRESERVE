/** Approval overlay (patched contract): APPROVAL_RESOLVE with server nonce; no nonce => named gap, disabled. */
import { useUiStore } from '../state/context';
import { STRINGS } from '../state/strings';

export function ApprovalOverlay() {
  const approvals = useUiStore((state) => state.approvals);
  const resolveApproval = useUiStore((state) => state.resolveApproval);

  if (approvals.length === 0) {
    return null;
  }
  return (
    <>
      {approvals.map((approval) => (
        <div key={approval.approvalId} className="fixed inset-0 z-40 flex items-center justify-center bg-ink-900/20 p-4">
          <section
            role="dialog"
            aria-modal="true"
            aria-label={STRINGS.approval.title}
            className="surface-panel w-[min(520px,94%)] rounded-[var(--radius-card)] p-4"
            style={{ boxShadow: 'var(--shadow-e3)' }}
          >
            <h2 className="type-heading-h3 text-ink-900">{STRINGS.approval.title}</h2>
            <dl className="mt-2 grid grid-cols-[100px_1fr] gap-y-1">
              <dt className="type-mono-meta text-meta">{STRINGS.approval.toolLabel}</dt>
              <dd className="type-mono-code text-ink-900">{approval.tool}</dd>
              <dt className="type-mono-meta text-meta">approval_id</dt>
              <dd className="type-mono-code text-ink-900">{approval.approvalId.slice(0, 12)}…</dd>
            </dl>
            {approval.nonce === null ? (
              <p className="type-mono-meta mt-2 rounded-[var(--radius-sm)] border border-amber bg-panel-alt p-2 text-ink-800">
                {STRINGS.approval.noCommand}
              </p>
            ) : null}
            <div className="mt-3 flex justify-end gap-2">
              <button
                type="button"
                className="type-label rounded-[var(--radius-button)] border border-signal px-3 py-1.5 text-signal disabled:opacity-50"
                disabled={approval.nonce === null}
                onClick={() => {
                  void resolveApproval(approval.approvalId, 'deny', 'operator denied');
                }}
              >
                {STRINGS.controls.deny}
              </button>
              <button
                type="button"
                className="type-label rounded-[var(--radius-button)] bg-honey-300 px-3 py-1.5 text-ink-900 disabled:opacity-50"
                disabled={approval.nonce === null}
                onClick={() => {
                  void resolveApproval(approval.approvalId, 'approve');
                }}
              >
                {STRINGS.controls.approve}
              </button>
            </div>
          </section>
        </div>
      ))}
    </>
  );
}
