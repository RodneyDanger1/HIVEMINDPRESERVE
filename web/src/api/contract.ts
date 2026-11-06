/**
 * Frozen wire contract (docs/contracts/UI-COMMANDS-v1.md §1-§5 + backend
 * `hiverouter-contract`). This file is the single client-side source of
 * command/event names, payload shapes and refusal codes; the API client and
 * the store only speak through it. Pure functions -> unit-testable.
 */
import { isAgentState, type AgentState } from '../state/agentState';

export const SCHEMA_VERSION = 'hiverouter.v1' as const;
export const MAX_PROMPT_CHARS = 8000;
export const MIN_ZOOM = 0.1;
export const MAX_ZOOM = 3.0;

export type FrameKind = 'COMMAND' | 'EVENT';

export interface Envelope<TPayload = unknown> {
  readonly schema: string;
  readonly type: FrameKind;
  readonly name: string;
  readonly id: string;
  readonly correlation_id: string;
  readonly run_id: string | null;
  readonly task_id: string | null;
  readonly seq: number | null;
  readonly ts: string;
  readonly payload: TPayload;
}

/* ------------------------------------------------------------------ commands */

export interface SubmitPromptPayload {
  readonly text: string;
  readonly project_id: string;
}
export interface SpawnSubHivePayload {
  readonly parent_task_id: string;
  readonly role: string;
}
export interface RewireTaskPayload {
  readonly task_id: string;
  readonly new_parent_id: string;
  readonly expected_rev: number;
}
export interface CancelRunPayload {
  readonly run_id: string;
}
export interface KillWorkerPayload {
  readonly task_id: string;
}
export interface ChatPinPayload {
  readonly pinned: boolean;
}
export interface TemplateInsertPayload {
  readonly template_id: string;
  readonly project_id: string;
}
export interface HistoryRevisionSetPayload {
  readonly run_id: string;
  readonly revision: number;
}
export interface ViewportSetPayload {
  readonly x: number;
  readonly y: number;
  readonly zoom: number;
}
export type ViewportRecenterPayload = Record<string, never>;

/* Contract patch 2026-11-06: approval, settings, file-tree commands. */

export interface ApprovalResolvePayload {
  readonly approval_id: string;
  readonly nonce: string;
  readonly decision: 'approve' | 'deny';
  readonly reason?: string;
}
export type SettingsStatusPayload = Record<string, never>;
export type FileTreeListPayload = Record<string, never>;
export interface FileReadPayload {
  /** Workspace-relative path; the server canonicalises and jails it. */
  readonly path: string;
}

export interface CommandPayloads {
  readonly SUBMIT_PROMPT: SubmitPromptPayload;
  readonly SPAWN_SUB_HIVE: SpawnSubHivePayload;
  readonly REWIRE_TASK: RewireTaskPayload;
  readonly CANCEL_RUN: CancelRunPayload;
  readonly KILL_WORKER: KillWorkerPayload;
  readonly APPROVAL_RESOLVE: ApprovalResolvePayload;
  readonly SETTINGS_STATUS: SettingsStatusPayload;
  readonly FILE_TREE_LIST: FileTreeListPayload;
  readonly FILE_READ: FileReadPayload;
  readonly CHAT_PIN: ChatPinPayload;
  readonly TEMPLATE_INSERT: TemplateInsertPayload;
  readonly HISTORY_REVISION_SET: HistoryRevisionSetPayload;
  readonly VIEWPORT_SET: ViewportSetPayload;
  readonly VIEWPORT_RECENTER: ViewportRecenterPayload;
}

export type CommandName = keyof CommandPayloads;

export const FROZEN_COMMAND_NAMES = [
  'SUBMIT_PROMPT',
  'SPAWN_SUB_HIVE',
  'REWIRE_TASK',
  'CANCEL_RUN',
  'KILL_WORKER',
  'APPROVAL_RESOLVE',
  'SETTINGS_STATUS',
  'FILE_TREE_LIST',
  'FILE_READ',
  'CHAT_PIN',
  'TEMPLATE_INSERT',
  'HISTORY_REVISION_SET',
  'VIEWPORT_SET',
  'VIEWPORT_RECENTER',
] as const satisfies readonly CommandName[];

/** Commands that actually go over the WebSocket (VIEWPORT_* are client-only). */
export const SERVER_COMMAND_NAMES = [
  'SUBMIT_PROMPT',
  'SPAWN_SUB_HIVE',
  'REWIRE_TASK',
  'CANCEL_RUN',
  'KILL_WORKER',
  'APPROVAL_RESOLVE',
  'SETTINGS_STATUS',
  'FILE_TREE_LIST',
  'FILE_READ',
  'CHAT_PIN',
  'TEMPLATE_INSERT',
  'HISTORY_REVISION_SET',
] as const satisfies readonly CommandName[];

/** Decision-gated names: present in the contract, frozen "do not implement". */
export const DECISION_GATED_COMMAND_NAMES = [
  'DIFF_PROPOSE',
  'DIFF_APPLY',
  'DIFF_REJECT',
] as const;

export type FrozenCommandName = (typeof FROZEN_COMMAND_NAMES)[number];
export type ServerCommandName = (typeof SERVER_COMMAND_NAMES)[number];

export function isCommandName(value: unknown): value is CommandName {
  return typeof value === 'string' && (FROZEN_COMMAND_NAMES as readonly string[]).includes(value);
}

/* -------------------------------------------------------------------- events */

export interface Usage {
  readonly prompt_tokens: number | null;
  readonly completion_tokens: number | null;
  readonly total_tokens: number | null;
  readonly prompt_cache_hit_tokens: number | null;
  readonly prompt_cache_miss_tokens: number | null;
  readonly reasoning_tokens: number | null;
}

export interface AckPayload {
  readonly command: CommandName;
  readonly correlation_id: string;
}
export interface ErrorPayload {
  readonly code: string;
  readonly message: string;
  readonly how_to_fix: string;
  readonly retryable: boolean;
}
export interface ServerHelloPayload {
  readonly last_seq: number | null;
  readonly schema: string;
}
export interface ResyncRequiredPayload {
  readonly expected_seq: number;
  readonly last_seq: number;
}
export interface RunCreatedPayload {
  readonly run_id: string;
  readonly status: AgentState;
}
export interface RunStatusPayload {
  readonly run_id: string;
  readonly status: AgentState;
  readonly error_code?: string | null;
}
export interface ReplyDeltaPayload {
  readonly run_id: string;
  readonly text_delta: string;
}
export interface ReplyDonePayload {
  readonly run_id: string;
  readonly usage: Usage;
}
/** Node type vocabulary: 4 node types + 1 broken-route error state (R-U5). */
export const NODE_KINDS = ['hyperagent', 'superagent', 'worker', 'tool', 'error'] as const;
export type NodeKind = (typeof NODE_KINDS)[number];
export function isNodeKind(value: unknown): value is NodeKind {
  return typeof value === 'string' && (NODE_KINDS as readonly string[]).includes(value);
}

/**
 * Frozen contract payload is `{task_id, state}`. Rendering metadata (kind,
 * label, parent, progress) is read when the server supplies it as extra
 * fields; absent fields stay absent (the card renders a generic task card and
 * the gap is recorded in PHASE-7-8-10-EVIDENCE.md). No metadata is invented.
 */
export interface NodeEventPayload {
  readonly task_id: string;
  readonly state: AgentState;
  readonly kind?: NodeKind | null;
  readonly label?: string | null;
  readonly parent_task_id?: string | null;
  readonly progress?: number | null;
  readonly error_code?: string | null;
}
export interface RunMetricsPayload {
  readonly superagents: number;
  readonly workers: number;
  readonly green_pct: number;
  readonly delta: number;
}
export interface ApprovalRequestPayload {
  readonly approval_id: string;
  readonly tool: string;
  readonly decision?: string | null;
  /** Patched contract: nonce arrives with the pending approval (optional for back-compat). */
  readonly nonce?: string | null;
}
export interface ApprovalResolvedPayload {
  readonly approval_id: string;
  readonly tool: string;
  readonly decision: string;
}
export interface HistorySnapshotPayload {
  readonly run_id: string;
  readonly revision: number;
  readonly graph: unknown;
}

/* Contract patch 2026-11-06: settings + file results. */
export interface SettingsStatusResultPayload {
  readonly provider: string;
  readonly model: string;
  readonly key_present: boolean;
  readonly thinking?: string;
  readonly reasoning_effort?: string;
}
export type FileTreeEntryKind = 'file' | 'dir';
export interface FileTreeEntry {
  readonly path: string;
  readonly name: string;
  readonly kind: FileTreeEntryKind;
  readonly size_bytes: number | null;
}
export interface FileTreeResultPayload {
  readonly root: string;
  readonly entries: readonly FileTreeEntry[];
}
export interface FileContentPayload {
  readonly path: string;
  readonly sha256: string;
  readonly bytes: number;
  readonly text: string;
}
export type EmptyPayload = Record<string, never>;

export interface EventPayloads {
  readonly ACK: AckPayload;
  readonly ERROR: ErrorPayload;
  readonly SERVER_HELLO: ServerHelloPayload;
  readonly RESYNC_REQUIRED: ResyncRequiredPayload;
  readonly RUN_CREATED: RunCreatedPayload;
  readonly RUN_STATUS: RunStatusPayload;
  readonly REPLY_DELTA: ReplyDeltaPayload;
  readonly REPLY_DONE: ReplyDonePayload;
  readonly NODE_SPAWN: NodeEventPayload;
  readonly NODE_DRAG: NodeEventPayload;
  readonly NODE_REWIRE: NodeEventPayload;
  readonly NODE_STATE: NodeEventPayload;
  readonly RUN_METRICS: RunMetricsPayload;
  readonly APPROVAL_REQUEST: ApprovalRequestPayload;
  readonly APPROVAL_RESOLVED: ApprovalResolvedPayload;
  readonly SETTINGS_STATUS_RESULT: SettingsStatusResultPayload;
  readonly FILE_TREE_RESULT: FileTreeResultPayload;
  readonly FILE_CONTENT: FileContentPayload;
  readonly HISTORY_SNAPSHOT: HistorySnapshotPayload;
  readonly PING: EmptyPayload;
  readonly PONG: EmptyPayload;
}

export type EventName = keyof EventPayloads;

export const EVENT_NAMES = [
  'ACK',
  'ERROR',
  'SERVER_HELLO',
  'RESYNC_REQUIRED',
  'RUN_CREATED',
  'RUN_STATUS',
  'REPLY_DELTA',
  'REPLY_DONE',
  'NODE_SPAWN',
  'NODE_DRAG',
  'NODE_REWIRE',
  'NODE_STATE',
  'RUN_METRICS',
  'APPROVAL_REQUEST',
  'APPROVAL_RESOLVED',
  'SETTINGS_STATUS_RESULT',
  'FILE_TREE_RESULT',
  'FILE_CONTENT',
  'HISTORY_SNAPSHOT',
  'PING',
  'PONG',
] as const satisfies readonly EventName[];

export function isEventName(value: unknown): value is EventName {
  return typeof value === 'string' && (EVENT_NAMES as readonly string[]).includes(value);
}

/* ------------------------------------------------------------------ refusals */

export interface Refusal {
  readonly code: string;
  readonly message: string;
  readonly how_to_fix: string;
  readonly retryable: boolean;
}

/** Every refusal code printed in UI-COMMANDS-v1 §5 (stable identifiers). */
export const KNOWN_REFUSAL_CODES = [
  'E_ORIGIN_REJECTED',
  'E_TOKEN_INVALID',
  'E_RESYNC_REQUIRED',
  'E_ZOOM_RANGE',
  'E_HISTORY_GAP',
  'E_DB_BUSY',
  'E_WS_DISCONNECTED',
  'E_COMMAND_UNKNOWN',
  'E_SCHEMA_INVALID',
  'E_PROMPT_EMPTY',
  'E_PROMPT_TOO_LONG',
  'E_RUN_ACTIVE',
  'E_KEY_MISSING',
  'E_SPAWN_DEPTH',
  'E_FANOUT_LIMIT',
  'E_CYCLE',
  'E_REWIRE_IN_FLIGHT',
  'E_RUN_TERMINAL',
  'E_RUN_NOT_FOUND',
  'E_TASK_NOT_FOUND',
  'E_TASK_TERMINAL',
  'E_KILL_DENIED',
  'E_BUDGET_EXCEEDED',
  'E_APPROVAL_REPLAY',
  'E_APPROVAL_TIMEOUT',
  'E_APPROVAL_NOT_FOUND',
  'E_APPROVAL_DECISION_INVALID',
  'E_DIFF_DECISION_REQUIRED',
  'E_TEMPLATE_UNKNOWN',
  'E_TEMPLATE_LICENSE_MISSING',
  'E_TEMPLATE_REF_FORBIDDEN',
  'E_PATH_OUTSIDE_JAIL',
  'E_PATH_NOT_FOUND',
  'E_FILE_TOO_LARGE',
  'E_TOOL_NOT_FOUND',
  'E_TOOL_SCHEMA',
  'E_JAIL_ESCAPE',
  'E_TOOL_TIMEOUT',
  'E_TOOL_CANCELLED',
  'E_PROVIDER_AUTH',
  'E_PROVIDER_RATE_LIMIT',
  'E_PROVIDER_TIMEOUT',
  'E_PROVIDER_INCOMPLETE',
  'E_TOOLCHAIN_MSRV',
  'E_NODE_MISSING',
] as const;

export type RefusalCode = (typeof KNOWN_REFUSAL_CODES)[number];

/* ------------------------------------------------------------- parse + build */

export type ParseResult<T> =
  | { readonly ok: true; readonly value: T }
  | { readonly ok: false; readonly code: string; readonly detail: string };

function isRecord(value: unknown): value is Record<string, unknown> {
  return typeof value === 'object' && value !== null && !Array.isArray(value);
}

function readString(source: Record<string, unknown>, key: string): string | null {
  const value = source[key];
  return typeof value === 'string' && value.length > 0 ? value : null;
}

function readNumber(source: Record<string, unknown>, key: string): number | null {
  const value = source[key];
  return typeof value === 'number' && Number.isFinite(value) ? value : null;
}

function readBoolean(source: Record<string, unknown>, key: string): boolean | null {
  const value = source[key];
  return typeof value === 'boolean' ? value : null;
}

function readAgentState(source: Record<string, unknown>, key: string): AgentState | null {
  const value = source[key];
  return isAgentState(value) ? value : null;
}

/** Structural validation of any wire frame (UI-COMMANDS-v1 §1). */
export function validateEnvelope(value: unknown): ParseResult<Envelope<unknown>> {
  if (!isRecord(value)) {
    return { ok: false, code: 'E_SCHEMA_INVALID', detail: 'frame is not an object' };
  }
  const schema = readString(value, 'schema');
  if (schema !== SCHEMA_VERSION) {
    return { ok: false, code: 'E_SCHEMA_INVALID', detail: 'schema mismatch' };
  }
  const frameType = value.type;
  if (frameType !== 'COMMAND' && frameType !== 'EVENT') {
    return { ok: false, code: 'E_SCHEMA_INVALID', detail: 'type is not COMMAND|EVENT' };
  }
  const name = readString(value, 'name');
  const id = readString(value, 'id');
  const correlation = readString(value, 'correlation_id');
  const ts = readString(value, 'ts');
  const payload = value.payload;
  if (!name || !id || !correlation || !ts || !isRecord(payload)) {
    return { ok: false, code: 'E_SCHEMA_INVALID', detail: 'missing envelope field' };
  }
  const seq = value.seq;
  if (seq !== null && seq !== undefined && typeof seq !== 'number') {
    return { ok: false, code: 'E_SCHEMA_INVALID', detail: 'seq must be number|null' };
  }
  const runIdRaw = value.run_id;
  if (runIdRaw !== null && runIdRaw !== undefined && typeof runIdRaw !== 'string') {
    return { ok: false, code: 'E_SCHEMA_INVALID', detail: 'run_id must be string|null' };
  }
  const taskIdRaw = value.task_id;
  if (taskIdRaw !== null && taskIdRaw !== undefined && typeof taskIdRaw !== 'string') {
    return { ok: false, code: 'E_SCHEMA_INVALID', detail: 'task_id must be string|null' };
  }
  const runId = typeof runIdRaw === 'string' ? runIdRaw : null;
  const taskId = typeof taskIdRaw === 'string' ? taskIdRaw : null;
  if (frameType === 'EVENT' && !isEventName(name)) {
    return { ok: false, code: 'E_SCHEMA_INVALID', detail: `unknown event ${name}` };
  }
  if (frameType === 'COMMAND' && !isCommandName(name)) {
    return { ok: false, code: 'E_COMMAND_UNKNOWN', detail: `unknown command ${name}` };
  }
  return {
    ok: true,
    value: {
      schema,
      type: frameType,
      name,
      id,
      correlation_id: correlation,
      run_id: runId,
      task_id: taskId,
      seq: typeof seq === 'number' ? seq : null,
      ts,
      payload,
    },
  };
}

export type DecodedEvent =
  | { readonly name: 'ACK'; readonly payload: AckPayload }
  | { readonly name: 'ERROR'; readonly payload: ErrorPayload }
  | { readonly name: 'SERVER_HELLO'; readonly payload: ServerHelloPayload }
  | { readonly name: 'RESYNC_REQUIRED'; readonly payload: ResyncRequiredPayload }
  | { readonly name: 'RUN_CREATED'; readonly payload: RunCreatedPayload }
  | { readonly name: 'RUN_STATUS'; readonly payload: RunStatusPayload }
  | { readonly name: 'REPLY_DELTA'; readonly payload: ReplyDeltaPayload }
  | { readonly name: 'REPLY_DONE'; readonly payload: ReplyDonePayload }
  | { readonly name: 'NODE_SPAWN'; readonly payload: NodeEventPayload }
  | { readonly name: 'NODE_DRAG'; readonly payload: NodeEventPayload }
  | { readonly name: 'NODE_REWIRE'; readonly payload: NodeEventPayload }
  | { readonly name: 'NODE_STATE'; readonly payload: NodeEventPayload }
  | { readonly name: 'RUN_METRICS'; readonly payload: RunMetricsPayload }
  | { readonly name: 'APPROVAL_REQUEST'; readonly payload: ApprovalRequestPayload }
  | { readonly name: 'APPROVAL_RESOLVED'; readonly payload: ApprovalResolvedPayload }
  | { readonly name: 'SETTINGS_STATUS_RESULT'; readonly payload: SettingsStatusResultPayload }
  | { readonly name: 'FILE_TREE_RESULT'; readonly payload: FileTreeResultPayload }
  | { readonly name: 'FILE_CONTENT'; readonly payload: FileContentPayload }
  | { readonly name: 'HISTORY_SNAPSHOT'; readonly payload: HistorySnapshotPayload }
  | { readonly name: 'PING'; readonly payload: EmptyPayload }
  | { readonly name: 'PONG'; readonly payload: EmptyPayload };

function invalid(detail: string): { ok: false; code: string; detail: string } {
  return { ok: false, code: 'E_SCHEMA_INVALID', detail };
}

function decodePayload(name: EventName, payload: Record<string, unknown>): ParseResult<DecodedEvent> {
  switch (name) {
    case 'ACK': {
      const command = payload.command;
      const correlation = readString(payload, 'correlation_id');
      if (!isCommandName(command) || !correlation) {
        return invalid('ACK payload');
      }
      return { ok: true, value: { name, payload: { command, correlation_id: correlation } } };
    }
    case 'ERROR': {
      const code = readString(payload, 'code');
      const message = readString(payload, 'message');
      const howToFix = readString(payload, 'how_to_fix');
      const retryable = readBoolean(payload, 'retryable');
      if (!code || !message || !howToFix || retryable === null) {
        return invalid('ERROR payload');
      }
      return {
        ok: true,
        value: { name, payload: { code, message, how_to_fix: howToFix, retryable } },
      };
    }
    case 'SERVER_HELLO': {
      const schema = readString(payload, 'schema');
      const lastSeqRaw = payload.last_seq;
      const last_seq =
        lastSeqRaw === null || lastSeqRaw === undefined
          ? null
          : readNumber(payload, 'last_seq');
      if (!schema || (lastSeqRaw !== null && lastSeqRaw !== undefined && last_seq === null)) {
        return invalid('SERVER_HELLO payload');
      }
      return { ok: true, value: { name, payload: { last_seq, schema } } };
    }
    case 'RESYNC_REQUIRED': {
      const expected = readNumber(payload, 'expected_seq');
      const last = readNumber(payload, 'last_seq');
      if (expected === null || last === null) {
        return invalid('RESYNC_REQUIRED payload');
      }
      return { ok: true, value: { name, payload: { expected_seq: expected, last_seq: last } } };
    }
    case 'RUN_CREATED': {
      const runId = readString(payload, 'run_id');
      const status = readAgentState(payload, 'status');
      if (!runId || !status) {
        return invalid('RUN_CREATED payload');
      }
      return { ok: true, value: { name, payload: { run_id: runId, status } } };
    }
    case 'RUN_STATUS': {
      const runId = readString(payload, 'run_id');
      const status = readAgentState(payload, 'status');
      if (!runId || !status) {
        return invalid('RUN_STATUS payload');
      }
      const errorCode = payload.error_code;
      const error_code =
        typeof errorCode === 'string' ? errorCode : errorCode === null ? null : undefined;
      return { ok: true, value: { name, payload: { run_id: runId, status, error_code } } };
    }
    case 'REPLY_DELTA': {
      const runId = readString(payload, 'run_id');
      const delta = typeof payload.text_delta === 'string' ? payload.text_delta : null;
      if (!runId || delta === null) {
        return invalid('REPLY_DELTA payload');
      }
      return { ok: true, value: { name, payload: { run_id: runId, text_delta: delta } } };
    }
    case 'REPLY_DONE': {
      const runId = readString(payload, 'run_id');
      const usage = payload.usage;
      if (!runId || !isRecord(usage)) {
        return invalid('REPLY_DONE payload');
      }
      const nullable = (key: string): number | null => readNumber(usage, key);
      const decoded: Usage = {
        prompt_tokens: nullable('prompt_tokens'),
        completion_tokens: nullable('completion_tokens'),
        total_tokens: nullable('total_tokens'),
        prompt_cache_hit_tokens: nullable('prompt_cache_hit_tokens'),
        prompt_cache_miss_tokens: nullable('prompt_cache_miss_tokens'),
        reasoning_tokens: nullable('reasoning_tokens'),
      };
      return { ok: true, value: { name, payload: { run_id: runId, usage: decoded } } };
    }
    case 'NODE_SPAWN':
    case 'NODE_DRAG':
    case 'NODE_REWIRE':
    case 'NODE_STATE': {
      const taskId = readString(payload, 'task_id');
      const state = readAgentState(payload, 'state');
      if (!taskId || !state) {
        return invalid(`${name} payload`);
      }
      const kind = isNodeKind(payload.kind) ? payload.kind : null;
      const label = typeof payload.label === 'string' ? payload.label : null;
      const parent = typeof payload.parent_task_id === 'string' ? payload.parent_task_id : null;
      const progressRaw = payload.progress;
      const progress =
        typeof progressRaw === 'number' && Number.isFinite(progressRaw) ? progressRaw : null;
      const errorCode = typeof payload.error_code === 'string' ? payload.error_code : null;
      return {
        ok: true,
        value: {
          name,
          payload: {
            task_id: taskId,
            state,
            kind,
            label,
            parent_task_id: parent,
            progress,
            error_code: errorCode,
          },
        },
      };
    }
    case 'RUN_METRICS': {
      const superagents = readNumber(payload, 'superagents');
      const workers = readNumber(payload, 'workers');
      const green = readNumber(payload, 'green_pct');
      const delta = readNumber(payload, 'delta');
      if (superagents === null || workers === null || green === null || delta === null) {
        return invalid('RUN_METRICS payload');
      }
      return {
        ok: true,
        value: { name, payload: { superagents, workers, green_pct: green, delta } },
      };
    }
    case 'APPROVAL_REQUEST': {
      const id = readString(payload, 'approval_id');
      const tool = readString(payload, 'tool');
      if (!id || !tool) {
        return invalid('APPROVAL_REQUEST payload');
      }
      const decision = payload.decision;
      const nonce = typeof payload.nonce === 'string' ? payload.nonce : null;
      return {
        ok: true,
        value: {
          name,
          payload: {
            approval_id: id,
            tool,
            decision: typeof decision === 'string' ? decision : null,
            nonce,
          },
        },
      };
    }
    case 'APPROVAL_RESOLVED': {
      const id = readString(payload, 'approval_id');
      const tool = readString(payload, 'tool');
      const decision = readString(payload, 'decision');
      if (!id || !tool || !decision) {
        return invalid('APPROVAL_RESOLVED payload');
      }
      return { ok: true, value: { name, payload: { approval_id: id, tool, decision } } };
    }
    case 'SETTINGS_STATUS_RESULT': {
      const provider = readString(payload, 'provider');
      const model = readString(payload, 'model');
      const keyPresent = readBoolean(payload, 'key_present');
      if (!provider || !model || keyPresent === null) {
        return invalid('SETTINGS_STATUS_RESULT payload');
      }
      const thinking = typeof payload.thinking === 'string' ? payload.thinking : undefined;
      const reasoningEffort =
        typeof payload.reasoning_effort === 'string' ? payload.reasoning_effort : undefined;
      return {
        ok: true,
        value: {
          name,
          payload: {
            provider,
            model,
            key_present: keyPresent,
            ...(thinking ? { thinking } : {}),
            ...(reasoningEffort ? { reasoning_effort: reasoningEffort } : {}),
          },
        },
      };
    }
    case 'FILE_TREE_RESULT': {
      const root = readString(payload, 'root');
      const entriesRaw = payload.entries;
      if (!root || !Array.isArray(entriesRaw)) {
        return invalid('FILE_TREE_RESULT payload');
      }
      const entries: FileTreeEntry[] = [];
      for (const candidate of entriesRaw) {
        if (!isRecord(candidate)) {
          return invalid('FILE_TREE_RESULT entry');
        }
        const path = readString(candidate, 'path');
        const entryName = readString(candidate, 'name');
        const kindRaw = candidate.kind;
        const isDir = candidate.is_dir;
        const kind: FileTreeEntryKind =
          kindRaw === 'file' || kindRaw === 'dir'
            ? kindRaw
            : typeof isDir === 'boolean'
              ? isDir
                ? 'dir'
                : 'file'
              : 'file';
        const size = candidate.size_bytes;
        if (
          !path ||
          !entryName ||
          (kindRaw !== undefined && kindRaw !== 'file' && kindRaw !== 'dir' && typeof isDir !== 'boolean')
        ) {
          return invalid('FILE_TREE_RESULT entry');
        }
        entries.push({
          path,
          name: entryName,
          kind,
          size_bytes: typeof size === 'number' && Number.isFinite(size) ? size : null,
        });
      }
      return { ok: true, value: { name, payload: { root, entries } } };
    }
    case 'FILE_CONTENT': {
      const path = readString(payload, 'path');
      const sha256 = readString(payload, 'sha256');
      const bytes = readNumber(payload, 'bytes');
      const text = typeof payload.text === 'string' ? payload.text : null;
      if (!path || !sha256 || bytes === null || text === null) {
        return invalid('FILE_CONTENT payload');
      }
      return { ok: true, value: { name, payload: { path, sha256, bytes, text } } };
    }
    case 'HISTORY_SNAPSHOT': {
      const runId = readString(payload, 'run_id');
      const revision = readNumber(payload, 'revision');
      if (!runId || revision === null) {
        return invalid('HISTORY_SNAPSHOT payload');
      }
      return { ok: true, value: { name, payload: { run_id: runId, revision, graph: payload.graph } } };
    }
    case 'PING':
    case 'PONG':
      return { ok: true, value: { name, payload: {} } };
  }
}

/** Parse one WebSocket text frame into a validated event. */
export function parseEventFrame(
  raw: string,
): ParseResult<{ readonly envelope: Envelope<unknown>; readonly event: DecodedEvent }> {
  let parsed: unknown;
  try {
    parsed = JSON.parse(raw);
  } catch {
    return invalid('frame is not valid JSON');
  }
  const envelope = validateEnvelope(parsed);
  if (!envelope.ok) {
    return envelope;
  }
  if (envelope.value.type !== 'EVENT') {
    return invalid('server sent a non-EVENT frame');
  }
  const name = envelope.value.name;
  if (!isEventName(name)) {
    return invalid(`unknown event ${name}`);
  }
  if (!isRecord(envelope.value.payload)) {
    return invalid('payload is not an object');
  }
  const event = decodePayload(name, envelope.value.payload);
  if (!event.ok) {
    return event;
  }
  return { ok: true, value: { envelope: envelope.value, event: event.value } };
}

export interface IdFactory {
  (): string;
}
export interface Clock {
  (): string;
}

export interface NewCommandFrameOptions {
  readonly newId: IdFactory;
  readonly now: Clock;
}

/** Build a COMMAND envelope; correlation_id starts equal to id (§1). */
export function buildCommandEnvelope<N extends CommandName>(
  name: N,
  payload: CommandPayloads[N],
  options: NewCommandFrameOptions,
): Envelope<CommandPayloads[N]> {
  const id = options.newId();
  return {
    schema: SCHEMA_VERSION,
    type: 'COMMAND',
    name,
    id,
    correlation_id: id,
    run_id: null,
    task_id: null,
    seq: null,
    ts: options.now(),
    payload,
  };
}

/** Raw hello control frame the Phase-2 transport expects (docs/plans/PHASE-2-RECON.md). */
export interface HelloFrame {
  readonly type: 'hello';
  readonly last_seq: number;
  readonly token?: string;
}

export function buildHelloFrame(lastSeq: number): HelloFrame {
  return { type: 'hello', last_seq: lastSeq };
}

/** Pre-send validation: return a frozen refusal code or null. */
export function validatePromptText(text: string): RefusalCode | null {
  const length = [...text].length;
  if (length === 0) {
    return 'E_PROMPT_EMPTY';
  }
  if (length > MAX_PROMPT_CHARS) {
    return 'E_PROMPT_TOO_LONG';
  }
  return null;
}

/** Pre-send zoom validation; the server validates again (E_ZOOM_RANGE). */
export function validateZoom(zoom: number): RefusalCode | null {
  if (!Number.isFinite(zoom) || zoom < MIN_ZOOM || zoom > MAX_ZOOM) {
    return 'E_ZOOM_RANGE';
  }
  return null;
}

const MAX_FILE_BYTES = 1024 * 1024;

/**
 * Client-side jail pre-check for FILE_READ/FILE_TREE paths (the server is
 * authoritative and repeats the check + canonicalisation). Rejects absolute
 * paths, drive letters, UNC, empty paths and any `..` segment.
 */
export function validateWorkspacePath(path: string): RefusalCode | null {
  if (typeof path !== 'string' || path.trim().length === 0) {
    return 'E_PATH_NOT_FOUND';
  }
  const normalized = path.replace(/\\/g, '/').trim();
  if (normalized.startsWith('/') || /^[a-zA-Z]:/.test(normalized) || normalized.startsWith('//')) {
    return 'E_PATH_OUTSIDE_JAIL';
  }
  if (normalized.split('/').some((segment) => segment === '..')) {
    return 'E_PATH_OUTSIDE_JAIL';
  }
  return null;
}

/** File preview size limit from the patched contract (>1 MiB => E_FILE_TOO_LARGE). */
export function checkFileSize(bytes: number): RefusalCode | null {
  return Number.isFinite(bytes) && bytes > MAX_FILE_BYTES ? 'E_FILE_TOO_LARGE' : null;
}

export function toRefusalPayload(refusal: Refusal): ErrorPayload {
  return {
    code: refusal.code,
    message: refusal.message,
    how_to_fix: refusal.how_to_fix,
    retryable: refusal.retryable,
  };
}
