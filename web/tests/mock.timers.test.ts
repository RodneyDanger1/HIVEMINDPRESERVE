import { afterEach, describe, expect, it, vi } from 'vitest';

import type { DecodedEvent } from '../src/api/contract';

/**
 * Dev-mock lifecycle (§7 test double): scenario timers are tracked and cleared
 * on close(), and a closed double cannot deliver a frame into a disposed store.
 * These tests construct the dev double explicitly (DEV + VITE_UI_MOCK=1); they
 * never run in the production path and never touch a real socket.
 */

const ORIGINAL_MOCK_FLAG = import.meta.env.VITE_UI_MOCK;

afterEach(() => {
  import.meta.env.VITE_UI_MOCK = ORIGINAL_MOCK_FLAG;
  vi.useRealTimers();
});

async function mockClient(): Promise<{
  client: InstanceType<typeof import('../src/api/mock').MockApiClient>;
  events: string[];
}> {
  import.meta.env.VITE_UI_MOCK = '1';
  const { MockApiClient } = await import('../src/api/mock');
  const client = new MockApiClient();
  await client.open();
  const events: string[] = [];
  client.subscribe((event: DecodedEvent) => {
    events.push(event.name);
  });
  return { client, events };
}

describe('dev mock timer lifecycle (WP-UI-1 asset repair)', () => {
  it('clears pending scenario timers on close and emits nothing afterwards', async () => {
    vi.useFakeTimers();
    const { client, events } = await mockClient();
    await client.send('SUBMIT_PROMPT', { text: 'hello', project_id: 'project-1' });
    expect(events).toContain('ACK');
    expect(vi.getTimerCount()).toBeGreaterThan(0);

    client.close();
    expect(vi.getTimerCount()).toBe(0);
    const before = events.length;
    vi.advanceTimersByTime(5_000);
    expect(events.length).toBe(before);
    expect(events.filter((name) => name === 'REPLY_DELTA' || name === 'REPLY_DONE')).toEqual([]);
  });

  it('never emits after a direct close, and close is idempotent', async () => {
    import.meta.env.VITE_UI_MOCK = '1';
    const { MockApiClient } = await import('../src/api/mock');
    vi.useFakeTimers();
    const client = new MockApiClient();
    await client.open();
    const events: string[] = [];
    client.subscribe((event: DecodedEvent) => {
      events.push(event.name);
    });
    await client.send('SUBMIT_PROMPT', { text: '!approval', project_id: 'project-1' });
    // The approval scenario schedules its own timer in addition to the deltas.
    expect(vi.getTimerCount()).toBeGreaterThan(1);

    client.close();
    client.close();
    const before = events.length;
    vi.advanceTimersByTime(10_000);
    expect(events.length).toBe(before);
    expect(events).not.toContain('APPROVAL_REQUEST');
  });

  it('keeps the explicit dev-only constructor guard', async () => {
    import.meta.env.VITE_UI_MOCK = '0';
    const { MockApiClient } = await import('../src/api/mock');
    expect(() => new MockApiClient()).toThrow(/dev-only/);
  });
});
