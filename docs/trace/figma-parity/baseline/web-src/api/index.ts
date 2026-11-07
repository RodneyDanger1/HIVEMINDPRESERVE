/**
 * UI API factory. One interface, two implementations:
 *  - real: WsApiClient against the localhost Rust server (always the default);
 *  - dev mock: MockApiClient only when `import.meta.env.DEV && VITE_UI_MOCK === '1'`.
 *
 * The dev mock is loaded through a DYNAMIC import inside the dev-only branch, so
 * the production entry never statically imports `./mock` and a production bundle
 * cannot ship the test double. `tests/apiBoundary.test.ts` enforces the module
 * boundary; `src/api/mock.ts` also refuses construction outside dev.
 *
 * The factory stays synchronous for the app shell (`createUiStore(createUiApi())`
 * reads `kind` and subscribes before `open()`), so a tiny lazy wrapper queues
 * listeners and resolves the real implementation on first use.
 */
import {
  WsApiClient,
  supportsPendingApprovalPull,
  type ConnectionStatus,
  type SessionInfo,
  type UiApi,
} from './client';
import { RefusalError, clientRefusal } from './refusals';
import type {
  CommandPayloads,
  DecodedEvent,
  Envelope,
  Refusal,
  ServerCommandName,
} from './contract';

/** Compile-time dev-only flag; in a production build this folds to `false`. */
const USE_DEV_MOCK = import.meta.env.DEV && import.meta.env.VITE_UI_MOCK === '1';

export type { ConnectionStatus, SessionInfo, Transport, TransportFactory, UiApi } from './client';
export { WsApiClient, PartyTransport } from './client';
export { DEFAULT_HTTP_TIMEOUT_MS } from './http';

type EventListener = (event: DecodedEvent, envelope: Envelope<unknown>) => void;
type StatusListener = (status: ConnectionStatus, detail?: Refusal) => void;
type ClientErrorListener = (refusal: Refusal) => void;

class DevAwareApi implements UiApi {
  private delegate: UiApi | null = null;
  private opening: Promise<UiApi> | null = null;
  private closed = false;
  private readonly eventListeners = new Set<EventListener>();
  private readonly statusListeners = new Set<StatusListener>();
  private readonly clientErrorListeners = new Set<ClientErrorListener>();
  private readonly eventUnsubs = new Map<EventListener, () => void>();
  private readonly statusUnsubs = new Map<StatusListener, () => void>();
  private readonly clientErrorUnsubs = new Map<ClientErrorListener, () => void>();

  get kind(): 'ws' | 'mock' {
    // The dev-mock decision is compile-time; before the delegate exists the
    // store's apiKind must already be correct for the mock banner.
    if (USE_DEV_MOCK) {
      return 'mock';
    }
    return this.delegate?.kind ?? 'ws';
  }

  private ensure(): Promise<UiApi> {
    if (this.delegate) {
      return Promise.resolve(this.delegate);
    }
    if (!this.opening) {
      this.opening = (USE_DEV_MOCK
        ? import('./mock').then((module) => new module.MockApiClient() as UiApi)
        : Promise.resolve(new WsApiClient() as UiApi)
      ).then((api) => {
        this.delegate = api;
        this.flushSubscriptions(api);
        if (this.closed) {
          api.close();
        }
        return api;
      });
    }
    return this.opening;
  }

  private flushSubscriptions(api: UiApi): void {
    for (const listener of this.eventListeners) {
      if (!this.eventUnsubs.has(listener)) {
        this.eventUnsubs.set(listener, api.subscribe(listener));
      }
    }
    for (const listener of this.statusListeners) {
      if (!this.statusUnsubs.has(listener)) {
        this.statusUnsubs.set(listener, api.subscribeStatus(listener));
      }
    }
    for (const listener of this.clientErrorListeners) {
      if (!this.clientErrorUnsubs.has(listener)) {
        this.clientErrorUnsubs.set(listener, api.subscribeClientError(listener));
      }
    }
  }

  async open(): Promise<SessionInfo> {
    const api = await this.ensure();
    return api.open();
  }

  /** Pull pending approvals over the existing hello/replay control frame. */
  async pullPendingApprovals(): Promise<void> {
    const api = await this.ensure();
    if (!supportsPendingApprovalPull(api)) {
      throw new RefusalError(
        clientRefusal('E_WS_DISCONNECTED', {
          message: 'This UI transport cannot pull pending approvals.',
        }),
      );
    }
    return api.pullPendingApprovals();
  }

  close(): void {
    this.closed = true;
    this.delegate?.close();
  }

  async send<N extends ServerCommandName>(
    name: N,
    payload: CommandPayloads[N],
    options?: { readonly correlationId?: string; readonly timeoutMs?: number },
  ): Promise<void> {
    const api = await this.ensure();
    return api.send(name, payload, options);
  }

  subscribe(listener: EventListener): () => void {
    this.eventListeners.add(listener);
    if (this.delegate) {
      this.eventUnsubs.set(listener, this.delegate.subscribe(listener));
    }
    return () => {
      this.eventListeners.delete(listener);
      const unsubscribe = this.eventUnsubs.get(listener);
      this.eventUnsubs.delete(listener);
      unsubscribe?.();
    };
  }

  subscribeStatus(listener: StatusListener): () => void {
    this.statusListeners.add(listener);
    if (this.delegate) {
      this.statusUnsubs.set(listener, this.delegate.subscribeStatus(listener));
    }
    return () => {
      this.statusListeners.delete(listener);
      const unsubscribe = this.statusUnsubs.get(listener);
      this.statusUnsubs.delete(listener);
      unsubscribe?.();
    };
  }

  subscribeClientError(listener: ClientErrorListener): () => void {
    this.clientErrorListeners.add(listener);
    if (this.delegate) {
      this.clientErrorUnsubs.set(listener, this.delegate.subscribeClientError(listener));
    }
    return () => {
      this.clientErrorListeners.delete(listener);
      const unsubscribe = this.clientErrorUnsubs.get(listener);
      this.clientErrorUnsubs.delete(listener);
      unsubscribe?.();
    };
  }
}

export function createUiApi(): UiApi {
  return new DevAwareApi();
}
