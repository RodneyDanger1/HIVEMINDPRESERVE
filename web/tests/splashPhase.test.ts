import { describe, expect, it } from 'vitest';
import {
  SPLASH_FADE_MS,
  SPLASH_MIN_VISIBLE_MS,
  SPLASH_OFFLINE_GRACE_MS,
  splashConnectionLabel,
  splashLeaveDelayMs,
  splashProgress,
  splashSettled,
  splashStatusLine,
} from '../src/splash/splashPhase';

describe('splash phase machine', () => {
  it('treats open/error/closed as settled and nothing else', () => {
    expect(splashSettled('open')).toBe(true);
    expect(splashSettled('error')).toBe(true);
    expect(splashSettled('closed')).toBe(true);
    expect(splashSettled('connecting')).toBe(false);
    expect(splashSettled('reconnecting')).toBe(false);
    expect(splashSettled('idle')).toBe(false);
  });

  it('never leaves before the minimum visible time unless skipped', () => {
    expect(splashLeaveDelayMs({ minimumElapsed: false, settled: true, skipped: false })).toBeNull();
    expect(splashLeaveDelayMs({ minimumElapsed: false, settled: false, skipped: false })).toBeNull();
    expect(splashLeaveDelayMs({ minimumElapsed: false, settled: false, skipped: true })).toBe(0);
  });

  it('leaves at once when settled and after a bounded grace when not', () => {
    expect(splashLeaveDelayMs({ minimumElapsed: true, settled: true, skipped: false })).toBe(0);
    expect(splashLeaveDelayMs({ minimumElapsed: true, settled: false, skipped: false })).toBe(
      SPLASH_OFFLINE_GRACE_MS,
    );
    expect(splashLeaveDelayMs({ minimumElapsed: true, settled: false, skipped: true })).toBe(0);
  });

  it('bounds the total gate so an offline UI cannot be blocked forever', () => {
    // worst case: the veil is visible for the minimum + the offline grace + fade
    const worstCase = SPLASH_MIN_VISIBLE_MS + SPLASH_OFFLINE_GRACE_MS + SPLASH_FADE_MS;
    expect(worstCase).toBeLessThanOrEqual(7_000);
  });

  it('reports progress and labels for every connection state', () => {
    expect(splashProgress('open')).toBe(100);
    expect(splashProgress('error')).toBe(100);
    expect(splashProgress('connecting')).toBe(46);
    expect(splashProgress('reconnecting')).toBe(18);
    expect(splashProgress('idle')).toBe(18);
    expect(splashConnectionLabel('open')).toBe('The hive is awake');
    expect(splashConnectionLabel('error')).toBe('Runtime is offline');
    expect(splashConnectionLabel('reconnecting')).toBe('Rejoining the hive');
    expect(splashConnectionLabel('connecting')).toBe('Bringing the hive online');
  });

  it('never claims a connection that has not settled', () => {
    expect(splashStatusLine('open')).toContain('Connected to the local runtime');
    expect(splashStatusLine('error')).toContain('offline');
    expect(splashStatusLine('connecting')).toContain('Opening the local socket');
  });
});
