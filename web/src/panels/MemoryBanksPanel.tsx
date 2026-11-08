/**
 * Read-only v14 memory-bank inspector. The server is authoritative; this panel
 * shows bank provenance, revision, trust and the active entries in one bank.
 * It never invents a bank or promotes trust from the UI.
 */
import { useEffect, useState } from 'react';
import { apiBaseUrl } from '../api/client';
import { useUiStore, useUiStoreApi } from '../state/context';
import { EmptyState } from './StatePack';

interface BankRow {
  readonly id: string;
  readonly kind: string;
  readonly scope_id: string;
  readonly display_name: string | null;
  readonly rev: number;
  readonly owner_task_id: string | null;
  readonly parent_bank_id: string | null;
}

interface EntryRow {
  readonly id: string;
  readonly key: string;
  readonly kind: string;
  readonly value: string;
  readonly trust: string | null;
  readonly source_kind: string;
  readonly created_at: string;
}

type Status = 'idle' | 'loading' | 'ok' | 'error';

export function MemoryBanksPanel() {
  const storeApi = useUiStoreApi();
  const runId = useUiStore((state) => state.run?.id ?? null);
  const runStatus = useUiStore((state) => state.run?.status ?? null);
  const [status, setStatus] = useState<Status>('idle');
  const [banks, setBanks] = useState<readonly BankRow[]>([]);
  const [selected, setSelected] = useState<string | null>(null);
  const [entries, setEntries] = useState<readonly EntryRow[]>([]);
  const [entryStatus, setEntryStatus] = useState<Status>('idle');
  const [clearReceipt, setClearReceipt] = useState<string | null>(null);

  useEffect(() => {
    if (runId === null) {
      setBanks([]);
      setSelected(null);
      setEntries([]);
      setStatus('idle');
      return;
    }
    let cancelled = false;
    setStatus('loading');
    fetch(`${apiBaseUrl()}/api/memory/banks?run_id=${encodeURIComponent(runId)}`, {
      headers: { Accept: 'application/json' },
    })
      .then(async (response) => {
        if (!response.ok) {
          throw new Error(`memory banks HTTP ${response.status}`);
        }
        const body = (await response.json()) as { banks?: unknown };
        if (!Array.isArray(body.banks)) {
          throw new Error('memory banks payload has no banks');
        }
        const decoded: BankRow[] = [];
        for (const raw of body.banks) {
          if (typeof raw !== 'object' || raw === null) {
            continue;
          }
          const item = raw as Record<string, unknown>;
          if (typeof item.id !== 'string' || typeof item.kind !== 'string') {
            continue;
          }
          decoded.push({
            id: item.id,
            kind: item.kind,
            scope_id: typeof item.scope_id === 'string' ? item.scope_id : '',
            display_name: typeof item.display_name === 'string' ? item.display_name : null,
            rev: typeof item.rev === 'number' ? item.rev : 0,
            owner_task_id: typeof item.owner_task_id === 'string' ? item.owner_task_id : null,
            parent_bank_id: typeof item.parent_bank_id === 'string' ? item.parent_bank_id : null,
          });
        }
        if (!cancelled) {
          setBanks(decoded);
          setSelected(decoded[0]?.id ?? null);
          setStatus('ok');
        }
      })
      .catch(() => {
        if (!cancelled) {
          setBanks([]);
          setStatus('error');
        }
      });
    return () => {
      cancelled = true;
    };
  }, [runId]);

  useEffect(() => {
    if (selected === null) {
      setEntries([]);
      setEntryStatus('idle');
      return;
    }
    let cancelled = false;
    setEntryStatus('loading');
    fetch(`${apiBaseUrl()}/api/memory/banks/${encodeURIComponent(selected)}/entries?limit=50`, {
      headers: { Accept: 'application/json' },
    })
      .then(async (response) => {
        if (!response.ok) {
          throw new Error(`memory entries HTTP ${response.status}`);
        }
        const body = (await response.json()) as { entries?: unknown };
        if (!Array.isArray(body.entries)) {
          throw new Error('memory entries payload has no entries');
        }
        const decoded: EntryRow[] = [];
        for (const raw of body.entries) {
          if (typeof raw !== 'object' || raw === null) {
            continue;
          }
          const item = raw as Record<string, unknown>;
          if (
            typeof item.id !== 'string' ||
            typeof item.key !== 'string' ||
            typeof item.kind !== 'string' ||
            typeof item.value !== 'string'
          ) {
            continue;
          }
          decoded.push({
            id: item.id,
            key: item.key,
            kind: item.kind,
            value: item.value,
            trust: typeof item.trust === 'string' ? item.trust : null,
            source_kind: typeof item.source_kind === 'string' ? item.source_kind : 'unknown',
            created_at: typeof item.created_at === 'string' ? item.created_at : '',
          });
        }
        if (!cancelled) {
          setEntries(decoded);
          setEntryStatus('ok');
        }
      })
      .catch(() => {
        if (!cancelled) {
          setEntries([]);
          setEntryStatus('error');
        }
      });
    return () => {
      cancelled = true;
    };
  }, [selected]);

  const clear = async (): Promise<void> => {
    if (!runId || !runStatus || !['completed', 'failed', 'cancelled', 'killed', 'interrupted'].includes(runStatus)) {
      setClearReceipt('Only a terminal run can be cleared.');
      return;
    }
    if (!window.confirm('Clear this run’s memory banks and legacy rows? This cannot be undone.')) {
      return;
    }
    const token = storeApi.getState().session?.token;
    if (!token) {
      setClearReceipt('Session token unavailable; nothing cleared.');
      return;
    }
    try {
      const response = await fetch(`${apiBaseUrl()}/api/memory/clear`, {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
          Accept: 'application/json',
          'x-hivemind-token': token,
        },
        body: JSON.stringify({ run_id: runId, confirm: true, reason: 'operator clear from bank inspector' }),
      });
      if (!response.ok) {
        const body = (await response.json().catch(() => ({}))) as { code?: string };
        throw new Error(body.code ?? `clear HTTP ${response.status}`);
      }
      const receipt = (await response.json()) as {
        flat_rows?: number;
        bank_entries?: number;
        banks_closed?: number;
      };
      setClearReceipt(
        `Cleared ${receipt.flat_rows ?? 0} flat rows, ${receipt.bank_entries ?? 0} bank entries, ${receipt.banks_closed ?? 0} banks.`,
      );
      setBanks([]);
      setEntries([]);
      setSelected(null);
    } catch (error: unknown) {
      setClearReceipt(error instanceof Error ? error.message : 'clear refused');
    }
  };

  return (
    <section aria-label="Memory banks" className="surface-panel mt-3 rounded-[14px] p-3">
      <div className="flex items-center justify-between gap-2">
        <h3 className="type-label text-ink-900">Memory banks</h3>
        <button
          type="button"
          className="h-7 rounded-[6px] border border-hairline px-2 type-mono-meta text-signal"
          disabled={runId === null}
          onClick={() => void clear()}
        >
          Clear run memory
        </button>
      </div>
      {clearReceipt ? <p className="type-mono-meta mt-1 text-meta">{clearReceipt}</p> : null}
      {runId === null ? <EmptyState message="No active run to inspect." /> : null}
      {status === 'loading' ? (
        <p className="type-mono-meta mt-1 text-meta">Loading memory banks…</p>
      ) : null}
      {status === 'error' ? (
        <p className="type-mono-meta mt-1 text-signal">Memory banks unavailable; no bank invented.</p>
      ) : null}
      {status === 'ok' && banks.length === 0 ? (
        <EmptyState message="No memory banks for this run yet." />
      ) : null}
      {status === 'ok' && banks.length > 0 ? (
        <div className="mt-2 grid gap-2 sm:grid-cols-2">
          <ul className="space-y-1">
            {banks.map((bank) => (
              <li key={bank.id}>
                <button
                  type="button"
                  className={`w-full rounded-[var(--radius-sm)] border px-2 py-1 text-left ${
                    selected === bank.id ? 'border-signal' : 'border-hairline'
                  }`}
                  onClick={() => setSelected(bank.id)}
                >
                  <div className="type-mono-code text-ink-900">
                    {bank.kind} · rev {bank.rev}
                  </div>
                  <div className="type-mono-meta text-meta">
                    {bank.display_name ?? bank.scope_id}
                    {bank.owner_task_id ? ` · task ${bank.owner_task_id.slice(0, 8)}` : ''}
                  </div>
                </button>
              </li>
            ))}
          </ul>
          <div>
            {entryStatus === 'loading' ? (
              <p className="type-mono-meta text-meta">Loading entries…</p>
            ) : null}
            {entryStatus === 'error' ? (
              <p className="type-mono-meta text-signal">Entries unavailable; no entry invented.</p>
            ) : null}
            {entryStatus === 'ok' && entries.length === 0 ? (
              <p className="type-mono-meta text-meta">No active entries in this bank.</p>
            ) : null}
            <ul className="space-y-1">
              {entries.map((entry) => (
                <li key={entry.id} className="rounded-[var(--radius-sm)] border border-hairline p-2">
                  <div className="type-mono-meta text-meta">
                    {entry.kind} · trust {entry.trust ?? 'unassessed'} · {entry.source_kind}
                  </div>
                  <div className="type-mono-code break-words text-ink-900">{entry.key}</div>
                  <div className="type-mono-meta break-words text-meta">{entry.value}</div>
                </li>
              ))}
            </ul>
          </div>
        </div>
      ) : null}
    </section>
  );
}
