import { describe, expect, it } from 'vitest';

import {
  FIGCRAFT_BOARDS,
  FIGCRAFT_GROUPS,
} from '../src/figcraft/catalog';
import { nextFigCraftGroupIndex } from '../src/figcraft/FigCraftGallery';
import { APP_VERSION } from '../src/version';

/**
 * Coverage for the 25 previously-unowned UI assets: `web/index.html`,
 * `web/package-lock.json`, `SOURCES.json` and the 21 served FigCraft SVG boards.
 * These files are build inputs, not source to edit; this suite detects drift
 * between the generated catalog and the served bytes without touching either.
 *
 * Files are read through Vite's raw glob (the web tsconfig intentionally has no
 * Node type definitions); hashes use the Web Crypto digest available in Vitest's
 * Node runtime.
 */

const FIGCRAFT_RAW = import.meta.glob('../public/figcraft/*', {
  query: '?raw',
  import: 'default',
  eager: true,
}) as Record<string, string>;

const BUILD_INPUTS = import.meta.glob('../{index.html,package-lock.json}', {
  query: '?raw',
  import: 'default',
  eager: true,
}) as Record<string, string>;

function basename(modulePath: string): string {
  return modulePath.split('/').slice(-1)[0] ?? modulePath;
}

function entryFor(fileName: string): string | undefined {
  const key = Object.keys(FIGCRAFT_RAW).find((candidate) => basename(candidate) === fileName);
  return key === undefined ? undefined : FIGCRAFT_RAW[key];
}

async function sha256(text: string): Promise<string> {
  const digest = await crypto.subtle.digest('SHA-256', new TextEncoder().encode(text));
  return [...new Uint8Array(digest)]
    .map((byte) => byte.toString(16).padStart(2, '0'))
    .join('');
}

describe('served FigCraft asset set (21 boards + SOURCES.json)', () => {
  it('serves exactly the catalog boards and nothing invented', () => {
    expect(FIGCRAFT_BOARDS).toHaveLength(21);
    const files = Object.keys(FIGCRAFT_RAW).map(basename).sort();
    const svgs = files.filter((file) => file.endsWith('.svg'));
    expect(svgs).toHaveLength(21);
    expect(files).toContain('SOURCES.json');
    expect(files).toHaveLength(22);
    expect([...new Set(FIGCRAFT_BOARDS.map((board) => board.svgFile))].sort()).toEqual(svgs);

    const sources = JSON.parse(entryFor('SOURCES.json') ?? '{}') as {
      counts?: { boards?: number };
    };
    expect(sources.counts?.boards).toBe(21);
  });

  it('matches every catalog board hash to the served SVG bytes', async () => {
    for (const board of FIGCRAFT_BOARDS) {
      const raw = entryFor(board.svgFile);
      expect(raw, board.svgFile).toBeDefined();
      expect(await sha256(raw ?? ''), board.svgFile).toBe(board.sha256);
    }
  });

  it('every board owner is a declared group', () => {
    const groupIds = new Set(FIGCRAFT_GROUPS.map((group) => group.id));
    for (const board of FIGCRAFT_BOARDS) {
      expect(groupIds.has(board.owner), `${board.id} owner ${board.owner}`).toBe(true);
    }
    expect(FIGCRAFT_GROUPS.map((group) => group.id)).toEqual([
      'figcraft-hivemind',
      'figcraft-a',
      'figcraft-b',
    ]);
  });
});

describe('UI build inputs', () => {
  it('keeps web/index.html the Vite entry with the app mount point', () => {
    const html = Object.values(BUILD_INPUTS).find((source) =>
      source.includes('<div id="root">'),
    );
    expect(html).toBeDefined();
    expect(html ?? '').toContain('src="/src/main.tsx"');
    expect(html ?? '').toContain('<title>HiveMind');
  });

  it('keeps a parseable pinned package-lock and a version identity in sync', () => {
    const lockRaw = Object.values(BUILD_INPUTS).find((source) =>
      source.includes('"lockfileVersion"'),
    );
    expect(lockRaw).toBeDefined();
    const lock = JSON.parse(lockRaw ?? '{}') as {
      lockfileVersion?: number;
      packages?: Record<string, unknown>;
    };
    expect(lock.lockfileVersion).toBeGreaterThanOrEqual(3);
    expect(lock.packages).toBeDefined();
    expect(APP_VERSION).toBe('1.2.0-dev');
  });
});

describe('FigCraft gallery empty-catalog guard', () => {
  it('never computes a NaN index for a non-existent group set', () => {
    expect(nextFigCraftGroupIndex(0, 0, 1)).toBeNull();
    expect(nextFigCraftGroupIndex(0, 3, 1)).toBe(1);
    expect(nextFigCraftGroupIndex(2, 3, 1)).toBe(0);
    expect(nextFigCraftGroupIndex(0, 3, -1)).toBe(2);
    expect(nextFigCraftGroupIndex(9, 3, 1)).toBeNull();
  });
});
