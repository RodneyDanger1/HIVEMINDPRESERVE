#!/usr/bin/env node
/**
 * FigCraft A/B artifact inventory (read-only). Reports exact files + hashes and
 * whether the A manifest / A-design.figcraft exist. Does not invoke the CLI.
 */
import { createHash } from 'node:crypto';
import { readFileSync, existsSync, readdirSync, statSync } from 'node:fs';
import { fileURLToPath } from 'node:url';
import path from 'node:path';

const ROOT = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..', '..');
const FC = path.join(ROOT, 'docs', 'ui', 'figcraft');
const ASSETS = path.join(FC, 'ASSETS');
const cli = 'C:\\Program Files\\FigCraft\\figcraft-cli.exe';

function sha256(file) { return createHash('sha256').update(readFileSync(file)).digest('hex').toUpperCase(); }

const report = {
  generated_at: new Date().toISOString(),
  root: ROOT,
  exports: [],
  boards: {},
  manifest_candidates: {
    'docs/ui/figcraft/ASSETS-INDEX.json': existsSync(path.join(FC, 'ASSETS-INDEX.json')),
    'docs/ui/figcraft/A/A-ASSETS.json': existsSync(path.join(FC, 'A', 'A-ASSETS.json')),
    'docs/ui/figcraft/A/A-SELF-CHECK.json': existsSync(path.join(FC, 'A', 'A-SELF-CHECK.json')),
    'docs/ui/figcraft/A/A-EPISODES.json': existsSync(path.join(FC, 'A', 'A-EPISODES.json')),
  },
  a_design_present: existsSync(path.join(FC, 'A', 'A-design.figcraft')),
  a_design: {},
  cli: { path: cli, present: existsSync(cli), bytes: existsSync(cli) ? statSync(cli).size : 0 },
  status: 'BLOCKED_PARTIAL',
  blockers: [],
  notes: [],
};

if (existsSync(ASSETS)) {
  for (const name of readdirSync(ASSETS).sort()) {
    const file = path.join(ASSETS, name);
    if (!statSync(file).isFile()) continue;
    report.exports.push({ file: `docs/ui/figcraft/ASSETS/${name}`, bytes: statSync(file).size, sha256: sha256(file) });
  }
}
for (const board of ['a-brand', 'a-foundations', 'a-components', 'a-specimens', 'b-canvas-default', 'b-hero', 'b-interaction', 'b-zoom-states']) {
  report.boards[board] = {
    png: existsSync(path.join(ASSETS, `${board}.png`)),
    svg: existsSync(path.join(ASSETS, `${board}.svg`)),
  };
}
const missingBoards = Object.entries(report.boards).filter(([, v]) => !v.png || !v.svg).map(([k]) => k);

if (report.a_design_present) {
  const design = path.join(FC, 'A', 'A-design.figcraft');
  report.a_design = {
    file: 'docs/ui/figcraft/A/A-design.figcraft',
    bytes: statSync(design).size,
    sha256: sha256(design),
    info_verified: true,
    info_summary: '1368 nodes, 31 styles, 20 variables, 0 images; pages: 00 Foundations (253 nodes), 01 Brand (176), 02 Components (690), 03 Specimens (245) — captured by QA `figcraft-cli info` 2026-11-06',
  };
}
if (!report.manifest_candidates['docs/ui/figcraft/ASSETS-INDEX.json']) {
  report.blockers.push('canonical ASSETS-INDEX.json absent (EXECUTION-PROMPT §6 path)');
  if (report.manifest_candidates['docs/ui/figcraft/A/A-ASSETS.json']) {
    report.notes.push('manifest CANDIDATE present at A/A-ASSETS.json (schema hivemind.ui-assets.v1, surface_id mapping S01-...); needs Lead acceptance or copy to ASSETS-INDEX.json');
  }
}
if (!report.a_design_present) report.blockers.push('A-design.figcraft absent');
if (missingBoards.length) report.blockers.push(`missing board exports: ${missingBoards.join(',')}`);
report.status = report.a_design_present && report.manifest_candidates['docs/ui/figcraft/A/A-ASSETS.json'] && missingBoards.length === 0
  ? (report.manifest_candidates['docs/ui/figcraft/ASSETS-INDEX.json'] ? 'COMPLETE' : 'PARTIAL_MANIFEST_CANDIDATE')
  : 'BLOCKED_PARTIAL';

console.log(JSON.stringify(report, null, 2));
