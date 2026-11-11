# Lead research note — pseudocode accuracy and HiveMind language choices

Status: research note for the finish plan. Written 2026-11-10 at HEAD `6a41250`.
This is not a product claim and not a completion report. Every external claim names an
official source; every local claim names a repo path and line. Claims that are inference
are labelled **INFERRED**. Nothing here was copied from an aggregator or forum.

## 1. Question list

1. What languages and runtimes does the current HiveMind actually use, where, and how much?
2. Why were they selected, and what part is documented vs inferred?
3. What do the official language references say about correct/idiomatic use, and what
   concrete review rules follow?
4. How should this project read and write pseudocode so a report is accurate, falsifiable,
   and useful for implementation rather than a plausible narrative?
5. What must the watchdog/hallucination audit check in the seven lane reports and final plan?

## 2. Verified language census

Derived from `git ls-files` at HEAD `6a41250` (1,573 tracked paths):

| Extension / language | Tracked files | Actual role in this repo |
|---|---:|---|
| `.rs` — Rust | 117 | backend server, contract, provider client, tools, integration/unit tests |
| `.ts` — TypeScript | 106 | UI state, API client, flow/chat helpers, tests |
| `.tsx` — TypeScript + JSX | 72 | React panels, canvas, splash, chat UI |
| `.py` — Python | 86 | audit/design generation and documentation tooling, not the product runtime |
| `.mjs` — JavaScript modules | 19 | Node CLI, QA gates, E2E probes, trace/evidence tools |
| `.ps1` — PowerShell | 10 | build, test, release, desktop start, secure key/profile setup |
| `.cs` — C# | 1 | Windows desktop launcher (`HiveMindLauncher.cs`) |
| `.md` | 363 | docs, reports, evidence and lane reports |
| `.json` | 233 | manifests, fixtures, evidence, configuration |
| `.svg` / `.png` | 55 / 271 | FigCraft/design and UI evidence assets |
| `.figcraft` | 42 | design-source artifacts, not runtime code |
| `.html` / `.css` | embedded/built | Vite entry and Tailwind/CSS surfaces (`web/index.html`, `web/src/styles/**`) |

The product runtime is therefore **Rust + TypeScript/React/SQLite**, with **Node.js,
PowerShell, C# and Python as build/QA/desktop support**. Python is not a backend runtime
language; it lives under `audit/`, `docs/CODEMAP/_tools/`, and design-generation trees.
SQL is embedded in the Rust migrations and stores (`backend/crates/hiverouter-core/src/db.rs`).

## 3. Where each language is used, with evidence

| Layer | Language | Local evidence | What it does |
|---|---|---|---|
| Server/runtime | Rust | `backend/Cargo.toml:11-30`; `backend/crates/hiverouter-core/Cargo.toml:1-38` | Axum HTTP/WS server, Tokio async runtime, SQLite stores, provider client, tool engine |
| Contract | Rust | `backend/crates/hiverouter-contract/**` | frozen commands/events/refusals/schema |
| Provider | Rust | `backend/crates/hiverouter-provider/**` | DeepSeek/OpenAI-compatible request, streaming, tool-call assembly |
| Persistence | SQLite via rusqlite | `backend/Cargo.toml:22`; `db.rs` migrations v1–v23 | one local database file; append-only migrations; backup before upgrade |
| UI | TypeScript + React 19 | `web/package.json:6-36`; `web/src/**` | localhost SPA, panels, canvas, splash, state projection |
| UI build | Vite 8, Tailwind 4 | `web/package.json:15-40`; `web/vite.config.ts` | dev/build pipeline; production `dist` embedded by `rust-embed` |
| UI state | Zustand | `web/package.json:26`; `web/src/state/store.ts` | single UI store, WS event reducer, selectors |
| Node tooling | JavaScript `.mjs` | `web/package.json:8-9` (`node >=22.12.0`); `tools/qa/**`, `e2e/**`, `tools/hivemind-cli.mjs` | CLI, gates, UI E2E over CDP, evidence probes |
| Desktop build/setup | PowerShell | `tools/build-desktop.ps1`; `tools/set-provider-key.ps1`; `tools/start-hivemind.ps1` | build/package, DPAPI key, launch, gates |
| Launcher | C# (.NET Framework, csc) | `tools/desktop-launcher/HiveMindLauncher.cs`; `tools/build-desktop.ps1:25,47-51` | find server, choose port/workspace, inject key, start server, open Edge/Chrome app window |
| Design/audit tooling | Python | `audit/tools/**`, `docs/CODEMAP/_tools/**`, `docs/trace/figma-parity/**` | offline artifact generation and documentation census, not runtime |

## 4. Why these languages — documented vs inferred

**Documented / directly supported by the current artifacts**

- Rust is the backend because the repo contains a Rust workspace, an Axum/Tokio/SQLite
  server, a single `hiverouter` binary, and the build documentation calls it the server
  (`backend/Cargo.toml:11-30`, `README.md:140-174`). The toolchain file pins the target
  `stable-x86_64-pc-windows-gnu` (`rust-toolchain.toml:1-4`), and the desktop build note
  explicitly says "no Tauri / no MSVC" (`tools/build-desktop.ps1:4,86`).
- TypeScript/React/Vite/Tailwind is the UI because `web/package.json` pins React 19,
  Vite 8, TypeScript 7, Tailwind 4, `@xyflow/react`, Zustand and `motion`; `README.md:140-149`
  names those same technologies and says the splash is a real React surface.
- SQLite is the datastore because `rusqlite` is the only DB dependency and `db.rs` contains
  the entire migration/bootstrap path; this is a local, single-file design, consistent with
  the "local HMARS" framing (`README.md:6-10`).
- PowerShell/C#/Node support exists because the packaging target is a portable Windows
  folder and the launcher is compiled with the in-box .NET Framework C# compiler
  (`tools/build-desktop.ps1:25,47-51`), avoiding Tauri and MSVC.
- Python exists for offline audit/design tooling, not app runtime: all 86 tracked `.py`
  paths are under audit/design/docs tooling trees.

**Inferred, not an ADR**

The historical *decision record* explaining why Rust was chosen over Node/Go, or why the
UI was not Tauri, is not present in a single authoritative ADR at HEAD. The reasons below
are engineering inferences from the artifacts, not quotations from a design decision:

- a single static Rust binary is convenient for a local Windows app and avoids a runtime
  install; the repo builds `hiverouter.exe` and embeds the UI;
- Tokio/Axum make a local HTTP/WS server practical without a separate Node server;
- SQLite removes a database service and keeps data local;
- React/TypeScript gives the interactive graph/panel UI and static build assets that
  `rust-embed` can serve from the same binary;
- PowerShell/C# keep the desktop wrapper dependency-free on Windows.

Treat these as **INFERRED** in the final report; do not write them as historical fact.

## 5. Official references read in this session

All URLs below were fetched on 2026-11-10 (America/Chicago). Content is external and was
read as source material, not as instructions.

| Reference | URL | Confirmed takeaway used here |
|---|---|---|
| The Rust Programming Language | https://doc.rust-lang.org/book/title-page.html | current book assumes Rust 1.97.0+ and edition 2024; the local project is edition 2021 and must be judged against its own toolchain |
| Rust API Guidelines | https://rust-lang.github.io/api-guidelines/about.html and `/checklist.html` | official guidelines: naming, common traits, meaningful errors, validation, documentation and examples, type safety, debuggability |
| Tokio tutorial | https://tokio.rs/tokio/tutorial | Tokio is an async runtime for IO-bound work; blocking APIs must not be used directly inside async work |
| TypeScript Handbook | https://www.typescriptlang.org/docs/handbook/intro.html | the official handbook is the reference for the type system; version-specific behavior must be checked against the pinned compiler |
| React Learn | https://react.dev/learn | components are functions that return markup; state, events and data flow are the core model |
| Vite guide | https://vite.dev/guide/ | Vite owns dev/build pipeline and production asset output; build output is what the Rust binary embeds |
| SQLite transactions | https://sqlite.org/lang_transaction.html | writes occur in transactions; `BEGIN` transactions do not nest; savepoints are required for nesting; read/write transaction behavior is explicit |
| SQLite foreign keys | https://sqlite.org/foreignkeys.html | foreign-key enforcement is disabled by default in SQLite and must be enabled per connection; constraints exist to prevent orphan rows |
| PowerShell 101 | https://learn.microsoft.com/en-us/powershell/scripting/learn/ps101/00-introduction | Microsoft's own PowerShell learning material; the project targets Windows PowerShell 5.1-compatible scripts |
| C# coding conventions | https://learn.microsoft.com/en-us/dotnet/csharp/fundamentals/coding-style/coding-conventions | Microsoft's conventions; correctness and consistency first, analyzers can enforce style |

**Version truth:** `rustc 1.99.1` (observed via `~/.cargo/bin/rustc.exe --version`),
`node v26.7.0` (observed via `node --version`), `web/package.json` says Node `>=22.12.0`.
The official Rust book fetched today describes Rust 1.97.0+, which does not match the
installed compiler exactly; therefore do not cite the book as proof of 1.99-only behavior.
The project declares `rust-version = "1.89.0"` and `edition = "2021"`
(`backend/Cargo.toml:11-12`), so its compatibility floor is 1.89, not 1.99. Likewise,
TypeScript's downloaded documentation is version-labelled by its site, but the repo pins
`typescript: 7.0.2`; use the pinned compiler for truth and do not invent 7.0-only APIs.

## 6. Language-specific review rules for the finish plan

### Rust

- Respect the workspace edition/rust-version; do not introduce edition-2024-only syntax
  into a 2021 workspace.
- Follow the Rust API Guidelines: meaningful error types, `Result` propagation, public
  items documented with failure/panic notes, no `unwrap` in production paths, validated
  arguments, `Debug` for public types.
- Use Tokio for IO-bound async work; never block the runtime thread with file/DB/CPU work
  without `spawn_blocking` or a bounded worker. Use cancellation tokens and bounded
  channels for task loops.
- SQLite: parameterized statements only; transactions for multi-row writes; foreign keys
  explicitly enabled; no `.unwrap()` around DB errors in request paths; preserve named
  `E_*` refusals.
- Clippy/fmt are not currently gates (see L6 report); add them rather than assuming style
  is enforced.
- Tool/shell code is security-sensitive: allowlists are not containment by themselves
  (L2 found `cmd.exe`); validate and jail paths/args, and test escapes.

### TypeScript / React

- Strict TypeScript; avoid `any`, unchecked casts, and silent `undefined`; the UI should
  render unknown as unknown, never as a plausible zero.
- Keep the server as the source of truth: no optimistic fake success, no UI-created run
  rows; use bounded fetches, abort signals, named refusal states.
- Components are functions; keep hooks dependency arrays correct; isolate side effects;
  use one store writer; do not duplicate WS reconnect paths.
- Accessibility is part of correctness: labels, roles, focus trap/modal semantics,
  reduced-motion path, keyboard operation, contrast.
- Preserve the FigCraft/HoneyGlass token authority; additive polish only.

### SQLite

- Enforce `PRAGMA foreign_keys = ON` per connection; do not claim FK behavior from schema
  alone.
- Migrations are append-only; each needs a real down path and a pre-migration backup.
  New columns should be nullable/additive; test upgrade/downgrade and `create_all` parity.
- Use `BEGIN IMMEDIATE`/transactions for check-then-act writes; test concurrent writers.
- Do not trust `rows.len()` as a revision; use a monotonic sequence or explicit version.

### PowerShell / C#

- `$ErrorActionPreference = 'Stop'`, explicit exit codes, no secret in command line or
  output; DPAPI for provider keys; validate install paths before delete/move.
- Launcher: drain redirected stdout/stderr concurrently (the current launcher redirects
  with no readers — L6 finding), avoid fixed port assumptions, no hardcoded developer path,
  dispose processes/readers, and verify server version/hash before opening the UI.

### Node/JavaScript tooling

- QA/gates must fail on zero-collected tests, stale `E2E_BASE_URL`, and mismatched
  build/version; never treat a skipped test as a pass.
- E2E must use the real binary and ephemeral ports; keep key redaction and teardown.

### Python tooling

- Offline generation only unless explicitly requested; pin paths; no runtime import from
  the product server; label generated artifacts and source inputs.

## 7. How to read/write pseudocode accurately

There is **no single authoritative universal pseudocode standard** in the sources read.
ISO/flowchart standards describe diagrams, not a universal executable pseudocode grammar;
textbooks and courses use their own conventions. Therefore the project should use an
explicit, declared grammar rather than pretending one standard exists.

### 7.1 Required pseudocode shape

For each behavior, write:

```text
NAME: <function/command/handler>
PRECONDITIONS:
  - <state, permissions, caps, existence checks>
INPUTS:
  - name: type, bounds, source
OUTPUTS:
  - success shape, named refusal shape
STATE:
  - reads: <tables/rows/files>
  - writes: <tables/rows/files, transaction boundary>
CONTROL:
  if <condition> then
    <effect + event/audit row>
  else
    return REFUSE(<code>, <message>)
POSTCONDITIONS:
  - invariant that must hold, including "no row was created on refusal"
ERRORS:
  - <named code> when <condition>; whether state changed
CITATIONS:
  - real/path.rs:line-line for each branch/claim
UNVERIFIED:
  - what was not observed/tested
```

### 7.2 Rules that make it truthful

1. **Pseudocode describes the current code, not the intended design.** Put proposed design
   in a separate `PROPOSED:` block.
2. **Every material claim needs a `path:line` citation.** If a line cannot be found, the
   claim is `UNVERIFIED`, not "probably".
3. **Include error paths and state changes.** A pseudocode block that only shows the happy
   path is incomplete; show named refusals and whether a row/event/approval was written.
4. **State transitions must be explicit.** Use a transition table (state × event × guard →
   next state, refusal) rather than prose; this is the logic-debugger method.
5. **Null/unknown is not zero.** Mark unknown inputs/outputs as `NULL`/`UNKNOWN` with the
   evidence needed to resolve them (truthseeker grade).
6. **Separate observed / read / inferred / proposed.** Use `VERIFIED` (tool output),
   `READ` (file inspected), `INFERRED`, `UNVERIFIED`, `PROPOSED`.
7. **No invented API names, file paths, line numbers, tests or counts.** A watchdog must
   re-open every citation; a false citation is `CONTRADICTED`, not a minor error.
8. **Keep pseudocode one-to-one with code blocks.** Each block names the function it maps
   to and the code range; a block that spans multiple unrelated functions must be split.
9. **Test the explanation against behavior.** For a bug claim, state a deterministic repro
   or test that would fail; without it the claim is a hypothesis.
10. **Do not use pseudocode to smuggle in a decision.** If the report recommends a change,
    label it PROPOSED and state the affected invariant/test.

### 7.3 Reading reports for hallucination

Apply the four-skill checks in this order:

- **research:** are the unknown questions answered by local code or an official source?
- **truthseeker:** every material claim has a grade and provenance; no null-vs-zero or
  measured-vs-mock confusion; migrations/claims are atomic and honest.
- **logic-debugger:** transitions and error paths are enumerated; a bug has a repro or is
  marked a hypothesis.
- **hallucination-redteam:** every cited file/line exists; quotes are exact; paths/counts
  are not invented; unsupported is not the same as contradicted.

## 8. Claim register for this note

| ID | Claim | Grade | Evidence |
|---|---|---|---|
| R-01 | 117 `.rs`, 178 `.ts/.tsx`, 19 `.mjs`, 10 `.ps1`, 1 `.cs`, 86 `.py` tracked paths | VERIFIED | `git ls-files` at `6a41250` |
| R-02 | Product runtime is Rust + TypeScript/React + SQLite | VERIFIED | `backend/Cargo.toml`, `web/package.json`, `README.md:6-10,140-149` |
| R-03 | Python is audit/design tooling, not runtime | VERIFIED | every tracked `.py` path under audit/docs design trees |
| R-04 | Rust project declares edition 2021 and rust-version 1.89.0 | VERIFIED | `backend/Cargo.toml:11-12` |
| R-05 | Installed Rust is 1.99.1; installed Node is 26.7.0 | VERIFIED | `rustc --version`; `node --version` |
| R-06 | Official Rust Book fetched says 1.97.0+/edition 2024 | VERIFIED | fetched URL 2026-11-10 |
| R-07 | No authoritative universal pseudocode standard was found | READ | web research; no primary standard identified |
| R-08 | Reasons for Rust/React/SQLite selection are partly inferred | INFERRED | artifacts describe use; no ADR found |
| R-09 | `cmd.exe` in shell allowlist is a security issue | READ | L2 report finding; source line cited there |
| R-10 | Tests/build were not run by this note | UNVERIFIED | this note is research only |

## 9. Unknowns that must stay unknown until evidence exists

- The historical ADR for language/framework selection: **UNVERIFIED**; no single decision
  record was found.
- The exact SQLite library version compiled into the current `hiverouter.exe`:
  **UNVERIFIED**; it is bundled by `rusqlite`, but no runtime `sqlite_version()` evidence
  was collected in this note.
- Whether the current UI compiles/passes under the pinned TypeScript 7.0.2:
  **UNVERIFIED**; no typecheck/build was run for this note.
- Whether the current pinned docs match every API used in the tree: **UNVERIFIED**;
  version-specific behavior must be checked against the compiler/crate source.
- Whether the seven lane reports collectively cover all 1,573 paths: **UNVERIFIED** until
  the census/watchdog reports land.

## 10. Design implications for the finish plan

1. Do not "modernize" the Rust edition or dependencies as part of the finish; work within
   the pinned workspace, add clippy/fmt gates, and fix named correctness/security defects.
2. Keep SQLite local and transactional; add task-instruction data through an additive,
   reversible migration, with an explicit foreign-key/transaction story.
3. Keep React as a projection; fix the S1 UI wiring defects and add real states before
   adding visual polish.
4. Treat the desktop launcher as security-sensitive: drain pipes, resolve paths from the
   installed location, verify the server version/hash, and never embed the dev workspace.
5. Require every implementation PR/round to include pseudocode, `path:line` citations,
   a transition/error-path note, and a RED/GREEN test when it fixes a bug.
6. The 40-round goal must include a claim ledger and hallucination audit before final
   completion; a report that cannot be re-cited is not evidence.

## 11. Playground assessment — Rust Playground and other language playgrounds

### 11.1 What the official Rust Playground actually is

The official repository is <https://github.com/rust-lang/rust-playground> and the hosted
service is <https://play.rust-lang.org/>. Its README (fetched 2026-11-10) states:

- a React frontend communicates with an Axum backend;
- Docker containers provide the compilers/tools and help isolate them;
- it compiles debug/release against stable/beta/nightly;
- the top 100 popular crates plus Rust Cookbook dependencies are available;
- `rustfmt` and Clippy can be run;
- there is no network connection between the compiler container and the outside world;
- memory and execution time are container-limited, but disk is **not** explicitly limited.

This is a strong **snippet** runner. It is not the HiveMind workspace, not a full
`cargo test --workspace`, not the repo's private dependencies, not the SQLite/UI/runtime,
and not a stable local capability. The README does not document a stable public
"execute" HTTP API, so any integration would first require reading the server routes and
pinning/validating that contract; that API is **UNVERIFIED** from the README alone.

### 11.2 Does it improve HiveMind?

| Use | Real improvement? | Why |
|---|---|---|
| Compile/run an isolated Rust snippet with Clippy/rustfmt | **Yes, modest** | official, low setup, useful for syntax/API experiments and teaching examples |
| Verify HiveMind's own workspace/code fix | **No** | playground has no HiveMind workspace, no `Cargo.lock`, no repo modules, no SQLite/DB, no web/UI; it only sees submitted source + permitted crates |
| Reproduce an integration/tool/agent bug | **No** | cannot start HiveMind, use its data dir, exercise approvals, or run its tests |
| Debug a dependency/version issue | **No/weak** | playground versions and crate set are not HiveMind's pinned lockfile |
| Test a UI/TypeScript behavior | **No** | the Rust Playground is not a TypeScript/browser runner |
| Keep code local/private by default | **No** | submitting source to an external service sends it off-machine; approval cannot un-send it |
| Availability/reproducibility | **Weak** | external rate limits, service versions, and network dependency; not part of the local build |

The decisive point: HiveMind already has `file_read`, `file_list`, `shell_exec` and
`cargo`/`git` allowlisted. The missing capability is not "compile a snippet"; it is
**safe, structured, repo-local verification** with real tests, approval, and cited output.
A playground does not close that gap.

### 11.3 Other "playgrounds"

- **TypeScript Playground** (<https://www.typescriptlang.org/play>) is an official
  browser tool, but the useful HiveMind check is `tsc --noEmit`/`vitest` against the real
  `web/` tree; the playground cannot see the repo or the embed contract.
- **Node/JavaScript** has no single official playground comparable to Rust's; local
  `node` plus the repo tests are authoritative.
- **SQLite** work must run against the real migrations and database; a web fiddle would
  not validate `db.rs`, WAL, backups, or HiveMind's schema.
- **Python/C#/PowerShell** are support/tooling languages here; local `python`, `csc`,
  and PowerShell are the correct executors.
- **Compiler Explorer** (Godbolt) is official for many compilers, but it is a third-party
  remote service with the same privacy/availability/version problems; useful for
  isolated assembly/compiler questions, not HiveMind self-verification.
- Self-hosting Rust Playground inside HiveMind would add Docker/container infrastructure
  to a portable Windows app and a large new attack surface; it is not justified by the
  finish plan's goals.

### 11.4 Recommendation

Do **not** make external playgrounds a core HiveMind capability now. The logical
priority order is:

1. **P0 local verification tool** (proposed): an approval-gated, structured
   `verify_code`/`sandbox_check` tool that runs a fixed set of commands (e.g. `cargo
   check/test/clippy/fmt`, `tsc --noEmit`, `vitest`, `node --test`) in a temporary
   workspace copy via the existing Job Object, with no network, bounded time/output,
   and a structured result. This supports snippets without trusting an external service
   and reuses the jail/shell infrastructure after its P0 fixes.
2. **Optional external playground connector** (only if the user explicitly wants it):
   a `playground_run` tool with `Network` permission, per-call operator approval, a
   fixed allowlist (`play.rust-lang.org`), bounded code size, no secrets/PII, no repo
   source by default, a documented API version, and a clear "source leaves this machine"
   warning. It must be disabled by default and never used for HiveMind self-verification.
3. **No** TypeScript/Node/SQLite/Python/C# playgrounds: the local toolchains are more
   accurate for this repository. Compiler Explorer remains a research-only external
   service, not a product dependency.

This conclusion is **INFERRED** from the official Rust Playground architecture and the
verified HiveMind tool/security model, not from a benchmark. If the user wants a
playground, the correct first acceptance test is: "compile a known-good and known-bad
snippet, prove the bad one fails, prove no key/secret is sent, and prove the tool is
unavailable by default." A playground that passes only the happy path is not evidence.

## 12. Playground claim register

| ID | Claim | Grade | Evidence |
|---|---|---|---|
| P-01 | Official Rust Playground is React frontend + Axum backend + Docker compilers | VERIFIED | official README, fetched 2026-11-10 |
| P-02 | Compiler container has no outbound network | VERIFIED | official README "Resource Limits / Network" |
| P-03 | Top 100 crates + Cookbook dependencies; rustfmt/Clippy; stable/beta/nightly | VERIFIED | official README "What's it do?" |
| P-04 | No disk limit is documented in that README | VERIFIED | official README "Resource Limits / Disk" |
| P-05 | Playground has no stable documented execute API in README | UNVERIFIED | README does not document one; routes must be read before integration |
| P-06 | A playground cannot test HiveMind's repo/DB/UI integration | VERIFIED / logical | it compiles submitted snippet + permitted crates only; no workspace |
| P-07 | Local `cargo`/`tsc`/`vitest` are the authoritative repo checks | VERIFIED | `backend/Cargo.toml`, `web/package.json`, existing test/gate scripts |
| P-08 | External playground sends source off-machine | VERIFIED / logical | it is a remote service; no local execution |
| P-09 | Adding playgrounds is not a core requirement for the finish plan | INFERRED | security/privacy/availability tradeoff vs existing local toolchain; not measured |
