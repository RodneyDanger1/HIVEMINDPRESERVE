import { describe, expect, it } from 'vitest';
import { canAuthorizeProposal, decodeProposals } from '../src/panels/ProposalsPanel';

describe('self-evolution proposal projection', () => {
  it('decodes a recorded proposal and preserves its governance state', () => {
    const rows = decodeProposals({
      vote_path: 'not_implemented',
      proposals: [
        {
          proposal_key: 'improvement:abc',
          area: 'governance',
          problem: 'SuperAgent votes are not persisted',
          proposal: 'Add four distinct SuperAgent votes plus operator approval',
          applied: false,
          authorized: false,
          current_superagent_approvals: 4,
          current_superagent_rejections: 0,
          vote_state: 'awaiting_operator',
          governance: {
            required_approvals: 5,
            superagent_approvals: 4,
            operator_approvals: 1,
            unanimous: true,
            state: 'awaiting_votes',
            can_apply: false,
          },
        },
      ],
    });
    expect(rows).toHaveLength(1);
    expect(rows[0].proposalKey).toBe('improvement:abc');
    expect(rows[0].applied).toBe(false);
    expect(rows[0].governance.state).toBe('awaiting_votes');
    expect(rows[0].governance.requiredApprovals).toBe(5);
    expect(rows[0].governance.superagentApprovals).toBe(4);
    expect(rows[0].governance.canApply).toBe(false);
    expect(canAuthorizeProposal(rows[0])).toBe(true);
  });

  it('rejects malformed proposal rows instead of inventing values', () => {
    expect(decodeProposals(null)).toEqual([]);
    expect(decodeProposals({ proposals: 'nope' })).toEqual([]);
    expect(
      decodeProposals({
        proposals: [{ proposal_key: 'improvement:x', area: 'a' }],
      }),
    ).toEqual([]);
  });
});
