/**
 * React context for the single UI store (create once in App, provide down).
 */
import { createContext, useContext } from 'react';
import { useStore } from 'zustand';
import type { StoreApi } from 'zustand/vanilla';
import type { UiState } from './store';

export const UiStoreContext = createContext<StoreApi<UiState> | null>(null);

export function useUiStore<T>(selector: (state: UiState) => T): T {
  const store = useContext(UiStoreContext);
  if (!store) {
    throw new Error('useUiStore must be used inside <UiStoreContext.Provider>');
  }
  return useStore(store, selector);
}

export function useUiStoreApi(): StoreApi<UiState> {
  const store = useContext(UiStoreContext);
  if (!store) {
    throw new Error('useUiStoreApi must be used inside <UiStoreContext.Provider>');
  }
  return store;
}
