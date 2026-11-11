import { describe, expect, it, vi } from 'vitest';
import { createElement } from 'react';
import type { ReactElement } from 'react';
import { renderToStaticMarkup } from 'react-dom/server';
import {
  CommandPalette,
  CostBar,
  DiffView,
  Editor,
  FileTree,
  SessionRail,
  Terminal,
  Timeline,
  TodoPanel,
  WORKBENCH_DEPENDENCY_REQUIREMENTS,
  approvalBlocksSave,
  budgetProgress,
  buildEditorLineNumbers,
  buildFileTreeNodes,
  canSubmitTerminal,
  countEditorLines,
  countFileNodes,
  diffLineAriaLabel,
  diffLines,
  editorLineForOffset,
  editorShortcut,
  effectiveTotalTokens,
  filterPaletteActions,
  flattenFileTree,
  formatDiffHunkHeader,
  formatFileSize,
  formatSessionCost,
  formatSessionUpdated,
  formatTimelineTime,
  formatTokenCount,
  formatUsd,
  groupTimelineSteps,
  isTodoComplete,
  limitDiffHunks,
  orderSessions,
  paletteNavigationTarget,
  scorePaletteAction,
  sessionNavigationTarget,
  sessionStatusLabel,
  sortTimelineSteps,
  timelineAgents,
  timelineStatusLabel,
  timelineTabTarget,
  todoProgressLabel,
  todoStatusLabel,
  todoSummary,
  todoToggleTarget,
  treeNavigationTarget,
  type DiffLine,
  type FileTreeNode,
  type PaletteAction,
  type SessionSummary,
  type TimelineStep,
  type TodoItem,
} from '../src/workbench';

function render(node: ReactElement): string {
  return renderToStaticMarkup(node);
}

const noop = (): void => undefined;

describe('CAP-4 FileTree', () => {
  const nodes: readonly FileTreeNode[] = [
    {
      path: 'src',
      name: 'src',
      kind: 'dir',
      children: [
        { path: 'src/App.tsx', name: 'App.tsx', kind: 'file', sizeBytes: 2048 },
        {
          path: 'src/lib',
          name: 'lib',
          kind: 'dir',
          children: [{ path: 'src/lib/util.ts', name: 'util.ts', kind: 'file' }],
        },
      ],
    },
    { path: 'README.md', name: 'README.md', kind: 'file', sizeBytes: 12 },
  ];

  it('flattens only visible rows and preserves depth/parent attribution', () => {
    const collapsed = flattenFileTree(nodes, []);
    expect(collapsed.map((row) => row.node.path)).toEqual(['src', 'README.md']);
    expect(collapsed[0]?.depth).toBe(0);
    expect(collapsed[0]?.expanded).toBe(false);

    const expanded = flattenFileTree(nodes, ['src', 'src/lib']);
    expect(expanded.map((row) => row.node.path)).toEqual([
      'src',
      'src/App.tsx',
      'src/lib',
      'src/lib/util.ts',
      'README.md',
    ]);
    expect(expanded[3]?.depth).toBe(2);
    expect(expanded[3]?.parentPath).toBe('src/lib');
  });

  it('maps tree keys to navigation, toggle and activation intents', () => {
    const rows = flattenFileTree(nodes, ['src']);
    expect(treeNavigationTarget(rows, 0, 'ArrowDown')?.index).toBe(1);
    expect(treeNavigationTarget(rows, 1, 'ArrowUp')?.index).toBe(0);
    expect(treeNavigationTarget(rows, 1, 'Home')?.index).toBe(0);
    expect(treeNavigationTarget(rows, 0, 'End')?.index).toBe(rows.length - 1);
    expect(treeNavigationTarget(rows, 0, 'ArrowRight')?.index).toBe(1);
    expect(treeNavigationTarget(rows, 0, 'ArrowLeft')?.toggle).toBe(true);
    expect(treeNavigationTarget(rows, 1, 'ArrowLeft')?.index).toBe(0);
    expect(treeNavigationTarget(rows, 0, 'Enter')?.activate).toBe(true);
    expect(treeNavigationTarget(rows, 0, 'PageDown')).toBeNull();
  });

  it('converts the frozen flat contract without inventing size or parents', () => {
    const converted = buildFileTreeNodes([
      { path: 'a.ts', name: 'a.ts', kind: 'file', size_bytes: null },
      { path: 'lib', name: 'lib', kind: 'dir', size_bytes: 0 },
    ]);
    expect(converted[0]?.sizeBytes).toBeNull();
    expect(converted[1]?.sizeBytes).toBe(0);
    expect(converted[0]?.children).toBeUndefined();
  });

  it('formats real sizes and honest unknown sizes', () => {
    expect(formatFileSize(12)).toBe('12 B');
    expect(formatFileSize(2048)).toBe('2.0 KB');
    expect(formatFileSize(2 * 1024 * 1024)).toBe('2.0 MB');
    expect(formatFileSize(null)).toBe('—');
    expect(formatFileSize(-1)).toBe('—');
    expect(countFileNodes(nodes)).toBe(3);
  });

  it('renders loading, error and empty states without rows', () => {
    const idle = render(createElement(FileTree, { nodes, status: 'idle', onRefresh: noop }));
    expect(idle).toContain('File tree has not loaded yet.');

    const loading = render(createElement(FileTree, { nodes, status: 'loading', onRefresh: noop }));
    expect(loading).toContain('role="status"');
    expect(loading).toContain('Loading workspace files');
    expect(loading).not.toContain('README.md');

    const error = render(
      createElement(FileTree, {
        nodes,
        status: 'error',
        errorMessage: 'E_PATH_NOT_FOUND',
        onRefresh: noop,
      }),
    );
    expect(error).toContain('role="alert"');
    expect(error).toContain('E_PATH_NOT_FOUND');

    const empty = render(createElement(FileTree, { nodes: [], status: 'ready' }));
    expect(empty).toContain('No files in this folder.');
  });

  it('renders an ARIA tree with roving focus and selected/expanded state', () => {
    const markup = render(
      createElement(FileTree, {
        nodes,
        status: 'ready',
        expandedPaths: ['src'],
        selectedPath: 'src/App.tsx',
        onSelectFile: noop,
      }),
    );
    expect(markup).toContain('role="tree"');
    expect(markup).toContain('role="treeitem"');
    expect(markup).toContain('aria-expanded="true"');
    expect(markup).toContain('aria-selected="true"');
    expect(markup).toContain('tabindex="0"');
    expect(markup).toContain('tabindex="-1"');
  });

  it('can use a flat directory listing as navigation without local expansion', () => {
    const markup = render(
      createElement(FileTree, {
        nodes: buildFileTreeNodes([
          { path: 'src', name: 'src', kind: 'dir', size_bytes: null },
        ]),
        status: 'ready',
        expandOnToggle: false,
        onToggleDir: noop,
      }),
    );
    expect(markup).toContain('aria-expanded="false"');
  });
});

describe('CAP-4 Editor', () => {
  it('counts lines and resolves the cursor line from an offset', () => {
    expect(countEditorLines('')).toBe(1);
    expect(countEditorLines('a')).toBe(1);
    expect(countEditorLines('a\nb')).toBe(2);
    expect(countEditorLines('a\n')).toBe(2);
    expect(editorLineForOffset('a\nb\nc', 0)).toBe(1);
    expect(editorLineForOffset('a\nb\nc', 2)).toBe(2);
    expect(editorLineForOffset('a\nb\nc', 999)).toBe(3);
    expect(editorLineForOffset('a\nb\nc', -4)).toBe(1);
  });

  it('caps gutter line numbers without changing the file line count', () => {
    const value = Array.from({ length: 2_001 }, (_, index) => `line ${index + 1}`).join('\n');
    const built = buildEditorLineNumbers(value);
    expect(built.numbers).toHaveLength(2_000);
    expect(built.numbers[1_999]).toBe(2_000);
    expect(built.truncated).toBe(true);
    expect(countEditorLines(value)).toBe(2_001);
  });

  it('recognises save shortcuts and approval blocks only unapproved writes', () => {
    expect(editorShortcut('s', { ctrlKey: true })).toBe('save');
    expect(editorShortcut('S', { metaKey: true })).toBe('save');
    expect(editorShortcut('s', {})).toBeNull();
    expect(editorShortcut('s', { ctrlKey: true, altKey: true })).toBeNull();
    expect(approvalBlocksSave(null)).toBe(false);
    expect(approvalBlocksSave({ id: 'a', tool: 'write', state: 'approved' })).toBe(false);
    expect(approvalBlocksSave({ id: 'a', tool: 'write', state: 'required' })).toBe(true);
  });

  it('renders an idle, error and read-only editor honestly', () => {
    const idle = render(createElement(Editor, { path: 'src/a.ts', value: '', status: 'idle' }));
    expect(idle).toContain('No file open');
    expect(idle).not.toContain('<textarea');

    const loading = render(createElement(Editor, { path: 'src/a.ts', value: '', status: 'loading' }));
    expect(loading).toContain('Opening file');

    const error = render(
      createElement(Editor, {
        path: 'src/a.ts',
        value: '',
        status: 'error',
        errorMessage: 'E_FILE_TOO_LARGE',
        onRetry: noop,
      }),
    );
    expect(error).toContain('role="alert"');
    expect(error).toContain('E_FILE_TOO_LARGE');

    const readOnly = render(
      createElement(Editor, {
        path: 'src/a.ts',
        value: 'const a = 1;',
        status: 'ready',
        readOnly: true,
        readOnlyReason: 'Binary file',
        onSave: noop,
      }),
    );
    expect(readOnly).toContain('readOnly=""');
    expect(readOnly).toContain('Read-only');
    expect(readOnly).toContain('Binary file');
  });

  it('blocks save while approval is required and shows the gate', () => {
    const markup = render(
      createElement(Editor, {
        path: 'src/a.ts',
        value: 'edited',
        status: 'ready',
        dirty: true,
        approval: { id: 'ap-1', tool: 'file_write', state: 'required' },
        onSave: noop,
        onRequestApproval: noop,
      }),
    );
    expect(markup).toContain('Approval required before saving');
    expect(markup).toContain('Request approval');
    expect(markup).toMatch(/disabled=""[^>]*aria-label="Save src\/a\.ts"/);
  });

  it('enables save only for a dirty, ready, editable file', () => {
    const clean = render(
      createElement(Editor, {
        path: 'a.ts',
        value: 'x',
        status: 'ready',
        dirty: false,
        onSave: noop,
        onChange: noop,
      }),
    );
    expect(clean).toMatch(/disabled=""[^>]*aria-label="Save a\.ts"/);

    const dirty = render(
      createElement(Editor, {
        path: 'a.ts',
        value: 'x',
        status: 'ready',
        dirty: true,
        onSave: noop,
        onChange: noop,
      }),
    );
    expect(dirty).not.toMatch(/disabled=""[^>]*aria-label="Save a\.ts"/);
  });
});

describe('CAP-4 DiffView', () => {
  it('computes a deterministic LCS diff with hunk headers', () => {
    const result = diffLines('a\nb\nc\n', 'a\nB\nc\n');
    expect(result.additions).toBe(1);
    expect(result.deletions).toBe(1);
    expect(result.coarse).toBe(false);
    expect(result.hunks).toHaveLength(1);
    expect(result.hunks[0]?.lines.map((line) => line.kind)).toEqual([
      'context',
      'remove',
      'add',
      'context',
    ]);
    expect(formatDiffHunkHeader(result.hunks[0] as NonNullable<typeof result.hunks[0]>)).toBe(
      '@@ -1,3 +1,3 @@',
    );
    expect(result.hunks[0]?.lines[1]?.oldLine).toBe(2);
    expect(result.hunks[0]?.lines[2]?.newLine).toBe(2);
  });

  it('reports no hunks for identical content and handles empty sides', () => {
    const same = diffLines('same\n', 'same\n');
    expect(same.hunks).toHaveLength(0);
    expect(same.additions).toBe(0);
    expect(same.deletions).toBe(0);

    const added = diffLines('', 'new\n');
    expect(added.additions).toBe(1);
    expect(added.deletions).toBe(0);

    const removed = diffLines('old\n', '');
    expect(removed.additions).toBe(0);
    expect(removed.deletions).toBe(1);

    const eol = diffLines('same', 'same\n');
    expect(eol.hunks).toHaveLength(0);
    expect(eol.eolChange).toBe(true);

    const crlf = diffLines('same\r\n', 'same\n');
    expect(crlf.hunks).toHaveLength(0);
    expect(crlf.eolChange).toBe(true);
  });

  it('labels every diff line for assistive technology', () => {
    const add: DiffLine = { kind: 'add', text: 'x', oldLine: null, newLine: 3 };
    const remove: DiffLine = { kind: 'remove', text: 'y', oldLine: 4, newLine: null };
    const context: DiffLine = { kind: 'context', text: '', oldLine: 5, newLine: 5 };
    expect(diffLineAriaLabel(add)).toContain('Added line 3');
    expect(diffLineAriaLabel(remove)).toContain('Removed line 4');
    expect(diffLineAriaLabel(context)).toContain('blank line');
  });

  it('caps rendered diff lines and flags truncation', () => {
    const result = diffLines('a\nb\nc\n', 'a\nX\nY\nc\n');
    const limited = limitDiffHunks(result, 2);
    expect(limited.truncated).toBe(true);
    expect(limited.hunks.flatMap((hunk) => hunk.lines)).toHaveLength(2);
  });

  it('shows approval-required, pending, denied and accepted states', () => {
    const required = render(
      createElement(DiffView, {
        filePath: 'src/a.ts',
        before: 'a\n',
        after: 'b\n',
        approvalState: 'required',
        onAccept: noop,
        onReject: noop,
        onRequestApproval: noop,
      }),
    );
    expect(required).toContain('Approval required before this diff can be applied.');
    expect(required).toContain('Request approval');
    expect(required).toMatch(/disabled=""[^>]*aria-label="Accept changes to src\/a\.ts"/);

    const pending = render(
      createElement(DiffView, {
        filePath: 'src/a.ts',
        before: 'a\n',
        after: 'b\n',
        approvalState: 'pending',
        onAccept: noop,
        onReject: noop,
      }),
    );
    expect(pending).toContain('Waiting for approval');

    const denied = render(
      createElement(DiffView, {
        filePath: 'src/a.ts',
        before: 'a\n',
        after: 'b\n',
        approvalState: 'denied',
        onAccept: noop,
        onReject: noop,
      }),
    );
    expect(denied).toContain('role="alert"');
    expect(denied).toContain('Approval was denied');

    const approved = render(
      createElement(DiffView, {
        filePath: 'src/a.ts',
        before: 'a\n',
        after: 'b\n',
        approvalState: 'approved',
        onAccept: noop,
        onReject: noop,
      }),
    );
    expect(approved).not.toMatch(/disabled=""[^>]*aria-label="Accept changes to src\/a\.ts"/);

    const accepted = render(
      createElement(DiffView, {
        filePath: 'src/a.ts',
        before: 'a\n',
        after: 'b\n',
        decision: 'accepted',
        onAccept: noop,
        onReject: noop,
      }),
    );
    expect(accepted).toContain('Accepted');
  });

  it('renders no-change, error and coarse-diff states without pretending to apply', () => {
    const loading = render(
      createElement(DiffView, {
        filePath: 'a.ts',
        before: 'a',
        after: 'b',
        status: 'loading',
      }),
    );
    expect(loading).toContain('Computing diff');
    expect(loading).not.toContain('aria-label="Accept changes to a.ts"');

    const unchanged = render(
      createElement(DiffView, { filePath: 'a.ts', before: 'same\n', after: 'same\n' }),
    );
    expect(unchanged).toContain('No changes: the proposed content matches the current file.');

    const error = render(
      createElement(DiffView, {
        filePath: 'a.ts',
        before: '',
        after: '',
        status: 'error',
        errorMessage: 'E_PATCH_CONTEXT',
        onRetry: noop,
      }),
    );
    expect(error).toContain('role="alert"');
    expect(error).toContain('E_PATCH_CONTEXT');

    // Above the cell budget the result is explicitly labelled as a coarse replace.
    const largeBefore = Array.from({ length: 1_100 }, (_, index) => `old ${index}`).join('\n');
    const largeAfter = Array.from({ length: 1_100 }, (_, index) => `new ${index}`).join('\n');
    const coarse = render(createElement(DiffView, { filePath: 'big.ts', before: largeBefore, after: largeAfter }));
    expect(coarse).toContain('whole-file replacement diff');
  });
});

describe('CAP-4 Terminal', () => {
  const base = {
    status: 'running' as const,
    hasSubmit: true,
  };

  it('enables input only for a live transcript without an unresolved approval', () => {
    expect(canSubmitTerminal({ ...base })).toBe(true);
    expect(canSubmitTerminal({ ...base, status: 'idle' })).toBe(true);
    expect(canSubmitTerminal({ ...base, hasSubmit: false })).toBe(false);
    expect(canSubmitTerminal({ ...base, disabled: true })).toBe(false);
    expect(canSubmitTerminal({ ...base, status: 'connecting' })).toBe(false);
    expect(canSubmitTerminal({ ...base, status: 'closed' })).toBe(false);
    expect(canSubmitTerminal({ ...base, status: 'error' })).toBe(false);
    expect(
      canSubmitTerminal({ ...base, approval: { tool: 'shell', state: 'required' } }),
    ).toBe(false);
    expect(
      canSubmitTerminal({ ...base, approval: { tool: 'shell', state: 'approved' } }),
    ).toBe(true);
  });

  it('formats timestamps and prompts without inventing a time or user', () => {
    expect(formatTimelineTime('2026-11-11T01:02:03.000Z')).toBe('01:02:03');
    expect(formatTimelineTime('not-a-date')).toBe('—');
    expect(formatTimelineTime(null)).toBe('—');
  });

  it('renders a real transcript, status and command form when a submit path exists', () => {
    const markup = render(
      createElement(Terminal, {
        status: 'running',
        cwd: 'C:\\work',
        shell: 'pwsh',
        lines: [
          { id: 'l1', kind: 'input', text: 'npm test', at: '2026-11-11T01:00:00.000Z' },
          { id: 'l2', kind: 'stdout', text: 'ok 12 tests' },
          { id: 'l3', kind: 'stderr', text: 'warning' },
        ],
        onSubmit: noop,
        onInterrupt: noop,
        onClear: noop,
      }),
    );
    expect(markup).toContain('role="log"');
    expect(markup).toContain('npm test');
    expect(markup).toContain('ok 12 tests');
    expect(markup).toContain('warning');
    expect(markup).toContain('Running');
    expect(markup).toContain('aria-label="Command"');
    expect(markup).not.toContain('Terminal is read-only');
  });

  it('renders empty, connecting, error, read-only and approval states', () => {
    const empty = render(createElement(Terminal, { status: 'idle', lines: [] }));
    expect(empty).toContain('No output yet.');
    expect(empty).toContain('Terminal is read-only');

    const connecting = render(createElement(Terminal, { status: 'connecting', lines: [] }));
    expect(connecting).toContain('Connecting to the shell');

    const error = render(
      createElement(Terminal, {
        status: 'error',
        lines: [],
        errorMessage: 'E_TOOL_TIMEOUT',
        onRetry: noop,
        onSubmit: noop,
      }),
    );
    expect(error).toContain('role="alert"');
    expect(error).toContain('E_TOOL_TIMEOUT');

    const approval = render(
      createElement(Terminal, {
        status: 'running',
        lines: [],
        approval: { state: 'pending', tool: 'shell' },
        onSubmit: noop,
      }),
    );
    expect(approval).toContain('Waiting for approval');
    expect(approval).toMatch(/aria-label="Command"[^>]*disabled=""/);
  });
});

describe('CAP-4 Timeline', () => {
  const steps: readonly TimelineStep[] = [
    { id: 'b', at: '2026-11-11T01:02:00.000Z', agentId: 'a2', title: 'second', status: 'ok' },
    { id: 'a', at: '2026-11-11T01:01:00.000Z', agentId: 'a1', title: 'first', status: 'awaiting_approval' },
    { id: 'c', at: 'not-a-date', agentId: 'a1', title: 'invalid time', status: 'failed' },
  ];

  it('sorts by real event time with invalid timestamps last', () => {
    expect(sortTimelineSteps(steps).map((step) => step.id)).toEqual(['a', 'b', 'c']);
    expect(formatTimelineTime('2026-11-11T01:01:02.000Z')).toBe('01:01:02');
    expect(formatTimelineTime('bad')).toBe('—');
    expect(timelineStatusLabel('awaiting_approval')).toBe('Awaiting approval');
    expect(timelineStatusLabel('bogus')).toBe('Unknown');
  });

  it('derives undeclared agents from steps and groups without dropping rows', () => {
    const derived = timelineAgents([{ id: 'a1', label: 'Planner' }], steps);
    expect(derived.map((agent) => agent.id)).toEqual(['a1', 'a2']);
    expect(derived[1]?.label).toBe('a2');
    const groups = groupTimelineSteps(steps, [{ id: 'a1', label: 'Planner' }]);
    expect(groups.map((group) => group.agent?.id)).toEqual(['a1', 'a2']);
    expect(groups[0]?.steps.map((step) => step.id)).toEqual(['a', 'c']);
  });

  it('wraps tab navigation and clamps empty lists', () => {
    expect(timelineTabTarget(0, 3, 'ArrowLeft')).toBe(2);
    expect(timelineTabTarget(2, 3, 'ArrowRight')).toBe(0);
    expect(timelineTabTarget(1, 3, 'Home')).toBe(0);
    expect(timelineTabTarget(1, 3, 'End')).toBe(2);
    expect(timelineTabTarget(9, 0, 'ArrowRight')).toBe(0);
  });

  it('renders per-agent tabs, UTC timestamps and approval steps', () => {
    const markup = render(
      createElement(Timeline, {
        status: 'ready',
        agents: [
          { id: 'a1', label: 'Planner' },
          { id: 'a2', label: 'Builder' },
        ],
        steps,
        selectedAgentId: 'a1',
      }),
    );
    expect(markup).toContain('role="tablist"');
    expect(markup).toContain('All agents');
    expect(markup).toContain('Planner');
    expect(markup).toContain('Awaiting approval');
    expect(markup).toContain('first');
    expect(markup).not.toContain('second');
    expect(markup).toContain('01:01:00');
  });

  it('renders empty, loading and error states', () => {
    const empty = render(
      createElement(Timeline, { status: 'ready', agents: [], steps: [] }),
    );
    expect(empty).toContain('No timeline events yet.');

    const idle = render(createElement(Timeline, { status: 'idle', agents: [], steps: [] }));
    expect(idle).toContain('Timeline has not loaded yet.');

    const loading = render(createElement(Timeline, { status: 'loading', agents: [], steps: [] }));
    expect(loading).toContain('role="status"');
    expect(loading).toContain('Loading timeline');

    const error = render(
      createElement(Timeline, {
        status: 'error',
        agents: [],
        steps: [],
        errorMessage: 'E_STREAM',
        onRetry: noop,
      }),
    );
    expect(error).toContain('role="alert"');
    expect(error).toContain('E_STREAM');
  });
});

describe('CAP-4 TodoPanel', () => {
  const items: readonly TodoItem[] = [
    { id: 't1', text: 'Implement parser', status: 'completed' },
    { id: 't2', text: 'Add tests', status: 'in_progress' },
    { id: 't3', text: 'Write docs', status: 'blocked' },
  ];

  it('summarises progress without inventing completion', () => {
    const summary = todoSummary(items);
    expect(summary).toEqual({ total: 3, completed: 1, blocked: 1, inProgress: 1, percent: 33 });
    expect(todoProgressLabel(summary)).toBe('1/3 done');
    expect(todoSummary([]).percent).toBe(0);
    expect(todoSummary([]).total).toBe(0);
    expect(isTodoComplete('completed')).toBe(true);
    expect(isTodoComplete('pending')).toBe(false);
    expect(todoToggleTarget('completed')).toBe('pending');
    expect(todoToggleTarget('pending')).toBe('completed');
    expect(todoToggleTarget('cancelled')).toBeNull();
    expect(todoStatusLabel('in_progress')).toBe('In progress');
    expect(todoStatusLabel('bogus')).toBe('Unknown');
  });

  it('renders items, progress and approval-gated checkboxes', () => {
    const markup = render(
      createElement(TodoPanel, {
        items: [
          ...items,
          {
            id: 't4',
            text: 'Apply patch',
            status: 'pending',
            approval: 'required',
            approvalTool: 'file_write',
          },
        ],
        onToggle: noop,
        onRequestApproval: noop,
      }),
    );
    expect(markup).toContain('role="progressbar"');
    expect(markup).toContain('aria-valuenow="25"');
    expect(markup).toContain('Implement parser');
    expect(markup).toContain('Approval required');
    expect(markup).toContain('Request');
    expect(markup).toMatch(/disabled=""[^>]*aria-label="Apply patch/);
  });

  it('renders empty, loading and error states', () => {
    const empty = render(createElement(TodoPanel, { items: [], status: 'ready' }));
    expect(empty).toContain('No plan yet.');
    const loading = render(createElement(TodoPanel, { items: [], status: 'loading' }));
    expect(loading).toContain('Loading plan');
    const error = render(
      createElement(TodoPanel, {
        items: [],
        status: 'error',
        errorMessage: 'E_DB_BUSY',
        onRetry: noop,
      }),
    );
    expect(error).toContain('role="alert"');
    expect(error).toContain('E_DB_BUSY');
  });

  it('keeps checkboxes disabled without an onToggle callback', () => {
    const markup = render(createElement(TodoPanel, { items, status: 'ready' }));
    expect(markup).toContain('disabled=""');
  });
});

describe('CAP-4 SessionRail', () => {
  const sessions: readonly SessionSummary[] = [
    { id: 's1', label: 'Feature work', status: 'idle', updatedAt: '2026-11-11T01:00:00.000Z' },
    { id: 's2', label: 'Current run', status: 'active', waitingApproval: true, costUsd: 0.42 },
  ];

  it('pins the current session first and keeps host order otherwise', () => {
    expect(orderSessions(sessions, 's2').map((session) => session.id)).toEqual(['s2', 's1']);
    expect(orderSessions(sessions, 'missing').map((session) => session.id)).toEqual(['s1', 's2']);
    expect(orderSessions(sessions, null).map((session) => session.id)).toEqual(['s1', 's2']);
  });

  it('formats timestamps/cost and never invents a status', () => {
    expect(formatSessionUpdated('2026-11-11T01:02:03.000Z')).toBe('2026-11-11 01:02');
    expect(formatSessionUpdated(null)).toBe('—');
    expect(formatSessionCost(0.42)).toBe('$0.42');
    expect(formatSessionCost(null)).toBe('—');
    expect(sessionStatusLabel('active')).toBe('Active');
    expect(sessionStatusLabel(undefined)).toBe('Unknown');
    expect(sessionNavigationTarget(0, 2, 'ArrowDown')).toBe(1);
    expect(sessionNavigationTarget(0, 2, 'ArrowUp')).toBe(1);
    expect(sessionNavigationTarget(9, 0, 'ArrowDown')).toBeNull();
  });

  it('renders current session, approval marker and horizontal-safe rail', () => {
    const markup = render(
      createElement(SessionRail, {
        sessions,
        currentSessionId: 's2',
        onSelect: noop,
        onNew: noop,
      }),
    );
    expect(markup).toContain('aria-current="true"');
    expect(markup).toContain('Feature work');
    expect(markup).toContain('Approval');
    expect(markup).toContain('$0.42');
    expect(markup).toContain('overflow-x-auto');
  });

  it('renders empty, loading and error states', () => {
    const empty = render(createElement(SessionRail, { sessions: [], status: 'ready' }));
    expect(empty).toContain('No sessions yet.');
    const idle = render(createElement(SessionRail, { sessions: [], status: 'idle' }));
    expect(idle).toContain('Sessions have not loaded yet.');
    const loading = render(createElement(SessionRail, { sessions: [], status: 'loading' }));
    expect(loading).toContain('Loading sessions');
    const error = render(
      createElement(SessionRail, {
        sessions: [],
        status: 'error',
        errorMessage: 'E_REMOTE_OFFLINE',
        onRetry: noop,
      }),
    );
    expect(error).toContain('role="alert"');
    expect(error).toContain('E_REMOTE_OFFLINE');
  });
});

describe('CAP-4 CostBar', () => {
  it('formats tokens/cost deterministically and derives only complete totals', () => {
    expect(formatTokenCount(1_234_567)).toBe('1,234,567');
    expect(formatTokenCount(0)).toBe('0');
    expect(formatTokenCount(null)).toBe('—');
    expect(formatTokenCount(Number.NaN)).toBe('—');
    expect(formatUsd(0.0042)).toBe('$0.0042');
    expect(formatUsd(1.2)).toBe('$1.20');
    expect(formatUsd(null)).toBe('—');
    expect(effectiveTotalTokens(null)).toEqual({ value: null, source: 'unknown' });
    expect(effectiveTotalTokens({ totalTokens: 30 })).toEqual({ value: 30, source: 'server' });
    expect(effectiveTotalTokens({ promptTokens: 10, completionTokens: 5 })).toEqual({
      value: 15,
      source: 'derived',
    });
    expect(effectiveTotalTokens({ promptTokens: 10 })).toEqual({ value: null, source: 'unknown' });
    expect(budgetProgress(5, 10)).toEqual({ ratio: 0.5, percent: 50, over: false });
    expect(budgetProgress(15, 10)).toEqual({ ratio: 1.5, percent: 150, over: true });
    expect(budgetProgress(5, 0)).toBeNull();
  });

  it('renders server usage and a budget bar without price math', () => {
    const markup = render(
      createElement(CostBar, {
        usage: { promptTokens: 100, completionTokens: 20, totalTokens: 120, cacheHitTokens: 40 },
        costUsd: 5,
        budgetUsd: 10,
      }),
    );
    expect(markup).toContain('120');
    expect(markup).toContain('$5.00');
    expect(markup).toContain('role="progressbar"');
    expect(markup).toContain('50%');
  });

  it('shows derived totals and an unreported cost honestly', () => {
    const markup = render(createElement(CostBar, { usage: { promptTokens: 10, completionTokens: 5 } }));
    expect(markup).toContain('15');
    expect(markup).toContain('derived');
    expect(markup).toContain('Cost not reported');
  });

  it('shows cost even when usage is absent, and handles unavailable/error/empty', () => {
    const costOnly = render(createElement(CostBar, { usage: null, costUsd: 0.02 }));
    expect(costOnly).toContain('Tokens not reported');
    expect(costOnly).toContain('$0.02');

    const unavailable = render(createElement(CostBar, { status: 'unavailable' }));
    expect(unavailable).toContain('Usage is incomplete');

    const error = render(
      createElement(CostBar, {
        status: 'error',
        errorMessage: 'E_USAGE_INCOMPLETE',
        onRetry: noop,
      }),
    );
    expect(error).toContain('role="alert"');
    expect(error).toContain('E_USAGE_INCOMPLETE');

    const idle = render(createElement(CostBar, { status: 'idle' }));
    expect(idle).toContain('No usage reported yet.');

    const loading = render(createElement(CostBar, { status: 'loading' }));
    expect(loading).toContain('Loading usage');
  });
});

describe('CAP-4 CommandPalette', () => {
  const actions: readonly PaletteAction[] = [
    {
      id: 'open-file',
      label: 'Open file',
      description: 'Open a workspace file',
      group: 'Files',
      keywords: ['read'],
      shortcut: 'Ctrl+O',
      onRun: noop,
    },
    {
      id: 'run-tests',
      label: 'Run tests',
      description: 'Execute the test suite',
      group: 'Execution',
      requiresApproval: true,
      onRun: noop,
    },
    {
      id: 'delete-file',
      label: 'Delete file',
      disabled: true,
      disabledReason: 'Approval required',
      onRun: noop,
    },
  ];

  it('ranks exact/prefix label matches and preserves caller order for an empty query', () => {
    expect(scorePaletteAction(actions[0] as PaletteAction, 'open file')).toBe(1_000);
    expect(scorePaletteAction(actions[0] as PaletteAction, 'open')).toBeGreaterThan(700);
    expect(scorePaletteAction(actions[0] as PaletteAction, 'read')).toBe(400);
    expect(scorePaletteAction(actions[0] as PaletteAction, 'zzz')).toBeNull();
    expect(filterPaletteActions(actions, '').map((action) => action.id)).toEqual([
      'open-file',
      'run-tests',
      'delete-file',
    ]);
    expect(filterPaletteActions(actions, 'open').map((action) => action.id)[0]).toBe('open-file');
  });

  it('maps palette navigation keys and keeps disabled actions selectable', () => {
    expect(paletteNavigationTarget(0, 3, 'ArrowDown')).toEqual({ kind: 'move', index: 1 });
    expect(paletteNavigationTarget(0, 3, 'ArrowUp')).toEqual({ kind: 'move', index: 2 });
    expect(paletteNavigationTarget(0, 3, 'Home')).toEqual({ kind: 'move', index: 0 });
    expect(paletteNavigationTarget(0, 3, 'End')).toEqual({ kind: 'move', index: 2 });
    expect(paletteNavigationTarget(0, 3, 'Enter')).toEqual({ kind: 'run' });
    expect(paletteNavigationTarget(0, 3, 'Escape')).toEqual({ kind: 'close' });
    expect(paletteNavigationTarget(0, 0, 'ArrowDown')).toBeNull();
    const filtered = filterPaletteActions(actions, 'delete');
    expect(filtered.map((action) => action.id)).toEqual(['delete-file']);
  });

  it('renders a closed palette as null and an open one as an accessible dialog', () => {
    expect(render(createElement(CommandPalette, { open: false, actions, onClose: noop }))).toBe('');
    const markup = render(createElement(CommandPalette, { open: true, actions, onClose: noop }));
    expect(markup).toContain('role="dialog"');
    expect(markup).toContain('aria-modal="true"');
    expect(markup).toContain('role="combobox"');
    expect(markup).toContain('role="listbox"');
    expect(markup).toContain('Open file');
    expect(markup).toContain('Ctrl+O');
    expect(markup).toContain('approval');
    expect(markup).toContain('aria-disabled="true"');
    expect(markup).toContain('Approval required');
    expect(markup).toContain('aria-activedescendant');
  });

  it('renders empty, loading and error states', () => {
    const empty = render(createElement(CommandPalette, { open: true, actions: [], onClose: noop }));
    expect(empty).toContain('No matching commands.');

    const loading = render(
      createElement(CommandPalette, { open: true, actions: [], onClose: noop, status: 'loading' }),
    );
    expect(loading).toContain('Loading commands');

    const error = render(
      createElement(CommandPalette, {
        open: true,
        actions: [],
        onClose: noop,
        status: 'error',
        errorMessage: 'E_COMMAND_UNKNOWN',
        onRetry: noop,
      }),
    );
    expect(error).toContain('role="alert"');
    expect(error).toContain('E_COMMAND_UNKNOWN');
  });

  it('runs only enabled actions through the caller callback', () => {
    const onRun = vi.fn();
    const onClose = vi.fn();
    const runnable: PaletteAction = { id: 'go', label: 'Go', onRun };
    const markup = render(createElement(CommandPalette, { open: true, actions: [runnable], onClose }));
    expect(markup).toContain('Go');
    // The component only invokes onRun from a real user event; SSR must not run it.
    expect(onRun).not.toHaveBeenCalled();
    expect(onClose).not.toHaveBeenCalled();
  });
});

describe('CAP-4 workbench integration contract', () => {
  it('exports every pane and the dependency requirements', () => {
    for (const pane of [CommandPalette, CostBar, DiffView, Editor, FileTree, SessionRail, Terminal, Timeline, TodoPanel]) {
      expect(typeof pane).toBe('function');
    }
    const deps = WORKBENCH_DEPENDENCY_REQUIREMENTS;
    expect(deps.editor.required['@codemirror/state']).toBe('6.7.6');
    expect(deps.editor.required['@codemirror/view']).toBe('6.43.15');
    expect(deps.editor.required['@codemirror/commands']).toBe('6.11.1');
    expect(deps.editor.required['@codemirror/language']).toBe('6.13.1');
    expect(deps.terminal.required['@xterm/xterm']).toBe('6.0.0');
    expect(deps.terminal.required['@xterm/addon-fit']).toBe('0.11.0');
  });
});
