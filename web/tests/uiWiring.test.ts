import { createElement, type ReactElement } from 'react';
import { renderToStaticMarkup } from 'react-dom/server';
import { describe, expect, it } from 'vitest';

import type { UiApi } from '../src/api/client';
import { AgentsPanel } from '../src/panels/AgentsPanel';
import { ApprovalOverlay } from '../src/panels/ApprovalOverlay';
import { chatToggleIntent, Header } from '../src/panels/Header';
import { SnapshotBanner, snapshotBannerTone } from '../src/panels/SnapshotBanner';
import { toastStackZClass } from '../src/panels/ToastRegion';
import {
  clampApprovalIndex,
  approvalQueueLabel,
} from '../src/panels/ApprovalOverlay';
import {
  proposalWriteHeaders,
} from '../src/panels/ProposalsPanel';
import { UiStoreContext } from '../src/state/context';
import {
  createUiStore,
  type ApprovalRecord,
  type SnapshotState,
  type TaskRecord,
  type UiState,
} from '../src/state/store';
import { STRINGS } from '../src/state/strings';
// @ts-expect-error -- the web tsconfig deliberately omits Node types; Vitest runs on Node.
import { readFileSync } from 'node:fs';

function readWebText(relativePath: string): string {
  return readFileSync(new URL(relativePath, import.meta.url), 'utf8');
}

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
      /* static renders never open a transport */
    },
    async send() {
      /* no commands in these tests */
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

function renderStore(store: ReturnType<typeof createUiStore>, node: ReactElement): string {
  (store as unknown as { getInitialState: () => UiState }).getInitialState = () =>
    store.getState();
  return renderToStaticMarkup(createElement(UiStoreContext.Provider, { value: store }, node));
}

function renderWith(node: ReactElement, state: Partial<UiState> = {}): string {
  const store = createUiStore(fakeApi());
  store.setState({ connection: 'open' as const, ...state });
  return renderStore(store, node);
}

function task(overrides: Partial<TaskRecord> & Pick<TaskRecord, 'id' | 'kind'>): TaskRecord {
  return {
    parentId: null,
    label: overrides.id,
    state: 'running',
    progress: null,
    errorCode: null,
    rev: null,
    ...overrides,
  };
}

function approval(overrides: Partial<ApprovalRecord> = {}): ApprovalRecord {
  return {
    approvalId: 'ap-1',
    tool: 'file_write',
    nonce: 'nonce-1',
    decision: null,
    requestedAt: 0,
    runId: 'run-1',
    taskId: 'task-1',
    toolCallId: 'call-1',
    argsSha256: 'a'.repeat(64),
    expiresAt: null,
    ...overrides,
  };
}

function snapshot(overrides: Partial<SnapshotState> = {}): SnapshotState {
  return {
    status: 'degraded',
    source: 'manual',
    runId: 'run-1',
    revision: null,
    revisionCount: 0,
    incomplete: ['messages-unavailable'],
    refusal: null,
    attempts: 2,
    updatedAt: Date.parse('2026-11-10T20:00:00.000Z'),
    ...overrides,
  };
}

describe('workspace tab navigation (S1 dead-end fix)', () => {
  it('setWorkspaceTab moves nav and tab together', () => {
    const store = createUiStore(fakeApi());
    store.setState({ view: { nav: 'settings', tab: 'history' } });
    store.getState().setWorkspaceTab('swarm');
    expect(store.getState().view).toEqual({ nav: 'workspace', tab: 'swarm' });

    // setTab alone keeps its hash-routing semantics (it must not force nav).
    store.setState({ view: { nav: 'agents', tab: 'workspace' } });
    store.getState().setTab('history');
    expect(store.getState().view).toEqual({ nav: 'agents', tab: 'history' });
  });

  it('LibraryPanel uses the nav-moving action in both click and keyboard paths', () => {
    const source = sourceOf('/panels/LibraryPanel.tsx');
    expect(source).toContain('state.setWorkspaceTab');
    expect((source.match(/setWorkspaceTab\(/g) ?? []).length).toBeGreaterThanOrEqual(2);
    expect(source).not.toMatch(/onClick=\{\(\) => setTab\(/);
  });
});

describe('header chat toggle (S1 dead control fix)', () => {
  it('decides navigate+show off Workspace and plain toggle inside it', () => {
    expect(chatToggleIntent(false, false)).toBe('navigate-show');
    expect(chatToggleIntent(false, true)).toBe('navigate-show');
    expect(chatToggleIntent(true, false)).toBe('show');
    expect(chatToggleIntent(true, true)).toBe('hide');
  });

  it('renders a Workspace navigation hint off Workspace instead of a no-op toggle', () => {
    const markup = renderWith(createElement(Header), {
      view: { nav: 'settings', tab: 'workspace' },
      chat: { pinned: false, visible: true },
    });
    expect(markup).toContain(STRINGS.chat.toggleWorkspaceHint);
    expect(markup).toContain(STRINGS.chat.toggleWorkspaceAria);
    expect(markup).toContain(STRINGS.chat.reopen);
  });
});

describe('snapshot truth banner (S2 invisible-state fix)', () => {
  it('renders nothing for idle/ready snapshots', () => {
    expect(snapshotBannerTone('failed')).toBe('signal');
    expect(snapshotBannerTone('degraded')).toBe('amber');
    expect(snapshotBannerTone('loading')).toBe('amber');
    expect(renderWith(createElement(SnapshotBanner, { snapshot: snapshot({ status: 'ready' }), onResync: () => undefined }))).toBe('');
    expect(renderWith(createElement(SnapshotBanner, { snapshot: snapshot({ status: 'idle' }), onResync: () => undefined }))).toBe('');
  });

  it('lists incomplete sources, attempts, timestamp and the existing Re-sync action', () => {
    const markup = renderWith(
      createElement(SnapshotBanner, { snapshot: snapshot(), onResync: () => undefined }),
    );
    expect(markup).toContain('data-testid="snapshot-banner"');
    expect(markup).toContain(`aria-label="${STRINGS.snapshot.aria}"`);
    expect(markup).toContain(STRINGS.snapshot.degraded);
    expect(markup).toContain('messages-unavailable');
    expect(markup).toContain(`${STRINGS.snapshot.attempts} 2`);
    expect(markup).toContain('2026-11-10T20:00:00.000Z');
    expect(markup).toContain(STRINGS.snapshot.resync);
    expect(markup).not.toContain('disabled=""');
  });

  it('marks a failed snapshot with the signal tone and disables re-sync while loading', () => {
    const failed = renderWith(
      createElement(SnapshotBanner, { snapshot: snapshot({ status: 'failed' }), onResync: () => undefined }),
    );
    expect(failed).toContain(STRINGS.snapshot.failed);
    expect(failed).toContain('border-signal');

    const loading = renderWith(
      createElement(SnapshotBanner, { snapshot: snapshot({ status: 'loading' }), onResync: () => undefined }),
    );
    expect(loading).toContain(STRINGS.snapshot.resyncing);
    expect(loading).toContain('disabled=""');
  });

  it('App mounts the banner above content and re-syncs through the existing store action', () => {
    const appSource = sourceOf('/App.tsx');
    expect(appSource).toContain('<SnapshotBanner');
    expect(appSource).toContain("refreshRunSnapshot('manual')");
    expect(sourceOf('/panels/SnapshotBanner.tsx')).toContain('isSnapshotDegraded');
  });
});

describe('approval queue (S2 unreachable-approvals fix)', () => {
  it('clamps the cursor and labels the queue position', () => {
    expect(clampApprovalIndex(0, 0)).toBe(0);
    expect(clampApprovalIndex(-1, 3)).toBe(0);
    expect(clampApprovalIndex(5, 3)).toBe(2);
    expect(clampApprovalIndex(1.5, 3)).toBe(0);
    expect(approvalQueueLabel(0, 3)).toBe(`${STRINGS.approval.queue.label} 1 ${STRINGS.approval.queue.of} 3`);
    expect(approvalQueueLabel(9, 3)).toBe(`${STRINGS.approval.queue.label} 3 ${STRINGS.approval.queue.of} 3`);
  });

  it('renders one dialog with Previous/Next for every pending approval', () => {
    const markup = renderWith(createElement(ApprovalOverlay), {
      approvals: [
        approval({ approvalId: 'ap-1', tool: 'file_write' }),
        approval({ approvalId: 'ap-2', tool: 'file_read' }),
        approval({ approvalId: 'ap-3', tool: 'web_fetch' }),
      ],
    });
    expect(markup).toContain('data-testid="approval-queue"');
    expect(markup).toContain(approvalQueueLabel(0, 3));
    expect(markup).toContain(STRINGS.approval.queue.previousShort);
    expect(markup).toContain(STRINGS.approval.queue.nextShort);
    // Only Previous is disabled on the first record; the actions stay enabled.
    expect((markup.match(/disabled=""/g) ?? []).length).toBe(1);
    expect(markup).toContain(STRINGS.approval.queue.previous);
    expect(markup).toContain(STRINGS.approval.queue.next);

    // Navigation updates one dialog in place so the existing focus restore/lock
    // (previousFocusRef + Tab trap) survives Previous/Next.
    const source = sourceOf('/panels/ApprovalOverlay.tsx');
    expect(source).toContain('key="approval-dialog"');
    expect(source).toContain('previousFocusRef');
  });

  it('drops the toast stack below the blocking dialog while approvals are pending', () => {
    expect(toastStackZClass(0)).toBe('z-50');
    expect(toastStackZClass(3)).toBe('z-30');
    const source = sourceOf('/panels/ToastRegion.tsx');
    expect(source).toContain('toastStackZClass');
    expect(source).toContain('data-approval-priority');
  });
});

describe('Agents SPAWN wiring (S1 replay-reason lie fix)', () => {
  it('passes spawnTask and the real spawnGate reason on a live run', () => {
    const taskRecord = task({ id: 'root', kind: 'hyperagent', label: 'Root', state: 'running' });
    const markup = renderWith(createElement(AgentsPanel), {
      tasks: { root: taskRecord },
      run: { id: 'run-1', status: 'running', errorCode: null },
      selectedTaskId: null,
      pendingCommands: {},
      history: {
        runs: [],
        latest: -1,
        count: 0,
        revisions: [],
        selected: 0,
        replayGraph: null,
        refreshStatus: 'idle',
        refreshRefusal: null,
      },
    });
    expect(markup).toContain('data-spawn-parent="root"');
    expect(markup).not.toContain('Replay is read-only');
    expect(markup).toContain('Spawn a superagent under Root');
  });

  it('keeps the honest replay reason while a committed revision is replayed', () => {
    const taskRecord = task({ id: 'root', kind: 'hyperagent', label: 'Root', state: 'running' });
    const markup = renderWith(createElement(AgentsPanel), {
      tasks: { root: taskRecord },
      run: { id: 'run-1', status: 'running', errorCode: null },
      selectedTaskId: null,
      pendingCommands: {},
      history: {
        runs: [],
        latest: 1,
        count: 1,
        revisions: [1],
        selected: 1,
        replayGraph: { revision: 1, graph: null },
        refreshStatus: 'ok',
        refreshRefusal: null,
      },
    });
    expect(markup).toContain('Replay is read-only. Exit replay to spawn a child.');
  });
});

describe('remaining WP-UI-1 write/auth/focus repairs', () => {
  it('sends proposal writes with x-hivemind-token, not Authorization: Bearer', () => {
    expect(proposalWriteHeaders('token-1')).toEqual({
      Accept: 'application/json',
      'x-hivemind-token': 'token-1',
    });
    expect(proposalWriteHeaders(null)).toEqual({ Accept: 'application/json' });
    expect(JSON.stringify(proposalWriteHeaders('token-1'))).not.toContain('Authorization');

    const source = sourceOf('/panels/ProposalsPanel.tsx');
    expect((source.match(/proposalWriteHeaders\(sessionToken\)/g) ?? []).length).toBe(2);
    expect(source).not.toContain('Authorization: `Bearer');
  });

  it('refetches memory entries per run with an AbortController and no stale rows', () => {
    const source = sourceOf('/panels/MemoryBanksPanel.tsx');
    expect(source).toContain('new AbortController()');
    expect(source).toMatch(/\}, \[selected, runId\]\);/);
    expect(source).toContain('signal: controller.signal');
    expect(source).toContain('setEntries([]);');
  });

  it('routes the kill confirmation through the shared ConfirmDialog', () => {
    const source = sourceOf('/panels/ControlBar.tsx');
    expect(source).toContain("from './ConfirmDialog'");
    expect(source).toContain('<ConfirmDialog');
    expect(source).toContain('confirmLabel={STRINGS.controls.kill}');
    expect(source).toContain('danger');
    expect(source).not.toContain('aria-labelledby="kill-confirm-title"');
  });

  it('removed the synthetic FigCraft gallery fallback and the NaN tab index path', () => {
    const source = sourceOf('/figcraft/FigCraftGallery.tsx');
    expect(source).not.toContain('FIRST_GROUP');
    expect(source).toContain('FIGCRAFT_GROUPS[0]');
    expect(source).toContain('No accepted FigCraft asset sets');
    expect(source).toContain('const count = FIGCRAFT_GROUPS.length');
    expect(source).toContain('if (count === 0)');
  });

  it('splash.css states the token palette + literal rgb alphas instead of a false claim', () => {
    const splashCss = readWebText('../src/styles/splash.css');
    expect(splashCss).not.toContain('Every color is a FigCraft token');
    expect(splashCss).toContain('Token palette');
    expect(splashCss).toContain('literal `rgb()` values');
  });

  it('LibraryPanel reads __APP_VERSION__/package.json instead of a hardcoded version', () => {
    const source = sourceOf('/panels/LibraryPanel.tsx');
    expect(source).toContain('APP_VERSION');
    expect(source).not.toContain('v1.2-dev');
    expect(sourceOf('/version.ts')).toContain('__APP_VERSION__');
  });
});
