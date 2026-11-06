# GATE-CY-2 — recon → plan → lead (cycle 2: paper + official docs + provider/budget requirement map)

## 1. RECON

- **Paper** (`audit/HIVEMIND-AUDIT-PAPER.md`, 244 lines, sha256 `3BAAD198…13D495`): subject HEAD `3988622` tag `v1.0.0`;
  3 P0/10 P1/16 P2/1 info; A-F01/A-F02/A-F03 all fixed in HEAD source per 6 of 8 seats; A-F04 (templates/handshake),
  A-F06 (money overflow→zero), A-F07 (duplicate majors), A-F08 (untracked receipts), A-F13+ (harness gaps) still relevant,
  but the paper itself is stale (S8/SEAT-05/SEAT-07 all flag baseline drift).
- **Official docs re-read (HTTP 200, this cycle):**
  1. `https://api-docs.deepseek.com/` — base_url `https://api.deepseek.com`; `deepseek-flash` / `deepseek-v4-pro`;
     `POST /chat/completions`; Bearer key; thinking toggle; stream flag.
  2. `/quick_start/pricing/` — max output **384K** (table); flash 0.003/0.006 hit, 0.15/0.3 miss, 0.6/1.2 output;
     pro 0.022/0.044, 0.66/1.32, 1.98/3.96; peak 01:00–04:00 & 06:00–10:00 UTC Mon–Fri; concurrency 2500/500.
  3. `/guides/thinking_mode/` — default enabled/high; effort map minimal→low, low→low, medium→high, high→high,
     xhigh→high, max/ultra→max; temperature/presence/frequency no effect; `reasoning_content` pass-back required when
     tools present (400 otherwise); top_p model/mode rules.
  4. `/api/create-chat-completion/` — messages≥1, model enum2; `max_tokens` 1..393216 (defaults 8K/64K/128K-max);
     `stream` + `stream_options.include_usage`; **usage rides the last chunk before `[DONE]`**; finish_reason enum incl
     `insufficient_system_resource`/`aborted`; `thinking.type` precedence over `reasoning_effort`; JSON mode caveat
     (non-JSON replaced by spaces → empty content possible).
- **Provider/budget requirement map (code vs docs):**
  - Adapter (`provider_executor.rs`): planner JSON-DAG prompt (`:22-25,108-126`), fence strip (`:191-200`), empty content
    → `E_PROVIDER_INVALID_RESPONSE` (`:163-189`); leaf non-streaming single delta (`:150-159`); budget preflight via
    `client.complete_with_budget` (`:75-95`).
  - Client (`provider/src/client.rs`): endpoint/no-`/v1` (test `:737-741`), SSE parser `:150-186`, 401/402 no-retry
    `:432-460`, key via Bearer header only `:269-279`, Debug redacts `:196-205`.
  - Budget (`provider/src/budget.rs`): B1 50¢/B2 400k/B3 40 calls/B7 in-flight 8/B8 3 attempts/B9 Retry-After ≤60 s/B10
    one key; preflight before send (`:235-284`); INCOMPLETE poisons further sends (`:236-241`); report null-vs-zero.
  - **Docs-vs-code drift found:** client doc-comment (`client.rs:11-13`) says streaming usage "NOT documented" → docs now
    state it is documented on the last chunk; fix comment + use it for real streaming (no behavior change required for
    correctness, but the plan should consume it).
  - **Security requirement:** tool subprocess spawns inherit server env (`S6-F-01`) — contradicting the custody claim
    "key never leaves the process"; fix = `env_clear()` + allowlist; acceptance test must prove a child cannot read the
    env var (without exposing a real key).

## 2. PLAN — decisions and rejected alternatives

- **D1 (streaming):** implement leaf streaming with the documented SSE path; adapter exposes delta callback or returns a
  stream; engine publishes `REPLY_DELTA` per chunk, usage from final-chunk usage; rejected: keep single-delta (fails UX
  claim; evidence doc already says open gap) and client-side fake chunking (fabrication).
- **D2 (planner retry):** one bounded repair attempt on empty content/invalid JSON (prompt asks for JSON-only, retries
  once with stricter instruction), counted in budget/call cap; never fabricate a plan; rejected: unlimited retries
  (cost), silent empty plan (fake success).
- **D3 (budget lifecycle):** per-run budget must be created on run start, consulted on every planner/leaf/retry call,
  and **evicted on terminal**; call counting unified: either engine passes a `RunBudget` handle into the executor (both
  caps share one counter) or provider remains authority and engine `check_calls` is removed; rejected: two independent
  counters (drift), no eviction (memory growth).
- **D4 (template registry):** load `assets/prompts` at boot with licence gate, fail by name (`E_TEMPLATE_*`), pass into
  engine; planner uses template if present else raw prompt with explicit decision; rejected: keep `default()` (A-F04).
- **D5 (subprocess env):** every `Command` spawn in tools/core clears env and re-adds an allowlist (`PATH`, `SystemRoot`,
  `TEMP`/`TMP`, `HOME`/`USERPROFILE` on win) + no `DEEPSEEK_*`/`HIVEROUTER_*` secrets; test = child prints env keys,
  assert key absent; rejected: rely on audit redaction only (does not stop exfiltration).
- **D6 (release evidence, not code):** acceptance requires clean commit C + tagged rebuild + HEAD-bound logs; reject
  accepting current dirty tree/artifacts.

## 3. LEAD — cycle gate

- Read gate: paper + 4 docs read this cycle; provider/budget/doc map recorded; every claim carries `path:line`/URL. PASS.
- Contradiction handling: stale paper claims marked; docs drift (streaming usage) noted; no inherited claim left
  unlabelled. PASS.
- **Cycle 2 verdict: PASS** — plan decisions D1–D6 are decision-complete; no unresolved capability gap; security fix
  prioritised before UX fixes.
