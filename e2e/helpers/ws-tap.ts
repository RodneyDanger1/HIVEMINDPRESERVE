import type { Page, WebSocket } from '@playwright/test';

export interface WsFrame {
  direction: 'sent' | 'received';
  payload: string;
  at: string;
}

export interface WsTap {
  frames: WsFrame[];
  sockets: WebSocket[];
  waitFor(predicate: (frame: WsFrame) => boolean, timeoutMs?: number): Promise<WsFrame>;
  close(): Promise<void>;
}

/**
 * Captures every WebSocket opened by the page (Playwright's websocket events).
 * Used to prove ACK/refusal/no-fake-success: a mutating control must produce an
 * ACK or typed ERROR *before* UI state mutates.
 */
export function tapWebSockets(page: Page): WsTap {
  const frames: WsFrame[] = [];
  const sockets: WebSocket[] = [];

  page.on('websocket', (socket) => {
    sockets.push(socket);
    const record = (direction: 'sent' | 'received') => (event: { payload: string | Buffer }) => {
      frames.push({
        direction,
        payload: typeof event.payload === 'string' ? event.payload : event.payload.toString('utf8'),
        at: new Date().toISOString(),
      });
    };
    socket.on('framesent', record('sent'));
    socket.on('framereceived', record('received'));
  });

  return {
    frames,
    sockets,
    async waitFor(predicate, timeoutMs = 5_000) {
      const deadline = Date.now() + timeoutMs;
      while (Date.now() < deadline) {
        const hit = frames.find(predicate);
        if (hit) return hit;
        await new Promise((resolve) => setTimeout(resolve, 50));
      }
      throw new Error('WSTap timeout: expected frame not observed (no fabricated pass)');
    },
    async close() {
      for (const socket of sockets) {
        try {
          await socket.close();
        } catch {
          // The app already closed it; ignore — this is teardown, not evidence.
        }
      }
    },
  };
}
