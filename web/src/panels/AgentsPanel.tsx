/**
 * S15-S19 agent registry (read/derived): one card per task, single state vocabulary.
 *
 * W12: the agents surface also hosts the tool-activity region. The store owns
 * `toolActivity`/`approvals` (server frames only); this panel passes the entries
 * to the panel owner's `ToolActivityPanel` and exposes a labelled region so
 * keyboard/screen-reader users find the activity list. No optimistic rows are
 * created here and this panel performs no store writes.
 */
import { useUiStore } from '../state/context';
import { NODE_KIND_LABELS } from '../state/agentState';
import { spawnGate } from '../state/spawnMatrix';
import { STRINGS, formatPercent } from '../state/strings';
import { StateGlyph } from '../flow/nodes';
import { PanelTitle, EmptyState } from './StatePack';
import { ToolActivityPanel, type ToolActivityEntry } from './ToolActivityPanel';
import { AgentMessagesPanel } from './AgentMessagesPanel';
import { AgentProfilesPanel } from './AgentProfilesPanel';
import { AgentMemoryPanel } from './AgentMemoryPanel';
import { MemoryBanksPanel } from './MemoryBanksPanel';
import { ProposalsPanel } from './ProposalsPanel';
import { LessonsPanel } from './LessonsPanel';
import { AgentHierarchy } from './AgentHierarchy';
import { AgentRosterPanel } from './AgentRosterPanel';
import { A2AInboxPanel } from './A2AInboxPanel';
import { Timeline } from '../workbench';
import type { TimelineAgent, TimelineStep } from '../workbench';

export function AgentsPanel() {
  const tasks = useUiStore((state) => state.tasks);
  const selectTask = useUiStore((state) => state.selectTask);
  const selectedTaskId = useUiStore((state) => state.selectedTaskId);
  const run = useUiStore((state) => state.run);
  const connection = useUiStore((state) => state.connection);
  const pendingCommands = useUiStore((state) => state.pendingCommands);
  const replayGraph = useUiStore((state) => state.history.replayGraph);
  const spawnSubHive = useUiStore((state) => state.spawnSubHive);
  // Structural reconciliation: the store's ToolActivityRecord is assignable to
  // the panel's ToolActivityEntry (same id/tool/hash/turn/outcome fields); the
  // store stays the single writer and this panel only reads.
  const toolActivity: readonly ToolActivityEntry[] = useUiStore((state) => state.toolActivity);
  const list = Object.values(tasks);
  const timelineAgents: readonly TimelineAgent[] = list.map((task) => ({
    id: task.id,
    label: task.label,
    kind: task.kind,
  }));
  const timelineSteps: readonly TimelineStep[] = toolActivity.map((record) => ({
    id: record.toolCallId,
    // Empty means the wire did not carry a timestamp; Timeline renders it as
    // unknown and sorts it last instead of inventing a time.
    at: record.at ?? '',
    agentId: record.taskId ?? 'unattributed',
    title: `${record.tool} · turn ${record.turn}`,
    detail:
      record.code === undefined
        ? `outcome: ${record.outcome}`
        : `outcome: ${record.outcome} · code: ${record.code}`,
    evidence: record.sha256 === undefined || record.sha256 === null
      ? null
      : `sha256 ${record.sha256.slice(0, 12)}…`,
    status:
      record.outcome === 'pending'
        ? 'running'
        : record.outcome === 'ok'
          ? 'ok'
          : record.outcome === 'denied'
            ? 'awaiting_approval'
            : 'failed',
    kind: 'tool',
  }));

  return (
    <div className="h-full min-h-0 overflow-y-auto p-4">
      <PanelTitle>
        {STRINGS.panels.agentsTitle} · {list.length}
      </PanelTitle>
      {list.length === 0 ? <EmptyState message={STRINGS.panels.agentsEmpty} /> : null}
      {list.length > 0 ? (
        // Live Agents surface: the hierarchy must carry the real spawn handler
        // and the real gate reason. Mounting it without one made every SPAWN
        // button claim "Replay is read-only" even during an active run.
        <AgentHierarchy
          tasks={tasks}
          selectedTaskId={selectedTaskId}
          selectTask={selectTask}
          spawnTask={
            replayGraph === null ? (taskId) => void spawnSubHive(taskId) : undefined
          }
          spawnReason={(task) =>
            spawnGate({
              parent: { id: task.id, kind: task.kind },
              runStatus: run?.status ?? null,
              connection,
              replaying: replayGraph !== null,
              pending: Object.keys(pendingCommands).length > 0,
            }).reason
          }
        />
      ) : null}
      <ul className="grid grid-cols-1 gap-2 sm:grid-cols-2 lg:grid-cols-3">
        {list.map((task) => (
          <li key={task.id}>
            <button
              type="button"
              aria-pressed={task.id === selectedTaskId}
              onClick={() => selectTask(task.id)}
              className={
                'surface-panel block w-full rounded-[var(--radius-card)] p-2 text-left ' +
                (task.id === selectedTaskId ? 'border-honey-500' : '')
              }
            >
              <div className="flex items-center justify-between gap-2">
                <span className="type-label text-ink-500">{NODE_KIND_LABELS[task.kind]}</span>
                <StateGlyph state={task.state} />
              </div>
              <div className="type-mono-code mt-1 truncate text-ink-900">{task.label}</div>
              <div className="type-mono-meta mt-1 flex justify-between text-meta">
                <span>{task.id.slice(0, 8)}</span>
                <span>{task.progress === null ? STRINGS.node.noMetrics : formatPercent(task.progress, 0)}</span>
              </div>
            </button>
          </li>
        ))}
      </ul>
      <div
        role="region"
        aria-label={STRINGS.toolActivity.regionAria}
        data-testid="agents-tool-activity"
        className="surface-panel mt-3 rounded-[var(--radius-card)] p-3"
      >
        <ToolActivityPanel entries={toolActivity} />
      </div>
      <div className="mt-3">
        <Timeline
          agents={timelineAgents}
          steps={timelineSteps}
          status="ready"
          emptyMessage="No tool-call timeline has been recorded for this run."
        />
      </div>
      <AgentProfilesPanel />
      <AgentRosterPanel />
      <A2AInboxPanel />
      <ProposalsPanel />
      <LessonsPanel />
      <AgentMemoryPanel />
      <MemoryBanksPanel />
      <AgentMessagesPanel />
    </div>
  );
}
