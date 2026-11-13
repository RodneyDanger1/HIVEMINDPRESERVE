/**
 * Snapshot truth strip.
 *
 * `snapshot.status`/`incomplete`/`attempts` are written by the store's
 * server-authoritative rehydration, but nothing rendered them: a degraded graph
 * looked identical to a confirmed one. This strip is the single consumer and
 * offers the existing `refreshRunSnapshot` action as "Re-sync". It renders
 * nothing while the snapshot is ready/idle, so it never fabricates a state.
 */
import { isSnapshotDegraded, type SnapshotState } from '../state/store';
import { STRINGS } from '../state/strings';

export type SnapshotBannerTone = 'amber' | 'signal';

export function snapshotBannerTone(status: SnapshotState['status']): SnapshotBannerTone {
  return status === 'failed' ? 'signal' : 'amber';
}

/** Honest title per degraded status; `ready`/`idle` are not banner states. */
export function snapshotBannerTitle(status: SnapshotState['status']): string | null {
  if (status === 'loading') {
    return STRINGS.snapshot.confirming;
  }
  if (status === 'failed') {
    return STRINGS.snapshot.failed;
  }
  if (status === 'degraded') {
    return STRINGS.snapshot.degraded;
  }
  return null;
}

export function snapshotIncompleteText(incomplete: SnapshotState['incomplete']): string {
  return incomplete.join(', ');
}

export function SnapshotBanner({
  snapshot,
  onResync,
}: {
  readonly snapshot: SnapshotState;
  readonly onResync: () => void;
}) {
  if (!isSnapshotDegraded(snapshot)) {
    return null;
  }
  // A fresh workspace has no committed revision yet. That is the normal empty
  // state, not an incident; showing a technical "unconfirmed" strip here makes
  // a new user think something is broken.
  if (
    snapshot.status !== 'failed' &&
    snapshot.incomplete.some((item) => item.includes('NO-COMMITTED-REVISION'))
  ) {
    return null;
  }
  const loading = snapshot.status === 'loading';
  const tone = snapshotBannerTone(snapshot.status);
  const title = snapshotBannerTitle(snapshot.status) ?? STRINGS.snapshot.degraded;
  const updatedAt =
    snapshot.updatedAt !== null && Number.isFinite(snapshot.updatedAt)
      ? new Date(snapshot.updatedAt).toISOString()
      : null;
  return (
    <div
      role="status"
      aria-live="polite"
      aria-label={STRINGS.snapshot.aria}
      data-testid="snapshot-banner"
      className={
        'type-mono-meta flex flex-wrap items-center gap-x-3 gap-y-1 border-b px-4 py-1 ' +
        (tone === 'signal'
          ? 'border-signal bg-panel text-signal'
          : 'border-amber bg-panel-alt text-ink-800')
      }
    >
      <span className="font-semibold">{title}</span>
      {snapshot.incomplete.length > 0 ? (
        <span>
          {STRINGS.snapshot.incomplete}: {snapshotIncompleteText(snapshot.incomplete)}
        </span>
      ) : null}
      <span>
        {STRINGS.snapshot.attempts} {snapshot.attempts}
      </span>
      {updatedAt === null ? null : (
        <time dateTime={updatedAt}>
          {STRINGS.snapshot.updated} {updatedAt}
        </time>
      )}
      <button
        type="button"
        disabled={loading}
        onClick={onResync}
        className="rounded-[var(--radius-sm)] border border-hairline px-2 py-0.5 text-ink-800 hover:bg-honey-100 disabled:cursor-not-allowed disabled:opacity-50"
      >
        {loading ? STRINGS.snapshot.resyncing : STRINGS.snapshot.resync}
      </button>
    </div>
  );
}
