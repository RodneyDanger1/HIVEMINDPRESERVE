import { chromium } from '@playwright/test';
const browser = await chromium.launch({ channel: 'chrome', headless: true });
const page = await browser.newPage();
const bad = [];
page.on('response', r => { if (r.status() >= 400) bad.push({ url: r.url(), status: r.status() }); });
page.on('requestfailed', r => bad.push({ url: r.url(), failure: r.failure()?.errorText }));
await page.goto('http://127.0.0.1:8799/', { waitUntil: 'networkidle', timeout: 30000 });
console.log(JSON.stringify(bad, null, 2));
// count key surface markers
const info = await page.evaluate(() => ({
  buttons: [...document.querySelectorAll('button')].map(b => (b.getAttribute('aria-label') || b.textContent || '').trim()).filter(Boolean),
  navItems: [...document.querySelectorAll('nav button, nav a')].map(x => (x.getAttribute('aria-label') || x.textContent || '').trim()),
  tabs: [...document.querySelectorAll('[role=tab]')].map(x => (x.textContent || '').trim()),
}));
console.log(JSON.stringify(info, null, 2));
await browser.close();
