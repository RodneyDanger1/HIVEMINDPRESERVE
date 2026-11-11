import { describe, expect, it } from 'vitest';
import {
  REDUCED_MOTION_KEY,
  readStoredBoolean,
  writeStoredBoolean,
  type StorageLike,
} from '../src/state/preferences';

function memoryStorage(initial: Record<string, string> = {}): StorageLike & { dump: () => Record<string, string> } {
  const data: Record<string, string> = { ...initial };
  return {
    getItem: (key) => (key in data ? data[key] : null),
    setItem: (key, value) => {
      data[key] = value;
    },
    dump: () => ({ ...data }),
  };
}

describe('persisted operator preferences', () => {
  it('reads a stored value, the default for a missing key, and the default when storage is absent', () => {
    expect(readStoredBoolean(memoryStorage({ k: 'true' }), 'k', false)).toBe(true);
    expect(readStoredBoolean(memoryStorage({ k: 'false' }), 'k', true)).toBe(false);
    expect(readStoredBoolean(memoryStorage(), 'k', true)).toBe(true);
    expect(readStoredBoolean(null, 'k', false)).toBe(false);
  });

  it('round-trips a written value', () => {
    const storage = memoryStorage();
    writeStoredBoolean(storage, 'k', true);
    expect(readStoredBoolean(storage, 'k', false)).toBe(true);
    writeStoredBoolean(storage, 'k', false);
    expect(readStoredBoolean(storage, 'k', true)).toBe(false);
    expect(storage.dump()).toEqual({ k: 'false' });
  });

  it('treats an unreadable store as "use the default" instead of throwing', () => {
    const throwing: StorageLike = {
      getItem: () => {
        throw new Error('SecurityError');
      },
      setItem: () => {
        throw new Error('QuotaExceededError');
      },
    };
    expect(readStoredBoolean(throwing, 'k', true)).toBe(true);
    expect(() => writeStoredBoolean(throwing, 'k', true)).not.toThrow();
  });

  it('pins the reduced-motion storage key so a rename cannot silently reset it', () => {
    expect(REDUCED_MOTION_KEY).toBe('hivemind.reduced-motion.v1');
  });
});
