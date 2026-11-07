#!/usr/bin/env node
/**
 * Merge live trace captures into docs/trace/controls.json.
 *
 * W12 hardening (WP-08):
 * - An EMPTY raw capture (`surfaces: []`) is refused with `E_CAPTURE_EMPTY`: it
 *   cannot promote anything and must not silently no-op as a success.
 * - Any non-UNVERIFIED hop must carry at least one non-empty evidence pointer
 *   (`E_CAPTURE_EVIDENCE_MISSING`).
 * - A capture generated before the current HEAD commit is stale
 *   (`E_CAPTURE_STALE`) and may not promote hops for a newer tree.
 *
 * Only hops that were actually captured are applied; everything else stays
 * PENDING/UNVERIFIED. No fabricated pass.
 *
 * `validateCapture()` is exported as a pure function (no I/O) so it can be
 * unit-tested without touching the repository; the CLI runs only when this
 * file is executed directly.
 */
import { existsSync, readFileSync, writeFileSync } from 'node:fs';
import { fileURLToPath, pathToFileURL } from 'node:url';
import path from 'node:path';
import { execFileSync } from 'node:child_process';

const ROOT = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..', '..');
export const CAPTURE = path.join(ROOT, 'docs', 'trace', 'live', 'trace-capture.json');
export const CONTROLS = path.join(ROOT, 'docs', 'trace', 'controls.json');

/**
 * Validate a raw capture document before it may promote any hop.
 *
 * Pure: deterministic over `capture` + `options`, never reads files or git.
 *
 * @param {unknown} capture parsed `docs/trace/live/trace-capture.json`
 * @param {{ headCommittedAt?: string | null }} [options] HEAD commit time
 *   (RFC3339). When supplied, a capture generated before it is refused as stale.
 * @returns {{ ok: boolean, code: string | null, errors: string[] }}
 */
export function validateCapture(capture, options = {}) {
  const errors = [];
  const fail = (code, message) => ({ ok: false, code, errors: [message, ...errors] });

  if (capture === null || typeof capture !== 'object') {
    return fail('E_CAPTURE_INVALID', 'capture is not a JSON object');
  }
  if (!Array.isArray(capture.surfaces)) {
    return fail('E_CAPTURE_INVALID', 'capture.surfaces is not an array');
  }
  if (capture.surfaces.length === 0) {
    return fail(
      'E_CAPTURE_EMPTY',
      'capture.surfaces is empty: an empty raw capture cannot promote any hop (refusing to apply as success)',
    );
  }

  const generatedAt = Date.parse(String(capture.generated_at ?? ''));
  let stale = false;
  if (!Number.isFinite(generatedAt)) {
    errors.push('capture.generated_at is missing or not a parseable timestamp');
  } else if (options.headCommittedAt) {
    const headAt = Date.parse(String(options.headCommittedAt));
    if (!Number.isFinite(headAt)) {
      errors.push('options.headCommittedAt is not a parseable timestamp');
    } else if (generatedAt < headAt) {
      stale = true;
      errors.push(
        `capture is stale: generated_at ${capture.generated_at} is older than HEAD commit ${options.headCommittedAt}`,
      );
    }
  }

  const seen = new Set();
  for (const [index, surface] of capture.surfaces.entries()) {
    if (surface === null || typeof surface !== 'object') {
      errors.push(`surfaces[${index}] is not an object`);
      continue;
    }
    const surfaceId = typeof surface.surface_id === 'string' ? surface.surface_id.trim() : '';
    if (surfaceId.length === 0) {
      errors.push(`surfaces[${index}] has no surface_id`);
    } else if (seen.has(surfaceId)) {
      errors.push(`surfaces[${index}] duplicates surface_id ${surfaceId}`);
    } else {
      seen.add(surfaceId);
    }
    if (surface.hops === null || typeof surface.hops !== 'object') {
      errors.push(`surfaces[${index}] (${surfaceId || '?'}) has no hops object`);
      continue;
    }
    for (const [hop, slot] of Object.entries(surface.hops)) {
      const verdict = slot && typeof slot === 'object' ? slot.verdict : undefined;
      if (typeof verdict !== 'string' || verdict.length === 0) {
        errors.push(`${surfaceId || `surfaces[${index}]`}/${hop} has no verdict`);
        continue;
      }
      if (verdict === 'UNVERIFIED' || verdict === 'PENDING') {
        continue;
      }
      const evidence = slot && typeof slot === 'object' ? slot.evidence : undefined;
      if (!Array.isArray(evidence) || evidence.length === 0) {
        errors.push(
          `${surfaceId || `surfaces[${index}]`}/${hop} is ${verdict} without evidence (verbatim capture evidence is required)`,
        );
        continue;
      }
      for (const [evidenceIndex, item] of evidence.entries()) {
        const pointer = item && typeof item === 'object' ? item.pointer : undefined;
        if (typeof pointer !== 'string' || pointer.trim().length === 0) {
          errors.push(
            `${surfaceId || `surfaces[${index}]`}/${hop} evidence[${evidenceIndex}] has no pointer`,
          );
        }
      }
    }
  }

  if (errors.length > 0) {
    return {
      ok: false,
      code: stale ? 'E_CAPTURE_STALE' : 'E_CAPTURE_EVIDENCE_MISSING',
      errors,
    };
  }
  return { ok: true, code: null, errors: [] };
}

/** HEAD commit time (RFC3339) used for the stale-capture guard. */
function headCommittedAt() {
  try {
    return execFileSync('git', ['log', '-1', '--format=%cI'], {
      cwd: ROOT,
      encoding: 'utf8',
      stdio: ['ignore', 'pipe', 'ignore'],
    }).trim();
  } catch {
    return null;
  }
}

function main() {
  if (!existsSync(CAPTURE)) {
    console.error('missing capture file: docs/trace/live/trace-capture.json (run the trace-hops spec first)');
    process.exit(1);
  }
  const capture = JSON.parse(readFileSync(CAPTURE, 'utf8'));
  const validation = validateCapture(capture, { headCommittedAt: headCommittedAt() });
  if (!validation.ok) {
    console.error(`${validation.code ?? 'E_CAPTURE_INVALID'}: ${validation.errors.join('; ')}`);
    process.exit(1);
  }

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
}

if (process.argv[1] && pathToFileURL(process.argv[1]).href === import.meta.url) {
  main();
}
