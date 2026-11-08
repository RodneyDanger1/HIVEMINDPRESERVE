# HiveMind Agent CLI

`tools/hivemind-cli.mjs` is the agent-native control surface over the same
localhost HTTP routes and WebSocket COMMAND/EVENT contract the UI uses. It is
not a second backend and does not read, print, or store the provider key.

> The shipped `hiverouter-cli.exe` is a separate, read-only diagnostics binary
> (version/preflight/schema/tools/cost/health). This `hivemind` front-door is the
> lifecycle + control client. Do not mix their command tables.

- Human wrapper: `tools/hivemind.cmd` or `tools/hivemind.ps1`
- Direct: `node tools/hivemind-cli.mjs`
- Agent skill: `skills/hivemind/SKILL.md`
- stdout: JSON when piped or `--json`; `commands`/`schema` for discovery
- stderr: human diagnostics
- Provider key: delegated to the server environment; never a CLI argument

## Global options

| Option | Meaning |
|---|---|
| `--json` | JSON envelope on stdout (automatic when stdout is not a TTY) |
| `--format json\|table\|jsonl\|human` | Explicit format; `jsonl` for `watch --follow` |
| `--url <url>` | Loopback base URL (default `http://127.0.0.1:8099` or CLI state) |
| `--data-dir <path>` | HiveMind data directory (state/logs) |
| `--workspace-root <path>` | Workspace root passed to a started server |
| `--exe <path>` | Server executable for `start` |
| `--timeout-ms <n>` | Per-command wait limit |
| `--dry-run` | Print the request/process plan; do not execute |
| `--yes` | Confirm a destructive operation |
| `--no-color` | Human output without color |

Environment fallbacks: `HIVEROUTER_URL`, `HIVEMIND_URL`,
`HIVEROUTER_DATA_DIR`, `HIVEROUTER_WORKSPACE_ROOT`. Only loopback URLs are
accepted. No environment variable is required for normal operation.

## Exit codes

| Code | Class |
|---|---|
| 0 | success |
| 2 | usage/validation/safety |
| 3 | named refusal, server unreachable, auth, or timeout (see `error.code` and `error.retryable`) |
| 4 | internal CLI failure |

This aligns with the shipped `hiverouter-cli` classes (0/2/3/4); semantic detail
comes from `error.code`, not from extra process codes.

## Full command list (49 commands)

### Lifecycle and local

| Command | Safety | Transport | Purpose |
|---|---|---|---|
| `start` | mutating | process | Start headless server, wait for `/health`; `--no-wait`, `--port`, `--bind`, `--data-dir`, `--db`, `--workspace-root`, `--exe`, `--open` |
| `stop` | destructive | process | Stop CLI-owned server; `--force` requires `--yes`; escalates once and reports `graceful:false` if Windows does not honour term |
| `restart` | destructive | process | Stop then start; local only |
| `status` | read | local/http | CLI state + health + current URL |
| `doctor` | read | local/http | Executable, state, loopback health, key presence boolean, schema |
| `open` | mutating | process | Open UI in Edge/Chrome app mode; `--browser`, `--no-app` |
| `logs` | read | local | Tail stdout/stderr from data dir; `--lines`, `--stream` |
| `commands` | read | local | Every command with safety and transport |
| `schema [command]` | read | local | Typed command/argument schema |
| `version` | read | local | CLI and contract version |

### Read-only server state

| Command | Transport | Server route / source |
|---|---|---|
| `health` | HTTP | `GET /health` |
| `diagnose` | HTTP | `GET /diagnose` |
| `runs` | HTTP | `GET /api/history/runs`; `--limit` |
| `revisions` | HTTP | `GET /api/history/runs/{run}/revisions`; `--run` |
| `state` | HTTP | `GET /api/history/runs/{run}/state?revision=`; latest if omitted |
| `messages` | HTTP | `GET /api/history/runs/{run}/messages`; newest window |
| `agent-messages` | HTTP | `GET /api/agents/runs/{run}/messages` |
| `roster` | HTTP | `GET /api/agents/runs/{run}/roster` |
| `approvals` | HTTP | `GET /api/approvals`; optional `--run` |
| `templates` | HTTP | `GET /api/templates` |
| `workspace` | HTTP | `GET /api/workspace` |
| `active-run` | HTTP | `GET /api/runs/active` |
| `profiles` | HTTP | `GET /api/agents/profiles` |
| `session` | HTTP | `GET /api/session`; token redacted in output |
| `memory` | HTTP | `GET /api/agents/memory` |
| `projects` | HTTP | `GET /api/projects` |
| `models` | HTTP | `GET /api/provider/models`; keyless refuses |
| `balance` | HTTP | `GET /api/provider/balance`; keyless refuses |
| `usage` | HTTP | `GET /api/usage/runs/{run}` |

### Mutating commands

| Command | Safety | Transport | Required / optional args |
|---|---|---|---|
| `submit` | mutating | WS `SUBMIT_PROMPT` | `--prompt` or `--prompt-file`; `--project`; `--wait` |
| `run` | mutating | WS `SUBMIT_PROMPT` + watch | `--prompt`/`--prompt-file`; `--project`; timeout |
| `cancel` | destructive | WS `CANCEL_RUN` | `--run`; requires `--yes` |
| `kill` | destructive | WS `KILL_WORKER` | `--task`; requires `--yes` |
| `rewire` | mutating | WS `REWIRE_TASK` | `--task`, `--parent`, `--expected-rev` |
| `spawn-subhive` | mutating | WS `SPAWN_SUB_HIVE` | `--parent`, `--role` |
| `approve` | destructive | WS `APPROVAL_RESOLVE` | `--approval`, `--nonce`; optional `--reason`; requires `--yes` |
| `deny` | destructive | WS `APPROVAL_RESOLVE` | `--approval`, `--nonce`; optional `--reason`; requires `--yes` |
| `template` | mutating | WS `TEMPLATE_INSERT` | `--template`, `--project` |
| `profile-import` | mutating | HTTP POST | `--file` profile JSON |
| `profile-delete` | destructive | HTTP DELETE | `--id`; requires `--yes` |
| `memory-clear` | destructive | HTTP DELETE | `--agent`; requires `--yes` |
| `chat-pin` | mutating | WS `CHAT_PIN` | `--pinned true\|false` |
| `viewport` | mutating | WS `VIEWPORT_SET` | `--x`, `--y`, `--zoom` |
| `recenter` | mutating | WS `VIEWPORT_RECENTER` | none |

### Protocol/replay/monitoring

| Command | Safety | Transport | Purpose |
|---|---|---|---|
| `select-revision` | read | WS `HISTORY_REVISION_SET` | Request a committed snapshot; returns `HISTORY_SNAPSHOT` |
| `settings` | read | WS `SETTINGS_STATUS` | Effective settings, key presence boolean |
| `file-tree` | read | WS `FILE_TREE_LIST` | Jailed workspace directory; `--path` |
| `file-read` | read | WS `FILE_READ` | Jailed workspace file; `--path` |
| `watch` | read | WS events | Bounded event collection; `--run`, `--max-events`, `--timeout-ms`, `--follow` |

`watch --follow` emits JSONL event envelopes (`--format jsonl`/non-TTY).
Non-follow mode returns `{events, truncated, timed_out}`.

## Coverage matrix — every server surface

### HTTP

| Server route | CLI |
|---|---|
| `GET /health` | `health` |
| `GET /diagnose` | `diagnose` |
| `GET /api/session` | `session` (token redacted) |
| `GET /api/history/runs` | `runs` |
| `GET /api/history/runs/{run}/revisions` | `revisions` |
| `GET /api/history/runs/{run}/state` | `state` |
| `GET /api/history/runs/{run}/messages` | `messages` |
| `GET /api/agents/runs/{run}/messages` | `agent-messages` |
| `GET /api/agents/runs/{run}/roster` | `roster` |
| `GET /api/approvals` | `approvals` |
| `GET /api/templates` | `templates` |
| `GET /api/workspace` | `workspace` |
| `GET /api/runs/active` | `active-run` |
| `GET /api/agents/profiles` | `profiles` |
| `POST /api/agents/profiles/import` | `profile-import` |
| `DELETE /api/agents/profiles/{id}` | `profile-delete --yes` |
| `GET /api/agents/memory` | `memory` |
| `DELETE /api/agents/memory/{key}` | `memory-clear --yes` |
| `GET /api/usage/runs/{run}` | `usage` |
| `GET /api/projects` | `projects` |
| `GET /api/provider/models` | `models` |
| `GET /api/provider/balance` | `balance` |
| `GET /` and static UI | `open` (human action; headless does not need it) |

### WebSocket commands

| Wire command | CLI |
|---|---|
| `SUBMIT_PROMPT` | `submit`, `run` |
| `SPAWN_SUB_HIVE` | `spawn-subhive` |
| `REWIRE_TASK` | `rewire` |
| `CANCEL_RUN` | `cancel --yes` |
| `KILL_WORKER` | `kill --yes` |
| `APPROVAL_RESOLVE` | `approve --yes`, `deny --yes` |
| `SETTINGS_STATUS` | `settings` |
| `FILE_TREE_LIST` | `file-tree` |
| `FILE_READ` | `file-read` |
| `CHAT_PIN` | `chat-pin` |
| `TEMPLATE_INSERT` | `template` |
| `HISTORY_REVISION_SET` | `select-revision` |
| `VIEWPORT_SET` | `viewport` |
| `VIEWPORT_RECENTER` | `recenter` |
| `DIFF_PROPOSE`, `DIFF_APPLY`, `DIFF_REJECT` | Not exposed: decision-gated contract, server refuses by name |
| server `EVENT` stream | `watch` |

## Not exposed on purpose


| Surface | Why |
|---|---|
| Raw `/api/session` token | The CLI uses it internally; `session` redacts it |
| Diff propose/apply/reject | Contract decision-gated and not frozen; server refuses `E_DECISION_REQUIRED` |
| Arbitrary process kill | `stop` refuses an external or unrecorded PID |
| Provider key set/read | Delegated to the server environment; never a CLI secret |
| Direct browser control | Only `open` launches a UI; all work is headless-capable |

## Agent loop

1. `doctor` / `start` → capture `url`, `pid`, health.
2. `commands` and `schema <command>` → discover exact arguments.
3. `--dry-run` for a mutating command → preview.
4. Execute; parse `ok`, `data`, `error.code`, `error.retryable`.
5. Use `runs`/`state`/`roster`/`watch` for bounded observation.
6. `stop --yes` only for a CLI-owned server.

## Verification status

The command tree is generated from the CLI's own registry and maps to real
routes/commands in source. Runtime verification of each command against a
fresh server is **not claimed** in this document; it is a separate gate and
must be run with the key absent so provider commands refuse by name.
