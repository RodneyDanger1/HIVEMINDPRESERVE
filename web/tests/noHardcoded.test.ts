import { describe, expect, it } from 'vitest';
import { SWATCHES } from '../src/tokens/tokens';

const TS_FILES = import.meta.glob('../src/**/*.{ts,tsx}', {
  query: '?raw',
  import: 'default',
  eager: true,
}) as Record<string, string>;
const CSS_FILES = import.meta.glob('../src/**/*.css', {
  query: '?raw',
  import: 'default',
  eager: true,
}) as Record<string, string>;

const TOKEN_FILES_ALLOWED_HEX = ['/tokens/tokens.ts', '/styles/tokens.css'];
const BANNED_ILLUSTRATIVE = ['98.2%', '12.4k', '4 SUPERAGENTS', '12 WORKERS'];

describe('G-UI-11 token isolation and no illustrative hardcoding', () => {
  it('zero hex literals outside the token module', () => {
    const offenders: string[] = [];
    for (const [path, source] of Object.entries({ ...TS_FILES, ...CSS_FILES })) {
      if (TOKEN_FILES_ALLOWED_HEX.some((allowed) => path.endsWith(allowed))) {
        continue;
      }
      if (/#[0-9a-fA-F]{6}\b/.test(source)) {
        offenders.push(path);
      }
    }
    expect(offenders).toEqual([]);
  });

  it('token module still carries exactly the 20 swatches with hex values', () => {
    expect(SWATCHES).toHaveLength(20);
    const source = TS_FILES['../src/tokens/tokens.ts'] ?? '';
    expect(source).toContain('#F8F4EB');
  });

  it('never hardcodes the PDF illustrative strings outside the string table', () => {
    const offenders: string[] = [];
    for (const [path, source] of Object.entries(TS_FILES)) {
      if (path.endsWith('/state/strings.ts')) {
        continue;
      }
      for (const banned of BANNED_ILLUSTRATIVE) {
        if (source.includes(banned)) {
          offenders.push(`${path}: ${banned}`);
        }
      }
    }
    expect(offenders).toEqual([]);
  });

  it('UI components do not import FigCraft assets while the gate is closed', () => {
    const manifest = TS_FILES['../src/assets/manifest.ts'] ?? '';
    expect(manifest).toContain('allowFrontend');
    const consumers = Object.entries(TS_FILES).filter(
      ([path, source]) =>
        !path.endsWith('/assets/manifest.ts') &&
        (source.includes('figcraft') || source.includes('a-foundations') || source.includes('b-boards')),
    );
    expect(consumers.map(([path]) => path)).toEqual([]);
  });
});
