# Improvement Plan From Papers — 3 candidate codeable changes

**Author:** Improvement Planner 4/4 · **Date:** 2026-11-10 · **Repo:** `C:\Users\Riley\Desktop\hiverouter-v1`
**Mode:** READ-ONLY plan. This file is the only write. HEAD `848735a`, tree dirty (19 M / 3 ??).
**Inputs:** `A Mechanistic Audit …` (paper 1), `Recursive Multi-Agent Evolutionary Topologies …` (paper 2),
`docs/plans/HIVEMIND-FINISH-PLAN-v3.md` (plan), current source. Papers are hypotheses, not authority.
**Scope law:** operator-gated only. No AGI claim, no autonomous source mutation, no weight training,
no "learning gain" claim. Every claim below is tagged VERIFIED (read this session), READ (paper/plan
statement, not re-derived), or UNVERIFIED.

## 1. Ranked candidates

| # | Candidate | Files touched | Size | Why ranked here |
|---|---|---|---|---|
| 1 | **EVO-GATE** — held-out evaluation receipt gate for evolution lessons | 1 new core module, `http.rs` apply/loader, 1 test file | S (≈250 LOC + tests) | Smallest change at the highest-trust boundary; authorize/apply/rollback already exist; directly tests paper H2 |
| 2 | **ROOT-MONITOR** — one bounded post-dispatch root monitor turn | `provider_executor.rs`, `execution.rs`, governance test | M (≈400 LOC + tests) | Plan WP-GOV-2 is explicitly not landed; product value highest, but needs a scripted-provider harness and careful caps |
| 3 | **EVIDENCE-LEDGER** — per-run oracle-integrity counters | `ledger.rs`, `tools_bridge.rs`, one read-only route, ledger test | S (≈150 LOC + tests) | Cheapest, privacy-safe, but low leverage alone; makes "no test tampering" falsifiable |

Shared non-goal: none of these judges open-ended quality. #1 verifies a receipt from an operator-run,
out-of-scope evaluator (paper 2 §III.2.5: the oracle must be immutable and outside agent write scope).

## 2. Candidate 1 — EVO-GATE (recommended)

**Files/symbols.** New `backend/crates/hiverouter-core/src/evolution_gate.rs`:
`pub struct EvalReceipt`, `pub fn verify_receipt(dir: &Path, proposal_id, head_commit) -> Result<VerifiedReceipt, Refusal>`,
`pub fn receipt_sha256(&EvalReceipt) -> String`. Wire in `backend/crates/hiverouter-core/src/lib.rs`
(module decl). Change `http.rs::apply_evolution_lesson` (`http.rs:2720-2830`),
`evolution_lessons` (`http.rs:2648`), and `execution.rs::load_authorized_lessons` (`execution.rs:89-106`).
Tests: new `backend/crates/hiverouter-core/tests/evolution_gate.rs`; update the fixture in the existing
unit test at `execution.rs:5713`.

**Current behavior path.** `POST /api/evolution/proposals/{id}/apply` requires an authorization memory
row and a proposal-sha match (`http.rs:2737-2744,2757-2765`), refuses replay (`:2746-2748`), sanitizes
four free-text fields (`:2796-2804`), then appends a lesson line whose only evidentiary field is
free-text `evidence` (`:2805-2813`) with `sha256`/`previous_sha256` receipts (`:2821-2830`).
`load_authorized_lessons` accepts any JSONL line with `area`/`problem`/`proposal` and injects up to 8
into prompts (`execution.rs:61-76,89-106`), at `execution.rs:2525,2528,4257-4284,4341`.
So cross-run influence — the only persistent learning channel — is operator-signed but **not
evidence-gated**, and a hand-written line on disk is trusted at boot.

**Minimal implementation sketch.**
1. Receipt file `data_dir/evolution/eval/<proposal_uuid>/receipt.json`, max 64 KiB, symlink-refused,
   canonicalized under `data_dir` (already denied to the workspace jail, `http.rs:350`; also deny in
   `tools_bridge` per WP-SEC-1 so the gate does not depend on one deny site). Schema
   `hivemind.evolution-eval.v1`: `evaluator {id, sha256, version}`, `task_set {id, sha256, n}`,
   `baseline {pass_rate, ci_lower, ci_upper}`, `candidate {pass_rate, ci_lower, ci_upper}`,
   `threshold {min_delta, pre_registered_at}`, `command`, `observed_at`, `head_commit`, `artifacts[]`.
2. Boot-pinned `evolution/eval/TASK-SET.sha256`; refuse if the receipt's `task_set.sha256` differs
   (held-out set unchanged since pre-registration).
3. Admission (`apply_evolution_lesson`, insert after `:2765`): missing/malformed/stale receipt →
   `409 ApprovalRequired` + `detail` naming the defect; `candidate.ci_lower <= baseline.ci_upper +
   min_delta` → `422 SchemaInvalid` + detail. Bind `receipt_sha256` into the authorization and the lesson
   line; copy `baseline`/`candidate`/`task_set` into the line, not the whole receipt.
4. Loader hardening (`execution.rs:89`): skip lines lacking `receipt_sha256`, `state=="applied_lesson"`,
   and an in-range `n`; log a count, never the line. This closes the boot-trust gap honestly without
   claiming the operator file is unwritable.
5. `GET /api/evolution/lessons` (`http.rs:2648`) adds `receipt_status`, `task_set_sha256`, `received_at`.

**Tests/commands.** `cargo test -p hiverouter-core --test evolution_gate --locked` covering: valid
receipt applies and the line carries the receipt sha; missing receipt refuses; task-set sha mismatch
refuses; `ci_lower` at/below baseline refuses; stale `head_commit` refuses; 64 KiB+1 receipt refuses;
symlinked receipt refuses; rollback (`http.rs:2856-2925`) removes only the gated line; loader skips a
receipt-less line. Existing `load_authorized_lessons_parses_records_and_skips_foreign_lines`
(`execution.rs:5713`) must go RED first (its fixture has no receipt) — that red is the gate's proof.
Then `cargo test --workspace --locked`; `node tools/qa/check-docs-truth.mjs`; `node --test tools/qa/tests/*.test.mjs`.

**Security/privacy.** Receipt is operator input but reaches a prompt: keep `memory_text_is_safe`
(`execution.rs:111-125`) checks on copied fields, cap lengths, strip absolute paths (store basenames
only), never log `command` or receipt contents; TOCTOU: hash the bytes once and verify against that
buffer; refuse non-regular files. Do not add new `Refusal` variants (they are counted: plan §1 records
78 codes and the docs checker cares) — reuse `ApprovalRequired`/`SchemaInvalid` with distinct `detail`.

**Rollback.** Revert `evolution_gate.rs`, the apply call, the loader filter, the endpoint projection.
Already-applied lesson lines remain parseable because extra fields are additive; real rollback of a
lesson already uses the existing `rollback_evolution_lesson` path (`http.rs:2856`). No migration.

**Measurable effect.** Lessons admitted without a bound receipt: currently possible, target 0
(asserted). Lessons carrying `baseline`/`candidate`/CI/task-set sha: 0 today → 4 fields in the gate
test. Claims of the form "lesson L raised held-out pass rate by Δ, 95% lower bound > 0" become
checkable per line.

**Why the papers.** Paper 2 §III.2.5 documents evaluator-visible hacking (DGM node 114, Baker et al.
2025) and prescribes exactly this control set; §IV.4 E2 makes hiding the evaluator the experimental
arm. Paper 1 §2.3-2.4 frames the system as proposal-only with no cross-run learning — the lesson file
is the one channel that *does* cross runs, so it is where the paper's own honesty constraints bite.

## 3. Candidate 2 — ROOT-MONITOR

**Files/symbols.** `provider_executor.rs`: `ROOT_CONTROL_SYSTEM` (`:39-53`), root tool schemas
(`spawn_superagent` `:586-598`, `assign_task` `:602-614`). `execution.rs`: post-dispatch root turn
(the root currently runs one control pass then the answer-only final; `execution.rs:2056` region per
plan §1.1). Tests: `tests/hyperagent_governance.rs`, `tests/proposal_authorization.rs`.

**Current behavior path.** Root vocabulary is spawn/assign plus four control tools; grep over
`backend/crates/hiverouter-core/src/**` finds **no** `monitor` symbol and no `send_instruction` /
`cancel_task` tool — VERIFIED. After dispatch there is no bounded root review turn; `kill_worker` is a
WS command, not a root tool. Plan WP-GOV-2 (`plan:193-203`) specifies this work and says the tools do
not exist.

**Sketch.** After dispatch and before final, if the run is non-terminal: one monitor turn, provider
budget taken from the existing cap formula (`caps.rs`), advertising exactly
`send_instruction {task_id, text}` (task must belong to the run; ≤2 000 chars; ≤1 per task) and
`cancel_task {task_id, reason}`. At most one replacement branch, only for a terminal failed task and
only under caps; no mutation after final; final stays tool-free and must report status/changes/
non-changes/evidence. Record each accepted monitor action as a scoped memory row (no schema).

**Tests/commands.** Scripted-provider test: assign → child uses instruction → monitor reacts → final;
late-mutation refusal; cap refusal *before* row creation; no zombie task. `cargo test -p
hiverouter-core --test hyperagent_governance --locked`, then workspace suite.

**Risks/rollback.** Prompt injection via child output and via `send_instruction` text — treat as data,
bound, never re-inject as system text; monitor must not raise caps. Rollback: revert turn + schemas;
memory rows are inert.

**Measurable effect.** Monitor turns per dispatched run 0 → 1; replacement branches ≤1; a scripted
failure shows repair rate 1/1 with no provider-call overrun.

**Papers.** Paper 1 §2.1 (two-turn root governor) and §2.3; paper 2 §III.2.6 (structured traces are
what make blame assignment tractable) — monitor actions must be structured rows, not chat.

## 4. Candidate 3 — EVIDENCE-LEDGER

**Files/symbols.** `hiverouter-tools/src/ledger.rs`: `LedgerRow` (`:82-105`), `LedgerRecord` (`:109-128`),
`RunLedger::record` (`:214`). Call sites in `tools_bridge.rs`. New read-only route
`GET /api/evidence/runs/{run_id}/integrity` in `http.rs` (routes block `:612-696`). Test:
`tests/approval_ledger.rs`.

**Current behavior path.** Ledger rows record `tool`, `permission_class`, `args_sha256`, `outcome`,
`refusal_code`, `partial_verdict` (`ledger.rs:82-105`) — **no path or target class**, so "did this run
touch tests, CI, or oracle files?" is unanswerable from the ledger; `/api/usage/runs/{id}` cannot
answer it either. Paper 2 §III.2.5 requires diff auditing of tests/CI/logging; today that is manual.

**Sketch.** Derive `target_class: Option<LedgerClass>` (`Tests|Ci|Oracle|Source|Other`) inside the
bridge from the resolved path *argument*, store only the class (never the path), add one serde-default
field so old rows parse; expose counts + refusal histogram via the new route; document the counter in
the claims register. Two lines of code at the bridge boundary; no migration (JSONL).

**Tests/commands.** A seeded `file_write` to `backend/.../tests/...` yields `tests_touched=1`; an
honest run yields 0; a `file_read` of a test file classifies as `Oracle` or `Tests` by one stated rule.
`cargo test -p hiverouter-core --test approval_ledger --locked`; workspace suite; docs checker.

**Risks/rollback.** Path leakage is the risk — mitigated by storing only the class; classification
rules must be documented or the count is itself a claim. Rollback: drop the field (serde default keeps
rows readable) and the route.

**Measurable effect.** Integrity counters per run 0 → 5 classes; a single "test-touching run" fixture
is distinguishable from an honest run, so a "no oracle tampering" claim becomes falsifiable.

**Papers.** Paper 2 §III.2.5 / §IV.4 E2 (hacking is detected by auditing test and CI diffs); paper 1
§3.1-3.2 (legible boundedness is the design property worth measuring).

## 5. Rejected ideas (cannot be built honestly here)

1. **Self-modifying scaffold evolution / Darwin Gödel Machine loop** — paper 2 §III.7 says containers
   are not a strong boundary for generated code; this repo runs one API key and 8 GB RAM; plan §6
   forbids autonomous source mutation. A production-quality held-out oracle for HiveMind's own source
   does not exist here. Reject.
2. **Sub-4 GB distillation pipeline** — requires gradients and a GPU fleet; paper 2 itself calls
   parity H (§II.3 H3) and cites ~$22k for one DGM run (§III.5.4). No training path exists in this
   repo. Reject, do not simulate.
3. **Evolutionary routing/topology search ("Colosseum and Senate")** — no evaluator corpus, no VPS
   simulation matrix, and paper 2 §III.2.3 warns program crossover is often destructive. Reject.
4. **"Autonomous learning" claims from lesson retrieval** — lessons are prompt data
   (`execution.rs:61-76`); nothing updates weights or policies. Keep the plan's L1-only wording.

## 6. Claim ledger

| ID | Claim | Status | Basis |
|---|---|---|---|
| C1 | Lessons are injected into 5 prompt sites, fenced as data | VERIFIED | `execution.rs:61-76,2525,2528,4257-4284,4341` |
| C2 | Lesson admission carries no held-out evaluation, only free-text `evidence` | VERIFIED | `http.rs:2796-2813` |
| C3 | Boot trusts receipt-less lesson lines on disk | VERIFIED | `execution.rs:89-106` |
| C4 | Apply/rollback already bind sha256 and refuse replay | VERIFIED | `http.rs:2746-2765,2805-2830,2870-2925` |
| C5 | v24 `tasks.instruction` is implemented in the dirty tree | VERIFIED | `db.rs:860-865` |
| C6 | Root schemas advertise `spawn_superagent`/`assign_task` with properties | VERIFIED | `provider_executor.rs:586-614` |
| C7 | No root monitor turn or `send_instruction`/`cancel_task` tool exists | VERIFIED | grep over `hiverouter-core/src` (0 hits) |
| C8 | Tool ledger records no path/target class | VERIFIED | `ledger.rs:82-105` |
| C9 | DGM SWE-bench 20.0→50.0, Polyglot 14.2→30.7 | UNVERIFIED | paper 2 §II.1; primary not fetched |
| C10 | ~2 bits knowledge/parameter capacity law | UNVERIFIED | paper 2 §III.4.2; primary not fetched |
| C11 | Proposal authorization requires N votes / TTL | NOT READ | `http.rs:2522-2645` not inspected this pass |
| C12 | Gate/test commands in this document pass | UNVERIFIED | not run (read-only, time-boxed) |

## 7. What would falsify each improvement

- **EVO-GATE:** a lesson line without `receipt_sha256`, written directly to
  `evolution/LESSONS.jsonl`, still reaches a prompt at boot; or the gate accepts a receipt whose
  `candidate.ci_lower ≤ baseline.ci_upper`, or whose task-set sha changed after pre-registration.
  Either defect kills the "held-out evaluator" claim and downgrades the lesson to operator attestation.
- **ROOT-MONITOR:** a scripted-provider test shows the root can send an instruction or cancel after
  final, or a second replacement branch is created under caps, or monitor turns exceed 1 per run.
  Also falsified if the monitor text is re-injected as system instruction rather than data.
- **EVIDENCE-LEDGER:** a test-file write and a source-file write produce the same stored row (class
  derivation unusable), or `/integrity` counts diverge from executed tool calls, or the stored class
  reconstructs a path. Then it is an unmeasured claim, not an observation.

## 8. Suggested order and gate note

1 (EVO-GATE) → 3 (EVIDENCE-LEDGER) → 2 (ROOT-MONITOR): #1/#3 are additive (no schema, no new refusal
codes); #2 changes provider-visible behavior last. Do not run these before WP-SEC-1's `tools_bridge`
data-dir deny lands; #1 depends on it for defense in depth. Commands stay in the plan's
environment-correct forms (`plan:29-32`); strict gate stays RED at G7 against sealed 1.1.0 — unchanged.
