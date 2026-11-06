import { chromium } from '@playwright/test';
const browser = await chromium.launch({ channel: 'chrome', headless: true });
const page = await browser.newPage();
const frames = [];
page.on('websocket', ws => {
  ws.on('framesent', f => frames.push({ dir: 'sent', payload: String(f.payload).slice(0, 500) }));
  ws.on('framereceived', f => frames.push({ dir: 'recv', payload: String(f.payload).slice(0, 500) }));
});
await page.goto('http://127.0.0.1:8799/', { waitUntil: 'networkidle', timeout: 30000 });
await page.getByRole('button', { name: 'Data', exact: true }).click();
await page.waitForTimeout(2500);
const bodyText = await page.locator('body').innerText();
const hasNote = bodyText.includes('note.txt');
const hasReadme = bodyText.includes('README.md');
const mainText = await page.locator('main').innerText();
console.log(JSON.stringify({ hasNote, hasReadme, frames: frames.filter(f => /FILE|file|TREE|ERROR/.test(f.payload)).slice(-20), mainText: mainText.slice(0, 1200) }, null, 2));
await browser.close();
