/** Shared surface-state primitives (S26 state pack): loading/empty/error/offline. */
import type { ReactNode } from 'react';
import { STRINGS } from '../state/strings';

export function PanelTitle({ children }: { children: ReactNode }) {
  return <h2 className="type-heading-h3 mb-2 text-ink-900">{children}</h2>;
}

export function EmptyState({ message }: { message: string }) {
  return (
    <div className="rounded-[var(--radius-md)] border border-dashed border-hairline bg-panel-alt px-3 py-4">
      <p className="type-body-l text-ink-500">{message}</p>
    </div>
  );
}

export function LoadingState({ label = STRINGS.states.loading }: { label?: string }) {
  return (
    <div role="status" className="type-mono-meta rounded-[var(--radius-md)] bg-panel-alt px-3 py-2 text-ink-500">
      {label}
    </div>
  );
}

export function ErrorState({ message }: { message: string }) {
  return (
    <div role="alert" className="rounded-[var(--radius-md)] border border-signal bg-panel px-3 py-2">
      <span className="type-mono-meta text-signal">{STRINGS.states.error}</span>
      <p className="type-body-l mt-1 text-ink-900">{message}</p>
    </div>
  );
}

export function ReplayBanner({ revision }: { revision: number }) {
  return (
    <div className="type-mono-meta rounded-[var(--radius-sm)] border border-hairline bg-recessed px-3 py-1 text-ink-800">
      {STRINGS.states.replay} · {STRINGS.scrubber.label} {revision}
    </div>
  );
}

export function DerivedBadge({ source }: { source: 'server' | 'derived' | 'none' }) {
  return (
    <span className="type-mono-meta text-meta" title={STRINGS.hud.deltaLabel}>
      {source === 'server'
        ? STRINGS.hud.sourceServer
        : source === 'derived'
          ? STRINGS.hud.sourceDerived
          : STRINGS.hud.sourceNone}
    </span>
  );
}
