#!/usr/bin/env node
/**
 * Regenerates the trace data seeds from e2e/data/control-matrix.json:
 *   docs/trace/controls.json  (28 controls × 11 hop slots, all PENDING)
 *   docs/trace/traces.json    (empty runs; no fabricated traces)
 *   docs/trace/events.json    (event parity skeleton, empty delivered[])
 * Safe to re-run; overwrites seeds only.
 */
import { readFileSync, writeFileSync, mkdirSync } from 'node:fs';
import { fileURLToPath } from 'node:url';
import path from 'node:path';

const ROOT = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..', '..');
const matrix = JSON.parse(readFileSync(path.join(ROOT, 'e2e', 'data', 'control-matrix.json'), 'utf8'));
const contract = readFileSync(path.join(ROOT, 'docs', 'contracts', 'UI-COMMANDS-v1.md'), 'utf8');
const traceDir = path.join(ROOT, 'docs', 'trace');
mkdirSync(traceDir, { recursive: true });

const section = (text, start, end) => {
  const from = text.indexOf(start);
  if (from < 0) return '';
  const to = text.indexOf(end, from + start.length);
  return text.slice(from + start.length, to < 0 ? text.length : to);
};
const events = [...new Set([...section(contract, '## 3. Events', '## 4. ').matchAll(/`([A-Z][A-Z0-9_]+)`/g)].map((m) => m[1]))];
const hopNames = matrix.hop_names;
const now = new Date().toISOString();

const controls = {
  _meta: {
    schema: 'hiverouter.trace.controls/v1',
    source: 'e2e/data/control-matrix.json',
    status: 'PENDING_APP_NOT_BUILT',
    evidence_rule: 'No verdict may leave PENDING without captured evidence (dom/request/ws-frame/db-row/log/file).',
    generated_at: now,
  },
  controls: matrix.surfaces.map((s) => ({
    surface_id: s.id,
    surface: s.surface,
    kind: s.kind,
    command: s.command,
    owner: s.owner,
    acceptance: s.acceptance,
    status: 'PENDING_APP_NOT_BUILT',
    hops: Object.fromEntries(hopNames.map((hop) => [hop, { verdict: 'PENDING', evidence: [] }])),
  })),
};

const traces = {
  _meta: {
    schema: 'hiverouter.trace.runs/v1',
    status: 'NO_RUNS',
    note: 'Appended by e2e/helpers/trace.ts when a Playwright run captures evidence; empty means nothing has been executed.',
    generated_at: now,
  },
  traces: [],
};

const eventsDoc = {
  _meta: {
    schema: 'hiverouter.trace.events/v1',
    status: 'NOT_COLLECTED',
    event_vocabulary_source: 'docs/contracts/UI-COMMANDS-v1.md §3 (frozen)',
    parity_rules: [
      'emitted set (server source) == subscribed set (client addEventListener) == delivered set (browser capture)',
      'SSE/WS named frames must be unwrapped only when the schema marker is present',
      'live payload shape must equal the persisted payload shape',
      'terminal run: run row status/error, terminal event payload, usage/cost/tokens must agree',
      'failed/budget terminal runs: runs.cost_usd/prompt_tokens/completion_tokens must be NULL/incomplete, never 0',
    ],
    generated_at: now,
  },
  emitted: [],
  subscribed: [],
  delivered: [],
  payload_parity: [],
  terminal_records: [],
  event_vocabulary: events,
};

writeFileSync(path.join(traceDir, 'controls.json'), `${JSON.stringify(controls, null, 2)}\n`, 'utf8');
writeFileSync(path.join(traceDir, 'traces.json'), `${JSON.stringify(traces, null, 2)}\n`, 'utf8');
writeFileSync(path.join(traceDir, 'events.json'), `${JSON.stringify(eventsDoc, null, 2)}\n`, 'utf8');

console.log(`wrote docs/trace/controls.json (${controls.controls.length} controls × ${hopNames.length} hops, all PENDING)`);
console.log('wrote docs/trace/traces.json (0 runs)');
console.log(`wrote docs/trace/events.json (event vocabulary ${events.length})`);
