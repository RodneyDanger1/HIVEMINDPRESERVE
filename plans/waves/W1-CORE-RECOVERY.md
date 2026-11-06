# W1 — Core recovery verification (Seat 1)

**Seat:** W1 Core recovery verification · **Repo:** `C:\Users\Riley\Desktop\hiverouter-v1`
**HEAD at recon:** `c9390f5` (dirty with 1.1 version / provider-key / truncation changes by the Lead)
**Write scope:** one new file `backend/crates/hiverouter-core/tests/recovery_gates.rs` + this report. No product edits,
no edits to existing tests, no commit. **Skills applied:** `/forgewright-recon`, `/forgewright-lead`,
`/logic-gatekeeper`, `/integration-logic`, `/logic-debugger`.

## Recon

- **Target behavior:** the optional provider-key boot check must be truthful and secret-free — it appears only when
  the provider is enabled, reports key absence as the named refusal `E_KEY_MISSING`, is never `required` (a missing key
  must degrade `/health`, not block boot), and no code path may print, serialize or log the DeepSeek key value.
- **Entry point:** `core/src/checks.rs:143` — `pub fn provider_key_check(enabled: bool, present: bool) -> Option<Check>`;
  `!enabled` → `None`; `present` → `Check::pass("deepseek_api_key", required=false, detail…)`; `!present` →
  `Check::fail(..., required=false, code "E_KEY_MISSING")`. The key value is never passed into this function: the
  signature accepts only booleans (`checks.rs:140-160`).
- **Callers:** `core/src/main.rs:102-108` — `run_checks()` then `config.provider.enabled` +
  `hiverouter_provider::ApiKey::present_in_env()` are passed to `provider_key_check`; the pushed `Check` joins the boot
  vector that is stored in `AppState` (`main.rs:123-128`) and serialized by `/health` (`http.rs` health payload).
  `ApiKey::present_in_env()` (`provider/src/secret.rs`) reads `DEEPSEEK_API_KEY` and returns a bool only; the value
  itself is only exposed by the `pub(crate)` bearer-header accessor.
- **Truncation path (reference, not re-tested here):** `core/src/provider_executor.rs:195-202` — when the provider's
  final `finish_reason` is `"length"` the adapter logs the truncation and returns `Refusal::ProviderIncomplete` instead
  of a successful reply. This rejected-truncation path is **already covered by the existing wiremock test**
  `backend/crates/hiverouter-core/tests/provider_streaming.rs:257-286`
  (`length_finish_reason_is_incomplete_not_success`): it streams a `finish_reason:"length"` chunk with usage, asserts
  `Err(Refusal::ProviderIncomplete)` and that the partial fragment was streamed before refusal. That file is part of
  the Lead's uncommitted change set and is **not edited** by this seat; this report records the reference.
- **Tests + command:** existing unit test `checks.rs:167-179` covers optional/present/absent but not serialization and
  not the env→bool seam; no integration test in `tests/` referenced `provider_key_check`. Gate command:
  `cargo test -p hiverouter-core --test recovery_gates`.
- **Data flow (success / failure):** boot env (`DEEPSEEK_API_KEY` value) → `ApiKey::present_in_env()` → **bool only** →
  `provider_key_check(enabled, present)` → `Check{name, required=false, ok, detail, code}` → `AppState.checks` →
  `/health` JSON. Failure path: absent key → `ok=false`, `code=Some("E_KEY_MISSING")`, `required=false`; the server
  still boots and live prompts later refuse `E_KEY_MISSING` (`main.rs` log line names the code).
- **Unknowns / evidence still missing:** whether `Check` serialization keeps `code` as the `E_KEY_MISSING` string (the
  struct derives `Serialize`; no test pins the wire shape) and whether any accidental value capture is possible through
  the check path (the signature says no; a canary test proves it at the integration boundary).
- **Files expected to touch:** `backend/crates/hiverouter-core/tests/recovery_gates.rs` (new) and this report only.

## Phase 1 (logic-gatekeeper) — JSON for the test mutation

```json
{
  "phase1": {
    "architecture": "hiverouter-core boots checks in main.rs (run_checks + optional provider_key_check), stores the Check vector in AppState, and serves it as /health JSON. checks.rs owns Check construction/serialization; provider/src/secret.rs owns reading DEEPSEEK_API_KEY and returns only presence booleans.",
    "dataFlow": "Untrusted/environment input DEEPSEEK_API_KEY (a secret) enters only at ApiKey::present_in_env() -> bool. provider_key_check(enabled, present: bool) cannot receive the value; its Check.detail contains the env var NAME and a refusal code only, then Check is serialized into /health.",
    "edgeCases": [
      "provider disabled: provider_key_check(false, *) must return None so no key check is reported at all",
      "provider enabled + key absent: Check.ok=false, code=Some(\"E_KEY_MISSING\"), required=false (server still boots, /health degraded not failing)",
      "provider enabled + key present: Check.ok=true, code=None, required=false; detail names the variable only, never a value",
      "a secret-shaped canary value in DEEPSEEK_API_KEY must not appear in any Check detail or serialized JSON (no sk- prefix, no Bearer, no value)"
    ],
    "rootCause": "The boot check's secrecy depends on the env→bool boundary being the only path from the key value into Check; no test currently pins that boundary or the E_KEY_MISSING wire shape, so a future refactor could pass a value-bearing string into Check.detail or serialize a wrong code without failing a gate."
  }
}
```

Phase 2 (after Phase 1 accepted): the mutation is a **test-only** gate — no product branch changes; callers
(`main.rs`, `/health`) stay stable; the contradiction (secret could reach a reported check) now fails closed because a
canary secret, if ever captured into the Check/serialized payload, makes the new test fail.

## Mutation (test-only)

New file `backend/crates/hiverouter-core/tests/recovery_gates.rs`:
1. `provider_key_check_is_optional_when_provider_is_disabled` — disabled returns `None` for both presence values.
2. `provider_key_check_states_never_expose_a_secret` — deterministic canary in `DEEPSEEK_API_KEY` (saved/restored with
   a drop guard), `ApiKey::present_in_env()` feeds the check; asserts enabled-absent and enabled-present checks, and
   their serialized JSON, never contain the canary, a `sk-` secret shape or `Bearer`; only the env var name appears.
3. `absent_key_check_serializes_named_refusal_and_is_not_required` — `Check` serialization: `name="deepseek_api_key"`,
   `required=false`, `ok=false`, `code=="E_KEY_MISSING"`; present case has `code:null`, `required=false`.
4. Reference note in the file header: truncation refusal is covered by
   `tests/provider_streaming.rs::length_finish_reason_is_incomplete_not_success` (not duplicated here; that file is
   outside this seat's write scope).

No production code is changed; no existing test is modified.

## Done / Not done / Unverified

- **Done:** recon ledger above; Phase 1 JSON; new deterministic test file with (a), (b) and the (c) reference; gate
  command run and quoted below.
- **Not done:** no fix to the existing `checks.rs` unit test (Lead's uncommitted file); no duplicate wiremock truncation
  test (existing one referenced); no commit (Lead-owned).
- **Unverified:** live `/health` JSON on a running 1.1 binary (no server start in this seat); the exact `/health`
  route's wrapping of the `checks` array (checked by reading `http.rs`/`main.rs` only); the truncation refusal under a
  real provider call (wiremock test is the coverage and is Lead-owned).

## Evidence (command + observed output)

```
$ cd backend && cargo test -p hiverouter-core --test recovery_gates --locked
running 3 tests
test absent_key_check_serializes_named_refusal_and_is_not_required ... ok
test provider_key_check_is_optional_when_provider_is_disabled ... ok
test provider_key_check_states_never_expose_a_secret ... ok

test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
RECOVERY_GATES_EXIT=0
```

First compile of the 1.1 workspace dependency graph took ~4m (`hiverouter-{contract,provider,tools,core} 1.1.0`);
the gate itself runs in 0.01s. File written (sha256):
`backend/crates/hiverouter-core/tests/recovery_gates.rs d3920c96…8652`; this report is the second deliverable.
`git status` for this seat shows exactly those two untracked paths;
all other dirty/untracked paths are pre-existing Lead/other-seat work and were not touched. No commit made.

### Gate outcome by requirement

| Requirement | Status | Evidence |
|---|---|---|
| (a) `provider_key_check` enabled/absent/present never exposes a secret | **PASS** | `provider_key_check_states_never_expose_a_secret` installs a `sk-`-shaped canary in `DEEPSEEK_API_KEY`, feeds `ApiKey::present_in_env()` (bool only) into the check, and asserts the canary, `sk-` and `Bearer` never appear in `detail` or serialized JSON for enabled-absent / enabled-present; disabled returns `None`. |
| (b) Check serialization has `code E_KEY_MISSING` and `required=false` | **PASS** | `absent_key_check_serializes_named_refusal_and_is_not_required`: `name="deepseek_api_key"`, `required=false`, `ok=false`, `code=="E_KEY_MISSING"`; the present case is `required=false`, `ok=true`, `code=null`. |
| (c) rejected truncation path covered by existing wiremock test (reference) | **PASS (reference)** | Existing Lead-owned test `tests/provider_streaming.rs::length_finish_reason_is_incomplete_not_success` (wiremock SSE with `finish_reason:"length"` → `Err(Refusal::ProviderIncomplete)`); referenced in the new test header, not duplicated or edited. |

Skill application: **recon** (entry/callers/tests/data-flow ledger above), **logic-gatekeeper** (Phase 1 JSON written
before the test mutation), **integration-logic** (env→bool→Check→/health seam traced; secret boundary pinned),
**logic-debugger** (failure/edge path: absent key is `E_KEY_MISSING` + `required=false`, not a boot failure; canary
proves no secret path), **forgewright-lead** (read code/tests before mutation; root cause stated first; tested by run;
no fake/placeholder code; commit left to Lead).

