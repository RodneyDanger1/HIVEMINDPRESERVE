/**
 * Startup veil for the local HMARS.
 *
 * The splash is intentionally a real boot surface, not a decorative cover:
 * it reports the live socket state, disappears only after the connection is
 * settled (or a bounded wait expires), and never blocks an offline UI forever.
 * Its material is the HiveMind "HoneyGlass" system: a frosted glass pane
 * (Glassmorphism), an inset neumorphic loading well, and one liquid-glass
 * honeycomb focal cluster. All color comes from the FigCraft token mirror.
 */
import { useEffect, useState, type CSSProperties } from 'react';
import { useUiStore } from '../state/context';

type SplashPhase = 'visible' | 'leaving' | 'gone';

const HEX_CELLS = [
  { x: 50, y: 10, size: 24, delay: '0ms' },
  { x: 72, y: 22, size: 20, delay: '80ms' },
  { x: 28, y: 22, size: 20, delay: '160ms' },
  { x: 50, y: 37, size: 30, delay: '40ms' },
  { x: 73, y: 50, size: 22, delay: '120ms' },
  { x: 27, y: 50, size: 22, delay: '200ms' },
  { x: 50, y: 65, size: 24, delay: '60ms' },
  { x: 70, y: 78, size: 16, delay: '180ms' },
  { x: 30, y: 78, size: 16, delay: '100ms' },
] as const;

function connectionLabel(connection: string): string {
  if (connection === 'open') return 'The hive is awake';
  if (connection === 'error' || connection === 'closed') return 'Runtime is offline';
  if (connection === 'reconnecting') return 'Rejoining the hive';
  return 'Bringing the hive online';
}

export function HiveMindSplash() {
  const connection = useUiStore((state) => state.connection);
  const [phase, setPhase] = useState<SplashPhase>('visible');
  const [minimumElapsed, setMinimumElapsed] = useState(false);
  const settled = connection === 'open' || connection === 'error' || connection === 'closed';

  useEffect(() => {
    const timer = window.setTimeout(() => setMinimumElapsed(true), 1_600);
    return () => window.clearTimeout(timer);
  }, []);

  useEffect(() => {
    if (!minimumElapsed) return undefined;
    const canLeave = settled;
    const timer = window.setTimeout(
      () => setPhase('leaving'),
      canLeave ? 0 : 4_400,
    );
    return () => window.clearTimeout(timer);
  }, [minimumElapsed, settled]);

  useEffect(() => {
    if (phase !== 'leaving') return undefined;
    const timer = window.setTimeout(() => setPhase('gone'), 420);
    return () => window.clearTimeout(timer);
  }, [phase]);

  if (phase === 'gone') return null;

  const label = connectionLabel(connection);
  const progress = settled ? 100 : connection === 'connecting' ? 46 : 18;

  return (
    <div
      className={`splash-root${phase === 'leaving' ? ' splash-leaving' : ''}`}
      data-splash="hivemind"
      role="status"
      aria-live="polite"
      aria-label="HiveMind is starting"
    >
      <div className="splash-ambient" aria-hidden="true" />
      <div className="splash-glass" data-testid="hivemind-splash">
        <div className="splash-specular" aria-hidden="true" />

        <div className="splash-hex-field" aria-hidden="true">
          {HEX_CELLS.map((cell, index) => (
            <span
              key={`${cell.x}-${cell.y}-${index}`}
              className="splash-hex"
              style={
                {
                  '--hex-x': `${cell.x}%`,
                  '--hex-y': `${cell.y}%`,
                  '--hex-size': `${cell.size}px`,
                  '--hex-delay': cell.delay,
                } as CSSProperties
              }
            />
          ))}
        </div>

        <div className="splash-content">
          <p className="splash-kicker">LOCAL HMARS · SPATIAL EXECUTION</p>
          <h1 className="splash-wordmark" aria-label="HiveMind">
            <span>HiveMind</span>
            <svg className="splash-drip" viewBox="0 0 260 18" aria-hidden="true">
              <path d="M10 3c28 4 52 2 78 6 28 4 58 1 84 3 22 2 44 0 78-3" />
              <path d="M74 12c0 5 3 6 5 6s5-2 3-6" />
              <path d="M184 13c0 4 2 5 4 5s5-2 3-5" />
            </svg>
          </h1>
          <p className="splash-description">
            One governing HyperAgent. SuperAgents, Agents, and temporary Subagents in a
            local hive that plans, delegates, remembers, and reports what actually happened.
          </p>

          <div className="splash-loading">
            <div className="splash-loading-label">
              <span className="splash-dot" aria-hidden="true" />
              <span>{label}</span>
              <span className="splash-percent" aria-hidden="true">{progress}%</span>
            </div>
            <div
              className="splash-well"
              role="progressbar"
              aria-valuemin={0}
              aria-valuemax={100}
              aria-valuenow={progress}
              aria-label="HiveMind startup progress"
            >
              <span className="splash-fill" style={{ width: `${progress}%` }} />
            </div>
            <p className="splash-status">
              {connection === 'open'
                ? 'Connected to the local runtime. Opening the workspace.'
                : connection === 'error' || connection === 'closed'
                  ? 'The local runtime is offline. The workspace will open with a named refusal.'
                  : 'Opening the local socket and loading the live hive state.'}
            </p>
          </div>
        </div>

        <p className="splash-credit" aria-label="Designed by Riley Christensen">
          Designed by <span>Riley Christensen</span>
        </p>
      </div>
    </div>
  );
}
