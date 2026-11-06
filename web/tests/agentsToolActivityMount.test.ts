import { createElement, type ReactElement } from 'react';
import { renderToStaticMarkup } from 'react-dom/server';
import { describe, expect, it } from 'vitest';

import type { UiApi } from '../src/api/client';
import { AgentsPanel } from '../src/panels/AgentsPanel';
import { ApprovalOverlay } from '../src/panels/ApprovalOverlay';
import { UiStoreContext } from '../src/state/context';
import {
  createUiStore,
  type ApprovalRecord,
  type ToolActivityRecord,
  type UiState,
} from '../src/state/store';
import { STRINGS } from '../src/state/strings';

/**
 * W12 wave-3 mount + copy reconciliation tests (source-only; node environment,
 * static render). They pin:
 * - the agents surface mounts the tool-activity region from store `toolActivity`;
 * - `ToolActivityPanel`/`ApprovalOverlay` resolve all copy from STRINGS
 *   (heading/empty/list/section/outcome/field/TTL labels) with no hardcoded copy;
 * - the approval card shows args hash + TTL + identity and keeps its honest
 *   empty (nothing rendered) / unknown / expired states;
 * - the exact selectors the e2e specs rely on stay intact.
 */

const SESSION = { token: 'test-token', schema: 'hiverouter.v1', last_seq: 0, ws: '/ws' };

const SOURCE_FILES = import.meta.glob('../src/**/*.{ts,tsx}', {
  query: '?raw',
  import: 'default',
  eager: true,
}) as Record<string, string>;

function sourceOf(suffix: string): string {
  const key = Object.keys(SOURCE_FILES).find((candidate) => candidate.endsWith(suffix));
  return key ? SOURCE_FILES[key] : '';
}

function fakeApi(): UiApi {
  return {
    kind: 'ws',
    async open() {
      return SESSION;
    },
    close() {
      /* no session to close in a static render */
    },
    async send() {
      /* commands are not sent by static renders */
    },
    subscribe() {
      return () => undefined;
    },
    subscribeStatus() {
      return () => undefined;
    },
    subscribeClientError() {
      return () => undefined;
    },
  } as unknown as UiApi;
}

function renderWith(node: ReactElement, state: Partial<UiState> = {}): string {
  const store = createUiStore(fakeApi());
  if (Object.keys(state).length > 0) {
    store.setState(state);
  }
  // zustand's server snapshot is the initial state; point it at the mutated
  // store so the static render reflects the state this test just seeded.
  (store as unknown as { getInitialState: () => UiState }).getInitialState = () =>
    store.getState();
  return renderToStaticMarkup(
    createElement(UiStoreContext.Provider, { value: store }, node),
  );
}

const OK_ARGS = 'a'.repeat(64);
const PENDING_ARGS = 'b'.repeat(64);
const WRITE_ARGS = 'c'.repeat(64);

const READ_OK: ToolActivityRecord = {
  toolCallId: 'call_ok',
  tool: 'file_read',
  argsSha256: OK_ARGS,
  turn: 1,
  outcome: 'ok',
};

const SHELL_PENDING: ToolActivityRecord = {
  toolCallId: 'call_pending',
  tool: 'shell_exec',
  argsSha256: PENDING_ARGS,
  turn: 2,
  outcome: 'pending',
};

function approval(overrides: Partial<ApprovalRecord> = {}): ApprovalRecord {
  return {
    approvalId: '01a1b175-0cbe-7d25-a25b-7e3c76dab55a',
    tool: 'file_write',
    nonce: 'nonce-1',
    decision: null,
    requestedAt: 0,
    runId: 'run-1',
    taskId: 'task-1',
    toolCallId: 'call-e2e-1',
    argsSha256: WRITE_ARGS,
    expiresAt: null,
    ...overrides,
  };
}

describe('W12 string table (tool activity + approval)', () => {
  it('exposes heading/empty/pending/resolving and field/TTL copy', () => {
    expect(STRINGS.toolActivity.heading).toBe('Tool activity');
    expect(STRINGS.toolActivity.sectionAria).toBe('Tool activity');
    expect(STRINGS.toolActivity.listAria).toBe('Tool calls');
    expect(STRINGS.toolActivity.empty).toContain('No tool activity yet');
    // Single source for outcome copy: no duplicate root-level `pending` key.
    expect(Object.keys(STRINGS.toolActivity)).not.toContain('pending');
    expect(STRINGS.toolActivity.outcomes.pending).toBe('Pending');
    expect(STRINGS.toolActivity.outcomes.ok).toBe('OK');
    expect(STRINGS.toolActivity.outcomes.timeout).toBe('Timed out');
    expect(STRINGS.toolActivity.outcomes.unknown).toBe('Unknown');
    expect(STRINGS.toolActivity.labels.argsSha256).toBe('args_sha256');
    expect(STRINGS.toolActivity.labels.toolCallId).toBe('tool_call_id');
    expect(STRINGS.approval.resolving).toContain('Resolving');
    expect(STRINGS.approval.resolveHint).toBe('APPROVAL_RESOLVE');
    expect(STRINGS.approval.fields.argsSha256).toBe('args_sha256');
    expect(STRINGS.approval.fields.runId).toBe('run_id');
    expect(STRINGS.approval.fields.taskId).toBe('task_id');
    expect(STRINGS.approval.fields.toolCallId).toBe('tool_call_id');
    expect(STRINGS.approval.fields.approvalId).toBe('approval_id');
    expect(STRINGS.approval.fields.expires).toBe('expires');
    expect(STRINGS.approval.expiry.unknown).toBe('unknown');
    expect(STRINGS.approval.expiry.expired).toBe('expired');
    expect(STRINGS.approval.expiry.expiresIn).toBe('expires in');
  });
});

describe('AgentsPanel mounts the tool-activity region (W12)', () => {
  it('renders store toolActivity through ToolActivityPanel inside a labelled region', () => {
    const markup = renderWith(createElement(AgentsPanel), {
      toolActivity: [READ_OK, SHELL_PENDING],
    });

    expect(markup).toContain('data-testid="agents-tool-activity"');
    expect(markup).toContain(`aria-label="${STRINGS.toolActivity.regionAria}"`);
    // Exact e2e selectors stay: section[aria-label], h2 "Tool activity · N", ul aria.
    expect(markup).toContain(`aria-label="${STRINGS.toolActivity.sectionAria}"`);
    expect(markup).toContain(`${STRINGS.toolActivity.heading} · 2`);
    expect(markup).toContain(`aria-label="${STRINGS.toolActivity.listAria}"`);
    expect(markup).toContain('aria-live="polite"');
    // Rows are server-data only, with outcome-aware accessible names + evidence labels.
    expect(markup).toContain('file_read');
    expect(markup).toContain('file_read tool call call_ok, OK');
    expect(markup).toContain('shell_exec tool call call_pending, Pending');
    expect(markup).toContain(STRINGS.toolActivity.labels.argsSha256);
    expect(markup).toContain(`${STRINGS.toolActivity.labels.turn} 1`);
    expect(markup).toContain(`${STRINGS.toolActivity.labels.toolCallId} call_ok`);
  });

  it('renders the honest empty state from STRINGS and no list', () => {
    const markup = renderWith(createElement(AgentsPanel), { toolActivity: [] });

    expect(markup).toContain('data-testid="agents-tool-activity"');
    expect(markup).toContain(STRINGS.panels.agentsEmpty);
    expect(markup).toContain(STRINGS.toolActivity.empty);
    expect(markup).not.toContain(`aria-label="${STRINGS.toolActivity.listAria}"`);
  });
});

describe('ApprovalOverlay renders from store approvals with STRINGS copy (W12)', () => {
  it('renders args hash, identity, TTL and accessible decisions for a pending approval', () => {
    const markup = renderWith(createElement(ApprovalOverlay), { approvals: [approval()] });

    expect(markup).toContain('role="dialog"');
    expect(markup).toContain('aria-modal="true"');
    expect(markup).toContain(`aria-label="${STRINGS.approval.title}: file_write"`);
    for (const label of [
      STRINGS.approval.fields.argsSha256,
      STRINGS.approval.fields.runId,
      STRINGS.approval.fields.taskId,
      STRINGS.approval.fields.toolCallId,
      STRINGS.approval.fields.approvalId,
      STRINGS.approval.fields.expires,
    ]) {
      expect(markup).toContain(`>${label}</dt>`);
    }
    expect(markup).toContain(WRITE_ARGS.slice(0, 12));
    // Full values live in title attributes (no truncation of the evidence).
    expect(markup).toContain(`title="${WRITE_ARGS}"`);
    expect(markup).toContain('title="run-1"');
    expect(markup).toContain('title="task-1"');
    expect(markup).toContain('title="call-e2e-1"');
    expect(markup).toContain('Approve tool call for file_write');
    expect(markup).toContain('Deny tool call for file_write');
    // Nonce present: the resolve hint, not the missing-nonce gap.
    expect(markup).toContain(
      `${STRINGS.approval.title}: ${STRINGS.approval.resolveHint} · file_write`,
    );
    expect(markup).not.toContain('did not carry the nonce');
  });

  it('never invents a time: null expiry renders unknown, past renders expired', () => {
    const unknown = renderWith(createElement(ApprovalOverlay), { approvals: [approval()] });
    expect(unknown).toContain(`>${STRINGS.approval.expiry.unknown}</dd>`);

    const expired = renderWith(createElement(ApprovalOverlay), {
      approvals: [approval({ expiresAt: '2000-01-01T00:00:00.000Z' })],
    });
    expect(expired).toContain(`>${STRINGS.approval.expiry.expired}</dd>`);

    const future = renderWith(createElement(ApprovalOverlay), {
      approvals: [approval({ expiresAt: '2099-01-01T00:00:00.000Z' })],
    });
    expect(future).toMatch(
      new RegExp(`>${STRINGS.approval.expiry.expiresIn} [^<]+</dd>`),
    );
  });

  it('keeps the honest missing-nonce state and renders nothing when no approval is pending', () => {
    const missingNonce = renderWith(createElement(ApprovalOverlay), {
      approvals: [approval({ nonce: null })],
    });
    expect(missingNonce).toContain(STRINGS.approval.nonceMissing);
    expect(missingNonce).toContain('disabled=""');

    expect(renderWith(createElement(ApprovalOverlay), { approvals: [] })).toBe('');
  });
});

describe('mount uniqueness and disabled-state contract (W12 wave 4)', () => {
  const countOf = (source: string, needle: string): number => source.split(needle).length - 1;
  const disabledCount = (markup: string): number => (markup.match(/disabled=""/g) ?? []).length;

  it('mounts each surface exactly once and never from App', () => {
    const appSource = sourceOf('/App.tsx');
    const agentsSource = sourceOf('/panels/AgentsPanel.tsx');
    expect(countOf(appSource, '<ToolActivityPanel')).toBe(0);
    expect(countOf(appSource, '<ApprovalOverlay')).toBe(1);
    expect(countOf(agentsSource, '<ToolActivityPanel')).toBe(1);
    expect(countOf(agentsSource, '<ApprovalOverlay')).toBe(0);
    expect(countOf(agentsSource, 'data-testid="agents-tool-activity"')).toBe(1);
  });

  it('drives both action buttons from nonce, expiry and resolving state', () => {
    // Pending, non-expired, nonce present: both actions enabled.
    const pending = renderWith(createElement(ApprovalOverlay), { approvals: [approval()] });
    expect(disabledCount(pending)).toBe(0);

    // Expired: read-only — both actions disabled.
    const expired = renderWith(createElement(ApprovalOverlay), {
      approvals: [approval({ expiresAt: '2000-01-01T00:00:00.000Z' })],
    });
    expect(disabledCount(expired)).toBe(2);

    // Missing nonce: both actions disabled + the named gap copy.
    const nonceLess = renderWith(createElement(ApprovalOverlay), {
      approvals: [approval({ nonce: null })],
    });
    expect(disabledCount(nonceLess)).toBe(2);
    expect(nonceLess).toContain(STRINGS.approval.nonceMissing);

    // Unparseable TTL is 'unknown', not expired: actions stay enabled.
    const unknown = renderWith(createElement(ApprovalOverlay), {
      approvals: [approval({ expiresAt: 'not-a-date' })],
    });
    expect(unknown).toContain(`>${STRINGS.approval.expiry.unknown}</dd>`);
    expect(disabledCount(unknown)).toBe(0);
  });

  it('pins the exact e2e selector surface (host, section, list, row)', () => {
    const markup = renderWith(createElement(AgentsPanel), { toolActivity: [READ_OK] });
    expect(markup).toMatch(/<div[^>]*role="region"[^>]*data-testid="agents-tool-activity"/);
    expect(markup).toContain(`<section aria-label="${STRINGS.toolActivity.sectionAria}"`);
    expect(markup).toContain(`<ul aria-label="${STRINGS.toolActivity.listAria}" aria-live="polite"`);
    expect(markup).toContain('data-outcome="ok"');
    expect(markup).toContain('aria-label="file_read tool call call_ok, OK"');
  });

  it('wires the resolving copy to an aria-live status line', () => {
    const source = sourceOf('/panels/ApprovalOverlay.tsx');
    expect(source).toContain('STRINGS.approval.resolving');
    expect(source).toMatch(/role="status"[\s\S]*?STRINGS\.approval\.resolving/);
  });
});

describe('copy reconciliation: panels consume STRINGS, no hardcoded copy (W12)', () => {
  it('ToolActivityPanel uses STRINGS for heading/empty/section/list/outcome copy', () => {
    const source = sourceOf('/panels/ToolActivityPanel.tsx');
    expect(source).toContain('STRINGS.toolActivity.heading');
    expect(source).toContain('STRINGS.toolActivity.empty');
    expect(source).toContain('STRINGS.toolActivity.sectionAria');
    expect(source).toContain('STRINGS.toolActivity.listAria');
    expect(source).toContain('STRINGS.toolActivity.outcomes');
    expect(source).toContain('STRINGS.toolActivity.labels');
    // Hardcoded copy that the string table replaced must not come back as JSX copy.
    expect(source).not.toMatch(/<section[^>]*aria-label="Tool activity"/);
    expect(source).not.toContain('No tool activity yet');
  });

  it('ApprovalOverlay uses STRINGS for field/TTL/resolving copy', () => {
    const source = sourceOf('/panels/ApprovalOverlay.tsx');
    expect(source).toContain('STRINGS.approval.fields');
    expect(source).toContain('STRINGS.approval.expiry');
    expect(source).toContain('STRINGS.approval.resolving');
    expect(source).toContain('STRINGS.approval.resolveHint');
    expect(source).not.toContain('Resolving approval…');
    expect(source).not.toMatch(/<dt[^>]*>args_sha256<\/dt>/);
  });
});
