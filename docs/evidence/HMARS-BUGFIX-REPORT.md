# HiveMind HMARS Bug-Fix Report

Date: 2026-11-09  
Repo: `C:\Users\Riley\Desktop\hiverouter-v1`  
GitHub: `https://github.com/RodneyDanger1/HiveMind`  
Base before this repair wave: `b039037`  
Final repair HEAD at report write: `717944c`  
Release binary: `target/release/hiverouter.exe` SHA-256 `ACA64B32F896EB466A448BC27D92A03601EA99FADC751AD824BCFA1B1E3DE4D7`  
Portable package: `dist/HiveMind.zip` SHA-256 `BDB35071309F96D8932983C039E458C4B832A5F8D322105431503821E6A6AA22`  
Packaged server: `dist/HiveMind/hiverouter.exe` SHA-256 `ACA64B32F896EB466A448BC27D92A03601EA99FADC751AD824BCFA1B1E3DE4D7`

## 1. What HiveMind is supposed to be

HiveMind is a local HMARS framework. One governing HyperAgent owns the run:
it plans, validates the hierarchy, governs the bounded root control turn, and
synthesizes the final answer. SuperAgents own hive branches and spawn Agents.
Agents own branch memory and may spawn temporary Subagents. Subagents cannot
spawn. Remote agents, light/garage controls and daemons are detached
control-plane records, never task rows or graph nodes.

```text
SUBMIT(prompt, project):
  validate prompt
  refuse if no key or another run is active
  create run + user message + receipt
  dispatch in background

DISPATCH(run):
  run.status = Planning
  claim one provider call
  plan = provider.plan(prompt)
  plan.validate()                       # ids, deps, parent depth, no cycle,
                                        # no parent+descendant dependency
  insert root HyperAgent task
  for planned node in depth order:
      parent = explicit parent or inferred depth-1 dep
      validate role class vs real parent class
      enforce depth 3 / fanout 8 / agents 64
      insert task row + graph node + bank + context
      publish NODE_SPAWN
  root_checkpoint (one bounded turn, max 4 root tools)
  Planning -> Running, snapshot
  dispatch ready nodes in bounded parallel batches
  each task: memory read set -> provider/tool loop -> result
  aggregate children into parents
  root final synthesis (answer only)
  terminal status + snapshot + usage

HIERARCHY:
  HyperAgent -> SuperAgent -> Agent -> Subagent
  Subagent -> none
  remote/daemons are tree-free records

MEMORY:
  run bank -> hive bank -> branch bank
  read: run + nearest hive + own/parent branch
  no sibling/foreign run
  HyperAgent: bounded run/hive summaries only

A2A:
  ask_parent -> durable open message + deadline
  -> parent bounded answer or operator mailbox
  -> CAS answered once, wake waiter, never fabricate
  -> timeout re-reads before expiring

REMOTE:
  create -> heartbeat/lease -> online
  action requires online + live lease + unrevoked remote/device + grant
  critical action: nonce + expiry approval, resolve revalidates everything
  boot: executing -> interrupted; never replay

DAEMON:
  durable office, no task row
  tick must report what actually ran; no fabricated alive

TRUTH:
  no invented success, no zero for unknown, no provider cost without provenance,
  every refusal named, crash-safe snapshots.
```

## 2. Live failures that drove this wave

| Live error | Root cause | Fixed |
|---|---|---|
| `E_PROVIDER_INVALID_RESPONSE` | DeepSeek-only `thinking`, `reasoning_effort`, `response_format`, and assistant `reasoning_content` sent to an OpenAI-compatible endpoint | External request translation strips all four; tests added |
| `E_USAGE_INCOMPLETE` | Token Lab reports `prompt_tokens_details.cached_tokens`, not DeepSeek's cache split | Usage conversion derives hit/miss; tests added |
| `E_CALL_LIMIT` | Root control loop kept calling tools until the cap failed the run | One bounded root turn, max 4 tools; run continues to dispatch |
| `E_SCHEMA_INVALID` | Planner emitted depth/deps without `parent`; inference plus a retained dependency deadlocked parent/child | Parent inferred from depth-matching dep, that dep removed; plan validation rejects parent+descendant dependency and parentless depth>1 |
| `E_LLM_RATE_LIMIT` | No request-rate governor; external runs can burst | Shared RPM governor (20/min external) on every attempt including retries; external task parallelism clamped to 2 |
| False provider cost | External points were priced as DeepSeek USD | External usage is measured and unpriced (`settled_usd = null`); DeepSeek pricing unchanged |
| False daemon success | Tick always wrote `outcome='alive'` without running a job | Tick marks `degraded` with `executed:false` and a reason; no fake alive |
| Inert root remote | HyperAgent-spawned light/garage had no device/grant | Root spawn creates one device + one narrow grant for light/garage; relay remains records-only |

## 3. Workstreams and commits

| Commit | Area | Summary |
|---|---|---|
| `632cfc5` | UI | truthful provider/model labels, independent models/balance fetch, confirmed A2A acceptance, 11px text floor |
| `e5c0799` | Graph | HyperAgent pinned at origin, radial outward branches, ancestor edges, live drag overlay, nodrag node buttons |
| `d86fcb8` | Graph | branch-side handles keep left-branch edges outside parent cards |
| `1a0e893` | Agent UX | class-matrix spawn controls, ControlBar/Swarm/AgentHierarchy SPAWN, root-control chat intents, confirmed A2A/agent messages |
| `ca20730` | UI lifecycle | generation-safe StrictMode lifecycle, replay-spawn guard, named errors |
| `a740960` | UI errors | runtime/message fetch failures carry code + message + how_to_fix |
| `f35a60a` | UI schema | malformed projections become `E_SCHEMA_INVALID` |
| `0b0702f` | Provider | RPM governor, external unpriced cost, transport-failure separation, cache split, 11 tests |
| `5b085ed` | Control plane | in-tx remote revalidation, approval sweep, audit rows, boot fails on unreconciled control plane, daemon degraded |
| `6fb4bcd` | Memory/recovery | fail-closed banks, scoped bank reads, validated writes, multi-run recovery repair |
| `b0b4b9d` | Core | real per-run provider call cap, hierarchy checks, A2A deadline/cap/context, bus ordering, snapshot lock, provider compatibility |
| `4cf35f6` | Core/UI | transport-failure classification, root remote device, final hierarchy and chat fit |
| `0ecb419` | Core | memory bank HTTP reads are run-scoped; terminal run transitions cannot be resurrected |
| `ad01fdf` | Provider settings | saved non-secret provider profile API + Settings provider editor; external profile applies on restart |
| `fef47c7` | Core | replay-consistent `current_seq` and a proven real provider-call cap test |
| `8c84193` | Core | a failed serialization no longer consumes a replay sequence or leaves a permanent gap |
| `aaf2c62` | Provider settings | profile GET exposes only defined fields; unknown file content cannot leak |
| `4d15389` | Daemon | real DB-health job executes and reports measured `ok`/`failed`; root-spawned watchdog uses it |
| `6a1a1fa` | Snapshot | terminal runs retain the newest 16 revisions; old files/index rows are pruned, latest recovery point preserved |
| `ecee300` | Engine hygiene | terminal runs release metrics, delivered-A2A ids and queued spawn ids |
| `8d99f79` | Provider settings | profile replacement keeps a backup, restores on failure, removes temp/backup on success |
| `40ba553` | Engine hygiene | task-context store is bounded to 1024 entries with oldest-first eviction |
| `c3ec90f` | Remote | HyperAgent-spawned relay gets a switch device + narrow grant instead of an inert record |
| `bb0b133` | Provider | complete saved `api_type` (`deepseek`/`tokenlabs`) and per-type key env (`DEEPSEEK_API_KEY`/`HIVEROUTER_API_KEY`); launcher/start script select the key env; UI selector |
| `717944c` | A2A | schema v23 adds persisted attempts/last-attempt plus `a2a_audit`; parent attempts and answers are audited atomically |

Additional Lead changes before `4cf35f6` include the persistent provider profile
(`tools/set-provider-profile.ps1`), external endpoint/wire model config, and
the runtime `billing`/`endpoint`/`parallel_dispatch` fields.

## 4. Verification

```text
Backend:
  cargo test --workspace --locked
  EXIT=0  SUITES=49  PASSED=612  FAILED=0  IGNORED=2

Web:
  npm.cmd run typecheck  -> exit 0
  npm.cmd test           -> 29 files, 225 tests passed, exit 0
  npm.cmd run build      -> exit 0 (531 modules; pre-existing >500 kB chunk warning)

Release:
  cargo build --release --locked -> exit 0
  target/release/hiverouter.exe SHA-256 ACA64B32F896EB466A448BC27D92A03601EA99FADC751AD824BCFA1B1E3DE4D7
  size 36,281,344 bytes

Smoke (no provider key, deliberately degraded):
  GET /health -> status "degraded", schema 22
  GET /api/runtime -> caps.max_calls 64, parallel_dispatch 32,
                      provider.key_present false,
                      provider.endpoint "deepseek", billing "deepseek_usd"
  UI screenshot: docs/evidence/live/ui-final2.png
  Portable package: dist/HiveMind/{HiveMind.exe,hiverouter.exe,README.txt}
  Portable server smoke: health "degraded", schema 22, port closed after stop
  Server stopped; port 8099 closed.
```

The no-key smoke is expected to be `degraded`; no run was submitted and no
provider was called.

## 5. Provider option (AMD Token Lab / DeepSeek)

The provider is a saved, non-secret profile plus a separately stored key:

- Settings → Provider → **Save provider profile** writes
  `%LOCALAPPDATA%\HiveMind\provider.json` through
  `POST /api/provider/profile` and reports `restart_required:true`.
- The API key is never in that file. It is DPAPI-protected by
  `tools/set-provider-key.ps1` (the desktop launcher injects it), or supplied
  through the process environment. Direct `hiverouter.exe` launches need the
  environment or the launcher.
- `GET /api/provider/profile` returns the saved profile plus the active
  endpoint/model, without secrets.

```powershell
# 1. Save the endpoint/model profile (no secret)
powershell -ExecutionPolicy Bypass -File tools\set-provider-profile.ps1 `
  -Name tokenlab -BaseUrl https://api.tokenlab.amd.com/v1 `
  -WireModel DeepSeek-V4.1-Flash

# 2. Save the key with the existing DPAPI script (never in the repo)
powershell -ExecutionPolicy Bypass -File tools\set-provider-key.ps1

# 3. Restart the server/desktop app (profile is read at boot)
```

Environment variables still override the saved profile:
`HIVEROUTER_API_TYPE`, `HIVEROUTER_BASE_URL`, `HIVEROUTER_WIRE_MODEL`,
`HIVEROUTER_MODEL`. The key environment is selected by API type:
`DEEPSEEK_API_KEY` for `deepseek`, `HIVEROUTER_API_KEY` for `tokenlabs`.
The launcher and `start-hivemind.ps1` read the saved profile and inject only
the selected key environment. An external endpoint requires a wire model and
an explicit base URL.

A2A rows now persist `attempts`/`last_attempt_at`, and every parent-answer
attempt plus operator answer appends an `a2a_audit` row atomically (schema v23).

## 6. Remaining / unverified

- **Key hygiene:** the AMD Token Lab key was pasted into chat during this
  session and must be treated as leaked. Rotate/revoke it before the next live
  test. The smoke above used no key.
- **Live end-to-end run after all fixes:** not run against a fresh key. The
  last provider call was before the final repair commits.
- **Daemon script runtime:** daemons execute the real DB-health job
  (`{"check":"db"}`), but arbitrary operator scripts are still not a supported
  runtime. Unknown jobs degrade by name; no fake `alive` is reported.
- **A2A attempt counter:** persisted attempts and atomic audit rows now exist
  (schema v23); a hard attempt cap is not yet enforced.
- **Session token exposure:** `/api/session` still returns the token to a
  loopback peer with no Origin; read routes remain un-tokenized. This is the
  local same-user threat model, not a hardened multi-user boundary.
- **Remote transports:** in-process fixture only; network remotes are not
  implemented. Audit actor uses `system`/`supervisor` for sweeps.
- **Browser automation:** CDP screenshots verify the connected workspace,
  Settings → Provider tab, API-type selector, and key-missing state. No
  automated drag/spawn interaction suite yet.
- **Scale:** accepted 40-node plan cap, 64 provider-call cap, 32-way dispatch
  bound. No 50–150/300–600 claim.
- **No mock/stub product code added.** Test doubles remain confined to tests;
  stub sweep on changed files returned only legitimate test doubles/docs.

## 7. Evidence files

- `docs/evidence/live/ui-api-type-connected.png` — connected workspace with no key
- `docs/evidence/live/ui-provider-settings.png` — Settings → Provider, API type and key status
- `docs/evidence/live/ui-api-type.png` — initial no-key workspace
- `tools/qa/provider-test.log` — provider subagent raw test log (untracked)
- Portable package: `dist/HiveMind/{HiveMind.exe,hiverouter.exe,README.txt}` + `dist/HiveMind.zip`
- Package smoke: `/health` `degraded`, schema **23**, caps 64/32, key absent, port closed after stop

## 8. Security note

The UI/API test credentials are not present in source, committed files, or
this report. The pasted Token Lab key remains compromised and must be rotated
before any further live run.
