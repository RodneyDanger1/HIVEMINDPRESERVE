/**
 * Health probe for Settings (S21 read surface). The v1 server exposes
 * `GET /health`; key status comes from the patched SETTINGS_STATUS command.
 * A failed probe is surfaced as UNVERIFIED, never as "no key"/"healthy".
 */
import { apiBaseUrl } from './client';
import { clientRefusal, RefusalError } from './refusals';

export interface HealthSnapshot {
  readonly status: 'ok' | 'degraded' | 'failing' | 'unknown';
  readonly checks: readonly string[];
  readonly version: string | null;
}

export async function fetchHealth(fetchImpl: typeof fetch = fetch): Promise<HealthSnapshot> {
  let response: Response;
  try {
    response = await fetchImpl(`${apiBaseUrl()}/health`, {
      credentials: 'same-origin',
      headers: { accept: 'application/json' },
    });
  } catch {
    throw new RefusalError(clientRefusal('E_WS_DISCONNECTED'));
  }
  let body: unknown = null;
  try {
    body = await response.json();
  } catch {
    body = null;
  }
  if (typeof body !== 'object' || body === null) {
    throw new RefusalError(clientRefusal('E_SCHEMA_INVALID', { message: 'GET /health returned no JSON.' }));
  }
  const record = body as Record<string, unknown>;
  const statusRaw = record.status;
  const status =
    statusRaw === 'ok' || statusRaw === 'degraded' || statusRaw === 'failing'
      ? statusRaw
      : 'unknown';
  const checksRaw = record.checks;
  const checks: string[] = [];
  if (Array.isArray(checksRaw)) {
    for (const check of checksRaw) {
      if (typeof check === 'object' && check !== null) {
        const checkRecord = check as Record<string, unknown>;
        if (typeof checkRecord.name === 'string') {
          checks.push(`${checkRecord.name}:${checkRecord.ok === true ? 'ok' : 'failed'}`);
        }
      }
    }
  }
  const version = typeof record.version === 'string' ? record.version : null;
  return { status, checks, version };
}
