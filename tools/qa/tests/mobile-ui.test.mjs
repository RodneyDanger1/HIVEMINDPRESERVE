// Mobile UI regression at 375x812 against the live local HiveMind server.
//
// The test uses the server that is already listening on http://127.0.0.1:8099
// (started by tools/start-hivemind.ps1) and authenticates with the launcher
// token stored at %LOCALAPPDATA%\HiveMind\launch.token. It never starts a
// second server and never makes a provider/model call.
//
// It asserts four things at a 375x812 mobile viewport:
//   1. `#composer` is the actual elementFromPoint hit (not covered by chrome),
//   2. the seven nav buttons expose aria-labels Workspace..Settings in order,
//   3. the document has no horizontal overflow,
//   4. the page produced no console errors and no uncaught exceptions.
//
// When the server, token or browser is unavailable the test SKIPS with the
// reason (node:test reports SKIP, never PASS). It never fakes a green run.
//
// Usage: node --test tools/qa/tests/mobile-ui.test.mjs
import test from 'node:test';
import assert from 'node:assert/strict';
import { existsSync, readFileSync } from 'node:fs';
import { createRequire } from 'node:module';
import os from 'node:os';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const ROOT = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..', '..', '..');
const BASE_URL = 'http://127.0.0.1:8099';
const VIEWPORT = Object.freeze({ width: 375, height: 812 });
const NAV_LABELS = Object.freeze([
  'Workspace',
  'Agents',
  'Data',
  'Models',
  'Routing',
  'Remote',
  'Settings',
]);
const LOCAL_APPDATA =
  process.env.LOCALAPPDATA || path.join(os.homedir(), 'AppData', 'Local');
const TOKEN_PATH = path.join(LOCAL_APPDATA, 'HiveMind', 'launch.token');

/** GET with a hard deadline; throws a readable error on timeout/network failure. */
async function fetchWithTimeout(url, init = {}, timeoutMs = 3_000) {
  const response = await fetch(url, { ...init, signal: AbortSignal.timeout(timeoutMs) });
  return response;
}

function skip(t, reason) {
  console.log(`SKIP: ${reason}`);
  t.skip(reason);
}

test(
  'mobile 375x812: composer uncovered, 7 nav labels, no horizontal overflow, no console errors',
  { timeout: 90_000 },
  async (t) => {
    // ---- Preconditions: a launchable live server. Anything missing is a SKIP.
    if (!existsSync(TOKEN_PATH)) {
      return skip(t, `launch token is missing at ${TOKEN_PATH}`);
    }
    const launchToken = readFileSync(TOKEN_PATH, 'utf8').trim();
    if (launchToken.length === 0) {
      return skip(t, `launch token at ${TOKEN_PATH} is empty`);
    }

    try {
      const health = await fetchWithTimeout(`${BASE_URL}/health`);
      if (!health.ok) {
        return skip(t, `server at ${BASE_URL} is not healthy (HTTP ${health.status})`);
      }
    } catch (error) {
      return skip(t, `server at ${BASE_URL} is unavailable: ${error.message}`);
    }

    // The token must be the one this server accepts, otherwise the UI would be
    // tested against a refused session and the result would be misleading.
    try {
      const session = await fetchWithTimeout(`${BASE_URL}/api/session`, {
        headers: { 'x-hivemind-launch': launchToken },
      });
      if (session.status === 401 || session.status === 403) {
        return skip(t, `server rejected the launch token from ${TOKEN_PATH} (HTTP ${session.status})`);
      }
      if (!session.ok) {
        return skip(t, `session probe returned HTTP ${session.status}`);
      }
    } catch (error) {
      return skip(t, `session probe failed: ${error.message}`);
    }

    // Playwright lives in e2e/node_modules (repo browser policy: installed
    // Chrome channel, never a downloaded browser).
    let chromium;
    try {
      const e2eRequire = createRequire(path.join(ROOT, 'e2e', 'package.json'));
      ({ chromium } = e2eRequire('playwright'));
    } catch (error) {
      return skip(t, `Playwright is not installed under e2e/: ${error.message}`);
    }

    let browser;
    try {
      browser = await chromium.launch({ channel: 'chrome', headless: true });
    } catch (error) {
      return skip(t, `installed Chrome channel is unavailable: ${error.message}`);
    }

    try {
      const context = await browser.newContext({
        viewport: VIEWPORT,
        deviceScaleFactor: 1,
        isMobile: true,
        hasTouch: true,
      });
      const page = await context.newPage();

      // The launch token rides in the document URL, so every string that can
      // reach test output (console text, error URLs, thrown messages) is
      // scrubbed before it is stored or rethrown. A secret never reaches a log.
      const redact = (value) => String(value).split(launchToken).join('[redacted-launch-token]');

      const consoleErrors = [];
      page.on('console', (message) => {
        if (message.type() === 'error') {
          consoleErrors.push({
            kind: 'console',
            text: redact(message.text()),
            url: redact(message.location().url),
          });
        }
      });
      page.on('pageerror', (error) => {
        consoleErrors.push({ kind: 'pageerror', text: redact(error), url: redact(page.url()) });
      });

      const launchUrl = `${BASE_URL}/?launch=${encodeURIComponent(launchToken)}`;
      let response;
      try {
        response = await page.goto(launchUrl, { waitUntil: 'domcontentloaded', timeout: 30_000 });
      } catch (error) {
        throw new Error(`GET ${BASE_URL}/ failed: ${redact(error?.message ?? error)}`);
      }
      assert.ok(response !== null, `GET ${BASE_URL}/ returned no response`);
      assert.equal(response.status(), 200, `GET ${BASE_URL}/ returned HTTP ${response.status()}`);

      // The startup veil is a real boot surface; dismiss it if it lingers and
      // wait until it is gone so it cannot be the elementFromPoint hit.
      const splash = page.locator('[data-testid="hivemind-splash"]');
      try {
        await splash.waitFor({ state: 'detached', timeout: 15_000 });
      } catch {
        const skipControl = page.getByTestId('splash-skip');
        if (await skipControl.isVisible().catch(() => false)) {
          await skipControl.click();
        }
        await splash.waitFor({ state: 'detached', timeout: 5_000 });
      }

      const composer = page.locator('#composer');
      await composer.waitFor({ state: 'visible', timeout: 15_000 });
      // Enabled means the session WebSocket settled; a dead socket must fail here
      // rather than silently pass a static/DOM-only check.
      await page.waitForFunction(
        () => {
          const element = document.querySelector('#composer');
          return element !== null && !element.disabled;
        },
        undefined,
        { timeout: 15_000 },
      );

      // ---- 1. #composer must be the elementFromPoint hit at its own center.
      const hit = await page.evaluate(() => {
        const element = document.querySelector('#composer');
        if (element === null) {
          return { ok: false, why: '#composer is missing from the document' };
        }
        const rect = element.getBoundingClientRect();
        const point = { x: rect.left + rect.width / 2, y: rect.top + rect.height / 2 };
        const inViewport =
          rect.width > 0 &&
          rect.height > 0 &&
          point.x >= 0 &&
          point.y >= 0 &&
          point.x <= window.innerWidth &&
          point.y <= window.innerHeight;
        if (!inViewport) {
          return {
            ok: false,
            why: '#composer center is outside the 375x812 viewport',
            rect: { x: rect.x, y: rect.y, width: rect.width, height: rect.height },
            point,
            viewport: { width: window.innerWidth, height: window.innerHeight },
          };
        }
        const top = document.elementFromPoint(point.x, point.y);
        const reachable = top !== null && (top === element || element.contains(top));
        return {
          ok: reachable,
          why: reachable
            ? null
            : '#composer is covered by another element at its center point',
          point,
          hit: top === null ? null : `${top.tagName.toLowerCase()}${top.id ? `#${top.id}` : ''}`,
        };
      });
      assert.equal(hit.ok, true, JSON.stringify(hit));

      // ---- 2. Seven nav buttons, aria-labels Workspace..Settings, in order.
      const nav = await page.evaluate(() => {
        const rail = document.querySelector('nav[aria-label="Primary navigation"]');
        if (rail === null) {
          return { found: false, labels: [] };
        }
        return {
          found: true,
          labels: Array.from(rail.querySelectorAll('button')).map((button) =>
            button.getAttribute('aria-label'),
          ),
        };
      });
      assert.equal(nav.found, true, 'nav[aria-label="Primary navigation"] was not found');
      assert.deepEqual(nav.labels, [...NAV_LABELS]);

      // ---- 3. No horizontal overflow at 375px.
      const width = await page.evaluate(() => ({
        innerWidth: window.innerWidth,
        documentScrollWidth: document.documentElement.scrollWidth,
        bodyScrollWidth: document.body.scrollWidth,
      }));
      const widest = Math.max(width.documentScrollWidth, width.bodyScrollWidth);
      assert.ok(
        widest <= width.innerWidth + 1,
        `horizontal overflow: content is ${widest}px wide in a ${width.innerWidth}px viewport`,
      );

      // ---- 4. No console errors / uncaught exceptions after the socket settles.
      await page.waitForTimeout(500);
      assert.deepEqual(consoleErrors, [], `console errors: ${JSON.stringify(consoleErrors)}`);

      t.diagnostic(
        JSON.stringify({
          verdict: 'PASS',
          url: BASE_URL,
          viewport: VIEWPORT,
          composerHit: hit.hit,
          navLabels: nav.labels,
          widths: { ...width, widest },
          consoleErrors,
        }),
      );
    } finally {
      await browser.close().catch(() => {});
    }
  },
);
