# HiveMind agent skill

Drive HiveMind headlessly through `tools/hivemind.cmd` (Windows) or
`node tools/hivemind-cli.mjs`. stdout is JSON whenever stdout is not a TTY.
Never print or store the provider key; the server owns key handling.

## Start / inspect / stop

```powershell
tools\hivemind.cmd start --json
tools\hivemind.cmd start --open --json
tools\hivemind.cmd status --json
tools\hivemind.cmd doctor --json
tools\hivemind.cmd health --json
tools\hivemind.cmd stop --json
```

`start` is headless and does not open a browser unless `--open` is passed.
Use `open` only when a human wants the UI. If the server is already healthy,
`start` reports `already_running` and records it as external (it will not kill
it). `stop` owns only a CLI-started server; `--force --yes` is required to stop
an external/unmanaged one.

## Discover the full surface

```powershell
tools\hivemind.cmd commands --json
tools\hivemind.cmd schema --json
tools\hivemind.cmd schema submit --json
```

Prefer `schema` over guessing flags. Every command supports `--json`,
`--dry-run` where meaningful, `--url http://127.0.0.1:8099`, and
`--timeout-ms`. Destructive commands require `--yes`.

## Read

```powershell
tools\hivemind.cmd runs --json --limit 10
tools\hivemind.cmd revisions --run <run-id> --json
tools\hivemind.cmd state --run <run-id> --json
tools\hivemind.cmd messages --run <run-id> --json
tools\hivemind.cmd agent-messages --run <run-id> --json
tools\hivemind.cmd roster --run <run-id> --json
tools\hivemind.cmd approvals --json
tools\hivemind.cmd approvals --run <run-id> --json
tools\hivemind.cmd templates --json
tools\hivemind.cmd workspace --json
tools\hivemind.cmd active-run --json
tools\hivemind.cmd profiles --json
tools\hivemind.cmd memory --json
tools\hivemind.cmd memory --run <run-id> --json
tools\hivemind.cmd projects --json
tools\hivemind.cmd usage --run <run-id> --json
```

Provider reads (`models`, `balance`) refuse by name with `E_KEY_MISSING` when
the key is absent. That is a successful refusal, not a bug.

## Mutate

```powershell
tools\hivemind.cmd submit --prompt "..." --project workspace --dry-run --json
tools\hivemind.cmd submit --prompt "..." --project workspace --json
tools\hivemind.cmd run --prompt "..." --project workspace --json
tools\hivemind.cmd cancel --run <run-id> --yes --json
tools\hivemind.cmd kill --task <task-id> --yes --json
tools\hivemind.cmd rewire --task <task-id> --parent <parent-id> --expected-rev <n> --json
tools\hivemind.cmd spawn-subhive --parent <task-id> --role superagent --json
```

`submit` does not bypass key policy. A keyless submit returns
`E_KEY_MISSING` and creates no run. A run created with a valid key is followed
with `run` or `watch`.

## Approvals

```powershell
tools\hivemind.cmd watch --json --follow
tools\hivemind.cmd approve --approval <id> --nonce <nonce> --yes --json
tools\hivemind.cmd deny --approval <id> --nonce <nonce> --yes --json
```

The nonce is single-use and comes from the server's `APPROVAL_REQUEST` frame.
Never approve a tool call you have not inspected. `--reason` is optional.

## History / replay

```powershell
tools\hivemind.cmd revisions --run <run-id> --json
tools\hivemind.cmd select-revision --run <run-id> --revision 3 --json
tools\hivemind.cmd state --run <run-id> --revision 3 --json
```

## Files and UI protocol

```powershell
tools\hivemind.cmd file-tree --path docs --json
tools\hivemind.cmd file-read --path docs/README.md --json
tools\hivemind.cmd settings --json
tools\hivemind.cmd viewport --x 0 --y 0 --zoom 1 --json
tools\hivemind.cmd recenter --json
```

These are the same authority paths the UI uses; they are not a second API.

## Error handling

Check `ok`, then `error.code`, `error.retryable`, and the process exit code:

- `0` success
- `2` usage/safety
- `3` refusal / unreachable / auth / timeout
- `4` internal CLI failure

Do not retry `E_KEY_MISSING`, `E_PROVIDER_AUTH`, `E_APPROVAL_REPLAY`, or
`E_TASK_TERMINAL`. Retry `E_TIMEOUT`/`E_WS_DISCONNECTED` only after checking
`status`/`doctor`.
