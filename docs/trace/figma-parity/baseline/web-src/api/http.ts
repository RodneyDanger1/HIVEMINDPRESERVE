/**
 * Bounded HTTP helper for the localhost API surfaces (`GET /api/session`,
 * `GET /health`). Every failure is surfaced as a named refusal from the frozen
 * contract table (`E_WS_DISCONNECTED` for transport/timeout, with an explicit
 * message) — never a bare AbortError/DOMException and never a fake success.
 *
 * The bound is enforced by Promise.race as well as AbortController, so a fetch
 * implementation that ignores the abort signal (older runtimes, test doubles)
 * still cannot hang the UI.
 */
import { clientRefusal, RefusalError } from './refusals';

/** Default upper bound for a single localhost HTTP request. */
export const DEFAULT_HTTP_TIMEOUT_MS = 8000;

export interface BoundedFetchOptions {
  /** Label used in refusal messages and logs, e.g. "GET /api/session". */
  readonly label: string;
  /** Upper bound in ms; defaults to DEFAULT_HTTP_TIMEOUT_MS. */
  readonly timeoutMs?: number;
  readonly fetchImpl?: typeof fetch;
  readonly init?: RequestInit;
  /** Caller-owned abort signal (e.g. component unmount). */
  readonly signal?: AbortSignal;
}

function transportRefusal(message: string, retryable = true): RefusalError {
  return new RefusalError(clientRefusal('E_WS_DISCONNECTED', { message, retryable }));
}

/**
 * Fetch with a hard upper bound. Resolves with the raw Response (status is left
 * to the caller, because `/health` legitimately returns non-2xx while degraded).
 */
export async function fetchWithBoundTimeout(
  url: string,
  options: BoundedFetchOptions,
): Promise<Response> {
  const timeoutMs = options.timeoutMs ?? DEFAULT_HTTP_TIMEOUT_MS;
  const fetchImpl = options.fetchImpl ?? fetch;
  const controller = new AbortController();
  const externalSignal = options.signal;
  let timedOut = false;
  let abortedByCaller = false;
  let timer: ReturnType<typeof setTimeout> | undefined;
  let externalAbortHandler: (() => void) | undefined;

  if (externalSignal?.aborted) {
    throw transportRefusal(`${options.label} request was aborted by the caller.`);
  }

  const timeout = new Promise<never>((_resolve, reject) => {
    timer = setTimeout(() => {
      timedOut = true;
      controller.abort();
      reject(
        transportRefusal(
          `${options.label} request timed out after ${timeoutMs} ms; the request was aborted.`,
        ),
      );
    }, timeoutMs);
  });

  const callerAbort = new Promise<never>((_resolve, reject) => {
    if (!externalSignal) {
      return;
    }
    externalAbortHandler = () => {
      abortedByCaller = true;
      controller.abort();
      reject(transportRefusal(`${options.label} request was aborted by the caller.`));
    };
    externalSignal.addEventListener('abort', externalAbortHandler, { once: true });
  });

  try {
    return await Promise.race([
      fetchImpl(url, { ...options.init, signal: controller.signal }),
      timeout,
      callerAbort,
    ]);
  } catch (error) {
    if (error instanceof RefusalError) {
      throw error;
    }
    if (abortedByCaller) {
      throw transportRefusal(`${options.label} request was aborted by the caller.`);
    }
    if (timedOut) {
      throw transportRefusal(
        `${options.label} request timed out after ${timeoutMs} ms; the request was aborted.`,
      );
    }
    const detail = error instanceof Error && error.message.length > 0 ? `: ${error.message}` : '';
    throw transportRefusal(`${options.label} request failed${detail}`);
  } finally {
    if (timer !== undefined) {
      clearTimeout(timer);
    }
    if (externalAbortHandler) {
      externalSignal?.removeEventListener('abort', externalAbortHandler);
    }
  }
}
