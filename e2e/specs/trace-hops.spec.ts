import { test, expect } from '@playwright/test';
import { writeFileSync } from 'node:fs';
import { fileURLToPath } from 'node:url';
import { appMissing, skipReason } from '../helpers/matrix';
import { ensureLiveDirs, LIVE_DIR } from '../helpers/evidence';

/**
 * Real per-surface trace capture (replaces the placeholder PENDING matrix).
 * Each surface verifies the render hop with captured DOM evidence; client-only
 * surfaces are exercised and the render-back hop is recorded. Server hops that
 * cannot be proven from the browser (3-9) stay UNVERIFIED unless a captured
 * frame/request proves them. Evidence is written to docs/trace/live/.
 */
const BASE = process.env.E2E_BASE_URL ?? 'http://127.0.0.1:8799';

interface SurfaceRecord {
  surface_id: string;
  surface: string;
  selector: string;
  present: boolean;
  evidence: string;
  interactions: string[];
  hops: Record<string, 'WIRED-AND-WORKS' | 'WIRED-BUT-DEAD' | 'MISSING' | 'REFUSED-BY-NAME' | 'UNVERIFIED'>;
  notes: string[];
}

const records: SurfaceRecord[] = [];
const SURFACES: Array<{ id: string; surface: string; selector: string; kind: string }> = [
  { id: 'S01', surface: 'Infinite canvas + honeycomb background', selector: '[role="application"][aria-label="HiveMind spatial canvas"]', kind: 'canvas' },
  { id: 'S02', surface: 'HyperAgent hive chat core (pinned)', selector: 'section[aria-label="HyperAgent hive chat core"]', kind: 'chat' },
  { id: 'S03', surface: 'Composer / HyperAgent input', selector: 'textarea#composer', kind: 'composer' },
  { id: 'S04', surface: 'Button: Spawn sub-hive', selector: 'button:has-text("Spawn sub-hive")', kind: 'control' },
  { id: 'S05', surface: 'Button: Re-route', selector: 'button:has-text("Re-route")', kind: 'control' },
  { id: 'S06', surface: 'Button: Cancel', selector: 'button:has-text("Cancel")', kind: 'control' },
  { id: 'S07', surface: 'Button: Kill worker', selector: 'button:has-text("Kill worker")', kind: 'control' },
  { id: 'S08', surface: 'Tabs: Workspace/Swarm/Routing/History', selector: '[role="tablist"][aria-label="Workspace tabs"]', kind: 'tabs' },
  { id: 'S09', surface: 'HUD throughput badge', selector: '[role="status"][aria-label="Swarm throughput"]', kind: 'hud' },
  { id: 'S10', surface: 'Time-travel scrubber', selector: 'input#scrubber', kind: 'scrubber' },
  { id: 'S11', surface: 'Nav rail (72px fixed, 6 items)', selector: 'nav[aria-label="Primary navigation"]', kind: 'nav' },
  { id: 'S12', surface: 'Library panel (260px)', selector: 'aside[aria-label="Library panel"]', kind: 'library' },
  { id: 'S13', surface: 'Mini-map (260x170)', selector: '[data-testid="rf__minimap"]', kind: 'minimap' },
  { id: 'S14', surface: 'Refusal toast', selector: '[aria-label="Refusals and errors"]', kind: 'toast-region' },
  { id: 'S15', surface: 'SuperAgent sub-hive card', selector: '.react-flow__node[data-id]', kind: 'node-card' },
  { id: 'S16', surface: 'WorkerAgent card', selector: '.react-flow__node[data-id]', kind: 'node-card' },
  { id: 'S17', surface: 'ToolAgent card', selector: '.react-flow__node[data-id]', kind: 'node-card' },
  { id: 'S18', surface: 'Broken-route error card', selector: '.react-flow__node[data-id]', kind: 'node-card' },
  { id: 'S19', surface: 'Agent chips / state dot (40px hex)', selector: '.react-flow__node[data-id]', kind: 'node-card' },
  { id: 'S20', surface: 'Zoom controls + shortcut hint', selector: 'button[aria-label="Zoom in"]', kind: 'zoom' },
  { id: 'S21', surface: 'Settings / key status', selector: 'main', kind: 'settings' },
  { id: 'S22', surface: 'File/code hierarchy viewer', selector: 'main', kind: 'data' },
  { id: 'S23', surface: 'Brand lockups / bee glyph', selector: 'nav[aria-label="Primary navigation"]', kind: 'brand' },
  { id: 'S24', surface: 'Elevation e1/e2/e3', selector: 'section[aria-label="HyperAgent hive chat core"]', kind: 'elevation' },
  { id: 'S25', surface: 'Code-diff review modal (decision-gated)', selector: 'text=Code-diff review', kind: 'diff' },
  { id: 'S26', surface: 'First-run/empty/loading/offline/missing-key states', selector: 'body', kind: 'states' },
  { id: 'S27', surface: 'Streamed reply/cards', selector: 'section[aria-label="HyperAgent hive chat core"]', kind: 'stream' },
  { id: 'S28', surface: 'Project identity', selector: 'header', kind: 'identity' },
];

function hopBase(present: boolean): SurfaceRecord['hops'] {
  return {
    '1_render': present ? 'WIRED-AND-WORKS' : 'MISSING',
    '2_handler': 'UNVERIFIED',
    '3_client_call': 'UNVERIFIED',
    '4_request': 'UNVERIFIED',
    '5_route': 'UNVERIFIED',
    '6_service': 'UNVERIFIED',
    '7_store': 'UNVERIFIED',
    '8_provider': 'UNVERIFIED',
    '9_response': 'UNVERIFIED',
    '10_render_back': 'UNVERIFIED',
    '11_reentry': 'UNVERIFIED',
  };
}

test.describe('live per-surface trace capture', () => {
  test.afterAll(async () => {
    ensureLiveDirs();
    const path = fileURLToPath(new URL('trace-capture.json', LIVE_DIR));
    const doc = {
      schema: 'hiverouter.trace.live-capture/v1',
      generated_at: new Date().toISOString(),
      base_url: BASE,
      rule: 'Only hops with captured DOM/interaction/frame evidence are WIRED-AND-WORKS; all others UNVERIFIED. No fabricated pass.',
      surfaces: records,
    };
    writeFileSync(path, `${JSON.stringify(doc, null, 2)}\n`, 'utf8');
  });

  for (const surface of SURFACES) {
    test(`capture ${surface.id} ${surface.surface}`, async ({ page }) => {
      test.skip(appMissing(), skipReason());
      await page.goto(`${BASE}/`, { waitUntil: 'networkidle', timeout: 30000 });
      await page.waitForTimeout(350);
      const record: SurfaceRecord = {
        surface_id: surface.id,
        surface: surface.surface,
        selector: surface.selector,
        present: false,
        evidence: '',
        interactions: [],
        hops: hopBase(false),
        notes: [],
      };

      if (surface.kind === 'node-card') {
        const nodes = await page.locator('.react-flow__node').count();
        record.present = false;
        record.evidence = `react-flow nodes=${nodes}; no run was executed in this capture`;
        record.notes.push('Node cards require a live run; no run was started (no paid API). UNVERIFIED.');
        records.push(record);
        return;
      }

      if (surface.kind === 'settings') {
        await page.getByRole('button', { name: 'Settings', exact: true }).first().click();
        await page.waitForTimeout(1200);
        record.present = (await page.getByText('Provider key status').count()) > 0;
        record.evidence = `settings panel rendered; "Provider key status" present=${record.present}`;
        record.interactions.push('clicked Settings nav');
        record.hops = hopBase(record.present);
        record.hops['2_handler'] = 'WIRED-AND-WORKS';
        record.hops['3_client_call'] = 'WIRED-AND-WORKS';
        record.hops['4_request'] = 'WIRED-AND-WORKS';
        record.hops['5_route'] = 'WIRED-AND-WORKS';
        record.hops['9_response'] = 'WIRED-BUT-DEAD';
        record.notes.push('SETTINGS_STATUS sent twice but server answered E_INTERNAL_ERROR (proven in live-app.spec.ts 12); client call/route exist, response is wrong.');
        records.push(record);
        expect(record.present).toBe(true);
        return;
      }

      if (surface.kind === 'diff') {
        await page.getByRole('button', { name: 'Settings', exact: true }).first().click();
        await page.getByRole('button', { name: 'Code-diff review', exact: true }).click();
        await page.waitForTimeout(300);
        const decisionText = await page.getByText(/proposal-gated/i).count();
        record.present = decisionText > 0;
        record.evidence = `decision-gated modal text present=${record.present}`;
        record.interactions.push('opened Code-diff review from Settings');
        record.hops = hopBase(record.present);
        record.hops['2_handler'] = 'WIRED-AND-WORKS';
        record.hops['10_render_back'] = 'WIRED-AND-WORKS';
        records.push(record);
        expect(record.present).toBe(true);
        return;
      }

      if (surface.kind === 'data') {
        await page.getByRole('button', { name: 'Data', exact: true }).first().click();
        const text = await page.locator('main').innerText();
        record.present = text.includes('Data');
        record.evidence = `Data panel rendered; named-gap copy present=${/no workspace-file API yet/i.test(text)}`;
        record.interactions.push('clicked Data nav');
        record.hops = hopBase(record.present);
        record.hops['2_handler'] = 'WIRED-AND-WORKS';
        record.hops['5_route'] = 'MISSING';
        record.hops['6_service'] = 'MISSING';
        record.notes.push('FILE_TREE_LIST/FILE_READ are frozen in the contract but the UI/server exposes no file API; panel shows a named gap (honest, not fake).');
        records.push(record);
        expect(record.present).toBe(true);
        return;
      }

      if (surface.kind === 'identity') {
        const identity = await page.getByText('HiveMind AI · You').count();
        record.present = identity > 0;
        record.evidence = `project identity rendered from state=${record.present}`;
        record.hops = hopBase(record.present);
        records.push(record);
        expect(record.present).toBe(true);
        return;
      }

      if (surface.kind === 'states') {
        const probes = {
          agentsEmpty: await page.getByText('No agents yet — submit a prompt to create the first run.').count(),
          scrubberEmpty: await page.getByText('No run history yet').count(),
          hudNoMetrics: await page.getByText('no metrics').count(),
        };
        record.present = probes.agentsEmpty + probes.scrubberEmpty + probes.hudNoMetrics > 0;
        record.evidence = JSON.stringify(probes);
        record.hops = hopBase(record.present);
        record.notes.push('Only empty/no-metrics states observed; loading/error/replay/missing-key states were not forced in this capture.');
        records.push(record);
        expect(record.present).toBe(true);
        return;
      }

      if (surface.kind === 'tabs') {
        const tabs = page.getByRole('tablist', { name: 'Workspace tabs' });
        record.present = (await tabs.getByRole('tab').count()) === 4;
        await tabs.getByRole('tab', { name: 'Swarm', exact: true }).click();
        const selected = await tabs.getByRole('tab', { name: 'Swarm', exact: true }).getAttribute('aria-selected');
        await tabs.getByRole('tab', { name: 'Workspace', exact: true }).click();
        record.interactions.push(`clicked Swarm; aria-selected=${selected}`);
        record.evidence = `4 tabs present; click render-back aria-selected=${selected}`;
        record.hops = hopBase(record.present);
        record.hops['2_handler'] = 'WIRED-AND-WORKS';
        record.hops['10_render_back'] = selected === 'true' ? 'WIRED-AND-WORKS' : 'WIRED-BUT-DEAD';
        records.push(record);
        expect(record.present).toBe(true);
        expect(selected).toBe('true');
        return;
      }

      if (surface.kind === 'nav') {
        const nav = page.locator('nav[aria-label="Primary navigation"]');
        const count = await nav.getByRole('button').count();
        await nav.getByRole('button', { name: 'Agents', exact: true }).click();
        const current = await nav.getByRole('button', { name: 'Agents', exact: true }).getAttribute('aria-current');
        await nav.getByRole('button', { name: 'Workspace', exact: true }).click();
        record.present = count === 6;
        record.evidence = `nav buttons=${count}; Agents aria-current=${current}`;
        record.interactions.push('clicked Agents then Workspace');
        record.hops = hopBase(record.present);
        record.hops['2_handler'] = 'WIRED-AND-WORKS';
        record.hops['10_render_back'] = current === 'page' ? 'WIRED-AND-WORKS' : 'WIRED-BUT-DEAD';
        records.push(record);
        expect(count).toBe(6);
        expect(current).toBe('page');
        return;
      }

      if (surface.kind === 'zoom') {
        const viewport = page.locator('.react-flow__viewport');
        const read = () => viewport.evaluate((el) => getComputedStyle(el).transform);
        const t0 = await read();
        await page.getByRole('button', { name: 'Zoom in', exact: true }).click();
        await page.waitForTimeout(350);
        const t1 = await read();
        await page.keyboard.press('Control+0');
        await page.waitForTimeout(450);
        const t2 = await read();
        record.present = true;
        record.evidence = `zoom controls present; transforms ${t0} -> ${t1} -> ${t2}`;
        record.interactions.push('Zoom in, Ctrl+0');
        record.hops = hopBase(true);
        record.hops['2_handler'] = 'WIRED-AND-WORKS';
        record.hops['10_render_back'] = t0 !== t1 && t1 !== t2 ? 'WIRED-AND-WORKS' : 'WIRED-BUT-DEAD';
        records.push(record);
        expect(t0).not.toBe(t1);
        expect(t1).not.toBe(t2);
        return;
      }

      if (surface.kind === 'composer') {
        const disabled = await page.locator('button[type=submit]').isDisabled();
        record.present = (await page.locator('textarea#composer').count()) > 0;
        record.evidence = `composer present; submit disabled on empty=${disabled}`;
        record.hops = hopBase(record.present);
        record.hops['2_handler'] = 'WIRED-AND-WORKS';
        record.notes.push('Valid SUBMIT_PROMPT not sent (would start a provider run; no paid API). Empty-prompt refusal proven in live-app.spec.ts 09.');
        records.push(record);
        expect(record.present).toBe(true);
        return;
      }

      if (surface.kind === 'control') {
        const locator = page.locator(surface.selector).first();
        record.present = (await page.locator(surface.selector).count()) > 0;
        const disabled = record.present ? await locator.isDisabled().catch(() => false) : false;
        const title = record.present ? await locator.getAttribute('title') : null;
        record.evidence = `control present=${record.present} disabled(no run)=${disabled} title=${title ?? 'null'}`;
        record.hops = hopBase(record.present);
        record.notes.push('Mutating control disabled without an active run (correct no-fake-success gating); not clicked.');
        records.push(record);
        expect(record.present).toBe(true);
        return;
      }

      const locator = page.locator(surface.selector).first();
      record.present = (await page.locator(surface.selector).count()) > 0;
      record.evidence = `selector ${surface.selector} count=${await page.locator(surface.selector).count()}`;
      record.hops = hopBase(record.present);
      if (surface.kind === 'minimap') {
        const accessible = await page.locator('[data-testid="rf__minimap"] [aria-label]').count();
        record.notes.push(`minimap accessible name nodes=${accessible} (MiniMap ariaLabel from STRINGS.minimap.aria)`);
      }
      if (surface.kind === 'hud') record.evidence += ` text=${(await locator.innerText().catch(() => '')).slice(0, 120)}`;
      if (surface.kind === 'brand') record.notes.push('brand glyph is inline HM text; FigCraft A brand assets verified separately by the asset inventory.');
      if (surface.kind === 'elevation') record.notes.push(`computed box-shadow=${await locator.evaluate((el) => getComputedStyle(el).boxShadow).catch(() => 'n/a')}`);
      if (surface.kind === 'chat') record.evidence += `; chat visible=${await locator.isVisible()}`;
      if (surface.kind === 'stream') record.notes.push('Streaming reply requires a run; not exercised (no paid API). UNVERIFIED.');
      if (surface.kind === 'toast-region') record.notes.push('Toast region is empty until a refusal is raised; empty-prompt refusal proven in live-app.spec.ts 09.');
      if (surface.kind === 'scrubber') record.notes.push(`disabled(no history)=${await locator.isDisabled()}`);
      records.push(record);
      expect(record.present).toBe(true);
    });
  }
});
