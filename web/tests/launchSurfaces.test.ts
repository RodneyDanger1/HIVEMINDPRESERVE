import { describe, expect, it } from 'vitest';

const SOURCE = import.meta.glob('../src/**/*.{ts,tsx}', {
  query: '?raw',
  import: 'default',
  eager: true,
}) as Record<string, string>;

function sourceOf(suffix: string): string {
  const key = Object.keys(SOURCE).find((candidate) => candidate.endsWith(suffix));
  return key === undefined ? '' : SOURCE[key];
}

describe('launch surfaces: crash boundary and skippable splash', () => {
  it('wraps the app frame in the crash boundary', () => {
    const app = sourceOf('/App.tsx');
    expect(app).toContain("from './splash/CrashScreen'");
    expect(app).toContain('<AppErrorBoundary>');
    const boundaryIndex = app.indexOf('<AppErrorBoundary>');
    const frameIndex = app.indexOf('<AppFrame />');
    const splashIndex = app.indexOf('<HiveMindSplash />');
    expect(boundaryIndex).toBeGreaterThan(-1);
    // the splash must sit inside the boundary, so a crash cannot hide behind it
    expect(frameIndex).toBeGreaterThan(boundaryIndex);
    expect(splashIndex).toBeGreaterThan(frameIndex);
  });

  it('turns a render error into a named screen with a reload action', () => {
    const crash = sourceOf('/splash/CrashScreen.tsx');
    expect(crash).toContain('static getDerivedStateFromError');
    expect(crash).toContain('componentDidCatch');
    expect(crash).toContain('reloadInterface');
    expect(crash).toContain('data-testid="ui-crash-screen"');
    expect(crash).toContain('role="alert"');
    expect(crash).toContain('STRINGS.crash.title');
    expect(crash).toContain('STRINGS.crash.reload');
    // a crash screen must not dump component state or credentials
    expect(crash).not.toContain('JSON.stringify');
    expect(crash).not.toContain('localStorage');
  });

  it('keeps the component stack and drops everything else from the crash log', () => {
    const crash = sourceOf('/splash/CrashScreen.tsx');
    expect(crash).toContain('info.componentStack');
    expect(crash).toContain('describeError(error)');
  });

  it('makes the splash skippable without changing its identity hooks', () => {
    const splash = sourceOf('/splash/HiveMindSplash.tsx');
    expect(splash).toContain('data-testid="hivemind-splash"');
    expect(splash).toContain('data-testid="splash-skip"');
    expect(splash).toContain("event.key === 'Escape'");
    expect(splash).toContain('splashLeaveDelayMs');
    // the veil must not be an unbounded gate
    expect(splash).toContain('SPLASH_MIN_VISIBLE_MS');
    expect(splash).toContain('SPLASH_FADE_MS');
  });

  it('does not put a live region on the whole veil', () => {
    const splash = sourceOf('/splash/HiveMindSplash.tsx');
    const rootStart = splash.indexOf('className={`splash-root');
    const rootEnd = splash.indexOf('>', rootStart);
    const rootTag = splash.slice(rootStart, rootEnd);
    expect(rootTag).not.toContain('aria-live');
    expect(rootTag).not.toContain('role="status"');
    expect(splash).toContain('role="status"');
  });

  it('persists the reduced-motion override through the store', () => {
    const store = sourceOf('/state/store.ts');
    expect(store).toContain('writeReducedMotionOverride(value)');
    expect(store).toContain('readReducedMotionOverride()');
    expect(store).not.toContain('reducedMotionOverride: false,');
  });
});
