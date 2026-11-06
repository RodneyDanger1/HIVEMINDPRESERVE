# EXECUTION ROADMAP — 101 tasks / 6 phases / 8 seats (within 60–120 requirement)

Tasks: **101** · Phases: **6** · Seats: **8**

| Phase | Tasks | Seats |
|---|---:|---|
| P1 Reconnaissance & Full Inventory | 15 | b-core, b-provider, figcraft-a, figcraft-b, strip-watchdog, ui-watchdog, watchdog-1, watchdog-2 |
| P2 Micro-File Exhaustive Analysis | 20 | b-core, b-provider, figcraft-a, figcraft-b, strip-watchdog, ui-watchdog, watchdog-1, watchdog-2 |
| P3 Architectural Mapping & Interaction Tracing | 20 | b-core, b-provider, figcraft-a, figcraft-b, strip-watchdog, ui-watchdog, watchdog-1, watchdog-2 |
| P4 Upstream Provenance & Donor Repo Audit | 15 | b-core, b-provider, figcraft-a, figcraft-b, strip-watchdog, ui-watchdog, watchdog-1, watchdog-2 |
| P5 Code Quality, Mathematical Rigor & Tech Debt Scoring | 15 | b-core, b-provider, figcraft-a, figcraft-b, strip-watchdog, ui-watchdog, watchdog-1, watchdog-2 |
| P6 Scientific Research Paper Synthesis & Final Roadmap | 16 | b-core, b-provider, figcraft-a, figcraft-b, strip-watchdog, ui-watchdog, watchdog-1, watchdog-2 |

## Task table

| ID | Phase | Seat | Task | Acceptance |
|---|---|---|---|---|
| T001 | P1 | b-core | Recursive directory walk and artifact census | Produce manifest-backed evidence; no claims without file/hash. |
| T002 | P1 | b-provider | Tracked-vs-ignored class reconciliation | Produce manifest-backed evidence; no claims without file/hash. |
| T003 | P1 | watchdog-1 | Language and LOC distribution profile | Produce manifest-backed evidence; no claims without file/hash. |
| T004 | P1 | watchdog-2 | Dependency manifest extraction (Cargo/npm) | Produce manifest-backed evidence; no claims without file/hash. |
| T005 | P1 | figcraft-a | Lockfile provenance and checksum census | Produce manifest-backed evidence; no claims without file/hash. |
| T006 | P1 | figcraft-b | Git-history vs manifest reconciliation | Produce manifest-backed evidence; no claims without file/hash. |
| T007 | P1 | ui-watchdog | File hash integrity baseline | Produce manifest-backed evidence; no claims without file/hash. |
| T008 | P1 | strip-watchdog | Seat-assignment coverage check (507/507) | Produce manifest-backed evidence; no claims without file/hash. |
| T009 | P1 | b-core | Generated-artifact tracking audit (runtime-test DB/logs) | Produce manifest-backed evidence; no claims without file/hash. |
| T010 | P1 | b-provider | Asset census (FigCraft/PDF/PNG/SVG) | Produce manifest-backed evidence; no claims without file/hash. |
| T011 | P1 | watchdog-1 | Binary artifact census (target/dist/exe) | Produce manifest-backed evidence; no claims without file/hash. |
| T012 | P1 | watchdog-2 | Directory ownership and module boundary map | Produce manifest-backed evidence; no claims without file/hash. |
| T013 | P1 | figcraft-a | Untracked/ignored artifact policy | Produce manifest-backed evidence; no claims without file/hash. |
| T014 | P1 | figcraft-b | Census reproducibility script | Produce manifest-backed evidence; no claims without file/hash. |
| T015 | P1 | ui-watchdog | Inventory sign-off report | Produce manifest-backed evidence; no claims without file/hash. |
| T016 | P2 | watchdog-1 | Core entrypoint and boot sequence analysis | Read the file; cite path:line; deep set 8-12 paragraphs. |
| T017 | P2 | watchdog-2 | Contract envelope/command/event/refusal analysis | Read the file; cite path:line; deep set 8-12 paragraphs. |
| T018 | P2 | figcraft-a | Transport/WS security and resync analysis | Read the file; cite path:line; deep set 8-12 paragraphs. |
| T019 | P2 | figcraft-b | Graph authority/cycle analysis | Read the file; cite path:line; deep set 8-12 paragraphs. |
| T020 | P2 | ui-watchdog | Run lifecycle/cancellation/caps analysis | Read the file; cite path:line; deep set 8-12 paragraphs. |
| T021 | P2 | strip-watchdog | Snapshot/migration/kill-restore analysis | Read the file; cite path:line; deep set 8-12 paragraphs. |
| T022 | P2 | b-core | Provider DTO/streaming/refusal analysis | Read the file; cite path:line; deep set 8-12 paragraphs. |
| T023 | P2 | b-provider | Budget/Decimal/pricing analysis | Read the file; cite path:line; deep set 8-12 paragraphs. |
| T024 | P2 | watchdog-1 | Tool registry/schema/allowlist analysis | Read the file; cite path:line; deep set 8-12 paragraphs. |
| T025 | P2 | watchdog-2 | Approval nonce/replay/TTL analysis | Read the file; cite path:line; deep set 8-12 paragraphs. |
| T026 | P2 | figcraft-a | Workspace jail escape analysis | Read the file; cite path:line; deep set 8-12 paragraphs. |
| T027 | P2 | figcraft-b | Audit ledger/observability analysis | Read the file; cite path:line; deep set 8-12 paragraphs. |
| T028 | P2 | ui-watchdog | Web token/contrast/a11y analysis | Read the file; cite path:line; deep set 8-12 paragraphs. |
| T029 | P2 | strip-watchdog | API client/ACK-before-mutation analysis | Read the file; cite path:line; deep set 8-12 paragraphs. |
| T030 | P2 | b-core | State store/event reducer analysis | Read the file; cite path:line; deep set 8-12 paragraphs. |
| T031 | P2 | b-provider | Canvas/nodes/edge analysis | Read the file; cite path:line; deep set 8-12 paragraphs. |
| T032 | P2 | watchdog-1 | Panels/chat/controls analysis | Read the file; cite path:line; deep set 8-12 paragraphs. |
| T033 | P2 | watchdog-2 | E2E harness/trace-spec analysis | Read the file; cite path:line; deep set 8-12 paragraphs. |
| T034 | P2 | figcraft-a | CLI/agent-native contract analysis | Read the file; cite path:line; deep set 8-12 paragraphs. |
| T035 | P2 | figcraft-b | Quality evidence per file | Read the file; cite path:line; deep set 8-12 paragraphs. |
| T036 | P3 | watchdog-1 | Cross-module import graph | Trace callers/emit/subscribe; cite path:line; render Mermaid. |
| T037 | P3 | watchdog-2 | Function/command call graph | Trace callers/emit/subscribe; cite path:line; render Mermaid. |
| T038 | P3 | figcraft-a | Event emitter/subscriber parity | Trace callers/emit/subscribe; cite path:line; render Mermaid. |
| T039 | P3 | figcraft-b | HTTP route inventory and handler map | Trace callers/emit/subscribe; cite path:line; render Mermaid. |
| T040 | P3 | ui-watchdog | WS command dispatch matrix | Trace callers/emit/subscribe; cite path:line; render Mermaid. |
| T041 | P3 | strip-watchdog | Store→DB→event flow trace | Trace callers/emit/subscribe; cite path:line; render Mermaid. |
| T042 | P3 | b-core | Provider call path and failure identity trace | Trace callers/emit/subscribe; cite path:line; render Mermaid. |
| T043 | P3 | b-provider | Tool execution/approval path trace | Trace callers/emit/subscribe; cite path:line; render Mermaid. |
| T044 | P3 | watchdog-1 | Static-UI/serve path trace | Trace callers/emit/subscribe; cite path:line; render Mermaid. |
| T045 | P3 | watchdog-2 | History/replay path trace | Trace callers/emit/subscribe; cite path:line; render Mermaid. |
| T046 | P3 | figcraft-a | Error/refusal propagation map | Trace callers/emit/subscribe; cite path:line; render Mermaid. |
| T047 | P3 | figcraft-b | Concurrency and backpressure map | Trace callers/emit/subscribe; cite path:line; render Mermaid. |
| T048 | P3 | ui-watchdog | Boundary/ownership map | Trace callers/emit/subscribe; cite path:line; render Mermaid. |
| T049 | P3 | strip-watchdog | Bottleneck and refactor-risk inventory | Trace callers/emit/subscribe; cite path:line; render Mermaid. |
| T050 | P3 | b-core | Mermaid topology diagram set | Trace callers/emit/subscribe; cite path:line; render Mermaid. |
| T051 | P3 | b-provider | Mermaid data-flow diagram | Trace callers/emit/subscribe; cite path:line; render Mermaid. |
| T052 | P3 | watchdog-1 | Mermaid state-machine diagram | Trace callers/emit/subscribe; cite path:line; render Mermaid. |
| T053 | P3 | watchdog-2 | Mermaid dependency/provenance diagram | Trace callers/emit/subscribe; cite path:line; render Mermaid. |
| T054 | P3 | figcraft-a | Diagram validation/export receipts | Trace callers/emit/subscribe; cite path:line; render Mermaid. |
| T055 | P3 | figcraft-b | Architecture sign-off | Trace callers/emit/subscribe; cite path:line; render Mermaid. |
| T056 | P4 | b-core | Crate dependency lineage extraction | Fetch official source or mark UNVERIFIED; cite URL+date. |
| T057 | P4 | b-provider | npm dependency lineage extraction | Fetch official source or mark UNVERIFIED; cite URL+date. |
| T058 | P4 | watchdog-1 | License compatibility matrix | Fetch official source or mark UNVERIFIED; cite URL+date. |
| T059 | P4 | watchdog-2 | Vendored/toolchain provenance audit | Fetch official source or mark UNVERIFIED; cite URL+date. |
| T060 | P4 | figcraft-a | FigCraft asset provenance audit | Fetch official source or mark UNVERIFIED; cite URL+date. |
| T061 | P4 | figcraft-b | Blueprint/donor pattern lineage map | Fetch official source or mark UNVERIFIED; cite URL+date. |
| T062 | P4 | ui-watchdog | Upstream repo currency and maintenance check | Fetch official source or mark UNVERIFIED; cite URL+date. |
| T063 | P4 | strip-watchdog | Donor mechanism adoption map | Fetch official source or mark UNVERIFIED; cite URL+date. |
| T064 | P4 | b-core | Candidate donor capability gaps | Fetch official source or mark UNVERIFIED; cite URL+date. |
| T065 | P4 | b-provider | Supply-chain security review (lockfiles/checksums) | Fetch official source or mark UNVERIFIED; cite URL+date. |
| T066 | P4 | watchdog-1 | Provenance UNVERIFIED register | Fetch official source or mark UNVERIFIED; cite URL+date. |
| T067 | P4 | watchdog-2 | Provenance Mermaid graph | Fetch official source or mark UNVERIFIED; cite URL+date. |
| T068 | P4 | figcraft-a | Third-party notice/attribution check | Fetch official source or mark UNVERIFIED; cite URL+date. |
| T069 | P4 | figcraft-b | Dependency upgrade risk list | Fetch official source or mark UNVERIFIED; cite URL+date. |
| T070 | P4 | ui-watchdog | Provenance sign-off | Fetch official source or mark UNVERIFIED; cite URL+date. |
| T071 | P5 | watchdog-1 | Per-file correctness evidence score | Score 1-5 with evidence; list debt with severity. |
| T072 | P5 | watchdog-2 | Maintainability/complexity score | Score 1-5 with evidence; list debt with severity. |
| T073 | P5 | figcraft-a | Modularity/coupling score | Score 1-5 with evidence; list debt with severity. |
| T074 | P5 | figcraft-b | Error-handling/refusal score | Score 1-5 with evidence; list debt with severity. |
| T075 | P5 | ui-watchdog | Security/authority score | Score 1-5 with evidence; list debt with severity. |
| T076 | P5 | strip-watchdog | Documentation score | Score 1-5 with evidence; list debt with severity. |
| T077 | P5 | b-core | Test coverage mapping by module | Score 1-5 with evidence; list debt with severity. |
| T078 | P5 | b-provider | Runtime/behavioral evidence audit | Score 1-5 with evidence; list debt with severity. |
| T079 | P5 | watchdog-1 | Mathematical/logic invariants audit | Score 1-5 with evidence; list debt with severity. |
| T080 | P5 | watchdog-2 | Money/Decimal integrity audit | Score 1-5 with evidence; list debt with severity. |
| T081 | P5 | figcraft-a | Concurrency/timeout invariant audit | Score 1-5 with evidence; list debt with severity. |
| T082 | P5 | figcraft-b | Generated-artifact debt catalog | Score 1-5 with evidence; list debt with severity. |
| T083 | P5 | ui-watchdog | Stale-doc/claim drift catalog | Score 1-5 with evidence; list debt with severity. |
| T084 | P5 | strip-watchdog | Debt severity/priority ranking | Score 1-5 with evidence; list debt with severity. |
| T085 | P5 | b-core | Quality matrix sign-off | Score 1-5 with evidence; list debt with severity. |
| T086 | P6 | watchdog-1 | Abstract/introduction drafting | Compile verified text only; reconcile every number with manifests. |
| T087 | P6 | watchdog-2 | Architecture & topology chapter | Compile verified text only; reconcile every number with manifests. |
| T088 | P6 | figcraft-a | Empirical file-analysis chapter | Compile verified text only; reconcile every number with manifests. |
| T089 | P6 | figcraft-b | Provenance & donor-lineage chapter | Compile verified text only; reconcile every number with manifests. |
| T090 | P6 | ui-watchdog | Quality/logic chapter | Compile verified text only; reconcile every number with manifests. |
| T091 | P6 | strip-watchdog | Current-vs-target state specification | Compile verified text only; reconcile every number with manifests. |
| T092 | P6 | b-core | Refactor wave 1 (P0 safety/correctness) | Compile verified text only; reconcile every number with manifests. |
| T093 | P6 | b-provider | Refactor wave 2 (contracts/observability) | Compile verified text only; reconcile every number with manifests. |
| T094 | P6 | watchdog-1 | Refactor wave 3 (UI/UX + assets) | Compile verified text only; reconcile every number with manifests. |
| T095 | P6 | watchdog-2 | Refactor wave 4 (packaging/toolchain) | Compile verified text only; reconcile every number with manifests. |
| T096 | P6 | figcraft-a | Acceptance gates per wave | Compile verified text only; reconcile every number with manifests. |
| T097 | P6 | figcraft-b | Risk/rollback plan | Compile verified text only; reconcile every number with manifests. |
| T098 | P6 | ui-watchdog | Diagram finalization | Compile verified text only; reconcile every number with manifests. |
| T099 | P6 | strip-watchdog | Zero-hallucination fact-check pass | Compile verified text only; reconcile every number with manifests. |
| T100 | P6 | b-core | Final paper compile + appendix | Compile verified text only; reconcile every number with manifests. |
| T101 | P6 | b-provider | Final deliverable sign-off | Compile verified text only; reconcile every number with manifests. |
