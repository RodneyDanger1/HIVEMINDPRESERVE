import { test, expect } from '@playwright/test';
import { readFileSync, readdirSync, statSync } from 'node:fs';
import { fileURLToPath, pathToFileURL } from 'node:url';
import path from 'node:path';
import { contrastRatio } from '../helpers/contrast';
import {
  eventFrame,
  installMockWs,
  startDistHarness,
  type DistHarness,
  type MockWsHarness,
} from '../helpers/replay-harness';
import {
  AGENT_STATES,
  DOT_STATES,
  MIN_AGENT_STATE_COUNT,
  STATE_TRANSITIONS,
  dotFor,
  isAgentState,
  visualFor,
} from '../../web/src/state/agentState';
import { STRINGS } from '../../web/src/state/strings';

/**
 * W9A — G-UI-1..12 DOM gates against the current production build (`web/dist`).
 *
 * Deterministic by construction: loopback static server over web/dist + mocked WebSocket
 * transport (`page.routeWebSocket`). No live server, no key, no provider call.
 * Gates that genuinely need the Rust WS/Origin/resync server are listed with owner and
 * acceptance in `e2e/GATE-EXCLUSIONS.md`; a registry test below proves every G-UI id is
 * either implemented here or explicitly excluded (no silent skip).
 */
const ROOT = fileURLToPath(new URL('../../', import.meta.url));
const WEB_SRC = path.join(ROOT, 'web', 'src');

let harness: DistHarness;

test.beforeAll(async () => {
  harness = await startDistHarness();
});

test.afterAll(async () => {
  await harness.close();
});

async function openConnected(page: import('@playwright/test').Page): Promise<MockWsHarness> {
  const ws = await installMockWs(page);
  await page.goto(harness.url);
  await expect(page.getByText(/Connected/).first()).toBeVisible({ timeout: 15_000 });
  return ws;
}

function rgbToHex(color: string): string {
  const m = color.match(/rgba?\((\d+),\s*(\d+),\s*(\d+)/);
  if (!m) throw new Error(`cannot parse colour: ${color}`);
  return `#${[m[1], m[2], m[3]].map((n) => Number(n).toString(16).padStart(2, '0')).join('')}`;
}

async function effectiveBackground(page: import('@playwright/test').Page, selector: string): Promise<string> {
  return page.locator(selector).first().evaluate((el) => {
    let node: Element | null = el;
    while (node) {
      const bg = getComputedStyle(node).backgroundColor;
      if (bg && bg !== 'rgba(0, 0, 0, 0)' && bg !== 'transparent') return bg;
      node = node.parentElement;
    }
    return getComputedStyle(document.body).backgroundColor;
  });
}

test.describe('G-UI-1..12 — deterministic DOM gates (web/dist + mocked WS)', () => {
  test('G-UI-1: computed token pairs satisfy the printed contrast table', async ({ page }) => {
    await openConnected(page);
    const styles = await page.evaluate(() => {
      const body = getComputedStyle(document.body);
      const panel = document.querySelector('.surface-panel');
      return {
        bodyColor: body.color,
        bodyBg: body.backgroundColor,
        panelColor: panel ? getComputedStyle(panel).color : null,
        panelBg: panel ? getComputedStyle(panel).backgroundColor : null,
        ink700: getComputedStyle(document.documentElement).getPropertyValue('--color-ink-700').trim(),
      };
    });
    expect(rgbToHex(styles.bodyBg)).toBe('#f8f4eb');
    expect(rgbToHex(styles.bodyColor)).toBe('#282010');
    expect(contrastRatio(rgbToHex(styles.bodyColor), rgbToHex(styles.bodyBg))).toBeGreaterThanOrEqual(4.5);
    expect(styles.panelColor).not.toBeNull();
    expect(styles.panelBg).not.toBeNull();
    if (styles.panelColor && styles.panelBg) {
      expect(contrastRatio(rgbToHex(styles.panelColor), rgbToHex(styles.panelBg))).toBeGreaterThanOrEqual(4.5);
    }
    expect(styles.ink700.toLowerCase()).toBe('#4e3a15');
    expect(contrastRatio('#4e3a15', '#ffffff'), 'focus ring vs panel').toBeGreaterThanOrEqual(3);
    expect(contrastRatio('#4e3a15', '#f8f4eb'), 'focus ring vs canvas').toBeGreaterThanOrEqual(3);
  });

  test('G-UI-2: Ctrl+0 recentres canvas but never hijacks text-field focus', async ({ page }) => {
    await openConnected(page);
    const viewport = page.locator('.react-flow__viewport');
    const readTransform = () => viewport.evaluate((el) => getComputedStyle(el).transform);
    await page.getByRole('button', { name: 'Zoom in', exact: true }).click();
    await page.waitForTimeout(400);
    const zoomed = await readTransform();

    // (a) focus in a text field: Ctrl+0 must NOT be intercepted (canvas untouched, focus retained).
    const composer = page.locator('#composer');
    await composer.click();
    await composer.fill('focus-not-hijacked');
    await expect(composer).toBeFocused();
    await page.keyboard.press('Control+0');
    await page.waitForTimeout(450);
    expect(await readTransform(), 'canvas must not recentre while a text field owns the shortcut').toBe(zoomed);
    await expect(composer).toBeFocused();
    await expect(composer).toHaveValue('focus-not-hijacked');
    const visualScale = await page.evaluate(() => window.visualViewport?.scale ?? 1);
    expect(visualScale, 'page zoom must not change').toBe(1);

    // (b) outside a text field: Ctrl+0 must recentre the canvas (transform changes).
    await page.locator('.react-flow__pane').click({ position: { x: 10, y: 10 } });
    await page.keyboard.press('Control+0');
    await page.waitForTimeout(450);
    expect(await readTransform(), 'Ctrl+0 with canvas focus must recentre').not.toBe(zoomed);
  });

  test('G-UI-4: one >=13-state vocabulary, total label/visual/transition coverage', async () => {
    expect(AGENT_STATES.length).toBeGreaterThanOrEqual(MIN_AGENT_STATE_COUNT);
    expect(MIN_AGENT_STATE_COUNT).toBeGreaterThanOrEqual(13);
    expect(new Set(AGENT_STATES).size).toBe(AGENT_STATES.length);
    expect(AGENT_STATES).toContain('interrupted');
    for (const state of AGENT_STATES) {
      expect(STRINGS.agentStates[state], `label for ${state}`).toBeTruthy();
      expect(STRINGS.agentStates[state].trim().length).toBeGreaterThan(0);
      const visual = visualFor(state);
      expect(visual.shape, `shape for ${state}`).toBeTruthy();
      expect(visual.toneToken, `tone token for ${state}`).toBeTruthy();
      expect(Object.prototype.hasOwnProperty.call(STATE_TRANSITIONS, state), `transition row for ${state}`).toBe(true);
    }
    expect(Object.keys(STATE_TRANSITIONS).sort()).toEqual([...AGENT_STATES].sort());
    expect(DOT_STATES).toEqual(['healthy', 'running', 'failed', 'queued']);
    for (const dot of DOT_STATES) expect(dotFor(dot)).toBe(dot);
    expect(isAgentState('healthy')).toBe(true);
    expect(isAgentState('not-a-real-state')).toBe(false);
  });

  test('G-UI-5: missing-state coverage — empty, offline, refusal, missing key, replay', async ({ page, browser }) => {
    const ws = await openConnected(page);
    const nav = page.locator('nav[aria-label="Primary navigation"]');
    await nav.getByRole('button', { name: 'Agents', exact: true }).click();
    await expect(page.getByText('No agents yet — submit a prompt to create the first run.').first()).toBeVisible();
    await nav.getByRole('button', { name: 'Data', exact: true }).click();
    await expect(page.getByText('Select a file from the tree to preview it here.').first()).toBeVisible();
    await nav.getByRole('button', { name: 'Routing', exact: true }).click();
    await expect(page.getByText('No edges in the current graph.').first()).toBeVisible();
    await expect(page.getByRole('status', { name: 'Swarm throughput' })).toContainText(/NO METRICS/i);

    // missing key: a typed ERROR event must render the named refusal toast (no fake success).
    ws.route().send(
      eventFrame(
        'ERROR',
        { code: 'E_KEY_MISSING', message: 'Provider key missing', how_to_fix: 'Set DEEPSEEK_API_KEY then retry', retryable: false },
        1,
      ),
    );
    await expect(page.getByText('E_KEY_MISSING')).toBeVisible();
    await expect(page.getByText('Set DEEPSEEK_API_KEY then retry')).toBeVisible();

    // replay: a HISTORY_SNAPSHOT event must enter the read-only replay state (canvas mounted).
    await nav.getByRole('button', { name: 'Workspace', exact: true }).click();
    ws.route().send(
      eventFrame(
        'HISTORY_SNAPSHOT',
        { run_id: 'run-g5', revision: 1, graph: { nodes: [{ task_id: 'snap-g5', state: 'completed', rev: 1 }], edges: [] } },
        2,
      ),
    );
    await expect(page.getByText(/Replay \(read-only\)/)).toBeVisible({ timeout: 10_000 });

    // offline: a page WITHOUT the WS mock must show the named offline banner and disable mutations.
    const offlineContext = await browser.newContext();
    const offlinePage = await offlineContext.newPage();
    try {
      await offlinePage.goto(harness.url);
      await expect(offlinePage.getByRole('alert').first()).toContainText(/unreachable|Reconnecting|Connection failed/i, { timeout: 15_000 });
      await expect(offlinePage.locator('#composer')).toBeDisabled();
    } finally {
      await offlineContext.close();
    }
  });

  test('G-UI-6: reduced motion stops animations and state still updates', async ({ page }) => {
    await openConnected(page);
    const before = await page.evaluate(() => document.documentElement.dataset.reducedMotion);
    expect(before).toBe('false');
    await page.emulateMedia({ reducedMotion: 'reduce' });
    await expect.poll(() => page.evaluate(() => document.documentElement.dataset.reducedMotion)).toBe('true');
    const running = await page.evaluate(() => document.getAnimations().filter((a) => a.playState === 'running').length);
    expect(running, 'no running animations under reduced motion').toBe(0);
    const tabs = page.getByRole('tablist', { name: 'Workspace tabs' });
    await tabs.getByRole('tab', { name: 'Swarm', exact: true }).click();
    await expect(tabs.getByRole('tab', { name: 'Swarm', exact: true })).toHaveAttribute('aria-selected', 'true');
  });

  test('G-UI-7: zoom clamps exactly at 0.1 and 3.0 (both reachable)', async ({ page }) => {
    await openConnected(page);
    await page.emulateMedia({ reducedMotion: 'reduce' });
    const scale = () =>
      page.locator('.react-flow__viewport').evaluate((el) => {
        const m = new DOMMatrixReadOnly(getComputedStyle(el).transform);
        return Number(m.a.toFixed(4));
      });
    const zoomOut = page.getByRole('button', { name: 'Zoom out', exact: true });
    for (let i = 0; i < 25; i += 1) {
      await zoomOut.click();
      await page.waitForTimeout(30);
    }
    expect(await scale(), 'min zoom clamp').toBeCloseTo(0.1, 2);
    const zoomIn = page.getByRole('button', { name: 'Zoom in', exact: true });
    for (let i = 0; i < 30; i += 1) {
      await zoomIn.click();
      await page.waitForTimeout(30);
    }
    expect(await scale(), 'max zoom clamp').toBeCloseTo(3.0, 2);
  });

  test('G-UI-8: keyboard focus indicators are visible and >= 3:1 (axe subcheck excluded, owned)', async ({ page }) => {
    await openConnected(page);
    await page.locator('body').click({ position: { x: 5, y: 5 } });
    const seen: string[] = [];
    for (let i = 0; i < 12; i += 1) {
      await page.keyboard.press('Tab');
      const info = await page.evaluate(() => {
        const el = document.activeElement as HTMLElement | null;
        if (!el) return null;
        const s = getComputedStyle(el);
        let bgNode: Element | null = el;
        let bg = 'rgba(0, 0, 0, 0)';
        while (bgNode) {
          const candidate = getComputedStyle(bgNode).backgroundColor;
          if (candidate && candidate !== 'rgba(0, 0, 0, 0)') {
            bg = candidate;
            break;
          }
          bgNode = bgNode.parentElement;
        }
        return {
          tag: el.tagName.toLowerCase(),
          label: el.getAttribute('aria-label') ?? el.textContent?.trim().slice(0, 40) ?? '',
          outlineStyle: s.outlineStyle,
          outlineWidth: s.outlineWidth,
          outlineColor: s.outlineColor,
          bg: bg === 'rgba(0, 0, 0, 0)' ? getComputedStyle(document.body).backgroundColor : bg,
        };
      });
      expect(info, `tab stop ${i + 1} must exist`).not.toBeNull();
      if (!info) continue;
      seen.push(`${info.tag}:${info.label}`);
      expect(info.outlineStyle, `focus outline style for ${info.tag}`).toBe('solid');
      expect(parseFloat(info.outlineWidth), `focus outline width for ${info.tag}`).toBeGreaterThanOrEqual(2);
      const ratio = contrastRatio(rgbToHex(info.outlineColor), rgbToHex(info.bg));
      expect(ratio, `focus contrast for ${info.tag} (${info.outlineColor} on ${info.bg})`).toBeGreaterThanOrEqual(3);
    }
    expect(seen.length, 'at least 8 keyboard stops must be observed').toBeGreaterThanOrEqual(8);
  });

  test('G-UI-9: refusal toast is named, readable and pre-mutation (no command sent)', async ({ page }) => {
    const ws = await openConnected(page);
    await page.locator('#composer').focus();
    await page.keyboard.press('Enter');
    const toast = page.locator('[aria-label="Refusals and errors"]');
    await expect(toast).toHaveAttribute('aria-live', 'assertive');
    await expect(toast.getByText('E_PROMPT_EMPTY')).toBeVisible();
    await expect(toast.getByText('REFUSED BY NAME · NOT SILENT')).toBeVisible();
    await expect(toast.getByText('Type a prompt before submitting (1-8000 characters).')).toBeVisible();
    await page.waitForTimeout(250);
    expect(ws.commands.filter((c) => c === 'SUBMIT_PROMPT'), 'refusal must not emit SUBMIT_PROMPT').toEqual([]);
  });

  test('G-UI-11: no hex outside the token module; every printed swatch has PDF provenance', async () => {
    const tokensTs = readFileSync(path.join(WEB_SRC, 'tokens', 'tokens.ts'), 'utf8');
    const tokensCss = readFileSync(path.join(WEB_SRC, 'styles', 'tokens.css'), 'utf8');
    const swatches = [...tokensTs.matchAll(/hex:\s*'(#[0-9a-fA-F]{6})',\s*pdfLine:\s*(\d+)/g)].map((m) => ({
      hex: m[1].toUpperCase(),
      pdfLine: Number(m[2]),
    }));
    expect(swatches.length, 'exactly 20 printed swatches').toBe(20);
    for (const swatch of swatches) expect(swatch.pdfLine, `pdfLine for ${swatch.hex}`).toBeGreaterThan(0);
    const cssHexes = [...new Set([...tokensCss.matchAll(/#[0-9a-fA-F]{6}\b/g)].map((m) => m[0].toUpperCase()))].sort();
    const tsHexes = [...new Set(swatches.map((s) => s.hex))].sort();
    expect(cssHexes).toEqual(tsHexes);
    expect(tokensCss).toMatch(/PDF:20-79/);
    const offenders: string[] = [];
    const walk = (dir: string): void => {
      for (const entry of readdirSync(dir)) {
        const abs = path.join(dir, entry);
        if (statSync(abs).isDirectory()) {
          walk(abs);
          continue;
        }
        if (!/\.(ts|tsx|css)$/.test(entry)) continue;
        if (abs.endsWith(path.join('tokens', 'tokens.ts')) || abs.endsWith(path.join('styles', 'tokens.css'))) continue;
        const text = readFileSync(abs, 'utf8');
        if (/#[0-9a-fA-F]{6}\b/.test(text)) offenders.push(path.relative(ROOT, abs));
      }
    };
    walk(WEB_SRC);
    expect(offenders, `hex literals outside token module: ${offenders.join(', ')}`).toEqual([]);
  });

  test('G-UI-12: P95 frame time measured (not assumed)', async ({ page }) => {
    await openConnected(page);
    const p95 = await page.evaluate(
      () =>
        new Promise<number>((resolve) => {
          const deltas: number[] = [];
          let last = performance.now();
          const step = (now: number): void => {
            deltas.push(now - last);
            last = now;
            if (deltas.length >= 120) {
              deltas.sort((a, b) => a - b);
              resolve(deltas[Math.floor(deltas.length * 0.95)] ?? Number.POSITIVE_INFINITY);
              return;
            }
            requestAnimationFrame(step);
          };
          requestAnimationFrame(step);
        }),
    );
    expect(Number.isFinite(p95)).toBe(true);
    expect(p95, `measured P95 frame time ${p95.toFixed(2)}ms`).toBeLessThan(50);
  });
});

test.describe('G-UI gate registry — no silent skips', () => {
  test('every G-UI-1..12 id is implemented or explicitly owned+accepted in GATE-EXCLUSIONS.md', async () => {
    const gates = Array.from({ length: 12 }, (_, i) => `G-UI-${i + 1}`);
    const spec = readFileSync(fileURLToPath(new URL('./gui-gates.spec.ts', import.meta.url)), 'utf8');
    const exclusions = readFileSync(path.join(ROOT, 'e2e', 'GATE-EXCLUSIONS.md'), 'utf8');
    for (const gate of gates) {
      const implemented = spec.includes(`'${gate}`) || spec.includes(`${gate}:`);
      const excluded = exclusions.includes(gate);
      expect(implemented || excluded, `${gate} must be implemented or explicitly excluded`).toBe(true);
      if (!implemented) {
        const row = exclusions.split(/\r?\n/).find((l) => l.includes(gate)) ?? '';
        const columns = row.split('|').map((c) => c.trim());
        expect(columns.length, `${gate} exclusion row must be a table row`).toBeGreaterThanOrEqual(6);
        expect(columns[4], `${gate} exclusion must name an owner`).toMatch(/owner|b-core|Lead|QA/i);
        expect(columns[5].length, `${gate} exclusion must carry an acceptance criterion`).toBeGreaterThan(25);
      }
    }
    expect(exclusions, 'exclusion table declares its acceptance column').toContain('Acceptance (what would close it)');
    for (const stub of ['W1', 'W2']) {
      expect(exclusions, `${stub} WS stub exclusion registered`).toContain(stub);
    }
    const fixmeCall = /test\.fixme\s*\(/;
    expect(fixmeCall.test(spec), 'gui-gates.spec.ts must not carry skipped gates').toBe(false);
    const a11y = readFileSync(fileURLToPath(new URL('./a11y-gates.spec.ts', import.meta.url)), 'utf8');
    const ws = readFileSync(fileURLToPath(new URL('./ws-contract.spec.ts', import.meta.url)), 'utf8');
    expect(fixmeCall.test(a11y), 'a11y-gates must not carry fixme gates').toBe(false);
    expect(fixmeCall.test(ws), 'ws-contract must not carry fixme stubs').toBe(false);
  });
});
