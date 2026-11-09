# External agent drivers — Hermes, DeepSeek Harness, and any shell agent

## What the application is

HiveMind is **local-first**, not a hosted website:

- `target/release/hiverouter.exe` (or `dist/HiveMind/HiveMind.exe`) starts the Rust
  server on a loopback address.
- The same binary embeds and serves the React/FigCraft UI.
- A human can open the loopback URL in a browser, or use the existing
  WebView2 desktop launcher (`tools/build-desktop.ps1`). A browser is **not**
  required for operation; it is only the renderer for the UI.
- No cloud service, Docker, Python, or remote database is required.

## External driver surface

Any desktop agent with a shell can drive the same authority paths the UI uses:

```powershell
# Repo root
.\HiveMind-Agent.cmd commands --json
.\HiveMind-Agent.cmd schema submit --json
.\HiveMind-Agent.cmd doctor --json
.\HiveMind-Agent.cmd start --json
.\HiveMind-Agent.cmd health --json
.\HiveMind-Agent.cmd runtime --json
.\HiveMind-Agent.cmd submit --prompt "Inspect this workspace" --project workspace --json
.\HiveMind-Agent.cmd watch --json --follow
.\HiveMind-Agent.cmd runs --json --limit 10
.\HiveMind-Agent.cmd roster --run <run-id> --json
.\HiveMind-Agent.cmd approvals --run <run-id> --json
.\HiveMind-Agent.cmd stop --json
```

The CLI is `tools/hivemind-cli.mjs`; `HiveMind-Agent.cmd` is the stable Windows
entry point. It emits JSON when piped, never prints the provider key, and uses
the same HTTP/WS contract as the UI. It is not a second backend.

### Driver protocol

1. `doctor --json` — confirm executable, state directory, loopback health and
   key-presence boolean.
2. `start --json` — headless start; add `--open` only for a human UI.
3. `commands --json` + `schema <command> --json` — discover exact flags.
4. `submit` or `run` — submit a prompt; keyless submit refuses by name and
   creates no run.
5. `watch --json --follow` — bounded event stream; use `runs`, `state`,
   `messages`, `roster`, `agent-messages`, `approvals` for pull reads.
6. Stop only the CLI-owned server; external servers require `--force --yes`.

This works from Hermes AI Agent Desktop, DeepSeek Harness Desktop, or any agent
that can run a local shell command. No browser automation is required for the
headless path.

### UI path

For a human-visible session:

```powershell
.\HiveMind-Agent.cmd open --json
# or the packaged desktop launcher:
powershell -ExecutionPolicy Bypass -File tools\build-desktop.ps1
.\dist\HiveMind\HiveMind.exe
```

The desktop launcher uses an Edge WebView2 app window around the same loopback
UI; it is not a separate web application.

## Provider boundary — important

HiveMind's **internal** HyperAgent/SuperAgent/Agent calls currently use the
DeepSeek client only. An external desktop agent can:

- start/stop the server,
- submit and observe runs,
- answer approvals/A2A questions,
- read state, roster, memory and audit,
- edit code/scripts and rebuild the app.

It **cannot** transparently become the model provider for internal HiveMind
agents. To run internal agents on a free/OpenAI-compatible endpoint, HiveMind
needs a provider adapter (endpoint/model/auth/capability mapping, cost accounting,
and refusal semantics). That is not implemented and would be an explicit,
separate change; the current DeepSeek-only refusal behavior must not be
silently weakened.

If the DeepSeek API is exhausted, use Hermes/DeepSeek Harness as the external
operator: it can still run HiveMind's control plane, inspect existing runs,
repair/build the app, and drive fixture/remote/daemon paths. It cannot make
internal agents think without a configured provider adapter.

## Current verification status

- CLI command tree and schema are source-verified; full runtime command-by-command
  verification against a fresh server is not claimed here.
- No live DeepSeek call or app launch was performed for this document.
- Hermes/DeepSeek Harness compatibility is by **shell + loopback HTTP/WS contract**,
  not claimed as a vendor-certified integration.
