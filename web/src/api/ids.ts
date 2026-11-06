/**
 * UUIDv7 ids (UI-COMMANDS-v1 §1: `id`/`correlation_id` are uuid-v7).
 * RFC 9562 layout: 48-bit unix-ms timestamp, version 7, variant RFC.
 */
const UUID_RE = /^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$/i;

function randomBytes(): Uint8Array {
  const bytes = new Uint8Array(16);
  if (typeof crypto !== 'undefined' && typeof crypto.getRandomValues === 'function') {
    crypto.getRandomValues(bytes);
    return bytes;
  }
  throw new Error('Web Crypto getRandomValues is not available; cannot mint contract ids');
}

export function uuidv7(nowMs: number = Date.now()): string {
  if (!Number.isSafeInteger(nowMs) || nowMs < 0) {
    throw new Error('uuidv7 timestamp must be a non-negative integer ms value');
  }
  const bytes = randomBytes();
  let timestamp = BigInt(nowMs);
  for (let index = 5; index >= 0; index -= 1) {
    bytes[index] = Number(timestamp & 0xffn);
    timestamp >>= 8n;
  }
  bytes[6] = 0x70 | (bytes[6] & 0x0f);
  bytes[8] = 0x80 | (bytes[8] & 0x3f);
  const hex = Array.from(bytes, (byte) => byte.toString(16).padStart(2, '0')).join('');
  return `${hex.slice(0, 8)}-${hex.slice(8, 12)}-${hex.slice(12, 16)}-${hex.slice(16, 20)}-${hex.slice(20)}`;
}

export function isUuid(value: unknown): value is string {
  return typeof value === 'string' && UUID_RE.test(value);
}

export function isUuidV7(value: unknown): value is string {
  return isUuid(value) && value.charAt(14) === '7';
}

export function shortId(id: string): string {
  return id.length > 8 ? id.slice(0, 8) : id;
}

export function nowIso(): string {
  return new Date().toISOString();
}
