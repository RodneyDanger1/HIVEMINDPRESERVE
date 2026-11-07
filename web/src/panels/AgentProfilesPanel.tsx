/**
 * Read-only boot-loaded agent profiles. A profile is a persona/template binding;
 * the editor/import surface is a later wave, but the shipped registry must be
 * visible rather than hidden behind task cards.
 */
import { useEffect, useState } from 'react';
import { EmptyState } from './StatePack';

interface AgentProfileRow {
  readonly id: string;
  readonly display_name: string;
  readonly tier: string;
  readonly template_id: string;
  readonly allowed_tool_ids: readonly string[];
  readonly output_schema_id: string | null;
  readonly refusal_policy: string;
}

type Status = 'loading' | 'ok' | 'error';

export function AgentProfilesPanel() {
  const [status, setStatus] = useState<Status>('loading');
  const [profiles, setProfiles] = useState<readonly AgentProfileRow[]>([]);

  useEffect(() => {
    let cancelled = false;
    fetch('/api/agents/profiles', { headers: { Accept: 'application/json' } })
      .then(async (response) => {
        if (!response.ok) {
          throw new Error(`profiles HTTP ${response.status}`);
        }
        const body = (await response.json()) as { profiles?: unknown };
        if (!Array.isArray(body.profiles)) {
          throw new Error('profiles payload has no profiles array');
        }
        const decoded: AgentProfileRow[] = [];
        for (const entry of body.profiles) {
          if (typeof entry !== 'object' || entry === null) {
            continue;
          }
          const item = entry as Record<string, unknown>;
          if (typeof item.id !== 'string' || typeof item.display_name !== 'string') {
            continue;
          }
          decoded.push({
            id: item.id,
            display_name: item.display_name,
            tier: typeof item.tier === 'string' ? item.tier : 'unknown',
            template_id: typeof item.template_id === 'string' ? item.template_id : '',
            allowed_tool_ids: Array.isArray(item.allowed_tool_ids)
              ? item.allowed_tool_ids.filter((tool): tool is string => typeof tool === 'string')
              : [],
            output_schema_id: typeof item.output_schema_id === 'string' ? item.output_schema_id : null,
            refusal_policy: typeof item.refusal_policy === 'string' ? item.refusal_policy : '',
          });
        }
        if (!cancelled) {
          setProfiles(decoded);
          setStatus('ok');
        }
      })
      .catch(() => {
        if (!cancelled) {
          setProfiles([]);
          setStatus('error');
        }
      });
    return () => {
      cancelled = true;
    };
  }, []);

  return (
    <section aria-label="Agent profiles" className="mt-4 border-t border-hairline pt-4">
      <h3 className="type-label text-ink-900">Agent profiles</h3>
      {status === 'loading' ? (
        <p className="type-mono-meta mt-1 text-meta">Loading boot profiles…</p>
      ) : null}
      {status === 'error' ? (
        <p className="type-mono-meta mt-1 text-signal">Profiles unavailable; no profiles invented.</p>
      ) : null}
      {status === 'ok' && profiles.length === 0 ? (
        <EmptyState message="No boot-loaded profiles." />
      ) : null}
      <ul className="mt-2 grid grid-cols-1 gap-2 sm:grid-cols-2">
        {profiles.map((profile) => (
          <li key={profile.id} className="surface-panel rounded-[var(--radius-sm)] p-2">
            <div className="flex items-center justify-between gap-2">
              <span className="type-mono-code text-ink-900">{profile.display_name}</span>
              <span className="type-mono-meta text-meta">{profile.tier}</span>
            </div>
            <div className="type-mono-meta mt-1 text-meta">
              {profile.template_id} · {profile.allowed_tool_ids.length} tools · {profile.refusal_policy}
            </div>
          </li>
        ))}
      </ul>
    </section>
  );
}
