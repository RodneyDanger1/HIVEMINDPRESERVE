import { chromium } from 'playwright';
import fs from 'node:fs';
import path from 'node:path';

const OUT = 'C:/Users/Riley/Desktop/hiverouter-v1/ledgers/visual-dock-8099/shots';
fs.mkdirSync(OUT, { recursive: true });
const results = { console: [], pageerrors: [], failedRequests: [], steps: [] };

const shot = async (page, name) => {
  const p = path.join(OUT, name + '.png');
  await page.screenshot({ path: p, fullPage: false });
  return p.replace(/\\/g, '/');
};

const box = async (loc) => {
  try {
    const b = await loc.boundingBox();
    return b ? { x: +b.x.toFixed(1), y: +b.y.toFixed(1), w: +b.width.toFixed(1), h: +b.height.toFixed(1), bottom: +(b.y + b.height).toFixed(1) } : null;
  } catch { return null; }
};

const browser = await chromium.launch({ channel: 'chrome', headless: true, args: ['--no-first-run'] });
const ctx = await browser.newContext({ viewport: { width: 1440, height: 900 } });
const page = await ctx.newPage();
page.on('console', m => { if (m.type() === 'error' || m.type() === 'warning') results.console.push({ type: m.type(), text: m.text().slice(0, 300) }); });
page.on('pageerror', e => results.pageerrors.push(String(e).slice(0, 300)));
page.on('requestfailed', r => results.failedRequests.push({ url: r.url().slice(0, 200), err: r.failure()?.errorText }));

await page.goto('http://127.0.0.1:8099/', { waitUntil: 'domcontentloaded', timeout: 20000 });
await page.waitForTimeout(3500);
const chat = page.locator('section[aria-label="HyperAgent hive chat core"]');
await chat.waitFor({ state: 'visible', timeout: 10000 });
const sep = page.locator('[role="separator"][aria-label="Resize chat panel"]');
const step = async (name, data) => { results.steps.push({ name, ...data }); };
const storeHeight = () => page.evaluate(() => localStorage.getItem('hivemind.chat-height'));

results.steps.push({ name: 'initial-1440', shot: await shot(page, '01-1440-initial'), chat: await box(chat), sep: await box(sep), storedHeight: await storeHeight(), viewport: page.viewportSize() });

// Drag the resize handle upward 120px: pointer leaves the 6px handle almost immediately.
const s = await box(sep);
await page.mouse.move(s.x + s.w / 2, s.y + s.h / 2);
await page.mouse.down();
for (let i = 1; i <= 12; i++) { await page.mouse.move(s.x + s.w / 2, s.y + s.h / 2 - i * 10, { steps: 1 }); await page.waitForTimeout(16); }
await page.mouse.up();
await page.waitForTimeout(250);
results.steps.push({ name: 'drag-leave-handle', shot: await shot(page, '02-1440-after-drag-usual'), chat: await box(chat), storedHeight: await storeHeight() });

// Hover the handle with no button down after the aborted drag: stale ref check.
await page.mouse.move(s.x + s.w / 2, s.y + s.h / 2);
await page.mouse.move(s.x + s.w / 2, s.y + s.h / 2 - 40, { steps: 5 });
await page.waitForTimeout(200);
results.steps.push({ name: 'hover-after-drag-stale', shot: await shot(page, '03-1440-hover-after-drag'), chat: await box(chat), storedHeight: await storeHeight() });

// A correct drag that stays inside the handle is geometrically impossible (handle is 6px tall); record attempt.
results.steps.push({ name: 'handle-metrics', sep: await box(sep), note: 'handlers bound only to separator; no setPointerCapture' });

// CONTROLS tab
const controls = page.getByRole('button', { name: 'CONTROLS', exact: true });
const stateTab = page.getByRole('button', { name: 'STATE & SETTINGS', exact: true });
results.steps.push({ name: 'tab-hit-targets', controls: await box(controls), state: await box(stateTab), hide: await box(page.locator('button[aria-label="Hide the chat dock"]')) });
const before = await box(chat);
await controls.click();
await page.waitForTimeout(350);
const afterCtrl = await box(chat);
results.steps.push({ name: 'controls-open', shot: await shot(page, '04-1440-controls'), before, after: afterCtrl, ariaExpanded: await controls.getAttribute('aria-expanded'), tray: await page.evaluate(() => {
  const b = [...document.querySelectorAll('section[aria-label="HyperAgent hive chat core"] button')].find(x => x.textContent.trim() === 'CONTROLS');
  const t = b?.parentElement?.parentElement?.querySelector('.overflow-y-auto');
  return t ? { clientH: t.clientHeight, scrollH: t.scrollHeight, overflow: t.scrollHeight - t.clientHeight } : null;
}), viewportH: 900 });

await stateTab.click();
await page.waitForTimeout(350);
results.steps.push({ name: 'state-open', shot: await shot(page, '05-1440-state'), chat: await box(chat), ariaExpanded: await stateTab.getAttribute('aria-expanded') });

// HIDE / SHOW
await page.locator('button[aria-label="Hide the chat dock"]').click();
await page.waitForTimeout(400);
const hiddenShot = await shot(page, '06-1440-hidden');
const showBtn = page.getByRole('button', { name: 'Show chat', exact: true }).last();
results.steps.push({ name: 'hidden', shot: hiddenShot, chatCount: await chat.count(), showBtn: await box(showBtn) });
await showBtn.click();
await page.waitForTimeout(500);
results.steps.push({ name: 'shown-again', shot: await shot(page, '07-1440-shown-again'), chat: await box(chat), activeEl: await page.evaluate(() => document.activeElement?.outerHTML.slice(0, 160)) });

// 375 viewport
const ctx2 = await browser.newContext({ viewport: { width: 375, height: 667 } });
const p2 = await ctx2.newPage();
p2.on('console', m => { if (m.type() === 'error') results.console.push({ type: 'error-375', text: m.text().slice(0, 300) }); });
p2.on('pageerror', e => results.pageerrors.push(String(e).slice(0, 300)));
await p2.goto('http://127.0.0.1:8099/', { waitUntil: 'domcontentloaded', timeout: 20000 });
await p2.waitForTimeout(3500);
const chat2 = p2.locator('section[aria-label="HyperAgent hive chat core"]');
await chat2.waitFor({ state: 'visible', timeout: 10000 });
const box2 = async (loc) => { const b = await loc.boundingBox().catch(() => null); return b ? { x: +b.x.toFixed(1), y: +b.y.toFixed(1), w: +b.width.toFixed(1), h: +b.height.toFixed(1), bottom: +(b.y + b.height).toFixed(1) } : null; };
results.steps.push({ name: 'initial-375', shot: (await p2.screenshot({ path: path.join(OUT, '08-375-initial.png') }), '08-375-initial.png'), chat: await box2(chat2) });
await p2.getByRole('button', { name: 'CONTROLS', exact: true }).click();
await p2.waitForTimeout(350);
results.steps.push({ name: 'controls-375', shot: (await p2.screenshot({ path: path.join(OUT, '09-375-controls.png') }), '09-375-controls.png'), chat: await box2(chat2), hide: await box2(p2.locator('button[aria-label="Hide the chat dock"]')), controls: await box2(p2.getByRole('button', { name: 'CONTROLS', exact: true })) });
// try resize on 375
const sep2box = await box2(p2.locator('[role="separator"][aria-label="Resize chat panel"]'));
await p2.mouse.move(sep2box.x + sep2box.w / 2, sep2box.y + sep2box.h / 2);
await p2.mouse.down();
for (let i = 1; i <= 10; i++) { await p2.mouse.move(sep2box.x + sep2box.w / 2, sep2box.y - i * 12, { steps: 1 }); }
await p2.mouse.up();
await p2.waitForTimeout(250);
results.steps.push({ name: 'drag-375', shot: (await p2.screenshot({ path: path.join(OUT, '10-375-after-drag.png') }), '10-375-after-drag.png'), chat: await box2(chat2), storedHeight: await p2.evaluate(() => localStorage.getItem('hivemind.chat-height')) });

// overlap check at 1440: chat vs canvas controls / nav rail geometry
results.overlap = await page.evaluate(() => {
  const chat = document.querySelector('section[aria-label="HyperAgent hive chat core"]');
  const all = [...document.querySelectorAll('button, [role="button"]')].filter(b => b.offsetParent !== null);
  if (!chat) return null;
  const c = chat.getBoundingClientRect();
  const hits = all.filter(b => { const r = b.getBoundingClientRect(); return !(r.right < c.left || r.left > c.right || r.bottom < c.top || r.top > c.bottom); })
    .map(b => ({ label: (b.getAttribute('aria-label') || b.textContent || '').trim().slice(0, 40), r: [Math.round(b.getBoundingClientRect().x), Math.round(b.getBoundingClientRect().y), Math.round(b.getBoundingClientRect().width), Math.round(b.getBoundingClientRect().height)] }));
  return { chatRect: [Math.round(c.x), Math.round(c.y), Math.round(c.width), Math.round(c.height)], overlapped: hits.slice(0, 12) };
});

fs.writeFileSync('C:/Users/Riley/Desktop/hiverouter-v1/ledgers/visual-dock-8099/results.json', JSON.stringify(results, null, 2));
console.log(JSON.stringify(results, null, 2).slice(0, 12000));
await browser.close();
