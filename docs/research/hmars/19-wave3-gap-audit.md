# 19 — Wave 3, seat 19: meta-gap and contradiction audit of waves 1–2 and the current HiveMind/UI

```yaml
gate: EVIDENCE-GATE-v1 (input) -> defines EVIDENCE-GATE-v2 (§7)
lane: wave 3, seat 19 — meta-gap / contradiction audit
lane_id: w3-s19-gap-audit
date_local: 2026-11-07T22:13:56-06:00
date_utc: 2026-11-08T04:13:56Z
freeze_time_local: 2026-11-07T22:13:56-06:00
workspace: C:\Users\Riley\Desktop\hiverouter-v1
head: bd575ba7adf3a5f2a9bdbc69f9bbf7084fe6fe65
head_subject: "feat: real SuperAgent/Subagent personas and persona prompt execution"
deployed_binary_sha256: 31252FABA1F62E2213763E22B2BEC0EA18975507EC8717DAC9EEBBEBBE63867D
deployed_binary_bytes: 15127040
deployed_build_time_local: 2026-11-07T22:00:18-06:00
deployed_launch_time_local: 2026-11-07T22:00:32-06:00
prior_binary_sha256: DDEA5496CF91FCFDA7DE07DCDB366B1FB1ED9A0D35666D9B028E1CF84A3EEEF6
prior_binary_build_time_local: 2026-11-07T21:43:54-06:00
live_app: http://127.0.0.1:8099
primary_discipline_requested: /deep-inspection
primary_discipline_used: /deep-investigate + /trace (substituted; see §1.2)
product_edits: none
paid_calls: none
restarts_by_this_seat: none
credentials_read: none
deliverable_path: docs/research/hmars/19-wave3-gap-audit.md
```

> **Deliverable:** this file only. No product source, test, config, migration, credential,
> server process or paid provider call was touched by this seat. The server was observed
> read-only; a concurrent lane rebuilt and restarted it at 22:00 local while this audit was
> running (§1.3). That is itself finding **F-01**.

---

## 0. Executive answer — twelve findings that change what wave 3 may assume

1. **The tree is not one artifact; it is four.** Wave 3 inherits HEAD `bd575ba`, a dirty
   working tree (5 Rust source files, 5 test files, 2 UI files, the launcher, the truth doc,
   plus build artifacts), a deployed binary built at 22:00:18 from that dirty tree, and 18
   research documents written against earlier states of all three. Every wave-2 report is
   correct *for a state it names*, and at least eight are now stale or contradicted for the
   current state (§3, §4).
2. **The deployed app is no longer the app the reports audited.** The 21:43 build could not
   have contained the Host guard (the guard was written at 21:52:53, after that build); the
   22:00 build rejects a hostile `Host` with **HTTP 421** (measured twice, §1.3). The 22:00
   build also carries the tool continuation persona fix, the `args_preview` render, and
   (inferred from build order) the 12-turn/40-call caps and `E_CALL_LIMIT`. No report in
   `01`–`18` records a deployed-build hash; none can be replayed.
3. **`E_CALL_LIMIT` now has at least five meanings** and its contract message is false for
   four of them. The working-tree “fix” replaced a lying `E_BUDGET_EXCEEDED` with a code whose
   frozen text says *“The run exceeded the 40-call cap or the 40-super-step plan ceiling.”*
   It is now also raised for the **12-turn** tool cap, the **8 approval waits**, and the
   **32-message** A2A budget. A money lie was traded for a count lie (§4 X2/X3).
4. **The tool-turn cap is enforced per run, not per task, despite its own doc comment.**
   `MAX_TOOL_TURNS` says “in one task”; `next_turn` receives `run_tool_counters(run.id)`.
   A 12-turn budget is therefore shared by every tool-using task in the run (the planner may
   emit up to 8 nodes): ~1.5 turns per task at full fan-out. Raising 4→12 did not fix the
   failure mode it was meant to fix (§4 X1).
5. **There are three different “40” limits** (provider `CALL_CAP=40`, tool
   `MAX_CALLS_PER_RUN=40`, engine `caps::MAX_CALLS=40` — the last is a stateless compare that
   never fires). Which one binds a run is undetermined and untested; the reports quote
   different ones as “the” cap (§4 X5).
6. **The workspace root is now the product’s own git checkout.** The portable launcher
   hard-codes `C:\Users\Riley\Desktop\hiverouter-v1` and passes it as `--workspace-root`; the
   running server’s command line proves it. File tools can read `.git`, source and docs, and
   (approval-gated) write them. The server’s documented env var
   `HIVEROUTER_WORKSPACE_ROOT` is **not** the launcher’s `HIVEMIND_WORKSPACE_ROOT`, so an
   operator who sets the documented variable is silently overridden by the hard-coded path
   (§4 X6).
7. **Approval evidence is better but not durable.** `args_preview` (≤512 UTF-16 units,
   redacted) is rendered by the deployed bundle, but the producer sets it only on the fresh
   publish path (`approval_request_event_with_args`); the persisted-row projection used for
   replay/hello hard-sets `args_preview: None`. After a reconnect the operator is blind again,
   by design, and the overlay still has no Dismiss/focus contract for stale entries (§4 X8).
8. **The persona fix is real but does not reach the default path.** Normalization plus a
   persona continuation are in the deployed binary, yet the planner prompt asks for
   `role:"worker"`, and there is **no `worker` persona**. Most leaves still get
   `TASK_SYSTEM`; the truth doc’s “leaf prompts use persona bodies” is true only for
   persona-named roles. Engine-side `allowed_tool_ids`/`max_children` lookup is still exact
   string, so a UI-spawned `SuperAgent` misses its persona caps and allowlist (§4 X9/X10).
9. **A security claim in three reports is now false in the working tree and true at HEAD.**
   `06` T13, `09` Q18 and `10` C79 say “no Host allowlist”. HEAD had none; the working tree
   gained `host_guard` at 21:52:53 (uncommitted at that instant, deployed by 22:00:32), and the
   live probe returns 421 twice. Any wave-3 report that repeats “no Host allowlist” is wrong
   (§4 X7).
10. **`/diagnose` lies about uptime and `/api/session` is unauthenticated.** `/diagnose`
    constructs a new `ServiceState` per request, so `uptime_secs` is always 0 (measured 0 vs
    `/health` 88 and 89, twice); the session token is issued to any loopback GET whose `Host`
    is loopback (97-byte JSON at 21:53) (§4 X12/X13).
11. **The contract has an ad-hoc refusal code.** `DELETE /api/agents/profiles/{id}` returns
    `E_PROFILE_NOT_FOUND` (`http.rs:823`), which is not in the 65-code census, not in
    `Refusal`, and not in `web/src/api/contract.ts`. A refused mutation can therefore render
    as an unknown error. This is a frozen-contract violation, not a style issue (§4 X14).
12. **Operator coverage is still a decision desert, not a design gap.** Autonomy modes 1–4
    (no definition, no enforcement locus), approval-policy settings, editable capabilities,
    all-agent card census, Remote/Watchdog entity, no-hard-default budget, Venn memory and
    “Now” are each either unbuilt or contradicted by a wave-2 design. Wave 2 produced three
    incompatible daemon models (12, 14, 18) and two memory topologies (04, 13) that both claim
    “one v11” (§5, §4 X20/X21).

**Verdict.** Wave 1’s code reads are strong and mostly still true at HEAD. Wave 2’s designs
are coherent *within each lane* and inconsistent between lanes; its evidence discipline is
better than wave 1 but still has no build identity, so a working tree that moved three times
under it (and once more under this audit) invalidated conclusions without invalidating the
documents’ confidence. The blocking wave-3 problem is not missing research. It is **unsettled
authority**: which artifact is the baseline, which code means which limit, and which document
owns each operator answer. §6 is the twenty questions that must be answered before any wave-3
implementation lane can be accepted.

---

## 1. Method, baseline, and the moving corpus

### 1.1 What was read

| Corpus | How read |
|---|---|
| `docs/research/hmars/01`–`08` | full claim registers, Q&A sets, top-10s and the sections whose claims are load-bearing for this audit; not every line (each report is 60–170 KB) |
| `09-synthesis-and-goal.md` | full |
| `10-gap-meta.md` | sections 2–9 in full (the prior meta-audit; audited in turn), appendices sampled |
| `11`–`18` | claim registers, decision lists, contradiction/errata sections, and the sections named in §3 |
| `OPERATOR-DECISIONS.md` | full (107 lines, 45 bullets — not 36; see §5.1) |
| `docs/provider/AGENT-ARCHITECTURE-TRUTH.md` | full (HEAD and working-tree diff) |
| current source | `caps.rs`, `tool_loop.rs`, `execution.rs` (tool loop, seed context, spawn policy, approval dispatch), `provider_executor.rs`, `http.rs` (router, host guard, session, mutations, usage), `refusal.rs`, `budget.rs`, `client.rs` (diff), `jail.rs`, `health.rs`, `store.ts`, `ApprovalOverlay.tsx`, bundle strings |
| live app | `GET /health`, `/diagnose`, `/api/session`, `/api/agents/profiles`, `/api/agents/memory`, `/api/history/runs`, `/api/projects`, `/api/provider/models`, plus two discriminating probes (§1.3). Read-only; no prompt submitted |

Not read line-by-line: the external-source sections of `01`, `03`, `06`, `11`, `13`, `16`,
`18` (Web citations, not local truth), and the deep design sections of `12`/`13`/`14` that
describe unbuilt systems. Those are marked **READ** below, never **VERIFIED**.

### 1.2 Discipline substitution — stated, not hidden

The brief names `/deep-inspection` as the primary discipline. It is **not installed** in this
profile (no such skill in the session catalog). Substitutes used, per contract §4:

- **`/deep-investigate`** (installed) — baseline, three hypotheses per symptom, elimination,
  reproduction twice. Applied to: Host-guard state, `/diagnose` uptime, live-vs-source drift.
- **`/trace`** — the UI/event/terminal ladder is used as the citation discipline for every
  claim of the form “the UI shows X because the wire carries Y”; the wire struct is cited
  before the component is.
- **`/deep-research`** is **not** used: no external page was fetched by this seat. All
  external facts below are **READ** from the report that fetched them, and are labelled so.

Capability limits recorded: no browser automation (so no DOM/screenshot evidence); no paid
provider call permitted (so no 12-turn/`E_CALL_LIMIT` runtime proof — see
`E_NOT_REPRODUCIBLE`, §4 X1/X2); `maxDepth: 1`, so no independent verifier child.

### 1.3 Baseline and live-probe log (America/Chicago; UTC = +6h)

| Time local | Probe / event | Observation | Status |
|---|---|---|---|
| 21:52:07 | session start; `git status` | 21 modified/untracked entries; `http.rs` **not** among them; HEAD `bd575ba` | VERIFIED |
| 21:52–21:56 | `GET /health` | 200; version `1.2.0-dev`; uptime 542 s; schema 10; `graph_nodes` 35; `allowed_origins` 2 | VERIFIED |
| 21:53 | `GET /diagnose` | 200; `call_cap 40`, `run_cost_cap_usd "0.50"`, `token_budget_combined 400000`, 65 refusal codes, 9 tools, 4 tiers | VERIFIED |
| 21:53 | `GET /api/session` | 200; returns the session token to any loopback GET (97-byte JSON) | VERIFIED |
| 21:53 | `GET /api/agents/profiles` | 200; 7 personas (chatbot, code-reviewer, content-writer, data-analyst, planner, subagent, superagent) | VERIFIED |
| 21:53 | `GET /api/agents/memory` | 200; 7 role banks, all arrays empty | VERIFIED |
| 21:53 | `GET /api/history/runs` | 200; 5 runs, 3 failed `E_BUDGET_EXCEEDED`, 2 completed; newest `2026-11-08T03:26:36Z` | VERIFIED |
| 21:54 | `GET /api/provider/models` | 200; 2 models; context 1,048,576; max output 393,216 | VERIFIED (app-served data) |
| 21:56 | process inspection | PID 9752 `hiverouter.exe --bind 127.0.0.1:8099 --data-dir … --workspace-root "C:\Users\Riley\Desktop\hiverouter-v1"`; binary hash `DDEA5496…` = `dist` = `target\release` build 21:43:54 | VERIFIED |
| 21:56:53 | `http.rs` mtime | **21:52:53** — another lane added `host_guard` (62 insertions, uncommitted) after my baseline | VERIFIED |
| 21:57 | server down | port 8099 closed; `hiverouter`/`HiveMind` processes gone; `cargo`/`rustc` running (started 21:56:24) | VERIFIED |
| 22:00:18 / 22:00:32 | rebuild + relaunch | new `dist\HiveMind\hiverouter.exe` hash `31252FAB…` (15,127,040 B); server PID 16616; uptime 43 s at first probe | VERIFIED |
| 22:01:00 | `client.rs` mtime | another lane rewrote 402/400/422 messages (uncommitted, **not** in the 22:00 build) | VERIFIED |
| 22:02:00 / 22:02:01 | Host probe ×2 | `Host: evil.example` → **421**; `Host: 127.0.0.1:8099` → **200** (both runs) | VERIFIED |
| 22:02:02 / 22:02:07 | uptime probe ×2 | `/health` 88 then 89; `/diagnose` **0** then **0** | VERIFIED |
| 22:02:0x | `GET /api/history/runs` | still 5 runs, newest `03:26:36Z` — no run has exercised 12/40 or `E_CALL_LIMIT` | VERIFIED |
| 22:12:09 | `GET /health` after the corpus grew | 200; uptime 646 s; same build hash `31252FAB…`; no new run | VERIFIED |
| 22:12–22:13 | research directory | wave-3 files `20`–`27` present, mtimes 22:02:02–22:12:27; `22` and `24` disagree on the live Host fact (X31) | VERIFIED |

**Freeze (hashes are SHA-256, first 16 hex; mtime local):**| File | mtime | hash16 | In HEAD? | In 22:00 build? |
|---|---|---|---|---|
| `backend/crates/hiverouter-core/src/provider_executor.rs` | 21:16:27 | `5656D179BDB68E74` | no | yes (inferred) |
| `backend/crates/hiverouter-core/src/execution.rs` | 21:31:35 | `8218C81ED1679503` | no | yes (inferred) |
| `backend/crates/hiverouter-core/src/tool_loop.rs` | 21:34:26 | `8D924F9E969B9F95` | no | yes (inferred) |
| `backend/crates/hiverouter-core/src/http.rs` | 21:52:53 | `F52B83B0AD889DF6` | no | yes (421 probe proves it) |
| `backend/crates/hiverouter-provider/src/client.rs` | 22:01:00 | not frozen — changed after build | no | **no** |
| `web/src/panels/ApprovalOverlay.tsx` | 21:39:44 | `2E360B16AE719FE1` | no | yes (bundle 21:40:12) |
| `web/src/state/store.ts` | 21:39:39 | `3C222E738E668DFA` | no | yes |
| `tools/desktop-launcher/HiveMindLauncher.cs` | 21:31:14 | `16B86ECB1A7AB6E3` | no | yes (`--workspace-root` observed) |

**“In build” is INFERRED from build order, not proven from the binary**, except where a
behaviour proves it (Host 421). The binary carries no commit fingerprint; `/health` reports
only `1.2.0-dev`. This is finding **F-01 — the corpus has no artifact identity** — and the
reason for gate rules G1/G5 in §7.

### 1.4 Three hypotheses tested (the `/deep-investigate` core)

**Symptom A — “the reports disagree about whether a Host allowlist exists.”**

| Hypothesis | Elimination | Result |
|---|---|---|
| H1: all reports are wrong; the guard has always been there | `git show HEAD:http.rs` has no `host_guard`/`host_allowed`; `git blame` marks those lines “Not Committed Yet” | eliminated |
| H2: the guard was added after the reports read the file | mtime 21:52:53; reports `06`/`09`/`10` were written 21:38–21:42; guard appears in the tree only after 21:52 | **confirmed** |
| H3: the guard exists but is not deployed | live probe: 421 twice on the 22:00 build; the 21:43 build was gone before the probe | H3 true for the 21:43 build (inferred), false for 22:00 |

**Root cause:** a code change landed during the audit window with no report-side version marker.
Reproduction is the twice-run Host probe: `421, 200` on both runs. Deterministic.

**Symptom B — “`/diagnose` says the server just started; `/health` says it has been up for
minutes.”**

| Hypothesis | Elimination | Result |
|---|---|---|
| H1: two different servers / stale connection | same bind, same PID in one request pair; both 200 in the same second | eliminated |
| H2: integer truncation | delta is 88–89 s, not <1 s | eliminated |
| H3: the diagnose payload is built from a fresh state object each request | `health.rs:315` `diagnostics()` constructs `ServiceState::new(...)`; its `started_at` is `Instant::now()` (`:161`) | **confirmed** |

**Root cause:** `uptime_secs` in `/diagnose` measures the payload construction, not the
process. Reproduction twice: `0` vs `88`, `0` vs `89`. Deterministic code path; no paid call.

**Symptom C — “did the 12/40 + `E_CALL_LIMIT` change actually deploy?”**

| Hypothesis | Elimination | Result |
|---|---|---|
| H1: the reports are right; it is uncommitted and unbuilt | source is at 21:31–21:34; a build completed 22:00:18; cargo builds the working tree | H1 false for the 22:00 build |
| H2: the change is in the deployed binary | build order + the fact that the 22:00 build also contains the 21:52 `http.rs` guard (proved by 421) | **INFERRED, not proven** |
| H3: the change is observable at runtime today | no run since 03:26Z; no paid call permitted; no scripted-provider path in the production binary | cannot test — `E_NOT_REPRODUCIBLE` |

**Honest result:** the deploy is inferred; the behaviour is untested. Reports must not say
“deployed” without a build fingerprint, and must not say “works” without a run.

---

## 2. What changed since wave 2, and which documents it invalidates

Wave 2’s own meta-audit (`10`) named five working-tree changes. Two more landed after it, one
during this audit. This table is the bridge from “wave 2 was right when written” to “what a
wave-3 implementation lane may rely on now”.

| # | Change (file, time) | Effect | Wave-2 documents invalidated or weakened |
|---|---|---|---|
| C1 | Tool continuation uses `system_prompt_for(task)`; role trim/lowercase/normalisation (`provider_executor.rs`, 21:16) | persona prompt survives tool turns for persona-named roles; Title-Case `SuperAgent`/`Subagent` now resolve; default `worker` still has no persona | `09` Q6, `07` §12.6/G2, `08` finding 1, `06` T04 — **STALE** (HEAD-true, tree-false) |
| C2 | `execution.rs` tool-cap branch returns `Refusal::CallLimit` (21:31) | tool-cap failures no longer claim a money cap | `10` X10/C87, `06` T25-adjacent — **STALE** for the working tree; the new message is also false (X2) |
| C3 | `MAX_TOOL_TURNS` 4→12, `MAX_CALLS_PER_RUN` 16→40, `next_turn`/`check_call_budget` return `CallLimit` (21:34) | tool loop allows 12 turns / 40 calls **per run** | `07` §8.2/§9.1, `16` §7.2 (4 turns), `06` §6.4 worst-case arithmetic, `03` L3 emphasis — **STALE** |
| C4 | Launcher passes `--workspace-root` (21:31) | fresh launches jail file tools to the repo (hard-coded default) | `10` D8/R3 (“live root = dist”, “not restarted”), `09` §8, `06` D6 — **STALE** |
| C5 | `argsPreview` in store + overlay (21:39) | fresh approval events render a redacted argument preview; replay still has none | `08` finding 4 (“renders only hash”) — **STALE, partially**; `15` §2.1 was correct |
| C6 | **`host_guard` + Host allowlist (21:52:53)** | hostile `Host` rejected 421; HEAD unaffected | `06` T13, `09` Q18, `10` C79 — **STALE** (not in any wave-2 report) |
| C7 | **402/400/422 message split (22:01:00)** | 402 message now says “insufficient balance”; code still `E_PROVIDER_AUTH`; 400/422 still `E_PROVIDER_INVALID_RESPONSE` | `01` §10.2, `06` C86/T25, `17` §2.2 A2 — **PARTIALLY STALE** (not in the 22:00 build) |

### 2.1 Wave-3 artifacts that landed during this audit (out of scope, one conflict flagged)

While this audit was writing, seats 20–27 wrote eight wave-3 design documents. They are **not**
audited here (the brief is waves 1–2 + the current UI), but they change the corpus and one of
them already contradicts a measured fact, so they are recorded with timestamps:

| File | mtime local | Relation to this audit |
|---|---|---|
| `20-implementation-architecture.md` | 22:06:38 | Convergent: lists the same six dirty changes incl. Host guard and `args_preview`; says Host guard is “present in the working tree, absent at HEAD” (correct) |
| `21-roles-daemons-implementation.md` | 22:12:27 | Out of scope (design); no runtime conflict observed in the sampled sections |
| `22-memory-bank-implementation.md` | 22:12:05 | **Conflicting live claim:** “live binary has no Host guard”, with `GET /health Host: evil.example → HTTP 200`. This seat measured **421 twice** at 22:02:00/01 on the 22:00 build (`31252FAB…`) |
| `23-a2a-implementation.md` | 22:11:26 | Out of scope |
| `24-ui-implementation.md` | 22:12:09 | Convergent: records 12 turns, Host guard “live 421 proof”, `args_preview` in the served bundle |
| `25-provider-cache-implementation.md` | 22:02:02 | Convergent: explicitly marks “4 turns” and the old `E_BUDGET_EXCEEDED` arithmetic stale |
| `26-scale-security-audit.md` | 22:05:26 | Convergent on X1 (“tool-turn cap is per-run, not per-task”) and on the non-portable launcher root |
| `27-open-source-daemon-catalog.md` | 22:07:27 | Out of scope |

**Finding F-02:** wave 3 began producing designs against an unfrozen baseline. `22` and `24`
disagree about a live fact measured within the same ten minutes; `22`’s probe is either
pre-22:00 (stale) or misreported. This is the exact failure the wave-3 gate must stop:
a design document citing a live probe without the build hash. It is registered as X31 and is
why §7 G1 requires the deployed binary hash.

**Consequence for the corpus.** A claim in `01`–`18` is a statement about one of four states:
HEAD, “tree at time T”, “deployed at time T”, or a design. None of the 19 documents records a
build hash or a deployed-binary identity. `08` recorded file hashes; `12` recorded a corpus
freeze; `10` recorded the five dirty hashes — the three best examples — and still none can
prove what ran. This audit fixes that for its own claims and requires it of wave 3 (§7 G1/G5).

---

## 3. Claim ledger across all reports

**Status legend (this audit).**

- **VERIFIED** — this seat read the code/runtime that decides the claim in this pass, at the
  time recorded. A VERIFIED row is not a claim about “HEAD in general”.
- **READ** — the report was read; its evidence was not independently re-executed. Safe as a
  summary, not as a foundation.
- **INFERRED** — follows from VERIFIED evidence by stated reasoning; no direct observation.
- **CONTRADICTED** — evidence in this pass is against it, for the state named in the note.
- **STALE** — it was true for a named earlier state (HEAD or an earlier tree) and is not true
  of the current tree/build. “Stale” is not “wrong when written”; it is “wrong to rely on now”.
- A row without a time qualifier is about the freeze in §1.3.

### 3.1 `01-deepseek-api.md` (provider surface)

| ID | Claim (short) | Cite | Status | This pass |
|---|---|---|---|---|
| L01 | Context 1M, max output 393,216 (`max_tokens` 1–393216) | `01` §8.3, §11 Q2 | VERIFIED (app-served) | `/api/provider/models` returns `context_window 1048576`, `max_output_tokens 393216`; the *official* page itself was not re-fetched (READ) |
| L02 | Chat Completions, thinking/effort, JSON, tools, streaming, usage/cache, models, balance are wired | `01` §10.2 | READ | Corroborated by live 2 models + balance route existing; per-path code not re-read |
| L03 | Files, Vision, Responses, Anthropic, FIM/Prefix, strict tools, `user_id` are not wired | `01` §10.3/§10.4 | READ | `ChatMessage.content: Option<String>` (`dto.rs:93`) is text-only — partial independent support; the rest inherited |
| L04 | Search/Fetch are absent; only per-request-billed surface | `01` §7.1, §10.3 | VERIFIED (absence) | live census 9 tools, none network; `PermissionClass::Network` unused (`/diagnose`) |
| L05 | Tool result is raw `role:tool`, no provenance fence | `01` §4.5 (via `06` T01) | VERIFIED (code path) | `provider_executor.rs:372-375` builds `[system, user]` then replays turns; no envelope in the continuation path |
| L06 | Prompt-token estimate is `bytes/4`; no context preflight | `01` §14 G8; `16` §4.1 | READ | `16` re-derived with the official tokenizer; not re-run here |
| L07 | Cache counters wired; realised hit rate never measured | `01` §4.6 | STALE (narrow) | `10` C11 observed one run at 1,920 hit / 2,100 prompt; “never measured” is now false for n=1, still not generalisable |
| L08 | 401/402 collapse to one auth error; 400/422 to invalid response | `01` §2.1 | CONTRADICTED (partially) | at HEAD true; tree at 22:01 splits messages but **reuses both codes**; the 22:00 deployed binary predates it |

### 3.2 `02-hmars-patterns.md` (patterns, invariants)

| ID | Claim | Cite | Status | This pass |
|---|---|---|---|---|
| L09 | `ask_parent` writes but no engine path answers | `02` §13.2 | VERIFIED | only consumer `store.rs:578` ← `http.rs:745`; no execution reader |
| L10 | `spawn_agent` runs the child inline and returns its result | `02` §13.2 | READ | `execution.rs:1691` dispatch read; await path inherited from `02`/`10` |
| L11 | Execution is sequential; one active run | `02` §13.2 | READ | corroborated by `03`/`17`; no scheduler file exists (absence grep) |
| L12 | There is no agent bus; the event bus is UI transport | `02` §13.2 | READ | `bus.rs` not re-read; `AGENT-ARCHITECTURE-TRUTH.md:26` agrees |
| L13 | Terminal states are absorbing | `02` §13.2 | READ | not re-read |
| L14 | A2A `TaskState` spellings unverified | `02` §13.2 | READ | still unverified; no A2A wire is implemented |
| L15 | X13: reject long-lived agent identity | `02` §13 | CONTRADICTED (by requirement) | operator requires Remote/Watchdog daemons (`OPERATOR-DECISIONS:32-33`); `10` X5 already ruled this unresolved |

### 3.3 `03-scale-concurrency.md` (local ceilings)

| ID | Claim | Cite | Status | This pass |
|---|---|---|---|---|
| L16 | In-flight provider requests = **8 process-wide** | `03` L1 | CONTRADICTED (scope) | `budget.rs:43` is 8, but the limiter is per client (`17` A7); “process-wide” is false once the provider proxies create clients |
| L17 | Agents/run 64; calls/run 40; tokens 400K; cost $0.50 | `03` L2–L5 | VERIFIED | `budget.rs:25,28,31,40`; live `/diagnose` repeats all four |
| L18 | Concurrent runs = 1 | `03` L6 | READ | `execution.rs` run-active path inherited; live has one run at a time |
| L19 | One SQLite `Mutex<Connection>`; WAL configured but unused for readers | `03` L7 | READ | `db.rs` not re-read |
| L20 | Event bus 1,024 slots / 256 replay; one `REPLY_DELTA` per fragment | `03` L8/L9 | READ | `bus.rs` not re-read |
| L21 | UI renders all nodes (no `onlyRenderVisibleElements`) | `03` L10 | READ | `Canvas.tsx` not re-read |
| L22 | “8 permits bind first” today | `03` §3, §10 | CONTRADICTED | execution is sequential (`17` A5); in-flight ≤1 before a dispatcher exists. `03`’s own S0 says ~0.4 calls/s |
| L23 | 50–150 / 300–600 impossible today | `03` §10; `09` §1 Q20 | VERIFIED (derived) | caps + serial dispatch + no scheduler; scale never exercised (no run in history exceeds a hand-sized plan) |
| L24 | One 16K planner call cannot enumerate 300–600 nodes | `03` §10.4 | INFERRED | arithmetic over node size; assumptions named in `03` |

### 3.4 `04-memory-architecture.md` (bank tree)

| ID | Claim | Cite | Status | This pass |
|---|---|---|---|---|
| L25 | `ScopedContext` is an in-memory per-task snapshot; parent facts copied; rewire clears | `04` C1 | READ | corroborated by `07` C11; `context.rs` not re-read |
| L26 | `agent_memory` is role-keyed, newest 8, no scope/trust/expiry | `04` C3/C4 | VERIFIED | `execution.rs:880` exact query call; live `/api/agents/memory` returns only role keys, 7 banks, empty |
| L27 | No per-hive/branch bank; no HyperAgent aggregate | `04` C8 | VERIFIED (absence) | live schema 10; no bank routes; HyperAgent allowlist is 3 tools (`/diagnose` census) |
| L28 | v11 not landed | `04` §12, §17 | VERIFIED | live `schema_version 10` |
| L29 | Prompt labels scoped facts “authoritative” | `04` C2 | READ | `context.rs:71` inherited from `10` C62 |
| L30 | Five banks: project, run, hive, branch, agent_role; no global | `04` §4.3 | CONTRADICTED (by 13) | `13` designs six scopes including `global` and calls its schema “one v11, not two” (§4 X20) |

### 3.5 `05-ui-ux-patterns.md` (UI patterns)

| ID | Claim | Cite | Status | This pass |
|---|---|---|---|---|
| L31 | Run budgets: 64 agents / 40 calls / 400k / $0.50 / 32 messages / depth 3 / fan-out 8 / 8-slot | `05` §1.3 | VERIFIED | all constants read in `budget.rs`/`execution.rs`; 8-slot caveat L16 |
| L32 | Cards carry at most four facts; the rest in an inspector | `05` §2.2, §3.4 | CONTRADICTED (by operator) | operator requires role/task/state/cost/tools/branch (`OPERATOR-DECISIONS:83-84`); `10` X13, `15` §2.3 attempt a compromise, neither settles it |
| L33 | One primary region + one inspector + status strip | `05` §3.1 | CONTRADICTED (by operator) | operator wants all projections enabled by default (`:81-82`); `10` X14 |
| L34 | Approval inbox + focused decision | `05` §2.5 | READ | design; current overlay is a single focused dialog with no inbox |
| L35 | 16 states / 5 visual families | `05` §3.5 | READ | not re-measured |
| L36 | LOD bands B0–B4 | `05` §3.2 | READ | design; no LOD code |

### 3.6 `06-security-cost.md` (threats, cost)

| ID | Claim | Cite | Status | This pass |
|---|---|---|---|---|
| L37 | T01: tool text enters the continuation unfenced | `06` T01 | VERIFIED (path) | `provider_executor.rs` tool-turn replay; no provenance marker |
| L38 | T07: `spawn_agent` skips approval despite Execute class | `06` T07 | VERIFIED | `execution.rs:1684-1691` explicit branch |
| L39 | T12: mutating routes have no token/Origin check | `06` T12 | VERIFIED | `import_agent_profile`/`delete_agent_profile`/`clear_agent_memory` read; no auth check. New `host_guard` covers **Host only** |
| L40 | T13: no `Host` validation anywhere; `/api/session` gives a token to no-Origin loopback | `06` T13 | STALE | HEAD true; tree at 21:52:53 false; deployed 22:00 false (421 ×2); `/api/session` still issues to loopback (200) |
| L41 | T04: tool continuation hard-codes `TASK_SYSTEM` | `06` T04 | STALE | tree uses `self.system_prompt_for(task)` (`provider_executor.rs:375`); deployed build inferred; HEAD true |
| L42 | T21: hard $0.50 / 400K / 40 / 64 are the only runaway bound | `06` T21 | VERIFIED | constants + live diagnose; no global/hive governor (no `E_HIVE_BUDGET` anywhere) |
| L43 | T22: preflight checks spent, not projected; one call can overshoot to ~$1.07 | `06` T22 | READ | `budget.rs` not re-read; arithmetic inherited |
| L44 | T25: 402/400/422 misclassified | `06` T25 | CONTRADICTED (partially) | tree at 22:01 splits messages; codes still `E_PROVIDER_AUTH` / `E_PROVIDER_INVALID_RESPONSE`; deployed binary predates |
| L45 | Cost is not persisted; `run_usage` has token columns only | `06` T23 | VERIFIED | `run_usage` route returns exactly prompt/completion/total/hit/miss/reasoning; no cost |

### 3.7 `07-code-backend.md` (backend truth)

| ID | Claim | Cite | Status | This pass |
|---|---|---|---|---|
| L46 | Tiers are exactly Hyper/Super/Worker/Tool; no Subagent | `07` §12.1 | VERIFIED | live `agent_tiers` = 4; persona `subagent` is tier `worker` |
| L47 | `task_agent_tier` maps unknown roles to Worker | `07` §12.2 | READ | `execution.rs:151-163` inherited |
| L48 | 7 personas; 4 worker persona ids rejected by `validate_spawn_role` | `07` §12.7 | VERIFIED | live 7 profiles; `execution.rs:113-142` name list has no `content-writer`/`data-analyst`/`code-reviewer`/`chatbot` |
| L49 | Tool-loop budget 4 turns / 16 calls / 8 waits / 64 KiB | `07` §8.2, §9.1 | STALE | tree: 12 / 40 / 8 / 64 KiB; deployed inferred; **and the turn cap is per run** (§4 X1) |
| L50 | Persona prompt used on first leaf call, not continuation | `07` §12.6 | STALE | tree fixes continuation; default `worker` has no persona, so “leaf” is narrower than stated |
| L51 | 9 tools, 4 permission classes, 4 allowlists | `07` §12.19 | VERIFIED | live census: 9 tools, 23 grants, 4 classes, 4 tiers |
| L52 | `spawn_agent` approval-free; `ask_parent`/`report_result`/`finish` are Read-class but write state | `07` §12.21/§12.22 | READ | spawn verified L38; class/write mismatch inherited |
| L53 | Planner prompt asks 1..8 nodes; validator allows ≤40 | `07` §12.25 | VERIFIED (prompt) / READ (validator) | `PLANNER_SYSTEM` says 1..=8, roles `worker`/`superagent`; validator limit not re-read |
| L54 | One active run, sequential dispatch, inline child | `07` §12.27 | READ | corroborated by `03`/`17`; no scheduler file |
| L55 | Caps 64/8/3/40/$0.50/400K | `07` §12.24 | VERIFIED | constants; live diagnose |

### 3.8 `08-code-ui.md` (UI truth)

| ID | Claim | Cite | Status | This pass |
|---|---|---|---|---|
| L56 | Persona identity lost on production paths; every UI child generic; turn 1 only | `08` #1 | STALE | tree normalises and uses persona on continuation; UI `SuperAgent`→`superagent`, `Subagent`→`subagent` resolve; `WorkerAgent`→`workeragent` and planned `worker` do not |
| L57 | No model/thinking/profile/spend settings; env-only | `08` #2 | VERIFIED | `SettingsPanel.tsx` has one reduced-motion checkbox; no settings route (grep: one comment) |
| L58 | Subagent has no visual/structural identity | `08` #3 | VERIFIED | `NODE_KINDS = hyperagent, superagent, worker, tool, error`; no subagent kind/card |
| L59 | Approvals show tool + truncated hash, no args/target; stale entries undismissable | `08` #4 | STALE (partially) | deployed bundle renders `args_preview` (bundle strings, 21:40:12); no Dismiss/focus trap remains; replay still `None` |
| L60 | Memory/A2A read-only fragments | `08` #5 | VERIFIED | routes are GET + delete-by-role; no compose/send route |
| L61 | Tool activity in two homes; no run-detail ledger | `08` #6 | READ | components not re-read |
| L62 | Graph taxonomy half-dead; replay flattens to worker | `08` #7 | READ | `graph.ts` inherited; `NODE_KINDS` has no subagent/daemon |
| L63 | Server never sends label/progress/metrics | `08` #8 | VERIFIED | `publish_node_spawn` sends `{task_id,state,rev,kind,parent_task_id}` only |
| L64 | Provider account passive/stale; no cost | `08` #9 | READ | `run_usage` verified only tokens |
| L65 | HMAS views read-only/absent; “Now” absent | `08` #10 | VERIFIED (absence) | no `Now` string in `web/src` (0 matches); `SwarmPanel`/projects read-only |
| L66 | Current app data empty | `08` §0 | STALE | 5 runs / 1 project / 35 graph nodes observed at 21:53 (matches `10` C104) |

### 3.9 `09-synthesis-and-goal.md` (lead synthesis)

| ID | Claim | Cite | Status | This pass |
|---|---|---|---|---|
| L67 | Q2: planner → plan → root + tasks → sequential dispatch → aggregation | `09` §1 Q2 | READ | matches code reads in `07`/`10` |
| L68 | Q5: no Subagent tier | `09` §1 Q5 | VERIFIED | L46 |
| L69 | Q6: continuation hard-codes `TASK_SYSTEM`; UI Title-Case misses persona map | `09` §1 Q6 | STALE | both halves fixed in tree for persona-named roles (L41/L56); engine-side lookups still exact (L77) |
| L70 | Q11: `ask_parent` is not a question/answer | `09` §1 Q11 | VERIFIED | L09 |
| L71 | Q13/Q14: memory role-global, 8 rows, no hive/branch | `09` §1 Q13/Q14 | VERIFIED | L26/L27 |
| L72 | Q17: approvals lack args/target and dismiss | `09` §1 Q17 | STALE (partially) | L59 |
| L73 | Q18: no Host allowlist; token in WS query | `09` §1 Q18 | STALE | Host guard in tree/deployed (421); WS token in query still true |
| L74 | Q20: 8 in-flight permits process-wide, 64/40/400K/$0.50 | `09` §1 Q20 | CONTRADICTED (scope) / VERIFIED (caps) | L16 for the permit scope; L17 for the constants |
| L75 | “Current app data is empty” | `09` §8 | STALE | L66 |
| L76 | 393,216 verified (384K×1024), context 1M | `09` §5 | VERIFIED (app-served) | L01 |

### 3.10 `10-gap-meta.md` (prior meta-audit — audited in turn)

| ID | Claim | Cite | Status | This pass |
|---|---|---|---|---|
| L77 | Five dirty changes; hashes recorded; findings time-scoped | `10` §2.4 | VERIFIED | matches at 21:52 except a sixth change (host guard) landed later |
| L78 | C30: 4 turns/16 calls at HEAD; 12/40 in tree; live runs used 4 | `10` C30 | VERIFIED at HEAD / STALE for tree | tree now 12/40 (L49); live runs remain 4-turn evidence only |
| L79 | X10/C87: the tool cap returns `E_BUDGET_EXCEEDED` | `10` X10 | STALE | tree returns `E_CALL_LIMIT`; deployed inferred; the new code’s message is false for 12/8 (X2) |
| L80 | Live workspace root was `dist\HiveMind`; server not restarted | `10` §2.4/¶112 | STALE | process args at 21:56 show repo root; restarted 21:44 and 22:00 |
| L81 | The late fixes are uncommitted, untested, unbuilt | `10` ¶110, R20 | STALE | uncommitted still true; **built/deployed** at 21:43 and 22:00 (Host probe proves the latter); untested still true |
| L82 | 148 claims: 115 VERIFIED / 16 READ / 4 INFERRED / 13 CONTRADICTED | `10` §3 summary | READ | self-reported; not re-derived. The VERIFIED bucket is inflated by inherited rows, as `10` itself warns |
| L83 | D7 options for a tool budget; D8 root; D12 HyperAgent memory | `10` §6 | READ | still open; wave-3 questions 2/4/9/19 restate them at the authority level |
| L84 | R2 snapshot scope: snapshots contain the global graph | `10` R2/C34 | READ | not re-tested live; no snapshot in this pass |
| L85 | Live observed 5 runs / 27 tasks / 3 `E_BUDGET` | `10` A.2 | VERIFIED (runs) | `/api/history/runs` still returns the same 5; task count not re-queried |
| L86 | V1 gate: mandatory header, claim ledger, contradiction protocol | `10` §9 | READ | inherited and tightened in §7 |

### 3.11 `11-donor-frameworks.md` (donor research)

| ID | Claim | Cite | Status | This pass |
|---|---|---|---|---|
| L87 | Field converged on delegate-as-tool + handoff; reject internal handoff | `11` §1.1 | READ | external-source claim; not re-fetched |
| L88 | No donor implements the operator’s four-tier hierarchy; two orchestrator tiers max | `11` §1.2 | READ | external-source claim |
| L89 | LangGraph approval semantics are the right model; storage copy is wrong | `11` §1.3 | READ | design judgement |
| L90 | A2A is the right data model, wrong internal transport | `11` §1.5 | READ | agrees with `02`/`14` |
| L91 | CrewAI `MemoryScope`/`MemorySlice` is the shape of the operator’s Venn | `11` §1.7 | READ | design input to `13`; no HiveMind code |
| L92 | Cargo manifests pin tokio/axum/rusqlite versions | `11` §0.3/§6 | VERIFIED | `backend/Cargo.toml` was read by `11`; not re-read here — mark READ in strict terms |

### 3.12 `12-role-capability-model.md` (roles, authority, daemons)

| ID | Claim | Cite | Status | This pass |
|---|---|---|---|---|
| L93 | Four coded tiers; no daemon | `12` C20 | VERIFIED | live census 4 tiers; 0 daemon identifiers in `*.rs` |
| L94 | Unknown role → Worker | `12` C21 | READ | inherited |
| L95 | Empty persona allowlist means “no narrowing” | `12` C22 | READ | code not re-read; consequence is fail-open |
| L96 | Persona allowlist/caps exact-string; UI Title-Case; executor normalizes | `12` C23 | VERIFIED | `persona_spawn_limits` `.get(role)` (`execution.rs:550-557`); `seed_task_context` `.get(&task.role)` (`:871-879`); executor normalizes (`provider_executor.rs:82-88`) |
| L97 | Planner default role `worker` has no persona record | `12` C24 | VERIFIED | `PLANNER_SYSTEM` asks `worker`/`superagent`; `personas.json` has no `worker` id |
| L98 | No daemon/office/ownership table or route | `12` C26 | VERIFIED (absence) | routes list; 0 grep matches; schema 10 |
| L99 | `spawn_agent` approval-free | `12` C29 | VERIFIED | L38 |
| L100 | Every agent gets a card as a roster invariant; card ≤4 facts | `12` §19 #9 | CONTRADICTED (card facts) | the census principle matches the operator; the ≤4-fact clause contradicts `OPERATOR-DECISIONS:83-84` |
| L101 | Daemon = immutable one-job obligation + short turns, generation on restart | `12` §6/§19 #4 | READ (design) | conflicts with `14` control plane and `18` record-vs-task (§4 X20) |

### 3.13 `13-memory-venn.md` (Venn memory)

| ID | Claim | Cite | Status | This pass |
|---|---|---|---|---|
| L102 | Current memory is role-keyed, cross-run; no bank table | `13` C1/C6 | VERIFIED | L26/L27 |
| L103 | Prompt calls facts “authoritative”; read errors swallowed | `13` C2/C3 | READ | inherited |
| L104 | Six scopes incl. `global` and `project`; one v11 | `13` §3/§11.1 | CONTRADICTED (vs 04) | `04` has five banks, no global; both claim v11 (§4 X20) |
| L105 | HyperAgent: digest; raw only as explicit audited setting | `13` §8.7 | CONTRADICTED (vs 04) | `04` allows summaries only, never raw; operator allows raw if leakage prevented (`:67-68`) (§4 X21) |
| L106 | Cache requires full prefix-unit match | `13` C15 | READ | external; `16` re-derives |
| L107 | The design is not implemented (`C22 FALSE`) | `13` C22 | VERIFIED | schema 10; no bank routes/tables |
| L108 | Retention explicit/human-controlled; `unset` never `forever` | `13` §6.4 | READ (design) | no retention code |

### 3.14 `14-a2a-bus-roster.md` (A2A, roster, bridge, remote)

| ID | Claim | Cite | Status | This pass |
|---|---|---|---|---|
| L109 | `ask_parent` has no answer path/deadline | `14` §17.3 | VERIFIED | L09 |
| L110 | Parent is suspended inside the inline child await | `14` §17.3 | READ | `02` L10 + await path; not re-read |
| L111 | No roster/topic/bridge/mute/remote surface | `14` §17.3 | VERIFIED (absence) | routes list; 0 grep |
| L112 | `AGENT_MESSAGE` has no id/status | `14` §17.3 | READ | event struct not re-read |
| L113 | Re-entrant answer turn is implementable | `14` §17.3 | INFERRED/PROPOSED | no build/run; highest-risk design element, correctly flagged |
| L114 | Remote/Watchdog control plane: tier+card, switch, state machine | `14` §9 | READ (design) | third daemon model in the corpus (§4 X20) |
| L115 | Mute/blind/invisible: human-only attention settings | `14` §8 | READ (design) | operator line 27 says these are human-only; line 52 says A2A must support them — tension not resolved (§4 X22) |

### 3.15 `15-ui-ux-goal.md` (UI goal, current-truth corrections)

| ID | Claim | Cite | Status | This pass |
|---|---|---|---|---|
| L116 | `args_preview` exists in contract + producer; the UI ignores it | `15` §2.1 | STALE (UI half) | contract/producer true; deployed bundle renders the row (bundle occurrence 4, 21:40:12) |
| L117 | Persona normalisation + continuation partially fix `08` #1; `worker` has no persona | `15` §2.1 | VERIFIED | matches tree (L96/L97) |
| L118 | `error` is client-only, never a server kind | `15` §2.1 | READ | contract vocabulary inherited |
| L119 | Producer omits `label`/`progress`; cards show `—` | `15` §2.2 #9 | VERIFIED | L63 |
| L120 | One graph/two projections; card registry by `agent_class` | `15` §3 | READ (design) | no `agent_class` on the wire; no `agentView.ts` |
| L121 | Settings/modes/severn meters must label “not available on this build” until routes exist | `15` §1.6 | READ (design) | no settings route exists (L57) |
| L122 | “Now” is not defined anywhere; nearest is the main-view requirement | `15` §2.4 U-?; `10` Q18 | VERIFIED (absence) | 0 `Now` strings; no live strip/destination |

### 3.16 `16-deepseek-cache-cost.md` (cache, tokenizer, budget)

| ID | Claim | Cite | Status | This pass |
|---|---|---|---|---|
| L123 | `max_tokens` 1–393216; context 1M; defaults 8K/64K/128K | `16` §12 | VERIFIED (app-served) / READ (official) | L01; defaults not re-read |
| L124 | Cache full-prefix rule and three persistence triggers | `16` §12 | READ | official page not re-fetched |
| L125 | `bytes/4` undercounts the facts block by 47% / code by 26% | `16` §4.2 | READ (their measurement) | tokenizer not re-run |
| L126 | No per-call cost persistence; no hive dimension; no request-billed dimension | `16` §12 | VERIFIED | `run_usage` route tokens only; no cost/hive columns exposed |
| L127 | Mode A default = no money cap; structural caps stay: **4 tool turns/task** | `16` §7.2 | STALE | tool cap is 12, and per **run** — the Mode A arithmetic is based on a retired number |
| L128 | 402 → new `E_PROVIDER_BALANCE`; do not reuse auth | `16` §7.2 | CONTRADICTED (tree at 22:01) | 402 now says “insufficient balance” but still returns `E_PROVIDER_AUTH`; no `E_PROVIDER_BALANCE` anywhere (0 grep) |
| L129 | Preflight reads `spent`, N-way overshoot; reservation required | `16` §7.3 | READ | `budget.rs` preflight not re-read; arithmetic inherited |

### 3.17 `17-scale-security-impl.md` (implementation plan)

| ID | Claim | Cite | Status | This pass |
|---|---|---|---|---|
| L130 | Six P0 symptoms eliminated to code paths (spawn, 402, Host, preflight, parent guard, deadline) | `17` §1.2 | READ | code paths plausible; no reproduction (correctly stated by `17`) |
| L131 | No Host middleware in `build_router` (`http.rs:510-531`) | `17` §1.2 | STALE | guard added 21:52:53; deployed 22:00 (421) |
| L132 | 8 permits are not the binding ceiling before parallel dispatch | `17` A5 | READ | agrees with `03` L22 contradiction; sequential execution not re-read |
| L133 | Limiter is per `ClientConfig`, not process-wide; proxy calls create clients | `17` A7 | READ | not re-read |
| L134 | Needs new codes `E_PROVIDER_BALANCE`, `E_QUEUE_FULL`, `E_DEADLINE`, `E_STALLED`, `E_HIVE_BUDGET`, `E_CONTEXT_WINDOW_EXCEEDED` | `17` §3.3 | VERIFIED (absence) | 0 matches for each in `backend`/`web/src`; live census still 65 |
| L135 | Current tests pin the wrong 402 behavior (`error_matrix.rs`) | `17` A16 | READ | still plausible: code still returns `E_PROVIDER_AUTH` for 402, so the old assertion still passes |
| L136 | The design adds a contract delta before S1/S3 | `17` §3.3 | READ (design) | not implemented |

### 3.18 `18-open-source-agents.md` (OSS donors, daemons)

| ID | Claim | Cite | Status | This pass |
|---|---|---|---|---|
| L137 | `11-donor-frameworks.md` does not exist | `18` §14 #8 | CONTRADICTED | `11` exists: 997 lines, mtime 21:32:48, before `18` was written (21:37:55); a false negative in a claim register |
| L138 | No `watchdog`/`daemon`/`remote_agent` identifier in `*.rs` | `18` §14 #6 | VERIFIED | 0 grep matches across `backend/**/*.rs` |
| L139 | 7 personas; none is a daemon/watchdog | `18` §14 #4 | VERIFIED | live 7 profiles, none daemon |
| L140 | `PersonaTier` has exactly four variants | `18` §14 #1 | READ | inferred from live 4 tiers; enum not re-read |
| L141 | 150 daemons × 288 ticks/day ≈ 43,200 rows/day; 1.30 M/month; 259–648 MB/month | `18` §13 C7 / §14 #23 | ESTIMATE (their arithmetic) | recomputed: 150×288=43,200; ×30=1,296,000; ×200 B=259.2 MB; ×500 B=648 MB — arithmetic is right, assumptions stand |
| L142 | Daemon as record vs task is an open entity question | `18` §6.1 | READ | unresolved; third daemon model (§4 X20) |
| L143 | AutoGen archived, must not be a dependency | `18` §14 #15 | READ | external; not re-fetched |

### 3.19 Ledger summary

| Status | Rows | Meaning for wave 3 |
|---|---:|---|
| VERIFIED (this pass) | 50 | Safe to build on **for the frozen state**; not a promise about HEAD or the next build |
| READ (inherited) | 58 | Usable as summary; must be re-derived before it appears in an implementation plan |
| INFERRED | 2 | State the inference; do not quote as observation |
| CONTRADICTED | 14 | Do not carry forward; one side must be retired with a settlement test (§4) |
| STALE | 18 | True for a named earlier state; a wave-3 lane that quotes it is wrong today |
| ESTIMATE (the report’s arithmetic) | 1 | Inputs and formula are checkable; the result is not an observation |
| **Total** | **143** | 143 material rows across 19 documents. Counts use each row’s first status where a row carries a compound status; the counts are this seat’s marks, not the reports’ self-assessment |

**The five claims the brief asked about, resolved:**

| Brief item | Status | Resolution |
|---|---|---|
| **12/40 caps** | STALE in 4 reports / VERIFIED current | True in tree: 12 turns / 40 calls. **Wrong in four reports** that still say 4/16. Worse: the 12-turn cap is measured against per-**run** counters, so it is shared across all tool-using tasks; and a third “40” (provider `CALL_CAP`) plus a dead fourth (`caps::MAX_CALLS`) make “the cap” ambiguous |
| **`E_CALL_LIMIT`** | VERIFIED (five meanings) / CONTRADICTED (message) | Now the code returned for the tool-turn cap, the tool-call cap and approval-wait exhaustion; already the code for the provider call cap and the 32-message A2A budget. Its frozen message names only the 40-call/40-step meanings. The money lie is fixed; the counting lie is not |
| **workspace root** | STALE (`10` D8) / VERIFIED (current) | Deployed root = `C:\Users\Riley\Desktop\hiverouter-v1` (repo). Launcher hard-codes it; portable fallback is cwd, not the package dir; documented server env `HIVEROUTER_WORKSPACE_ROOT` is ignored by the launcher, whose own var is `HIVEMIND_WORKSPACE_ROOT` |
| **approval argsPreview** | STALE (`08` #4) / PARTIAL (current) | Field exists in the contract at HEAD; producer attaches it on fresh publish; UI renders it in the deployed bundle; replay/persisted projection omits it; no Dismiss/focus; no target/path derivation |
| **tool continuation persona** | STALE (`07`/`08`/`09`/`06` T04) / VERIFIED fix / default path CONTRADICTED | Fixed in tree and (inferred) deployed; **does not reach the default `worker` role**, and engine-side allowlist/caps lookups remain exact-string, so UI Title-Case roles miss capability narrowing |

---

## 4. Contradiction register — every contradiction with a settlement test

A contradiction is settled only by a **command, a named assertion, or a quoted artifact** that
another seat can re-run. “Operator decides” is recorded as *unsettled by design*, with the
question id that carries it (§6). “Both sides are design” is recorded as a merge conflict,
not a runtime contradiction.

### 4.1 The register

| ID | Contradiction | Side A | Side B | Class | Settlement test (re-runnable) |
|---|---|---|---|---|---|
| **X1** | Tool-turn cap scope | `tool_loop.rs:13` “Maximum model turns in **one task**” | `execution.rs:1592-1595` feeds `run_tool_counters(run.id)` into `next_turn`; `tool_loop_engine.rs` asserts `(MAX_TOOL_TURNS, MAX_TOOL_TURNS)` per run | **Runtime semantics (code vs doc)** | Scripted provider: one plan with 2 sequential tool tasks × 7 turns each. Assert task 2 is refused at its 5th turn with `E_CALL_LIMIT`. If yes, the cap is per run and the comment is wrong. No paid call: use the existing `ScriptedExecutor` harness |
| **X2** | `E_CALL_LIMIT` message vs five raising conditions | `refusal.rs:158-160` “The run exceeded the 40-call cap or the 40-super-step plan ceiling” | raised at `execution.rs:2028` (8 approval waits), `:2452`/`:2573` (32-message budget), `:2849` (12-turn cap), `:2857` (call budget); `tool_loop.rs` says the approval cap uses the “frozen `E_CALL_LIMIT`” | **Contract truth** | Table test: force each of the five conditions in isolation and assert `code + message`. Expected today: four of five print a message that names neither the cause nor the right number |
| **X3** | There are three “40” call limits | `budget.rs:31 CALL_CAP=40` (provider calls incl. retries) | `tool_loop.rs:16 MAX_CALLS_PER_RUN=40` (tool calls); `caps.rs:13 MAX_CALLS=40` compared statelessly at `calls=1`, so it never fires (`check_calls(1)`) | **Which bound binds** | Instrument one research run with counters per layer; assert which code fires at 41. Separately delete `caps::MAX_CALLS` and prove no test changes — if none does, it is dead code |
| **X4** | Turn cap “raised to 12” fixes normal research | `10` D7 / working-tree change | per-run scope (X1) + 8 approval waits + 40 provider calls: an 8-node plan gets ~1.5 turns/task | **Claim vs arithmetic** | Same harness as X1 with the planner's default 8 nodes; assert completion or refusal, and record turns/task. This is the test that should have accompanied 4→12 |
| **X5** | `E_BUDGET_EXCEEDED` meaning | `refusal.rs:152-154` “The run exceeded the $0.50 cost cap” | live runs 21:23–21:25 failed with this code at ~3.3K tokens (not $0.50); tree now returns `CallLimit` for the tool cap | **Historical truth / current** | Re-run the same three prompts on the 22:00 build; assert the code is `E_CALL_LIMIT` and the message is still wrong (X2). No paid call permitted → `E_NOT_REPRODUCIBLE` for the live half; the code half is settled by reading |
| **X6** | Workspace root | `06` D6/`10` D8: live root `dist\HiveMind`; server not restarted | process args at 21:56: `--workspace-root C:\Users\Riley\Desktop\hiverouter-v1`; portable launcher hard-codes that path (`HiveMindLauncher.cs:17,61`) | **Observed vs observed (time)** | `Get-CimInstance Win32_Process` command line; then set `HIVEROUTER_WORKSPACE_ROOT=D:\tmp\proj`, relaunch through `HiveMind.exe`, and assert the resolved root. Expected: still the repo, because the launcher ignores the server's env var |
| **X7** | Host allowlist exists | `06` T13, `09` Q18, `10` C79: no Host validation | `http.rs:533,542-593` host_guard (tree, 21:52:53); live 22:02: **421** twice vs 200 control | **Code-state drift** | `Invoke-WebRequest /health -Headers @{Host='evil.example'}` → expect 421. Also `git show HEAD:...http.rs | Select-String host_guard` → empty. Both are the settlement |
| **X8** | Approval preview durability | `15` §2.1 / bundle: UI renders `args_preview` | `tools_bridge.rs:571` uses `approval_request_event` (sets `None`); only `:464/:477` attach args | **Partial fix** | Request a `file_write` approval, disconnect, reconnect/replay the run: assert the preview is absent and the digest present. Then assert a fresh event carries ≤512 chars and redacts `sk-`/key-shaped values |
| **X9** | “Leaf prompts use persona bodies” | `AGENT-ARCHITECTURE-TRUTH.md:22` and its claims register: VERIFIED | `PLANNER_SYSTEM` asks `role:"worker"`; no `worker` persona in `personas.json`; `system_prompt_for` falls back to `TASK_SYSTEM` | **Default-path false** | Scripted two-turn worker task: assert the system message on both turns is `TASK_SYSTEM`, not a persona body. Then repeat with `role:"content-writer"` as control |
| **X10** | Persona caps/allowlist apply to UI spawns | `12` C23/`02` T? implies they do via persona | `.get(&task.role)` exact string (`execution.rs:550-557,871-879`); UI sends `SuperAgent`/`WorkerAgent` | **Fail-open gap** | Spawn a child from a UI-spawned `SuperAgent` with `max_children` exceeded; assert whether `E_FANOUT_LIMIT` fires. Expected: no — the persona is not found. This is a capability-authority defect, not a prompt defect |
| **X11** | Persona capabilities are validated | `12` decision 7: fail closed; operator: validated/audited | `07` C46/`18` #3: `allowed_tool_ids` not resolved by `Persona::validate`; unknown ids silently do nothing | **Fail-open** | Import a profile with `allowed_tool_ids:["nope"]`; assert import succeeds and the tool is simply unavailable, with no refusal/audit. Then assert `E_TOOL_NOT_ALLOWED` is indistinguishable from a typo |
| **X12** | `/diagnose` uptime | `/health` 88 s / 89 s | `/diagnose` 0 / 0 (two runs) | **Truth defect** | `health.rs:315` builds `ServiceState::new` per call; assert `uptime_secs` grows by ≥1 s between two calls 1 s apart. It does not |
| **X13** | `/api/session` auth | `06` T12/T13: token gate missing | 200 with token to any loopback GET (21:53); Origin is checked (403 on `https://evil`), Host is now checked, token is not | **Partial** | `curl` loopback without Origin → 200 + token. Assert no header/credential is required. Then assert a mutation needs no token (X14) |
| **X14** | Every refusal is a contract code | `caps.rs` header: “always the named contract code” | `http.rs:823` returns `E_PROFILE_NOT_FOUND`, absent from `Refusal`, the 65-code census and `KNOWN_REFUSAL_CODES` | **Contract violation** | `DELETE /api/agents/profiles/nope` → body code `E_PROFILE_NOT_FOUND`; assert `Refusal::from_code` is `None` and the web decoder has no string. Settlement is the two greps plus the HTTP body |
| **X15** | The app is live and usable | brief: live app at 8099 | down 21:57–22:00; rebuilt 22:00:18; no run since 03:26Z | **Environment** | `Get-NetTCPConnection -LocalPort 8099` + `/health`. Record the up/down interval in every report that claims live UI |
| **X16** | 12 turns are deployed | `10` D7 “uncommitted, untested” | 22:00 build after 21:34 edit; Host guard in the same build proves the build is post-21:52 | **Build identity** | No settlement is possible from the binary today. Add a build fingerprint endpoint (§6 Q2); until then mark INFERRED. This is the single largest evidence hole in wave 3 |
| **X17** | 402 is an auth error | `01` §10.2 / `06` T25 / `17` A2 | tree at 22:01: 402 message says “insufficient balance”, code still `E_PROVIDER_AUTH`; no provider test file is dirty, so the old assertion was not updated | **Partial fix** | Trigger a real 402 (expensive) or unit-test `classify_http(402)`; assert code ≠ message intent and that no `E_PROVIDER_BALANCE` exists (0 grep) |
| **X18** | Daemon entity | `12`: immutable one-job obligation, short turns, generation | `14`: control plane with a tier/card/state machine; `18`: record vs task, graph node not run task | **Three designs, no owner** | Not a runtime test: a merge table naming one normative model and the fields each lane must drop. Until then no schema can be written |
| **X19** | Daemon persistence | operator “persist everything” | `18` C7: 1.30 M rows/month; proposes rollups | **Requirement vs arithmetic** | A migration/retention test that samples ticks and asserts the operator-visible report count is exact while tick rows are compacted. Needs the daemon spec first (X18) |
| **X20** | Memory scope set / one v11 | `04`: five banks, no global; “v11” | `13`: six scopes incl. global/project; “one v11, not two” | **Schema merge conflict** | A union DDL review: list every scope id each doc requires. If `global` is a bank, `04`’s read-set theorem changes. Settle with the operator (Q19) then one DDL test |
| **X21** | HyperAgent raw memory access | `04`: summaries only, never raw | `13`: raw is an explicit audited operator setting; operator allows raw “only if leakage/noise is prevented” | **Policy conflict** | Prompt/read-set test: assert raw entries are absent by default and present with the setting, and an audit row exists. Needs the setting schema (Q20) |
| **X22** | A2A visibility authority | operator `:27`: retention/visibility/blindness are **human-only** | operator `:52`: all A2A must support mute/blind/invisible; `14` §8 designs them as settings; `12` designs daemon non-escalation | **Internal requirement tension** | A settings-authority table + test: can a SuperAgent mute its own watchdog? `14` recommends no; the operator says human-only. Assert server-side enforcement, not UI-only |
| **X23** | Card fact set | operator: role/task/state/cost/tools/branch | `05`: ≤4 facts; `12` #9: ≤4 facts; `15` §2.3: a two-line + strip compromise (OPINION) | **Product decision** | A card contract test: every kind renders the agreed fields or honest `—`. Cost cannot be rendered today (X24) |
| **X24** | Cost is a card fact, but cost is not persisted | `05`/`15` design cost meters and a card cost fact | `run_usage` returns tokens only; `06` T23; `16` §6 | **Data availability** | `GET /api/usage/runs/{id}` → assert no cost field. A card that shows `$0.00` is a fabricated number; a card that shows `—` is honest |
| **X25** | “All agents get a card” | operator `:31` | `NODE_KINDS` has no subagent/daemon/branch/cluster; HyperAgent is filtered from the canvas; error is client-only; tool kind rarely produced | **Census gap** | A DOM/registry census test: for each task/persona kind, assert exactly one card or a named “not reported” class. Today the census fails for ≥3 kinds |
| **X26** | “Now” exists as a product surface | brief and operator `:81-82` (hierarchy + live execution + memory/A2A edges) | 0 `Now` strings; no live strip/destination; `05` deliberately splits inspectors | **Undefined requirement** | Define the surface and its data contract (§6 Q17), then a DOM test with an empty state, a live state and a reconnect state |
| **X27** | Autonomy modes 1–4 | operator `:19-21`, default mode 1 | no mode enum, no route, no settings, no enforcement point; `12` proposes a permission matrix but does not own mode numbers | **Unbuilt requirement** | A mode × capability test matrix, server-enforced. Until the mode owner is named, every approval/spawn/budget test is unanchored |
| **X28** | Budget default | operator: no hard default; soft guard acceptable | live `/diagnose`: `$0.50`, 400K, 40; `16` designs Mode A (no money cap); `17` Q17 recommends keeping caps during the security wave | **Three postures** | A policy test on a sandbox account: with Mode A, assert no money refusal under a fixed cap; with Mode C, assert preflight refuses before HTTP. Requires Q18 |
| **X29** | A2A mesh/topics | operator: mesh allowed by settings; bounded notice bus with optional topics | `02` X1/X7 rejects an agent bus; `14` designs a bounded notice log and bridge approval; `09` target says “roster/topics by policy” | **Design conflict** | A settings matrix + loop/no-self-delivery tests. No implementation exists, so this is a merge decision (Q16-adjacent) |
| **X30** | “36 answers” | task brief and `OPERATOR-DECISIONS:3` say 36 | the file contains 45 bullets and no ids (`10` §5.0) | **Process fact** | Count the bullets; assign stable ids. Any mapping table that says “answer 17” is unverifiable until ids exist |
| **X31** | Does the deployed binary reject a hostile `Host`? | `22` §1.2: live `GET /health Host: evil.example → HTTP 200`; “the live binary has no Host guard” | `24` §4.2: “live 421 proof”; this seat: **421** at 22:02:00 and 22:02:01, control **200** twice | **Wave-3 live-fact conflict** | Re-probe with the build hash recorded in the same request; `22` must name the uptime/probe time and the `deployed_build` hash. If 421, `22` is stale; if 200, the binary changed again. Either way the missing field is the defect |

### 4.2 The eight that block wave 3 (detail)

**X1 — the 12-turn cap is a run budget, not a task budget.** This is the highest-value finding
of the audit because it invalidates the one fix that was supposed to unblock normal research.
The constant’s comment, the counter’s key and the test’s assertion disagree:
`MAX_TOOL_TURNS` says “in one task”; the engine passes `run_tool_counters(run.id)`; the test
asserts `(MAX_TOOL_TURNS, MAX_TOOL_TURNS)` for a **single-task** run, so it cannot distinguish
the two readings. The live runs that motivated the change (3 failures at exactly 4 turns) were
single-prompt runs; raising 4→12 may let one more task through, but under an 8-node plan the
run-level budget is still ~1.5 turns/task. Settlement is a two-task scripted plan (no paid
call). If it fails as predicted, the 4→12 change is a mitigation, not a fix, and the claim
“research tasks now complete” must be withdrawn.

**X2/X3 — the count-lie replaced the money-lie.** The working tree fixed the false
`E_BUDGET_EXCEEDED` (good) by routing the tool cap to `E_CALL_LIMIT`. But `E_CALL_LIMIT`’s
frozen message names “the 40-call cap or the 40-super-step plan ceiling”. The tool-turn cap is
12; approval waits are 8; the A2A message budget is 32; the tool-call cap is 40 but is a
different counter from the provider’s 40. Four of the five conditions now produce a message
that misstates the limit, and one (`caps::MAX_CALLS`) is unreachable. A UI that maps the code
to a spend meter will still be wrong — exactly the failure `10` X10 identified, one layer
deeper. The settlement is a table test over five forced conditions; it is cheap and should be
a merge gate.

**X6 — the root is the product’s own source tree, and the documented override is ignored.**
The launcher’s `ResolveWorkspaceRoot` reads `HIVEMIND_WORKSPACE_ROOT`, then a hard-coded
`C:\Users\Riley\Desktop\hiverouter-v1`, then cwd; the server’s own config reads
`HIVEROUTER_WORKSPACE_ROOT`. An operator who sets the server’s documented variable and
launches through `HiveMind.exe` gets the hard-coded dev path. On another machine the
`Directory.Exists(DefaultRoot)` check fails and the fallback is `Directory.GetCurrentDirectory()`
— which is **not** `AppDomain.BaseDirectory` for a shell-launched GUI app, so a portable copy
can jail to an unexpected working directory. The live consequence today: file tools can list
and read `.git`, source and research docs, and (with one approval) write them. The settlement
is the env-var probe in X6 plus a `file_list` of the root; both are read-only.

**X7 — the corpus’s biggest drift example.** Three reports state “no Host allowlist” as a
security fact. It was true at HEAD. Another lane added the guard at 21:52:53, the server was
rebuilt at 22:00:18, and the live probe returned 421 twice. The guard itself has a status/code
mismatch — `421 Misdirected Request` with `E_ORIGIN_REJECTED` — and it is Host-only: the
mutations still require no token and no Origin. So the correct current statement is narrower
and more useful than either side: *rebinding is mitigated in the deployed build; same-machine
loopback CSRF is not.* The settlement is the two Host probes plus the three handler reads.

**X8 — approval evidence is honest but not durable.** `args_preview` is a real contract field,
properly redacted and bounded. The UI now renders it. But the persisted approval row stores
only the digest, so the projection used on replay (`approval_request_event`) hard-sets `None`;
the comment says this is deliberate (“must not be reconstructed from a lossy source”). That is
correct behaviour for a digest-only store, but it means a reconnecting operator sees the old
blind overlay for every pre-existing approval. The overlay still lacks Dismiss for stale
entries and a focus trap. Settlement: approve-request → disconnect → reconnect; assert preview
absent; then assert the overlay cannot be dismissed without a decision.

**X13/X14 — the local boundary is half-closed and one refusal is invented.** Host is checked;
Origin is checked on `/api/session` and the provider proxies; the session token is not required
by any mutation; and `DELETE /profiles/{id}` returns a code that does not exist in the
contract. The first two are documented wave-1 findings; the third is new and is a frozen-
contract violation. A UI that renders `E_PROFILE_NOT_FOUND` as “unknown error” is the visible
symptom; the actual defect is that `Refusal::from_code` cannot round-trip it.

**X17 — the 402 fix is half-done and unbuilt.** The tree’s message now tells the truth
(“insufficient balance; top up or replace the key”) but the code remains `E_PROVIDER_AUTH`,
whose remediation is “check DEEPSEEK_API_KEY”. So the operator sees two contradictory
instructions in one refusal. `16`/`17` both prescribe a distinct `E_PROVIDER_BALANCE`; the
implementation chose message-only. Because the edit is at 22:01:00, it is **not** in the 22:00
build. Settlement: unit-test the classifier; assert code and message agree about remedy.

**X18/X19/X20/X21 — the merge conflicts wave 3 must resolve before schema work.** Three
daemon models, two memory topologies, two HyperAgent raw-access policies, and one operator
requirement that “everything” is persisted against arithmetic that says 1.3 M rows/month.
None of these is a runtime bug; all of them block a migration number. The settlement is not a
test — it is a normative-owner decision (Q14/Q15/Q19/Q20) followed by one DDL and one
retention test. Wave 3 must not let a fourth lane add a fourth model.

---

## 5. Gaps against the operator’s answers

### 5.1 The artifact itself: “36” is not a stable key

`OPERATOR-DECISIONS.md:3` says “the operator's answers to the 36 planning questions”. The file
contains **45 bullets** (counted at 21:52) and no numbering or ids. Consequences:

- Any sentence of the form “operator answer 17” is unverifiable. `10` §5.0 flagged this; no
  wave-2 lane fixed it.
- The six brief-named areas are scattered: settings/modes at bullets 5–9; cards at 10/33;
  daemons at 11; budget/caching at 8 + 41; Venn memory at 24–29; UI/Now at 31–34.
- **Gap G-00:** assign stable ids `OP-01…OP-45` before wave 3 maps anything to them. This is a
  one-file edit and it unblocks every coverage table. It is also the reason `10`’s “45-row
  matrix” and this section cannot be mechanically joined.

`10` §5.1 remains the only full 45-row census. This section re-marks the six areas the brief
names, plus the ones where wave 2 changed the answer, and does not repeat the other rows.

### 5.2 Settings and autonomy modes (OP 5–9) — **the largest gap**

| Operator ask | Current state (frozen) | Wave-2 outcome | Verdict |
|---|---|---|---|
| Autonomy modes **1–4**, default mode 1 | No mode enum, no route, no settings table, no enforcement point. `/diagnose` shows only `side_effects_enabled: true` | `09` names autonomy; `10` D3 lists four mode interpretations; `12` designs a permission matrix but never numbers modes 1–4; `15` §5.7 designs the screen | **UNBUILT, UNDEFINED** |
| Approval policy: all options settings; default side effects only; no silent bypass | `spawn_agent` executes with no approval; same-parent sibling messages skip Mutate approval; write/shell go through approval | `02`/`06`/`07` document the bypass; `12` proposes per-call approvals; no policy setting | **PARTIAL, VIOLATED** |
| Agent capabilities editable, validated/audited, default enabled | Import/delete API + UI; `allowed_tool_ids` unvalidated; no audit of a change; no edit route | `07`/`08`/`18` agree it is unvalidated; `12` proposes capability grants | **PARTIAL, FAIL-OPEN** |
| Retention/visibility/blindness human-only, audited | No retention setting, no blindness setting, no audit UI; `audit_log_dir: null` in live `/diagnose` | `04` designs retention; `13` designs retention/journal; neither is built; `14` designs mute/blind | **MISSING** |

**What wave 3 must not assume:** that a “mode” is a UI-only filter. The only current
enforcement point is `side_effects_enabled` plus the approval gate in `execute_tool_plan`. If a
mode is not enforced server-side at that gate, it is a label.

**Settings surface reality (source + live):** `SettingsPanel.tsx` exposes one reduced-motion
checkbox; `http.rs` has no settings route (one comment only); all provider values are boot
env/CLI; `/diagnose` exposes effective values read-only. This is `08` #2, verified again.

### 5.3 All-agent cards (OP 10, 33) — **census fails for at least three kinds**

| Kind | Card today | Evidence |
|---|---|---|
| HyperAgent | Chat card exists but is filtered out of the canvas; the chat overlay is its surface | `graph.ts:151`; live `NODE_KINDS` |
| SuperAgent | Card exists; rendered only when the planner emits `superagent` (prompt asks for it, validator allows any role) | `NODE_KINDS`; `PLANNER_SYSTEM` |
| Worker | Card exists; default role `worker` has no persona, so identity is generic | `personas.json`; L97 |
| **Subagent** | **No kind, no card.** Role + persona only; tier maps to Worker | live census 4 tiers; `NODE_KINDS` |
| **Remote / Watchdog** | **No kind, no card, no code** | 0 grep matches; live census |
| **ToolAgent** | Kind exists; produced only for a tool-tier task, which the planner never asks for | `NODE_KINDS`; `PLANNER_SYSTEM` |
| Error | **Client-only** broken-route state; never a server kind | `15` §2.1 |
| Branch / Cluster / Hive | No card classes; Hive is a SuperAgent container by convention only | `15` §3.1 (design) |

**Gap G-10:** there is no census test. `12` #9 (“every agent gets a card as a roster invariant”)
is a principle; `15` §3.3 is a card registry design; neither produces one row per live task
kind. The operator’s sentence (“every agent, including tiny Watchdog/Remote, must have a card”)
cannot be accepted without such a test.

**Gap G-33a:** the default fact set is still contested (`05`/`12` say ≤4; operator says six;
`15` proposes a compromise), and two of the six operator facts have **no producer at all**:
`cost` (not persisted) and `tools` (not on `NODE_*`). A card contract cannot be written until
Q13/Q23 in §4 are settled.

### 5.4 Remote / Watchdog daemons (OP 11) — **no runtime model, three designs**

| Aspect | Operator | Corpus | Code |
|---|---|---|---|
| Existence | “deliberately simple switches/daemons: one job forever, report to owner” | `09` target; `12` one-job obligation; `14` control plane; `18` record-vs-task | 0 identifiers in `*.rs`; 0 routes; no table |
| Card | every agent gets one | `12` daemon card; `14` remote card; `18` graph node card | none |
| Schedule | example “every 5 minutes” | `12` 30 s heartbeat / 300 s report; `14` interval/state machine; `18` Healthchecks-style period+grace | none |
| Ownership | under HyperAgent or a SuperAgent | `18` owner_hyper vs owner_task cascade; `12` exclusive office; `14` owner | none |
| Persistence | “persist everything” | `18` C7 arithmetic → rollups; `12` generation; `14` records | none |
| Kill / disable | implied by “switch” | `12` kill terminal Retired; `14` kill; `18` disable | none |

**Gap G-11:** wave 2 produced **three incompatible daemon models and no normative owner**. The
`12` model cannot coexist with `14`’s control plane without deciding whether a daemon is a
task, a record, or a service; `18` explicitly leaves the entity question open. Any wave-3
schema lane that starts now will pick one by accident. This is Q14/Q15 (§6).

**Negative evidence:** `backend/**/*.rs` contains no `watchdog`, `daemon`, or `remote_agent`
match (0). The “file watcher” in `tools/` is a filesystem watcher, not this tier.

### 5.5 No hard default budget + caching (OP 8, 41) — **live policy contradicts the operator**

| Fact | Evidence | Status |
|---|---|---|
| Live hard cap: **$0.50/run**, **400,000 combined tokens**, **40 calls**, 64 agents | `/diagnose` at 21:53 and 22:01; `budget.rs:24-43` | VERIFIED |
| Live budget failures exist: 3 of 5 runs `E_BUDGET_EXCEEDED` at ~03:23–03:25Z | `/api/history/runs` | VERIFIED |
| No global/hive/day guard, no kill switch, no soft guard | 0 matches for `E_HIVE_BUDGET`, `E_SPEND_GUARD`, `E_SPEND_HALTED`; no budget route | VERIFIED (absence) |
| Cost not persisted (tokens only) | `/api/usage/runs/{id}` schema; L45 | VERIFIED |
| Cache counters wired, per-run only; one observed 91.4% hit sample at 03:23Z (from `10`, not re-measured) | `/api/usage` route; `10` C11 | READ/VERIFIED split |
| No prefix contract, no per-agent hit/miss persistence, no cross-agent reuse implementation | source absence; `13`/`16` designs | VERIFIED (absence) |

**The three-way contradiction:** operator says no hard default; the code says $0.50/400K/40;
`16` designs Mode A (no money cap) as default; `17` Q17 recommends **keeping** the current
caps through the security wave. All three are defensible; none is authoritative. And the
current hard cap is the thing that produced the only three live failures in the database —
which `10` correctly separated as a tool-cap false positive, not a money decision.

**Gap G-8/G-41:** there is no implemented *spend* posture to test. A soft guard needs a cost
number; cost is not persisted; the token counters are per run; there is no daily/global key.
So “soft guard + emergency stop” is not a small setting — it requires Q17/Q18 (effective
date + metering source) before any code.

**Cache truth:** the operator makes caching first-class and forbids unmeasured superiority
claims. The corpus honours the prohibition (no “better than DeepSeek” claim was found). But
there is **no persisted per-agent/per-prefix measurement**; one live sample exists (91.4% hit,
2,100 prompt tokens) and it is not evidence of a reusable prefix. `13`/`16` both propose
schemas for this; both are unbuilt.

### 5.6 Venn memory (OP 24–29) — **two topologies, both trees, both claim v11**

| Operator scope | `04` (five banks) | `13` (six scopes) | Code |
|---|---|---|---|
| Global | **absent** | present (universal curated core) | absent |
| Project | present | present | `projects` table (name only) |
| Run | present | present | no bank; run rows exist |
| Hive | present | present | no hive key |
| Branch | present | present | no branch key |
| Per-role | present (`agent_role`) | present (overlay, read-only) | `agent_memory(agent_key)` — the only real scope |
| Multi-membership / Venn | one-level promotion only | read-up + time-bound grants + promotion | none |
| HyperAgent raw access | **never raw** | raw as explicit audited setting | no aggregate read |
| One v11 | v11 | “one v11, not two” | schema 10 |

**Gap G-24:** the operator asked for a *connected Venn cluster*; both designs answer with a
tree plus limited cross-links, and they disagree on the scope set and on HyperAgent raw
access. `13` is the later and broader design, but `04` is the one whose leakage theorem is
built on a five-bank read set. Merging them is not a copy-paste: adding `global` changes the
read-set proof; allowing raw access changes the poisoning blast radius. This is Q19/Q20, and
it is a blocker for any v11 DDL.

**Current live truth:** 7 role banks, all empty; the prompt header calls all scoped facts
“authoritative” (READ); memory reads swallow errors (`if let Ok`); retention is unbounded
because there is no retention code. The operator’s “prompt-injected content must not become
authoritative memory” is violated by the header, not merely unimplemented.

### 5.7 UI “now” (OP 31–34) — **the surface does not exist**

| Operator statement | Current | Gap |
|---|---|---|
| Main view = hierarchy + live execution + memory/A2A edges | Canvas + chat + read-only Swarm counters; no memory/A2A edges; no live strip | **G-31** |
| All projections (canvas, chat, graph, dashboard) are settings, all enabled by default | Canvas/chat exist; Tree is a read-only indented list; “dashboard” is stat-tile/hero slop recommended for removal; no projection settings | **G-32** |
| A2A UI = dedicated topology/memory inspector by default; drawer/canvas/chat are settings | Read-only mailbox; no inspector; no settings | **G-31a** |
| Agent cards: role, task, state, cost, tools, branch | Kind + state only; cost/tools/branch absent from the wire | **G-33** |
| Beautiful, FigCraft-derived | FigCraft parity pinned for main composition; chat 400×480 vs board 420×500 (READ); no option-4 research record | **G-34** |

**“Now” is undefined.** The word appears in the brief, not in `OPERATOR-DECISIONS.md`. `10`
Q18 asked; `15` proposed a “Now strip”; neither is normative. There is no `Now` string in
`web/src` (0 matches) and no route. The nearest requirement is OP 32 (main view). A wave-3 UI
lane that builds a “Now” without Q17 is building an opinion, not a requirement.

**UI-now, precisely:** at 21:52 the deployed bundle contained the `args_preview` row and the
persona/role fixes; at 22:00 it was rebuilt with the Host guard; the visible app still has no
settings, no Now, no Subagent/Remote card, no memory/A2A edges, no approval inbox. The app is
“live” in the sense that it serves and has 5 persisted runs; it is not “live” in the sense of
showing a multi-agent hierarchy that the runtime can produce.

### 5.8 Gaps the brief did not name but that block the same work

| Gap | Operator bullet | Why it blocks |
|---|---|---|
| **G-00 stable ids** | artifact itself | every mapping table is unverifiable without them |
| **G-15 ownership models** | OP 15 | SuperAgent ownership is switchable in the requirement; there is no ownership record, no per-hive owner, no UI. Blocks hive memory and daemon owner |
| **G-18 HiveWide collision/usage daemon** | OP 18 | no duplicate-tool detector, no CPU/RAM/load monitor, no sandbox beyond the path jail. Blocks safe fan-out and “computer/browser use” |
| **G-19/G-22 A2A visibility** | OP 19/22 | no mute/blind/invisible, no notice bus, no topics; `02` rejects a bus. Blocks the A2A UI and the settings matrix |
| **G-43 retry/escalation** | OP 43 | provider retries 3 attempts; no branch-failure decision, no HyperAgent decide/escalate, no operator escalation control. Blocks autonomous mode 3/4 |
| **G-44 persistence** | OP 44 | cost, artifacts, approvals ledger incomplete; no ephemeral setting. Blocks “everything persisted” acceptance |
| **G-45 hierarchical planning** | OP 45 | planner schema has no `parent`; a flat plan cannot express the four-level hierarchy. Blocks 300–600 and the SuperAgent tier in practice |

---

## 6. Twenty new hard questions

These are new relative to `10` Q1–Q20, `17` §8, `15` §8, `12` §15, `14` §16 and `18` §12.
Novelty is by **decision**, not by wording: each question asks for an authority, a scope, or an
evidence rule that none of the prior lists owns. Where a prior question is adjacent, that is
named so the operator is not asked twice. Options are 3–5; “operator” marks a product decision,
“engineering” a decision the Lead can settle with evidence.

---

### Q1 — Which artifact is the wave-3 implementation baseline? (engineering + operator)

The four artifacts disagree: HEAD `bd575ba`; the dirty tree at freeze (7 source files); the
22:00 deployed build; and 19 documents written against earlier states.

- [ ] **HEAD commit only** — wave 3 starts from a clean checkout; all dirty changes are re-landed as reviewed commits.
- [ ] **The frozen dirty tree** — hash-pinned in the report header; wave 3 may build on it but must commit it first.
- [ ] **The deployed build** — whatever runs at 8099 is the product; source is documentation until it matches.
- [ ] **Freeze now, forbid edits** — no lane may touch product source until the wave-3 evidence gate passes.

**Why it blocks:** every other question’s answer is relative to a state. Today a lane can prove
“no Host allowlist” (HEAD) and “Host is checked” (deployed) and both be right. Until the
baseline is named, no claim is falsifiable and no diff is reviewable. This is the root of
F-01 and X16.

---

### Q2 — What must `/health` expose so a report can prove which bytes ran? (engineering)

- [ ] **Git commit + dirty flag + binary SHA-256 + build timestamp** (four fields).
- [ ] **Binary SHA-256 + build timestamp only** (no repo identity).
- [ ] **`git describe --dirty` only** (human-readable, weaker).
- [ ] **Nothing** — reports continue to infer from file mtimes and build order.

**Why it blocks:** every runtime claim (`421`, 12/40, `E_CALL_LIMIT`) is currently tied to a
binary by inference. A build fingerprint turns inference into observation and makes the wave-3
gate mechanically checkable. Without it, “deployed” is an opinion and a stale claim cannot be
expired when a new build lands.

---

### Q3 — Does every refusal cause get its own code, or may codes be reused with a detail field? (engineering, contract-owner)

- [ ] **1:1 codes** — add `E_TOOL_TURN_LIMIT`, `E_TOOL_CALL_LIMIT`, `E_APPROVAL_WAIT_LIMIT`, `E_MESSAGE_LIMIT`; retirement of a code requires a ledger row.
- [ ] **Reuse `E_CALL_LIMIT` with a structured `detail.limit` field** and one honest message (“a call/turn/approval budget was exhausted”).
- [ ] **Reuse with a message only** (current direction) — no schema change.
- [ ] **Collapse into one `E_BUDGET_EXCEEDED`** with a `kind` field.

**Why it blocks:** the frozen contract promises “a refusal is always the named contract code”
(`caps.rs` header). Today `E_CALL_LIMIT` covers five causes and its message names one. Every
UI meter, test and operator diagnosis depends on this choice; `17`/`16` already proposed four
new codes and the tree added none.

---

### Q4 — Is the tool-turn budget per task, per branch, or per run? (engineering)

- [ ] **Per task** — each task gets its own 12 turns; run-level protection is a separate call cap.
- [ ] **Per run** — one 12-turn pool for all tasks (current behaviour, despite the comment).
- [ ] **Per branch** — a hive/branch pool, so one branch cannot starve another.
- [ ] **Adaptive** — planner assigns a turn allowance per node at plan time.

**Why it blocks:** the 4→12 change was justified by three single-prompt failures. Under an
8-node plan a per-run 12 gives ~1.5 turns/task. The choice changes the scheduler, the prompt,
the refusal, and the test that must accompany the change (X1/X4).

---

### Q5 — Which “40” is the authoritative call ceiling, and what happens at 41? (engineering)

- [ ] **Provider `CALL_CAP=40` is the only real cap**; tool counts are evidence, not a second cap.
- [ ] **Tool `MAX_CALLS_PER_RUN=40` is authoritative**; provider calls include retries and may exceed it once.
- [ ] **Keep both** and require the refusal to name which one fired (needs Q3).
- [ ] **Delete `caps::MAX_CALLS`** (unreachable) and let the two real counters speak.

**Why it blocks:** `caps::check_calls` is a stateless compare (`check_calls(1)`), so it never
fires; the other two are different counters with the same number. A run that hits 41 tool calls
and 39 provider calls is in a state no document describes. Scale claims and cost arithmetic
depend on which one binds (X3).

---

### Q6 — Complete the local boundary: Host only, Host + Origin, Host + Origin + token, or token only? (engineering + security)

- [ ] **Host + Origin + session token on every mutation**; token only in an `Authorization` header (not the WS query string).
- [ ] **Host + Origin on mutations**, token only for WS (smallest change from today).
- [ ] **Host only** (current deployed state) — accept same-machine loopback CSRF as out of scope.
- [ ] **Token only**, with a per-launch secret file; Host/Origin become defence-in-depth.

**Why it blocks:** the deployed build now blocks rebinding (421) but still lets any loopback
origin POST/DELETE and still issues a token without a credential. `06` T12 and `17` C2b demand
different scopes; the wave-3 security gate cannot be written until one is chosen (X7/X13/X14).

---

### Q7 — Which workspace-root precedence is normative? (operator + engineering)

- [ ] **Launcher flag > server env > project setting > cwd**; the portable fallback is the package directory, never cwd.
- [ ] **Server env only** (`HIVEROUTER_WORKSPACE_ROOT`); the launcher must not invent a root.
- [ ] **Per-project root set at project creation**, visible in the run header; no global default.
- [ ] **Fixed app data directory** (never the operator’s repo).

**Why it blocks:** the live server jails to the product’s own checkout; the documented env var
is ignored by the launcher; a portable copy falls back to cwd. Every file result, every
“repository” prompt, and the security story for `.git` depend on this. `10` D8 asked *how*; this
asks *which source wins* and what the portable fallback is (X6).

---

### Q8 — How long must approval evidence survive? (engineering + operator)

- [ ] **Persist the redacted preview** (≤512 chars) with the approval row so replay shows what was approved.
- [ ] **Digest only** (current) — replay shows `—`; the decision is bound by hash.
- [ ] **Persist a derived target/path summary** (tool-specific, e.g. `file_write → path`) instead of raw args.
- [ ] **Keep live args in memory only**; a reconnect invalidates the approval and requires a new request.

**Why it blocks:** the operator’s approval decision is the product’s consent moment. A preview
that vanishes on reconnect makes the audit trail weaker than the decision it records. This
also decides whether the overlay can ever show “what will happen” rather than a hash (X8).

---

### Q9 — What is the authority for persona lookup, and does a miss fail open or closed? (engineering)

- [ ] **Exact persona id only; a miss falls back to the tier default prompt** (current prompt behaviour) but capability lookup fails closed (no tools beyond tier).
- [ ] **Normalise everywhere** (executor *and* `seed_task_context`/`persona_spawn_limits`); one canonical key.
- [ ] **Require a persona id on every spawn**; refuse unknown ids with `E_PERSONA_UNKNOWN`.
- [ ] **Persona is prompt-only**; capabilities always come from the tier, never the persona.

**Why it blocks:** the executor normalises, the engine does not. A UI `SuperAgent` therefore
gets the right prompt and the wrong (fail-open) capability set. Fail-open vs fail-closed is a
security decision; `12` chose fail-closed, the code has not (X10/X11).

---

### Q10 — Where are autonomy modes 1–4 enforced? (operator + engineering)

- [ ] **Server-side at the approval/spawn gate**; the UI only displays the mode.
- [ ] **Server-side capability grant computed at spawn**, attenuated per mode; the mode is a grant input.
- [ ] **UI-side policy** (the client decides what to ask); server remains permissive.
- [ ] **Hybrid**: server enforces hard limits; mode only controls prompting/UX.

**Why it blocks:** “default mode 1” is a label until there is an enforcement point. `12`
proposes a matrix, `15` a settings screen, `10` D3 four interpretations — none names the
enforcement locus. Every approval, spawn, budget and A2A test is unanchored until this is
answered (X27).

---

### Q11 — What happens to an imported profile with an invalid or unknown tool id? (engineering + operator)

- [ ] **Refuse at import** with `E_TEMPLATE_INVALID`/`E_PERSONA_UNKNOWN`; nothing is stored.
- [ ] **Store, warn, and resolve at spawn**; unknown tools are dropped with an audit row.
- [ ] **Store and ignore silently** (current) — a typo disables a tool with no signal.
- [ ] **Quarantine**: store disabled until an operator approves the capability set.

**Why it blocks:** the operator wants capabilities editable, validated and audited. Today
`Persona::validate` does not resolve `allowed_tool_ids`; an imported profile can silently lose
a tool, and there is no audit of the change. This is the smallest exam of the capability
authority that every later grant model will inherit (X11).

---

### Q12 — What is the card census, exactly? (operator)

- [ ] **Every runtime task kind** (hyper/super/worker/tool/error) + Subagent + Remote daemon; no cards for branch/cluster.
- [ ] **Every task kind + every daemon + project/hive containers** (a card for every aggregate).
- [ ] **Only agent instances**; branches/clusters are canvas aggregates, not cards.
- [ ] **Per-tier configurable**; the census test enumerates the enabled set.

**Why it blocks:** the operator’s “every agent gets a card” cannot become a test until the
kind list is closed. `15` §3.3 has a registry design; `12` has a principle; neither has a
census that fails for the three missing kinds today (X25).

---

### Q13 — What does a card show when cost does not exist? (operator + engineering)

- [ ] **Tokens only**, labelled “cost not settled”; `—` for unknown; no dollar sign.
- [ ] **Estimated cost** from the price table, labelled `estimated` (never silently settled).
- [ ] **Block the cost fact** until the cost ledger ships; the card shows five facts, not six.
- [ ] **Show `$0.00`** when no usage exists (rejected: this is fabricated data).

**Why it blocks:** two of the six operator-required facts (`cost`, `tools`) have no producer.
A card contract that includes them will either fabricate or show `—` permanently; `05`/`15`
both design meters that assume a cost source that does not exist (X23/X24).

---

### Q14 — What *is* a Remote/Watchdog daemon: task, record, service, or process? (operator)

- [ ] **A durable record** with a schedule/lease, no task row; its tick creates short executions.
- [ ] **A long-lived task** under its owner, pinned across runs.
- [ ] **A service** with its own supervisor and no task identity (closest to systemd/Healthchecks).
- [ ] **A recurring task template** instantiated per tick, no long-lived identity.

**Why it blocks:** `12`, `14` and `18` each assume a different answer. The entity choice
determines the schema, migration number, catch-up semantics, kill path, card fields and
whether “one job forever” survives a restart. No daemon code can be written before this is
settled (X18).

---

### Q15 — Who owns a daemon, and what happens when the owner is deleted or restarted? (operator + engineering)

- [ ] **Owner may be HyperAgent (durable) or a task (cascades)**; delete disables and reports once (18’s proposal).
- [ ] **Operator-owned only**; no agent can create/delete a daemon.
- [ ] **SuperAgent-owned**, exclusive office (12’s model); rebind requires quiescence.
- [ ] **Owner is advisory**; the daemon survives any owner change with an audit row.

**Why it blocks:** orphaned daemons are the failure mode the operator’s “simple switch”
requirement is least prepared for. Ownership decides cascade, authorization, budget
attribution and the kill path — and `14`’s “no self-control” rule conflicts with agent-owned
ownership unless this is answered (X18/X22).

---

### Q16 — May an agent mute/blind itself or its children, or are visibility settings human-only? (operator)

- [ ] **Human-only**, server-enforced; agents can request, never set.
- [ ] **Agents may mute/blind their own descendants**, within their grant; the operator can always override.
- [ ] **SuperAgent may mute its hive but not its own supervision** (watchdog exempt).
- [ ] **Any agent may set its own visibility**; audit only.

**Why it blocks:** operator line 27 says visibility/blindness are human-only; line 52 says all
A2A must support mute/blind/invisible; `14` designs precedence and `12` designs daemon
non-escalation. A settings schema cannot be authored until “who may set it” is settled, and a
compromised agent silencing its watchdog is a real authority hazard (X22).

---

### Q17 — What is “Now”, and which event stream proves it? (operator + engineering)

- [ ] **A live strip on the default view** (hierarchy + current executions + memory/A2A edges), fed by the existing WS events, no new route.
- [ ] **A separate destination** (Run detail “Now”) with its own subscription and filters.
- [ ] **A projection of the run snapshot + deltas** (replayable, deterministic) rather than a live-only view.
- [ ] **The default main view itself**; canvas/tree become toggles.

**Why it blocks:** the word is not in the operator file, there is no `Now` string in `web/src`,
and `05` deliberately splits inspectors. A wave-3 UI lane needs a definition and a data source;
without it, “Now” will be another stat tile (X26).

---

### Q18 — What is the default spend posture, and from which date is it effective? (operator)

- [ ] **Mode A (no money cap) immediately**, with structural caps + balance floor + 402 freeze + kill switch.
- [ ] **Keep $0.50/400K/40 through the security wave**, then migrate to Mode A after the gate (17’s recommendation).
- [ ] **Mode B (soft guard) immediately** with an operator-set threshold; hard caps opt-in.
- [ ] **Operator-set per-project posture**, defaulting to current caps until changed.

**Why it blocks:** the live system has hard caps the operator did not ask for; `16` designs them
away by default; `17` wants them kept; no document names the effective date. Denial-of-wallet
and the operator’s promise cannot both be tested against an unnamed default (X28).

---

### Q19 — Which memory scope set is normative for v11? (operator + engineering)

- [ ] **The operator’s five**: global, run, hive, branch, role (project as retention metadata, not a bank).
- [ ] **`13`’s six**: global, project, run, hive, branch, role.
- [ ] **`04`’s five**: project, run, hive, branch, agent_role (no global).
- [ ] **A flat store with scope + read filters** (no bank tree).

**Why it blocks:** `04` and `13` both claim v11 and disagree on `global` and on the read-set
proof. The schema, leakage theorem, read API and UI all follow from the scope set. This is not
a naming preference; adding `global` changes who can read what (X20).

---

### Q20 — Is HyperAgent raw memory access off by default, audited-on, or per-bank? (operator)

- [ ] **Off by default; audited raw on explicit request** (13’s position, closest to the operator’s “summaries/audited first”).
- [ ] **Never raw** (04’s position) — summaries and counts only.
- [ ] **Raw to all banks as an explicit operator setting** (operator’s “raw only as an explicit setting”).
- [ ] **Per-bank ACL**: some banks raw, some summary-only.

**Why it blocks:** the operator allows raw “only if leakage/noise is prevented”; `04` forbids
it; `13` permits it as a setting. The choice changes the prompt budget, the audit schema, the
poisoning blast radius and the leakage theorem. A v11 read policy cannot be written without it
(X21).

---

## 7. EVIDENCE-GATE-v2 — acceptance gate for wave 3

### G0. Why v2 exists

`10` §9 (v1) named four failure classes: stale trees, assumed scope, untimestamped runtime,
inherited numbers. Wave 2 did not fail v1 loudly; it failed it **quietly**, in ways v1 has no
rule for:

| Observed in this audit | v1 rule it slipped through |
|---|---|
| A source file (`http.rs`) changed **during** the audit; three reports’ security conclusion flipped | v1 G6 asks for freshness; it does not expire a claim when a build/deploy happens |
| A lane’s claim “`11` does not exist” was false when the report was accepted; `11` existed 5 minutes before `18` was written | v1 has no rule that a negative existence claim must be re-checked at write time |
| The 22:00 build was deployed with no commit/binary identity; reports now say “deployed” on build-order inference | v1 G1 records HEAD and dirty hashes but not a **deployed** identity |
| The live app was down for ~3 minutes mid-audit; no report has an up/down rule | v1 G4 records probes but not availability windows |
| `E_CALL_LIMIT` reused for five causes; a claim register would mark “code exists” VERIFIED and miss the semantics | v1 G3 has claim types but no **one-code-one-cause** rule for refusals |
| Two lanes both claim “one v11” with different scope sets; v1 has no cross-lane merge test | v1 G5 covers contradictions but not **schema ownership** |
| A wave-2 lane’s own claim register says a file does not exist, while another lane wrote it — both accepted | v1 has no inter-report consistency check |

**v2 is therefore not “v1 with more fields”. It adds four things v1 lacks: (1) a deployed-build
identity, (2) claim expiry on commit/build/deploy, (3) negative-claim re-check, and (4) a
cross-lane merge test for any artifact two reports both claim to define.**

### G1. Mandatory header (a missing field rejects the report)

```yaml
gate: EVIDENCE-GATE-v2
lane: <number and name>
lane_id: <stable id>
date_local: <ISO 8601 with offset>
date_utc: <ISO 8601 Z>
workspace: C:\Users\Riley\Desktop\hiverouter-v1
head: <40-hex sha>
head_subject: <first line of the commit>
tree_state: clean | dirty
tree_hashes:                      # every source/test/config file changed vs HEAD
  - path: <repo-relative>
    mtime_local: <ISO 8601>
    sha256_16: <16 hex>
deployed_build:
  binary_sha256: <64 hex or ABSENT>
  build_time_local: <ISO 8601 or ABSENT>
  launch_time_local: <ISO 8601 or ABSENT>
  served_url: <http://127.0.0.1:PORT or ABSENT>
  fingerprint_endpoint: present | ABSENT
live_window:
  up: <ISO 8601>
  down: <ISO 8601 or "still up">
  probes: <count>
freeze_time_local: <ISO 8601>
claims_expire_on: commit | build | deploy | 60-minutes
paid_calls: none | <count + approval ref>
product_edits: none | <paths>
restarts_by_this_seat: none | <count + why>
credentials_read: none
```

Rules:

- `head` + `tree_hashes` + `deployed_build` are **all three** mandatory. A report that cannot
  name the deployed binary must write `binary_sha256: ABSENT` and mark every runtime claim
  **INFERRED**; it may not write “deployed” in prose.
- `freeze_time_local` is when the last evidence was gathered. A claim is valid only for that
  state (G5).
- A live window that contains a down interval must list it; a probe during a down interval is
  `E_UNREACHABLE`, never “no evidence”.

### G2. Claim ledger shape (mandatory; one row per material claim)

```yaml
- id: <lane>-C001
  claim: <one sentence, falsifiable>
  source_cite: <file:line or report:section>
  status: VERIFIED | READ | INFERRED | CONTRADICTED | STALE
  evidence_class: runtime | code | external-doc | arithmetic | design | absence
  observed_at_local: <ISO 8601>          # required for runtime/code
  verified_against: HEAD | tree@<time> | build@<sha256> | live@<time>
  inherited_from: <report id or none>    # READ rows must name the source
  contradicts: <claim id or none>
  settlement_test: <command or assertion; required for CONTRADICTED>
  expires_on: commit | build | deploy | 60-minutes
```

Rules:

- **No bare VERIFIED.** A VERIFIED row must name `observed_at_local` and
  `verified_against`. An inherited row is READ, never VERIFIED (v1 said this; v2 enforces it
  in the schema).
- **No CONTRADICTED without a settlement test.** “Operator decides” is not a settlement test;
  it is `status: CONTRADICTED (unsettled by decision)` plus a question id.
- **No STALE without both states.** The row must name the state in which it was true and the
  state in which it is false (HEAD vs tree vs build vs live).
- A row may not cite a report as code evidence. “`07` says X” is READ; the code line is VERIFIED.
- Arithmetic rows must show the inputs and the operation, not just the result.

### G3. Evidence requirements by claim type

| Claim type | Minimum evidence | What does **not** count |
|---|---|---|
| **Runtime behaviour** | A quoted request/response or command output with timestamp, plus a control where a discriminator exists; two runs for a flaky/boolean probe | “the server was running”; a screenshot without a timestamp; a single run for a boolean that could race |
| **Code path** | The exact `path:line` read at the frozen hash, with the surrounding branch | A report citation; a grep hit without the branch; a comment describing intent |
| **Absence / negative** | The search scope (paths), the pattern, the tool, and the zero result; the same search re-run at write time | “no route found” without naming the router; “no daemon code” without the glob |
| **Provider fact** | The official URL, HTTP 200, fetch date, and the quoted sentence | A secondary blog; an AI-generated page; memory |
| **Arithmetic** | Inputs + formula + recomputation | A number copied from another report |
| **Design** | Clearly labelled DESIGN/OPINION; not promotable to VERIFIED by repetition | A design described in the present tense |
| **Contract fact** | `refusal.rs`/`event.rs`/`schemas/contract-v1.json` read, plus the web mirror | A test name alone |

**Special rule — refusal semantics (new in v2).** A claim that a refusal code “means” X must
list **every** raising condition for that code in the frozen tree, or the row is `CONTRADICTED`.
`E_CALL_LIMIT` today has five; a report that equates it with the provider call cap fails this
rule.

**Special rule — deployed behaviour (new in v2).** A claim about the running app must name the
`binary_sha256`/build time from G1. If the app was rebuilt after the probe, the claim expires
(G5) and the report must say so.

### G4. Contradiction protocol

1. **Register** both sides with a citation each; do not paraphrase one side.
2. **Classify**: runtime semantics / code-state drift / contract truth / design conflict /
   requirement-vs-design / process fact. A design conflict is not settled by a test.
3. **Settle with the cheapest re-runnable artifact.** Preferred order: a one-line HTTP probe →
   a unit test on a pure function → a scripted-provider harness → a two-run reproduction → a
   schema/census grep. Every settlement test must say what result would **confirm** and what
   would **refute**.
4. **Two-run rule** for any boolean runtime probe: both runs are quoted with timestamps. If the
   runs disagree, the contradiction is `E_INCONCLUSIVE` and the next discriminating check is
   named.
5. **`E_NOT_REPRODUCIBLE`** is a valid result. When a paid call is forbidden or the app is down,
   the runtime half of a contradiction is recorded as unreproducible, and the code half stands
   alone. Do not upgrade a code read to a runtime claim.
6. **Retire** the losing side explicitly (`status: CONTRADICTED`, with the settlement result).
   An unresolved contradiction may not be cited as a premise in a wave-3 plan.

### G5. Drift control and claim expiry (new in v2)

The corpus moved three times during wave 2 and once during this audit. v2 makes that visible
and bounded.

- **Freeze before writing.** Record `freeze_time_local`; every `observed_at_local` must be
  ≤ freeze.
- **Expiry triggers.** A claim expires when any of these happens after its `observed_at_local`:
  a commit (`git rev-parse HEAD` changes), a write to a file it cites, a build that produces a
  new binary, a deploy/launch, or 60 minutes elapse. The report must say `claims_expire_on`.
- **Re-verify at write time.** Every negative claim, every “current” fact, and every runtime
  claim is re-checked once immediately before the write; the re-check time goes in the header.
- **Fingerprint required for renewal.** A claim cannot be renewed after a build without the G1
  build identity. This is why Q2 is a gate prerequisite.
- **Stale-quote rule.** A wave-3 plan may cite a STALE row only with the words “was true at
  <state>; not true now”, and must name the replacement row.

### G6. Live-runtime protocol (read-only, no paid prompts)

- Only `GET` endpoints; no `SUBMIT_PROMPT`, no `POST/PUT/DELETE`, no approval decision, no
  paid prompt. Reads that may reach the provider are marked **provider-touching** and declared:
  in this audit `/api/provider/models` was called read-only (free metadata); `/api/provider/balance`
  was not called. A paid prompt (`SUBMIT_PROMPT`) was never submitted.
- Every probe row: method, path, status, byte length, a short quote, timestamp.
- Record the availability window. “Down during the probe” is data.
- Never restart, rebuild, or deploy as part of an audit. If another lane does, record the
  build/launch times and the new hash; do not adopt the new state silently.
- If a probe requires a `Host`/`Origin` override to prove a boundary, that is allowed (it is
  read-only) and must be quoted with both the hostile and control values.

### G7. Coverage and negative claims

- **Operator coverage** is against `OP-01…OP-45` (created by G-00). A report that says “all 36”
  or “all 45” must show the table, with a status per id and a citation per non-MISSING row.
- **Census tests** replace prose for “every X has a Y”: every task kind → card; every refusal
  code → message/raising conditions; every persona → allowlist; every daemon → card/schedule.
  A census claim without a census artifact is READ at best.
- **Negative claims** (absence) must name the search scope, pattern, and the zero result, and
  be re-run at write time. `18`’s false “`11` does not exist” is the canonical failure this
  rule exists to prevent.
- **Cross-lane merge test** (new): if two reports both claim to define the same artifact
  (schema, daemon entity, card contract, read policy), the gate requires a merge table naming
  one owner and every field the other must drop. “Both are research” is a rejection.

### G8. Safety and scope

- No product edits, builds, restarts, migrations, credentials, or paid calls by an audit lane.
- If a required capability is absent (e.g. `/deep-inspection`), name the substitute and its
  limits (this report: §1.2).
- If a lane cannot satisfy a mandatory field, it returns a blocker report; it does not invent
  a number, a build id, or a runtime observation.
- Any secret encountered is reported as leaked-by-presence, never quoted (contract §27).

### G9. Acceptance checklist (all mandatory; any “no” rejects)

```
[ ] G1 header complete; head + tree_hashes + deployed_build present (ABSENT is allowed only with consequences stated)
[ ] freeze_time_local present; every observed_at_local <= freeze
[ ] every claim has id, status, evidence_class, verified_against
[ ] zero bare VERIFIED; zero inherited VERIFIED; zero CONTRADICTED without settlement test
[ ] every STALE row names both states
[ ] every runtime claim has a timestamp and a control/2nd run where boolean
[ ] every negative claim names scope+pattern+zero and was re-run at write time
[ ] every refusal claim lists all raising conditions (or is CONTRADICTED)
[ ] deployed-behaviour claims name the build identity, or are INFERRED
[ ] operator coverage table uses OP-01..OP-45 and has a status per id
[ ] every cross-lane duplicate artifact has a merge table with one owner
[ ] live window (up/down) recorded; no paid call; no product edit/restart by the lane
[ ] contradictions: all settled or explicitly deferred with a question id
[ ] the report states what it did NOT verify (the "Unverified" table)
```

### G10. Verdicts and what v2 rejects

| Verdict | Meaning |
|---|---|
| **ACCEPTED** | All G9 boxes; no unresolved CONTRADICTED row is used as a premise |
| **ACCEPTED WITH DEBT** | All boxes except a named, non-load-bearing UNVERIFIED set; debt is listed |
| **RETURNED** | A mandatory field/rule is missing; the report is re-worked, not merged |
| **BLOCKED** | A capability or evidence source is absent and the lane reports it honestly |

v2 **rejects** explicitly:

- “Deployed” inferred from build order (must be ABSENT or proven).
- “All 36/45 answered” without the table.
- A claim register with no timestamps.
- A negative claim not re-run at write time.
- A contradiction “settled” by preference or by “operator will decide” without a question id.
- A design described in the present tense.
- A schema/entity defined twice by two lanes with no owner.
- A code read promoted to a runtime claim.

### G11. Expiry, renewal, and gate ownership

- A lane’s gate record is valid until the first expiry trigger (G5). After a commit/build/deploy,
  the affected rows are re-marked STALE and the report must be re-issued or an addendum written.
- Gate ownership: the Lead accepts/rejects; the lane that wrote the report cannot accept its own.
- An audit lane (this seat) may **mark** other reports, never edit them; the marks expire by the
  same rules.
- The next gate (v3) is required if either: (a) two lanes still disagree on a schema/entity
  owner after v2, or (b) a build fingerprint is still ABSENT when an implementation lane starts.

### G12. Gate record (copy into every wave-3 report)

```yaml
gate_record:
  gate: EVIDENCE-GATE-v2
  lane_id: <id>
  freeze_time_local: <ISO 8601>
  head: <sha>
  deployed_build_sha256: <sha or ABSENT>
  claims_total: <n>
  claims_verified: <n>
  claims_read: <n>
  claims_inferred: <n>
  claims_contradicted: <n>
  claims_stale: <n>
  contradictions_open: <n>
  operator_ids_covered: <n>/45
  unverified_items: <n>
  verdict: ACCEPTED | ACCEPTED-WITH-DEBT | RETURNED | BLOCKED
  debt_or_blocker: <one sentence or NONE>
```

---

## 8. Top 10 for the Lead (and the report’s own honesty tables)

### 8.1 Top 10

1. **Freeze the baseline and add a build fingerprint before any wave-3 code.** Four artifacts
   disagree; the deployed binary has no identity; `Q1`/`Q2` are prerequisites for every other
   answer. Eight wave-3 design files (`20`–`27`) landed while this audit was writing, and two of
   them (`22` vs `24`) already disagree about a measured live fact (X31). Without a freeze and a
   fingerprint, wave 3 will repeat wave 2’s drift at a higher cost.
2. **The 12-turn cap is per run, not per task** (`execution.rs:1592` + `tool_loop.rs:13`
   comment). The 4→12 fix does not deliver “normal research completes” under an 8-node plan.
   Settle with a two-task scripted harness before claiming the fix.
3. **`E_CALL_LIMIT` now has five meanings and a false message.** The money lie is fixed; the
   count lie is not. Decide 1:1 codes vs a detail field (`Q3`), then write the five-condition
   table test. Do not let the UI map it to a spend meter yet.
4. **Three “40” limits and one dead one.** Name the authoritative ceiling (`Q5`); delete or
   document `caps::MAX_CALLS`; make the refusal say which counter fired.
5. **The workspace root is the product’s own checkout, and the documented env var is
   ignored by the launcher.** Decide precedence and portable fallback (`Q7`); otherwise every
   file result and the `.git` surface are unowned.
6. **The Host guard landed mid-audit; three reports are now stale and the guard is
   Host-only.** Update the corpus (X7), then finish the boundary (`Q6`: Origin + token on
   mutations). The deployed build blocks rebinding; it does not block same-machine loopback
   CSRF.
7. **Approval evidence is better but not durable.** Persist a redacted preview or a derived
   target (`Q8`); today a reconnect returns the operator to a digest-only blind overlay, and
   stale entries still lack Dismiss.
8. **The persona fix does not reach the default path, and capability lookup is fail-open for
   UI roles.** Decide lookup authority and fail-closed semantics (`Q9`/`Q11`). The truth doc’s
   “leaf prompts use persona bodies” must be narrowed to persona-named roles.
9. **The operator’s six named areas are still decisions, not designs:** modes 1–4 (`Q10`),
   all-agent cards (`Q12`/`Q13`), Remote/Watchdog (`Q14`/`Q15`), no-hard-budget (`Q18`), Venn
   memory (`Q19`/`Q20`), and “Now” (`Q17`). Wave 2 designed three daemons and two memories; the
   Lead must name one owner per artifact before a migration number is assigned.
10. **Two small truth defects are cheap to prove and cheap to fix:** `/diagnose` uptime is
    always 0 (fresh `ServiceState` per request), and `E_PROFILE_NOT_FOUND` is an ad-hoc refusal
    absent from the contract and the web map. Both are one-test defects; both undermine a
    “truthful UI” claim out of proportion to their size.

### 8.2 Done (with evidence)

| Item | Evidence |
|---|---|
| Read all 19 reports (`01`–`18` + `09`) and `OPERATOR-DECISIONS`; sampled claim registers/Q&A/errata in each | §1.1; §3 has 143 rows with per-report citations |
| Read `AGENT-ARCHITECTURE-TRUTH.md` at HEAD and in the working tree | §2 C1–C5; file read + `git diff` |
| Read the current source for caps, tool loop, spawn policy, persona lookup, HTTP boundary, jail, approvals, budget, refusal | §3 rows; §4 settlement tests |
| Probed the live app read-only (health/diagnose/session/profiles/memory/runs/projects/models) and recorded up/down | §1.3, 17 probes/observations with timestamps |
| Reproduced Host guard and `/diagnose` uptime twice each | §1.4 (421/200 ×2; 0 vs 88, 0 vs 89) |
| Built a 143-row claim ledger across all reports with five statuses | §3 |
| Registered 31 contradictions, each with a re-runnable settlement test | §4 |
| Mapped the six brief-named operator gaps plus six additional blocking gaps | §5 |
| Wrote exactly 20 new questions with 3–5 options and a blocking rationale each | §6 |
| Defined EVIDENCE-GATE-v2 with header, ledger schema, evidence classes, contradiction protocol, expiry, checklist and record | §7 |
| No product edits, no paid calls, no credentials, no restarts by this seat | this file only; §1.3 records the other lane’s restart |

### 8.3 Not done (and why)

| Item | Reason |
|---|---|
| `/deep-inspection` | Not installed; `/deep-investigate` + `/trace` substituted (§1.2) |
| Runtime proof of 12 turns / `E_CALL_LIMIT` / persona continuation | No paid prompt permitted; no scripted-provider path in the production binary → `E_NOT_REPRODUCIBLE` |
| Reading all 19 reports line-by-line | Corpus is ~1.8 MB; claim registers, Q&A and load-bearing sections were read; unread sections are labelled READ/not claimed |
| Re-fetching official DeepSeek pages | No external fetch by this seat; external facts marked READ from the lane that fetched them |
| Fixing any defect found | Audit-only brief; no product edits |
| Verifying the 22:00 binary contains 12/40 by binary inspection | No build fingerprint/symbol proof; INFERRED from build order, explicitly labelled |
| Independent verifier child | `maxDepth: 1`, remaining levels 0 |

### 8.4 Unverified (what would prove it)

| Claim | Status | What would prove it |
|---|---|---|
| The 22:00 build contains 12/40, `E_CALL_LIMIT`, persona continuation, `args_preview` | INFERRED (build order + Host 421 proves only the 21:52 `http.rs` change is in) | G1 build fingerprint + a scripted 13-turn task returning `E_CALL_LIMIT` |
| The tool-turn cap is shared per run | INFERRED from `run_tool_counters(run.id)` + `next_turn`; deterministic code path, not executed | Two-task scripted harness (X1) |
| `E_CALL_LIMIT` five-condition mapping | VERIFIED by code reads; messages not exercised | Table test over five forced conditions (X2) |
| `/diagnose` uptime always 0 | VERIFIED twice; code cause read | A third probe after 10 minutes; or a unit test on `diagnostics()` |
| Approval preview absent on replay | INFERRED from `tools_bridge.rs:571` + contract comment; not exercised | Request → drop WS → reconnect; assert `args_preview` absent |
| Workspace env var ignored by the launcher | INFERRED from the two variable names; not exercised | Launch with `HIVEROUTER_WORKSPACE_ROOT=D:\tmp\proj`; inspect resolved root |
| Live run history has no post-fix success | VERIFIED at 21:53 and 22:02 (5 runs, newest 03:26Z) | A new run after the 22:00 build |
| Operator count is 45, not 36 | VERIFIED by count | Stable ids (G-00) and an operator ack |
| `04` vs `13` memory conflict is load-bearing | VERIFIED (both claim v11, differ on global/raw) | One DDL review with both authors |
| Corpus claims about external pages | READ only | Re-fetch the named URLs and quote |

### 8.5 Gate record for this audit (v2 applied to itself)

```yaml
gate_record:
  gate: EVIDENCE-GATE-v2
  lane_id: w3-s19-gap-audit
  freeze_time_local: 2026-11-07T22:13:56-06:00
  head: bd575ba7adf3a5f2a9bdbc69f9bbf7084fe6fe65
  deployed_build_sha256: 31252FABA1F62E2213763E22B2BEC0EA18975507EC8717DAC9EEBBEBBE63867D
  claims_total: 143
  claims_verified: 50
  claims_read: 58
  claims_inferred: 2
  claims_contradicted: 14
  claims_stale: 18
  contradictions_registered: 31
  contradictions_open: 0          # all 31 are settled by a named test or deferred to a question id
  operator_ids_covered: 0/45      # ids do not exist yet (G-00); §5 covers the six named areas + 6 more
  unverified_items: 10
  verdict: ACCEPTED-WITH-DEBT
  debt_or_blocker: "deployed binary has no build fingerprint; 12/40 and E_CALL_LIMIT are inferred, not runtime-proven; operator ids absent"
```

**Self-assessment:** this report passes the G9 checklist except the items it records as debt
(no build fingerprint in the product, no operator ids, no runtime proof of the 12/40 path). It
does not edit the corpus it audits; all marks are in this file and expire on the next
commit/build.

**Bottom line.** Wave 1 is a strong code read with a weak time model. Wave 2 is a strong design
corpus with a weak authority model. Wave 3’s first deliverable must not be another design; it
must be a frozen baseline, a build identity, and a decision owner for each artifact the corpus
has defined twice. The twenty questions in §6 are that owner list.

---

*End of seat 19. Only this file was written. No product source, test, config, migration,
credential, server process or paid provider call was touched by this seat.*
