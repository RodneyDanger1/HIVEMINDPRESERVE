/**
 * Named HTTP failure copy for the read-only panel routes.
 *
 * The frozen contract requires every failure to surface code + message +
 * how_to_fix, never a bare "HTTP 500". A server refusal body is authoritative;
 * when the route returns a non-JSON/bodyless error the failure is named as the
 * transport refusal E_WS_DISCONNECTED with the exact route + status, so the
 * operator always has an actionable fix.
 */
import type { Refusal } from './contract';
import { clientRefusal, RefusalError } from './refusals';

function asRecord(value: unknown): Record<string, unknown> | null {
  return typeof value === 'object' && value !== null && !Array.isArray(value)
    ? (value as Record<string, unknown>)
    : null;
}

/** Builds a named failure from an already-parsed error body. */
export function refusalFromBody(body: unknown, status: number, label: string): RefusalError {
  const record = asRecord(body);
  if (record !== null && typeof record.code === 'string') {
    const base = clientRefusal(record.code);
    return new RefusalError({
      code: record.code,
      message:
        typeof record.message === 'string'
          ? record.message
          : `${label} refused with ${record.code}.`,
      how_to_fix:
        typeof record.how_to_fix === 'string' ? record.how_to_fix : base.how_to_fix,
      retryable: typeof record.retryable === 'boolean' ? record.retryable : base.retryable,
    });
  }
  return new RefusalError(
    clientRefusal('E_WS_DISCONNECTED', {
      message: `${label} returned HTTP ${status} without a refusal body.`,
      retryable: true,
    }),
  );
}

/** Converts a non-2xx Response into a named RefusalError (code/message/fix). */
export async function httpFailure(response: Response, label: string): Promise<RefusalError> {
  let body: unknown = null;
  try {
    body = await response.json();
  } catch {
    // A non-JSON error body is still an error; the route + status below names it.
    body = null;
  }
  return refusalFromBody(body, response.status, label);
}

/** One line for an inline error surface: code, message, and the fix. */
export function failureText(error: unknown, fallback: string): string {
  if (error instanceof RefusalError) {
    const refusal = error.refusal;
    return `${refusal.code}: ${refusal.message} — ${refusal.how_to_fix}`;
  }
  if (error instanceof Error && error.message.length > 0) {
    return error.message;
  }
  return fallback;
}

/** Structured form when a surface renders the three fields separately. */
export function failureOf(error: unknown, fallback: string): Refusal {
  if (error instanceof RefusalError) {
    return error.refusal;
  }
  return clientRefusal('E_WS_DISCONNECTED', {
    message: error instanceof Error && error.message.length > 0 ? error.message : fallback,
    retryable: true,
  });
}
