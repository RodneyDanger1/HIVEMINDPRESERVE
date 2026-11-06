// FINAL DELTA verifier live probe — browser + WS command round-trips on the frozen binary.
import { writeFileSync } from 'node:fs';
import { createRequire } from 'node:module';

const require = createRequire('file:///C:/Users/Riley/Desktop/hiverouter-v1/e2e/');
const { chromium } = require('playwright');

const BASE = 'http://127.0.0.1:8799';
const OUT = 'C:/Users/Riley/Desktop/hiverouter-v1/docs/audit/evidence/final/live-probe-final.json';

const result = { generated_at: new Date().toISOString(), base: BASE, browser: null, page: {}, network: { cdp4xx5xx: [] }, console: [], ws: null, ui: {}, ok: null };
const browser = await chromium.launch({ channel: 'chrome', headless: true });
result.browser = `chrome ${browser.version()}`;
const ctx = await browser.newContext({ viewport: { width: 1440, height: 900 } });
const page = await ctx.newPage();
const cdp = await ctx.newCDPSession(page);
await cdp.send('Network.enable');
cdp.on('Network.responseReceived', (ev) => { if (ev.response.status >= 400) result.network.cdp4xx5xx.push({ url: ev.response.url, status: ev.response.status }); });
page.on('console', (m) => { if (m.type() === 'error') result.console.push(m.text()); });
page.on('pageerror', (e) => result.console.push('pageerror: ' + String(e)));

const resp = await page.goto(BASE + '/', { waitUntil: 'networkidle', timeout: 30000 });
result.page = {
  status: resp?.status() ?? null,
  title: await page.title(),
  root: await page.locator('#root').count(),
  buttons: await page.locator('button').count(),
  navItems: await page.locator('nav button, nav a').count(),
  tabs: await page.locator('[role=tab]').count(),
};

// UI: Data panel must render the live file tree (S22), Settings must show key status (S21).
await page.getByRole('button', { name: 'Data' }).first().click().catch(() => {});
await page.waitForTimeout(700);
result.ui.dataPanelText = await page.locator('body').innerText();
await page.getByRole('button', { name: 'Settings' }).first().click().catch(() => {});
await page.waitForTimeout(900);
result.ui.settingsText = await page.locator('body').innerText();
result.ui.dataShowsReadme = /README\.md/.test(result.ui.dataPanelText ?? '');
result.ui.dataShowsNote = /note\.txt/.test(result.ui.dataPanelText ?? '');
result.ui.settingsShowsKey = /key/i.test(result.ui.settingsText ?? '');

// WS command matrix over the real browser transport.
const wsEvidence = await page.evaluate(async (base) => {
  const out = { session: null, hello: null, roundtrips: [], errors: [] };
  try {
    const s = await (await fetch(base + '/api/session')).json();
    out.session = { schema: s.schema, token_present: typeof s.token === 'string' && s.token.length > 0, ws: s.ws };
    const ws = new WebSocket(`ws://127.0.0.1:8799/ws?token=${encodeURIComponent(s.token)}&last_seq=0`);
    const frames = [];
    ws.onmessage = (ev) => frames.push(JSON.parse(ev.data));
    const opened = await new Promise((res) => { ws.onopen = () => res(true); ws.onerror = () => res(false); setTimeout(() => res(false), 5000); });
    if (!opened) { out.errors.push('WS_DID_NOT_OPEN'); return out; }
    ws.send(JSON.stringify({ type: 'hello', last_seq: 0 }));
    const waitFor = async (pred, ms) => { const t0 = Date.now(); while (Date.now() - t0 < ms) { const hit = frames.find(pred); if (hit) return hit; await new Promise((r) => setTimeout(r, 50)); } return null; };
    out.hello = await waitFor((f) => f.name === 'SERVER_HELLO', 5000);
    const uuid = () => (globalThis.crypto && crypto.randomUUID ? crypto.randomUUID() : 'c-' + Math.random().toString(16).slice(2));
    const send = async (name, payload, ms = 4000) => {
      const id = uuid(); const corr = uuid(); const start = frames.length;
      ws.send(JSON.stringify({ schema: 'hiverouter.v1', type: 'COMMAND', name, id, correlation_id: corr, run_id: null, task_id: null, seq: null, ts: new Date().toISOString(), payload }));
      await new Promise((r) => setTimeout(r, ms));
      return { command: name, frames: frames.slice(start).map((f) => ({ name: f.name, code: f.payload?.code ?? null, payload: f.payload })) };
    };
    out.roundtrips.push(await send('SETTINGS_STATUS', {}));
    out.roundtrips.push(await send('FILE_TREE_LIST', {}));
    out.roundtrips.push(await send('FILE_READ', { path: 'note.txt' }));
    out.roundtrips.push(await send('FILE_READ', { path: '../Cargo.toml' }));
    out.roundtrips.push(await send('APPROVAL_RESOLVE', { approval_id: '00000000-0000-7000-8000-000000000001', nonce: 'never-issued', decision: 'approve' }));
    out.roundtrips.push(await send('SUBMIT_PROMPT', { text: '', project_id: 'p1' }));
    out.roundtrips.push(await send('SUBMIT_PROMPT', { text: 'verifier final probe', project_id: 'p1' }, 5000));
    out.roundtrips.push(await send('SPAWN_SUB_HIVE', { parent_task_id: '00000000-0000-7000-8000-000000000001', role: 'worker' }));
    out.roundtrips.push(await send('REWIRE_TASK', { task_id: '00000000-0000-7000-8000-000000000001', new_parent_id: '00000000-0000-7000-8000-000000000002', expected_rev: 0 }));
    out.roundtrips.push(await send('CANCEL_RUN', { run_id: '00000000-0000-7000-8000-000000000001' }));
    out.roundtrips.push(await send('KILL_WORKER', { task_id: '00000000-0000-7000-8000-000000000001' }));
    out.roundtrips.push(await send('CHAT_PIN', { pinned: false }));
    ws.close();
  } catch (e) { out.errors.push(String(e)); }
  return out;
}, BASE);

result.ws = {
  session: wsEvidence.session,
  hello: wsEvidence.hello ? wsEvidence.hello.name : null,
  roundtrips: wsEvidence.roundtrips?.map((r) => ({ command: r.command, frames: r.frames.map((f) => ({ name: f.name, code: f.code })) })),
  errors: wsEvidence.errors,
};
// expected classification: no E_INTERNAL_ERROR anywhere on the four new commands
const flat = (result.ws.roundtrips ?? []).flatMap((r) => r.frames.map((f) => ({ cmd: r.command, ...f })));
result.newCommandFrames = flat.filter((f) => ['SETTINGS_STATUS', 'FILE_TREE_LIST', 'FILE_READ', 'APPROVAL_RESOLVE'].includes(f.cmd));
result.anyInternalError = flat.some((f) => f.code === 'E_INTERNAL_ERROR');
result.ok = wsEvidence.session?.token_present === true && wsEvidence.errors.length === 0 && wsEvidence.hello !== null && !result.anyInternalError;

await page.waitForTimeout(300);
await browser.close();
writeFileSync(OUT, JSON.stringify(result, null, 2));
console.log(JSON.stringify({
  ok: result.ok,
  page: result.page,
  ui: { dataShowsReadme: result.ui.dataShowsReadme, dataShowsNote: result.ui.dataShowsNote, settingsShowsKey: result.ui.settingsShowsKey },
  cdp4xx5xx: result.network.cdp4xx5xx,
  console: result.console,
  hello: result.ws.hello,
  newCommandFrames: result.newCommandFrames,
  allRoundtrips: result.ws.roundtrips,
  anyInternalError: result.anyInternalError,
  errors: result.ws.errors,
}, null, 2));
