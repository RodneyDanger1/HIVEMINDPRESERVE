/**
 * Bottom-dock geometry for the chat core.
 *
 * The dock is an operator surface, not a modal: the transcript keeps a
 * readable minimum, the composer stays on screen, and only the optional
 * controls/state tray may add height. Keeping the arithmetic here (instead of
 * inline in JSX) makes the resize clamp, the tray delta and the persisted
 * height independently testable.
 */

export const CHAT_HEIGHT_STORAGE_KEY = 'hivemind.chat-height';

/** Base dock height before any tray is open. */
export const DEFAULT_CHAT_HEIGHT = 560;
/**
 * Floor chosen so the transcript still gets a readable slice after the
 * header, a compact error, the cap notice, composer, hints and footer are
 * laid out (140px of transcript at the worst case).
 */
export const MIN_CHAT_HEIGHT = 460;
export const MAX_CHAT_HEIGHT = 760;

/** Height added by the CONTROLS / STATE & SETTINGS tray. */
export const DOCK_TRAY_HEIGHT = 240;

export type DockTab = 'controls' | 'state' | null;

/** Clamp a requested dock height; non-finite input falls back to the default. */
export function clampChatHeight(value: number): number {
  if (!Number.isFinite(value)) {
    return DEFAULT_CHAT_HEIGHT;
  }
  return Math.min(MAX_CHAT_HEIGHT, Math.max(MIN_CHAT_HEIGHT, Math.round(value)));
}

/** Total panel height for the current tray state. */
export function dockHeight(chatHeight: number, tab: DockTab): number {
  return clampChatHeight(chatHeight) + (tab === null ? 0 : DOCK_TRAY_HEIGHT);
}

/**
 * Read the persisted height. The two old defaults (390/480) are migrated to
 * 560 because they left too little transcript once a run refusal or the
 * session block was visible; a deliberate resize is preserved.
 *
 * `storage` is injectable so the fallback path is testable without a DOM.
 */
export function readChatHeight(storage?: Pick<Storage, 'getItem'> | null): number {
  try {
    const target =
      storage ?? (typeof localStorage === 'undefined' ? null : localStorage);
    if (target === null) {
      return DEFAULT_CHAT_HEIGHT;
    }
    const stored = target.getItem(CHAT_HEIGHT_STORAGE_KEY);
    if (stored === null) {
      return DEFAULT_CHAT_HEIGHT;
    }
    const raw = Number(stored);
    if (raw === 390 || raw === 480) {
      return DEFAULT_CHAT_HEIGHT;
    }
    return Number.isFinite(raw) ? clampChatHeight(raw) : DEFAULT_CHAT_HEIGHT;
  } catch {
    return DEFAULT_CHAT_HEIGHT;
  }
}
