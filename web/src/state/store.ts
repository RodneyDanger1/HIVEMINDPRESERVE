/**
 * The one UI store: run/task graph, chat stream, approvals/history/settings/
 * file state, connection surface, refusals and pending commands. Mutating UI
 * state is applied only from server events (ACK-before-mutation); every
 * failure is a named refusal toast, never a silent drop or a fake success.
 */
import { createStore, type StoreApi } from 'zustand/vanilla';
import type {
  CommandPayloads,
  FileContentPayload,
  FileTreeEntry,
  Refusal,
  RunMetricsPayload,
  ServerCommandName,
} from '../api/contract';
import {
  isRunActive,
  isTerminal,
  type AgentState,
  type NodeKindLabel,
} from './agentState';
import { isNodeKind, validatePromptText, validateWorkspacePath } from '../api/contract';
import { shortId, uuidv7 } from '../api/ids';
import { RefusalError, clientRefusal } from '../api/refusals';
import type { ConnectionStatus, SessionInfo, UiApi } from '../api/client';
import { fetchHealth } from '../api/health';
import type { DecodedEvent } from '../api/contract';
import { deriveHud } from './metrics';
import { formatStateOf, STRINGS } from './strings';

export type NavSection = 'workspace' | 'agents' | 'data' | 'models' | 'routing' | 'settings';
export type TabId = 'workspace' | 'swarm' | 'routing' | 'history';
export type ToastRefusal = { readonly id: string; readonly refusal: Refusal; readonly at: number };
export type PendingCommand = { readonly name: ServerCommandName; readonly startedAt: number };

export interface TaskRecord {
  readonly id: string;
  readonly parentId: string | null;
  readonly kind: NodeKindLabel;
  readonly label: string;
  readonly state: AgentState;
  readonly progress: number | null;
  readonly errorCode: string | null;
  readonly rev: number;
}

export interface ChatMessage {
  readonly id: string;
  readonly role: 'user' | 'assistant';
  readonly text: string;
  readonly runId: string | null;
  readonly streaming: boolean;
}

export interface ApprovalRecord {
  readonly approvalId: string;
  readonly tool: string;
  readonly nonce: string | null;
  readonly decision: string | null;
  readonly requestedAt: number;
}

export interface RunRecord {
  readonly id: string;
  readonly status: AgentState;
  readonly errorCode: string | null;
}

export interface ReplayGraph {
  readonly revision: number;
  readonly graph: unknown;
}

export interface SettingsState {
  readonly status: 'unknown' | 'pending' | 'ok' | 'error';
  readonly provider: string | null;
  readonly model: string | null;
  readonly keyPresent: boolean | null;
  readonly thinking: string | null;
  readonly reasoningEffort: string | null;
  readonly refusal: Refusal | null;
  readonly health: 'unknown' | 'ok' | 'degraded' | 'failing';
  readonly healthVersion: string | null;
  readonly checks: readonly string[];
  readonly lastProbe: number | null;
}

export interface FileTreeState {
  readonly status: 'idle' | 'loading' | 'ok' | 'error';
  readonly root: string | null;
  readonly entries: readonly FileTreeEntry[];
  readonly refusal: Refusal | null;
  readonly contentStatus: 'idle' | 'loading' | 'ok' | 'error';
  readonly content: FileContentPayload | null;
  readonly selectedPath: string | null;
}

export interface ViewportState {
  readonly x: number;
  readonly y: number;
  readonly zoom: number;
}

export interface UiState {
  readonly initialized: boolean;
  readonly apiKind: 'ws' | 'mock';
  readonly connection: ConnectionStatus;
  readonly connectionRefusal: Refusal | null;
  readonly session: SessionInfo | null;
  readonly lastSeq: number;
  readonly run: RunRecord | null;
  readonly tasks: Readonly<Record<string, TaskRecord>>;
  readonly graphRev: number;
  readonly selectedTaskId: string | null;
  readonly messages: readonly ChatMessage[];
  readonly streamingRunId: string | null;
  readonly metrics: RunMetricsPayload | null;
  readonly history: {
    readonly latest: number;
    readonly selected: number;
    readonly replayGraph: ReplayGraph | null;
  };
  readonly approvals: readonly ApprovalRecord[];
  readonly refusals: readonly ToastRefusal[];
  readonly pendingCommands: Readonly<Record<string, PendingCommand>>;
  readonly view: { readonly nav: NavSection; readonly tab: TabId };
  readonly viewport: ViewportState;
  readonly chat: { readonly pinned: boolean; readonly visible: boolean };
  readonly settings: SettingsState;
  readonly fileTree: FileTreeState;
  readonly project: { readonly id: string; readonly name: string; readonly owner: string };
  readonly reducedMotionOverride: boolean;
  readonly diffOpen: boolean;

  init(): Promise<void>;
  dispose(): void;
  submitPrompt(text: string): Promise<void>;
  spawnSubHive(): Promise<void>;
  rewireSelected(newParentId: string): Promise<void>;
  cancelRun(): Promise<void>;
  killWorker(): Promise<void>;
  resolveApproval(approvalId: string, decision: 'approve' | 'deny', reason?: string): Promise<void>;
  pinChat(pinned: boolean): Promise<void>;
  insertTemplate(templateId: string): Promise<void>;
  setHistoryRevision(revision: number): Promise<void>;
  setNav(nav: NavSection): void;
  setTab(tab: TabId): void;
  setViewport(viewport: ViewportState): void;
  selectTask(taskId: string | null): void;
  setChatVisible(visible: boolean): void;
  openDiffModal(): void;
  closeDiffModal(): void;
  dismissRefusal(id: string): void;
  setReducedMotionOverride(value: boolean): void;
  probeSettings(): Promise<void>;
  probeHealth(): Promise<void>;
  requestFileTree(): Promise<void>;
  openFile(path: string): Promise<void>;
  formatSelectedRevision(): string;
}

const HEALTHY_GREEN: readonly AgentState[] = ['healthy', 'completed'];

function baseProject(): { id: string; name: string; owner: string } {
  return {
    id: uuidv7(),
    name: STRINGS.library.projectFallback,
    owner: STRINGS.chat.userLabel,
  };
}

function initialSettings(): SettingsState {
  return {
    status: 'unknown',
    provider: null,
    model: null,
    keyPresent: null,
    thinking: null,
    reasoningEffort: null,
    refusal: null,
    health: 'unknown',
    healthVersion: null,
    checks: [],
    lastProbe: null,
  };
}

function initialFileTree(): FileTreeState {
  return {
    status: 'idle',
    root: null,
    entries: [],
    refusal: null,
    contentStatus: 'idle',
    content: null,
    selectedPath: null,
  };
}

export function createUiStore(api: UiApi): StoreApi<UiState> {
  const unsubscribes: Array<() => void> = [];

  return createStore<UiState>()((set, get) => {
    const pushRefusal = (refusal: Refusal): void => {
      const current = get().refusals;
      const dedupe = current.filter(
        (entry) => entry.refusal.code !== refusal.code || Date.now() - entry.at > 500,
      );
      set({
        refusals: [
          { id: uuidv7(), refusal, at: Date.now() },
          ...dedupe,
        ].slice(0, 4),
      });
    };

    const applyEvent = (event: DecodedEvent): void => {
      switch (event.name) {
        case 'ACK':
          return;
        case 'ERROR': {
          pushRefusal(event.payload);
          if (event.payload.code === 'E_KEY_MISSING') {
            set({ settings: { ...get().settings, keyPresent: false } });
          }
          return;
        }
        case 'SERVER_HELLO': {
          set({ lastSeq: event.payload.last_seq ?? get().lastSeq, connection: 'open' });
          return;
        }
        case 'RESYNC_REQUIRED': {
          set({ connection: 'reconnecting' });
          return;
        }
        case 'RUN_CREATED': {
          set({
            run: { id: event.payload.run_id, status: event.payload.status, errorCode: null },
            graphRev: get().graphRev + 1,
          });
          return;
        }
        case 'RUN_STATUS': {
          const run = get().run;
          if (!run || run.id !== event.payload.run_id) {
            return;
          }
          set({
            run: {
              id: run.id,
              status: event.payload.status,
              errorCode: event.payload.error_code ?? null,
            },
            streamingRunId: isTerminal(event.payload.status) ? null : get().streamingRunId,
            graphRev: get().graphRev + 1,
          });
          return;
        }
        case 'REPLY_DELTA': {
          const runId = event.payload.run_id;
          const messages = [...get().messages];
          const lastIndex = messages.length - 1;
          const last = messages[lastIndex];
          if (last && last.role === 'assistant' && last.runId === runId && last.streaming) {
            messages[lastIndex] = { ...last, text: last.text + event.payload.text_delta };
          } else {
            messages.push({
              id: uuidv7(),
              role: 'assistant',
              text: event.payload.text_delta,
              runId,
              streaming: true,
            });
          }
          set({ messages, streamingRunId: runId });
          return;
        }
        case 'REPLY_DONE': {
          set({
            messages: get().messages.map((message) =>
              message.role === 'assistant' && message.runId === event.payload.run_id
                ? { ...message, streaming: false }
                : message,
            ),
            streamingRunId: null,
          });
          return;
        }
        case 'NODE_SPAWN':
        case 'NODE_DRAG':
        case 'NODE_REWIRE':
        case 'NODE_STATE': {
          const payload = event.payload;
          const existing = get().tasks[payload.task_id];
          const kind =
            payload.kind !== null && payload.kind !== undefined
              ? payload.kind
              : (existing?.kind ?? 'worker');
          const record: TaskRecord = {
            id: payload.task_id,
            parentId:
              payload.parent_task_id !== null && payload.parent_task_id !== undefined
                ? payload.parent_task_id
                : (existing?.parentId ?? null),
            kind,
            label: payload.label ?? existing?.label ?? `Task ${shortId(payload.task_id)}`,
            state: payload.state,
            progress:
              payload.progress !== null && payload.progress !== undefined
                ? payload.progress
                : (existing?.progress ?? null),
            errorCode: payload.error_code ?? existing?.errorCode ?? null,
            rev: get().graphRev + 1,
          };
          set({
            tasks: { ...get().tasks, [payload.task_id]: record },
            graphRev: get().graphRev + 1,
          });
          return;
        }
        case 'RUN_METRICS': {
          set({ metrics: event.payload });
          return;
        }
        case 'APPROVAL_REQUEST': {
          const approval: ApprovalRecord = {
            approvalId: event.payload.approval_id,
            tool: event.payload.tool,
            nonce: event.payload.nonce ?? null,
            decision: event.payload.decision ?? null,
            requestedAt: Date.now(),
          };
          set({
            approvals: [...get().approvals.filter((a) => a.approvalId !== approval.approvalId), approval],
          });
          return;
        }
        case 'APPROVAL_RESOLVED': {
          set({
            approvals: get().approvals.filter(
              (approval) => approval.approvalId !== event.payload.approval_id,
            ),
          });
          return;
        }
        case 'SETTINGS_STATUS_RESULT': {
          set({
            settings: {
              ...get().settings,
              status: 'ok',
              provider: event.payload.provider,
              model: event.payload.model,
              keyPresent: event.payload.key_present,
              thinking: event.payload.thinking ?? null,
              reasoningEffort: event.payload.reasoning_effort ?? null,
              refusal: null,
              lastProbe: Date.now(),
            },
          });
          return;
        }
        case 'FILE_TREE_RESULT': {
          set({
            fileTree: {
              ...get().fileTree,
              status: 'ok',
              root: event.payload.root,
              entries: event.payload.entries,
              refusal: null,
            },
          });
          return;
        }
        case 'FILE_CONTENT': {
          set({
            fileTree: {
              ...get().fileTree,
              contentStatus: 'ok',
              content: event.payload,
              selectedPath: event.payload.path,
            },
          });
          return;
        }
        case 'HISTORY_SNAPSHOT': {
          set({
            history: {
              latest: Math.max(get().history.latest, event.payload.revision),
              selected: event.payload.revision,
              replayGraph: { revision: event.payload.revision, graph: event.payload.graph },
            },
          });
          return;
        }
        case 'PING':
        case 'PONG':
          return;
      }
    };

    const run = async <N extends ServerCommandName>(
      name: N,
      payload: CommandPayloads[N],
    ): Promise<boolean> => {
      const correlationId = uuidv7();
      set({
        pendingCommands: {
          ...get().pendingCommands,
          [correlationId]: { name, startedAt: Date.now() },
        },
      });
      try {
        await api.send(name, payload, { correlationId });
        return true;
      } catch (error) {
        const refusal =
          error instanceof RefusalError ? error.refusal : clientRefusal('E_WS_DISCONNECTED');
        // Server ERROR frames are already toasted by applyEvent; only local
        // pre-send/transport refusals need a toast here.
        if (refusal.code === 'E_WS_DISCONNECTED' || refusal.code === 'E_PATH_OUTSIDE_JAIL' || refusal.code === 'E_FILE_TOO_LARGE') {
          pushRefusal(refusal);
        }
        if (refusal.code === 'E_KEY_MISSING') {
          set({ settings: { ...get().settings, keyPresent: false } });
        }
        return false;
      } finally {
        const pending = { ...get().pendingCommands };
        delete pending[correlationId];
        set({ pendingCommands: pending });
      }
    };

    const guardActiveRun = (): RunRecord | null => {
      const current = get().run;
      if (!current || !isRunActive(current.status)) {
        pushRefusal(clientRefusal('E_RUN_NOT_FOUND'));
        return null;
      }
      return current;
    };

    return {
      initialized: false,
      apiKind: api.kind,
      connection: 'idle',
      connectionRefusal: null,
      session: null,
      lastSeq: 0,
      run: null,
      tasks: {},
      graphRev: 0,
      selectedTaskId: null,
      messages: [],
      streamingRunId: null,
      metrics: null,
      history: { latest: 0, selected: 0, replayGraph: null },
      approvals: [],
      refusals: [],
      pendingCommands: {},
      view: { nav: 'workspace', tab: 'workspace' },
      viewport: { x: 0, y: 0, zoom: 1 },
      chat: { pinned: true, visible: true },
      settings: initialSettings(),
      fileTree: initialFileTree(),
      project: baseProject(),
      reducedMotionOverride: false,
      diffOpen: false,

      async init(): Promise<void> {
        if (get().initialized) {
          return;
        }
        set({ initialized: true, connection: 'connecting' });
        unsubscribes.push(
          api.subscribe((event) => applyEvent(event)),
          api.subscribeStatus((status, detail) => {
            set({ connection: status, connectionRefusal: detail ?? get().connectionRefusal });
          }),
          api.subscribeClientError((refusal) => pushRefusal(refusal)),
        );
        try {
          const session = await api.open();
          set({ session, connection: 'open', lastSeq: session.last_seq });
        } catch (error) {
          const refusal =
            error instanceof RefusalError ? error.refusal : clientRefusal('E_WS_DISCONNECTED');
          set({ connection: 'error', connectionRefusal: refusal });
          pushRefusal(refusal);
        }
      },

      dispose(): void {
        for (const unsubscribe of unsubscribes) {
          unsubscribe();
        }
        unsubscribes.length = 0;
        api.close();
      },

      async submitPrompt(text: string): Promise<void> {
        const refusalCode = validatePromptText(text);
        if (refusalCode) {
          pushRefusal(clientRefusal(refusalCode));
          return;
        }
        set({
          messages: [
            ...get().messages,
            { id: uuidv7(), role: 'user', text, runId: null, streaming: false },
          ],
        });
        await run('SUBMIT_PROMPT', { text, project_id: get().project.id });
      },

      async spawnSubHive(): Promise<void> {
        const current = guardActiveRun();
        if (!current) {
          return;
        }
        const selected = get().selectedTaskId;
        const parent =
          selected ??
          Object.values(get().tasks).find((task) => task.kind === 'superagent')?.id ??
          Object.keys(get().tasks)[0];
        if (!parent) {
          pushRefusal(clientRefusal('E_TASK_NOT_FOUND'));
          return;
        }
        await run('SPAWN_SUB_HIVE', { parent_task_id: parent, role: STRINGS.node.superAgent });
      },

      async rewireSelected(newParentId: string): Promise<void> {
        const current = guardActiveRun();
        if (!current) {
          return;
        }
        const taskId = get().selectedTaskId;
        if (!taskId) {
          pushRefusal(clientRefusal('E_TASK_NOT_FOUND'));
          return;
        }
        await run('REWIRE_TASK', {
          task_id: taskId,
          new_parent_id: newParentId,
          expected_rev: get().graphRev,
        });
      },

      async cancelRun(): Promise<void> {
        const current = get().run;
        if (!current || !isRunActive(current.status)) {
          pushRefusal(clientRefusal('E_RUN_TERMINAL'));
          return;
        }
        await run('CANCEL_RUN', { run_id: current.id });
      },

      async killWorker(): Promise<void> {
        const taskId = get().selectedTaskId;
        if (!taskId) {
          pushRefusal(clientRefusal('E_TASK_NOT_FOUND'));
          return;
        }
        const task = get().tasks[taskId];
        if (!task || (task.kind !== 'worker' && task.kind !== 'tool')) {
          pushRefusal(clientRefusal('E_KILL_DENIED'));
          return;
        }
        await run('KILL_WORKER', { task_id: taskId });
      },

      async resolveApproval(
        approvalId: string,
        decision: 'approve' | 'deny',
        reason?: string,
      ): Promise<void> {
        const approval = get().approvals.find((candidate) => candidate.approvalId === approvalId);
        if (!approval) {
          pushRefusal(clientRefusal('E_APPROVAL_NOT_FOUND'));
          return;
        }
        if (!approval.nonce) {
          pushRefusal(
            clientRefusal('E_SCHEMA_INVALID', {
              message: 'APPROVAL_REQUEST carried no nonce (contract gap); cannot resolve.',
            }),
          );
          return;
        }
        await run('APPROVAL_RESOLVE', {
          approval_id: approvalId,
          nonce: approval.nonce,
          decision,
          ...(reason === undefined ? {} : { reason }),
        });
      },

      async pinChat(pinned: boolean): Promise<void> {
        const ok = await run('CHAT_PIN', { pinned });
        if (ok) {
          set({ chat: { ...get().chat, pinned } });
        }
      },

      async insertTemplate(templateId: string): Promise<void> {
        await run('TEMPLATE_INSERT', {
          template_id: templateId,
          project_id: get().project.id,
        });
      },

      async setHistoryRevision(revision: number): Promise<void> {
        const current = get().run;
        if (!current) {
          pushRefusal(clientRefusal('E_RUN_NOT_FOUND'));
          return;
        }
        const bounded = Math.max(0, Math.min(revision, get().history.latest));
        await run('HISTORY_REVISION_SET', { run_id: current.id, revision: bounded });
      },

      setNav(nav: NavSection): void {
        set({ view: { ...get().view, nav } });
        if (nav === 'data' && get().fileTree.status === 'idle') {
          void get().requestFileTree();
        }
      },

      setTab(tab: TabId): void {
        set({ view: { ...get().view, tab } });
      },

      setViewport(viewport: ViewportState): void {
        set({ viewport });
      },

      selectTask(taskId: string | null): void {
        set({ selectedTaskId: taskId });
      },

      setChatVisible(visible: boolean): void {
        set({ chat: { ...get().chat, visible } });
      },

      openDiffModal(): void {
        set({ diffOpen: true });
      },

      closeDiffModal(): void {
        set({ diffOpen: false });
      },

      dismissRefusal(id: string): void {
        set({ refusals: get().refusals.filter((entry) => entry.id !== id) });
      },

      setReducedMotionOverride(value: boolean): void {
        set({ reducedMotionOverride: value });
      },

      async probeSettings(): Promise<void> {
        // Exactly one SETTINGS_STATUS per Settings open: the first call flips
        // status to 'pending' synchronously, so a second call (React StrictMode
        // double-effect or a rapid refresh) is a no-op until the response lands.
        if (get().settings.status === 'pending') {
          return;
        }
        set({ settings: { ...get().settings, status: 'pending', refusal: null } });
        const priorRefusalIds = new Set(get().refusals.map((entry) => entry.id));
        const ok = await run('SETTINGS_STATUS', {});
        if (!ok) {
          const refusal =
            get().refusals.find((entry) => !priorRefusalIds.has(entry.id))?.refusal ?? null;
          set({
            settings: {
              ...get().settings,
              status: 'error',
              refusal,
              lastProbe: Date.now(),
            },
          });
        }
      },

      async probeHealth(): Promise<void> {
        try {
          const snapshot = await fetchHealth();
          set({
            settings: {
              ...get().settings,
              health: snapshot.status,
              healthVersion: snapshot.version,
              checks: snapshot.checks,
              lastProbe: Date.now(),
            },
          });
        } catch (error) {
          const refusal =
            error instanceof RefusalError ? error.refusal : clientRefusal('E_WS_DISCONNECTED');
          set({
            settings: { ...get().settings, health: 'unknown', checks: [], lastProbe: Date.now() },
          });
          pushRefusal(refusal);
        }
      },

      async requestFileTree(): Promise<void> {
        set({ fileTree: { ...get().fileTree, status: 'loading', refusal: null } });
        const ok = await run('FILE_TREE_LIST', {});
        if (!ok) {
          set({ fileTree: { ...get().fileTree, status: 'error' } });
        }
      },

      async openFile(path: string): Promise<void> {
        const jailRefusal = validateWorkspacePath(path);
        if (jailRefusal) {
          pushRefusal(clientRefusal(jailRefusal));
          return;
        }
        set({
          fileTree: {
            ...get().fileTree,
            contentStatus: 'loading',
            selectedPath: path,
            refusal: null,
          },
        });
        const ok = await run('FILE_READ', { path });
        if (!ok) {
          set({ fileTree: { ...get().fileTree, contentStatus: 'error' } });
        }
      },

      formatSelectedRevision(): string {
        const history = get().history;
        if (history.latest < 0) {
          return STRINGS.scrubber.empty;
        }
        return formatStateOf(history.selected, Math.max(history.latest + 1, 1));
      },
    };
  });
}

export type UiStoreApi = StoreApi<UiState>;
export { deriveHud, isNodeKind, HEALTHY_GREEN };
