# WATCHDOG LEDGER — HiveMind analysis swarm

**Auditor:** ledger watchdog / fact-checker / report compiler (read-only lane).
**Repo:** `C:\Users\Riley\Desktop\hiverouter-v1` — HEAD `6a41250`, branch `master`, working tree dirty only by untracked swarm artifacts.
**Reports audited (all seven present and stable):**

| # | Report | Lines | Bytes |
|---|---|---:|---:|
| L1 | `docs/analysis/lanes/L1-runtime-governance.md` | 652 | 72,849 |
| L2 | `docs/analysis/lanes/L2-tools-security.md` | 481 | 63,315 |
| L3 | `docs/analysis/lanes/L3-state-memory.md` | 559 | 76,547 |
| L4 | `docs/analysis/lanes/L4-api-provider.md` | 571 | 81,164 |
| L5 | `docs/analysis/lanes/L5-ui-ux.md` | 589 | 71,007 |
| L6 | `docs/analysis/lanes/L6-tests-release-install.md` | 743 | 80,316 |
| L7 | `docs/analysis/lanes/L7-docs-claims.md` | 776 | 63,919 |

**Machine-readable companion:** `docs/analysis/WATCHDOG-LEDGER.jsonl` — 100 rows, one per checked claim.
**No source, test, doc, build, server, provider or database was touched by this lane.** The only write targets are the two files named above.

---

## 0. Verdict

**`AUDITED`** — all seven reports exist; 100 decision-grade claims carry a status; 3 are refuted or unsupported, 7 are stale, 2 remain unverified and 1 is test-name-only.

The verdict is scoped precisely (see §0.3). It does **not** mean the seven reports are error-free, and it does **not** mean their findings are runtime-verified — no test, build, server or provider was executed by any lane, including this one.

### 0.1 Counts by status

| Status | Rows | Share | Meaning |
|---|---:|---:|---|
| `CONFIRMED` | 87 | 87 % | cited source opens at the cited lines and shows the claimed text/behaviour |
| `STALE` | 7 | 7 % | claim was true of an earlier tree/line map; the current source contradicts the detail |
| `UNSUPPORTED` | 3 | 3 % | the source contradicts the claim, or the evidence range cannot support it |
| `UNVERIFIED` | 2 | 2 % | could not be established from source alone; not passed |
| `CITED-FROM-TEST-ONLY` | 1 | 1 % | citation names a test but neither the report nor this audit read its body |
| **Total** | **100** | | |

Severity after normalisation (§Appendix A): **CRITICAL 2 · HIGH 34 · MEDIUM 41 · LOW 23.**

### 0.2 Counts by lane

| Lane | Rows | CONFIRMED | STALE | UNSUPPORTED | UNVERIFIED | CITED-FROM-TEST-ONLY |
|---|---:|---:|---:|---:|---:|---:|
| L1-runtime-governance | 25 | 22 | 1 | 1 | 0 | 1 |
| L2-tools-security | 15 | 14 | 0 | 1 | 0 | 0 |
| L3-state-memory | 11 | 11 | 0 | 0 | 0 | 0 |
| L4-api-provider | 15 | 12 | 2 | 0 | 1 | 0 |
| L5-ui-ux | 10 | 9 | 0 | 1 | 0 | 0 |
| L6-tests-release-install | 12 | 10 | 2 | 0 | 0 | 0 |
| L7-docs-claims | 12 | 9 | 2 | 0 | 1 | 0 |

L3 is the cleanest lane (11/11). L2 has the single most damaging error (one HIGH claim refuted outright) but is otherwise 14/15 with unusually honest `[V]/[U]` tagging.

### 0.3 Claim-set definition (what "every material claim" means here)

No report defines claim ids, and none is exhaustive sentence-by-sentence. `claim_id` in the JSONL is therefore the ledger's own stable id (`Lx-NNN`). The audited claim set is:

1. every numbered item in a report's executive summary/verdict;
2. every row of its top-risk or top-bug table;
3. every file-count, path and coverage-table claim that a reader would use to decide coverage;
4. every claim whose correction would change a P0/P1 or CRITICAL/HIGH decision;
5. a representative sample of the remaining path:line citations (every citation class was opened at least once).

Out of scope, and stated as such rather than silently passed: long-tail descriptive prose (e.g. L5's per-panel accessibility inventories, L7's 378-file appendix index) has not been individually statused. If the Lead needs a sentence-level audit of those, it is new work, not a defect in this ledger.

---

## 1. Per-lane coverage vs `git ls-files`

Repository census at audit time: **1,573 tracked files**; **315** under code scopes (`backend/`, `web/src/`, `web/tests/`, `tools/`, `e2e/`).

Method: for each lane, take the path prefixes it claims to own, list every tracked code file under them, and check whether the report names the file. A file named only in another lane is marked covered-elsewhere. Full repo-wide basename check: **every tracked code file is named by at least one lane except `backend/crates/hiverouter-core/build.rs`.**

| Lane | Owned prefix(es) | Tracked code files | Named in report | Missing from report |
|---|---|---:|---:|---|
| L1 | `backend/crates/hiverouter-core/src/**`, `assets/prompts/` | 39 | 20 | 19 — all 12 core `src` files are named by another lane; the 7 other `.prompty` files belong to L2 |
| L2 | `backend/crates/hiverouter-tools/**` | 25 | 25 | **0** |
| L3 | `backend/crates/hiverouter-core/src/**` (stated subset) | 29 | 20 | 9 — all named by L1/L2/L4; the report's own coverage ledger is accurate |
| L4 | `backend/crates/hiverouter-provider/src/**`, `hiverouter-contract/src/**` | 19 | 19 | **0** |
| L5 | `web/src/**` | 74 | 74 | **0** |
| L6 | `backend/**/tests/**`, `tools/**`, `e2e/**` | 86 | 85 | `e2e/package-lock.json` only (lockfile, immaterial) |
| L7 | `docs/**` | 258 | 8 | Not a defect: L7's job is the doc corpus, and Appendix A is a complete 378-path index |

**Verdict on coverage:** no lane over-claims ownership. L1 and L3 explicitly disclaim the files they did not read (`store.rs`, `db.rs`, `ws.rs`, `http.rs` for L1; `planner.rs`, `caps.rs`, `context.rs` for L3) and those files are genuinely owned elsewhere. The only genuinely unowned file in the repository is:

- `backend/crates/hiverouter-core/build.rs` — named by no lane, read by no lane. See §4.

**Thin coverage (named but not audited):**

- `backend/crates/hiverouter-core/src/handshake.rs` — named only in passing by L6 (3 mentions); no lane states it was read. It is a boot/transport contract file and therefore a real coverage hole.
- `tools/qa/check-seal-topology.mjs` — L6 reads its refusals and call site, but no lane audits its predicate logic; its unit tests are also never executed (L6 D2).
- `web/src/api/mock.ts` — L5 reads only to line 70; the dev-only scenario body is unaudited (mocked-data risk is gated by `web/tests/apiBoundary.test.ts`, which L5 cites but did not run).

---

## 2. Cross-lane contradictions and duplicates

### 2.1 Resolved contradictions (a correction is required)

| # | Issue | Lanes | Resolution | Evidence |
|---|---|---|---|---|
| C1 | `finish` / `report_result` have no engine effect; "`finish` cannot finish" | L1 says they complete the task; L2 calls the opposite a HIGH "functional deception" | **L1 is right, L2 R6 is refuted.** The tool-turn dispatcher (`execution.rs:2652-2709`) has no `finish` branch, but the caller inspects `turn.executed` immediately afterwards and completes the task | `execution.rs:5411-5431` (`if let Some((finish_plan, finish_result)) ... return self.complete_task(...)`) and `:5432-5462` for `report_result` |
| C2 | Proposals panel sends `Authorization: Bearer` while other panels send `x-hivemind-token`; "one of the two is wrong … these buttons can be the only writes that 401" | L5 | **L5 item 6 is unsupported.** The server explicitly accepts either form for mutations | `http.rs:767-779`: `Authorization` → `strip_prefix("Bearer ")` `.or_else(|| x-hivemind-token)` |
| C3 | `db.rs` / `store.rs` size | L1 says 1,735 / 3,525; L3 says 1,806 / 3,653 | **L3 is right.** L1 used non-blank counts for these two files only | `(Get-Content).Count` = 1,806 and 3,653; L1's other 16 size claims match total-line counts |
| C4 | Parallel dispatch default "32" used to argue reachability of the approval race | L1 only | **Holds with a nuance L1 missed:** external endpoints are clamped to 2 | `http.rs:365-379` (`unwrap_or(32)` then `parallel.min(2)` when `is_external_endpoint()`) |

### 2.2 Genuine duplicates (do not double-count; the lanes agree)

| Finding | Lanes | Status |
|---|---|---|
| Snapshot revision allocated by row count; `INSERT OR REPLACE` overwrite after prune | L1 §2.11 / L3 #7 | Both CONFIRMED; identical conclusion, different fix text |
| Subagent persona claims `send_to_sibling` its tier does not grant | L2 §7.2 / L7 §0.5 | Both CONFIRMED; L2 adds the runtime-intersection reason it is invisible |
| Root runs an unregistered second tool vocabulary | L1 §2.6 / L2 R3 | Both CONFIRMED |
| ToolAgent grants unreachable (7/35 dead grants) | L1 / L2 §2.2 | Both CONFIRMED |
| Release evidence and HEAD are stale | L6 #2 / L7 #3 | Both CONFIRMED; L6 measures the binary, L7 reads the doc |
| `output_schema_id` is never enforced | L1 §10 / L2 §7.2 (persona scoping) | Both CONFIRMED |

### 2.3 Compatible-looking claims that are not contradictions

- **`/health` and `/diagnose` and the launch shim (L5 #7 vs L4 §1.4).** L5 notes the shim only covers `/api/*`; L4 notes the server launch gate also only covers `GET /api/*`. Both are right; the shim gap is harmless today and becomes a bug only if the gate widens. L5 already hedges the server impact as UNVERIFIED.
- **Route count (L4).** 60 `.route()` calls expand to 63 method+path pairs (three routes register `get` + `post`); 62 is correct only if `/ws` is excluded. Counting rule, not a lane disagreement.

---

## 3. Non-confirmed claims — complete list (10 unsupported/stale + 3 secondary)

Only 13 of 100 rows are non-confirmed, so all are listed rather than truncated to 15. Ordered by severity.

### 3.1 Unsupported / stale (10)

| # | ID | Sev | Claim | Correction location | Required correction | Observed evidence |
|---|---|---|---|---|---|---|
| 1 | `L2-001` | **HIGH** | "`finish` and `report_result` … the dispatcher has no branch for them, so `finish` cannot finish" (R6, "functional deception") | L2 §3.4 (line 141), §3.5 (lines 147-149), §4 note (line 222), §11 missing-coverage (line 373), §12 R6 (line 386) | Delete R6 and both "no engine effect" conclusions. Replace with: both are task-terminal and are completed by the engine after the turn (`execution.rs:5416-5430`, `5437-5462`); the real gap is that no test covers them | `execution.rs:5416` `if let Some((finish_plan, finish_result)) = turn.executed.iter().find(...)` → `:5430` `return self.complete_task(run, task, &terminal);` |
| 2 | `L7-004` | MEDIUM | Shipped `dist/HiveMind/README.txt:18-19` tells the operator to run `tools\set-provider-key.ps1` | L7 §4 item 4 (line 277) | Change citation to `README.txt:14-15`; the substance (tools/ absent from the zip) stands | Zip listing = `HiveMind.exe`, `hiverouter.exe`, `README.txt`; the key instruction is at `:14-15`; `:18-19` is the windowless-launcher paragraph |
| 3 | `L5-004` | MEDIUM | "Proposals … send `Authorization: Bearer` … one of the two is wrong against the server, so these buttons can be the only writes that 401" | L5 §9 item 6 (line 497), §10 P0 item 4 (line 533) | Downgrade to a header-style inconsistency. The server accepts both headers; there is no 401 path from the header choice | `ProposalsPanel.tsx:194` sends Bearer; `http.rs:767-779` accepts Bearer **or** `x-hivemind-token` |
| 4 | `L6-001` | MEDIUM | Strict gate dirty tree names `?? docs-analysis-tmp-index.txt` | L6 §5.2 D1 (line 224), §11 risk 2 (line 575) | Drop the filename; state "untracked swarm artifacts". Note the analysis swarm itself makes `docs/analysis/**` untracked, so the gate cannot be green while these reports exist | `git status --porcelain` now = `?? .census-tmp-files.txt`, `?? docs/analysis/` |
| 5 | `L1-024` | LOW | §1 is a coverage table of "every file in the lane" | L1 §1 (line 52) | Retitle "files I read" and list the 12 core `src` files it deliberately excludes (all are named by L1's own text as other lanes' or are unread) | L1 §1 lists 29 items but omits `approval_wait`, `bus`, `checks`, `config`, `daemon`, `handshake`, `lib`, `main`, `remote`, `remote_supervisor`, `static_ui`, `tools_bridge` |
| 6 | `L4-007` | LOW | "62 method+path routes" | L4 §0 (line 17), §2 (line 108) | State the counting rule: 63 pairs including `GET /ws`, 62 excluding it | `http.rs:609-699` has 60 `.route()` calls; `/api/remotes`, `/api/daemons`, `/api/provider/profile` each register two methods |
| 7 | `L4-009` | LOW | "78 refusal codes" | L4 §0 (line 17) | Recount macro-aware: a unique scan of quoted `E_*` literals returns 79 | `refusal.rs:17-22` is a macro-generated enum; 79 distinct `E_` strings in the file |
| 8 | `L6-007` | LOW | `hivemind-cli.mjs` is 80,075 B | L6 §11 risk 7 (line 580) | 80,007 B | `(Get-Item tools/hivemind-cli.mjs).Length` = 80007; file is git-clean |
| 9 | `L1-022` | LOW | `db.rs` (1735) and `store.rs` (3525) line counts | L1 §1 rows 13-14 (lines 68-69) | 1,806 and 3,653, or state the table mixes non-blank counts | `(Get-Content).Count`; all other 16 L1 size claims match total-line counts |
| 10 | `L7-011` | LOW | HMARS bugfix report is "HEAD-stale" | L7 §4 item 9 (line 282) | Keep, but say exactly which field is stale: `:7` records `02ef24e`, HEAD is `6a41250`; the artifact hashes at `:8-10` are current | `HMARS-BUGFIX-REPORT.md:7`; `git rev-parse --short HEAD` = 6a41250 |

### 3.2 Secondary (not unsupported/stale, still must not be passed as fact)

| ID | Status | Claim | Why it is not confirmed | What would prove it |
|---|---|---|---|---|
| `L4-011` | `UNVERIFIED` | "Every URL the React UI calls exists on the backend" | Only the direct fetches were re-derived (`/api/session`, `/api/history/runs*`); most panels call through the `UiApi` interface, so a URL-by-URL closure was not reproduced | Publish the `UiApi` method → route table as evidence |
| `L7-009` | `UNVERIFIED` | Live evidence JSONs are aggregate receipts, so "VERIFIED (live)" overstates them | Filed count (26) confirmed; per-file field audit not performed | List the missing fields (frames, prompt text, timestamps) per receipt |
| `L1-025` | `CITED-FROM-TEST-ONLY` | Test-name citations at `scale_s1.rs:101`, `snapshot_history.rs:*`, `graph_authority.rs:*`, `kill_restore.rs:*`, etc. | Test **names and lines** were verified to exist exactly; **bodies were not read** by L1 or by this audit | Read the bodies and assert the claims, or run the suite |

---

## 4. Missing / uncited areas (repo-wide)

1. **`backend/crates/hiverouter-core/build.rs`** — the only tracked code file named by no lane. Build-script behaviour is therefore unaudited, including whether it can mutate embedded assets and whether it is reproducible.
2. **`handshake.rs`** — named only in passing by L6; no lane read it. Both the HTTP and WS boot contracts depend on it.
3. **No test, build or server execution anywhere.** Every "passes/works" statement in every lane is unexecuted. This is deliberate under the read-only mandate but must not be read as a green suite. In particular: L1 cannot state the approval race is reachable, L2 cannot state `cmd.exe` executes, L6 cannot state the gate fails — all three are static deductions that this ledger confirms only as code paths.
4. **`docs/evidence/live/*.json` (26 files)** — counted, not field-audited (see `L7-009`).
5. **`web/src/api/mock.ts`** body (lines 70-368) and **`web/src/figcraft/FigCraftGallery.tsx`** (lines 140-344) — explicitly partially read by L5; unaudited here.
6. **No runtime UI observation.** L5's "feels alive", focus order, toast stacking and animation cost are judgements from source, correctly labelled as such.
7. **`backend/crates/hiverouter-core/src/approval_wait.rs`, `bus.rs`, `history.rs`, `static_ui.rs`** — each is named by a lane but only API-scanned or partially read; no lane owns them deeply.

---

## 5. Cross-verified findings the Lead should not miss

These are `CONFIRMED` independently by this audit (not merely restated), highest impact first. They are the claims a fix plan should act on.

| # | Finding | Lanes | Confirmed evidence |
|---|---|---|---|
| 1 | Approved `shell_exec` is arbitrary code execution: default allowlist is `cmd.exe`, `git.exe`, `cargo.exe`; the jail bounds only cwd | L2 | `shell.rs:66-72`; `shell.rs:94-103`; no OS sandbox |
| 2 | The tool-bridge jail never denies the data directory while the engine jail does, so file tools can read the ledger and approved writes can corrupt the SQLite DB | L2 | `http.rs:259-262` vs `:347-351`; default `data/` sits inside the workspace |
| 3 | Root control runs a second, unregistered tool vocabulary with no registry/schema/approval/census coverage | L1, L2 | `provider_executor.rs:39-45,583-666`; `execution.rs:4381-4811` |
| 4 | Approval grants ignore TTL and identity: the clock is discarded and lookup is `(run, tool, args_sha256)` only | L2 | `approval.rs:935` (`let _ = now_unix_secs;`), `:942-951` |
| 5 | Ledger-write failure is fail-open: a decided row cannot be cancelled, and the timeout path later executes the committed `allow_once` label | L2 | `tools_bridge.rs:387-394`; `approval.rs:640-643`; `execution.rs:3459-3468` |
| 6 | Approval suspension is single-slot per run; a second concurrent suspend is illegal and fails the run | L1 | `run_state.rs:510-512`; `execution.rs:3108`; parallel default 32 (`http.rs:368`) / 2 for external endpoints (`:374-378`) |
| 7 | Structural rows are settled `Completed` without executing and without a `task_results` row; a single-node plan never runs the root | L1 | `execution.rs:1784-1791`, `:1814`, `:2206` |
| 8 | 64 provider calls cannot cover a legal 40-node plan with task sends and continuations | L1 | `caps.rs:18`; `planner.rs:10`; `tool_loop.rs:14` |
| 9 | Detached "remote" principals are in-process fixtures (`transport_kind=in_process`, `endpoint_ref=fixture`) while prompts imply real remotes | L1 | `execution.rs:5180-5182` |
| 10 | The current tree cannot pass `run-gates.ps1` (strict dirty; rehearsal G7 hash mismatch; release pre-release exit 3) | L6 | artifact sizes vs manifest; `web/package.json` 1.2.0-dev vs manifest 1.1.0 |
| 11 | Seal-predicate unit tests exist but no gate runs `node --test` | L6 | 0 grep hits; both `.test.mjs` files present |
| 12 | 12 tools / 5 classes / 35 grants, and 7 of those grants are unreachable (ToolAgent 4 + HyperAgent 3) | L2, L7 | `tool.rs:460-516`; `role_class` has no `tool` arm; root never enters the tool loop |

---

## 6. Verification log

What was actually done (no execution of product code):

- Read all seven reports end-to-end in lane order; extracted 100 decision-grade claims.
- Opened every cited file at the cited lines with a line-numbered dumper (`Get-Content -Encoding UTF8`, explicit ranges, file length printed).
- Recomputed every count claim: line counts (`.Count`, not `Measure-Object -Line`, which excludes blank lines and would have produced false STALEs), `git ls-files` (1,573), route count, command/event counts, test-attribute count (186), web spec count (31+1), refusal-string count (79), corpus counts (386 md / 990 docs / 26 JSON / 36,417 tree files).
- Compared `release-manifest.json` bytes to on-disk binary sizes (exact match to L6's figures).
- Listed the portable zip members with `System.IO.Compression.ZipFile`.
- Validated every JSONL line with `ConvertFrom-Json` and asserted all nine required fields and an allowed status on each row (100/100 valid).
- Verified 20+ test-name citations resolve to the exact test function at the exact line.

What was **not** done: no `cargo test`, no `cargo build`, no `npm test`, no server start, no provider call, no browser, no database open, no source edit, no commit. No key was printed; no secret was read.

---

## 7. Limitations of this ledger

1. **Static only.** A `CONFIRMED` row means "the source says this", not "the behaviour was observed".
2. **Sampling.** 100 claims across 4,371 report lines. Every top-risk row and verdict item is included; long-tail prose is not (see §0.3).
3. **No test bodies.** Where a report leans on a test, the ledger can confirm the test exists and is named as claimed, not that its assertions hold (only `L1-025` is explicitly test-only; other test-citing claims were graded on whether the *cited assertion text* was visible).
4. **Line numbers are HEAD `6a41250`.** Any commit invalidates the line half of every citation; the `observed_evidence` column preserves the quoted text so the claim can be re-anchored.
5. **Severity is the auditor's normalisation** of each report's own scale (§Appendix A). It is not a re-scoring of the underlying risk.

---

## Appendix A — status and severity normalisation

**Status values** (exhaustive): `CONFIRMED`, `STALE`, `UNSUPPORTED`, `UNVERIFIED`, `CITED-FROM-TEST-ONLY`.

**Severity mapping applied to the reports' mixed scales:**

| Report scale | Ledger value |
|---|---|
| `S1`, `P0`, `MED-HIGH` | `HIGH` |
| `S2`, `P1` | `MEDIUM` |
| `S3`, `P2` | `LOW` |
| `CRITICAL`, `HIGH`, `MEDIUM`, `LOW` | unchanged |

**JSONL row schema** (all 100 rows carry every field):

```json
{"report":"Lx-....md","claim_id":"Lx-NNN","claim":"...","cited_path":"...","cited_lines":"...","observed_evidence":"...","status":"CONFIRMED|STALE|UNSUPPORTED|UNVERIFIED|CITED-FROM-TEST-ONLY","severity":"CRITICAL|HIGH|MEDIUM|LOW","correction":"..."}
```

`correction` is non-empty for every non-`CONFIRMED` row and for 15 `CONFIRMED` rows that carry a nuance or partial-coverage note (e.g. a caveat that does not change the verdict).

---

## Appendix B — how to use this ledger

1. **Act on §3.1 first.** Ten corrections; one (C1/`L2-001`) would otherwise send the Lead to "fix" a working feature, and one (`L5-004`) would chase a non-existent 401.
2. **Dedupe before planning.** Snapshot-revision, subagent-persona, root-vocabulary and dead-grant findings are reported twice (§2.2); count them once.
3. **Do not reuse any lane's line numbers after a commit.** Re-anchor on the quoted text in `observed_evidence`.
4. **Treat every "tests pass" as unverified.** The only green evidence in the repository is `e2e/test-results/.last-run.json`, which cannot distinguish 95 passed from 0 collected (`L6-005`).
5. **Keep the ten confirmed HIGH/CRITICAL findings in §5 in the plan**; they are the ones this audit could trace to source without ambiguity.

---

*End of ledger. Both outputs were produced read-only with respect to the repository; the swarm's reports were not rewritten.*
