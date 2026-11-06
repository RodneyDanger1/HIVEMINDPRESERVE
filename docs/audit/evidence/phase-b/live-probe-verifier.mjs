// Independent verifier live probe v2 — browser e2e evidence for C-026 / Phase B.
// Uses installed Chrome via the e2e project's Playwright (no browser download).
import { writeFileSync } from 'node:fs';
import { createRequire } from 'node:module';

const require = createRequire('file:///C:/Users/Riley/Desktop/hiverouter-v1/e2e/');
const { chromium } = require('playwright');

const BASE = 'http://127.0.0.1:8799';
const OUT = 'C:/Users/Riley/Desktop/hiverouter-v1/docs/audit/evidence/phase-b/live-probe-verifier.json';

const result = {
  generated_at: new Date().toISOString(),
  base: BASE,
  browser: null,
  page: {},
  network: { failedResponses: [], cdp4xx5xx: [], requestFailures: [] },
  console: [],
  ws: null,
  ok: null,
};

const browser = await chromium.launch({ channel: 'chrome', headless: true });
result.browser = `chrome ${browser.version()}`;
const ctx = await browser.newContext({ viewport: { width: 1440, height: 900 } });
const page = await ctx.newPage();

const cdp = await ctx.newCDPSession(page);
await cdp.send('Network.enable');
cdp.on('Network.responseReceived', (ev) => {
  if (ev.response.status >= 400) result.network.cdp4xx5xx.push({ url: ev.response.url, status: ev.response.status, mimeType: ev.response.mimeType });
});

page.on('console', (m) => result.console.push({ type: m.type(), text: m.text() }));
page.on('response', (r) => {
  if (r.status() >= 400) result.network.failedResponses.push({ url: r.url(), status: r.status(), ct: r.headers()['content-type'] ?? null });
});
page.on('requestfailed', (r) => result.network.requestFailures.push({ url: r.url(), failure: r.failure()?.errorText ?? null }));
page.on('pageerror', (e) => result.console.push({ type: 'pageerror', text: String(e) }));

const resp = await page.goto(BASE + '/', { waitUntil: 'networkidle', timeout: 30000 });
result.page.status = resp?.status() ?? null;
result.page.title = await page.title();
result.page.root = await page.locator('#root').count();
result.page.buttons = await page.locator('button').count();
result.page.navItems = await page.locator('nav button, nav a').count();
result.page.tabs = await page.locator('[role=tab]').count();
result.page.canvasAria = await page.locator('[aria-label="HiveMind spatial canvas"]').count();
result.page.chatCoreAria = await page.locator('[aria-label="HyperAgent hive chat core"]').count();
result.page.minimapTestid = await page.locator('[data-testid="rf__minimap"]').count();
result.page.hudText = await page.locator('[aria-label="Swarm throughput"]').innerText().catch(() => null);
result.page.scrubberDisabled = await page.locator('input[aria-label="Time-travel scrubber"]').isDisabled().catch(() => null);

// --- session token + live WS command round-trips from inside the page (real browser Origin) ---
const wsEvidence = await page.evaluate(async (base) => {
  const out = { session: null, hello: null, roundtrips: [], errors: [] };
  try {
    const s = await (await fetch(base + '/api/session')).json();
    out.session = { schema: s.schema, token_present: typeof s.token === 'string' && s.token.length > 0, ws: s.ws, last_seq: s.last_seq };
    const ws = new WebSocket(`ws://127.0.0.1:8799/ws?token=${encodeURIComponent(s.token)}&last_seq=0`);
    const frames = [];
    ws.onmessage = (ev) => frames.push({ t: Date.now(), frame: JSON.parse(ev.data) });
    const opened = await new Promise((resolve) => {
      ws.onopen = () => resolve(true);
      ws.onerror = () => resolve(false);
      setTimeout(() => resolve(false), 5000);
    });
    if (!opened) { out.errors.push('WS_DID_NOT_OPEN'); return out; }
    // protocol: first client frame must be a hello control frame
    ws.send(JSON.stringify({ type: 'hello', last_seq: 0 }));
    const waitFor = async (pred, ms) => {
      const t0 = Date.now();
      while (Date.now() - t0 < ms) {
        const hit = frames.find((f) => pred(f.frame));
        if (hit) return hit.frame;
        await new Promise((r) => setTimeout(r, 50));
      }
      return null;
    };
    out.hello = await waitFor((f) => f.name === 'SERVER_HELLO', 5000);
    const uuid = () => (globalThis.crypto && crypto.randomUUID ? crypto.randomUUID() : 'c-' + Math.random().toString(16).slice(2));
    const send = async (name, payload, ms = 5000) => {
      const id = uuid();
      const corr = uuid();
      const start = frames.length;
      ws.send(JSON.stringify({ schema: 'hiverouter.v1', type: 'COMMAND', name, id, correlation_id: corr, run_id: null, task_id: null, seq: null, ts: new Date().toISOString(), payload }));
      await new Promise((r) => setTimeout(r, ms));
      return {
        command: name,
        sent: { name, correlation_id: corr, payload },
        frames: frames.slice(start).map((f) => ({ type: f.frame.type, name: f.frame.name, code: f.frame.payload?.code ?? null, payload: f.frame.payload })),
      };
    };
    out.roundtrips.push(await send('SETTINGS_STATUS', {}));
    out.roundtrips.push(await send('FILE_TREE_LIST', {}));
    out.roundtrips.push(await send('SUBMIT_PROMPT', { text: '', project_id: 'p1' }));
    out.roundtrips.push(await send('SUBMIT_PROMPT', { text: 'verifier live probe: one tiny run', project_id: 'p1' }, 7000));
    out.roundtrips.push(await send('SPAWN_SUB_HIVE', { parent_task_id: '00000000-0000-7000-8000-000000000001', role: 'worker' }));
    out.totalFrames = frames.length;
    ws.close();
  } catch (e) {
    out.errors.push(String(e));
  }
  return out;
}, BASE);

result.ws = {
  session: wsEvidence.session,
  hello: wsEvidence.hello ? { schema: wsEvidence.hello.schema, name: wsEvidence.hello.name, payload: wsEvidence.hello.payload } : null,
  roundtrips: wsEvidence.roundtrips?.map((r) => ({ command: r.command, sent: r.sent, frames: r.frames })),
  totalFrames: wsEvidence.totalFrames ?? 0,
  errors: wsEvidence.errors,
};
result.ok = wsEvidence.session?.token_present === true && wsEvidence.errors.length === 0 && wsEvidence.hello !== null;

await page.waitForTimeout(500);
await browser.close();
writeFileSync(OUT, JSON.stringify(result, null, 2));
console.log(JSON.stringify({
  ok: result.ok,
  page: result.page,
  cdp4xx5xx: result.network.cdp4xx5xx,
  failedResponses: result.network.failedResponses,
  requestFailures: result.network.requestFailures,
  console: result.console,
  session: result.ws.session,
  hello: result.ws.hello?.name ?? null,
  roundtrips: result.ws.roundtrips,
  errors: result.ws.errors,
}, null, 2));
