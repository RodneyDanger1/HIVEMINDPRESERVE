/**
 * S02 pinned chat core + S03 composer + S27 streaming reply. The core hive is
 * a fixed overlay: panning/zooming the canvas never moves it (PDF:109-115).
 * Submits only SUBMIT_PROMPT; replies stream from REPLY_DELTA events; every
 * refusal is named by ToastRegion, never silently dropped.
 */
import {
  memo,
  useEffect,
  useMemo,
  useRef,
  useState,
  type FormEvent,
  type PointerEvent as ReactPointerEvent,
} from 'react';
import { useUiStore, useUiStoreApi } from '../state/context';
import { STRINGS } from '../state/strings';
import { isTerminal } from '../state/agentState';
import { useAppReducedMotion } from '../state/useAppReducedMotion';
import { validatePromptText } from '../api/contract';
import { clientRefusal } from '../api/refusals';
import { InfoTip } from '../panels/InfoTip';
import { ControlBar } from '../panels/ControlBar';
import { DOT_STATES, visualFor } from '../state/agentState';
import { StateDot } from '../flow/nodes';
import { APP_VERSION } from '../version';
import { CONTROL_INTENTS } from '../state/chatIntents';
import type { ChatMessage, ToastRefusal } from '../state/store';
import { MessageBody, TypingDots, WorkingBee } from './ChatMotion';

const CHAT_HEIGHT_KEY = 'hivemind.chat-height';
const MIN_CHAT_HEIGHT = 300;
const DOCK_TRAY_HEIGHT = 210;

function readChatHeight(): number {
  try {
    const stored = localStorage.getItem(CHAT_HEIGHT_KEY);
    if (stored === null) return 560;
    const raw = Number(stored);
    // The old 390/480 defaults left too little transcript once a run refusal
    // or the session panel was visible; migrate those exact defaults forward
    // without overriding a deliberate larger resize.
    if (raw === 390 || raw === 480) return 560;
    return Number.isFinite(raw) ? Math.min(760, Math.max(MIN_CHAT_HEIGHT, raw)) : 560;
  } catch {
    return 560;
  }
}

const SESSION_STORAGE_KEY = 'hivemind.session-id';
const POLICY_STORAGE_KEY = 'hivemind.session-policy';
const POLICY_VALUES = ['classic', 'plan', 'standard', 'trusted'] as const;
type StoredPolicy = (typeof POLICY_VALUES)[number];

function readStoredPolicy(): StoredPolicy {
  try {
    const raw = localStorage.getItem(POLICY_STORAGE_KEY);
    return POLICY_VALUES.includes(raw as StoredPolicy) ? (raw as StoredPolicy) : 'classic';
  } catch {
    return 'classic';
  }
}

/** Baselines captured immediately before SUBMIT_PROMPT is dispatched. */
export interface PromptSubmissionBaseline {
  readonly messageIds: ReadonlySet<string>;
  readonly refusalIds: ReadonlySet<string>;
  readonly startedAt: number;
}

export function promptSubmissionBaseline(
  messages: readonly ChatMessage[],
  refusals: readonly ToastRefusal[],
  startedAt: number,
): PromptSubmissionBaseline {
  return {
    messageIds: new Set(messages.map((message) => message.id)),
    refusalIds: new Set(refusals.map((entry) => entry.id)),
    startedAt,
  };
}

/** The refusal the store published while this submission was in flight, if any. */
export function findSubmissionRefusal(
  baseline: PromptSubmissionBaseline,
  refusals: readonly ToastRefusal[],
): ToastRefusal | null {
  return (
    refusals.find(
      (entry) => !baseline.refusalIds.has(entry.id) && entry.at >= baseline.startedAt,
    ) ?? null
  );
}

/** The optimistic user row this submission appended, if it is still present. */
export function findOptimisticPrompt(
  baseline: PromptSubmissionBaseline,
  messages: readonly ChatMessage[],
  text: string,
): ChatMessage | null {
  return (
    messages.find(
      (message) =>
        message.role === 'user' && !baseline.messageIds.has(message.id) && message.text === text,
    ) ?? null
  );
}

/** ACK: clear only the sent text, never text typed while the command was in flight. */
export function draftAfterAck(current: string, submitted: string): string {
  return current === submitted ? '' : current;
}

/** Refusal: restore the submitted text unless the operator already typed a replacement. */
export function draftAfterRefusal(current: string, submitted: string): string {
  return current.trim().length === 0 ? submitted : current;
}

const STARTERS = [
  { label: 'Map this repository', prompt: 'Map the architecture of this repository and identify the highest-risk module.' },
  { label: 'Review an API change', prompt: 'Review the current API change for regressions and missing tests.' },
  { label: 'Explain a failure', prompt: 'Explain the last failure and propose the smallest safe fix.' },
] as const;

/** DOM/transcript cap: long runs must not mount an unbounded message list. */
export const MAX_RENDERED_CHAT_MESSAGES = 200;

/** Newest slice of the transcript; identity is preserved when under the cap. */
export function visibleChatMessages(messages: readonly ChatMessage[]): readonly ChatMessage[] {
  return messages.length > MAX_RENDERED_CHAT_MESSAGES
    ? messages.slice(messages.length - MAX_RENDERED_CHAT_MESSAGES)
    : messages;
}

/**
 * Finalized rows are identity-stable in the store, so memo stops every REPLY_DELTA
 * from re-rendering (and re-parsing) the entire history. Only the streaming tail
 * changes identity and re-renders.
 */
const ChatMessageRow = memo(function ChatMessageRow({ message }: { message: ChatMessage }) {
  return (
    <article
      className={
        'chat-message mb-2 rounded-[10px] px-2 py-1.5 last:mb-0 ' +
        (message.role === 'assistant' ? 'bg-honey-100' : 'bg-panel-alt') +
        (message.streaming ? ' chat-streaming' : '')
      }
    >
      <div className="type-mono-meta text-meta">
        {message.role === 'assistant' ? STRINGS.chat.assistantName : STRINGS.chat.userLabel}
        {message.streaming ? ` · ${STRINGS.states.loading}` : ''}
      </div>
      <MessageBody text={message.text} />
      {message.streaming ? (
        <span
          className="ml-0.5 inline-block h-[12px] w-[2px] animate-pulse bg-honey-500 align-middle"
          aria-hidden="true"
        />
      ) : null}
    </article>
  );
});

export function ChatCore() {
  const messages = useUiStore((state) => state.messages);
  const submitPrompt = useUiStore((state) => state.submitPrompt);
  const pending = useUiStore((state) => state.pendingCommands);
  const connection = useUiStore((state) => state.connection);
  const chat = useUiStore((state) => state.chat);
  const setChatVisible = useUiStore((state) => state.setChatVisible);
  const setActiveSessionId = useUiStore((state) => state.setActiveSessionId);
  const streamingRunId = useUiStore((state) => state.streamingRunId);
  const runStatus = useUiStore((state) => state.run?.status ?? null);
  const cancelRun = useUiStore((state) => state.cancelRun);
  const replaying = useUiStore((state) => state.history.replayGraph !== null);
  const runErrorCode = useUiStore((state) => state.run?.errorCode ?? null);
  const runId = useUiStore((state) => state.run?.id ?? null);
  const [dismissedErrorRunId, setDismissedErrorRunId] = useState<string | null>(null);
  const settings = useUiStore((state) => state.settings);
  const probeSettings = useUiStore((state) => state.probeSettings);
  const keyPresent = settings.keyPresent;
  const approvals = useUiStore((state) => state.approvals);
  const resolvingApprovals = useUiStore((state) => state.resolvingApprovals);
  const resolveApproval = useUiStore((state) => state.resolveApproval);
  const pendingApproval = approvals[0] ?? null;
  const approvalBusy =
    pendingApproval === null || resolvingApprovals.includes(pendingApproval.approvalId);
  const toolActivity = useUiStore((state) => state.toolActivity);
  const tasks = useUiStore((state) => state.tasks);
  const activeAgents = useMemo(
    () => Object.values(tasks).filter((task) => !isTerminal(task.state)).length,
    [tasks],
  );
  const [sessions, setSessions] = useState<
    readonly {
      id: string;
      name: string;
      archived_at?: string | null;
      permission_policy?: 'classic' | 'plan' | 'standard' | 'trusted';
    }[]
  >([]);
  const [namedSession, setNamedSession] = useState(() => {
    try {
      return localStorage.getItem(SESSION_STORAGE_KEY) ?? '';
    } catch {
      return '';
    }
  });
  const [permissionPolicy, setPermissionPolicy] =
    useState<StoredPolicy>(readStoredPolicy);
  const [sessionsLoaded, setSessionsLoaded] = useState(false);
  const [sessionLoadError, setSessionLoadError] = useState(false);
  const [includeArchived, setIncludeArchived] = useState(false);
  const [sessionReload, setSessionReload] = useState(0);
  const [sessionBusy, setSessionBusy] = useState(false);
  const [newSessionName, setNewSessionName] = useState('');
  const [attachError, setAttachError] = useState<string | null>(null);
  const fileInputRef = useRef<HTMLInputElement | null>(null);
  const [chatHeight, setChatHeight] = useState(readChatHeight);
  const [dockTab, setDockTab] = useState<'controls' | 'state' | null>(null);
  const [resizing, setResizing] = useState(false);
  const resizeRef = useRef<{ startY: number; startHeight: number } | null>(null);

  const startResize = (event: ReactPointerEvent<HTMLDivElement>): void => {
    resizeRef.current = { startY: event.clientY, startHeight: chatHeight };
    setResizing(true);
    event.currentTarget.setPointerCapture(event.pointerId);
  };
  const moveResize = (event: ReactPointerEvent<HTMLDivElement>): void => {
    if (resizeRef.current === null) return;
    const next = Math.min(
      760,
      Math.max(MIN_CHAT_HEIGHT, resizeRef.current.startHeight + (resizeRef.current.startY - event.clientY)),
    );
    setChatHeight(next);
  };
  const endResize = (): void => {
    resizeRef.current = null;
    setResizing(false);
    try {
      localStorage.setItem(CHAT_HEIGHT_KEY, String(chatHeight));
    } catch {
      // Selection still works for this session.
    }
  };
  const fileTreeRoot = useUiStore((state) => state.fileTree.root);
  const projectId = useUiStore((state) => state.project.id);
  const [configuredWorkspace, setConfiguredWorkspace] = useState<string | null>(null);
  useEffect(() => {
    if (connection !== 'open') return;
    let cancelled = false;
    void (async () => {
      try {
        const response = await fetch('/api/runtime', { headers: { accept: 'application/json' } });
        if (!response.ok) return;
        const body = (await response.json()) as { workspace_root?: string };
        if (!cancelled && body.workspace_root) setConfiguredWorkspace(body.workspace_root);
      } catch {
        // The create control stays disabled; no invented workspace path.
      }
    })();
    return () => {
      cancelled = true;
    };
  }, [connection]);
  const sessionWorkspace = fileTreeRoot ?? configuredWorkspace;
  useEffect(() => {
    if (connection !== 'open') return;
    let cancelled = false;
    void (async () => {
      try {
        const handshake = await fetch('/api/session', { headers: { accept: 'application/json' } });
        if (!handshake.ok) throw new Error('session handshake failed');
        const token = (await handshake.json()) as { token?: string };
        const headers: Record<string, string> = { accept: 'application/json' };
        if (token.token) headers.Authorization = `Bearer ${token.token}`;
        const response = await fetch(
          includeArchived ? '/api/sessions?include_archived=true' : '/api/sessions',
          { headers },
        );
        if (!response.ok) throw new Error('sessions unavailable');
        const body = (await response.json()) as {
          sessions?: readonly {
            id: string;
            name: string;
            archived_at?: string | null;
            permission_policy?: 'classic' | 'plan' | 'standard' | 'trusted';
          }[];
        };
        if (!cancelled) {
          setSessions(body.sessions ?? []);
          setSessionsLoaded(true);
          setSessionLoadError(false);
        }
      } catch {
        if (!cancelled) setSessionLoadError(true);
      }
    })();
    return () => {
      cancelled = true;
    };
  }, [connection, includeArchived, sessionReload]);
  useEffect(() => {
    if (
      sessionsLoaded &&
      namedSession &&
      !sessions.some((session) => session.id === namedSession)
    ) {
      setNamedSession('');
      setActiveSessionId(null);
    }
    // The policy selector is also used for a new/default run when no named
    // session is selected; do not silently reset an explicit operator choice.
  }, [sessions, sessionsLoaded, namedSession, setActiveSessionId]);
  useEffect(() => {
    try {
      if (namedSession) localStorage.setItem(SESSION_STORAGE_KEY, namedSession);
      else localStorage.removeItem(SESSION_STORAGE_KEY);
    } catch {
      // Storage is optional; the in-memory selection still works.
    }
    setActiveSessionId(namedSession || null);
  }, [namedSession, setActiveSessionId]);
  useEffect(() => {
    try {
      localStorage.setItem(POLICY_STORAGE_KEY, permissionPolicy);
    } catch {
      // Storage is optional.
    }
  }, [permissionPolicy]);
  const selectedSession = sessions.find((session) => session.id === namedSession) ?? null;
  const changePolicy = async (
    next: 'classic' | 'plan' | 'standard' | 'trusted',
  ): Promise<void> => {
    const previous = permissionPolicy;
    setPermissionPolicy(next);
    if (!namedSession) return;
    try {
      const handshake = await fetch('/api/session', { headers: { accept: 'application/json' } });
      if (!handshake.ok) throw new Error('session handshake failed');
      const token = (await handshake.json()) as { token?: string };
      const headers: Record<string, string> = {
        accept: 'application/json',
        'content-type': 'application/json',
      };
      if (token.token) headers.Authorization = `Bearer ${token.token}`;
      const response = await fetch(
        `/api/sessions/${encodeURIComponent(namedSession)}/policy`,
        {
          method: 'POST',
          headers,
          body: JSON.stringify({ permission_policy: next }),
        },
      );
      if (!response.ok) throw new Error(`policy returned ${response.status}`);
      setSessionReload((current) => current + 1);
    } catch {
      // Persist failed: do not show a policy the session does not have.
      setPermissionPolicy(previous);
      setSessionLoadError(true);
    }
  };

  const createSession = async (): Promise<void> => {
    const name = newSessionName.trim();
    if (!name || !sessionWorkspace || sessionBusy) return;
    setSessionBusy(true);
    try {
      const handshake = await fetch('/api/session', { headers: { accept: 'application/json' } });
      if (!handshake.ok) throw new Error('session handshake failed');
      const token = (await handshake.json()) as { token?: string };
      const headers: Record<string, string> = {
        accept: 'application/json',
        'content-type': 'application/json',
      };
      if (token.token) headers.Authorization = `Bearer ${token.token}`;
      const response = await fetch('/api/sessions', {
        method: 'POST',
        headers,
        body: JSON.stringify({
          name,
          project_id: projectId,
          workspace_root: sessionWorkspace,
          permission_policy: permissionPolicy,
        }),
      });
      if (!response.ok) throw new Error(`create returned ${response.status}`);
      const created = (await response.json()) as {
        id: string;
        name?: string;
        permission_policy?: 'classic' | 'plan' | 'standard' | 'trusted';
        archived_at?: string | null;
      };
      setNewSessionName('');
      // Insert first so the reload guard cannot treat the fresh selection as a
      // hidden/removed session before the refetched list includes it.
      setSessions((current) => [
        {
          id: created.id,
          name: created.name ?? name,
          permission_policy: created.permission_policy ?? permissionPolicy,
          archived_at: created.archived_at ?? null,
        },
        ...current,
      ]);
      setNamedSession(created.id);
      setActiveSessionId(created.id);
      if (created.permission_policy) setPermissionPolicy(created.permission_policy);
      setSessionReload((current) => current + 1);
      setSessionLoadError(false);
    } catch {
      setSessionLoadError(true);
    } finally {
      setSessionBusy(false);
    }
  };

  const toggleArchive = async (): Promise<void> => {
    if (!selectedSession || sessionBusy) return;
    setSessionBusy(true);
    try {
      const handshake = await fetch('/api/session', { headers: { accept: 'application/json' } });
      if (!handshake.ok) throw new Error('session handshake failed');
      const token = (await handshake.json()) as { token?: string };
      const headers: Record<string, string> = { accept: 'application/json' };
      if (token.token) headers.Authorization = `Bearer ${token.token}`;
      const response = await fetch(
        `/api/sessions/${encodeURIComponent(selectedSession.id)}/${
          selectedSession.archived_at ? 'unarchive' : 'archive'
        }`,
        { method: 'POST', headers: { ...headers, 'content-type': 'application/json' }, body: '{}' },
      );
      if (!response.ok) throw new Error(`archive returned ${response.status}`);
      setSessionReload((current) => current + 1);
      if (!selectedSession.archived_at) {
        setNamedSession('');
        setActiveSessionId(null);
        setPermissionPolicy('classic');
      }
    } catch {
      setSessionLoadError(true);
    } finally {
      setSessionBusy(false);
    }
  };
  const submitOptions = {
    ...(namedSession ? { sessionId: namedSession } : {}),
    policy: permissionPolicy,
  };
  const liveLabel =
    connection !== 'open'
      ? 'offline'
      : runStatus === 'planning'
        ? 'planning'
        : streamingRunId !== null || runStatus === 'running' || runStatus === 'thinking'
          ? `${activeAgents} active`
          : 'idle';
  const runRefusal = runErrorCode === null ? null : clientRefusal(runErrorCode);
  const reduced = useAppReducedMotion();
  const store = useUiStoreApi();
  const [draft, setDraft] = useState('');
  const listRef = useRef<HTMLDivElement | null>(null);
  const composerRef = useRef<HTMLTextAreaElement | null>(null);
  // Synchronous double-submit guard: the derived `submitting` updates on render,
  // so two fast Enter presses could otherwise both pass the disabled check.
  const inFlightRef = useRef(false);

  const submitting = useMemo(
    () => Object.values(pending).some((entry) => entry.name === 'SUBMIT_PROMPT'),
    [pending],
  );
  const renderedMessages = useMemo(() => visibleChatMessages(messages), [messages]);
  const hiddenMessageCount = messages.length - renderedMessages.length;
  const recentActivity = useMemo(
    () =>
      [...toolActivity]
        .sort((a, b) => a.turn - b.turn || a.toolCallId.localeCompare(b.toolCallId))
        .slice(-4),
    [toolActivity],
  );

  useEffect(() => {
    // The State tab is only useful when it shows the live provider/key state;
    // without this the values stay "—" until the operator opens Settings.
    if (connection === 'open') void probeSettings();
  }, [connection, probeSettings]);

  useEffect(() => {
    const node = listRef.current;
    if (node) {
      // An empty thread must show its starters from the top. Scrolling an
      // empty log to its measured height hid the first starter under the
      // header once the root-control block was added.
      node.scrollTop = messages.length === 0 ? 0 : node.scrollHeight;
    }
  }, [messages]);

  if (!chat.visible) {
    // The header owns the one canonical Show/Hide control; a second floating
    // reopen pill only duplicated it and collided with the HUD.
    return null;
  }

  /**
   * Validate -> submit -> ACK/refusal settlement. The draft is never cleared
   * before the command is acknowledged; on a named refusal the optimistic user
   * message the store appended is retracted and the typed prompt stays in the
   * composer (M-018: no silent loss).
   */
  const submit = async (): Promise<void> => {
    if (inFlightRef.current) {
      return;
    }
    const text = draft;
    if (keyPresent === false) {
      // Do not create a doomed run just to have the server fail E_KEY_MISSING.
      // Keep the typed draft and surface the same named refusal the server uses.
      const refusal = clientRefusal('E_KEY_MISSING');
      const current = store.getState();
      store.setState({
        refusals: [
          ...current.refusals,
          { id: crypto.randomUUID(), refusal, at: Date.now() },
        ],
      });
      return;
    }
    if (validatePromptText(text) !== null) {
      // The store validates before appending and raises the named refusal
      // toast; this call therefore sends nothing and renders no user message.
      await submitPrompt(text, submitOptions);
      return;
    }
    const before = store.getState();
    const baseline = promptSubmissionBaseline(before.messages, before.refusals, Date.now());
    inFlightRef.current = true;
    try {
      await submitPrompt(text, submitOptions);
    } finally {
      inFlightRef.current = false;
    }
    const after = store.getState();
    // submitPrompt returns void (store-owned API), so the ACK outcome is read
    // from the refusal the store publishes for this command: server ERROR
    // frames land before the send promise settles, and local refusals are
    // pushed by the same path. A store API that returns the ACK outcome would
    // replace this observation without changing the rollback below.
    const refusal = findSubmissionRefusal(baseline, after.refusals);
    if (refusal === null) {
      setDraft((current) => draftAfterAck(current, text));
      return;
    }
    // Refused: keep/restore the draft and retract exactly the optimistic row.
    setDraft((current) => draftAfterRefusal(current, text));
    const optimistic = findOptimisticPrompt(baseline, after.messages, text);
    if (optimistic) {
      store.setState({
        messages: store.getState().messages.filter((message) => message.id !== optimistic.id),
      });
    }
  };

  const onSubmit = (event: FormEvent<HTMLFormElement>): void => {
    event.preventDefault();
    void submit();
  };

  const attachFile = async (file: File | null): Promise<void> => {
    if (file === null) return;
    if (file.size > 60_000) {
      setAttachError('Text attachments are limited to 60 KB.');
      return;
    }
    try {
      const text = await file.text();
      setDraft((current) =>
        `${current}${current.trim().length > 0 ? '\n\n' : ''}[Attached text file: ${file.name}]\n\`\`\`\n${text}\n\`\`\``,
      );
      setAttachError(null);
      composerRef.current?.focus();
    } catch {
      setAttachError('That file could not be read as text.');
    } finally {
      if (fileInputRef.current) fileInputRef.current.value = '';
    }
  };

  return (
    <div className="pointer-events-none fixed inset-x-0 bottom-0 left-[72px] z-40 flex items-end justify-center max-[640px]:bottom-14 max-[640px]:left-0">
      <section
        aria-label={STRINGS.chat.aria}
        className="chat-core pointer-events-auto surface-panel z-40 flex max-h-[calc(100vh-52px)] w-full flex-col rounded-t-[18px] border-x-0 border-b-0 p-3 max-[640px]:!h-auto max-[640px]:max-h-[calc(100dvh-110px)] max-[640px]:overflow-y-auto"
        style={{
          height: chatHeight + (dockTab === null ? 0 : DOCK_TRAY_HEIGHT),
          boxShadow: 'var(--shadow-e2)',
          border: 'none',
          transition:
            reduced || resizing
              ? 'none'
              : 'height 180ms cubic-bezier(0.2, 0, 0, 1), box-shadow 160ms ease',
        }}
      >
        <div
          role="separator"
          aria-label="Resize chat panel"
          aria-orientation="horizontal"
          title="Drag to resize"
          className="mx-auto mb-1 h-3 w-16 shrink-0 cursor-row-resize touch-none rounded-full bg-white/35 hover:bg-white/60"
          onPointerDown={startResize}
          onPointerMove={moveResize}
          onPointerUp={endResize}
          onPointerCancel={endResize}
        />
        <header
          className="-mx-3 mb-2 flex h-11 items-center justify-between gap-2 rounded-t-[18px] px-3 text-white"
          style={{ background: 'linear-gradient(135deg, var(--color-ink-800), var(--color-honey-800))' }}
        >
          <div className="flex min-w-0 flex-1 items-center gap-2">
            <WorkingBee
              running={
                streamingRunId !== null ||
                runStatus === 'running' ||
                runStatus === 'thinking' ||
                runStatus === 'planning'
              }
            />
            <div className="truncate text-[14px] font-semibold leading-[18px]">{STRINGS.chat.title}</div>
          </div>
          <div className="flex shrink-0 items-center gap-1">
            <span
              className="type-mono-meta flex items-center gap-1 whitespace-nowrap px-1 text-[11px] text-honey-100/80"
              role="status"
              aria-live="polite"
            >
              <span
                className={
                  'h-1.5 w-1.5 rounded-full ' +
                  (connection !== 'open'
                    ? 'bg-signal'
                    : streamingRunId !== null || runStatus === 'running'
                      ? 'motion-hive-live-dot bg-honey-300'
                      : 'bg-honey-100/70')
                }
              />
              {liveLabel}
            </span>
            <button
              type="button"
              className="type-mono-meta flex min-h-6 min-w-6 items-center justify-center whitespace-nowrap rounded-[var(--radius-sm)] border border-white/25 px-1.5 py-0.5 text-[16px] leading-none text-white/90 hover:bg-white/10 min-h-11 min-w-11"
              onClick={() => setChatVisible(false)}
              aria-label={STRINGS.chat.closeAria}
            >
              ×
            </button>
          </div>
        </header>

        {runRefusal === null || runId === dismissedErrorRunId ? null : (
          <div
            role="alert"
            className="mt-2 rounded-[var(--radius-sm)] border border-signal bg-ink-900 px-2 py-1.5 text-canvas"
          >
            <div className="flex items-center gap-2">
              <span className="type-label text-honey-200">Run refused</span>
              <span className="type-mono-meta text-honey-200">{runRefusal.code}</span>
              <button
                type="button"
                className="type-mono-meta ml-auto min-h-7 min-w-7 rounded-[4px] border border-white/20 text-honey-100 hover:bg-white/10"
                aria-label="Dismiss run refusal"
                onClick={() => setDismissedErrorRunId(runId)}
              >
                ×
              </button>
            </div>
            <div className="mt-1 line-clamp-2 text-[12px] leading-[16px]">{runRefusal.message}</div>
            <details className="mt-1">
              <summary className="type-mono-meta cursor-pointer text-[10px] text-honey-100/85">
                What to do
              </summary>
              <div className="mt-1 text-[11px] leading-4 text-honey-100/85">
                {runRefusal.how_to_fix}
                {runErrorCode === 'E_SCHEMA_INVALID'
                  ? ' The planner returned a plan that did not match the schema; retry once, then shorten the prompt if it repeats.'
                  : ''}
              </div>
            </details>
          </div>
        )}

        {hiddenMessageCount > 0 ? (
          <p
            role="status"
            data-testid="chat-transcript-cap"
            className="type-mono-meta mt-2 rounded-[var(--radius-sm)] border border-hairline bg-panel-alt px-2 py-1 text-meta"
          >
            Showing the latest {MAX_RENDERED_CHAT_MESSAGES} of {messages.length} messages —{' '}
            {hiddenMessageCount} earlier {hiddenMessageCount === 1 ? 'message is' : 'messages are'}{' '}
            hidden.
          </p>
        ) : null}

        <div
          ref={listRef}
          className="mt-2 min-h-0 flex-1 overflow-y-auto rounded-[var(--radius-sm)] bg-panel-alt px-2 py-2"
          aria-label="Chat transcript"
        >
          {messages.length === 0 ? (
            <div>
              <p className="text-[12.5px] leading-[18px] text-ink-500">{STRINGS.chat.empty}</p>
              <div className="mt-3 flex flex-col gap-1.5">
                {STARTERS.map((starter) => (
                  <button
                    key={starter.label}
                    type="button"
                    className="type-label rounded-[8px] border border-hairline bg-panel px-2.5 py-1.5 text-left text-[10.5px] text-ink-800 hover:border-honey-400 hover:bg-honey-100"
                    onClick={() => {
                      setDraft(starter.prompt);
                      composerRef.current?.focus();
                    }}
                  >
                    {starter.label}
                  </button>
                ))}
              </div>
              <div className="mt-6 flex flex-col items-center gap-1.5 opacity-70" aria-hidden="true">
                <svg width="46" height="46" viewBox="0 0 46 46">
                  <path d="M23 4 39 13v18L23 40 7 31V13Z" fill="none" stroke="var(--color-honey-400)" strokeWidth="1.2" />
                  <path d="M23 13 32 18v10l-9 5-9-5V18Z" fill="var(--color-honey-100)" stroke="var(--color-honey-500)" strokeWidth="1" />
                  <circle cx="23" cy="22" r="3" fill="var(--color-honey-500)" />
                </svg>
                <span className="type-mono-meta text-[11px] tracking-[0.14em] text-meta">READY · NO RUN YET</span>
              </div>
            </div>
          ) : (
            renderedMessages.map((message) => (
              <ChatMessageRow key={message.id} message={message} />
            ))
          )}
          {toolActivity.length > 0 ? (
            <section
              aria-label="Tool activity in chat"
              className="mt-2 rounded-[8px] border border-hairline bg-panel px-2 py-1.5"
            >
              <div className="type-mono-meta flex items-center justify-between text-[11px] text-meta">
                <span>ACTIONS</span>
                <span>{toolActivity.length}</span>
              </div>
              <ul className="mt-1 space-y-0.5">
                {recentActivity.map((entry) => (
                  <li key={entry.toolCallId} className="type-mono-meta flex items-center gap-1.5 text-[11px]">
                    <span className="text-ink-800">{entry.tool}</span>
                    <span
                      className={
                        entry.outcome === 'ok'
                          ? 'text-verify'
                          : entry.outcome === 'pending'
                            ? 'text-meta'
                            : 'text-signal'
                      }
                    >
                      {entry.outcome}
                    </span>
                    {entry.text === undefined || entry.text.length === 0 ? null : (
                      <span className="min-w-0 flex-1 truncate text-meta">{entry.text}</span>
                    )}
                  </li>
                ))}
              </ul>
            </section>
          ) : null}
          {streamingRunId ? (
            <div className="type-mono-meta flex items-center gap-2 text-meta" role="status">
              {renderedMessages.some((message) => message.streaming && message.text.length > 0) ? null : (
                <TypingDots />
              )}
              {STRINGS.agentStates.streaming}
            </div>
          ) : null}
        </div>

        <details className="mt-2 max-h-[132px] shrink-0 overflow-y-auto rounded-[8px] border border-hairline bg-panel-alt/50 px-2 py-1">
          <summary className="type-mono-meta cursor-pointer select-none text-[10px] tracking-[0.08em] text-ink-700">
            SESSION &amp; PERMISSIONS · {permissionPolicy}
          </summary>
        <div className="mt-2 grid grid-cols-1 gap-2 sm:grid-cols-2">
          <label className="type-mono-meta flex flex-col gap-0.5 text-[10px] text-meta">
            SESSION
            <select
              className="type-mono-code rounded-[8px] border border-hairline bg-panel px-1.5 py-1 text-[11px] text-ink-900"
              value={namedSession}
              disabled={connection !== 'open' || replaying}
              onChange={(event) => {
                const nextId = event.target.value;
                setNamedSession(nextId);
                setActiveSessionId(nextId || null);
                const chosen = sessions.find((session) => session.id === nextId);
                if (chosen?.permission_policy) setPermissionPolicy(chosen.permission_policy);
              }}
            >
              <option value="">Default session</option>
              {sessions.map((session) => (
                <option key={session.id} value={session.id}>
                  {session.name}
                  {session.archived_at ? ' (archived)' : ''}
                </option>
              ))}
            </select>
            {sessionLoadError ? (
              <span className="text-signal">Sessions unavailable; default will be used.</span>
            ) : null}
          </label>
          <label className="type-mono-meta flex flex-col gap-0.5 text-[10px] text-meta">
            POLICY
            <select
              className="type-mono-code rounded-[8px] border border-hairline bg-panel px-1.5 py-1 text-[11px] text-ink-900 disabled:cursor-not-allowed disabled:opacity-60"
              value={permissionPolicy}
              disabled={connection !== 'open' || replaying}
              title={
                namedSession
                  ? 'Policy for this named session/run'
                  : 'Policy for a new session or the default run'
              }
              onChange={(event) =>
                void changePolicy(
                  event.target.value as 'classic' | 'plan' | 'standard' | 'trusted',
                )
              }
            >
              <option value="classic" title="approve every side effect">classic — ask</option>
              <option value="plan" title="refuse mutations">plan — no writes</option>
              <option value="standard" title="auto-approve workspace edits">standard — workspace</option>
              <option value="trusted" title="explicit full-local opt-in">trusted — full local</option>
            </select>
          </label>
        </div>
        <button
          type="button"
          aria-pressed={permissionPolicy === 'trusted'}
          className={
            'type-mono-meta mt-1 flex items-center gap-1.5 rounded-[8px] border px-2 py-1 text-[10px] ' +
            (permissionPolicy === 'trusted'
              ? 'border-signal bg-signal/10 text-signal'
              : 'border-hairline bg-panel text-ink-700 hover:bg-honey-100')
          }
          title="Trusted policy runs side effects without approval prompts. Use only in a workspace you trust."
          onClick={() => void changePolicy(permissionPolicy === 'trusted' ? 'classic' : 'trusted')}
        >
          <span aria-hidden="true">{permissionPolicy === 'trusted' ? '●' : '○'}</span>
          AUTO-APPROVE {permissionPolicy === 'trusted' ? 'ON' : 'OFF'}
        </button>
        <div className="mt-1 flex flex-wrap items-center gap-2">
          <label className="type-mono-meta flex items-center gap-1 text-[10px] text-meta">
            <input
              type="checkbox"
              checked={includeArchived}
              disabled={connection !== 'open'}
              onChange={(event) => setIncludeArchived(event.target.checked)}
            />
            Show archived
          </label>
          <button
            type="button"
            className="type-mono-meta rounded-[6px] border border-hairline bg-panel px-1.5 py-0.5 text-[10px] text-ink-800 hover:border-honey-400 disabled:opacity-50"
            disabled={!selectedSession || sessionBusy || connection !== 'open'}
            title={
              selectedSession
                ? 'Archive or restore the selected named session'
                : 'Select a named session to enable archive'
            }
            onClick={() => void toggleArchive()}
          >
            {selectedSession?.archived_at ? 'Restore session' : 'Archive session'}
          </button>
        </div>
        <div className="mt-1 flex flex-wrap items-center gap-1">
          <label className="sr-only" htmlFor="new-session-name">
            New session name
          </label>
          <input
            id="new-session-name"
            className="type-mono-code min-w-0 flex-1 rounded-[6px] border border-hairline bg-panel px-2 py-0.5 text-[10px] text-ink-900"
            placeholder="New session name"
            value={newSessionName}
            disabled={sessionBusy || connection !== 'open'}
            onChange={(event) => setNewSessionName(event.target.value)}
          />
          <button
            type="button"
            className="type-mono-meta rounded-[6px] border border-honey-500 bg-honey-300 px-1.5 py-0.5 text-[10px] font-semibold text-ink-900 disabled:opacity-50"
            disabled={
              sessionBusy ||
              connection !== 'open' ||
              !sessionWorkspace ||
              newSessionName.trim().length === 0
            }
            title={
              sessionWorkspace
                ? 'Create a named session in the current workspace'
                : 'Workspace is not loaded yet'
            }
            onClick={() => void createSession()}
          >
            Create
          </button>
        </div>
        <div className="mt-1 flex flex-wrap items-center gap-1 max-[640px]:hidden">
          <span className="flex items-center gap-1 type-mono-meta text-[10px] tracking-[0.12em] text-meta">
            ROOT CONTROL
            <InfoTip label="What root control does">
              These buttons only fill the composer. The server decides whether the HyperAgent may
              spawn a remote or daemon; nothing is spawned by clicking here.
            </InfoTip>
          </span>
          {CONTROL_INTENTS.map((intent) => (
            <button
              key={intent.id}
              type="button"
              className="type-label rounded-[8px] border border-hairline bg-panel px-1.5 py-0.5 text-[10px] text-ink-800 hover:border-honey-400 hover:bg-honey-100 disabled:opacity-50"
              data-control-intent={intent.id}
              disabled={connection !== 'open' || replaying}
              title={`Fill the composer with a SUBMIT_PROMPT asking the HyperAgent root control for ${intent.label.toLowerCase()}. The server decides; no spawn is claimed here.`}
              onClick={() => {
                setDraft(intent.prompt);
                composerRef.current?.focus();
              }}
            >
              {intent.label}
            </button>
          ))}
        </div>

        </details>
        {pendingApproval ? (
          <div role="alert" className="mt-2 rounded-[10px] border border-amber bg-panel-alt px-2.5 py-2">
            <div className="flex items-center justify-between gap-2">
              <span className="type-mono-meta text-[10px] tracking-[0.1em] text-ink-800">
                APPROVAL NEEDED
              </span>
              <span className="type-mono-code truncate text-[10px] text-ink-900">{pendingApproval.tool}</span>
            </div>
            {pendingApproval.argsPreview ? (
              <p className="mt-1 line-clamp-2 break-all font-mono text-[10px] leading-4 text-ink-500">
                {pendingApproval.argsPreview}
              </p>
            ) : null}
            <div className="mt-2 flex items-center gap-2">
              <button
                type="button"
                aria-label={`Approve ${pendingApproval.tool}`}
                disabled={approvalBusy || pendingApproval.nonce === null || connection !== 'open'}
                className="flex min-h-9 flex-1 items-center justify-center gap-1.5 rounded-[8px] border border-verify bg-verify/10 text-[11px] font-semibold text-verify disabled:opacity-50"
                onClick={() => void resolveApproval(pendingApproval.approvalId, 'approve')}
              >
                <span aria-hidden="true">✓</span> Approve
              </button>
              <button
                type="button"
                aria-label={`Deny ${pendingApproval.tool}`}
                disabled={approvalBusy || pendingApproval.nonce === null || connection !== 'open'}
                className="flex min-h-9 flex-1 items-center justify-center gap-1.5 rounded-[8px] border border-signal bg-signal/10 text-[11px] font-semibold text-signal disabled:opacity-50"
                onClick={() => void resolveApproval(pendingApproval.approvalId, 'deny', 'operator denied in chat')}
              >
                <span aria-hidden="true">✕</span> Deny
              </button>
            </div>
          </div>
        ) : null}
        <form className="mt-2 flex h-14 items-center gap-2 rounded-[12px] border border-hairline bg-panel px-3" onSubmit={onSubmit}>
          <label className="sr-only" htmlFor="composer">
            {STRINGS.composer.placeholder}
          </label>
          <textarea
            id="composer"
            ref={composerRef}
            className="h-full min-w-0 flex-1 resize-none border-0 bg-transparent py-4 text-[13px] leading-[18px] text-ink-900 outline-none placeholder:text-ink-500 disabled:opacity-60"
            placeholder={STRINGS.composer.placeholder}
            value={draft}
            onChange={(event) => setDraft(event.target.value)}
            onKeyDown={(event) => {
              if (event.key === 'Enter' && !event.shiftKey) {
                event.preventDefault();
                void submit();
              }
            }}
            rows={1}
            disabled={connection !== 'open' || replaying}
          />
          <input
            ref={fileInputRef}
            type="file"
            className="hidden"
            accept=".txt,.md,.json,.jsonl,.py,.js,.mjs,.cjs,.ts,.tsx,.jsx,.rs,.css,.html,.yml,.yaml,.toml,.csv,.log"
            onChange={(event) => void attachFile(event.target.files?.[0] ?? null)}
          />
          <button
            type="button"
            className="flex h-8 w-8 shrink-0 items-center justify-center rounded-full border border-hairline text-ink-700 hover:bg-honey-100 disabled:cursor-not-allowed disabled:opacity-50"
            aria-label="Attach a text file to the prompt"
            title="Attach a text file (up to 60 KB); its contents are added to the prompt."
            disabled={connection !== 'open' || replaying}
            onClick={() => fileInputRef.current?.click()}
          >
            <svg width="14" height="14" viewBox="0 0 18 18" aria-hidden="true">
              <path d="M6.2 9.2 10.8 4.6a2.4 2.4 0 0 1 3.4 3.4l-5.6 5.6a3.6 3.6 0 0 1-5.1-5.1l5.4-5.4" fill="none" stroke="currentColor" strokeWidth="1.4" strokeLinecap="round" />
            </svg>
          </button>
          <button
            type="button"
            className="flex h-8 w-8 shrink-0 items-center justify-center rounded-full border border-hairline text-ink-800 hover:bg-honey-100 disabled:cursor-not-allowed disabled:opacity-50"
            aria-label="Stop the active run"
            disabled={connection !== 'open' || replaying || (streamingRunId === null && runStatus !== 'running')}
            onClick={() => {
              void cancelRun();
            }}
          >
            <svg width="11" height="11" viewBox="0 0 12 12" aria-hidden="true">
              <rect x="2.5" y="2.5" width="7" height="7" rx="1" fill="currentColor" />
            </svg>
          </button>
          <button
            type="submit"
            className="flex h-8 w-8 shrink-0 items-center justify-center rounded-full bg-honey-300 text-ink-900 hover:bg-honey-400 disabled:cursor-not-allowed disabled:opacity-50"
            aria-label={STRINGS.composer.submit}
            disabled={connection !== 'open' || replaying || submitting || draft.trim().length === 0}
          >
            <svg width="15" height="15" viewBox="0 0 20 20" aria-hidden="true">
              <path d="M3 10h13M11 4l6 6-6 6" fill="none" stroke="currentColor" strokeWidth="1.8" strokeLinecap="round" strokeLinejoin="round" />
            </svg>
          </button>
        </form>
        {attachError ? <p className="type-mono-meta mt-1 text-signal">{attachError}</p> : null}
        <p className="type-mono-meta mt-1 text-meta">{STRINGS.composer.hint}</p>
        <p className="type-mono-meta mt-1 text-[10px] leading-4 text-meta">
          CAPABILITIES: text + workspace files + approved tools. <strong className="text-ink-700">No screen, vision, or desktop control.</strong>
        </p>
        <div className="mt-1 shrink-0 border-t border-hairline pt-1">
          <div className="flex items-center gap-1">
            <button
              type="button"
              aria-expanded={dockTab === 'controls'}
              className="type-mono-meta min-h-8 rounded-[6px] px-2 py-1 text-[10px] text-ink-700 hover:bg-honey-100 aria-expanded:bg-honey-200"
              onClick={() => setDockTab(dockTab === 'controls' ? null : 'controls')}
            >
              CONTROLS
            </button>
            <button
              type="button"
              aria-expanded={dockTab === 'state'}
              className="type-mono-meta min-h-8 rounded-[6px] px-2 py-1 text-[10px] text-ink-700 hover:bg-honey-100 aria-expanded:bg-honey-200"
              onClick={() => setDockTab(dockTab === 'state' ? null : 'state')}
            >
              STATE &amp; SETTINGS
            </button>
            <button
              type="button"
              aria-label="Clear the visible chat transcript"
              title="Clear the transcript shown here. The run ledger keeps the recorded history."
              className="type-mono-meta ml-auto min-h-8 rounded-[6px] px-2 py-1 text-[10px] text-ink-500 hover:bg-honey-100"
              onClick={() => store.setState({ messages: [], refusals: [] })}
            >
              CLEAR
            </button>
            <button
              type="button"
              aria-label="Hide the chat dock"
              className="type-mono-meta min-h-8 rounded-[6px] px-2 py-1 text-[10px] text-ink-500 hover:bg-honey-100"
              onClick={() => setChatVisible(false)}
            >
              HIDE
            </button>
          </div>
          {dockTab === null ? null : (
            <div className="dock-tray mt-1 max-h-[190px] overflow-y-auto">
              {dockTab === 'controls' ? (
                <ControlBar />
              ) : (
                <div className="grid grid-cols-2 gap-x-3 gap-y-1 px-1 py-1">
                  <span className="type-mono-meta text-[10px] text-meta">provider</span>
                  <span className="type-mono-code truncate text-[10px] text-ink-800">{settings.provider ?? '—'}</span>
                  <span className="type-mono-meta text-[10px] text-meta">model</span>
                  <span className="type-mono-code truncate text-[10px] text-ink-800">{settings.model ?? '—'}</span>
                  <span className="type-mono-meta text-[10px] text-meta">key</span>
                  <span className="type-mono-code text-[10px] text-ink-800">{settings.keyPresent === null ? '—' : settings.keyPresent ? 'present' : 'missing'}</span>
                  <span className="type-mono-meta text-[10px] text-meta">reasoning</span>
                  <span className="type-mono-code text-[10px] text-ink-800">{settings.reasoningEffort ?? '—'}</span>
                  <span className="type-mono-meta text-[10px] text-meta">version</span>
                  <span className="type-mono-code text-[10px] text-ink-800">{APP_VERSION}</span>
                  <div className="col-span-2 mt-1 flex flex-wrap gap-2 border-t border-hairline pt-1">
                    {DOT_STATES.map((state) => (
                      <span key={state} className="type-mono-meta flex items-center gap-1 text-[10px] text-meta">
                        <StateDot state={state} size={7} />
                        {visualFor(state).label}
                      </span>
                    ))}
                  </div>
                </div>
              )}
            </div>
          )}
        </div>
      </section>
    </div>
  );
}
