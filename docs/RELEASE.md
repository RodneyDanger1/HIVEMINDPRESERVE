# HiveMind 1.1.0 — release notes

## What this release is

HiveMind 1.1.0 is the second official build of the localhost HiveMind runtime: one Rust server binary + embedded
React UI, one CLI, a real DeepSeek provider path, and **true SSE streaming**. `SUBMIT_PROMPT` reaches the provider,
the planner returns a model-authored DAG, and the leaf reply is delivered as ordered `REPLY_DELTA` frames as it is
generated (not as a single post-completion delta). At seal time, `1.1.0` was declared in `web/package.json` and all
four `backend/crates/*/Cargo.toml`. This document describes the sealed 1.1.0 release only; the working tree has since
moved to an unreleased 1.2 pre-release line (status and evidence: `docs/RELEASE-EVIDENCE.md`). No 1.2 feature is
released, built, or end-to-end verified.

## Verified in the 1.1.0 build

- **True SSE streaming.** The provider adapter streams the completion and forwards each text fragment to the delta
  sink (`backend/crates/hiverouter-core/src/provider_executor.rs` `stream_reply`); usage settles from the documented
  final chunk; malformed/incomplete streams refuse by name and are never replayed after the first fragment.
  Offline evidence: `backend/crates/hiverouter-core/tests/provider_streaming.rs` (wiremock; ordered fragments,
  cancellation, pre-first-byte retry) and `docs/evidence/1.1-backend-tests.log`.
- **Replay browser proof.** `HISTORY_REVISION_SET` returns a read-only `HISTORY_SNAPSHOT`; the web canvas projects
  snapshot nodes/edges and the replay layout is browser-proven in the 1.1 wave (live canvas never mutates).
- **State parity.** Backend and web vocabularies are 16 states including `interrupted`.
- **Provider wiring, budget and key custody.** `hiverouter-provider` is a dependency of `hiverouter-core`; a missing
  key or disabled provider falls back to the honest `E_KEY_MISSING` executor. `DEEPSEEK_API_KEY` is environment-only,
  never stored/logged/echoed; zeroized/redacted provider type; child processes receive an allowlisted environment.
- **Reasoning controls.** `HIVEROUTER_MODEL`, `HIVEROUTER_THINKING`, `HIVEROUTER_LEAF_THINKING`,
  `HIVEROUTER_REASONING_EFFORT`, `HIVEROUTER_MAX_OUTPUT_TOKENS`, `HIVEROUTER_PLANNER_MAX_OUTPUT_TOKENS`,
  `HIVEROUTER_PROVIDER` (auto/disabled). Settings shows model, thinking, effort and key presence read-only.
- **Budget.** Per-run `RunBudget` ($0.50 cap, 40 calls, 8 in-flight); absent usage is INCOMPLETE, never 0;
  401/402 are not retried.
- **Exact artifact hashes** are recorded only by the sealed 1.1.0 build in `docs/RELEASE-EVIDENCE.md` and the
  authoritative `tools/release-manifest.json` (`tools/build-release.ps1`). This document deliberately does not
  hand-copy hashes.

## Known limitations / not claimed in 1.1.0

- Autonomous model tool-calling is a recorded decision for a later wave; tools/approval/jail remain available to the
  application surfaces, but the model does not yet drive multi-turn tool calls in 1.1.0.
- A paid live streaming/replay smoke was run on the sealed 1.1.0 binary and recorded: 5 passed / 0 failed, exit 0
  (`docs/evidence/live-1.1-paid.log`; frames in `docs/trace/live/captures/w11/`). **No 1.2 live paid run exists.**
- The 1.2 pre-release working tree (approval suspension, side effects, web replay, live specs) is unverified WIP; it
  is not part of this release and no 1.2 feature is claimed.
- Full trace ladder, pixel-level design fidelity, Tauri/installer and every P2 audit item remain out of scope.
- No key material is in the repository or release artifacts. Any key supplied in a chat transcript should be rotated.

## Install / run

1. Set `DEEPSEEK_API_KEY` in the environment of the server process (never in a committed file, never in the UI).
2. Run `hiverouter.exe`; it binds loopback only (`127.0.0.1:8080` by default).
3. Open the printed URL. `/health` and the Settings panel report key presence (boolean only).
4. Use `hiverouter-cli health/diagnose` for status.

## Release gate

The sealed 1.1.0 build exists and its release record is `tools/release-manifest.json` plus `docs/RELEASE-EVIDENCE.md`.
`v1.0.0` (the historical private tag) stays immutable and `hivemind-1.1.0` remains the released tag/source. **No 1.2
tag, artifact, manifest, live proof, or release-gate PASS exists**; the 1.2 release build is blocked until the 1.2
fixes and gates pass (`plans/foresight/COMPILED-LEDGER.md`, `plans/foresight/PROOF-RELEASE-PLAN.md`).

<!-- docs-truth:historical:start -->
## Historical: HiveMind 1.0.0 (SUPERSEDED)

- 1.0.0 delivered the first provider-wired build; its exact artifact hashes, release-binary smoke and the retired
  limitation (leaf reply emitted as one delta after a non-streaming completion; true SSE tracked for 1.1) are kept
  in the historical section of `docs/RELEASE-EVIDENCE.md`. They are historical only and are not current claims.
<!-- docs-truth:historical:end -->
