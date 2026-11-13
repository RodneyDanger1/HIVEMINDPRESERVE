/**
 * Splash phase machine (pure).
 *
 * The startup veil is a real boot surface: it must leave when the socket is
 * settled, must never block an offline UI forever, and must be skippable. Those
 * rules live here as plain functions so they are testable without a DOM (the
 * web test environment is `node`, not jsdom).
 */

export type SplashPhase = 'visible' | 'leaving' | 'gone';

/** Minimum time the veil stays up so the boot does not flash. */
export const SPLASH_MIN_VISIBLE_MS = 4_200;
/** Longest extra wait for a socket that never settles. */
export const SPLASH_OFFLINE_GRACE_MS = 1_800;
/** Fade-out duration before the veil unmounts. */
export const SPLASH_FADE_MS = 420;

/** A connection state that will not change by waiting (opened or refused). */
export function splashSettled(connection: string): boolean {
  return connection === 'open' || connection === 'error' || connection === 'closed';
}

export function splashProgress(connection: string): number {
  if (connection === 'open') return 100;
  // Offline is settled but not complete: showing 100% next to "runtime is
  // offline" would claim a boot that did not happen.
  if (connection === 'error' || connection === 'closed') return 0;
  return connection === 'connecting' ? 46 : 18;
}

export function splashConnectionLabel(connection: string): string {
  if (connection === 'open') return 'The hive is awake';
  if (connection === 'error' || connection === 'closed') return 'Runtime is offline';
  if (connection === 'reconnecting') return 'Rejoining the hive';
  return 'Bringing the hive online';
}

export function splashStatusLine(connection: string): string {
  if (connection === 'open') {
    return 'Connected to the local runtime. Opening the workspace.';
  }
  if (connection === 'error' || connection === 'closed') {
    return 'The local runtime is offline. The workspace will open with a named refusal.';
  }
  return 'Opening the local socket and loading the live hive state.';
}

export interface SplashLeaveInput {
  readonly minimumElapsed: boolean;
  readonly settled: boolean;
  readonly skipped: boolean;
}

/**
 * Delay before the veil starts leaving, or `null` when no timer should run yet.
 *
 * - an explicit skip wins immediately (the user asked to go on);
 * - otherwise the minimum visible time is honoured first;
 * - a settled socket leaves at once, an unsettled one gets a bounded grace.
 */
export function splashLeaveDelayMs(input: SplashLeaveInput): number | null {
  if (input.skipped) {
    return 0;
  }
  if (!input.minimumElapsed) {
    return null;
  }
  return input.settled ? 0 : SPLASH_OFFLINE_GRACE_MS;
}
