/** Models (read-only): only values the server reports; contract reference labelled READ. */
import { useEffect, useState } from 'react';
import { apiBaseUrl } from '../api/client';
import { failureText, httpFailure } from '../api/failure';
import { clientRefusal, RefusalError } from '../api/refusals';
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

/**
 * Provider metadata only. Wire support is the server's call: the UI must not
 * claim or deny a model is accepted from a hardcoded provider list.
 */
async function providerFailure(response: Response): Promise<Error> {
  return httpFailure(response, 'GET /api/provider/models');
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
  const [usageFailure, setUsageFailure] = useState<string | null>(null);
  const [providerModels, setProviderModels] = useState<readonly ProviderModel[]>([]);
  const [providerBalances, setProviderBalances] = useState<readonly ProviderBalance[]>([]);
  const [providerAvailable, setProviderAvailable] = useState<boolean | null>(null);
  const [providerBalanceSupported, setProviderBalanceSupported] = useState(true);
  const [providerBalanceNote, setProviderBalanceNote] = useState<string | null>(null);
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
    setProviderError(null);

    // Models and balance are independent: one failing endpoint must not hide
    // the other's truthful result.
    void fetch(`${apiBaseUrl()}/api/provider/models`, { headers: { Accept: 'application/json' } })
      .then(async (modelsResponse) => {
        if (!modelsResponse.ok) {
          throw await providerFailure(modelsResponse);
        }
        const modelsBody = (await modelsResponse.json()) as { models?: unknown };
        if (!Array.isArray(modelsBody.models)) {
          throw new RefusalError(
            clientRefusal('E_SCHEMA_INVALID', {
              message: 'GET /api/provider/models returned no models array.',
            }),
          );
        }
        if (!cancelled) {
          setProviderModels(modelsBody.models as ProviderModel[]);
        }
      })
      .catch((error: unknown) => {
        if (!cancelled) {
          setProviderModels([]);
          setProviderError(
            failureText(error, 'Provider metadata unavailable.'),
          );
        }
      })
      .finally(() => {
        if (!cancelled) {
          setProviderLoading(false);
        }
      });

    void fetch(`${apiBaseUrl()}/api/provider/balance`, { headers: { Accept: 'application/json' } })
      .then(async (balanceResponse) => {
        if (!balanceResponse.ok) {
          if (!cancelled) {
            setProviderBalances([]);
            setProviderAvailable(null);
            setProviderBalanceSupported(false);
            setProviderBalanceNote(
              'This endpoint does not report an account balance; no balance or auth state is inferred.',
            );
          }
          return;
        }
        const balanceBody = (await balanceResponse.json()) as {
          balances?: unknown;
          is_available?: unknown;
          supported?: unknown;
          reason?: unknown;
        };
        if (cancelled) {
          return;
        }
        if (balanceBody.supported === false) {
          setProviderBalances([]);
          setProviderAvailable(null);
          setProviderBalanceSupported(false);
          setProviderBalanceNote(
            typeof balanceBody.reason === 'string'
              ? balanceBody.reason
              : 'This endpoint does not report an account balance.',
          );
          return;
        }
        setProviderBalanceSupported(true);
        setProviderBalanceNote(null);
        setProviderBalances(
          Array.isArray(balanceBody.balances) ? (balanceBody.balances as ProviderBalance[]) : [],
        );
        setProviderAvailable(
          typeof balanceBody.is_available === 'boolean' ? balanceBody.is_available : null,
        );
      })
      .catch(() => {
        if (!cancelled) {
          setProviderBalances([]);
          setProviderAvailable(null);
          setProviderBalanceSupported(false);
          setProviderBalanceNote(
            'Balance status was not reported by the server; nothing is inferred.',
          );
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
      setUsageFailure(null);
      return;
    }
    let cancelled = false;
    fetch(`${apiBaseUrl()}/api/usage/runs/${encodeURIComponent(runId)}`, {
      headers: { Accept: 'application/json' },
    })
      .then(async (response) => {
        if (!response.ok) {
          throw await httpFailure(response, `GET /api/usage/runs/${runId}`);
        }
        const body = (await response.json()) as { usage?: unknown };
        if (body.usage === null) {
          if (!cancelled) {
            setUsage(null);
            setUsageError(false);
            setUsageFailure(null);
          }
          return;
        }
        if (typeof body.usage !== 'object' || body.usage === null) {
          throw new RefusalError(
            clientRefusal('E_SCHEMA_INVALID', {
              message: 'The usage route returned a malformed usage payload.',
            }),
          );
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
          setUsageFailure(null);
        }
      })
      .catch((error: unknown) => {
        if (!cancelled) {
          setUsage(null);
          setUsageError(true);
          setUsageFailure(failureText(error, 'Usage is unavailable for this run.'));
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
        <p className="type-mono-meta mt-3 text-signal">
          {usageFailure ?? 'Usage unavailable; no token count invented.'}
        </p>
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
        {providerError !== null || providerLoading ? (
          <p className="type-mono-meta mt-2 text-signal" role={providerError === null ? undefined : 'alert'}>
            {providerError === null ? 'Reading provider models…' : providerError}
          </p>
        ) : null}
        <div className="mt-2 grid gap-3 md:grid-cols-2">
          <div>
            <div className="type-mono-meta text-meta">Balance</div>
            {!providerBalanceSupported ? (
              <p className="type-mono-meta mt-1 text-meta">
                {providerBalanceNote ?? 'This endpoint does not report an account balance.'}
              </p>
            ) : null}
            {providerAvailable === false ? (
              <p className="type-mono-meta mt-1 text-signal">Account balance is insufficient.</p>
            ) : null}
            {providerBalanceSupported && providerBalances.length === 0 ? (
              <p className="type-mono-meta mt-1 text-meta">No balance rows reported.</p>
            ) : null}
            {providerBalanceSupported ? (
              <ul className="mt-1 space-y-1">
                {providerBalances.map((balance) => (
                  <li key={balance.currency} className="flex items-baseline justify-between gap-2">
                    <span className="type-mono-code text-ink-900">{balance.currency}</span>
                    <span className="type-mono-code text-ink-900">{balance.total_balance}</span>
                  </li>
                ))}
              </ul>
            ) : null}
          </div>
          <div>
            <div className="type-mono-meta text-meta">Available models</div>
            <div className="type-mono-meta mt-1 text-meta">
              Configured model: {settings.model ?? STRINGS.models.notReported}
            </div>
            {providerError === null ? (
              <ul className="mt-1 space-y-1">
                {providerModels.map((model) => (
                  <li key={model.id} className="rounded-[var(--radius-sm)] border border-hairline bg-panel-alt px-2 py-1">
                    <div className="flex items-center justify-between gap-2">
                      <span className="type-mono-code text-ink-900">{model.id}</span>
                      <span className="type-mono-meta text-[11px] text-meta">
                        {model.id === settings.model ? 'CONFIGURED' : 'CATALOG'}
                      </span>
                    </div>
                    <div className="type-mono-meta text-[11px] text-meta">
                      {model.name ?? model.id}
                      {model.context_window === undefined ? '' : ` · ${model.context_window} ctx`}
                      {model.max_output_tokens === undefined ? '' : ` · ${model.max_output_tokens} max out`}
                    </div>
                  </li>
                ))}
              </ul>
            ) : (
              <p className="type-mono-meta mt-1 text-meta">
                Model list unavailable; no model is invented and balance above is still server-reported.
              </p>
            )}
          </div>
        </div>
      </section>
      <p className="type-mono-meta mt-3 text-meta">{STRINGS.models.contractNote}</p>
    </div>
  );
}
