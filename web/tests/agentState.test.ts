import { describe, expect, it } from 'vitest';
import {
  AGENT_STATES,
  DOT_STATES,
  MIN_AGENT_STATE_COUNT,
  canTransition,
  dotFor,
  isAgentState,
  isTerminal,
  visualFor,
} from '../src/state/agentState';
import { STRINGS } from '../src/state/strings';

describe('single agent-state vocabulary (R-U4)', () => {
  it('has >= 13 states and unique values', () => {
    expect(AGENT_STATES.length).toBeGreaterThanOrEqual(MIN_AGENT_STATE_COUNT);
    expect(new Set(AGENT_STATES).size).toBe(AGENT_STATES.length);
  });

  it('maps exactly the four PDF dots', () => {
    expect(DOT_STATES).toEqual(['healthy', 'running', 'failed', 'queued']);
    for (const state of AGENT_STATES) {
      expect(dotFor(state)).toBe(DOT_STATES.includes(state as never) ? state : null);
    }
  });

  it('every non-dot state still renders a shape/text cue', () => {
    for (const state of AGENT_STATES) {
      const visual = visualFor(state);
      expect(visual.label.length).toBeGreaterThan(0);
      expect(STRINGS.agentStates[state]).toBe(visual.label);
      if (dotFor(state) === null) {
        expect(visual.shape).not.toBe('dot');
        expect(visual.dot).toBeNull();
      } else {
        expect(visual.shape).toBe('dot');
      }
    }
  });

  it('transition matrix only allows declared changes', () => {
    expect(canTransition('queued', 'running')).toBe(true);
    expect(canTransition('running', 'completed')).toBe(true);
    expect(canTransition('completed', 'running')).toBe(false);
    expect(canTransition('killed', 'running')).toBe(false);
    expect(isTerminal('killed')).toBe(true);
    expect(isTerminal('streaming')).toBe(false);
  });

  it('guards unknown wire values', () => {
    expect(isAgentState('healthy')).toBe(true);
    expect(isAgentState('definitely-not-a-state')).toBe(false);
  });
});
