import { describe, expect, it } from 'vitest';
import { createUiApi } from '../src/api';
import { WsApiClient } from '../src/api/client';

// Raw source/bundle text via Vite's glob transform (no node builtins needed, so
// the guard typechecks under the browser-lib tsconfig used by the web app).
const RAW_SOURCES = import.meta.glob('../src/api/*.ts', {
  query: '?raw',
  import: 'default',
  eager: true,
}) as Record<string, string>;
const BUNDLE_JS = import.meta.glob('../dist/assets/*.js', {
  query: '?raw',
  import: 'default',
  eager: true,
}) as Record<string, string>;

const entrySource = (): string => RAW_SOURCES['../src/api/index.ts'] ?? '';
const mockSource = (): string => RAW_SOURCES['../src/api/mock.ts'] ?? '';

describe('dev-mock module boundary (production entry)', () => {
  it('never statically imports or re-exports ./mock from the API entry', () => {
    const entry = entrySource();
    expect(entry.length).toBeGreaterThan(0);
    const staticImport = /import\s+[^;]*\bfrom\s+['"]\.\/mock['"]/m.test(entry);
    const staticReexport = /export\s+[^;]*\bfrom\s+['"]\.\/mock['"]/m.test(entry);
    expect({ staticImport, staticReexport }).toEqual({ staticImport: false, staticReexport: false });
  });

  it('loads the mock through a dynamic import gated on DEV && VITE_UI_MOCK', () => {
    const entry = entrySource();
    expect(entry).toMatch(/import\(\s*['"]\.\/mock['"]\s*\)/);
    expect(entry).toContain('import.meta.env.DEV');
    expect(entry).toContain('VITE_UI_MOCK');
    const flagIndex = entry.indexOf('USE_DEV_MOCK');
    const dynamicIndex = entry.indexOf("import('./mock')");
    const devFlagIndex = entry.indexOf('import.meta.env.DEV');
    expect(flagIndex).toBeGreaterThan(-1);
    expect(devFlagIndex).toBeGreaterThan(-1);
    expect(dynamicIndex).toBeGreaterThan(flagIndex);
  });

  it('dev mock refuses construction without DEV && VITE_UI_MOCK=1', async () => {
    const { MockApiClient } = await import('../src/api/mock');
    expect(() => new MockApiClient()).toThrow(/dev-only/);
    expect(mockSource()).toContain('VITE_UI_MOCK');
    expect(mockSource()).toContain('import.meta.env.DEV');
  });

  it('createUiApi() resolves to the real WS client on the production path', () => {
    const api = createUiApi();
    expect(api.kind).toBe('ws');
    expect(new WsApiClient({ baseUrl: 'http://127.0.0.1:8080' }).kind).toBe('ws');
  });

  it('production bundle, when built, contains no dev-mock marker', () => {
    const offenders = Object.entries(BUNDLE_JS)
      .filter(([, source]) => source.includes('dev-mock-session-token'))
      .map(([path]) => path);
    expect(offenders).toEqual([]);
    // Guard the guard: the marker must genuinely exist in the dev-only module.
    expect(mockSource()).toContain('dev-mock-session-token');
  });
});
