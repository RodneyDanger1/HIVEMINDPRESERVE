# PROVENANCE.md — donor lineage & original code (ui-watchdog seat)

**Fetched:** 2026-11-06 via official registries/APIs (crates.io API, registry.npmjs.org, api.github.com). Versions are the exact lockfile/manifest versions read locally. Where a field could not be confirmed it is marked UNVERIFIED.

## 1. What is original vs donor-pattern

| class | disposition | evidence |
|---|---|---|
| HiveRouter domain (`hiverouter-{contract,core,provider,tools}` crates, command/event envelope, WAIT-set state machine, guardrails) | **Original** synthesis from the HiveMind blueprint (source of record `hivemind-blueprint/source/HIVEMIND POSSIBLITY.pruned.txt`, sha `7B85760E…D713`); no upstream repo matches | README 'Built from the verified blueprint'; CHANGE-LEDGER-IMPL rows CL-I-001/002; ER + sequence diagrams cite blueprint phases |
| Architecture patterns (axum router + WS, tokio runtime, petgraph cycle refusal, rusqlite WAL store, tower middleware, reqwest+rustls) | **Donor-pattern usage** — idiomatic upstream library APIs, not copied code | direct pins in `backend/Cargo.toml`; crate repos in §2 |
| Canvas UI (React 19 + @xyflow/react 12.12.1 + zustand + motion + partysocket + dagre) | **Donor-pattern usage** — library composition; product surfaces S01-S28 are original | `web/package.json`; `docs/diagrams/ui-control-event-sequence.mmd`; `web/src/styles/tokens.css` referenced by A11Y plan |
| Diagrams (5 Mermaid sources + mmdc SVG/PNG) | **Generated original content on donor tool** (mermaid MIT) | `.mmd` headers cite blueprint/contracts; renders match viewBox ratios |
| Trace/QA harness docs + captures | **Original** (Playwright Apache-2.0 harness) | QA-HARNESS §4 layout; live/README run log paths |
| Toolchain w64devkit | **Donor binary** (Unlicense) | `.cargo/config.toml`; GitHub license fetched |

## 2. Upstream donors — official-source verification

| donor | version(s) used | license (source, fetched) | role | what derives |
|---|---|---|---|---|
| axum | 0.8.9 | MIT — https://github.com/tokio-rs/axum (fetched 2026-11-06); crates.io https://crates.io/crates/axum | HTTP/WS server | routing/WS/Origin+token middleware patterns |
| tokio | 1.54.1 | MIT — https://github.com/tokio-rs/tokio (2026-11-06); https://crates.io/crates/tokio | async runtime | runtime/scheduler/cancellation patterns |
| tokio-tungstenite / tungstenite | 0.29.0 | MIT (crates.io repo field; license via upstream repo) — UNVERIFIED per-repo fetch | WS framing | test/WS wire support |
| petgraph | 0.8.3 | Apache-2.0 — https://github.com/petgraph/petgraph (2026-11-06); https://crates.io/crates/petgraph | graph authority | cycle-refusal graph patterns |
| rusqlite / libsqlite3-sys | 0.41.1 / 0.39.1 | MIT — https://github.com/rusqlite/rusqlite (2026-11-06); https://crates.io/crates/rusqlite | SQLite store | WAL/transaction/idempotency patterns |
| reqwest | 0.13.5 | Apache-2.0 — https://github.com/seanmonstar/reqwest (2026-11-06); https://crates.io/crates/reqwest | provider HTTP | rustls client, timeout/retry patterns |
| uuid | 1.29.0 | MIT or Apache-2.0 (crates.io full text UNVERIFIED) — https://github.com/uuid-rs/uuid | id generation | v7 correlation ids |
| rust_decimal | 1.43.0 | MIT — https://github.com/paupino/rust-decimal | money/tokens | Decimal accounting, NULL-not-zero |
| wiremock | 0.6.5 | MIT — https://github.com/LukeMathWalker/wiremock-rs; https://crates.io/crates/wiremock | test doubles | offline refusal matrix |
| rust-embed | 8.13.0 | MIT — https://github.com/pyrossh/rust-embed (repo via crates.io) | asset embedding | single-binary UI serving |
| jsonschema / schemars | 0.60.2 / 1.2.3 | MIT (jsonschema repo `Stranger6667/jsonschema`; schemars `GREsau/schemars`) — crates.io repo URLs; spdx UNVERIFIED per-repo fetch | schema validation | envelope/schema tests |
| tower-http | 0.7.1 (and 0.6.11) | MIT — https://github.com/tower-rs/tower-http | HTTP middleware | trace/cors/limit/request-id |
| prompty | 2.1.0 | MIT? (repo https://github.com/microsoft/prompty via crates.io; spdx UNVERIFIED) | prompt templates | templates only |
| @xyflow/react (+@xyflow/system) | 12.12.1 | MIT — npm registry `license` + https://github.com/xyflow/xyflow (2026-11-06) | canvas | node/edge/minimap composition |
| react / react-dom | 19.3.1 | MIT — https://registry.npmjs.org/react/19.3.1, https://github.com/react/react | UI runtime | component model |
| zustand | 5.0.16 | MIT — https://registry.npmjs.org/zustand/5.0.16; https://github.com/pmndrs/zustand | store | client state patterns |
| motion | 14.9.1 | MIT — https://registry.npmjs.org/motion/14.9.1; https://github.com/motiondivision/motion | animation | reduced-motion-aware motion |
| partysocket | 1.3.0 | MIT — https://registry.npmjs.org/partysocket/1.3.0; repo https://github.com/cloudflare/partykit | WS reconnect | reconnect/backoff helper |
| @dagrejs/dagre (+graphlib) | 3.1.3 | MIT — https://registry.npmjs.org/@dagrejs/dagre/3.1.3; https://github.com/dagrejs/dagre | layout | DAG layout |
| vite / @vitejs/plugin-react | 8.3.6 / 6.2.1 | MIT — npm registry; https://github.com/vitejs/vite | build | bundling/dev server |
| tailwindcss / @tailwindcss/vite | 4.3.4 | MIT — npm registry; https://github.com/tailwindlabs/tailwindcss | styling | utility CSS pipeline |
| typescript | 7.0.2 | Apache-2.0 — https://registry.npmjs.org/typescript/7.0.2; https://github.com/microsoft/TypeScript | typecheck | type system |
| vitest | 5.1.0 | MIT — npm registry; https://github.com/vitest-dev/vitest | unit tests | test runner |
| mermaid (mmdc) | diagram tool | MIT — https://github.com/mermaid-js/mermaid (2026-11-06) | diagram render | SVG/PNG renders |
| Playwright (e2e) | 1.65.0 | Apache-2.0 — https://github.com/microsoft/playwright (2026-11-06) | browser QA | live evidence harness |
| w64devkit | 16.2.0 (per CHANGE-LEDGER CL-I-002) | **Unlicense** — https://github.com/skeeto/w64devkit (2026-11-06) | C toolchain for rusqlite bundled | build toolchain only |

## 3. Provenance conclusions

1. **No vendored upstream source code was found in the assigned partition.** All files are original docs/configs/assets produced by this program; the only third-party code is consumed as pinned dependencies (not copied into the repo).
2. **Design authority** is the supplied FigCraft PDF (`00 · Cover.pdf`, sha `F2D9B5AF…006A` per blueprint PROVENANCE P4), and the build plan is the pruned source of record (sha `7B85760E…D713`); this repo is a **new synthesis**, not a fork.
3. **Donor-pattern mapping** is at library level (§1/§2): idiomatic axum/tokio/petgraph/rusqlite/reqwest backend + React/xyflow/zustand/motion frontend patterns; product domain, contract envelope, state machine, guardrails, trace harness and FigCraft assets are original.
4. **License hygiene:** all researched npm packages carry declared licenses (9×MIT, 1×Apache-2.0); researched GitHub donors are MIT/Apache-2.0/Unlicense. Crate-level SPDX for crates where the crates.io API returned a null `license` field is recorded from the GitHub repos when fetched and otherwise marked UNVERIFIED.

## 4. UNVERIFIED / not researched

- crates.io API `license` field returned `null` for the sampled crates (API behaviour on this date); licence SPDX for `uuid` full text, `tokio-tungstenite`, `jsonschema`, `schemars`, `prompty`, `rust-embed` not fetched from their repos → UNVERIFIED (fetch the repo LICENSE file to confirm).
- No code-similarity/copy-detection run (e.g. against public repos); conclusion 'no vendored donor code' rests on file inventory + file content review of this partition, not a similarity scan.
- Transitive dependency licences (405 cargo / 147 npm) not individually fetched; only direct deps + key donors researched. Full licence report would require per-package registry fetches.
- Whether the private HiveMind blueprint itself derives from a public donor was not established (blueprint provenance chain documented in `hivemind-blueprint/blueprint/PROVENANCE.md`, outside this repo).
