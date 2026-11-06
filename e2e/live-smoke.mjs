import { chromium } from '@playwright/test';
const browser = await chromium.launch({ channel: 'chrome', headless: true });
const page = await browser.newPage({ viewport: { width: 1440, height: 900 } });
const errors = [];
page.on('console', m => { if (m.type() === 'error') errors.push(m.text()); });
page.on('pageerror', e => errors.push(String(e)));
const resp = await page.goto('http://127.0.0.1:8799/', { waitUntil: 'networkidle', timeout: 30000 });
const title = await page.title();
const text = await page.locator('body').innerText();
await page.screenshot({ path: '../docs/trace/live/app-boot.png', fullPage: false });
const counts = await page.evaluate(() => ({
  buttons: document.querySelectorAll('button').length,
  nav: document.querySelectorAll('nav *').length,
  svg: document.querySelectorAll('svg').length,
  root: !!document.querySelector('#root'),
  bodyTextLen: document.body.innerText.length,
}));
console.log(JSON.stringify({ status: resp?.status(), title, counts, errors, textHead: text.slice(0, 800) }, null, 2));
await browser.close();
