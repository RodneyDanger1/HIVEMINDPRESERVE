/**
 * Seal-topology predicate tests (CODE-PROOF §6.3): exactly five node:test cases
 * that pin the source-C / seal-D allowlist model. These are pure, no filesystem
 * or git access; the CLI is not executed here.
 */
import test from 'node:test';
import assert from 'node:assert/strict';

import {
  SEAL_ALLOWLIST,
  SEALED_SCHEMA,
  evaluateSealTopology,
  normalizeRepoPath,
  sealPathAllowed,
} from '../check-seal-topology.mjs';

const SOURCE_C = 'a'.repeat(40);
const SEAL_D = 'b'.repeat(40);
const OTHER = 'c'.repeat(40);

function sealedManifest(overrides = {}) {
  return {
    schema: SEALED_SCHEMA,
    version: '1.2.0',
    tag: 'hivemind-1.2.0',
    tag_target: null,
    source_commit: SOURCE_C,
    sealed: true,
    seal_allowlist: [...SEAL_ALLOWLIST],
    seal_diff: {
      from: SOURCE_C,
      to: SEAL_D,
      sha256: 'd'.repeat(64),
      files: ['tools/release-manifest.json', 'dist/HiveMind-1.2.0-win64.zip'],
    },
    artifacts: [{ path: 'target/release/hiverouter.exe', sha256: 'e'.repeat(64), bytes: 10 }],
    ...overrides,
  };
}

function gitInfo(overrides = {}) {
  return {
    head: SEAL_D,
    tagTarget: SEAL_D,
    diffPaths: ['tools/release-manifest.json', 'dist/HiveMind-1.2.0-win64.zip'],
    artifactHashes: [
      { path: 'target/release/hiverouter.exe', expected: 'e'.repeat(64), actual: 'e'.repeat(64) },
    ],
    sourceAncestor: true,
    ...overrides,
  };
}

test('source-only manifest is not sealed', () => {
  const result = evaluateSealTopology(sealedManifest({ sealed: false }), gitInfo());
  assert.equal(result.ok, false);
  assert.equal(result.code, 'E_SEAL_UNSEALED');
});

test('sealed when HEAD == tag target D and diff is a subset of the allowlist', () => {
  const result = evaluateSealTopology(sealedManifest(), gitInfo());
  assert.deepEqual(result, { ok: true, code: null, source: SOURCE_C, seal: SEAL_D });
  // Allowlist matcher: exact paths and dist/** only; unsafe paths never match.
  assert.equal(sealPathAllowed('dist/HiveMind-1.2.0-win64.zip', SEAL_ALLOWLIST), true);
  assert.equal(sealPathAllowed('README.md', SEAL_ALLOWLIST), true);
  assert.equal(sealPathAllowed('../outside.txt', SEAL_ALLOWLIST), false);
  assert.equal(normalizeRepoPath('/abs.txt'), null);
});

test('fails when the seal diff contains a source file', () => {
  const result = evaluateSealTopology(
    sealedManifest(),
    gitInfo({
      diffPaths: [
        'tools/release-manifest.json',
        'backend/crates/hiverouter-core/src/execution.rs',
      ],
    }),
  );
  assert.equal(result.ok, false);
  assert.equal(result.code, 'E_SEAL_DIFF');
  assert.match(result.message, /execution\.rs/);
});

test('fails when HEAD is not the tag target', () => {
  const result = evaluateSealTopology(sealedManifest(), gitInfo({ head: OTHER, tagTarget: SEAL_D }));
  assert.equal(result.ok, false);
  assert.equal(result.code, 'E_SEAL_TAG');
});

test('fails when an artifact hash drifted', () => {
  const result = evaluateSealTopology(
    sealedManifest(),
    gitInfo({
      artifactHashes: [
        {
          path: 'target/release/hiverouter.exe',
          expected: 'e'.repeat(64),
          actual: 'f'.repeat(64),
        },
      ],
    }),
  );
  assert.equal(result.ok, false);
  assert.equal(result.code, 'E_SEAL_ARTIFACT');
});
