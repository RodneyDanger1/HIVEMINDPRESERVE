#!/usr/bin/env node
/**
 * Generate the FigCraft catalog, the served SVG copies and SOURCES.json from
 * the canonical acceptance index.
 *
 * Input  : docs/ui/figcraft/ASSETS-INDEX.json (schema v2, per-board sha + audit)
 * Outputs: web/src/figcraft/catalog.ts
 *          web/public/figcraft/<board>.svg
 *          web/public/figcraft/SOURCES.json
 *
 * Every source SVG is hashed and must match the acceptance index before it is
 * copied; the copied bytes are hashed again. A board marked "watchdog-audited"
 * is cross-checked against ledgers/ui-watchdog.json, and a "self-verified"
 * board is cross-checked against docs/ui/figcraft/HiveMind/build/verify-report.json.
 * Nothing here invents a board, a dimension or a status.
 *
 * Usage:
 *   node tools/build-figcraft-catalog.mjs           # write outputs
 *   node tools/build-figcraft-catalog.mjs --check   # verify no drift, no writes
 */
import { createHash } from 'node:crypto';
import { existsSync, mkdirSync, readFileSync, readdirSync, renameSync, rmSync, writeFileSync } from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const ROOT = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');
const FC = path.join(ROOT, 'docs', 'ui', 'figcraft');
const INDEX_PATH = path.join(FC, 'ASSETS-INDEX.json');
const WATCHDOG_PATH = path.join(ROOT, 'ledgers', 'ui-watchdog.json');
const OUT_DIR = path.join(ROOT, 'web', 'public', 'figcraft');
const CATALOG_PATH = path.join(ROOT, 'web', 'src', 'figcraft', 'catalog.ts');
const SOURCES_PATH = path.join(OUT_DIR, 'SOURCES.json');

const AUDIT_STATUSES = new Set(['watchdog-audited', 'self-verified', 'proposed', 'unverified']);
const GROUPS = [
  {
    id: 'figcraft-hivemind',
    label: 'HiveMind \u00b7 self-verified / proposed',
    description:
      'HiveMind spatial style system: its own CLI verify report passed, but it is not in the A/B watchdog ledger. Boards marked PROPOSED-ADDITION or motion proposal are labelled proposed.',
  },
  {
    id: 'figcraft-a',
    label: 'Design system \u00b7 A \u00b7 watchdog-audited',
    description:
      'Watchdog-audited design-system manifest A (foundations, brand, components, specimens). The ledger verdict covers this set per artifact.',
  },
  {
    id: 'figcraft-b',
    label: 'Canvas \u00b7 B \u00b7 watchdog-audited',
    description:
      'Watchdog-audited canvas/interaction manifest B (canvas default, zoom states, interaction, hero). The ledger verdict covers this set per artifact.',
  },
];

function fail(message) {
  throw new Error(message);
}

function sha256File(file) {
  return createHash('sha256').update(readFileSync(file)).digest('hex');
}

function sha256Text(text) {
  return createHash('sha256').update(text, 'utf8').digest('hex');
}

function readJson(file, label) {
  if (!existsSync(file)) {
    fail(`${label} is missing: ${path.relative(ROOT, file)}`);
  }
  try {
    return JSON.parse(readFileSync(file, 'utf8'));
  } catch (error) {
    fail(`${label} is not valid JSON: ${error instanceof Error ? error.message : String(error)}`);
  }
}

function repoRelative(absolute) {
  return path.relative(ROOT, absolute).split(path.sep).join('/');
}

function resolveInside(relativePath, base, label) {
  if (typeof relativePath !== 'string' || relativePath.trim() === '') {
    fail(`${label} must be a non-empty repository-relative path`);
  }
  if (relativePath.includes('\\') || path.isAbsolute(relativePath)) {
    fail(`${label} must use forward slashes and be repository-relative: ${relativePath}`);
  }
  const absolute = path.resolve(ROOT, relativePath);
  const within = path.relative(base, absolute);
  if (within === '' || within.startsWith('..') || path.isAbsolute(within)) {
    fail(`${label} escapes ${repoRelative(base)}: ${relativePath}`);
  }
  return absolute;
}

function assertBoard(board) {
  const where = `board ${JSON.stringify(board.id)}`;
  if (typeof board.id !== 'string' || !/^[a-z0-9][a-z0-9-]*$/.test(board.id)) {
    fail(`${where}: invalid id`);
  }
  if (!GROUPS.some((group) => group.id === board.owner)) {
    fail(`${where}: unknown owner ${JSON.stringify(board.owner)}`);
  }
  for (const key of ['title', 'source_svg', 'sha256_svg', 'manifest', 'svg_file']) {
    if (typeof board[key] !== 'string' || board[key].trim() === '') {
      fail(`${where}: missing ${key}`);
    }
  }
  if (!/^[a-z0-9][a-z0-9-]*\.svg$/.test(board.svg_file)) {
    fail(`${where}: unsafe svg_file ${JSON.stringify(board.svg_file)}`);
  }
  if (!Number.isInteger(board.width) || !Number.isInteger(board.height) || board.width <= 0 || board.height <= 0) {
    fail(`${where}: invalid dimensions ${board.width}x${board.height}`);
  }
  if (!Number.isInteger(board.bytes) || board.bytes <= 0) {
    fail(`${where}: invalid bytes ${board.bytes}`);
  }
  if (!Array.isArray(board.surfaces)) {
    fail(`${where}: surfaces must be an array`);
  }
  for (const surface of board.surfaces) {
    if (typeof surface.surfaceId !== 'string' || typeof surface.name !== 'string') {
      fail(`${where}: invalid surface record`);
    }
  }
  const audit = board.audit;
  if (audit === null || typeof audit !== 'object' || !AUDIT_STATUSES.has(audit.status)) {
    fail(`${where}: invalid audit status ${JSON.stringify(audit?.status)}`);
  }
  if (typeof audit.basis !== 'string' || audit.basis.trim() === '') {
    fail(`${where}: audit basis is required`);
  }
  if (board.served === false) {
    fail(`${where}: served=false boards must not be in the served board list`);
  }
}

function validateIndex(index) {
  if (index === null || typeof index !== 'object') {
    fail('ASSETS-INDEX.json must contain a JSON object');
  }
  if (typeof index.schema !== 'string' || !index.schema.startsWith('hiverouter.assets-index.')) {
    fail(`unsupported index schema ${JSON.stringify(index.schema)}; run tools/build_assets_index.py`);
  }
  if (!Array.isArray(index.boards) || index.boards.length === 0) {
    fail('ASSETS-INDEX.json has no boards; run tools/build_assets_index.py');
  }
  if (typeof index.generated_at !== 'string' || index.generated_at.trim() === '') {
    fail('ASSETS-INDEX.json is missing generated_at');
  }
  if (typeof index.counts?.boards !== 'number') {
    fail('ASSETS-INDEX.json is missing counts.boards');
  }
  const ids = new Set();
  const files = new Set();
  for (const board of index.boards) {
    assertBoard(board);
    if (ids.has(board.id)) fail(`duplicate board id ${board.id}`);
    if (files.has(board.svg_file)) fail(`duplicate board svg_file ${board.svg_file}`);
    ids.add(board.id);
    files.add(board.svg_file);
  }
  if (index.counts.boards !== index.boards.length) {
    fail(`counts.boards ${index.counts.boards} does not match ${index.boards.length} board records`);
  }
  const statusCount = (status) => index.boards.filter((board) => board.audit.status === status).length;
  const expected = {
    watchdog_audited_boards: statusCount('watchdog-audited'),
    self_verified_boards: statusCount('self-verified'),
    proposed_boards: statusCount('proposed'),
    unverified_boards: statusCount('unverified'),
    surface_mappings: index.boards.reduce((sum, board) => sum + board.surfaces.length, 0),
    distinct_surfaces: new Set(
      index.boards.flatMap((board) => board.surfaces.map((surface) => surface.surfaceId)),
    ).size,
    owners: new Set(index.boards.map((board) => board.owner)).size,
  };
  for (const [key, value] of Object.entries(expected)) {
    if (index.counts[key] !== undefined && index.counts[key] !== value) {
      fail(`counts.${key} ${index.counts[key]} does not match the board records (${value})`);
    }
  }
  if (Array.isArray(index.assets) && index.counts.assets !== index.assets.length) {
    fail(`counts.assets ${index.counts.assets} does not match ${index.assets.length} asset rows`);
  }
}

function validateEvidence(index) {
  const watchdog = readJson(WATCHDOG_PATH, 'watchdog ledger');
  const artifacts = new Map(
    (watchdog.artifacts ?? [])
      .filter((artifact) => typeof artifact.file === 'string')
      .map((artifact) => [artifact.file, artifact]),
  );
  const verifyPath = index.hivemind_verify?.report
    ? resolveInside(index.hivemind_verify.report, FC, 'hivemind_verify.report')
    : null;
  if (verifyPath !== null) {
    if (!existsSync(verifyPath)) fail(`HiveMind verify report is missing: ${repoRelative(verifyPath)}`);
    const recorded = String(index.hivemind_verify.sha256 ?? '').toLowerCase();
    const actual = sha256File(verifyPath);
    if (recorded !== actual) {
      fail(`HiveMind verify report sha mismatch: index ${recorded} actual ${actual}`);
    }
    if (index.hivemind_verify.status !== 'PASS') {
      fail(`HiveMind verify report status is ${JSON.stringify(index.hivemind_verify.status)}, not PASS`);
    }
  }
  for (const board of index.boards) {
    const source = resolveInside(board.source_svg, FC, `${board.id}.source_svg`);
    if (!existsSync(source)) fail(`${board.id}: source SVG is missing: ${board.source_svg}`);
    const actual = sha256File(source);
    if (actual !== board.sha256_svg.toLowerCase()) {
      fail(`${board.id}: source sha mismatch: index ${board.sha256_svg} actual ${actual}`);
    }
    const bytes = readFileSync(source).length;
    if (bytes !== board.bytes) {
      fail(`${board.id}: source byte count mismatch: index ${board.bytes} actual ${bytes}`);
    }
    if (board.audit.status === 'watchdog-audited') {
      const artifact = artifacts.get(board.source_svg);
      if (artifact === undefined) {
        fail(`${board.id}: marked watchdog-audited but absent from ledgers/ui-watchdog.json`);
      }
      if (artifact.verdict !== 'PASS' && artifact.verdict !== 'PASS-WITH-NOTES') {
        fail(`${board.id}: watchdog verdict is ${JSON.stringify(artifact.verdict)}`);
      }
      if (String(artifact.sha256 ?? '').toLowerCase() !== actual) {
        fail(`${board.id}: watchdog sha mismatch: ledger ${artifact.sha256} actual ${actual}`);
      }
    }
    if (board.audit.status === 'self-verified' && index.hivemind_verify?.status !== 'PASS') {
      fail(`${board.id}: marked self-verified but the HiveMind verify report is not PASS`);
    }
  }
}

function buildCatalog(index, indexSha) {
  const groups = GROUPS.map((group) => ({
    id: group.id,
    label: group.label,
    description: group.description,
  }));
  const boards = index.boards.map((board) => ({
    id: board.id,
    owner: board.owner,
    title: board.title,
    width: board.width,
    height: board.height,
    svgFile: board.svg_file,
    sourcePath: board.source_svg,
    sha256: board.sha256_svg.toLowerCase(),
    manifest: board.manifest,
    audit: {
      status: board.audit.status,
      verdict: board.audit.verdict ?? null,
      ledger: board.audit.ledger ?? null,
      basis: board.audit.basis,
    },
    surfaces: board.surfaces.map((surface) => ({
      surfaceId: surface.surfaceId,
      name: surface.name,
    })),
  }));
  const sourceManifests = [
    {
      name: 'ASSETS-INDEX',
      path: 'docs/ui/figcraft/ASSETS-INDEX.json',
      sha256: indexSha.toUpperCase(),
    },
    ...Object.entries(index.source_manifests ?? {}).map(([name, entry]) => ({
      name,
      path: entry.path,
      sha256: String(entry.sha256 ?? '').toUpperCase(),
    })),
  ];
  const counts = {
    boards: index.counts.boards,
    assets: index.counts.assets,
    surfaces: index.counts.surfaces,
    surfaceMappings: index.counts.surface_mappings ?? boards.reduce((sum, board) => sum + board.surfaces.length, 0),
    distinctSurfaces: index.counts.distinct_surfaces ?? index.counts.surfaces,
    owners: index.counts.owners ?? new Set(boards.map((board) => board.owner)).size,
    watchdogAuditedBoards: index.counts.watchdog_audited_boards ?? 0,
    selfVerifiedBoards: index.counts.self_verified_boards ?? 0,
    proposedBoards: index.counts.proposed_boards ?? 0,
    unverifiedBoards: index.counts.unverified_boards ?? 0,
    watchdog: index.watchdog?.overall ?? 'UNKNOWN',
    allowFrontend: index.watchdog?.allow_frontend === true,
  };
  const header = [
    '/**',
    ' * GENERATED FigCraft catalog - do not hand-edit.',
    ' *',
    ' * Generator: tools/build-figcraft-catalog.mjs',
    ' * Source   : docs/ui/figcraft/ASSETS-INDEX.json',
    ` * Index sha: ${indexSha.toUpperCase()}`,
    ` * Index at : ${index.generated_at}`,
    ' * Served   : web/public/figcraft/<svgFile>; provenance in SOURCES.json',
    ' *',
    ' * Per-board audit status is part of the data, not a UI guess:',
    ' *  - watchdog-audited: PASS/PASS-WITH-NOTES artifact in ledgers/ui-watchdog.json',
    ' *  - self-verified   : HiveMind CLI verify report PASS; NOT a watchdog audit',
    ' *  - proposed        : PROPOSED-ADDITION / motion proposal; not watchdog-audited',
    ' *  - unverified      : no accepted audit evidence',
    ' */',
    "import type {",
    '  FigCraftBoard,',
    '  FigCraftGroup,',
    '  FigCraftSourceManifest,',
    "} from './types';",
    '',
  ].join('\n');
  const body = [
    `export const FIGCRAFT_GROUPS: readonly FigCraftGroup[] = ${JSON.stringify(groups, null, 2)};`,
    '',
    `export const FIGCRAFT_BOARDS: readonly FigCraftBoard[] = ${JSON.stringify(boards, null, 2)};`,
    '',
    `export const FIGCRAFT_COUNTS = ${JSON.stringify(counts, null, 2)} as const;`,
    '',
    `export const FIGCRAFT_SOURCE_MANIFESTS: readonly FigCraftSourceManifest[] = ${JSON.stringify(sourceManifests, null, 2)};`,
    '',
  ].join('\n');
  return `${header}${body}`;
}

function buildSources(index, indexSha, boards) {
  const counts = {
    boards: index.counts.boards,
    assets: index.counts.assets,
    surfaces: index.counts.surfaces,
    surfaceMappings: index.counts.surface_mappings ?? boards.reduce((sum, board) => sum + board.surfaces.length, 0),
    distinctSurfaces: index.counts.distinct_surfaces ?? index.counts.surfaces,
    owners: index.counts.owners ?? new Set(boards.map((board) => board.owner)).size,
    watchdogAuditedBoards: index.counts.watchdog_audited_boards ?? 0,
    selfVerifiedBoards: index.counts.self_verified_boards ?? 0,
    proposedBoards: index.counts.proposed_boards ?? 0,
    unverifiedBoards: index.counts.unverified_boards ?? 0,
    watchdog: index.watchdog?.overall ?? 'UNKNOWN',
    allowFrontend: index.watchdog?.allow_frontend === true,
  };
  const sources = {
    schema: 'hivemind.figcraft.public-assets/v2',
    generated_at: index.generated_at,
    generated_by: 'tools/build-figcraft-catalog.mjs from docs/ui/figcraft/ASSETS-INDEX.json',
    source_index: {
      path: 'docs/ui/figcraft/ASSETS-INDEX.json',
      sha256: indexSha.toUpperCase(),
      schema: index.schema,
    },
    watchdog: index.watchdog,
    hivemind_verify: index.hivemind_verify,
    counts,
    boards: boards.map((board) => ({
      publicPath: `/figcraft/${board.svg_file}`,
      sourcePath: board.source_svg,
      sha256: board.sha256_svg.toLowerCase(),
      bytes: board.bytes,
      owner: board.owner,
      title: board.title,
      audit: board.audit,
    })),
  };
  return `${JSON.stringify(sources, null, 2)}\n`;
}

function atomicWrite(file, content) {
  mkdirSync(path.dirname(file), { recursive: true });
  const temp = `${file}.tmp-${process.pid}`;
  writeFileSync(temp, content, 'utf8');
  renameSync(temp, file);
}

function atomicCopy(source, target) {
  mkdirSync(path.dirname(target), { recursive: true });
  const temp = `${target}.tmp-${process.pid}`;
  writeFileSync(temp, readFileSync(source));
  renameSync(temp, target);
}

function expectedBoardFiles(index) {
  return new Set(index.boards.map((board) => board.svg_file));
}

function inspectOutputs(index, catalog, sources) {
  const problems = [];
  const expected = expectedBoardFiles(index);
  const actualFiles = existsSync(OUT_DIR)
    ? readdirSync(OUT_DIR).filter((name) => name.endsWith('.svg')).sort()
    : [];
  if (existsSync(OUT_DIR)) {
    for (const name of readdirSync(OUT_DIR)) {
      if (name === 'SOURCES.json' || (name.endsWith('.svg') && expected.has(name))) {
        continue;
      }
      problems.push(`unexpected public file: web/public/figcraft/${name}`);
    }
  }
  for (const name of actualFiles) {
    if (!expected.has(name)) problems.push(`stale served SVG: web/public/figcraft/${name}`);
  }
  for (const board of index.boards) {
    const target = path.join(OUT_DIR, board.svg_file);
    if (!existsSync(target)) {
      problems.push(`missing served SVG: web/public/figcraft/${board.svg_file}`);
      continue;
    }
    const actual = sha256File(target);
    if (actual !== board.sha256_svg.toLowerCase()) {
      problems.push(`served sha mismatch: ${board.svg_file} (${actual})`);
    }
  }
  if (!existsSync(CATALOG_PATH)) {
    problems.push('missing generated catalog: web/src/figcraft/catalog.ts');
  } else if (readFileSync(CATALOG_PATH, 'utf8') !== catalog) {
    problems.push('generated catalog is out of date: web/src/figcraft/catalog.ts');
  }
  if (!existsSync(SOURCES_PATH)) {
    problems.push('missing provenance file: web/public/figcraft/SOURCES.json');
  } else if (readFileSync(SOURCES_PATH, 'utf8') !== sources) {
    problems.push('generated provenance is out of date: web/public/figcraft/SOURCES.json');
  }
  return problems;
}

function main() {
  const check = process.argv.includes('--check');
  const unknown = process.argv.slice(2).filter((arg) => arg !== '--check');
  if (unknown.length > 0) {
    fail(`unknown argument(s): ${unknown.join(' ')} (usage: node tools/build-figcraft-catalog.mjs [--check])`);
  }
  const index = readJson(INDEX_PATH, 'ASSETS-INDEX.json');
  validateIndex(index);
  validateEvidence(index);
  const indexSha = sha256File(INDEX_PATH);
  const catalog = buildCatalog(index, indexSha);
  const sources = buildSources(index, indexSha, index.boards);

  if (check) {
    const problems = inspectOutputs(index, catalog, sources);
    const report = {
      mode: 'check',
      ok: problems.length === 0,
      boards: index.boards.length,
      problems,
      catalog_sha256: sha256Text(catalog),
      sources_sha256: sha256Text(sources),
    };
    console.log(JSON.stringify(report, null, 2));
    if (problems.length > 0) process.exitCode = 1;
    return;
  }

  let copied = 0;
  let unchanged = 0;
  mkdirSync(OUT_DIR, { recursive: true });
  for (const board of index.boards) {
    const source = resolveInside(board.source_svg, FC, `${board.id}.source_svg`);
    const target = path.join(OUT_DIR, board.svg_file);
    if (existsSync(target) && sha256File(target) === board.sha256_svg.toLowerCase()) {
      unchanged += 1;
      continue;
    }
    atomicCopy(source, target);
    if (sha256File(target) !== board.sha256_svg.toLowerCase()) {
      fail(`copied bytes do not match the accepted sha for ${board.id}`);
    }
    copied += 1;
  }
  const expected = expectedBoardFiles(index);
  let removed = 0;
  for (const name of readdirSync(OUT_DIR)) {
    if (name.endsWith('.svg') && !expected.has(name)) {
      rmSync(path.join(OUT_DIR, name));
      removed += 1;
    }
  }
  atomicWrite(CATALOG_PATH, catalog);
  atomicWrite(SOURCES_PATH, sources);
  const problems = inspectOutputs(index, catalog, sources);
  if (problems.length > 0) {
    fail(`generated outputs failed read-back verification: ${problems.join('; ')}`);
  }
  console.log(
    JSON.stringify(
      {
        mode: 'write',
        boards: index.boards.length,
        copied,
        unchanged,
        removed,
        catalog: 'web/src/figcraft/catalog.ts',
        catalog_sha256: sha256Text(catalog),
        sources: 'web/public/figcraft/SOURCES.json',
        sources_sha256: sha256Text(sources),
      },
      null,
      2,
    ),
  );
}

try {
  main();
} catch (error) {
  console.error(`build-figcraft-catalog: ${error instanceof Error ? error.message : String(error)}`);
  process.exitCode = 1;
}
