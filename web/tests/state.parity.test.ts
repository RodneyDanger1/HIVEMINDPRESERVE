import { describe, expect, it } from 'vitest';
import {
  AGENT_STATES,
  DOT_STATES,
  MIN_AGENT_STATE_COUNT,
  STATE_TRANSITIONS,
  isRunActive,
  isTerminal,
  type AgentState,
} from '../src/state/agentState';
import { STRINGS } from '../src/state/strings';

/** Exact frozen 16-state vocabulary (contract: 16 declared states, five terminal). */
const FROZEN_16 = [
  'healthy',
  'running',
  'queued',
  'failed',
  'completed',
  'cancelled',
  'killed',
  'planning',
  'thinking',
  'awaiting_approval',
  'streaming',
  'reconnecting',
  'replay',
  'empty',
  'loading',
  'interrupted',
] as const;

const TERMINAL: readonly AgentState[] = ['failed', 'completed', 'cancelled', 'killed', 'interrupted'];
const NON_TERMINAL: readonly AgentState[] = FROZEN_16.filter(
  (state) => !TERMINAL.includes(state),
);

describe('16-state parity — exact list (A-F03 regression pin)', () => {
  it('is exactly the frozen 16 states, order included', () => {
    expect(AGENT_STATES).toEqual(FROZEN_16);
    expect(AGENT_STATES).toHaveLength(16);
    expect(new Set(AGENT_STATES).size).toBe(16);
    expect(AGENT_STATES.length).toBeGreaterThanOrEqual(MIN_AGENT_STATE_COUNT);
  });

  it('keeps exactly the four PDF dot states', () => {
    expect(DOT_STATES).toEqual(['healthy', 'running', 'failed', 'queued']);
  });

  it('labels every state in the single string table', () => {
    for (const state of AGENT_STATES) {
      const label = STRINGS.agentStates[state];
      expect(label, `missing label for ${state}`).toBeTruthy();
      expect(label.length).toBeGreaterThan(0);
    }
    expect(STRINGS.agentStates.interrupted).toBe('Interrupted');
  });

  it('declares a transition row for every state (matrix completeness)', () => {
    for (const state of AGENT_STATES) {
      expect(STATE_TRANSITIONS[state], `missing transition row for ${state}`).toBeDefined();
    }
  });
});

describe('terminality — five terminal states including failed (contract parity)', () => {
  it('treats failed as terminal', () => {
    expect(isTerminal('failed')).toBe(true);
  });

  it('treats all five contract terminal states as terminal', () => {
    for (const state of TERMINAL) {
      expect(isTerminal(state), `${state} must be terminal`).toBe(true);
      expect(isRunActive(state), `${state} must not be run-active`).toBe(false);
    }
  });

  it('treats every other state as non-terminal', () => {
    for (const state of NON_TERMINAL) {
      expect(isTerminal(state), `${state} must not be terminal`).toBe(false);
    }
  });

  it('does not consider failed or interrupted an active run', () => {
    expect(isRunActive('failed')).toBe(false);
    expect(isRunActive('interrupted')).toBe(false);
    // Discriminator: a running run is still active.
    expect(isRunActive('running')).toBe(true);
  });
});
