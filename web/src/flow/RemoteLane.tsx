/**
 * Detached remote lane overlay. Remotes are not task nodes and have no graph
 * edges; this overlay is the only place they appear "on" the hive canvas.
 */
import { useEffect, useState } from 'react';
import { apiBaseUrl } from '../api/client';
import { useUiStore } from '../state/context';
import { STRINGS } from '../state/strings';

interface RemoteChip {
  readonly id: string;
  readonly name: string;
  readonly state: string;
  readonly deviceCount: number;
}

interface DaemonChip {
  readonly id: string;
  readonly name: string;
  readonly kind: string;
  readonly state: string;
}

export function RemoteLane() {
  const setNav = useUiStore((state) => state.setNav);
  const [remotes, setRemotes] = useState<readonly RemoteChip[]>([]);
  const [daemons, setDaemons] = useState<readonly DaemonChip[]>([]);
  const [unavailable, setUnavailable] = useState(false);

  useEffect(() => {
    let cancelled = false;
    const load = (): void => {
      fetch(`${apiBaseUrl()}/api/remotes`, { headers: { Accept: 'application/json' } })
        .then(async (response) => {
          if (!response.ok) throw new Error(`remote lane HTTP ${response.status}`);
          const body = (await response.json()) as { remotes?: unknown };
          if (!Array.isArray(body.remotes)) throw new Error('remote lane missing rows');
          const decoded: RemoteChip[] = [];
          for (const raw of body.remotes) {
            if (typeof raw !== 'object' || raw === null) continue;
            const item = raw as Record<string, unknown>;
            const remote =
              typeof item.remote === 'object' && item.remote !== null
                ? (item.remote as Record<string, unknown>)
                : null;
            if (
              remote === null ||
              typeof remote.remote_id !== 'string' ||
              typeof remote.display_name !== 'string' ||
              typeof remote.state !== 'string'
            ) {
              continue;
            }
            decoded.push({
              id: remote.remote_id,
              name: remote.display_name,
              state: remote.state,
              deviceCount: typeof item.device_count === 'number' ? item.device_count : 0,
            });
          }
          if (!cancelled) {
            setRemotes(decoded);
            setUnavailable(false);
          }
        })
        .catch(() => {
          if (!cancelled) {
            setRemotes([]);
            setUnavailable(true);
          }
        });
      fetch(`${apiBaseUrl()}/api/daemons`, { headers: { Accept: 'application/json' } })
        .then(async (response) => {
          if (!response.ok) throw new Error(`daemon lane HTTP ${response.status}`);
          const body = (await response.json()) as { daemons?: unknown };
          if (!Array.isArray(body.daemons)) throw new Error('daemon lane missing rows');
          const decoded: DaemonChip[] = [];
          for (const raw of body.daemons) {
            if (typeof raw !== 'object' || raw === null) continue;
            const item = raw as Record<string, unknown>;
            if (
              typeof item.daemon_id !== 'string' ||
              typeof item.display_name !== 'string' ||
              typeof item.kind !== 'string' ||
              typeof item.state !== 'string'
            ) {
              continue;
            }
            decoded.push({
              id: item.daemon_id,
              name: item.display_name,
              kind: item.kind,
              state: item.state,
            });
          }
          if (!cancelled) setDaemons(decoded);
        })
        .catch(() => {
          if (!cancelled) setDaemons([]);
        });
    };
    load();
    const timer = window.setInterval(load, 15_000);
    return () => {
      cancelled = true;
      window.clearInterval(timer);
    };
  }, []);

  return (
    <section
      aria-label="Detached remote lane"
      className="surface-panel pointer-events-auto absolute bottom-24 right-4 z-20 w-[220px] rounded-[var(--radius-card)] p-2"
    >
      <button
        type="button"
        className="flex w-full items-center justify-between gap-2 text-left"
        onClick={() => setNav('remote')}
      >
        <span className="type-mono-meta text-[11px] tracking-[0.12em] text-meta">
          REMOTE {remotes.length} · DAEMON {daemons.length}
        </span>
        <span className="type-mono-meta text-[11px] text-ink-500">OPEN</span>
      </button>
      {unavailable ? (
        <p className="type-mono-meta mt-1 text-[11px] text-signal">Remote lane unavailable.</p>
      ) : remotes.length === 0 ? (
        <p className="type-mono-meta mt-1 text-[11px] text-meta">{STRINGS.panels.remoteTitle}: none enrolled</p>
      ) : (
        <ul className="mt-1 space-y-1">
          {remotes.slice(0, 4).map((remote) => (
            <li key={remote.id} className="flex items-center justify-between gap-1">
              <span className="flex min-w-0 items-center gap-1">
                <span
                  aria-hidden="true"
                  className={
                    'h-1.5 w-1.5 shrink-0 rounded-full ' +
                    (remote.state === 'online'
                      ? 'bg-verify'
                      : remote.state === 'offline'
                        ? 'bg-hairline'
                        : 'bg-honey-500')
                  }
                />
                <span className="type-mono-meta truncate text-[11px] text-ink-800">{remote.name}</span>
              </span>
              <span className="type-mono-meta text-[11px] text-meta">{remote.deviceCount}d</span>
            </li>
          ))}
        </ul>
      )}
      {daemons.length > 0 ? (
        <ul className="mt-1 space-y-1 border-t border-hairline pt-1">
          {daemons.slice(0, 3).map((daemon) => (
            <li key={daemon.id} className="flex items-center justify-between gap-1">
              <span className="type-mono-meta truncate text-[11px] text-ink-800">
                {daemon.kind} · {daemon.name}
              </span>
              <span className="type-mono-meta text-[11px] text-meta">{daemon.state}</span>
            </li>
          ))}
        </ul>
      ) : null}
    </section>
  );
}
