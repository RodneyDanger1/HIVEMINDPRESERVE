# 16 — DeepSeek cache prefix, tokenizer, streaming, budgets, and cost truth

**Lane:** second-wave research, lane 7.
**Date:** 2026-11-07 (America/Chicago). Every external fact was fetched on this date.
**Repository:** `hiverouter-v1`. **Source HEAD read:** workspace tree as of 2026-11-07 (no commits made).
**Deliverable:** this file only — `docs/research/hmars/16-deepseek-cache-cost.md`.
**No product edit, no paid provider call, no credential read, no key material in this file.**

**Documentation pin:** `version_pinned: false`.
`version_source: https://api-docs.deepseek.com/updates/` — the API docs are live and unversioned;
the newest documented change is **2026-10-17**. Everything below marked DOCUMENTED will rot; the
re-fetch list is §13.3.

## How this file is built (evidence discipline)

| Mark | Meaning |
|---|---|
| **DOCUMENTED** | Read verbatim on an official page listed in §13.2 during this pass, on 2026-11-07. |
| **MEASURED** | Produced by a command run in this pass; the command and its output are quoted. |
| **READ** | Read in this repository; seam cited as `path:line`. |
| **DESIGN** | A recommendation from this lane. Not implemented; not a claim about current behaviour. |
| **UNVERIFIED** | Not proven here. Never build a money or cache claim on it. Every such cache assumption is collected in §11. |

Two things were actually executed rather than asserted, and they carry most of the document's weight:

1. The official tokenizer artifact was downloaded and run against HiveMind's real prompt strings
   (§4.2). That converts "bytes/4 is roughly right" into a measured error table with signs.
2. Every prompt-assembly byte path was read in the provider/engine crates (§2). That converts
   "we think the prefix is stable" into a list of specific bytes that break it.

No live DeepSeek request was made. Anything about *this account's* realised cache-hit rate, live
`/models` metadata, or billed amounts is **UNVERIFIED by construction** and labelled so.

---

## 0. One-screen answer

**Cache.** DeepSeek's disk cache is not "prefix overlap"; it is a set of **cache prefix units**, and
a request hits only by **fully matching a persisted unit** (DOCUMENTED, `/guides/kv_cache`). Units
appear at (a) the end of the user input and the end of the model output of each request, (b) a
common prefix the system detects across requests, and (c) fixed token intervals inside long inputs
or outputs. Therefore the only thing HiveMind controls is **byte-identical leading text**, and the
engineering goal is: *one long, immutable, hive-wide head; every volatile byte strictly after it.*

HiveMind today has the right instinct (a fixed `TASK_SYSTEM`/persona system message plus a stable
per-tier tool array) and four specific byte-level defects that destroy reuse: **UUIDs inside the
model-visible fact keys**, **re-serialised tool-call arguments on replay**, **`reasoning_content`
invented as `""`**, and **per-task facts assembled in UUID-sorted order**. Fix those and the same
prefix that already exists starts hitting; do not redesign the transport to chase caching.

**Tokenizer.** `bytes/4` is not a tokenizer and its error has **both signs**: measured on the
official tokenizer it **undercounts** HiveMind's own fact block by **47 %** and Rust code by **26 %**,
and **overcounts** plain English and CJK by **10–25 %** and the whole serialised planner body by
**32 %**. An undercount is the dangerous direction: it lets an over-context or over-budget call
through a gate that exists to stop it. Replace it with the **official `tokenizer.json`** (vendored,
hashed, embedded) applied to the **rendered chat prompt**, not to the JSON body, and keep a
calibration loop against provider-reported `prompt_tokens`.

**Cost truth.** The provider already tells us the truth per call (hit / miss / output); the system
throws it away. `run_usage` stores six token integers per run, never money, never per-agent, never
the peak class, never completeness (`db.rs:198-217`, `execution.rs:698-719`), and
`GET /api/usage/runs/{id}` cannot say **INCOMPLETE** even when the run is
(`http.rs:842-875`, `accounting.rs:56-73`). Add a **per-call ledger** with exact integer money, the
prefix fingerprint, and provenance; expose run / agent / hive / day rollups; gate every displayed
dollar on `COMPLETE`.

**Budget policy.** The operator wants **no hard default money cap** and a visible soft guard plus a
kill switch. That is compatible with designing out denial-of-wallet only if the bound stops being a
dollar constant and becomes: **structural caps (agents, calls, fan-out, depth, wall-clock, per-call
`max_tokens`, context preflight) + an account-anchored balance floor + a 402 freeze + an explicit
kill switch + a persisted ledger**. Hard dollar caps become **opt-in presets with a reservation
model**; today's `$0.50`/`400 000`/`40-call` defaults are exactly what refuses a 150-agent run and
must move from "default" to "preset" (`budget.rs:24-43`, `refusal.rs:152-160`).

**Search / Fetch.** The only per-request-billed surface: **$0.75 per 1 000 successful requests**,
billed only on HTTP 200 **with a result** (DOCUMENTED, `/guides/search`, `/quick_start/pricing`).
Adopting it without a new cost dimension makes every run report false (`pricing.rs:47-64` has no
per-request row). **Files** is image-only storage (no text/PDF path) and **Vision** is Flash-only at
≤1024 tokens/image; both stay out of v1, with Files allowed only as vision's transport later.

---

## 1. Verified external facts this design rests on

All DOCUMENTED rows were fetched 2026-11-07 from the URLs in §13.2. No number here is inferred.

### 1.1 Cache mechanics (the whole rule set)

| # | Fact | Consequence for HiveMind |
|---|---|---|
| C1 | Cache is on by default, no code change, disk-backed. | Nothing to enable; only arrangement matters. |
| C2 | A hit requires **fully matching a cache prefix unit** — each cached prefix is an independent complete unit, due to Sliding Window Attention. | Partial/overlapping prefixes do **not** hit. "Mostly the same prompt" is a miss. |
| C3 | Units are persisted **at request boundaries**: end of user input and end of model output. | A later request that reproduces the whole `[… , user]` prefix hits it exactly. |
| C4 | Units are persisted when the system **detects a common prefix across multiple requests**. | Two sibling agents that share a head **teach** the cache that head; the third request can hit it (documented Example 2: `A+B`, `A+C` miss; after `A` is detected, `A+D` hits). First two calls are the tuition. |
| C5 | Units are carved at **fixed token intervals** inside long inputs/outputs. | A very long stable block is not uncacheable merely because it never ends a request. Exact interval size is **not documented** → UNVERIFIED (A3). |
| C6 | Cache matching covers the **prefix only**; output is always recomputed (temperature still applies). | Cache never changes the answer; it changes input cost. No correctness risk from reuse. |
| C7 | Best-effort: "does not guarantee a 100 % cache hit rate"; construction takes seconds; entries clear "usually within a few hours to a few days". | Hit rate is an observation, never a guarantee. A cold start after idle is normal, not a bug. |
| C8 | Counters: `prompt_cache_hit_tokens`, `prompt_cache_miss_tokens`; `prompt_tokens` equals their sum. | The split is the only honest measure of reuse; it is already parsed (`dto.rs:578-628`). |
| C9 | `user_id` provides **KVCache isolation** (plus content-safety and scheduling isolation); ≤512 bytes UTF-8 ASCII recommended. | Sending a per-hive `user_id` **deliberately forfeits cross-hive reuse**. A policy decision, not a default. |

### 1.2 Tokenizer, limits, and the estimator's only legitimate anchor

| # | Fact | Consequence |
|---|---|---|
| T1 | The docs publish character ratios: **1 English char ≈ 0.3 token**, **1 Chinese char ≈ 0.6 token**, and state "the actual number of tokens processed each time is based on the model's return". | A ratio is a *fallback*, not a tokenizer. The provider's `usage` is the truth. |
| T2 | The docs publish a downloadable tokenizer: `https://cdn.deepseek.com/api-docs/deepseek_v4_tokenizer.zip`. | An official offline counter exists — see §4.2 for what is actually inside it (MEASURED). |
| T3 | Image tokens: images are resized (small upscaled toward ~544×544, large downscaled toward ~1300×1300 total pixels) and capped at **1024 tokens per image**, counted per image; billed as ordinary input tokens. | Vision cost is bounded and small; there is no separate vision line item. |
| T4 | `max_tokens` **1–393216**; default **8K** non-thinking, **64K** thinking, **128K** at effort `max`. | Any per-call output cap HiveMind sets (8192 / 16384) is far below the ceiling — output caps are a cost lever, not a limit risk. |
| T5 | Context length **1M** (both models); "the total length of input tokens and generated tokens is limited by the model's context length". | Preflight must compare `prompt + max_tokens` against the window, not `prompt` alone. |
| T6 | `stream_options.include_usage` only controls whether **earlier** chunks carry `usage: null`; "either way, the last chunk before `data: [DONE]` carries the token usage statistics". | HiveMind's stream pump **does** receive usage without `stream_options` (`client.rs:678-686`) — so the INCOMPLETE-marking path is a real failure signal, not an artefact of a missing field. **Verified**, closing 01's U-level doubt. |
| T7 | `usage` fields: `prompt_tokens` required, `completion_tokens` required, `prompt_cache_hit_tokens` required, `prompt_cache_miss_tokens` required, `prompt_tokens_details.cached_tokens` = hit alias, `completion_tokens_details.reasoning_tokens`. | The hit/miss split is a *required* field, so "provider omitted the split" should be rare — if it happens, INCOMPLETE is the right verdict. |
| T8 | `/models` metadata: `context_window` = "total token capacity … counting both input and output"; `max_output_tokens` = max accepted `max_tokens`; `effort.supported_levels`; `input_modalities`. Metadata "may be omitted". | Context preflight can use live metadata when present. HiveMind parses only 6 of these fields today (`dto.rs:891-909`). |
| T9 | Prices per 1M tokens (USD): Flash hit 0.003 / 0.006, miss 0.15 / 0.30, out 0.60 / 1.20; Pro hit 0.022 / 0.044, miss 0.66 / 1.32, out 1.98 / 3.96 (off-peak / peak). Peak = 01:00–04:00 and 06:00–10:00 UTC Mon–Fri, excluding Chinese public holidays; off-peak is half. | The hit↔miss gap is **50× (Flash)** and **30× (Pro)**: cache reuse is the largest single cost lever after "send fewer tokens". |
| T10 | Concurrency: 2500 Flash / 500 Pro per account; 429 when exceeded; `user_id` splitting applies only to accounts with increased quotas. | Cache design must not assume a request is "alone"; concurrent siblings racing the same unit is normal. |

### 1.3 Per-request billing and the image surfaces

| # | Fact | Consequence |
|---|---|---|
| S1 | Search `POST /v1/tools/search`, Fetch `POST /v1/tools/fetch`, same Bearer key, base `https://api.deepseek.com`. | One key, two more endpoints — not tools on the chat endpoint. |
| S2 | Billed **per successful request**: $0.75 / 1 000. Billable only on HTTP 200 **and** a produced result (Search: non-empty `results`; Fetch: page content). Failed requests, empty results, timeouts are **not billed**. `count` does not change the price. | A "successful request" is the billable quantum; it is **not** a token. The existing `PriceRow` cannot express it. |
| S3 | Search request bounds: `query` 1–128 chars; `count` 1–50 (default 10); `ori_question` optional; `localization` is an IP; `need_content` default false, and `content` is always `""` when false. | `need_content=false` keeps returned tokens ~10 × (600–800 chars) — a bounded, cheap default. |
| S4 | Search result fields include `snippet` (~100–200 chars, for UI), `summary` (~600–800 chars, "intended to be read by a model"), `rerank_score` 0–3 (≥1.5 high quality), `authority` 0–4 (−1 unknown). | Pay for `summary`; use `rerank_score`/`authority` to *filter* before spending prompt tokens. |
| S5 | Fetch: `url` ≤8192 chars, http(s) with host, no user/password; response `{title, content}` with content as **Markdown** and unbounded length in the schema. | Fetched content must be sliced before it becomes prompt tokens or a 48 MiB body problem. |
| S6 | Statuses: 400 · 401 · 402 · **403 security policy** · **404 no usable content** · 408 · 413 · 429 · 500 · 504; retry only **408/429/5xx**; 402 top-up; other 4xx do not retry. | 403/404 are refusals, not empty results; retrying them burns account QPS for nothing. |
| S7 | QPS: Search **100**, Fetch **25**, account-level, independent, plus an additional **per-domain** Fetch limit (number not documented). | Local limiters must be per-surface, and per-domain for Fetch. |
| S8 | Server-side search inside Harness/Claude Code/Codex is also billed per successful request. | There is no free search path left; do not model one. |
| S9 | Files API is **image storage only** (JPEG/PNG/GIF/WebP, content-detected); `purpose` must be `user_data`; ≤64 MiB upload, 25 GiB, 10 000 files, expiry 1 h–30 d or permanent; **no documented per-upload or per-GB charge**. | Files cannot hold repo text, PDFs, or research documents. Storage is free; tokens appear only when an image is referenced. |
| S10 | Vision is **`deepseek-flash` only**; `deepseek-v4-pro` does not process images. Images only in `user` messages (system/assistant images ⇒ 400). ≤600 images/request; body 48 MiB; ≤32 MiB per image by URL/base64, ≤64 MiB via `file_id`. | Any vision roadmap is Flash-only and user-turn-only — a content-part model, not a boolean. |

---

## 2. What HiveMind actually sends today (READ)

This is the input to §3. Every row is a byte source in the request, in the order it is assembled.

### 2.1 The three request shapes

| Shape | Assembled at | Bytes, in order |
|---|---|---|
| **Planner** (one per run, non-streaming) | `provider_executor.rs:296-318` | `system`: hard-coded `PLANNER_SYSTEM` (`:25-29`); `user`: `"Create the execution-plan DAG for this user task. Return JSON only.\n\nTask:\n"` + `planning_prompt`. Flags: thinking enabled, effort from env, `max_tokens=16384`, `response_format=json_object`. **No tools.** |
| **Leaf** (per task, streaming) | `provider_executor.rs:330-344` | `system`: persona system text, else hard-coded `TASK_SYSTEM` (`:31-33`); `user`: `task_prompt(...)` = `"Original user prompt:\n{prompt}\n\nAssigned role: {role}\nSpawn depth: {depth}\nReturn the final answer for this task only."` (`:444-450`); `tools`: the tier allowlist's 9 Worker specs (`tool_loop.rs:206-219`, `tool.rs:408-421`); `tool_choice=auto`. |
| **Continuation** (tool turn) | `provider_executor.rs:362-429` | Rebuilds `[system, user]` **identically**, then per prior turn: an `assistant` message whose `tool_calls[].function.arguments` are **re-serialised from parsed JSON** (`:386-392`) and whose `reasoning_content` is **always set** (`:402`), followed by one `role:"tool"` message per bounded result. |

`prompt` in the leaf is *not* the run prompt: it is `run.prompt` **with a scoped-facts block appended**
(`execution.rs:2755-2756` → `:53-58` → `context.rs:66-95`).

### 2.2 The facts block — where reuse currently dies

`ScopedContext::prompt_block` renders:

```
Scoped facts for this branch (authoritative):
- parent:0192f3a1-7c4e-7b2a-…-3d5c8e6a1b02:style = honeycomb
- memory:code-reviewer:last_finding = …
```

Four properties are load-bearing:

1. **A random UUID sits inside the prompt text** (`context.rs:105-111` `fact_key` uses `parent_prefix`
   = `parent:<TaskId>:`). Any two tasks — even identical siblings — therefore have different fact
   bytes, and the shared prefix ends at the first fact line.
2. **Facts are emitted in `BTreeMap` key order** (`context.rs:20-29`, `:72`), so the *order* of the
   same fact set differs by UUID value. A stable semantic set still produces unstable bytes.
3. The block is capped at **4 000 chars** (`execution.rs:54`), so it can be a large fraction of a
   small task's prompt — exactly the part worth caching.
4. Memory rows are injected per task at creation (`execution.rs:865-886`: `memory:<agent_key>:<key>`
   from the newest 8 rows), so **two tasks in one run can read different memory** and diverge
   mid-run, invalidating everything after the divergence for that call.

### 2.3 The planner's prompt is a `Debug`-formatted template

`planning_prompt` renders the planner template and then flattens **both** messages into one string
with `format!("{:?}: {}", message.role, message.content)` (`execution.rs:1552-1565`), which
`plan()` then appends after `"…Task:\n"`. Consequences:

- the planner template's *system* text arrives inside the **user** message, labelled `System: ` —
  role semantics are lost and the instruction set is duplicated (`PLANNER_SYSTEM` + template text);
- the planner's persona never occupies the system slot, so the planner call cannot share the leaf
  system block;
- the stable part of that string (`System: You are the HiveMind planner…\nUser: Plan the run for: `)
  is a fine cache head **until** `{{ task }}` interpolates (assets/prompts/planner.prompty:20) — so
  keep it last, which it already is.

### 2.4 What is already cache-friendly (and must not be "improved" away)

| Property | Where | Why it matters |
|---|---|---|
| Stable system text per persona | `http.rs:258-269` (persona map), `provider_executor.rs:82-92` | Identical bytes for every task of that persona → cross-agent unit. |
| Stable tool array per tier | `tool.rs:391-430` allowlist order, `tool_loop.rs:206-219`, `serde_json` `preserve_order` (`backend/Cargo.toml:21`) | Same specs, same order, every call of that tier. Reordering or conditionally enabling tools invalidates the head. |
| No clock, run id, or cost number in any prompt | verified by reading `provider_executor.rs`, `context.rs`, `execution.rs:1552-1565` | The head is time-invariant; only UUIDs (above) poison it. |
| Append-only continuation transcript | `provider_executor.rs:378-410` | The correct shape for `A+B` → `A+B+C` (C3). |

### 2.5 What is not: exact defects, with the fix

| # | Defect | Seam | Effect on reuse | Fix |
|---|---|---|---|---|
| P1 | UUID in model-visible fact keys | `context.rs:105-111`, `:66-95` | Kills the fact block as a unit; every task diverges at line 1 of the block | Render **semantic keys only** (`branch:style`), keep the owning `TaskId` in the storage layer, not the prompt. Put the block **after** the run prompt. |
| P2 | Facts in UUID-sorted order | `context.rs:20-29`, `:72` | Same semantic set ⇒ different bytes across tasks | Sort by semantic key; render from a canonical struct, not a map of prefixed strings. |
| P3 | Re-serialised tool-call arguments on replay | `provider_executor.rs:386-392` | The assistant turn is **not byte-identical** to what the model emitted (`{"path": "a"}` → `{"path":"a"}`), so the model-output unit cannot be matched; turn *n ≥ 3* pays full miss | Persist the raw `FunctionCall.arguments` string and replay it verbatim. Add a test that the replayed bytes equal the captured bytes. |
| P4 | `reasoning_content` always set, even empty | `provider_executor.rs:402` | Injects content the model never produced (`"reasoning_content":""`) and perturbs the assistant turn | Set only when the provider returned a non-empty value *and* thinking was enabled for that turn. |
| P5 | Per-task memory read at creation | `execution.rs:865-886`, `:2755` | Two tasks of one run may read different memory → mid-run divergence (C2 ⇒ whole-tail miss) | Freeze a **run-scoped memory snapshot** at plan time; all tasks read the frozen snapshot; new snapshots only at run boundaries or operator action. |
| P6 | Estimator on the JSON body | `provider_executor.rs:436-442` | Counts JSON keys/braces that are not in the prompt; misses dense content (measured §4.2) | Tokenize the rendered prompt (§4). |
| P7 | History/UI shows token splits with no completeness | `http.rs:842-875`, `ModelsPanel.tsx:174-189` | A number is displayed that may be an undercount of an INCOMPLETE run | Carry `status` + `calls_measured/calls_total`; render "INCOMPLETE" instead of digits (§7). |

No defect above requires a protocol change, a second endpoint, or a new provider feature. All seven
are local, testable, and cheap.

---

## 3. The cache-prefix design

### 3.1 The model to design against

Forget "overlap". Under C2–C5 the cache is a set of immutable prefix units, and a request either

- **fully matches** a unit (hit for that unit's tokens), or
- teaches the system a new unit — at the end of its user input, at the end of the model output, or as
  a detected common prefix — or
- carves one at a fixed interval inside a long stretch.

Two consequences drive the whole design:

1. **Ordering is the only real lever.** If a volatile byte appears before a stable byte, the stable
   byte cannot be part of any unit shared by two requests. There is no "mostly stable" partial hit.
2. **Units are learned, not declared.** A head that two calls share becomes a unit only after the
   system observes the common prefix (C4). So the design targets *repetition of an identical head
   across many calls* — the hive-wide head is hit by call ~3 onward, not call 1.

### 3.2 Canonical prefix order (exact, head → tail)

One `system` message, then one `user` message, then the transcript. Sections **inside** the system
message are ordered L1→L3; nothing volatile may appear before L4.

```
L0  tools[]                     (top-level field, not a message)
L1  GLOBAL block                identity · safety · output discipline · JSON-mode hint when applicable
L2  PERSONA block               persona id identity · tool policy · refusal policy · output schema
L3  MEMORY SNAPSHOT block       frozen, versioned, semantic-keyed, bounded
L4  RUN CONTEXT (user msg)      run prompt (byte-identical for the whole run)
L5  TASK ASSIGNMENT             role · depth · task instruction
L6  TRANSCRIPT                  assistant/tool turns, append-only, raw provider bytes
L7  TAIL DIRECTIVE              "return the final answer for this task only" — always last
```

Rules, in priority order:

- **R1 — one system message, three ordered sections.** Splitting L1–L3 into three `system` messages
  would let L1 be shared alone, but multiple leading system messages on Chat Completions are **not
  documented** (A1). A single message keeps the same token prefix (the divergence point is inside the
  message either way) with zero unverified dependencies and fewer boundary tokens (measured ≈2
  tokens per boundary, §4.2).
- **R2 — L1 is hive-wide and immutable.** Same bytes for planner, superagents, workers and
  subagents. Bump `PREFIX_VERSION` on edit and treat a bump as a deliberate cache reset.
- **R3 — L2 is per persona id, not per display name.** Today's lookup normalises case and `-`/space
  (`provider_executor.rs:82-92`), which is good; but UI-created roles arrive Title-Cased while the
  persona map is lowercase-id keyed (09 §1 Q6). Canonicalise **once**, at the message builder.
- **R4 — L3 is a frozen snapshot, replaced wholesale.** Never mutate a fact mid-run (P5). Name it by
  content (`snapshot:sha256:…`) so a changed snapshot is visibly a different unit.
- **R5 — L4 precedes L5.** The run prompt is the largest text shared by every task in a run; it must
  come before role/depth so the shared head survives. Today's `task_prompt` already puts it first —
  keep that, and move the facts block (L3) *out* of it.
- **R6 — no identifiers in L1–L3.** No `TaskId`, `RunId`, clock, cost, attempt number, or random
  token. If a value can change between two calls of the same shape, it belongs at L5 or later.
- **R7 — tools are versioned with L1.** A tool added, removed, reordered, re-described or re-schema'd
  changes prompt bytes in a position HiveMind does not control (A2). Keep the allowlist fixed per
  tier and hash it.
- **R8 — transcript bytes are the provider's bytes.** Replay `arguments` verbatim (P3) and only the
  `reasoning_content` the provider produced (P4).
- **R9 — the estimator and the ledger both record `prefix_fingerprint`** (see §6.4), so a hit-rate
  change can be attributed to a specific arrangement rather than guessed.

### 3.3 The five sharing tiers this produces

| Unit | Bytes | Shared by | Learned by |
|---|---|---|---|
| **U-global** | L0 + L1 | every call in the hive, planner included (once the planner uses L1 — see §3.6) | common-prefix detection across the first two calls |
| **U-tier** | U-global + tier tool array | all calls of one agent tier | as above (tools are inside L0) |
| **U-persona** | U-tier + L2 | all calls of one persona (e.g. every `code-reviewer`) | as above |
| **U-run** | U-persona + L3 + L4 | every task of one run under that persona | common-prefix detection across the run's first two tasks (their user inputs diverge only at L5) |
| **U-task** | U-run + L5 | one task's own turns | the request-boundary unit at the end of that task's user input, plus the model-output unit |

The only unit with a guaranteed hit today is **U-task** (each continuation reproduces its own
earlier prefix). U-persona/U-tier/U-global are exactly what C4 has to learn — and they are the units
worth engineering, because at 150 agents they are shared by hundreds of calls per run.

### 3.4 Invalidation matrix

The rule: a change at layer *N* invalidates *N and everything after it* for every request whose
bytes change.

| Change | Invalidates | Reusable after | Notes |
|---|---|---|---|
| L1 wording edit | every unit in the hive | next calls (cold) | Batch all L1 edits into one version bump. Never edit L1 per run. |
| L2 persona edit | that persona's units | next calls of that persona | Other personas keep U-tier. |
| L3 snapshot replaced | U-run for that run | calls after the replacement | Freeze at plan time so it happens once per run, not once per task. |
| A fact written mid-run | the whole tail for every later call in that run | — | **Forbidden** by R4; the current code does this (P5). |
| L4 run prompt (never changes within a run) | new run only | every task of the new run | Runs never share a U-run unit; that is expected. |
| Tool list / order / description / schema | U-tier and deeper, for that tier | next calls of that tier | Hash the array; assert stability in a test. |
| Replayed `arguments` re-serialised | every unit from the assistant turn onward | — | Fix P3 and the model-output unit becomes reusable. |
| `reasoning_content` invented as `""` | same | — | Fix P4. |
| Model id change (`flash` ↔ `pro`) | everything | — | Different model ⇒ different cache; also different price row. |
| `user_id` change | everything (KVCache is isolated per `user_id`) | — | Only if a per-hive `user_id` policy is adopted (§3.7). |
| Cache eviction (hours–days, C7) | everything | next calls (cold) | Normal. The ledger must show cold starts, not hide them. |

### 3.5 What cross-agent reuse is *correct*

Cross-agent reuse is byte sharing, not knowledge sharing. It is correct when all of:

1. the same account key (one key, `budget.rs:52`);
2. the same model id;
3. the same `user_id` (default: none);
4. the same L1–L3 bytes, i.e. same persona and same memory snapshot;
5. the same tool array (same tier).

It is **incorrect** — as a product decision, not a technical one — when two hives must not share
cache state for privacy. C9 makes that enforceable (`user_id` per hive) at the price of reuse. The
default should be **reuse** (one operator, one machine, one account); the setting should be
per-hive isolation with a UI note that it costs input tokens.

Note also what reuse cannot do: results are never reused. Two agents asking the same question still
pay output tokens twice. The cache saves **input** tokens only (T9: 50×/30× cheaper), so the design
target is "long head, short tail", not "dedupe work".

### 3.6 Planner and subagent specifics

- **Planner.** Fix the `Debug`-flattening (`execution.rs:1552-1565`): emit
  `[system: L1 + planner section][user: "Create the execution-plan DAG…\n\nTask:\n" + run prompt]`.
  That gives the planner U-global sharing with leaves and removes the duplicated instruction set.
  Keep the template's `{{ task }}` interpolation last (it already is) so the template text is head.
- **SuperAgent / Subagent tiers.** Same L1; L2 differs by persona; tools differ by tier, so they are
  separate U-tier units. Do **not** give Subagents a shorter L1 — that would fragment U-global.
- **Watchdog/Remote agents.** If they run on a timer, their prompt is nearly all L1 (cheap hits) —
  this is the cheapest possible agent shape and fits the operator's "one job forever" description.

### 3.7 `user_id`: the trade-off, stated once

| Option | Reuse | Privacy / isolation | Recommendation |
|---|---|---|---|
| No `user_id` (today; `ChatRequest` has no field, `dto.rs:396-422`) | maximal cross-hive reuse | none beyond the account | **Default.** |
| Per-hive `user_id` | cross-hive reuse forfeited (C9) | per-hive KVCache + scheduling isolation | Opt-in setting, surfaced with its cost. |
| Per-agent `user_id` | almost no reuse | maximal isolation | Do not offer: it converts the cheapest tokens in the product into misses. |

Also note C9's scheduling rule: `user_id` concurrency splitting applies **only** to accounts with
increased quotas. On a standard account, `user_id` buys no concurrency and only costs cache.

### 3.8 Non-goals (do not do these to "improve caching")

- Do not switch to the Responses API or Anthropic Messages "for caching" — cache accounting is a
  property of the account/model, not the protocol, and both add a second parser (§01 §10.4).
- Do not add `prompt_cache_key`-style fields: not documented on Chat Completions (only mentioned as
  *ignored* on Responses, 01 §7.2).
- Do not move volatile data earlier to "keep prompts similar". Similar is not matched (C2).
- Do not claim a hit rate from theory. Measure (§7) or say UNVERIFIED.

---

## 4. Tokenizer and prompt-token measurement

### 4.1 Why `bytes/4` must go

`provider_executor.rs:436-442`:

```rust
fn estimate_prompt_tokens(request: &ChatRequest) -> Option<u64> {
    let bytes = serde_json::to_vec(request).ok()?;
    Some(((bytes.len() as u64) / 4).max(1))
}
```

Two independent errors are baked in:

1. **It measures the wrong object.** The serialised JSON body contains key names, quotes, commas and
   `\n` escapes that are not prompt tokens. Measured on the exact planner request: **894 body bytes
   for 653 bytes of message text** — ≈240 bytes (≈60 tokens at `bytes/4`) counted against the budget
   that the model never sees. The prompt the model sees is the **chat-template rendering**, not the
   JSON.
2. **A constant tokens-per-byte assumption cannot hold across content classes.** UUIDs, JSON, code
   and CJK tokenize at very different densities (MEASURED, §4.2).

The direction of the error is not constant, so the 400 000-token preflight (and any future context
preflight) is neither conservative nor aggressive — it is unpredictable. 06's T22 and 01's §14 gap 1
both flagged this; §4.2 quantifies it.

### 4.2 MEASURED: the official tokenizer against HiveMind's real prompt strings

**Method (reproducible).**
`https://cdn.deepseek.com/api-docs/deepseek_v4_tokenizer.zip` was downloaded on 2026-11-07
(zip SHA-256 `E7310D1D…6FE495`), expanded, and `tokenizer.json` (6 367 096 bytes, SHA-256
`89085F12…DC9614`) loaded with the HuggingFace `tokenizers` Python wheel **0.23.3** (the same Rust
core as the `tokenizers` crate). Strings were copied verbatim from
`provider_executor.rs:25-33`, `:444-450` and `context.rs:66-95`. `bytes4` is `len(bytes) // 4`, i.e.
exactly the current heuristic; `err` is `(bytes4 − tokens) / tokens`.

| String (verbatim from source) | bytes | tokens | tok/byte | bytes/4 | err |
|---|---|---|---|---|---|
| `TASK_SYSTEM` (fallback leaf system) | 218 | 48 | 0.220 | 54 | **+12.5 %** |
| `PLANNER_SYSTEM` | 397 | 108 | 0.272 | 99 | −8.3 % |
| Run prompt, English prose | 181 | 41 | 0.227 | 45 | +9.8 % |
| **Facts block** (4 lines, UUID keys) | 339 | 159 | **0.469** | 84 | **−47.2 %** |
| **Assembled leaf user message** (prompt + facts + role/depth) | 632 | 228 | 0.361 | 158 | **−30.7 %** |
| Planner user message | 256 | 58 | 0.227 | 64 | +10.3 % |
| Tool specs JSON (2 representative specs) | 570 | 123 | 0.216 | 142 | +15.4 % |
| **Rust code** | 162 | 54 | **0.333** | 40 | **−25.9 %** |
| CJK sentence | 121 | 24 | 0.198 | 30 | +25.0 % |
| **Full planner request body** (exact serde order, no tools) | 894 | **169** (prompt) | — | **223** | **+32.0 %** |

Reading of the table:

- **Undercount is real and concentrated exactly where HiveMind is dense.** The facts block (−47 %),
  the assembled leaf user message (−31 %) and code (−26 %) are all *beneath* the gate that is
  supposed to stop them. Solving `estimate = (1 + err) × true` for a 400 000-token projection gives
  **≈577 000 real tokens** at the assembled-leaf density and **≈758 000** at the facts-block density —
  a prompt can be 1.4–1.9× over the budget that admitted it.
- **Overcount happens on sparse text** (+10 % prose, +25 % CJK) and on the JSON envelope (+32 % for a
  short planner body). Overcount is safe for a gate but makes "budget remaining" pessimistic and the
  UI's pre-send estimate wrong in the other direction.
- **The documented ratios are a different lever from the tokenizer.** T1 says ≈0.3 token per English
  character; the measured prose density is **0.22** tokens/byte, i.e. the documented ratio is
  conservative for plain prose and unsafe for code/UUIDs. Use the tokenizer; keep the ratio only as a
  documented fallback with a stated multiplier.
- **The chat-template envelope is small.** A literal application of the template from
  `tokenizer_config.json` (`bos` + system + `<｜User｜>` + user + `<｜Assistant｜>`) costs single-digit
  tokens: `<｜begin▁of▁sentence｜>` = 1 token (id 0), `<｜User｜>` = 1 (id 128803),
  `<｜Assistant｜>` = 1 (id 128804), `<｜end▁of▁sentence｜>` = 1 (id 1). (Note `<｜Tool｜>` is **not** a
  single added token — it encodes to 5 — so do not hand-build tool turns by string concatenation;
  tokenize the text, never synthesise marker strings.) The envelope is **not** the error source; the
  JSON body and content density are.

**What the artifact actually is (MEASURED).** The zip contains
`deepseek_tokenizer.py` (a short demo that calls
`transformers.AutoTokenizer.from_pretrained(..., trust_remote_code=True)`),
`tokenizer.json` (HuggingFace `tokenizers` format, BPE, vocab 129 280) and `tokenizer_config.json`
(`tokenizer_class: LlamaTokenizerFast`, `add_bos_token/add_eos_token: false`,
`model_max_length: 16384`, and a full Jinja `chat_template`). Two traps:

- **`model_max_length: 16384` is a tokenizer-config default, not the model context.** T5 documents 1M.
  Never derive the context window from this file.
- The config is a *shared/Llama-style* artifact; its fidelity for `deepseek-flash` is an
  **estimate** until calibrated against `usage.prompt_tokens` (A9). The provider's counter remains
  the only billing truth (T1).

### 4.3 Acquisition plan (vendored, hashed, embedded, pinned)

1. **Vendor the artifact** at `backend/crates/hiverouter-provider/assets/tokenizer/`:
   `tokenizer.json`, `tokenizer_config.json`, `deepseek_tokenizer.py`, plus a `PROVENANCE.md`
   recording `source_url`, `fetched_at=2026-11-07`, `zip_sha256`, `tokenizer_json_sha256`, byte size,
   and the upstream licence as shipped in the zip. (Licence text is not included in the zip as read;
   record "not stated in artifact" rather than guessing — A10.)
2. **Embed** with `include_bytes!` (the release is a single portable binary; prompt assets already use
   `include_str!`, `prompts.rs:38-73`). Cost: **+6.4 MB** binary. Acceptable; state it in the release
   notes. If unacceptable, ship as a sidecar with an integrity check and refuse live calls with
   `E_TOKENIZER_UNAVAILABLE` when it is missing — never silently fall back to `bytes/4` for gating.
3. **Dependency:** the `tokenizers` crate (Apache-2.0 upstream; verify the exact pinned version and
   its regex backend before adding — the wheel I validated is 0.23.3, the crate version must be
   pinned per §28 of the contract). Load `tokenizer.json` once at boot into an `Arc<Tokenizer>`.
4. **Golden fixtures:** every row of the §4.2 table becomes a unit test with the exact string and the
   exact token count (48 / 108 / 41 / 159 / 228 / 58 / 123 / 54 / 24 / 169). If the Rust crate
   disagrees on any row, the acquisition has failed — do not ship the dependency.
5. **Rendering, not concatenation:** produce the prompt text by applying the documented template
   mechanically (`bos + system + "<｜User｜>" + user + "<｜Assistant｜>"` for a single turn; for tool
   turns, render the assistant/tool sections per the config's `chat_template`) and tokenize that. Keep
   the renderer in one function so its output is testable; do **not** hand-assemble marker strings
   (see the `<｜Tool｜>` note).
6. **Fail-closed at boot:** a hash mismatch or a missing asset is `E_TOKENIZER_UNAVAILABLE` at start,
   not a runtime surprise.

### 4.4 Estimator contract

```
estimate_prompt_tokens(request) =
    tokenize(render_chat_template(request.messages))
  + tokenized_tools_bytes(request.tools)          // count once, from the same renderer
  // provenance: Assumed. Never written to a settled cost (accounting.rs V3/V4).
```

Gating rule: `gate_tokens = ceil(estimate × SAFETY)` with `SAFETY = 1.10` for pre-send *refusals*
(a 10 % margin covers the shared-artifact uncertainty and the interval-unit unknowns), while the
**displayed** estimate stays unadjusted and labelled `Assumed`. The margin is a policy number, not a
measurement — record it as such and revisit after calibration (A9).

**Calibration loop (this is the part that makes it accurate):** for every settled call store
`estimated_prompt_tokens` and `measured_prompt_tokens` (= reported `prompt_tokens`) and expose the
ratio. Gate health monitor: p50 ratio should sit in 0.97–1.10; if p95 > 1.05 (estimating under), the
margin is raised or the estimator is fixed; if p50 < 0.9 (over-estimating), the margin is too wide but
the gate is safe. A run with no calibration samples yet must say so in the UI — an uncalibrated
estimator is **Assumed**, not measured.

### 4.5 Retirement criteria for the ratio fallback

Keep a fallback path (`0.3 tokens/English char`, `0.6/Chinese char`, documented T1, × 1.25 safety) for
the case where the tokenizer asset is unavailable and the operator insists on running. It must be
labelled `Assumed(ratio)` and it must **not** be used for a claim about cost. Retire the fallback when
two consecutive weeks of calibration show the tokenizer within ±5 % of provider counters.

---

## 5. Preflight: context window and max output

Two different gates are currently conflated (the 400 000-token *budget* gate); neither the context
window nor the projected cost is checked (`01` §14 gaps 1–2; `06` T22).

### 5.1 The formula

```
ctx_limit      = min( MODEL_CONTEXT_ASSUMED , models[id].context_window when reported )
                 where MODEL_CONTEXT_ASSUMED = 1_000_000    # documented "1M" (T5); integer UNVERIFIED (A11)
max_out        = request.max_tokens or documented default (8K / 64K / 128K at effort=max)  # T4
needed         = gate_tokens(estimate §4.4) + max_out
refuse         = needed > ctx_limit                        → E_CONTEXT_WINDOW_EXCEEDED
                 max_out > MAX_OUTPUT_TOKENS (393216)      → E_SCHEMA_INVALID   (exists, dto.rs:546-553)
                 max_out > models[id].max_output_tokens    → E_SCHEMA_INVALID   (when reported)
```

Because the context counts **input + output** (T5, T8), a prompt that fits with `max_tokens=1024` may
not fit with `max_tokens=16384`. That is why the check takes the actual request, not a size class.

### 5.2 Where it runs

| Gate | Location | Must hold |
|---|---|---|
| Request schema | `dto.rs:537-575` `validate()` | already refuses empty messages, out-of-range `max_tokens`, forced tool in thinking |
| **Context window** | new `preflight_context()` called **before** `client.complete*/stream*` | zero HTTP requests on refusal; assert with a wiremock request-count test |
| **Projected cost** | extend `RunBudget::preflight` (`budget.rs:235-284`) | refuses on `spent + projected > cap`, not only `spent >= cap` |
| Stream open | `client.rs:562-607` | preflight runs per attempt already (`:576-579`); add the context gate before the loop to avoid spending attempts on a request that can never fit |

Refusal text must carry the numbers (`prompt≈N + max_tokens=M > context=1 000 000`), because the
operator's next action (shrink the prompt, the cap, or the model) depends on which side overflowed.

### 5.3 The 400 trap this closes

A 400 from an over-context request is currently reported as `E_PROVIDER_INVALID_RESPONSE`
("undocumented provider HTTP status", `client.rs:520-523`). After preflight the only way to reach a
400 should be a *real* client bug, so map 400/422 to a request-schema refusal naming the documented
cause (also `06` T25) — otherwise a preflight regression is invisible.

---

## 6. Measuring and exposing cache hits and cost (per run / hive / agent)

### 6.1 The rule

Nothing about cost or cache is claimed unless it comes from provider counters, is stored with
provenance, and is aggregated exactly. The current system satisfies the first (V1–V6 in
`accounting.rs:1-16`) and fails the rest:

| Gap | Evidence | Consequence |
|---|---|---|
| No per-call record | `accounting.rs:205-293` returns a `CallCost` that the executor **drops** (`provider_executor.rs:233-243`) | cannot attribute cost to an agent, a tier, or a peak class after the fact |
| No money on disk | `run_usage` has six token columns only (`db.rs:198-217`) | "what did this hive spend?" is unanswerable |
| No completeness on the read path | `http.rs:842-875` returns digits for an INCOMPLETE run; `ModelsPanel.tsx:174-189` renders them | the UI can show a number that the accounting layer refused to settle |
| Aggregate can undercount silently | `Usage::checked_add` maps `(Some, None) → Some` (`usage.rs:52-71`) while the doc-comment says otherwise | a run total can omit an unknown call yet still look numeric |
| No hive concept | no `hive_id` anywhere in `runs`/`run_usage` | "cost per hive" has nothing to key on |
| No request-billed dimension | `PriceRow` = hit/miss/output only (`pricing.rs:47-64`) | Search/Fetch cannot be costed |

### 6.2 Schema (DESIGN) — migration version 11 and 12, `db.rs`

```sql
-- v11: one row per provider call (streaming and non-streaming alike)
CREATE TABLE provider_calls (
  id            TEXT PRIMARY KEY,             -- uuid v7
  run_id        TEXT NOT NULL,
  task_id       TEXT NOT NULL,                -- agent identity
  hive_id       TEXT NOT NULL,                -- see §6.3
  agent_key     TEXT NOT NULL,                -- role/persona key, the cache-sharing key
  tier          TEXT NOT NULL,                -- hyperagent|superagent|worker|subagent|tool
  model         TEXT NOT NULL,
  peak_class    TEXT NOT NULL,                -- peak|off_peak (pricing.rs:28-45)
  call_kind     TEXT NOT NULL,                -- planner|leaf|continuation
  turn_index    INTEGER NOT NULL,
  attempt_index INTEGER NOT NULL,             -- retries counted separately
  prefix_fp     TEXT NOT NULL,                -- sha256(model|tools|L1|L2|L3|L4)  §6.4
  est_prompt_tokens     INTEGER,              -- Assumed
  prompt_tokens         INTEGER,              -- Measured (NULL stays NULL)
  cache_hit_tokens      INTEGER,
  cache_miss_tokens     INTEGER,
  completion_tokens     INTEGER,
  reasoning_tokens      INTEGER,
  amount_nano_usd       INTEGER,              -- exact 1e-9 USD, see §6.5
  amount_usd            TEXT,                 -- decimal string for display
  price_source_url      TEXT NOT NULL,
  price_fetched_at      TEXT NOT NULL,
  usage_provenance      TEXT NOT NULL,        -- measured|unknown
  amount_provenance     TEXT NOT NULL,        -- derived|unknown
  created_at            TEXT NOT NULL
);
CREATE INDEX idx_provider_calls_run  ON provider_calls(run_id);
CREATE INDEX idx_provider_calls_hive ON provider_calls(hive_id, created_at);

-- v12: non-token charges (Search/Fetch, §8) — same ledger, different quantum
CREATE TABLE request_charges (
  id            TEXT PRIMARY KEY,
  run_id        TEXT NOT NULL,
  task_id       TEXT NOT NULL,
  hive_id       TEXT NOT NULL,
  surface       TEXT NOT NULL,                -- search|fetch
  http_status   INTEGER NOT NULL,
  billable      INTEGER NOT NULL,             -- 1 only on 200 + produced result (S2)
  units         INTEGER NOT NULL,             -- 0 or 1
  amount_nano_usd INTEGER NOT NULL,           -- 750000 per billable unit
  price_source_url TEXT NOT NULL,
  price_fetched_at TEXT NOT NULL,
  created_at    TEXT NOT NULL
);
```

`run_usage` stays as the cheap rollup, but it must gain `status TEXT NOT NULL` and
`calls_measured INTEGER`, `calls_total INTEGER` so the read path can refuse to print digits.

### 6.3 What "hive" means (a definition, because the schema has none)

`hive_id` = **the run's root branch**: `runs.project_id` + the HyperAgent task id of that run. When a
run has a single root (today's shape: one planner task at depth 0, `execution.rs:1321-1335`) that is
`run:<run_id>`. When a SuperAgent branch owns a subtree (the target HMARS), `hive_id` =
`<run_id>/<superagent task id>`. Rules:

- every task carries the `hive_id` of its nearest SuperAgent ancestor, else the run root;
- a cost rollup by `hive_id` is therefore a strict partition of the run total (no double counting);
- "cost per agent" is `task_id`; "cost per role" is `agent_key`; "cost per cache unit" is `prefix_fp`.

### 6.4 `prefix_fp` — the attribution key that makes cache work measurable

```
prefix_fp = sha256( model_id || tools_sha256 || sha256(L1) || sha256(L2) || sha256(L3) || sha256(L4) )
```

Computed in the message builder (one place), stored per call, and shown in the cost panel. Then:

- hit-rate by `prefix_fp` proves which arrangement wins (e.g. before/after the UUID fix);
- a `prefix_fp` that changes mid-run is a defect the ledger can see (P5-class bug) — alert on it;
- **no A/B claim needs a live provider feature**: two arrangements are two `prefix_fp` values.

`hiverouter_tools::sha256_hex` already exists (`tool_loop.rs:61` calls it), but note the layering:
`hiverouter-provider/Cargo.toml:13-24` does **not** depend on `hiverouter-tools` and has no hash crate,
so the fingerprint must be computed where the messages are assembled — `hiverouter-core` (which
depends on both) — and passed into the request/ledger, or the provider gains a pinned `sha2`
dependency. Do not duplicate a hash implementation.

### 6.5 Money on disk, exactly

- Cost arithmetic stays `Decimal` (`accounting.rs:329-335`, no f64 on cost paths).
- Storage adds an **integer nano-USD** column for aggregation because SQLite `SUM()` on TEXT decimal
  is wrong and floats are banned. Exactness check: the smallest real charge is one Flash cache-hit
  token = `0.003 / 1e6` USD = **3 nano-USD** → representable. Largest plausible row (1e9 tokens ×
  $1.32/1M) = 1.32e12 nano-USD → far inside i64. The display string is the `Decimal.normalize()`
  value, the same one `report()` exports today (`accounting.rs:312-316`).
- Aggregation is done **in Rust with Decimal over the nano-USD column**, never `f64`, never
  `SUM(TEXT)`. A test pins `run → agent → hive` sums to the same decimal as the in-memory ledger.

### 6.6 Definitions the UI must use (and must not blur)

| Metric | Definition | Gate |
|---|---|---|
| **Cache hit rate** | `Σ hit / (Σ hit + Σ miss)` per scope | only when every call in scope is measured |
| **Effective input price** | `(Σ hit·hit_price + Σ miss·miss_price) / Σ prompt_tokens` | same |
| **Cache saving (derived)** | `Σ hit · (miss_price − hit_price)`, labelled `derived` | only when `COMPLETE` |
| **Cost per task / agent / hive / day** | Decimal sum over `provider_calls` + `request_charges` | only when `COMPLETE` |
| **Cost status** | `COMPLETE` / `INCOMPLETE(+reason)` from `accounting.rs:56-73` | always shown next to any money |
| **Cold-start window** | calls whose hit rate is 0 within 10 min of process start or after > 1 h idle | excludes design failures from "cache broken" |
| **Estimate ratio** | `measured / estimated` (p50, p95) | labelled `Assumed`; never a cost input |

### 6.7 Routes and UI (DESIGN, with seams)

| Surface | Route / seam | Content | Acceptance |
|---|---|---|---|
| Run cost | `GET /api/cost/runs/{run_id}` (new, `http.rs:514-528` router) | totals, status, per-`call_kind`, per-`agent_key`, per-`prefix_fp`, cold-start flag | 200 with `status:INCOMPLETE` and **null** money when any call is unmeasured; a wiremock test pins it |
| Agent cost | `GET /api/cost/tasks/{task_id}` | per-call rows for one agent | sums to the run's per-agent line |
| Hive cost | `GET /api/cost/hives?from&to` | partition by `hive_id`, plus day totals | partitions sum exactly to the global total |
| Live cost frame | extend `RUN_METRICS` (`event.rs:273-284`) with `cost_status`, `spent_usd`, `cache_hit_tokens`, `cache_miss_tokens`, `calls_measured/calls_total` | contract change ⇒ `CHANGE-LEDGER-IMPL.md` row + schema version bump | client decodes, UI shows live meter |
| Budget state | `GET /api/budget` + `POST /api/budget/{arm,disarm,kill,rearm}` | mode, guard state, caps, kill switch | kill switch blocks the next preflight with `E_SPEND_HALTED` |
| Usage truth | harden `GET /api/usage/runs/{id}` (`http.rs:842-875`) | add `status`, `calls_measured`, `calls_total` | never returns numbers for an INCOMPLETE run |
| Cost panel | new `NavSection 'cost'` (`store.ts:62`), `NavRail.tsx:9-16`, `App.tsx:89-102`, new `web/src/panels/CostPanel.tsx` | hive/agent/run rollups, cache hit rate, effective input price, savings (derived), estimate ratio | screenshots at 1440×900; empty state says "no settled cost" |
| Agent card | `AgentCard` cost chip (08 §4) | per-agent settled cost or `—` | never a fabricated 0 |
| Models panel | `ModelsPanel.tsx:174-189` | add status + cost rows next to hit/miss | INCOMPLETE renders words, not digits |

**Order of work:** persist per-call rows first (they are the only source), then the read routes, then
the UI. A UI built before the ledger would have to invent numbers — which is the defect this lane
exists to remove.

---

## 7. Budget policy: no hard default cap, an opt-in cap, a soft guard, and a kill switch

### 7.1 The conflict, stated plainly

Operator (OPERATOR-DECISIONS §Budget, §DeepSeek): **no hard default budget**, caching is the goal, a
visible soft spend guard / emergency stop is acceptable, and **denial-of-wallet must still be designed
out**. Code today (`budget.rs:24-43`): a hard **$0.50** run cap, **400 000** combined tokens, **40**
calls, **64** agents. Those four constants are precisely what refuses a 150-agent run (`03` §8.3:
3.3–15× short). "No default budget" and "keep the current caps" cannot both hold.

Resolution: the money constant is removed from the default path and replaced by a bound that is not a
dollar guess — **structure + account balance + human-gated escalation** — and dollar caps move to
opt-in presets.

### 7.2 Three modes

| Mode | Money cap | Token cap | Structural caps | Guard behaviour | Default |
|---|---|---|---|---|---|
| **A. Observe** | none | none | **always on**: agents/run, fan-out, depth, calls/run, per-call `max_tokens`, context preflight, in-flight, wall-clock deadline | balance floor + 402 freeze + kill switch; no automatic stop on spend | **yes** |
| **B. Soft guard** | operator-set **soft** threshold | optional | as A | crossing the threshold **pauses new sends** and asks for one click (`E_SPEND_GUARD`); in-flight calls finish or are cancelled per setting | offered at first launch as an explicit choice |
| **C. Hard caps** | operator-set **hard** per-run/per-hive/per-day | operator-set | as A | preflight refuses by name before send; reservation model prevents N-way overshoot | opt-in |

Mode A is only defensible because four things remain hard:

1. **Structural envelope** — a run cannot exceed `agents × (calls/agent) × max_tokens` by construction,
   and every dimension is a named refusal. For the default: 64 agents, `call_cap = max(40, agents+16)`
   per run (the current fixed 40 is smaller than the target agent count), 4 tool turns/task
   (`tool_loop.rs:14`), `max_tokens` 8192 leaf / 16384 planner, plus a **run wall-clock deadline**
   (proposed; not present today — `06` §6.5 W6).
2. **Account anchor** — the balance is the only true ceiling and it is provider-enforced. Read
   `GET /user/balance` (`client.rs:275-277`) at boot and on demand; when `is_available == false` or
   `total_balance` falls below `balance_floor` (default: **5 % of the boot balance**, a *relative*
   floor, not a dollar default), stop new sends with the guard. This is why no invented dollar default
   is needed.
3. **402 freeze** — map 402 to a **new** `E_PROVIDER_BALANCE` (today it is `E_PROVIDER_AUTH`, whose
   remediation says "check DEEPSEEK_API_KEY", `refusal.rs:254-256`; `client.rs:508-511`), do not retry,
   freeze the budget for the process, and surface a top-up link. A test pins request count = 1.
4. **Kill switch** — a single `POST /api/budget/kill` that (a) cancels in-flight runs' tokens, (b)
   blocks every preflight with `E_SPEND_HALTED`, (c) persists the armed state in `app_meta` so a
   restart does not silently re-arm spending, (d) is reachable from a topbar control with no settings
   navigation. Re-arm is an explicit operator action, auditable.

### 7.3 Reservation, not observation (Mode C correctness)

`preflight` reads `spent_usd` (`budget.rs:252-253`). With N agents in flight, all N read the same
`spent` and all N pass — the run can overshoot by N × per-call cost, and the "worst case $1.07"
arithmetic in `06` §6.4 becomes N × that. Mode C therefore needs **reserve-then-settle**:

```
preflight:  reserve = projected_cost(estimate, max_out, peak_now)
            if spent + reserved + reserve > cap → E_BUDGET_EXCEEDED
            reserved += reserve
settle:     reserved -= reserve;  spent += actual (measured usage)
unknown:    keep the reservation; mark INCOMPLETE (usage.rs V1–V6 semantics)
```

`projected_cost` uses the rate class at **send time** (`pricing.rs:191-197`) because prices double in
peak windows (T9). Store the class actually billed (§6.2 `peak_class`).

### 7.4 What changes in code (and what must not)

| Change | Seam | Note |
|---|---|---|
| `BudgetLimits` gains `mode`, optional caps, `balance_floor_pct`, `wall_clock` | `budget.rs:60-82` | `Default` must become Mode A (no money cap) — this is a deliberate behaviour change, so it needs a decision row |
| `preflight` gains projection + reservation + guard/kill/402 checks | `budget.rs:235-284` | keep the "before any HTTP" property and its test (`:554-576`) |
| Refusal texts stop hard-coding $0.50 / 400 000 / 40 | `refusal.rs:152-160` | as written they would *lie* about the active cap |
| New codes: `E_CONTEXT_WINDOW_EXCEEDED`, `E_SPEND_GUARD`, `E_SPEND_HALTED`, `E_PROVIDER_BALANCE`, `E_REQUEST_BUDGET_EXCEEDED`, `E_TOKENIZER_UNAVAILABLE` | `refusal.rs:73-269` table | each needs code + message + remediation (the table is the contract) |
| Per-run `RunBudget` map keyed by run | `provider_executor.rs:39`, `:104-107` | already there; must also expose read-only spend for the meter |
| Kill switch / guard state | new `budget_state` in `app_meta` (`db.rs:36-51`) + routes | must survive restart |

**Do not** simply delete the caps: the operator asked for *no hard default*, not for *no bound*. A
commit that removes `$0.50` without adding the structural envelope + guard + 402 freeze + ledger is a
denial-of-wallet regression, and the acceptance tests in §9 fail it.

### 7.5 Denial-of-wallet scenarios under the new default

| Scenario | Bound under Mode A | Residual |
|---|---|---|
| Injected content spawns max children at every level | agents/run + fan-out + depth + call cap; spawn approval (P0 in 09) | a full 64-agent run is allowed by design; the ledger shows it live |
| Operator restarts a capped run repeatedly | no cap to restart around; per-day soft guard (Mode B) or account floor | Mode A alone has no daily ceiling → **this is why Mode B is offered at first launch** |
| 150-agent soak | in-flight limiter (8 today; must rise with the scheduler) + call cap + deadline | hourly burn is real money; the live meter + kill switch are the control |
| One enormous prompt | context preflight (§5) + tokenizer estimate | estimator error bounded by the calibration loop |
| 402 at the provider | 402 freeze, no retry, `E_PROVIDER_BALANCE` | none |
| Run that never terminates | per-run wall-clock deadline + per-call timeouts | new: deadline does not exist today |
| UI/browser-triggered spend | origin allowlist on provider routes (`http.rs:926-935`); mutations still lack a Host allowlist (`09` Q18) | **P0 security gap, not a budget gap** |

---

## 8. Search / Fetch billing, and the Files / Vision decision

### 8.1 Search / Fetch: the only per-request-priced surface

Documented (S1–S8): $0.75 per 1 000 **successful** requests, per API; billable only on HTTP 200
**with a produced result**; failures, empty result sets and timeouts are free; `count` does not change
the price; QPS 100 (Search) / 25 (Fetch) plus a per-domain Fetch limit.

**What must exist before the first call** (DESIGN):

1. **A per-request price row**, separate from token rows. Do not overload `PriceRow`: add
   `RequestPriceRow { surface, unit: "successful_request", amount_nano_usd: 750_000, source_url,
   fetched_at }` in `pricing.rs`, next to `PricingTable` (`:47-64`, `:93-115`). 0.75/1000 USD =
   exactly **750 000 nano-USD** — inside the exact-integer money design (§6.5).
2. **A billable predicate** that is one function, used by both the executor and the ledger:
   `billable = status == 200 && (surface == Search ? !results.is_empty() : !content.is_empty())`.
   Tests pin every documented non-billable case (403, 404, 408, 413, 429, 5xx, 200-with-empty-results).
3. **Reserve one unit before the call**, settle after (Mode C only; in Mode A the unit is still
   *recorded*, because "no cap" is not "no truth").
4. **A distinct refusal** `E_REQUEST_BUDGET_EXCEEDED` for a per-run/per-day request cap, so the
   operator can tell "token budget" from "search budget".
5. **Approval**: use the existing unused `PermissionClass::Network` (`tool.rs:38`, `:49`) so Search and
   Fetch are approval-gated by default (operator: "gated by approval/budget").
6. **Retry policy from S6**: retry 408/429/5xx only, never 403/404/402/413/400; a retry that
   eventually returns 200 bills **once**.
7. **Content discipline** (cost, not just quality): default `need_content=false` (S3) so a search
   returns ~10 × 600–800-char summaries, not full pages; filter by `rerank_score ≥ 1.5` /
   `authority ≥ 3` before spending prompt tokens (S4); slice Fetch Markdown to a configured cap
   before it enters the prompt (S5 has no documented response-size bound).

**Double cost is real:** a search result enters the prompt as **cache-miss input tokens** *and*
carries a per-request charge. Budget both. A 10-result search at `need_content=false` returns roughly
10 × 600–800 chars ≈ 7 500 chars ≈ 1 900 tokens ≈ $0.00029 (Flash off-peak) **plus** $0.00075 — the
request charge dominates by ≈2.6×. That is the design fact that makes "search once, summarise into
memory, reuse the memory" cheaper than "search per agent".

**Where the cost dimension breaks today:** `RunAccounting::record` only accepts token usage
(`accounting.rs:205-293`), `CostReport` has no non-token field (`:94-116`), and `run_usage` has no
column for it (`db.rs:198-217`). So a Search call today would either be invisible or force a false
`COMPLETE`. Both are unacceptable; the `request_charges` table (§6.2) is the minimum fix.

### 8.2 Files: image-only, therefore not v1's data surface

Decision: **do not adopt Files for v1**, and record why so it is not rediscovered as a "quick win":

- It stores **images only** (JPEG/PNG/GIF/WebP, content-detected); there is no text, PDF or document
  purpose (S9). HiveMind's data surface is a **local text workspace jail** (`tool.rs:364-380`).
- There is **no documented per-upload or per-GB charge** (S9) — storage is free, so there is no cost
  reason to adopt or avoid it.
- The one legitimate future use is **vision transport**: a `file_id` image may be up to 64 MiB and is
  exempt from the 32 MiB inline check (S10), and reuse across requests avoids re-uploading.
- Account-scoped: all keys under the account share files (S9). Single-key product, so no isolation
  work — but also no per-hive segregation if hives are ever multi-tenant.

If adopted later: 25 GiB / 10 000 files / 1 h–30 d expiry are the bounds to enforce in settings, and
`expires_after` should always be set (permanent storage of operator screenshots is a privacy choice,
not a default).

### 8.3 Vision: bounded cost, Flash-only, not v1

Decision: **not in v1**; the operator said "important later, text/code first". The cost facts that
should shape that later decision:

| Fact | Value | Consequence |
|---|---|---|
| Model support | `deepseek-flash` only; `deepseek-v4-pro` does not process images (S10) | a vision task forces the Flash price row, which is 4.4× cheaper than Pro — no cost problem, a capability routing rule |
| Token cost | ≤1024 tokens/image, counted per image (T3) | 1 image at Flash off-peak = 1024 × 0.15/1M = **$0.0001536**; 600 images = **$0.092** worst case per request |
| Placement | `user` messages only; system/assistant images ⇒ 400 (S10) | the content-part model must be user-turn-scoped; a "system prompt with a diagram" is impossible |
| Transport | base64/URL ≤32 MiB, `file_id` ≤64 MiB, body ≤48 MiB, ≤600 images (S10) | base64 in a JSON body inflates ~33 %; large images need Files |
| Cache | image-token cache behaviour is **not documented** (A7) | do not promise "images are cached"; measure or stay silent |

The code has no content-part model at all — `ChatMessage.content` is `Option<String>`
(`dto.rs:88-106`) — so vision is a type change plus a UI, not a flag. That is the honest cost of the
decision.

---

## 9. File / route / UI seams with acceptance tests

Every row is DESIGN unless marked READ. Tests are wiremock/unit-level and require **no paid call**.

### 9.1 Prefix and prompt assembly

| # | Seam | Change | Acceptance test |
|---|---|---|---|
| S1 | `context.rs:105-118` | semantic fact keys; drop the owning `TaskId` from prompt text | `context.rs::fact_keys_contain_no_uuid` — render a block, assert `regex(uuid)` matches nothing, and that two sibling contexts built from the same facts render **byte-identical** blocks |
| S2 | `context.rs:66-95`, `execution.rs:53-58` | facts render after the run prompt, not inside it | `execution.rs::prompt_prefix_is_shared_by_siblings` — build two tasks of one run, assert `common_prefix_len(prompt_a, prompt_b) ≥ len(run_prompt) + 64` |
| S3 | `provider_executor.rs:82-92` | canonical persona id resolution (lowercase, `-`/space → `_`) in one function | `provider_executor.rs::title_case_role_resolves_to_persona` — `"Code Reviewer"` and `"code-reviewer"` both return the persona system text |
| S4 | `provider_executor.rs:386-402` | persist raw `arguments`; replay verbatim; set `reasoning_content` only when non-empty | `provider_executor.rs::replayed_arguments_equal_provider_bytes` — feed `{"path": "a.txt"}` (with space) through a turn and assert the replayed string is byte-equal; second test asserts no `reasoning_content` key when thinking was disabled |
| S5 | `provider_executor.rs:296-318`, `execution.rs:1552-1565` | planner prompt = system(L1 + planner section) + user(instruction + run prompt); delete the `Debug`-flattened join | `execution.rs::planner_prompt_has_one_instruction_set` — assert the rendered prompt contains the planner instruction **once** and contains no `System: ` marker; assert `prefix_fp` equals the leaf `prefix_fp` when persona/tools match |
| S6 | new `prefix.rs` (message builder) | one function returns `(messages, prefix_fp, tools_sha256)` | `prefix.rs::fingerprint_changes_only_when_a_layer_changes` — table test: edit L1 ⇒ changes; edit L5 ⇒ unchanged; reorder tools ⇒ changes |
| S7 | `tool.rs:391-430` | freeze tool order; hash it | `tool.rs::tool_specs_are_byte_stable` — serialize twice, assert equal hash; assert the 9 Worker specs hash to a pinned value |

### 9.2 Tokenizer, estimator, preflight

| # | Seam | Change | Acceptance test |
|---|---|---|---|
| S8 | new `provider/src/tokenizer.rs` + `assets/tokenizer/` | vendored artifact, embedded, SHA-256 verified at boot | `tokenizer.rs::golden_counts` — the ten §4.2 strings (pinned verbatim in the test) reproduce 48/108/41/159/228/58/123/54/24/169; `::asset_hash_matches` |
| S9 | `provider_executor.rs:436-442` | replace `bytes/4` with the tokenizer on the rendered prompt; keep the old function only as `Assumed(ratio)` fallback behind a flag | `::estimate_matches_tokenizer` — for the planner request, estimate differs from the tokenizer count by 0; `::cjk_and_uuid_content_are_not_undercounted` — the facts block estimates ≥ its tokenizer count × 0.95 |
| S10 | `budget.rs:235-284` | context gate + projected cost + reservation | `preflight_refuses_over_context_with_zero_requests` — wiremock count 0; `projected_cost_refuses_before_send` — spent $0.49 of $0.50, next call projected > cap ⇒ `E_BUDGET_EXCEEDED` with `attempts()==0`; `reservation_prevents_n_way_overshoot` — 16 concurrent reservers, total reserved never exceeds cap |
| S11 | `client.rs:492-530` | 402 ⇒ `E_PROVIDER_BALANCE` (not auth), 400/422 ⇒ request-schema refusal | `error_matrix::http_402_is_balance_not_auth` (request count 1, no retry); `::http_400_names_the_request_error` |
| S12 | `dto.rs:891-909` | parse `context_window`, `max_output_tokens`, `effort.supported_levels`, `input_modalities` | `dto.rs::models_metadata_roundtrips` — a documented payload keeps every field; boot test rejects a configured effort outside `supported_levels` |

### 9.3 Cost ledger and exposure

| # | Seam | Change | Acceptance test |
|---|---|---|---|
| S13 | `db.rs` v11/v12 (§6.2) | `provider_calls`, `request_charges`; `run_usage` gains status | `db.rs::migration_11_and_12_apply_and_roll_back` — apply, assert tables, apply `down`, assert gone |
| S14 | `provider_executor.rs:233-243`, `:315-328` | persist the `CallCost` returned by `record_call` | `tests/cost_ledger.rs::settled_cost_is_persisted_per_call` — wiremock returns hit 400 / miss 600 / out 500; row amount = `0.00003912` decimal (the figure pinned in `accounting.rs:466-467`), `peak_class` and `price_source_url/fetched_at` present |
| S15 | `execution.rs:698-719` | stop dropping completeness; persist `status` + counts | `::incomplete_run_persists_status_and_null_money` — one null-usage call ⇒ `status=INCOMPLETE`, no settled figure anywhere |
| S16 | `http.rs:842-875` + new `/api/cost/*` | expose status + rollups | `tests/cost_routes.rs::incomplete_run_returns_words_not_digits`; `::hive_rollup_partitions_the_run_total` — Σ hives == run total, exact decimal, no float tolerance |
| S17 | `event.rs:273-284` | extend `RUN_METRICS` (contract change + ledger row) | `contract` schema test: old client ignores unknown fields; new client decodes; `CHANGE-LEDGER-IMPL.md` row present |
| S18 | `web/src/panels/CostPanel.tsx` + nav | cost panel | Vitest: INCOMPLETE shows "INCOMPLETE — 3 of 7 calls unmeasured" and no `$`; COMPLETE shows decimals from the route verbatim (no client-side arithmetic) |
| S19 | `ModelsPanel.tsx:174-189` | render status | Vitest: INCOMPLETE hides the hit/miss digits and explains why |

### 9.4 Budget controls

| # | Seam | Change | Acceptance test |
|---|---|---|---|
| S20 | `budget.rs`, new `budget_state` | modes A/B/C, guard, kill switch | `::mode_a_has_no_money_cap` — default `BudgetLimits` has no USD cap and the structural caps are non-zero; `::kill_switch_blocks_preflight` — after kill, `E_SPEND_HALTED` with zero requests; `::kill_state_survives_restart` — reopen the DB, still halted |
| S21 | `refusal.rs:152-160` | parameterised cap text | `refusal.rs::cap_text_names_the_active_limit` — the message contains the actual number |
| S22 | `http.rs` new routes | arm/disarm/kill/rearm | `tests/budget_routes.rs::kill_is_idempotent_and_audited` — 2 kills ⇒ one audit row, both 200 |

### 9.5 Search / Fetch (only if adopted)

| # | Seam | Change | Acceptance test |
|---|---|---|---|
| S23 | `pricing.rs` | `RequestPriceRow` 750 000 nano-USD | `::request_price_is_exact` — 1 request = 750 000; 4 = 3 000 000 |
| S24 | new `provider/src/search.rs` | billable predicate + retry policy | `::only_successful_results_are_billable` — table over 200-with-results, 200-empty, 403, 404, 408, 413, 429, 500, 504 |
| S25 | `accounting.rs` | non-token charge in the report | `::run_total_sums_tokens_and_requests` — a run with 2 searches and 1 000 tokens exports the exact sum; a failed search adds 0 **and does not mark INCOMPLETE** |
| S26 | `tool.rs` | `Network` permission class wired | `::search_requires_approval` — no approval ⇒ `E_APPROVAL_REQUIRED`, zero HTTP calls |

---

## 10. Twenty questions

**Q1. What exactly is a cache hit under the current documented model?**
A request must **fully match a persisted cache prefix unit**; units are created at request boundaries
(end of user input, end of model output), by detected common prefixes across requests, and at fixed
token intervals inside long inputs/outputs (C2–C5). `A+B` then `A+C` misses; after `A` is detected as
a common prefix, `A+D` hits (documented Example 2). So: identical leading bytes + repetition, not
similarity. **DOCUMENTED.**

**Q2. Which bytes of HiveMind's prompts are cacheable today, and which kill reuse?**
Cacheable: the per-tier tool array (`tool.rs:391-430`), the persona system text (`http.rs:258-269`),
and the run prompt at the start of the leaf user message. Killing reuse: UUIDs inside the fact keys
(`context.rs:105-111`), UUID-sorted fact order (`BTreeMap`), re-serialised tool arguments on replay
(`provider_executor.rs:386-392`), `reasoning_content: ""` (`:402`), and per-task memory reads
(`execution.rs:865-886`). **READ.**

**Q3. What is the correct layer order?**
`tools[]` → `L1` global immutable → `L2` persona → `L3` frozen memory snapshot → `L4` run prompt →
`L5` task assignment → `L6` transcript → `L7` tail directive. One system message holding L1–L3, then
one user message holding L4–L5, then the transcript. Rationale and rules in §3.2. **DESIGN.**

**Q4. Where does memory go so it does not invalidate the prefix?**
Into L3 as a **frozen, content-addressed snapshot** with semantic keys, replaced at most once per run
(R4). Never mutate a fact mid-run; never read per-task memory in the message builder; never put a
`TaskId` in the text. **DESIGN** (fixes P1/P2/P5).

**Q5. What invalidates what?**
Layer *N* invalidates *N* and everything after it. The full matrix is §3.4. The two non-obvious rows:
a tool-array change invalidates the whole tier's head, and a mid-run memory write invalidates the
whole tail for the rest of that run. **DESIGN from DOCUMENTED rules.**

**Q6. When is cross-agent reuse correct?**
Same account key, same model, same `user_id`, same L1–L3 bytes, same tool array. It is a *byte*
sharing mechanism, not a result-sharing one — outputs are always recomputed (C6). Cross-hive reuse is
correct by default for one operator; isolation is a deliberate opt-in that costs input tokens.
**DESIGN.**

**Q7. Does `user_id` help or hurt?**
Hurt reuse, help isolation. `user_id` provides KVCache isolation (C9), which by definition prevents
cross-`user_id` hits. On a standard-quota account it buys **no** extra concurrency (scheduling
splitting applies only to increased-quota accounts). Default: send none. **DOCUMENTED** fact +
**DESIGN** recommendation.

**Q8. How do we know the realised hit rate?**
From `prompt_cache_hit_tokens` / `prompt_cache_miss_tokens` on every call (C8, T7), stored per call
with a `prefix_fp` so the rate can be grouped by arrangement (§6.4). Today the counters are used to
price a run and then thrown away (`accounting.rs:233-258` → dropped at
`provider_executor.rs:233-243`), so **the current product has no hit-rate number at all** — any
statement about caching saving money today is UNVERIFIED. **READ.**

**Q9. Is `bytes/4` usable as a gate?**
No. Measured error: **−47 % on the facts block, −31 % on the assembled leaf prompt, −26 % on code**,
**+10 % prose, +25 % CJK, +32 % on the planner JSON body** (§4.2). Undercount is unsafe; overcount is
merely pessimistic. Replace it and keep it only as a labelled fallback. **MEASURED.**

**Q10. How do we get an accurate tokenizer, concretely?**
Vendor the official `tokenizer.json` (6 367 096 bytes, SHA-256 pinned), load it with the `tokenizers`
crate, tokenize the **rendered chat prompt** (not the JSON body), gate with a 10 % margin, and
calibrate against reported `prompt_tokens` forever. Ten golden counts from §4.2 are the acceptance
test. **MEASURED/DESIGN.** Note the artifact is a Llama-style fast tokenizer config whose
`model_max_length: 16384` is **not** the model context (T5).

**Q11. What is the preflight, exactly, and where does it run?**
`gate_tokens(estimate) + max_tokens ≤ context_window` and
`spent + reserved + projected_cost(estimate, max_out, peak_now) ≤ cap` (when a cap exists), both
**before** any HTTP request, with named refusals and zero provider calls on refusal (§5, §7.3).
**DESIGN.**

**Q12. What happens when the provider omits usage?**
The run becomes `INCOMPLETE`, no settled cost is exported, and that must be visible on every surface
(`accounting.rs:56-73` is right; `http.rs:842-875` and `ModelsPanel.tsx:174-189` are wrong today
because they print numbers anyway). **READ.**

**Q13. How is money stored so aggregates are exact?**
`rust_decimal` in memory; on disk an integer **nano-USD** column plus the decimal string for display;
aggregation in Rust with Decimal; no `f64`, no `SUM()` over TEXT. A single Flash cache-hit token is
3 nano-USD, so the unit is fine enough to be exact (§6.5). **DESIGN.**

**Q14. What does "cost per hive" mean when the schema has no hive?**
`hive_id` = run root, or the nearest SuperAgent ancestor for a branch (§6.3). Define it explicitly,
store it per call, and prove the partition sums to the run total. Otherwise "per hive" is a label, not
a number. **DESIGN.**

**Q15. What is the default budget policy?**
Mode A — **no money cap, no token cap**, but the structural envelope is always on (agents, calls,
fan-out, depth, per-call `max_tokens`, context preflight, in-flight, wall-clock), plus an
account-anchored balance floor, a 402 freeze, and a kill switch (§7.2). Today's `$0.50`/400 000/40 are
moved to opt-in presets (`budget.rs:24-43`). **DESIGN.**

**Q16. How does the operator turn on a hard cap, and what changes?**
Mode C: per-run/per-hive/per-day USD or token caps, **projected** preflight, reserve-then-settle,
named refusals, exact ledger rows. The refusal text must name the active limit (`refusal.rs:152-160`
hard-codes $0.50 today, so it would lie). **DESIGN.**

**Q17. With no default cap, what stops denial-of-wallet?**
Five things, none of which is a dollar constant: structural envelope, balance floor (5 % of boot
balance, relative not absolute), 402 freeze with no retry, spawn approval (P0 in 09), and an explicit
kill switch that survives restart — plus the ledger that makes any of it visible. Residual risk: Mode
A alone has no *daily* ceiling, which is exactly why Mode B is offered as an explicit first-launch
choice (§7.5). **DESIGN.**

**Q18. How are Search / Fetch billed and costed?**
$0.75 per 1 000 **successful** requests (200 + produced result); failures and empty results are free;
`count` is irrelevant; Search QPS 100, Fetch QPS 25 plus per-domain (S2, S7). Cost them as a
**separate quantum** (750 000 nano-USD/unit) and remember the returned text is *also* cache-miss
input tokens. **DOCUMENTED** (price) + **DESIGN** (quantum).

**Q19. Files and Vision: in or out?**
Out of v1. Files is image-only (no text/PDF), free to store, useful only as vision transport later.
Vision is Flash-only, `user`-message-only, ≤1024 tokens/image (≈$0.0001536/image at Flash off-peak),
≤600 images, 48 MiB body, and its cache behaviour is undocumented (A7). **DOCUMENTED + DESIGN.**

**Q20. What must be measured before claiming "better than DeepSeek's harness"?**
A controlled comparison on the same task corpus: two `prefix_fp` arrangements (prefix-stable vs
today's UUID-poisoned), n ≥ 50 calls each, reporting realised cache-hit rate, effective input price,
$/task, p50/p95 estimate ratio, and retry overhead — with the same model, peak class and prompt sizes.
No claim of superiority without it; and note there is no public per-call Harness cost figure to
compare against, so the honest claim is **"our own before/after"**, not "better than X". **DESIGN.**

---

## 11. Unverified cache assumptions (flag list)

Each row is an assumption this design depends on, why it matters if false, and the cheapest decisive
evidence. **None of these may be used as a stated fact.**

| # | Assumption | If false | Cheapest decisive test |
|---|---|---|---|
| **A1** | A leading `system` message accepts multiple sections and we may use one `system` message with internal sections (no multi-`system` requirement). | Splitting would be needed; the single-message design already avoids the risk. | One live call with a 3-section system message; compare `prompt_tokens` and output to the same content as one section. |
| **A2** | The server renders `tools[]` at a **fixed position** relative to `messages`, so a stable tool array preserves a stable prompt prefix. | Adding/removing a tool could invalidate more (or less) than the message head suggests. | Two live calls, same messages, tools present vs absent; compare hit/miss split and `prompt_tokens` delta. |
| **A3** | The **fixed token interval** granularity (C5) is small enough that a long stable block becomes cacheable without ending a request. Interval size is not documented. | Very long L3 blocks might never become units until a request ends inside them. | Live: 3 calls with a 20 k-token identical L3 and different tails; watch when hits appear. |
| **A4** | Common-prefix detection (C4) needs only **two** prior requests to teach a head, and works at sub-message (token) granularity. | The design's "third call hits" expectation is wrong; more repetition is needed. | Live: N identical L1 calls with different tails; find the first call index with `hit > 0`. |
| **A5** | Cache units are shared across **different personas** as long as the leading bytes are identical (persona text sits after the shared head). | Cross-persona reuse is impossible; per-persona units only. | Live: two personas, same L1, different L2; check hit > 0 on the second persona after repetition. |
| **A6** | Cache is **account-wide**, not connection- or process-scoped, and survives a client restart. | Every restart is a cold start; the "cache epoch" metric changes meaning. | Live: run, restart the server, repeat the identical request, observe hit tokens. |
| **A7** | Image tokens participate in prefix caching like text tokens; re-sending the same `file_id` image is a hit. | Vision cost is always miss-priced; the ≤1024-token bound is still the ceiling. | Live: same image twice + a third call; observe hit tokens vs the 1024 bound. |
| **A8** | `reasoning_content` echoed back counts as **prompt** tokens and is part of the matched prefix (it is concatenated into context). | Tool-loop replay cost is mis-estimated; the replayed assistant turn is not cacheable. | Live: tool loop with thinking on; compare `prompt_tokens` growth against the reasoning length. |
| **A9** | The official `tokenizer.json` (a Llama-style shared artifact) matches the served model's tokenizer within a few percent. | The estimate is systematically biased; the safety margin may be wrong. | Live: 20 calls, compare estimated vs reported `prompt_tokens`; require p95 ratio ≤ 1.05 after margin. |
| **A10** | The tokenizer artifact's licence permits redistribution inside the binary. The zip contains no licence file (MEASURED). | Cannot vendor; must fetch at build time or ship a sidecar. | Read the DeepSeek terms/licence for the artifact; if unclear, keep it a download-at-build asset. |
| **A11** | `context_window` is exactly 1 000 000 (docs say "1M") or is reported verbatim by `/models`. | A preflight at exactly 1 000 000 could still overflow if the true value is smaller. | Live `GET /models` capture; use the smaller of the two values. |
| **A12** | Peak/off-peak classification matches the provider's billing instant for a request that starts near a boundary. | A call at 03:59:59 UTC could be billed peak while priced off-peak (or vice versa). | Compare a boundary-time call's invoice line with the stored `peak_class`; if it disagrees, price by completion time or add a boundary guard. |
| **A13** | Cache eviction ("hours to days") is not correlated with our run cadence in a way that makes a 150-agent run re-learn its head mid-run. | Hit rate collapses mid-run and the ledger blames the design. | Observe hit rate over one long soak; alert if it falls after N minutes with no prefix change. |
| **A14** | Provider-reported `prompt_tokens` includes every byte we sent (tools, system, transcript) and nothing else. | The calibration loop compares incomparable numbers. | Live: one call with a known tokenizer count within ±2 %; if not, model the fixed overhead as an offset. |

---

## 12. Claims register

| Claim | Status | Evidence |
|---|---|---|
| Cache hit requires a **full match of a cache prefix unit**; three persistence triggers | **VERIFIED** (DOCUMENTED) | `/guides/kv_cache`, fetched 2026-11-07 |
| `A+B` then `A+C` misses; after common-prefix detection `A+D` hits | **VERIFIED** (DOCUMENTED example) | same page, Example 2 |
| The stream's last chunk carries `usage` even without `stream_options` | **VERIFIED** (DOCUMENTED) | `/api/create-chat-completion` |
| `prompt_cache_hit_tokens` / `miss_tokens` are required fields; `prompt_tokens` = hit + miss | **VERIFIED** (DOCUMENTED) | same page |
| `max_tokens` 1–393 216; defaults 8K/64K/128K; context 1M | **VERIFIED** (DOCUMENTED) | same page + `/quick_start/pricing` |
| Prices, peak windows, 50×/30× hit-vs-miss gap | **VERIFIED** (DOCUMENTED) | `/quick_start/pricing` |
| Search/Fetch $0.75 per 1 000 successful requests; billable only on 200 + result; QPS 100/25 | **VERIFIED** (DOCUMENTED) | `/guides/search`, `/quick_start/pricing` |
| Files is image-only; 64 MiB/25 GiB/10 000 files; no documented storage price | **VERIFIED** (DOCUMENTED) | `/guides/files_api` |
| Vision is Flash-only, `user`-only, ≤1024 tokens/image, ≤600 images, 48 MiB body | **VERIFIED** (DOCUMENTED) | `/guides/vision` |
| `user_id` isolates KVCache; quota splitting only on increased-quota accounts | **VERIFIED** (DOCUMENTED) | `/quick_start/rate_limit` |
| The official tokenizer zip exists, contains `tokenizer.json` (6 367 096 bytes) + config + a short Python demo | **VERIFIED** (MEASURED) | download + SHA-256 in §4.2 |
| `bytes/4` undercounts the facts block by 47 %, code by 26 %; overcounts prose by ~10 %, the planner body by 32 % | **VERIFIED** (MEASURED, n=10 strings) | §4.2 table; reproducible |
| Chat-template envelope costs single-digit tokens | **VERIFIED** (MEASURED) | special-token ids 0/1/128803/128804; `<｜Tool｜>` = 5 tokens |
| HiveMind has no per-call cost persistence; no hive dimension; no request-billed dimension | **VERIFIED** (READ) | `db.rs:198-217`, `accounting.rs:94-116`, `pricing.rs:47-64` |
| `GET /api/usage/runs/{id}` cannot report INCOMPLETE | **VERIFIED** (READ) | `http.rs:842-875` |
| Replayed tool arguments are re-serialised (not provider bytes) | **VERIFIED** (READ) | `provider_executor.rs:386-392` |
| `reasoning_content` is always set on replay | **VERIFIED** (READ) | `provider_executor.rs:402` |
| Fact keys embed a `TaskId` UUID | **VERIFIED** (READ) | `context.rs:105-111` |
| Planner prompt flattens two template messages into one user string with `Debug` labels | **VERIFIED** (READ) | `execution.rs:1552-1565` |
| Realised cache-hit rate for this account/corpus | **UNVERIFIED** | needs a keyed run; no paid call made |
| The tokenizer's fidelity for `deepseek-flash` | **UNVERIFIED** (A9) | calibration against `usage.prompt_tokens` |
| Whether cache units are shared across personas / survive restarts / cover images | **UNVERIFIED** (A5, A6, A7) | targeted live probes |
| Any claim that HiveMind is "better than DeepSeek's harness" at caching | **UNVERIFIED — do not make it** | needs the §10 Q20 protocol |

---

## 13. Sources

### 13.1 Repository seams read in this pass

`backend/crates/hiverouter-core/src/{provider_executor.rs,execution.rs,context.rs,config.rs,http.rs,db.rs,caps.rs,tool_loop.rs,prompts.rs}`,
`backend/crates/hiverouter-provider/src/{dto.rs,client.rs,accounting.rs,budget.rs,pricing.rs}`,
`backend/crates/hiverouter-tools/src/tool.rs`,
`backend/crates/hiverouter-contract/src/{event.rs,refusal.rs,usage.rs}`,
`backend/crates/hiverouter-core/assets/prompts/{planner,code-reviewer}.prompty`,
`web/src/{App.tsx,state/store.ts,panels/NavRail.tsx,panels/ModelsPanel.tsx}`,
`backend/Cargo.toml`.

### 13.2 Official pages fetched 2026-11-07 (HTTP 200, this pass)

| # | URL | What it establishes here |
|---|---|---|
| D1 | https://api-docs.deepseek.com/guides/kv_cache | prefix units, three persistence triggers, full-match rule, both worked examples, counters, best-effort + eviction |
| D2 | https://api-docs.deepseek.com/quick_start/token_usage/ | character ratios, official tokenizer download, image-token calculator, "usage is the source of truth" |
| D3 | https://api-docs.deepseek.com/quick_start/pricing/ | model versions, 1M context, 384K output, all prices, peak windows, Search/Fetch pricing, deduction rule |
| D4 | https://api-docs.deepseek.com/quick_start/rate_limit/ | concurrency, `user_id` isolation (incl. KVCache), keep-alive, 10-minute cutoff |
| D5 | https://api-docs.deepseek.com/api/create-chat-completion | `max_tokens` bounds/defaults, `stream_options.include_usage` semantics, required usage fields, `finish_reason` set, JSON-mode space caveat, context = input+output |
| D6 | https://api-docs.deepseek.com/api/list-models/ | `context_window` (input+output), `max_output_tokens`, `effort.supported_levels`, modalities, "metadata may be omitted" |
| D7 | https://api-docs.deepseek.com/guides/search | Search/Fetch fields, response schemas, operator syntax, error table, **billing rules**, QPS |
| D8 | https://api-docs.deepseek.com/guides/vision | Flash-only images, three input methods, detail levels, ≤1024 tokens/image, full limit table, user-only restriction |
| D9 | https://api-docs.deepseek.com/guides/files_api | image-only scope, four operations, 64 MiB/25 GiB/10 000 files, expiry, Anthropic-variant differences |
| D10 | https://api-docs.deepseek.com/updates/ | newest change 2026-10-17; dating of model/pricing changes |
| D11 | https://cdn.deepseek.com/api-docs/deepseek_v4_tokenizer.zip | the tokenizer artifact (downloaded; **MEASURED**) |

### 13.3 Re-fetch list (these rows rot fastest)

1. D3 prices, peak windows, and the model list — they make every money report false.
2. D1 cache rules — if "full match of a unit" changes, §3 is wrong wholesale.
3. D5 `max_tokens` bounds/defaults and the `stream_options` paragraph.
4. D7 Search/Fetch billing + QPS — the only per-request charge in the product.
5. D8/D9 limits — image/upload numbers move when models change.
6. D10 — diff against **2026-10-17** before relying on anything here.

---

## 14. Deliverable status

### 14.1 Done (with evidence)

| Item | Evidence |
|---|---|
| Official cache/billing/limit facts re-verified for this pass | §13.2 (11 URLs, HTTP 200, 2026-11-07) |
| Cache-prefix design with exact ordering + invalidation matrix | §3.2–§3.4 |
| Every prompt-assembly byte path read and defects located | §2, `path:line` throughout |
| Tokenizer acquired, hashed, and measured against HiveMind strings | §4.2; zip SHA-256 `E7310D1D…6FE495`, `tokenizer.json` SHA-256 `89085F12…DC9614`, ten measured rows |
| Cost/cache measurement + exposure design (schema, routes, UI, definitions) | §6 |
| Budget policy: no hard default, opt-in caps, soft guard, kill switch | §7 |
| Search/Fetch per-request billing + Files/Vision decisions | §8 |
| Context-window / max-output preflight | §5 |
| 20 Q&A, seams, acceptance tests, unverified-assumption register | §10, §9, §11 |

### 14.2 Not done (and why)

| Item | Reason |
|---|---|
| A live cache-hit measurement for this account/corpus | No paid call was permitted in this lane (`07`/`01` discipline); §11 A2–A9 remain open |
| Any product code change | Out of scope by instruction ("write only this file") |
| Licence determination for the tokenizer artifact | The zip ships no licence file; recorded as A10 rather than guessed |
| Rendering the Jinja `chat_template` with a real engine | The `tokenizers` Python wheel (0.23.3) does not expose `apply_chat_template`; the envelope was measured by literal application of the template's own concatenation, and the count (single-digit tokens) is robust to that limitation |

### 14.3 Unverified

| Item | What would prove it |
|---|---|
| Realised cache-hit rate, per arrangement | A keyed run: 3+ calls per `prefix_fp`, counters recorded (§11 A2–A6) |
| Tokenizer fidelity for `deepseek-flash` | Calibration over ≥20 calls (A9) |
| `context_window` exact integer | One keyed `GET /models` capture (A11) |
| Peak-class agreement with the invoice at boundary times | One boundary-time call compared to billing (A12) |
| Image-token cache behaviour | Vision probe (A7) |

### 14.4 Stub sweep

The mandatory sweep (`TODO|FIXME|XXX|HACK|placeholder|NotImplemented|not implemented|mock|stub|dummy|lorem|simplified for|for illustration`)
was run over this file. Hits are the sweep record itself plus the words "placeholder"/"mock" used in
the sense of *refusal-text placeholders* and *wiremock test doubles* (§9 acceptance tests). Both are
**permitted**, not defects: no unfinished implementation is described as done anywhere in this file,
and every recommendation is marked DESIGN or UNVERIFIED.
