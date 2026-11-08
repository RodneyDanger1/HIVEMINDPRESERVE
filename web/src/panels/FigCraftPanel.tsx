/**
 * FigCraft representation panel (Lead task-11).
 *
 * Self-contained representation surface for the accepted FigCraft evidence in
 * `docs/ui/figcraft/**`. It renders the boards copied into
 * `web/public/figcraft/` and the manifest-derived catalog in
 * `web/src/figcraft/catalog.ts`. It reads no run state, writes no store state,
 * and performs no network call beyond loading the static SVG public assets.
 *
 * Integration is owned by the Lead (see the task receipt): mounting this panel
 * in an existing view does not require any change inside this directory.
 */
import { FIGCRAFT_BOARDS, FIGCRAFT_COUNTS, FIGCRAFT_SOURCE_MANIFESTS } from '../figcraft/catalog';
import { FigCraftGallery } from '../figcraft/FigCraftGallery';

export function FigCraftPanel() {
  return (
    <section aria-labelledby="figcraft-panel-title" className="flex h-full min-h-0 flex-col gap-3 overflow-auto p-4">
      <header className="min-w-0">
        <div className="type-mono-meta text-ink-500">ACCEPTED DESIGN EVIDENCE · READ-ONLY</div>
        <h2 id="figcraft-panel-title" className="type-heading-h3 m-0 mt-1 text-ink-900">
          FigCraft representation
        </h2>
        <p className="type-mono-code m-0 mt-1 max-w-[70ch] text-ink-500">
          The in-app representation of the accepted FigCraft boards: manifest titles, board sizes, source
          hashes and the HiveMind surfaces each board covers. The SVG copies are static public assets; the
          source evidence under <code className="type-mono-code">docs/ui/figcraft/</code> remains authoritative.
        </p>
      </header>

      {FIGCRAFT_COUNTS.allowFrontend ? (
        <p role="note" className="type-mono-meta m-0 rounded-[var(--radius-sm)] border border-hairline bg-panel-alt px-3 py-2 text-ink-500">
          UI watchdog: {FIGCRAFT_COUNTS.watchdog} — frontend consumption allowed. {FIGCRAFT_COUNTS.boards}{' '}
          accepted boards cover {FIGCRAFT_COUNTS.surfaces} manifest surfaces ({FIGCRAFT_COUNTS.assets} asset
          rows across {FIGCRAFT_COUNTS.owners} sets). This surface is a static representation; it does not
          claim pixel-fidelity or runtime behaviour.
        </p>
      ) : (
        <p role="alert" className="type-mono-code m-0 rounded-[var(--radius-sm)] border border-signal bg-panel px-3 py-2 text-signal">
          UI watchdog: {FIGCRAFT_COUNTS.watchdog} — frontend consumption is not allowed; showing manifest
          metadata only.
        </p>
      )}

      {FIGCRAFT_BOARDS.length === 0 ? (
        <p role="status" className="type-mono-code m-0 rounded-[var(--radius-sm)] border border-hairline bg-panel-alt px-3 py-2 text-ink-500">
          No accepted FigCraft boards are present in the catalog. The representation surface is unavailable
          rather than fabricated.
        </p>
      ) : (
        <div className="min-h-0 flex-1">
          <FigCraftGallery />
        </div>
      )}

      <footer className="type-mono-meta mt-auto border-t border-hairline pt-2 text-ink-500">
        <p className="m-0">
          Public copies: <code className="type-mono-code">web/public/figcraft/</code> · provenance:{' '}
          <code className="type-mono-code">web/public/figcraft/SOURCES.json</code> · generated catalog:{' '}
          <code className="type-mono-code">web/src/figcraft/catalog.ts</code>.
        </p>
        <p className="m-0 mt-1">
          Source manifests:{' '}
          {FIGCRAFT_SOURCE_MANIFESTS.map((manifest, index) => (
            <span key={manifest.path}>
              {index > 0 ? ' · ' : ''}
              <code className="type-mono-code">{manifest.path}</code>
            </span>
          ))}
          .
        </p>
        <p className="m-0 mt-1">
          Honest limits: copied SVGs are byte-identical to the accepted exports (sha256 verified at copy
          time); pixel-fidelity and runtime UI behaviour remain UNVERIFIED and are not claimed here.
        </p>
      </footer>
    </section>
  );
}

export default FigCraftPanel;
