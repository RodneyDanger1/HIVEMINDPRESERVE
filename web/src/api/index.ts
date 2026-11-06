/**
 * UI API factory. One interface, two implementations:
 *  - real: WsApiClient against the localhost Rust server (always the default);
 *  - dev mock: MockApiClient only when `import.meta.env.DEV && VITE_UI_MOCK === '1'`.
 * There is no production branch that fabricates success.
 */
import { WsApiClient, type UiApi } from './client';
import { MockApiClient } from './mock';

export type { ConnectionStatus, SessionInfo, Transport, TransportFactory, UiApi } from './client';
export { WsApiClient, PartyTransport } from './client';
export { MockApiClient } from './mock';

export function createUiApi(): UiApi {
  if (import.meta.env.DEV && import.meta.env.VITE_UI_MOCK === '1') {
    return new MockApiClient();
  }
  return new WsApiClient();
}
