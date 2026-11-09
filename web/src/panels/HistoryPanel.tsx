/**
 * History view: committed revisions are the authority. The panel only asks the
 * store to load a revision; replay itself is read-only and owned by the store.
 */
import { useUiStore } from '../state/context';
import { isRunActive } from '../state/agentState';
import { STRINGS } from '../state/strings';
import { EmptyState, PanelTitle } from './StatePack';
import { RunDetailPanel } from './RunDetailPanel';
import { RunTranscriptPanel } from './RunTranscriptPanel';

export function HistoryPanel() {
  const history = useUiStore((state) => state.history);
  const run = useUiStore((state) => state.run);
  const setHistoryRevision = useUiStore((state) => state.setHistoryRevision);
  const selectHistoryRun = useUiStore((state) => state.selectHistoryRun);
  const exitHistoryReplay = useUiStore((state) => state.exitHistoryReplay);
  const refreshRunSnapshot = useUiStore((state) => state.refreshRunSnapshot);
  const replaying = history.replayGraph !== null;
  const revisions = history.revisions;
  const refreshFailure = history.refreshRefusal;

  return (
    <aside className="flex h-full w-[300px] shrink-0 flex-col border-r border-hairline bg-panel p-3" aria-label="Committed run history">
      <PanelTitle>{STRINGS.tabs.history}</PanelTitle>
      {history.runs.length > 1 ? (
        <label className="mb-2 block">
          <span className="type-mono-meta block text-meta">Run</span>
          <select
            className="mt-1 h-8 w-full rounded-[var(--radius-sm)] border border-hairline bg-panel px-2 type-mono-code text-ink-900 disabled:opacity-50"
            value={run?.id ?? ''}
            disabled={run !== null && isRunActive(run.status)}
            title={
              run !== null && isRunActive(run.status)
                ? 'Finish or cancel the active run before viewing another run'
                : 'Select a committed run'
            }
            onChange={(event) => {
              void selectHistoryRun(event.target.value);
            }}
          >
            {history.runs.map((candidate) => (
              <option key={candidate.runId} value={candidate.runId}>
                {candidate.runId.slice(0, 8)} · {candidate.status}
              </option>
            ))}
          </select>
        </label>
      ) : null}
      {refreshFailure === null ? null : (
        <div
          role="alert"
          className="mb-2 rounded-[var(--radius-sm)] border border-signal bg-panel-alt p-2"
        >
          <div className="type-mono-meta text-signal">{refreshFailure.code}</div>
          <p className="type-body-l mt-1 text-ink-900">{refreshFailure.message}</p>
          <button
            type="button"
            className="type-label mt-2 rounded-[var(--radius-sm)] border border-hairline px-2 py-1 text-ink-900 hover:bg-honey-100"
            onClick={() => {
              void refreshRunSnapshot('manual');
            }}
          >
            Retry history read
          </button>
        </div>
      )}
      {run === null ? (
        <EmptyState message="No run yet. Start a run in Workspace; committed revisions will appear here." />
      ) : revisions.length === 0 ? (
        <EmptyState message="No committed revisions yet. Replay becomes available after the run commits a revision." />
      ) : (
        <>
          <div className="type-mono-meta mb-2 flex items-center justify-between text-meta">
            <span>{run.id.slice(0, 8)}</span>
            <span>{revisions.length} revisions</span>
          </div>
          <ol className="min-h-0 flex-1 overflow-y-auto">
            {revisions.map((revision) => {
              const selected = revision === history.selected;
              return (
                <li key={revision}>
                  <button
                    type="button"
                    aria-pressed={selected}
                    className={
                      'mb-1 flex w-full items-center justify-between rounded-[var(--radius-sm)] border px-2 py-1.5 text-left ' +
                      (selected
                        ? 'border-honey-400 bg-honey-100 text-ink-900'
                        : 'border-hairline bg-panel text-ink-700 hover:bg-panel-alt')
                    }
                    onClick={() => {
                      void setHistoryRevision(revision);
                    }}
                  >
                    <span className="type-mono-meta text-[9px]">rev {revision}</span>
                    <span className="type-mono-meta text-[8px] text-meta">
                      {revision === history.latest ? 'LATEST' : 'COMMITTED'}
                    </span>
                  </button>
                </li>
              );
            })}
          </ol>
          <button
            type="button"
            className="type-label mt-2 h-9 rounded-[var(--radius-sm)] border border-hairline px-3 text-ink-900 hover:bg-honey-100 disabled:opacity-50"
            onClick={() => exitHistoryReplay()}
            disabled={!replaying}
          >
            {STRINGS.scrubber.live}
          </button>
        </>
      )}
      <RunDetailPanel />
      <RunTranscriptPanel />
    </aside>
  );
}
