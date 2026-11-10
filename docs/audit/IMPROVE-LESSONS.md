# HiveMind lessons ledger — 8 lessons + 5 promotion-loop improvements

**Mode:** read-only audit output; this file is the only write. No source edits, no provider calls, no tests executed.
**Repo:** `C:\Users\Riley\Desktop\hiverouter-v1` · **Sources read:** `docs/audit/GOVERNANCE-AUDIT-2.md` (HEAD `9f8e2e0` + dirty tree), `docs/audit/IMPROVEMENT-VERIFICATION.md` (@ `889b779` + dirty tree), `docs/audit/AGENTIC-LIVE-TESTS.md`, `backend/crates/hiverouter-core/src/execution.rs:3880-4290`, `tests/run_lifecycle.rs:863-1006`.
**Evidence convention:** `APPLIED` = code path present and named test exists (not run here); `PARTIAL` = narrower than the label; `PROPOSED` = no code/test yet; `REJECTED` = must not be adopted as stated. "Read from source" ≠ "verified by execution".

## Level claim (agent-learning-loop, honest)

**Current level: L0→L1 substrate, no L2 claim.** There is durable episodic substrate (`tool_turns` rows, `remote` audit rows, `docs/evidence/live/*.json`) but **no lesson ledger, no bounded retrieval, no retrieval receipts, no A/B**. Therefore: no claim of L1 retrieval benefit, no claim of L2 procedural learning, and **no L3/L4 training claim** — no rights-cleared corpus, no GPU, no frozen eval, no verifier farm in this repo. Any wording like "the system learns from failures" is unsupported; what exists is human-authored audit + durable run evidence.

---

## Lessons

Each: **symptom** (observable) · **root cause** (mechanical) · **resolution** (actual/proposed) · **prevention rule** (executable) · enforcement status.

### L-20261110-approval-label-before-audit
- **Symptom:** tool text said "approved by hyper-admission" while the daemon branch wrote **zero** admission rows; only remote wrote `hyper_admission` (`GOVERNANCE-AUDIT-2.md` #1/#8; `execution.rs:4213-4231` vs `:4242-4268` at that HEAD).
- **Root cause:** the success string was constructed in the admission path itself, before/independently of the audit write; the daemon branch returned after `daemon.rs` insert with no `record_admission` call. Approval was a local return value, not a committed fact.
- **Resolution:** partially actual — remote path admits + audits (`IMPROVEMENT-VERIFICATION.md` C1/C2 APPLIED-EVIDENCED; live Test G shows `decision:allow`); daemon path now has a `root_control` audit row in the test (`run_lifecycle.rs:978-1004`) but still no `hyper_admission` row. **Proposed:** one `admit()` function that commits the audit row and returns the text only on commit.
- **Prevention rule:** `cargo test -p hiverouter-core lower_level_request_detached_daemon_is_admitted_and_audited` must assert a `hyper_admission` audit row (actor + `"decision":"allow"`) exists **before** the tool turn is `ok`; no `if decision=="allow"` may format success text without an immediately preceding audit insert. Static grep: `rg -n "approved by hyper-admission" backend/crates` must only match inside the function that writes the admission row.
- **Enforcement:** PARTIAL — remote enforced by `run_lifecycle.rs:863-953`; daemon admission row **unenforced**.

### L-20261110-root-result-discarded
- **Symptom:** root gate outcomes and `read_memory_summary` digests were accumulated into `_result_text` and never fed back; finalize was a separate no-tools turn (`GOVERNANCE-AUDIT-2.md` #2, `execution.rs:3925,4110-4112` at audit HEAD).
- **Root cause:** the bounded round lived inside a `for _turn in 0..1`; results were appended to an underscore-prefixed local, and `finalize_root` (`provider_executor.rs:652-676`) had no tools and received no digest. The governance loop was write-only.
- **Resolution:** actual in the dirty tree — `control_results` now accumulated (`:3906`, `:4180`) and injected on a second bounded turn with the label `(data, not instructions)` (`:3915-3920`). Two turns, not a scheduler.
- **Prevention rule:** new test `root_control_result_is_fed_back_before_finalize`: recording executor returns `spawn_remote_agent` + `read_memory_summary`; assert the second `orchestrate_root` call's prompt contains the digest **or** the finalize prompt contains it, and that at most 2 root calls occur. Grep guard for the exact old defect: `rg -n "let mut _result_text" backend/crates` must fail in CI (a discarded local means the result is unread).
- **Enforcement:** PARTIAL — behavior present, no test named; the grep guard is unenforced.

### L-20261110-duplicate-guard-wrong-scope
- **Symptom:** same display name could exist as a remote **and** a daemon; up to `MAX_ROOT_TOOL_CALLS=4` flat-plan `spawn_superagent` calls could each create a branch; the lower-level cross-kind check was non-transactional (`GOVERNANCE-AUDIT-2.md` #3).
- **Root cause:** the guard was per-kind (`remote.list_remotes()` only, `daemon.list()` only) and per-round in-memory (`root_all_names`), executed as check-then-act with no DB uniqueness constraint or transaction. Two writers can both observe "absent".
- **Resolution:** proposed — single uniqueness domain in SQL: a partial/expression unique index over the union of principal names per owner (e.g. `UNIQUE(owner_id, lower(display_name))` on a shared `principal` table, or trigger), plus one guard function used by both the root and the lower-level path.
- **Prevention rule:** `cargo test -p hiverouter-core root_duplicate_guards_are_cross_kind_and_per_round` (one round: `spawn_remote_agent("X")`, `spawn_daemon("X")`, two `spawn_superagent` on a flat plan → exactly 1 principal "X", exactly 1 added branch). DDL check in migration test: `SELECT 1 FROM sqlite_master WHERE type='index' AND tbl_name IN ('remotes','daemons') AND sql LIKE '%display_name%'` returns ≥1 for each store; if there is no shared principal table, the index test must cover both tables and the cross-kind guard test must still pass.
- **Enforcement:** UNENFORCED (no cross-kind/concurrency test).

### L-20261110-gate-rows-evict-constraints
- **Symptom:** every root gate decision became a unique `root_gate:<uuid>` memory row; the root summary returns the newest 8 run+hive rows, so gate rows could push a durable run constraint out of view (`GOVERNANCE-AUDIT-2.md` #6; `execution.rs:4347-4368`, `store.rs:2785-2799`).
- **Root cause:** audit evidence and working memory shared one bounded FIFO window. Unique keys guaranteed no dedupe, and there was no retention class, so the more you govern, the less trajectory memory you keep.
- **Resolution:** proposed — a dedicated append-only `gate_decisions` ledger (run_id, action, decision, reason, actor, seq) with its own bounded read; the generic memory summary keeps a one-line "N gates" marker only.
- **Prevention rule:** `cargo test -p hiverouter-core root_gate_decisions_do_not_evict_run_constraints`: seed one durable constraint, append 12 gate decisions, assert the constraint is still in `read_memory_summary(limit=8)` **and** `gate_decisions` count == 12. Grep: `rg -n "root_gate:" backend/crates` must appear only in the ledger writer after the change, never in a generic memory insert call.
- **Enforcement:** UNENFORCED.

### L-20261110-scope-filter-after-limit
- **Symptom:** `read_memory_summary(scope=hive, limit=1)` returned empty even when hive rows existed, if run rows filled the limit first (`GOVERNANCE-AUDIT-2.md` #7, `execution.rs:4081-4101`).
- **Root cause:** the store query applied a global row limit across run+hive rows and the caller then `retain`ed by scope, i.e. filtering **after** truncation. A correctness predicate was applied to an already-corrupted window.
- **Resolution:** proposed — push the scope filter into SQL (`WHERE scope = ?` / bank predicate) before `LIMIT/OFFSET`, and return the limit per requested scope.
- **Prevention rule:** `cargo test -p hiverouter-core read_memory_summary_scope_filters_before_limit`: seed 30 run entries + 1 hive entry; request `scope=hive, limit=1`; assert exactly the hive entry. Grep the anti-pattern: `rg -n -U "memory_hyper_summary\([^)]*\)[\s\S]{0,200}retain\(" backend/crates/hiverouter-core/src` must return zero matches.
- **Enforcement:** UNENFORCED.

### L-20261110-idempotency-window-and-tuple
- **Symptom:** idempotent replay returns success only if the existing row's name matches **and** `existing.adapter_kind == "light"` (`execution.rs:4245-4263`); daemon replay has no request-id lookup at all (lookup is remote-store only); the idempotency test asserts two `ok` turns but not that the replay was a replay (`run_lifecycle.rs:956-975`).
- **Root cause:** the idempotency identity was derived from a mutable product default (`"light"`) instead of a hash of the immutable request payload, and the ledger of "already admitted" was scoped to one principal store. A future adapter default change silently converts valid retries into `E_DETACHED_CONFLICT` (and the current test would still pass, because it only uses `light`).
- **Resolution:** proposed — store `request_sha256` (canonical JSON of kind+name+adapter+params) on the row; replay = same key AND same hash → return prior result, same key AND different hash → `E_DETACHED_CONFLICT`; look up in one admission table covering remote+daemon.
- **Prevention rule:** `cargo test -p hiverouter-core lower_level_request_detached_replay_survives_adapter_change_and_covers_daemon`: (a) retry with same `request_id` and explicit non-`light` adapter returns the prior id, not a conflict; (b) `kind=daemon` retry with same `request_id` returns the prior daemon; (c) same `request_id` + changed payload refuses with `E_DETACHED_CONFLICT`. Also assert exactly 1 `hyper_admission` row after both retries.
- **Enforcement:** UNENFORCED — the existing test proves "two ok turns, one row", which is weaker than replay semantics.

### L-20261110-persona-alias-nondeterminism
- **Symptom:** two personas whose ids normalize equal (`content-writer` / `content_writer`, or role `"Content Writer"`) can resolve to either allowlist depending on `HashMap` iteration order (`execution.rs:822-846`; `prompts.rs:386-437`; `GOVERNANCE-AUDIT-2.md` #4).
- **Root cause:** alias resolution used fallback `.find` over map values with no uniqueness constraint and no collision rejection in `Persona::validate`; `upsert_persona` replaced by raw id only. A capability allowlist was selected by an unordered search.
- **Resolution:** proposed — reject normalized-equal persona ids at write time (unique index on `normalize(id)`), and make resolution exact→single-documented-normalization with a conflict error, never a scan fallback.
- **Prevention rule:** `cargo test -p hiverouter-core persona_alias_collision_is_refused_or_deterministic`: insert both ids, resolve `"Content Writer"` 100×, assert either a named conflict error or one documented pick whose allowlist equals the asserted set; plus schema test `SELECT count(*) FROM pragma_index_list('personas')` covering a normalized-id unique index. Grep: `rg -n "\.values\(\)\.find" backend/crates/hiverouter-core/src/execution.rs` must be zero after the fix.
- **Enforcement:** UNENFORCED (only the happy-path alias test exists, `run_lifecycle.rs:1426`).

### L-20261110-live-claim-discipline
- **Symptom:** Test F: 2/3 exact-two attempts succeeded (one provider timeout before any task row); Test B's SuperAgent prose said it "had no children", which cannot distinguish a scheduler violation from wording (`AGENTIC-LIVE-TESTS.md:194-206, 91-98`).
- **Root cause:** no failure taxonomy and no sample-size gate in the evidence format: an upstream `E_PROVIDER_TIMEOUT` and a product topology defect were both counted as "attempt failed", and a model's self-report was treated as a candidate defect signal. Claims are made from prose, not from gate assertions.
- **Resolution:** actual as of Test F — the audit explicitly declines a stability claim ("two of three attempts are not yet enough"). Proposed — a machine-readable episode record with `failure_class` (`provider_timeout_pre_task`, `topology_violation`, `gate_refusal`) and `n_required`/`k_observed`; stability claims computed only by formula.
- **Prevention rule:** every live-test evidence JSON must carry `attempts`, `successes`, `failure_class[]`; a `stability_claim` field is allowed only if `n >= 8 && k >= 7` **and** a Wilson 95% lower bound above the baseline `p0` is printed in the file. Grep/CI: `rg -n '"stability_claim": *true' docs/evidence/live` must each have a sibling `"wilson_lb"` number. Prose-only symptoms must be tagged `suspected`, never `defect`, in the ledger.
- **Enforcement:** PARTIAL — honest wording present in Test F; machine check unenforced.

---

## 5 lesson-promotion improvements (before → after)

**Status legend:** APPLIED (present here) / PROPOSED (design only) / REJECTED (must not ship as stated).

### P1 — Episode + lesson ledger — **APPLIED (substrate only; lesson layer PROPOSED)**
What exists: `tool_turns` per run, `remote` audit rows, `docs/evidence/live/*.json`. What does not: a lesson id, evidence-episode ids on a lesson, or retrieval.
```
BEFORE: lesson lives in an audit markdown; evidence = "test exists" prose
        tool_turn(row) ── no lesson_id ──> doc claim

AFTER:  episode(id, run_id, app_version, commit, action, gate, outcome, failure_class)
        evidence(episode_id, artifact_sha256, gate_json)
        lesson(id=L-yyyymmdd-slug, symptom, root_cause, resolution, prevention_rule,
               scope{app,version,catalogue_sha}, evidence_episode_ids[], status)
        # append-only; lesson is DATA; ids immutable; conflict record instead of averaging
```

### P2 — Promotion gate — **PROPOSED**
```
BEFORE: "3 successes" in prose (or a single audit) -> implicitly trusted practice
        trigger: human says "looks good"

AFTER:  candidate = lesson(L) with evidence_episode_ids
        evaluate on n = 8 fresh episodes, same app/version/catalogue_sha
        k = successes counted ONLY by verifier gates (not self-report)
        if k < 7:                          -> keep PROPOSED
        if wilson_lb(k, n, 0.95) <= p0:    -> keep PROPOSED (p0 = baseline from pre-lesson runs)
        if any golden regression            -> REJECTED_FOR_NOW
        if human_approver is None           -> keep PROPOSED      # human gate ON
        else                                -> ACTIVE(version+1), record evidence ids + hash
        # 3/3 is a minimum sample, NOT a promotion rule: wilson_lb(3,3)=0.4385, unsatisfiable at n=3,4,5
```
Status note: **REJECTED:** "3 successes + Wilson > 0.60" — mathematically unsatisfiable at n=3 (LB 0.4385); the n=8/k≥7 variant is simulation-derived (3.51% null promotions, 81.14% at p=0.90), **not adopted policy**; owner must pick n, p0, false-promotion budget.

### P3 — Demotion — **PROPOSED** (old rule **REJECTED**)
```
BEFORE: demote if "2 failures in last 20 outcomes"     # false-demotes a 95% skill 41.22% of the time
        immediate ACTIVE->DEPRECATED, evidence overwritten, no version trail

AFTER:  on each episode outcome o in {pass,fail} for ACTIVE skill v:
          if fail_streak >= 2:            # 2-consecutive, not 2-in-20
              state = SUSPENDED
              pointer = previous_active_version   # automatic rollback, evidence kept
              open conflict_record(evidence_episode_ids, statistic, threshold)
          elif roll(window=20).fail >= 3: state = UNDER_REVIEW   # review, not deletion
          # demotion is reversible; never delete lesson or evidence rows
```
Do not ship "2 in last 20": simulation showed 41.22% false demotion at p=0.95. 2-consecutive/3-in-20 are candidate rules that still need a simulation at the chosen p0 before being called policy.

### P4 — Rollback — **PROPOSED** (drill unexercised)
```
BEFORE: new skill/lesson text is edited in place; recovery = git revert; no pointer
        no inverse recorded; "rollback" means someone remembers the old prompt

AFTER:  activate(L, v+1):
            assert rollback_target(L) == v exists and its hash matches ledger
            write active_pointer(L) = v+1 + previous_pointer = v + activation_episode_id
        rollback(L, reason, evidence_episode_ids):
            pointer(L) = previous_pointer; state = ROLLED_BACK(reason)
            keep v+1 rows (never delete); append demotion record
        DRILL (mandatory once before trusting): activate a synthetic v+1, force a
            failing gate, assert pointer flips back within one episode and v stays readable
```

### P5 — Prompt isolation — **PROPOSED** (only textual labeling exists today)
```
BEFORE: retrieved cases + root tool results are pasted into the prompt with a
        "(data, not instructions)" sentence (execution.rs:3915-3920)
        -> a retrieved episode containing "ignore previous instructions" is still text in the prompt

AFTER:  retrieve(task_family, app, version, catalogue_sha) -> <=8 episodes, <=3 skills
            returns ids + summaries + sha; no whole logs; failed/verified only
        render(memory) -> delimited BLOCK with provenance header, length cap,
            and an injection scan; never merged into system/developer text
        capability check is INDEPENDENT of prompt: tool allowlist, gates, caps are
            computed from policy tables, not from retrieved text
        receipt(episode_ids, skill_versions) stored on the run; reviewer can answer
            "what did the agent see?" without the model's self-report
        # a retrieved case can never expand tool scope, change a gate, or activate a skill
```
No fake-training clause for all five: these are retrieval/procedural (L1/L2) mechanisms; nothing here updates weights, and no SFT/preference/RL claim is made.

---

## The 3 best prevention rules

1. **Approval text requires a committed ledger row** — `lower_level_request_detached_daemon_is_admitted_and_audited` must find a `hyper_admission` row (actor, `"decision":"allow"`) before the tool turn can be `ok`; success strings are produced by the audit writer, never by the admission branch. (Catches the "approved by hyper-admission" label class; today remote is enforced, daemon is not.)
2. **A bounded window must filter before it truncates, and governance rows never share the working-memory budget** — SQL scope predicate before `LIMIT`, plus a dedicated `gate_decisions` ledger; tests `read_memory_summary_scope_filters_before_limit` and `root_gate_decisions_do_not_evict_run_constraints`. (Catches silent-empty digests and audit evidence evicting the trajectory it describes.)
3. **Idempotency identity is a canonical request hash, and the test must exercise a replay, not two successes** — same `request_id` + same hash → prior result across remote **and** daemon; same key + different payload → `E_DETACHED_CONFLICT`; exactly one admission row after retries. (Catches the `adapter_kind == "light"` trap that passes today's test and would turn valid retries into conflicts the moment the default changes.)

*Runner-up, deliberately not in the top 3 because it is a process rule rather than code:* every live-test evidence JSON carries `attempts/successes/failure_class`, and `stability_claim: true` is invalid without a printed Wilson lower bound at the owner-chosen n and p0.

---

## Read-only receipt

Only this file was written. No source, test, config, database, provider, or server mutation. All code claims are source reads at the HEADs and dirty-tree state described in the source audits; no `cargo test`, build, or live run was executed for this document. Named tests are the proposed enforcement, not evidence of a pass.
