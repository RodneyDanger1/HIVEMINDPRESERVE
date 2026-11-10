# HiveMind HyperAgent — Truth Report

**Date:** 2026-11-10
**Tree:** `C:\Users\Riley\Desktop\hiverouter-v1`, `1.2.0-dev`, unsealed working tree
**Mode:** code/test/doc evidence plus live TokenLab runs. `DEEPSEEK_API_KEY`
is not present, so no DeepSeek calls were made; TokenLab points were used for
the recorded runs. Balance was not converted to USD.
**Skills applied:** capability-auditor, truthseeker, research, agent-experience,
logic-gatekeeper.

This report answers one question honestly: **what is HiveMind now?**

It is **not** an AGI, not Meta's HyperAgents, and not a self-modifying system.
It is a **local, capped, single-provider hierarchical multi-agent framework**
with a real root governor, durable branch memory, A2A messaging, a deterministic
detached-admission gate, proposal-only self-improvement, and an observable UI.

---

## 1. Claim ledger

Grades: **VERIFIED** = executed test/build evidence; **READ** = file read;
**INFERRED** = conclusion from read facts; **UNVERIFIED** = no proof.

| ID | Claim | Grade | Evidence |
|---|---|---|---|
| T1 | Backend gate is green | **VERIFIED** | `cargo test --workspace --locked`: 50 suites, 633 passed, 0 failed, 2 ignored |
| T2 | Web gate is green | **VERIFIED** | typecheck 0, 231 tests passed, production build 0 |
| T3 | Portable package builds | **VERIFIED** | `build-desktop.ps1` exit 0; `dist/HiveMind.zip` SHA-256 `B95F2BA5DD559AC00B66FB9E0656C4AC57586C03A97F3358FD0E292FD996FF28` |
| T4 | Server binary hash | **VERIFIED** | `ACBAD87E2721F02CCCB513A764A6D16C4F9CAF7EB45A879DAC7AC4863A86B74C` |
| T5 | Exact hierarchy exists | **VERIFIED (live)** | Exact-two live run: 7 tasks (1 hyper / 2 super / 2 agent / 2 subagent) |
| T6 | A2A ask_parent works | **VERIFIED (live)** | One message, attempts=1, answered by parent |
| T7 | Root-created remote+daemon works | **VERIFIED (live)** | Porch light + Pulse watchdog, device+grant, audit rows |
| T8 | Lower-level request_detached works live | **VERIFIED (live)** | Agent-created Lantern remote; 3 tasks; 6,079 tokens; audit rows |
| T9 | Duplicate detached request refused | **VERIFIED (test)** | `lower_level_request_detached_duplicate...` |
| T10 | Daemon admission is audited | **VERIFIED (test)** | `lower_level_request_detached_daemon_is_admitted_and_audited` |
| T11 | Root control feeds results into second turn | **VERIFIED (test)** | `hyperagent_checkpoint_refuses...` asserts second prompt contains prior results |
| T12 | Root decisions are durable | **VERIFIED (test)** | run-bank `root_gate:` entry + `root_control` audit row |
| T13 | Same-run rewire guard | **VERIFIED (test)** | `rewire_to_a_foreign_run_parent_is_refused_before_mutation` |
| T14 | HyperAgent memory writes to run bank | **VERIFIED (test)** | `hyperagent_can_spawn...` asserts run bank, not branch bank |
| T15 | Provider advertises tier∩persona tools | **VERIFIED (test)** | `superagent_request_advertises_tier_persona_intersection_only` captures the request and asserts no `file_write`/`shell_exec` |
| T16 | Persona aliases resolve | **READ** | alias unit test exists; no live provider proof |
| T17 | Root `spawn_superagent` allow path | **READ / DEAD** | every validated plan has depth-1; allow branch unreachable |
| T18 | Network remote / satellite | **UNVERIFIED / ABSENT** | `in_process` only |
| T19 | Daemon live tick | **UNVERIFIED** | enrolled record; DB-check job exists; no live tick proof |
| T20 | Full 40-node live scale | **UNVERIFIED / NOT RUN** | caps allow; no evidence |
| T21 | 3-repeat exact-two stability | **UNVERIFIED** | 2 of 3 completed; one provider timeout with 0 tasks |
| T22 | Cross-run learning / weight update | **ABSENT** | no training loop, no corpus, no weight mutation |
| T23 | Meta HyperAgent self-modification | **ABSENT** | no editable task+meta program, no archive/evolution sandbox |
| T24 | AGI | **FALSE** | six-condition test below fails five conditions |
| T25 | Live request_id persisted | **VERIFIED (live)** | run `01a1c619-5ab9-7208-9691-7a73c543bd36`: 8,415 tokens; `remote_created` carries `live-final-1`; one remote/device/grant |
| T26 | Turn-2 cannot create boundaries | **VERIFIED (test)** | `root_second_control_turn_cannot_create_boundaries`; audit `turn_authority` refusal |
| T27 | Final-build exact-two regression | **VERIFIED (live)** | run `01a1c62c-4e1f-7172-9945-6a70af9230e1`: completed, 7 tasks, 17,486 tokens |
| T28 | Read-only file tool use | **VERIFIED (live)** | run `01a1c639-fdef-76ed-8014-3fcd2bed9dce`: `file_read` ok; exact heading returned |
| T29 | Approval-gated file write + artifact | **VERIFIED (live)** | run `01a1c63b-53f4-71a1-b634-ef9aae1357d2`: approval nonce approved; `write-test.txt` = `APPROVED_WRITE_OK` |
| T30 | Web research tool | **VERIFIED (live)** | `web_fetch` approval-gated; run `01a1c675-1afd-71c8-afa2-0ef0a202ba8c`: HTTP 200, body contains `Example Domain`, 9,351 tokens |

No claim of "AGI", "self-improving", "autonomous HyperAgent", or "network
satellite" survives this ledger.

---

## 2. What the HyperAgent really is

```text
HyperAgent (root task, class hyper, depth 0)
  persists: root bank, run memory, usage, gate/audit rows
  can:
    - call planner once (one counted repair on schema failure)
    - run two bounded control turns:
        spawn_superagent (refused when plan already has branch leads)
        spawn_remote_agent (fixture principal + device + grant)
        spawn_daemon (enrolled record)
        read_memory_summary (run/hive/branch, bounded)
        propose_improvement (proposal-only, applied:false)
        finish
    - synthesize all completed child results in a tool-less final turn
  cannot:
    - edit its own code or swap modules
    - run continuously / observe asynchronously
    - decide an arbitrary lower-level request in a live LLM turn
    - add a branch when the planner already supplied one
```

The root control loop is **two turns**, not one. Turn 1 may act; turn 2 receives
the durable result text (`Prior root-control results...`) and may finish or make
one final call. This is bounded observation, not autonomous self-modification.

**Root branch dead path:** `has_planned_branch` is true for every valid plan;
the `spawn_superagent` allow branch cannot run. The tool still exists and is
advertised. This is a known truth gap, not a capability.

**Lower-level admission:** `request_detached` is available to SuperAgent/Worker
only. A deterministic gate checks caller tier, reason length, cross-kind name
uniqueness, and a cap of 4 lower-level detached principals per run. It creates
with `agent:<task>` attribution and writes `hyper_admission`/`root_control`
audit rows. It is **not** an asynchronous HyperAgent approval turn.

**Proposal-only improvement:** `propose_improvement` writes a structured
proposal to the run bank with `applied:false` and `requires_human_review:true`.
No code is edited, executed, or promoted. This is the intentional safety
boundary against a Skynet-style recursive self-modification loop.

---

## 3. What each level really is

| Level | Is | Is not |
|---|---|---|
| HyperAgent | Root planner/synthesizer + bounded governor | Meta self-modifying hyperagent; continuous orchestrator; async approval daemon |
| SuperAgent | Depth-1 branch lead with hive bank | NVIDIA product; DeepSeek Super-agent; shell/file-write capable under its effective persona |
| Agent | Depth-2 worker + spawner of Subagents | Root; remote creator; unrestricted shell |
| Subagent | Depth-3 temporary leaf | Spawner; writer; requester of detached resources |
| Remote | Tree-free in-process fixture principal/device/grant | Model, network satellite, task-tree member |
| Daemon | Enrolled tree-free record; DB-check job | Continuously running OS daemon in this build |
| ToolAgent | Census tier | Shipped producer or callable runtime role |

Tool tiers: **10** registered tools, **5** permission classes, **32** tier grants.
`Control` is the new permission class for the governed `request_detached` tool.
Effective model-facing surface is **tier ∩ persona**; a SuperAgent is not offered
the tier's `file_write`/`shell_exec` because its persona does not allow them.

---

## 4. What improved (20 improvements, with status)

Status labels: **APPLIED-EVIDENCED** (test or live), **APPLIED-UNTESTED**
(source only), **PROPOSED**, **RESEARCH-ONLY**.

| # | Improvement | Status | Evidence |
|---:|---|---|---|
| 1 | Lower-level `request_detached` tool | APPLIED-EVIDENCED | live Lantern run; duplicate/daemon tests |
| 2 | Deterministic HyperAgent admission policy (tier, reason, name, cap 4) | APPLIED-EVIDENCED | `execute_request_detached`; tests |
| 3 | `hyper_admission` audit row with decision/reason | APPLIED-EVIDENCED | `record_admission`; live lower-level run |
| 4 | `root_control` audit row for root decisions | APPLIED-EVIDENCED | `record_control_decision`; root gate test |
| 5 | Durable `root_gate:` run-bank decision record | APPLIED-EVIDENCED | `record_root_gate_decision`; root gate test |
| 6 | Root control result fed into a second bounded turn | APPLIED-EVIDENCED | second prompt assertion; root tests |
| 7 | Root `propose_improvement` proposal-only ledger | APPLIED-EVIDENCED | root proposal test (`applied:false`) |
| 8 | One extra root SuperAgent per control run | APPLIED-EVIDENCED | `root_super_spawned` guard; tests |
| 9 | Cross-kind duplicate detached names refused | APPLIED-EVIDENCED | unified `root_all_names`; tests |
| 10 | Daemon admission audit | APPLIED-EVIDENCED | daemon test asserts `root_control` row |
| 11 | Device/grant attribution to real creator | APPLIED-EVIDENCED | device/grant audit tests; live Lantern |
| 12 | Remote creation actor from `hyper:`/`agent:` | APPLIED-EVIDENCED | live remote data; audit rows |
| 13 | Cross-run rewire refused before mutation | APPLIED-EVIDENCED | `same_run_rewire`; WS test |
| 14 | HyperAgent memory writes to run bank | APPLIED-EVIDENCED | run-bank vs branch-bank test |
| 15 | Persona aliases (`super`/`worker`/`hyper`) | APPLIED-UNTESTED | alias unit test; no live request capture |
| 16 | Provider advertises tier ∩ persona tools | APPLIED-EVIDENCED | focused request-capture test |
| 17 | Child result explicitly labelled direct child | APPLIED-UNTESTED | prompt construction; no model-quality test |
| 18 | Root memory summary honors scope and limit | APPLIED-UNTESTED | source; no focused test |
| 19 | A2A duplicate attempt audited | APPLIED-UNTESTED | source; no duplicate-refusal audit test |
| 20 | Root detached duplicate guards (remote/daemon) | APPLIED-EVIDENCED | guards + duplicate tests |
| 21 | `request_id` idempotent remote admission | APPLIED-EVIDENCED | same-key retry test; different payload refuses |

Also present from earlier waves: exact-two root gate, planner repair, usage
checkpoint/hard-kill recovery, rate governor, JSON contract, HoneyGlass UI.
These are real, but they are not a claim of general intelligence.

---

## 5. What is still missing

1. **No asynchronous HyperAgent approval turn.** Lower-level admission is a
   deterministic policy gate; the root does not wake up to reason about a request.
2. **Dead root `spawn_superagent` allow path.** Remove it or define a valid
   no-branch plan topology.
3. **No network remote / satellite.** Current transport is `in_process` fixture.
4. **No daemon runtime proof.** Enrolled record only; no live tick evidence.
5. **No self-modification / meta-agent.** The proposal ledger is not an
   editable meta-level improvement mechanism.
6. **No cross-run learning.** Memory and proposals exist; weights and procedural
   skills do not change.
7. **No full 40-node live scale proof.**
8. **Exact-two stability is 2/3**, not 3/3; one provider timeout.
9. **Root decisions are split between memory and `remote_audit`** rather than a
   dedicated immutable control ledger; retention/export is not yet first-class.
10. **Persona/tier intersection lacks a focused request-capture test.**
11. **Rewire changes parent/context but does not rebind memory-bank ownership.**
12. **Detached grants are permanent fixture grants**, not per-request nonce/TTL.
13. **A2A late/expired/terminal answer audit is incomplete.**
14. **Remote/daemon memory scoping is absent** (no demonstrated leak, but no
    isolation proof).
15. **No learned world model, curiosity signal, open-ended goal generation, or
    self-model calibration.** These are the AGI conditions HiveMind does not meet.

---

## 6. AGI: why the answer is no

A useful six-condition test:

1. **D1 — held-out breadth:** performance across unrelated domains without
   per-domain engineering. HiveMind is a generic orchestrator, not a learned
   generalist.
2. **D2 — transfer:** improvement on one task improves a different task. No
   weight/procedure learning exists.
3. **D3 — continual learning:** retain and improve across runs with a forgetting
   bound. Memory is storage, not learning.
4. **D4 — calibrated self-model:** reliably know its own limits. Refusals and
   caps are hard-coded; no calibrated self-assessment.
5. **D5 — open-ended goals:** generate and pursue its own goals. The operator
   supplies the prompt; the planner generates a bounded DAG.
6. **D6 — safety envelope:** caps, named refusals, approvals, sandbox, audit.
   **Partially met** — real caps/refusals/approvals, but no verified formal
   containment or adversarial campaign.

HiveMind fails D1–D5 and only partially satisfies D6. It is a **narrow,
single-run, capped HMAS**, not AGI. The correct ambition is a better HMAS, not a
fake AGI claim.

---

## 7. Why self-modification was rejected

The pseudocode archive includes a self-modifying HyperAgent sandbox. That design
is not implemented, deliberately:

- **No predictability:** rewritten logic cannot be fully tested before it runs.
- **Goal/alignment drift:** a meta agent can change its own success criteria.
- **Sandbox escape:** any code-execution path is a security boundary, not a
  feature.
- **Runaway loops:** recursive mutation can consume compute and budget without a
  stop condition.
- **Provenance loss:** a mutated binary has no stable audit identity.

The safe substitute is the **proposal ledger**: the agent can propose, but only a
human/rigid external gate can apply, test, and promote. HiveMind records
`applied:false` and `requires_human_review:true`; it has no code-write path from
the proposal tool.

---

## 8. Live evidence summary

| Run | Result |
|---|---|
| Exact-two | completed; 7 tasks; 15,551 tokens; 106 s |
| Exact-two repeat | completed; 7 tasks; 13,751 tokens |
| Exact-two repeat 2 | **failed `E_PROVIDER_TIMEOUT`**, 0 tasks |
| A2A ask_parent | completed; 3 tasks; attempts 1; 6,924 tokens |
| Root remote + daemon | completed; 3 tasks; Porch + Pulse; 5,555 tokens |
| Lower-level request_detached | completed; 3 tasks; Lantern; 6,079 tokens |
| Rate governor | `E_LLM_RATE_LIMIT` in 0 s, 0 tasks, retry succeeded |
| Hard kill | 449 measured tokens retained; 7 tasks recovered as `interrupted` |

These are real Token Lab runs, not transcripts. They prove the hierarchy,
admission, A2A, detached records, and crash accounting. They do not prove
network remotes, daemon ticks, 40-node scale, or stable 3/3 exact topology.

---

## 9. Final verdict

**HiveMind is a credible local HMARS prototype/working framework.** Its root
governor, branch hierarchy, A2A, memory banks, detached fixture records, audit
rows, crash-safe usage, and UI are real and tested. Its strongest new governance
improvement is the deterministic lower-level detached admission gate with audit
and a two-turn root feedback loop.

**It is not AGI, not Meta HyperAgents, not a self-modifying agent, and not a
network HMARS.** The remaining work is concrete engineering — network transport,
daemon runtime, root branch simplification, dedicated control ledger, live
scale/stability, and focused tests for persona filtering and A2A duplicate
audit — not a marketing claim.

This is the truthful state to hand to the next session.

## 10. Code-level evidence (current tree, 2026-11-10)

The claims above are not prose-only. These are the exact current anchors.

### Root control is two bounded turns, with feedback

`backend/crates/hiverouter-core/src/execution.rs:3903-3925`:

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

Turn 2 cannot create boundaries (`execution.rs:3940-3952`), and the refusal is
written through `record_root_gate_decision` (audit + run bank). This is the
anti-escalation rule.

### Lower-level detached admission

`execution.rs:4214-4294` parses `kind`, `display_name`, `reason`, optional
`request_id`, and a SHA-256 payload fingerprint. The admission policy:
- caller tier must be SuperAgent or WorkerAgent;
- reason must be at least 12 characters;
- name must be unique across remotes and daemons;
- lower-level detached cap is 4;
- same request_id + same fingerprint returns the existing remote;
- same request_id + different fingerprint returns `E_DETACHED_CONFLICT`.

```rust
let request_fingerprint = hiverouter_tools::sha256_hex(
    serde_json::json!({"kind": kind, "display_name": display_name, "reason": reason})
        .to_string().as_bytes(),
);
if let (Some(request_id), Some(store)) = (request_id, self.remote.as_ref()) {
    if let Ok(Some((existing, stored))) = store.find_by_request_id(request_id) {
        if existing.display_name.eq_ignore_ascii_case(display_name)
            && existing.adapter_kind == "light"
            && stored.as_deref() == Some(request_fingerprint.as_str()) {
            return ToolExecution::ok(..., "idempotent replay ...");
        }
        return refused(Refusal::DetachedConflict.code(),
            "request_id was reused with a different detached payload");
    }
}
```

### Audit and attribution

- `remote.rs:424-452` reads the original request id and its payload hash from
  `remote_audit`.
- `remote.rs:1377-1405` writes `root_control` rows.
- `remote.rs` `create_remote` writes `remote_created`; `create_device` and
  `create_grant` derive actor kind from `created_by` (`hyper:`/`agent:` ->
  `system`), never silently `operator`.
- `execution.rs` lower-level remote branch writes `hyper_admission` with
  `decision:"allow"` and the caller/reason.
- Every lower-level `request_detached` refusal also writes a `root_control`
  audit row with decision `refused` through the same admission closure; a
  duplicate name and a payload-mismatched request id are attributable, not
  silent.

### Tool surface (12 tools, 5 permission classes)

`hiverouter-tools/src/tool.rs` registers `request_detached` as
`PermissionClass::Control`; SuperAgent/WorkerAgent allowlists include it;
Subagent does not. Effective provider advertisement is
`provider_executor.rs` `tools_for_task`: the tier tool list is filtered by the
persona allowlist. Test:
`superagent_request_advertises_tier_persona_intersection_only`.

### Same-run rewire and memory

- `ws.rs:398-400` defines `same_run_rewire`; `ws.rs:817-821` refuses a foreign
  run parent with `E_TASK_NOT_FOUND` before any graph/context write.
- `store.rs` `caller_bank_for_class` maps `hyper -> bank:run:<run>`,
  `super -> bank:hive:<task>`, `agent/subagent -> bank:branch:<task>`.
- Class-hyper completion writing to the run bank is pinned by
  `hyperagent_can_spawn_a_detached_remote_without_a_task_row`.

### Tests that prove these claims

- `hyperagent_governance.rs` (4 tests): admission, duplicate, root gate memory,
  foreign-run rewire.
- `run_lifecycle.rs`: root two-turn feedback, root turn-2 authority, idempotent
  retry, idempotent mismatched payload, daemon admission, duplicate names, run
  bank.
- `provider_adapter.rs`: tier∩persona request capture.
- Full gate: **50 suites, 633 passed, 0 failed, 2 ignored**.

### What this does NOT prove

No network transport. No daemon live tick. No 40-node live scale. No 3/3
exact-two stability. No self-modification or AGI. The detached remote is still
an `in_process` fixture; the daemon is an enrolled record.
## 19. Authorized self-evolution contract

HiveMind distinguishes a **recorded proposal** from an **applied change**. The root
`propose_improvement` payload now carries an explicit five-seat gate:

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

It also carries `truthful_scope.what_changes`, `what_does_not_change`, and
`side_effects_while_awaiting`. Recording never mutates code, schema, tool grants,
prompts, or running agents. A proposal stays `awaiting_votes` until four spawned
SuperAgent approvals and the operator approval are recorded **and unanimous**; the
current tree does not implement the vote-submission path or automatic application.
Test: `root_can_record_but_never_auto_apply_an_improvement_proposal`.
## 20. `web_fetch` network capability (approval-gated)

The registry now has an 12th tool, `web_fetch`, permission class `Network`.
It is granted to SuperAgent and WorkerAgent, not to HyperAgent or Subagent.
The call path is:

`model tool call -> authorize_tool_call -> request_model_tool_approval ->
APPROVAL_REQUEST -> operator nonce -> execute_approved -> execute_web_fetch ->
reqwest GET -> bounded tool result`

The implementation (`execution.rs`, `execute_web_fetch`):
- accepts absolute `https` only, default port only, no URL credentials;
- resolves the host and refuses loopback/private/link-local/unspecified/multicast
  IPv4 and IPv6 destinations;
- does not follow redirects;
- times out at 15 seconds and caps the body at 1 MiB;
- returns `E_TOOL_TIMEOUT`, `E_REMOTE_TRANSPORT`, `E_FILE_TOO_LARGE`, or
  `E_TOOL_CANCELLED` by name; no silent success.

Live evidence: approval-gated fetch of `https://example.com` returned HTTP 200,
577 bytes, containing `Example Domain`, with exactly one `web_fetch` tool row.
### Web-fetch negative test

With `HIVEROUTER_SIDE_EFFECTS=0`, the same `web_fetch` request was approved but
refused with `E_TOOL_NOT_ALLOWED`; tool rows were `web_fetch: refused` and
`report_result: ok` only. No HTTP effect was claimed. Evidence:
`docs/evidence/live/live-tokenlab-web-fetch-killswitch.json`.
### Hardening found by the read-only web audit

The first implementation passed a live approval test, then a read-only audit
found four real defects: the global side-effect kill switch did not cover
`web_fetch`; DNS resolution was unbounded and uncancellable; the 1 MiB cap was
checked after `response.bytes()` buffered the body; and a network-only fetch
required a filesystem jail. All four are now fixed: DNS has a 10s timeout plus
cancellation, the body is streamed chunk-by-chunk with a pre-buffer cap, the
jail requirement is bypassed only for `web_fetch`, and the side-effect switch is
checked before the fetch. A separate real bug was fixed: hyphenated persona ids
(`content-writer`) no longer normalize to underscores and silently fall back to
the full tier list; test
`hyphenated_persona_uses_its_own_allowlist_not_the_full_tier`.

A negative live run with `HIVEROUTER_SIDE_EFFECTS=0` approved the call and got
`E_TOOL_NOT_ALLOWED`; tool rows were only `web_fetch: refused` and
`report_result: ok`. Evidence:
`docs/evidence/live/live-tokenlab-web-fetch-killswitch.json`.
## 21. Proposal surface (read-only)

`GET /api/agents/runs/{run_id}/proposals` exposes recorded improvement proposals
and their governance block. The response explicitly sets `applied:false` and
`vote_path:"superagent_votes_recorded; operator_apply_not_implemented"`. The web `ProposalsPanel` renders this projection
and contains no approve/apply button.

The proposal payload now carries a stable `proposal_id`, and SuperAgents can
record a durable `approve`/`reject` vote with `vote_evolution` (SuperAgent-only,
one vote per task, no code effect). The API aggregates votes into
`awaiting_votes`, `awaiting_operator`, or `rejected_by_superagent`. The operator
has a fifth-seat authorization endpoint/UI; it requires four distinct approvals
and zero rejections, and records `authorized_not_applied`. `POST .../apply`
appends one lesson line to `evolution/LESSONS.jsonl`; `POST .../rollback` removes
it after verifying the apply hash. Scope is lesson artifact only; no code,
schema, prompt, tool, or persona mutation exists and `can_apply` is false.
Live verification of the proposal surface: run `01a1c6a2-6c25-7333-a970-8ef8d6a61a0c`
proposed the same content twice across the two root-control turns; the ledger
returned **count=1**, `applied=false`, `awaiting_votes`, and a SuperAgent vote path that records votes but never applies them. Content dedupe is enforced, but this is still not an
authorization or apply mechanism.