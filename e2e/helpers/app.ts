import type { APIRequestContext } from '@playwright/test';

export interface AppProbe {
  ready: boolean;
  evidence: string[];
  healthStatus?: string;
  schema?: string;
  token?: string;
  lastSeq?: number;
}

/**
 * Probe the running app without touching a paid provider.
 * Uses the frozen HTTP surface from backend/crates/hiverouter-core/src/http.rs:
 *   GET /health -> {status, schema, transport:{last_seq}, ...}
 *   GET /api/session -> {token, schema, last_seq, ws}
 */
export async function probeApp(request: APIRequestContext): Promise<AppProbe> {
  const evidence: string[] = [];
  const health = await request.get('/health');
  evidence.push(`GET /health -> ${health.status()}`);
  if (health.status() !== 200) {
    return { ready: false, evidence };
  }
  const healthBody = (await health.json()) as {
    status?: string;
    schema?: string;
    transport?: { last_seq?: number };
  };
  evidence.push(`health.status=${JSON.stringify(healthBody.status)}`);
  if (healthBody.status !== 'ok' && healthBody.status !== 'degraded') {
    return { ready: false, evidence };
  }

  const session = await request.get('/api/session');
  evidence.push(`GET /api/session -> ${session.status()}`);
  if (session.status() !== 200) {
    return { ready: false, evidence };
  }
  const sessionBody = (await session.json()) as {
    token?: string;
    schema?: string;
    last_seq?: number;
  };
  evidence.push(`session.schema=${JSON.stringify(sessionBody.schema)} token=${sessionBody.token ? 'present' : 'absent'}`);
  return {
    ready: true,
    evidence,
    healthStatus: healthBody.status,
    schema: sessionBody.schema ?? healthBody.schema,
    token: sessionBody.token,
    lastSeq: sessionBody.last_seq ?? healthBody.transport?.last_seq,
  };
}

/** Exchanges the session token at the app origin for the WS URL (no secrets logged). */
export function wsUrl(baseURL: string): string {
  const url = new URL(baseURL);
  url.protocol = url.protocol === 'https:' ? 'wss:' : 'ws:';
  url.pathname = '/ws';
  return url.toString();
}
