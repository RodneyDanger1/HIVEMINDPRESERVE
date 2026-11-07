/** Models (read-only): only values the server reports; contract reference labelled READ. */
import { useEffect, useState } from 'react';
import { apiBaseUrl } from '../api/client';
import { useUiStore } from '../state/context';
import { STRINGS } from '../state/strings';
import { PanelTitle, EmptyState } from './StatePack';

interface RunUsage {
  readonly prompt_tokens: number | null;
  readonly completion_tokens: number | null;
  readonly total_tokens: number | null;
  readonly prompt_cache_hit_tokens: number | null;
  readonly prompt_cache_miss_tokens: number | null;
  readonly reasoning_tokens: number | null;
}

function tokenLabel(value: number | null): string {
  return value === null ? STRINGS.models.notReported : String(value);
}

export function ModelsPanel() {
  const settings = useUiStore((state) => state.settings);
  const runId = useUiStore((state) => state.run?.id ?? null);
  const runStatus = useUiStore((state) => state.run?.status ?? null);
  const [usage, setUsage] = useState<RunUsage | null>(null);
  const [usageError, setUsageError] = useState(false);

  useEffect(() => {
    if (runId === null) {
      setUsage(null);
      setUsageError(false);
      return;
    }
    let cancelled = false;
    fetch(`${apiBaseUrl()}/api/usage/runs/${encodeURIComponent(runId)}`, {
      headers: { Accept: 'application/json' },
    })
      .then(async (response) => {
        if (!response.ok) {
          throw new Error(`usage HTTP ${response.status}`);
        }
        const body = (await response.json()) as { usage?: unknown };
        if (body.usage === null) {
          if (!cancelled) {
            setUsage(null);
            setUsageError(false);
          }
          return;
        }
        if (typeof body.usage !== 'object' || body.usage === null) {
          throw new Error('usage payload is malformed');
        }
        const item = body.usage as Record<string, unknown>;
        if (!cancelled) {
          setUsage({
            prompt_tokens: typeof item.prompt_tokens === 'number' ? item.prompt_tokens : null,
            completion_tokens: typeof item.completion_tokens === 'number' ? item.completion_tokens : null,
            total_tokens: typeof item.total_tokens === 'number' ? item.total_tokens : null,
            prompt_cache_hit_tokens:
              typeof item.prompt_cache_hit_tokens === 'number' ? item.prompt_cache_hit_tokens : null,
            prompt_cache_miss_tokens:
              typeof item.prompt_cache_miss_tokens === 'number' ? item.prompt_cache_miss_tokens : null,
            reasoning_tokens:
              typeof item.reasoning_tokens === 'number' ? item.reasoning_tokens : null,
          });
          setUsageError(false);
        }
      })
      .catch(() => {
        if (!cancelled) {
          setUsage(null);
          setUsageError(true);
        }
      });
    return () => {
      cancelled = true;
    };
  }, [runId, runStatus]);

  return (
    <div className="mx-auto max-w-[760px] p-4">
      <PanelTitle>{STRINGS.panels.modelsTitle}</PanelTitle>
      {settings.provider === null && settings.model === null ? (
        <EmptyState message={STRINGS.models.none} />
      ) : (
        <dl className="surface-panel grid grid-cols-[140px_1fr] gap-y-1 rounded-[var(--radius-md)] p-3">
          <dt className="type-mono-meta text-meta">{STRINGS.models.providerLabel}</dt>
          <dd className="type-mono-code text-ink-900">{settings.provider ?? STRINGS.models.notReported}</dd>
          <dt className="type-mono-meta text-meta">{STRINGS.models.modelLabel}</dt>
          <dd className="type-mono-code text-ink-900">{settings.model ?? STRINGS.models.notReported}</dd>
        </dl>
      )}
      {usage !== null ? (
        <dl className="surface-panel mt-3 grid grid-cols-[140px_1fr] gap-y-1 rounded-[var(--radius-md)] p-3">
          <dt className="type-mono-meta text-meta">Prompt tokens</dt>
          <dd className="type-mono-code text-ink-900">{tokenLabel(usage.prompt_tokens)}</dd>
          <dt className="type-mono-meta text-meta">Completion tokens</dt>
          <dd className="type-mono-code text-ink-900">{tokenLabel(usage.completion_tokens)}</dd>
          <dt className="type-mono-meta text-meta">Total tokens</dt>
          <dd className="type-mono-code text-ink-900">{tokenLabel(usage.total_tokens)}</dd>
          <dt className="type-mono-meta text-meta">Cache hit / miss</dt>
          <dd className="type-mono-code text-ink-900">
            {tokenLabel(usage.prompt_cache_hit_tokens)} / {tokenLabel(usage.prompt_cache_miss_tokens)}
          </dd>
          <dt className="type-mono-meta text-meta">Reasoning tokens</dt>
          <dd className="type-mono-code text-ink-900">{tokenLabel(usage.reasoning_tokens)}</dd>
        </dl>
      ) : null}
      {usageError ? (
        <p className="type-mono-meta mt-3 text-signal">Usage unavailable; no token count invented.</p>
      ) : null}
      <p className="type-mono-meta mt-3 text-meta">{STRINGS.models.contractNote}</p>
    </div>
  );
}
