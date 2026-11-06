import { describe, expect, it, vi } from 'vitest';
import { beeAngle, clampZoom, handleCanvasShortcut, nextRovingIndex } from '../src/a11y/keyboard';

function event(overrides: Partial<Parameters<typeof handleCanvasShortcut>[0]> = {}) {
  return {
    key: '0',
    ctrlKey: false,
    metaKey: false,
    shiftKey: false,
    altKey: false,
    target: null,
    ...overrides,
  };
}

describe('keyboard + zoom gates (R-U7 / G-UI-2)', () => {
  it('Ctrl+0 recentres on Windows', () => {
    const recenter = vi.fn();
    const handled = handleCanvasShortcut(event({ ctrlKey: true }), {
      recenter,
      zoomIn: vi.fn(),
      zoomOut: vi.fn(),
      fitView: vi.fn(),
    });
    expect(handled).toBe(true);
    expect(recenter).toHaveBeenCalledOnce();
  });

  it('Meta+0 is accepted only as the macOS alias', () => {
    const recenter = vi.fn();
    handleCanvasShortcut(event({ metaKey: true }), {
      recenter,
      zoomIn: vi.fn(),
      zoomOut: vi.fn(),
      fitView: vi.fn(),
    });
    expect(recenter).toHaveBeenCalledOnce();
  });

  it('accepts physical Digit0 code when key differs (automation/layout)', () => {
    const recenter = vi.fn();
    const handled = handleCanvasShortcut(event({ ctrlKey: true, key: 'x', code: 'Digit0' }), {
      recenter,
      zoomIn: vi.fn(),
      zoomOut: vi.fn(),
      fitView: vi.fn(),
    });
    expect(handled).toBe(true);
    expect(recenter).toHaveBeenCalledOnce();
  });

  it('never hijacks typing controls', () => {
    const input = { tagName: 'INPUT', isContentEditable: false } as unknown as HTMLElement;
    const recenter = vi.fn();
    const handled = handleCanvasShortcut(event({ ctrlKey: true, target: input }), {
      recenter,
      zoomIn: vi.fn(),
      zoomOut: vi.fn(),
      fitView: vi.fn(),
    });
    expect(handled).toBe(false);
    expect(recenter).not.toHaveBeenCalled();
  });

  it('clamps zoom to the frozen 0.1..3.0 range', () => {
    expect(clampZoom(0.01)).toBe(0.1);
    expect(clampZoom(9)).toBe(3);
    expect(clampZoom(Number.NaN)).toBe(0.1);
  });

  it('bee orientation follows the data direction', () => {
    expect(beeAngle({ x: 0, y: 0 }, { x: 10, y: 0 })).toBeCloseTo(0);
    expect(beeAngle({ x: 0, y: 0 }, { x: 0, y: 10 })).toBeCloseTo(Math.PI / 2);
  });

  it('roving focus wraps', () => {
    expect(nextRovingIndex(0, -1, 6)).toBe(5);
    expect(nextRovingIndex(5, 1, 6)).toBe(0);
  });
});
