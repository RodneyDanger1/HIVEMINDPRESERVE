/**
 * Combined reduced-motion flag: OS preference OR the in-app toggle (R-U10).
 */
import { useUiStore } from './context';
import { usePrefersReducedMotion } from '../a11y/reducedMotion';

export function useAppReducedMotion(): boolean {
  const override = useUiStore((state) => state.reducedMotionOverride);
  const prefers = usePrefersReducedMotion();
  return override || prefers;
}
