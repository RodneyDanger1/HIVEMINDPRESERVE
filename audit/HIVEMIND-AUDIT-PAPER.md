# An Exhaustive Empirical Audit of HiveMind v1: Architecture, Provenance, Quality, and a Migration Roadmap

**Authors:** HiveMind Audit Team (Lead + 8 specialized seats) · **Date:** 2026-11-06 · **Revision:** draft-v1 (seat synthesis pending)
**Subject of record:** `C:\Users\Riley\Desktop\hiverouter-v1`, git `HEAD 3988622`, tag `v1.0.0` (clean tree).
**Evidence basis:** `audit/inventory/FILE-MANIFEST.csv` (sha256 per tracked file), seat records in `audit/parts/<seat>/`, and the frozen blueprint hashes in `C:\Users\Riley\Desktop\hivemind-blueprint\blueprint\BUILD-CHECKS.json`.

---

## Abstract

This paper reports an exhaustive, evidence-first audit of the HiveMind v1 repository at its first release tag. The audit covers **507 git-tracked files** (63.98 MiB), of which **188 are classified as code** (79 Rust / 21,669 LOC; 40 TypeScript / 5,870; 21 TSX / 1,708; 27 Python / 6,293, including 30 build/design scripts under `docs/`; the manifest's strict `source` category alone contains 158 files) totalling **36,802 source lines**, and separately censuses **six ignored/generated trees** (Rust build target 13,525 files/8,087 MB; npm vendor 4,170 files; toolchain 9,452 files; packaged `dist/` 6 files). We hash every tracked file, partition the repository into eight disjoint audit sectors, trace module dependencies and command/event flows, map upstream dependency lineage (405 Cargo-lock packages; 147 npm-lock packages), score code quality per file, and derive a **101-task, 6-phase, 8-seat** migration roadmap. The empirical layer confirms a compact, contract-first localhost architecture (single Rust binary + embedded React SPA) with a rich test corpus (233 backend tests, 46 frontend tests, 79 passing live-browser tests with 14 explicit skips), and identifies concrete defects and debt: **the shipped core never wires the provider client, so a live model run is impossible even with a key (P0)**; **history replay rendering is dead (P0)**; a **backend-vs-frontend state parity defect** (16 declared contract states vs 15 UI states, missing `interrupted`); 82 tracked runtime artifacts (SQLite DB/WAL/SHM/logs, including duplicate shipped binaries); untracked headline evidence; an absolute-path cargo config; mixed FigCraft tracking/ignore policy; stale verification counts in shipped reports; and 14 explicit e2e `test.fixme` skips. A total of **30 consolidated findings** is registered (3 P0, 10 P1, 16 P2, 1 informational), each with file/line evidence; a further 18 gitignored design deliverables (22,009,234 B / 21 MiB) are censused in Appendix D because they sit outside the tracked manifest. The report does not claim what it did not verify: vendor trees are censused rather than prose-analysed, pixel-level fidelity remains human-reviewable, and every unsupported assertion is labelled `UNVERIFIED` with the evidence that would settle it.

**Keywords:** software audit, zero-hallucination verification, Rust, React, localhost security, provenance, technical debt, migration roadmap.

---

## 1. Introduction

### 1.1 Motivation

HiveMind v1 is a single-binary localhost "spatial execution environment": a Rust server (tokio/axum/reqwest/rusqlite/tracing/rust-embed) serving a React 19 + Vite 8 + @xyflow/react 12.12.1 UI, driven by one DeepSeek key. It was built from a large, externally verified blueprint in one working session and shipped as `v1.0.0`. Claims made during delivery — "233 tests pass", "the UI serves", "the jail fails closed" — need independent, file-level grounding. This audit asks four questions:

1. **What exists?** A complete, hashed census of every tracked file and every ignored/generated class.
2. **How does it work?** File-by-file mechanics and a traced architecture: modules, commands, events, state, storage, security boundaries.
3. **Where did it come from?** Upstream dependency and donor-pattern lineage, with licences and provenance status.
4. **What should change?** Quantified quality scores and a prioritised migration roadmap.

### 1.2 Contributions

- A reproducible, hash-anchored manifest of 507 tracked files and a classified census of generated/vendor trees.
- Eight disjoint seat audits with per-file records, dependency matrices, quality scores, and findings.
- Exact static architecture facts: four internal crates with the dependency chain `contract → provider → tools → core`; 14 frozen commands, 21 events, 65 refusal codes, 16 declared agent states; 405 Cargo-lock and 147 npm-lock packages.
- A verified list of defects/debt, including the state-parity defect and tracked runtime artifacts.
- A 101-task, 6-phase, 8-seat roadmap with acceptance gates.

### 1.3 Scope and non-claims

Tracked files are analysed individually; ignored/generated trees are censused (counts/bytes) and explicitly excluded from prose analysis. The requirement of "8–12 paragraphs per file" is applied in full to the 56-file **deep set**; remaining first-party files receive structured records (metadata, purpose, mechanics, dependency/caller matrix, quality, evidence). Documentation and generated assets receive provenance/role/quality records rather than fictional code analysis. No live paid-provider call was made; runtime/terminal claims are therefore bounded. Pixel-level design fidelity remains `needs_human`.

---

## 2. Methodology and Data

### 2.1 Census and hashing

`git ls-files` enumerated the tracked corpus; each file was read as bytes and hashed with SHA-256 into `FILE-MANIFEST.csv/.json`. Ignored trees were measured separately (`DIRECTORY-CENSUS.md`). The manifest is the single source of truth for path, size, hash, language, category, seat and depth. Binary/generated files are classified, not decoded.

### 2.2 Partitioning

Assignments are deterministic and exhaustive (`ASSIGNMENTS.json`): **b-core** 54 files (contract + core), **b-provider** 33 (provider + tools + manifests), **watchdog-1** 55 (web), **watchdog-2** 122 (e2e + tools + runtime-test), **figcraft-a** 103 (docs/ui + ledgers), **figcraft-b** 91 (docs plans/contracts/claims/reports/audit), **ui-watchdog** 49 (trace/diagrams/root + provenance), **strip-watchdog** independent verifier. Sum = 507.

### 2.3 Evidence rules

Each claim must cite `path:line`, a command output, or an official URL with fetch date. Unverifiable claims are marked `UNVERIFIED` with the proving action. Seats load the requested skills (`forgewright-recon`, `forgewright-lead`, `integration-logic`, `smartfix`, `research`, `auditor`, `truthseeker`, `trace`, `weaver`, `forge`) and write only inside `audit/parts/<seat>/`. The verifier independently samples hashes/claims and checks that no product file changed.

### 2.4 Metrics

Counts: files, bytes, LOC (`CODE-STATS.md`), dependency counts, test counts, commits/contributors. Note: the 507-file manifest covers **tracked** files only; 18 gitignored primary design deliverables are separately censused in `audit/APPENDIX-IGNORED-DELIVERABLES.md` (22,009,234 B, hashes recomputed). Quality: per-file 1–5 scores across correctness evidence, maintainability, modularity, error handling, security, documentation, each with evidence; aggregate in `QUALITY-MATRIX.csv`. Debt: findings ranked P0 (correctness/security/data), P1 (contract/observability), P2 (hygiene/UX), P3 (cosmetic).

### 2.5 Limitations

Ignored vendor/build trees are not hashed per file; quality scores are evidence-weighted human judgments, not coverage instruments; no paid model run; trace data is partial by design; single-working-session history limits historical provenance.

---

## 3. Architecture and Topology

HiveMind v1 is a **four-crate Rust workspace** plus a **React SPA** and an **agent-native CLI**:

- `hiverouter-contract` — frozen protocol: envelope, commands, events, refusals, state, JSON schemas. No internal dependencies.
- `hiverouter-provider` — DeepSeek HTTP client, DTOs, pricing, Decimal budget/accounting. Depends on contract.
- `hiverouter-tools` — tool registry, approval gate (nonce/replay/TTL), workspace jail, shell containment (win32job), JSONL audit ledger, capability census, CLI binary. Depends on contract + provider.
- `hiverouter-core` — binary/server: config, DB, HTTP routes (`/health`, `/api/session`, history, `/diagnose`, static UI), WS transport (Origin allowlist, session token, hello/resync), graph authority (petgraph), run lifecycle/execution, snapshots/recovery, tools bridge, embedded `web/dist`. Depends on contract + tools.

The browser talks to the binary only over loopback: a session token from `/api/session` authenticates a WebSocket handshake; a foreign `Origin` is refused; every mutating command is ACK-or-refusal, with events fan-out carrying monotonic sequence numbers. Provider calls are bounded by the blueprint's budget core; cost math uses `rust_decimal` only. The design intentionally collapses transport, execution, storage, UI delivery and diagnostics into one machine-local process — reducing deployment risk while making the in-process boundaries the critical audit surface.

**Diagrams:** `audit/diagrams/01-system-topology`, `02-data-flow`, `03-module-dependencies`, `04-state-inventory`, `05-dependency-lineage`, `06-migration-target` (`.mmd` + exported `.svg/.png`).

**Verified architecture facts:** internal dependency chain above (each crate's `Cargo.toml`); 14 commands / 21 events / 65 refusals (contract test counts); 16 declared `AgentState` variants with five terminal states (`failed, completed, cancelled, killed, interrupted`); 405 packages in `Cargo.lock`; 147 in `package-lock.json`; loopback bind refusal and Origin/token checks covered by tests.

---

## 4. Empirical File Analysis

### 4.1 Corpus profile

| Dimension | Count / value |
|---|---|
| Tracked files | 507 |
| Tracked bytes | 63.98 MiB |
| Code-classified files (incl. docs-embedded build scripts) | 188 (manifest strict `source` category: 158) |
| Source LOC | 36,802 |
| Total text LOC (incl. docs/data) | 106,840 |
| Top directories | docs 236, backend 87, runtime-test 82, web 55, e2e 24, tools 16, roots 4, ledgers 2, .cargo 1 |
| Languages | Rust 79 files/21,669 LOC; TypeScript 40/5,870; TSX 21/1,708; Python 27/6,293; JSON 88/60,049; Markdown 57/4,193; JavaScript 12/798; CSS 243; PowerShell 198; HTML 17; C 6 (listed subtotal 35,540 + 1,262 = 36,802 source LOC) |
| Cargo lock packages | 405 |
| npm lock packages | 147 |
| Git history | 23 commits, 2 author identities, all dated 2026-11-06 |

### 4.2 Empirical findings (consolidated register: `audit/FINDINGS-REGISTER.md`)

Thirty findings were registered (3 P0, 10 P1, 16 P2, 1 informational). Headline items:

- **A-F01 (P0, live-run blocker): the shipped server cannot call the model.** `hiverouter-core/Cargo.toml` depends only on `hiverouter-contract` and `hiverouter-tools`; `core/src/http.rs:77,117,154` constructs an `UnconfiguredExecutor`, so `SUBMIT_PROMPT` never reaches `hiverouter-provider`. Provider crate tests pass in isolation, but the binary run path is unconfigured. Evidence: `backend/crates/hiverouter-core/Cargo.toml`; `core/src/http.rs`.
- **A-F02 (P0, dead replay):** `web/src/flow/Canvas.tsx:113-127` passes `replayNodes = []`; `HISTORY_SNAPSHOT` is stored (`store.ts:379-385`) but never rendered, so the time-travel scrubber cannot display the graph.
- **A-F03 (P0, contract parity):** `contract/state.rs:18-51` declares 16 `AgentState` variants (including `interrupted`); `web/src/state/agentState.ts:8-24` declares 15 and its header wrongly calls the contract a "14-state list".
- **A-F04 (P1):** prompt/persona assets never loaded by the production server (`http.rs:82` `TemplateRegistry::default()`); handshake/context-switch code has no live caller (`ws.rs:338-353`).
- **A-F05 (P1):** Settings renders a refusal from `refusals[0]` instead of the `SETTINGS_STATUS` error (`store.ts:666`).
- **A-F06 (P1):** money overflow silently yields `Decimal::ZERO` in `price()` (`provider/src/accounting.rs:330`).
- **A-F07 (P1):** duplicate major versions resolved in `Cargo.lock` (`thiserror` 1.0.69+2.0.21; `tower-http` 0.6.11+0.7.1); `cargo tree -d` verdict remains `UNVERIFIED`.
- **A-F08 (P1):** headline verification receipts (`final-gate-tests.log`, `run-live-FINAL-artifact.log`, etc.) are **untracked** and therefore absent from the 507-file manifest.
- **A-F09 (P1):** 82 tracked `runtime-test/` artifacts (SQLite DB/WAL/SHM, snapshots, logs), including the shipped binary three times (identical hash), a stale build and a duplicate CLI.
- **A-F10 (P1):** documentation drift: FINAL-REPORT trace counts (260 UNVERIFIED/5 MISSING) disagree with `docs/trace/controls.json` (270 PENDING/6 MISSING); FINAL-DELTA's claim-register FAIL predates its repair; one committed `.last-run.json` says `failed` while closeout says passed.
- **A-F11 (P1):** `.cargo/config.toml` hardcodes absolute host paths for linker/CC/AR.
- **A-F12 (P1):** 18 primary design deliverables are gitignored and outside the tracked manifest (Appendix D censuses them; hashes preserved in A/B manifests and the watchdog ledger).
- **A-F13…A-F24 (P2):** unwired ad-hoc e2e probe scripts; skip-as-green env guard; wiped test logs; trace completeness gaps; tests writing into `docs/`; default baseURL mismatch; unbundled fonts; dev mock statically imported; missing HTTP timeouts; favicon hex; optional approval nonce; stale docs.
- **FA-F2…FA-F6 (P2/info):** ASSETS-INDEX drops all B export hashes; index watchdog block stale vs ledger; B episode schema non-standard; tracked CPython-3.14 `.pyc`/scratch files; iterative patch scripts tracked.

### 4.3 Seat results (empirical)

| Seat | Coverage | Deep | Quality mean (correctness / docs) | Top finding |
|---|---:|---:|---|---|
| b-core | 54/54 (sha 54/54) | 21/21 | 4.85 / 4.81 | A-F01 provider not wired |
| b-provider | 33/33 (sha 33/33) | 15/15 | 4.55 / 3.58 | A-F06 silent zero cost overflow |
| watchdog-1 | 55/55 (sha 55/55) | 16/16 | 4.47 / 3.98 | A-F02 dead replay; A-F05 wrong setting refusal |
| watchdog-2 | 122/122 (sha 122/122) | 4/4 | 3.74 / 2.51 | A-F09 runtime artifacts; harness gaps |
| figcraft-b | 91/91 (sha 91/91) | 0 | 4.20 / 2.97 | A-F10 docs/claims drift |
| figcraft-a | 103/103 (sha 103/103) | 0 | 4.50 / 3.31 | A-F12 ignored deliverables; index hash gaps |
| ui-watchdog | 49/49 (sha 49/49) | 0 | 3.92 / 3.69 | A-F11 absolute paths; provenance/licence gaps |
| strip-watchdog | independent 507/507 hashes verified | — | — | verification report in `audit/parts/strip-watchdog/VERIFICATION.md` |

All **507/507** files are accounted for by the 7 analytic seats plus Lead census and the independent verifier. FigCraft-A's final `FILES.md` contains 103/103 path records (the verifier's 422/507 figure was a mid-write poll); no Lead fallback records were needed after de-duplication. Per-seat records: `audit/parts/<seat>/FILES.md`.
---

## 5. Upstream Provenance and Donor Lineage

### 5.1 Dependency lineage (static, verified)

The Rust workspace pins **exact versions** in `backend/Cargo.toml` (e.g., `tokio =1.54.1`, `axum =0.8.9`, `reqwest =0.13.5`, `rusqlite =0.41.1` with `bundled`, `petgraph =0.8.3`, `rust_decimal =1.43.0`, `schemars =1.2.3`, `jsonschema =0.60.2`, `tracing =0.1.44`, `rust-embed =8.13.0`, `prompty =2.1.0`, `zeroize =1.9.1`, `win32job =2.0.3`). The web app pins React 19.3.1, Vite 8.3.6, TypeScript 7.0.2, Tailwind 4.3.4, @xyflow/react 12.12.1, dagre 3.1.3, zustand 5.0.16, motion 14.9.1, partysocket 1.3.0. `Cargo.lock` resolves 405 packages; `package-lock.json` resolves 147.

### 5.2 Donor lineage

The repository does **not** vendor a donor runtime. Mechanisms were adopted through interface-level adaptation, documented in `docs/contracts/ADAPTERS.md`: DeepSeek HTTP contract reimplemented from the vendor API; approval/jail/shell mechanisms are in-house; `wiremock` is dev-only; FigCraft is the external design-production tool, and its outputs are imported as assets, not as code. The blueprint's external research register records accepted repositories/packages and rejections; this audit treats those as design provenance, not committed source. **Every licence claim not fetched is `UNVERIFIED`; seat `ui-watchdog/PROVENANCE.md` records per-dependency findings with URL+date.**

### 5.3 Supply-chain observations

- The workspace license is `MIT OR Apache-2.0`; individual dependency licenses were not independently fetched in this pass (UNVERIFIED pending seat research).
- The build toolchain is local (`w64devkit` GCC 16.2.0 + rustup GNU toolchain); the shipped binary links statically but the toolchain tree is **not** tracked.
- `dist/` is release output (6 files, 20 MB) and is tracked; this is a packaging choice with binary-blob implications.

---

## 6. Code Quality and Mathematical/Logical Integrity

### 6.1 Evidence-backed test posture

| Gate | Result | Evidence |
|---|---|---|
| Rust contract/core/provider/tools | 233 passed / 0 failed / 1 ignored | `docs/audit/evidence/final-gate-tests.log` |
| Frontend unit/typecheck/build | 46 tests, typecheck 0, Vite build 0 | `web/`; FRONTEND status |
| Live browser e2e | 79 passed / 0 failed / 14 explicit skips | `e2e/test-results/run-live-FINAL-artifact.log` |
| Frozen contract integrity | 14 commands / 21 events / 65 refusals / 16 states | contract tests + source |
| Live server | `/health`, `/diagnose`, `/api/session`, `/` all 200; traversal 404; foreign Origin 403 | `docs/audit/PHASE-B-VERIFICATION.md` |
| Packaging | server+CLI zip, clean-extraction smoke | `dist/` + `FINAL-CLOSEOUT.md` |

### 6.2 Logical invariants

Verified by tests and source: one graph authority with cycle refusal; ACK-before-mutation; named refusals (65 codes); nonce-bound single-use approvals with replay/TTL refusal; workspace jail fails closed on 6 escape vectors; Decimal-only cost math; NULL usage ≠ zero (`INCOMPLETE`, `settled_usd:null`); loopback-only bind; Origin+token WS boundary. The audit does not re-derive their proofs here; file-level evidence is in the parts.

### 6.3 Quality scoring (final)

Per-file scores were collected for all **507** files and merged into `audit/QUALITY-MATRIX.csv` (`audit/tools/merge_quality.py`). Code-only aggregates (184 of 188 code-classified files scored; 4 binary/C fixtures not scored) are:

| Dimension | Mean /5 |
|---|---:|
| correctness | 4.43 |
| maintainability | 4.25 |
| modularity | 4.32 |
| error handling | 4.15 |
| security | 4.11 |
| documentation | 3.84 |

Seat means (all files) are in `audit/QUALITY-SUMMARY.json`; code-only per-seat table in `audit/QUALITY-CODE-STATS.md`. **No score is assigned without cited evidence**, and the scores measure the audited tree, not a coverage guarantee: docs/assets drag the all-file means down (3.34 documentation), while core Rust scores highest (b-core 4.85/4.81). The scoring does not mask the P0/P1 findings above — a high maintainability score is not a correctness claim.
---

## 7. Refactoring and Migration Roadmap

### 7.1 Current → target

Current: a working, compact, single-binary v1.0.0 with strong contract discipline and a rich test corpus, but with hygiene debt (tracked runtime/evidence artifacts), a contract-parity defect (16 vs 15 states), a partial trace/verification surface, and documentation-heavy corpus. Target v1.1: a generated-artifact-clean repository; backend/frontend protocol parity enforced by CI; a complete evidence-backed trace matrix (or explicitly gated UNVERIFIED modes); human-approved pixel fidelity; reproducible packaging with CLI parity.

### 7.2 Multi-agent roadmap

`audit/ROADMAP.md` and `audit/ROADMAP-TASKS.csv` define **101 tasks across 6 phases and 8 seats** (within the requested 60–120 task envelope): P1 Recon & Inventory (15), P2 Micro-file analysis (20), P3 Architecture & tracing (20), P4 Provenance (15), P5 Quality & debt (15), P6 Paper & final roadmap (16). Each task has an objective, owner seat, deliverable, acceptance criterion, required evidence, dependency and priority.

### 7.3 Prioritised waves (driven by the audit findings)

- **Wave 1 (P0 — correctness):** wire `hiverouter-provider` into `hiverouter-core` (A-F01) and add a live/mocked run-path test; render `HISTORY_SNAPSHOT` replay (A-F02); fix `interrupted` state parity in the UI + contract drift test (A-F03).
- **Wave 2 (P1 — contract/hygiene):** prompt/persona loading + handshake wiring (A-F04); settings refusal source (A-F05); Decimal overflow refusal (A-F06); duplicate-major gate `cargo tree -d` (A-F07); track or regenerate headline evidence (A-F08); remove/relocate 82 runtime artifacts + duplicate binaries (A-F09); reconcile stale docs/claims (A-F10); parameterise `.cargo/config.toml` (A-F11); fix design-artifact tracking policy (A-F12).
- **Wave 3 (P2 — verification/UX):** wire the six ad-hoc probe scripts or delete them; make skip conditions fail-closed; persist test logs; implement/declare the 12 DOM G-UI gates; normalise baseURL/artifact preflight; bundle fonts; remove dev mock from production bundle; HTTP timeouts; token-isolation scope; approval nonce in the frozen contract; regenerate ASSETS-INDEX with all B hashes.
- **Wave 4 (P3 — maintainability):** remove tracked `.pyc`/scratch/patch intermediates, consolidate documentation, decide S25 code-diff and Tauri/installer futures.

Acceptance gates per wave are recorded in `ROADMAP.md`; every task requires a command or `path:line` receipt before it can be marked complete. The 101-task roadmap table (`ROADMAP-TASKS.csv`) remains the execution schedule; the findings above are the prioritized backlog it should be sequenced against.
---

## 8. Conclusion

HiveMind v1 is a compact, unusually well-instrumented localhost application: a frozen Rust protocol, a real React UI, a packaged CLI, 233 backend tests, 79 live-browser tests, and an audit trail that names its own unverified regions. This audit makes the claim checkable: every one of the 507 tracked files is hashed and assigned, all 56 deep files are analysed, 507 files are quality-scored, dependencies and provenance are mapped, and the defects are stated plainly rather than polished away. The decisive finding is that **the shipped binary does not yet wire the provider run path** (A-F01), so v1.0.0 is a verified shell/transport/UI release rather than a verified end-to-end model runtime; the second is that replay rendering (A-F02) and state parity (A-F03) are incomplete. The right sequence is therefore: wire the provider, revive replay, fix parity, then clean the verification/hygiene debt. The codebase's strongest asset — its refusal-first, no-fake-success discipline — is exactly what makes this honest audit possible and what the roadmap must preserve.

---

## Appendix A — Deliverables and evidence index

| Artifact | Purpose |
|---|---|
| `audit/inventory/FILE-MANIFEST.csv/.json`, `FILE-MANIFEST-ENRICHED.*` | 507 tracked files: sha256, bytes, language, category, seat, depth, LOC |
| `audit/inventory/ASSIGNMENTS.json` | exhaustive per-seat partition |
| `audit/inventory/DEEP-LIST.md`, `DIRECTORY-CENSUS.md`, `CODE-STATS.md` | deep set, class census, LOC statistics |
| `audit/parts/<seat>/{FILES,DEPENDENCIES,QUALITY,FINDINGS,EVIDENCE}.md/csv` | per-file empirical records |
| `audit/diagrams/*.mmd/.svg/.png` | topology, data flow, module deps, state inventory, lineage, migration |
| `audit/ROADMAP.md`, `ROADMAP-TASKS.csv` | 101 tasks / 6 phases / 8 seats |
| `audit/parts/strip-watchdog/VERIFICATION.md` | independent fact-check |
| `audit/QUALITY-MATRIX.csv`, `QUALITY-SUMMARY.json`, `QUALITY-CODE-STATS.md` | merged quality scores and code-only aggregates |
| `audit/FINDINGS-REGISTER.md` | consolidated 30-finding register (P0/P1/P2) |
| `audit/APPENDIX-FILE-CENSUS.md`, `APPENDIX-IGNORED-DELIVERABLES.md` | 507-file census; 18 gitignored design deliverables |

## Appendix B — Reproducibility

```powershell
git -C C:\Users\Riley\Desktop\hiverouter-v1 rev-parse HEAD   # 3988622
python audit/tools/build_inventory.py                         # manifest
python audit/tools/enrich_manifest.py                        # LOC/stats
python audit/tools/build_roadmap.py                          # 101 tasks
Get-FileHash audit/inventory/FILE-MANIFEST.csv -Algorithm SHA256
```

## Appendix C — Limitations register

- Ignored/generated trees are censused, not individually hashed or prose-analysed; 18 gitignored primary design deliverables (196 MiB) are separately censused in `audit/APPENDIX-IGNORED-DELIVERABLES.md` with recomputed hashes.
- 8–12 paragraph depth applies to the 56-file deep set; other files use structured records.
- Quality scores are evidence-weighted judgments; they are not coverage metrics.
- No paid-provider run; runtime terminal/streaming/approval-positive claims remain UNVERIFIED.
- Trace ladder is partial; skipped e2e gates are explicit `test.fixme`, not passes.
- Pixel-level design fidelity and licence provenance of all transitive packages require human/legal review; unfetched claims are `UNVERIFIED`.
