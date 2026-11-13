/** Small accessible hover/focus explanation for terse operator labels. */
export function InfoTip({ label, children }: { label: string; children: string }) {
  return (
    <span className="hm-info" tabIndex={0} role="note" aria-label={label}>
      <span aria-hidden="true" className="hm-info-dot">i</span>
      <span role="tooltip" className="hm-info-bubble">{children}</span>
    </span>
  );
}
