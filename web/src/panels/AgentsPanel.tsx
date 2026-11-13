/**
 * S15-S19 agent registry (read/derived): one card per task, single state vocabulary.
 *
 * W12: the agents surface also hosts the tool-activity region. The store owns
 * `toolActivity`/`approvals` (server frames only); this panel passes the entries
 * to the panel owner's `ToolActivityPanel` and exposes a labelled region so
 * keyboard/screen-reader users find the activity list. No optimistic rows are
 * created here and this panel performs no store writes.
 */
import { useMemo } from 'react';
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
import { JobsPanel } from './JobsPanel';
import { SessionTodos } from './SessionTodos';
import { Timeline } from '../workbench';
import type { TimelineAgent, TimelineStatus, TimelineStep } from '../workbench';
import type { ConnectionStatus } from '../api/client';

/**
 * The timeline is fed only by live server frames. It is therefore "ready" when
 * the socket is open, or when it still holds steps recorded before a drop; an
 * empty timeline on a dead connection is idle/loading, never a false "no
 * events were recorded for this run".
 */
export function timelineStatusForConnection(
  connection: ConnectionStatus,
  recordedSteps: number,
): TimelineStatus {
  if (connection === 'open' || recordedSteps > 0) {
    return 'ready';
  }
  return connection === 'connecting' || connection === 'reconnecting' ? 'loading' : 'idle';
}

function timelineConnectionNote(connection: ConnectionStatus): string | null {
  switch (connection) {
    case 'connecting':
      return 'Connecting — the timeline will resume when the socket opens.';
    case 'reconnecting':
      return 'Reconnecting — showing the last recorded timeline.';
    case 'closed':
    case 'error':
      return 'Disconnected — showing the last recorded timeline.';
    default:
      return null;
  }
}

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
  const list = useMemo(() => Object.values(tasks), [tasks]);
  const timelineAgents: readonly TimelineAgent[] = useMemo(
    () =>
      list.map((task) => ({
        id: task.id,
        label: task.label,
        kind: task.kind,
      })),
    [list],
  );
  const timelineSteps: readonly TimelineStep[] = useMemo(() => toolActivity.map((record) => ({
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
          : record.outcome === 'cancelled'
            ? 'skipped'
            : 'failed',
    kind: 'tool',
  })), [toolActivity]);
  const timelineStatus = timelineStatusForConnection(connection, timelineSteps.length);
  const timelineNote =
    timelineSteps.length > 0 ? timelineConnectionNote(connection) : null;
  // Never claim a run recorded nothing when there is no run at all.
  const timelineEmptyMessage =
    run === null
      ? 'No run yet — the timeline appears here once a run records tool calls.'
      : 'No tool-call timeline has been recorded for this run.';

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
        // One labelled landmark: ToolActivityPanel already renders the
        // "Tool activity" section. A second role="region" wrapper here made
        // screen readers announce the same activity list twice, and its label
        // promised approvals that this wrapper never contains.
        data-testid="agents-tool-activity"
        className="surface-panel mt-3 rounded-[var(--radius-card)] p-3"
      >
        <ToolActivityPanel entries={toolActivity} />
      </div>
      <div className="mt-3">
        {timelineNote === null ? null : (
          <p role="status" className="type-mono-meta mb-2 text-meta">
            {timelineNote}
          </p>
        )}
        <Timeline
          agents={timelineAgents}
          steps={timelineSteps}
          status={timelineStatus}
          emptyMessage={timelineEmptyMessage}
        />
      </div>
      <JobsPanel />
      <SessionTodos />
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
