import { chromium } from '@playwright/test';

const B = 'http://127.0.0.1:8799';
const b = await chromium.launch({ channel: 'chrome', headless: true });
const c = await b.newContext({ viewport: { width: 1500, height: 950 } });
const p = await c.newPage();
const frames = [];
const errors = [];
p.on('websocket', (ws) => {
  ws.on('framesent', (f) => frames.push({ dir: 'sent', raw: String(f.payload) }));
  ws.on('framereceived', (f) => frames.push({ dir: 'recv', raw: String(f.payload) }));
});
p.on('console', (m) => { if (m.type() === 'error') errors.push(m.text()); });
const resp = await p.goto(B + '/', { waitUntil: 'networkidle' });
await p.waitForTimeout(800);
const html = await p.content();
const asset = html.match(/assets\/index-[A-Za-z0-9_-]+\.js/)?.[0] ?? null;

// Settings
const start = frames.length;
await p.getByRole('button', { name: 'Settings', exact: true }).first().click();
await p.waitForTimeout(1500);
const settingsText = await p.locator('main').innerText();
const parsed = frames.slice(start).map((f) => { try { const j = JSON.parse(f.raw); return { dir: f.dir, type: j.type, name: j.name, code: j.payload?.code ?? null }; } catch { return { dir: f.dir, raw: f.raw.slice(0, 60) }; } });
// Data panel
await p.getByRole('button', { name: 'Data', exact: true }).first().click();
await p.waitForTimeout(500);
const dataText = await p.locator('main').innerText();
// bundle approval text
const bundleHasStaleApproval = await p.evaluate(async () => {
  const s = document.querySelector('script[type=module]');
  if (!s) return null;
  const text = await (await fetch(s.src)).text();
  return /no approval-response command/i.test(text);
});
console.log(JSON.stringify({
  status: resp?.status(),
  asset,
  settings: { commands: parsed.filter((x) => x.dir === 'sent' && x.name === 'SETTINGS_STATUS').length, responses: parsed.filter((x) => x.dir === 'recv' && (x.name === 'ACK' || x.name === 'ERROR' || x.name === 'SETTINGS_STATUS_RESULT')).map((x) => x.name + ':' + (x.code ?? '')), textHead: settingsText.slice(0, 260) },
  data: { staleGapClaim: /no workspace-file API yet/i.test(dataText), textHead: dataText.slice(0, 300) },
  bundleHasStaleApproval,
  consoleErrors: errors,
}, null, 2));
await b.close();
