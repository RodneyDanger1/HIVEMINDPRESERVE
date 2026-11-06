import { readFileSync } from 'node:fs';
import { fileURLToPath } from 'node:url';

export type ControlKind =
  | 'client-state'
  | 'consume+client-state'
  | 'consume-only'
  | 'derived'
  | 'mutating-command'
  | 'static-asset'
  | 'decision-gated';

export interface Surface {
  id: string;
  guide_ref: string;
  surface: string;
  kind: ControlKind;
  command: string | null;
  secondary_commands?: string[];
  success_events: string[];
  refusal_codes: string[];
  ack_required: boolean;
  owner: string;
  acceptance: string;
  status: 'PENDING_APP_NOT_BUILT' | 'DECISION_GATED' | 'BLOCKED';
  notes?: string;
}

export interface ControlMatrix {
  _meta: Record<string, unknown>;
  hop_names: string[];
  hop_verdict_vocabulary: string[];
  surfaces: Surface[];
}

const matrixUrl = new URL('../data/control-matrix.json', import.meta.url);

export function loadMatrix(): ControlMatrix {
  return JSON.parse(readFileSync(fileURLToPath(matrixUrl), 'utf8')) as ControlMatrix;
}

/** True while the React app does not exist (no web/src build, no dev server). */
export function appMissing(): boolean {
  return process.env.E2E_APP_READY !== '1';
}

export function skipReason(): string {
  return (
    'APP_NOT_BUILT: the wired 28-surface app is not delivered (web/src is a milestone-1 shell) and no dev ' +
    'server is running; set E2E_APP_READY=1 after the app boots. See docs/trace/QA-HARNESS.md'
  );
}

/** Frozen command vocabulary from docs/contracts/UI-COMMANDS-v1.md §2 (updated 2026-11-06T04:30). */
export const FROZEN_COMMANDS = [
  'SUBMIT_PROMPT',
  'SPAWN_SUB_HIVE',
  'REWIRE_TASK',
  'CANCEL_RUN',
  'KILL_WORKER',
  'APPROVAL_RESOLVE',
  'SETTINGS_STATUS',
  'FILE_TREE_LIST',
  'FILE_READ',
  'CHAT_PIN',
  'TEMPLATE_INSERT',
  'HISTORY_REVISION_SET',
  'VIEWPORT_SET',
  'VIEWPORT_RECENTER',
] as const;

/** Commands that are explicitly NOT frozen (decision-gated, must not ship). */
export const NOT_FROZEN_COMMANDS = ['DIFF_PROPOSE', 'DIFF_APPLY', 'DIFF_REJECT'] as const;
