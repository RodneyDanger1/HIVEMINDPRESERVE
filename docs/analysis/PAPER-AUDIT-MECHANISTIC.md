# Paper Audit — Mechanistic Claims vs. Current Code

**Auditor:** Paper Auditor 1/4 (read-only) · **Date:** 2026-11-10 · **Artifact:** this file only.
**Paper under audit:** *A Mechanistic Audit of a Capped Hierarchical Multi-Agent Framework* (local text, 689 lines). It is an **analytical reference, not peer-reviewed authority** — every statement sourced from it is labelled `paper-claim`, never fact.
**Code baseline:** working tree at `848735a` (`feat(ui): repair wiring…`). **The worktree is dirty** (`execution.rs`, `http.rs`, `db.rs`, `store.rs`, `tools/*`, tests modified), so line numbers below are the *current working tree*, and may not match a clean HEAD checkout. No code, tests, config, doc, commit, server, provider, or key was touched.

## 1. Claim-by-claim verification

### C1 — Two-turn root governor with turn-2 creation refusal
- **Paper claim (§2.1):** root control is exactly two turns; turn 2 may `finish`/one final call but may not create agents/remotes/daemons; refusal recorded via `record_root_gate_decision` at `execution.rs:3940–3952`.
- **Code:** `execution.rs:4342` (`for turn in 0..2`); `4333` (`MAX_ROOT_TOOL_CALLS = 4`); `4376–4393` (turn>0 refuses `spawn_superagent|spawn_remote_agent|spawn_daemon|convene_evolution_vote` and calls `record_root_gate_decision`); definition at `execution.rs:5314`. The paper's `3940–3952` citation is **stale line drift**, not a behavioural error.
- **Verdict: CONFIRMED** (behaviour; citation stale).
- **Improvement:** cite symbols, not line ranges, in analysis docs; enforce with a test that a turn-2 `convene_evolution_vote` is refused (the paper omits that turn 2 also loses *vote convening*, not just creation).

### C2 — Root cannot widen a planner-supplied topology
- **Paper claim (§3.3 dead paths):** “root cannot add a branch when the planner supplied one.”
- **Code:** `execution.rs:4328` (`has_planned_branch = plan.nodes.iter().any(|n| n.depth == 1)`); `4397–4407` refuses `spawn_superagent` when planned branches exist; MAX_ROOT_TOOL_CALLS=4 at `4333`.
- **Verdict: CONFIRMED.**
- **Improvement:** the paper's §2.1 turn-1 menu (`spawn_superagent`, `spawn_remote_agent`, `spawn_daemon`, `read_memory_summary`, `propose_improvement`, `finish`) is incomplete — code also exposes `assign_task` (`execution.rs:4440`) and `convene_evolution_vote` (`4799`). Correct the menu, or generate it from the match arms so it cannot drift.

### C3 — Deterministic detached admission
- **Paper claim (§2.2):** lower tiers call `request_detached{kind,display_name,reason}`; gate is deterministic (no LLM turn): tier ∈ {SuperAgent, WorkerAgent}; reason ≥ 12 chars; name unique across remotes **and** daemons; lower-level cap < 4; optional `request_id` + SHA-256 fingerprint gives replay vs. `E_DETACHED_CONFLICT`.
- **Code:** `execution.rs:5013–5144`: tier `5031–5036`; reason `.chars().count() < 12` `5061–5066`; kind `5102–5107`; fingerprint over `{kind,display_name,reason}` `5073–5081`; replay only when name+kind `"light"`+fingerprint match `5083–5095`, else conflict `5096–5099`; cross-remote/daemon uniqueness `5118–5129`; run-owner cap `< 4` `5139–5144`; admission row `5165`.
- **Verdict: CONFIRMED.**
- **Improvement:** the replay branch runs *before* `kind` validation (`5082` vs `5102`), so `kind:"daemon"` with a replayed remote `request_id` falls into the fingerprint-mismatch conflict rather than `kind must be remote or daemon`. Order the schema check first and pin it with a test; the current ordering is safe but yields a misleading named refusal.

### C4 — Five-seat proposal-only evolution
- **Paper claim (§2.3):** payload has `required_approvals:5, operator_approvals:1, superagent_approvals:4, unanimous:true, current_approvals:0, state:"awaiting_votes", can_apply:false`; four SuperAgents vote via `vote_evolution`; operator endpoint requires four **distinct** approvals and no rejection; apply appends one durable `evolution/LESSONS.jsonl` line; rollback removes it; scope is lesson artifact only.
- **Code:** payload exactly as quoted `execution.rs:4721–4729`, `applied:false`+`requires_human_review:true` `4714–4715`, truthful-scope block `4730–4740`. Voters spawned 4× `execution.rs:1625–1669`. Authorize: distinct-voter set `http.rs:2569–2591`, gate `2598`, authorization-only payload `2612–2621`, apply `2720+`, rollback `2856+`, path fixed under `data_dir/evolution/LESSONS.jsonl` `2693`.
- **Verdict: PARTIAL.** Two material gaps: (a) `authorize` accepts **≥4** distinct completed SuperAgents (`voters.len() < 4` at `http.rs:2598`) while the doc comment says “exactly four” (`2519–2520`); (b) `vote_evolution` validates tier, proposal existence and duplicate-voter only (`execution.rs:4915–4920, 4957–4982`) — it never checks the voter was one of the four voters convened for that proposal. So the enforceable property is “any four completed SuperAgents of the run”, not “the four designated seats”. That is a real weakening of the five-seat claim, and the paper states the stronger version.
- **Improvement:** see §2, Improvement 1.

### C5 — Memory partitioning and the learning boundary
- **Paper claim (§2.4):** `bank:run:<run>` / `bank:hive:<task>` / `bank:branch:<task>` partition; HyperAgent completion writes to the run bank, never an orphan branch bank; A2A inbox/roster excluded from inherited facts; authorized lessons injected as bounded labelled data (8 lines, 400 chars) into planner/root/task prompts; prompt-level carryover only, no weight update.
- **Code:** bank mapping `store.rs:263–265` and `1387–1394`; HyperAgent→run bank `store.rs:2746`; fail-closed identity for unscoped writes pinned by `tests/mem_isolation.rs:1–8` (“must fail closed to its own branch, never to the run bank”); lesson render `.take(8)` + 400 chars `execution.rs:63–76`; loader `take(64)` lines → 8 rendered, control-char scrub `78–83`, `89–106`; reload after apply/rollback `http.rs:2842, 2980`.
- **Verdict: PARTIAL.** Partition, fail-closed writes and 8×400 injection are **CONFIRMED**. The “A2A inbox/roster filtered out of inherited facts” sub-claim was **not verified** in this pass (no line read); the “live seeded-lesson run returned the lesson token” is a **paper-claim** backed by files named in `CODEBASE-CENSUS.md:1438–1449`, not re-executed here. Treat both as UNVERIFIED, not false.
- **Improvement:** add a test asserting the inherited-facts filter excludes `a2a:`/`roster` keys, and a boot test that the lesson block is byte-identical in planner, root and task prompts. Cheapest honesty gain: an evidence-bundle JSON per claim, not prose.

### C6 — Caps: “depth 3, fan-out 8, 64 agents, 64 provider calls, 40 tool calls/run, 12 tool turns, 32 agent messages, 2 A2A attempts”
- **Paper claim (§1.2):** those exact numbers, all named refusals, all pre-spend/pre-row.
- **Code:** `caps.rs:9` depth 3, `11` fan-out 8, `13` agents 64, `18` calls 64; `tool_loop.rs:14` turns 12; `execution.rs:142` messages 32, `146` A2A attempts 2; `check_spawn`/`check_calls` named refusals `caps.rs:52–74`.
- **Verdict: CONFIRMED for every cap except “40 tool calls/run” → CONTRADICTED.** There is no 40-tool-call constant in the tree; 40 is `MAX_PLAN_NODES` (`planner.rs:9–10`) and the comment at `caps.rs:15–18` explains why the provider-call cap is 64 rather than 40. The paper conflated the plan-node ceiling with a tool-call cap.
- **Improvement:** a single `CAPS` table in code (constant → contract code → enforcing function) rendered into docs, so a cap claim cannot be paraphrased wrongly. See Improvement 2.

### C7 — AGI distance / self-improvement coefficient zero
- **Paper claim (§2.5, §3.3):** system fails operational AGI criteria; “self-modification is deliberately not implemented”; `can_apply:false`; improvements are governed contextual updates, not a closure property.
- **Code:** no training/weight-mutation path exists in `hiverouter-core`; apply writes exactly one lesson line and reloads it (`http.rs:2720–2842`); the proposal payload keeps `can_apply:false` at creation (`execution.rs:4728`); apply/rollback are operator endpoints, not agent tools.
- **Verdict: CONFIRMED as a code fact** (“no weight update / no autonomous code mutation”), with the AGI-axis scoring itself **PAPER/INFERRED** — it is an external framework judgment, not testable against this repo. Do not promote the paper's “fails every criterion” into a repo claim.
- **Improvement:** none codeable; the honest move is to keep AGI language out of code comments and tests.

### C8 — Tool-surface table (HyperAgent `spawn_agent/report_result/finish`; SuperAgent shell/file/web; 12 tools, 5 permission classes)
- **Paper claim (§1.2):** per-tier effective tool surfaces, 12 registered tools, 5 permission classes.
- **Code:** `execution.rs:4293–4295` states root tools are **local to `root_checkpoint`**, never enter the global registry, and cannot be requested by leaf tasks — so the HyperAgent row's registry triple is not established by this path. 5 permission classes **CONFIRMED** (`tool.rs:24–66`: Read/Mutate/Execute/Network/Control). Tool count **UNVERIFIED** (registry is enum-derived; no count read).
- **Verdict: PARTIAL.**
- **Improvement:** replace the prose table with a test that renders the tier ∩ permission class matrix and fails if it changes; that is the only version of this claim that cannot rot.

## 2. Codeable improvements (recommended, not implemented)

### Improvement 1 — Bind evolution votes to the four designated seats
- **Files:** `backend/crates/hiverouter-core/src/execution.rs` (`spawn_evolution_voters` ~1625–1669: record `evolution_seat:{proposal}:{voter_task}` per voter; `execute_vote_evolution` ~4906–4982: require that seat key), `backend/crates/hiverouter-core/src/http.rs` (`authorize_evolution_proposal` ~2569–2598: count only designated seats; require exactly 4, zero rejections), test `backend/crates/hiverouter-core/tests/proposal_authorization.rs`.
- **Tests:** (a) non-convened SuperAgent `vote_evolution` → `E_TOOL_NOT_ALLOWED`; (b) 4 designated approvals → authorize OK; (c) 5th approval is refused or ignored, not silently accepted; (d) missing seat → `E_APPROVAL_REQUIRED`.
- **Honesty gain:** makes the paper's strongest governance claim true as written, rather than “any four SuperAgents”.
- **Rollback:** single revert; no schema migration (seat rows are ordinary `agent_memory` entries). Old proposals become un-authorizable after rollback-in, which is the correct fail-closed direction — record it in the changelog.

### Improvement 2 — One cap ledger rendered into docs and asserted by tests
- **Files:** `backend/crates/hiverouter-core/src/caps.rs` (add a `pub const CAP_LEDGER: &[CapEntry]` mapping constant → `Refusal` code → enforcing fn), unit test in `caps.rs`, doc render step in `docs/analysis/` only if regenerated (no product doc rewrite).
- **Tests:** table-driven: every entry has a distinct refusal code; a spawn above each numeric cap returns that code; plan-node 40 and tool-turn 12 are asserted separately from provider calls 64, so “40 tool calls” cannot reappear.
- **Honesty gain:** prevents the exact C6 defect (paper invented a 40-tool-call cap) and makes cap claims mechanically checkable.
- **Rollback:** revert; pure additive constant + test, no behaviour change.

## 3. Claim ledger

| ID | Claim | Grade | Basis |
|---|---|---|---|
| L1 | Root control is two turns; turn 2 cannot create principals | **CODE** | `execution.rs:4342,4376–4393` |
| L2 | Root cannot add a branch when planner supplied depth-1 nodes | **CODE** | `execution.rs:4328,4397–4407` |
| L3 | Detached admission is deterministic, audited, not an LLM turn | **CODE** | `execution.rs:5008–5165` |
| L4 | Five-seat block is exactly 5 = 4 SuperAgent + 1 operator, unanimous | **PAPER** on designation; **CODE** only for the static payload | `execution.rs:4721–4729`; `http.rs:2598` weaker |
| L5 | Authorize requires four *distinct designated* voters | **CONTRADICTED** as written | `http.rs:2598` is `>= 4`; `execution.rs:4915` has no seat check |
| L6 | Lesson injection is bounded 8 lines × 400 chars, labelled data | **CODE** | `execution.rs:63–76,89–106` |
| L7 | HyperAgent completion lands in the run bank, never an orphan branch | **CODE** | `store.rs:2746`; `mem_isolation.rs:1–8` |
| L8 | A2A inbox/roster excluded from inherited facts | **UNVERIFIED** | not read this pass |
| L9 | Live seeded-lesson / five-seat apply was observed end-to-end | **UNVERIFIED** (paper-claim) | evidence filenames only (`CODEBASE-CENSUS.md:1438–1449`) |
| L10 | Caps: depth 3, fan-out 8, 64 agents, 64 provider calls, 12 turns, 32 messages, 2 A2A | **CODE** | `caps.rs:9–18`; `tool_loop.rs:14`; `execution.rs:142,146` |
| L11 | “40 tool calls/run” cap | **CONTRADICTED** | 40 is `MAX_PLAN_NODES` (`planner.rs:9–10`) |
| L12 | 5 permission classes / 12 registered tools | **CODE** / **UNVERIFIED** | `tool.rs:24–66`; count not read |
| L13 | No weight update; self-modification not implemented | **CODE** | apply writes one JSONL line, reloads only (`http.rs:2720–2842`) |
| L14 | Fails operational AGI criteria | **PAPER/INFERRED** | external frameworks (Chojecki, Morris et al.), not repo-testable |
| L15 | Worktree equals HEAD | **CONTRADICTED** | `git status`: 20 modified paths |

## 4. Do not implement (explicit)

1. **No AGI scoring harness or benchmark claims** (GAIA/SWE-bench) in this repo — out of scope, unverifiable here.
2. **No weight updates, fine-tuning, or autonomous code apply** — the lesson-artifact boundary is the governance property; widening it destroys the audited surface.
3. **No LLM approval turn for detached admission** — would replace deterministic refusal with plausible refusal; contradicts the paper's own strongest result.
4. **No multi-provider “adversarial evaluator”** introduced to rescue voter independence — needs a second key/provider, outside this audit's authority.
5. **No cap relaxation** (more root turns, >4 voters, higher fan-out) to make demos pass.
6. **No network remote transport / daemon live-tick work** as part of an audit follow-up.
7. **No edits** to source, tests, config, plans, ledgers, or commits from this audit; this file is the only output.

## 5. Bottom line

The paper's three strongest mechanistic claims — two-turn root gate, deterministic detached admission, and bounded lesson injection — survive contact with the current code. Its **five-seat claim is overstated** (any four completed SuperAgents, not four designated seats) and its **cap list contains one invented number** (40 tool calls/run). Its AGI-distance argument is an external framework judgment, correctly labelled here as paper-level, not a repo fact. Fix the vote-seat binding and the cap ledger, and the paper's governance claims become mechanically true.
