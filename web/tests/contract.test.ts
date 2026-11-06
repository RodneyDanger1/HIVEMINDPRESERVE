import { describe, expect, it } from 'vitest';
import {
  FROZEN_COMMAND_NAMES,
  EVENT_NAMES,
  KNOWN_REFUSAL_CODES,
  SCHEMA_VERSION,
  buildCommandEnvelope,
  parseEventFrame,
  validateEnvelope,
  validatePromptText,
  validateWorkspacePath,
  validateZoom,
  checkFileSize,
} from '../src/api/contract';
import { uuidv7, isUuidV7 } from '../src/api/ids';

const newId = (): string => '00000000-0000-7000-8000-000000000000';
const now = (): string => '2026-11-06T00:00:00.000Z';

function frame(name: string, payload: unknown, correlation = newId()): unknown {
  return {
    schema: SCHEMA_VERSION,
    type: 'EVENT',
    name,
    id: newId(),
    correlation_id: correlation,
    run_id: null,
    task_id: null,
    seq: 1,
    ts: now(),
    payload,
  };
}

describe('frozen contract (UI-COMMANDS-v1)', () => {
  it('declares all 14 frozen commands including the patched ones', () => {
    expect(FROZEN_COMMAND_NAMES).toHaveLength(14);
    for (const name of [
      'APPROVAL_RESOLVE',
      'SETTINGS_STATUS',
      'FILE_TREE_LIST',
      'FILE_READ',
      'SUBMIT_PROMPT',
      'HISTORY_REVISION_SET',
    ]) {
      expect(FROZEN_COMMAND_NAMES).toContain(name);
    }
  });

  it('declares all 21 events', () => {
    expect(EVENT_NAMES).toHaveLength(21);
    expect(EVENT_NAMES).toContain('SETTINGS_STATUS_RESULT');
    expect(EVENT_NAMES).toContain('FILE_TREE_RESULT');
    expect(EVENT_NAMES).toContain('FILE_CONTENT');
  });

  it('contains the patched refusal codes', () => {
    for (const code of [
      'E_APPROVAL_NOT_FOUND',
      'E_APPROVAL_DECISION_INVALID',
      'E_PATH_OUTSIDE_JAIL',
      'E_PATH_NOT_FOUND',
      'E_FILE_TOO_LARGE',
    ]) {
      expect(KNOWN_REFUSAL_CODES).toContain(code);
    }
  });

  it('validates envelopes and rejects malformed frames', () => {
    const ok = validateEnvelope(frame('ACK', { command: 'CHAT_PIN', correlation_id: newId() }));
    expect(ok.ok).toBe(true);
    expect(validateEnvelope({ schema: 'nope' }).ok).toBe(false);
    expect(validateEnvelope(frame('NOT_AN_EVENT', {})).ok).toBe(false);
  });

  it('parses typed ACK / ERROR / NODE_* payloads', () => {
    const ack = parseEventFrame(
      JSON.stringify(frame('ACK', { command: 'SETTINGS_STATUS', correlation_id: 'c-1' }, 'c-1')),
    );
    expect(ack.ok && ack.value.event.name).toBe('ACK');

    const error = parseEventFrame(
      JSON.stringify(
        frame('ERROR', { code: 'E_KEY_MISSING', message: 'm', how_to_fix: 'f', retryable: false }),
      ),
    );
    expect(error.ok && error.value.event.name === 'ERROR' && error.value.event.payload.code).toBe(
      'E_KEY_MISSING',
    );

    const node = parseEventFrame(
      JSON.stringify(
        frame('NODE_SPAWN', {
          task_id: 't1',
          state: 'running',
          kind: 'superagent',
          label: 'Backend Architecture',
          parent_task_id: null,
          progress: 10,
        }),
      ),
    );
    expect(node.ok && node.value.event.name).toBe('NODE_SPAWN');
    expect(node.ok && node.value.event.name === 'NODE_SPAWN' && node.value.event.payload.kind).toBe(
      'superagent',
    );
  });

  it('builds COMMAND envelopes with schema/id/correlation', () => {
    const envelope = buildCommandEnvelope('CHAT_PIN', { pinned: true }, { newId, now });
    expect(envelope.schema).toBe(SCHEMA_VERSION);
    expect(envelope.type).toBe('COMMAND');
    expect(envelope.correlation_id).toBe(envelope.id);
    expect(envelope.ts).toBe(now());
    expect(isUuidV7(uuidv7(0))).toBe(true);
  });

  it('enforces frozen preconditions', () => {
    expect(validatePromptText('')).toBe('E_PROMPT_EMPTY');
    expect(validatePromptText('x'.repeat(8001))).toBe('E_PROMPT_TOO_LONG');
    expect(validatePromptText('hello')).toBeNull();
    expect(validateZoom(0.05)).toBe('E_ZOOM_RANGE');
    expect(validateZoom(1.5)).toBeNull();
    expect(validateWorkspacePath('/etc/passwd')).toBe('E_PATH_OUTSIDE_JAIL');
    expect(validateWorkspacePath('src\\..\\secret.txt')).toBe('E_PATH_OUTSIDE_JAIL');
    expect(validateWorkspacePath('src/main.tsx')).toBeNull();
    expect(checkFileSize(2 * 1024 * 1024)).toBe('E_FILE_TOO_LARGE');
    expect(checkFileSize(1024)).toBeNull();
  });

  it('uuidv7 carries version and variant bits', () => {
    const value = uuidv7(1_700_000_000_000);
    expect(isUuidV7(value)).toBe(true);
    // RFC 9562 variant: first hex of the fourth group is 8, 9, a or b.
    expect(value.charAt(19)).toMatch(/^[89ab]$/);
  });
});
