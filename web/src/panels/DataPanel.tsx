/** S22 Data: FILE_TREE_LIST / FILE_READ against the patched contract, jailed client-side too. */
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

  return (
    <div className="grid h-full min-h-0 grid-cols-[280px_1fr] gap-3 p-4">
      <section aria-label={STRINGS.panels.dataTitle} className="min-h-0 overflow-y-auto">
        <PanelTitle>{STRINGS.panels.dataTitle}</PanelTitle>
        {fileTree.status === 'loading' ? <LoadingState /> : null}
        {fileTree.status === 'error' ? <ErrorState message={STRINGS.panels.dataError} /> : null}
        {fileTree.status === 'ok' && fileTree.entries.length === 0 ? (
          <EmptyState message={STRINGS.states.empty} />
        ) : null}
        <ul className="space-y-1">
          {fileTree.entries.map((entry) => (
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
                  {entry.kind === 'dir' ? '/' : `${entry.size_bytes ?? 0} B`}
                </span>
              </button>
            </li>
          ))}
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
