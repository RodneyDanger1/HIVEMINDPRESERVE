import { describe, expect, it } from 'vitest';
import {
  DECLARED_CONTRAST_ROWS,
  RADII_STOPS,
  SPACING_STOPS,
  SWATCHES,
  TYPE_SCALE,
  swatch,
} from '../src/tokens/tokens';
import { contrastRatio, passesAA, passesAALarge, passesNonText } from '../src/tokens/contrast';

describe('token module (G-UI-11 / R-U1..R-U3)', () => {
  it('ships exactly the 20 printed PDF swatches', () => {
    expect(SWATCHES).toHaveLength(20);
    expect(new Set(SWATCHES.map((entry) => entry.id)).size).toBe(20);
    expect(new Set(SWATCHES.map((entry) => entry.hex)).size).toBe(20);
  });

  it('every swatch carries PDF provenance (pdfLine 21..78)', () => {
    for (const entry of SWATCHES) {
      expect(entry.pdfLine).toBeGreaterThanOrEqual(20);
      expect(entry.pdfLine).toBeLessThanOrEqual(92);
      expect(entry.hex).toMatch(/^#[0-9A-F]{6}$/);
    }
  });

  it('reproduces the six declared contrast rows', () => {
    expect(DECLARED_CONTRAST_ROWS).toHaveLength(6);
    for (const row of DECLARED_CONTRAST_ROWS) {
      const computed = contrastRatio(swatch(row.foreground).hex, swatch(row.background).hex);
      expect(Math.abs(computed - row.ratio)).toBeLessThan(0.05);
    }
  });

  it('corrections pass: meta ink-500 >= 4.5, focus ink-700 >= 3:1', () => {
    const panel = swatch('panel').hex;
    const canvas = swatch('canvas').hex;
    const meta = swatch('ink-500').hex;
    const focus = swatch('ink-700').hex;
    expect(passesAA(contrastRatio(meta, panel))).toBe(true);
    expect(passesAA(contrastRatio(meta, canvas))).toBe(true);
    expect(passesNonText(contrastRatio(focus, panel))).toBe(true);
    expect(passesNonText(contrastRatio(focus, canvas))).toBe(true);
  });

  it('amber is large-text only (3.32 declared)', () => {
    const ratio = contrastRatio(swatch('amber').hex, swatch('canvas').hex);
    expect(passesAALarge(ratio)).toBe(true);
    expect(passesAA(ratio)).toBe(false);
  });

  it('printed type/space/radius rows are recorded', () => {
    expect(TYPE_SCALE).toHaveLength(8);
    expect(SPACING_STOPS).toEqual([4, 8, 12, 16, 20, 24, 32, 40, 48, 64, 80, 96]);
    expect(RADII_STOPS).toEqual([4, 8, 12, 16, 24, 48, 999]);
  });
});
