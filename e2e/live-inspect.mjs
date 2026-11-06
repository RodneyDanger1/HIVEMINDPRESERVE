/**
 * QA live-app inspection probe (read-only; no paid API, channel chrome only).
 * Writes docs/trace/live/inspect.json + app-boot.png and prints a compact summary.
 */
import { chromium } from '@playwright/test';
import { writeFileSync, mkdirSync } from 'node:fs';
import { fileURLToPath } from 'node:url';

const BASE = process.env.E2E_BASE_URL ?? 'http://127.0.0.1:8799';
const OUT = new URL('../docs/trace/live/', import.meta.url);
mkdirSync(OUT, { recursive: true });

const browser = await chromium.launch({ channel: 'chrome', headless: true });
const context = await browser.newContext({ viewport: { width: 1440, height: 900 } });
const page = await context.newPage();

const consoleErrors = [];
const pageErrors = [];
const badResponses = [];
const wsLog = [];
page.on('console', (m) => { if (m.type() === 'error') consoleErrors.push(m.text()); });
page.on('pageerror', (e) => pageErrors.push(String(e)));
page.on('response', (r) => { if (r.status() >= 400) badResponses.push({ url: r.url(), status: r.status() }); });
page.on('websocket', (ws) => {
  const entry = { url: ws.url(), sent: [], received: [] };
  wsLog.push(entry);
  ws.on('framesent', (f) => entry.sent.push(typeof f.payload === 'string' ? f.payload : f.payload.toString('base64')));
  ws.on('framereceived', (f) => entry.received.push(typeof f.payload === 'string' ? f.payload : f.payload.toString('base64')));
});

const resp = await page.goto(BASE + '/', { waitUntil: 'networkidle', timeout: 30000 });
await page.waitForTimeout(1500);

const session = await (await context.request.get(BASE + '/api/session')).json();
await page.screenshot({ path: fileURLToPath(new URL('app-boot.png', OUT)), fullPage: false });

const info = await page.evaluate(() => {
  const txt = (el) => (el?.textContent ?? '').replace(/\s+/g, ' ').trim();
  const byAria = (label) => document.querySelector(`[aria-label="${label}"]`);
  const collect = (sel) => [...document.querySelectorAll(sel)].map((el) => ({
    tag: el.tagName.toLowerCase(),
    text: txt(el).slice(0, 120),
    aria: el.getAttribute('aria-label'),
    role: el.getAttribute('role'),
    testid: el.getAttribute('data-testid'),
    id: el.id || null,
    disabled: el.hasAttribute('disabled') || el.getAttribute('aria-disabled') === 'true',
  }));
  const style = (el) => {
    if (!el) return null;
    const s = getComputedStyle(el);
    return { color: s.color, backgroundColor: s.backgroundColor, fontFamily: s.fontFamily, fontSize: s.fontSize };
  };
  const buttons = collect('button');
  return {
    title: document.title,
    root: Boolean(document.querySelector('#root')),
    bodyTextLen: document.body.innerText.length,
    skipLink: (() => { const a = document.querySelector('a.skip-link, a[href="#hive-canvas"]'); return a ? { text: txt(a), href: a.getAttribute('href') } : null; })(),
    nav: { label: byAria('Primary navigation')?.tagName ?? null, items: byAria('Primary navigation') ? [...byAria('Primary navigation').querySelectorAll('button,a')].map((x) => ({ text: txt(x), current: x.getAttribute('aria-current'), testid: x.getAttribute('data-testid') })) : [] },
    tabs: { label: byAria('Workspace tabs')?.tagName ?? null, items: [...document.querySelectorAll('[role=tab]')].map((x) => ({ text: txt(x), selected: x.getAttribute('aria-selected'), testid: x.getAttribute('data-testid') })) },
    chat: { core: Boolean(byAria('HyperAgent hive chat core')), text: txt(byAria('HyperAgent hive chat core')).slice(0, 300), input: collect('input,textarea').map((x) => ({ ...x, placeholder: document.querySelector('input,textarea')?.getAttribute('placeholder') })) },
    composerPlaceholder: document.querySelector('textarea,input[type=text]')?.getAttribute('placeholder') ?? null,
    buttons,
    countButtons: buttons.length,
    hud: { el: Boolean(byAria('Swarm throughput')), text: txt(byAria('Swarm throughput')).slice(0, 300) },
    scrubber: { el: Boolean(byAria('Time-travel scrubber')), text: txt(byAria('Time-travel scrubber')).slice(0, 200) },
    library: { el: Boolean(byAria('Library panel')), text: txt(byAria('Library panel')).slice(0, 400) },
    minimap: { el: Boolean(byAria('Canvas minimap')), svg: Boolean(byAria('Canvas minimap')?.querySelector('svg')) },
    canvas: { hiveCanvasId: Boolean(document.querySelector('#hive-canvas')), reactFlowNodes: document.querySelectorAll('.react-flow__node').length, edges: document.querySelectorAll('.react-flow__edge').length, svgs: document.querySelectorAll('svg').length },
    connection: txt(document.querySelector('[role=alert]')) || null,
    tokenColorProbe: {
      body: style(document.body),
      canvasRoot: style(document.querySelector('.bg-canvas') ?? document.body),
      panel: style(document.querySelector('.bg-panel')),
      meta: style(document.querySelector('.type-mono-meta')),
    },
    allAriaLabels: [...document.querySelectorAll('[aria-label]')].map((el) => el.getAttribute('aria-label')),
    testids: [...document.querySelectorAll('[data-testid]')].map((el) => el.getAttribute('data-testid')),
  };
});

const result = {
  generated_at: new Date().toISOString(),
  base: BASE,
  status: resp?.status(),
  session: { token_present: Boolean(session?.token), schema: session?.schema, last_seq: session?.last_seq, ws: session?.ws },
  consoleErrors,
  pageErrors,
  badResponses,
  websockets: wsLog.map((w) => ({
    url: w.url,
    token_in_url: Boolean(new URL(w.url).searchParams.get('token')),
    sent_count: w.sent.length,
    received_count: w.received.length,
    sent_first: w.sent.slice(0, 3).map((s) => s.slice(0, 220)),
    received_first: w.received.slice(0, 3).map((s) => s.slice(0, 220)),
  })),
  info,
};
writeFileSync(new URL('inspect.json', OUT), JSON.stringify(result, null, 2) + '\n', 'utf8');
console.log(JSON.stringify({
  status: result.status,
  session: result.session,
  consoleErrors: consoleErrors.length,
  pageErrors: pageErrors.length,
  badResponses,
  websockets: result.websockets,
  title: info.title,
  root: info.root,
  skipLink: info.skipLink,
  navItems: info.nav.items,
  tabs: info.tabs.items,
  chat: info.chat.core,
  composer: info.composerPlaceholder,
  buttons: info.countButtons,
  hud: info.hud,
  scrubber: info.scrubber,
  library: info.library.el,
  minimap: info.minimap,
  canvas: info.canvas,
  connection: info.connection,
  colors: info.tokenColorProbe,
  testids: info.testids,
  bodyTextLen: info.bodyTextLen,
}, null, 2));
await browser.close();
