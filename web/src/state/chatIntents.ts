/**
 * Root-control intents for the HyperAgent chat surface.
 *
 * `spawn_remote_agent` and `spawn_daemon` are model tools inside the
 * HyperAgent's bounded root-control turn (`execution.rs::root_checkpoint`);
 * they are NOT wire commands. The only supported front-end path is a
 * SUBMIT_PROMPT whose text asks for them. The server decides whether to call
 * them, so this module only produces prompt text — it never claims a spawn.
 */
export interface ControlIntent {
  readonly id: string;
  readonly label: string;
  readonly prompt: string;
}

export const CONTROL_INTENTS: readonly ControlIntent[] = [
  {
    id: 'spawn-remote-agent',
    label: 'Spawn remote agent',
    prompt:
      'Use your root-control tools to spawn a detached remote agent (a light principal named "Operator Light") for this run. Report the server-assigned remote id, or the exact refusal.',
  },
  {
    id: 'spawn-daemon',
    label: 'Spawn daemon',
    prompt:
      'Use your root-control tools to spawn a daemon/watchdog named "Hive watchdog" with a 60 second interval for this run. Report the server-assigned daemon id, or the exact refusal.',
  },
];
