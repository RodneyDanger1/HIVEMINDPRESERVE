# 25 — Provider/cache/cost implementation audit (wave 3, seat 25)

**File:** `docs/research/hmars/25-provider-cache-implementation.md`
**Author:** wave-3 seat 25 (provider/cache/cost implementation audit)
**Date:** 2026-11-07 (America/Chicago), probes at ≈21:53–21:56.
**Base:** `bd575ba` ("feat: real SuperAgent/Subagent personas and persona prompt execution"); tree is **dirty**.
**Write scope:** this file only. No product edit, no restart, no paid provider call, no credential read, no key printed.
**Skills:** `/deep-research` loaded and its evidence discipline applied. `/deep-research`'s independent
source-verification pass could not be run as a separate agent in this seat; external provider facts are therefore
inherited **READ/DOCUMENTED** from `16-deepseek-cache-cost.md` and `01-deepseek-api.md` (fetched 2026-11-07 by
those lanes). The two live endpoints the Lead authorised were re-measured in this pass (§0). Everything else is
code read at `path:line` or explicitly DESIGN/UNVERIFIED.

**Claim classes**

| Tag | Meaning |
|---|---|
| **VERIFIED** | Exact `path:line` read in this pass, or an HTTP response observed in this pass. |
| **DOCUMENTED** | Official DeepSeek fact; read in `16`/`01` (fetched 2026-11-07), not re-fetched here. |
| **READ** | Read in a first-wave/second-wave report; not independently re-measured. |
| **DESIGN** | Recommendation of this seat; not implemented; not a claim about current behaviour. |
| **UNVERIFIED** | Not proven. Never use it as a budget/cache fact. |

Documentation pin inherited from `16`: `version_pinned: false`; newest documented change 2026-10-17
(`https://api-docs.deepseek.com/updates/`). All provider facts below will rot; the re-fetch list is `16` §13.3.

---

## 0. Measured runtime and dirty-tree snapshot (read-only)

### 0.1 Live endpoints (authorised read-only probes; no key material returned or recorded)

| Probe | Observed | Label |
|---|---|---|
| `GET /health` | `200` `{"status":"ok","version":"1.2.0-dev","schema":"hiverouter.v1","schema_version":10,…,"uptime_secs":542,"checks":[rustc ok,node ok,deepseek_api_key present (value never logged)],"database":{"schema_version":10},"transport":{"graph_nodes":35,"allowed_origins":2},"prompt_assets":{"loaded":true}}` | VERIFIED |
| `GET /api/provider/models` | `200`, `cache-control: no-store`, `object:"list"`, 2 models: `deepseek-flash` (`DeepSeek-V4.1-Flash`, `context_window:1048576`, `max_output_tokens:393216`) and `deepseek-v4-pro` (`DeepSeek-V4-Pro`, same limits) | VERIFIED |

Consequences: (a) live model metadata exists and is finer than `16`'s `MODEL_CONTEXT_ASSUMED=1_000_000`
(`16` §5.1); use `1_048_576` when reported, with the documented 1M form as a conservative fallback. (b) The
server is up and serving; `graph_nodes:35` also shows the global-snapshot defect from `10-gap-meta.md`
(X9) is live, but that is not provider/cache scope. (c) The process uptime (542 s) shows someone else restarted
the server between lanes; **this seat did not restart it**.

### 0.2 Dirty tree (as read this pass; uncommitted, unverified by tests)

`git status` shows source changes in:

| File | Diff (read) | Effect on this audit |
|---|---|---|
| `backend/crates/hiverouter-core/src/provider_executor.rs` | `system_prompt_for` normalises role (`trim().to_ascii_lowercase()`, `-`/space → `_`, fallback) and `continue_with_tools` now uses `self.system_prompt_for(task)` instead of `TASK_SYSTEM` | **Fixes `16`'s P2/P3-class persona findings**; persona reaches tool turns in the working tree. Still no `worker` persona, planner still uses `PLANNER_SYSTEM`, and the persona-map lookup is per-call rather than canonicalised in a single prefix builder. |
| `backend/crates/hiverouter-core/src/http.rs` | **Changed during this pass (concurrent lane): +63 lines adding a `host_guard` middleware and `host_allowed`/`loopback_host` checks** (current `:533-589`), applied to the whole router | Host/DNS-rebinding P0 gap is now being closed in the dirty tree, but the change is uncommitted/unverified here; **all `http.rs` handler line anchors below are as-read before this insertion — current tree shifts handlers by ≈+61** (route list now `:514-531`, `run_usage` `:903`, provider models `:942`, provider balance `:965`, `provider_origin_allowed` `:987`). |
| `backend/crates/hiverouter-provider/src/client.rs` | **Changed during this pass (concurrent lane): 402 gets a balance-specific message instead of the auth message; 400/422 are now mapped to a request-shape `ProviderInvalidResponse`** (current `:507-534`) | **Code identity is still `Refusal::ProviderAuth` for 402** — the machine-readable `E_PROVIDER_BALANCE` gap remains; 400/422 mapping improves `16`'s §5.3 goal. Uncommitted/unverified here. |
| `backend/crates/hiverouter-core/src/execution.rs` | tool-cap failure now returns `Refusal::CallLimit` (was `BudgetExceeded`) | **Makes `16`'s §7 arithmetic and `10` X10 stale**: the tool cap no longer spends `E_BUDGET_EXCEEDED`. |
| `backend/crates/hiverouter-core/src/tool_loop.rs` | `MAX_TOOL_TURNS: 4 → 12`, `MAX_CALLS_PER_RUN: 16 → 40`, cap refusals `BudgetExceeded → CallLimit` | **`16`'s "4 tool turns/task" and its §7.2 structural envelope are stale.** Current dirty limits: 12 turns, 40 calls, 8 approval waits, 64 KiB result. |
| `backend/crates/hiverouter-core/tests/{http_boot,prompt_assets,prompt_assets_wiring,tool_loop_engine,tool_loop_matrix}.rs` | cap/persona test updates | Test evidence must be re-run before trusting the dirty tree (not run here). |
| `web/src/{panels/ApprovalOverlay.tsx,state/store.ts}` | approval UX changes | Not provider scope; no effect on cache/cost. |
| `tools/desktop-launcher/HiveMindLauncher.cs` | workspace-root fix | Not provider scope. |

**What the dirty tree still does not contain:** no tokenizer asset or crate, no prefix/message builder, no
per-call cost ledger, no cost/budget HTTP routes, no budget mode/governor, no context-window preflight, no
402→balance mapping, no planner-usage capture, no per-call `CallCost` receipt to the engine, no new refusal
codes, and **no schema migration beyond v10** (live `/health` confirms v10). `16`'s §6/§7/§8 designs are
therefore still DESIGN, not implemented. The new Host guard closes a security gap, not a cache/cost gap.

> **Tree-drift warning:** `http.rs` and `client.rs` were modified by other lanes while this audit was being
> written (Host guard; 402 message/400/422 mapping). Route/handler/code references in this file are marked with
> their read-state line numbers; when coding, re-run the anchor grep against the then-current tree. The dirty tree
> is unverified by this seat.

---

## 1. Executive answer (≤20 lines)

`16` is accurate on the external DeepSeek facts it re-fetched and on most of the byte-level cache defects, but it
is **not implementation-ready**, and the dirty tree changes several of its premises:

1. **Planner usage is dropped entirely.** `plan()` uses `complete()` then discards `Completion.usage`; the engine
   only adds leaf/continuation usage (`execution.rs:3016`). `run_usage` therefore undercounts every run, before
   any ledger discussion.
2. **No `CallCost` reaches the engine.** Both non-streaming (`complete_with_budget`) and streaming paths record
   cost inside the provider's `RunBudget`; the executor discards the returned `CallCost` (`provider_executor.rs:234`)
   and the client returns only `Completion`/`StreamHandle`. `16`'s S14 "persist the `CallCost` returned by
   `record_call`" has no transport path as written.
3. **`16`'s `provider_calls.task_id NOT NULL` is impossible for the planner call**: the planning call runs before
   the root task is created (`execution.rs:1314-1327`). The schema must allow a null task and carry
   `call_kind="planner"`.
4. **Migration ownership collides across reports**: `16` wants v11 `provider_calls`/v12 `request_charges`; `17`
   wants v11 `cost_ledger`/v12 `hive_budget`; `04` wants v11 `memory_banks`; `15` wants v12
   `provider_calls`/settings/watchdog. Only one lane can own each version, and `16`/`17` describe **overlapping
   ledgers with different schemas and different `hive_id` definitions**. A single provider-ledger schema must be
   agreed before coding.
5. **`16` and `17` contradict each other on the default budget and the estimator**: `16` defaults to Mode A (no
   money cap) and retires `bytes/4` for a tokenizer; `17` defaults to `soft`, keeps the current hard caps during
   the security wave, and proposes `bytes/3` with no new dependency. Under current structural caps, Mode A's
   worst case is roughly **$57/run at Pro peak** for 40 full-context calls (`§6.4` arithmetic), so Mode A is not
   denial-of-wallet-safe without the account floor + 402 freeze + daily guard + kill switch all implemented.
6. **The dirty tree makes `16` stale on tool caps** (12 turns / 40 calls; `E_CALL_LIMIT`) and **partially fixes
   persona continuation**; `16`'s `P2/P3` cost/prefix defects remain.
7. **No tokenizer artifact exists in the repository** (verified by name search), and `tokenizers` is not a
   provider dependency; `16`'s measured artifact/hashes are not reproducible from the tree. Vendoring, licence,
   crate pinning, and the chat-template renderer are open blockers.
8. **Live `/models` resolves `16`'s A11**: context is reported as **1,048,576** (not 1,000,000), max output
   **393,216**; use the reported value when present, the documented 1M as conservative fallback.
9. **Search/Fetch should stay out of v1**; Files should be deferred as vision transport only; Vision deferred
   with an explicit revisit trigger. This matches the operator's "yes only for vision/image" posture without
   pretending vision is in v1.
10. **Recommendation:** freeze the provider-ledger schema + migration number with the Lead, land the planner
    usage/receipt path and a single canonical prefix builder first, then implement tokenizer + context preflight +
    projected-cost reservation behind a `budget_mode=soft` default that preserves current caps until the P0 gates
    (kill switch, 402 freeze, balance floor, ledger) are green.

---

## 2. Audit of `16` against the current tree (claims → corrections)

Legend: **C** = `16` claim; **T** = current truth; **V** = verdict/correction.

| # | `16` claim (source) | Current truth (this pass) | Verdict |
|---|---|---|---|
| 1 | Cache hits require a full match of a persisted prefix unit; units persist at request boundaries, common prefixes, fixed intervals (`16` §1.1 C1–C7, DOCUMENTED) | Not re-fetched here; no code contradicts. Current code has no cache-key logic at all (no `user_id`, no `prompt_cache_key`, no cache controls: `ChatRequest` fields at `dto.rs:396-422`) | **Accepted as DESIGN premise; DOCUMENTED inherited. Re-fetch before coding.** |
| 2 | HiveMind has a stable system block + tier tool array but UUID fact keys, UUID-sorted facts, re-serialised replay args, empty `reasoning_content`, per-task memory reads break reuse (`16` §2.5 P1–P5) | `context.rs:100-110` prefixes facts with `parent:<TaskId>:`; facts are `BTreeMap` (`context.rs:24`) and appended after the run prompt by `prompt_with_context` (`execution.rs:53-58`, used at `:2755`); replay `serde_json::to_string(&plan.args)` (`provider_executor.rs:384-392`); `assistant.with_reasoning_content(turn.reasoning.clone())` always set (`:402`); memory read per task (`execution.rs:880-884`) | **CONFIRMED (VERIFIED).** |
| 3 | Planner prompt flattens template with `Debug` labels (`16` §2.3) | `execution.rs:1552-1565`: `format!("{:?}: {}", message.role, message.content)` joined into the plan user prompt | **CONFIRMED.** |
| 4 | `bytes/4` estimator measures the JSON body; `16` measured error table (`16` §4.1–4.2) | `provider_executor.rs:436-442`: `serde_json::to_vec(request)` len/4, used by `complete`/`stream_reply` (`:100-103`, `:136-139`); planner and leaf both | **CONFIRMED.** |
| 5 | No per-call `CallCost` persistence; executor drops the returned value (`16` §6.1, §9.3 S14) | `stream_reply` calls `budget.record_call(...).map_err(...)?` and discards the `Ok(CallCost)` (`provider_executor.rs:233-243`); client `complete_with_budget` returns `Completion { response, usage, attempts, http_status }` and no cost (`client.rs:361-401`); `stream_with_budget` returns only `StreamHandle` (`:553-560`) | **CONFIRMED, and stronger than `16`: no transport path exists.** |
| 6 | `run_usage` has six token integers only; no money/hive/peak/completeness (`16` §6.1) | `db.rs:198-217` v8 table: run_id + six token columns + created_at; live schema v10; `store.rs:678-730`; `execution.rs:698-719` | **CONFIRMED.** |
| 7 | `/api/usage/runs/{id}` prints digits for an INCOMPLETE run (`16` §6.1, §1 Q12) | `http.rs:903-936` (pre-guard `:842-875`) returns `usage` row values with no status; `run_usage` is persisted at terminal dispatch (`execution.rs:1305`) after a run may have failed; failed-planner/partial runs can have a row of successful-leaf tokens | **CONFIRMED (VERIFIED path); the exact INCOMPLETE case needs a fixture test.** |
| 8 | `Usage::checked_add` maps `(Some,None)→Some` while its doc says absent stays absent (`16` §6.1) | `backend/crates/hiverouter-contract/src/usage.rs:59-77`: `(Some(a), None) | (None, Some(a)) => Some(a)`; doc comment at `:52-55` says "keeping absent values absent unless both sides carry a value" | **CONFIRMED: code and doc conflict. For billing aggregation this is unsafe; ledger per-call rows are the fix; also fix the comment or the implementation.** |
| 9 | No hive dimension; `16` defines `hive_id` (`16` §6.3) | No `hive_id` in `runs`/`tasks`/`run_usage` (`db.rs:32-249`); `17` defines `hive_id = projects.id` (§4.7), which conflicts | **CONFIRMED gap; definition conflict between reports.** |
| 10 | `16`'s `provider_calls` schema: `task_id TEXT NOT NULL`, `hive_id NOT NULL`, `amount_nano_usd`, `run_usage` gains status (`16` §6.2) | Planner call occurs before root task exists (`execution.rs:1314-1327`); no task id available; retries have no usage; `request_charges` table is a second ledger | **CORRECTION NEEDED: nullable `task_id` + `call_kind`, attempt-status rows, and merged ledger.** |
| 11 | `16` migration v11/v12 (`16` §6.2, `17` §4.7, `04` §5, `15` §6.3) | Current DB is v10 (live `/health`); reports disagree: `16` v11 provider_calls/v12 request_charges; `17` v11 cost_ledger/v12 hive_budget; `04` v11 memory_banks; `15` v12 provider_calls/settings/watchdog | **BLOCKING CROSS-LANE CONFLICT.** Needs one allocation table (§5.1). |
| 12 | `16` Mode A default = no money cap; Mode B soft; Mode C hard caps with reservation (`16` §7.2) | `budget.rs:24-43` hard $0.50/400K/40 calls; `preflight` checks `spent >= cap` only (`:235-284`), attempts cap, token projection; no reservation; no mode; `RunBudget::new(BudgetLimits::default())` constructed inside the adapter (`provider_executor.rs:104-107`, `:141-143`) so no config path to change mode | **DESIGN vs code; and `16` default conflicts with `17` default.** |
| 13 | `16` says 4 tool turns/task in structural envelope (`16` §7.2) | Dirty tree `tool_loop.rs:14-16` = **12 turns / 40 calls**; refusal now `CallLimit` (`execution.rs:2843-2857`) | **STALE in `16`; update arithmetic/tests.** |
| 14 | `16` says `E_BUDGET_EXCEEDED` is the tool-cap code today (X10 in `10`) | Dirty tree raises `Refusal::CallLimit` for tool caps; not committed/tested | **STALE at HEAD, true in tree-with-changes; fix must land with tests.** |
| 15 | `16` lists 6 new refusal codes (`E_CONTEXT_WINDOW_EXCEEDED`, `E_SPEND_GUARD`, `E_SPEND_HALTED`, `E_PROVIDER_BALANCE`, `E_REQUEST_BUDGET_EXCEEDED`, `E_TOKENIZER_UNAVAILABLE`) | Contract has 65 codes; `refusal.rs` has `E_BUDGET_EXCEEDED`, `E_TOKEN_BUDGET_EXCEEDED`, `E_CALL_LIMIT`, `E_USAGE_INCOMPLETE`, `E_PRICE_UNSOURCED`; none of the six new codes exist; web `KNOWN_REFUSAL_CODES` is mirrored/frozen (`web/src/api/contract.ts:426`). Dirty tree now maps 402 to a balance-specific message but **still refuses with `E_PROVIDER_AUTH`**, and 400/422 to request-shape refusal (`client.rs:507-534` current) | **CONTRACT PATCH REQUIRED: refusal table + web mirror + matrix/golden tests; `E_PROVIDER_BALANCE` still needed for machine-readable identity; update `error_matrix` expectations for the new 400/422 branch.** |
| 16 | `16` §5 context preflight assumes `MODEL_CONTEXT_ASSUMED=1_000_000`; A11 open | Live `/models` reports `context_window:1048576`, `max_output_tokens:393216`; `ModelInfo` parses those fields (`dto.rs:890-909`) but nothing consumes them | **A11 RESOLVED by live probe; code must use reported metadata with 1M fallback; no preflight exists.** |
| 17 | `16` §4.2 tokenizer measured externally; `16` §4.3 plan vendors `tokenizer.json` + `tokenizers` crate | No tokenizer files in repo (name search); `hiverouter-provider/Cargo.toml` has no `tokenizers`/`sha2` dep; artifact hashes cannot be reproduced from tree | **PLAN SOUND, but acquisition/licence/crate pin/move-to-core are unresolved.** |
| 18 | `16` §6.4 `prefix_fp = sha256(model‖tools_sha‖L1‖L2‖L3‖L4)` | `hiverouter-provider` has no hash crate (Cargo.toml deps); `hiverouter_tools::sha256_hex` exists; no builder exists; formula omits `user_id` and the transcript/previous prefix, so continuation calls share one fp | **CORRECTION NEEDED: fp must hash the exact bytes that can be cached (model, user_id policy, tools, system, first user message, and for continuations the prior transcript boundary); compute in core, pass to provider budget/ledger.** |
| 19 | `16` §7.2 balance floor = 5 % of boot balance; 402 freeze; kill switch persisted in `app_meta`; `E_SPEND_HALTED` | No code for any of these; balance is read-only via provider route (`http.rs:965-986`, pre-guard `:903-924`); no budget state table; `app_meta` exists (`db.rs:36-51`) | **DESIGN ONLY; no seams implemented.** |
| 20 | `16` §8 Search/Fetch `$0.75/1k`; Files image-only; Vision Flash-only/1024 tok | `PermissionClass::Network` exists unused (`tool.rs:30-49`); no network tool ids (`tool.rs:134-142`); `ChatMessage.content: Option<String>` (`dto.rs:88-106`); no files/image DTO; `01`/`16` agree | **CONFIRMED; decisions in §7.** |
| 21 | `16` claims live cache-hit rate unverified and says no paid call was made | Still no paid call; live `/health`/`/models` only; no `usage` row inspected | **CONFIRMED UNVERIFIED.** |
| 22 | `16` §14.1 "Done" includes tokenizer acquired/hashed/measured | In this tree the artifact is absent; only the report's hashes exist | **READ (external pass), not reproducible from repo; must be re-acquired/vendored.** |
| 23 | `16`'s cost arithmetic uses `06`'s $1.07/run one-call overshoot | Current cap arithmetic is unchanged ($0.50/400K/40 attempts), but dirty 12-turn cap changes worst-case call composition, not the cap | **Accept $1.07 one-call overshoot; revise model-token arithmetic in §6.4.** |

---

## 3. Canonical prefix order for HiveMind (implementation-ready)

`16` §3.2 defines the layers; this section pins the **bytes**, the **message split**, the **hash**, and the
**phase plan**, and records exactly what today's code does instead.

### 3.1 The order (head → tail)

```
L0  tools[]                    top-level request field, tier-stable, canonical JSON, hashed
L1  GLOBAL                     immutable hive-wide identity + safety + output discipline
L2  PERSONA                    persona/template id, instructions, tool policy, refusal policy, output schema
L3  MEMORY SNAPSHOT            frozen, content-addressed, semantic-keyed, bounded (empty until memory v11)
----- message boundary: ONE system message containing L1..L3 (in that order) -----
L4  RUN CONTEXT                run prompt, byte-identical for every task of the run
L5  TASK ASSIGNMENT            role · depth · task instruction (stable for that task)
L6  TRANSCRIPT                 assistant/tool messages, raw provider bytes, append-only
L7  TAIL DIRECTIVE             "Return the final answer for this task only." — last bytes of the first user message
```

Hard rules (DESIGN; motivated by C2/C4 and the current defects):

- **R1** One `system` message holding L1–L3, then one `user` message holding L4–L5–L7, then transcript messages.
  Do not split into multiple leading system messages (undocumented; `16` A1).
- **R2** No volatile bytes before L4: no `TaskId`, `RunId`, clock, cost, attempt number, random value, memory
  `run_id`/`task_id`, or UUID-keyed fact names in L1–L3. L4 is the first place run-specific text may appear.
- **R3** L4 must be byte-identical across tasks of a run. Today `task_prompt` puts the run prompt first
  (`provider_executor.rs:444-450`) — keep that — but the context block must move out of the user prompt into L3
  (or, before v11, to a **post-L5 volatile section** that does not claim cache value).
- **R4** L3 is frozen at plan time (one snapshot per run) and replaced wholesale; a mid-run fact write is
  forbidden for cache reasons. Until memory v11 lands, L3 is empty and its absence is stated in the prefix hash
  (`L3=""`).
- **R5** L5 is after L4; role/depth/task instruction may differ per task. `task_prompt` currently emits
  `Original user prompt:…\n\nAssigned role:…\nSpawn depth:…\nReturn…` — the tail directive (L7) is correct, but
  the whole block contains UUID-free text already; keep role/depth here.
- **R6** L6 replays **provider bytes**: tool-call `arguments` must be stored raw as returned by the assembler
  (not `serde_json::to_string` of parsed args), and `reasoning_content` must be emitted only when non-empty and
  thinking was enabled for that turn.
- **R7** L0 is fixed per tier; its canonical serialization must be deterministic (sorted object keys, no
  whitespace variance) and hashed; adding/reordering a tool bumps `TOOLS_VERSION` and invalidates the tier head.
- **R8** One `PREFIX_VERSION` constant covers L1/L2 wording. Bump it in the same commit as the wording change;
  the bump is a deliberate cache reset and must be recorded in the ledger (all hits go to 0 for new fp).
- **R9** The estimator and the ledger record the same `prefix_fp` (§8.2), so hit-rate changes are attributable.

### 3.2 Per-call shapes

- **Planner**: `[system: L1 + planner section][user: "Create the execution-plan DAG…\n\nTask:\n" + L4]`.
  No tools. The planner shares L1 with leaves; its planner section is after L1, so the shared prefix is L1.
- **Leaf first call**: `[system: L1+L2(+empty L3)][user: L4+L5+L7]` + L0 tools.
- **Continuation**: `[system: same L1+L2+L3][user: L4+L5+L7][assistant tool_calls raw][tool results]…` —
  append-only; do not repeat L7; do not rebuild L4/L5.
- **Spawned/planned children**: same L1/L2 rules; L4 is the run prompt; L5 carries parent role/depth; a child
  instruction goes in L5, **never** before L4.

### 3.3 Exact hash inputs (DESIGN; replaces `16` §6.4 formula)

```
tools_sha256  = sha256(canonical_json(tools[]))                     // L0
prefix_fp     = sha256( "v" || PREFIX_VERSION || "|" ||
                        model.id() || "|" || user_id_or_empty || "|" ||
                        tools_sha256 || "|" || sha256(L1) || "|" || sha256(L2) || "|" || sha256(L3) || "|" ||
                        sha256(L4) || "|" || sha256(first_user_message_prefix_up_to_L4) )
transcript_fp = sha256(prefix_fp || "|" || concat(sha256(each prior assistant/tool message bytes)))
```

`prefix_fp` identifies the **head arrangement** (what caching is meant to reuse); `transcript_fp` identifies the
call's full current prefix for continuity analysis. `user_id` is included because C9 makes it a cache-isolation
key. Compute both in `hiverouter-core` (it owns message assembly and already depends on the tools crate); pass
them into `CallEstimate`/ledger; do not add a hash crate to `hiverouter-provider` just for this.

### 3.4 Current code vs the order (all VERIFIED)

| Layer | Current code | Gap |
|---|---|---|
| L0 | `tool_specs_for_tier` (`tool_loop.rs:206-219`), appended in `provider_executor.rs:328-344`/`:414-420` | Order stable in practice; no canonical hash, no version |
| L1 | Does not exist: system is `TASK_SYSTEM` (`:31-33`) or persona only (`:328`, `:375`) | No hive-wide global block |
| L2 | Persona prompt from `http.rs:258-275`, lookup normalised in dirty tree (`provider_executor.rs:82-92`); `worker` has no persona; planner uses `PLANNER_SYSTEM` | No planner persona, no single builder, no output-schema/tool-policy block |
| L3 | No frozen snapshot; per-task facts are appended **after** the run prompt (`execution.rs:53-58`, `:2755`; `context.rs:66-95`) with UUID keys (`context.rs:100-110`) | Wrong position, wrong keys, no content address |
| L4 | `task_prompt` puts `Original user prompt:…` first (`:444-450`); context is appended after it by `prompt_with_context` | Context after L4 breaks the L4 head; move context to L3/volatile tail |
| L5 | Role/depth in `task_prompt` | Fine; no canonical labels |
| L6 | Re-serialised args (`:384-392`); `reasoning_content` always set (`:402`) | Not raw provider bytes; reasoning fabricated when empty |
| L7 | `Return the final answer for this task only.` in `task_prompt` | Fine |
| Planner | `execution.rs:1552-1565` flattens template with `Debug` labels into user text; `provider_executor.rs:296-318` | No L1 sharing, duplicated instruction set |

**Phase plan:** P0 = L1/L2/L4/L5/L7 + L6 raw bytes + hash (no memory v11 needed; L3 empty). P1 = L3 frozen
snapshot when `04` v11 lands. P2 = cache measurement/UI. Do **not** wait for v11 to fix the UUID/position/raw-bytes
defects: they are the cheap wins.

---

## 4. Tokenizer integration plan (audit + concrete plan)

### 4.1 What `16` gets right

- Official tokenizer artifact exists (`16` §4.2, DOCUMENTED/MEASURED external pass); ten golden strings/rows are
  the right regression fixtures; rendering the chat prompt (not JSON bytes) is correct; `SAFETY=1.10` as a gate
  margin is a policy number, not a measurement; calibration against `usage.prompt_tokens` is the only way to
  prove fidelity (`16` A9).

### 4.2 Implementation gaps/corrections

| # | Gap | Evidence | Required change |
|---|---|---|---|
| T1 | **Artifact is absent from the tree**; only `16`'s hashes exist externally | name search over repo (no tokenizer files); `provider/Cargo.toml` deps (no `tokenizers`) | Acquire zip, vendor under `backend/crates/hiverouter-provider/assets/tokenizer/`, record `source_url`, `fetched_at`, zip/tokenizer SHA-256, byte size, licence status |
| T2 | **Licence unknown** (`16` A10: zip has no licence file) | `16` §14.2 | Do not embed until licence is confirmed; otherwise build-time download + sidecar with integrity check, or defer tokenizer and use documented-ratio fallback labelled `Assumed(ratio)` |
| T3 | **Crate/binary cost unverified**: `16` says "verify pinned version"; `tokenizers` is a large dependency with regex/backends | `provider/Cargo.toml:13-25`; §28 dependency discipline | Pin exact `tokenizers` version; measure binary size delta; verify licence/dependencies; feature-gate it; record in release notes |
| T4 | **Estimator lives in core, not provider** | `provider_executor.rs:436-442`; `16` S9 proposes `provider/src/tokenizer.rs` | Put tokenizer + rendered-prompt estimator in provider (it owns `ChatRequest` and can expose `estimate_prompt_tokens`), or a core module; do not duplicate. The engine keeps only the gate (`CallEstimate`) and ledger fields |
| T5 | **Chat-template renderer is a second implementation** | tokenizer config's Jinja template; `16` §4.3 step 5 | One renderer function, pinned template version, golden tests; do not hand-build `<｜Tool｜>` strings (16 measured 5 tokens, not a single added token) |
| T6 | **Tools token overhead** | `16` estimator contract includes tools "count once from the same renderer"; provider's exact tool rendering is not documented | Count canonical tool JSON via the same renderer; calibrate; a 10 % margin covers unknown layout, but a separate `TOOLS_VERSION` makes changes visible |
| T7 | **Fail-closed vs fallback ambiguity** | `16` §4.3 step 6 says `E_TOKENIZER_UNAVAILABLE`; §4.5 keeps ratio fallback if operator insists | Decide: default fail-closed for budget/context **refusals**, but allow an explicit `estimator=ratio` setting with `Assumed(ratio)` label; never silently fall back |
| T8 | **Context limit source** | Live `/models` returns 1,048,576; `16` formula assumes 1,000,000; `ModelInfo` (`dto.rs:890-909`) parses it; no consumer | Use `models[id].context_window` when available (cache with TTL at boot/on-demand), else documented 1M conservative; max output likewise `models[id].max_output_tokens` else 393,216 |
| T9 | **Calibration requires paid calls** (unavoidable) | `16` A9 | Store `estimated_prompt_tokens` and measured `prompt_tokens` per call in the ledger; gate on p95 ratio; report "uncalibrated" in UI until samples exist; no paid calls in this seat |
| T10 | **The old bytes/3 proposal in `17` conflicts** | `17` §4.7/§4.8: `max(ceil(bytes/3),64)`; `16` rejects bytes heuristics | One estimator: tokenizer (gated) + documented-ratio fallback; do **not** ship both in production; `bytes/3` may be an emergency fallback only and must be labelled |

### 4.3 Acquisition/implementation steps (DESIGN)

1. Fetch `https://cdn.deepseek.com/api-docs/deepseek_v4_tokenizer.zip` at build time or vendor it; verify SHA-256
   and record provenance (source URL/date/size/licence status).
2. Add pinned `tokenizers` dependency (feature-gated, e.g. `tokenizer` feature) and `include_bytes!` the
   `tokenizer.json` + config; lazy-load once into `Arc<Tokenizer>` at boot; fail fast if missing/hash-mismatch.
3. Implement `render_prompt(request)` exactly once (system/user/tool-turn sections per the pinned template), then
   `estimate_prompt_tokens = token_count(render) + token_count(canonical_tools_text)`, return
   `{estimate, rendered_hash, tools_sha256, prefix_fp}`.
4. Gate: `gate_tokens = ceil(estimate × 1.10)` for context/token refusals; display the unadjusted estimate with
   `Assumed` provenance.
5. Calibration: ledger stores `est_tokens` and `measured_prompt_tokens`; expose p50/p95 ratio in Provider/Cache
   panel; alert when p95 > 1.05; `SAFETY` and estimator are settings (human-visible, audited).
6. Golden tests: the ten `16` strings reproduce their counts; CJK/code/facts rows must not undercount; a rendered
   tool-turn test; artifact hash test; renderer determinism test; no paid call needed.
7. Context gate uses live model metadata cached at boot/60 s; `max_out > model.max_output_tokens` →
   `E_SCHEMA_INVALID` (existing code path in `dto.rs:546-553`); prompt+max_out > context →
   `E_CONTEXT_WINDOW_EXCEEDED` (new code, contract patch).

---

## 5. Cost ledger, schema, routes (merged audit of `16` §6 and `17` §4.7)

### 5.1 Migration numbering/ownership conflict (BLOCKING)

| Report | Proposed migration | Schema | Conflict |
|---|---|---|---|
| `04-memory-architecture.md` | v11 `memory_banks` | bank tree, entries, snapshots, journal | Owns v11 by design; all provider work must start at v12+ |
| `16-deepseek-cache-cost.md` | v11 `provider_calls`; v12 `request_charges` | per-call token/cost ledger + request charges | Collides with `04` v11 |
| `17-scale-security-impl.md` | v11 `cost_ledger`; v12 `hive_budget` | overlapping ledger + hive reservation table | Collides with `04`, and duplicates `16` with a different schema |
| `15-ui-ux-goal.md` | v12 `provider_calls`/`settings`/`watchdog` | UI needs cost/provider tables + settings + daemons | Collides with both; three different v12 claims |

**Recommendation (DESIGN):** one migration registry owned by the Lead before any schema code:

| Version | Owner | Contents |
|---|---|---|
| v11 | `04` memory | `memory_banks`, `memory_entries`, `memory_context_snapshots`, `memory_journal` |
| v12 | provider/cost (merge `16` + `17`) | `provider_calls` (single ledger; see §5.2), `request_charges`, `run_usage` additive status/counters, `budget_state` |
| v13 | settings/watchdog (`15`) | `settings`, `settings_audit`, `watchdogs` |
| v14+ | scale/security (`17`) | `hive_budget`/global budget reservations, DB actor indexes, etc. |

Do not add a second ledger for `cost_ledger`; merge `16`'s `provider_calls` and `17`'s `cost_ledger` into one
table, and keep `hive_budget` separate only if reservation state is truly per-hive (v14).

### 5.2 Corrected `provider_calls` schema (DESIGN; every current-code constraint cited)

```sql
CREATE TABLE provider_calls (
  id                TEXT PRIMARY KEY,             -- uuid v7
  run_id            TEXT NOT NULL,
  task_id           TEXT,                          -- NULL for the planner call (plan precedes root task)
  hive_id           TEXT NOT NULL,                 -- run root: 'run:<run_id>' (or '<run>/<superagent>' later)
  agent_key         TEXT NOT NULL,                 -- role/persona id ('planner' for the planner call)
  tier              TEXT NOT NULL,                 -- hyper|super|worker|subagent|tool
  model             TEXT NOT NULL,
  peak_class        TEXT NOT NULL,                 -- peak|off_peak, from pricing.rs:28-45 at send time
  call_kind         TEXT NOT NULL,                 -- planner|leaf|continuation
  turn_index        INTEGER NOT NULL,              -- 0 for planner; tool turn for leaf
  attempt_index     INTEGER NOT NULL,              -- 1..=max_attempts; retries are rows with status
  attempt_status    TEXT NOT NULL,                 -- succeeded|failed|refused|cancelled|incomplete
  http_status       INTEGER,                       -- NULL when no HTTP response (timeout/refusal)
  finish_reason     TEXT,                          -- provider finish_reason; NULL when absent
  provider_request_id TEXT,                        -- provider response id when reported
  prefix_fp         TEXT NOT NULL,                 -- §3.3
  transcript_fp     TEXT NOT NULL,
  tools_sha256      TEXT NOT NULL,
  est_prompt_tokens INTEGER,                       -- Assumed (tokenizer gate input)
  prompt_tokens     INTEGER,                       -- Measured (NULL stays NULL)
  cache_hit_tokens  INTEGER,
  cache_miss_tokens INTEGER,
  completion_tokens INTEGER,
  reasoning_tokens  INTEGER,
  amount_nano_usd   INTEGER,                       -- exact 1e-9 USD; NULL when usage incomplete
  amount_usd        TEXT,                          -- normalized Decimal display string; NULL when incomplete
  price_source_url  TEXT NOT NULL,
  price_fetched_at  TEXT NOT NULL,
  usage_provenance  TEXT NOT NULL,                 -- measured|unknown
  amount_provenance TEXT NOT NULL,                 -- derived|unknown
  cost_status       TEXT NOT NULL,                 -- COMPLETE|INCOMPLETE
  incomplete_reason TEXT,
  started_at        TEXT NOT NULL,
  finished_at       TEXT,
  created_at        TEXT NOT NULL
);
CREATE INDEX idx_provider_calls_run  ON provider_calls(run_id, created_at);
CREATE INDEX idx_provider_calls_hive ON provider_calls(hive_id, created_at);
CREATE INDEX idx_provider_calls_fp   ON provider_calls(prefix_fp, created_at);
```

Corrections vs `16`: `task_id` nullable (planner); `attempt_status`/`http_status`/`finish_reason`/timestamps/
provider request id added (needed for retries and cache diagnosis); `cost_status`/`incomplete_reason` per row;
`transcript_fp`/`tools_sha256`; `hive_id` defined as run root, not `projects.id` (contradiction with `17` §4.7);
no prompt text is stored (privacy).

`request_charges` (DESIGN, only if Search/Fetch adopted): same shape with `surface`, `http_status`, `billable`,
`units`, `amount_nano_usd=750_000`, `prefix_fp` nullable, provenance; no token columns.

`run_usage` stays a cheap rollup but gains `status TEXT NOT NULL`, `calls_measured`, `calls_total`,
`cost_status`, `settled_nano_usd`, `updated_at`; write it in the same transaction as the terminal call/rollup.
It remains the route `GET /api/usage/runs/{id}` source, but the route must return `usage:null` when
`status=INCOMPLETE` or `calls_measured < calls_total` (never digits).

### 5.3 Receipt path (the missing implementation seam)

Current path: `client.complete_with_budget → Completion{usage}` and
`client.stream_with_budget → StreamHandle`; the provider-side `RunBudget` owns `RunAccounting` and returns
`CallCost` from `record_call`, but:

- `complete_with_budget` discards the `CallCost` (`client.rs:391-401`);
- `stream_with_budget` returns no receipt (`client.rs:553-560`); `stream_reply` calls `record_call` and discards
  `CallCost` (`provider_executor.rs:233-243`);
- the engine only sees `StreamedReply.usage` for completed leaves (`execution.rs:3008-3016`) and **never sees the
  planner usage** (`plan()` returns `Plan`, `provider_executor.rs:291-318`).

**Design seam:** add a `CallReceipt` value (`acct: RunAccounting`-derived row fields: run/task/hive/agent/model/
peak/call_kind/attempt/prefix_fp/usage/CallCost/cost_status/incomplete_reason/timestamps) and a `LedgerSink`
trait implemented by the engine/DB, passed into the adapter/client:

```rust
pub trait CallLedger: Send + Sync {
    fn record_call(&self, receipt: CallReceipt);
    fn record_attempt(&self, attempt: AttemptRecord);
}
```

Minimum viable change if a trait is too invasive: return `Vec<CallReceipt>` from `plan()` (as `PlanWithUsage`) and
extend `Completion`/`StreamedReply` with `receipt: Option<CallReceipt>`; then the engine writes rows after each
call. Either way, **the planner receipt must be captured before the root task exists** (nullable `task_id`).

### 5.4 Persistence timing and aggregation rules (DESIGN)

- Write one `provider_calls` row per **provider attempt** after its outcome; successes settle amount; failures
  write `attempt_status=failed`, null usage/cost, no `INCOMPLETE` marking (a failed attempt is not a missing
  measurement); missing usage on a **successful** attempt marks the run `INCOMPLETE` and writes a row with
  `cost_status=INCOMPLETE`, null amount.
- Planner attempt is recorded with `task_id=NULL`, `call_kind=planner`, `agent_key='planner'`, `hive_id='run:<id>'`.
- Update `run_usage` rollup in the same transaction as each settled row (or a bounded batch); never derive
  `run_usage` by `SUM(TEXT)`; store nano integer and decimal string; aggregate in Rust with `Decimal`.
- Terminal `execution.rs:1305` `persist_run_usage` must be extended to persist cost status and counts; failed/
  cancelled runs must retain the rows and mark incomplete reasons, not drop them.
- Retries: a retry that eventually succeeds bills once; earlier failed attempts have no usage; `attempts` count
  includes retries (matching `budget.rs:31`).
- No historical backfill of cost (NULL stays NULL); forward-only.

### 5.5 Routes (DESIGN) and security

| Route | Purpose | Rules |
|---|---|---|
| `GET /api/usage/runs/{id}` (extend existing `http.rs:903-936`; pre-guard anchor `:842-875`) | run token usage + status + calls measured/total + settled cost (nullable) | returns `usage:null`/`status:INCOMPLETE` rather than digits when incomplete; no key |
| `GET /api/cost/runs/{id}` (new) | run/agent/hive/prefix/call-kind rollups + cache metrics + price provenance | COMPLETE-only money; `derived` labels; no prompt text |
| `GET /api/cost/tasks/{task_id}` | per-agent call rows | deterministic ordering; bounded |
| `GET /api/cost/hives?from&to` | hive/day rollups (after hive_id exists) | exact partition of run total; UTC; no double count |
| `GET /api/provider/account` (or extend models/balance) | key present/model/catalogue/balance `as_of`/health | no key return; origin check; cache TTL |
| `GET /api/budget` | mode, caps, soft threshold, kill state, reserved/settled | read-only |
| `POST /api/budget/{arm\|disarm\|kill\|rearm}` | operator controls | mutation: Host+Origin+CSRF/session-token gate (06 T12); idempotent; audit row; kill survives restart (`app_meta`/`budget_state`) |
| live meter | extend `RUN_METRICS`/new `USAGE_UPDATE` event | additive contract change; coalesced ≤4/s; client ignores unknown fields |
| Search/Fetch (only if adopted) | `/api/search`, `/api/fetch` or a WebSocket tool path | network permission, approval, SSRF/URL policy, per-request ledger, QPS limiter |

Origin checks exist only for provider metadata routes (`http.rs:987-999` in the current tree); the dirty tree
adds a Host-header guard (`http.rs:533-589`) which closes the DNS-rebinding class from `06` T12, but the
uncommitted guard is unverified here, and the mutating routes (profile import/delete, memory clear, future
budget routes) still need an authenticated/operator boundary (session token) in addition to Host/Origin. The live
server has `allowed_origins:2`; no token on cost/budget routes.

### 5.6 UI hooks (cross-reference)

`15-ui-ux-goal.md` already defines the seven meters and Provider/Cache panel. This lane adds the source rules:
meters 4–6 (calls/tokens/cost) must read this ledger; `INCOMPLETE` renders words, never digits; cache hit ratio
only when every call in scope has the split; no client-side money arithmetic. The `ModelsPanel` currently renders
the usage digits without status (`web/src/panels/ModelsPanel.tsx:174-195`) — it must consume the new status.

---

## 6. Exact preflight formula (the missing piece in `16` §5/§7.3)

### 6.1 Inputs (per attempt, before any HTTP)

```
M       = model row from live /models (context_window, max_output_tokens) or documented fallback
E       = gated_prompt_estimate(request) =
            ceil( tokenizer_estimate(render_prompt(request))
                  * SAFETY (1.10) )
          (+ tools token count; computed by the same renderer)
O       = request.max_tokens if set, else documented default per thinking/effort
P       = peak_at_unix(now)  (send-time class, pricing.rs:191-197)
usage   = measured run totals from ledger: prompt, completion, hit, miss, reasoning; NULLs stay NULL
settled = Decimal from ledger (COMPLETE calls only) — never from estimates
reserved= Σ live reservations for this scope (run/hive/global)
cap     = active cap(s) by mode (soft/hard/per-run/per-hive/global; NULL = none)
```

### 6.2 Order (all checks before `open`; zero HTTP on refusal)

1. **Request schema** (`dto.rs:validate`) — existing.
2. **Kill switch** (`budget_state.killed`) → `E_SPEND_HALTED`; state persisted (`app_meta`/`budget_state`).
3. **Usage completeness** — if any prior settled call has incomplete usage, refuse with `E_USAGE_INCOMPLETE`
   (existing semantics, `budget.rs:235-242`).
4. **Attempt cap** — `attempts >= CALL_CAP` → `E_CALL_LIMIT` (existing, `budget.rs:243-251`).
5. **Context window** — `E + O > min(M.context_window, MODEL_CONTEXT_FALLBACK)` →
   `E_CONTEXT_WINDOW_EXCEEDED`; if `O > M.max_output_tokens` or `O > MAX_OUTPUT_TOKENS` →
   `E_SCHEMA_INVALID` (existing range check at `dto.rs:546-553`).
6. **Token cap (only if a token cap is enabled)**:
   `projected_tokens = measured_total_tokens? + reserved_tokens + E + O`; if known and `> cap.tokens` →
   `E_TOKEN_BUDGET_EXCEEDED`; if measured is null/partial → `E_USAGE_INCOMPLETE`.
7. **Projected cost (Mode C hard cap, and Mode B guard trigger)**:

```
projected_cost = if hard_gate:
    E       * miss_price(P)                       // worst case: no cache assumption
  + O       * output_price(P)                     // peak class at send time
else: // soft guard display/trigger: expected value using measured hit ratio
    E * (hit_ratio * hit_price(P) + (1-hit_ratio) * miss_price(P))
  + O * output_price(P)

refuse if hard cap and settled + reserved + projected_cost > cap.money   -> E_BUDGET_EXCEEDED
pause/alert if soft threshold and settled + reserved + projected_cost > soft_limit -> E_SPEND_GUARD (Mode B)
```

Notes: `hit_ratio` may be `None` (then use worst-case `miss_price`); per-call peak class is stored in the ledger;
cost is `Decimal` throughout; store `amount_nano_usd` on settle; **reserve before send and release on settle/
failure** (`16` §7.3); reservations are held under one lock per scope to stop N-way overshoot; failed attempts
add no reservation (only successful billing does, but the attempt cap still increments); retry attempts must not
double-reserve. In Mode A no money refusal, but the same projection is recorded for the soft guard/alert and the
kill switch.

8. **Reservation** — insert reservation row (scope, estimate/amount, expires_at = now + max call timeout); on
   settle replace with actual; on refusal/failure release. Recovery: expired reservations are released at boot
   with an audit row (crash safety).

### 6.3 Worst-case arithmetic (why Mode A needs more than structure)

Using the current structural caps (40 provider attempts; planner max output 16,384; leaf 8,192; context
1,048,576; Pro peak prices 1.32 miss / 3.96 output per 1M):

- One max-context Pro-peak call: prompt `1,048,576-16,384 ≈ 1,032,192` miss tokens → **$1.3625**, output
  `16,384` → **$0.0649** ⇒ **≈ $1.43/call**.
- 40 attempts → **≈ $57.1/run** worst case; Flash off-peak equivalent: `1,032,192×0.15/1M=$0.1548` +
  `8,192×0.60/1M=$0.0049` ⇒ **≈$0.16/call**, **≈$6.4/run**.
- If the target raises calls to `max(40, agents+16)` (166 for 150 agents) at Pro peak: **≈$237/run** worst case.
- One-call overshoot remains possible today because `preflight` checks `spent >= cap` before send (`budget.rs:252-261`);
  `06` §6.4 computes worst-case **≈$1.07** per run at the $0.50 cap for one full 400K-token call.

**Conclusion:** Mode A without a daily/account guard is not denial-of-wallet-safe even though it is
"structurally bounded". The default must be **Mode B (soft) with a real account/day guard**, or Mode A only with
(a) account balance floor enforced at preflight, (b) 402 freeze, (c) kill switch, (d) a daily spend threshold
alert/pause, (e) the ledger. `17`'s "soft default + keep current caps" is the safer reconciliation with the
operator's answer; `16`'s Mode A should be an opt-in preset after the gates pass.

---

## 7. Cache-hit measurement (corrected and exact)

### 7.1 Per-call fields

Every `provider_calls` row stores: `prompt_tokens`, `cache_hit_tokens`, `cache_miss_tokens`,
`completion_tokens`, `reasoning_tokens`, `peak_class`, `prefix_fp`, `transcript_fp`, `call_kind`, `agent_key`,
`hive_id`, `started_at`, `finished_at`, `cost_status`, `est_prompt_tokens`.

### 7.2 Definitions (exact; no fuzzy claims)

| Metric | Formula | Gate |
|---|---|---|
| Call hit rate | `hit_i / (hit_i + miss_i)` if both present and `hit+miss>0`; else `null` | never show a rate built from nulls |
| Scope hit rate | `Σ hit_i / Σ (hit_i + miss_i)` over calls with both present | only if **all** calls in scope measured; otherwise `INCOMPLETE` and no rate |
| Effective input price | `Σ_i (hit_i·hit_price(P_i) + miss_i·miss_price(P_i)) / Σ_i (hit_i+miss_i)` | per-call peak class, Decimal; COMPLETE only |
| Cache saving | `Σ_i hit_i · (miss_price(P_i) − hit_price(P_i))` | COMPLETE only; labelled `derived` |
| Cache saving % | `saving / Σ_i (hit_i+miss_i)·miss_price(P_i)` | COMPLETE only |
| Cold-start window | call with hit=0 within 10 min of process start or after >1 h idle | flag, do not count as design failure |
| Estimate ratio | `measured_prompt_tokens / est_prompt_tokens` per call; report p50/p95 | estimator calibration only; never cost input |
| Estimate accuracy | `p95 ratio ≤ 1.05` (tokenizer after margin) | alert when violated |
| Cross-agent reuse | distinct `agent_key`/`task_id` per `prefix_fp`; hit tokens on calls sharing fp | attributed to arrangement, not knowledge |

### 7.3 Corrections to `16`

- `prefix_fp` must include `model`, `user_id` policy, `tools_sha256`, L1–L4 bytes, and (for continuations) the
  transcript boundary; `16`'s `L1..L4` only is insufficient (continuations share one fp).
- `transcript_fp` separates "head reuse" from "this call's full prefix".
- Per-call peak class × cache price must be stored to compute saving correctly; the run aggregate alone is not
  enough (`run_usage` has no peak class).
- Ratio denominator: `Σ(hit+miss)`, not `prompt_tokens` when the split is missing; if `prompt_tokens` is present
  but split missing, mark incomplete (never infer miss = prompt − hit).
- Cache-hit rate must not be aggregated across different `prefix_fp` arrangements as one number; show per
  arrangement/agent/hive, plus a global only when the set is comparable (same model/user_id).
- Cold-start/eviction: `16` C7 says entries clear in hours–days and best effort; no TTL is documented. Do not
  model a fixed TTL; monitor.
- Claim discipline: any "caching saved X" statement requires COMPLETE rows; until then the UI says
  "hit rate not measured on this build" — the current product has **no** hit-rate number (`16` is right).

---

## 8. Budget modes (reconciled with `17`; operator decision pending)

`16` §7.2 A/B/C and `17` §4.7 `observe|soft|hard` describe the same product with different defaults. Decision
needed by the operator; implementation-safe recommendation:

| Mode | Money cap | Token cap | Structural caps | Guard behaviour | Default |
|---|---|---|---|---|---|
| **A. Observe** (`observe`) | none | none | agents/run, fan-out, depth, calls/run, per-call `max_tokens`, context preflight, in-flight, wall-clock deadline | balance floor + 402 freeze + kill switch + ledger; alert on soft thresholds, no auto-refuse on spend | **not default until gates pass** |
| **B. Soft guard** (`soft`) | operator sets soft threshold (daily/run) | optional | as A | crossing threshold pauses new sends (`E_SPEND_GUARD`), one click to continue/raise; in-flight settle/cancel per setting | **recommended default for launch** |
| **C. Hard caps** (`hard`) | run/hive/day caps (USD and/or tokens) | optional | as A | projected preflight + reservation refuses by name (`E_BUDGET_EXCEEDED`/`E_HIVE_BUDGET_EXCEEDED`) | opt-in; current `$0.50/400K/40` becomes a preset, not a silent default |

**Reconciliation:** `17`'s security-wave posture (keep current hard caps; add soft/observe modes; no silent
removal) is safer than `16`'s Mode A default and still matches the operator's "no hard default" intent once the
P0 gates (kill switch, 402 freeze, balance floor, ledger, daily guard) are green. Do not ship Mode A by deleting
`budget.rs` constants; that is a denial-of-wallet regression (`16` §7.4 itself says this).

Kill switch: `POST /api/budget/kill` cancels active run tokens, blocks preflight with `E_SPEND_HALTED`, persists
in `app_meta`/`budget_state`, idempotent, audited; re-arm explicit. 402: map to new `E_PROVIDER_BALANCE`, no retry,
freeze new sends for the process/until operator action; current mapping 401/402→`E_PROVIDER_AUTH` is misleading
(`client.rs:507-511`; `refusal.rs:??` auth message says check key). Balance floor: relative to boot balance
(5 % proposed), refreshed on provider account reads; if balance unavailable, show `unknown` and fall back to
Mode B threshold + kill; never invent a balance.

---

## 9. Search / Fetch / Files / Vision decisions (reconciled with operator)

| Surface | Decision for v1 | Prerequisites before first call | Revisit trigger |
|---|---|---|---|
| **Search** (`POST /v1/tools/search`, $0.75/1k successful) | **Not in v1; conditional-adopt (P3)** | `PermissionClass::Network` tool ids, approval gate, SSRF/URL policy, per-request `request_charges` ledger + cost dimension in `RunAccounting`, billable predicate (200 + non-empty results), QPS limiter (100), `need_content=false` default, result-filtering before prompt, cost panel row | After provider ledger/cache + approval coverage; one search-per-agent cost dominated by request charge (`16` §8.1) |
| **Fetch** (`POST /v1/tools/fetch`, $0.75/1k successful) | **Not in v1; conditional-adopt (P3)** | same + URL ≤8192/no credentials, 403/404/408/413 handling as refusals, Markdown size cap before prompt, retry only 408/429/5xx, per-domain QPS (25) | Same as Search; Fetch can be first if a URL is operator-supplied |
| **Files** (image-only storage) | **Out of v1**; only as future vision transport | content-part model, vision decision, 25 GiB/10k/expiry settings, account-scoped storage policy, retention | A real image workflow is approved; do not adopt for text/repo files (`01`/`16`) |
| **Vision** (Flash-only, user-turn, ≤1024 tok/image) | **Out of v1**; deferred with explicit trigger | `ChatMessage.content` content-part type (`dto.rs:88-106` is text-only), Flash-only routing, image-token accounting (≤1024/image), cache behavior measurement (undocumented), UI image surface | Operator says "important later"; revisit when an image workflow exists and provider cache image behaviour is measured |
| **Responses / Anthropic Messages / FIM** | Not needed | — | `01`/`16` agree; no action |

Note the posture conflict: `01` says Files/Vision "NOT NEEDED"; the operator says Files default yes only for
vision/image and Vision "important later"; `10-gap-meta` flags this. The honest resolution is **"deferred, not
excluded"**: record the revisit trigger and do not claim a decision beyond v1.

---

## 10. File:line seams (implementation handoff)

| Area | Seam (current) | Change |
|---|---|---|
| Prefix builder | `provider_executor.rs:297-318` (planner), `:330-344` (leaf), `:374-420` (continuation), `:444-450` (`task_prompt`), `execution.rs:53-58`, `:2755-2756` (`prompt_with_context`), `context.rs:66-110` | New `core/src/prompt.rs`/`prefix.rs`: one builder returns `(messages, prefix_fp, transcript_fp, tools_sha256)`; semantic keys; context/memory order; raw replay |
| Planner prompt | `execution.rs:1552-1565`; `provider_executor.rs:286-318` | System L1+planner section; user instruction+run prompt; delete `Debug` flattening; capture planner usage receipt |
| Usage/receipt | `provider_executor.rs:94-120`, `:128-286`, `:291-318`; `client.rs:361-401`, `:553-607`, `:634-727`; `execution.rs:698-719`, `:3008-3016`, `:1305` | `CallReceipt`/`LedgerSink`; persist per call incl. planner; terminal rollup status |
| Budget | `provider_executor.rs:104-107`, `:141-143` (hard-coded default); `budget.rs:24-52`, `:235-305`; `client.rs:385-388` (`preflight`+`record_attempt`), `:576-579` | `BudgetPolicy` from config/settings; projected cost + reservation; modes; kill/402/balance; new codes |
| Estimator/tokenizer | `provider_executor.rs:436-442`; new `provider/src/tokenizer.rs` + `assets/tokenizer/`; `dto.rs:890-909` (metadata) | Gate estimator + calibration; context metadata cache |
| Cost ledger | `db.rs:32-249` (v1–v10), `:198-217` (`run_usage`), `store.rs:678-730`, `http.rs:903-936` (pre-guard `:842-875`), `execution.rs:698-719` | v11/v12 owner decision; merged `provider_calls`; routes; status |
| Refusals | `contract/src/refusal.rs:145-190` (budget/token/call/usage) | Add the six codes only after contract decision; update web `KNOWN_REFUSAL_CODES` (`web/src/api/contract.ts:426`) and golden/matrix tests |
| 402 | `client.rs:492-534` (401 remains `ProviderAuth`; 402 now has a balance message but **still `ProviderAuth` code**; 400/422 mapped to request-shape) | Split `E_PROVIDER_BALANCE` for 402, no retry, freeze; update `provider/tests/error_matrix.rs` |
| Live model metadata | `dto.rs:890-909`; `http.rs:942-964` (pre-guard `:881-901`) | Cache catalogue/metadata with `as_of`, use limits in preflight |
| Search/Fetch | `tools/src/tool.rs:30-49` (`Network` unused), `:134-142` (ids), `executor` | Only if adopted: network tool + approval + request ledger |
| Files/Vision | `dto.rs:88-106` (`ChatMessage.content: Option<String>`), `client.rs:266-313` (GET-only) | Deferred; content-part type + limits when triggered |
| UI | `web/src/panels/ModelsPanel.tsx:174-195`; new cost panel/routes; `15` seven meters | Status words, no digits on INCOMPLETE; no client money math |

---

## 11. Twenty acceptance tests (no paid calls; wiremock/unit/DB temp only)

| # | Test file / name | Assertion |
|---|---|---|
| 1 | `provider/tests/tokenizer.rs::golden_counts` | The ten `16` §4.2 strings reproduce 48/108/41/159/228/58/123/54/24/169 (or documented correction if renderer differs); artifact SHA-256 matches provenance |
| 2 | `provider/tests/tokenizer.rs::estimate_uses_rendered_prompt_not_json` | Planner request estimate equals tokenizer count of rendered prompt + tools; JSON body bytes/4 is not used |
| 3 | `provider/tests/tokenizer.rs::cjk_and_uuid_content_not_undercounted` | Facts-block fixture estimate ≥ measured tokenizer count × 0.95 (guards `16`'s −47 % defect) |
| 4 | `core/tests/prompt_prefix.rs::prefix_bytes_are_stable` | Two sibling tasks of one run render byte-identical `L1/L2/L4`; no UUID regex appears in the prompt; context is after L4 |
| 5 | `core/tests/prompt_prefix.rs::prefix_fp_changes_only_with_the_layer` | Edit L1/L2/L4 ⇒ fp changes; edit L5/tail ⇒ fp unchanged; reorder tools ⇒ fp changes; `PREFIX_VERSION` bump ⇒ changes |
| 6 | `provider_executor/tests::replayed_arguments_are_provider_bytes` | A tool call with spaced JSON replayed byte-identically; `reasoning_content` absent when empty/thinking-off |
| 7 | `core/tests/planner_usage.rs::planner_usage_is_persisted` | Wiremock planner returns usage; `provider_calls` gets a planner row (`task_id NULL`, `call_kind=planner`) and `run_usage` includes it; absent usage ⇒ INCOMPLETE, no digits |
| 8 | `provider/tests/cost_ledger.rs::settled_call_receipt_is_exact` | hit 400/miss 600/out 500 → exact nano/decimal, peak class, price source, prefix fp; `CallCost` reaches the ledger via receipt |
| 9 | `core/tests/cost_ledger.rs::incomplete_usage_persists_no_money` | Null/partial usage on a successful call ⇒ row `cost_status=INCOMPLETE`, `amount=NULL`, `run_usage.status=INCOMPLETE`, route returns no digits |
| 10 | `core/tests/cost_ledger.rs::hive_partition_is_exact` | Σ per-hive Decimal equals run total exactly; no double count; planner row assigned to run root |
| 11 | `web/tests/usageStatus.test.ts` (or core route test) | `/api/usage/runs/{id}` returns status/calls; INCOMPLETE renders words not digits in ModelsPanel |
| 12 | `provider/tests/budget.rs::mode_b_soft_guard_pauses` | Soft threshold crossing ⇒ `E_SPEND_GUARD` on the next preflight, zero HTTP calls; settle releases reservation |
| 13 | `provider/tests/budget.rs::mode_a_has_no_money_cap` | Mode A default for the selected preset refuses no call on money; structural call/agent caps still refuse by name |
| 14 | `provider/tests/budget.rs::mode_c_projected_cost_refuses_before_send` | spent $0.49/$0.50, projected Pro-peak call > remaining ⇒ `E_BUDGET_EXCEEDED`, 0 attempts, wiremock count 0 |
| 15 | `provider/tests/budget.rs::reservation_prevents_n_way_overshoot` | 16 concurrent reservers: total reserved never exceeds cap; exactly one succeeds at the boundary |
| 16 | `provider/tests/budget.rs::context_window_refuses_zero_requests` | prompt+max_out > live `context_window` ⇒ `E_CONTEXT_WINDOW_EXCEEDED`, 0 HTTP calls; max_out > model max ⇒ `E_SCHEMA_INVALID` |
| 17 | `provider/tests/budget.rs::kill_switch_blocks_and_survives_restart` | After kill, preflight `E_SPEND_HALTED` with 0 requests; reopen DB, still halted; rearm works; two kills ⇒ one audit row |
| 18 | `provider/tests/error_matrix.rs::http_402_is_balance_not_auth` | 402 ⇒ `E_PROVIDER_BALANCE`, no retry, request count 1; 401 remains auth; 400/422 map to request-schema refusal |
| 19 | `core/tests/cost_ledger.rs::retry_attempts_are_recorded_not_billed` | Failed attempt rows have `attempt_status=failed`, null usage/cost; final success bills once; attempts counter increments; no double count |
| 20 | `core/tests/cost_ledger.rs::migration_and_rollback` | Apply v11/v12 (after allocation), insert/settle/rollup, assert exact values, run `down`, assert tables gone; v10 fixture intact; no second ledger/duplicate schema |

(Commands: `cd backend; cargo test -p hiverouter-provider --lib --tests; cargo test -p hiverouter-core --tests`;
`cd web; npm.cmd test -- --run` for the UI status test. None were run in this read-only seat.)

---

## 12. Twenty new open questions (not settled by `16`/`17`/this audit)

1. **Who owns migration numbers v11+?** `04` memory, `16`/`17` provider ledger, `15` settings/watchdog all claim
   v11/v12. Need a single registry before code.
2. **Merge `provider_calls` and `cost_ledger`?** Two overlapping designs; one table or two? If one, whose fields
   win (nano vs decimal string, status, attempt rows)?
3. **What is `hive_id`?** `16`: run root / nearest SuperAgent; `17`: `projects.id`; operator memory model uses
   project→run→hive→branch. Which becomes the storage key, and how is it derived for the planner call?
4. **How is the planner call attributed with no task row?** Nullable `task_id` + `call_kind=planner`, or a
   synthetic pre-root task row? The latter changes `tasks`/`NODE_SPAWN` semantics.
5. **Where does the tokenizer asset live and under what licence?** Vendor in-repo, build-time download, or
   sidecar? `16` A10 says the zip has no licence file — is redistribution permitted?
6. **Which tokenizer/crate version/features are pinned, and what is the acceptable binary-size delta?**
7. **Is the estimator fail-closed or fallback-by-default?** `16` says `E_TOKENIZER_UNAVAILABLE` but also keeps a
   ratio fallback; a fail-closed default can block all runs if the asset is missing.
8. **How is the chat template rendered?** Reuse the artifact's Jinja via a pinned renderer or hand-code the
   template? How is template drift detected?
9. **What is the canonical tool serialization?** Field order, whitespace, `additionalProperties`, and whether the
   provider counts tools inside the same template; needs calibration.
10. **Does L1 belong in a prompt asset or code?** If asset, versioning/release pipeline; if code, cache reset
    policy and `PREFIX_VERSION` ownership.
11. **Should planner L1 be shared with leaves, or is planner's separate instruction set a deliberate cache
    boundary?** `16` recommends sharing L1 only.
12. **How is the memory snapshot frozen before v11?** L3 is empty; do context facts move to a volatile post-L5
    section, or stay in L4 until v11 (and accept the cache loss)?
13. **Mode default: observe vs soft vs hard?** `16` A, `17` soft; operator says no hard default. Which ships, and
    what is the first-launch choice?
14. **What is the daily/global guard value and unit?** Relative to boot balance (5 %), absolute operator-set, or
    run-based? How does it interact with the account balance floor?
15. **Kill-switch scope:** cancel current run only, block new sends, freeze provider account, or all? Persist in
    `app_meta` vs a new `budget_state` table; who can re-arm?
16. **402 freeze scope:** process, run, or account/until-top-up? What resets it?
17. **Peak-class boundary:** `16` A12 — price by send time or completion time? Chinese public holidays are not in
    `pricing.rs`; accept conservative overstatement or add a holiday calendar?
18. **Backfill policy:** no historical backfill of cost (forward-only) — but what about existing `run_usage`
    rows: leave them, mark legacy, or recompute from tokens (which would be a settled claim without per-call
    split)?
19. **Search/Fetch go/no-go and cost cap:** after the ledger, is there a per-run/per-day request budget? Which
    approval class, SSRF policy, and result-size cap?
20. **Files/Vision revisit trigger:** which concrete workflow (image input for UI screenshots? docs scanning?)
    changes the decision, and does the operator accept content-part/limit work as a prerequisite?

**Additional follow-ups (recorded beyond the required 20, not decision-blocking yet):** cache-hit wording in the
UI before any keyed run ("not measured on this build"); per-hive `user_id` isolation default (reuse vs isolation
cost); ledger privacy/retention (prefix/transcript hashes only, no prompt text); retry/duplicate-billing
reconciliation with the provider statement; and ledger idempotency/owner for crash/replay
(`(run_id, call_index, attempt_index)`).

---

## 13. Claim register / limitations

| Claim | Status | Evidence |
|---|---|---|
| Live `/health` schema v10; `/models` two models, 1,048,576 ctx, 393,216 max out | VERIFIED (this pass) | §0 |
| HEAD `bd575ba`, dirty tree includes persona/cap changes, no provider-ledger/tokenizer code | VERIFIED (this pass) | §0.2 |
| Planner usage is dropped; no per-call `CallCost` receipt | VERIFIED | `provider_executor.rs:291-318`, `:233-243`; `client.rs:361-401`, `:553-560` |
| `run_usage` six token columns, no money/status; route cannot report INCOMPLETE | VERIFIED | `db.rs:198-217`; `http.rs:903-936` (pre-guard `:842-875`) |
| `task_id NOT NULL` impossible for planner; migrations collide | VERIFIED | `execution.rs:1314-1327`; report table §5.1 |
| Dirty tool caps = 12 turns / 40 calls; `E_CALL_LIMIT` | VERIFIED | `tool_loop.rs:14-16`; `execution.rs:2843-2857` |
| `16`'s external facts (cache units, prices, Search/Fetch, Files, Vision) | READ/DOCUMENTED, not re-fetched this pass | `16` §1, §13.2; `01` |
| `16` tokenizer hashes/measurements | READ, not reproducible from tree | §4.2 |
| Mode A worst-case ≈$57/run | ESTIMATE (arithmetic in §6.3; assumed max-context/max-output every call) | `pricing.rs` rates, `caps.rs`, `dto.rs` limits |
| Search/Fetch/Files/Vision decisions | DESIGN | §9 |

**Limitations (deep-research gate):** no independent re-fetch of DeepSeek docs in this pass (external facts
inherited READ/DOCUMENTED); no paid call (realised cache-hit rate, tokenizer fidelity, 402 billing, image cache
remain UNVERIFIED); no test/build run; dirty tree unverified; only `/health` and `/models` re-measured. A later
verification seat must re-fetch D1/D3/D5/D7/D11 and run the keyed calibration before any caching/cost claim is
made.

---

## 14. Top 10 findings

1. **Planner usage is discarded entirely.** `plan()` drops `Completion.usage`; only leaf/continuation replies
   reach `add_run_usage` (`execution.rs:3016`). Every run's tokens/cost are undercounted before the ledger exists.
2. **No per-call cost receipt reaches the engine.** `record_call` returns `CallCost`, but both streaming and
   non-streaming paths drop it (`provider_executor.rs:233-243`; `client.rs:361-401`); `16`'s "persist the
   `CallCost`" has no transport.
3. **`16`'s ledger schema is not implementable as written for the planner** (`task_id NOT NULL`, planning precedes
   the root task) and **collides with `17`/`04`/`15` migration numbers and `hive_id` definitions**; merge before
   coding.
4. **`16` and `17` give contradictory budget defaults and estimators** — Mode A/no cap vs soft/keep caps; tokenizer
   vs `bytes/3`. Current caps + dirty 12-turn tool loop make Mode A's worst case ≈$57/run (Pro peak), so a soft
   default with account/402/kill guards is the safe reconciliation.
5. **The dirty tree fixes persona continuation and changes tool caps to 12/40 + `E_CALL_LIMIT`**, making `16`'s
   "4 turns" and `E_BUDGET_EXCEEDED` arithmetic stale; the tree still has no tokenizer, prefix builder, ledger,
   context preflight, cost routes, or budget modes.
6. **`run_usage` can display digits for an incomplete run** (route has no status; failures/partial results persist
   leaf tokens), and `Usage::checked_add`'s code contradicts its doc — per-call rows plus strict status gating are
   mandatory.
7. **The tokenizer plan lacks a reproducible asset**: no tokenizer files in the tree, no `tokenizers` dependency,
   licence unknown, and the estimator currently lives in core against the JSON body (`bytes/4`). Vendor + pin +
   render + calibrate before trusting any gate.
8. **Context preflight is absent** even though live `/models` reports 1,048,576 ctx / 393,216 max out; the A11
   assumption (1M) is resolved by the live probe, but no code consumes it.
9. **402 is still code-labelled auth** (`client.rs:507-534` current: the dirty tree adds a balance-specific
   message but keeps `Refusal::ProviderAuth`; 400/422 now map to a request-shape refusal), and new refusal codes
   (`E_PROVIDER_BALANCE`, `E_CONTEXT_WINDOW_EXCEEDED`, `E_SPEND_GUARD/HALTED`, `E_REQUEST_BUDGET_EXCEEDED`,
   `E_TOKENIZER_UNAVAILABLE`) do not exist in the frozen 65-code table/web mirror; contract patch + matrix tests
   required.
10. **Search/Fetch/Files/Vision decisions are right but not schedulable yet:** Search/Fetch must wait for a
    per-request ledger + Network approval + SSRF policy; Files/Vision are deferred with an explicit revisit
    trigger, not silently "needed" or "not needed".

**Next implementation order (DESIGN):** (1) migration allocation + merged ledger schema; (2) planner usage
receipt + `CallReceipt` path; (3) canonical prefix builder + UUID/order/raw-bytes fixes; (4) tokenizer asset
decision + estimator + context preflight; (5) projected-cost reservation + modes/kill/402/balance; (6) routes +
UI status words; (7) only then Search/Fetch.
