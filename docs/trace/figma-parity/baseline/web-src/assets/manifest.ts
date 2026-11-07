/**
 * FigCraft asset seam — WIRED to the UI watchdog verdict (2026-11-06).
 *
 * Watchdog: `ledgers/ui-watchdog.json` → overall PASS-WITH-NOTES, FAIL 0,
 * `allow_frontend: true`, A 28/28 + B 27/27 surfaces verified, A tokens exact
 * (20/20). Accepted exports are recorded below with their audited sha256.
 *
 * RULES (binding, from the verdict + UI-WIRING-GUIDE):
 *  - Use the token module (`src/tokens/*`) for all values; never sample pixels
 *    from a board export.
 *  - Do NOT adopt the off-token grey spinner from A 03 Specimens; A is
 *    fixing it. Use ink-500/hairline instead.
 *  - Pixel-layout fidelity stays UNVERIFIED/needs_human.
 *  - S25 code-diff stays DECISION-GATED (no DIFF_* commands implemented).
 *  - No export is copied into the bundle at W1: boards are reference evidence
 *    served from `docs/ui/figcraft/ASSETS`; consuming bytes in the web bundle is
 *    a Phase-12 packaging decision. The seam records acceptance so nothing is
 *    fabricated or silently dropped.
 */
export interface FigCraftAcceptedExport {
  readonly id: string;
  readonly path: string;
  readonly svgSha256: string;
  readonly pngSha256: string;
  readonly note: string;
}

export const FIGCRAFT_GATE = {
  allowFrontend: true,
  overall: 'PASS-WITH-NOTES',
  failCount: 0,
  checkedAt: '2026-11-06T04:54:33.224146-06:00',
  ledger: 'ledgers/ui-watchdog.json',
  assetsIndex: 'docs/ui/figcraft/ASSETS-INDEX.json',
  counts: { aSurfaces: 28, bSurfaces: 27, aTokens: 20 },
  notes: [
    'A specimens carry one off-token grey spinner in 03/load/spinner; A is fixing it — never adopt that colour.',
    'Pixel-layout fidelity is UNVERIFIED/needs_human (boards are text-layer; contact sheets are shadow sprites).',
    'S25 code-diff review stays DECISION-GATED; no DIFF_* command is implemented.',
  ],
} as const;

export const ACCEPTED_EXPORTS: readonly FigCraftAcceptedExport[] = [
  {
    id: 'a-foundations',
    path: 'docs/ui/figcraft/ASSETS/a-foundations.svg',
    svgSha256: '24536354E6FFE9BA4DDA2C9682B526BB44BEDEA6AEA7F1D21FD4E3AEB75E551A',
    pngSha256: 'E49B788A812E8A4962A29A926D25970B0A39218ADE8DD34EE9C8C7079BDC53BC',
    note: 'Foundations board (tokens/type/spacing/radii/elevation). Values taken from token module, not pixels.',
  },
  {
    id: 'a-brand',
    path: 'docs/ui/figcraft/ASSETS/a-brand.svg',
    svgSha256: '94CF946703C14355601813367F9A5FA11105D4E744990DD66AC7B0E5AD0A9522',
    pngSha256: '47680984AA5DACB1BF6A3576B42A11B01864C646068F33E5CA4ACFE124F4A918',
    note: 'Brand board (lockups/bee glyph). Inline PROPOSED-ADDITION mark remains until bundle decision.',
  },
  {
    id: 'a-components',
    path: 'docs/ui/figcraft/ASSETS/a-components.svg',
    svgSha256: '553E64308A50162F95F22F29C16DDB5555AD543CAC426E248A95D1ADAA53D0CE',
    pngSha256: 'BC71B15999593B6081FDBD2087432D9C80DFAA2C1430CA5A8DD4173BDA786C58',
    note: 'Components board (S02-S22 surfaces).',
  },
  {
    id: 'a-specimens',
    path: 'docs/ui/figcraft/ASSETS/a-specimens.svg',
    svgSha256: 'D8128E43D7726A39D3BE07CF5D579A050FE50945296098E717D2BF346AF61563',
    pngSha256: '99321E1FDEB2446CA2457BABF865D6353FCC693C3AB3E29361A5D8E4D1027FE7',
    note: 'Specimens board; PASS-WITH-NOTES: carries an off-token grey spinner in 03/load/spinner (A fixing) — not adopted.',
  },
  {
    id: 'b-canvas-default',
    path: 'docs/ui/figcraft/ASSETS/b-canvas-default.svg',
    svgSha256: 'B5A5A824145A85848247A10A663FAB2D4783CC120D04DE51FB93316A9EE33D9F',
    pngSha256: 'AA6066D38F73D391B45C623C62E285BDD19D541F3443DB984279627F2DF7A694',
    note: 'B canvas/default board 1440x900 (S01-S28 interaction layout).',
  },
  {
    id: 'b-zoom-states',
    path: 'docs/ui/figcraft/ASSETS/b-zoom-states.svg',
    svgSha256: 'AB1EE97AE2B9D042B9D8B78EF22F10F1D4B594041FBC51ACD27C32112A9493AE',
    pngSha256: 'A02CB4A1963A8C3588AE3CC66FA74C0F7D415E1A46E95089D191794D4F79141C',
    note: 'B zoom-states board 1440x900.',
  },
  {
    id: 'b-interaction',
    path: 'docs/ui/figcraft/ASSETS/b-interaction.svg',
    svgSha256: '927514772AC5321C3771ED7293AE72B67E53526CB544222B21AF408550A2E945',
    pngSha256: 'DE50112439898E88B8BBF334DD8427F0F8AF2FA2FBCC6BED338334F34F0D00A4',
    note: 'B interaction-states board 1440x900.',
  },
  {
    id: 'b-hero',
    path: 'docs/ui/figcraft/ASSETS/b-hero.svg',
    svgSha256: '2BF8D20FACBA986B2C148702A774187538886658C680F7D323AA0E785A678477',
    pngSha256: '719A28BE8DE4926612ECFE5537EC7BDB98E04EAC5338D34FCC6EFBB62B54834E',
    note: 'B brand hero board 1440x900.',
  },
] as const;

export function canConsumeFigCraftAssets(): boolean {
  return FIGCRAFT_GATE.allowFrontend;
}

export function acceptedExport(id: string): FigCraftAcceptedExport {
  const found = ACCEPTED_EXPORTS.find((entry) => entry.id === id);
  if (!found) {
    throw new Error(`No accepted FigCraft export "${id}" in the watchdog ledger`);
  }
  return found;
}
