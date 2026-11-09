/**
 * Agent profile registry: the boot profiles plus validated imported profiles.
 * Import is a real POST (server validates template/tools and persists); delete
 * removes the persisted row and restores any embedded profile it overrode.
 */
import { useCallback, useEffect, useState } from 'react';
import { apiBaseUrl } from '../api/client';
import { useUiStoreApi } from '../state/context';
import { EmptyState } from './StatePack';

interface AgentProfileRow {
  readonly id: string;
  readonly display_name: string;
  readonly tier: string;
  readonly template_id: string;
  readonly allowed_tool_ids: readonly string[];
  readonly output_schema_id: string | null;
  readonly refusal_policy: string;
  readonly max_children: number | null;
  readonly max_depth: number | null;
}

type Status = 'loading' | 'ok' | 'error';
const BUILTIN_IDS = new Set([
  'planner',
  'content-writer',
  'data-analyst',
  'code-reviewer',
  'chatbot',
  'superagent',
  'subagent',
]);
const EXAMPLE_PROFILE = `{
  "id": "custom-writer",
  "display_name": "Custom Writer",
  "tier": "worker",
  "template_id": "content-writer",
  "allowed_tool_ids": ["file_read", "ask_parent", "finish"],
  "output_schema_id": null,
  "refusal_policy": "strict"
}`;

export function AgentProfilesPanel() {
  const storeApi = useUiStoreApi();
  const [status, setStatus] = useState<Status>('loading');
  const [profiles, setProfiles] = useState<readonly AgentProfileRow[]>([]);
  const [importText, setImportText] = useState(EXAMPLE_PROFILE);
  const [importStatus, setImportStatus] = useState<string | null>(null);

  const loadProfiles = useCallback(async () => {
    setStatus('loading');
    try {
      const response = await fetch(`${apiBaseUrl()}/api/agents/profiles`, {
        headers: { Accept: 'application/json' },
      });
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
          max_children: typeof item.max_children === 'number' ? item.max_children : null,
          max_depth: typeof item.max_depth === 'number' ? item.max_depth : null,
        });
      }
      setProfiles(decoded);
      setStatus('ok');
    } catch {
      setProfiles([]);
      setStatus('error');
    }
  }, []);

  useEffect(() => {
    void loadProfiles();
  }, [loadProfiles]);

  const importProfile = (): void => {
    setImportStatus(null);
    let parsed: unknown;
    try {
      parsed = JSON.parse(importText);
    } catch {
      setImportStatus('Import refused: the text is not valid JSON.');
      return;
    }
    const token = storeApi.getState().session?.token;
    fetch(`${apiBaseUrl()}/api/agents/profiles/import`, {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
        Accept: 'application/json',
        ...(token ? { 'x-hivemind-token': token } : {}),
      },
      body: JSON.stringify({ profile: parsed }),
    })
      .then(async (response) => {
        const body = (await response.json().catch(() => ({}))) as { code?: string };
        if (!response.ok) {
          throw new Error(body.code ?? `import HTTP ${response.status}`);
        }
        setImportStatus('Imported.');
        await loadProfiles();
      })
      .catch((error: unknown) => {
        setImportStatus(`Import refused: ${error instanceof Error ? error.message : 'unknown error'}`);
      });
  };

  const removeProfile = (profileId: string): void => {
    const token = storeApi.getState().session?.token;
    fetch(`${apiBaseUrl()}/api/agents/profiles/${encodeURIComponent(profileId)}`, {
      method: 'DELETE',
      headers: token ? { 'x-hivemind-token': token } : {},
    })
      .then(async (response) => {
        if (!response.ok) {
          const body = (await response.json().catch(() => ({}))) as { code?: string };
          throw new Error(body.code ?? `delete HTTP ${response.status}`);
        }
        await loadProfiles();
      })
      .catch((error: unknown) => {
        setImportStatus(`Delete refused: ${error instanceof Error ? error.message : 'unknown error'}`);
      });
  };

  return (
    <section aria-label="Agent profiles" className="surface-panel mt-3 rounded-[var(--radius-card)] p-3">
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
            {profile.max_children === null && profile.max_depth === null ? null : (
              <div className="type-mono-meta mt-1 text-meta">
                limits: {profile.max_children ?? '—'} children · {profile.max_depth ?? '—'} depth
              </div>
            )}
            {BUILTIN_IDS.has(profile.id) ? null : (
              <button
                type="button"
                className="type-mono-meta mt-2 rounded-[var(--radius-sm)] border border-hairline px-2 py-1 text-ink-800 hover:bg-honey-100"
                onClick={() => removeProfile(profile.id)}
              >
                Delete imported profile
              </button>
            )}
          </li>
        ))}
      </ul>
      <div className="mt-3">
        <label className="type-mono-meta block text-meta" htmlFor="profile-import">
          Import native profile JSON
        </label>
        <textarea
          id="profile-import"
          className="mt-1 h-32 w-full rounded-[var(--radius-sm)] border border-hairline bg-recessed p-2 type-mono-code text-ink-900"
          value={importText}
          onChange={(event) => setImportText(event.target.value)}
        />
        <button
          type="button"
          className="type-mono-meta mt-2 rounded-[var(--radius-sm)] border border-hairline px-2 py-1 text-ink-800 hover:bg-honey-100"
          onClick={importProfile}
        >
          Import profile
        </button>
        {importStatus === null ? null : (
          <p className="type-mono-meta mt-1 text-meta">{importStatus}</p>
        )}
      </div>
    </section>
  );
}
