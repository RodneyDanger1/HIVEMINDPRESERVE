/**
 * THE token module (R-U1). Every colour/type/space/radius/elevation value the
 * UI uses is declared here with its PDF provenance (`PDF:<page>:<line>`,
 * evidence/pdf/pages.txt). Components must not contain hex literals; the
 * only other place with hex values is `styles/tokens.css`, the CSS mirror of
 * this file (both are the token module; G-UI-11 covers them together).
 *
 * Contract texts: `blueprint/UI-WIRING-GUIDE.md` R-U1/R-U2/R-U3/R-U14,
 * `reports/ui/UI-REQUIREMENTS.md` UI-R-01..12, PDF pages 2-4,6,7.
 */

export interface Swatch {
  readonly id: string;
  readonly name: string;
  readonly hex: string;
  /** 1-based line in evidence/pdf/pages.txt that prints the value. */
  readonly pdfLine: number;
  readonly group: 'surface' | 'honey' | 'ink' | 'state' | 'accent';
}

/** Exactly the 20 printed swatches (R-U2). Count is asserted by test. */
export const SWATCHES: readonly Swatch[] = [
  { id: 'canvas', name: 'Canvas', hex: '#F8F4EB', pdfLine: 21, group: 'surface' },
  { id: 'panel', name: 'Paper', hex: '#FFFFFF', pdfLine: 24, group: 'surface' },
  { id: 'panel-alt', name: 'Panel Alt', hex: '#FBF8F1', pdfLine: 27, group: 'surface' },
  { id: 'recessed', name: 'Recess', hex: '#F1E9CF', pdfLine: 30, group: 'surface' },
  { id: 'hairline', name: 'Hairline', hex: '#E5E1D8', pdfLine: 33, group: 'surface' },
  { id: 'honey-100', name: 'Honey 100', hex: '#FBF3DC', pdfLine: 36, group: 'honey' },
  { id: 'honey-200', name: 'Honey 200', hex: '#F6E7BE', pdfLine: 39, group: 'honey' },
  { id: 'honey-300', name: 'Honey 300', hex: '#F0C050', pdfLine: 42, group: 'honey' },
  { id: 'honey-400', name: 'Honey 400', hex: '#E8B850', pdfLine: 45, group: 'honey' },
  { id: 'honey-500', name: 'Honey 500', hex: '#D9A02F', pdfLine: 48, group: 'honey' },
  { id: 'honey-600', name: 'Honey 600', hex: '#C88030', pdfLine: 51, group: 'honey' },
  { id: 'honey-700', name: 'Honey 700', hex: '#8A5A1C', pdfLine: 54, group: 'honey' },
  { id: 'honey-800', name: 'Honey 800', hex: '#604818', pdfLine: 57, group: 'honey' },
  { id: 'ink-900', name: 'Ink 900', hex: '#282010', pdfLine: 60, group: 'ink' },
  { id: 'ink-800', name: 'Ink 800', hex: '#3F3115', pdfLine: 63, group: 'ink' },
  { id: 'ink-700', name: 'Ink 700', hex: '#4E3A15', pdfLine: 66, group: 'ink' },
  { id: 'ink-500', name: 'Ink 500', hex: '#75663F', pdfLine: 69, group: 'ink' },
  { id: 'amber', name: 'Amber', hex: '#B7791F', pdfLine: 72, group: 'accent' },
  { id: 'signal', name: 'Signal', hex: '#B23A2E', pdfLine: 75, group: 'state' },
  { id: 'verify', name: 'Verify', hex: '#3F7A55', pdfLine: 78, group: 'state' },
] as const;

export function swatch(id: string): Swatch {
  const found = SWATCHES.find((entry) => entry.id === id);
  if (!found) {
    throw new Error(`Unknown token "${id}"`);
  }
  return found;
}

export interface DeclaredContrastRow {
  readonly foreground: string;
  readonly background: string;
  readonly ratio: number;
  readonly grade: 'AAA' | 'AA' | 'AA-LARGE';
  readonly pdfLine: number;
}

/** The six rows the PDF prints in its measured contrast table (PDF:80-92). */
export const DECLARED_CONTRAST_ROWS: readonly DeclaredContrastRow[] = [
  { foreground: 'ink-900', background: 'canvas', ratio: 14.67, grade: 'AAA', pdfLine: 81 },
  { foreground: 'ink-900', background: 'honey-400', ratio: 8.74, grade: 'AAA', pdfLine: 83 },
  { foreground: 'panel', background: 'ink-800', ratio: 12.64, grade: 'AAA', pdfLine: 85 },
  { foreground: 'amber', background: 'canvas', ratio: 3.32, grade: 'AA-LARGE', pdfLine: 87 },
  { foreground: 'signal', background: 'canvas', ratio: 5.41, grade: 'AA', pdfLine: 89 },
  { foreground: 'verify', background: 'canvas', ratio: 4.64, grade: 'AA', pdfLine: 91 },
] as const;

/** WCAG corrections the blueprint forces (R-U3, DELTA R-10). */
export const CORRECTIONS = {
  /** Card meta: PDF prints #8A7A52 at 9.5px (4.21 white / 3.84 canvas = fails AA). */
  metaText: { tokenId: 'ink-500', pdfSource: 'PDF:223', replacementOf: '#8A7A52' as const },
  /** Focus ring: PDF prints honey-300 (1.55-1.70 = fails 3:1 non-text). */
  focusRing: { tokenId: 'ink-700', pdfSource: 'PDF:237', replacementOf: 'honey-300' as const },
  /** Amber 3.32:1 is large-text only (>=24px / >=18.66px bold). */
  amberLargeOnly: true,
} as const;

export interface TypeStep {
  readonly id: string;
  readonly family: 'display' | 'ui' | 'mono';
  readonly sizePx: number;
  readonly linePx: number;
  readonly trackingEm: number;
  readonly pdfLine: number;
}

/** Printed type steps only (8); declared totals (13) do not reconcile (R-U2). */
export const TYPE_SCALE: readonly TypeStep[] = [
  { id: 'display-xxl', family: 'display', sizePx: 96, linePx: 100, trackingEm: -0.03, pdfLine: 111 },
  { id: 'display-l', family: 'display', sizePx: 56, linePx: 60, trackingEm: -0.02, pdfLine: 112 },
  { id: 'heading-h1', family: 'ui', sizePx: 40, linePx: 48, trackingEm: -0.015, pdfLine: 114 },
  { id: 'heading-h3', family: 'ui', sizePx: 24, linePx: 32, trackingEm: -0.005, pdfLine: 117 },
  { id: 'body-l', family: 'ui', sizePx: 17, linePx: 28, trackingEm: 0, pdfLine: 119 },
  { id: 'label', family: 'ui', sizePx: 12, linePx: 16, trackingEm: 0.08, pdfLine: 122 },
  { id: 'mono-code', family: 'mono', sizePx: 13, linePx: 22, trackingEm: 0, pdfLine: 123 },
  { id: 'mono-meta', family: 'mono', sizePx: 11, linePx: 16, trackingEm: 0.08, pdfLine: 124 },
] as const;

/** 4px base scale exactly as printed (PDF:132-133). Tailwind spacing matches it. */
export const SPACING_STOPS: readonly number[] = [4, 8, 12, 16, 20, 24, 32, 40, 48, 64, 80, 96];
export const SPACING_BASE_PX = 4;

/** Printed radius stops {4,8,12,16,24,48,999}; 10 and 14 are exceptions (R-U14). */
export const RADII_STOPS: readonly number[] = [4, 8, 12, 16, 24, 48, 999];
export const RADIUS_EXCEPTIONS = {
  button: { px: 10, pdfLine: 240 },
  superCard: { px: 14, pdfLine: 224 },
} as const;

/**
 * Elevation: all three printed tiers carry the same value (#282010 @ 7-13%).
 * Tier differentiation is UNVERIFIED (UI-R-10 / U1 C-10): the UI therefore
 * uses one warm-ink shadow family and does not claim tier semantics.
 */
export const ELEVATION = {
  ink: 'ink-900',
  alphaMin: 0.07,
  alphaMax: 0.13,
  tierDifferentiation: 'UNVERIFIED',
  pdfLines: [137, 146] as const,
} as const;

/** Bee/edge semantics: the bee is a routing arrow, never decoration (R-U6). */
export const ROUTING = {
  snapRadiusPx: 150,
  honeycombSpacingPx: 24,
  pdfLine: 227,
  sourceLine: 898,
} as const;

export const ZOOM = {
  min: 0.1,
  max: 3.0,
  pdfLine: 304,
} as const;

/** Token-correction helpers the components read instead of raw hex. */
export const SEMANTIC = {
  canvas: 'canvas',
  panel: 'panel',
  panelAlt: 'panel-alt',
  recessed: 'recessed',
  hairline: 'hairline',
  textPrimary: 'ink-900',
  textSecondary: 'ink-500',
  textMuted: 'ink-500',
  metaText: 'ink-500',
  focusRing: 'ink-700',
  error: 'signal',
  success: 'verify',
  accentLargeTextOnly: 'amber',
  nodeHealthy: 'honey-300',
  nodeRunning: 'honey-500',
  nodeFailed: 'signal',
  bee: 'ink-800',
} as const;
