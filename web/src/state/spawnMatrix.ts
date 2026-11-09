/**
 * The strict HMAS spawn class matrix, mirrored from the backend authority
 * (`execution.rs::class_may_spawn` / `validate_spawn_role`):
 *
 *   HyperAgent -> SuperAgent
 *   SuperAgent -> Agent (wire role `worker`)
 *   Agent      -> Subagent
 *   Subagent / tool / error -> cannot spawn
 *
 * The UI must never render a spawn control as enabled when the server would
 * refuse it. `spawnGate` is the single source of truth for the control's
 * disabled state and its visible reason.
 */
import { isRunActive, type AgentState, type NodeKindLabel } from './agentState';

/** Wire-role strings accepted by `role_class` in the backend. */
export type SpawnRole = 'superagent' | 'worker' | 'subagent';

export interface SpawnParent {
  readonly id: string;
  readonly kind: NodeKindLabel;
}

export interface SpawnPlan {
  readonly parentId: string;
  readonly parentKind: NodeKindLabel;
  readonly role: SpawnRole;
  /** Node kind the server will project for the child. */
  readonly childKind: NodeKindLabel;
}

const CHILD_BY_PARENT: Partial<
  Record<NodeKindLabel, { readonly role: SpawnRole; readonly childKind: NodeKindLabel }>
> = {
  hyperagent: { role: 'superagent', childKind: 'superagent' },
  superagent: { role: 'worker', childKind: 'worker' },
  worker: { role: 'subagent', childKind: 'subagent' },
};

/** The exact plan for a parent, or null when the class matrix forbids it. */
export function spawnPlanFor(parent: SpawnParent | null): SpawnPlan | null {
  if (parent === null) {
    return null;
  }
  const child = CHILD_BY_PARENT[parent.kind];
  if (child === undefined) {
    return null;
  }
  return {
    parentId: parent.id,
    parentKind: parent.kind,
    role: child.role,
    childKind: child.childKind,
  };
}

/** Why this parent cannot spawn, independent of run/transport state. */
export function spawnMatrixReason(parent: SpawnParent | null): string {
  if (parent === null) {
    return 'Select a HyperAgent, SuperAgent, or Agent card to choose the parent.';
  }
  if (parent.kind === 'subagent') {
    return 'A Subagent cannot spawn: the class matrix ends at Subagent.';
  }
  if (parent.kind === 'tool') {
    return 'A tool node is not an agent and cannot spawn.';
  }
  if (parent.kind === 'error') {
    return 'An error node is not an agent and cannot spawn.';
  }
  return 'This task class cannot spawn a child.';
}

export interface SpawnGateInput {
  readonly parent: SpawnParent | null;
  readonly runStatus: AgentState | null;
  readonly connection: string;
  readonly replaying: boolean;
  readonly pending: boolean;
}

export interface SpawnGate {
  readonly plan: SpawnPlan | null;
  /** Non-null exactly when `plan` is null; safe to render as the disabled title. */
  readonly reason: string | null;
}

/**
 * Availability of SPAWN for one parent. Ordering is deliberate: transport and
 * run preconditions are reported before class-matrix reasons, because the same
 * button must explain the first thing that actually blocks the command.
 */
export function spawnGate(input: SpawnGateInput): SpawnGate {
  if (input.replaying) {
    return { plan: null, reason: 'Replay is read-only. Exit replay to spawn a child.' };
  }
  if (input.connection !== 'open') {
    return {
      plan: null,
      reason: 'Not connected to the server; a spawn cannot be acknowledged.',
    };
  }
  if (input.runStatus === null) {
    return { plan: null, reason: 'No run yet. Submit a prompt before spawning a child.' };
  }
  if (!isRunActive(input.runStatus)) {
    return {
      plan: null,
      reason: `The run is ${input.runStatus}; no child can be spawned into it.`,
    };
  }
  if (input.pending) {
    return {
      plan: null,
      reason: 'A command is awaiting its ACK. Wait for it to settle before spawning.',
    };
  }
  const plan = spawnPlanFor(input.parent);
  if (plan === null) {
    return { plan: null, reason: spawnMatrixReason(input.parent) };
  }
  return { plan, reason: null };
}
