#!/usr/bin/env node
/**
 * Pure seal-topology checker for the HiveMind 1.2 release model (CODE-PROOF §6).
 *
 * Topology: the release tag points at seal commit D, and D is a child of source
 * commit C. Only allowlisted evidence/artifact paths may differ between C and D
 * (`tools/release-manifest.json`, `dist/**`, release docs/README). The manifest
 * records `sealed: true`, `source_commit: C`, `seal_diff: {from: C, to: D,
 * sha256}`, `seal_allowlist`, and the artifact/web hashes.
 *
 * `evaluateSealTopology(manifest, gitInfo)` is pure (no I/O) so the same
 * predicate can be unit-tested and used by the gate. The CLI collects git +
 * disk facts and is read-only: it never writes, builds, tags or touches the
 * network.
 *
 * Exit codes: 0 PASS | 1 seal refusal (`E_SEAL_*`) | 2 setup/git/manifest error.
 */

import { createHash } from 'node:crypto';
import { execFileSync } from 'node:child_process';
import { existsSync, readFileSync } from 'node:fs';
import { dirname, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';

/** Manifest schema that carries the seal fields. */
export const SEALED_SCHEMA = 'hivemind.release-manifest.v2';

/** Exact allowlist from CODE-PROOF §6 (source-C..seal-D may only touch these). */
export const SEAL_ALLOWLIST = Object.freeze([
  'tools/release-manifest.json',
  'dist/**',
  'docs/RELEASE-EVIDENCE.md',
  'plans/waves/README.md',
  'docs/RELEASE.md',
  'README.md',
]);

/** Named refusals emitted by the checker. */
export const SEAL_CODES = Object.freeze({
  unsealed: 'E_SEAL_UNSEALED',
  tag: 'E_SEAL_TAG',
  diff: 'E_SEAL_DIFF',
  artifact: 'E_SEAL_ARTIFACT',
  manifest: 'E_SEAL_MANIFEST',
  determinism: 'E_SEAL_DETERMINISM',
  git: 'E_SEAL_GIT',
});

const COMMIT_RE = /^[0-9a-f]{40}$/i;

function fail(code, message) {
  return { ok: false, code, message };
}

function shortCommit(value) {
  return typeof value === 'string' && value.length >= 8 ? value.slice(0, 8) : String(value);
}

/** Normalize a repo-relative path; returns null when it is unsafe. */
export function normalizeRepoPath(value) {
  if (typeof value !== 'string') {
    return null;
  }
  let path = value.replace(/\\/g, '/').trim();
  while (path.startsWith('./')) {
    path = path.slice(2);
  }
  if (path.length === 0 || path.startsWith('/') || /^[A-Za-z]:/.test(path)) {
    return null;
  }
  const segments = path.split('/');
  if (segments.some((segment) => segment === '..')) {
    return null;
  }
  return path;
}

/**
 * Exact/`dir/**` allowlist match. Unsafe paths never match (fail closed).
 */
export function sealPathAllowed(value, allowlist = SEAL_ALLOWLIST) {
  const path = normalizeRepoPath(value);
  if (path === null) {
    return false;
  }
  for (const rawEntry of allowlist) {
    if (typeof rawEntry !== 'string') {
      continue;
    }
    const entry = rawEntry.replace(/\\/g, '/').trim();
    if (entry.length === 0) {
      continue;
    }
    if (entry.endsWith('/**')) {
      const prefix = entry.slice(0, -3).replace(/\/+$/, '');
      if (path === prefix || path.startsWith(`${prefix}/`)) {
        return true;
      }
    } else if (path === entry) {
      return true;
    }
  }
  return false;
}

function validCommit(value) {
  return typeof value === 'string' && COMMIT_RE.test(value);
}

/**
 * Pure topology predicate.
 *
 * @param {object|null} manifest parsed tools/release-manifest.json
 * @param {object|null} gitInfo
 *   `{ head, tagTarget, diffPaths, artifactHashes, sourceAncestor }` where
 *   `artifactHashes` is `{ path, expected, actual }[]` (expected/actual sha256
 *   strings; null actual means the file is missing) and `sourceAncestor` is true
 *   only when source commit C is an ancestor of HEAD.
 * @returns {{ok:true, code:null, source:string, seal:string} |
 *           {ok:false, code:string, message:string}}
 */
export function evaluateSealTopology(manifest, gitInfo) {
  if (!manifest || typeof manifest !== 'object') {
    return fail(SEAL_CODES.manifest, 'release manifest is missing or not an object');
  }
  if (manifest.sealed !== true) {
    return fail(
      SEAL_CODES.unsealed,
      'manifest is not sealed (sealed=false); a source-only build cannot be a release',
    );
  }
  if (manifest.schema !== SEALED_SCHEMA) {
    return fail(
      SEAL_CODES.manifest,
      `sealed manifest schema must be ${SEALED_SCHEMA}, got ${String(manifest.schema)}`,
    );
  }
  if (!validCommit(manifest.source_commit)) {
    return fail(SEAL_CODES.manifest, 'manifest.source_commit must be a full 40-hex commit id');
  }
  const sealDiff = manifest.seal_diff;
  if (!sealDiff || typeof sealDiff !== 'object') {
    return fail(SEAL_CODES.manifest, 'manifest.seal_diff is missing');
  }
  if (!validCommit(sealDiff.from) || !validCommit(sealDiff.to)) {
    return fail(SEAL_CODES.manifest, 'manifest.seal_diff.from/to must be full 40-hex commit ids');
  }
  if (sealDiff.from.toLowerCase() !== manifest.source_commit.toLowerCase()) {
    return fail(
      SEAL_CODES.diff,
      `seal_diff.from ${shortCommit(sealDiff.from)} != manifest.source_commit ${shortCommit(manifest.source_commit)}`,
    );
  }
  if (!Array.isArray(manifest.seal_allowlist) || manifest.seal_allowlist.length === 0) {
    return fail(SEAL_CODES.manifest, 'manifest.seal_allowlist must be a non-empty array');
  }
  const canonical = [...SEAL_ALLOWLIST].sort().join('|');
  const recorded = [...manifest.seal_allowlist].map((entry) => String(entry)).sort().join('|');
  if (recorded !== canonical) {
    return fail(
      SEAL_CODES.manifest,
      'manifest.seal_allowlist differs from the fixed 1.2 seal allowlist; a widened allowlist cannot certify a seal',
    );
  }
  const allowlist = SEAL_ALLOWLIST;

  if (!gitInfo || typeof gitInfo !== 'object') {
    return fail(SEAL_CODES.manifest, 'gitInfo is missing');
  }
  const head = typeof gitInfo.head === 'string' ? gitInfo.head.toLowerCase() : '';
  const tagTarget = typeof gitInfo.tagTarget === 'string' ? gitInfo.tagTarget.toLowerCase() : '';

  if (!validCommit(head)) {
    return fail(SEAL_CODES.git, 'git HEAD is unknown');
  }
  if (gitInfo.sourceAncestor !== true) {
    return fail(SEAL_CODES.diff, 'HEAD is not a descendant of manifest.source_commit C; the seal commit must descend from the source commit');
  }
  if (!validCommit(tagTarget)) {
    return fail(SEAL_CODES.tag, `manifest tag '${String(manifest.tag)}' does not resolve to a commit`);
  }
  if (head !== tagTarget) {
    return fail(
      SEAL_CODES.tag,
      `HEAD ${shortCommit(head)} != tag target ${shortCommit(tagTarget)}; the release tag must point at the seal commit`,
    );
  }
  // `seal_diff.to` is the seal commit D passed to -RecordSeal *before* the
  // manifest-inclusion amend (D'), so it is not compared to HEAD here. The gate
  // predicate is exactly CODE-PROOF §6.4: HEAD == tag target, and the diff from
  // source commit C to HEAD is a subset of the allowlist.
  if (!Array.isArray(gitInfo.diffPaths)) {
    return fail(SEAL_CODES.diff, 'git diff between source commit C and HEAD is unavailable');
  }
  const offenders = [];
  for (const entry of gitInfo.diffPaths) {
    const path = normalizeRepoPath(entry);
    if (path === null || !sealPathAllowed(path, allowlist)) {
      offenders.push(typeof entry === 'string' ? entry : String(entry));
    }
  }
  if (offenders.length > 0) {
    return fail(
      SEAL_CODES.diff,
      `seal diff contains non-allowlisted path(s): ${offenders.slice(0, 10).join(', ')}`,
    );
  }
  if (!Array.isArray(gitInfo.artifactHashes) || gitInfo.artifactHashes.length === 0) {
    return fail(SEAL_CODES.artifact, 'manifest artifact/web hashes are unavailable for verification');
  }
  for (const row of gitInfo.artifactHashes) {
    const path = row && typeof row.path === 'string' ? row.path : '?';
    const expected = row && typeof row.expected === 'string' ? row.expected.toLowerCase() : null;
    const actual = row && typeof row.actual === 'string' ? row.actual.toLowerCase() : null;
    if (expected === null) {
      return fail(SEAL_CODES.artifact, `manifest has no recorded sha256 for ${path}`);
    }
    if (actual === null) {
      return fail(SEAL_CODES.artifact, `artifact is missing on disk: ${path}`);
    }
    if (actual !== expected) {
      return fail(
        SEAL_CODES.artifact,
        `artifact hash drifted: ${path} expected ${expected} actual ${actual}`,
      );
    }
  }
  return {
    ok: true,
    code: null,
    source: manifest.source_commit.toLowerCase(),
    // The tagged/checked-out seal commit (D'); manifest.seal_diff.to records the
    // pre-amend D that -RecordSeal was given, before the manifest-inclusion amend.
    seal: tagTarget,
  };
}

function sha256File(path) {
  const hash = createHash('sha256');
  hash.update(readFileSync(path));
  return hash.digest('hex');
}

function git(repo, args) {
  return execFileSync('git', ['-C', repo, ...args], {
    encoding: 'utf8',
    stdio: ['ignore', 'pipe', 'pipe'],
  }).trim();
}

function artifactRows(manifest) {
  const rows = [];
  const expectedFor = (value) => (typeof value === 'string' && value.length > 0 ? value : null);
  if (Array.isArray(manifest.artifacts)) {
    for (const artifact of manifest.artifacts) {
      if (artifact && typeof artifact.path === 'string') {
        rows.push({ path: artifact.path, expected: expectedFor(artifact.sha256) });
      }
    }
  }
  if (manifest.package && typeof manifest.package === 'object') {
    rows.push({ path: manifest.package.zip, expected: expectedFor(manifest.package.zip_sha256) });
    if (manifest.package.sidecar) {
      rows.push({
        path: manifest.package.sidecar,
        expected: expectedFor(manifest.package.sidecar_sha256),
      });
    }
    if (manifest.package.checksums) {
      rows.push({
        path: manifest.package.checksums,
        expected: expectedFor(manifest.package.checksums_sha256),
      });
    }
  }
  if (manifest.web && Array.isArray(manifest.web.files)) {
    for (const file of manifest.web.files) {
      if (file && typeof file.path === 'string') {
        rows.push({ path: file.path, expected: expectedFor(file.sha256) });
      }
    }
  }
  return rows.filter((row) => typeof row.path === 'string' && row.path.length > 0);
}

/** Read-only collection of the git/disk facts the pure predicate needs. */
export function collectGitInfo(repo, manifest) {
  const head = git(repo, ['rev-parse', 'HEAD']);
  let tagTarget = null;
  if (manifest && typeof manifest.tag === 'string' && manifest.tag.length > 0) {
    try {
      tagTarget = git(repo, ['rev-list', '-n1', manifest.tag]);
    } catch {
      tagTarget = null;
    }
  }
  let diffPaths = null;
  let sourceAncestor = false;
  if (manifest && validCommit(manifest.source_commit)) {
    const range = `${manifest.source_commit}..${head}`;
    try {
      git(repo, ['merge-base', '--is-ancestor', manifest.source_commit, head]);
      sourceAncestor = true;
    } catch {
      sourceAncestor = false;
    }
    try {
      diffPaths = git(repo, ['diff', '--name-only', range])
        .split('\n')
        .map((line) => line.trim())
        .filter((line) => line.length > 0);
    } catch {
      diffPaths = null;
    }
  }
  const artifactHashes = [];
  for (const row of artifactRows(manifest || {})) {
    const full = resolve(repo, row.path);
    let actual = null;
    try {
      actual = existsSync(full) ? sha256File(full) : null;
    } catch {
      actual = null;
    }
    artifactHashes.push({ path: row.path, expected: row.expected, actual });
  }
  return { head, tagTarget, diffPaths, artifactHashes, sourceAncestor };
}

/** Default repo root: two levels above tools/qa/. */
export function defaultRepoRoot() {
  return resolve(dirname(fileURLToPath(import.meta.url)), '..', '..');
}

function parseArgs(argv) {
  const options = {
    repo: defaultRepoRoot(),
    manifest: null,
    json: false,
  };
  for (let index = 0; index < argv.length; index += 1) {
    const argument = argv[index];
    if (argument === '--json') {
      options.json = true;
    } else if (argument === '--repo' && index + 1 < argv.length) {
      index += 1;
      options.repo = resolve(argv[index]);
    } else if (argument === '--manifest' && index + 1 < argv.length) {
      index += 1;
      options.manifest = resolve(argv[index]);
    } else if (argument === '--help' || argument === '-h') {
      options.help = true;
    } else {
      throw new Error(`unknown argument: ${argument}`);
    }
  }
  return options;
}

function main() {
  let options;
  try {
    options = parseArgs(process.argv.slice(2));
  } catch (error) {
    console.error(`[E_SEAL_MANIFEST] ${error.message}`);
    process.exitCode = 2;
    return;
  }
  if (options.help) {
    console.log(
      'usage: node tools/qa/check-seal-topology.mjs [--repo <dir>] [--manifest <file>] [--json]',
    );
    return;
  }
  const manifestPath = options.manifest || resolve(options.repo, 'tools', 'release-manifest.json');
  let manifest;
  try {
    manifest = JSON.parse(readFileSync(manifestPath, 'utf8'));
  } catch (error) {
    const result = {
      ok: false,
      code: SEAL_CODES.manifest,
      message: `cannot read manifest ${manifestPath}: ${error.message}`,
    };
    printResult(result, options.json);
    process.exitCode = 2;
    return;
  }
  let gitInfo;
  try {
    gitInfo = collectGitInfo(options.repo, manifest);
    // The recorded pre-amend seal commit D must exist; a missing object makes the
    // manifest unverifiable even before the pure predicate runs.
    const sealTo = manifest && manifest.seal_diff ? manifest.seal_diff.to : null;
    if (typeof sealTo === 'string' && sealTo.length > 0) {
      execFileSync('git', ['-C', options.repo, 'cat-file', '-e', `${sealTo}^{commit}`], {
        stdio: ['ignore', 'ignore', 'ignore'],
      });
    }
  } catch (error) {
    const result = {
      ok: false,
      code: SEAL_CODES.manifest,
      message: `seal commit or git state is unreadable: ${error.message}`,
    };
    printResult(result, options.json);
    process.exitCode = 2;
    return;
  }
  const result = evaluateSealTopology(manifest, gitInfo);
  printResult(result, options.json);
  if (!result.ok) {
    process.exitCode = 1;
  }
}

function printResult(result, asJson) {
  if (asJson) {
    console.log(JSON.stringify(result));
    return;
  }
  if (result.ok) {
    console.log(`PASS sealed source=${result.source} seal=${result.seal}`);
  } else {
    console.error(`[${result.code}] ${result.message}`);
  }
}

const invokedAsMain =
  process.argv[1] !== undefined &&
  resolve(process.argv[1]) === resolve(fileURLToPath(import.meta.url));

if (invokedAsMain) {
  main();
}
