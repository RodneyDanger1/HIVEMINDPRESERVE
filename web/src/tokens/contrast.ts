/**
 * WCAG 2.1 contrast maths for the token table (G-UI-1 / UI-R-02..05).
 * Pure functions, no UI state; used by tests and by the a11y gate table.
 */

export interface Rgb {
  readonly r: number;
  readonly g: number;
  readonly b: number;
}

export function hexToRgb(hex: string): Rgb {
  const value = hex.trim().replace(/^#/, '');
  if (!/^[0-9a-fA-F]{6}$/.test(value)) {
    throw new Error(`Not a 6-digit hex colour: ${hex}`);
  }
  return {
    r: Number.parseInt(value.slice(0, 2), 16),
    g: Number.parseInt(value.slice(2, 4), 16),
    b: Number.parseInt(value.slice(4, 6), 16),
  };
}

function channelToLinear(channel: number): number {
  const normalized = channel / 255;
  return normalized <= 0.04045
    ? normalized / 12.92
    : ((normalized + 0.055) / 1.055) ** 2.4;
}

/** WCAG relative luminance. */
export function relativeLuminance(rgb: Rgb): number {
  return (
    0.2126 * channelToLinear(rgb.r) +
    0.7152 * channelToLinear(rgb.g) +
    0.0722 * channelToLinear(rgb.b)
  );
}

/** WCAG contrast ratio (1..21). */
export function contrastRatio(foregroundHex: string, backgroundHex: string): number {
  const a = relativeLuminance(hexToRgb(foregroundHex));
  const b = relativeLuminance(hexToRgb(backgroundHex));
  const lighter = Math.max(a, b);
  const darker = Math.min(a, b);
  return (lighter + 0.05) / (darker + 0.05);
}

export function passesAA(ratio: number): boolean {
  return ratio >= 4.5;
}

export function passesAALarge(ratio: number): boolean {
  return ratio >= 3;
}

export function passesNonText(ratio: number): boolean {
  return ratio >= 3;
}

export function formatRatio(ratio: number): string {
  return `${ratio.toFixed(2)}:1`;
}
