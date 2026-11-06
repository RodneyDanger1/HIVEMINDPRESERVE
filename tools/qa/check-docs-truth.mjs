#!/usr/bin/env node
/**
 * docs-truth checker for HiveMind release documentation.
 *
 * Enforces that current-facing docs agree with the product truth:
 *   VERSION   web/package.json == every backend crate version (current source line); release-facing docs
 *             (README/RELEASE) state the sealed release version from tools/release-manifest.json, and a
 *             pre-release working tree (e.g. 1.2.0-dev) is never presented as the released version.
 *   SEALED    RELEASE-EVIDENCE keeps the sealed artifact hashes from tools/release-manifest.json and (when the
 *             tree is a pre-release) marks the working tree as pre-release.
 *   STREAMING current docs claim real SSE/token streaming (not the retired one-delta behavior).
 *   HASHES    superseded 1.0 artifact hashes never appear as current; RELEASE-EVIDENCE points at
 *             tools/release-manifest.json as the exact-hash authority (no invented hashes).
 *   STATE     the retired "15 states" claim never appears as current.
 *   DOCS      UI-COMMANDS-v1 lists every contract event (23) and agent state (16) derived from source.
 *   CLAIMS    claim-register rows/notes carry stale behavior/hashes only when marked historical.
 *
 * Historical content is legitimate and is masked by:
 *   - `<!-- docs-truth:historical:start -->` ... `<!-- docs-truth:historical:end -->`
 *   - a markdown heading matching /historical|supersed|pre-1\.1|1\.0(\.0)? (release|record|artifact)/i
 *     (mask ends at the next heading of the same or higher level)
 *   - claims CSV rows containing superseded/historical/pre-1.1
 *   - claims JSON under a path containing "historical" or a value containing "SUPERSEDED"
 *
 * Usage:
 *   node tools/qa/check-docs-truth.mjs [--root <dir>] [--quiet]
 *
 * Exit codes: 0 = all checks pass; 1 = one or more truth violations; 2 = file/parse error.
 */
import { readFileSync, existsSync, readdirSync, statSync } from 'node:fs';
import { fileURLToPath } from 'node:url';
import path from 'node:path';

const HERE = path.dirname(fileURLToPath(import.meta.url));
const DEFAULT_ROOT = path.resolve(HERE, '..', '..');

const OLD_HASHES = [
  'b17eb3d6469c347f623a4dedc94537b15bb633a76215828742624fa6fe062a9a',
  '49f46e2be488dd675280f3dae7e8d455ac5eeec2aa46d7fd44f1c2557c386623',
  '46305fcb1fe1bee23d3645a94a82635a68ee47fbcd3ce0537df747de3c6105ef',
  '184d51a7c3cca7b9d50954b677c5ad8a14472700a651fdf0d270441be3eac9a9',
  '9b2ed5a7',
  'bc7a2bad',
];
const OLD_HASH_RE = new RegExp(`(${OLD_HASHES.map((h) => h.slice(0, 8)).join('|')})`, 'i');
const STALE_STREAMING_RE = /\b(one|single)\s+(?:`?REPLY_DELTA`?\s+)?delta\b|\bnon[- ]streaming\b|\bnot\s+stream(?:ing|ed)\b|streaming\s+(?:is\s+)?(?:not|still\s+(?:pending|in\s+progress|being\s+completed))\b|true\s+SSE[^.\n]{0,60}(?:not\s+claimed|is\s+1\.1|pending|tracked\s+for\s+1\.1)|\bchunk-by-chunk[^.\n]{0,60}(?:not\s+claimed|is\s+1\.1)/i;
const STREAMING_POSITIVE_RE = /\bSSE\b|server-sent|token[- ]by[- ]token|stream(?:s|ing|ed)?\s+(?:as|while|chunk|token|real|implemented|now|live)|REPLY_DELTA\s+(?:frames?|as\s+they|arriv)/i;
const STALE_STATE_RE = /\b15[ -]states?\b|\bfifteen\s+states\b|\b15\s+of\s+16\b/i;
const STALE_VERSION_RE = /\bHiveMind\s+1\.0(?:\.0)?\b|\bHiveMind\s+v1\.0\b/i;
const MANIFEST_POINTER_RE = /tools\/release-manifest\.json/;

const args = process.argv.slice(2);
let root = DEFAULT_ROOT;
let quiet = false;
for (let i = 0; i < args.length; i += 1) {
  if (args[i] === '--root' && args[i + 1]) { root = path.resolve(args[i + 1]); i += 1; }
  else if (args[i] === '--quiet') quiet = true;
  else { console.error(`unknown argument: ${args[i]}`); process.exit(2); }
}

const violations = [];
const checks = [];
function fail(file, line, rule, message) {
  violations.push({ file, line, rule, message });
}
function check(name, ok, detail) {
  checks.push({ name, ok, detail });
  if (!quiet) console.log(ok ? `PASS  ${name}` : `FAIL  ${name} — ${detail}`);
}
function fileError(message) {
  console.error(`FILE ERROR: ${message}`);
  process.exit(2);
}
function read(rel) {
  const p = path.join(root, rel);
  if (!existsSync(p) || !statSync(p).isFile()) fileError(`required file missing: ${rel}`);
  return readFileSync(p, 'utf8');
}
function readJson(rel) {
  const text = read(rel);
  try { return JSON.parse(text); }
  catch (e) { fileError(`invalid JSON in ${rel}: ${e.message}`); }
}

// ---------------------------------------------------------------- historical masking
function historicalMask(text) {
  const lines = text.split(/\r?\n/);
  const mask = new Array(lines.length).fill(false);
  let inMarker = false;
  let histLevel = 0;
  for (let i = 0; i < lines.length; i += 1) {
    const line = lines[i];
    if (line.includes('<!-- docs-truth:historical:start -->')) { inMarker = true; mask[i] = true; continue; }
    if (line.includes('<!-- docs-truth:historical:end -->')) { inMarker = false; mask[i] = true; continue; }
    if (inMarker) { mask[i] = true; continue; }
    const heading = /^(#{1,6})\s+(.*)$/.exec(line);
    if (heading) {
      const level = heading[1].length;
      const title = heading[2];
      if (/historical|supersed|pre-1\.1|1\.0(\.0)?\s+(release|record|artifact)/i.test(title)) {
        histLevel = level;
        mask[i] = true;
        continue;
      }
      if (histLevel > 0 && level <= histLevel) histLevel = 0;
    }
    if (histLevel > 0) mask[i] = true;
  }
  return mask;
}
function checkLines(rel, rules) {
  const text = read(rel);
  const mask = historicalMask(text);
  const lines = text.split(/\r?\n/);
  for (let i = 0; i < lines.length; i += 1) {
    if (mask[i]) continue;
    for (const rule of rules) {
      if (rule.re.test(lines[i])) fail(rel, i + 1, rule.id, `${rule.message} :: ${lines[i].trim().slice(0, 160)}`);
    }
  }
  return { text, mask, lines };
}
function unmaskedText(text, mask) {
  return text.split(/\r?\n/).filter((_, i) => !mask[i]).join('\n');
}

// ---------------------------------------------------------------- VERSION
const pkg = readJson('web/package.json');
const currentVersion = pkg.version;
if (typeof currentVersion !== 'string' || !/^\d+\.\d+\.\d+/.test(currentVersion)) {
  fileError('web/package.json has no semver version');
}
// The sealed release record is the authority for release-facing docs/artifacts; the working tree may be a
// pre-release (e.g. 1.2.0-dev) only while the sealed version/source/tag remain documented and unchanged.
const releaseManifest = readJson('tools/release-manifest.json');
const sealedVersion = String(releaseManifest.version || '');
const sealedSource = String(releaseManifest.source_commit || '');
const sealedTag = String(releaseManifest.tag || '');
if (!/^\d+\.\d+\.\d+/.test(sealedVersion)) fileError('tools/release-manifest.json has no semver version');
if (!sealedSource || !sealedTag) fileError('tools/release-manifest.json must record source_commit and tag');
const escapeRe = (s) => s.replace(/[.*+?^${}()|[\]\\]/g, '\\$&');
const sourceVersionRe = new RegExp(`\\b${escapeRe(currentVersion)}\\b`);
const sealedVersionRe = new RegExp(`\\b${escapeRe(sealedVersion)}\\b`);
const preRelease = currentVersion !== sealedVersion;
const crateDir = path.join(root, 'backend', 'crates');
if (!existsSync(crateDir)) fileError('backend/crates missing');
const crateVersions = [];
for (const name of readdirSync(crateDir)) {
  const toml = path.join(crateDir, name, 'Cargo.toml');
  if (!existsSync(toml)) continue;
  const m = /^\s*version\s*=\s*"([^"]+)"/m.exec(readFileSync(toml, 'utf8'));
  if (m) crateVersions.push({ crate: name, version: m[1] });
}
const badCrates = crateVersions.filter((c) => c.version !== currentVersion);
check('VERSION: web and all backend crates agree', crateVersions.length > 0 && badCrates.length === 0,
  `web=${currentVersion} crates=${crateVersions.map((c) => `${c.crate}:${c.version}`).join(', ')}`);
if (badCrates.length > 0) {
  for (const c of badCrates) fail('web/package.json', 1, 'VERSION', `crate ${c.crate} version ${c.version} != web ${currentVersion}`);
}

const currentDocs = ['README.md', 'docs/RELEASE.md'];
for (const doc of currentDocs) {
  const { text, mask } = checkLines(doc, [
    { id: 'STREAMING', re: STALE_STREAMING_RE, message: 'retired one-delta/non-streaming claim in current-facing docs (SSE streaming is current)' },
    { id: 'HASHES', re: OLD_HASH_RE, message: 'superseded 1.0 artifact hash presented in current-facing docs (move to a historical section)' },
    { id: 'STATE', re: STALE_STATE_RE, message: 'retired 15-state claim in current-facing docs (current truth is 16 states)' },
    { id: 'VERSION', re: STALE_VERSION_RE, message: 'stale 1.0 release identity in current-facing docs' },
  ]);
  const live = unmaskedText(text, mask);
  // Release-facing docs describe the sealed release; the working tree version is documented in RELEASE-EVIDENCE.
  check(`VERSION: ${doc} states sealed release ${sealedVersion}`, sealedVersionRe.test(live), `missing sealed release "${sealedVersion}"`);
  if (!sealedVersionRe.test(live)) fail(doc, 1, 'VERSION', `release-facing doc must state the sealed release version ${sealedVersion}`);
  if (preRelease) {
    const claimsUnreleased = live.includes(currentVersion);
    check(`VERSION: ${doc} does not present unreleased ${currentVersion}`, !claimsUnreleased, `must not present the pre-release ${currentVersion} as the released version`);
    if (claimsUnreleased) fail(doc, 1, 'VERSION', `${doc} must not present the unreleased ${currentVersion} as the released version`);
  }
  check(`STREAMING: ${doc} claims real streaming`, STREAMING_POSITIVE_RE.test(live), 'missing positive SSE/streaming claim');
  if (!STREAMING_POSITIVE_RE.test(live)) fail(doc, 1, 'STREAMING', 'current-facing doc must state real SSE/token streaming');
}

// ---------------------------------------------------------------- RELEASE-EVIDENCE
{
  const { text, mask } = checkLines('docs/RELEASE-EVIDENCE.md', [
    { id: 'STREAMING', re: STALE_STREAMING_RE, message: 'retired one-delta/non-streaming claim outside historical section' },
    { id: 'HASHES', re: OLD_HASH_RE, message: 'superseded 1.0 artifact hash outside a historical section' },
    { id: 'STATE', re: STALE_STATE_RE, message: 'retired 15-state claim outside historical section' },
    { id: 'VERSION', re: STALE_VERSION_RE, message: 'stale 1.0 release identity outside historical section' },
  ]);
  const live = unmaskedText(text, mask);
  check(`VERSION: RELEASE-EVIDENCE states current source version ${currentVersion}`, sourceVersionRe.test(live), `missing current source version "${currentVersion}"`);
  if (!sourceVersionRe.test(live)) fail('docs/RELEASE-EVIDENCE.md', 1, 'VERSION', `current source section must state ${currentVersion}`);
  check(`VERSION: RELEASE-EVIDENCE documents sealed release ${sealedVersion}`, sealedVersionRe.test(live), `missing sealed release "${sealedVersion}"`);
  if (!sealedVersionRe.test(live)) fail('docs/RELEASE-EVIDENCE.md', 1, 'VERSION', `documentation must keep the sealed release record ${sealedVersion}`);
  const sealedHashEntries = [];
  for (const a of (Array.isArray(releaseManifest.artifacts) ? releaseManifest.artifacts : [])) {
    if (a && a.sha256) sealedHashEntries.push({ name: String(a.path || a.name || 'artifact'), sha256: String(a.sha256) });
  }
  const sealedPackage = releaseManifest.package && typeof releaseManifest.package === 'object' ? releaseManifest.package : {};
  if (sealedPackage.zip_sha256) sealedHashEntries.push({ name: String(sealedPackage.zip || 'package zip'), sha256: String(sealedPackage.zip_sha256) });
  const missingSealedHashes = sealedHashEntries.filter((e) => !text.includes(e.sha256));
  check('SEALED: RELEASE-EVIDENCE preserves sealed artifact hashes',
    sealedHashEntries.length > 0 && missingSealedHashes.length === 0,
    `sealed hashes checked=${sealedHashEntries.length} missing=${missingSealedHashes.length}`);
  if (sealedHashEntries.length === 0 || missingSealedHashes.length > 0) {
    fail('docs/RELEASE-EVIDENCE.md', 1, 'SEALED', `sealed artifact/package hash(es) must remain in release evidence (missing ${missingSealedHashes.map((e) => e.name).join(', ') || '(none parsed)'})`);
  }
  const sealedTagSourceDocumented = Boolean(sealedTag) && Boolean(sealedSource) && text.includes(sealedTag) && text.includes(sealedSource.slice(0, 8));
  check('SEALED: RELEASE-EVIDENCE records the sealed tag and source', sealedTagSourceDocumented, `sealed tag/source must be cited (${sealedTag} @ ${sealedSource.slice(0, 8)})`);
  if (!sealedTagSourceDocumented) {
    fail('docs/RELEASE-EVIDENCE.md', 1, 'SEALED', `release evidence must cite sealed tag ${sealedTag} and sealed source ${sealedSource.slice(0, 8)}`);
  }
  if (preRelease) {
    check('SEALED: RELEASE-EVIDENCE marks the working tree pre-release', /pre[- ]release/i.test(live), 'pre-release tree must be labelled in release evidence');
    if (!/pre[- ]release/i.test(live)) fail('docs/RELEASE-EVIDENCE.md', 1, 'SEALED', 'working tree must be labelled pre-release, not released');
  }
  check('HASHES: RELEASE-EVIDENCE points at tools/release-manifest.json', MANIFEST_POINTER_RE.test(live), 'missing authoritative-manifest pointer');
  if (!MANIFEST_POINTER_RE.test(live)) fail('docs/RELEASE-EVIDENCE.md', 1, 'HASHES', 'current section must point at tools/release-manifest.json as the exact-hash authority');
  check('HISTORY: RELEASE-EVIDENCE keeps a masked 1.0.0 historical record', mask.some(Boolean), 'no historical section found');
  if (!mask.some(Boolean)) fail('docs/RELEASE-EVIDENCE.md', 1, 'HISTORY', 'historical 1.0.0 record must be preserved under a historical section');
}

// ---------------------------------------------------------------- UI-COMMANDS (frozen contract doc)
{
  const rel = 'docs/contracts/UI-COMMANDS-v1.md';
  const doc = read(rel);
  const eventSrc = read('backend/crates/hiverouter-contract/src/event.rs');
  const expectedEvents = [...new Set(
    [...eventSrc.matchAll(/EventName::\w+\s*=>\s*"([A-Z][A-Z0-9_]*)"/g)].map((m) => m[1]),
  )];
  const eventsSection = (doc.match(/^## 3\. Events[^\n]*\n([\s\S]*?)(?=^## 4\.)/m) || ['', ''])[1];
  const listedEvents = [...new Set([...eventsSection.matchAll(/`([A-Z][A-Z0-9_]+)`/g)].map((m) => m[1]))];
  const missingEvents = expectedEvents.filter((e) => !listedEvents.includes(e));
  check('DOCS: UI-COMMANDS-v1 lists every contract event',
    expectedEvents.length > 0 && missingEvents.length === 0,
    `contract=${expectedEvents.length} listed=${listedEvents.length} missing=${missingEvents.join('|') || 'none'}`);
  if (expectedEvents.length === 0 || missingEvents.length > 0) {
    fail(rel, 1, 'DOCS-EVENTS', `UI-COMMANDS-v1 must list all ${expectedEvents.length} contract events; missing ${missingEvents.join(', ') || '(none parsed)'}`);
  }
  const stateSrc = read('web/src/state/agentState.ts');
  const expectedStates = [...new Set(
    [...stateSrc.matchAll(/^\s+'([a-z_]+)',$/gm)].map((m) => m[1]),
  )];
  const statesSection = (doc.match(/^## 4\. Single agent-state enum[^\n]*\n([\s\S]*?)(?=^## 5\.)/m) || ['', ''])[1];
  const stateListPart = statesSection.split(/The PDF's/)[0];
  const listedStates = [...new Set(
    stateListPart.replace(/`/g, '').split(/[\n,]/).map((s) => s.trim().replace(/[.,;:]+$/, '')).filter((s) => /^[a-z_]+$/.test(s)),
  )];
  const missingStates = expectedStates.filter((s) => !listedStates.includes(s));
  check('DOCS: UI-COMMANDS-v1 lists every agent state',
    expectedStates.length > 0 && missingStates.length === 0 && listedStates.length === expectedStates.length,
    `states=${expectedStates.length} listed=${listedStates.length} missing=${missingStates.join('|') || 'none'}`);
  if (expectedStates.length === 0 || missingStates.length > 0 || listedStates.length !== expectedStates.length) {
    fail(rel, 1, 'DOCS-STATES', `UI-COMMANDS-v1 must list exactly ${expectedStates.length} states; missing ${missingStates.join(', ') || '(none parsed)'} listed=${listedStates.length}`);
  }
}

// ---------------------------------------------------------------- claims (JSON)
{
  const rel = 'docs/claims/CLAIM-REGISTER.json';
  const registry = readJson(rel);
  const walk = (value, jsonPath) => {
    if (typeof value === 'string') {
      const historical = /historical|supersed/i.test(jsonPath) || /SUPERSEDED|historical/i.test(value);
      if (!historical) {
        if (OLD_HASH_RE.test(value)) fail(rel, 1, 'HASHES', `stale 1.0 hash in ${jsonPath}`);
        if (STALE_STREAMING_RE.test(value)) fail(rel, 1, 'STREAMING', `stale one-delta claim in ${jsonPath}`);
        if (STALE_STATE_RE.test(value)) fail(rel, 1, 'STATE', `retired 15-state claim in ${jsonPath}`);
      }
      return;
    }
    if (Array.isArray(value)) { value.forEach((v, i) => walk(v, `${jsonPath}[${i}]`)); return; }
    if (value && typeof value === 'object') {
      for (const [k, v] of Object.entries(value)) walk(v, `${jsonPath}.${k}`);
    }
  };
  walk(registry, '$');
  const notes = String(registry.notes || '');
  const notesMarked = /supersed|historical|1\.0\.0\s+(release|record|artifact)/i.test(notes);
  check(`CLAIMS: JSON notes state sealed release ${sealedVersion}`, notes.includes(sealedVersion), `notes must mention sealed release ${sealedVersion}`);
  if (!notes.includes(sealedVersion)) fail(rel, 1, 'VERSION', `registry notes must state sealed release ${sealedVersion}`);
  if (!notesMarked && OLD_HASH_RE.test(notes)) fail(rel, 1, 'HASHES', 'notes present 1.0 hashes without a superseded/historical marker');
  if (STALE_STREAMING_RE.test(notes) && !notesMarked) fail(rel, 1, 'STREAMING', 'notes present a stale one-delta claim');
  if (STALE_STATE_RE.test(notes) && !notesMarked) fail(rel, 1, 'STATE', 'notes present a retired 15-state claim');
}

// ---------------------------------------------------------------- claims (CSV)
{
  const rel = 'docs/claims/CLAIM-REGISTER.csv';
  const text = read(rel);
  const lines = text.split(/\r?\n/);
  const header = lines[0].split(',');
  for (let i = 0; i < lines.length; i += 1) {
    if (i === 0 || !lines[i].trim()) continue;
    const row = lines[i];
    const historicalRow = /supersed|historical|pre-1\.1|1\.0\.0\s+(release|record|artifact)/i.test(row);
    if (historicalRow) continue;
    if (OLD_HASH_RE.test(row)) fail(rel, i + 1, 'HASHES', 'superseded 1.0 hash in a current claim row (mark the row historical/superseded)');
    if (STALE_STREAMING_RE.test(row)) fail(rel, i + 1, 'STREAMING', 'retired one-delta claim in a current claim row');
    if (STALE_STATE_RE.test(row)) fail(rel, i + 1, 'STATE', 'retired 15-state claim in a current claim row');
  }
  check('CLAIMS: CSV header and rows parsed', header.length >= 8, `columns=${header.length}`);
  check(`CLAIMS: CSV records sealed release ${sealedVersion}`, text.includes(sealedVersion), `missing sealed release ${sealedVersion} in claims CSV`);
  if (!text.includes(sealedVersion)) fail(rel, 1, 'VERSION', `claims CSV must record the sealed release ${sealedVersion}`);
}

// ---------------------------------------------------------------- summary
if (!quiet) {
  console.log('');
  console.log(`root: ${root}`);
  console.log(`current source version: ${currentVersion}`);
  console.log(`sealed release version: ${sealedVersion}${preRelease ? ' (pre-release working tree)' : ''}`);
  console.log(`checks: ${checks.filter((c) => c.ok).length}/${checks.length} pass; violations: ${violations.length}`);
  for (const v of violations) console.log(`VIOLATION ${v.file}:${v.line} [${v.rule}] ${v.message}`);
}
if (violations.length > 0) {
  console.error(`DOCS TRUTH: FAIL (${violations.length} violation(s))`);
  process.exit(1);
}
console.log('DOCS TRUTH: PASS');
process.exit(0);
