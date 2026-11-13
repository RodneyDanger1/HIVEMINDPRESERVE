/**
 * uiux-gate — bottom dock + Help / capability / named-refusal gate.
 *
 * Headless only, installed-Chrome channel, no browser download, no paid provider
 * run. All measurements and screenshots are written under the directory named by
 * UIUX_EVIDENCE_DIR (default docs/evidence/live/uiux-gate-<utc-stamp>) so the run
 * is repeatable and every assertion has a byte-backed receipt.
 *
 * Preconditions:
 *   E2E_BASE_URL points at a released hiverouter serving web/dist (default
 *   http://127.0.0.1:8100). The refusal test requires a keyless data dir; it
 *   asserts the dock's STATE tab reports key "missing" BEFORE it ever types or
 *   clicks Submit, so a key-present server can never trigger a paid run.
 *
 * Run:
 *   $env:E2E_BASE_URL='http://127.0.0.1:8100'
 *   $env:UIUX_EVIDENCE_DIR='C:\...\docs\evidence\live\uiux-gate-<stamp>'
 *   npx playwright test specs/uiux-gate.spec.ts --project=chrome --workers=1
 */
import { test, expect, type Page } from '@playwright/test';
import fs from 'node:fs';
import path from 'node:path';

const EVIDENCE_DIR =
  process.env.UIUX_EVIDENCE_DIR ??
  path.resolve(process.cwd(), '..', 'docs/evidence/live', `uiux-gate-${new Date().toISOString().replace(/[:.]/g, '-')}`);
fs.mkdirSync(EVIDENCE_DIR, { recursive: true });

const DOCK = 'section[aria-label="HyperAgent hive chat core"]';
const SEP = '[role="separator"][aria-label="Resize chat panel"]';

type Watch = ReturnType<typeof watchPage>;

function watchPage(page: Page) {
  const consoleErrors: Array<{ type: string; text: string }> = [];
  const pageErrors: string[] = [];
  const failedRequests: Array<{ url: string; error: string }> = [];
  page.on('console', (message) => {
    if (message.type() === 'error') consoleErrors.push({ type: message.type(), text: message.text().slice(0, 500) });
  });
  page.on('pageerror', (error) => pageErrors.push(String(error).slice(0, 500)));
  page.on('requestfailed', (request) =>
    failedRequests.push({ url: request.url().slice(0, 300), error: request.failure()?.errorText ?? 'unknown' }),
  );
  return { consoleErrors, pageErrors, failedRequests };
}

async function shot(page: Page, name: string): Promise<string> {
  const file = path.join(EVIDENCE_DIR, `${name}.png`);
  await page.screenshot({ path: file, fullPage: false });
  return file;
}

function rect(box: { x: number; y: number; width: number; height: number } | null) {
  if (box === null) return null;
  const r = (value: number) => Math.round(value * 10) / 10;
  return { x: r(box.x), y: r(box.y), w: r(box.width), h: r(box.height), right: r(box.x + box.width), bottom: r(box.y + box.height) };
}

async function openApp(page: Page): Promise<void> {
  await page.goto('/', { waitUntil: 'domcontentloaded' });
  const dock = page.locator(DOCK);
  await dock.waitFor({ state: 'visible', timeout: 20_000 });
  const composer = page.locator('textarea[placeholder="Type your prompt..."]');
  await expect.poll(async () => composer.isEnabled(), { timeout: 20_000 }).toBe(true);
  await page.waitForTimeout(500);
}

type Evidence = {
  name: string;
  status: 'PASS' | 'FAIL';
  startedAt: string;
  baseURL: string;
  viewport: { width: number; height: number };
  steps: Array<Record<string, unknown>>;
  checks: Array<{ name: string; pass: boolean; detail: string }>;
  consoleErrors: Array<{ type: string; text: string }>;
  pageErrors: string[];
  failedRequests: Array<{ url: string; error: string }>;
  error?: string;
  artifacts: string[];
};

function check(evidence: Evidence, label: string, pass: boolean, detail: string): void {
  evidence.checks.push({ name: label, pass, detail });
  if (!pass) throw new Error(`GATE FAIL: ${label} — ${detail}`);
}

async function withEvidence(name: string, page: Page, watch: Watch, body: (evidence: Evidence) => Promise<void>): Promise<void> {
  const evidence: Evidence = {
    name,
    status: 'PASS',
    startedAt: new Date().toISOString(),
    baseURL: page.url(),
    viewport: page.viewportSize() ?? { width: 0, height: 0 },
    steps: [],
    checks: [],
    consoleErrors: watch.consoleErrors,
    pageErrors: watch.pageErrors,
    failedRequests: watch.failedRequests,
    artifacts: [],
  };
  try {
    await body(evidence);
  } catch (error) {
    evidence.status = 'FAIL';
    evidence.error = String((error as Error).stack ?? error).slice(0, 4000);
    try {
      evidence.artifacts.push(await shot(page, `FAIL-${name}`));
    } catch {
      // page may already be closed; the text evidence still lands.
    }
    throw error;
  } finally {
    const consoleOk = watch.consoleErrors.length === 0;
    const pageOk = watch.pageErrors.length === 0;
    evidence.checks.push({ name: 'zero console errors', pass: consoleOk, detail: `${watch.consoleErrors.length} error console message(s)` });
    evidence.checks.push({ name: 'zero page errors', pass: pageOk, detail: `${watch.pageErrors.length} uncaught page error(s)` });
    if (!consoleOk || !pageOk) {
      evidence.status = 'FAIL';
      evidence.error = `browser errors — console=${watch.consoleErrors.length} page=${watch.pageErrors.length}`;
      evidence.checks.push({ name: 'browser error gate', pass: false, detail: evidence.error });
    }
    const file = path.join(EVIDENCE_DIR, `${name}.json`);
    fs.writeFileSync(file, JSON.stringify(evidence, null, 2));
    evidence.artifacts.push(file);
    if (!consoleOk || !pageOk) {
      throw new Error(`GATE FAIL: browser errors — console=${watch.consoleErrors.length} page=${watch.pageErrors.length}`);
    }
  }
}

test.describe('uiux gate — dock, help, capability, refusal', () => {
  test('1440: dock flush geometry, resize persistence, trays, Help tutorials', async ({ browser }) => {
    test.setTimeout(90_000);
    const context = await browser.newContext({ viewport: { width: 1440, height: 900 } });
    const page = await context.newPage();
    const watch = watchPage(page);
    await openApp(page);

    await withEvidence('01-1440-dock-help', page, watch, async (evidence) => {
      const dock = page.locator(DOCK);
      const initial = rect(await dock.boundingBox());
      evidence.steps.push({ step: 'initial-1440', dock: initial });
      evidence.artifacts.push(await shot(page, '01-1440-initial'));

      check(evidence, 'dock x flush to 72px rail', initial !== null && Math.abs(initial.x - 72) <= 1.5, JSON.stringify(initial));
      check(evidence, 'dock right edge flush to 1440', initial !== null && Math.abs(initial.right - 1440) <= 1.5, JSON.stringify(initial));
      check(evidence, 'dock bottom flush to 900', initial !== null && Math.abs(initial.bottom - 900) <= 1.5, JSON.stringify(initial));

      const initialStored = await page.evaluate(() => localStorage.getItem('hivemind.chat-height'));
      evidence.steps.push({ step: 'initial-stored-height', initialStored, initialHeight: initial?.h });

      const sep = rect(await page.locator(SEP).boundingBox());
      check(evidence, 'resize separator visible', sep !== null && sep.h >= 8, JSON.stringify(sep));
      if (sep === null) return;
      await page.mouse.move(sep.x + sep.w / 2, sep.y + sep.h / 2);
      await page.mouse.down();
      for (let i = 1; i <= 12; i += 1) {
        await page.mouse.move(sep.x + sep.w / 2, sep.y + sep.h / 2 - i * 10, { steps: 1 });
        await page.waitForTimeout(16);
      }
      await page.mouse.up();
      await page.waitForTimeout(400);

      const afterDrag = rect(await dock.boundingBox());
      const storedAfterDrag = await page.evaluate(() => localStorage.getItem('hivemind.chat-height'));
      evidence.steps.push({ step: 'after-drag', dock: afterDrag, storedAfterDrag });
      evidence.artifacts.push(await shot(page, '02-1440-after-resize'));
      check(
        evidence,
        'drag upward increases dock height by ~120px',
        initial !== null && afterDrag !== null && afterDrag.h - initial.h >= 100 && afterDrag.h - initial.h <= 140,
        `before=${initial?.h} after=${afterDrag?.h}`,
      );
      check(evidence, 'dock bottom stays fixed after resize', afterDrag !== null && Math.abs(afterDrag.bottom - 900) <= 1.5, JSON.stringify(afterDrag));
      check(
        evidence,
        'resized height persisted to localStorage',
        afterDrag !== null && storedAfterDrag !== null && Math.abs(Number(storedAfterDrag) - afterDrag.h) <= 1,
        `stored=${storedAfterDrag} measured=${afterDrag?.h}`,
      );

      await page.reload({ waitUntil: 'domcontentloaded' });
      await page.locator(DOCK).waitFor({ state: 'visible', timeout: 20_000 });
      await page.waitForTimeout(500);
      const afterReload = rect(await page.locator(DOCK).boundingBox());
      evidence.steps.push({ step: 'after-reload', dock: afterReload });
      evidence.artifacts.push(await shot(page, '03-1440-after-reload'));
      check(
        evidence,
        'resize persists across reload',
        afterDrag !== null && afterReload !== null && Math.abs(afterReload.h - afterDrag.h) <= 1.5,
        `beforeReload=${afterDrag?.h} afterReload=${afterReload?.h}`,
      );
      check(evidence, 'dock bottom stays fixed after reload', afterReload !== null && Math.abs(afterReload.bottom - 900) <= 1.5, JSON.stringify(afterReload));

      const controls = page.getByRole('button', { name: 'CONTROLS', exact: true });
      await controls.click();
      await page.waitForTimeout(450);
      const controlsOpen = rect(await page.locator(DOCK).boundingBox());
      const controlsAria = await controls.getAttribute('aria-expanded');
      evidence.steps.push({ step: 'controls-open', dock: controlsOpen, ariaExpanded: controlsAria });
      evidence.artifacts.push(await shot(page, '04-1440-controls'));
      check(
        evidence,
        'CONTROLS expands the dock (210px tray, clamped by the 848px max-height)',
        afterReload !== null &&
          controlsOpen !== null &&
          controlsOpen.h > afterReload.h &&
          (Math.abs(controlsOpen.h - afterReload.h - 210) <= 3 || Math.abs(controlsOpen.h - 848) <= 1.5),
        `closed=${afterReload?.h} open=${controlsOpen?.h} clampApplied=${controlsOpen !== null && Math.abs(controlsOpen.h - 848) <= 1.5}`,
      );
      check(evidence, 'CONTROLS sets aria-expanded', controlsAria === 'true', String(controlsAria));
      check(evidence, 'bottom stays fixed when CONTROLS expands', controlsOpen !== null && Math.abs(controlsOpen.bottom - 900) <= 1.5, JSON.stringify(controlsOpen));
      check(evidence, 'CONTROLS tray renders spawn control', (await page.getByRole('button', { name: 'Spawn sub-hive', exact: true }).count()) === 1, 'Spawn sub-hive button count');

      const stateTab = page.getByRole('button', { name: 'STATE & SETTINGS', exact: true });
      await stateTab.click();
      await page.waitForTimeout(450);
      const stateOpen = rect(await page.locator(DOCK).boundingBox());
      evidence.steps.push({
        step: 'state-open',
        dock: stateOpen,
        stateAria: await stateTab.getAttribute('aria-expanded'),
        controlsAria: await controls.getAttribute('aria-expanded'),
      });
      evidence.artifacts.push(await shot(page, '05-1440-state'));
      check(evidence, 'STATE & SETTINGS expands', (await stateTab.getAttribute('aria-expanded')) === 'true', 'state aria-expanded');
      check(evidence, 'CONTROLS collapses when STATE opens', (await controls.getAttribute('aria-expanded')) === 'false', 'controls aria-expanded');
      check(evidence, 'bottom stays fixed when STATE expands', stateOpen !== null && Math.abs(stateOpen.bottom - 900) <= 1.5, JSON.stringify(stateOpen));
      check(evidence, 'STATE tray shows provider row', (await page.locator(DOCK).getByText('provider', { exact: true }).count()) === 1, 'provider label');

      // Help page (nav destination, not the guide overlay): tutorials must render.
      await page.getByRole('button', { name: 'Help', exact: true }).click();
      await page.waitForTimeout(450);
      const helpHeading = page.getByRole('heading', { level: 1, name: 'What do you want to do?' });
      await expect(helpHeading).toBeVisible();
      const tutorialTitles = await page.locator('section h2').allTextContents();
      const helpText = await page.locator('main').innerText();
      evidence.steps.push({
        step: 'help',
        heading: await helpHeading.innerText(),
        tutorialTitles,
        hasDockTutorial: helpText.includes('Drag the top grip up or down to resize the chat.'),
        hasCapabilityText: /cannot see your desktop, browser, or screen/i.test(helpText),
        helpTextLength: helpText.length,
      });
      evidence.artifacts.push(await shot(page, '06-help'));
      check(evidence, 'Help page loads', await helpHeading.isVisible(), 'h1 What do you want to do?');
      check(
        evidence,
        'Help page carries tutorial sections First run / The dock / Safety',
        ['First run', 'The dock', 'Safety'].every((title) => tutorialTitles.includes(title)),
        JSON.stringify(tutorialTitles),
      );
      check(evidence, 'Help tutorial describes dock resize', helpText.includes('Drag the top grip up or down to resize the chat.'), 'tutorial copy');
      check(
        evidence,
        'Help capability card names the no-screen limit',
        /cannot see your desktop, browser, or screen/i.test(helpText),
        'capability copy',
      );
    });

    await context.close();
  });

  test('refusal: CLEAR and Dismiss remove the visible refusal', async ({ browser }) => {
    test.setTimeout(60_000);
    const context = await browser.newContext({ viewport: { width: 1440, height: 900 } });
    const page = await context.newPage();
    const watch = watchPage(page);
    await openApp(page);

    await withEvidence('02-refusal-clear-dismiss', page, watch, async (evidence) => {
      const dock = page.locator(DOCK);
      // Safety interlock: only a keyless server may be exercised. Never submit
      // a prompt when a provider key is present (that would start a paid run).
      await page.getByRole('button', { name: 'STATE & SETTINGS', exact: true }).click();
      await page.waitForTimeout(350);
      const missingCount = await dock.getByText('missing', { exact: true }).count();
      evidence.steps.push({ step: 'key-status', missingCount });
      check(evidence, 'server reports provider key missing before any submit', missingCount === 1, `missing label count=${missingCount}`);

      const composer = page.locator('textarea[placeholder="Type your prompt..."]');
      const submit = page.getByRole('button', { name: 'Submit prompt', exact: true });
      await composer.fill('summarise the capability limits of this workspace');
      check(evidence, 'submit enabled with text and open socket', await submit.isEnabled(), 'submit disabled');

      await submit.click();
      const refusal = page.locator('[role="alert"]').filter({ hasText: 'E_KEY_MISSING' });
      await expect(refusal).toBeVisible({ timeout: 8_000 });
      const refusalText = await refusal.innerText();
      evidence.steps.push({ step: 'refusal-raised', text: refusalText.slice(0, 500) });
      evidence.artifacts.push(await shot(page, '07-refusal-visible'));
      check(evidence, 'visible refusal names E_KEY_MISSING', refusalText.includes('E_KEY_MISSING'), refusalText.slice(0, 200));
      check(evidence, 'refusal is labelled REFUSED BY NAME', refusalText.includes('REFUSED BY NAME'), refusalText.slice(0, 200));

      const posts: string[] = [];
      page.on('request', (request) => {
        if (request.method() === 'POST' && request.url().includes('/api/')) posts.push(request.url());
      });
      await page.getByRole('button', { name: 'Clear the visible chat transcript', exact: true }).click();
      await page.waitForTimeout(450);
      evidence.artifacts.push(await shot(page, '08-after-clear'));
      check(evidence, 'CLEAR removes the visible refusal', (await page.locator('[role="alert"]').filter({ hasText: 'E_KEY_MISSING' }).count()) === 0, 'E_KEY_MISSING alert still present');
      check(evidence, 'CLEAR issued no POST /api request', posts.length === 0, JSON.stringify(posts));

      await composer.fill('summarise the capability limits of this workspace');
      await submit.click();
      const refusal2 = page.locator('[role="alert"]').filter({ hasText: 'E_KEY_MISSING' });
      await expect(refusal2).toBeVisible({ timeout: 8_000 });
      evidence.artifacts.push(await shot(page, '09-refusal-again'));
      const dismiss = refusal2.getByRole('button', { name: 'Dismiss refusal', exact: true });
      check(evidence, 'refusal toast exposes Dismiss', (await dismiss.count()) === 1, 'Dismiss button count');
      await dismiss.click();
      await page.waitForTimeout(450);
      evidence.artifacts.push(await shot(page, '10-after-dismiss'));
      check(evidence, 'Dismiss removes the visible refusal', (await page.locator('[role="alert"]').filter({ hasText: 'E_KEY_MISSING' }).count()) === 0, 'E_KEY_MISSING alert still present');

      // Capability copy is visible in the dock and must not overclaim vision.
      const dockText = await dock.innerText();
      evidence.steps.push({ step: 'dock-capability', text: dockText.slice(0, 1200) });
      check(
        evidence,
        'dock capability copy denies screen/vision/desktop control',
        /No screen, vision, or desktop control\./i.test(dockText),
        'capability line missing',
      );
    });

    await context.close();
  });

  test('375: auto-height, no horizontal scroll, composer visible', async ({ browser }) => {
    test.setTimeout(60_000);
    const context = await browser.newContext({ viewport: { width: 375, height: 667 } });
    const page = await context.newPage();
    const watch = watchPage(page);
    await openApp(page);

    await withEvidence('03-375-mobile', page, watch, async (evidence) => {
      const dock = page.locator(DOCK);
      const initial = rect(await dock.boundingBox());
      const layout = await page.evaluate(() => {
        const section = document.querySelector('section[aria-label="HyperAgent hive chat core"]') as HTMLElement | null;
        const composer = document.querySelector('textarea[placeholder="Type your prompt..."]') as HTMLElement | null;
        const style = section ? getComputedStyle(section) : null;
        return {
          innerWidth: window.innerWidth,
          docScrollWidth: document.documentElement.scrollWidth,
          bodyScrollWidth: document.body.scrollWidth,
          inlineHeight: section?.style.height ?? null,
          computedHeight: style?.height ?? null,
          clientHeight: section?.clientHeight ?? null,
          scrollHeight: section?.scrollHeight ?? null,
          composerRect: composer ? (() => { const r = composer.getBoundingClientRect(); return { x: r.x, y: r.y, right: r.right, bottom: r.bottom, w: r.width, h: r.height }; })() : null,
        };
      });
      evidence.steps.push({ step: '375-initial', dock: initial, layout });
      evidence.artifacts.push(await shot(page, '11-375-initial'));
      check(evidence, '375: no horizontal document scroll', layout.docScrollWidth <= layout.innerWidth && layout.bodyScrollWidth <= layout.innerWidth, JSON.stringify(layout));
      check(evidence, '375: dock sits above bottom nav (bottom≈611)', initial !== null && Math.abs(initial.bottom - 611) <= 2, JSON.stringify(initial));
      check(evidence, '375: dock x is 0 and right is 375', initial !== null && Math.abs(initial.x) <= 1.5 && Math.abs(initial.right - 375) <= 1.5, JSON.stringify(initial));
      check(
        evidence,
        '375: desktop inline height is overridden by the mobile auto-height cap',
        layout.computedHeight !== layout.inlineHeight && layout.clientHeight !== null && layout.scrollHeight !== null && layout.scrollHeight <= layout.clientHeight + 1,
        JSON.stringify({ inlineHeight: layout.inlineHeight, computedHeight: layout.computedHeight, clientHeight: layout.clientHeight, scrollHeight: layout.scrollHeight }),
      );
      check(
        evidence,
        '375: composer fully inside viewport',
        layout.composerRect !== null && layout.composerRect.x >= 0 && layout.composerRect.right <= 375 && layout.composerRect.y >= 0 && layout.composerRect.bottom <= 667,
        JSON.stringify(layout.composerRect),
      );

      await page.getByRole('button', { name: 'CONTROLS', exact: true }).click();
      await page.waitForTimeout(450);
      const controlsOpen = rect(await dock.boundingBox());
      const afterControls = await page.evaluate(() => ({
        innerWidth: window.innerWidth,
        docScrollWidth: document.documentElement.scrollWidth,
        bodyScrollWidth: document.body.scrollWidth,
      }));
      evidence.steps.push({ step: '375-controls', dock: controlsOpen, layout: afterControls });
      evidence.artifacts.push(await shot(page, '12-375-controls'));
      check(evidence, '375 CONTROLS: no horizontal scroll', afterControls.docScrollWidth <= afterControls.innerWidth && afterControls.bodyScrollWidth <= afterControls.innerWidth, JSON.stringify(afterControls));
      check(evidence, '375 CONTROLS: bottom stays fixed', controlsOpen !== null && Math.abs(controlsOpen.bottom - 611) <= 2, JSON.stringify(controlsOpen));
      check(evidence, '375 CONTROLS: composer remains visible', (await page.locator('textarea[placeholder="Type your prompt..."]').isVisible()), 'composer hidden');
    });

    await context.close();
  });
});
