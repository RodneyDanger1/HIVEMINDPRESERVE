#!/usr/bin/env node
/**
 * Merge the live Playwright trace capture (docs/trace/live/trace-capture.json) into
 * docs/trace/controls.json. Only verdicts that were actually captured are applied;
 * everything else stays PENDING/UNVERIFIED. No fabricated pass.
 */
import { readFileSync, writeFileSync, existsSync } from 'node:fs';
import { fileURLToPath } from 'node:url';
import path from 'node:path';

const ROOT = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..', '..');
const CAPTURE = path.join(ROOT, 'docs', 'trace', 'live', 'trace-capture.json');
const CONTROLS = path.join(ROOT, 'docs', 'trace', 'controls.json');
if (!existsSync(CAPTURE)) {
  console.error('missing capture file: docs/trace/live/trace-capture.json (run the trace-hops spec first)');
  process.exit(1);
}
const capture = JSON.parse(readFileSync(CAPTURE, 'utf8'));
const controls = JSON.parse(readFileSync(CONTROLS, 'utf8'));
const byId = new Map(capture.surfaces.map((s) => [s.surface_id, s]));
let touched = 0;
let verdictsSet = 0;

for (const control of controls.controls) {
  const record = byId.get(control.surface_id);
  if (!record) continue;
  touched += 1;
  const evidenceBase = `docs/trace/live/trace-capture.json#${record.surface_id}`;
  for (const [hop, verdict] of Object.entries(record.hops)) {
    if (hop === '1_render' || verdict !== 'UNVERIFIED') {
      control.hops[hop] = {
        verdict,
        evidence: verdict === 'UNVERIFIED' ? [] : [{ kind: 'dom', pointer: evidenceBase, quote: record.evidence }],
      };
      if (verdict !== 'UNVERIFIED') verdictsSet += 1;
    }
  }
  if (record.present) {
    control.status = 'PARTIALLY_VERIFIED';
    control.notes = [control.notes, record.evidence, ...(record.notes ?? [])].filter(Boolean).join(' | ');
  } else {
    control.status = 'PENDING_APP_NOT_BUILT';
    control.notes = [control.notes, record.evidence].filter(Boolean).join(' | ');
  }
}

controls._meta = {
  ...controls._meta,
  live_capture: {
    source: 'docs/trace/live/trace-capture.json',
    generated_at: capture.generated_at,
    base_url: capture.base_url,
    controls_touched: touched,
    hops_evidence_backed: verdictsSet,
    rule: capture.rule,
  },
};
writeFileSync(CONTROLS, `${JSON.stringify(controls, null, 2)}\n`, 'utf8');
console.log(JSON.stringify({ controls: controls.controls.length, touched, hops_evidence_backed: verdictsSet, partially_verified: controls.controls.filter((c) => c.status === 'PARTIALLY_VERIFIED').length }, null, 2));
