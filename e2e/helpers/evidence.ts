import { mkdirSync, writeFileSync } from 'node:fs';
import { fileURLToPath } from 'node:url';
import type { Page, TestInfo } from '@playwright/test';

export const LIVE_DIR = new URL('../../docs/trace/live/', import.meta.url);
export const LIVE_APP_DIR = new URL('../../docs/trace/live/live-app/', import.meta.url);

export function ensureLiveDirs(): void {
  mkdirSync(fileURLToPath(LIVE_DIR), { recursive: true });
  mkdirSync(fileURLToPath(LIVE_APP_DIR), { recursive: true });
}

export function writeEvidence(name: string, data: unknown): void {
  ensureLiveDirs();
  writeFileSync(new URL(name, LIVE_APP_DIR), `${JSON.stringify(data, null, 2)}\n`, 'utf8');
}

export async function shot(page: Page, name: string, testInfo?: TestInfo): Promise<string> {
  ensureLiveDirs();
  const safe = name.replace(/[^a-z0-9_-]+/gi, '-').toLowerCase();
  const file = new URL(`${safe}.png`, LIVE_APP_DIR);
  await page.screenshot({ path: fileURLToPath(file), fullPage: false });
  if (testInfo) {
    testInfo.attachments.push({ name: `${safe}.png`, path: fileURLToPath(file), contentType: 'image/png' });
  }
  return `docs/trace/live/live-app/${safe}.png`;
}

export interface FrameRecord {
  dir: 'sent' | 'received';
  at: number;
  payload: string;
}

/** Captures all WS frames from now on (Playwright page-level tap). */
export function captureFrames(page: Page): { frames: FrameRecord[] } {
  const frames: FrameRecord[] = [];
  page.on('websocket', (ws) => {
    ws.on('framesent', (f) => frames.push({ dir: 'sent', at: Date.now(), payload: String(f.payload) }));
    ws.on('framereceived', (f) => frames.push({ dir: 'received', at: Date.now(), payload: String(f.payload) }));
  });
  return { frames };
}

export function parseFrames(frames: FrameRecord[]): Array<Record<string, unknown> & { dir: string; at: number }> {
  return frames.map((frame) => {
    try {
      const json = JSON.parse(frame.payload) as Record<string, unknown>;
      return { dir: frame.dir, at: frame.at, ...json };
    } catch {
      return { dir: frame.dir, at: frame.at, raw: frame.payload };
    }
  });
}

export function waitForFrame(
  frames: FrameRecord[],
  predicate: (json: Record<string, unknown>) => boolean,
  timeoutMs = 6000,
): Promise<Record<string, unknown>> {
  return new Promise((resolve, reject) => {
    const started = Date.now();
    const timer = setInterval(() => {
      for (const frame of frames) {
        try {
          const json = JSON.parse(frame.payload) as Record<string, unknown>;
          if (predicate(json)) {
            clearInterval(timer);
            resolve(json);
            return;
          }
        } catch {
          // non-JSON frame; ignored for contract assertions (recorded in evidence)
        }
      }
      if (Date.now() - started > timeoutMs) {
        clearInterval(timer);
        reject(new Error('timed out waiting for expected WS frame'));
      }
    }, 50);
  });
}
