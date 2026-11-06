/**
 * Client-side refusal construction. Only codes from the frozen contract
 * (UI-COMMANDS-v1 §5) may be used; copy comes from the string table. A server
 * ERROR frame is always preferred over local copy — this module exists for
 * transport-level failures that produce no server frame.
 */
import type { Refusal } from './contract';
import { STRINGS } from '../state/strings';

export class RefusalError extends Error {
  readonly refusal: Refusal;

  constructor(refusal: Refusal) {
    super(refusal.message);
    this.name = 'RefusalError';
    this.refusal = refusal;
  }
}

const LOCAL_TABLE: Record<
  string,
  { message: string; howToFix: string; retryable: boolean }
> = STRINGS.clientRefusals;

export function clientRefusal(
  code: string,
  overrides?: Partial<Omit<Refusal, 'code'>>,
): Refusal {
  const known = LOCAL_TABLE[code];
  if (known) {
    return {
      code,
      message: overrides?.message ?? known.message,
      how_to_fix: overrides?.how_to_fix ?? known.howToFix,
      retryable: overrides?.retryable ?? known.retryable,
    };
  }
  // Server-only refusal codes have no local copy; never invent a cause.
  return {
    code,
    message: overrides?.message ?? `${STRINGS.clientRefusalUnknown.message} (${code})`,
    how_to_fix: overrides?.how_to_fix ?? STRINGS.clientRefusalUnknown.howToFix,
    retryable: overrides?.retryable ?? false,
  };
}

export function refusalFromUnknown(value: unknown, fallbackCode: string): Refusal {
  if (value instanceof RefusalError) {
    return value.refusal;
  }
  return clientRefusal(fallbackCode);
}
