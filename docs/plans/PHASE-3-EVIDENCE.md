# PHASE 3 EVIDENCE — Agent Context & Orchestration Handshake

Date: 2026-11-06 (America/Chicago). Executor: B-Core. Recon: `PHASE-3-RECON.md`.
Commands from `C:\Users\Riley\Desktop\hiverouter-v1\backend`, PATH = cargo + `tools\w64devkit\bin`,
`CARGO_PROFILE_DEV_DEBUG=0`, `CARGO_INCREMENTAL=0`.

## 1. Preflight

`cargo test -p hiverouter-core` compiles the Phase 3 modules with the pinned `prompty 2.1.0` crate:
`prompty v2.1.0` compiled, `hiverouter-core v0.1.0` compiled, all test executables produced, exit 0 (the full-suite notes
below are from that build). No new crate was added beyond the already-pinned workspace dependencies.

## 2. Acceptance tests (blueprint Phase 3 card)

Command `cargo test -p hiverouter-core` (full suite, observed after the disk-wipe rebuild):

```
running 34 tests        (lib: context store + handshake unit tests + all earlier modules)
test result: ok. 34 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 4.10s
...
Running tests\context_switch.rs
running 2 tests
test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.16s
Running tests\handshake.rs
running 2 tests
test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.08s
Running tests\prompt_assets.rs
running 2 tests
test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.35s
CORE_TEST_EXIT=0
```

Card command mapping:

| Blueprint acceptance command | Test | Result |
|---|---|---|
| `cargo test prompt_schema_roundtrip` | `tests/prompt_assets.rs::prompt_schema_roundtrip` (typed persona round-trip, invalid persona -> `E_PERSONA_UNKNOWN`, missing template -> `E_TEMPLATE_UNKNOWN`) | ok |
| `cargo test template_schema_roundtrip` | `tests/prompt_assets.rs::template_schema_roundtrip` (`.prompty` fixture -> typed struct -> rendered `system:`/`user:` roles; `${env:}` -> `E_TEMPLATE_REF_FORBIDDEN`; GPL licence -> `E_TEMPLATE_LICENSE_MISSING`; unknown id -> `E_TEMPLATE_UNKNOWN`) | ok |
| `cargo test rewire_context_switch` | `tests/context_switch.rs::rewire_context_switch` (previous parent's scoped facts absent from the new token set; new parent facts injected) | ok |
| error-path `malformed child result -> E_SCHEMA_INVALID` | `tests/handshake.rs::handshake_schema_roundtrip` (`{"status":"ok"}` result refused; parent gets the refusal, never fake data) | ok |
| duplicate correlation id idempotent | `tests/handshake.rs::duplicate_correlation_id_is_idempotent` (replay returns the prior frame, log length stays 1) | ok |

## 3. Implementation facts (read back from the code)

- `src/prompts.rs`: `TemplateRegistry::load(dir)` reads `manifest.json`, enforces the SPDX allowlist
  (`MIT, Apache-2.0, BSD-2-Clause/3-Clause, ISC, CC0-1.0`) plus non-empty `source_url`/`fetched_at`
  (`E_TEMPLATE_LICENSE_MISSING`), refuses `${env:...}`/`${file:...}` in user assets (`E_TEMPLATE_REF_FORBIDDEN`), then
  parses the `.prompty` file with `prompty::loader::load` inside `catch_unwind` and maps a panic/parse failure to
  `E_TEMPLATE_INVALID` (a bad asset can never kill the server). Rendering is deterministic over the typed `Agent`
  `instructions` body with `system:`/`user:`/`assistant:` role markers and `{{ name }}` substitution.
- `src/handshake.rs`: typed `HandshakeFrame {run_id, task_id, parent_task_id, depth, spawn_seq, lease_seconds,
  capability_set, correlation_id, body: Join|Ack|Result|Refuse}`; depth outside 1..=3 -> `E_SPAWN_DEPTH`; deep frame
  without a parent -> `E_SCHEMA_INVALID`; `HandshakeLog` makes duplicate correlation ids idempotent (bounded log).
- `src/context.rs`: facts are keyed `parent:<parent id>:<key>`; `switch_context` clears every old-parent-prefixed fact,
  keeps root facts, injects the new parent's facts and replaces tool allowlist/output schema; `ContextStore` is the
  per-task owner and returns `E_TASK_NOT_FOUND` for an untracked task.
- Assets added: `backend/crates/hiverouter-core/assets/prompts/{planner.prompty,manifest.json,personas.json}`.
  These assets are authored in this repository (no external fetch), so the manifest records
  `source_url: "authored in this repository (no external fetch)"` and the project MIT licence.

## 4. Deviations recorded (not silently resolved)

1. `prompty`'s pipeline `prepare()` signature is not documented on the fetched pages; role rendering is implemented in
   this crate over the typed `Agent` (deterministic + tested) instead of calling an unverified API.
2. `prompty` resolves `${env:}`/`${file:}` itself at load time; the policy layer refuses such assets **before** calling
   the loader, so the crate can never read host env/files for user templates.
3. The typed `HandshakeFrame` is in-process in this phase; it is dispatched by the Phase 4 run engine over the single
   Phase 2 socket (no second socket was added, per blueprint WP 3.5).

## 5. Done / Not done / Unverified

| Done | Evidence |
|---|---|
| prompty-based prompt asset load (typed) + licence + reference policy + deterministic role render | §2 `prompt_assets` |
| Persona schema round-trip and registry validation | §2 |
| Typed handshake frame, depth derivation, idempotent duplicates, result validation | §2 `handshake` |
| Rewire context switch clears old parent scoped facts and injects new ones | §2 `context_switch` |

| Not done (reason) | |
|---|---|
| Provider-backed prompt personalisation | Phase 5 owns the provider client; no live key in tests |

| Unverified | What would prove it |
|---|---|
| prompty Jinja2 subset conformance beyond the tested fixtures | cross-runtime conformance vectors (not documented by the crate) |
