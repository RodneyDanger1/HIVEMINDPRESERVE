/**
 * Persisted operator preferences.
 *
 * Preferences are UI state the operator set on purpose, so they survive a reload
 * and a relaunch (the launcher reuses the same per-user browser profile). Reads
 * never fabricate a value: an unreadable store returns the documented default,
 * and a refused write leaves the in-memory value authoritative for this session.
 */

/** Minimal storage surface so the rules are testable without a DOM. */
export interface StorageLike {
  getItem(key: string): string | null;
  setItem(key: string, value: string): void;
}

export const REDUCED_MOTION_KEY = 'hivemind.reduced-motion.v1';

export function readStoredBoolean(storage: StorageLike | null, key: string, fallback: boolean): boolean {
  if (storage === null) {
    return fallback;
  }
  try {
    const raw = storage.getItem(key);
    if (raw === null) {
      return fallback;
    }
    return raw !== 'false';
  } catch {
    return fallback;
  }
}

export function writeStoredBoolean(storage: StorageLike | null, key: string, value: boolean): void {
  if (storage === null) {
    return;
  }
  try {
    storage.setItem(key, value ? 'true' : 'false');
  } catch {
    // A privacy shell may reject storage; the in-memory control still works for
    // this session and the next launch falls back to the OS preference.
  }
}

function localStore(): StorageLike | null {
  try {
    return typeof window === 'undefined' ? null : window.localStorage;
  } catch {
    return null;
  }
}

/** The in-app reduced-motion override; defaults to off so the OS setting applies. */
export function readReducedMotionOverride(): boolean {
  return readStoredBoolean(localStore(), REDUCED_MOTION_KEY, false);
}

export function writeReducedMotionOverride(value: boolean): void {
  writeStoredBoolean(localStore(), REDUCED_MOTION_KEY, value);
}
