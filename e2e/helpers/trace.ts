import { readFileSync, writeFileSync, existsSync } from 'node:fs';
import { fileURLToPath } from 'node:url';

const TRACE_DIR = new URL('../../docs/trace/', import.meta.url);

export interface HopRecord {
  verdict: 'WIRED-AND-WORKS' | 'WIRED-BUT-DEAD' | 'MISSING' | 'REFUSED-BY-NAME' | 'UNVERIFIED';
  evidence: Array<{ kind: 'dom' | 'request' | 'ws-frame' | 'db-row' | 'log' | 'file'; pointer: string; quote?: string }>;
  captured_at: string;
}

export interface TraceRun {
  run_id: string;
  surface_id: string;
  base_url: string;
  started_at: string;
  hops: Record<string, HopRecord>;
  notes?: string;
}

function readJson<T>(file: string, fallback: T): T {
  const path = fileURLToPath(new URL(file, TRACE_DIR));
  if (!existsSync(path)) return fallback;
  return JSON.parse(readFileSync(path, 'utf8')) as T;
}

/** Appends a captured trace run; never invents verdicts (caller supplies evidence). */
export function appendTrace(run: TraceRun): void {
  const path = fileURLToPath(new URL('traces.json', TRACE_DIR));
  const doc = readJson<{ traces: TraceRun[] }>('traces.json', { traces: [] });
  doc.traces.push(run);
  writeFileSync(path, `${JSON.stringify(doc, null, 2)}\n`, 'utf8');
}

export function readControls(): unknown {
  return readJson('controls.json', { controls: [] });
}
