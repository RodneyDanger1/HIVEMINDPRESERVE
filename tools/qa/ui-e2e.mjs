#!/usr/bin/env node
/**
 * Local UI acceptance: launch the release server, drive Edge over CDP, open
 * Settings -> Provider, save a TokenLab profile, then read the saved JSON back.
 * No provider key is used and no model call is made.
 *
 * Usage: node tools/qa/ui-e2e.mjs [screenshot.png]
 */
import { spawn } from 'node:child_process';
import { existsSync, mkdtempSync, readFileSync, rmSync, writeFileSync } from 'node:fs';
import { tmpdir } from 'node:os';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..', '..');
const exe = path.join(root, 'target', 'release', 'hiverouter.exe');
const edge = process.env.HIVEROUTER_E2E_EDGE
  || 'C:\\Program Files (x86)\\Microsoft\\Edge\\Application\\msedge.exe';
const viewportWidth = Number(process.env.HIVEROUTER_E2E_WIDTH || 1440);
const viewportHeight = Number(process.env.HIVEROUTER_E2E_HEIGHT || 900);
const screenshot = process.argv[2] || path.join(
  root, 'docs', 'evidence', 'live',
  `ui-e2e-provider-save-${viewportWidth}x${viewportHeight}.png`,
);
const dataDir = mkdtempSync(path.join(tmpdir(), 'hmars-ui-e2e-'));
const profileFile = path.join(dataDir, 'provider.json');
const edgeProfile = path.join(dataDir, 'edge-profile');
const port = 8099;
const cdpPort = 9223;

const sleep = (ms) => new Promise((resolve) => setTimeout(resolve, ms));
const healthUrl = `http://127.0.0.1:${port}/health`;
let server = null;
let browser = null;
let ws = null;

async function waitForHealth(timeoutMs = 30_000) {
  const deadline = Date.now() + timeoutMs;
  while (Date.now() < deadline) {
    try {
      const response = await fetch(healthUrl);
      if (response.ok) return await response.json();
    } catch {
      // not up yet
    }
    await sleep(300);
  }
  throw new Error('server did not become healthy');
}

async function cdpTarget() {
  const deadline = Date.now() + 15_000;
  while (Date.now() < deadline) {
    try {
      const response = await fetch(`http://127.0.0.1:${cdpPort}/json/list`);
      const pages = await response.json();
      const page = pages.find((candidate) => candidate.type === 'page');
      if (page) return page;
    } catch {
      // browser still starting
    }
    await sleep(250);
  }
  throw new Error('Edge CDP endpoint did not appear');
}

async function main() {
  if (!existsSync(exe)) throw new Error(`missing ${exe}; build release first`);
  if (!existsSync(edge)) throw new Error(`missing Edge at ${edge}`);
  try {
    const response = await fetch(healthUrl);
    if (response.ok) throw new Error(`port ${port} already has a healthy server; stop it first`);
  } catch (error) {
    if (error instanceof Error && error.message.includes('already has')) throw error;
  }

  const launchToken = 'e2elaunchtoken' + Date.now().toString(16);
  const env = { ...process.env, HIVEROUTER_PROVIDER_FILE: profileFile, HIVEROUTER_LAUNCH_TOKEN: launchToken };
  delete env.DEEPSEEK_API_KEY;
  delete env.HIVEROUTER_API_KEY;
  delete env.HIVEROUTER_BASE_URL;
  delete env.HIVEROUTER_WIRE_MODEL;
  delete env.HIVEROUTER_API_TYPE;
  server = spawn(exe, [
    '--bind', `127.0.0.1:${port}`,
    '--data-dir', dataDir,
    '--workspace-root', root,
  ], { cwd: root, env, stdio: 'ignore', windowsHide: true });
  const health = await waitForHealth();
  if (health.status !== 'degraded') throw new Error(`expected no-key degraded health, got ${health.status}`);

  browser = spawn(edge, [
    '--headless=new', '--disable-gpu', '--no-first-run',
    `--remote-debugging-port=${cdpPort}`, `--user-data-dir=${edgeProfile}`,
    'about:blank',
  ], { stdio: 'ignore', windowsHide: true });
  const page = await cdpTarget();
  ws = new WebSocket(page.webSocketDebuggerUrl);
  let id = 0;
  const pending = new Map();
  ws.onmessage = (event) => {
    const message = JSON.parse(event.data);
    if (message.id && pending.has(message.id)) {
      pending.get(message.id)(message);
      pending.delete(message.id);
    }
  };
  await new Promise((resolve, reject) => { ws.onopen = resolve; ws.onerror = reject; });
  const send = (method, params = {}) => new Promise((resolve) => {
    const next = ++id;
    pending.set(next, resolve);
    ws.send(JSON.stringify({ id: next, method, params }));
  });
  const evaluate = async (expression) => {
    const reply = await send('Runtime.evaluate', { expression, returnByValue: true, awaitPromise: true });
    return reply.result?.result?.value;
  };

  await send('Page.enable');
  await send('Page.navigate', { url: `http://127.0.0.1:${port}/?launch=${launchToken}` });
  await send('Emulation.setDeviceMetricsOverride', {
    width: viewportWidth,
    height: viewportHeight,
    deviceScaleFactor: 1,
    mobile: viewportWidth < 720,
  });
  const connectDeadline = Date.now() + 15_000;
  while (Date.now() < connectDeadline) {
    if (await evaluate("document.body.innerText.includes('CONNECTED')")) break;
    await sleep(300);
  }
  if (!(await evaluate("document.body.innerText.includes('CONNECTED')"))) {
    throw new Error('UI did not report CONNECTED');
  }
  if (viewportWidth < 720) {
    const narrow = await evaluate("({overflow: document.documentElement.scrollWidth > window.innerWidth + 1, text: document.body.innerText.includes('HiveMind')})");
    if (!narrow || narrow.overflow || !narrow.text) {
      throw new Error(`375px app-shell check failed: ${JSON.stringify(narrow)}`);
    }
  }
  await evaluate("(()=>{const b=Array.from(document.querySelectorAll('button')).find(x=>x.textContent.trim()==='Settings'); if(!b) return false; b.click(); return true;})()");
  await sleep(700);
  const providerClicked = await evaluate("(()=>{const b=Array.from(document.querySelectorAll('button')).find(x=>/provider/i.test(x.textContent)); if(!b) return false; b.click(); return true;})()");
  if (!providerClicked) throw new Error('Provider tab not found');
  await sleep(700);
  const optionValues = await evaluate("Array.from(document.querySelectorAll('select')).map(s=>Array.from(s.options).map(o=>o.value))");
  if (!Array.isArray(optionValues[0]) || !optionValues[0].includes('tokenlabs')) {
    throw new Error(`API type selector missing tokenlabs: ${JSON.stringify(optionValues)}`);
  }

  const setField = async (selector, value) => {
    const expression = `(()=>{const el=document.querySelector(${JSON.stringify(selector)}); if(!el) return false; const setter=Object.getOwnPropertyDescriptor(Object.getPrototypeOf(el),'value').set; setter.call(el,${JSON.stringify(value)}); el.dispatchEvent(new Event('input',{bubbles:true})); el.dispatchEvent(new Event('change',{bubbles:true})); return true;})()`;
    if (!(await evaluate(expression))) throw new Error(`field not found: ${selector}`);
  };
  await setField('select', 'tokenlabs');
  await setField('input', 'https://api.tokenlab.amd.com/v1');
  const inputs = await evaluate("Array.from(document.querySelectorAll('input')).map(i=>({type:i.type,value:i.value,placeholder:i.placeholder}))");
  if (!Array.isArray(inputs) || inputs.length < 2) throw new Error(`expected at least 2 inputs, got ${JSON.stringify(inputs)}`);
  await evaluate("(()=>{const el=document.querySelectorAll('input')[1]; const setter=Object.getOwnPropertyDescriptor(Object.getPrototypeOf(el),'value').set; setter.call(el,'DeepSeek-V4.1-Flash'); el.dispatchEvent(new Event('input',{bubbles:true})); el.dispatchEvent(new Event('change',{bubbles:true})); return true;})()");
  const saved = await evaluate("(()=>{const b=Array.from(document.querySelectorAll('button')).find(x=>/save provider profile/i.test(x.textContent)); if(!b) return false; b.click(); return true;})()");
  if (!saved) throw new Error('Save provider profile button not found');
  await sleep(1200);
  if (!existsSync(profileFile)) throw new Error(`provider profile was not written: ${profileFile}`);
  const profile = JSON.parse(readFileSync(profileFile, 'utf8'));
  if (profile.api_type !== 'tokenlabs') throw new Error(`api_type not saved: ${JSON.stringify(profile)}`);
  if (profile.base_url !== 'https://api.tokenlab.amd.com/v1') throw new Error(`base_url not saved: ${JSON.stringify(profile)}`);
  if (profile.wire_model !== 'DeepSeek-V4.1-Flash') throw new Error(`wire_model not saved: ${JSON.stringify(profile)}`);
  const shot = await send('Page.captureScreenshot', { format: 'png' });
  writeFileSync(screenshot, Buffer.from(shot.result.data, 'base64'));
  console.log(JSON.stringify({ ok: true, health: health.status, profile, screenshot }, null, 2));
}

try {
  await main();
} finally {
  if (ws) ws.close();
  if (browser) {
    try {
      if (process.platform === 'win32') {
        spawn('taskkill', ['/PID', String(browser.pid), '/T', '/F'], { stdio: 'ignore', windowsHide: true });
      } else {
        browser.kill('SIGKILL');
      }
    } catch {
      // best effort cleanup
    }
  }
  if (server) {
    try { server.kill(); } catch { /* best effort */ }
  }
  await sleep(500);
  try { rmSync(dataDir, { recursive: true, force: true }); } catch { /* best effort */ }
}
