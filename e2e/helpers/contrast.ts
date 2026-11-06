/** WCAG 2.1 relative-luminance contrast helpers (pure; runnable now). */

export function parseHex(hex: string): [number, number, number] {
  const value = hex.trim().replace(/^#/, '');
  if (!/^[0-9a-fA-F]{6}$/.test(value)) {
    throw new Error(`invalid hex colour: ${hex}`);
  }
  return [
    parseInt(value.slice(0, 2), 16),
    parseInt(value.slice(2, 4), 16),
    parseInt(value.slice(4, 6), 16),
  ];
}

function channelLuminance(channel: number): number {
  const c = channel / 255;
  return c <= 0.03928 ? c / 12.92 : Math.pow((c + 0.055) / 1.055, 2.4);
}

export function relativeLuminance(hex: string): number {
  const [r, g, b] = parseHex(hex);
  return 0.2126 * channelLuminance(r) + 0.7152 * channelLuminance(g) + 0.0722 * channelLuminance(b);
}

export function contrastRatio(foreground: string, background: string): number {
  const a = relativeLuminance(foreground);
  const b = relativeLuminance(background);
  const [hi, lo] = a >= b ? [a, b] : [b, a];
  return (hi + 0.05) / (lo + 0.05);
}

/** Expected corrected token values (source: reports/ui/UI-REQUIREMENTS.md §1; hexes PDF pages.txt:20-79). */
export const EXPECTED_CONTRAST = [
  { pair: 'ink-900/canvas', fg: '#282010', bg: '#F8F4EB', expected: 14.67, level: 'AAA' },
  { pair: 'ink-900/honey-400', fg: '#282010', bg: '#E8B850', expected: 8.74, level: 'AAA' },
  { pair: 'white/ink-800', fg: '#FFFFFF', bg: '#3F3115', expected: 12.64, level: 'AAA' },
  { pair: 'amber/canvas', fg: '#B7791F', bg: '#F8F4EB', expected: 3.32, level: 'AA-LARGE' },
  { pair: 'error/canvas', fg: '#B23A2E', bg: '#F8F4EB', expected: 5.41, level: 'AA' },
  { pair: 'verify/canvas', fg: '#3F7A55', bg: '#F8F4EB', expected: 4.64, level: 'AA' },
] as const;

/** Failing component pairs that must be replaced (UI-REQUIREMENTS UI-R-03/04). */
export const KNOWN_FAILURES = [
  { pair: 'card-meta/panel', fg: '#8A7A52', bg: '#FFFFFF', failing: 4.21, threshold: 4.5 },
  { pair: 'card-meta/canvas', fg: '#8A7A52', bg: '#F8F4EB', failing: 3.84, threshold: 4.5 },
  { pair: 'focus-ring/panel', fg: '#F0C050', bg: '#FFFFFF', failing: 1.70, threshold: 3.0 },
  { pair: 'focus-ring/canvas', fg: '#F0C050', bg: '#F8F4EB', failing: 1.55, threshold: 3.0 },
] as const;
