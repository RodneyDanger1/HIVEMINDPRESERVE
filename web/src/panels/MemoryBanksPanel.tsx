/**
 * Read-only v14 memory-bank inspector. The server is authoritative; this panel
 * shows bank provenance, revision, trust and the active entries in one bank.
 * It never invents a bank or promotes trust from the UI.
 */
import { useEffect, useState } from 'react';
import { apiBaseUrl } from '../api/client';
import { failureText, httpFailure, refusalFromBody , schemaFailure} from '../api/failure';
import { useUiStore, useUiStoreApi } from '../state/context';
import { EmptyState } from './StatePack';
import { ConfirmDialog } from './ConfirmDialog';

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

interface JournalRow {
  readonly action: string;
  readonly bank_id: string | null;
  readonly created_at: string;
}

interface HyperSummaryRow {
  readonly bank_id: string;
  readonly key: string;
  readonly value: string;
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
  const [failure, setFailure] = useState<string | null>(null);
  const [entryFailure, setEntryFailure] = useState<string | null>(null);
  const [auxFailure, setAuxFailure] = useState<string | null>(null);
  const [confirm, setConfirm] = useState<{ kind: 'clear' } | { kind: 'prune'; count: number } | null>(
    null,
  );
  const [journal, setJournal] = useState<readonly JournalRow[]>([]);
  const [journalVersion, setJournalVersion] = useState(0);
  const [journalIntegrity, setJournalIntegrity] = useState<string | null>(null);
  const [hyperSummary, setHyperSummary] = useState<readonly HyperSummaryRow[]>([]);

  useEffect(() => {
    if (runId === null) {
      setJournalIntegrity(null);
      return;
    }
    let cancelled = false;
    fetch(`${apiBaseUrl()}/api/memory/journal/verify?run_id=${encodeURIComponent(runId)}`, {
      headers: { Accept: 'application/json' },
    })
      .then(async (response) => {
        if (!response.ok) {
          throw await httpFailure(response, 'GET /api/memory/journal/verify');
        }
        const body = (await response.json()) as {
          valid?: boolean;
          checked?: number;
          unchained?: number;
        };
        if (!cancelled) {
          setJournalIntegrity(
            `${body.valid ? 'valid' : 'BROKEN'} · ${body.checked ?? 0} checked · ${body.unchained ?? 0} unchained`,
          );
        }
      })
      .catch((error: unknown) => {
        if (!cancelled) {
          setJournalIntegrity(failureText(error, 'unavailable'));
        }
      });
    return () => {
      cancelled = true;
    };
  }, [runId, journalVersion]);

  useEffect(() => {
    if (runId === null) {
      setJournal([]);
      return;
    }
    let cancelled = false;
    fetch(`${apiBaseUrl()}/api/memory/journal?run_id=${encodeURIComponent(runId)}&limit=20`, {
      headers: { Accept: 'application/json' },
    })
      .then(async (response) => {
        if (!response.ok) {
          throw await httpFailure(response, 'GET /api/memory/journal');
        }
        const body = (await response.json()) as { actions?: unknown };
        if (!Array.isArray(body.actions) || cancelled) {
          return;
        }
        const decoded: JournalRow[] = [];
        for (const raw of body.actions) {
          if (typeof raw !== 'object' || raw === null) {
            continue;
          }
          const item = raw as Record<string, unknown>;
          if (typeof item.action !== 'string') {
            continue;
          }
          decoded.push({
            action: item.action,
            bank_id: typeof item.bank_id === 'string' ? item.bank_id : null,
            created_at: typeof item.created_at === 'string' ? item.created_at : '',
          });
        }
        setJournal(decoded);
      })
      .catch((error: unknown) => {
        if (!cancelled) {
          setJournal([]);
          setAuxFailure(failureText(error, 'Memory journal is unavailable.'));
        }
      });
    return () => {
      cancelled = true;
    };
  }, [runId, journalVersion]);

  useEffect(() => {
    if (runId === null) {
      setHyperSummary([]);
      return;
    }
    let cancelled = false;
    fetch(`${apiBaseUrl()}/api/memory/summary?run_id=${encodeURIComponent(runId)}`, {
      headers: { Accept: 'application/json' },
    })
      .then(async (response) => {
        if (!response.ok) throw await httpFailure(response, 'GET /api/memory/summary');
        const body = (await response.json()) as { summaries?: unknown };
        if (!Array.isArray(body.summaries) || cancelled) return;
        const decoded: HyperSummaryRow[] = [];
        for (const raw of body.summaries) {
          if (typeof raw !== 'object' || raw === null) continue;
          const item = raw as Record<string, unknown>;
          if (
            typeof item.bank_id !== 'string' ||
            typeof item.key !== 'string' ||
            typeof item.value !== 'string'
          ) {
            continue;
          }
          decoded.push({ bank_id: item.bank_id, key: item.key, value: item.value });
        }
        setHyperSummary(decoded);
      })
      .catch((error: unknown) => {
        if (!cancelled) {
          setHyperSummary([]);
          setAuxFailure(failureText(error, 'Memory summary is unavailable.'));
        }
      });
    return () => {
      cancelled = true;
    };
  }, [runId]);

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
          throw await httpFailure(response, 'GET /api/memory/banks');
        }
        const body = (await response.json()) as { banks?: unknown };
        if (!Array.isArray(body.banks)) {
          throw schemaFailure('memory banks payload has no banks');
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
          setFailure(null);
          setStatus('ok');
        }
      })
      .catch((error: unknown) => {
        if (!cancelled) {
          setBanks([]);
          setFailure(failureText(error, 'Memory banks are unavailable.'));
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
    // The URL embeds `runId`, so a same-bank run switch must refetch and must
    // not leave the previous run's rows visible while the read is in flight.
    const controller = new AbortController();
    setEntries([]);
    setEntryFailure(null);
    setEntryStatus('loading');
    fetch(`${apiBaseUrl()}/api/memory/banks/${encodeURIComponent(selected)}/entries?run_id=${encodeURIComponent(runId ?? '')}&limit=50`, {
      headers: { Accept: 'application/json' },
      signal: controller.signal,
    })
      .then(async (response) => {
        if (!response.ok) {
          throw await httpFailure(response, 'GET memory bank entries');
        }
        const body = (await response.json()) as { entries?: unknown };
        if (!Array.isArray(body.entries)) {
          throw schemaFailure('memory entries payload has no entries');
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
          setEntryFailure(null);
          setEntryStatus('ok');
        }
      })
      .catch((error: unknown) => {
        if (!cancelled) {
          setEntries([]);
          setEntryFailure(failureText(error, 'Memory entries are unavailable.'));
          setEntryStatus('error');
        }
      });
    return () => {
      cancelled = true;
      controller.abort();
    };
  }, [selected, runId]);

  const executeClear = async (): Promise<void> => {
    if (!runId) {
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
        const body = (await response.json().catch(() => ({}))) as Record<string, unknown>;
        throw refusalFromBody(body, response.status, 'POST /api/memory/clear');
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
      setClearReceipt(failureText(error, 'clear refused'));
    }
  };

  const askClear = (): void => {
    if (!runId || !runStatus || !['completed', 'failed', 'cancelled', 'killed', 'interrupted'].includes(runStatus)) {
      setClearReceipt('Only a terminal run can be cleared.');
      return;
    }
    setConfirm({ kind: 'clear' });
  };

  const prune = async (): Promise<void> => {
    if (!runId) {
      return;
    }
    const token = storeApi.getState().session?.token;
    if (!token) {
      setClearReceipt('Session token unavailable; nothing pruned.');
      return;
    }
    try {
      const dry = await fetch(`${apiBaseUrl()}/api/memory/prune`, {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
          Accept: 'application/json',
          'x-hivemind-token': token,
        },
        body: JSON.stringify({ run_id: runId, confirm: true, dry_run: true }),
      });
      if (!dry.ok) {
        const body = (await dry.json().catch(() => ({}))) as Record<string, unknown>;
        throw refusalFromBody(body, dry.status, 'POST /api/memory/prune');
      }
      const preview = (await dry.json()) as { removed?: number };
      const count = preview.removed ?? 0;
      if (count === 0) {
        setClearReceipt('No inactive entries to prune.');
        return;
      }
      setConfirm({ kind: 'prune', count });
    } catch (error: unknown) {
      setClearReceipt(failureText(error, 'prune refused'));
    }
  };

  const executePrune = async (): Promise<void> => {
    if (!runId) {
      return;
    }
    const token = storeApi.getState().session?.token;
    if (!token) {
      setClearReceipt('Session token unavailable; nothing pruned.');
      return;
    }
    try {
      const applied = await fetch(`${apiBaseUrl()}/api/memory/prune`, {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
          Accept: 'application/json',
          'x-hivemind-token': token,
        },
        body: JSON.stringify({ run_id: runId, confirm: true, dry_run: false }),
      });
      if (!applied.ok) {
        const body = (await applied.json().catch(() => ({}))) as Record<string, unknown>;
        throw refusalFromBody(body, applied.status, 'POST /api/memory/prune');
      }
      const receipt = (await applied.json()) as { removed?: number };
      setClearReceipt(`Pruned ${receipt.removed ?? 0} inactive entries.`);
      setJournalVersion((version) => version + 1);
    } catch (error: unknown) {
      setClearReceipt(failureText(error, 'prune refused'));
    }
  };

  const confirmExecute = (): void => {
    const action = confirm;
    setConfirm(null);
    if (action?.kind === 'clear') {
      void executeClear();
    } else if (action?.kind === 'prune') {
      void executePrune();
    }
  };

  const promote = async (entry: EntryRow): Promise<void> => {    if (!runId || !selected || !entry.trust || entry.trust === 'verified' || entry.trust === 'authoritative') {
      return;
    }
    const token = storeApi.getState().session?.token;
    if (!token) {
      setClearReceipt('Session token unavailable; trust unchanged.');
      return;
    }
    try {
      const response = await fetch(`${apiBaseUrl()}/api/memory/trust`, {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
          Accept: 'application/json',
          'x-hivemind-token': token,
        },
        body: JSON.stringify({
          run_id: runId,
          bank_id: selected,
          entry_id: entry.id,
          trust: 'verified',
        }),
      });
      if (!response.ok) {
        const body = (await response.json().catch(() => ({}))) as Record<string, unknown>;
        throw refusalFromBody(body, response.status, 'POST /api/memory/trust');
      }
      setEntries((current) =>
        current.map((row) => (row.id === entry.id ? { ...row, trust: 'verified' } : row)),
      );
      setJournalVersion((version) => version + 1);
    } catch (error: unknown) {
      setClearReceipt(failureText(error, 'trust refused'));
    }
  };

  return (
    <section aria-label="Memory banks" className="surface-panel mt-3 rounded-[var(--radius-card)] p-3">
      <div className="flex items-center justify-between gap-2">
        <h3 className="type-label text-ink-900">Memory banks</h3>
        <div className="flex items-center gap-1">
          <button
            type="button"
            className="type-label h-8 rounded-[var(--radius-button)] border border-hairline px-3 text-ink-800 hover:bg-honey-100 transition-colors"
            disabled={runId === null}
            onClick={() => void prune()}          >
            Prune inactive
          </button>
          <button
            type="button"
            className="type-label h-8 rounded-[var(--radius-button)] border border-signal px-3 text-signal hover:bg-recessed transition-colors"
            disabled={runId === null}
            onClick={askClear}
          >
            Clear run memory
          </button>
        </div>
      </div>
      <div className="mt-2 rounded-[var(--radius-sm)] border border-hairline bg-panel-alt p-2">
        <div className="type-label text-ink-900">HyperAgent summaries · bounded values, full raw disabled</div>
        <div className="type-mono-meta mt-1 text-meta">
          Only run/hive summaries are projected, truncated at 512 characters; no sibling branch
          raw entries.
        </div>
        {hyperSummary.length === 0 ? (
          <div className="type-mono-meta mt-1 text-meta">No summaries recorded.</div>
        ) : (
          <ul className="mt-1 space-y-0.5">
            {hyperSummary.map((row) => (
              <li key={`${row.bank_id}-${row.key}`} className="type-mono-meta text-meta">
                {row.bank_id} · {row.key} · {row.value}
              </li>
            ))}
          </ul>
        )}
      </div>      {clearReceipt ? <p className="type-mono-meta mt-1 text-meta">{clearReceipt}</p> : null}
      {runId === null ? <EmptyState message="No active run to inspect." /> : null}
      {status === 'loading' ? (
        <p className="type-mono-meta mt-1 text-meta">Loading memory banks…</p>
      ) : null}
      {status === 'error' ? (
        <p role="alert" className="type-mono-meta mt-1 text-signal">
          {failure ?? 'Memory banks unavailable; no bank invented.'}
        </p>
      ) : null}
      {auxFailure !== null && status !== 'error' ? (
        <p role="alert" className="type-mono-meta mt-1 text-signal">
          {auxFailure}
        </p>
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
              <p role="alert" className="type-mono-meta text-signal">
                {entryFailure ?? 'Entries unavailable; no entry invented.'}
              </p>
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
                  {entry.trust === 'unverified' || entry.trust === 'quarantined' ? (
                    <button
                      type="button"
                      className="mt-1 type-label h-8 rounded-[var(--radius-button)] border border-honey-500 bg-honey-300 px-3 font-semibold text-ink-900 hover:bg-honey-400 transition-colors"
                      onClick={() => void promote(entry)}
                    >
                      Promote to verified
                    </button>
                  ) : null}
                </li>
              ))}
            </ul>
          </div>
        </div>
      ) : null}
      {journal.length > 0 ? (
        <div className="mt-2 border-t border-hairline pt-2">
          <div className="type-mono-meta text-meta">Recent journal</div>
          {journalIntegrity ? (
            <div className="type-mono-meta text-meta">Chain: {journalIntegrity}</div>
          ) : null}
          <ul className="mt-1 space-y-0.5">
            {journal.map((row, index) => (
              <li key={`${row.action}-${row.created_at}-${index}`} className="type-mono-meta text-meta">
                {row.action} · {row.bank_id?.slice(0, 24) ?? 'run'} · {row.created_at}
              </li>
            ))}
          </ul>
        </div>
      ) : null}
      <ConfirmDialog
        open={confirm !== null}
        title={confirm?.kind === 'clear' ? 'Clear run memory' : 'Prune inactive memory'}
        body={
          confirm?.kind === 'clear'
            ? 'This deletes this terminal run’s legacy rows and bank entries. The journal and snapshot hashes remain as audit records.'
            : `Permanently remove ${confirm?.kind === 'prune' ? confirm.count : 0} inactive memory entries? Active entries are kept.`
        }
        confirmLabel={confirm?.kind === 'clear' ? 'Clear memory' : 'Prune entries'}
        danger={confirm?.kind === 'clear'}
        onConfirm={confirmExecute}
        onCancel={() => setConfirm(null)}
      />
    </section>
  );
}
