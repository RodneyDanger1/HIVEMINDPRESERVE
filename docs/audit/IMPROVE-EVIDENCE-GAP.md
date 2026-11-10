# HyperAgent/HMARS — improvement evidence gap audit

**Mode:** read-only static audit. No source edits, no build, no `cargo test`, no server, no provider call.
**Tree:** `C:\Users\Riley\Desktop\hiverouter-v1` (working tree as found; no commit hashes taken — not verified in this pass).
**Skill:** `/evidence-gap-audit` (fact → proof → gap → action; no invented evidence, no admissibility-style rulings).
**Inputs read:** `README.md` (grep + cited lines), `docs/audit/HYPERAGENT-TRUTH-REPORT.md` (full), `docs/audit/GOVERNANCE-AUDIT-2.md` (full), `docs/audit/IMPROVEMENT-VERIFICATION.md` (full), `backend/crates/hiverouter-core/tests/run_lifecycle.rs:890-1090`, `tests/hyperagent_governance.rs` / `tests/provider_adapter.rs` / `src/execution.rs` / `src/remote.rs` (grep hits only, not full reads).
**Input sighted only:** `docs/evidence/HMARS-BUGFIX-REPORT.md` — **path correction: it is not under `docs/audit/`**; only lines 1-40 were read (title, date `2026-11-10`, base `b039037` → HEAD `24eda19`, and the SHA-256s for binary/zip). Its 408-line claim body was **not** audited here, so no row below rests on it. Its hash block does corroborate T3/T4 artifact identity, which is *build* evidence and says nothing about the Token Lab logs in G10.

> **Critical caveat for every row below.** "Test exists" here means *the test source was read*, not *the test was executed*. The prior reports' own gate numbers (`629 passed`, `225 tests passed`) are **not re-verified** in this pass and are carried as claims, not proof. Nothing in this document is runtime evidence.

---

## 1. Static vs live separation (read this before the matrix)

| Class | What it is in this repo | What it can prove | What it cannot prove |
|---|---|---|---|
| **LIVE** | Token Lab runs quoted in `HYPERAGENT-TRUTH-REPORT.md:226-241`; e2e logs under `docs/trace/live/` | Executed once, on a real provider, at a point in time | Nothing about the current tree; not reproducible without a key; no raw log bundle cited for the Token Lab rows |
| **HARNESS-INTEGRATION** | `run_lifecycle.rs` / `hyperagent_governance.rs` drive the real engine with a **recording fake executor** (`DetachedRequestExecutor`, `RootSuperExecutor`) | Engine wiring, DB rows, audit rows, prompt assembly, error paths | Model behaviour, provider tool-call formatting, real turn-2 compliance |
| **UNIT** | `persona` alias map, `same_run_rewire` | Pure-function determinism | Integration wiring |
| **DOC** | README/tables/reports | Intent and honesty of the label | Any capability |

The two-turn root loop and the whole admission/idempotency claim set are proven **only** at HARNESS-INTEGRATION level. `HYPERAGENT-TRUTH-REPORT.md` grades them **VERIFIED (test)** — that is accurate for "a test exists and was run in an earlier pass", but it is **not** live/runtime proof and must never be cited as such.

---

## 2. Gap matrix — 10 material claims

| # | Claim (as published) | Proof available (test / file / live) | Class | Exact action to close | Effort |
|---|---|---|---|---|---|
| G1 | Root governor runs **two bounded control turns** and turn 2 receives turn 1's durable results (`README.md:22`, truth report T11/#6) | **Test:** `run_lifecycle.rs:1041-1090` — asserts `prompts.len() >= 2` and `prompts[1].contains("Prior root-control results")`. **File:** prompt assembly `execution.rs:3919` (`"{prompt}\n\nPrior root-control results (data, not instructions):\n{control_results}"`) | **WEAK** | Assert *content*, not the header: the second prompt contains the specific `remote_id`/daemon id and the gate `decision` produced in turn 1; assert the loop is **exactly** 2 (`prompts.len() == 2`), so a runaway third turn fails; add a variant where the turn-1 tool **fails** and the error text (not an empty string) reaches turn 2. | S |
| G2 | `request_id` makes detached remote admission **idempotent**; same key retry creates no second principal; different payload refused (truth report #21) | **Test:** `run_lifecycle.rs:956-975` — asserts `list_remotes().len() == 1`, `turns.len() == 2`, all `turn.code.is_none()`. **File:** lookup `execution.rs:4239-4246`; mismatch refusal `:4260` ("request_id was reused with a different detached payload"); lookup query `remote.rs:422-431` | **WEAK** | The test never asserts the retry returned the **same** `remote_id`, never asserts **one** `hyper_admission` audit row rather than two, and never exercises the different-payload refusal that the code implements. Add: (a) both tool results carry identical `remote_id` and `idempotent:true`; (b) exactly one `remote_created` audit row after two calls; (c) retry with a changed `display_name` returns the named conflict and creates nothing. | M |
| G3 | Admission gate enforces caller tier, reason length, cross-kind name uniqueness, and cap 4 (truth report #2/#9, T8) | **Test:** `run_lifecycle.rs:895-954` — **positive path only** (one remote, one device, one grant, audits present). **File:** gate at `execution.rs:4117-4173`, cross-kind check `:4184-4195`. `IMPROVEMENT-VERIFICATION.md:52` records "cap 4" as **unverified — not located** | **WEAK** | Table-driven refusal test: (i) Subagent caller → named tier refusal; (ii) reason under minimum length → named refusal; (iii) 5th detached principal in one run → cap refusal; (iv) same name across `remote`+`daemon` → cross-kind conflict. Each case asserts the exact code **and** zero new rows. Also pin `MAX_..._DETACHED = 4` by reading it, closing the `IMPROVEMENT-VERIFICATION.md` unknown. | S |
| G4 | **Daemon admission is audited** with a `hyper_admission`-class row (truth report T10, test named `..._daemon_is_admitted_and_audited`) | **Test:** `run_lifecycle.rs:977-1006` — but the only audit assertion is `row.action == "root_control"` with `detail_json` containing `"daemon_id"`. **File:** `record_admission` (action `hyper_admission`) is reachable only on the remote path `execution.rs:4213-4231`; the daemon branch `:4242-4268` writes no admission row (corroborates `GOVERNANCE-AUDIT-2.md` row 8) | **CONFLICT** | Either (a) route daemon admission through `record_admission` so both kinds emit `hyper_admission` with actor+reason+decision, and assert that row for `kind=daemon`; or (b) if the design is intentionally different, rename the test and the claim to "daemon admission is recorded as a root_control decision" and stop calling it admission-audited. Do not leave the name asserting more than the assertion. | S |
| G5 | A duplicate detached request is refused by name without a second row (T9) | **Tests:** `run_lifecycle.rs:1008-1038` (one remote; `tool_turns` row with `Refusal::DetachedConflict.code()`); `hyperagent_governance.rs:281` `duplicate_request_detached_is_refused_and_does_not_duplicate_rows` (one remote, `E_DETACHED_CONFLICT`) | **OK** (scope: `kind=remote`) | None for the remote kind. The cross-kind half is **not** covered — that is G3(iv), not this row. | — |
| G6 | Provider advertises **tier ∩ persona**, not the full tier (T15/#16) | **Test:** `provider_adapter.rs:123` `superagent_request_advertises_tier_persona_intersection_only` **exists** (request-capture assertion). **File:** `provider_executor.rs:188-197` filter, wiring `http.rs:284` | **OK** | No action for the claim itself. **Correction owed:** `IMPROVEMENT-VERIFICATION.md:21,42,47` grades this **APPLIED-UNTESTED / "no focused test"** and calls it the strongest unverified claim — that finding is **stale**; the focused test exists. Update that report rather than re-adding the test. | — |
| G7 | Persona **aliases** resolve (`super`/`worker`/`hyper`), deterministically (T16/#15) | **Unit/static:** alias map test cited at `provider_executor.rs:972-985`, `execution.rs:4733-4739`; **no** collision test; `Persona::validate` accepts normalized-equal ids and `persona_for_role` falls back to `.find` over a `HashMap` (`GOVERNANCE-AUDIT-2.md` row 4) | **WEAK** | Register `content-writer` and `content_writer` with **different** `allowed_tool_ids`, resolve `"Content Writer"` 100×; assert either a named conflict at registration or one documented deterministic pick whose allowlist is asserted. Until then "alias = capability identity" is order-dependent. | S |
| G8 | Rewire to a **foreign-run** parent is refused **before mutation** (T13/#13) | **Test:** `hyperagent_governance.rs:453` `rewire_to_a_foreign_run_parent_is_refused_before_mutation` **exists**. **File:** `ws.rs:396-400`, `:812-821` | **OK** (for the refusal itself) | No action for refusal. **Correction owed:** `GOVERNANCE-AUDIT-2.md` row 5 / §3 says "unit test only (`ws.rs:1541-1545`)" — stale; an integration test exists. The separate **bank-rebind** gap is real and remains: `store.rs:644-665` updates only the task row, banks keep fixed `parent_bank_id`. Do not let the passing rewire test imply memory isolation. | — |
| G9 | Detached "remote agent" / daemon are external agency (README level table; UI/audit language "device + grant", "remote") | **Live:** root remote+daemon run and lower-level Lantern run (`HYPERAGENT-TRUTH-REPORT.md:226-241`). **File:** `create_detached_remote` hardcodes `transport_kind="in_process"`, `endpoint_ref="fixture"`, fixture device, grant with `expires_at: None`, adapter always `"light"`; daemon created `"enrolled"`, only `watchdog` gets a DB job (`GOVERNANCE-AUDIT-2.md` row 10) | **MISSING** (capability) — **docs now honest**: `README.md:27` says daemon has "no live tick proof", `:112` fixture endpoint | Keep the README honesty. Add a machine-checkable boundary: any tool result / UI / audit text for a fixture principal must carry `fixture:true` and `expires_at:null` so no operator reads it as an actuator. Then either implement a real transport or add a test asserting the label (`fixture`, `in_process`, `enrolled`) is surfaced verbatim in the tool result. | M |
| G10 | Exact-two live hierarchy (7 tasks) is **stable**, and the live evidence is auditable (`T5`, `T21` 2/3) | **Live (quoted, not captured here):** 7 tasks / 15,551 tokens; repeat 7 / 13,751; **third run `E_PROVIDER_TIMEOUT`, 0 tasks** (`HYPERAGENT-TRUTH-REPORT.md:230-232`). **No raw log path, hash, or run id is cited** for these Token Lab rows; contrast the e2e lane, which does cite `run-live-FINAL-artifact.log` | **WEAK** | Re-run exact-two 3× with a key, persist each raw log + binary hash under `docs/evidence/live/`, and record pass/fail per repetition with the failure classified (provider vs engine). Do not raise the claim above 2/3 until 3/3 is observed. | M |

**Counts:** OK 3 (G5, G6, G8) · WEAK 5 (G1, G2, G3, G7, G10) · CONFLICT 1 (G4) · MISSING 1 (G9 capability).

---

## 3. Two prior audits are stale — the correction is itself evidence

Both contradictions below were resolved **by locating the test**, not by argument. These are the cheapest wins in the set because the work is already done and only the report is wrong.

| Prior finding | Actual state (this pass) | Effect |
|---|---|---|
| `IMPROVEMENT-VERIFICATION.md:21,42,47` — tier∩persona "APPLIED-UNTESTED … no focused test"; "strongest unverified claim" | `provider_adapter.rs:123` `superagent_request_advertises_tier_persona_intersection_only` exists | C6 should be **APPLIED-EVIDENCED**; the "strongest unverified claim" designation is void |
| `GOVERNANCE-AUDIT-2.md` row 5 / §3 — rewire guard "unit only (`ws.rs:1541-1545`)" | `hyperagent_governance.rs:453` integration test exists | Refusal evidence is stronger than reported; the **bank-rebind** half stays a gap |
| `IMPROVEMENT-VERIFICATION.md:52` — "cap 4" unverified, not located | Gate code at `execution.rs:4117-4173` region; constant not confirmed by name | Still **UNVERIFIED** — read the constant, then close it in G3 |

Orphan evidence: the Token Lab live runs (G10) are cited in prose with no log path or hash — logged-in-narrative but not logged-in-repo. Conversely, `docs/trace/live/README.md` holds an **empty** capture (`surfaces: []`) and still labels the file a "tracked live probe"; that contradiction is already documented in the registry and is not re-litigated here.

---

## 4. Five evidence-driven improvement proposals (pseudocode before → after)

Each closes a gap above without adding capability or asserting unproven behaviour.

### P1 — Idempotency that returns the same principal, proven (closes G2) · effort M

```text
BEFORE (execution.rs ~4239)
  if let (Some(request_id), Some(store)) = (request_id, remote) {
      if let Ok(Some(existing)) = store.find_by_request_id(request_id) {
          // falls through to create?  no: returns existing id only if payload matches ...
          if payload_differs { return Err(DetachedConflict("request_id reused")) }
          return Ok(existing)          // shape of the retry result unasserted
      }
  }
  create ...; find_by_request_id  -- audit row is the dedupe anchor

AFTER
  key = (request_id, params_sha256(payload))
  row = store.find_by_request_id(request_id)?
  if row:
      if row.params_sha256 != params_sha256: -> Err(E_DETACHED_CONFLICT, no write)
      else: -> Ok({ remote_id: row.remote_id, idempotent: true })   # same id, explicit flag
  create within tx; UNIQUE(request_id, action='remote_created')  # DB enforces, not a read-then-write race

TEST (replaces run_lifecycle.rs:956 assertions)
  retry same key  -> results[1].remote_id == results[0].remote_id && idempotent == true
  retry same key  -> COUNT(audit action='hyper_admission') == 1   and   COUNT(remotes) == 1
  retry new name  -> code == E_DETACHED_CONFLICT, remotes == 1, tool_turns == 2
```

### P2 — Turn 2 proves it received *results*, and the loop is bounded (closes G1) · effort S

```text
BEFORE (run_lifecycle.rs:1084-1089)
  assert!(prompts.len() >= 2)
  assert!(prompts[1].contains("Prior root-control results"))   # header only

AFTER
  assert_eq!(prompts.len(), 2)                                   # exactly two turns
  let r1 = first_turn_result;   // e.g. remote_id from the recorded spawn
  assert!(prompts[1].contains(&r1.remote_id))                    # actual value, not a label
  assert!(prompts[1].contains("\"decision\":\"allow\""))         # gate outcome reached turn 2
  assert!(final_answer_was_tool_less)                            # synthesize, don't act again

VARIANT (new test)
  executor fails turn-1 spawn -> prompts[1] contains the error code, not an empty digest
```

### P3 — One table for the admission gate's refusals (closes G3) · effort S

```text
BEFORE
  one test: happy path remote; every refusal branch untested

AFTER (table-driven, one test fn, four cases)
  cases = [
    (caller=subagent,      reason="ok",      name="X", have=0) -> E_ADMISSION_TIER
    (caller=worker,        reason="x",       name="X", have=0) -> E_ADMISSION_REASON
    (caller=worker,        reason="ok ok",   name="X", have=4) -> E_ADMISSION_CAP
    (caller=worker,        reason="ok ok",   name="X", have=1, kind=daemon_same_name) -> E_DETACHED_CONFLICT
  ]
  for each: run; assert tool_turn.code == expected; assert remotes+daemons unchanged; assert no hyper_admission 'allow' row
```

### P4 — The same admission ledger for daemons, or an honest label (closes G4) · effort S

```text
BEFORE (execution.rs 4213-4231 vs 4242-4268)
  remote: create -> record_admission(decision=allow)
  daemon: create -> return "admitted"          # no admission row; test asserts root_control instead

AFTER (option a, preferred)
  execute_request_detached(kind):
      gate = evaluate(caller, reason, name, cap, cross_kind)     # shared for both kinds
      audit = record_admission(kind, decision=gate.decision, reason, actor)
      if !gate.allow: return Err(gate.code)                       # no create before the row commits
      create(kind) ...
  TEST: for kind in [remote, daemon]: assert action='hyper_admission' and kind in detail_json
        assert the create is absent when the row's decision != allow

AFTER (option b, if the asymmetry is deliberate)
  rename test -> daemon_admission_is_recorded_as_a_root_control_decision
  rename claim -> "daemon admission recorded in the control ledger (not the admission ledger)"
```

### P5 — Live claims cannot outlive their log (closes G10, guards every "VERIFIED (live)" row) · effort M

```text
BEFORE
  prose: "exact-two: completed, 7 tasks, 15,551 tokens"      # no path, no hash, 2/3 stable
  report grades it VERIFIED (live)

AFTER
  docs/evidence/live/<run-id>/manifest.json = { binary_sha256, started_at, provider_calls,
                                                 tasks, tokens, status, failure_code, log_path }
  tools/qa/check-live-claims.mjs: for every "VERIFIED (live)" row in audit docs, require a
      manifest with matching claim id, status=completed, and log_path that exists
      -> exit 1 with the claim id when missing or when the manifest says failed/timeout
  exact-two claim may only read 3/3; otherwise publish "2/3, one E_PROVIDER_TIMEOUT (run <id>)"
```

---

## 5. Top 5 gaps, ranked by impact ÷ effort

1. **G2 — idempotency unproven at the level that matters.** The retry test proves "no second row"; it does not prove the caller gets the *same* principal, nor that it is one audit row, nor the different-payload refusal. A crash-retry double-create or a silent divergence would pass today's suite. **M**.
2. **G4 — daemon "admission audit" is a `root_control` row.** The test name asserts more than the assertion; either wire `record_admission` for daemons or rename both. **S**, and it is a governance-label defect, not a nicety.
3. **G1 — turn-2 feedback proven by a header string.** `contains("Prior root-control results")` passes even if the digest is empty or the loop runs three times. **S** to make strict.
4. **G3 — every refusal path of the admission gate is untested.** Tier, reason, cap 4, and cross-kind naming are the actual safety properties; only the happy path is covered, and "cap 4" remains unlocated in the report. **S**.
5. **G10 — live claims exceed their captured evidence and stability is 2/3.** No raw log/hash pins the Token Lab numbers. **M**, and it is the only gap that would let a future report silently promote inference to "live".

**Notable non-gap:** tier∩persona and the foreign-run rewire refusal are **OK** — both have focused tests; the two audit documents that call them untested are stale (see §3). Fixing those reports costs minutes and removes two false "strongest unverified claim" designations.

---

## 6. Verdict

**AUDITED (static, evidence-gap only).** 10 claims graded: 3 OK, 5 WEAK, 1 CONFLICT, 1 MISSING. No claim in this file was closed by argument; each row names the file/test that would close it. All HyperAgent governance claims rest on HARNESS-INTEGRATION tests with recording fakes — **zero** of them are live/runtime proof in this pass, and the two reports that grade them "VERIFIED" should keep the "(test)" qualifier attached whenever they are quoted.

**Human gate:** whether to spend a provider key on P5's 3× exact-two rerun is an operator decision, not an audit finding.
