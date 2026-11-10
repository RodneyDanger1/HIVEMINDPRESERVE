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
## Test K — approval-gated file_write, live

Prompt required one Agent to call `file_write` for `write-test.txt` with
`APPROVED_WRITE_OK`. The run suspended on a real approval; the operator approved
via the CLI with the single-use nonce; the run completed and the file existed
with exactly `APPROVED_WRITE_OK`.

This proves the full write path: tool request → approval row/nonce → operator
approval → execution → artifact content. It used an isolated temporary
workspace, not the repository. Evidence:
`docs/evidence/live/live-tokenlab-file-write-approval.json`.
## Test L — approval-gated web_fetch, live

Prompt required one Agent to `web_fetch https://example.com` once. The run
suspended on a real approval, the operator approved it, and the run completed.
The tool row is `web_fetch: ok`; the final answer confirmed the fetched body
contains `Example Domain` (HTTP 200, 577 bytes). No shell/file tool was used.

Usage: 8,047 prompt / 554 completion / 8,601 total. Evidence:
`docs/evidence/live/live-tokenlab-web-fetch.json`.
## Test M — web_fetch blocked by the global side-effect kill switch

Started with `HIVEROUTER_SIDE_EFFECTS=0`; the run requested `web_fetch`,
the operator approved, and the tool executed **zero** network effects and was
refused with `E_TOOL_NOT_ALLOWED`. The only tool rows were
`web_fetch: refused` and `report_result: ok`.

Usage: 8,601 / 1,257 / 9,858. Evidence:
`docs/evidence/live/live-tokenlab-web-fetch-killswitch.json`.
## Test N — hardened web_fetch, live

On the hardened build (DNS timeout, streamed 1 MiB cap, kill-switch check),
the approval-gated `web_fetch https://example.com` completed: exactly one
`web_fetch: ok` row, HTTP 200, 577 bytes, and the Agent reported
`Example Domain`. Usage 8,411 / 940 / 9,351.

Evidence: `docs/evidence/live/live-tokenlab-web-fetch.json`; the disabled
negative case is `docs/evidence/live/live-tokenlab-web-fetch-killswitch.json`.
## Test O — approval-gated shell_exec, live

Run `01a1c678-fcbb-7182-9bfa-8f04218520ff` requested `shell_exec` with
`cmd.exe /c echo shell-ok`; the operator approved it, the tool row was
`shell_exec: ok`, and the Agent reported stdout `shell-ok` with exit code 0.
This is the live shell path, not just a unit test. Evidence:
`docs/evidence/live/live-tokenlab-shell-exec.json`.
## Test P — durable grant verification before web_fetch

Final build run `01a1c689-e005-72cf-8f36-eefbedf973bb` completed with exactly
one `web_fetch: ok` row (9,257 tokens). The durable single-use grant is now
verified/consumed before the HTTP request, so an approval cannot be replayed
from an in-memory waiter alone. Evidence:
`docs/evidence/live/live-tokenlab-web-fetch.json`.
## Test Q — proposal dedupe and read-only ledger

Final build run `01a1c6a2-6c25-7333-a970-8ef8d6a61a0c` proposed the same
improvement twice across the two root-control turns. The proposal ledger returned
**count=1**, `applied=false`, `vote_path="not_implemented"`, and
`state="awaiting_votes"`. The duplicate was deduplicated by content, not by a new
UUID. Evidence: `docs/evidence/live/live-tokenlab-proposal-dedupe.json`.
## Test R — vote-aware proposal API

Final vote-tool build run `01a1c6bc-be10-7721-839b-15b8aeb365b2` completed with
one proposal, `applied=false`, zero votes, and
`vote_path="superagent_votes; operator_authorize; lesson_apply; lesson_rollback; code_apply_not_implemented"`.
Evidence: `docs/evidence/live/live-tokenlab-proposal-votes.json`.
## Test S — operator authorization refused without four votes

Final build, run `01a1c6bc-be10-7721-839b-15b8aeb365b2`, proposal
`01a1c6bc-d355-761f-9266-5b98b8e0fc75`: `POST .../authorize` with the
per-boot session token returned **409** because there were zero SuperAgent
approvals. No authorization memory row and no apply occurred. Evidence:
`docs/evidence/live/live-tokenlab-authorize-refusal.json`.
## Test T — voter-identity rejection on final build

Final build `C885…` started against the recorded proposal data; `POST
.../authorize` with the session token and zero votes returned **409**. The
endpoint now also rejects voters that are not completed SuperAgents in this run.
No authorization row and no apply occurred.
## Test U — unauthorized lesson apply refused

Final build, run `01a1c6bc-…`, proposal `01a1c6bc-d355-…`:
`POST .../apply` without five-seat authorization returned **409**; no lesson
artifact and no `evolution_apply` row were written. Apply is scoped to a lesson
artifact only. Evidence: `docs/evidence/live/live-tokenlab-lesson-apply-refusal.json`.
## Test V — read-only authorized lesson surface

Final build served `GET /api/evolution/lessons` with `count=0` and
`artifact="evolution/LESSONS.jsonl"` before any authorized apply. The route is
read-only; it cannot apply or roll back. Evidence:
`docs/evidence/live/live-tokenlab-lessons-empty.json`.
## Test W — cross-run authorized lesson retrieval

Final build started with a seeded authorized lesson in
`<data_dir>/evolution/LESSONS.jsonl`; the prompt asked for the lesson's token.
The run completed and both the child and final answers returned **HONEYCOMB**
(7,936 tokens). This proves the boot loader injects bounded lessons into the
planner, root, and task prompts as labelled data.

Evidence: `docs/evidence/live/live-tokenlab-lesson-retrieval.json`.
## Test X — positive four-SuperAgent authorize → apply → rollback

`backend/crates/hiverouter-core/tests/proposal_authorization.rs` drives the real
HTTP routes against an in-process server and real SQLite rows:

1. insert one root task and four completed `superagent` tasks in one run;
2. record one approval vote per SuperAgent and one proposal;
3. `POST .../authorize` → **200**, `authorized_not_applied`;
4. `POST .../apply` → **200**, `applied:true`, `scope:"lesson_artifact_only"`;
5. read back `evolution/LESSONS.jsonl` and see the proposal id;
6. `POST .../rollback` → **200**, `rolled_back:true`, no proposal id remains.

This proves the positive five-seat authorization plus lesson apply/rollback path
at the route/database level. The four voter tasks are seeded test doubles, not
model-spawned agents.
Tamper branch (same test): after apply, mutate the target lesson line in
`evolution/LESSONS.jsonl`; rollback returns **409** and the tampered bytes remain
unchanged. Restoring the exact applied bytes makes rollback succeed and removes
only the proposal line. A foreign line append no longer blocks per-line rollback.
## Test Y — hot reload, per-line rollback, and rollback projection

The positive route test now also proves: apply hot-reloads the lesson snapshot
without restart; rollback removes the exact line by stored `line_sha256` (not the
whole-file hash); mutating the target line returns **409** and preserves it; and
the proposals projection reports `lesson_applied:false`, `lesson_state:"rolled_back"`
after a successful rollback. Backend gate: 51 suites / 635 passed.
## Test Z — parsed, fenced lesson retrieval

Final build, seeded apply-shaped JSON lesson (`area/problem/proposal/evidence/state`):
the new loader parsed the record, skipped foreign lines, and injected it inside
`<<<AUTHORIZED_LESSONS_DATA … >>>`; run `01a1c740-…` completed and returned
**HONEYCOMB** (3,478 tokens). Evidence:
`docs/evidence/live/live-tokenlab-lesson-retrieval-parsed.json`.
## Test AA — projection and authorize use the same voter validation

Before authorize, GET proposals reports `current_superagent_approvals=4` and
`vote_state="awaiting_operator"` only when the four votes belong to completed
same-run SuperAgent tasks. Invalid voters are no longer counted in the UI
projection, so the button cannot be offered on votes the authorize route would
reject. Backend gate: 51 suites / 636 passed.
## Test AB — authorization binds the proposal hash

Authorization records `proposal_sha256` over `area/problem/proposal/evidence`.
Apply re-computes it from the current proposal and refuses with **409** on any
mismatch, so an authorization cannot be reused for changed proposal text. Run
memory clear now refuses **409** when an `evolution_authorization:` or
`evolution_apply:` row exists, so a run-memory wipe cannot orphan an applied
lesson. Backend gate: 51 suites / 636 passed.
Negative paths now exercised in the same route test: after authorization, the
proposal is mutated → apply returns **409**; restoring the proposal → apply
returns **200**. After rollback, the run is set terminal and
`POST /api/memory/clear` with confirm/reason returns **409** because the
governance rows still exist.
## Test AC — proposals/lessons surface state and bounded fetches

The proposal and lesson panels now use the shared bounded fetch helper with an
abort controller, render `LoadingState`/`ErrorState` (role=status/alert), show
an explicit idle instruction when no run is selected, and render
`can_apply`/`applied` as `unknown`/`yes`/`no` rather than `String(null)`.
Web gate: 31 files / 231 tests passed, typecheck 0, production build 0.
## Test AD — live auto-spawned four SuperAgent voters + five-seat apply

Run `01a1c77e-…` (TokenLab DeepSeek-V4.1-Flash, 30,288 tokens) recorded one
proposal; `propose_improvement` auto-spawned **four** SuperAgent voters with A2A
vote requests; the roster shows four depth-1 `super` tasks; the proposal API
reports four approve votes, zero rejections, `vote_state="awaiting_operator"`.

Against the same data directory, the final build then returned:
`POST .../authorize` **200** `authorized_not_applied`; `POST .../apply` **200**
`lesson_artifact_only` with one lesson; `GET lessons` **count=1**;
`POST .../rollback` **200** and `GET lessons` **count=0**.

Evidence: `docs/evidence/live/live-tokenlab-auto-voters-five-seat.json`.
The proposal `truthful_scope.side_effects_while_awaiting` now names the real
side effects: one memory row, one root-control audit row, four spawned SuperAgent
voter tasks, and four A2A vote requests.
## Test AE — live SuperAgent dissent blocks apply

Run `01a1c78b-…` auto-spawned four SuperAgents for one proposal (four depth-1
`super` tasks). Only two voted: one reject, one approve. The proposal state became
`rejected_by_superagent`; `POST .../authorize` returned **409** and apply also
**409**. No lesson was written. Evidence:
`docs/evidence/live/live-tokenlab-voter-dissent.json`.

This is the required single-dissent block, demonstrated with real model-spawned
voters.
## Test AF — data directory denied to workspace file tools

`WorkspaceJail::deny_path` now refuses any resolved path under the canonical data
directory. Production boot wires `config.data_dir` into the jail. Test
`denied_data_subtree_is_refused_even_inside_the_root` proves a seeded
`data/secret.txt` is refused with `E_PATH_OUTSIDE_JAIL`. Backend gate: 51 suites /
637 passed.
## Test AG — no terminal run with live tasks

The dispatch path now calls `settle_unfinished_tasks` after draining the spawn
queue and before the root’s final answer. It executes Queued tasks, resets stale
Running tasks to Queued, and fails the run with `E_INTERNAL_ERROR` if a non-root
task still cannot settle after 64 passes. Backend gate: 51 suites / 637 passed.