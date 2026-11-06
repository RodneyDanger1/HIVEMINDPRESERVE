# PLANNER-01 — HiveMind 1.0 Architecture/Provider/Budget plan

**Seat:** PLANNER P1 (Architecture · Provider · Budget) · **Repo:** `C:\Users\Riley\Desktop\hiverouter-v1`
· **Evidence base:** 8 seat reports (`plans/seats/SEAT-01..08/`), `audit/HIVEMIND-AUDIT-PAPER.md` (HEAD `3988622`),
4 official DeepSeek docs pages (fetched HTTP 200, 2026-11-06). **Planning HEAD:** `5c69b66` (dirty tree; tag `v1.0.0`
= `3988622`). Read-only planner; gates in `GATE-CY-1/2/3.md`.

## 1. Outcome

After this plan executes on a clean branch and a sealed commit `C`, HiveMind 1.0 can run a real DeepSeek prompt with:
(1) **no key exposure through tool subprocesses** (env-clear allowlist), (2) **incremental streamed replies** from the
documented SSE path instead of one burst delta, (3) a **bounded planner repair** so a terse prompt cannot hard-fail the
run on an empty/JSON-invalid planner response, (4) **one unified per-run call budget** that is enforced on every planner,
leaf and retry call and released when the run terminates, (5) the **authored prompt/persona templates loaded at boot**
with named refusals, and (6) a **HEAD-bound acceptance gate** whose evidence (JUnit/logs/artifact hashes) is reproducible
on commit `C` — without any paid call in CI and with one explicitly operator-authorized live smoke.

## 2. Current behavior (evidence, `path:line`, HEAD `5c69b66`)

| Area | Current behavior | Evidence |
|---|---|---|
| Provider wiring | wired at HEAD: core→provider dep, `build_executor` → `ProviderExecutorAdapter::from_env`, honest `E_KEY_MISSING` fallback; paper A-F01 describes the pre-wiring artifact `3988622` and is **stale for HEAD** | `core/Cargo.toml:21`; `core/src/http.rs:67-82,101,141,178`; `provider_executor.rs:54-57,99-160`; SEAT-01 §0, SEAT-02 V10, SEAT-05 V12, SEAT-08 V1 |
| Leaf reply | non-streaming: one `ChatRequest`, one completion, `deltas: vec![text]` — UI receives a single `REPLY_DELTA` burst | `provider_executor.rs:150-159`; `docs/evidence/live-provider-smoke.md:12`; SEAT-01 S1-R1, SEAT-04 V6 |
| Planner retry | empty/tool-call-only content → `E_PROVIDER_INVALID_RESPONSE` and the run fails; no repair attempt (terse prompt once produced empty planner output) | `provider_executor.rs:163-189`; `live-provider-smoke.md:12`; SEAT-01 S1-R2 |
| Budget lifecycle | `RunBudget` per `RunId` in `Mutex<HashMap>`; entry created lazily on first call; **never removed** → process-lifetime growth | `provider_executor.rs:32-36,79-82`; SEAT-01 S1-R4 |
| Call caps | provider `RunBudget` caps attempts at 40 incl. retries; engine only does `check_calls(1)` for the single planning call; leaf calls are not counted by the engine → two counters, one authority unclear | `execution.rs:477-478`; `budget.rs:24-52,235-284`; SEAT-01 S1-R5 |
| Usage honesty | absent usage marks run INCOMPLETE and poisons further sends; docs confirm usage rides the last stream chunk (client comment claims "not documented") | `budget.rs:235-242`; `client.rs:11-13`; official `/api/create-chat-completion/` (this cycle) |
| Templates | production server passes `TemplateRegistry::default()` (empty); authored `assets/prompts/*` only loaded by tests → A-F04 still open at HEAD | `http.rs:102,142,179`; `tests/prompt_assets.rs`; SEAT-01 S1-R3 |
| Key custody | env-only, `Zeroizing`, Debug `ApiKey(***)`, one `expose()` for Bearer header; no first-party log/DB/API leak | `secret.rs:20-81`; `client.rs:269-279,730-742`; SEAT-06 V1–V5 |
| **Subprocess leak** | tool subprocesses inherit server env (`Command::new` without `env_clear`/`env_remove` in 3 sites); any child can read `DEEPSEEK_API_KEY` | `tools/src/shell.rs:201-208`; `tools/src/cli_support.rs:41`; `tools/src/jail.rs:389`; SEAT-06 F-01 (P1) |
| Release/evidence | shipped binary `184D51A7…` (09:10) + zip `BC7A2BAD…` (09:33) predate provider commit (10:59); tag behind HEAD; dirty tree; `release-manifest.json` appeared late with contradicting hashes; docs carry stale zip hash `9B2ED5A7…` | SEAT-05 S5-01/02/03/12; SEAT-08 S-1..S-14; SEAT-07 S7-F01/F04 |

**Explicit contradictions resolved:** paper A-F01/A-F02/A-F03 are **artifact-era**, fixed in HEAD source (6 independent
seats agree); live-provider claims are **READ only** (no commit/artifact binding — SEAT-06 F-02, SEAT-04 §6); the release
manifest is **untrusted** until regenerated (SEAT-05 S5-12 supersedes S5-02); `failed`-terminal (SEAT-03 D1) and
NODE_REWIRE hardcoded `running` (SEAT-02 F-S2-02) are real defects **outside this plan's provider scope** → routed to the
Lead's UI/transport owner rather than silently ignored.

## 3. Decisions (with rejected alternatives)

- **D1 — Stream the leaf reply via documented SSE.** Use the client's SSE support (`data:`/`[DONE]`/keep-alive,
  `client.rs:150-186`) with `stream: true` + `stream_options.include_usage`; adapter forwards ordered deltas; engine
  publishes each `REPLY_DELTA` and `REPLY_DONE(usage)` from the final chunk. *Rejected:* keep single-delta (violates the
  streaming UX claim and the evidence doc's own open-gap note); fake chunking client-side (fabrication).
- **D2 — One bounded planner repair.** On empty visible content or JSON parse failure, retry once with a stricter
  "JSON only, no prose/fences" message (same budget, counted call, no unbounded loop); then refuse
  `E_PROVIDER_INVALID_RESPONSE` with `finish_reason` in the detail. *Rejected:* unlimited retries (cost), silent empty
  plan (fake success), client-side fallback plan (engine must own plan validity).
- **D3 — Budget lifecycle owned by the run.** Create `RunBudget` when the run starts; every provider call (planner, leaf,
  repair, retries) goes through `preflight`/`record`; drop/evict the entry in a `forget(run_id)` on terminal (success,
  failure, cancel). *Rejected:* lazy create + never evict (memory growth); two independent counters (engine vs provider).
- **D4 — One call-count authority.** Provider `RunBudget.attempts` (40 incl. retries) is the sole authority; engine
  `check_calls(1)` is kept only as a pre-send fast refusal for planning and documented as advisory. *Rejected:* duplicate
  engine counter for leaves (drift/undefined semantics), removing provider cap (retries uncounted).
- **D5 — Load prompt/persona templates at boot.** `TemplateRegistry::load(assets/prompts)` + `load_personas`, fail by
  name (`E_TEMPLATE_*`, `E_PERSONA_UNKNOWN`); pass the registry into `RunEngine`; planner uses `planner` template when
  present, raw prompt otherwise (explicit branch, no silent fallback). *Rejected:* keep empty registry (A-F04 remains),
  load lazily per run (licence gate at wrong time).
- **D6 — Subprocess env allowlist.** All `Command` spawns in tools/core call `env_clear()` then add an allowlist
  (`PATH`, `SystemRoot`, `SystemDrive`, `TEMP`, `TMP`, `HOME`, `USERPROFILE`, `PATHEXT`, locale vars) and pass no
  `DEEPSEEK_*`/`HIVEROUTER_*` variables; regression test spawns a child that dumps env keys and asserts `DEEPSEEK_API_KEY`
  absent (no real key needed — use a canary env value). *Rejected:* rely on audit-ledger redaction (does not stop child
  exfiltration), document-only fix (claim unsafe in user docs).
- **D7 — Acceptance gate is HEAD-bound.** Clean commit `C` + tag re-cut; `cargo test` 4 crates with HEAD/exit-code log;
  wiremock-based provider run-path test; env-clear test; template-load test; web tests; single operator-authorized live
  smoke on the rebuilt artifact. *Rejected:* accepting current dirty tree / pre-integration artifacts / prose receipts
  (SEAT-07 S7-F01, SEAT-08 S-1..S-8).

## 4. File-by-file change

1. **`backend/crates/hiverouter-provider/src/client.rs`** — add a public streaming entry
   (`stream_chat_completion(budget, request, estimate, cancel, on_delta) -> Completion/Usage`) reusing `open_stream`
   (`:509-524`), `parse_sse_line` (`:150-186`), retry/status mapping (`:352-357,432-460`), and the documented
   `stream_options.include_usage`; fix the stale "usage not documented" comment (`:11-13`). Edge cases: idle timeout
   (`stream_idle_timeout`), `[DONE]` without chunks, 401/402 no-retry, empty stream → `E_PROVIDER_INVALID_RESPONSE`,
   usage absent on final chunk → `E_USAGE_INCOMPLETE` path preserved.
2. **`backend/crates/hiverouter-core/src/provider_executor.rs`** — `execute` uses streaming and forwards deltas
   (order preserved, cancel via `select!`); planner repair attempt (D2); `complete()` keeps preflight; add
   `forget(run_id)` / move budgets map behind lifecycle helpers; log `finish_reason`, request id, latency (no key).
3. **`backend/crates/hiverouter-core/src/execution.rs`** — after terminal status (completed/failed/cancelled), call
   adapter-agnostic budget cleanup seam (extend `ProviderExecutor` trait with `fn forget_run(&self, run_id)` default
   no-op) and remove/downgrade `check_calls(1)` semantics per D4 (advisory pre-check only); keep partial-state
   preservation and refusals unchanged.
4. **`backend/crates/hiverouter-provider/src/budget.rs`** — no cap-value changes (B1–B10 are PROPOSED-POLICY); add
   explicit `RunBudget::release()`/Drop note or engine-owned map; ensure `preflight` remains the only send gate; add
   test that retries count toward `call_cap` and that a released run cannot be recharged.
5. **`backend/crates/hiverouter-core/src/http.rs`** — load templates at boot (`build_templates(&config)`), fail-fast by
   named refusal if assets are missing/corrupt/licence-invalid; pass registry to every `RunEngine::new` (all three
   constructors); optional `/health` provider status fields (enabled/key-present/model — no key value).
6. **`backend/crates/hiverouter-core/src/config.rs`** — add `workspace_root`-independent `assets_dir` resolution
   (repo-relative default, `HIVEROUTER_PROMPTS_DIR` override); keep unknown env values refused.
7. **`backend/crates/hiverouter-tools/src/shell.rs`** — `Command` spawn: `env_clear()` + allowlist builder + keep
   `kill_on_drop`, stdio capture, win32job containment; no behavior change to jail/size/timeout refusals.
8. **`backend/crates/hiverouter-tools/src/cli_support.rs`** — `--version` probe spawn: env_clear allowlist.
9. **`backend/crates/hiverouter-tools/src/jail.rs`** — `cmd` spawn for jail helpers: env_clear allowlist.
10. **`backend/crates/hiverouter-core/src/checks.rs`** — `node --version` spawn: env_clear allowlist (keep node floor).
11. **`backend/crates/hiverouter-core/tests/`** — new `provider_stream.rs` (wiremock SSE → ordered deltas + usage),
    `provider_retry.rs` (empty then valid plan; 2 calls), `budget_lifecycle.rs` (eviction + cap incl. retries),
    `templates_boot.rs` (assets load; missing asset → named refusal), `subprocess_env.rs` (canary env absent in child,
    POSIX+win), `ws_transport.rs` (run-path over WS with wiremock executor: ACK→RUN_CREATED→deltas→REPLY_DONE→RUN_STATUS,
    no paid call).
12. **`backend/crates/hiverouter-provider/tests/`** — extend `refusal_matrix.rs`: streaming 401/402/429/timeout/budget
    and malformed SSE → named codes; final-chunk usage absent → `E_USAGE_INCOMPLETE` (no zero).
13. **`docs/evidence/live-provider-smoke.md` / `docs/RELEASE-EVIDENCE.md`** — regenerate only from the sealed build:
    commit `C`, binary sha256, command + exit code, ordered delta count, usage; current files stay as historical READ
    evidence and are superseded, never edited to look green.
14. **`tools/build-release.ps1` / `tools/release-manifest.json` / `dist/CHECKSUMS.txt`** — one deterministic release
    command: web build → cargo build → copy → CHECKSUMS + zip → manifest with version/tag/commit/rustc/web-dist hash;
    manifest+CHECKSUMS must agree with the binaries actually in `dist/` (fixes SEAT-05 S5-02/03/12, SEAT-08 S-2/S-3/S-4).

## 5. Failure modes

- **Key absent/disabled:** server boots, `UnconfiguredExecutor` → `SUBMIT_PROMPT` fails by name `E_KEY_MISSING`; never a
  fake reply (preserved; http.rs fallback).
- **Streaming mid-call failure:** partial deltas already published; run fails with the named provider code and preserves
  partial state (`run_lifecycle` semantics); no `REPLY_DONE` fabrication.
- **Planner repair fails twice:** run fails `E_PROVIDER_INVALID_RESPONSE` with finish_reason; budget counters incremented
  for both attempts; no plan invented.
- **Budget/call cap reached:** `E_CALL_LIMIT`/`E_BUDGET_EXCEEDED`/`E_TOKEN_BUDGET_EXCEEDED` before any HTTP send
  (preflight order unchanged); run fails with cost/tokens NULL where unmeasured (never 0).
- **Usage absent on final chunk:** run becomes INCOMPLETE; further sends refused `E_USAGE_INCOMPLETE` (existing rule).
- **Subprocess spawn with env allowlist:** child missing PATH/temp → named failure from the tool path (no silent success);
  a child that attempts to read `DEEPSEEK_API_KEY` sees it absent.
- **Template asset missing/corrupt/bad licence at boot:** server fails closed with the named `E_TEMPLATE_*` refusal (or
  refuses to start, decision recorded) rather than silently using raw prompts.
- **Concurrent runs:** single-run policy unchanged (`E_RUN_ACTIVE`); budget map keyed by run and released on terminal;
  no cross-run cap contamination.
- **Rollback flag:** `HIVEROUTER_PROVIDER=disabled` returns to `UnconfiguredExecutor` (runtime kill switch).

## 6. Tests + commands (exact acceptance)

```powershell
# 0) quiescence (clean tree at commit C, not current dirty tree)
git rev-parse HEAD ; git status --porcelain   # expect: C ; empty

# 1) backend gate (4 crates), HEAD-bound log
cd backend
$env:PATH="$env:USERPROFILE\.cargo\bin;$PWD\..\tools\w64devkit\bin;"+$env:PATH
cargo test -p hiverouter-contract -p hiverouter-core -p hiverouter-provider -p hiverouter-tools 2>&1 | Tee-Object gate-C.log
# accept: exit 0; "test result: ok." 0 failed; JUnit/JSON counts read from artifact; every ignored test named
cargo test -p hiverouter-core --test provider_stream --test provider_retry --test budget_lifecycle --test templates_boot --test subprocess_env
cargo test -p hiverouter-provider --test refusal_matrix
cargo test -p hiverouter-core --test ws_transport --test http_boot --test run_lifecycle

# 2) frontend gate (no key, no paid call)
cd ..\web; npm.cmd ci; npm.cmd run typecheck; npm.cmd test -- --run; npm.cmd run build
# accept: exit 0 each; ≥46 unit tests; replay/state/settings tests from SEAT-03 D7 included by the UI wave (owner)

# 3) release build + artifact evidence (no provider call)
cd ..\backend; cargo build --release --locked
Get-FileHash target\release\hiverouter.exe -Algorithm SHA256
# accept: manifest + dist/CHECKSUMS agree with rebuilt binaries; docs updated from this output only

# 4) live smoke — ONLY with explicit operator key + --yes, one call, budget cap (outside CI)
$env:DEEPSEEK_API_KEY='<operator supplied>'; $env:HIVEROUTER_LIVE_SMOKE='1'
cd .. ; cargo test -p hiverouter-core --test provider_live -- --ignored --nocapture
# accept: plan valid; streamed deltas ≥2; reply non-empty; usage.prompt_tokens Some; no key in logs; cost within cap
```

Acceptance criteria (all required): (a) subprocess child cannot see `DEEPSEEK_API_KEY`; (b) wiremock stream produces
order-preserving deltas + final usage; (c) planner repair observed (2 calls, second request contains stricter instruction);
(d) budget entry evicted after terminal (map size 0 after run); (e) 40-call cap includes retries; (f) template boot fails
by name on a missing/corrupt asset and engine receives the loaded registry; (g) 4-crate suite green with 0 failures and
named ignores; (h) artifact hashes/manifest/docs agree from one release command; (i) live smoke receipt bound to the
sealed binary+commit.

## 7. Rollback

- Each area is an independent, additive commit; inverse = `git revert <commit>` (no DB schema/data migration in this plan).
- Runtime kill switch: `HIVEROUTER_PROVIDER=disabled` restores `UnconfiguredExecutor` (`http.rs:65-82`) without a rebuild.
- Streaming change: keep `complete` non-streaming path compiled and selectable by config flag during the first release
  cycle (`HIVEROUTER_STREAMING=off`), then remove the flag after live proof — rejected only if a flag is judged to add
  production branches; otherwise revert the adapter/client commit.
- Template boot: revert to `TemplateRegistry::default()` returns the pre-change behaviour (raw prompt), documented as the
  previous state.
- Env-clear allowlist: if a legitimate child needs an extra variable, add it to the allowlist in a fix-forward commit; the
  allowlist itself is an explicit, reviewable function.
- Do **not** move tag `v1.0.0`; cut a new tag from `C` and mark the old one superseded (SEAT-05 §7).

## 8. Out-of-scope / blockers

- **Out of scope:** UI/transport fixes owned by other seats (SEAT-03 `failed`-terminal D1, replay read-only D2, replay
  shape D3, HTTP timeouts D5; SEAT-02 UI refusal table 45/65, NODE_REWIRE state, constant-time token compare); e2e
  harness fixes (skip-as-green, baseURL, S-APPROVAL); trace/claims/doc regeneration beyond the release evidence files;
  new product features; paid-key provisioning; Tauri/installer.
- **Blockers for execution (not for planning):** (1) current tree dirty and tag/HEAD mismatch → need a clean branch and
  a sealing decision; (2) no operator authorization for the single live smoke yet; (3) `tools/release-manifest.json` and
  docs hash disagreements must be regenerated, not patched by hand; (4) A-F06 money-overflow fix (`provider/src/accounting.rs:330`)
  is outside this plan's files — Owner: provider/budget wave (required before any cost claim); (5) A-F07 duplicate lock
  majors unresolved (`cargo tree -d` not run) — release evidence must record the verdict or an accepted deviation.

## 9. UNVERIFIED (must not be quoted as proven by this plan)

1. Any pass count at HEAD `5c69b66` — no test/build was run in planning; landed logs are pre-integration (SEAT-07).
2. Live provider behaviour at HEAD (streaming shape/cost/time) — no key/paid call made; evidence docs are READ-only.
3. Whether the current `dist/` artifacts were built from HEAD or the dirty tree — no build metadata (SEAT-05 §8).
4. `cargo tree -d` duplicate-major verdict (A-F07) — not run.
5. `docs/evidence/*` git-tracked/manifest status after the latest doc edits — not re-checked.
6. Whether the UI's replay/state fixes were browser-proven — SEAT-03 §6 lists them UNVERIFIED (e2e owner).
7. Exact e2e counts on a rebuilt artifact — the 79/0/14 log predates the provider commit.
