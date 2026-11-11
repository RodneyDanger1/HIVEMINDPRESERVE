# Round 95 — Independent Watchdog Report

**Watchdog:** independent verifier. **Observed:** 2026-11-10T23:34–23:52 (America/Chicago).
**Repo:** `C:\Users\Riley\Desktop\hiverouter-v1` · **HEAD:** `11293498ad4812e7389055beaf187558e7ff084f` (`1129349`) — `VERIFIED` (`git rev-parse HEAD`).
**Working tree at verification:** `M README.md`, `M docs/CODEMAP/02-ARCHITECTURE.md`, `M docs/RELEASE-EVIDENCE.md`, `M docs/UI-UX-ROADMAP.md`; untracked `docs/audit/round-95/`, `docs/design/`, `docs/evidence/live/{INDEX.md,full-battery-20261110-232942.json}`, `docs/plans/HIVEMIND-FINISH-PLAN-v3-STATUS.md`, `ARCHITECTURE.md`, `docs/DOWNLOADS.md`, `docs/audit/HIVEMIND-TRUTH-AND-ROADMAP-2026-11-10.md` — `VERIFIED` (`git status --short`).
**Scope:** this file is the only write; the five audit documents and both design documents were read-only.

A prior draft of this path existed when this pass began. Its checks were re-run; two of its factual claims are contradicted by the files at verification time (§4.6). Its verified checks that I independently re-confirmed are retained.

---

## 1. Required counts (independently counted)

Method: PowerShell regex over the exact section body. Use-cases and 06 cannots are `^\s*(\d+)\.\s+\*\*`; 05 cannots are `^\s*(\d+)\.\s` (no bold lead-in).

| Requirement | Counted | Numbering | Verdict |
|---|---:|---|---|
| `06-use-cases.md` §1 numbered use cases | **100** | 1…100, ascending, zero gaps, zero duplicates | **PASS** |
| `06-use-cases.md` §2 numbered cannots | **100** | 1…100, ascending, zero gaps, zero duplicates | **PASS** |
| `05-operations.md` §4 numbered cannots | **100** | 1…100, ascending, zero gaps, zero duplicates | **PASS** |

`06` §1 headings A–G and §2 headings A–J are all present. `05` §4 headings A–I are all present.

---

## 2. Structure, non-emptiness, truncation, order

| Document | Bytes / lines | End state | Verdict |
|---|---:|---|---|
| `01-core.md` | 20,296 / 215 | Ends with a complete `Unverified` table row ("HiveWire intended meaning") | **PASS (structure)** |
| `02-plan.md` | 27,086 / 300 | Ends with a complete one-line bottom line ("…labelling the review-only pass a monitor.") | **PASS (structure)** |
| `04-models.md` | 7,571 / 83 | Ends with a complete sources list (Claude models overview link) | **PASS (structure)** |
| `05-operations.md` | 29,029 / 297 | Ends with a complete `Unverified` table row (README budget enforcement) | **PASS (structure)** |
| `06-use-cases.md` | 39,158 / 353 | Ends with a complete `Unverified` table row (100-list exhaustiveness) | **PASS (structure)** |
| `HIVEMIND-BRAND-SYSTEM-v1.md` | 21,736 / 346 | Ends with the "Not verified in this pass" paragraph | **PASS (structure)** |
| `HIVEMIND-DESIGN-SPECS-v1.md` | 28,545 / 346 | Ends with the Unverified paragraph after the §18 verified/proposed split | **PASS (structure)** |

All seven are non-empty, end mid-nothing (complete sentences/tables), and have coherent section order (`0/1…n`, then Done → Not done → Unverified where present). All five round-95 documents begin with a provenance block and end with report-shape sections.

**Coverage gap:** no `03-*.md` exists in `round-95/`; the numbered set is `01, 02, 04, 05, 06`. This is a missing work package, not truncation. Task ids referenced are `task-24` (01), `task-25` (02), `task-28` (05), `task-29` (06); no 26–27 evidence exists in this set.

---

## 3. Independent spot checks against code/file state

| # | Claim | Evidence observed | Verdict |
|---:|---|---|---|
| 1 | HEAD is `1129349` | `git rev-parse HEAD` → `11293498ad4812e7389055beaf187558e7ff084f` | **VERIFIED** |
| 2 | `1129349` is "two receipts only" | `git show --stat 1129349` → **4 files, 139 insertions**: `crash-drill…json`, `scale-drill…json`, `tools/qa/run-crash-drill.ps1`, `tools/qa/run-scale-receipt.ps1` | **REFUTED** (05 §0 is wrong; 02/06's "2 commits, QA files included" is right) |
| 3 | Installed = `dist` build | `dist/HiveMind/BUILD.json` and `%LOCALAPPDATA%\Programs\HiveMind\app\BUILD.json` byte-equal in all fields: `commit 806251b80ef…`, `version 1.2.0-dev`, `dirty:false`, server sha `d9d769…2c78` | **VERIFIED** |
| 4 | Installed build is 2 commits behind HEAD, no product-code diff | `git log 806251b..HEAD` → `1129349`, `0af9ed6`; combined stat = 5 files / 148 insertions (`docs/evidence/live/*`, `tools/qa/*`) | **VERIFIED** (02/06 correct; 05 line 19 says "one commit ahead") |
| 5 | Newest battery is HEAD-bound | `full-battery-20261110-232942.json`: `source_commit` = full `1129349…`, `working_tree_dirty:true`, `verdict PASS`, `executed 8`, `skipped 0`, `failed 0`, 8 steps | **VERIFIED** |
| 6 | That receipt is tracked | `git status --short` → `?? docs/evidence/live/full-battery-20261110-232942.json`; untracked | **VERIFIED (untracked)** |
| 7 | Caps 3 / 8 / 64 / 64 | `caps.rs:9,11,13,18` (`MAX_SPAWN_DEPTH=3`, `MAX_FANOUT=8`, `MAX_AGENTS=64`, `MAX_CALLS=64`) | **VERIFIED** |
| 8 | Tool-loop budgets 12 turns / 40 calls / 64 KiB / 8 waits | `tool_loop.rs:14,16,18,21` | **VERIFIED** |
| 9 | 12 tools, 5 tiers | `hiverouter-tools/src/tool.rs:117-139` → 12 `ToolKind` variants; tiers at `:78-86` (5) | **VERIFIED** |
| 10 | 35 grants | `grant_count()` exists (`tool.rs:603-607`) summing allowlists, but no literal 35 was independently recounted | **UNVERIFIED** |
| 11 | Monitor is review-only | `execution.rs:2342` "One bounded, read-only post-dispatch monitor turn"; `provider_executor.rs:709` "No tools are advertised"; install receipt's `what_this_does_not_prove` says it cannot `send_instruction`/`cancel_task`/repair | **VERIFIED** |
| 12 | EVO-GATE is shape-only | `evolution_gate.rs:178` is the **only** `head_commit` occurrence (fixture); receipt adds schema/`proposal_id`/n≥8/evaluator/CI checks; artifacts sha256 format-checked only (`:144-157`); 64 KiB cap (`:23`) | **VERIFIED** (head_commit never compared to HEAD) |
| 13 | No vector/embedding memory | Grep `embedding|vector|cosine|ANN|faiss` over `hiverouter-core/src/*.rs` → no memory hits (only channel/escape-vector comments); retrieval is `ORDER BY created_at DESC LIMIT` in `store.rs` | **VERIFIED for core** (whole-crate claim not re-run) |
| 14 | No HiveWire | Case-insensitive grep over `*.rs,*.ts,*.tsx,*.json,*.ps1,*.toml` excluding `target/node_modules/dist/docs` → **0 matches** | **VERIFIED (non-doc code)** |
| 15 | Design header is 52px, not 56–64px | `web/src/tokens/tokens.ts:167` `topbar: 52`; `Header.tsx:50` `h-[52px]`; `HIVEMIND-DESIGN-SPECS-v1.md:16,28` says Header 52px | **VERIFIED** |
| 16 | 06 "can perform today" despite key blocks | 25 occurrences of "blocked until key" + 5 "requires (an) existing …" in 06 | **VERIFIED** (headline overclaim, §4.5) |
| 17 | 04 price arithmetic | 8.00/0.15 = 53.3×; 8.00/0.30 = 26.7×; 40.00/0.60 = 66.7×; 40.00/1.20 = 33.3× | **VERIFIED (arithmetic only)** |
| 18 | 04 model/pricing facts | Vendor pages were not re-fetched in this pass; no Anthropic adapter grep was re-run; no USD price constants exist in repo (only `peak/off-peak` and provenance machinery in `accounting.rs`/`pricing.rs`) | **UNVERIFIED** |

---

## 4. Contradictions and stale claims

### 4.1 `05-operations.md` provenance — **FAIL**
- `05-operations.md:19`: "installed app == `dist\HiveMind` bytes; HEAD **one commit ahead**".
- `05-operations.md:21`: "`1129349` changes **only** `docs/evidence/live/*.json` (two new receipts)."
- **Fact:** `806251b..HEAD` is **two** commits; `1129349` alone changes **4 files / 139 insertions**, adding two QA drill scripts (`tools/qa/run-crash-drill.ps1`, `tools/qa/run-scale-receipt.ps1`) beside the two receipts. `0af9ed6` then edits the install receipt. No product code differs, so the *capability* conclusion survives; the *provenance wording* does not.
- Corrected wording: "Installed package is `806251b`; HEAD is two commits ahead. `1129349` adds two receipts **and two QA scripts**; `0af9ed6` updates the install receipt. No product code differs."

### 4.2 Battery receipt identity — **three documents stale**
- `01-core.md:7`: the committed battery receipt "records `source_commit: be03094` … therefore **cannot** be used to claim a green battery for this HEAD."
- `02-plan.md:56`: "Battery was **not re-run at HEAD** `1129349`; its `source_commit` is `c49f90c`."
- `05-operations.md:257` (G3): the receipts "are for a different commit than HEAD."
- **Fact:** `full-battery-20261110-232942.json` exists, is `PASS` 8/8 (0 skipped, 0 failed), `source_commit` = `1129349…` = HEAD, but **`working_tree_dirty:true`** and **untracked**. Older tracked receipts (`215831`→`be03094`, `225940`→`c49f90c`) are also dirty.
- Corrected wording: "The newest battery is HEAD-bound, PASS 8/8, but dirty and untracked; it proves the suite passes **with the uncommitted tree**, not a clean-HEAD seal. Older receipts are not HEAD."
- Consequence: 01's "no HEAD-matching receipt", 02's "not re-run at HEAD", and 05's G3 are now false. None invalidates the PASS; all three provenance sentences need replacement.

### 4.3 Working-tree claims have drifted
All five audit docs say the tree had exactly one uncommitted change, `backend/crates/hiverouter-core/src/http.rs` (+125 lines, `/api/evidence/ledger`). At verification `http.rs` is **clean** (`git diff --stat` empty) and `README.md` + three docs are modified instead, plus untracked files. `http.rs` at HEAD contains no `/api/evidence/ledger` route (grep found only a "lesson ledger" comment). I cannot reconstruct whether the `http.rs` edit was reverted by another process or was never present at this HEAD; the historical claim is **UNVERIFIED**, and every "only `http.rs` is dirty" sentence is false at read time.

### 4.4 `01-core.md` HiveWire search is self-inconsistent
01 says a repo-wide search for `HiveWire|Hivewire|hivewire|hive_wire|HIVEWIRE` returned **0 matches across code, docs, web, e2e and plans** (`01-core.md:63`). The audit file itself contains "HiveWire" repeatedly, so the search cannot have included the audit directory (or predated writing). The narrower claim I can confirm: **0 matches in non-doc code/config/scripts**. The docs half is **UNVERIFIED** and the wording is overstated.

### 4.5 `06-use-cases.md` headline overclaim — **FAIL**
- Heading `06-use-cases.md:25`: "100 concrete use cases HiveMind **can perform today**."
- Same file labels **25** items "blocked until key" and **5** more "requires (an) existing …". Only the read/operator surfaces are usable today; all model-turn items refuse `E_KEY_MISSING`.
- Corrected wording: "100 supported use-case surfaces; 25 are blocked until a provider key is configured and 5 require an existing run. Read-only/operator surfaces work today; model-dependent items are labelled."

### 4.6 Prior draft of this file — two corrections
A draft at this path claimed `HIVEMIND-DESIGN-SPECS-v1.md` was **11,173 B / 181 lines** and fixed the header at **56–64px**. At verification the file is **28,545 B / 346 lines**, and it states Header **52px** (`:16,28`), consistent with `tokens.ts:167` and `Header.tsx:50`. The 56–64px defect is **withdrawn**; the size/line figures were wrong. The draft's `http.rs M +125` observation is also not reproducible now (§4.3).

### 4.7 `04-models.md` external facts
The 384K output figure and all prices/benchmarks are vendor-source assertions not re-fetched here. Operating-contract §21 says to assume **neither 393,216 nor 384K** until a versioned official source is quoted; the document cites official pages but does not pin versions, so M1/M3/M4 remain **UNVERIFIED** in this watch. The "no Anthropic adapter" claim was not re-grepped; price arithmetic is internally correct.

---

## 5. Stub / placeholder sweep

Pattern: `TODO|FIXME|XXX|HACK|placeholder|NotImplemented|not implemented|mock|stub|dummy|lorem|simplified for|for illustration|TBD`.

**Result: zero `TODO`, `FIXME`, `XXX`, `HACK`, `dummy`, `lorem`, `simplified for`, `for illustration` in any of the seven documents.** Every match is classified:

| Hit | Classification | Why |
|---|---|---|
| `01-core.md:63` "NOT DOCUMENTED / NOT IMPLEMENTED" | Permitted — negative finding | Audit conclusion, not unfinished work |
| `02-plan.md:53` "mock-timer close"; `:197` "provider mock counts calls" | Permitted — test-double plan | Mock provider/clock under §7 boundaries |
| `02-plan.md:54` "not implemented [U]" | Permitted — defect statement | Explicit UI-2 gap |
| Brand `:99,:299` "mock banner/strip"; specs `:19,:39` "mock strip" | Permitted — product feature name | Mock-mode UI strip, not fake data |
| Brand `:270` "dark mode (not implemented, do not claim)" | Permitted — honest non-claim | Proposed-only, flagged |
| Brand `:310` "placeholder text in inputs"; specs `:263` "Label/placeholder distinction"; `:312` "no placeholder" | Permitted — UI/acceptance language | Not implementation stubs |

No document hides fabricated implementation behind stub wording. The audit files are reports, not code; their risk is stale evidence, not placeholders.

---

## 6. PASS / FAIL per section

| Section | Verdict | Reason |
|---|---|---|
| **01-core.md** | **PASS (technical) / FAIL (provenance sentence)** | Tier/spawn/caps/monitor/EVO-GATE/memory claims resolve at the cited lines; §1, §2, §5–§7 sound. Stale: `:7` battery and `:3,:193` working-tree statements. HiveWire wording overstated (§4.4). |
| **02-plan.md** | **PASS (technical) / correction** | WP matrix is specific and sampled citations resolve; installed build correctly stated as `806251b`, 2 commits behind. Stale: `:56` "battery not re-run at HEAD". Test counts (663/391/16) not recounted → **UNVERIFIED**. |
| **04-models.md** | **PASS (internal) / external UNVERIFIED** | Arithmetic correct; explicit UNVERIFIED table; honest "no Anthropic adapter". All vendor facts/market prices → **UNVERIFIED**; 384K conflicts with contract §21 until pinned. |
| **05-operations.md** | **FAIL** | 100-cannot list is exactly 100 and contiguous; operational content is strong. But §0 provenance is factually wrong: "one commit ahead" / "only two receipts" vs actual two commits / four files including two QA scripts (§4.1), and G3 battery sentence is stale (§4.2). |
| **06-use-cases.md** | **FAIL (headline) / PASS (counts)** | Exactly 100 + 100, contiguous, well grouped. But "can perform today" contradicts its own 25 key-blocked + 5 run-dependent items (§4.5). |
| **Brand system** | **PASS (structure)** | 14 sections, palette/topbar/375 rules resolve; screenshot files exist. Contrast ratios not recomputed → **UNVERIFIED**. |
| **Design specs** | **PASS** | 18 sections, complete acceptance checklist; header 52px matches code; no 56–64px defect (prior draft error corrected). No stub placeholders. |

---

## 7. Final verdict

**Overall: CONDITIONAL PASS on technical substance; FAIL on provenance accuracy and headline truthfulness.** Binary answer: **FAIL pending correction** — the round cannot be cited as green as written.

What is solid:
- Both required counts pass exactly: **100 + 100 in 06, 100 in 05**, contiguous 1–100.
- All seven documents exist, are non-empty, complete at EOF, coherently ordered; no truncation.
- Caps, tool-loop budgets, monitor review-only, EVO-GATE shape-only (no `head_commit` comparison), no-vector memory, no-HiveWire-in-code, and the 12-tool surface all re-verify at the cited code.
- Installed build identity is exactly as 02/06 say: `806251b`, `1.2.0-dev`, `dirty:false`, byte-equal `dist`/install `BUILD.json`, two commits behind HEAD, no product-code diff.
- Stub sweep is clean of TODO/FIXME/placeholder defects; every hit is a legitimate feature name or honest gap statement.

What must be fixed before this round is trusted:
1. **05 §0 provenance** (two commits, four files, two QA scripts) — factual error.
2. **Battery sentences in 01, 02, 05** — a HEAD-bound PASS receipt now exists but is **dirty and untracked**; state that precisely.
3. **06 §1 heading** — 25/100 items are blocked until a key; "can perform today" overclaims.
4. **Working-tree sentences in all five docs** — `http.rs` is clean at verification and README/docs are modified; re-state or re-verify.
5. **01 HiveWire "0 matches across docs"** — narrow to code (verified) or re-run including audit docs.

### Unverified (this watch)
1. Vendor model facts in 04 (context/output/prices/benchmarks); 384K conflicts with contract §21.
2. "No Anthropic adapter" (not re-grepped); installed provider profile beyond the receipt's `missing_key:true`.
3. 35-grant count (`grant_count()` exists; sum not recounted); 663 Rust / 391 web / 16 Playwright test counts.
4. Live exercise of the monitor path; monitor code was read, not run.
5. EVO-GATE full schema validation beyond `head_commit`, 64 KiB cap, and artifact sha format; evaluator execution.
6. HiveWire repo-wide search including docs/web/e2e; the `01-core.md` claim is self-inconsistent.
7. History of the `http.rs` working-tree change (present at audit time, absent now).
8. Brand contrast ratios (computed claim, not re-derived); UI click-throughs for the cited panels.
