# L7 — Documentation, Claims, Embedded Prompts, and Release Truth

**Lane:** L7 (7/7) · documentation, claims, embedded prompts/personas, audits/plans/evidence, release truth
**Repository:** `C:\Users\Riley\Desktop\hiverouter-v1`
**HEAD audited:** `6a41250` (`docs: distinct voter review lenses and final hashes`, 2026-11-10 18:11 -0600)
**Version on disk:** `1.2.0-dev`; sealed release authority: `1.1.0` (`tools/release-manifest.json`, tag `hivemind-1.1.0` → `87079ce`)
**Mode:** read-only. No source, config, test, prompt, or git mutation. This file is the only write.
**Execution limits:** no `cargo test`, build, server start, browser run, or provider call was performed by this lane. Code facts below are **static reads at HEAD** (graded VERIFIED-static). Recorded gate counts and live TokenLab rows are **READ** from documents/receipts, not independently re-executed.

> **Grade legend.** `VERIFIED` = directly confirmed by source/doc read at HEAD (static); `READ` = recorded in a doc/evidence receipt but not re-executed; `UNVERIFIED` = no adequate evidence; `STALE` = contradicted by the current tree; `PROPOSED` = design/roadmap only, not implemented. No mock, seeded, or wiremock evidence is counted as live proof.

---

## 0. Executive verdict

The documentation corpus is large, unusually honest in tone, and materially out of date at the exact places an operator would trust most:

1. **Evolution/votes/UI are implemented, but three current-truth docs still say they are not.** `vote_evolution`, auto-spawned four-voter election, operator authorization, lesson apply/rollback, `ProposalsPanel` buttons, and `LessonsPanel` all exist (`backend/crates/hiverouter-tools/src/tool.rs:450-455`; `backend/crates/hiverouter-core/src/execution.rs:1625-1669`; `backend/crates/hiverouter-core/src/http.rs:637-652`; `web/src/panels/ProposalsPanel.tsx:294-326`). `README.md:348-349`, `docs/audit/HYPERAGENT-TRUTH-REPORT.md:390`, and `docs/audit/HYPERAGENT-PSEUDOCODE-REPORT.md:283,304,328` are stale.
2. **The tool surface is 12 tools / 5 permission classes / 35 tier grants, not 10, 11, 9, 32, or 30.** The registry enum is `tool.rs:115-140` (12 variants), contracts `tool.rs:382-456`, allowlists `tool.rs:460-516` (3+12+11+5+4 = 35). README simultaneously says 12 (`README.md:30`), 10 (`README.md:270`), and "11th" (`README.md:316`); the truth report says 10/32 (`HYPERAGENT-TRUTH-REPORT.md:118`) and 12 (`:336`).
3. **Release evidence no longer describes this tree.** `docs/RELEASE-EVIDENCE.md:25-58` records HEAD `889b779`, 475 backend / 179 web tests, and "no 1.2 artifact"; HEAD is `6a41250`, README records 637/231 (`README.md:169-170`), and a 1.2-dev portable package exists at `dist/HiveMind.zip` (14,416,165 B, SHA-256 matches `README.md:171-172`, observed by `Get-FileHash`).
4. **Installed-build/Downloads documentation is broken.** The current portable zip contains only `HiveMind.exe`, `hiverouter.exe`, and `README.txt` (observed zip listing). The 1.1.0 package contained `CHECKSUMS.txt`, `hiverouter-cli.exe`, `README.md`, `RELEASE.md`, and `RUNBOOK.md` (`dist/HiveMind-1.1.0-win64/`). The shipped `README.txt` tells the operator to run `tools\set-provider-key.ps1` (`dist/HiveMind/README.txt:18-19`), but `tools/` is not in the package. README has no Downloads/install section for this artifact.
5. **A persona claims a tool its tier does not grant.** `personas.json:32-36` gives the `subagent` persona `tier:"worker"` and `send_to_sibling`; the Subagent tier allowlist is only ask/report/finish/read/list (`tool.rs:498-507`). The provider intersection hides it at runtime (`provider_executor.rs:191-200`), so this is a claim/config bug, not an observed escalation — but the persona record is wrong and unvalidated.
6. **The provider architecture "truth" doc is a 2026-11-07 snapshot that is now substantially false** about subagents, per-hive memory, and root memory access (`docs/provider/AGENT-ARCHITECTURE-TRUTH.md:20,28-31,110-113`).
7. **CODEMAP's "exact" census is stale by 768 tracked files.** It claims 805 tracked / 36,480 working-tree files (`docs/CODEMAP/00-START-HERE.md:11-17`); measured at HEAD: 1,573 tracked / 36,408 working-tree files excluding `.git`. Its risks page still says persona loading, chat history, library assets, and worker spawning are absent (`docs/CODEMAP/09-RISKS-GAPS.md:5-12`), all contradicted by current code.
8. **Historical audit snapshots are cited without supersession banners.** `GOVERNANCE-AUDIT-2.md` (HEAD `9f8e2e0`) says root results are discarded and daemon admission is unaudited; both were fixed (`execution.rs:4319-4328`; `run_lifecycle.rs:1109`). `IMPROVEMENT-VERIFICATION.md:10` says the three PSEUDOCODE documents do not exist; they do. `IMPROVE-LESSONS.md:79` says the lesson layer is PROPOSED; apply/rollback/retrieval are implemented.
9. **The evidence files support READ-level claims only.** The 26 `docs/evidence/live/*.json` files are aggregate receipts (run id, usage, counts, HTTP codes), not raw frames or reproducible transcripts. They are still useful and honest, but a claim labelled "VERIFIED (live)" is stronger than the artifact supports; downgrade to READ unless the raw capture is committed.
10. **The honest core of the docs is sound.** The anti-AGI claims, caps, deterministic detached admission, proposal-only boundary, trace truth (1 WIRED / 264 PENDING / 5 MISSING / 49 UNVERIFIED of 319 — recomputed from `docs/trace/controls.json`), 1.1.0 sealed artifact hashes, and the "not a network satellite" caveats are consistent with code and should not be diluted.

---

## 1. Method, coverage, and measured census

### 1.1 What was read

| Area | Files | Depth |
|---|---:|---|
| `README.md` | 1 | Full (378 lines) |
| `docs/audit/**` | 24 md | Full: truth report (456), pseudocode report (388), live tests (477), bugfix report (444), governance audit 2, improvement verification, taxonomy, capability matrix, phase-B, final-delta; headers/index for the rest |
| `docs/claims/**` | 2 | Full CSV (81 rows) and skill-use record |
| `docs/plans/**` | 28 md | Full for phase evidence samples + finish plan; index and supersession check for all |
| `docs/evidence/**` | 4 md + 26 live JSON + 12 PNG | Full bugfix report and key-validation/smoke headers; live JSON fields read |
| `docs/ui/**` | 9 md | Design system, UI gap audit (targeted), roadmap, FigCraft brief/watchdog |
| `docs/research/**` | 38 md | Full for provider truth, agent-definitions, selected hmars 19/24/26 sections; index for all |
| `docs/contracts/**` | 2 md | Indexed + ADAPTERS/UI-COMMANDS headers |
| `docs/CODEMAP/**` | 19 md + 3 incoming | Full 00/09/17; 08/10/05/13/15 via targeted lines |
| `docs/provider/**` | 2 md | Full truth doc; DeepSeek integration header |
| `docs/trace/**` | 49 md | QA harness, capture registry, live README, orca/round summaries; `controls.json` recomputed |
| `docs/reports/**` | 1 md | Full FINAL-REPORT (77 lines) |
| `ledgers/**` | 2 | Full watchdog ledger |
| `plans/**` | 114 files | Full COMPILED-LEDGER/PROOF-RELEASE-PLAN relevant sections; waves/verification index |
| `audit/**` | 98 files | Full census/appendix headers and findings register index; historical classification |
| `runtime-test/**` | 82 files | Full pkg-final README/RUNBOOK headers; package inventories |
| Root markdown + prompts | 8 root md + 10 assets | Full `CHANGE-LEDGER-IMPL.md`, README/AGENTS/CLAUDE; all 8 `.prompty` + `personas.json` + `manifest.json` |
| `dist/**` docs | 5 md + zip | Package contents/README/RUNBOOK; zip listings and hashes observed |

### 1.2 Measured corpus (this audit)

| Measurement | Value | Method |
|---|---:|---|
| Markdown files outside `target/.git/node_modules` | **378** | recursive enumeration |
| Files under `docs/` (all types) | **981** | recursive enumeration |
| Non-markdown docs artifacts (json/log/png/txt/csv/svg) | **560** | recursive enumeration |
| Live evidence JSON receipts | **26** | `docs/evidence/live/*.json` |
| Files under `plans/` | **114** | recursive enumeration |
| Files under `audit/` | **98** | recursive enumeration |
| Files under `runtime-test/` | **82** | recursive enumeration |
| Git-tracked files | **1,573** | `git ls-files` |
| Working-tree files excluding `.git` | **36,408** | recursive enumeration |
| Tags | `hivemind-1.0.0`, `hivemind-1.1.0`, `v1.0.0` | `git tag --list` |

There is no 1.2 tag. The four backend crates and `web/package.json` declare `1.2.0-dev`.

### 1.3 Documentation family map

| Family | Count | Authority today | L7 verdict |
|---|---:|---|---|
| `README.md`, `docs/RELEASE*.md`, `docs/reports/FINAL-REPORT.md` | 5 | README is the operator truth; release docs mixed sealed/dev | **Current with stale rows** |
| `docs/audit/**` | 24 | HYPERAGENT-TRUTH-REPORT is the intended truth authority | **Current with stale sections; historical audits lack banners** |
| `docs/claims/**` | 2 | Register is dated 2026-11-06 and omits all 11/10 evolution work | **Stale/incomplete** |
| `docs/plans/**`, `plans/**` | 142 | Phase evidence is historical; root `plans/foresight` is 1.2 release planning | **Historical/provenance; plan-time claims stale** |
| `docs/evidence/**` | 34 md/json | Aggregate live receipts + bugfix report | **Receipts READ-grade; bugfix header stale** |
| `docs/ui/**`, `docs/UI-UX-ROADMAP.md`, FigCraft | 10 | Design system/roadmap + provenance assets | **Mostly honest; gap audit stale** |
| `docs/research/**` | 38 | Research/design input, not product truth | **Proposed/research; never cite as implemented** |
| `docs/contracts/**` | 2 | Frozen UI command contract + donor adapters | **READ; freeze date predates 1.2 commands? (not re-audited)** |
| `docs/CODEMAP/**` | 22 | Frozen encyclopedia at `e021b12` | **Frozen-valid at that commit; actively misleading at HEAD** |
| `docs/provider/**` | 2 | Provider/agent architecture explanation | **Stale** |
| `docs/trace/**` | 49 | Trace ladder truth | **Current (recomputed); historical round docs** |
| `audit/**` | 98 | Historical exhaustive audit at 507-tracked-file era | **Historical only; no banner in README** |
| `runtime-test/**` | 82 | 1.0/1.1 package smoke artifacts | **Historical; package docs stale** |
| Prompt assets | 10 | Embedded personas/prompts | **Live, but one persona exceeds tier** |

---

## 2. Claim-drift register

Every row is a material architecture, capability, count, release, or prompt claim. Line citations are to the document that makes the claim. Code evidence is the current HEAD.

| # | Claim (document:line) | Grade | Current code/artifact evidence | Contradiction / gap | Owner doc to fix |
|---:|---|---|---|---|---|
| D1 | "12 registered tools, 5 permission classes" (`README.md:30`) | **VERIFIED** | 12 `ToolId` variants `tool.rs:115-140`; 12 contracts `:382-456`; 5 classes `:26-66` | Correct | — |
| D2 | `tool.rs` "registers **10** tools" (`README.md:270`) | **STALE** | Same registry has 12 | Count predates `web_fetch` + `vote_evolution` | README §10 |
| D3 | `web_fetch` is "the 11th registry tool" (`README.md:316`) | **READ** (ordinal) | `web_fetch` 11th, `vote_evolution` 12th (`tool.rs:136-139,157-158`) | Ordinal is true but reads as a count of 11; truth report calls it "12th" (`HYPERAGENT-TRUTH-REPORT.md:394`) | README §11, truth report §20 |
| D4 | "full gate is 635 backend / 231 web" (`README.md:326`) | **STALE** | README elsewhere records 637/231 (`README.md:169-170`); bugfix report 637 (`docs/evidence/HMARS-BUGFIX-REPORT.md:389`) | 635 vs 637 | README §11 |
| D5 | "does **not** implement vote submission, SuperAgent spawning for the vote, or automatic application" (`README.md:348-349`) | **STALE** | `vote_evolution` tool `tool.rs:450-455`; `spawn_evolution_voters` `execution.rs:1625-1669`; call `:4676-4704`; routes `http.rs:637-652` | Two of three are implemented; only automatic code application is not | README §12 |
| D6 | Root may spawn four SuperAgents and requires five unanimous approvals (`README.md:359-360`) | **VERIFIED** | `spawn_evolution_voters` + `required_approvals:5` (README JSON) + authorize gate `http.rs:2598` | Correct; contradicts D5 in the same README | README §12 |
| D7 | Effect surface is tier ∩ persona (`README.md:40-42,270-274`) | **VERIFIED with gap** | `provider_executor.rs:191-200` filters tier specs by persona allowlist | Unknown persona returns the **full tier** list (`:193-195`); the "always intersection" claim is false on the fallback path | README §10 + provider_executor |
| D8 | SuperAgent effective persona has no write/shell (`README.md:23`) | **VERIFIED** | `personas.json:23-25` omits `file_write`/`shell_exec`; tier includes them `tool.rs:465-481` | Correct | — |
| D9 | Tier allowlists as listed (`README.md:35-38`) | **VERIFIED** | Hyper 3, Super 12, Worker 11, Subagent 5, ToolAgent 4 (`tool.rs:460-516`) | 35 grants total; truth report says 32 (`HYPERAGENT-TRUTH-REPORT.md:118`) | truth report §3 |
| D10 | Caps depth 3 / fan-out 8 / 64 agents / 64 calls / 40 tool calls / 12 turns / 32 messages / 2 A2A / root 4×2 (`README.md:44-46`) | **VERIFIED** | `caps.rs:9-20`; `tool_loop.rs:14-21`; `execution.rs:142-146,4319-4328` | Correct | — |
| D11 | Truth report: "**10** registered tools, **5** classes, **32** tier grants" (`HYPERAGENT-TRUTH-REPORT.md:118`) | **STALE** | 12 tools / 35 grants (`tool.rs:115-140,460-516`) | Internally contradicted by `:336` "12 tools" | truth report §3 |
| D12 | Truth report: tool surface "12 tools, 5 permission classes" (`HYPERAGENT-TRUTH-REPORT.md:336`) | **VERIFIED** | Registry at HEAD | Correct | — |
| D13 | Truth report: "does not implement four-voter spawning" (`HYPERAGENT-TRUTH-REPORT.md:390`) | **STALE** | Auto-spawn `execution.rs:1625-1669,4676-4704`; live Test AD `AGENTIC-LIVE-TESTS.md:422-437` | Implemented after report text | truth report §19 |
| D14 | Truth report: `ProposalsPanel` "contains no approve/apply button" (`HYPERAGENT-TRUTH-REPORT.md:442`) | **STALE** | Buttons at `web/src/panels/ProposalsPanel.tsx:294-326`; predicates `:104-118`; API routes `http.rs:641-652` | UI now has authorize/apply/rollback | truth report §21 |
| D15 | Truth report: five-seat proposal payload and `applied:false` (`HYPERAGENT-TRUTH-REPORT.md:369-390`) | **VERIFIED** | API sets `"applied":false` and `can_apply:false` `http.rs:2490,2501,2511`; `vote_path` `:2512` | Correct | — |
| D16 | Truth report live rows T25–T30 "VERIFIED (live)" (`HYPERAGENT-TRUTH-REPORT.md:51-56`) | **READ** | Aggregate receipts in `docs/evidence/live/*.json` (run ids, token totals, HTTP codes) | No raw frames/logs committed; cannot be independently reproduced without key | truth report §1 + evidence index |
| D17 | Pseudocode report: registry "11 tools / 5 classes" (`HYPERAGENT-PSEUDOCODE-REPORT.md:264`) | **STALE** | 12/5 at HEAD | Wrong count | pseudocode report §5 |
| D18 | Pseudocode report: four voters/operator auth/apply/rollback "remain absent" (`HYPERAGENT-PSEUDOCODE-REPORT.md:283,304,328`) | **STALE** | All four exist: `execution.rs:1625-1669`, `http.rs:641-652,2598` | Same doc later documents vote/apply (`:332-409`) — internal stale summary | pseudocode report §5 |
| D19 | Pseudocode report: "12 tools / 35 grants" (`HYPERAGENT-PSEUDOCODE-REPORT.md:354`) | **VERIFIED** | `tool.rs:115-140` + grant count 35 | Correct; contradicts `:264` | pseudocode report §7 |
| D20 | Pseudocode report: backend "50 suites / 635" (`HYPERAGENT-PSEUDOCODE-REPORT.md:294,434`) | **STALE** | Current recorded gate 51 suites / 637 (`README.md:169`, truth report `:27`) | Stale intermediate counts | pseudocode report §§5,7 |
| D21 | AGENTIC: "51 suites / 635" (`AGENTIC-LIVE-TESTS.md:387`) | **STALE** | Same doc records 637 at `:454,460,477`; README 637 | Stale intermediate count | live tests §Test Y |
| D22 | AGENTIC: "636 passed" (`AGENTIC-LIVE-TESTS.md:401,409`) | **STALE** | 637 current | Stale intermediate count | live tests §Test AA/AB |
| D23 | AGENTIC Test X uses seeded four voter tasks, not model-spawned (`AGENTIC-LIVE-TESTS.md:367-376`) | **VERIFIED (test-only)** | Route test description; no live claim | Honest; do not cite as live | — |
| D24 | AGENTIC Test AD live auto-voters + five-seat (`AGENTIC-LIVE-TESTS.md:422-437`) | **READ** | `docs/evidence/live/live-tokenlab-auto-voters-five-seat.json` fields: 4 voters, 4 approvals, HTTP 200/200/200 | Aggregate receipt only; no raw frames | live tests §Test AD |
| D25 | AGENTIC Test AE dissent blocks authorize (`AGENTIC-LIVE-TESTS.md:438-447`) | **READ** | `live-tokenlab-voter-dissent.json` (1 approve/1 reject/2 no vote, 409/409) + `http.rs:2598` | Receipt + code agree | live tests §Test AE |
| D26 | Bugfix report header: "Final repair HEAD at report write: `02ef24e`" (`HMARS-BUGFIX-REPORT.md:5-10`) | **STALE** | HEAD `6a41250`; hashes updated in the same header to the 11/10 package | Header mixes old HEAD with new hashes | bugfix report header |
| D27 | Bugfix report: web 227 (`HMARS-BUGFIX-REPORT.md:390`) | **STALE** | Same report `:405` says 231; README 231 | Stale count | bugfix report §13 |
| D28 | Bugfix report: "50 suites, 634 passed" (`HMARS-BUGFIX-REPORT.md:404`) | **STALE** | Same report `:389` says 51/637 | Stale count | bugfix report §14 |
| D29 | Bugfix report: root-control results/branch-gate decisions "not persisted as audit rows" (`HMARS-BUGFIX-REPORT.md:381`) | **STALE** | `root_control` audit rows + `root_gate:` memory documented in same report `:397-399`; code `execution.rs:4347-4368` | Self-contradiction | bugfix report §13 |
| D30 | Release evidence: HEAD `889b779`, 475 backend / 179 web, "no 1.2 artifact", "no end-to-end verification" (`docs/RELEASE-EVIDENCE.md:25-58`) | **STALE** | HEAD `6a41250`; recorded 637/231; `dist/HiveMind.zip` exists (14,416,165 B, hash matches README `:171-172`) | Whole current-tree section is 3 days old | `docs/RELEASE-EVIDENCE.md` |
| D31 | Release evidence: "no 1.2 live paid proof" (`docs/RELEASE-EVIDENCE.md:46`) | **STALE (scope)** | 11/10 TokenLab live runs recorded under `docs/evidence/live/` (e.g. auto-voters, lessons) | No 1.2 **release** proof is true; "no 1.2 live proof" is false for the dev tree | `docs/RELEASE-EVIDENCE.md` |
| D32 | Release notes 1.1.0: model does not drive multi-turn tool calls (`docs/RELEASE.md:39-40`) | **STALE (context)** | Tool loop + approval + live web_fetch/shell/file_write evidence (`README.md:300-313`) | Historically true for 1.1.0, but doc is still presented as current release page | `docs/RELEASE.md` |
| D33 | Release notes: "No 1.2 tag, artifact, manifest..." (`docs/RELEASE.md:60-62`) | **PARTIAL/STALE** | No tag/manifest/release; a 1.2-dev portable package exists | "Artifact" is false if the dev zip counts; needs precise wording | `docs/RELEASE.md` |
| D34 | FINAL-REPORT banner: "Current truth: 1.0.0 artifacts" (`docs/reports/FINAL-REPORT.md:3`) | **STALE** | Sealed authority is 1.1.0 (`tools/release-manifest.json`); tree 1.2.0-dev | Banner predates 1.1.0 seal | FINAL-REPORT banner |
| D35 | CODEMAP "exact" 805 tracked / 36,480 tree (`docs/CODEMAP/00-START-HERE.md:11-17`) | **STALE** | Measured 1,573 tracked / 36,408 tree at HEAD | Frozen at `e021b12`; not labeled stale at HEAD | CODEMAP 00/08/15 |
| D36 | CODEMAP: no chat-history endpoint (`docs/CODEMAP/09-RISKS-GAPS.md:5`) | **STALE** | `/api/history/runs/{run_id}/messages` `http.rs:617` | Route exists | CODEMAP 09 item 1 |
| D37 | CODEMAP: only planner has prompt/DAG assets (`09-RISKS-GAPS.md:6`) | **STALE** | 8 `.prompty` + manifest `assets/prompts/manifest.json` | All 8 shipped | CODEMAP 09 item 2 |
| D38 | CODEMAP: persona loader absent (`09-RISKS-GAPS.md:8`) | **STALE** | `load_personas_embedded` `prompts.rs:458-468`; wired `http.rs:150,283` | Personas load at boot | CODEMAP 09 item 4 |
| D39 | CODEMAP: worker-role tasks refused spawn (`09-RISKS-GAPS.md:10`) | **STALE** | Worker tier grants `spawn_agent` `tool.rs:485`; agent persona includes it `personas.json:72` | Worker can spawn subagents | CODEMAP 09 item 6 |
| D40 | CODEMAP risk: "No 1.2 artifact" (`09-RISKS-GAPS.md:16-17`) | **STALE (artifact)** | Dev package `dist/HiveMind.zip` exists | Dev-vs-release distinction missing | CODEMAP 09 proof gaps |
| D41 | CODEMAP: "Four Library templates" (`17-CLAIM-REGISTER.md:15`) | **PARTIAL** | 8 persona/template assets exist; UI library count not re-verified | "Four" likely refers to library catalog, not prompt assets | CODEMAP 17 C-08 |
| D42 | CAPABILITY-MATRIX: 9 contracts / 30 grants (`docs/audit/CAPABILITY-MATRIX.md:123,375`) | **STALE** | 12 / 35 at HEAD | Snapshot HEAD `919c339` | CAPABILITY-MATRIX |
| D43 | Taxonomy: SuperAgent has 7 tools; `super` gets no persona/9 tools (`AGENT-TAXONOMY-VERIFICATION.md:484,795`) | **STALE** | SuperAgent persona 10 tools `personas.json:25`; tier 12; `super→superagent` alias `provider_executor.rs:165-176` | Persona/alias corrected after snapshot | taxonomy report |
| D44 | Provider truth: Subagent is not a separate tier; `task_agent_tier` maps it to Worker (`docs/provider/AGENT-ARCHITECTURE-TRUTH.md:20`) | **STALE** | `task_agent_tier` maps `subagent` → `AgentTier::Subagent` `execution.rs:284-294`; tier in `tool.rs` | Contradicted | provider truth |
| D45 | Provider truth: per-hive bank not coded; root cannot aggregate child memory (`AGENT-ARCHITECTURE-TRUTH.md:28-31`) | **STALE** | `caller_bank_for_class` maps run/hive/branch `store.rs:261-266`; `read_memory_summary` exists (README `:74,99-102`) | Banks and summary implemented | provider truth |
| D46 | Provider truth summary: "Per-hive memory absent" / "Subagent separate tier NOT CODED" (`AGENT-ARCHITECTURE-TRUTH.md:110-113`) | **STALE** | Same as D44/D45 | Whole table is a 11/07 snapshot | provider truth §7 |
| D47 | IMPROVEMENT-VERIFICATION: PSEUDOCODE-P1/P2/P3 "do not exist" (`docs/audit/IMPROVEMENT-VERIFICATION.md:10`) | **STALE** | `docs/audit/PSEUDOCODE-P1-META.md`, `P2-HMARS.md`, `P3-AGI.md` exist | False at HEAD; may have been true when written | IMPROVEMENT-VERIFICATION §0 |
| D48 | GOVERNANCE-AUDIT-2: daemon admission unaudited; root results discarded; cross-kind duplicate unchecked (`docs/audit/GOVERNANCE-AUDIT-2.md:1-6,17-20`) | **STALE** | Daemon audit test `run_lifecycle.rs:1109`; two-turn test `:1222-1225,1301`; cross-kind guard documented README `:78-80` | Findings fixed after snapshot; doc has no superseded banner | GOVERNANCE-AUDIT-2 header |
| D49 | IMPROVE-LESSONS P1: lesson layer PROPOSED (`docs/audit/IMPROVE-LESSONS.md:79`) | **STALE** | Apply/rollback routes `http.rs:641-652`; hot reload `:2842,2980`; live retrieval `live-tokenlab-applied-lesson-retrieval.json` | Lesson layer implemented | IMPROVE-LESSONS P1 |
| D50 | UI gap audit: settings read-only; `POST /api/provider/key` not implemented (`docs/ui/HIVEMIND-UI-GAP-AUDIT.md:658,667`) | **PARTIAL/STALE** | Settings saves `/api/provider/profile` `SettingsPanel.tsx:165-195`; no key endpoint, key stays DPAPI/env (`dist/HiveMind/README.txt:14-18`) | "read-only" false; key-entry design still absent | UI gap audit §settings |
| D51 | Persona `subagent` has `tier:"worker"` and `send_to_sibling` (`assets/prompts/personas.json:32-36`) | **STALE** | Subagent tier lacks `send_to_sibling` (`tool.rs:498-507`); intersection filters it (`provider_executor.rs:191-200`), but persona record exceeds grant | Config claim exceeds actual grant; validation does not check tool-vs-tier | `personas.json` + persona validation |
| D52 | Prompt planner: "at most 8 children, depth at most 3" (`planner.prompty:17-20`) | **VERIFIED** | `caps.rs:9-14`; plan node cap 8 | Correct | — |
| D53 | Prompt superagent: spawn workers, ask parent, bounded sibling coordination (`superagent.prompty:17`) | **VERIFIED** | `spawn_agent`, `ask_parent`, `send_to_sibling` granted (`tool.rs:465-481`, persona `:25`) | Correct | — |
| D54 | Prompt subagent: ask_parent when blocked, precise result (`subagent.prompty:17`) | **VERIFIED (intent)** | `ask_parent`/report/finish granted (`tool.rs:498-507`) | No behavior test; prompt claim is behavioral, not grant-level | — |
| D55 | "HiveMind learns across runs" FALSE as shipped (`README.md:185`) vs boot lessons/hot reload (`README.md:372-378`) | **READ** | Lesson data is loaded/fenced (`execution.rs:63-75,2525-2528`); no weight/procedure learning | Nuance is explicitly stated; keep "data, not learning" wording | README §7/§13 |
| D56 | Claim register is the claim authority (`docs/claims/CLAIM-REGISTER.csv:1`) | **STALE/incomplete** | 81 rows, last row C-340 dated 2026-11-06 (`:81`); no rows for 12 tools/35 grants, votes/authorize/apply/rollback, lessons retrieval, installed package, or 637/231 | The register predates every 11/10 governance/evolution change | `docs/claims/CLAIM-REGISTER.csv` |
| D57 | UI/UX material, overlay, motion, and system-completion work (`docs/UI-UX-ROADMAP.md:43-62`) | **PROPOSED** | Unchecked `[ ]` roadmap items; no implementation claim in the same doc | Correctly labeled roadmap; do not present as shipped | `docs/UI-UX-ROADMAP.md` |
| D58 | 40-node live scale and 3/3 exact-two stability (`README.md:190-191`; truth report T20/T21 `:46-47`) | **UNVERIFIED** | Caps permit 64 agents (`caps.rs:13`); three-repeat run recorded 2/3 with one provider timeout (`AGENTIC-LIVE-TESTS.md`/truth report §8); no 40-node run | Honest non-claims; keep UNVERIFIED, never "proven" | README §7, truth report §5 |

---

## 3. Deep dives

### 3.1 README: internally inconsistent in four places

- **Tool count:** `README.md:30` says 12; `:270` says 10. Code says 12 (`tool.rs:115-140`).
- **Test count:** `:169-170` says 637/231; `:326` says 635/231.
- **Evolution:** `:348-349` says votes and voter spawning are not implemented; `:359-368` describes both as implemented. Code and live receipts agree with `:359-368`.
- **Stability:** `:191` says 3-repeat exact-two is 2/3; `:311` says exact hierarchy "verified live (7-task exact-two)". The latter is a single run, not 3/3 stability; the table should say "single exact-two run verified; 3-repeat 2/3".

README is otherwise the best current truth document: caps, tier/persona caveat, admission policy, key custody, refusal behavior, and the "not AGI" table match code.

### 3.2 Truth report: six stale statements, one grading inflation

Stale: tool count/grants (`:118`), four-voter absence (`:390`), proposal panel buttons (`:442`), `web_fetch` ordinal (`:394`). Inflated: T25–T30 are labelled "VERIFIED (live)" but only aggregate JSON receipts exist (`docs/evidence/live/`), no raw frames or run logs. The report's own §8 calls them "real Token Lab runs, not transcripts" (`:245`), which is the honest label; §1 should use READ, not VERIFIED.

The report's anti-AGI section, caps, dead-root-branch disclosure (`:89-91`), fixture remote caveat, and 2/3 stability are accurate and should be preserved.

### 3.3 Evolution/votes/UI: the stale "not implemented" cluster

Implemented at HEAD:

- `vote_evolution` tool, SuperAgent-only via tier allowlist `tool.rs:450-455,465-481`; persona includes it `personas.json:25`.
- Four auto-spawned voters with distinct review lenses: `execution.rs:1625-1669` and call sites `:4676-4704`; HEAD commit `b039c3f` added the lens assignment, tested in `run_lifecycle.rs`.
- API: proposals, authorize, apply, rollback routes `http.rs:637-652`; authorizer requires four distinct completed same-run SuperAgent voters (`:2598`); five-seat state and `vote_path` `:2511-2512`.
- Lessons ledger apply/rollback + hot reload `http.rs:2842,2980`; boot injection `http.rs:344`; bounded prompt text `execution.rs:63-75`.
- UI: `ProposalsPanel` authorize/apply/rollback buttons `ProposalsPanel.tsx:294-326`; `LessonsPanel` in `AgentsPanel.tsx:20-21,81-82`; read-only lesson projection.
- Live receipts: `live-tokenlab-auto-voters-five-seat.json` (4 voters, 4 approvals, HTTP 200/200/200, rollback 200), `live-tokenlab-voter-dissent.json` (1 reject blocks with 409), `live-tokenlab-applied-lesson-retrieval.json` (later run quotes the applied lesson).

Still absent (correctly): automatic code application, code/schema/prompt/tool/persona mutation, quality evaluation, and any claim that the lesson changes behavior. `can_apply:false` remains for code (`http.rs:2501`). The proposal-only boundary is intact.

**Fix:** replace the three stale sentences with the above split; do not overclaim "self-evolution" — it is a governed lesson-artifact loop with human authorization.

### 3.4 Tool registry: count/grants drift

At HEAD:

- 12 tool ids: spawn, ask, sibling, report, finish, read, list, write, shell, request_detached, web_fetch, vote_evolution (`tool.rs:115-140`).
- 5 classes: Read, Mutate, Execute, Network, Control (`tool.rs:26-66`).
- 35 grants: Hyper 3, Super 12, Worker 11, Subagent 5, ToolAgent 4 (`tool.rs:460-516`).
- `web_fetch` is 11th, `vote_evolution` 12th (`tool.rs:157-158`).

Wrong counts to fix: README 10 (`:270`), truth report 10/32 (`:118`), pseudocode 11 (`:264`), CAPABILITY-MATRIX 9/30 (`:123,375`), taxonomy 7 (`:484,795`), FINAL-REPORT 7 (`:44`, historical but banner should say so), PHASE-B/FINAL-DELTA 7 (historical snapshots).

The census test derives counts (`tool.rs:597-616`) and should have caught doc drift; it does not read docs. Add a docs-truth check or a generated count section.

### 3.5 Embedded prompts/personas vs tool grants

All 10 assets were read: 8 `.prompty`, `personas.json`, `manifest.json`.

- **Personas:** 8 entries. `planner` is hyper with **empty** allowlist (correct: planner is a one-shot plan call, root control is separate). `superagent` (super) has 10 tools and excludes `file_write`/`shell_exec` (matches README's effective-surface caveat). Worker personas: agent 11, content-writer/data-analyst/code-reviewer 7 (including `web_fetch`), chatbot 4. `subagent` (worker tier) lists 6, including `send_to_sibling`.
- **Mismatch:** the `subagent` persona is named for the Subagent tier but declares `tier:"worker"` and includes `send_to_sibling`, which the Subagent tier does not grant (`tool.rs:498-507`). Effective advertisement is filtered to five tools (`provider_executor.rs:191-200`), so runtime is safe; the persona data is wrong and `Persona::validate` (`prompts.rs:411-437`) does not check allowed tools against the persona tier.
- **Fallback widening:** if `task.role` resolves to no persona key, `tools_for_task` returns the full tier list (`provider_executor.rs:193-195`). The docs state an unconditional tier ∩ persona rule. Production roles are constrained (`task_agent_tier`, `execution.rs:284-294`), so this is likely a dead path, but the claim and the code should agree: either refuse unknown personas or document the fallback.
- **Prompt claims:** planner's "at most 8 children/depth 3" matches caps; superagent's spawn/ask/sibling instructions match grants; subagent's ask_parent/report instructions match grants. No prompt instructs a tool the effective persona lacks, except the subagent persona's `send_to_sibling` data row.
- **Model pin in prompty front-matter** is `deepseek-flash` for all 8. Runtime provider selection is settings-driven (`README.md:108-118`), so this appears to be template metadata, not the live model selector; not verified in the executor. Do not claim these pins drive TokenLab runs.

### 3.6 Release and installed-build truth

**Verified artifact facts (observed by `Get-FileHash`, read-only):**

| Artifact | Size | SHA-256 | Match |
|---|---:|---|---|
| `dist/HiveMind.zip` | 14,416,165 B | `15BF4E61C05AEEFC8ABACB341283568CA3705FB5C6E226510CD33B88D6CA5ADA` | `README.md:171-172` ✅ |
| `dist/HiveMind/hiverouter.exe` | 36,540,416 B | `F369D7AC4CC523EF72FE7DD88DE0191A3C7D0BBB852FF743E542AEFA6A7CF871` | `README.md:173-174` ✅ |
| `target/release/hiverouter.exe` | 36,540,416 B | `F369D7AC...A7CF871` | same ✅ |
| `dist/HiveMind.zip` members | 3 | `HiveMind.exe`, `hiverouter.exe`, `README.txt` | — |
| `dist/HiveMind-1.1.0-win64.zip` | 8,613,122 B | `af2345f6...63e56` | manifest ✅ (sealed 1.1) |

**Problems:**

1. The current package is a **dev artifact** (36.5 MB server vs 13.6 MB for 1.1.0) and is not labeled as such anywhere in the package. Its `README.txt` says "HiveMind portable desktop build" without version or commit (`dist/HiveMind/README.txt:1-3`).
2. The package's key-rotation instruction points at `tools\set-provider-key.ps1` (`README.txt:18-19`), which is not shipped. A downloader cannot perform the documented key setup from the package.
3. The package omits `hiverouter-cli.exe`, `CHECKSUMS.txt`, `RELEASE`/`RUNBOOK` docs that the 1.1.0 package shipped (`dist/HiveMind-1.1.0-win64/RUNBOOK.md`). There is no checksum sidecar in the zip itself.
4. The launcher still hard-codes a dev workspace root (`tools/desktop-launcher/HiveMindLauncher.cs:17,244-264`). The package README does not disclose what workspace the agents will read/write, which is a material operator truth.
5. `docs/RELEASE-EVIDENCE.md` denies the existence of the 1.2 artifact (`:38`) and reports old gates (`:40,43`), while `README.md` records the package and current gates.
6. `docs/RELEASE.md` install steps (`:50-55`) are source-binary steps, not portable-package steps; no "Download / Install" doc exists.

**Required Downloads page/section:** version + commit + build date, exact zip name/size/SHA-256, contents list, prerequisites (Windows 10/11 x64, WebView2), first-run, DPAPI key script location or in-package replacement, data dir (`%LOCALAPPDATA%\HiveMind`), workspace root semantics, CLI availability, SmartScreen note, uninstall/rollback, and the dev-vs-sealed label.

### 3.7 Audits/plans: snapshots presented as current

- `docs/audit/CAPABILITY-MATRIX.md` is pinned to `919c339` and says 9 tools/30 grants; it is a valid snapshot but has no "superseded" banner and is titled "code-derived capability matrix".
- `docs/audit/AGENT-TAXONOMY-VERIFICATION.md` is pinned to `919c339`; its SuperAgent/persona findings are fixed by the alias + persona changes (`provider_executor.rs:165-176`).
- `GOVERNANCE-AUDIT-2.md` is pinned to `9f8e2e0`; its top findings 1–3 (daemon audit, result feedback, cross-kind names) were subsequently fixed. README nonetheless cites it as current governance evidence (`README.md:206`).
- `IMPROVEMENT-VERIFICATION.md` asserts the PSEUDOCODE sources are absent (`:10`); they exist. Its C6 says no focused tier∩persona test exists; `provider_adapter.rs:123` is that test.
- `IMPROVE-LESSONS.md` says the lesson layer is PROPOSED (`:79`); apply/rollback/reload exist.
- `docs/RELEASE-EVIDENCE.md`, `plans/foresight/PROOF-RELEASE-PLAN.md:16,22`, and `plans/foresight/COMPILED-LEDGER.md:308` were written against earlier HEADs (`889b779`, `37f0e7c`) and now conflict with the current tree ("no committed tool loop", "no 1.2 artifact"). They are plan-time documents and should carry explicit "plan-time; re-validate at HEAD" banners.
- `docs/audit/PHASE-B-VERIFICATION.md` (HEAD `ab3a33b`) and `FINAL-DELTA-VERIFICATION.md` (2026-11-06) are historical verifier receipts; they still show 7-tool census and D-2 `E_INTERNAL_ERROR`. No current banner.
- Root `audit/**` is an exhaustive 507-tracked-file audit; today 1,573 files are tracked. It is historical, not current, and README does not say so.

### 3.8 UI, design, and research docs

- `docs/UI-UX-ROADMAP.md` is honest: implemented splash checked, material/motion/overlay items unchecked and labeled roadmap (`:43-62`). Keep.
- `docs/ui/HIVEMIND-HONEYGLASS-DESIGN-SYSTEM.md:3` says "implemented foundation" and its token provenance is consistent with `web/src/tokens`; no false capability claim observed.
- `docs/ui/HIVEMIND-UI-GAP-AUDIT.md` is a 2026-11-08 gap analysis. Its settings section (`:658,667`) is stale: settings now saves a non-secret provider profile (`SettingsPanel.tsx:165-195`) and the key remains DPAPI/env. It does not implement `POST /api/provider/key`; that design item remains absent, so the fix is to reword, not delete.
- Research docs (`docs/research/**`) are design/research inputs. They must never be cited as implemented; the documents themselves often say so (e.g. `hmars/19-wave3-gap-audit.md` is a snapshot).
- `docs/provider/AGENT-ARCHITECTURE-TRUTH.md` is the worst stale "truth" doc: its central table (subagent tier, memory banks, root aggregate, memory isolation) is contradicted by `execution.rs:284-294`, `store.rs:261-266`, and `read_memory_summary`.
- `docs/trace/QA-HARNESS.md:3-8` trace counts are current: recomputed from `docs/trace/controls.json` = 1 WIRED-AND-WORKS / 264 PENDING / 5 MISSING / 49 UNVERIFIED of 319. This is a good example of a doc that labels snapshots correctly at the top.

### 3.9 Evidence-grade audit (no mocks counted as live)

- **Live receipts** (`docs/evidence/live/*.json`) are aggregate observations: run id, provider, usage, counts, HTTP status. They do not include prompts, frames, or raw HTTP. Grade READ, not VERIFIED.
- **Seeded tests** (`AGENTIC-LIVE-TESTS.md:367-376`) explicitly use four seeded SuperAgent tasks. They prove route validation, not the auto-spawn live path. Do not merge them with Test AD.
- **Dev doubles:** `wiremock` is dev-only; `web/src/api/mock.ts` is behind a DEV flag. No product path returns fabricated data in the docs reviewed. `README.md` and `docs/evidence/HMARS-BUGFIX-REPORT.md:273-274` state this; the stub sweep evidence in `docs/audit/PHASE-B-VERIFICATION.md:24` is a 2026-11-06 snapshot, not a current sweep.
- **No mock/demo claim found that presents a mock as production.** The risk is the opposite: aggregate receipts being cited as "VERIFIED (live)".

---

## 4. Top-10 documentation/truth bugs

Severity: **P0** = an operator or release decision would be wrong today; **P1** = material capability/release misstatement; **P2** = count/consistency drift.

| # | Sev | Bug | Exact location(s) | Exact fix |
|---:|---|---|---|---|
| 1 | **P0** | Evolution/votes/UI falsely described as not implemented | `README.md:348-349`; `docs/audit/HYPERAGENT-TRUTH-REPORT.md:390,442`; `docs/audit/HYPERAGENT-PSEUDOCODE-REPORT.md:283,304,328` | Replace with: vote submission, auto-spawn of 4 voters, operator authorization, lesson apply/rollback implemented; code application/evaluation not implemented. Delete "no approve/apply button". |
| 2 | **P0** | Tool count/grants contradictory across current-truth docs | `README.md:270`; `HYPERAGENT-TRUTH-REPORT.md:118`; `HYPERAGENT-PSEUDOCODE-REPORT.md:264`; `CAPABILITY-MATRIX.md:123,375`; `AGENT-TAXONOMY-VERIFICATION.md:484,795` | Set all to 12 tools / 5 classes / 35 grants (or mark the audit snapshot "at 919c339: 9/30"). |
| 3 | **P0** | Release evidence describes a three-day-old tree and denies the current package | `docs/RELEASE-EVIDENCE.md:25-58` (esp. `:36,38,40,43,46,50-53`) | Replace current-tree section with HEAD `6a41250`, 637/231 recorded, `dist/HiveMind.zip` + hash/size, live 1.2-dev receipts, and keep sealed 1.1 separately. |
| 4 | **P1** | Installed-build/Downloads docs broken and incomplete | `dist/HiveMind/README.txt:1-24` (esp. `:18-19`); `README.md` §6/§9; package contents | Add Downloads/Install section; ship `hiverouter-cli.exe`, `CHECKSUMS.txt`, `RUNBOOK.md` or remove references; document workspace root, first-run, key rotation, uninstall, dev-vs-release label. |
| 5 | **P1** | Persona claims exceed tier grant | `backend/crates/hiverouter-core/assets/prompts/personas.json:32-36`; validation `prompts.rs:411-437` | Remove `send_to_sibling` from subagent persona (or add to Subagent tier only if intended); set `tier:"subagent"` or document that field is metadata; add boot validation that persona tools ⊆ tier. |
| 6 | **P1** | Provider "truth" doc contradicts current architecture | `docs/provider/AGENT-ARCHITECTURE-TRUTH.md:20,28-31,60-67,89-92,110-113` | Rewrite as a dated snapshot with current tier/bank/root-summary facts, or banner it "2026-11-07 snapshot — superseded". |
| 7 | **P1** | CODEMAP census/risks are stale at HEAD | `docs/CODEMAP/00-START-HERE.md:11-17`; `08-FILE-CATALOG.md`; `09-RISKS-GAPS.md:5-12,16-17`; `10-VERIFICATION.md:35` | Add HEAD banner + measured counts (1,573/36,408), regenerate or mark frozen at `e021b12`; strike fixed risks. |
| 8 | **P1** | Audit snapshots cited without supersession | `GOVERNANCE-AUDIT-2.md:1-6`; `IMPROVEMENT-VERIFICATION.md:10`; `IMPROVE-LESSONS.md:79`; `CAPABILITY-MATRIX.md:1-12`; `AGENT-TAXONOMY-VERIFICATION.md:1-20`; `PHASE-B-VERIFICATION.md:1-7`; `FINAL-DELTA-VERIFICATION.md:1-10` | Add a top banner: pinned HEAD, findings fixed/current status, and "not current truth". Correct the PSEUDOCODE-absent claim. |
| 9 | **P2** | Bugfix report self-contradictory and HEAD-stale | `docs/evidence/HMARS-BUGFIX-REPORT.md:5-10,381,390,404` | Update HEAD to `6a41250`; 637/231; remove "root decisions not audit rows"; keep live-run caveats. |
| 10 | **P2** | README count/consistency drift | `README.md:270,316,326` (vs `:30,169-170,191,311`) | 12 tools, remove "11th" as count implication, 637 backend; label exact hierarchy as single run vs 2/3 stability. |

Adjacent fixes that should ride with the above: `docs/ui/HIVEMIND-UI-GAP-AUDIT.md:658,667` settings wording; `docs/claims/CLAIM-REGISTER.csv` missing 11/10 rows; `docs/reports/FINAL-REPORT.md:3` banner.

---

## 5. Required update checklist

### 5.1 Truth report (`docs/audit/HYPERAGENT-TRUTH-REPORT.md`)

- [ ] §1: downgrade T25–T30 from "VERIFIED (live)" to "READ (live receipt)" and link each receipt; state no raw frames are committed.
- [ ] §3: 12 tools / 5 classes / **35** grants; correct `web_fetch` ordinal (11th; `vote_evolution` 12th).
- [ ] §19: replace "does not implement four-voter spawning" with implemented status + remaining non-goals (code application, quality evaluation).
- [ ] §21: delete "contains no approve/apply button"; state authorize/apply/rollback buttons and `LessonsPanel`.
- [ ] Add a "Current tree snapshot" header: HEAD `6a41250`, 637/231 (recorded), `dist/HiveMind.zip` hash/size, date 2026-11-10.
- [ ] Add T31/T32 for installed package and lessons across runs (READ-grade unless re-run).

### 5.2 README (`README.md`)

- [ ] §6: update gate line 326 to 637; add package size and contents; state dev package not sealed.
- [ ] §10: `tool.rs` registers **12** tools; keep the tier list; note `vote_evolution` is 12th.
- [ ] §11: `web_fetch` is the 11th of 12 tools; remove the 635 line.
- [ ] §12/§13: replace the stale "does not implement vote submission…" sentence; keep "scope is lesson artifact only".
- [ ] §5/§6: add a **Downloads / Install** subsection (see 5.3).
- [ ] §9: explicitly say "dev portable package built at 2026-11-10; not tagged or published".
- [ ] Correct the stability wording: single exact-two run verified; 3-repeat 2/3.

### 5.3 Downloads / installed-build docs (new `docs/DOWNLOADS.md` or README §6.1)

- [ ] Artifact: `HiveMind.zip`, 14,416,165 B, SHA-256 `15BF4E61…6CA5ADA`, built 2026-11-10, source HEAD `6a41250`, version `1.2.0-dev` (dev, not sealed).
- [ ] Contents: `HiveMind.exe`, `hiverouter.exe`, `README.txt`; state that CLI/CHECKSUMS/RUNBOOK are absent (and either ship them or say so).
- [ ] First run: double-click `HiveMind.exe`; loopback port 8099; data `%LOCALAPPDATA%\HiveMind`; WebView2 dependency.
- [ ] Key: DPAPI `provider.key`; rotation script is **not in the package** — document the repo path or ship `set-provider-key.ps1`; never print key.
- [ ] Workspace: launcher default root is the dev checkout when present; fallback is the executable directory; on a clean machine state exactly what agents can read/write.
- [ ] Integrity: `Get-FileHash` command + expected hash; SmartScreen note.
- [ ] Uninstall/rollback: close app (stops owned server), remove `%LOCALAPPDATA%\HiveMind`, keep workspace files; no registry/service changes.
- [ ] Sealed vs dev: link `docs/RELEASE.md` for 1.1.0 only; no 1.2 release claim.

### 5.4 Other docs

- [ ] `docs/RELEASE-EVIDENCE.md`: rewrite current-tree section; mark plan-time rows; keep sealed 1.1.0 record.
- [ ] `docs/RELEASE.md`: add current-tree note; fix "no 1.2 artifact"; add package install pointer.
- [ ] `docs/claims/CLAIM-REGISTER.csv`: add rows for 12 tools/35 grants, vote/authorize/apply/rollback, lessons retrieval, `dist/HiveMind.zip` hash, 637/231; mark C-026/C-340-era rows historical.
- [ ] `docs/audit/AGENTIC-LIVE-TESTS.md`: 635/636 → 637; add a receipt-grade legend.
- [ ] `docs/evidence/HMARS-BUGFIX-REPORT.md`: HEAD/gates/root-audit-row corrections.
- [ ] `docs/audit/CAPABILITY-MATRIX.md`, `AGENT-TAXONOMY-VERIFICATION.md`, `GOVERNANCE-AUDIT-2.md`, `IMPROVEMENT-VERIFICATION.md`, `IMPROVE-LESSONS.md`, `PHASE-B-VERIFICATION.md`, `FINAL-DELTA-VERIFICATION.md`: top supersession banners.
- [ ] `docs/CODEMAP/00,08,09,10,15`: HEAD banner + measured counts; fix fixed-risk rows.
- [ ] `docs/provider/AGENT-ARCHITECTURE-TRUTH.md`: rewrite or banner.
- [ ] `docs/ui/HIVEMIND-UI-GAP-AUDIT.md`: settings wording; distinguish profile save from key entry.
- [ ] `personas.json`: fix subagent persona; add `validate_tools_subset_of_tier` (or document metadata).
- [ ] `docs/reports/FINAL-REPORT.md:3`: banner sealed 1.1.0, not 1.0.0.
- [ ] `tools/release-manifest.json` status_note: clarify "no sealed 1.2 release; dev package exists at dist/HiveMind.zip".

---

## 6. Lane ownership and coverage

Lane codes used for the path index: **L1** contract/core, **L2** provider, **L3** tools/safety, **L4** execution/governance/plans, **L5** web/UI, **L6** e2e/release/tooling, **L7** docs/claims/audit/evidence/prompts.

| Required area | Coverage | Owner | State |
|---|---|---|---|
| `README.md` | Full | L7 | Current with 5 internal drifts |
| `docs/audit/**` | 24 indexed; 10 deep | L7 | Current truth + stale snapshots |
| `docs/plans/**` | 28 indexed; phase evidence spot-read | L7 (authored by L4) | Historical provenance |
| `docs/evidence/**` | 4 md + 26 live receipts + 12 PNG | L7 | READ-grade receipts; bugfix header stale |
| `docs/ui/**` | 9 indexed; 3 deep | L5 | Honest roadmap; gap audit partly stale |
| `docs/research/**` | 38 indexed; key sections | L7 | Research only; never cite as implemented |
| `docs/contracts/**` | 2 indexed | L1 | Frozen 1.0-era contract |
| `docs/claims/**` | Register 81 rows read | L7 | Stale/incomplete (no 11/10 rows) |
| `docs/CODEMAP/**` | 22 indexed; 4 deep | L7 | Frozen at `e021b12`; stale at HEAD |
| `docs/diagrams/**` | binary exports only | L7 | No markdown truth claims |
| `docs/provider/**` | 2 read/spot | L2 | Stale truth doc |
| `docs/trace/**` | 49 indexed; QA harness + controls | L6 | Trace counts current |
| `docs/reports/**` | FINAL-REPORT read | L7 | Historical banner stale |
| `ledgers/**` | watchdog read | L5 | 2026-11-06 PASS; valid for that snapshot |
| `plans/**` | 114 indexed; foresight 2 deep | L4 | 1.2 plan-time; stale vs HEAD |
| `audit/**` | 98 indexed; census headers | L7 | Historical 507-file audit |
| `runtime-test/**` | 82 indexed; pkg docs read | L6 | Historical 1.0/1.1 smokes |
| Root markdown | 4 read + GitNexus templates | L7 | `CHANGE-LEDGER-IMPL` current through CL-I-009 |
| Prompt assets | 10/10 read | L4/L3 | One persona exceeds tier |

---

## 7. What is verified vs not (no overclaim)

**Verified by static read at HEAD:** tool registry/counts/grants, tier/persona intersection behavior, caps, evolution routes and auto-voter code, lessons prompt bounds and reload, proposal panel buttons, persona contents, trace counts, artifact hashes (Get-FileHash), package contents (zip listing), version/tags/HEAD.

**READ, not verified by execution:** 637 backend / 231 web gate counts; live TokenLab runs and their token totals; 1.1.0 sealed hashes (not re-hashed this pass); UI screenshot claims; FigCraft/watcom PASS ledgers; all plan-time performance claims.

**UNVERIFIED/absent:** 40-node live scale, 3/3 exact-two stability, network remote transport, daemon live tick, cross-run weight/procedure learning, code self-modification, automatic lesson application, quality evaluation, no-raw-frame live claims.

**No mock/demo evidence was counted as live.** Seeded route tests and wiremock are explicitly test-only; the live JSON files are aggregate receipts and are graded READ.

---

## Appendix A — complete documentation path index (378 markdown files)

Each line is `<lane>  <path>`. Ownership is by functional lane; L7 indexes all documentation regardless of author.

L7  .claude\skills\gitnexus-cli\SKILL.md
L7  .claude\skills\gitnexus-debugging\SKILL.md
L7  .claude\skills\gitnexus-exploring\SKILL.md
L7  .claude\skills\gitnexus-guide\SKILL.md
L7  .claude\skills\gitnexus-impact-analysis\SKILL.md
L7  .claude\skills\gitnexus-refactoring\SKILL.md
L7  AGENTS.md
L7  audit\APPENDIX-FILE-CENSUS.md
L7  audit\APPENDIX-IGNORED-DELIVERABLES.md
L7  audit\FINDINGS-REGISTER.md
L7  audit\HIVEMIND-AUDIT-PAPER.md
L7  audit\inventory\CODE-STATS.md
L7  audit\inventory\DEEP-LIST.md
L7  audit\inventory\DIRECTORY-CENSUS.md
L7  audit\parts\b-core\DEPENDENCIES.md
L7  audit\parts\b-core\EVIDENCE.md
L7  audit\parts\b-core\FILES.md
L7  audit\parts\b-core\FINDINGS.md
L7  audit\parts\b-provider\DEPENDENCIES.md
L7  audit\parts\b-provider\EVIDENCE.md
L7  audit\parts\b-provider\FILES.md
L7  audit\parts\b-provider\FINDINGS.md
L7  audit\parts\figcraft-a\DEPENDENCIES.md
L7  audit\parts\figcraft-a\EVIDENCE.md
L7  audit\parts\figcraft-a\FILES.md
L7  audit\parts\figcraft-a\FINDINGS.md
L7  audit\parts\figcraft-b\DEPENDENCIES.md
L7  audit\parts\figcraft-b\EVIDENCE.md
L7  audit\parts\figcraft-b\FILES.md
L7  audit\parts\figcraft-b\FINDINGS.md
L7  audit\parts\strip-watchdog\EVIDENCE.md
L7  audit\parts\strip-watchdog\VERIFICATION.md
L7  audit\parts\ui-watchdog\DEPENDENCIES.md
L7  audit\parts\ui-watchdog\EVIDENCE.md
L7  audit\parts\ui-watchdog\FILES.md
L7  audit\parts\ui-watchdog\FINDINGS.md
L7  audit\parts\ui-watchdog\PROVENANCE.md
L7  audit\parts\watchdog-1\DEPENDENCIES.md
L7  audit\parts\watchdog-1\EVIDENCE.md
L7  audit\parts\watchdog-1\FILES.md
L7  audit\parts\watchdog-1\FINDINGS.md
L7  audit\parts\watchdog-2\DEPENDENCIES.md
L7  audit\parts\watchdog-2\EVIDENCE.md
L7  audit\parts\watchdog-2\FILES.md
L7  audit\parts\watchdog-2\FINDINGS.md
L7  audit\QUALITY-CODE-STATS.md
L7  audit\ROADMAP.md
L7  audit\UNSLOP-REPORT.md
L7  audit\VERIFICATION.md
L7  audit\WAVE-SHEET.md
L7  CHANGE-LEDGER-IMPL.md
L7  CLAUDE.md
L6  dist\HiveMind-1.1.0-win64\README.md
L6  dist\HiveMind-1.1.0-win64\RELEASE.md
L6  dist\HiveMind-1.1.0-win64\RUNBOOK.md
L6  dist\HiveMind-v1-win64\README.md
L6  dist\HiveMind-v1-win64\RUNBOOK.md
L6  docs\AGENT-CLI.md
L7  docs\audit\AGENTIC-LIVE-TESTS.md
L7  docs\audit\AGENTIC-TEST-PLAN.md
L7  docs\audit\AGENT-TAXONOMY-VERIFICATION.md
L7  docs\audit\BASELINE-VERIFICATION.md
L7  docs\audit\CAPABILITY-MATRIX.md
L7  docs\audit\evidence\final\pw-out\ws-contract-WS-transport-c-9a060-refusal-has-code-how-to-fix-chrome\error-context.md
L7  docs\audit\FINAL-CLOSEOUT.md
L7  docs\audit\FINAL-DELTA-VERIFICATION.md
L7  docs\audit\GOVERNANCE-AUDIT-2.md
L7  docs\audit\GOVERNANCE-TESTS.md
L7  docs\audit\hivemind-memory-a2a\AUDIT.md
L7  docs\audit\hivemind-memory-a2a\README.md
L7  docs\audit\hivemind-memory-a2a\TEST-PLAN.md
L7  docs\audit\HYPERAGENT-PSEUDOCODE-REPORT.md
L7  docs\audit\HYPERAGENT-TRUTH-REPORT.md
L7  docs\audit\IMPROVE-DEEPDESLOP.md
L7  docs\audit\IMPROVE-EVIDENCE-GAP.md
L7  docs\audit\IMPROVE-LESSONS.md
L7  docs\audit\IMPROVEMENT-VERIFICATION.md
L7  docs\audit\IMPROVE-RESEARCH.md
L7  docs\audit\IMPROVE-SCIENTIFIC.md
L7  docs\audit\IMPROVE-TRUTHSEEKER.md
L7  docs\audit\PHASE-B-VERIFICATION.md
L7  docs\audit\PSEUDOCODE-P1-META.md
L7  docs\audit\PSEUDOCODE-P2-HMARS.md
L7  docs\audit\PSEUDOCODE-P3-AGI.md
L7  docs\audit\QA-ENV.md
L7  docs\audit\REMOTE-PERMISSION-GAPS.md
L7  docs\claims\SKILL-USE.md
L7  docs\CODEMAP\_incoming\DIAGRAM-ANALYSIS.md
L7  docs\CODEMAP\_incoming\DUMP-ANALYSIS.md
L7  docs\CODEMAP\_incoming\DUMP-DELTA.md
L7  docs\CODEMAP\00-START-HERE.md
L7  docs\CODEMAP\01-SCOPE-AND-CENSUS.md
L7  docs\CODEMAP\02-ARCHITECTURE.md
L7  docs\CODEMAP\03-BACKEND.md
L7  docs\CODEMAP\04-WEB.md
L7  docs\CODEMAP\05-E2E-TOOLS.md
L7  docs\CODEMAP\06-DOCS-PLANS-AUDIT.md
L7  docs\CODEMAP\07-RUNTIME-BINARIES.md
L7  docs\CODEMAP\08-FILE-CATALOG.md
L7  docs\CODEMAP\09-RISKS-GAPS.md
L7  docs\CODEMAP\10-VERIFICATION.md
L7  docs\CODEMAP\11-AUDITOR-FINDINGS.md
L7  docs\CODEMAP\12-GITHUB-PUSH.md
L7  docs\CODEMAP\13-HISTORICAL-AUDIT.md
L7  docs\CODEMAP\14-GITDIAGRAM-ARCHITECTURE.md
L7  docs\CODEMAP\15-INPUT-PROVENANCE.md
L7  docs\CODEMAP\16-GITNEXUS-FINDINGS.md
L7  docs\CODEMAP\17-CLAIM-REGISTER.md
L7  docs\CODEMAP\18-HMAS-RESEARCH.md
L1  docs\contracts\ADAPTERS.md
L1  docs\contracts\UI-COMMANDS-v1.md
L7  docs\evidence\HMARS-BUGFIX-REPORT.md
L7  docs\evidence\key-validation.md
L7  docs\evidence\live-provider-smoke.md
L7  docs\evidence\live-trace-1.1.md
L6  docs\EXTERNAL-DRIVERS.md
L7  docs\plans\AGENT-WAVE-REPORTS.md
L7  docs\plans\D2-D3-WIRING-EVIDENCE.md
L7  docs\plans\FRONTEND-STATUS.md
L7  docs\plans\HASH-VERIFICATION.md
L7  docs\plans\HIVEMIND-MASTER-FINISH-PLAN.md
L7  docs\plans\PHASE-0-RECON.md
L7  docs\plans\PHASE-11-EVIDENCE.md
L7  docs\plans\PHASE-11-RECON.md
L7  docs\plans\PHASE-12-EVIDENCE.md
L7  docs\plans\PHASE-12-RECON.md
L7  docs\plans\PHASE-1-EVIDENCE.md
L7  docs\plans\PHASE-1-RECON.md
L7  docs\plans\PHASE-2-EVIDENCE.md
L7  docs\plans\PHASE-2-RECON.md
L7  docs\plans\PHASE-3-EVIDENCE.md
L7  docs\plans\PHASE-3-RECON.md
L7  docs\plans\PHASE-4-EVIDENCE.md
L7  docs\plans\PHASE-4-RECON.md
L7  docs\plans\PHASE-5-EVIDENCE.md
L7  docs\plans\PHASE-5-RECON.md
L7  docs\plans\PHASE-6-EVIDENCE.md
L7  docs\plans\PHASE-6-RECON.md
L7  docs\plans\PHASE-7-8-10-EVIDENCE.md
L7  docs\plans\PHASE-7-8-10-RECON.md
L7  docs\plans\PHASE-9-EVIDENCE.md
L7  docs\plans\PHASE-9-RECON.md
L7  docs\plans\SESSION-PLAN.md
L7  docs\plans\TOOLCHAIN.md
L2  docs\provider\AGENT-ARCHITECTURE-TRUTH.md
L2  docs\provider\DEEPSEEK-INTEGRATION.md
L6  docs\RELEASE.md
L6  docs\RELEASE-EVIDENCE.md
L7  docs\reports\FINAL-REPORT.md
L7  docs\research\agent-definitions\DRAFT.md
L7  docs\research\agent-definitions\VERIFIED.md
L7  docs\research\hmars\01-deepseek-api.md
L7  docs\research\hmars\02-hmars-patterns.md
L7  docs\research\hmars\03-scale-concurrency.md
L7  docs\research\hmars\04-memory-architecture.md
L7  docs\research\hmars\05-ui-ux-patterns.md
L7  docs\research\hmars\06-security-cost.md
L7  docs\research\hmars\07-code-backend.md
L7  docs\research\hmars\08-code-ui.md
L7  docs\research\hmars\09-synthesis-and-goal.md
L7  docs\research\hmars\10-gap-meta.md
L7  docs\research\hmars\11-donor-frameworks.md
L7  docs\research\hmars\12-role-capability-model.md
L7  docs\research\hmars\13-memory-venn.md
L7  docs\research\hmars\14-a2a-bus-roster.md
L7  docs\research\hmars\15-ui-ux-goal.md
L7  docs\research\hmars\16-deepseek-cache-cost.md
L7  docs\research\hmars\17-scale-security-impl.md
L7  docs\research\hmars\18-open-source-agents.md
L7  docs\research\hmars\19-wave3-gap-audit.md
L7  docs\research\hmars\20-implementation-architecture.md
L7  docs\research\hmars\21-roles-daemons-implementation.md
L7  docs\research\hmars\22-memory-bank-implementation.md
L7  docs\research\hmars\23-a2a-implementation.md
L7  docs\research\hmars\24-ui-implementation.md
L7  docs\research\hmars\25-provider-cache-implementation.md
L7  docs\research\hmars\26-scale-security-audit.md
L7  docs\research\hmars\27-open-source-daemon-catalog.md
L7  docs\research\hmars\OPERATOR-DECISIONS.md
L7  docs\research\remote-agents\00-README.md
L7  docs\research\remote-agents\01-remote-agent-model.md
L7  docs\research\remote-agents\02-memory-isolation.md
L7  docs\research\remote-agents\03-device-capabilities.md
L7  docs\research\remote-agents\04-contracts-api-ws-refusals.md
L7  docs\research\remote-agents\05-ui-representation.md
L7  docs\research\remote-agents\06-phased-plan.md
L7  docs\research\remote-agents\SOURCES.md
L6  docs\trace\A11Y-CONTRAST-PLAN.md
L6  docs\trace\agent-cli\api-map.md
L6  docs\trace\agent-cli\cli-bugfix-audit.md
L6  docs\trace\agent-cli\cli-design.md
L6  docs\trace\agent-cli\lifecycle-map.md
L6  docs\trace\CAPTURE-REGISTRY.md
L6  docs\trace\live\README.md
L6  docs\trace\live\ROUND8-LIVE.md
L6  docs\trace\orca-wave\01-trace.md
L6  docs\trace\orca-wave\02-weaver.md
L6  docs\trace\orca-wave\03-forge.md
L6  docs\trace\orca-wave\04-capability.md
L6  docs\trace\orca-wave\05-driver.md
L6  docs\trace\orca-wave\06-logic.md
L6  docs\trace\provider\01-deepseek-capability-matrix.md
L6  docs\trace\provider\02-agent-runtime-wiring.md
L6  docs\trace\provider\03-ui-actions.md
L6  docs\trace\provider\04-audit.md
L6  docs\trace\provider\05-figcraft-ux-gaps.md
L6  docs\trace\QA-HARNESS.md
L6  docs\trace\round9\findings-register.md
L6  docs\trace\round9\lead-plan.md
L6  docs\trace\round9\recon-backend.md
L6  docs\trace\round9\recon-ui.md
L6  docs\trace\round9\skill-frontend.md
L6  docs\trace\round9\skill-modernize.md
L6  docs\trace\round9\skill-unslop.md
L6  docs\trace\round9\truthseeker-precode.md
L6  docs\trace\rounds\round20-bug-sweep.md
L6  docs\trace\rounds\round24-bug-sweep.md
L6  docs\trace\rounds\round25-feature.md
L6  docs\trace\rounds\round26-bug-sweep.md
L6  docs\trace\rounds\round28-bug-sweep.md
L6  docs\trace\rounds\round30-bug-sweep.md
L6  docs\trace\rounds\round32-bug-sweep.md
L6  docs\trace\rounds\round34-bug-sweep.md
L6  docs\trace\rounds\round36-bug-sweep.md
L6  docs\trace\rounds\round38-bug-sweep.md
L6  docs\trace\rounds\round40-bug-sweep.md
L6  docs\trace\terminal-truth.md
L6  docs\trace\TRACE-SCHEMA.md
L6  docs\trace\ui-rebuild\01-shell-parity.md
L6  docs\trace\ui-rebuild\02-card-parity.md
L6  docs\trace\ui-rebuild\04-hmas-surface-parity.md
L6  docs\trace\ui-rebuild\05-visual-qa.md
L6  docs\trace\ui-rebuild\07-ui-stack.md
L6  docs\trace\ui-rebuild\09-live-visual-qa.md
L6  docs\trace\ui-rebuild\10-representation-audit.md
L6  docs\trace\ui-rebuild\11-control-wiring.md
L5  docs\ui\figcraft\BRIEF-A.md
L5  docs\ui\figcraft\BRIEF-B.md
L5  docs\ui\figcraft\C\SUPERSEDED.md
L5  docs\ui\figcraft\HiveMind\README.md
L5  docs\ui\figcraft\HiveMind\source-evidence\FIGCRAFT-COMMAND-ANALYSIS.md
L5  docs\ui\figcraft\HiveMind\source-evidence\HiveRouter-Style-Kit.md
L5  docs\ui\figcraft\HiveMind\source-evidence\SPEC\HIVEROUTER-TARGET-SPEC.md
L5  docs\ui\figcraft\HiveMind\SPEC\HIVEMIND-TARGET-SPEC.md
L5  docs\ui\figcraft\WATCHDOG-BRIEF.md
L5  docs\ui\HIVEMIND-HONEYGLASS-DESIGN-SYSTEM.md
L5  docs\ui\HIVEMIND-UI-GAP-AUDIT.md
L5  docs\UI-UX-ROADMAP.md
L6  e2e\GATE-EXCLUSIONS.md
L6  e2e\README.md
L6  ledgers\ui-watchdog.md
L4  plans\1.1-WAVE-1-RECOVERY.md
L4  plans\foresight\APPROVAL-SUSPENSION-PLAN.md
L4  plans\foresight\APPROVAL-UX-PLAN.md
L4  plans\foresight\CODE-APPROVAL-ENGINE.md
L4  plans\foresight\CODE-APPROVAL-UX.md
L4  plans\foresight\CODE-PROOF.md
L4  plans\foresight\CODE-SIDE-EFFECTS.md
L4  plans\foresight\COMPILED-LEDGER.md
L4  plans\foresight\PROOF-RELEASE-PLAN.md
L4  plans\foresight\RUN-1.md
L4  plans\foresight\RUN-2.md
L4  plans\foresight\RUN2-COMPILED-LEDGER.md
L4  plans\foresight\RUN2-ENGINE-CODE.md
L4  plans\foresight\RUN2-PROOF-CODE.md
L4  plans\foresight\RUN2-SIDE-EFFECT-CODE.md
L4  plans\foresight\RUN2-WEB-CODE.md
L4  plans\foresight\RUN-3.md
L4  plans\foresight\SIDE-EFFECT-EXECUTOR-PLAN.md
L4  plans\HIVEMIND-1.0-EXECUTION-PLAN.md
L4  plans\HIVEMIND-1.0-SEAT-ASSIGNMENTS.md
L4  plans\planners\PLANNER-01\GATE-CY-1.md
L4  plans\planners\PLANNER-01\GATE-CY-2.md
L4  plans\planners\PLANNER-01\GATE-CY-3.md
L4  plans\planners\PLANNER-01\PLANNER-01.md
L4  plans\planners\PLANNER-02\GATE-CY-1.md
L4  plans\planners\PLANNER-02\GATE-CY-2.md
L4  plans\planners\PLANNER-02\GATE-CY-3.md
L4  plans\planners\PLANNER-02\PLANNER-02.md
L4  plans\planners\PLANNER-03\GATE-CY-1.md
L4  plans\planners\PLANNER-03\GATE-CY-2.md
L4  plans\planners\PLANNER-03\GATE-CY-3.md
L4  plans\planners\PLANNER-03\PLANNER-03.md
L4  plans\planners\PLANNER-04\GATE-CY-1.md
L4  plans\planners\PLANNER-04\GATE-CY-2.md
L4  plans\planners\PLANNER-04\GATE-CY-3.md
L4  plans\planners\PLANNER-04\PLANNER-04.md
L4  plans\planners\PLANNER-04\READLOG.md
L4  plans\seats\SEAT-01\EVIDENCE.md
L4  plans\seats\SEAT-01\GATE-CY-1.md
L4  plans\seats\SEAT-01\GATE-CY-2.md
L4  plans\seats\SEAT-01\GATE-CY-3.md
L4  plans\seats\SEAT-01\READLOG.md
L4  plans\seats\SEAT-01\REPORT.md
L4  plans\seats\SEAT-02\EVIDENCE.md
L4  plans\seats\SEAT-02\GATE-CY-1.md
L4  plans\seats\SEAT-02\GATE-CY-2.md
L4  plans\seats\SEAT-02\GATE-CY-3.md
L4  plans\seats\SEAT-02\READLOG.md
L4  plans\seats\SEAT-02\REPORT.md
L4  plans\seats\SEAT-03\EVIDENCE.md
L4  plans\seats\SEAT-03\GATE-CY-1.md
L4  plans\seats\SEAT-03\GATE-CY-2.md
L4  plans\seats\SEAT-03\GATE-CY-3.md
L4  plans\seats\SEAT-03\READLOG.md
L4  plans\seats\SEAT-03\REPORT.md
L4  plans\seats\SEAT-04\EVIDENCE.md
L4  plans\seats\SEAT-04\GATE-CY-1.md
L4  plans\seats\SEAT-04\GATE-CY-2.md
L4  plans\seats\SEAT-04\GATE-CY-3.md
L4  plans\seats\SEAT-04\READLOG.md
L4  plans\seats\SEAT-04\REPORT.md
L4  plans\seats\SEAT-05\EVIDENCE.md
L4  plans\seats\SEAT-05\GATE-CY-1.md
L4  plans\seats\SEAT-05\GATE-CY-2.md
L4  plans\seats\SEAT-05\GATE-CY-3.md
L4  plans\seats\SEAT-05\READLOG.md
L4  plans\seats\SEAT-05\REPORT.md
L4  plans\seats\SEAT-06\EVIDENCE.md
L4  plans\seats\SEAT-06\GATE-CY-1.md
L4  plans\seats\SEAT-06\GATE-CY-2.md
L4  plans\seats\SEAT-06\GATE-CY-3.md
L4  plans\seats\SEAT-06\READLOG.md
L4  plans\seats\SEAT-06\REPORT.md
L4  plans\seats\SEAT-07\EVIDENCE.md
L4  plans\seats\SEAT-07\GATE-CY-1.md
L4  plans\seats\SEAT-07\GATE-CY-2.md
L4  plans\seats\SEAT-07\GATE-CY-3.md
L4  plans\seats\SEAT-07\READLOG.md
L4  plans\seats\SEAT-07\REPORT.md
L4  plans\seats\SEAT-08\EVIDENCE.md
L4  plans\seats\SEAT-08\GATE-CY-1.md
L4  plans\seats\SEAT-08\GATE-CY-2.md
L4  plans\seats\SEAT-08\GATE-CY-3.md
L4  plans\seats\SEAT-08\READLOG.md
L4  plans\seats\SEAT-08\REPORT.md
L4  plans\verification\FINAL-AUDIT.md
L4  plans\verification\LEAD-BUGCHECK.md
L4  plans\verification\LEAD-BUGCHECK-RUN2.md
L4  plans\verification\RELEASE-1.1-AUDIT.md
L4  plans\verification\RELEASE-GATE-AUDIT.md
L4  plans\waves\README.md
L4  plans\waves\W10-TOOL-LOOP-DECISION.md
L4  plans\waves\W11-CONTRACT.md
L4  plans\waves\W11-GATES.md
L4  plans\waves\W11-GUI-RESIDUAL.md
L4  plans\waves\W11-SUSPENSION-DESIGN.md
L4  plans\waves\W11-TOOL-ASSEMBLER.md
L4  plans\waves\W11-TRACE.md
L4  plans\waves\W11-UI-TOOL-ACTIVITY.md
L4  plans\waves\W12-INTERFACES.md
L4  plans\waves\W1-CORE-RECOVERY.md
L4  plans\waves\W2-PROVIDER-MATRIX.md
L4  plans\waves\W3-STREAMING.md
L4  plans\waves\W4-LIVE-HARNESS.md
L4  plans\waves\W4-REPLAY-PROOF.md
L4  plans\waves\W4-TRACE-CAPTURE.md
L4  plans\waves\W5B-UI-TESTS.md
L4  plans\waves\W5-PROMPT-ASSETS.md
L4  plans\waves\W7-RELEASE-ENGINEERING.md
L4  plans\waves\W8-DOCS-CHECK.md
L4  plans\waves\W8-DOCS-TRUTH.md
L4  plans\waves\W9-GUI-E2E.md
L4  plans\waves\W9-TRACE-TRUTH.md
L7  README.md
L6  runtime-test\pkg-cli-final\README.md
L6  runtime-test\pkg-cli-final\RUNBOOK.md
L6  runtime-test\pkg-final\README.md
L6  runtime-test\pkg-final\RUNBOOK.md
L6  runtime-test\pkg-smoke\README.md
L6  runtime-test\pkg-smoke\RUNBOOK.md
L6  runtime-test\workspace\README.md
L7  skills\hivemind\SKILL.md
L6  tools\desktop-launcher\README.md
L6  tools\qa\README.md
L6  tools\runbook.md
L6  tools\w64devkit\README.md
L6  tools\w64devkit\src\README.md

## Appendix B — commands and observations

```powershell
git rev-parse --short HEAD                 # 6a41250
git ls-files | Measure-Object              # Count 1573
git tag --list                             # hivemind-1.0.0, hivemind-1.1.0, v1.0.0
(Get-ChildItem docs -Recurse -File).Count  # 981
Get-FileHash dist\HiveMind.zip -Algorithm SHA256
# 15BF4E61C05AEEFC8ABACB341283568CA3705FB5C6E226510CD33B88D6CA5ADA
Get-FileHash dist\HiveMind\hiverouter.exe -Algorithm SHA256
# F369D7AC4CC523EF72FE7DD88DE0191A3C7D0BBB852FF743E542AEFA6A7CF871
# Zip entries: HiveMind.exe, hiverouter.exe, README.txt
# controls.json verdict recompute: 1 WIRED-AND-WORKS / 264 PENDING / 5 MISSING / 49 UNVERIFIED (319)
```

No source/config/test/prompt file was edited, no server was started, no provider was called, and no key was read or printed by this lane. The only write is this report.
