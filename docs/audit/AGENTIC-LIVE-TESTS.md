# Agentic Live Tests — HMARS Behavior on Token Lab

Date: 2026-11-10
Provider: Token Lab `https://api.tokenlab.amd.com/v1`, wire model `DeepSeek-V4.1-Flash`
Billing: `external_unverified` (no DeepSeek USD claim)

These are observed runs, not simulated transcripts. Each test uses a fresh
SQLite data directory. The provider key stayed in the child environment and is
not present in this file.

## Test A — exact-two hierarchy and root branch governance

**Prompt**

> Use exactly two SuperAgents. Each SuperAgent must spawn exactly one Agent.
> Each Agent must spawn exactly one temporary Subagent. Do not add extra branch
> leads. Do not inspect the repository. Each branch returns one short sentence
> about a launch checklist; the HyperAgent synthesizes both sentences.

**Observed**

- `completed`; 7 tasks total.
- Root `planner` class `hyper` depth 0.
- Two `superagent` class `super` depth 1.
- Two `agent` class `agent` depth 2.
- Two `subagent` class `subagent` depth 3.
- Usage 14,407 prompt / 1,144 completion / 15,551 total.
- ~106 s wall.

**Pseudocode of what happened**

```text
planner(prompt) -> 2 super leads, each with 1 child agent + 1 child subagent
root_checkpoint -> requested spawn_superagent
if plan already has depth-1 leads:
    root request refused with no new task row
dispatch children in depth order
root_final -> synthesize 2 branch sentences
status = completed
```

**What was right**

- Class matrix held at every depth.
- Root did not add extra branch leads after the fix.
- Final answer used both branches.

**What can still fail**

- The model can still ignore exact counts inside the planner response; the root
  gate only protects the root-control turn.
- No assertion yet that two independent runs produce the same count.

## Test B — A2A ask_parent

**Prompt**

> Create exactly one SuperAgent and one Agent under it. The Agent must call
> ask_parent exactly once to ask its SuperAgent which launch-checklist item to
> verify, then report the parent's answer in one sentence. Do not inspect the
> repository.

**Observed**

- `completed`; 3 tasks: HyperAgent → SuperAgent → Agent.
- Agent used `ask_parent` once.
- A2A row: `status=answered`, `attempts=1`,
  `answer_actor=parent:<superagent>`.
- Parent answer: “Pre-launch verification item: confirm the release’s
  rollback/abort plan is defined and documented before launch.”
- Usage 6,169 / 755 / 6,924.

**Pseudocode**

```text
agent.execute:
  ask_parent(text)
  record open message + deadline
  record attempt(parent, parent_answer)  -> attempts=1, audit row
  parent.answer_escalation -> JSON answer
  CAS answer_agent_message -> answered + audit row
  return answer to agent
```

**What was right**

- One durable message, one attempt, one parent answer.
- Attempt and answer audit rows exist; no duplicate answer.
- Parent context path worked live.

**What was wrong or weak**

- One SuperAgent message said it had no children when it emitted a result. The
  dispatch readiness code does require every direct child to be complete before
  a parent is collected, so this is not yet proven to be a scheduler violation;
  it may be a task with a different branch or a wording artifact. A targeted
  test must assert parent-after-child ordering explicitly rather than infer it
  from prose.

## Test C — root-created detached remote and daemon

**Prompt**

> Create one SuperAgent with one Agent. In your root control turn, create one
> detached remote light named Porch and one watchdog daemon named Pulse. Answer
> with one sentence.

**Observed**

- `completed`; 3 tasks; 162 s.
- Remote: `Porch`, adapter `light`, `in_process`, `enrolled`, **device_count 1**,
  **grant_count 1**, created by `hyper:<root>`, audit action `remote_created`.
- Daemon: `Pulse`, `watchdog`, job `{"check":"db"}`, `enrolled`.
- Usage 5,305 / 250 / 5,555.

**Pseudocode**

```text
root_checkpoint:
  spawn_remote_agent(display_name=Porch, kind=light)
    -> remote row + device + narrow grant + audit
  spawn_daemon(display_name=Pulse, kind=watchdog)
    -> daemon row job={"check":"db"} state=enrolled
dispatch planned children
root_final -> one sentence
```

**What was right**

- Detached records are tree-free: no task rows, no graph children.
- Remote has a real device/grant, not just a label.
- Daemon is enrolled with a real DB-check job.
- Telemetry preserved the hyper actor id.

**Boundary**

- Remote transport remains in-process fixture. It is not a network satellite.
- Daemon starts enrolled, not running; that is the honest default.

## Test D — local rate governor refuses before provider spend

An earlier identical A2A request returned `E_LLM_RATE_LIMIT` in 0 s with 0 tasks
while the Token Lab 20 RPM window was exhausted. No provider request was made by
the app. After one window wait, the same prompt completed. This validates the
local governor, not a provider guarantee.

## Test E — usage accounting survives hard kill

After a measured reply, the process was hard-killed. Restart recovery reported
`interrupted` with 7 tasks, and the measured usage remained:

```text
checkpoint: 319 prompt / 130 completion / 449 total
after restart: 319 prompt / 130 completion / 449 total
```

This is the required behavior for API-burn honesty.

## Cross-test conclusions

| Claim | Verdict |
|---|---|
| HyperAgent governs the run | Evidence supports |
| SuperAgent is a branch lead | Evidence supports |
| Agent can ask its parent | Evidence supports |
| Subagent is depth-3 terminal | Evidence supports in exact-two run |
| Remote light is detached and actuable | Evidence supports fixture device/grant |
| Daemon is detached and has a real job | Evidence supports DB-check job |
| Root cannot add branches when planner supplied them | Evidence supports |
| A2A attempts are durable/audited | Evidence supports |
| No hidden provider spend on interrupted run | Evidence supports after checkpoint |
| Network remote transport | NOT IMPLEMENTED |
| Lower-level request → HyperAgent approval | IMPLEMENTED as a deterministic admission gate; not an asynchronous LLM decision |

## Next tests

1. Parent scheduling: ensure a SuperAgent is not finalized while an Agent child
   is still running and depending on it.
2. Lower-level remote request: Agent calls `request_detached`; test the
   deterministic HyperAgent admission allow/duplicate paths and audit (unit
   tests now pass; live provider test remains).
3. Memory Venn: prove no sibling/foreign-run bleed under a real branch tree.
4. Repeat exact-two three times; assert stable task count and no extra branch.
5. Full 40-node offline / bounded live probe with measured dispatch.

## Evidence files

- `docs/evidence/live/live-tokenlab-hierarchy.json`
- `docs/evidence/live/live-tokenlab-a2a.json`
- `docs/evidence/live/live-tokenlab-remote-daemon.json`
- `docs/evidence/live/live-tokenlab-rate-limit.json`
- `docs/evidence/live/live-tokenlab-kill-usage.json`

## Test F — repeat stability (2026-11-10)

Two additional exact-two runs were executed against the rebuilt release binary:

| Attempt | Result | Tasks | Usage |
|---|---|---|---|
| 1 | completed | 7 (1 Hyper, 2 Super, 2 Agent, 2 Subagent) | 13,751 |
| 2 | failed `E_PROVIDER_TIMEOUT` before the first task row | 0 | unknown |

The completed attempt reproduced the exact topology. The timeout is an
upstream/provider failure at planning, not a topology violation; it is counted
as a failure, and two of three attempts are not yet enough for a stability
claim. Evidence: `docs/evidence/live/live-tokenlab-exact-two-stability.json`.
## Test G — lower-level detached request, live

**Prompt:** one SuperAgent with one Agent; the Agent must call
`request_detached` once for a detached remote named `Lantern`.

**Observed:** completed; 3 tasks (HyperAgent, SuperAgent, Agent); usage
5,818 / 261 / 6,079. Tool turn `request_detached` outcome `ok`. Remote `Lantern`
was created with `owner_id=<run>`, `created_by=agent:<agent task>`, adapter
`light`, one device and one grant. Audit rows: `remote_created`,
`device_created`, `grant_created`, and `hyper_admission` with
`decision:allow`, all attributed to `system` / `agent:<task>`.

This proves the **deterministic HyperAgent admission path** end to end. It does
not prove an asynchronous LLM HyperAgent decision, and no network satellite was
created. Evidence: `docs/evidence/live/live-tokenlab-detached-request.json`.
## Test H — request_id idempotency, live

**Prompt:** one SuperAgent + one Agent; Agent calls `request_detached` once with
`request_id="live-once-1"` and does not call ask_parent or spawn a Subagent.

**Observed:** completed; 3 tasks (hyper, superagent, worker); usage 7,649 / 411 /
8,060. Tool turn `request_detached` outcome `ok`. Remote `Lantern` created with
`request_id=live-once-1` in `remote_created`, one device + one grant,
`hyper_admission` allow, and `created_by=agent:<task>`. Root control recorded one
`spawn_superagent` refusal (the planner already supplied a branch).

This proves the idempotency key is carried into the durable audit row in a live
run. It does **not** prove the retry path live; that is covered by the offline
idempotency test. Evidence: `docs/evidence/live/live-tokenlab-request-id.json`.
### Test H — failed first attempt (honest)

An earlier prompt asking for **two** idempotent `request_detached` calls did not
exercise the path. The model chose `ask_parent` instead; that call timed out
(`E_TOOL_TIMEOUT`), the Agent/Subagent finished, and the root remained running
when the test was stopped. No remote was created. This run is recorded as a
**failure/behavioral deviation**, not as an idempotency result:
`docs/evidence/live/live-tokenlab-idempotent-attempt.json`.

The successful live Test H used a simpler prompt that demanded one
`request_detached` call and forbade `ask_parent`.
## Test I — final build, request_id persisted live

**Prompt:** one SuperAgent + one Agent; Agent calls `request_detached` once with
`request_id="live-final-1"` and reason `"status light for run state"`.

**Observed:** completed; 3 tasks; usage 7,908 / 507 / 8,415. Tool
`request_detached` outcome `ok`. Remote `Lantern` with one device, one grant, and
`request_id=live-final-1` in `remote_created`. This was run against the final
fingerprint-binding build.

Evidence: `docs/evidence/live/live-tokenlab-request-id-final.json`.
## Test J — final-build exact-two regression

Final build (refusal auditing + fingerprint idempotency): **completed**, 7 tasks
(1 HyperAgent, 2 SuperAgents, 2 Agents, 2 Subagents), 17,486 tokens. No extra
branch lead. Evidence: `docs/evidence/live/live-tokenlab-final-exact-two.json`.