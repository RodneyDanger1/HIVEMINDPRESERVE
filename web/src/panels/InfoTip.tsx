/** Small accessible hover/focus explanation for terse operator labels. */
import { useId } from 'react';

export function InfoTip({ label, children }: { label: string; children: string }) {
  const tooltipId = useId();
  return (
    <span className="hm-info" tabIndex={0} role="note" aria-label={label} aria-describedby={tooltipId}>
      <span aria-hidden="true" className="hm-info-dot">
        <svg width="16" height="16" viewBox="0 0 16 16" focusable="false">
          <path
            d="M8 1.1 14.1 4.6v6.8L8 14.9 1.9 11.4V4.6Z"
            fill="var(--color-honey-100)"
            stroke="var(--color-honey-500)"
            strokeWidth="1.1"
            strokeLinejoin="round"
          />
          <path
            d="M8 6.4v4.1"
            stroke="var(--color-ink-700)"
            strokeWidth="1.4"
            strokeLinecap="round"
          />
          <circle cx="8" cy="4.4" r=".9" fill="var(--color-ink-700)" />
        </svg>
      </span>
      <span id={tooltipId} role="tooltip" className="hm-info-bubble">{children}</span>
    </span>
  );
}
