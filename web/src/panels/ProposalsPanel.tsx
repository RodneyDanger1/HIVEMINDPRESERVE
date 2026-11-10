/**
 * Read-only self-evolution proposal projection. Proposals are recorded by the
 * root tool; this panel never votes, applies, or mutates them. The server's
 * `vote_path` and `applied` fields are rendered verbatim so a proposal is never
 * presented as adopted when it is only awaiting authorization.
 */
import { useEffect, useState } from 'react';
import { apiBaseUrl } from '../api/client';
import { failureText, httpFailure } from '../api/failure';
import { useUiStore } from '../state/context';
import { EmptyState } from './StatePack';

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

export function ProposalsPanel() {
  const runId = useUiStore((state) => state.run?.id ?? null);
  const [status, setStatus] = useState<Status>('idle');
  const [rows, setRows] = useState<readonly ProposalRow[]>([]);
  const [votePath, setVotePath] = useState<string | null>(null);
  const [failure, setFailure] = useState<string | null>(null);

  useEffect(() => {
    if (runId === null) {
      setRows([]);
      setStatus('idle');
      return;
    }
    let cancelled = false;
    setStatus('loading');
    fetch(`${apiBaseUrl()}/api/agents/runs/${encodeURIComponent(runId)}/proposals`, {
      headers: { Accept: 'application/json' },
    })
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
    };
  }, [runId]);

  return (
    <section
      aria-label="Self-evolution proposals"
      className="surface-panel mt-3 rounded-[var(--radius-card)] p-3"
    >
      <div className="flex items-baseline justify-between gap-2">
        <h3 className="type-heading-h4 text-ink-900">Self-evolution proposals</h3>
        <span className="type-mono-meta text-meta">read-only</span>
      </div>
      {status === 'loading' ? <p className="type-mono-meta mt-1 text-meta">Loading proposals…</p> : null}
      {status === 'error' ? <p className="type-mono-meta mt-1 text-signal">{failure}</p> : null}
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
                  {row.governance.state ?? 'state unknown'}
                </span>
              </div>
              <p className="type-mono-meta mt-1 text-ink-700">{row.problem}</p>
              <p className="type-body-sm mt-1 text-ink-900">{row.proposal}</p>
              <div className="type-mono-meta mt-2 flex flex-wrap gap-x-3 gap-y-1 text-meta">
                <span>applied: {String(row.applied)}</span>
                <span>can_apply: {String(row.governance.canApply)}</span>
                <span>
                  votes required: {row.governance.requiredApprovals ?? 'unknown'} (operator{' '}
                  {row.governance.operatorApprovals ?? 'unknown'} + superagents{' '}
                  {row.governance.superagentApprovals ?? 'unknown'}, unanimous{' '}
                  {String(row.governance.unanimous)})
                </span>
              </div>
            </li>
          ))}
        </ul>
      ) : null}
      <p className="type-mono-meta mt-2 text-meta">
        {votePath === null
          ? 'Vote path: unknown — this panel never applies a proposal.'
          : `Vote path: ${votePath} — this panel never applies a proposal.`}
      </p>
    </section>
  );
}
