/**
 * Operator-gated self-evolution proposal projection. Proposals are recorded by
 * the root tool; four SuperAgents vote, the operator authorizes, and apply or
 * rollback changes only the lesson ledger. This panel cannot change code,
 * prompts, tools, personas, or gates, and it reloads server truth after every
 * mutation instead of patching rows optimistically.
 */
import { useEffect, useState } from 'react';
import { apiBaseUrl } from '../api/client';
import { failureText, httpFailure } from '../api/failure';
import { useUiStore } from '../state/context';
import { EmptyState, ErrorState, LoadingState } from './StatePack';
import { fetchWithBoundTimeout } from '../api/http';

interface ProposalGovernance {
  readonly requiredApprovals: number | null;
  readonly superagentApprovals: number | null;
  readonly operatorApprovals: number | null;
  readonly unanimous: boolean | null;
  readonly state: string | null;
  readonly canApply: boolean | null;
}

export interface ProposalRow {
  readonly proposalKey: string;
  readonly area: string;
  readonly problem: string;
  readonly proposal: string;
  readonly applied: boolean;
  readonly voteState: string | null;
  readonly currentApprovals: number;
  readonly currentRejections: number;
  readonly authorized: boolean;
  readonly lessonApplied: boolean;
  readonly lessonSha256: string | null;
  readonly governance: ProposalGovernance;
}

type Status = 'idle' | 'loading' | 'ok' | 'error';

function text(value: unknown): string | null {
  return typeof value === 'string' ? value : null;
}

function bool(value: unknown): boolean | null {
  return typeof value === 'boolean' ? value : null;
}

function num(value: unknown): number | null {
  return typeof value === 'number' && Number.isFinite(value) ? value : null;
}

/** Decodes the server proposal projection without inventing missing fields. */
export function decodeProposals(body: unknown): ProposalRow[] {
  if (typeof body !== 'object' || body === null) {
    return [];
  }
  const raw = (body as Record<string, unknown>).proposals;
  if (!Array.isArray(raw)) {
    return [];
  }
  const rows: ProposalRow[] = [];
  for (const item of raw) {
    if (typeof item !== 'object' || item === null) {
      continue;
    }
    const record = item as Record<string, unknown>;
    const proposalKey = text(record.proposal_key);
    const area = text(record.area);
    const problem = text(record.problem);
    const proposal = text(record.proposal);
    if (proposalKey === null || area === null || problem === null || proposal === null) {
      continue;
    }
    const governanceRaw =
      typeof record.governance === 'object' && record.governance !== null
        ? (record.governance as Record<string, unknown>)
        : {};
    rows.push({
      proposalKey,
      area,
      problem,
      proposal,
      applied: bool(record.applied) ?? false,
      voteState: text(record.vote_state),
      currentApprovals: num(record.current_superagent_approvals) ?? 0,
      currentRejections: num(record.current_superagent_rejections) ?? 0,
      authorized: bool(record.authorized) ?? false,
      lessonApplied: bool(record.lesson_applied) ?? false,
      lessonSha256: text(record.lesson_sha256),
      governance: {
        requiredApprovals: num(governanceRaw.required_approvals),
        superagentApprovals: num(governanceRaw.superagent_approvals),
        operatorApprovals: num(governanceRaw.operator_approvals),
        unanimous: bool(governanceRaw.unanimous),
        state: text(governanceRaw.state),
        canApply: bool(governanceRaw.can_apply),
      },
    });
  }
  return rows;
}

export function canAuthorizeProposal(row: ProposalRow): boolean {
  return (
    !row.authorized &&
    row.applied === false &&
    row.currentRejections === 0 &&
    row.currentApprovals >= 4 &&
    row.governance.canApply === false
  );
}

export function canApplyLesson(row: ProposalRow): boolean {
  return row.authorized && !row.lessonApplied;
}

export function canRollbackLesson(row: ProposalRow): boolean {
  return row.lessonApplied;
}

/**
 * Mutation auth for the three prose writes. The server accepts
 * `Authorization: Bearer` or `x-hivemind-token`; every other proven write panel
 * uses `x-hivemind-token`, so this panel matches that single pattern (style
 * consistency, not a 401 fix).
 */
export function proposalWriteHeaders(token: string | null): Record<string, string> {
  return {
    Accept: 'application/json',
    ...(token === null ? {} : { 'x-hivemind-token': token }),
  };
}

export function ProposalsPanel() {
  const runId = useUiStore((state) => state.run?.id ?? null);
  const sessionToken = useUiStore((state) => state.session?.token ?? null);
  const [status, setStatus] = useState<Status>('idle');
  const [rows, setRows] = useState<readonly ProposalRow[]>([]);
  const [votePath, setVotePath] = useState<string | null>(null);
  const [failure, setFailure] = useState<string | null>(null);
  const [actionError, setActionError] = useState<string | null>(null);
  const [authorizing, setAuthorizing] = useState(false);
  const [reloadToken, setReloadToken] = useState(0);

  useEffect(() => {
    if (runId === null) {
      setRows([]);
      setStatus('idle');
      return;
    }
    let cancelled = false;
    const controller = new AbortController();
    setStatus('loading');
    fetchWithBoundTimeout(
      `${apiBaseUrl()}/api/agents/runs/${encodeURIComponent(runId)}/proposals`,
      {
        label: 'GET proposals',
        signal: controller.signal,
        init: { headers: { Accept: 'application/json' } },
      },
    )
      .then(async (response) => {
        if (!response.ok) {
          throw await httpFailure(response, 'GET proposals');
        }
        const body = (await response.json()) as Record<string, unknown>;
        if (!Array.isArray(body.proposals)) {
          throw new Error('proposal route returned no proposals array');
        }
        if (!cancelled) {
          setRows(decodeProposals(body));
          setVotePath(typeof body.vote_path === 'string' ? body.vote_path : null);
          setFailure(null);
          setStatus('ok');
        }
      })
      .catch((error: unknown) => {
        if (!cancelled) {
          setRows([]);
          setFailure(failureText(error, 'Evolution proposals are unavailable.'));
          setStatus('error');
        }
      });
    return () => {
      cancelled = true;
      controller.abort();
    };
  }, [runId, reloadToken]);

  async function authorize(row: ProposalRow): Promise<void> {
    const proposalId = row.proposalKey.replace(/^improvement:/, '');
    if (runId === null || proposalId.length === 0) {
      return;
    }
    setAuthorizing(true);
    setActionError(null);
    try {
      const response = await fetchWithBoundTimeout(
        `${apiBaseUrl()}/api/agents/runs/${encodeURIComponent(runId)}/proposals/${encodeURIComponent(proposalId)}/authorize`,
        {
          label: 'POST proposal authorization',
          init: {
            method: 'POST',
            headers: proposalWriteHeaders(sessionToken),
          },
        },
      );
      if (!response.ok) {
        throw await httpFailure(response, 'POST proposal authorization');
      }
      window.dispatchEvent(new Event('hivemind:evolution-changed'));
      setReloadToken((value) => value + 1);
    } catch (error: unknown) {
      setActionError(failureText(error, 'The five-seat authorization was refused.'));
    } finally {
      setAuthorizing(false);
    }
  }

  async function applyOrRollback(row: ProposalRow, action: 'apply' | 'rollback'): Promise<void> {
    const proposalId = row.proposalKey.replace(/^improvement:/, '');
    if (runId === null || proposalId.length === 0) {
      return;
    }
    setAuthorizing(true);
    setActionError(null);
    try {
      const response = await fetchWithBoundTimeout(
        `${apiBaseUrl()}/api/agents/runs/${encodeURIComponent(runId)}/proposals/${encodeURIComponent(proposalId)}/${action}`,
        {
          label: `POST proposal ${action}`,
          init: {
            method: 'POST',
            headers: proposalWriteHeaders(sessionToken),
          },
        },
      );
      if (!response.ok) {
        throw await httpFailure(response, `POST proposal ${action}`);
      }
      window.dispatchEvent(new Event('hivemind:evolution-changed'));
      setReloadToken((value) => value + 1);
    } catch (error: unknown) {
      setActionError(failureText(error, `The ${action} request was refused.`));
    } finally {
      setAuthorizing(false);
    }
  }

  return (
    <section
      aria-label="Self-evolution proposals"
      className="surface-panel mt-3 rounded-[var(--radius-card)] p-3"
    >
      <div className="flex items-baseline justify-between gap-2">
        <h3 className="type-heading-h4 text-ink-900">Self-evolution proposals</h3>
        <span className="type-mono-meta text-meta">operator-gated · lesson only</span>
      </div>
      {status === 'idle' ? <EmptyState message="Select a run to read evolution proposals." /> : null}
      {status === 'loading' ? <LoadingState label="Loading proposals..." /> : null}
      {status === 'error' ? <ErrorState message={failure ?? 'Evolution proposals are unavailable.'} /> : null}
      {actionError === null ? null : (
        <p role="alert" className="type-mono-meta mt-1 text-signal">{actionError}</p>
      )}
      {status === 'ok' && rows.length === 0 ? (
        <EmptyState message="No self-evolution proposals recorded in this run." />
      ) : null}
      {rows.length > 0 ? (
        <ul className="mt-2 space-y-2">
          {rows.map((row) => (
            <li key={row.proposalKey} className="rounded-[var(--radius-card)] border border-hairline p-2">
              <div className="flex flex-wrap justify-between gap-2">
                <span className="type-label text-ink-900">{row.area}</span>
                <span className="type-mono-meta text-meta">
                  {row.voteState ?? row.governance.state ?? 'state unknown'}
                </span>
              </div>
              <p className="type-mono-meta mt-1 text-ink-700">{row.problem}</p>
              <p className="type-body-sm mt-1 text-ink-900">{row.proposal}</p>
              <div className="type-mono-meta mt-2 flex flex-wrap gap-x-3 gap-y-1 text-meta">
                <span>applied: {row.applied ? 'yes' : 'no'}</span>
                <span>
                  superagent votes: {row.currentApprovals} approve / {row.currentRejections} reject
                </span>
                <span>
                  can_apply:{' '}
                  {row.governance.canApply === null
                    ? 'unknown'
                    : row.governance.canApply
                      ? 'yes'
                      : 'no'}
                </span>
                <span>
                  votes required: {row.governance.requiredApprovals ?? 'unknown'} (operator{' '}
                  {row.governance.operatorApprovals ?? 'unknown'} + superagents{' '}
                  {row.governance.superagentApprovals ?? 'unknown'}, unanimous{' '}
                  {String(row.governance.unanimous)})
                </span>
              </div>
              {canAuthorizeProposal(row) ? (
                <button
                  type="button"
                  className="type-label mt-2 rounded-[var(--radius-button)] bg-honey-300 px-3 py-1.5 text-ink-900 disabled:opacity-50"
                  disabled={authorizing}
                  onClick={() => {
                    void authorize(row);
                  }}
                  title="Requires four distinct SuperAgent approvals; records operator authorization only, never applies code"
                >
                  Authorize with operator (5th seat)
                </button>
              ) : null}
              {canApplyLesson(row) ? (
                <button
                  type="button"
                  className="type-label mt-2 ml-2 rounded-[var(--radius-button)] border border-hairline px-3 py-1.5 text-ink-800 disabled:opacity-50"
                  disabled={authorizing}
                  onClick={() => {
                    void applyOrRollback(row, 'apply');
                  }}
                  title="Applies a durable lesson note only; it never changes code, prompts, tool grants, personas, or gates"
                >
                  Apply authorized lesson
                </button>
              ) : null}
              {canRollbackLesson(row) ? (
                <button
                  type="button"
                  className="type-label mt-2 ml-2 rounded-[var(--radius-button)] border border-signal px-3 py-1.5 text-signal disabled:opacity-50"
                  disabled={authorizing}
                  onClick={() => {
                    void applyOrRollback(row, 'rollback');
                  }}
                >
                  Roll back lesson
                </button>
              ) : null}
            </li>
          ))}
        </ul>
      ) : null}
      <p className="type-mono-meta mt-2 text-meta">
        {votePath === null
          ? 'Vote path: unknown — Apply and rollback change only the lesson ledger, never code.'
          : `Vote path: ${votePath} — Apply and rollback change only the lesson ledger, never code.`}
      </p>
    </section>
  );
}
