/**
 * CAP-4 workbench public surface.
 *
 * The Lead composes these panes inside the existing Workspace/Data/Agents
 * surfaces. Every component is prop-driven and stateless with respect to the
 * runtime: no component reads or writes the store, opens a socket, or invents
 * data. The only local state is UI state (focus, expanded folders, filter,
 * search query, unsent command text).
 *
 * Dependencies deliberately NOT installed by this work package:
 * the CodeMirror and xterm packages below are required for the full editor /
 * ANSI-PTY experience. This build ships a functional textarea editor and an
 * accessible line-oriented terminal transcript; it does not claim syntax
 * highlighting or ANSI emulation without these packages.
 */
export { FileTree, FILE_TREE_COPY, buildFileTreeNodes, countFileNodes, flattenFileTree, treeNavigationTarget, formatFileSize } from './FileTree';
export type {
  FileTreeProps,
  FileTreeNode,
  FileTreeRow,
  FileTreeStatus,
  FileNodeKind,
  FlatFileTreeEntry,
  TreeNavigationTarget,
} from './FileTree';

export {
  Editor,
  EDITOR_COPY,
  MAX_RENDERED_LINE_NUMBERS,
  approvalBlocksSave,
  buildEditorLineNumbers,
  countEditorLines,
  editorLineForOffset,
  editorShortcut,
  editorStatusLabel,
} from './Editor';
export type {
  EditorProps,
  EditorStatus,
  EditorApproval,
  EditorApprovalState,
  EditorLineNumbers,
  EditorShortcut,
} from './Editor';

export {
  DiffView,
  DIFF_COPY,
  DIFF_CONTEXT_LINES,
  MAX_DIFF_CELLS,
  MAX_DIFF_LINES,
  MAX_RENDERED_DIFF_LINES,
  diffLines,
  diffLineAriaLabel,
  formatDiffHunkHeader,
  limitDiffHunks,
} from './DiffView';
export type {
  DiffViewProps,
  DiffStatus,
  DiffApprovalState,
  DiffDecision,
  DiffLine,
  DiffLineKind,
  DiffHunk,
  DiffResult,
  LimitedDiff,
} from './DiffView';

export {
  Terminal,
  TERMINAL_COPY,
  canSubmitTerminal,
  formatTerminalTime,
  terminalLineLabel,
  terminalLineTone,
  terminalPrompt,
  terminalStatusLabel,
  terminalStatusTone,
} from './Terminal';
export type {
  TerminalProps,
  TerminalStatus,
  TerminalLine,
  TerminalLineKind,
  TerminalApproval,
  TerminalApprovalState,
  TerminalInputAvailability,
} from './Terminal';

export {
  Timeline,
  TIMELINE_COPY,
  formatTimelineTime,
  groupTimelineSteps,
  sortTimelineSteps,
  timelineAgents,
  timelineStatusLabel,
  timelineStatusTone,
  timelineTabTarget,
} from './Timeline';
export type {
  TimelineProps,
  TimelineStatus,
  TimelineStepStatus,
  TimelineStep,
  TimelineAgent,
  TimelineGroup,
  TimelineTabKey,
} from './Timeline';

export {
  TodoPanel,
  TODO_COPY,
  isTodoComplete,
  todoApprovalLabel,
  todoProgressLabel,
  todoStatusLabel,
  todoStatusTone,
  todoSummary,
  todoToggleTarget,
} from './TodoPanel';
export type {
  TodoPanelProps,
  TodoPanelStatus,
  TodoItem,
  TodoStatus,
  TodoApprovalState,
  TodoSummary,
} from './TodoPanel';

export {
  SessionRail,
  SESSION_COPY,
  formatSessionCost,
  formatSessionUpdated,
  orderSessions,
  sessionAriaLabel,
  sessionNavigationTarget,
  sessionStatusLabel,
  sessionStatusTone,
  truncateSessionId,
} from './SessionRail';
export type { SessionRailProps, SessionRailStatus, SessionSummary, SessionStatus } from './SessionRail';

export {
  CostBar,
  COST_COPY,
  budgetProgress,
  costStatusLabel,
  effectiveTotalTokens,
  formatTokenCount,
  formatUsd,
  usageSourceLabel,
} from './CostBar';
export type {
  CostBarProps,
  CostStatus,
  Usage,
  UsageSource,
  EffectiveTotal,
  BudgetProgress,
} from './CostBar';

export {
  CommandPalette,
  PALETTE_COPY,
  filterPaletteActions,
  paletteNavigationTarget,
  scorePaletteAction,
} from './CommandPalette';
export type {
  CommandPaletteProps,
  PaletteStatus,
  PaletteAction,
  PaletteNavigation,
} from './CommandPalette';

/**
 * Exact package/version pins the Lead must add to `web/package.json` and
 * `web/package-lock.json` before claiming a full CodeMirror editor or ANSI/PTY
 * terminal. Versions were read from the npm registry on 2026-11-11; nothing was
 * installed by this work package. Language packs are per file type and are not
 * required for the plain-text editor.
 */
export const WORKBENCH_DEPENDENCY_REQUIREMENTS = {
  editor: {
    required: {
      '@codemirror/state': '6.7.6',
      '@codemirror/view': '6.43.15',
      '@codemirror/commands': '6.11.1',
      '@codemirror/language': '6.13.1',
    },
    perLanguage: ['@codemirror/lang-javascript', '@codemirror/lang-json', '@codemirror/lang-markdown', '@codemirror/lang-rust', '@codemirror/lang-css', '@codemirror/lang-html'],
    reason:
      'syntax highlighting, bracket matching, undo history, selection and large-document editing; the current Editor is an accessible textarea fallback',
  },
  terminal: {
    required: {
      '@xterm/xterm': '6.0.0',
      '@xterm/addon-fit': '0.11.0',
    },
    reason:
      'ANSI/VT emulation, PTY resize and interactive keystrokes; the current Terminal is an accessible line-oriented transcript, not an emulator',
  },
} as const;
