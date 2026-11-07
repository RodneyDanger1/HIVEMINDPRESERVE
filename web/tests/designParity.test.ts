import { describe, expect, it } from 'vitest';
import { LAYOUT } from '../src/tokens/tokens';

const SOURCE = import.meta.glob('../src/**/*.{ts,tsx}', {
  query: '?raw',
  import: 'default',
  eager: true,
}) as Record<string, string>;

function sourceOf(suffix: string): string {
  const key = Object.keys(SOURCE).find((candidate) => candidate.endsWith(suffix));
  return key === undefined ? '' : SOURCE[key];
}

describe('HiveMind spatial design parity pins', () => {
  it('pins the authoritative board-07 geometry', () => {
    expect(LAYOUT.topbar).toBe(52);
    expect(LAYOUT.navRail).toBe(72);
    expect(LAYOUT.library).toBe(240);
    expect(LAYOUT.chat).toEqual({ width: 400, height: 480, radius: 18 });
    expect(LAYOUT.hud).toEqual({ width: 460, height: 44 });
    expect(LAYOUT.minimap).toEqual({ width: 220, height: 150 });
    expect(LAYOUT.cards.superagent).toEqual({ width: 420, height: 150, radius: 14 });
    expect(LAYOUT.cards.worker).toEqual({ width: 320, height: 96, radius: 12 });
    expect(LAYOUT.cards.tool).toEqual({ width: 420, height: 240, radius: 14 });
  });

  it('uses the geometry constants in the canvas, rail, and panel', () => {
    expect(sourceOf('/flow/graph.ts')).toContain('LAYOUT.cards.superagent');
    expect(sourceOf('/flow/Canvas.tsx')).toContain('LAYOUT.minimap.width');
    expect(sourceOf('/panels/LibraryPanel.tsx')).toContain('width: LAYOUT.library');
    expect(sourceOf('/panels/Header.tsx')).toContain('h-[52px]');
  });

  it('has no user-facing HiveRouter or mojibake in the UI source', () => {
    for (const suffix of [
      '/App.tsx',
      '/panels/Header.tsx',
      '/panels/NavRail.tsx',
      '/panels/LibraryPanel.tsx',
      '/chat/ChatCore.tsx',
      '/state/strings.ts',
    ]) {
      const source = sourceOf(suffix);
      expect(source, suffix).not.toMatch(/HiveRouter/);
      expect(source, suffix).not.toMatch(/Â·|â€”|â€™/);
    }
  });
});
