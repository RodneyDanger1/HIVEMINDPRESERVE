import { createElement } from 'react';
import { renderToStaticMarkup } from 'react-dom/server';
import { describe, expect, it } from 'vitest';

import type { UiApi } from '../src/api/client';
import { MAX_ARGS_PREVIEW_CHARS } from '../src/api/contract';
import { ApprovalOverlay } from '../src/panels/ApprovalOverlay';
import {
  APPROVAL_PREVIEW_STRINGS,
  ApprovalPreview,
  parseArgsPreview,
  pathsInPatch,
} from '../src/panels/ApprovalPreview';
import { UiStoreContext } from '../src/state/context';
import { createUiStore, type ApprovalRecord, type UiState } from '../src/state/store';

/**
 * Approval args preview rendering. The wire preview is display-only, capped at
 * 512 UTF-16 code units, and may be cut mid-JSON. These tests pin:
 * - defensive parsing (missing / complete / truncated / malformed, never throws);
 * - apply_patch renders path(s) plus scrolling patch text and claims no application;
 * - file_write renders a path plus a bounded content preview;
 * - other tools render key/value rows, not the raw JSON wall;
 * - a missing preview stays visibly unknown and the approve/deny policy is untouched.
 */

const SESSION = { token: 'test-token', schema: 'hiverouter.v1', last_seq: 0, ws: '/ws' };

function renderPreview(tool: string, argsPreview: string | null | undefined): string {
  return renderToStaticMarkup(createElement(ApprovalPreview, { tool, argsPreview }));
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

function renderOverlay(state: Partial<UiState>): string {
  const store = createUiStore(fakeApi());
  store.setState({ connection: 'open' as const, ...state });
  (store as unknown as { getInitialState: () => UiState }).getInitialState = () =>
    store.getState();
  return renderToStaticMarkup(
    createElement(UiStoreContext.Provider, { value: store }, createElement(ApprovalOverlay)),
  );
}

function approval(overrides: Partial<ApprovalRecord> = {}): ApprovalRecord {
  return {
    approvalId: '01a1b175-0cbe-7d25-a25b-7e3c76dab55a',
    tool: 'file_write',
    nonce: 'nonce-1',
    decision: null,
    requestedAt: 0,
    runId: 'run-1',
    taskId: 'task-1',
    toolCallId: 'call-preview-1',
    argsSha256: 'c'.repeat(64),
    expiresAt: null,
    ...overrides,
  };
}

const TRUNCATED_FILE_WRITE =
  '{"path":"src/a.ts","content":"' + 'x'.repeat(470) + '…';

describe('parseArgsPreview (defensive, display-only)', () => {
  it('reports a missing preview instead of inventing one', () => {
    expect(parseArgsPreview(null).status).toBe('missing');
    expect(parseArgsPreview(undefined).status).toBe('missing');
    expect(parseArgsPreview('   ').status).toBe('missing');
  });

  it('parses complete JSON and keeps the parsed value', () => {
    const parsed = parseArgsPreview('{"path":"web/src/App.tsx"}');
    expect(parsed.status).toBe('parsed');
    expect(parsed.value).toEqual({ path: 'web/src/App.tsx' });
  });

  it('distinguishes a cut/truncated preview from malformed JSON without throwing', () => {
    // The server appends U+2026 when it cuts the bounded preview.
    expect(parseArgsPreview(TRUNCATED_FILE_WRITE).status).toBe('truncated');
    // An unterminated string is structurally incomplete, not merely malformed.
    expect(parseArgsPreview('{"path":"a","content":"abc').status).toBe('truncated');
    // Balanced-but-invalid JSON is malformed: the editor did not cut it.
    expect(parseArgsPreview('{"path": }').status).toBe('malformed');
    expect(parseArgsPreview('not json').status).toBe('malformed');
    expect(() => parseArgsPreview('{"unterminated":"')).not.toThrow();
  });
});

describe('ApprovalPreview missing/unparseable states', () => {
  it('shows "preview unavailable" for a missing preview, never a fabricated value', () => {
    for (const value of [null, undefined, '']) {
      const html = renderPreview('file_write', value);
      expect(html).toContain(APPROVAL_PREVIEW_STRINGS.unavailable);
      expect(html).not.toContain('data-testid="approval-preview-raw"');
      expect(html).not.toContain('data-testid="approval-preview-file-write"');
    }
  });

  it('marks a cut preview as truncated and shows only the raw bounded bytes', () => {
    const html = renderPreview('file_write', TRUNCATED_FILE_WRITE);
    expect(html).toContain(APPROVAL_PREVIEW_STRINGS.truncated);
    expect(html).toContain('data-testid="approval-preview-raw"');
    expect(html).toContain('src/a.ts');
    expect(html).not.toContain('data-testid="approval-preview-file-write"');
  });

  it('marks malformed JSON and renders the raw bytes unmodified', () => {
    const html = renderPreview('file_write', '{"path": }');
    expect(html).toContain(APPROVAL_PREVIEW_STRINGS.malformed);
    expect(html).toContain('data-testid="approval-preview-raw"');
  });
});

describe('apply_patch preview', () => {
  it('renders path(s) and patch text in a scrolling monospace pre without claiming application', () => {
    const patch = '@@ -1,2 +1,2 @@\n-old line\n+new line\n+added <tag>';
    const html = renderPreview(
      'apply_patch',
      JSON.stringify({
        path: 'web/src/panels/Example.tsx',
        patch,
        expected_sha256: 'a'.repeat(64),
      }),
    );

    expect(html).toContain('data-testid="approval-preview-apply-patch"');
    expect(html).toContain('web/src/panels/Example.tsx');
    expect(html).toContain('data-testid="approval-patch-text"');
    expect(html).toMatch(/<pre[^>]*data-testid="approval-patch-text"/);
    expect(html).toContain('overflow-auto');
    expect(html).toContain('max-h-56');
    expect(html).toContain('@@ -1,2 +1,2 @@');
    expect(html).toContain('+new line');
    expect(html).toContain('&lt;tag&gt;');
    expect(html).toContain(APPROVAL_PREVIEW_STRINGS.patchNote);
    expect(html).toContain('expected_sha256');
    // The note may say "not applied/verified"; it must never claim success.
    expect(html).not.toMatch(/hunks applied|applied successfully|line numbers verified/i);
  });

  it('derives multiple paths from patch headers when the args carry no path', () => {
    const patch =
      '*** Begin Patch\n*** Update File: src/a.ts\n@@ -1 +1 @@\n-a\n+b\n' +
      '*** Update File: src/b.ts\n@@ -1 +1 @@\n-c\n+d\n*** End Patch';
    const html = renderPreview('apply_patch', JSON.stringify({ patch }));

    expect(html).toContain('src/a.ts');
    expect(html).toContain('src/b.ts');
    expect(html).toContain(APPROVAL_PREVIEW_STRINGS.pathsLabel);
    expect(pathsInPatch('--- a/src/one.ts\n+++ b/src/one.ts')).toEqual(['src/one.ts']);
    expect(pathsInPatch('--- a/src/gone.ts\n+++ /dev/null')).toEqual(['src/gone.ts']);
    expect(pathsInPatch('diff --git a/src/two.ts b/src/two.ts')).toEqual(['src/two.ts']);
    // An added diff line that starts with "++" is not a file header.
    expect(pathsInPatch('+++ b/not-a-header')).toEqual([]);
  });

  it('keeps a missing patch visibly unavailable instead of faking one', () => {
    const html = renderPreview('apply_patch', JSON.stringify({ path: 'src/a.ts' }));
    expect(html).toContain(APPROVAL_PREVIEW_STRINGS.patchUnavailable);
    expect(html).not.toContain('data-testid="approval-patch-text"');
  });

  it('keeps a missing path visibly unavailable', () => {
    const html = renderPreview(
      'apply_patch',
      JSON.stringify({ patch: '@@ -1 +1 @@\n-a\n+b' }),
    );
    expect(html).toContain(APPROVAL_PREVIEW_STRINGS.pathUnavailable);
  });
});

describe('file_write preview', () => {
  it('renders the path and a bounded content preview', () => {
    const content = 'export const x = 1;\nconsole.log("hi <there>");';
    const html = renderPreview(
      'file_write',
      JSON.stringify({
        path: 'web/src/example.ts',
        content,
        expected_sha256: 'b'.repeat(64),
      }),
    );

    expect(html).toContain('data-testid="approval-preview-file-write"');
    expect(html).toContain('web/src/example.ts');
    expect(html).toContain('data-testid="approval-file-write-content"');
    expect(html).toContain('export const x = 1;');
    expect(html).toContain('&lt;there&gt;');
    expect(html).toContain(APPROVAL_PREVIEW_STRINGS.contentNote);
    expect(html).toContain('expected_sha256');
  });

  it('caps oversized content at the display bound and says so', () => {
    const long = 'c'.repeat(2_000);
    const html = renderPreview(
      'file_write',
      JSON.stringify({ path: 'note.txt', content: long }),
    );

    expect(html).toContain(APPROVAL_PREVIEW_STRINGS.displayTruncated);
    const pre =
      html.match(
        /data-testid="approval-file-write-content"[^>]*>([\s\S]*?)<\/pre>/,
      )?.[1] ?? '';
    expect(pre.length).toBeLessThanOrEqual(MAX_ARGS_PREVIEW_CHARS);
  });

  it('keeps missing content visibly unavailable', () => {
    const html = renderPreview('file_write', JSON.stringify({ path: 'note.txt' }));
    expect(html).toContain(APPROVAL_PREVIEW_STRINGS.contentUnavailable);
    expect(html).not.toContain('data-testid="approval-file-write-content"');
  });
});

describe('generic tool preview', () => {
  it('renders readable key/value rows instead of one raw JSON block', () => {
    const html = renderPreview(
      'web_fetch',
      JSON.stringify({
        url: 'https://example.com/path',
        method: 'GET',
        headers: { accept: 'text/html' },
        retries: 2,
      }),
    );

    expect(html).toContain('data-testid="approval-preview-fields"');
    expect(html).toContain('>url</dt>');
    expect(html).toContain('https://example.com/path');
    expect(html).toContain('>method</dt>');
    expect(html).toContain('GET');
    expect(html).toContain('>headers</dt>');
    expect(html).toContain('>retries</dt>');
    expect(html).not.toContain('data-testid="approval-preview-raw"');
    expect(html).not.toContain('>{"url"');
  });

  it('renders array arguments as a compact key/value row', () => {
    const html = renderPreview(
      'shell_exec',
      JSON.stringify({ program: 'git', args: ['status', '--short'] }),
    );
    expect(html).toContain('>program</dt>');
    expect(html).toContain('>args</dt>');
    expect(html).toContain('[&quot;status&quot;,&quot;--short&quot;]');
  });

  it('states an empty parsed preview instead of rendering a blank block', () => {
    const html = renderPreview('file_read', '{}');
    expect(html).toContain(APPROVAL_PREVIEW_STRINGS.empty);
    expect(html).not.toContain('data-testid="approval-preview-raw"');
  });
});

describe('ApprovalOverlay integration', () => {
  it('renders the structured preview inside the dialog and keeps both decisions enabled', () => {
    const html = renderOverlay({
      approvals: [
        approval({
          argsPreview: JSON.stringify({
            path: 'web/src/a.ts',
            content: 'hello',
          }),
        }),
      ],
    });

    expect(html).toContain('data-testid="approval-preview-file-write"');
    expect(html).toContain('web/src/a.ts');
    expect(html).toContain('Approve tool call for file_write');
    expect(html).toContain('Deny tool call for file_write');
    expect(html).not.toContain('disabled=""');
  });

  it('keeps the missing-preview state visible without changing approve/deny policy', () => {
    const html = renderOverlay({
      approvals: [approval({ argsPreview: null })],
    });

    expect(html).toContain(APPROVAL_PREVIEW_STRINGS.unavailable);
    expect(html).toContain('Approve tool call for file_write');
    expect(html).toContain('Deny tool call for file_write');
    expect(html).not.toContain('disabled=""');
  });
});
