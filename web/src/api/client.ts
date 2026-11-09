/**
 * Typed UI API client over the localhost WebSocket transport.
 *
 * Wiring (docs/plans/PHASE-2-RECON.md + UI-COMMANDS-v1):
 *   GET /api/session -> {token, schema, last_seq, ws}
 *   ws://127.0.0.1:8080/ws?token=<uuid-v7>&last_seq=<n>
 *   then a raw `{"type":"hello","last_seq":n}` control frame, then ENVELOPE frames.
 *
 * Commands resolve only on ACK and reject only on a typed ERROR frame (or a
 * transport refusal) — there is no fake success on any path. partysocket
 * supplies reconnection; every reopen replays hello{last_seq}.
 */
import { WebSocket as ReconnectingWebSocket } from 'partysocket';
import {
  buildCommandEnvelope,
  buildHelloFrame,
  parseEventFrame,
  validateWorkspacePath,
  type CommandPayloads,
  type DecodedEvent,
  type Envelope,
  type Refusal,
  type ServerCommandName,
} from './contract';
import { nowIso, uuidv7 } from './ids';
import { DEFAULT_HTTP_TIMEOUT_MS, fetchWithBoundTimeout } from './http';
import { RefusalError, clientRefusal } from './refusals';

export type ConnectionStatus = 'idle' | 'connecting' | 'open' | 'reconnecting' | 'closed' | 'error';

export interface SessionInfo {
  readonly token: string;
  readonly schema: string;
  readonly last_seq: number;
  readonly ws: string;
}

export interface Transport {
  send(data: string): void;
  close(): void;
  onOpen(listener: () => void): void;
  onClose(listener: (code?: number) => void): void;
  onMessage(listener: (data: unknown) => void): void;
  onError(listener: (message?: string) => void): void;
}

export type TransportFactory = (url: string) => Transport;

export interface UiApi {
  readonly kind: 'ws' | 'mock';
  open(): Promise<SessionInfo>;
  close(): void;
  send<N extends ServerCommandName>(
    name: N,
    payload: CommandPayloads[N],
    options?: { readonly correlationId?: string; readonly timeoutMs?: number },
  ): Promise<void>;
  subscribe(listener: (event: DecodedEvent, envelope: Envelope<unknown>) => void): () => void;
  subscribeStatus(listener: (status: ConnectionStatus, detail?: Refusal) => void): () => void;
  subscribeClientError(listener: (refusal: Refusal) => void): () => void;
}

/**
 * Optional capability: ask the server to re-publish its bounded batch of open
 * approvals on demand.
 *
 * This is not a new wire command. The pull reuses the `hello` control frame the
 * reconnect path already sends (see {@link WsApiClient.pullPendingApprovals}),
 * which the server answers with `SERVER_HELLO` plus ordinary
 * `APPROVAL_REQUEST` frames (replace-by-id, so replay is idempotent). It stays
 * out of {@link UiApi} so that implementations that cannot pull — the dev mock
 * and the test doubles implementing `UiApi` — are unaffected; narrow with
 * {@link supportsPendingApprovalPull} before calling it.
 */
export interface PendingApprovalPull {
  pullPendingApprovals(): Promise<void>;
}

/** Narrows a {@link UiApi} to one that can pull pending approvals. */
export function supportsPendingApprovalPull(
  api: UiApi,
): api is UiApi & PendingApprovalPull {
  const candidate = api as Partial<PendingApprovalPull>;
  return typeof candidate.pullPendingApprovals === 'function';
}

export interface WsApiClientOptions {
  readonly baseUrl?: string;
  readonly wsUrl?: string;
  readonly fetchImpl?: typeof fetch;
  readonly transportFactory?: TransportFactory;
  readonly newId?: () => string;
  readonly now?: () => string;
  readonly requestTimeoutMs?: number;
  readonly handshakeTimeoutMs?: number;
  /** Hard upper bound for the `GET /api/session` bootstrap request. */
  readonly httpTimeoutMs?: number;
}

interface PendingCommand {
  readonly command: ServerCommandName;
  readonly resolve: () => void;
  readonly reject: (error: Error) => void;
  readonly timer: ReturnType<typeof setTimeout>;
}

export class PartyTransport implements Transport {
  private readonly socket: ReconnectingWebSocket;

  constructor(url: string) {
    this.socket = new ReconnectingWebSocket(url, null, {
      maxRetries: 12,
      minReconnectionDelay: 500,
      maxReconnectionDelay: 8000,
      reconnectionDelayGrowFactor: 1.6,
      connectionTimeout: 6000,
      maxEnqueuedMessages: 0,
    });
  }

  send(data: string): void {
    this.socket.send(data);
  }

  close(): void {
    this.socket.close();
  }

  onOpen(listener: () => void): void {
    this.socket.addEventListener('open', () => listener());
  }

  onClose(listener: (code?: number) => void): void {
    this.socket.addEventListener('close', (event) => listener(event.code));
  }

  onMessage(listener: (data: unknown) => void): void {
    this.socket.addEventListener('message', (event) => listener(event.data));
  }

  onError(listener: (message?: string) => void): void {
    this.socket.addEventListener('error', (event) => listener(event.message));
  }
}

function defaultBaseUrl(): string {
  const configured = import.meta.env.VITE_API_BASE;
  if (typeof configured === 'string' && configured.trim().length > 0) {
    return configured.replace(/\/+$/, '');
  }
  if (typeof window !== 'undefined' && window.location.port === '5173') {
    return 'http://127.0.0.1:8080';
  }
  return '';
}

/** Base URL used by diagnostics (e.g. GET /health) — same resolution as the client. */
export function apiBaseUrl(): string {
  return defaultBaseUrl();
}

function httpToWs(baseUrl: string): string {
  if (baseUrl.startsWith('https://')) {
    return `wss://${baseUrl.slice('https://'.length)}`;
  }
  if (baseUrl.startsWith('http://')) {
    return `ws://${baseUrl.slice('http://'.length)}`;
  }
  return baseUrl;
}

function parseRefusalBody(body: unknown): Refusal | null {
  if (typeof body !== 'object' || body === null) {
    return null;
  }
  const record = body as Record<string, unknown>;
  const nested =
    typeof record.error === 'object' && record.error !== null
      ? (record.error as Record<string, unknown>)
      : record;
  const code = nested.code;
  const message = nested.message;
  const howToFix = nested.how_to_fix;
  const retryable = nested.retryable;
  if (typeof code !== 'string') {
    return null;
  }
  return {
    code,
    message: typeof message === 'string' ? message : clientRefusal(code).message,
    how_to_fix: typeof howToFix === 'string' ? howToFix : clientRefusal(code).how_to_fix,
    retryable: typeof retryable === 'boolean' ? retryable : clientRefusal(code).retryable,
  };
}

export class WsApiClient implements UiApi {
  readonly kind = 'ws' as const;
  private readonly requestTimeoutMs: number;
  private readonly handshakeTimeoutMs: number;
  private readonly httpTimeoutMs: number;
  private readonly fetchImpl: typeof fetch | undefined;
  private readonly baseUrl: string | undefined;
  private readonly wsUrl: string | undefined;
  private readonly transportFactory: TransportFactory | undefined;
  private transport: Transport | null = null;
  private session: SessionInfo | null = null;
  private status: ConnectionStatus = 'idle';
  private lastSeq = 0;
  private manuallyClosed = false;
  private recoveryTimer: ReturnType<typeof setTimeout> | null = null;
  private recoveryInFlight = false;
  private opening: Promise<SessionInfo> | null = null;
  /**
   * Incremented by every close(). An open() that started before the increment
   * belongs to a dead socket generation: it must never publish a session,
   * transport, or status after a StrictMode dispose/re-init.
   */
  private closeEpoch = 0;
  private readonly pending = new Map<string, PendingCommand>();
  private readonly eventListeners = new Set<
    (event: DecodedEvent, envelope: Envelope<unknown>) => void
  >();
  private readonly statusListeners = new Set<
    (status: ConnectionStatus, detail?: Refusal) => void
  >();
  private readonly clientErrorListeners = new Set<(refusal: Refusal) => void>();
  private readonly newId: () => string;
  private readonly now: () => string;

  constructor(options: WsApiClientOptions = {}) {
    this.requestTimeoutMs = options.requestTimeoutMs ?? 15000;
    this.handshakeTimeoutMs = options.handshakeTimeoutMs ?? 8000;
    this.httpTimeoutMs = options.httpTimeoutMs ?? DEFAULT_HTTP_TIMEOUT_MS;
    this.fetchImpl = options.fetchImpl;
    this.baseUrl = options.baseUrl;
    this.wsUrl = options.wsUrl;
    this.transportFactory = options.transportFactory;
    this.newId = options.newId ?? uuidv7;
    this.now = options.now ?? nowIso;
  }

  getStatus(): ConnectionStatus {
    return this.status;
  }

  async open(): Promise<SessionInfo> {
    if (this.session && this.status === 'open') {
      return this.session;
    }
    if (this.opening) {
      return this.opening;
    }
    const opening = this.openInternal().finally(() => {
      if (this.opening === opening) {
        this.opening = null;
      }
    });
    this.opening = opening;
    return opening;
  }

  private setStatus(status: ConnectionStatus, detail?: Refusal): void {
    this.status = status;
    for (const listener of this.statusListeners) {
      listener(status, detail);
    }
  }

  private async openInternal(): Promise<SessionInfo> {
    const epoch = this.closeEpoch;
    const superseded = (): RefusalError =>
      new RefusalError(
        clientRefusal('E_WS_DISCONNECTED', {
          message: 'Session open was superseded by a disconnect.',
        }),
      );
    const ensureCurrent = (): void => {
      if (epoch !== this.closeEpoch) {
        throw superseded();
      }
    };
    this.setStatus('connecting');
    const baseUrl = this.baseUrl ?? defaultBaseUrl();
    let response: Response;
    try {
      response = await fetchWithBoundTimeout(`${baseUrl}/api/session`, {
        label: 'GET /api/session',
        timeoutMs: this.httpTimeoutMs,
        fetchImpl: this.fetchImpl,
        init: {
          credentials: 'same-origin',
          headers: { accept: 'application/json' },
        },
      });
      ensureCurrent();
    } catch (error) {
      if (epoch !== this.closeEpoch) {
        throw superseded();
      }
      const refusal =
        error instanceof RefusalError ? error.refusal : clientRefusal('E_WS_DISCONNECTED');
      this.setStatus('error', refusal);
      throw new RefusalError(refusal);
    }
    if (!response.ok) {
      ensureCurrent();
      let body: unknown = null;
      try {
        body = await response.json();
      } catch {
        body = null;
      }
      ensureCurrent();
      const refusal = parseRefusalBody(body) ?? clientRefusal('E_ORIGIN_REJECTED');
      this.setStatus('error', refusal);
      throw new RefusalError(refusal);
    }
    const body = (await response.json()) as Record<string, unknown>;
    ensureCurrent();
    const token = body.token;
    const schema = body.schema;
    const lastSeq = body.last_seq;
    const wsPath = body.ws;
    if (
      typeof token !== 'string' ||
      typeof schema !== 'string' ||
      typeof lastSeq !== 'number' ||
      typeof wsPath !== 'string'
    ) {
      const refusal = clientRefusal('E_SCHEMA_INVALID', {
        message: 'GET /api/session did not match the frozen session shape.',
      });
      this.setStatus('error', refusal);
      throw new RefusalError(refusal);
    }
    this.session = { token, schema, last_seq: lastSeq, ws: wsPath };
    this.lastSeq = lastSeq;
    ensureCurrent();
    const base = this.wsUrl ?? `${httpToWs(baseUrl)}${wsPath}`;
    const separator = base.includes('?') ? '&' : '?';
    const url = `${base}${separator}token=${encodeURIComponent(token)}&last_seq=${lastSeq}`;
    const factory = this.transportFactory ?? ((target: string) => new PartyTransport(target));
    const transport = factory(url);
    this.transport = transport;
    this.manuallyClosed = false;
    this.attach(transport);
    await this.waitForOpen(transport);
    if (epoch !== this.closeEpoch) {
      transport.close();
      if (this.transport === transport) {
        this.transport = null;
      }
      throw superseded();
    }
    transport.send(JSON.stringify(buildHelloFrame(this.lastSeq)));
    this.setStatus('open');
    return this.session;
  }

  private attach(transport: Transport): void {
    transport.onOpen(() => {
      if (this.status === 'reconnecting') {
        transport.send(JSON.stringify(buildHelloFrame(this.lastSeq)));
        this.setStatus('open');
      }
    });
    transport.onClose((code) => {
      if (this.manuallyClosed) {
        this.setStatus('closed');
        return;
      }
      const detail = clientRefusal('E_WS_DISCONNECTED', {
        message: `WebSocket closed${code === undefined ? '' : ` (code ${code})`}.`,
      });
      this.failPending(detail);
      this.setStatus('reconnecting', detail);
      this.scheduleSessionRefresh();
    });
    transport.onError((message) => {
      const detail = clientRefusal('E_WS_DISCONNECTED', {
        message: message && message.length > 0 ? message : 'WebSocket transport error.',
      });
      this.setStatus('reconnecting', detail);
      this.scheduleSessionRefresh();
    });
    transport.onMessage((data) => {
      if (typeof data !== 'string') {
        this.reportClientError(
          clientRefusal('E_SCHEMA_INVALID', {
            message: 'The server sent a non-text WebSocket frame.',
          }),
        );
        return;
      }
      const parsed = parseEventFrame(data);
      if (!parsed.ok) {
        this.reportClientError(
          clientRefusal('E_SCHEMA_INVALID', { message: `${parsed.code}: ${parsed.detail}` }),
        );
        return;
      }
      const { envelope, event } = parsed.value;
      if (typeof envelope.seq === 'number' && envelope.seq > this.lastSeq) {
        this.lastSeq = envelope.seq;
      }
      this.routePending(envelope, event);
      for (const listener of this.eventListeners) {
        listener(event, envelope);
      }
    });
  }

  private scheduleSessionRefresh(): void {
    if (this.manuallyClosed || this.recoveryTimer !== null) {
      return;
    }
    this.recoveryTimer = setTimeout(() => {
      this.recoveryTimer = null;
      void this.refreshSessionAndOpen();
    }, 1500);
  }

  /**
   * A server restart mints a new session token. Reconnect by re-reading
   * `/api/session`, not by endlessly re-helloing the dead token URL.
   */
  private async refreshSessionAndOpen(): Promise<void> {
    if (this.manuallyClosed || this.recoveryInFlight) {
      return;
    }
    this.recoveryInFlight = true;
    try {
      this.transport?.close();
      this.transport = null;
      this.session = null;
      this.manuallyClosed = false;
      await this.open();
      if (this.recoveryTimer !== null) {
        clearTimeout(this.recoveryTimer);
        this.recoveryTimer = null;
      }
    } catch {
      this.setStatus(
        'reconnecting',
        clientRefusal('E_WS_DISCONNECTED', {
          message: 'Trying the local HiveMind session again.',
        }),
      );
      this.scheduleSessionRefresh();
    } finally {
      this.recoveryInFlight = false;
    }
  }

  private routePending(envelope: Envelope<unknown>, event: DecodedEvent): void {
    if (event.name === 'ACK') {
      const pending = this.pending.get(event.payload.correlation_id);
      if (!pending) {
        return;
      }
      this.pending.delete(event.payload.correlation_id);
      clearTimeout(pending.timer);
      if (pending.command !== event.payload.command) {
        pending.reject(
          new RefusalError(
            clientRefusal('E_SCHEMA_INVALID', {
              message: `ACK named ${event.payload.command} for a ${pending.command} command.`,
            }),
          ),
        );
        return;
      }
      pending.resolve();
      return;
    }
    if (event.name === 'ERROR') {
      const pending = this.pending.get(envelope.correlation_id);
      const refusal = event.payload;
      this.failOne(envelope.correlation_id, pending, refusal);
      return;
    }
  }

  private failOne(
    correlationId: string,
    pending: PendingCommand | undefined,
    refusal: Refusal,
  ): void {
    if (!pending) {
      return;
    }
    this.pending.delete(correlationId);
    clearTimeout(pending.timer);
    pending.reject(new RefusalError(refusal));
  }

  private failPending(refusal: Refusal): void {
    for (const [correlationId, pending] of this.pending) {
      clearTimeout(pending.timer);
      pending.reject(new RefusalError(refusal));
      this.pending.delete(correlationId);
    }
  }

  private waitForOpen(transport: Transport): Promise<void> {
    return new Promise((resolve, reject) => {
      let settled = false;
      const timer = setTimeout(() => {
        if (!settled) {
          settled = true;
          reject(
            new RefusalError(
              clientRefusal('E_WS_DISCONNECTED', {
                message: 'Timed out waiting for the WebSocket to open.',
              }),
            ),
          );
        }
      }, this.handshakeTimeoutMs);
      transport.onOpen(() => {
        if (!settled) {
          settled = true;
          clearTimeout(timer);
          resolve();
        }
      });
      transport.onClose(() => {
        if (!settled) {
          settled = true;
          clearTimeout(timer);
          reject(new RefusalError(clientRefusal('E_WS_DISCONNECTED')));
        }
      });
    });
  }

  async send<N extends ServerCommandName>(
    name: N,
    payload: CommandPayloads[N],
    options?: { readonly correlationId?: string; readonly timeoutMs?: number },
  ): Promise<void> {
    if (!this.transport || !this.session || this.status !== 'open') {
      throw new RefusalError(clientRefusal('E_WS_DISCONNECTED'));
    }
    if (name === 'FILE_READ') {
      const filePayload = payload as CommandPayloads['FILE_READ'];
      const pathRefusal = validateWorkspacePath(filePayload.path);
      if (pathRefusal) {
        throw new RefusalError(clientRefusal(pathRefusal));
      }
    }
    const envelope = buildCommandEnvelope(name, payload, {
      newId: this.newId,
      now: this.now,
    });
    const correlationId = options?.correlationId ?? envelope.correlation_id;
    const framed: Envelope<CommandPayloads[N]> = { ...envelope, correlation_id: correlationId };
    return new Promise<void>((resolve, reject) => {
      const timer = setTimeout(() => {
        this.pending.delete(correlationId);
        reject(
          new RefusalError(
            clientRefusal('E_WS_DISCONNECTED', {
              message: `No ACK for ${name} within ${options?.timeoutMs ?? this.requestTimeoutMs} ms.`,
            }),
          ),
        );
      }, options?.timeoutMs ?? this.requestTimeoutMs);
      this.pending.set(correlationId, {
        command: name,
        resolve,
        reject,
        timer,
      });
      try {
        this.transport?.send(JSON.stringify(framed));
      } catch {
        clearTimeout(timer);
        this.pending.delete(correlationId);
        reject(new RefusalError(clientRefusal('E_WS_DISCONNECTED')));
      }
    });
  }

  subscribe(listener: (event: DecodedEvent, envelope: Envelope<unknown>) => void): () => void {
    this.eventListeners.add(listener);
    return () => this.eventListeners.delete(listener);
  }

  subscribeStatus(listener: (status: ConnectionStatus, detail?: Refusal) => void): () => void {
    this.statusListeners.add(listener);
    return () => this.statusListeners.delete(listener);
  }

  subscribeClientError(listener: (refusal: Refusal) => void): () => void {
    this.clientErrorListeners.add(listener);
    return () => this.clientErrorListeners.delete(listener);
  }

  private reportClientError(refusal: Refusal): void {
    for (const listener of this.clientErrorListeners) {
      listener(refusal);
    }
  }

  /**
   * Client-initiated pull for pending approvals.
   *
   * The frozen vocabulary has no `APPROVAL_HYDRATE` command (adding one is not
   * additive — see `hiverouter-contract/src/command.rs`), but the transport
   * already carries a real pull: the `{"type":"hello","last_seq":n}` control
   * frame. The server answers every hello with `SERVER_HELLO` and then
   * re-publishes its bounded batch of open approvals as `APPROVAL_REQUEST`
   * events, so the response side needs no new event and replay is idempotent
   * (the store replaces by `approval_id`).
   *
   * `last_seq` is the sequence this client has applied, exactly like a
   * reconnect: when it is current the server replays no stream events at all
   * (`EventBus::replay_after` -> `Replay::Current`), and when events were
   * missed it replays precisely that range. Nothing is fabricated and no
   * approval is re-created.
   *
   * The frames arrive asynchronously through {@link subscribe}; this resolves
   * once the control frame is handed to the transport. Rejects with
   * `E_WS_DISCONNECTED` when the socket is not open.
   */
  async pullPendingApprovals(): Promise<void> {
    if (!this.transport || !this.session || this.status !== 'open') {
      throw new RefusalError(clientRefusal('E_WS_DISCONNECTED'));
    }
    try {
      this.transport.send(JSON.stringify(buildHelloFrame(this.lastSeq)));
    } catch {
      const refusal = clientRefusal('E_WS_DISCONNECTED', {
        message: 'The WebSocket refused the hello pull frame.',
      });
      this.setStatus('error', refusal);
      throw new RefusalError(refusal);
    }
  }

  close(): void {
    this.manuallyClosed = true;
    this.closeEpoch += 1;
    this.opening = null;
    if (this.recoveryTimer !== null) {
      clearTimeout(this.recoveryTimer);
      this.recoveryTimer = null;
    }
    this.failPending(clientRefusal('E_WS_DISCONNECTED'));
    this.transport?.close();
    this.transport = null;
    this.setStatus('closed');
  }
}
