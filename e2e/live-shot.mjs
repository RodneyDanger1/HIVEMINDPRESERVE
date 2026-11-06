import { chromium } from '@playwright/test';
const browser = await chromium.launch({ channel: 'chrome', headless: true });
const page = await browser.newPage({ viewport: { width: 1440, height: 900 } });
await page.goto('http://127.0.0.1:8799/', { waitUntil: 'networkidle', timeout: 30000 });
await page.screenshot({ path: '../docs/trace/live/app-boot.png', fullPage: false });
await page.screenshot({ path: '../docs/trace/live/app-boot-full.png', fullPage: true });
console.log('screenshots written');
await browser.close();
