/**
 * Keyboard/focus helpers (G-UI-2/G-UI-8, R-U7/R-U15). Pure functions so the
 * gates are unit-tested without a DOM.
 */
import { MAX_ZOOM, MIN_ZOOM } from '../api/contract';

export interface KeyLikeEvent {
  readonly key: string;
  readonly ctrlKey: boolean;
  readonly metaKey: boolean;
  readonly shiftKey: boolean;
  readonly altKey: boolean;
  readonly target: EventTarget | null;
}

export interface ShortcutHandlers {
  readonly recenter: () => void;
  readonly zoomIn: () => void;
  readonly zoomOut: () => void;
  readonly fitView: () => void;
}

function isEditableElement(target: EventTarget | null): boolean {
  if (target === null || typeof target !== 'object') {
    return false;
  }
  const element = target as {
    tagName?: unknown;
    isContentEditable?: unknown;
    getAttribute?: (name: string) => string | null;
  };
  const tag = typeof element.tagName === 'string' ? element.tagName.toLowerCase() : '';
  if (tag === 'input' || tag === 'textarea' || tag === 'select') {
    return true;
  }
  if (typeof element.isContentEditable === 'boolean') {
    return element.isContentEditable;
  }
  if (typeof element.getAttribute === 'function') {
    const editable = element.getAttribute('contenteditable');
    return editable === '' || editable === 'true';
  }
  return false;
}

/**
 * Ctrl+0 recentres on Windows (the only v1 target); Meta+0 is accepted as an
 * alias only so a future macOS build works. Typing controls are never hijacked.
 */
export function handleCanvasShortcut(
  event: KeyLikeEvent,
  handlers: ShortcutHandlers,
): boolean {
  if (isEditableElement(event.target)) {
    return false;
  }
  const chord = event.ctrlKey || event.metaKey;
  if (event.altKey) {
    return false;
  }
  if (chord && event.key === '0' && !event.shiftKey) {
    handlers.recenter();
    return true;
  }
  if (chord && event.key === '=') {
    handlers.zoomIn();
    return true;
  }
  if (chord && event.key === '-') {
    handlers.zoomOut();
    return true;
  }
  if (chord && (event.key === '1' || event.key === 'f')) {
    handlers.fitView();
    return true;
  }
  return false;
}

export function clampZoom(zoom: number): number {
  if (!Number.isFinite(zoom)) {
    return MIN_ZOOM;
  }
  return Math.min(MAX_ZOOM, Math.max(MIN_ZOOM, zoom));
}

export interface Point {
  readonly x: number;
  readonly y: number;
}

/** Bee orientation: the arrow flies along source → target (R-U6), radians. */
export function beeAngle(from: Point, to: Point): number {
  return Math.atan2(to.y - from.y, to.x - from.x);
}

/** Roving-tabindex helper for the nav rail / tab lists. */
export function nextRovingIndex(current: number, delta: 1 | -1, length: number): number {
  if (length <= 0) {
    return 0;
  }
  return (current + delta + length) % length;
}
