# HiveMind HMARS Bug-Fix Report

Date: 2026-11-09  
Repo: `C:\Users\Riley\Desktop\hiverouter-v1`  
GitHub: `https://github.com/RodneyDanger1/HiveMind`  
Base before this repair wave: `b039037`  
Final repair HEAD at report write: `f988ef2`  
Release binary: `target/release/hiverouter.exe` SHA-256 `DB0183EAF279BC4436F6634BF9AB3D2665012468C2F5B1136AA678D9AF16926F`  
Portable package: `dist/HiveMind.zip` SHA-256 `664A78FA1CFD4B96320A66439FC3EC5F2F2A9128223F553BC183AA3468481C71`  
Packaged server: `dist/HiveMind/hiverouter.exe` SHA-256 `DB0183EAF279BC4436F6634BF9AB3D2665012468C2F5B1136AA678D9AF16926F`

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
| `cc90b4c` | A2A | duplicate open questions are refused before a second parent call; persisted attempts are capped at 2 |
| `ea49218` | Daemon | allowlisted `.exe`/`.ps1` jobs run outside the DB lock with bounded args/output and timeout tree-kill |
| `68ff3c4` | Launcher | boot-timeout path stops the child process instead of leaving a server running |
| `fc88323` | Security | launcher/start script create a per-launch token; `/api/session` refuses a request without a matching `x-hivemind-launch` header |
| `bc5ab0d` | Security | all `/api/*` reads require the launcher token when configured; UI wraps same-origin API fetch to send it |
| `3a43ceb` | Runtime | planner/control usage checkpointed; invalid plans get one counted repair call; root checkpoint cannot add a branch when the planner already supplied one |
| `113586c` | UI | 10–20% Unslop polish: all off-grid spacing and header weight fixed; audit before/after in `audit/` |
| `f988ef2` | UI | HoneyGlass startup veil (glassmorphism pane, liquid honeycomb, neumorphic progress, finite motion) + design-system/roadmap docs |

Additional Lead changes before `4cf35f6` include the persistent provider profile
(`tools/set-provider-profile.ps1`), external endpoint/wire model config, and
the runtime `billing`/`endpoint`/`parallel_dispatch` fields.

## 4. Verification

```text
Backend:
  cargo test --workspace --locked
  EXIT=0  SUITES=49  PASSED=617  FAILED=0  IGNORED=2

Web:
  npm.cmd run typecheck  -> exit 0
  npm.cmd test           -> 29 files, 225 tests passed, exit 0
  npm.cmd run build      -> exit 0 (531 modules; pre-existing >500 kB chunk warning)

Release:
  cargo build --release --locked -> exit 0
  target/release/hiverouter.exe SHA-256 E1FB02E407D16CD9EEE0929F520EB68AA664A2FB33FEAEFEDE6AB4F8AE4127AC
  size 36,315,136 bytes

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

## 6. Live Token Lab evidence (2026-11-10)

Two real runs against `https://api.tokenlab.amd.com/v1` with
`DeepSeek-V4.1-Flash`; no DeepSeek account was used.

| Run | Result | Tasks | Usage | Wall |
|---|---|---|---|---|
| Small prompt | `completed`, assistant `hello hive` | 2 (HyperAgent + SuperAgent) | 1,698 prompt / 3 completion / 1,701 total | ~26 s |
| Exact two-branch hierarchy | `completed` | 7 (HyperAgent, 2 SuperAgents, 2 Agents, 2 Subagents) | 14,407 prompt / 1,144 completion / 15,551 total | ~106 s |
| Earlier two-branch attempt | `completed` before the root-cap fix | 10 (extra depth-1 `super` leads) | 19,791 / 883 / 20,674 | ~290 s |
| Hard-kill accounting | process killed after checkpoint; restart `interrupted` | 7 recovered rows | 449 tokens retained in `run_usage` | n/a |

The root-cap fix is what makes the exact-two run produce exactly 7 tasks: the
checkpoint refuses `spawn_superagent` when the planner already supplied branch
leads. The earlier 10-task run is the reproduction of that over-spawn.

Raw summaries: `docs/evidence/live/live-tokenlab-simple.json`,
`docs/evidence/live/live-tokenlab-hierarchy.json`.
Billing remains `external_unverified`; no USD figure is claimed for these runs.

## 7. UI polish (Unslop, 10–20% cap)

`audit/UNSLOP-REPORT.md` records the five-line direction, nine-pass scope,
before/after audit, and accepted findings. Deterministic audit:
**71 → 47 errors / 3 → 5 warnings**, with **U006 spacing 23 → 0** and
**U003 weight 1 → 0**. Remaining U005 is the intentional FigCraft motion
language; U001/U011 are false positives from Tailwind `!` utilities and the
domain action `unlock`; the two new U010 warnings are the splash kicker and
loading label, deliberately tracked mono. Baseline/final renders are
`audit/baseline.light.png` and `audit/final.light.png`; connected product proof
is `docs/evidence/live/ui-polished-final.png`.

The startup veil is documented in
`docs/ui/HIVEMIND-HONEYGLASS-DESIGN-SYSTEM.md` and
`docs/UI-UX-ROADMAP.md`. It shows the live connection state, a bounded loading
window, a FigCraft wordmark with honey drip, and the bottom-right glass credit
"Designed by Riley Christensen". The splash screenshot is
`docs/evidence/live/ui-splash.png`.

## 8. Remaining / unverified

- **Key hygiene:** the operator has confirmed the Token Lab key is **not
  compromised**. No key material is present in source, committed files, this
  report, or the saved profile. Never print or commit a provider key.
- **Live end-to-end after the final repair:** the exact-two run used the final
  root gate and measured usage; the planner repair path was added after an
  earlier `E_SCHEMA_INVALID` attempt and is covered by unit tests, not by a
  fresh live malformed-plan run.
- **Planner repair:** covered by unit tests; not yet driven by a fresh live
  malformed-plan response.
- **Daemon script runtime:** daemons execute the real DB-health job and an
  allowlisted script job (`{"kind":"script","program":"name.ps1","args":[],"timeout_secs":N}`)
  rooted under the workspace. Only `.exe`/`.ps1`, plain file names, <=16 bounded
  args, 1..60s timeout, 8 KiB output caps, and timeout tree-kill are supported;
  no arbitrary shell execution.
- **A2A attempt counter:** persisted attempts and atomic audit rows exist
  (schema v23); duplicate open questions are refused and attempts are capped at
  2. The operator UI shows the persisted count.
- **Session token exposure:** the session bootstrap and every `/api/*` read
  require the launcher-issued `x-hivemind-launch` token when
  `HIVEROUTER_LAUNCH_TOKEN` is set; without it the route is 403. `/health` and
  static UI assets remain tokenless by design.
- **Remote transports:** in-process fixture only; network remotes are not
  implemented. Audit actor uses `system`/`supervisor` for sweeps.
- **Browser automation:** `tools/qa/ui-e2e.mjs` now drives Edge over CDP,
  opens Settings → Provider, saves a TokenLab profile and reads the persisted
  JSON. Drag/spawn interaction automation is still not covered.
- **Scale:** accepted 40-node plan cap, 64 provider-call cap, 32-way dispatch
  bound. No 50–150/300–600 claim.
- **No mock/stub product code added.** Test doubles remain confined to tests;
  stub sweep on changed files returned only legitimate test doubles/docs.

## 7. Evidence files

- `docs/evidence/live/ui-api-type-connected.png` — connected workspace with no key
- `docs/evidence/live/ui-provider-settings.png` — Settings → Provider, API type and key status
- `docs/evidence/live/ui-api-type.png` — initial no-key workspace
- `tools/qa/provider-test.log` — provider subagent raw test log (untracked)
- `docs/evidence/live/ui-e2e-provider-save.png` — CDP-driven Settings → Provider save proving `api_type=tokenlabs`, base URL and wire model persisted
- `tools/qa/ui-e2e.mjs` — repeatable local UI E2E (no provider key, no model call)
- Portable package: `dist/HiveMind/{HiveMind.exe,hiverouter.exe,README.txt}` + `dist/HiveMind.zip`
- Package smoke: `/health` `degraded`, schema **23**, caps 64/32, key absent, port closed after stop

## 8. Security note

The UI/API test credentials are not present in source, committed files, or
this report. The operator has confirmed the Token Lab key is not compromised;
no key material is stored in the repository or the saved non-secret profile.

## 10. Live and UI evidence files

- `docs/evidence/live/live-tokenlab-simple.json` - completed 2-task TokenLab run
- `docs/evidence/live/live-tokenlab-hierarchy.json` - completed exact-two 7-task TokenLab run
- `docs/evidence/live/ui-polished-final.png` - connected UI after the 10-20% polish
- `audit/UNSLOP-REPORT.md`, `audit/baseline.light.png`, `audit/final.light.png`
- `docs/evidence/live/ui-e2e-provider-save.png` - CDP Settings -> Provider save
- `tools/qa/ui-e2e.mjs` - repeatable local UI E2E (no provider key)
## 11. HoneyGlass documentation

- `docs/ui/HIVEMIND-HONEYGLASS-DESIGN-SYSTEM.md` - material rules, splash lifecycle, pseudocode/real-code analysis, roadmap section.
- `docs/UI-UX-ROADMAP.md` - dedicated UI/UX roadmap with the 10-20% scope boundary.
- `docs/evidence/live/ui-splash.png` - 1440x900 HoneyGlass startup veil.
- `docs/evidence/live/ui-splash-375.png` - 375x812 CDP-emulated splash proof.
- `docs/evidence/live/live-tokenlab-kill-usage.json` - checkpoint survives hard kill and restart.
- `docs/evidence/live/ui-polished-final.png` - connected workspace after polish.