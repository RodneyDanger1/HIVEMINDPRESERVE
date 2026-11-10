# HiveMind HyperAgent improvement verification — task-22

**Mode:** read-only static audit. No source edits, no provider key, no paid call, no server/browser/device action. Only this file was written.
**Tree:** `C:\Users\Riley\Desktop\hiverouter-v1` @ `889b779` + dirty working tree (other lanes' edits present).
**Skills applied:** `/auditor` (independent evidence, no reporter prose), `/truthseeker` (claim provenance; unknown is not a pass), `/evidence-gap-audit` (fact → proof → gap).
**Time-box:** 8-minute seat; all statements below are file-read or test-name evidence, not runtime execution.

## 0. PSEUDOCODE source documents

`docs/audit/PSEUDOCODE-P1-META.md`, `PSEUDOCODE-P2-HMARS.md`, `PSEUDOCODE-P3-AGI.md` **do not exist** in the tree at audit time (repo-wide `PSEUDOCODE*` glob returned zero files). The only surviving source of the improvement claims is the post-audit addendum in `docs/audit/AGENT-TAXONOMY-VERIFICATION.md:824-855`; the three pseudocode documents are therefore UNVERIFIED as sources and any claim attributed only to them is RESEARCH-ONLY/unknown, not implemented evidence.

## 1. Claim register (class: APPLIED-EVIDENCED / APPLIED-UNTESTED / NOT-APPLIED / RESEARCH-ONLY)

| ID | Claim | Class | Evidence (file:line or test name) |
|---|---|---|---|
| C1 | `request_detached` is routed through a deterministic HyperAgent admission gate (caller tier, reason, name conflict, cap) | **APPLIED-EVIDENCED** | `backend/crates/hiverouter-core/src/execution.rs:2498-2499` dispatch; `:4117-4130` `execute_request_detached` doc/entry; tests `backend/crates/hiverouter-core/tests/run_lifecycle.rs:863` `lower_level_request_detached_is_admitted_by_hyper_gate`; `tests/hyperagent_governance.rs:219` `lower_level_request_detached_creates_attributed_remote_and_audit` |
| C2 | Admission emits a durable `hyper_admission` audit row naming gate/decision/reason | **APPLIED-EVIDENCED** | `backend/crates/hiverouter-core/src/remote.rs:1340-1362` `record_admission` → audit action `"hyper_admission"`; tests `run_lifecycle.rs:910-915` (action + `"decision":"allow"`), `hyperagent_governance.rs:270-275`; remote audit read `remote.rs:1373-1385` |
| C3 | `root_gate` decision is durable run-bank memory (refuse/allow/action/reason) | **APPLIED-EVIDENCED** | `execution.rs:4347-4367` `record_root_gate_decision` writes `hyper`/`root_gate:<uuid>` with scope `"run"`; tests `run_lifecycle.rs:956-997` (`root_gate` + `"action":"spawn_superagent"` + `"decision":"refused"`), `hyperagent_governance.rs:392-431`; memory read at `execution.rs:4142-4149` |
| C4 | Rewire cannot attach a task to a parent in a foreign run (same-run guard, before mutation) | **APPLIED-EVIDENCED** | `backend/crates/hiverouter-core/src/ws.rs:398-401` `same_run_rewire`; enforcement `ws.rs:812-821` returns `E_TASK_NOT_FOUND` before graph/context write; test `tests/hyperagent_governance.rs:453` `rewire_to_a_foreign_run_parent_is_refused_before_mutation` |
| C5 | HyperAgent (class `hyper`) memory/root output is written to the run bank, not an orphan branch bank | **APPLIED-EVIDENCED** | `execution.rs:4419-4429` `record_agent_memory_scoped("hyper", …, "run")`; run-bank seed `execution.rs:1798-1820`; test `run_lifecycle.rs:790-858` asserts `bank:run:<run>` exists and `bank:branch:<root>` does not; docs claim `AGENT-TAXONOMY-VERIFICATION.md:826-828` |
| C6 | Provider advertises tier ∩ persona tool set (least-privilege narrowing, not tier-only) | **APPLIED-UNTESTED** | `provider_executor.rs:188-197` `tools_for_task` filters tier specs by `persona_tool_allowlists`; wiring `http.rs:284`; `provider_executor.rs:157-162` builder. **No focused test name asserts advertised=intersection** (searched core tests: only generic `provider_adapter.rs:80` "tools advertised", and persona seeding tests `run_lifecycle.rs:1426`). Static code only; runtime advertisement unverified. |
| C7 | Remote/device/grant creation is attributed to the real actor (HyperAgent root vs lower-level agent), with separate audit rows | **APPLIED-EVIDENCED** | `remote.rs:495-500` `actor_kind` derived from `created_by`; `run_lifecycle.rs:790-838` (remote/device/grant + `created_by`/`granted_by` start `hyper:`; audit `remote_created`/`grant_created` actor_kind `system`, actor_id `created_by`); `run_lifecycle.rs:863-920` lower-level path asserts `agent:` attribution + `hyper_admission`; `hyperagent_governance.rs:219-270` |
| C8 | Duplicate root/request names are refused by name without a second row | **APPLIED-EVIDENCED** | `execution.rs` request-detached duplicate branch (source path exercised by test); test `tests/hyperagent_governance.rs:281` `duplicate_request_detached_is_refused_and_does_not_duplicate_rows` asserts one remote and `tool_turns` row with `E_DETACHED_CONFLICT`; duplicate variant `run_lifecycle.rs:924-953` |
| C9 | A2A duplicate attempts are recorded in the A2A audit trail | **APPLIED-UNTESTED** | `execution.rs:3252-3267` duplicate `ask_parent` records `record_agent_message_attempt(..., "duplicate_refused", …, MAX_A2A_ATTEMPTS)`; audit insert `store.rs:950-1010`; inline test `store.rs:3064` `a2a_attempts_and_audit_are_recorded_once_per_transition` covers attempt/audit counts, **but no test drives the duplicate-refusal audit path**. `docs/audit/REMOTE-PERMISSION-GAPS.md:445-453` still records late/expired/terminal A2A audit gaps. |
| C10 | Meta “HyperAgents” self-modifying meta-agent / evolution loop | **RESEARCH-ONLY** | `docs/audit/AGENT-TAXONOMY-VERIFICATION.md:834-855`: paper comparison; explicitly no self-modifying task+meta program, editable improvement procedure, variant archive, or evolution sandbox. Not implemented. |
| C11 | “AGI” / AGI-adjacent claim | **RESEARCH-ONLY** | Same addendum `:851-855`; honest claim is “hierarchical HMAS with a governing root”, not Meta HyperAgent and not AGI. PSEUDOCODE P3-AGI absent; no implementation artifact. |

### Counts

| Class | Count |
|---|---|
| APPLIED-EVIDENCED | **7** (C1, C2, C3, C4, C5, C7, C8) |
| APPLIED-UNTESTED | **2** (C6, C9) |
| NOT-APPLIED | **0** |
| RESEARCH-ONLY | **2** (C10, C11) |
| Total | **11** |

## 2. Evidence-gap matrix (highest-value gaps)

| Fact/claim | Proof available | Class | Gap / action |
|---|---|---|---|
| C6 advertised tool set = tier ∩ persona | Source: `provider_executor.rs:188-197`; wiring `http.rs:284`; no test | **WEAK** | Add/execute a focused provider-adapter test with a custom persona allowlist; assert request `tools[]` names equal the intersection and that a tier-only tool (e.g. `file_write` for a 7-tool persona) is absent. Until then the least-privilege advertisement claim is APPLIED-UNTESTED. |
| C9 duplicate A2A audit completeness | Source: duplicate path records attempt; store audit test covers attempts, not duplicate refusal; docs G5 says late/expired/terminal missing | **WEAK** | Add a deterministic duplicate `ask_parent` test asserting an `a2a_audit` row with duplicate reason; separately decide/implement deadline/terminal audit per `REMOTE-PERMISSION-GAPS.md:445-453`. |
| PSEUDOCODE P1–P3 as claim sources | None; files absent repo-wide | **MISSING** | Obtain the actual documents or remove claims that cite them; do not treat the addendum as the pseudocode source. |
| C10/C11 implementation equivalence | Paper abstracts/addendum only (`AGENT-TAXONOMY-VERIFICATION.md:834-855`) | **RESEARCH-ONLY** | Keep as research; no code claim should be promoted without a dated primary paper reading/citation. |

**Strongest unverified claim: C6 — the provider advertises tier ∩ persona tools.** It is the only security-relevant least-privilege claim with source implementation but **no focused test or captured request evidence**; a regression that advertises the full tier again would pass the current test suite (the generic `leaf_execute_advertises_tools_on_the_first_request` only checks non-empty). C9 is the next gap because the duplicate-refusal audit row is implemented but untested and adjacent late/expired audit gaps remain documented.

## 3. Claim-provenance corrections / unknowns

- The addendum at `AGENT-TAXONOMY-VERIFICATION.md:824-833` asserts “personas now resolve aliases; provider advertises tier ∩ persona tools; HyperAgent memory writes to the run bank; foreign-run rewire is refused; `request_detached` goes through … admission gate (caller tier, reason, name conflict, cap 4) with a `hyper_admission` audit row.” C1–C5/C7/C8 are corroborated by tests/source above; **C6 remains untested**, so the addendum sentence is only partly corroborated.
- “cap 4” in the addendum was not independently located in this 8-minute pass; it remains **UNVERIFIED** (not used as a class driver).
- A2A late/expired/terminal audit remains documented as incomplete in `REMOTE-PERMISSION-GAPS.md:445-453`; no claim of complete A2A audit is made here.
- No live/runtime behavior was executed; test names are file-read evidence, not run output.

## 4. Verdict

**AUDITED (static).** 7/11 claims APPLIED-EVIDENCED, 2 APPLIED-UNTESTED (C6, C9), 0 NOT-APPLIED, 2 RESEARCH-ONLY (C10, C11). PSEUDOCODE P1–P3 sources absent. Strongest unverified claim: **C6 persona/tier tool intersection**.
