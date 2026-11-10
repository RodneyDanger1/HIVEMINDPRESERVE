import { describe, expect, it } from 'vitest';
import {
  APPROVAL_DECISIONS,
  approvalActionAriaLabel,
  approvalAriaLabel,
  canResolveApproval,
  formatArgsSha256,
  networkApprovalTargetKnown,
  formatExpiry,
  truncateId,
} from '../src/panels/ApprovalOverlay';
import {
  TOOL_ACTIVITY_OUTCOMES,
  formatActivityArgsSha256,
  isToolActivityOutcome,
  sortToolActivity,
  toolActivityAriaLabel,
  toolActivityOutcomeLabel,
  toolActivityOutcomeTone,
  toolActivitySummary,
  truncateActivityValue,
  type ToolActivityEntry,
} from '../src/panels/ToolActivityPanel';

const NOW = Date.parse('2026-11-06T23:00:00.000Z');

function entry(overrides: Partial<ToolActivityEntry> & Pick<ToolActivityEntry, 'toolCallId'>): ToolActivityEntry {
  return {
    tool: 'file_read',
    argsSha256: 'a'.repeat(64),
    turn: 1,
    outcome: 'pending',
    ...overrides,
  };
}

describe('ApprovalOverlay view model (W12)', () => {
  it('truncates ids/hashes and never invents a value for missing data', () => {
    expect(truncateId('0123456789abcdef')).toBe('0123456789ab…');
    expect(truncateId('short')).toBe('short');
    expect(truncateId(null)).toBe('—');
    expect(truncateId(undefined)).toBe('—');
    expect(truncateId('')).toBe('—');
    expect(formatArgsSha256('f'.repeat(64))).toBe('ffffffffffff…');
    expect(formatArgsSha256(null)).toBe('—');
  });

  it('renders deterministic expiry countdown without fabricating a time', () => {
    expect(formatExpiry(undefined, NOW)).toBe('unknown');
    expect(formatExpiry(null, NOW)).toBe('unknown');
    expect(formatExpiry('not-a-date', NOW)).toBe('unknown');
    expect(formatExpiry(new Date(NOW - 1_000).toISOString(), NOW)).toBe('expired');
    expect(formatExpiry(new Date(NOW + 42_000).toISOString(), NOW)).toBe('42s');
    expect(formatExpiry(new Date(NOW + 65_000).toISOString(), NOW)).toBe('1m 05s');
    expect(formatExpiry(new Date(NOW + 3_725_000).toISOString(), NOW)).toBe('1h 02m');
  });

  it('disables approve and deny while resolving or without a nonce (fail-closed)', () => {
    expect(canResolveApproval({ nonce: 'n-1', resolving: false })).toBe(true);
    expect(canResolveApproval({ nonce: 'n-1', resolving: true })).toBe(false);
    expect(canResolveApproval({ nonce: null, resolving: false })).toBe(false);
    expect(canResolveApproval({ nonce: null, resolving: true })).toBe(false);
  });

  it('uses approve/deny semantics with tool context in accessible names', () => {
    expect(APPROVAL_DECISIONS).toEqual({ approve: 'approve', deny: 'deny' });
    expect(approvalAriaLabel('file_write')).toContain('file_write');
    expect(approvalActionAriaLabel('approve', 'file_write')).toMatch(/approve.*file_write/i);
    expect(approvalActionAriaLabel('deny', 'file_write')).toMatch(/deny.*file_write/i);
  });
  it('renders web_fetch through the generic approval and activity model', () => {
    expect(approvalAriaLabel('web_fetch')).toContain('web_fetch');
    expect(approvalActionAriaLabel('approve', 'web_fetch')).toMatch(/web_fetch/i);
    const webEntry = entry({ tool: 'web_fetch', toolCallId: 'call-web-1', outcome: 'ok' });
    expect(toolActivityAriaLabel(webEntry)).toContain('web_fetch');
    expect(toolActivitySummary([webEntry])).toEqual({
      total: 1,
      counts: { pending: 0, ok: 1, refused: 0, denied: 0, timeout: 0, cancelled: 0 },
    });
  });

  it('fails closed when a replayed web_fetch approval has no target preview', () => {
    expect(networkApprovalTargetKnown('web_fetch', '{"url":"https://example.com"}')).toBe(true);
    expect(networkApprovalTargetKnown('web_fetch', null)).toBe(false);
    expect(networkApprovalTargetKnown('web_fetch', '')).toBe(false);
    expect(networkApprovalTargetKnown('file_write', null)).toBe(true);
  });
});

describe('ToolActivityPanel logic (W12)', () => {
  it('declares exactly the six required outcomes', () => {
    expect(TOOL_ACTIVITY_OUTCOMES).toEqual(['pending', 'ok', 'refused', 'denied', 'timeout', 'cancelled']);
  });

  it('maps every outcome to a distinct, non-empty label and tone', () => {
    const labels = TOOL_ACTIVITY_OUTCOMES.map((outcome) => toolActivityOutcomeLabel(outcome));
    for (const label of labels) {
      expect(label.length).toBeGreaterThan(0);
    }
    expect(new Set(labels).size).toBe(TOOL_ACTIVITY_OUTCOMES.length);
    for (const outcome of TOOL_ACTIVITY_OUTCOMES) {
      expect(toolActivityOutcomeTone(outcome)).toMatch(/border-/);
    }
    expect(toolActivityOutcomeLabel('not-an-outcome')).toBe('Unknown');
    expect(toolActivityOutcomeLabel('ok')).toBe('OK');
    expect(toolActivityOutcomeLabel('timeout')).toBe('Timed out');
  });

  it('rejects undeclared outcomes and reports an empty summary as all zeros', () => {
    expect(isToolActivityOutcome('ok')).toBe(true);
    expect(isToolActivityOutcome('pending')).toBe(true);
    expect(isToolActivityOutcome('bogus')).toBe(false);
    const summary = toolActivitySummary([]);
    expect(summary.total).toBe(0);
    for (const outcome of TOOL_ACTIVITY_OUTCOMES) {
      expect(summary.counts[outcome]).toBe(0);
    }
  });

  it('counts declared outcomes and ignores undeclared ones without fake rows', () => {
    const entries = [
      entry({ toolCallId: 'c1', outcome: 'pending' }),
      entry({ toolCallId: 'c2', outcome: 'ok' }),
      entry({ toolCallId: 'c3', outcome: 'refused' }),
      entry({ toolCallId: 'c4', outcome: 'denied' }),
      entry({ toolCallId: 'c5', outcome: 'timeout' }),
      entry({ toolCallId: 'c6', outcome: 'cancelled' }),
      entry({ toolCallId: 'c7', outcome: 'bogus' as never }),
    ];
    const summary = toolActivitySummary(entries);
    expect(summary.total).toBe(7);
    expect(summary.counts).toEqual({ pending: 1, ok: 1, refused: 1, denied: 1, timeout: 1, cancelled: 1 });
  });

  it('sorts deterministically by turn then tool_call_id without mutating input', () => {
    const input = [
      entry({ toolCallId: 'b', turn: 2 }),
      entry({ toolCallId: 'a', turn: 2 }),
      entry({ toolCallId: 'z', turn: 1 }),
    ];
    const sorted = sortToolActivity(input);
    expect(sorted.map((item) => item.toolCallId)).toEqual(['z', 'a', 'b']);
    expect(input.map((item) => item.toolCallId)).toEqual(['b', 'a', 'z']);
  });

  it('truncates ids/hashes and keeps accessible labels outcome-aware', () => {
    expect(formatActivityArgsSha256('f'.repeat(64))).toBe('ffffffffffff…');
    expect(formatActivityArgsSha256(undefined)).toBe('—');
    expect(truncateActivityValue(null)).toBe('—');
    const label = toolActivityAriaLabel(entry({ toolCallId: '0123456789abcdef', tool: 'file_list', outcome: 'refused' }));
    expect(label).toContain('file_list');
    expect(label).toContain('Refused');
    expect(label).toContain('0123456789ab…');
  });
});
