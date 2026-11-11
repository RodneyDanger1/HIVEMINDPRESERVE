/**
 * Build/version identity for the UI.
 *
 * `__APP_VERSION__` is the Vite `define` seam (the Lead-owned config maps it to
 * `package.json.version`). When the define is absent (unit tests, a plain dev
 * server) the authoritative `package.json` value is used, so the UI never
 * ships a second hand-maintained version string.
 */
import packageJson from '../package.json';

declare const __APP_VERSION__: string | undefined;

export function resolveAppVersion(): string {
  if (typeof __APP_VERSION__ === 'string' && __APP_VERSION__.trim().length > 0) {
    return __APP_VERSION__;
  }
  return packageJson.version;
}

export const APP_VERSION: string = resolveAppVersion();
