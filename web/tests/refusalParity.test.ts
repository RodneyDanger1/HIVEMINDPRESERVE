import { describe, expect, it } from 'vitest';
import refusalSource from '../../backend/crates/hiverouter-contract/src/refusal.rs?raw';
import { KNOWN_REFUSAL_CODES } from '../src/api/contract';
import { STRINGS } from '../src/state/strings';

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

  it('pins the frozen 78-code contract', () => {
    expect(new Set(KNOWN_REFUSAL_CODES).size).toBe(78);
    expect(KNOWN_REFUSAL_CODES.length).toBe(78);
  });

  it('covers every client refusal with named copy and a fix', () => {
    const known = new Set<string>(KNOWN_REFUSAL_CODES);
    for (const [code, copy] of Object.entries(STRINGS.clientRefusals)) {
      expect(known.has(code), code).toBe(true);
      expect(copy.message.length, code).toBeGreaterThan(0);
      expect(copy.howToFix.length, code).toBeGreaterThan(0);
    }
    for (const code of [
      'E_CALL_LIMIT',
      'E_LLM_RATE_LIMIT',
      'E_PROVIDER_INVALID_RESPONSE',
      'E_USAGE_INCOMPLETE',
    ]) {
      expect(STRINGS.clientRefusals[code as keyof typeof STRINGS.clientRefusals], code).toBeDefined();
    }
  });
});
