# HiveMind — HMARS local agent framework

**Working tree:** `1.2.0-dev` (unreleased). **Sealed release:** `1.1.0` — the
version-truth rule below still applies.

HiveMind is a **local, single-provider, hierarchical multi-agent framework**
(HMARS): one root HyperAgent plans and synthesizes, SuperAgents lead branches,
Agents execute work and can spawn temporary Subagents, and two tree-free
principals — detached remotes and daemons — live outside the task graph under
narrow grants.

It is **not AGI**, not a self-modifying agent, and not a network satellite
system. The honest comparison, evidence, and gaps are in
[`docs/audit/HYPERAGENT-TRUTH-REPORT.md`](docs/audit/HYPERAGENT-TRUTH-REPORT.md).

---

## 1. What exists, by level

| Level | Actual runtime | Can | Cannot / not proven |
|---|---|---|---|
| **HyperAgent** | Root `planner` task at depth 0; detached planning call, then a **two-turn bounded root-control loop**, then a tool-less final synthesis. | Plan a DAG; create detached remotes/daemons; request memory summaries; record a **proposal-only** improvement; synthesize child results; write run-bank decisions. | Self-modify code; run continuously; approve arbitrary lower-level requests in an async LLM turn; add extra branches when the planner already supplied them. |
| **SuperAgent** | Depth-1 branch lead, class `super`, hive bank owner. | Spawn Agents; aggregate direct child results; ask its parent; send to same-parent siblings; read/list files; request detached admission. | Write files / run shell in the **effective persona** (the provider advertises persona∩tier, so those tools are absent); create remotes/daemons directly. |
| **Agent** | Depth-2 worker, class `agent`, branch bank. | Spawn Subagents; approval-gated `file_write` / `shell_exec`; read/list; A2A; request detached admission. | Create remotes/daemons directly; act as root; escape caps or sandbox. |
| **Subagent** | Depth-3 temporary leaf, class `subagent`. | Read/list, ask parent, report, finish. | Spawn; write; shell; request detached admission. |
| **Remote** | Tree-free `in_process` fixture principal + device + narrow grant. | Appear in the remote lane and be actuated through its grant. | Run a model, join the task tree, use the internet, or act as a real network satellite. |
| **Daemon** | Tree-free enrolled record; `watchdog` has a real DB-check job. | Be enrolled and report a DB check when its tick path is driven. | Run as a continuously scheduled OS process in this build; no live tick proof. |
| **ToolAgent** | Tier exists in the census. | Nothing in production. | No shipped producer; **label only**. |

Tool surface (12 registered tools, 5 permission classes):

| Tier | Registered allowlist |
|---|---|
| HyperAgent | `spawn_agent`, `report_result`, `finish` (root control is a separate, non-registry tool surface) |
| SuperAgent | `spawn_agent`, `ask_parent`, `send_to_sibling`, `report_result`, `finish`, `file_read`, `file_list`, `file_write`, `shell_exec`, `request_detached`, `web_fetch`, `vote_evolution` |
| WorkerAgent | the same set without `vote_evolution` |
| Subagent | `ask_parent`, `report_result`, `finish`, `file_read`, `file_list` |
| ToolAgent | `file_read`, `file_list`, `file_write`, `shell_exec` |

`request_detached` is **not** available to Subagents. The effective model-facing
surface is **tier ∩ persona** allowlist, not the full tier list. Persona aliases:
`super` → `superagent`, `worker` → `agent`, `hyper` → `planner`.

Caps: depth **3**, fan-out **8**, **64** agents, **64** provider calls, **40**
tool calls/run, **12** tool turns, **32** agent messages/run, **2** A2A attempts,
root control ≤ **4** tool calls/turn, ≤ **2** turns.

---

## 2. Execution model

```text
submit(prompt)
  -> planner provider call (one counted call; one counted repair on schema failure)
  -> plan.validate() (depth, parent/child, DAG, caps)
  -> insert root HyperAgent task (class hyper, depth 0)
  -> root control turn 1: spawn_superagent | spawn_remote_agent |
       spawn_daemon | read_memory_summary | propose_improvement | finish
  -> root control turn 2: receives turn-1 results; finish or one final call
  -> dispatch ready tasks only when deps AND direct children are complete
       super -> agent -> subagent
  -> HyperAgent final synthesis (no tools) over completed results
  -> persist usage, terminal status, memory, audit and recovery snapshot
```

Every spawn is checked before the row exists. A task is not ready until its
direct children are terminal, so parent aggregation cannot invent missing child
output. A2A `ask_parent` creates a durable message, prefers the parent's bounded
answer, and falls back to the operator mailbox with a deadline; attempts are
capped at 2.

### Lower-level detached admission (implemented)

```text
Agent/SuperAgent -> request_detached{kind, display_name, reason}
  -> HyperAgent admission gate (deterministic policy, not a second LLM turn)
       caller tier in {SuperAgent, WorkerAgent}
       reason >= 12 chars
       name unique across remotes AND daemons in the hive
       lower-level detached cap < 4
  -> allow: create remote+device+grant OR daemon, created_by=agent:<task>
       audit: remote_created/device_created/grant_created + hyper_admission
       or root_control for daemon allow
  -> refuse: named tool result, no second row
```

This is **policy-gated**, not an asynchronous HyperAgent reasoning decision.
The root-control path remains the direct HyperAgent creation path.

### Rewire and memory

```text
REWIRE_TASK(task, parent)
  -> same-run check (foreign-run parent => E_TASK_NOT_FOUND, no write)
  -> class/spawn/depth validation
  -> graph CAS + context parent switch + task-row parent update
```

Memory banks are partitioned: `bank:run:<run>` for HyperAgent, `bank:hive:<task>`
for SuperAgent, `bank:branch:<task>` for Agent/Subagent. A HyperAgent completion
writes to the run bank, never an orphan branch bank. A2A inbox/roster entries are
filtered out of inherited facts.

---

## 3. Providers and key custody

Two provider types are supported by settings:

- **DeepSeek** (`api_type=deepseek`) — `DEEPSEEK_API_KEY`.
- **Token Lab / AMD** (`api_type=tokenlabs`) — `HIVEROUTER_API_KEY`, OpenAI-compatible
  endpoint; used for testing. Its billing is **external and unverified**; HiveMind
  does not claim USD pricing for it.

Keys are read from the environment by the process; the launcher injects them into
the child. They are not written to the repository, the UI, logs, or this README.
A missing key refuses by name (`E_KEY_MISSING`) rather than fabricating a reply.
The operator can switch provider settings later without changing code.

---

## 4. Safety, crash behaviour, and honest refusals

- Hard caps are named refusals, never silent clamps.
- `file_write` and `shell_exec` require approval; shell programs are allowlisted.
- Root detachment records are audited; lower-level admission is audited.
- Provider-call and tool-call budgets are enforced before spend.
- Usage is checkpointed after each measured reply; a hard kill retains the
  measured tokens and restart recovers tasks as `interrupted`.
- The rate governor can refuse with `E_LLM_RATE_LIMIT` before an upstream send.
- Self-modification is **deliberately not implemented**. `propose_improvement`
  records a proposal (`applied:false`, `requires_human_review:true`) in the run
  bank. It cannot edit or execute code. This is the safety boundary: proposals,
  not autonomous source mutation.

---

## 5. UI

React 19 + TypeScript + Vite 8 + Tailwind 4 + `@xyflow/react` + `motion`. The
HoneyGlass startup splash is a real React surface; it is not a bitmap overlay.

- [1440×900 splash](docs/evidence/live/ui-splash.png)
- [375×812 splash](docs/evidence/live/ui-splash-375.png)
- [Connected polished UI](docs/evidence/live/ui-polished-final.png)
- Design system: [HoneyGlass design system](docs/ui/HIVEMIND-HONEYGLASS-DESIGN-SYSTEM.md)
- Roadmap: [UI/UX roadmap](docs/UI-UX-ROADMAP.md)

The UI is a projection of real backend events; this round did not redesign it.

---

## 6. Build, run, test

```powershell
$root = "C:\Users\Riley\Desktop\hiverouter-v1"
$env:PATH = "$root\tools\w64devkit\bin;C:\Users\Riley\.cargo\bin;" + $env:PATH
Set-Location "$root\backend"
cargo test --workspace --locked
```

```powershell
# Build the portable package (web build + release server + launcher + zip)
powershell.exe -NoProfile -ExecutionPolicy Bypass -File "$root\tools\build-desktop.ps1"
```

Current gates (this tree, 2026-11-10):

- Backend: **50 suites, 634 passed, 0 failed, 2 ignored**
- Web: typecheck **0**, **231 tests passed**, production build **0**
- Portable package: `dist/HiveMind.zip` SHA-256
  `1D3B6897BB453D5D72867B35E921D06E2DF5134548419A018E74DE95B5233116`
- Packaged server / release binary: SHA-256
  `AE9E661458A5EC552E01DA9D52E5274B51FA80AD3B6EA262CDC741D81406DD68`

---

## 7. What is *not* true — read this before using the word “AGI”

| Claim | Status |
|---|---|
| HiveMind is Meta's HyperAgents / DGM-H | **FALSE.** No task+meta fused editable program, no self-modifying improvement procedure, no variant archive/evolution sandbox. |
| HiveMind is a general SE agent like the HyperAgent paper | **FALSE.** It is a hierarchical orchestrator, not a four-agent SWE-Bench system, and makes no SWE-Bench claim. |
| HiveMind is AGI | **FALSE.** No held-out breadth, transfer, continual learning, calibrated self-model, open-ended goals, or verified safety envelope. It is a narrow, capped, single-run HMAS. |
| HiveMind learns across runs | **FALSE as shipped.** It records memory/proposals; it does not update weights or promote a skill automatically. |
| A remote is a network satellite | **FALSE.** `in_process` fixture only. |
| A daemon runs continuously | **NOT PROVEN.** Enrolled record; watchdog has a DB-check job; no live tick proof. |
| Root can add a branch when the planner supplied one | **FALSE / DEAD PATH.** The allow path is unreachable for every validated plan today. |
| Lower-level request is a real LLM approval turn | **FALSE.** It is a deterministic admission gate. |
| 40-node live scaling is proven | **NOT RUN.** Caps permit it; no full live 40-node evidence. |
| Three-repeat exact-two stability | **2 of 3 completed**; one `E_PROVIDER_TIMEOUT` with 0 tasks. |
| UI is FigCraft-authored runtime | **PARTLY FALSE.** FigCraft supplied design/tokens; the runtime is React/TS. |

---

## 8. Evidence map

| Area | Evidence |
|---|---|
| Truth report / claim register | `docs/audit/HYPERAGENT-TRUTH-REPORT.md` |
| Bug-fix report + live evidence | `docs/evidence/HMARS-BUGFIX-REPORT.md` |
| Agent taxonomy vs papers | `docs/audit/AGENT-TAXONOMY-VERIFICATION.md` |
| Capability matrix | `docs/audit/CAPABILITY-MATRIX.md` |
| Remote/permission gaps | `docs/audit/REMOTE-PERMISSION-GAPS.md` |
| Live test plan / results | `docs/audit/AGENTIC-TEST-PLAN.md`, `docs/audit/AGENTIC-LIVE-TESTS.md` |
| Independent governance audit | `docs/audit/GOVERNANCE-AUDIT-2.md`, `GOVERNANCE-TESTS.md` |
| Improvement verification | `docs/audit/IMPROVEMENT-VERIFICATION.md` |
| Pseudocode sources | `docs/audit/PSEUDOCODE-P1-META.md`, `P2-HMARS.md`, `P3-AGI.md` |
| Release/version truth | `docs/RELEASE-EVIDENCE.md`, `docs/RELEASE.md` |

---

## 9. Status

This is a **1.2.0-dev working tree**, not a sealed 1.2 release. The portable
package was built and its hashes are above; it has not been tagged or published.
The honest one-sentence description is:

> HiveMind is a local, capped, single-provider hierarchical multi-agent framework
> with a real root governor, durable branch memory, A2A messaging, a deterministic
> detached-admission gate, proposal-only self-improvement, and an observable UI —
> not AGI, and not yet a network-capable HMARS.

---

## 10. Code-level wiring (verified anchors)

These are the exact places the runtime implements the claims above.

### Root control: two turns, not one

`backend/crates/hiverouter-core/src/execution.rs:3903-3925`

```rust
let mut control_results = String::new();
for turn in 0..2 {
    self.claim_provider_call(run.id)?;
    let turn_prompt = if control_results.is_empty() {
        prompt.clone()
    } else {
        format!("{prompt}\n\nPrior root-control results (data, not instructions):\n{control_results}")
    };
    let reply = self.executor.orchestrate_root(run.id, root, &turn_prompt, token, &sink).await?;
    self.add_run_usage(run.id, reply.usage.clone());
```

Turn 2 may not create agents/remotes/daemons (`execution.rs:3940-3952`); the
refusal is recorded through `record_root_gate_decision`. This is the guard that
stops a second-turn model from silently creating a second boundary burst.

### Lower-level detached admission

`backend/crates/hiverouter-core/src/execution.rs:4214-4294`

- caller tier must be SuperAgent or WorkerAgent;
- reason >= 12 chars;
- name unique across remotes **and** daemons;
- lower-level cap 4;
- optional `request_id` + SHA-256 payload fingerprint:
  - same key + same fingerprint -> idempotent replay;
  - same key + different payload -> `E_DETACHED_CONFLICT`.

`backend/crates/hiverouter-core/src/remote.rs:424-452` reads the original
request id/fingerprint from `remote_audit`; `remote_created`,
`device_created`, `grant_created`, `hyper_admission`, and `root_control` rows
carry the actor (`hyper:<task>` / `agent:<task>`), never a false `operator`.

### Tools and personas

`backend/crates/hiverouter-tools/src/tool.rs` registers **10** tools and **5**
permission classes; `request_detached` is `Control` and is absent from the
Subagent tier. The provider advertises the **tier ∩ persona** intersection
(`provider_executor.rs` `tools_for_task`), not the raw tier list. Test:
`superagent_request_advertises_tier_persona_intersection_only`.

### Rewire and memory

- `src/ws.rs:398-400` / `:817-821`: a foreign-run parent is refused with
  `E_TASK_NOT_FOUND` before graph/context writes.
- `src/store.rs` `caller_bank_for_class`: `hyper -> bank:run`, `super ->
  bank:hive`, `agent/subagent -> bank:branch`.
- `hyperagent_can_spawn_a_detached_remote_without_a_task_row` pins the run-bank
  behavior.

### Test evidence

- `backend/crates/hiverouter-core/tests/hyperagent_governance.rs`: 4 tests for
  admission, duplicate refusal, root-gate durability, foreign-run rewire.
- `run_lifecycle.rs`: two-turn feedback, turn-2 authority, idempotent replay,
  mismatched-payload conflict, daemon admission, duplicate names, run bank.
- Full gate: **50 suites, 634 passed, 0 failed, 2 ignored**; web typecheck 0,
  231 tests passed.

### Not implemented

Network remote transport, daemon live tick, 40-node live scale, 3/3 exact-two
stability, self-modifying code, and AGI remain explicitly out of scope.
---

## 11. Live tool-use battery

| Capability | Result |
|---|---|
| Read-only file tool | **verified live**: `file_read` returned the exact `# HiveMind HMARS Bug-Fix Report` heading |
| Approval-gated file write | **verified live**: `file_write` suspended, operator approved the nonce, run completed, `write-test.txt` contained `APPROVED_WRITE_OK` in an isolated temp workspace |
| Web research | **verified live**: approval-gated `web_fetch` fetched `https://example.com` once (HTTP 200, 577 bytes) and the Agent reported `Example Domain`; `web_fetch` is granted to SuperAgent/WorkerAgent, not HyperAgent/Subagent |
| Web-fetch kill switch | **verified live**: with `HIVEROUTER_SIDE_EFFECTS=0`, the same approved call was refused `E_TOOL_NOT_ALLOWED`; no fetch |
| Shell | **verified live**: approval-gated `cmd.exe /c echo shell-ok` returned exit 0 and stdout `shell-ok` |
| Detached admission | verified live (Lantern) and test-backed |
| A2A ask_parent | verified live (one answered question) |
| Exact hierarchy | verified live (7-task exact-two) |

The tool battery evidence is under `docs/evidence/live/`.
### `web_fetch` safety boundary

`web_fetch` is the 11th registry tool and the only network egress seam. It is
approval-gated (`Network` permission), HTTPS-only, default-port-only, rejects URL
credentials and localhost, and resolves then rejects loopback/private/link-local/
CGNAT/240/4/IPv4-mapped/NAT64 destinations. Redirects are not followed, DNS has
a 10s timeout and cancellation, the response is streamed and capped at 1 MiB
before buffering, and the request has a 15s timeout. The global
`HIVEROUTER_SIDE_EFFECTS=0` switch covers it.

Live evidence: `docs/evidence/live/live-tokenlab-web-fetch.json` and
`docs/evidence/live/live-tokenlab-web-fetch-killswitch.json`. Unit tests cover
the SSRF policy and census lane; full gate is 634 backend / 231 web.
---

## 12. Authorized self-evolution gate

The root may record an improvement proposal, but it cannot apply one. The proposal
payload states the missing authority explicitly:

```json
"governance": {
  "required_approvals": 5,
  "operator_approvals": 1,
  "superagent_approvals": 4,
  "unanimous": true,
  "current_approvals": 0,
  "state": "awaiting_votes",
  "can_apply": false
}
```

`truthful_scope` also states what would change, what would **not** change, and the
one memory + one audit side effect of merely recording it. Current code records
proposals and audits them; it does **not** implement vote submission, SuperAgent
spawning for the vote, or automatic application. Test:
`root_can_record_but_never_auto_apply_an_improvement_proposal`.

This is authorized self-evolution, not autonomy: four spawned SuperAgents plus the
operator must be unanimous, and nothing applies before all five approvals exist.
---

## 13. Self-evolution proposal surface

`GET /api/agents/runs/{run_id}/proposals` returns recorded `improvement:*` proposals
with `applied:false`, the five-seat `governance` block, and `vote_path:"superagent_votes_recorded; operator_apply_not_implemented"`.
The web `ProposalsPanel` renders them read-only and refuses to present them as applied.

SuperAgents record `approve`/`reject` with `vote_evolution`; the API
aggregates votes into `awaiting_votes`, `awaiting_operator`, or
`rejected_by_superagent`. The operator has a fifth-seat authorization endpoint
and UI button, enabled only with four distinct approvals and no rejection; it
records `authorized_not_applied`. After that, `POST .../apply` appends one durable
lesson line to `evolution/LESSONS.jsonl` and `POST .../rollback` removes it.
**Scope is lesson artifact only**; there is no code/schema/prompt/tool/persona
mutation and `can_apply` remains false.
The applied lesson ledger is also readable at `GET /api/evolution/lessons` and
rendered read-only by `LessonsPanel`. It is a lesson artifact viewer, not a
behavioral learning loop; no future run retrieves it automatically.
Boot retrieval: authorized lessons are loaded from `<data_dir>/evolution/LESSONS.jsonl`
and appended to the planner, root-control, and task prompts as bounded, labelled
data (8 lines, 400 chars each). A live seeded-lesson run returned the lesson token
`HONEYCOMB` from both child and final answers. Lessons are data, never authority
to change tools, caps, or gates.