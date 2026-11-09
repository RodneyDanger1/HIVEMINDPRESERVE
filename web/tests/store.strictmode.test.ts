/**
 * React StrictMode lifecycle pins for the UI store (audit item 1).
 *
 * StrictMode mounts, cleans up, then mounts again. The store survives, so a
 * stale boot generation must never write into the new generation's state,
 * must never consume the new generation's snapshot slot, and must never close
 * the shared API after the new generation has opened it.
 */
import { describe, expect, it } from 'vitest';
import { createUiStore } from '../src/state/store';
import { FakeUiApi, emptyRunsResponse, until } from './helpers/fakeUiApi';

const immediateFetch = (async () => emptyRunsResponse()) as unknown as typeof fetch;

describe('StrictMode dispose / re-init', () => {
  it('resets initialized and reopens the socket on a second init', async () => {
    const api = new FakeUiApi();
    const store = createUiStore(api, { fetchImpl: immediateFetch });

    const first = store.getState().init();
    await until(() => api.openCalls.length === 1);
    api.resolveOpen(0);
    await first;
    expect(store.getState().initialized).toBe(true);
    expect(store.getState().connection).toBe('open');

    store.getState().dispose();
    expect(store.getState().initialized).toBe(false);
    expect(api.closes).toBe(1);

    const second = store.getState().init();
    await until(() => api.openCalls.length === 2);
    api.resolveOpen(1);
    await second;
    expect(store.getState().initialized).toBe(true);
    expect(store.getState().connection).toBe('open');
    expect(api.opens).toBe(2);
  });

  it('a stale open resolution cannot close the new generation socket', async () => {
    const api = new FakeUiApi();
    const store = createUiStore(api, { fetchImpl: immediateFetch });

    const first = store.getState().init();
    await until(() => api.openCalls.length === 1);
    store.getState().dispose();
    const second = store.getState().init();
    await until(() => api.openCalls.length === 2);

    // New generation opens first; the old (disposed) generation resolves later.
    api.resolveOpen(1);
    await second;
    expect(store.getState().connection).toBe('open');
    api.resolveOpen(0);
    await first;

    // No second close: the shared API belongs to the live generation now.
    expect(api.closes).toBe(1);
    expect(store.getState().connection).toBe('open');
    expect(store.getState().initialized).toBe(true);
  });

  it('an aborted stale boot read cannot fail or block the new snapshot', async () => {
    const api = new FakeUiApi();
    let calls = 0;
    const fetchImpl = (async () => {
      calls += 1;
      if (calls === 1) {
        // Never settles on its own; dispose() aborts the bounded fetch.
        return new Promise<Response>(() => {});
      }
      return emptyRunsResponse();
    }) as unknown as typeof fetch;
    const store = createUiStore(api, { fetchImpl });

    const first = store.getState().init();
    await until(() => calls === 1);
    store.getState().dispose();
    const second = store.getState().init();
    await until(() => calls === 2 && api.openCalls.length === 1);
    api.resolveOpen(0);
    await second;
    await first;

    expect(store.getState().snapshot.status).toBe('ready');
    expect(store.getState().snapshot.refusal).toBeNull();
    expect(store.getState().refusals).toEqual([]);
    expect(store.getState().connection).toBe('open');
  });
});
