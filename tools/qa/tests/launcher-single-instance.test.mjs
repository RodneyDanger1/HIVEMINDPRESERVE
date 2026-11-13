/**
 * Static single-instance contract for the HiveMind desktop launcher.
 *
 * The launcher is a C# WinForms executable and is NOT run here. These tests
 * read tools/desktop-launcher/HiveMindLauncher.cs as text and pin the pieces
 * that make a second launch focus the already-running app instead of
 * duplicating the server or opening a second window:
 *
 *   1. one named, process-wide Mutex arbitrates the instance;
 *   2. acquisition is non-blocking (WaitOne(0));
 *   3. a launch that loses the Mutex focuses the existing window;
 *   4. the Mutex is released in a finally block;
 *   5. the healthy-server reuse path is gated by reusedExisting before it
 *      focuses a window;
 *   6. focusing recognises both Edge and Chrome windows.
 *
 * Structural checks run against a same-length sanitized copy of the source in
 * which comments and string/char literals are blanked, so braces inside
 * literals cannot confuse block extraction. Text checks (mutex name, browser
 * names) run against the original source at the same offsets.
 */
import test from 'node:test';
import assert from 'node:assert/strict';
import { existsSync, readFileSync, statSync } from 'node:fs';

const LAUNCHER_URL = new URL('../../desktop-launcher/HiveMindLauncher.cs', import.meta.url);

let sourceCache = null;

/** Read the launcher source once per process. */
function launcherSource() {
  if (sourceCache !== null) {
    return sourceCache;
  }
  assert.ok(existsSync(LAUNCHER_URL), `launcher source is missing: ${LAUNCHER_URL.pathname}`);
  sourceCache = readFileSync(LAUNCHER_URL, 'utf8').replace(/^\uFEFF/, '');
  return sourceCache;
}

/**
 * Blank comments and string/char literals while preserving every offset and
 * newline, so indices into the sanitized text also address the original.
 */
function sanitizeCSharp(source) {
  const chars = source.split('');
  const blank = (from, to) => {
    for (let i = from; i < to && i < chars.length; i += 1) {
      if (chars[i] !== '\n' && chars[i] !== '\r') {
        chars[i] = ' ';
      }
    }
  };

  let i = 0;
  while (i < source.length) {
    const ch = source[i];
    const next = source[i + 1];

    if (ch === '/' && next === '/') {
      let end = i + 2;
      while (end < source.length && source[end] !== '\n') {
        end += 1;
      }
      blank(i, end);
      i = end;
      continue;
    }

    if (ch === '/' && next === '*') {
      let end = i + 2;
      while (end < source.length && !(source[end] === '*' && source[end + 1] === '/')) {
        end += 1;
      }
      const stop = Math.min(source.length, end + 2);
      blank(i, stop);
      i = stop;
      continue;
    }

    if (ch === '@' && next === '"') {
      let end = i + 2;
      while (end < source.length) {
        if (source[end] === '"') {
          if (source[end + 1] === '"') {
            end += 2;
            continue;
          }
          end += 1;
          break;
        }
        end += 1;
      }
      blank(i, end);
      i = end;
      continue;
    }

    if (ch === '"' || ch === "'") {
      let end = i + 1;
      while (end < source.length) {
        if (source[end] === '\\') {
          end += 2;
          continue;
        }
        if (source[end] === ch) {
          end += 1;
          break;
        }
        if (source[end] === '\n') {
          break;
        }
        end += 1;
      }
      blank(i, end);
      i = end;
      continue;
    }

    i += 1;
  }

  return chars.join('');
}

/** Return the brace-delimited block that opens after `fromIndex`. */
function blockStartingAt(sanitized, fromIndex, label) {
  const open = sanitized.indexOf('{', fromIndex);
  assert.ok(open >= 0, `no opening brace found for ${label}`);
  let depth = 0;
  for (let i = open; i < sanitized.length; i += 1) {
    if (sanitized[i] === '{') {
      depth += 1;
    } else if (sanitized[i] === '}') {
      depth -= 1;
      if (depth === 0) {
        return { start: open, end: i, body: sanitized.slice(open + 1, i) };
      }
    }
  }
  assert.fail(`unbalanced braces while locating ${label}`);
}

/** Locate a method by its exact signature and return its block. */
function methodBlock(sanitized, signature) {
  const index = sanitized.indexOf(signature);
  assert.ok(index >= 0, `method not found: ${signature}`);
  return blockStartingAt(sanitized, index, signature);
}

test('launcher source exists and is the single-instance entry point', () => {
  const source = launcherSource();
  const stats = statSync(LAUNCHER_URL);
  assert.ok(stats.isFile(), 'launcher source must be a regular file');
  assert.ok(stats.size > 10_000, `launcher source looks truncated (${stats.size} bytes)`);
  assert.match(source, /namespace\s+HiveMindLauncher/, 'expected the HiveMindLauncher namespace');
  assert.match(source, /internal\s+static\s+class\s+Program/, 'expected the Program entry class');
  assert.match(source, /\[STAThread\]\s*private\s+static\s+void\s+Main\s*\(\s*\)/, 'expected the STAThread Main entry point');
});

test('declares one named, process-wide instance Mutex', () => {
  const source = launcherSource();
  const field = source.match(
    /static\s+readonly\s+Mutex\s+(\w+)\s*=\s*new\s+Mutex\s*\(\s*(false|true)\s*,\s*"((?:[^"\\]|\\.)+)"\s*\)\s*;/,
  );
  assert.ok(
    field,
    'expected `static readonly Mutex <name> = new Mutex(initiallyOwned, "name")`; an unnamed Mutex would not arbitrate across launches',
  );
  assert.equal(field[1], 'InstanceMutex', 'the arbitrating Mutex must be the InstanceMutex field');
  assert.equal(field[2], 'false', 'the Mutex must be created unowned so the first launch owns it');
  const mutexName = field[3].replace(/\\\\/g, '\\');
  assert.ok(
    mutexName.startsWith('Local\\') || mutexName.startsWith('Global\\'),
    `Mutex name must be a Local\\ or Global\\ session name, got ${JSON.stringify(mutexName)}`,
  );
  assert.match(mutexName, /HiveMind/i, 'Mutex name must identify this product');
});

test('arbitrates without blocking: instanceOwned = InstanceMutex.WaitOne(0)', () => {
  const sanitized = sanitizeCSharp(launcherSource());
  const main = methodBlock(sanitized, 'private static void Main()');
  assert.match(
    main.body,
    /instanceOwned\s*=\s*InstanceMutex\.WaitOne\s*\(\s*0\s*\)\s*;/,
    'expected the non-blocking acquisition `instanceOwned = InstanceMutex.WaitOne(0);`',
  );
  assert.doesNotMatch(
    main.body,
    /InstanceMutex\s*\.\s*WaitOne\s*\(\s*\)/,
    'InstanceMutex.WaitOne() without a timeout would block a second launch',
  );
  const abandoned = /catch\s*\(\s*AbandonedMutexException\s*\)/.exec(main.body);
  assert.ok(abandoned, 'an abandoned Mutex must be caught instead of crashing the launcher');
});

test('a launch that loses the Mutex focuses the existing window and returns', () => {
  const sanitized = sanitizeCSharp(launcherSource());
  const main = methodBlock(sanitized, 'private static void Main()');
  const guard = /if\s*\(\s*!instanceOwned\s*\)/.exec(main.body);
  assert.ok(guard, 'expected the `if (!instanceOwned)` contention path');
  const contention = blockStartingAt(sanitized, main.start + 1 + guard.index, '!instanceOwned block');
  assert.match(contention.body, /FocusExistingWindow\s*\(\s*\d*\s*\)\s*;/, 'contention must focus the existing window');
  assert.match(contention.body, /return\s*;/, 'contention must return and never start a second server');
  const focusAt = contention.body.indexOf('FocusExistingWindow');
  const returnAt = contention.body.indexOf('return');
  assert.ok(focusAt >= 0 && focusAt < returnAt, 'focus must happen before the contention path returns');
});

test('releases the Mutex exactly once, inside the finally block', () => {
  const sanitized = sanitizeCSharp(launcherSource());
  const main = methodBlock(sanitized, 'private static void Main()');
  const finallyMatch = /\bfinally\b/.exec(main.body);
  assert.ok(finallyMatch, 'expected a finally block so every early return releases the Mutex');
  const finallyBlock = blockStartingAt(sanitized, main.start + 1 + finallyMatch.index, 'finally block');
  const releases = [...main.body.matchAll(/InstanceMutex\s*\.\s*ReleaseMutex\s*\(\s*\)\s*;/g)];
  assert.equal(releases.length, 1, `expected exactly one InstanceMutex.ReleaseMutex() in Main, found ${releases.length}`);
  const releaseAt = main.start + 1 + releases[0].index;
  assert.ok(
    releaseAt > finallyBlock.start && releaseAt < finallyBlock.end,
    'InstanceMutex.ReleaseMutex() must sit inside the finally block, not on a success path',
  );
});

test('the healthy-server reuse path is gated by reusedExisting before focusing', () => {
  const sanitized = sanitizeCSharp(launcherSource());
  const main = methodBlock(sanitized, 'private static void Main()');
  const guard = /reusedExisting\s*&&\s*FocusExistingWindow\s*\(\s*\d*\s*\)/.exec(main.body);
  assert.ok(
    guard,
    'expected the short-circuit guard `reusedExisting && FocusExistingWindow()` so a freshly started server still opens its window',
  );
  const declaration = /bool\s+reusedExisting\s*=\s*port\s*!=\s*0\s*;/.exec(main.body);
  assert.ok(declaration, 'expected `bool reusedExisting = port != 0;`');
  assert.ok(declaration.index < guard.index, 'reusedExisting must be computed before the guarded focus call');
  const findPort = /FindReusablePort\s*\(\s*\)/.exec(main.body);
  assert.ok(findPort, 'expected the health probe FindReusablePort()');
  assert.ok(findPort.index < declaration.index, 'the reuse guard must be derived from the health probe result');
});

test('FocusExistingWindow recognises both Edge and Chrome windows', () => {
  const source = launcherSource();
  const sanitized = sanitizeCSharp(source);
  const focus = methodBlock(sanitized, 'private static bool FocusExistingWindow(int timeoutMs = 0)');
  const originalBody = source.slice(focus.start, focus.end + 1);

  assert.match(
    originalBody,
    /new\s*\[\s*\]\s*\{\s*"msedge"\s*,\s*"chrome"\s*\}/,
    'focus must enumerate both msedge and chrome process names',
  );
  assert.match(
    originalBody,
    /Process\.GetProcessesByName\s*\(\s*name\s*\)/,
    'each process name must be resolved through Process.GetProcessesByName',
  );
  assert.match(
    originalBody,
    /MainWindowTitle[\s\S]{0,120}?IndexOf\s*\(\s*"HiveMind"/,
    'focus must match the HiveMind window title before raising it',
  );
  assert.match(
    originalBody,
    /SetForegroundWindow\s*\(\s*process\.MainWindowHandle\s*\)\s*;[\s\S]{0,40}?return\s+true\s*;/,
    'a matched HiveMind window must be raised and reported as found',
  );
  assert.match(originalBody, /return\s+false\s*;/, 'focus must return false when no HiveMind window exists');
});
