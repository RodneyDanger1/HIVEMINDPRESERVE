/**
 * Composer session-policy truth (regression pin).
 *
 * A named session stores its own `permission_policy` on the server (`sessions`
 * table, returned by `GET /api/sessions`). The composer used to submit
 * `policy: 'classic'` from its initial state no matter which session was
 * selected, so a `plan` session (mutations refused) could be silently submitted
 * as `classic` (approve every side effect). This test executes the real
 * session-`<select>` onChange handler extracted from ChatCore.tsx and pins two
 * facts: selecting a named session applies that session's stored policy, and
 * the submitted payload reads the same `permissionPolicy` state the selector
 * shows.
 *
 * This is source-check style (`?raw`), like the rest of web/tests: the handler
 * is a closure over component state, no pure helper is exported for it, and
 * this Vitest environment is `node` (no jsdom), so there is no DOM to fire a
 * change event at. The extracted body is plain JS, so `new Function` runs the
 * shipped handler with recording setters instead of adding a test-only seam to
 * product code.
 */
import { describe, expect, it } from 'vitest';
import chatCoreSource from '../src/chat/ChatCore.tsx?raw';

const POLICIES = ['classic', 'plan', 'standard', 'trusted'] as const;
type Policy = (typeof POLICIES)[number];

interface SessionRow {
  readonly id: string;
  readonly name: string;
  readonly archived_at?: string | null;
  readonly permission_policy?: Policy;
}

interface SelectEvent {
  readonly target: { readonly value: string };
}

type SelectHandler = (
  event: SelectEvent,
  sessions: readonly SessionRow[],
  setNamedSession: (sessionId: string) => void,
  setActiveSessionId: (sessionId: string | null) => void,
  setPermissionPolicy: (policy: Policy) => void,
) => void;

interface SubmitOptionsBuilder {
  (namedSession: string, permissionPolicy: Policy): {
    readonly sessionId?: string;
    readonly policy?: Policy;
  };
}

/**
 * `source[openIndex..]` through the brace that closes the block opened there.
 * Quotes are skipped so a future string literal cannot unbalance the scan.
 */
function balancedBlock(source: string, openIndex: number): string {
  if (source[openIndex] !== '{') {
    throw new Error(`expected '{' at index ${openIndex}`);
  }
  let depth = 0;
  let quote: string | null = null;
  for (let index = openIndex; index < source.length; index += 1) {
    const char = source[index];
    if (quote !== null) {
      if (char === '\\') {
        index += 1;
      } else if (char === quote) {
        quote = null;
      }
      continue;
    }
    if (char === "'" || char === '"' || char === '`') {
      quote = char;
      continue;
    }
    if (char === '{') {
      depth += 1;
    } else if (char === '}') {
      depth -= 1;
      if (depth === 0) {
        return source.slice(openIndex, index + 1);
      }
    }
  }
  throw new Error(`unbalanced block starting at index ${openIndex}`);
}

/** The `{ ... }` body of the session `<select>` onChange handler. */
function sessionSelectHandlerBody(): string {
  const anchor = chatCoreSource.indexOf('value={namedSession}');
  if (anchor < 0) {
    throw new Error('ChatCore.tsx: session <select value={namedSession}> not found');
  }
  const changeAt = chatCoreSource.indexOf('onChange=', anchor);
  if (changeAt < 0) {
    throw new Error('ChatCore.tsx: session select onChange not found');
  }
  const arrowAt = chatCoreSource.indexOf('=>', changeAt);
  if (arrowAt < 0) {
    throw new Error('ChatCore.tsx: session select onChange is not an arrow function');
  }
  const openAt = chatCoreSource.indexOf('{', arrowAt);
  if (openAt < 0) {
    throw new Error('ChatCore.tsx: session select onChange has no block body');
  }
  return balancedBlock(chatCoreSource, openAt);
}

/** The `{ ... }` object literal the composer passes to `submitPrompt`. */
function submitOptionsSource(): string {
  const anchor = 'const submitOptions =';
  const at = chatCoreSource.indexOf(anchor);
  if (at < 0) {
    throw new Error('ChatCore.tsx: submitOptions not found');
  }
  const openAt = chatCoreSource.indexOf('{', at);
  if (openAt < 0) {
    throw new Error('ChatCore.tsx: submitOptions is not an object literal');
  }
  return balancedBlock(chatCoreSource, openAt);
}

function compileSessionSelectHandler(): SelectHandler {
  const body = sessionSelectHandlerBody();
  return new Function(
    'event',
    'sessions',
    'setNamedSession',
    'setActiveSessionId',
    'setPermissionPolicy',
    body,
  ) as SelectHandler;
}

function compileSubmitOptions(): SubmitOptionsBuilder {
  const objectSource = submitOptionsSource();
  return new Function(
    'namedSession',
    'permissionPolicy',
    `return ${objectSource};`,
  ) as SubmitOptionsBuilder;
}

interface SelectionEffects {
  readonly namedSession: () => string;
  readonly activeSessionId: () => string | null;
  readonly appliedPolicies: readonly Policy[];
}

/** Run the shipped onChange handler against a session list and record effects. */
function selectSession(
  sessions: readonly SessionRow[],
  selectedId: string,
): SelectionEffects {
  const handler = compileSessionSelectHandler();
  let namedSession = '';
  let activeSessionId: string | null = null;
  const appliedPolicies: Policy[] = [];
  handler(
    { target: { value: selectedId } },
    sessions,
    (sessionId) => {
      namedSession = sessionId;
    },
    (sessionId) => {
      activeSessionId = sessionId;
    },
    (policy) => {
      appliedPolicies.push(policy);
    },
  );
  return {
    namedSession: () => namedSession,
    activeSessionId: () => activeSessionId,
    appliedPolicies,
  };
}

describe('composer session policy truth (plan must not submit as classic)', () => {
  it('applies the selected session permission_policy for every policy value', () => {
    for (const policy of POLICIES) {
      const row: SessionRow = {
        id: `session-${policy}`,
        name: `${policy} session`,
        permission_policy: policy,
      };
      const effects = selectSession([row], row.id);
      expect(effects.appliedPolicies, `stored policy for ${row.id}`).toEqual([policy]);
      expect(effects.namedSession()).toBe(row.id);
      expect(effects.activeSessionId()).toBe(row.id);
    }
  });

  it('submits the policy the selector shows, so a plan session is not sent as classic', () => {
    const planSession: SessionRow = {
      id: 'session-plan',
      name: 'Plan-only session',
      permission_policy: 'plan',
    };
    // Composer starts at 'classic' (the default that used to leak through).
    const effects = selectSession([planSession], planSession.id);
    const selectedPolicy = effects.appliedPolicies.at(-1);
    expect(selectedPolicy).toBe('plan');
    expect(selectedPolicy).not.toBe('classic');

    const payload = compileSubmitOptions()(effects.namedSession(), selectedPolicy!);
    expect(payload).toEqual({ sessionId: 'session-plan', policy: 'plan' });
    expect(payload.policy).not.toBe('classic');
  });

  it('never applies an undefined policy when the server omits permission_policy', () => {
    const legacy: SessionRow = { id: 'session-legacy', name: 'Legacy session' };
    const effects = selectSession([legacy], legacy.id);
    const invalid = effects.appliedPolicies.filter(
      (policy) => !(POLICIES as readonly string[]).includes(policy as string),
    );
    expect(invalid).toEqual([]);
  });

  it('keeps the sessions payload field and one policy state for selector and submit', () => {
    const handlerBody = sessionSelectHandlerBody();
    // The handler must look the chosen row up and copy its stored field...
    expect(handlerBody).toContain('sessions.find');
    expect(handlerBody).toContain('chosen.permission_policy');
    // ...never substitute a hardcoded policy for a plan session.
    expect(handlerBody).not.toMatch(/['"](classic|plan|standard|trusted)['"]/);

    // Dropping permission_policy from the fetched row type is how the selector
    // goes back to guessing 'classic': both session shapes must keep it.
    expect((chatCoreSource.match(/permission_policy\?:/g) ?? []).length).toBeGreaterThanOrEqual(2);

    // One policy state drives the visible selector and the command payload.
    expect(chatCoreSource).toContain('value={permissionPolicy}');
    expect(chatCoreSource).toContain('policy: permissionPolicy');
    expect((chatCoreSource.match(/submitPrompt\(text, submitOptions\)/g) ?? []).length).toBe(2);
    expect(compileSubmitOptions()('', 'classic')).toEqual({ policy: 'classic' });
  });
});
