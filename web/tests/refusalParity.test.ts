import { describe, expect, it } from 'vitest';
import refusalSource from '../../backend/crates/hiverouter-contract/src/refusal.rs?raw';
import { KNOWN_REFUSAL_CODES } from '../src/api/contract';

function backendCodes(): string[] {
  const codes = new Set<string>();
  for (const match of refusalSource.matchAll(/"(E_[A-Z0-9_]+)"/g)) {
    if (match[1] !== 'E_NOT_A_REAL_CODE') {
      codes.add(match[1]);
    }
  }
  return [...codes].sort();
}

describe('refusal-code parity', () => {
  it('lists every real backend refusal code exactly once', () => {
    const frontend = [...new Set(KNOWN_REFUSAL_CODES)].sort();
    expect(frontend).toEqual(backendCodes());
  });

  it('pins the frozen 76-code contract', () => {
    expect(new Set(KNOWN_REFUSAL_CODES).size).toBe(76);
    expect(KNOWN_REFUSAL_CODES.length).toBe(76);
  });
});
