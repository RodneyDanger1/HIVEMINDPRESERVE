/**
 * S22 Data: FILE_TREE_LIST / FILE_READ against the patched contract, jailed client-side too.
 *
 * FILE_TREE_LIST carries no path argument, so the tree can only be listed at the
 * workspace root. Directory rows are therefore rendered as non-actionable rows with
 * honest copy instead of dispatching FILE_READ against a directory (which the tool
 * layer refuses with E_TOOL_SCHEMA); only regular files dispatch FILE_READ.
 */
import { useEffect } from 'react';
import { useUiStore } from '../state/context';
import { STRINGS } from '../state/strings';
import { PanelTitle, EmptyState, LoadingState, ErrorState } from './StatePack';

export function DataPanel() {
  const fileTree = useUiStore((state) => state.fileTree);
  const requestFileTree = useUiStore((state) => state.requestFileTree);
  const openFile = useUiStore((state) => state.openFile);

  useEffect(() => {
    if (fileTree.status === 'idle') {
      void requestFileTree();
    }
  }, [fileTree.status, requestFileTree]);

  const hasDirectories = fileTree.entries.some((entry) => entry.kind === 'dir');

  return (
    <div className="grid h-full min-h-0 grid-cols-[280px_1fr] gap-3 p-4">
      <section aria-label={STRINGS.panels.dataTitle} className="min-h-0 overflow-y-auto">
        <PanelTitle>{STRINGS.panels.dataTitle}</PanelTitle>
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
        {fileTree.status === 'ok' && hasDirectories ? (
          <p className="type-mono-meta mb-2 mt-2 text-meta">{STRINGS.panels.dataUnavailable}</p>
        ) : null}
        <ul className="space-y-1">
          {fileTree.entries.map((entry) => {
            if (entry.kind === 'dir') {
              // No path-scoped FILE_TREE_LIST exists in the frozen contract, so a
              // directory cannot expand here. It is a row, not a FILE_READ target.
              return (
                <li key={entry.path}>
                  <div
                    aria-disabled="true"
                    title={STRINGS.panels.dataUnavailable}
                    className="type-mono-code w-full cursor-not-allowed truncate rounded-[var(--radius-sm)] border border-hairline bg-panel px-2 py-1 text-left text-ink-500"
                  >
                    {entry.name}
                    <span className="type-mono-meta ml-2 text-meta">
                      / · {STRINGS.states.unavailable}
                    </span>
                  </div>
                </li>
              );
            }
            return (
              <li key={entry.path}>
                <button
                  type="button"
                  className={
                    'type-mono-code w-full truncate rounded-[var(--radius-sm)] border px-2 py-1 text-left ' +
                    (fileTree.selectedPath === entry.path
                      ? 'border-honey-400 bg-honey-100'
                      : 'border-hairline bg-panel hover:bg-panel-alt')
                  }
                  onClick={() => {
                    void openFile(entry.path);
                  }}
                >
                  {entry.name}
                  <span className="type-mono-meta ml-2 text-meta">
                    {entry.size_bytes === null ? '—' : `${entry.size_bytes} B`}
                  </span>
                </button>
              </li>
            );
          })}
        </ul>
      </section>
      <section aria-label={STRINGS.panels.dataTitle} className="min-h-0 overflow-auto">
        {fileTree.contentStatus === 'loading' ? <LoadingState /> : null}
        {fileTree.contentStatus === 'error' ? <ErrorState message={STRINGS.states.unavailable} /> : null}
        {fileTree.content === null && fileTree.contentStatus === 'idle' ? (
          <EmptyState message={STRINGS.panels.dataPreviewHint} />
        ) : null}
        {fileTree.content ? (
          <div className="surface-panel rounded-[var(--radius-md)] p-3">
            <div className="type-mono-meta text-meta">
              {fileTree.content.path} · {fileTree.content.bytes} B · sha {fileTree.content.sha256.slice(0, 12)}…
            </div>
            <pre className="type-mono-code mt-2 overflow-x-auto whitespace-pre-wrap text-ink-900">
              {fileTree.content.text}
            </pre>
          </div>
        ) : null}
      </section>
    </div>
  );
}
