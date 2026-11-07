// W12 / WP-08 trace-tool guard tests (source-only; run with `node --test`).
//
// These tests do not touch the network, Playwright, the live server or the paid
// provider path. They import the two pure guards:
//   - `validateCapture` from tools/qa/apply-live-trace.mjs (empty/stale capture)
//   - `unresolvedEvidencePointers` from tools/qa/verify-harness.mjs (pointer exists)
// and prove each rejects what it must reject and accepts what it must accept.
import test from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { fileURLToPath } from 'node:url';
import path from 'node:path';

import { validateCapture } from '../apply-live-trace.mjs';
import {
  evidencePathCandidates,
  unresolvedEvidencePointers,
} from '../verify-harness.mjs';

const ROOT = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..', '..', '..');
const CONTROLS = path.join(ROOT, 'docs', 'trace', 'controls.json');

/** A capture that satisfies every structural rule (paths below all exist). */
function fullCapture() {
  return {
    schema: 'hiverouter.trace.live-capture/v1',
    generated_at: '2026-11-07T00:00:00.000Z',
    base_url: 'http://127.0.0.1:8799',
    rule: 'Only captured evidence may be WIRED-AND-WORKS; no fabricated pass.',
    surfaces: [
      {
        surface_id: 'S10',
        present: true,
        evidence: 'history revision replay frames captured',
        hops: {
          '2_handler': {
            verdict: 'WIRED-AND-WORKS',
            evidence: [{ kind: 'dom', pointer: 'docs/trace/controls.json#S10', quote: 'HISTORY_REVISION_SET handled' }],
          },
          '10_render_back': {
            verdict: 'WIRED-AND-WORKS',
            evidence: [
              {
                kind: 'ws-frame',
                pointer: 'docs/trace/live/captures/w11/live-streaming.json#HISTORY_SNAPSHOT',
                quote: 'snapshot rendered read-only',
              },
            ],
          },
        },
      },
      {
        surface_id: 'S22',
        present: true,
        evidence: 'file tree roundtrip captured',
        hops: {
          '6_service': {
            verdict: 'WIRED-AND-WORKS',
            evidence: [
              { kind: 'ws-frame', pointer: 'docs/trace/live/live-app/14-file-tree.json', quote: 'FILE_TREE_RESULT rendered' },
            ],
          },
        },
      },
    ],
  };
}

test('rejects empty capture with E_CAPTURE_EMPTY', () => {
  const empty = {
    schema: 'hiverouter.trace.live-capture/v1',
    generated_at: '2026-11-07T00:00:00.000Z',
    base_url: 'http://127.0.0.1:8799',
    surfaces: [],
  };
  const result = validateCapture(empty);
  assert.equal(result.ok, false, 'empty capture must be refused');
  assert.equal(result.code, 'E_CAPTURE_EMPTY');
  assert.match(result.errors.join(' '), /surfaces is empty/i);
  assert.equal(result.ok, false, 'no success no-op is allowed for an empty capture');
});

test('rejects unresolved evidence pointer (and real tree pointers resolve)', () => {
  // Unresolved pointer on a non-UNVERIFIED hop => the pointer guard reports it.
  const controls = {
    controls: [
      {
        surface_id: 'S-TEST',
        hops: {
          '6_service': {
            verdict: 'WIRED-AND-WORKS',
            evidence: [
              { kind: 'log', pointer: 'docs/trace/live/captures/w12/CAP-DOES-NOT-EXIST/manifest.json#hop' },
            ],
          },
        },
      },
    ],
  };
  const unresolved = unresolvedEvidencePointers(controls, ROOT);
  assert.equal(unresolved.length, 1);
  assert.match(unresolved[0].reason, /missing/);
  assert.equal(unresolved[0].surface, 'S-TEST');
  assert.equal(unresolved[0].hop, '6_service');

  // Path extraction keeps fragments out and tolerates multi-file/line pointers.
  assert.deepEqual(evidencePathCandidates('docs/trace/controls.json#S10'), ['docs/trace/controls.json']);
  assert.deepEqual(evidencePathCandidates('web/src/flow/Canvas.tsx:114-115; web/src/flow/graph.ts:96'), [
    'web/src/flow/Canvas.tsx',
    'web/src/flow/graph.ts',
  ]);

  // The current repository controls must resolve (otherwise the gate is lying).
  const realControls = JSON.parse(readFileSync(CONTROLS, 'utf8'));
  assert.deepEqual(unresolvedEvidencePointers(realControls, ROOT), []);
});

test('accepts a full capture (and refuses a stale one)', () => {
  const capture = fullCapture();
  const accepted = validateCapture(capture);
  assert.equal(accepted.ok, true, `full capture must pass: ${accepted.errors.join('; ')}`);
  assert.equal(accepted.code, null);

  // Stale capture: generated before the HEAD commit it would promote for.
  const stale = validateCapture(capture, { headCommittedAt: '2026-11-08T00:00:00.000Z' });
  assert.equal(stale.ok, false);
  assert.equal(stale.code, 'E_CAPTURE_STALE');
  assert.match(stale.errors.join(' '), /stale/);

  // Non-UNVERIFIED hop without evidence is refused, never promoted.
  const evidenceLess = fullCapture();
  evidenceLess.surfaces[0].hops['10_render_back'] = { verdict: 'WIRED-AND-WORKS', evidence: [] };
  const refused = validateCapture(evidenceLess);
  assert.equal(refused.ok, false);
  assert.equal(refused.code, 'E_CAPTURE_EVIDENCE_MISSING');
});
