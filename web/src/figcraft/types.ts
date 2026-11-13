/**
 * FigCraft representation types (W13 / Lead task-11).
 *
 * The FigCraft evidence lives in `docs/ui/figcraft/**` (read-only outside this
 * surface). This module describes the accepted boards copied into
 * `web/public/figcraft/` for the in-app representation gallery; it contains no
 * pixel data and no fabricated dimensions.
 */

/** Accepted FigCraft manifest sets (one owner per board). */
export type FigCraftOwner = 'figcraft-hivemind' | 'figcraft-a' | 'figcraft-b';

/**
 * Audit provenance of one board. Only `watchdog-audited` means the UI watchdog
 * ledger accepted that board. `self-verified` means the owning set's own verify
 * report passed; `proposed` means the set labels the board as a proposal or
 * PROPOSED-ADDITION. Neither may be shown as watchdog-accepted.
 */
export type FigCraftAuditStatus =
  | 'watchdog-audited'
  | 'self-verified'
  | 'proposed'
  | 'unverified';

/** Per-board audit evidence, generated from ASSETS-INDEX.json. */
export interface FigCraftAudit {
  readonly status: FigCraftAuditStatus;
  /** Watchdog verdict (`PASS` / `PASS-WITH-NOTES`) when one exists. */
  readonly verdict: string | null;
  /** Ledger or verify-report path that backs the status, when one exists. */
  readonly ledger: string | null;
  /** One-line human-readable basis for the status. */
  readonly basis: string;
}

/** One selectable manifest set rendered as an accessible tab. */
export interface FigCraftGroup {
  readonly id: FigCraftOwner;
  readonly label: string;
  readonly description: string;
}

/** One HiveMind surface covered by a board (from the source manifest). */
export interface FigCraftSurface {
  readonly surfaceId: string;
  readonly name: string;
}

/** One accepted FigCraft board and the public SVG copied for it. */
export interface FigCraftBoard {
  /** Stable slug derived from the exported SVG file name. */
  readonly id: string;
  /** Manifest set that owns the board. */
  readonly owner: FigCraftOwner;
  /** Human title from the source manifest (frame/page). */
  readonly title: string;
  /** Board size in CSS pixels as recorded by the manifest. */
  readonly width: number;
  readonly height: number;
  /** File name under `web/public/figcraft/` (served at `/figcraft/<file>`). */
  readonly svgFile: string;
  /** Repository-relative accepted source SVG (evidence path, never fetched). */
  readonly sourcePath: string;
  /** sha256 of the source SVG recorded by the accepting manifest. */
  readonly sha256: string;
  /** Manifest file the board was read from. */
  readonly manifest: string;
  /** Audit provenance; never claim more than this says. */
  readonly audit: FigCraftAudit;
  /** Surfaces this board covers (empty for pure design boards). */
  readonly surfaces: readonly FigCraftSurface[];
}

/** One source manifest and its accepted hash. */
export interface FigCraftSourceManifest {
  readonly name: string;
  readonly path: string;
  readonly sha256: string;
}
