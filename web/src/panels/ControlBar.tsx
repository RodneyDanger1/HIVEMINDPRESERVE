/**
 * S04-S07 control bar: spawn / re-route / cancel / kill. Every mutating
 * control is disabled unless its precondition is visibly true, and every
 * attempt surfaces ACK-pending or a named refusal (never a silent no-op).
 */
import { useState } from 'react';
import { useUiStore } from '../state/context';
import { STRINGS } from '../state/strings';
import { isRunActive, isTerminal } from '../state/agentState';

export function ControlBar() {
  const run = useUiStore((state) => state.run);
  const tasks = useUiStore((state) => state.tasks);
  const selectedTaskId = useUiStore((state) => state.selectedTaskId);
  const pending = useUiStore((state) => state.pendingCommands);
  const connection = useUiStore((state) => state.connection);
  const spawnSubHive = useUiStore((state) => state.spawnSubHive);
  const cancelRun = useUiStore((state) => state.cancelRun);
  const killWorker = useUiStore((state) => state.killWorker);
  const rewireSelected = useUiStore((state) => state.rewireSelected);
  const selectTask = useUiStore((state) => state.selectTask);
  const replaying = useUiStore((state) => state.history.replayGraph !== null);
  const [target, setTarget] = useState('');

  const selected = selectedTaskId ? tasks[selectedTaskId] : null;
  const runActive = run !== null && isRunActive(run.status);
  const anyPending = Object.keys(pending).length > 0;
  const connected = connection === 'open';
  const parents = Object.values(tasks).filter(
    (task) => (task.kind === 'superagent' || task.kind === 'hyperagent') && task.id !== selectedTaskId,
  );

  const canSpawn = connected && runActive && !anyPending && !replaying;
  const canRewire =
    connected && runActive && !anyPending && !replaying && selected !== null && selected.kind === 'worker' && target !== '';
  const canCancel = connected && runActive && !anyPending && !replaying;
  const canKill =
    connected && runActive && !anyPending && !replaying && selected !== null && selected.kind === 'worker';

  return (
    <div className="flex min-h-[52px] flex-wrap items-center gap-2 border-t border-hairline bg-panel px-3 py-1.5">
      <span className="type-mono-meta text-meta">Target</span>
      <select
        className="type-mono-meta h-9 rounded-[var(--radius-sm)] border border-hairline bg-panel-alt px-2 text-ink-800"
        aria-label={`${STRINGS.node.workerLabel}: ${STRINGS.controls.reroute}`}
        value={selectedTaskId ?? ''}
        disabled={replaying}
        onChange={(event) => selectTask(event.target.value === '' ? null : event.target.value)}
      >
        <option value="">{STRINGS.node.noMetrics}</option>
        {Object.values(tasks).map((task) => (
          <option key={task.id} value={task.id}>
            {task.label}
          </option>
        ))}
      </select>

      <button
        type="button"
        className="type-label h-10 rounded-[var(--radius-button)] border border-honey-500 bg-honey-300 px-4 font-semibold text-ink-900 hover:bg-honey-400 disabled:opacity-50"
        disabled={!canSpawn}
        title={runActive ? STRINGS.controls.spawn : STRINGS.controls.disabledNoRun}
        onClick={() => {
          void spawnSubHive();
        }}
      >
        {STRINGS.controls.spawn}
      </button>

      <select
        className="type-mono-meta h-9 rounded-[var(--radius-sm)] border border-hairline bg-panel-alt px-2 text-ink-800"
        aria-label={STRINGS.controls.reroute}
        value={target}
        onChange={(event) => setTarget(event.target.value)}
      >
        <option value="">{STRINGS.controls.reroute}</option>
        {parents.map((parent) => (
          <option key={parent.id} value={parent.id}>
            {parent.label}
          </option>
        ))}
      </select>
      <button
        type="button"
        className="type-label h-10 rounded-[var(--radius-button)] border border-hairline px-4 text-ink-900 hover:bg-honey-100 disabled:opacity-50"
        disabled={!canRewire}
        title={selected ? STRINGS.controls.reroute : STRINGS.controls.disabledNoSelection}
        onClick={() => {
          if (target !== '') {
            void rewireSelected(target);
          }
        }}
      >
        {STRINGS.controls.reroute}
      </button>

      <button
        type="button"
        className="type-label h-10 rounded-[var(--radius-button)] border border-hairline px-4 text-ink-900 hover:bg-honey-100 disabled:opacity-50"
        disabled={!canCancel}
        onClick={() => {
          void cancelRun();
        }}
      >
        {STRINGS.controls.cancel}
      </button>
      <button
        type="button"
        className="type-label h-10 rounded-[var(--radius-button)] border border-signal px-4 text-signal hover:bg-recessed disabled:opacity-50"
        disabled={!canKill}
        onClick={() => {
          void killWorker();
        }}
      >
        {STRINGS.controls.kill}
      </button>

      <span className="type-mono-meta ml-auto text-meta" aria-live="polite">
        {anyPending
          ? STRINGS.states.loading
          : run
            ? `${run.id.slice(0, 8)} · ${isTerminal(run.status) ? STRINGS.states.unavailable : run.status}`
            : STRINGS.states.empty}
      </span>
    </div>
  );
}
