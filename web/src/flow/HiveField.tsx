/**
 * Reference board: HiveMind spatial canvas
 * (the authoritative 1440x900 spatial composition).
 *
 * Presentational backdrop for the spatial canvas: honeycomb vector field, the
 * five-lobe beehive, ambient glow and sheen. Extracted from Canvas.tsx so the
 * live canvas and replay can share one static layer. A single <svg> element, no
 * filters, no SMIL and no per-frame JS; the glow and sheen motion comes from
 * styles/motion.css (`motion-honeycomb-glow`, `motion-hive-sheen`) and every
 * colour is a CSS variable.
 */

/** Beehive art box, unchanged from Canvas.tsx. */
const HIVE_BOX_WIDTH = 620;
const HIVE_BOX_HEIGHT = 660;

/** Lobe centre lines inside the 620x660 box; every lobe is rx 270 / ry 108. */
const LOBE_CY = [95, 215, 335, 455, 575] as const;

/** Sheen box mirrors the hive box so the highlight lands on the beehive. */
const SHEEN_BOX = 'pointer-events-none absolute h-[660px] w-[620px]';
const SHEEN_BOX_CENTRED = `${SHEEN_BOX} left-1/2 top-1/2 -translate-x-1/2 -translate-y-1/2`;
const SHEEN_BOX_ANCHORED = `${SHEEN_BOX} left-0 top-0`;

export function HiveField({ center = true }: { center?: boolean }) {
  // The hive is placed as a 620x660 viewport at 50%/50% of the field and
  // pulled back by half its box, so no measurement or JS is involved.
  const hiveOrigin = center ? '50%' : 0;
  const hiveTransform = center
    ? `translate(-${HIVE_BOX_WIDTH / 2} -${HIVE_BOX_HEIGHT / 2})`
    : undefined;

  return (
    <div aria-hidden="true" className="pointer-events-none absolute inset-0 z-0 overflow-hidden">
      {/* Glow sits under the field so the honeycomb lines and the hive stay crisp. */}
      <div className="motion-honeycomb-glow absolute inset-0" />
      <svg className="absolute inset-0 h-full w-full" width="100%" height="100%" focusable="false">
        <defs>
          <pattern id="hivefield-honeycomb" width="48" height="42" patternUnits="userSpaceOnUse">
            <path
              d="M12 0L36 0L48 21L36 42L12 42L0 21Z"
              fill="none"
              stroke="var(--color-honey-200)"
              strokeWidth="1"
            />
          </pattern>
          {/* Lobe body: cream core high on the lobe, gold shoulders, honey-700 silhouette. */}
          <radialGradient id="hivefield-lobe" cx="50%" cy="34%" r="60%">
            <stop offset="0%" stopColor="var(--color-honey-100)" stopOpacity="0.98" />
            <stop offset="28%" stopColor="var(--color-honey-200)" stopOpacity="0.96" />
            <stop offset="46%" stopColor="var(--color-honey-300)" stopOpacity="0.94" />
            <stop offset="64%" stopColor="var(--color-honey-500)" stopOpacity="0.95" />
            <stop offset="82%" stopColor="var(--color-honey-600)" stopOpacity="0.96" />
            <stop offset="100%" stopColor="var(--color-honey-700)" stopOpacity="0.98" />
          </radialGradient>
          {/* Darker rim at the bottom of each lobe, where the next lobe tucks under. */}
          <linearGradient id="hivefield-rim" x1="0" y1="0" x2="0" y2="1">
            <stop offset="0%" stopColor="var(--color-honey-700)" stopOpacity="0" />
            <stop offset="52%" stopColor="var(--color-honey-800)" stopOpacity="0.04" />
            <stop offset="80%" stopColor="var(--color-honey-800)" stopOpacity="0.22" />
            <stop offset="100%" stopColor="var(--color-honey-800)" stopOpacity="0.52" />
          </linearGradient>
          {/* Soft top highlight plus a tighter specular core on the crest. */}
          <radialGradient id="hivefield-highlight" cx="50%" cy="50%" r="50%">
            <stop offset="0%" stopColor="var(--color-honey-100)" stopOpacity="0.98" />
            <stop offset="58%" stopColor="var(--color-honey-100)" stopOpacity="0.3" />
            <stop offset="100%" stopColor="var(--color-honey-200)" stopOpacity="0" />
          </radialGradient>
          <radialGradient id="hivefield-spec" cx="50%" cy="50%" r="50%">
            <stop offset="0%" stopColor="var(--color-honey-100)" stopOpacity="0.85" />
            <stop offset="100%" stopColor="var(--color-honey-100)" stopOpacity="0" />
          </radialGradient>
          {/* Very subtle warm drop shadow: a gradient, never a filter. */}
          <radialGradient id="hivefield-shadow" cx="50%" cy="50%" r="50%">
            <stop offset="0%" stopColor="var(--color-honey-800)" stopOpacity="0.18" />
            <stop offset="60%" stopColor="var(--color-honey-800)" stopOpacity="0.07" />
            <stop offset="100%" stopColor="var(--color-honey-800)" stopOpacity="0" />
          </radialGradient>
          {/* Lobe stack in the 620x660 hive space; drawn once, stamped by <use>. */}
          <symbol id="hivefield-hive" viewBox="0 0 620 660">
            {LOBE_CY.map((cy) => (
              <g key={cy}>
                <ellipse cx="310" cy={cy + 14} rx="274" ry="111" fill="url(#hivefield-shadow)" />
                <ellipse cx="310" cy={cy} rx="270" ry="108" fill="url(#hivefield-lobe)" />
                <ellipse cx="310" cy={cy} rx="270" ry="108" fill="url(#hivefield-rim)" />
                <ellipse
                  cx="310"
                  cy={cy - 28}
                  rx="168"
                  ry="44"
                  fill="url(#hivefield-highlight)"
                />
                <ellipse cx="310" cy={cy - 44} rx="104" ry="24" fill="url(#hivefield-spec)" />
              </g>
            ))}
          </symbol>
        </defs>

        <rect width="100%" height="100%" fill="url(#hivefield-honeycomb)" opacity="0.9" />
        <use
          href="#hivefield-hive"
          x={hiveOrigin}
          y={hiveOrigin}
          width={HIVE_BOX_WIDTH}
          height={HIVE_BOX_HEIGHT}
          transform={hiveTransform}
        />
      </svg>
      <div className={center ? SHEEN_BOX_CENTRED : SHEEN_BOX_ANCHORED}>
        <div className="motion-hive-sheen absolute inset-0" />
      </div>
    </div>
  );
}
