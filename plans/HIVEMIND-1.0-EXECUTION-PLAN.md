# HiveMind 1.0 — final execution plan and release record

**Status:** built and release-gated on sealed source commit `6a2b1c6`; artifacts committed at `b2ac2b9`; official tag `hivemind-1.0.0` (the old private `v1.0.0` tag is left untouched to avoid a destructive history rewrite). This plan is the synthesis of the 8 analysis seats (`plans/seats/SEAT-01..08`) and 4 planners (`plans/planners/PLANNER-01..04`), each of which read the audit paper, the 4 official DeepSeek docs, and its assigned files in full.

## 1. Outcome

HiveMind 1.0 is the first official build whose live path is real and evidenced:

- `hiverouter-core` depends on `hiverouter-provider`; `build_executor` wires `ProviderExecutorAdapter` when `DEEPSEEK_API_KEY` is present and falls back to the honest `E_KEY_MISSING` executor otherwise.
- `SUBMIT_PROMPT` → planner DAG → leaf call → reply → truthful usage → terminal run status; release-binary smoke produced `HIVE-ARTIFACT-OK`, `completed`, usage 99 tokens.
- `HISTORY_REVISION_SET` returns a real `HISTORY_SNAPSHOT`; web `buildReplayTasks` renders it read-only; 16-state parity includes `interrupted`; `failed` is terminal; replay nodes are non-draggable/non-selectable.
- Key custody is env-only, zeroized/redacted, never committed/logged/rendered, and **removed from spawned tool children** (`shell.rs`, `cli_support.rs`) with a regression test.
- Release artifacts: server `b17eb3d6…`, CLI `49f46e2b…`, zip `a49dbb2b…`; manifest with source commit/toolchain/web-dist hash in `tools/release-manifest.json`.
- Sealed-commit gates: Rust **237 passed / 0 failed / 2 ignored**, web **46/46**, typecheck/build green (`docs/evidence/final-*`).

## 2. Current behavior and decisions (from the 12 reports)

| Area | Decision / state |
|---|---|
| Provider wiring (A-F01) | Fixed in source: core dep + adapter + key fallback. Live proof in `docs/evidence/live-provider-smoke.md`; artifact proof in `docs/RELEASE-EVIDENCE.md`. |
| Replay (A-F02) | Fixed: backend arm emits snapshot; `Canvas.tsx` projects it; `graph.ts` read-only in replay; no synthetic nodes. Browser-level replay test remains 1.1 (P2). |
| State parity (A-F03) | Fixed: 16/16 incl. `interrupted`; `isTerminal` now includes `failed` (S3 D1). |
| Key custody | Env-only, `.env*` ignored, zeroize/redaction; child spawns remove the key; tracked-tree scan 0 hits. `env_clear` allowlist hardening (P1 recommendation) remains 1.1. |
| Streaming | **Not claimed in 1.0**: leaf reply is one delta after non-streaming completion. True SSE delta publishing (`stream_options.include_usage`, ordered `REPLY_DELTA`) is 1.1. |
| Budget | Per-run budget/call cap; budget map evicted on run finish; usage absent → INCOMPLETE. Wiremock error-path coverage is still required (S7-F05). |
| Planner robustness | Terse prompt was hardened by wrapping the user task in a DAG instruction; one bounded planner retry on empty/invalid JSON is 1.1. |
| UI truthfulness | Settings shows model/thinking/effort/key; settings refusal source fixed; UI `KNOWN_REFUSAL_CODES` is 45/65 → generated drift test is 1.1; `NODE_REWIRE` still hardcodes `running` → 1.1 (needs read-only graph accessor). |
| Release provenance | Fixed by this record: sealed source commit, rebuild, artifact hashes, manifest, HEAD-bound gate logs. Old `v1.0.0` tag remains at the audit checkpoint; no force-move. |

## 3. Release artifact record

| Artifact | SHA-256 | Bytes |
|---|---|---:|
| `hiverouter.exe` | `b17eb3d6469c347f623a4dedc94537b15bb633a76215828742624fa6fe062a9a` | 13,253,120 |
| `hiverouter-cli.exe` | `49f46e2be488dd675280f3dae7e8d455ac5eeec2aa46d7fd44f1c2557c386623` | 5,373,952 |
| `dist/HiveMind-1.0.0-win64.zip` | `a49dbb2bda65d0c3ad500157e04960638c54e2b08df8c8da5885583cf0e895e8` | 8,444,038 |
| source commit | `6a2b1c6d461a81e7ee66951d0d0eb6b1063bf540` | — |
| tag | `hivemind-1.0.0` (annotated; old `v1.0.0` untouched) | — |

## 4. Failure modes proven or explicitly bounded

- `E_KEY_MISSING` when key absent/provider disabled; `E_PROVIDER_AUTH` on 401/402 (no retry); `E_LLM_RATE_LIMIT` on exhausted 429; `E_PROVIDER_TIMEOUT`; `E_BUDGET_EXCEEDED` pre-send; `E_SCHEMA_INVALID` for malformed plan; `E_USAGE_INCOMPLETE` when usage is absent (never 0); cancellation returns `E_TOOL_CANCELLED` without late events.
- Child tools cannot read `DEEPSEEK_API_KEY` (test pinned).
- Replay malformed snapshot yields an empty graph rather than invented nodes; unknown states are skipped rather than rendered as a false state.
- Not yet proven: live 401/402/429/timeout mapping end-to-end, true SSE ordering, browser replay pointer behavior, 45/65 UI refusal code parity, `NODE_REWIRE` real state. These are named in the 1.1 roadmap, not claimed.

## 5. 1.1 roadmap (from planners, prioritized)

1. **W1 Evidence binding** — JUnit XML/exit code/HEAD for every gate; no pass counts read from prose; stale audit claims get a dated supersession block.
2. **W2 Provider run-path matrix** — wiremock/stub-provider tests for missing key, 401/402, 429, timeout, budget, malformed plan, usage absent; then one operator-gated live smoke on the rebuilt artifact.
3. **W3 True streaming** — SSE `data:`/`[DONE]`, `stream_options.include_usage`, ordered `REPLY_DELTA` publishing; no fake chunking.
4. **W4 Replay/state browser proof** — scrubber→snapshot→read-only nodes; 16-state/failed terminality; malformed-snapshot banner; no interactive replay nodes.
5. **W5 Contract/UI parity** — generate `KNOWN_REFUSAL_CODES` from `refusal.rs` (45→65) with drift test; make `NODE_REWIRE` emit the real graph state via a read-only accessor.
6. **W6 Key hardening** — replace key-only removal with a minimal environment allowlist (`env_clear` + PATH/SystemRoot/TEMP) after security sign-off; add child-env canary across every spawn site.
7. **W7 Release engineering** — one deterministic `build-release.ps1` that rebuilds web+binaries, stages dist, writes CHECKSUMS/manifest with commit/toolchain/web-dist/provider fields, verifies extraction, and fails on any mismatch; no absolute paths.
8. **W8 Docs truth** — update/supersede PHASE-12, FINAL-REPORT, claims C-340, README old zip hash, 15-state/260-5/“no paid run” statements; add a checker that fails on known stale hashes.
9. **W9 Trace/G-UI completeness** — implement or explicitly exclude the 12 skipped DOM gates; complete the trace ladder; no skip-as-green.
10. **W10 Tool-call loop and Tauri** — explicit product decisions, not silent scope creep.

## 6. Verification program (already run for this release)

- `cargo test --workspace --locked` on `6a2b1c6`: 237 passed / 0 failed / 2 ignored; log includes HEAD/command/exit (`docs/evidence/final-gate-tests.log`).
- `npm test -- --run`: 46/46; typecheck 0; production build green (`final-web-tests.log`, `final-web-build.log`).
- Provider adapter live test: planner + leaf, `HIVE-PROVIDER-OK`, usage 99 tokens (ignored by default).
- Release binary: health `ok` v1.0.0; live `HIVE-ARTIFACT-OK` completed usage 99; settings model/thinking/effort/key; replay 2-node snapshot; key scan 0 hits.
- Package extraction: six members, checksums present; zip hash recorded.

## 7. Rollback

Revert `b2ac2b9`/`6a2b1c6` release commits; keep `hivemind-1.0.0` immutable and publish a retraction/1.0.1 if a defect is found. `HIVEROUTER_PROVIDER=disabled` restores the honest offline executor without a code revert. The old private `v1.0.0` tag is untouched; no destructive tag move occurred.

## 8. Out of scope / blockers

- Model-driven multi-turn tool calls, true SSE streaming, Tauri/installer, pixel-fidelity review, full trace ladder, and every P2 audit item are 1.1.
- The old `v1.0.0` tag points at `3988622` (pre-fix); treat it as a private audit checkpoint, not the official release.
- The key was exposed in chat; rotation after release is recommended. No key material is in the repository, package, logs, or evidence.

## 9. Seat/planner evidence index

- `plans/seats/SEAT-01..08/` — reports, READLOGs, evidence, 3-cycle gates.
- `plans/planners/PLANNER-01..04/` — plans, 3-cycle gates.
- `plans/HIVEMIND-1.0-SEAT-ASSIGNMENTS.md` — mandatory reading assignments.
- `audit/HIVEMIND-AUDIT-PAPER.md` — pre-integration audit (P0s now source-fixed; see supersession).
- `docs/RELEASE-EVIDENCE.md`, `docs/evidence/*` — release proof and gate logs.
