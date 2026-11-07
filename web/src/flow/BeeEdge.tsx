/**
 * Bee edge (S06/R-U6): direction source -> target, bee glyph at the midpoint
 * oriented along the data flow. The bee is a routing marker, not decoration.
 */
import {
  BaseEdge,
  EdgeLabelRenderer,
  getStraightPath,
  type EdgeProps,
} from '@xyflow/react';
import { beeAngle } from '../a11y/keyboard';

export function BeeEdge(props: EdgeProps) {
  const [path] = getStraightPath({
    sourceX: props.sourceX,
    sourceY: props.sourceY,
    targetX: props.targetX,
    targetY: props.targetY,
  });
  const labelX = (props.sourceX + props.targetX) / 2;
  const labelY = (props.sourceY + props.targetY) / 2;
  const angle = (beeAngle(
    { x: props.sourceX, y: props.sourceY },
    { x: props.targetX, y: props.targetY },
  ) * 180) /
    Math.PI;
  const broken = (props.data as { broken?: boolean } | undefined)?.broken === true;
  return (
    <>
      <BaseEdge
        id={props.id}
        path={path}
        markerEnd={broken ? 'url(#bee-arrow-broken)' : 'url(#bee-arrow)'}
        style={{
          stroke: broken ? 'var(--color-signal)' : 'var(--color-honey-400)',
          strokeWidth: 1.5,
          strokeDasharray: broken ? '6 6' : '10 10',
          animation: 'bee-dash 2.4s linear infinite',
        }}
      />
      <EdgeLabelRenderer>
        <div
          style={{
            position: 'absolute',
            transform: `translate(-50%, -50%) translate(${labelX}px, ${labelY}px) rotate(${angle}deg)`,
            pointerEvents: 'none',
            color: broken ? 'var(--color-signal)' : 'var(--color-ink-800)',
          }}
          aria-hidden="true"
        >
          <div className="bee-bob">
            <svg width="18" height="18" viewBox="0 0 18 18">
              <ellipse cx="9" cy="9" rx="5" ry="3.2" fill="currentColor" />
              <path d="M3 9h12M5.5 6.5h7M5.5 11.5h7" stroke="var(--color-honey-300)" strokeWidth="1.1" />
              <path d="M7 5.5C5.5 3.7 3.6 3.4 2.5 4.2c1.4.4 2.2 1 3.8 1.9ZM11 5.5c1.5-1.8 3.4-2.1 4.5-1.3-1.4.4-2.2 1-3.8 1.9Z" fill="var(--color-honey-100)" />
            </svg>
          </div>
        </div>
      </EdgeLabelRenderer>
    </>
  );
}
