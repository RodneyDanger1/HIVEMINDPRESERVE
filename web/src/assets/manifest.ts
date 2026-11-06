/**
 * FigCraft asset seam (no fabricated provenance).
 *
 * The UI watchdog gates consumption of FigCraft exports. At W1 the gate is
 * CLOSED: `ledgers/ui-watchdog.json` does not exist, `docs/ui/figcraft/
 * ASSETS-INDEX.json` does not exist, and only A exports exist on disk. The UI
 * therefore renders PDF-derived tokens and inline PROPOSED-ADDITION marks; it
 * must never claim a FigCraft asset's sha/author or copy an asset path that
 * the watchdog has not audited.
 */
export type FigCraftAssetStatus = 'not-consumed' | 'pending-ui-watchdog' | 'in-progress' | 'allowed';

export interface FigCraftAssetSeam {
  readonly id: string;
  /** Expected path once the gate opens; NOT consumed while the gate is closed. */
  readonly expectedPath: string;
  readonly status: FigCraftAssetStatus;
  readonly reason: string;
}

export const FIGCRAFT_GATE = {
  allowFrontend: false,
  checkedAt: '2026-11-06',
  evidence: [
    'ledgers/ui-watchdog.json: absent at W1 (no allow_frontend flag)',
    'docs/ui/figcraft/ASSETS-INDEX.json: absent at W1',
    'docs/ui/figcraft/ASSETS/a-foundations.{svg,png} and a-brand.{svg,png}: exported by FigCraft A, not yet watchdog-approved',
    'B assets: in progress',
  ],
} as const;

export const FIGCRAFT_ASSET_SEAM: readonly FigCraftAssetSeam[] = [
  {
    id: 'a-foundations',
    expectedPath: 'docs/ui/figcraft/ASSETS/a-foundations.svg',
    status: 'pending-ui-watchdog',
    reason: 'Exported, but the UI watchdog has not set allow_frontend; do not copy provenance from filename alone.',
  },
  {
    id: 'a-brand',
    expectedPath: 'docs/ui/figcraft/ASSETS/a-brand.svg',
    status: 'pending-ui-watchdog',
    reason: 'Exported, but the UI watchdog has not set allow_frontend.',
  },
  {
    id: 'b-foundations',
    expectedPath: 'docs/ui/figcraft/ASSETS/b-foundations.svg',
    status: 'in-progress',
    reason: 'FigCraft B export not finished at W1.',
  },
  {
    id: 'b-components',
    expectedPath: 'docs/ui/figcraft/ASSETS/b-components.svg',
    status: 'in-progress',
    reason: 'FigCraft B export not finished at W1.',
  },
];

export function canConsumeFigCraftAssets(): boolean {
  return FIGCRAFT_GATE.allowFrontend;
}

/**
 * Components that want an audited FigCraft asset must call this seam; while
 * the gate is closed it fails loudly instead of silently falling back to a
 * fabricated asset provenance.
 */
export function requireFigCraftAssets(feature: string): never {
  throw new Error(
    `FigCraft assets are not allowlisted for the frontend (requested by ${feature}); ` +
      `gate: ${FIGCRAFT_GATE.evidence.join(' | ')}`,
  );
}
