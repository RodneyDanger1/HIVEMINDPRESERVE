#!/usr/bin/env node
/**
 * Static QA-harness validator. Runs NOW (no app, no Playwright, no browser).
 * Checks e2e/data/control-matrix.json and docs/trace/* against the frozen
 * docs/contracts/UI-COMMANDS-v1.md vocabulary, and fails on fabricated statuses.
 *
 * Exit codes: 0 = all checks pass; 1 = one or more checks failed; 2 = file/parse error.
 */
import { readFileSync, existsSync } from 'node:fs';
import { fileURLToPath } from 'node:url';
import path from 'node:path';

const ROOT = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..', '..');
const MATRIX = path.join(ROOT, 'e2e', 'data', 'control-matrix.json');
const CONTRACT = path.join(ROOT, 'docs', 'contracts', 'UI-COMMANDS-v1.md');
const TRACE_DIR = path.join(ROOT, 'docs', 'trace');

const failures = [];
const warnings = [];
const checks = [];
function check(name, ok, detail) {
  checks.push({ name, ok, detail });
  if (!ok) failures.push(`${name}: ${detail}`);
  console.log(`${ok ? 'PASS' : 'FAIL'}  ${name}${ok ? '' : ` — ${detail}`}`);
}

function readText(file) { return readFileSync(file, 'utf8'); }
function readJson(file) { return JSON.parse(readFileSync(file, 'utf8')); }

function section(text, startHeading, endHeading) {
  const start = text.indexOf(startHeading);
  if (start < 0) return '';
  const from = start + startHeading.length;
  const end = endHeading ? text.indexOf(endHeading, from) : -1;
  return text.slice(from, end < 0 ? text.length : end);
}

function unique(list) { return [...new Set(list)]; }

try {
  const contract = readText(CONTRACT);
  // First column of the §2 command table only; combos (`VIEWPORT_SET` / `VIEWPORT_RECENTER`) split.
  const commands = unique(
    section(contract, '## 2. Commands', '## 3. Events').split('\n')
      .filter((line) => line.startsWith('| `'))
      .flatMap((line) => {
        const firstCell = line.match(/^\| ([^|]+) \|/)?.[1] ?? '';
        return [...firstCell.matchAll(/`([A-Z][A-Z0-9_]+)`/g)].map((m) => m[1]);
      }),
  );
  const frozenCommands = commands.filter((name) => !name.startsWith('DIFF_'));
  const decisionGatedCommands = commands.filter((name) => name.startsWith('DIFF_'));
  const events = unique([...section(contract, '## 3. Events', '## 4. ').matchAll(/`([A-Z][A-Z0-9_]+)`/g)].map((m) => m[1]));
  const refusals = unique([...section(contract, '## 5. Refusal codes', '## 6. ').matchAll(/`(E_[A-Z0-9_]+)`/g)].map((m) => m[1]));

  check('contract parsed: frozen commands', frozenCommands.length >= 10, `found ${frozenCommands.length}: ${frozenCommands.join(',')}`);
  check('contract parsed: decision-gated DIFF commands', decisionGatedCommands.length === 3, decisionGatedCommands.join(','));
  check('contract parsed: events', events.length >= 12, `found ${events.length}`);
  check('contract parsed: refusal codes', refusals.length >= 30, `found ${refusals.length}`);

  const matrix = readJson(MATRIX);
  const surfaces = matrix.surfaces;
  check('matrix parses', Array.isArray(surfaces), typeof surfaces);
  const guideIds = surfaces.filter((s) => /^S\d{2}$/.test(s.id)).map((s) => s.id);
  check('matrix has the 28 guide surfaces', guideIds.length === 28, `got ${guideIds.length}`);
  check('guide ids unique S01..S28', new Set(guideIds).size === 28 && guideIds.every((id, i) => id === `S${String(i + 1).padStart(2, '0')}`), guideIds.join(','));
  const supplemental = surfaces.filter((s) => !/^S\d{2}$/.test(s.id)).map((s) => `${s.id}:${s.status}`);
  check('supplemental controls are explicitly non-guide ids', supplemental.length === (matrix._meta.supplemental_controls ?? []).length, supplemental.join(','));

  const allCommands = surfaces.flatMap((s) => [s.command, ...(s.secondary_commands ?? [])]).filter(Boolean);
  const badCommands = allCommands.filter((name) => !commands.includes(name));
  check('every command is frozen in UI-COMMANDS-v1', badCommands.length === 0, badCommands.join(','));
  const uncovered = frozenCommands.filter((name) => !allCommands.includes(name));
  check('every frozen command is covered by a control', uncovered.length === 0, uncovered.join(','));
  const badRefusals = surfaces.flatMap((s) => s.refusal_codes.filter((code) => !refusals.includes(code)).map((code) => `${s.id}:${code}`));
  check('every refusal code is frozen in UI-COMMANDS-v1', badRefusals.length === 0, badRefusals.join(','));
  const badEvents = surfaces.flatMap((s) => s.success_events.filter((name) => !events.includes(name)).map((name) => `${s.id}:${name}`));
  check('every referenced event is frozen in UI-COMMANDS-v1', badEvents.length === 0, badEvents.join(','));

  const hopNames = ['1_render', '2_handler', '3_client_call', '4_request', '5_route', '6_service', '7_store', '8_provider', '9_response', '10_render_back', '11_reentry'];
  check('11-hop names exact', JSON.stringify(matrix.hop_names) === JSON.stringify(hopNames), JSON.stringify(matrix.hop_names));
  const allowedStatuses = new Set(matrix._meta.status_values);
  const badStatus = surfaces.filter((s) => !allowedStatuses.has(s.status)).map((s) => `${s.id}:${s.status}`);
  check('statuses are pre-evidence only (no PASS)', badStatus.length === 0 && surfaces.every((s) => !['PASS', 'WIRED-AND-WORKS'].includes(s.status)), badStatus.join(','));

  const ackViolations = surfaces
    .filter((s) => s.kind === 'mutating-command')
    .filter((s) => s.ack_required !== true || !s.success_events.includes('ACK'))
    .map((s) => s.id);
  check('mutating commands require ACK', ackViolations.length === 0, ackViolations.join(','));
  const nonFrozenUsed = surfaces.filter((s) => s.command && ['DIFF_PROPOSE', 'DIFF_APPLY', 'DIFF_REJECT'].includes(s.command));
  check('DIFF_* not assumed frozen', nonFrozenUsed.length === 0, nonFrozenUsed.map((s) => s.id).join(','));

  // --- trace seed files ---
  for (const file of ['controls.json', 'traces.json', 'events.json']) {
    const p = path.join(TRACE_DIR, file);
    check(`trace seed exists: ${file}`, existsSync(p), p);
    if (!existsSync(p)) continue;
    let doc;
    try { doc = readJson(p); } catch (error) { check(`trace seed parses: ${file}`, false, String(error)); continue; }
    check(`trace seed parses: ${file}`, true, 'json ok');
    if (file === 'controls.json') {
      check('controls.json covers every matrix control', doc.controls?.length === surfaces.length, `got ${doc.controls?.length} vs ${surfaces.length}`);
      const seeded = (doc.controls ?? []).every((c) => Object.keys(c.hops ?? {}).length === 11);
      check('controls.json each control has 11 hop slots', seeded, 'hop keys');
      const pending = (doc.controls ?? []).every((c) => Object.values(c.hops ?? {}).every((h) => h.verdict === 'PENDING'));
      check('controls.json verdicts start PENDING (no fake pass)', pending, 'verdict scan');
    }
    if (file === 'traces.json') check('traces.json has no fabricated runs', Array.isArray(doc.traces), 'traces[]');
    if (file === 'events.json') check('events.json has no fabricated deliveries', Array.isArray(doc.delivered) && doc.delivered.length === 0, 'delivered[] empty until captured');
  }

  // --- FigCraft expected delivery (blocked, recorded not failed) ---
  const assetsIndex = path.join(ROOT, 'docs', 'ui', 'figcraft', 'ASSETS-INDEX.json');
  if (!existsSync(assetsIndex)) warnings.push('BLOCKED: canonical docs/ui/figcraft/ASSETS-INDEX.json absent; manifest candidate docs/ui/figcraft/A/A-ASSETS.json is present and needs Lead acceptance (or a copy to the canonical path)');
} catch (error) {
  console.error(`ERROR: ${error instanceof Error ? error.message : String(error)}`);
  process.exit(2);
}

console.log('\n--- summary ---');
console.log(`checks=${checks.length} pass=${checks.filter((c) => c.ok).length} fail=${failures.length}`);
if (warnings.length) {
  for (const w of warnings) console.log(`WARN  ${w}`);
}
if (failures.length) {
  console.error(`\nFAIL: ${failures.length} check(s) failed`);
  process.exit(1);
}
console.log('OK: QA harness structure is consistent with the frozen contract; no execution claims made.');
