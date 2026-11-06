import { test, expect } from '@playwright/test';
import { loadMatrix, skipReason } from '../helpers/matrix';

const matrix = loadMatrix();

test.describe('control matrix — integrity (static; no app required)', () => {
  test('28 guide surfaces S01-S28 (+ explicit supplemental controls), unique ids', () => {
    const guide = matrix.surfaces.filter((s) => /^S\d{2}$/.test(s.id));
    expect(guide).toHaveLength(28);
    const ids = guide.map((s) => s.id);
    expect(new Set(ids).size).toBe(28);
    for (let i = 1; i <= 28; i += 1) {
      expect(ids).toContain(`S${String(i).padStart(2, '0')}`);
    }
    const supplemental = matrix.surfaces.filter((s) => !/^S\d{2}$/.test(s.id));
    expect(supplemental.map((s) => s.id)).toEqual(matrix._meta.supplemental_controls ?? []);
    for (const surface of matrix.surfaces) {
      expect(surface.guide_ref, `${surface.id} guide_ref`).toMatch(/UI-WIRING-GUIDE\.md:\d+|UI-COMMANDS-v1\.md:\d+/);
      expect(surface.owner.length).toBeGreaterThan(0);
      expect(surface.acceptance.length).toBeGreaterThan(0);
    }
  });

  test('11-hop vocabulary is exact and in order', () => {
    expect(matrix.hop_names).toEqual([
      '1_render',
      '2_handler',
      '3_client_call',
      '4_request',
      '5_route',
      '6_service',
      '7_store',
      '8_provider',
      '9_response',
      '10_render_back',
      '11_reentry',
    ]);
    expect(matrix.hop_verdict_vocabulary.sort()).toEqual(
      ['MISSING', 'REFUSED-BY-NAME', 'UNVERIFIED', 'WIRED-AND-WORKS', 'WIRED-BUT-DEAD'].sort(),
    );
  });

  test('frozen command/refusal vocabulary matches docs/contracts/UI-COMMANDS-v1.md', async () => {
    // The static validator (tools/qa/verify-harness.mjs) cross-checks every command and refusal
    // against the frozen markdown contract and runs in CI without Playwright. This test re-asserts
    // the minimum shape so a run never starts against a drifted matrix.
    const { FROZEN_COMMANDS, NOT_FROZEN_COMMANDS } = await import('../helpers/matrix');
    const commandNames = matrix.surfaces
      .map((s) => s.command)
      .filter((name): name is string => typeof name === 'string');
    for (const name of commandNames) {
      expect(FROZEN_COMMANDS as readonly string[], `${name} not frozen`).toContain(name);
    }
    for (const surface of matrix.surfaces) {
      if (surface.command && (NOT_FROZEN_COMMANDS as readonly string[]).includes(surface.command)) {
        expect(surface.status, `${surface.id} decision-gated command must not claim readiness`).toBe(
          'DECISION_GATED',
        );
      }
    }
  });

  test('no row claims a pass before evidence exists', () => {
    const allowed = new Set(matrix._meta.status_values as string[]);
    for (const surface of matrix.surfaces) {
      expect(allowed.has(surface.status), `${surface.id} status=${surface.status}`).toBe(true);
      expect(['PASS', 'WIRED-AND-WORKS']).not.toContain(surface.status);
    }
  });
});

test.describe('28-surface control probes — app required (SKIP until web app exists)', () => {
  for (const surface of matrix.surfaces) {
    test(`${surface.id} ${surface.surface}`, async ({ page, request }) => {
      test.skip(skipReason());
      // Probe outline (only runs once the app exists; evidence must be captured, never assumed):
      // 1. Navigate to the surface (selector/route PENDING — DOM not built).
      // 2. Capture the 11 hops from e2e/helpers/trace.ts.
      // 3. For mutating controls: tap WS frames before clicking; assert ACK or typed ERROR
      //    arrives BEFORE any optimistic UI mutation, and that a refused command leaves
      //    state unchanged (no fake success).
      // 4. Record request/response, WS frames, DB row (via /health metrics where exposed),
      //    and render-back state into docs/trace/traces.json.
      void page;
      void request;
      void surface;
    });
  }
});
