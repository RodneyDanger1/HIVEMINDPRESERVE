/**
 * Durable role memory: inspect and clear. The server is the source; a clear is
 * a real DELETE and the panel refreshes from the result, never locally guessed.
 */
import { useCallback, useEffect, useState } from 'react';
import { apiBaseUrl } from '../api/client';
import { EmptyState } from './StatePack';

interface MemoryRow {
  readonly key: string;
  readonly value: string;
}
interface MemoryProfile {
  readonly id: string;
  readonly display_name: string;
  readonly memories: readonly MemoryRow[];
}
type Status = 'loading' | 'ok' | 'error';

export function AgentMemoryPanel() {
  const [status, setStatus] = useState<Status>('loading');
  const [profiles, setProfiles] = useState<readonly MemoryProfile[]>([]);

  const refresh = useCallback(() => {
    setStatus('loading');
    fetch(`${apiBaseUrl()}/api/agents/memory`, { headers: { Accept: 'application/json' } })
      .then(async (response) => {
        if (!response.ok) {
          throw new Error(`memory HTTP ${response.status}`);
        }
        const body = (await response.json()) as { profiles?: unknown };
        if (!Array.isArray(body.profiles)) {
          throw new Error('memory payload has no profiles array');
        }
        const decoded: MemoryProfile[] = [];
        for (const entry of body.profiles) {
          if (typeof entry !== 'object' || entry === null) {
            continue;
          }
          const item = entry as Record<string, unknown>;
          if (typeof item.id !== 'string') {
            continue;
          }
          const memories: MemoryRow[] = [];
          if (Array.isArray(item.memories)) {
            for (const memory of item.memories) {
              if (typeof memory !== 'object' || memory === null) {
                continue;
              }
              const row = memory as Record<string, unknown>;
              if (typeof row.key === 'string' && typeof row.value === 'string') {
                memories.push({ key: row.key, value: row.value });
              }
            }
          }
          decoded.push({
            id: item.id,
            display_name: typeof item.display_name === 'string' ? item.display_name : item.id,
            memories,
          });
        }
        setProfiles(decoded);
        setStatus('ok');
      })
      .catch(() => {
        setProfiles([]);
        setStatus('error');
      });
  }, []);

  useEffect(() => {
    refresh();
  }, [refresh]);

  const clear = (agentKey: string): void => {
    fetch(`${apiBaseUrl()}/api/agents/memory/${encodeURIComponent(agentKey)}`, { method: 'DELETE' })
      .then((response) => {
        if (!response.ok) {
          throw new Error(`clear HTTP ${response.status}`);
        }
        refresh();
      })
      .catch(() => setStatus('error'));
  };

  return (
    <section aria-label="Agent memory" className="surface-panel mt-3 rounded-[14px] p-3">
      <h3 className="type-label text-ink-900">Agent memory</h3>
      {status === 'loading' ? (
        <p className="type-mono-meta mt-1 text-meta">Loading durable memory…</p>
      ) : null}
      {status === 'error' ? (
        <p className="type-mono-meta mt-1 text-signal">Memory unavailable; no rows invented.</p>
      ) : null}
      {status === 'ok' && profiles.every((profile) => profile.memories.length === 0) ? (
        <EmptyState message="No durable memories for the boot profiles." />
      ) : null}
      <ul className="mt-2 space-y-2">
        {profiles
          .filter((profile) => profile.memories.length > 0)
          .map((profile) => (
            <li key={profile.id} className="surface-panel rounded-[var(--radius-sm)] p-2">
              <div className="flex items-center justify-between gap-2">
                <span className="type-mono-code text-ink-900">{profile.display_name}</span>
                <button
                  type="button"
                  className="type-mono-meta rounded-[var(--radius-sm)] border border-hairline px-2 py-1 text-ink-800 hover:bg-honey-100"
                  onClick={() => clear(profile.id)}
                >
                  Clear
                </button>
              </div>
              <ul className="mt-1 space-y-1">
                {profile.memories.map((memory) => (
                  <li key={memory.key} className="type-mono-meta text-meta">
                    {memory.key}: {memory.value}
                  </li>
                ))}
              </ul>
            </li>
          ))}
      </ul>
    </section>
  );
}
