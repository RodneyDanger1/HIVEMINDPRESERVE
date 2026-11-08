/** Models (read-only): only values the server reports; contract reference labelled READ. */
import { useEffect, useState } from 'react';
import { apiBaseUrl } from '../api/client';
import { useUiStore } from '../state/context';
import { STRINGS } from '../state/strings';
import { PanelTitle } from './StatePack';

interface RunUsage {
  readonly prompt_tokens: number | null;
  readonly completion_tokens: number | null;
  readonly total_tokens: number | null;
  readonly prompt_cache_hit_tokens: number | null;
  readonly prompt_cache_miss_tokens: number | null;
  readonly reasoning_tokens: number | null;
}

interface ProviderModel {
  readonly id: string;
  readonly name?: string;
  readonly context_window?: number;
  readonly max_output_tokens?: number;
}

interface ProviderBalance {
  readonly currency: string;
  readonly total_balance: string;
  readonly granted_balance?: string;
  readonly topped_up_balance?: string;
}

const WIRE_MODELS = new Set(['deepseek-flash', 'deepseek-v4-pro']);

async function providerFailure(response: Response): Promise<Error> {
  try {
    const body = (await response.json()) as { code?: unknown; message?: unknown };
    const code = typeof body.code === 'string' ? body.code : `HTTP ${response.status}`;
    const message = typeof body.message === 'string' ? body.message : 'Provider metadata unavailable.';
    return new Error(`${code}: ${message}`);
  } catch {
    return new Error(`HTTP ${response.status}: provider metadata unavailable.`);
  }
}

function tokenLabel(value: number | null): string {
  return value === null ? STRINGS.models.notReported : String(value);
}

export function ModelsPanel() {
  const settings = useUiStore((state) => state.settings);
  const probeSettings = useUiStore((state) => state.probeSettings);
  const runId = useUiStore((state) => state.run?.id ?? null);
  const runStatus = useUiStore((state) => state.run?.status ?? null);
  const [usage, setUsage] = useState<RunUsage | null>(null);
  const [usageError, setUsageError] = useState(false);
  const [providerModels, setProviderModels] = useState<readonly ProviderModel[]>([]);
  const [providerBalances, setProviderBalances] = useState<readonly ProviderBalance[]>([]);
  const [providerAvailable, setProviderAvailable] = useState<boolean | null>(null);
  const [providerError, setProviderError] = useState<string | null>(null);
  const [providerLoading, setProviderLoading] = useState(true);
  const [refreshToken, setRefreshToken] = useState(0);

  useEffect(() => {
    if (settings.status === 'unknown') {
      void probeSettings();
    }
  }, [settings.status, probeSettings]);

  useEffect(() => {
    let cancelled = false;
    setProviderLoading(true);
    Promise.all([
      fetch(`${apiBaseUrl()}/api/provider/models`, { headers: { Accept: 'application/json' } }),
      fetch(`${apiBaseUrl()}/api/provider/balance`, { headers: { Accept: 'application/json' } }),
    ])
      .then(async ([modelsResponse, balanceResponse]) => {
        if (!modelsResponse.ok) {
          throw await providerFailure(modelsResponse);
        }
        if (!balanceResponse.ok) {
          throw await providerFailure(balanceResponse);
        }
        const modelsBody = (await modelsResponse.json()) as { models?: unknown };
        const balanceBody = (await balanceResponse.json()) as { balances?: unknown; is_available?: unknown };
        if (!cancelled) {
          setProviderModels(Array.isArray(modelsBody.models) ? (modelsBody.models as ProviderModel[]) : []);
          setProviderBalances(
            Array.isArray(balanceBody.balances) ? (balanceBody.balances as ProviderBalance[]) : [],
          );
          setProviderAvailable(typeof balanceBody.is_available === 'boolean' ? balanceBody.is_available : null);
          setProviderError(null);
          setProviderLoading(false);
        }
      })
      .catch((error: unknown) => {
        if (!cancelled) {
          setProviderError(error instanceof Error ? error.message : 'Provider metadata unavailable.');
          setProviderLoading(false);
        }
      });
    return () => {
      cancelled = true;
    };
  }, [refreshToken]);

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
        <p className="type-mono-meta rounded-[var(--radius-md)] border border-hairline bg-panel-alt px-3 py-2 text-meta">
          {settings.status === 'pending'
            ? 'Probing provider status…'
            : 'Provider status has not been reported yet; the account data below is live.'}
        </p>
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
      {usage === null && !usageError ? (
        <p className="type-mono-meta mt-3 text-meta">No token usage reported for this run yet.</p>
      ) : null}
      <section className="surface-panel mt-3 rounded-[var(--radius-md)] p-3" aria-label="Provider account">
        <div className="flex items-center justify-between gap-2">
          <div className="type-label text-ink-500">Provider account</div>
          <button
            type="button"
            className="type-mono-meta rounded-[var(--radius-sm)] border border-hairline px-2 py-1 text-ink-800 hover:bg-honey-100 disabled:opacity-50"
            disabled={providerLoading}
            onClick={() => setRefreshToken((value) => value + 1)}
          >
            Refresh
          </button>
        </div>
        {providerLoading ? (
          <p className="type-mono-meta mt-2 text-meta">Reading provider account…</p>
        ) : providerError !== null ? (
          <p className="type-mono-meta mt-2 text-signal" role="alert">
            {providerError}
          </p>
        ) : (
          <div className="mt-2 grid gap-3 md:grid-cols-2">
            <div>
              <div className="type-mono-meta text-meta">Balance</div>
              {providerAvailable === false ? (
                <p className="type-mono-meta mt-1 text-signal">Account balance is insufficient.</p>
              ) : null}
              {providerBalances.length === 0 ? (
                <p className="type-mono-meta mt-1 text-meta">No balance rows reported.</p>
              ) : (
                <ul className="mt-1 space-y-1">
                  {providerBalances.map((balance) => (
                    <li key={balance.currency} className="flex items-baseline justify-between gap-2">
                      <span className="type-mono-code text-ink-900">{balance.currency}</span>
                      <span className="type-mono-code text-ink-900">{balance.total_balance}</span>
                    </li>
                  ))}
                </ul>
              )}
            </div>
            <div>
              <div className="type-mono-meta text-meta">Available models</div>
              <ul className="mt-1 space-y-1">
                {providerModels.map((model) => (
                  <li key={model.id} className="rounded-[6px] border border-hairline bg-panel-alt px-2 py-1">
                    <div className="flex items-center justify-between gap-2">
                      <span className="type-mono-code text-ink-900">{model.id}</span>
                      <span className="type-mono-meta text-[7.5px] text-meta">
                        {WIRE_MODELS.has(model.id) ? 'WIRE-SUPPORTED' : 'NOT ACCEPTED'}
                      </span>
                    </div>
                    <div className="type-mono-meta text-[8px] text-meta">
                      {model.name ?? model.id}
                      {model.context_window === undefined ? '' : ` · ${model.context_window} ctx`}
                      {model.max_output_tokens === undefined ? '' : ` · ${model.max_output_tokens} max out`}
                    </div>
                  </li>
                ))}
              </ul>
            </div>
          </div>
        )}
      </section>
      <p className="type-mono-meta mt-3 text-meta">{STRINGS.models.contractNote}</p>
    </div>
  );
}
