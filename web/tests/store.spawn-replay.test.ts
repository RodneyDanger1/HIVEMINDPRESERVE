/**
 * Replay-safety pins for NODE_SPAWN (audit item 6).
 *
 * The bounded server replay window can re-deliver a task's creation event after
 * a reconnect. A spawn is a creation fact, never a state transition: it must
 * not reset a completed/killed node, must not duplicate the task card, and a
 * genuine NODE_STATE transition must still apply.
 */
import { describe, expect, it } from 'vitest';
import { createUiStore } from '../src/state/store';
import { FakeUiApi, emptyRunsResponse, until } from './helpers/fakeUiApi';

async function connectedStore(): Promise<{
  api: FakeUiApi;
  store: ReturnType<typeof createUiStore>;
}> {
  const api = new FakeUiApi();
  const store = createUiStore(api, {
    fetchImpl: (async () => emptyRunsResponse()) as unknown as typeof fetch,
  });
  const init = store.getState().init();
  await until(() => api.openCalls.length === 1);
  api.resolveOpen(0);
  await init;
  return { api, store };
}

describe('NODE_SPAWN replay guard', () => {
  it('a replayed spawn keeps the completed state and one task card', async () => {
    const { api, store } = await connectedStore();

    api.emit({
      name: 'NODE_SPAWN',
      payload: { task_id: 't-1', state: 'completed', rev: 2, kind: 'worker', label: 'Agent t-1' },
    });
    expect(store.getState().tasks['t-1']?.state).toBe('completed');

    // Replay window re-delivers the original creation event (older rev).
    api.emit({
      name: 'NODE_SPAWN',
      payload: { task_id: 't-1', state: 'queued', rev: 1, kind: 'worker', label: 'Agent t-1' },
    });
    expect(store.getState().tasks['t-1']?.state).toBe('completed');
    expect(store.getState().tasks['t-1']?.rev).toBe(2);
    expect(Object.keys(store.getState().tasks)).toEqual(['t-1']);
  });

  it('a spawn can never reset a state even when it claims a newer revision', async () => {
    const { api, store } = await connectedStore();

    api.emit({
      name: 'NODE_SPAWN',
      payload: { task_id: 't-2', state: 'completed', rev: 2 },
    });
    api.emit({
      name: 'NODE_SPAWN',
      payload: { task_id: 't-2', state: 'queued', rev: 5 },
    });

    // Metadata/rev may advance, but the creation event still cannot rewrite state.
    expect(store.getState().tasks['t-2']?.state).toBe('completed');
    expect(store.getState().tasks['t-2']?.rev).toBe(5);
    expect(Object.keys(store.getState().tasks)).toEqual(['t-2']);
  });

  it('keeps applying genuine NODE_STATE transitions after a spawn', async () => {
    const { api, store } = await connectedStore();

    api.emit({ name: 'NODE_SPAWN', payload: { task_id: 't-3', state: 'queued', rev: 1 } });
    expect(store.getState().tasks['t-3']?.state).toBe('queued');

    api.emit({ name: 'NODE_STATE', payload: { task_id: 't-3', state: 'running', rev: 2 } });
    expect(store.getState().tasks['t-3']?.state).toBe('running');

    api.emit({ name: 'NODE_STATE', payload: { task_id: 't-3', state: 'completed', rev: 3 } });
    expect(store.getState().tasks['t-3']?.state).toBe('completed');

    graphIdentity(store);
  });
});

/** The task map is the card identity: no duplicate ids can exist. */
function graphIdentity(store: ReturnType<typeof createUiStore>): void {
  const ids = Object.keys(store.getState().tasks);
  expect(new Set(ids).size).toBe(ids.length);
  expect(ids).toEqual(['t-3']);
}
