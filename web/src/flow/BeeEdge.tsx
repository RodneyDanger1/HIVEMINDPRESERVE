/**
 * Bee edge (S06/R-U6): direction source -> target, bee glyph at the midpoint
 * oriented along the data flow. The bee is a routing marker, not decoration.
 */
import {
  BaseEdge,
  EdgeLabelRenderer,
  getBezierPath,
  type EdgeProps,
} from '@xyflow/react';
import { beeAngle } from '../a11y/keyboard';

export function BeeEdge(props: EdgeProps) {
  const [path, labelX, labelY] = getBezierPath({
    sourceX: props.sourceX,
    sourceY: props.sourceY,
    targetX: props.targetX,
    targetY: props.targetY,
    sourcePosition: props.sourcePosition,
    targetPosition: props.targetPosition,
  });
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
        style={{
          stroke: broken ? 'var(--color-signal)' : 'var(--color-honey-400)',
          strokeWidth: 1.5,
          strokeDasharray: broken ? '6 6' : '10 10',
          animation: 'bee-dash 1.2s linear infinite',
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
          <svg width="14" height="14" viewBox="0 0 14 14">
            <ellipse cx="7" cy="7" rx="4.2" ry="2.7" fill="currentColor" />
            <path d="M3 7H1M11 7h2" stroke="currentColor" strokeWidth="1.2" />
            <path d="M7 4.3c-1.2-1.6-3-1.9-4.2-1.4C4.2 4 5.4 4.4 7 4.6Z" fill="var(--color-honey-400)" />
          </svg>
        </div>
      </EdgeLabelRenderer>
    </>
  );
}
