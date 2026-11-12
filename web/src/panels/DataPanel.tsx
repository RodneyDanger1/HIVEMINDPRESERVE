/**
 * S22 Data: FILE_TREE_LIST / FILE_READ against the patched contract, jailed client-side too.
 *
 * FILE_TREE_LIST accepts an optional path. Directory rows expand in place; only
 * regular files dispatch FILE_READ.
 */
import { useEffect } from 'react';
import { useUiStore } from '../state/context';
import { STRINGS } from '../state/strings';
import { Editor, FileTree, buildFileTreeNodes } from '../workbench';
import type { FileTreeStatus } from '../workbench';
import { PanelTitle, EmptyState, LoadingState, ErrorState } from './StatePack';

const FILE_TREE_STATUS: Record<string, FileTreeStatus> = {
  idle: 'idle',
  loading: 'loading',
  ok: 'ready',
  error: 'error',
};

export function DataPanel() {
  const fileTree = useUiStore((state) => state.fileTree);
  const requestFileTree = useUiStore((state) => state.requestFileTree);
  const openFile = useUiStore((state) => state.openFile);

  useEffect(() => {
    if (fileTree.status === 'idle') {
      void requestFileTree();
    }
  }, [fileTree.status, requestFileTree]);

  return (
    <div className="grid h-full min-h-0 grid-cols-[280px_1fr] gap-3 p-4">
      <section aria-label={STRINGS.panels.dataTitle} className="min-h-0 overflow-y-auto">
        <PanelTitle>{STRINGS.panels.dataTitle}</PanelTitle>
        {fileTree.status === 'ok' ? (
          <button
            type="button"
            className="type-mono-meta mb-2 rounded-[var(--radius-sm)] border border-hairline px-2 py-1 text-ink-800 hover:bg-honey-100"
            onClick={() => {
              void requestFileTree();
            }}
          >
            {STRINGS.controls.refresh} root
          </button>
        ) : null}
        {fileTree.status === 'loading' ? <LoadingState /> : null}
        {fileTree.status === 'error' ? (
          <div className="space-y-2">
            <ErrorState message={STRINGS.panels.dataError} />
            {/*
              M-020: the copy names Refresh, so the named control must exist and
              re-run the same request path. requestFileTree is allowed from the
              error state (it sets loading, then sends FILE_TREE_LIST again).
            */}
            <button
              type="button"
              className="type-mono-meta rounded-[var(--radius-sm)] border border-hairline px-2 py-1 text-ink-800 hover:bg-honey-100"
              onClick={() => {
                void requestFileTree();
              }}
            >
              {STRINGS.controls.refresh}
            </button>
          </div>
        ) : null}
        {fileTree.status === 'ok' && fileTree.entries.length === 0 ? (
          <EmptyState message={STRINGS.states.empty} />
        ) : null}
        <FileTree
          nodes={buildFileTreeNodes(fileTree.entries)}
          selectedPath={fileTree.selectedPath}
          root={STRINGS.panels.dataTitle}
          status={FILE_TREE_STATUS[fileTree.status] ?? 'idle'}
          errorMessage={fileTree.status === 'error' ? STRINGS.panels.dataError : null}
          onSelectFile={(path) => {
            void openFile(path);
          }}
          onToggleDir={(path) => {
            void requestFileTree(path);
          }}
          onRefresh={() => {
            void requestFileTree();
          }}
          expandOnToggle={false}
        />
      </section>
      <section aria-label={STRINGS.panels.dataTitle} className="min-h-0 overflow-auto">
        {fileTree.contentStatus === 'loading' ? <LoadingState /> : null}
        {fileTree.contentStatus === 'error' ? <ErrorState message={STRINGS.states.unavailable} /> : null}
        {fileTree.content === null && fileTree.contentStatus === 'idle' ? (
          <EmptyState message={STRINGS.panels.dataPreviewHint} />
        ) : null}
        {fileTree.content ? (
          <Editor
            path={fileTree.content.path}
            value={fileTree.content.text}
            readOnly
            readOnlyReason="Workspace reads are preview-only here; approved mutations run through the agent tool path."
            status={fileTree.contentStatus === 'error' ? 'error' : 'ready'}
            errorMessage={fileTree.contentStatus === 'error' ? STRINGS.states.unavailable : null}
            onRetry={() => {
              void openFile(fileTree.content?.path ?? '');
            }}
          />
        ) : null}
      </section>
    </div>
  );
}
