# L6 — Test suites, QA/build/release/install tooling, E2E, evidence & gate integrity

**Lane:** 6 of 7 · **Repo:** `C:\Users\Riley\Desktop\hiverouter-v1` · **Date of analysis:** 2026-11-10
**Analyst mode:** READ-ONLY. No source, config, or test file was modified; no server was started; no provider was called; no key was read or printed. This document is the only artifact written.
**Tree analysed:** `HEAD = 6a4125070186da1bbe3f088b3634f3f6d5402fab` (`6a41250`), web/crate version `1.2.0-dev`.

> **Status legend used throughout**
> **VERIFIED** — I read the file/line or executed a read-only command and the quoted text/output is the evidence.
> **UNVERIFIED** — structurally present but I did not execute it in this lane, so no pass is claimed.
> **PROPOSED** — my recommendation; not present in the tree.
> **Rule compliance:** nothing here is a mock/demo claim. Every "passes" claim from a committed log is attributed to that log, not adopted as my own run.

---

## 0. Verdict in one paragraph

The test and release machinery in this repo is unusually good on **honesty mechanics** (named skips, no-false-green guards, sealed hash registries, redaction, fail-closed preflight) and unusually weak on **execution freshness and install identity**. There are **559 declared Rust test functions** (373 unit + 186 integration), **236 declared web tests**, **95 declared Playwright test sites**, and a 15 Hz `run-gates.ps1` G0–G7 harness that is genuinely fail-closed. But: (a) the newest committed gate evidence is `gate-7969f5b9-offline.json` at commit `7969f5b9`, dated 2026-11-09T00:19, i.e. **one day and at least one commit behind the tree being finished**; (b) **no gate runs the three `tools/qa/tests/*.test.mjs` guard suites at all**; (c) on the exact current tree, `run-gates.ps1` **cannot pass** — strict mode dies at G0 on a dirty tree, `-Rehearsal` dies at G7 on `target/release` + `web/dist` hash drift, and only `-Release` on a documented pre-release tree (exit **3**, "not a release") can go green; (d) there is **no install smoke that proves the Desktop-installed binary is the current build** — four different `hiverouter.exe` builds exist on disk with four different hashes, and the launcher neither verifies nor records which one it ran. Those are the lane's headline findings.

---

## 1. Coverage map — every file in scope

### 1.1 Backend integration tests (`backend/crates/hiverouter-core/tests/**`) — 28 files, 186 test fns

| File | Test fns | `#[ignore]` | LOC | Kind |
|---|---|---|---|---|
| `approval_ledger.rs` | 3 | 0 | 216 | ledger durability |
| `chat_store.rs` | 5 | 0 | 150 | SQLite store |
| `context_switch.rs` | 2 | 0 | 69 | planner context |
| `daemon_tick_degraded.rs` | 2 | 0 | 99 | degraded daemon |
| `graph_authority.rs` | 7 | 0 | 121 | graph authority |
| `handshake.rs` | 2 | 0 | 79 | WS handshake |
| `http_boot.rs` | 28 | 0 | 1409 | HTTP surface |
| `hyperagent_governance.rs` | 4 | 0 | 523 | governance/approval |
| `kill_restore.rs` | 3 | 1 | 118 | crash/restore (child proc) |
| `lead_bugcheck_boot.rs` | 1 | 0 | 127 | boot regression |
| `mem_isolation.rs` | 4 | 0 | 248 | workspace isolation |
| `prompt_assets.rs` | 3 | 0 | 137 | asset loading |
| `prompt_assets_wiring.rs` | 11 | 0 | 274 | asset wiring |
| `proposal_authorization.rs` | 1 | 0 | 271 | proposal authz |
| `provider_adapter.rs` | 9 | 0 | 310 | provider adapter |
| `provider_live.rs` | 1 | 1 | 94 | **paid live call** |
| `provider_streaming.rs` | 7 | 0 | 287 | SSE streaming |
| `recovery_gates.rs` | 3 | 0 | 120 | recovery gates |
| `recovery_multi_run.rs` | 1 | 0 | 162 | multi-run recovery |
| `remote_control_plane.rs` | 7 | 0 | 383 | remote plane |
| `run_lifecycle.rs` | 27 | 0 | 1962 | run state machine |
| `scale_s0.rs` | 1 | 0 | 122 | scale S0 |
| `scale_s1.rs` | 2 | 0 | 190 | scale S1 |
| `snapshot_history.rs` | 6 | 0 | 291 | snapshots |
| `suspension.rs` | 16 | 0 | 1159 | suspension/resume |
| `tool_loop_engine.rs` | 12 | 0 | 746 | tool loop |
| `tool_loop_matrix.rs` | 5 | 0 | 474 | tool loop matrix |
| `ws_transport.rs` | 13 | 0 | 544 | WS transport |
| `support/mod.rs` | 0 (harness) | 0 | 352 | in-process server + RFC6455 client |

**Counts VERIFIED** by regex count of `#[test]` / `#[tokio::test]` per file. 186 total; 2 ignored (`kill_restore.rs:21`, `provider_live.rs:19`).

### 1.2 Meaningful unit-test modules (`#[cfg(test)]` in `src/`) — 58 modules, 373 test fns

| Crate | Modules with `#[cfg(test)]` | Test fns |
|---|---|---|
| `hiverouter-contract` | `command.rs` 5, `event.rs` 10, `refusal.rs` 3, `state.rs` 2, `timestamp.rs` 5, `usage.rs` 2 | 27 |
| `hiverouter-core` | `run_state.rs` 20, `store.rs` 28, `tools_bridge.rs` 12, `db.rs` 11, `execution.rs` 10, `recovery.rs` 10, `planner.rs` 9, `ws.rs` 9, `provider_executor.rs` 8, `context.rs` 7, `config.rs` 7, `daemon.rs` 6, `tool_loop.rs` 6, `checks.rs` 5, `approval_wait.rs` 4, `bus.rs` 4, `graph.rs` 4, `handshake.rs` 4, `remote.rs` 4, `remote_supervisor.rs` 3, `caps.rs` 2, `http.rs` 2, `static_ui.rs` 2 | 177 |
| `hiverouter-provider` | `tool_assembler.rs` 17, `dto.rs` 10, `budget.rs` 9, `accounting.rs` 4, `client.rs` 3, `pricing.rs` 3, `error.rs` 2, `secret.rs` 2 | 50 |
| `hiverouter-tools` | `approval.rs` 18, `cli.rs` 14, `jail.rs` 14, `side_effects.rs` 10, `executor.rs` 9, `ledger.rs` 7, `shell.rs` 7, `tool.rs` 6, `cancel.rs` 4, `census.rs` 4, `commands.rs` 4, `cli_support.rs` 3, `db_admin.rs` 3, `audit.rs` 2, `env_safety.rs` 2, `health.rs` 5, `settings.rs` 2, `sha256.rs` 2, `watcher.rs` 2, `error.rs` 1 | 119 |

**373 VERIFIED** by regex over `src/**/*.rs` filtered to files containing `#[cfg(test)]`.

### 1.3 Web tests — 31 files, 236 test sites

`web/tests/`: `agentState` 5, `agentsToolActivityMount` 12, `agents_spawn_matrix` 8, `apiBoundary` 7, `apiClient` 7, `apiLifecycle` 15, `approval.integration` 9, `approval.state` 32, `chatMotion` 4, `client.strictmode` 1, `contract` 10, `designParity` 7, `failure` 4, `graph.replay` 8, `health` 5, `keyboard` 7, `layout_hive` 12, `lessons` 2, `metrics` 3, `noHardcoded` 5, `proposals` 2, `refusalParity` 3, `state.parity` 8, `store.spawn-replay` 3, `store.strictmode` 3, `store` 9, `streamingRender` 6, `tokens` 6, `toolActivity.integration` 5, `toolActivity.store` 16, `toolActivityPanels` 12 → **236**, zero `it.skip`/`test.skip` (**VERIFIED**).

Test config: `web/vite.config.ts:33-37` — `environment: 'node'`, `include: ['tests/**/*.test.ts']`, reporters `['default']`. There is **no** `jsdom`/`happy-dom` environment (**VERIFIED** by reading the config; no `environment: 'jsdom'` anywhere in it).

### 1.4 E2E — 16 specs, 95 test sites, 48 gated skips

| Spec | Tests | skip sites | Role |
|---|---|---|---|
| `live-app.spec.ts` | 20 | 15 | live app surfaces (skips when app absent) |
| `gui-gates.spec.ts` | 16 | 0 | G-UI gates vs static `web/dist` + mocked WS |
| `gui-residual.spec.ts` | 6 | 0 | loading/thinking/streaming/reconnecting |
| `approval-flow.spec.ts` | 8 | 0 | approval UI |
| `tool-loop.spec.ts` | 8 | 0 | tool loop UI |
| `control-matrix.spec.ts` | 7 | 1 | matrix integrity |
| `live-security.spec.ts` | 6 | 6 | G-UI-3/W1 origin+token |
| `live-trace-capture.spec.ts` | 5 | 6 | trace capture |
| `live-resync.spec.ts` | 4 | 4 | G-UI-10/W2 resync |
| `live-tool-turn.spec.ts` | 4 | 9 | paid tool turn |
| `live-streaming.spec.ts` | 3 | 3 | paid streaming |
| `live-replay.spec.ts` | 2 | 2 | paid replay |
| `trace-hops.spec.ts` | 2 | 1 | 11-hop capture |
| `a11y-gates.spec.ts` | 2 | 0 | contrast math (real) |
| `replay-state.spec.ts` | 1 | 0 | replay state |
| `ws-contract.spec.ts` | 1 | 1 | WS stub (excluded) |

Helpers: `app.ts`, `contrast.ts`, `evidence.ts`, `live-server.ts`, `matrix.ts`, `replay-harness.ts`, `trace.ts`, `ws-tap.ts` (**VERIFIED** listing). Config `e2e/playwright.config.ts`; runner `e2e/run-guarded.mjs`.

### 1.5 Tooling in scope

`tools/qa/`: `run-gates.ps1` (753 L), `check-docs-truth.mjs`, `check-seal-topology.mjs`, `verify-harness.mjs`, `apply-live-trace.mjs`, `build-trace-seeds.mjs`, `figcraft-assets.mjs`, `ui-e2e.mjs`, `check-env.ps1`, `apply_wp01.py`, `provider-test.log`, `README.md`, `tests/seal-topology.test.mjs`, `tests/trace-tools.test.mjs`.
`tools/`: `build-release.ps1` (45 KB), `build-desktop.ps1`, `test.ps1`, `start-hivemind.ps1`, `set-provider-key.ps1`, `set-provider-profile.ps1`, `hivemind-cli.mjs`, `hivemind.ps1/.cmd`, `build_assets_index.py`, `release-manifest.json`, `runbook.md`, `bin/`, `desktop-launcher/{HiveMind.exe,HiveMindLauncher.cs,README.md}`, `cc-test/`, vendored `w64devkit/` (out of scope).
`e2e/**`, `runtime-test/**` (logs, 4 package dirs, 9 data dirs), `ledgers/{ui-watchdog.json,ui-watchdog.md}`, `dist/**` (4 packages + 3 `.release-manifest-*.json` backups), `docs/evidence/**` (~100 files).

**Explicitly NOT in scope / not inspected:** `tools/w64devkit/**` (vendored toolchain, ~14k files), `target/**` beyond hashes, `e2e/node_modules/**`, `e2e/playwright-report/**` internals.

---

## 2. Backend integration tests — what is actually verified

### 2.1 The harness is real, not a fake (`tests/support/mod.rs`)

- **VERIFIED** `support/mod.rs:42-92` — `TestServer::start` resolves a **real** `Config` (`:55`), opens a **real** SQLite `Database` and runs **real** migrations (`:63-64`), builds the **real** `AppState` (`:65-75`), binds a **real** `TcpListener` on `127.0.0.1:0` (`:76`) and serves the **real** `http::serve` (`:81`). No mock server, no stub router.
- **VERIFIED** `support/mod.rs:104-153` — HTTP handshake is hand-rolled over a real socket; `:171` shows the token is passed as a real query param (`?token=…&last_seq=…`).
- **VERIFIED** `support/mod.rs:156-318` — a complete minimal RFC6455 client (masked client frames at `:233-255`, extended lengths `:262-273`, ping→pong at `:219-221`, close at `:218`). This is real protocol framing, not a fake transport.
- **VERIFIED** `support/mod.rs:95-102` — `Drop` sends the shutdown oneshot **and** aborts the join handle: no leaked server tasks between tests.
- **Port/lease risk: LOW.** `:76` binds port `0` (ephemeral) and `:77` reads the assigned addr back. No fixed ports, therefore no cross-test port collision. Compare `tools/qa/ui-e2e.mjs` and the launcher, which both hardcode `8099` (§6.4, §11).

**Test doubles present (legitimate, §7-permitted):** SQLite temp-file DB (real engine, temp dir), `tempfile::TempDir`, tokio timeouts, `wiremock`-class HTTP doubles in the provider crate (see §3.1). **No mock of the unit-under-test's own logic appears in any integration file I read.** **No `NODE_ENV=test`-style fabricated-data branch** was found in the harness.

### 2.2 Suite-by-suite substance

| Suite | What is actually verified (file:line evidence) |
|---|---|
| `http_boot.rs` (28) | Real HTTP boot surface: routing, auth, refusal codes, health/session shapes against `AppState` (`http_boot.rs:1-1409`, 28 tests). Largest HTTP contract suite. |
| `run_lifecycle.rs` (27) | Run state machine transitions end-to-end (`run_lifecycle.rs:1-1962`). Largest lifecycle suite; pairs with `src/run_state.rs` 20 unit tests. |
| `suspension.rs` (16) | Suspend/resume across **process restarts** — the file's own module doc (`suspension.rs:39`) explicitly says the gap "stays active (no `#[ignore]`) so the gap is visible". Honest-by-construction. |
| `ws_transport.rs` (13) | Real WS frames over `WsClient`: hello, replay, ordering, refusals (`ws_transport.rs:1-544`). |
| `tool_loop_engine.rs` (12) + `tool_loop_matrix.rs` (5) | Tool-loop execution + matrix coverage. |
| `prompt_assets_wiring.rs` (11) | Asset→prompt wiring (11 cases). |
| `provider_adapter.rs` (9), `provider_streaming.rs` (7) | Adapter + SSE streaming, **without** live calls. |
| `provider_live.rs` (1) | **The only paid test.** `provider_live.rs:19` `#[ignore = "live DeepSeek provider call; set DEEPSEEK_API_KEY and run with --ignored"]`. Correctly ignored by default. |
| `kill_restore.rs` (3, 1 ignored) | `kill_restore.rs:21` ignores the helper that is "spawned by the kill_restore test as the crashing writer process" — a legitimately ignored harness entry point, not a weakened assertion. |
| `scale_s0.rs` (1) / `scale_s1.rs` (2) | Scale gates; correlate with `docs/evidence/scale-s0-9c0b8fad-offline.json`, `scale-s1-cf8bff0f-offline.json`. |
| `mem_isolation.rs` (4) | Jail/workspace isolation (pairs with `hiverouter-tools/src/jail.rs` 14 unit tests). |
| `remote_control_plane.rs` (7) | Remote plane (pairs with `src/remote.rs` 4 + `remote_supervisor.rs` 3). |

### 2.3 Gaps in the integration layer

1. **No single test exercises the packaged/installed binary.** Every `hiverouter-core` integration test builds the crate into the test harness; nothing runs `dist/HiveMind/hiverouter.exe` or `target/release/hiverouter.exe`. The only binary-level smoke in the repo is `runtime-test/pkg-final/smoke.out` (a hand-recorded 2026-11-06 log, §8.1).
2. **No test asserts build/version identity** (e.g. that `/health.version` equals the crate version). `runtime-test/pkg-final/smoke.out:2` literally logs `hiverouter 0.1.0` while the workspace is `1.2.0-dev` — proof that the reported product version can diverge from the manifest version and nothing catches it.
3. **`suspension.rs` carries a self-declared open gap** (`suspension.rs:39`) with no tracking id in `docs/`.
4. **No fuzz/property tests, no `proptest`/`quickcheck` dependency**, no mutation testing, no coverage instrumentation (`cargo-llvm-cov`/`tarpaulin` absent from every script I read).
5. **Flakiness surface:** the WS helpers use fixed 5 s timeouts (`support/mod.rs:322`) and `recv_until` caps at 64 frames (`:330`). On a loaded machine these are the most likely flake points; there is no retry and no `RUST_TEST_THREADS` pinning in `tools/test.ps1` or `run-gates.ps1`.

---

## 3. Unit-test modules — substance and doubles

### 3.1 `hiverouter-provider` (50 unit tests)

- `tool_assembler.rs` 17, `dto.rs` 10, `budget.rs` 9, `accounting.rs` 4, `client.rs` 3, `pricing.rs` 3, `error.rs` 2, `secret.rs` 2 (**VERIFIED** counts).
- `tools/test.ps1:38-40` labels this gate **"provider crate tests (wiremock offline matrix)"** — i.e. HTTP is doubled with `wiremock`, which is a §7-permitted HTTP-client double. **VERIFIED** the label; **UNVERIFIED** that every provider test uses it (I did not audit each of the 50 bodies).
- `secret.rs` (2 tests) is the right place for key handling; combined with `run-gates.ps1` G6 (§5.4) this is the repo's key-leak defence-in-depth.

### 3.2 `hiverouter-core` (177 unit tests)

Concentrated in the state/data layer: `store.rs` 28, `run_state.rs` 20, `tools_bridge.rs` 12, `db.rs` 11, `execution.rs` 10, `recovery.rs` 10 (**VERIFIED**). `static_ui.rs` 2 verifies the embedded `web/dist` asset path; `config.rs` 7 covers resolution precedence (the same function the e2e harness relies on for `HIVEROUTER_ALLOWED_ORIGINS`).

### 3.3 `hiverouter-tools` (119 unit tests)

Security-relevant density is good: `approval.rs` 18, `jail.rs` 14, `cli.rs` 14, `side_effects.rs` 10, `shell.rs` 7, `ledger.rs` 7, `env_safety.rs` 2, `settings.rs` 2 (**VERIFIED**). `jail.rs` 14 + `shell.rs` 7 + `env_safety.rs` 2 is the correct triple for path/shell/env confinement.

### 3.4 `hiverouter-contract` (27 unit tests)

`event.rs` 10, `command.rs` 5, `timestamp.rs` 5, `refusal.rs` 3, `state.rs` 2, `usage.rs` 2 (**VERIFIED**). These are the frozen-vocabulary tests that `tools/qa/verify-harness.mjs` cross-checks against `docs/contracts/UI-COMMANDS-v1.md`.

### 3.5 Unit-layer gaps

1. **No unit test anywhere asserts the CLI's reported version string** — the root cause of the `0.1.0` observation (§2.3 item 2).
2. **`hivemind-cli.mjs` (80 KB) has zero test file in the repo.** The only automated check is `node --check tools/hivemind-cli.mjs` recorded in `docs/evidence/gate-033bcde0-offline.json` (`"cli": {"command": "node --check tools/hivemind-cli.mjs", "exit": 0}`) — a **syntax check, not a behaviour test** (**VERIFIED** from that JSON). This is the single largest untested surface in the codebase.
3. No tests for `tools/qa/*.mjs` except `tests/seal-topology.test.mjs` and `tests/trace-tools.test.mjs` — and those two are never executed by any gate (§5.6).

---

## 4. Web test suite and configs

### 4.1 What is real

- `web/vite.config.ts:33-37` — `include: ['tests/**/*.test.ts']`; all 31 files are `*.test.ts` under `web/tests/`, so **the include glob matches every file** (**VERIFIED**).
- `web/package.json` scripts (**VERIFIED**): `typecheck: "tsc --noEmit"`, `test: "vitest run"`, `build: "tsc --noEmit && vite build"`. Note `build` re-runs typecheck (redundant with G3a but harmless and fail-closed).
- Zero skipped web tests (**VERIFIED**: 0 matches for `it.skip`/`test.skip`/`it.todo`).
- The suite is heavily state-machine oriented: `approval.state` 32, `toolActivity.store` 16, `apiLifecycle` 15, `agentsToolActivityMount` 12, `layout_hive` 12, `toolActivityPanels` 12 (**VERIFIED**). That is the right shape for a WS-driven UI whose truth lives in the store.
- `noHardcoded.test.ts` (5) and `designParity.test.ts` (7) are repo-hygiene/parity guards — real assertions about source text and token parity, not demos.
- `contract.test.ts` (10) + `refusalParity.test.ts` (3) + `state.parity.test.ts` (8) pin the frozen vocabulary on the client side, mirroring the Rust contract tests. This is genuine cross-language contract enforcement.

### 4.2 Gaps

1. **No DOM environment.** `web/vite.config.ts:34` sets `environment: 'node'`; there is no `jsdom`/`happy-dom` in the config and no `@testing-library/react` in `package.json` scripts I read. Consequence: **no React component is ever rendered in a test**. `agentsToolActivityMount.test.ts` (12) and `toolActivityPanels.test.ts` (12) are named like mount tests but cannot mount under `environment: 'node'`; they must be asserting on exported helpers/strings. **RISK: MEDIUM** — the entire render path's only automated proof is Playwright against a built bundle.
2. **No coverage threshold** anywhere in `vite.config.ts`; `vitest run` has no `--coverage`.
3. **No `test:watch`, no CI workflow file** — I found **no** `.github/workflows/**` in the repo (verified by absence in the top-level listing and the absence of `.github`). Every gate is local-only by design (`tools/test.ps1:3-6` says so explicitly: "no Docker, no cloud CI, no live provider").

---

## 5. `tools/qa/**` — the gate machinery

### 5.1 `run-gates.ps1` — what it actually proves

**VERIFIED** structure (`run-gates.ps1:1-37` synopsis; `:178-419` preflight; `:488-711` gates; `:713-753` summary):

| Gate | Command (exact, from `run-gates.ps1`) | Line | Fail-closed? |
|---|---|---|---|
| G0 preflight | git clean/tag/ancestor checks + web↔crate version parity + manifest schema/seal allowlist + toolchain presence | `:178-467` | **Yes** — every problem is a named `E_GATE_*`/`E_SEAL_*` and exits 1 before any gate runs (`:463-467`) |
| G1 docs-truth | `node tools/qa/check-docs-truth.mjs` | `:489-491` | Yes (`Invoke-Gate` throws on non-zero, `:110-131`) |
| G2 cargo | `cargo test --workspace --locked` in `backend/` | `:494-501` | Yes |
| G3a/b/c web | `npm.cmd run typecheck` / `npm.cmd test -- --run` / `npm.cmd run build` | `:504-512` | Yes |
| G4 e2e | `node run-guarded.mjs` in `e2e/` | `:525-527` | **Only if env armed**; otherwise SKIPPED by name (`:515-523`) |
| G5 harness | `node tools/qa/verify-harness.mjs` | `:531-533` | Yes |
| G6 key scan | regex `sk-[A-Za-z0-9]{32,}` over `git ls-files` text files | `:536-571` | **Yes** — a hit **throws** (`:568-570`) |
| G7 manifest/artifacts | sha256+bytes of `target/release/*.exe`, package zip/sidecar/`CHECKSUMS.txt`, zip member hashes, `web/dist` per-file + tree hash | `:574-711` | Yes, strict mode |

**Genuinely strong properties I verified:**
- **Skips can never become PASS.** `Add-E2E-Skip` records status `SKIPPED` (`:128`); `-RequireE2E` converts a skip into a named FAIL (`:133`); `-Release` fails if `skip > 0` (`:718`, `:735-739`); summary prints "skips are never PASS" (`:731`).
- **HEAD is re-read at the end and a change fails the run** (`:714`, `:724`, `:735`). This catches a build mutating the tree.
- **Exit-code discipline:** `0` pass / `1` gate or preflight failure / `2` setup error / `3` pre-release (`:1-37`, `:87-92`, `:741-745`).
- **G6 is a real secret scan with near-miss telemetry** (`:540-541`, `:565-567`) and binary-file skipping by NUL byte sniff (`:552-554`), not just extension.
- **G7 hashes inside the zip**, not only the zip file (`:645-659`) — so a zip that "looks right" cannot hide a swapped member.
- **The script never prints the key** and forces `LIVE_ONE_PROMPT=0` unless `-Live` (`:470-481`).
- **`-DryRun` truly executes nothing** and exits before `:488` (`:432-461`).

### 5.2 Gate-integrity defects (the important part)

**D1 — The current tree cannot pass `run-gates.ps1` in any mode that certifies.** **VERIFIED by command + arithmetic:**
- Strict mode: `git status --porcelain` returns 1 path (`?? docs-analysis-tmp-index.txt`) → `E_GATE_DIRTY_TREE` (`:224-228`) → exit 1.
- `-Rehearsal`: dirty tree is downgraded to a warning (`:226`) and version mismatch to a warning (`:268`), **but G7 is not rehearsal-aware**. `$preReleaseMode = [bool]$script:PreReleaseAccepted` (`:577`) and `$script:PreReleaseAccepted` is only ever set inside the `-Release` branch (`:409`). Therefore in `-Rehearsal`, `$preReleaseMode = $false` and the artifact comparisons at `:609-618` are hard failures. Measured drift:
  - `target/release/hiverouter.exe` = 36,540,416 B, sha `f369d7ac…f871` vs manifest 13,624,320 B, sha `9c3174fb…d7207` → **mismatch**.
  - `target/release/hiverouter-cli.exe` = 7,386,624 B, sha `709da2e9…b339` vs manifest 5,372,416 B, sha `f757f558…14df` → **mismatch**.
  - `web/dist/index.html` sha `f20171f1…9a48` vs manifest `01f31347…e9e8f`; asset names are entirely different (`index-BNz-sfds.js` / `index-Dlq9NyYb.css` vs manifest `index-DRvC4Qb-.js` / `index-D8k8-PfA.css`); `web/dist/figcraft/**` (22 files) is not in the manifest at all.
  ⇒ **`-Rehearsal` fails at G7.**
- `-Release`: `web/package.json` version `1.2.0-dev` contains `-` and differs from manifest `1.1.0`, so `$preReleaseCandidate = $true` (`:213`) → the pre-release branch (`:359-412`) → requires `docs/RELEASE-EVIDENCE.md` + `plans/waves/README.md` to document it, and then **exits 3** (`:741-745`) with "not a release".
**Conclusion: today the only non-red invocation is `-Release` returning exit 3 (PRE-RELEASE). No invocation of `run-gates.ps1` can return exit 0 on this tree.** That is defensible by design (no false green) but it means **"gates pass" is not currently an available claim**, and any finish plan must state that explicitly.

**D2 — `tools/qa/tests/*.test.mjs` are never run.** **VERIFIED:** grep for `node --test` across `tools/qa/*.ps1`, `tools/*.ps1`, `e2e/package.json`, `web/package.json`, `README.md` returns **zero** matches. `tools/qa/tests/seal-topology.test.mjs` ("exactly five node:test cases", its own header) and `tools/qa/tests/trace-tools.test.mjs` ("run with `node --test`", its own header) are the unit tests **for the release-seal predicate and the trace guard** — the two most safety-critical non-Rust scripts — and **the gate that depends on them never executes their tests**. This is the highest-leverage one-line fix in the repo.

**D3 — G4's default is a skip.** With no `E2E_APP_READY`/`E2E_BASE_URL`, G4 is SKIPPED and the run can still print `GATES: PASS (with N named SKIP(s))` (`:747-749`). Only `-RequireE2E` or `-Release` closes that. `-RequireE2E` requires the operator to remember a flag (`:415-419`); it is not the default.

**D4 — the e2e suite is not proven to have executed anything recently.** `e2e/test-results/.last-run.json` says `{"status":"passed","failedTests":[]}` — with **no test count, no timestamp, and no commit** (**VERIFIED** by reading the file). That file cannot distinguish "95 passed" from "0 tests ran and nothing failed". `run-guarded.mjs` mitigates the all-skip case (`run-guarded.mjs:25-35`) and exits 1 when 0 passed / 0 failed / skipped>0 — good — but it does **not** fail when **0 tests are collected at all** (`stats` exists with all zeros; `passed===0 && failed===0 && skipped>0` is false when `skipped===0`). **PROPOSED:** add `if (passed + failed + skipped === 0) exit 1`.

**D5 — `-Live` does not verify the running app is the current build.** `run-gates.ps1:470-477` checks `E2E_APP_READY`, `E2E_BASE_URL`, and key presence — **not** that `E2E_BASE_URL`'s `/health.version` matches `web/package.json`. A paid run can therefore be recorded against a stale binary. **PROPOSED:** assert `health.version` equals `$version` before setting `LIVE_ONE_PROMPT=1`.

**D6 — G6 scans only git-tracked text files.** `:537` uses `git ls-files`. Untracked secrets (e.g. a stray `.env`, `runtime-test/**/server.*.log`) are invisible to the scan. **VERIFIED** by reading the loop; `runtime-test/server*.out.txt` and `tools/qa/provider-test.log` (20,774 B) exist on disk and are outside the scan unless tracked.

**D7 — `dist/HiveMind` portable output escapes every hash check.** G7 verifies `target/release/*` + the sealed 1.1.0 zip; it never inspects `dist/HiveMind/hiverouter.exe` (the artifact `build-desktop.ps1` actually ships) — **VERIFIED** by reading `:602-665` (only `manifest.artifacts`, `manifest.package`, `manifest.web`). So the desktop-installed binary has **no manifest coverage at all**.

### 5.3 `check-docs-truth.mjs` — real doc-vs-product checks

**VERIFIED** rules: version parity `web`↔crates (`:158`), release-facing docs must state the **sealed** version and must not present the unreleased one as released (`:172-176`), a **positive** real-SSE streaming claim (`:179`), `docs/RELEASE-EVIDENCE.md` must carry both versions (`:192-194`), must retain sealed artifact/package hashes (`:206`), must cite sealed tag+source (`:211`), must be labelled `pre-release` (`:215`), and must point at `tools/release-manifest.json` as the hash authority (`:218`). This is a genuine anti-stale-docs gate, not a linter.

### 5.4 `verify-harness.mjs` — vocabulary + matrix integrity

**VERIFIED** checks at `:102-111`: frozen commands ≥10, decision-gated DIFF commands **exactly 3** (`:103`), events ≥12, refusal codes ≥30, matrix parses, and **the 28 guide surfaces must be exactly 28** (`:111`). The strict `===3` and `===28` equality checks are the right kind of brittle: they fail when the frozen contract drifts.

### 5.5 `check-seal-topology.mjs` — the release predicate

**VERIFIED** named refusals: `E_SEAL_ARTIFACT`, `E_SEAL_DETERMINISM`, `E_SEAL_DIFF`, `E_SEAL_GIT`, `E_SEAL_MANIFEST`, `E_SEAL_TAG`, `E_SEAL_UNSEALED` (grep over the file). `run-gates.ps1:330` shells this out with `--json` and promotes any refusal to a preflight problem (`:338-348`). The seal model is source `C` → seal `D`, tag at `D`, `C..D` diff ⊆ fixed allowlist (`run-gates.ps1:256-261` refuses a **widened** allowlist with a dedicated message). That is a sound integrity design.

### 5.6 Other QA scripts

- `ui-e2e.mjs` — drives **installed Edge over CDP** against `target/release/hiverouter.exe` to prove Settings→Provider save round-trips to `provider.json` (`ui-e2e.mjs:1-25`). **Hardcodes `port = 8099` and `cdpPort = 9223`** (`:22-23`) → collides with the desktop launcher's fixed 8099 (§6.4). Not wired into `run-gates.ps1`.
- `apply-live-trace.mjs`, `build-trace-seeds.mjs`, `figcraft-assets.mjs`, `apply_wp01.py`, `check-env.ps1` — trace/asset tooling; `figcraft-assets.mjs` is also exposed as `npm run verify:figcraft` in `e2e/package.json`.
- `tools/qa/provider-test.log` (20,774 B) is a **committed log inside the QA tooling directory**, not in `docs/evidence/`. Untracked-vs-tracked status would decide whether G6 scans it; its presence outside the evidence convention is a hygiene defect. **PROPOSED:** move to `docs/evidence/` or delete.

---

## 6. Build, release, install scripts

### 6.1 `tools/test.ps1` — the local developer gate (76 lines)

**VERIFIED** by reading the whole file: runs `cargo test -p hiverouter-provider|hiverouter-tools|hiverouter-contract|hiverouter-core --locked` (`:38-49`) then `npm run typecheck` + `npm test -- --run` (`:60-62`), prepending cargo + `tools/w64devkit/bin` to `PATH` (`:18-20`), setting `CARGO_PROFILE_DEV_DEBUG=0` / `CARGO_INCREMENTAL=0` (`:21-22`), and exiting on the first failure (`:29-32`).

**Gaps:** no `--workspace` (four crates enumerated by hand → a fifth crate would be silently skipped); **no clippy, no `cargo fmt --check`**, no e2e, no gate harness, no docs-truth, no key scan. `:54` writes "skipping web gates (recorded as not-run, not passed)" and then sets `$global:LASTEXITCODE = 0` — honest wording, but the gate then reports success for a not-run step. **PROPOSED:** make it `-BackendOnly`-equivalent or return a distinct exit code.

### 6.2 `tools/build-release.ps1` (45 KB) — deterministic release builder

**VERIFIED** from the synopsis (`:1-40`): preflight (git clean/HEAD/tag/toolchain/version/drift policy) → web typecheck/tests/build with per-file dist hashes + tree hash → `cargo build --release --locked` → drift policy → stage `README.md`/`RUNBOOK.md`/`RELEASE.md` + both binaries + `CHECKSUMS.txt` → deterministic zip + `.sha256` sidecar → extract-and-verify every hash → write `tools/release-manifest.json` → absolute-path scan. Flags include `-DryRun`, `-AllowDirty`, `-AllowTagAncestor`, `-SkipWebTests`, `-SourceOnly`, `-RecordSeal -SealCommit`, `-KeepVerifyDir` (`:22-34`). Exit codes `0/2/3/4/5/6/7` are named (`:36`); it states "No tag is created, moved, or deleted … No key is read. No absolute paths are emitted."

**Strengths (VERIFIED):** extract-and-verify after zipping defeats "zip looks fine" drift; `-SkipWebTests` exists but the gate harness never passes it (`run-gates.ps1:508` always runs web tests) → **no weakening in the gated path**. `-RecordSeal` re-verifies the allowlisted `C..D` diff and recorded hashes and writes only manifest fields, no rebuild (`:22-26`) — correct: you cannot "reseal" by rebuilding.
**Gaps:** `-AllowDirty` and `-SkipWebTests` are footguns if used outside the gate harness (nothing prevents `-AllowDirty -SkipWebTests` in one command; the manifest records `source_dirty` (`release-manifest.json:12`) so it is at least auditable). The 45 KB script has **no unit tests** — only the pure seal predicate it feeds is tested (`tools/qa/tests/seal-topology.test.mjs`), and per D2 those tests never run in the gate.

### 6.3 `tools/build-desktop.ps1` (101 lines) — the portable Windows app

**VERIFIED** by reading the whole file: `[1/5]` `npm.cmd run build` (`:30`), `[2/5]` `cargo build --release --locked` with cargo + w64devkit on `PATH` (`:39-40`), `[3/5]` compiles the WinForms launcher with the **in-box** `csc.exe` from `%WINDIR%\Microsoft.NET\Framework64\v4.0.30319` (`:25`, `:47-54`), `[4/5]` copies launcher + `target\release\hiverouter.exe` into `dist\HiveMind` and writes `README.txt` (`:56-88`), `[5/5]` `Compress-Archive` → `dist\HiveMind.zip` (`:90-98`).

**Defects found (all VERIFIED against measured state):**
- **No build identity is written.** Nothing stamps the commit, version, or hash into `dist\HiveMind\`. Compare `build-release.ps1`, which writes a full manifest. So `dist/HiveMind/hiverouter.exe` (36,540,416 B, sha `f369d7ac…f871`, mtime 2026-11-10 18:10) is **unidentifiable** except by mtime.
- **`-Configuration` is decorative for the copy.** `:59` reads `target\$Configuration\hiverouter.exe` but `[2/5]` always runs `cargo build --release` (`:40`). `-Configuration Debug` therefore copies a path that the build never produced (or a stale one). **PROPOSED:** build `--profile $Configuration` or drop the parameter.
- **`README.txt` claims a free port that the launcher does not implement** — `:69` "starts hiverouter.exe … on a free loopback port" vs launcher `PreferredPort = 8099` with no port search (§6.4). Documented behaviour and actual behaviour diverge.
- **No post-assembly smoke.** `:100` prints a directory listing and exits 0. It never launches `dist\HiveMind\HiveMind.exe`, never checks `/health`, never verifies the server exe hash. **This is the central install-integrity gap of the lane.**
- **`Compress-Archive` is not reproducible** (no fixed timestamps/ordering) — `dist/HiveMind.zip` (14,416,165 B) cannot be byte-stable across runs, unlike the deterministic zip in `build-release.ps1`. Only matters if this zip is ever treated as a release artifact; today it is not in the manifest (§5.2 D7).

### 6.4 `tools/desktop-launcher/**` — the installed entry point

**VERIFIED** by reading all 321 lines of `HiveMindLauncher.cs`:

| Aspect | Finding | Evidence |
|---|---|---|
| Server discovery | `hiverouter.exe` **beside the launcher** first, then `HIVEMIND_ROOT`, then `DefaultRoot`, then `target\release` | `:254-273` |
| **Hardcoded developer path** | `DefaultRoot = @"C:\Users\Riley\Desktop\hiverouter-v1"` | `:17` |
| **Fixed port, no search** | `PreferredPort = 8099`; if busy-but-not-healthy → error dialog; if busy-and-healthy → reuse | `:16`, `:44-55` |
| Port conflict UX | refuses with "Port 8099 is in use by another application" | `:46-55` |
| Health gate | `WaitForHealth(port, 60000)` polling `/health` every 300 ms | `:72`, `:307-319` |
| Key handling | DPAPI `CurrentUser` unprotect of `%LOCALAPPDATA%\HiveMind\provider.key` with entropy `HiveMind.ProviderKey.v1`, injected into `start.EnvironmentVariables[keyEnv]` only | `:20`, `:155-168`, `:215-235` |
| Key env selection | `tokenlabs` → `HIVEROUTER_API_KEY`, else `DEEPSEEK_API_KEY`, mirroring `start-hivemind.ps1:59` | `:160-162` |
| Key redaction | no key printed anywhere; parse failures `catch` silently and fall back to env | `:230-234` |
| Token | 32-hex GUID launch token persisted to `launch.token`, reused when ≥16 chars | `:132-153` |
| **Stdout/stderr redirect without draining** | `RedirectStandardOutput = true`, `RedirectStandardError = true`, and **no `BeginOutputReadLine`/`ReadToEndAsync` anywhere** | `:66-67` |
| Build identity check | **none** — the launcher never validates the server version/hash | whole file |
| Workspace root | `HIVEMIND_WORKSPACE_ROOT` → `DefaultRoot` → cwd | `:237-252` |
| Window | installed Edge, else Chrome, `--app=` + `--user-data-dir` | `:18-19`, `:89-98` |
| Server lifetime | killed on window exit only if the window lived ≥3 s (avoids killing the server when Edge hands off to an existing window) | `:101-115` |

**Defects:**
- **HIGH — pipe-buffer deadlock risk.** `:66-67` redirect both streams and nothing reads them. A child that writes more than the pipe buffer (~4 KiB) blocks forever; `WaitForHealth` then fails and the user sees "did not become ready within 60 seconds" while the real cause is an unread pipe. Whether this bites depends on server verbosity; `runtime-test/pkg-final/smoke.out` shows the server logs at least 2 lines per boot plus `INFO` per request, so a chatty session is plausible. **Fix location:** `HiveMindLauncher.cs:66-67` — remove the redirects, or attach `OutputDataReceived`/`ErrorDataReceived` handlers that append to a bounded ring buffer.
- **MEDIUM — no install identity.** Nothing compares the launched server's `/health.version` (the endpoint returns `version`; `e2e/helpers/live-server.ts:57` models it) with an expected version. An operator can be running the 2026-11-06 `tools/bin` binary (sha `184d51a7…`) while believing they run today's build. **Fix location:** `HiveMindLauncher.cs:72` (after `WaitForHealth`, verify `version` against an embedded constant, and refuse/warn on mismatch).
- **MEDIUM — dev path in shipped source.** `:17` embeds `C:\Users\Riley\Desktop\hiverouter-v1`. Harmless on this machine, wrong for any other install and a privacy/hygiene leak in a public artifact. **Fix location:** `:17` — drop `DefaultRoot` and rely on `HIVEMIND_ROOT` + cwd.
- **LOW — no single-instance lock.** Two launches racing before the port is open could both try to spawn. The port check at `:56` narrows but does not close the window.
- **LOW — `ResolveServer` prefers `besideLauncher`,** which is correct for portability but means the *older* `dist/HiveMind` copy silently wins over a freshly built `target/release` one.

`tools/desktop-launcher/README.md:30` documents copying the launcher to `C:\Users\Riley\Desktop\HiveMind.exe` — another personal path, and a **second** install location outside `dist/`. Neither location is hash-recorded anywhere.

### 6.5 Provider-key handling — `set-provider-key.ps1` / `set-provider-profile.ps1`

**VERIFIED** `set-provider-key.ps1` (50 lines): hidden `Read-Host -AsSecureString` (`:20`), reject empty/whitespace (`:25-27`), DPAPI `Protect` with entropy `HiveMind.ProviderKey.v1` and `DataProtectionScope::CurrentUser` (`:28-34`), write `%LOCALAPPDATA%\HiveMind\provider.key` (`:35-38`), then `ZeroFreeBSTR` + `Array::Clear` in `finally` (`:40-49`). It prints only the destination path (`:39`). **This is correct secret handling.** No key material is written to the repo, a command line, or a log.

**Gaps:** (a) **no key-shape validation** — any non-empty string is accepted, so a paste error becomes a runtime `E_KEY_MISSING`/401 rather than an immediate refusal; a cheap `^sk-[A-Za-z0-9]{16,}$` guard before `Protect` would fail fast without printing the value. (b) `Read-Host -AsSecureString` **requires an interactive host**; in a non-interactive/CI session it cannot run, so there is no unattended install path. (c) No rotation/backup story beyond "run it again" (`:14-15`).

**VERIFIED** `set-provider-profile.ps1` (65 lines): `ValidateSet('deepseek','tokenlab','custom')` (`:21`), requires `-BaseUrl`/`-WireModel` for tokenlab/custom (`:39-44`, `:48-54`), `api_type='tokenlabs'` for both non-DeepSeek variants (`:45`, `:54`), writes **non-secret** `provider.json` (`:58-62`), and states explicitly that this file contains no secret (`:64`). Consistent with the launcher's `ResolveApiType` (`:170-213`) which handles `"api_type"` and falls back to a `"tokenlab"` substring probe.

**Cross-script consistency (VERIFIED):** `start-hivemind.ps1:59` and `HiveMindLauncher.cs:160-162` agree on `tokenlabs → HIVEROUTER_API_KEY`. `start-hivemind.ps1:52-54` additionally maps a `name` matching `^tokenlab` to `tokenlabs`, which the C# launcher does via a bare substring test (`:202-205`) — equivalent in effect, different in mechanism.

### 6.6 `tools/start-hivemind.ps1` (113 lines)

**VERIFIED:** requires `target\release\hiverouter.exe` (`:12-14`), reuses a healthy server after a 2 s `/health` probe and opens the browser with the stored launch token (`:22-33`), otherwise creates `%LOCALAPPDATA%\HiveMind`, generates/persists a launch token (`:35-39`), exports `HIVEROUTER_LAUNCH_TOKEN` (`:40`), resolves `api_type` from env → `provider.json` (`:42-58`), DPAPI-unprotects `provider.key` into the child env with a warning-only fallback (`:63-81`), starts the exe with explicit `--bind/--data-dir/--workspace-root` (`:84-86`), polls `/health` for 20 s and **stops the child if it never becomes healthy** (`:88-108`), then prints pid/health/data/stop and opens the UI (`:110-113`).

**Defects:** (a) `-DataDir` defaults to `%LOCALAPPDATA%\HiveMind` (`:5`) — the same directory the DPAPI key lives in, so server data, logs, and the key file share one folder with no ACL boundary; (b) the 20 s boot budget is shorter than the launcher's 60 s (`HiveMindLauncher.cs:72`) — two different definitions of "booted"; (c) `:102 if ($null -eq $health)` relies on `$health` being undefined rather than initialised; correct today, but it would break under `Set-StrictMode`, which is not set anywhere in `tools/*.ps1`.

---

## 7. E2E harness

### 7.1 `e2e/playwright.config.ts`

**VERIFIED:** `testDir './specs'`, `timeout 30_000`, `expect.timeout 5_000`, `fullyParallel: false`, **`workers: 1`** (comment: "one worker keeps WS/event ordering deterministic for trace evidence"), `retries: 0`, `forbidOnly: true`, three reporters (`list`, `html` → `playwright-report`, `json` → `test-results/results.json` for the skip guard), `headless: true`, `trace: 'retain-on-failure'`, `screenshot: 'only-on-failure'`, `video: 'off'`, and **two projects using installed channels** `chrome` / `msedge` with an explicit "no browser download" policy (`:1-14`, `:43-48`). `BASE_URL` defaults to `http://127.0.0.1:8080` (`:17`) but the documented target is `8799` (`e2e/README.md:34`, `e2e/playwright.config.ts:11`), so a forgotten `E2E_BASE_URL` silently targets the wrong port.

**Port/lease risk:** **LOW** for the spawned-server specs (`live-server.ts:166-187` reserves an ephemeral port and `:383` retries 3×), **MEDIUM** for the external-app specs (they inherit whatever `E2E_BASE_URL` says) and **HIGH** for the repo as a whole because three tools hardcode `8099`/`8799` (§6.4, §5.6). Note the TOCTOU in `reserveLoopbackPort`: it closes the probe socket before the child binds (`:178-185`), so another process can steal the port; the 3-attempt loop (`:383`) is the mitigation.

**Flakiness risk:** `workers: 1` and `retries: 0` mean **any** timing flake becomes a hard red with no retry. `timeout: 30_000` against 1.7-minute recorded paid runs (`docs/evidence/live-1.1-paid.log` shows individual tests at 3.1–6.9 s, 5 passed in 1.7 m) leaves little headroom for a 5-test paid lane on a loaded host.

### 7.2 `e2e/run-guarded.mjs` and the suite-truth guard

**VERIFIED:** wraps the local `@playwright/test/cli.js` (`:9`), `spawnSync` with `stdio: 'inherit'` and `PLAYWRIGHT_JSON_OUTPUT_NAME` (`:10-13`), then reads `test-results/results.json` and **exits 1 when `expected===0 && unexpected+flaky===0 && skipped>0`** unless `E2E_ALLOW_ALL_SKIPS=1` (`:25-35`). `GATE-EXCLUSIONS.md:32` documents that this was verified both ways ("all app-bound tests skip -> exit 1 … full run (passes present -> exit 0)").

**Gaps:** (a) `E2E_ALLOW_ALL_SKIPS=1` is an unguarded escape hatch — nothing requires the "written reason" the message asks for; (b) **zero collected tests is not caught** (all three counters 0 → the `else` path cannot fire, `process.exit(exit)` with Playwright's 0/inconclusive); (c) the JSON report path is hardcoded relative to cwd, so running from the repo root instead of `e2e/` silently finds nothing and degrades to the raw exit code.

### 7.3 `e2e/helpers/live-server.ts` — the real-server harness (607 lines)

**VERIFIED and genuinely strong:**
- Spawns the **real binary** with `--bind 127.0.0.1:<ephemeral> --data-dir <tmp> --workspace-root <tmp>` (`:385-393`).
- Binary discovery `E2E_SERVER_BIN` → `target/debug` → `target/release` (`:24-28`), with a variant label (`:32-35`, `:99-106`) surfaced in every failure (`:209-212`). **Note: `tools/bin/hiverouter.exe` — the path `e2e/README.md:34` documents as the app target — is NOT in `BINARY_CANDIDATES`.** Docs and code disagree.
- Health gate validates **`schema === 'hiverouter.v1'`** and rejects `status=failing` (`:239-247`), then `/api/session` shape `{token,schema,last_seq,ws}` with schema equality (`:277-288`), then **proves the embedded UI is served** by asserting `<div id="root">` in `/` (`:306-311`). That last check is exactly the "installed build actually serves its UI" assertion the desktop path lacks.
- **Key redaction is enforced before logging**: `redactSecrets` strips the env key and `/sk-[A-Za-z0-9_-]{8,}/g` (`:84-93`), and all child output flows through it (`:195-207`), capped at 200 lines (`:201`).
- **Teardown is guaranteed**: `killAndWait` → SIGKILL escalation → temp-dir removal with 3 retries (`:315-331`, `:430-447`).
- **Paid gating is two-axis and fail-closed**: `paidGate('prompt')` needs `LIVE_ONE_PROMPT=1` + key + binary; `paidGate('tool-turn')` needs the **independent** `LIVE_TOOL_TURN=1` (`:132-158`). A missing flag/key is a **named skip**, never a fake pass (`:118-155`).
- Child env hygiene: `HIVEROUTER_BIND/DATA_DIR/WORKSPACE_ROOT/ALLOWED_ORIGINS` are **deleted** so the harness owns them (`:468-471`); `DEEPSEEK_API_KEY` is deleted when `withProviderKey === false` (`:464-466`).
- `allowedOrigins` are validated against an exact loopback regex (`:347-362`) — no wildcards can be injected into the child.

**Gaps:** `buildChildEnv` copies the **entire** parent `process.env` (`:463`), so an operator shell with a key present will hand it to every spawned server even for "no-key" specs unless each spec passes `withProviderKey: false`. The fail-closed property then depends on spec discipline, not on the harness.

### 7.4 Spec inventory truth

- **VERIFIED** `e2e/README.md:3-8` claims "FINAL CHROME RUN 79 PASSED / 0 FAILED / 14 SKIPPED", "93 tests", Playwright **1.65.0**, app target `tools/bin/hiverouter.exe` (sha prefix `184D51A7…C9A9`), serving `index-Cbfm1IuM.js` at `:8799`.
- **VERIFIED** `e2e/package.json` pins `"@playwright/test": "1.65.0"` and the installed `e2e/node_modules/@playwright/test` reports **1.65.0** → pin matches install. Good.
- **VERIFIED** `tools/bin/hiverouter.exe` sha256 = `184d51a7c3cca7b9d50954b677c5ad8a14472700a651fdf0d270441be3eac9a9`, 9,329,152 B, mtime **2026-11-06 09:10** — i.e. the README's "shipped binary" is a **four-day-old 1.x-era build**, not the current `1.2.0-dev` HEAD build. The e2e "79 passed" record therefore describes a *different* binary than the one under analysis.
- **VERIFIED** `web/dist/assets/` currently contains `index-BNz-sfds.js` / `index-Dlq9NyYb.css`, **not** the README's `index-Cbfm1IuM.js` → the bundle under test has changed since that run.
- **VERIFIED** static test-site count 95 (§1.4) vs the README's "93 tests" — a documentation drift, and the README's own table totals (15+28+29+2+1 rows) do not reconcile to 93 either. **Marked UNVERIFIED as a live count**: only a real run can settle it.
- **VERIFIED** `e2e/test-results/.last-run.json` = `{"status":"passed","failedTests":[]}` — no count, no commit, no timestamp.
- **VERIFIED** `e2e/GATE-EXCLUSIONS.md` is an honest exclusion registry: G-UI-3/W1, G-UI-10/W2, G-UI-8 axe subcheck, W1/W2 are **explicitly not passes** (`:12-16`), and `:3` says the registry test in `specs/gui-gates.spec.ts` asserts every G-UI-1..12 id is implemented or listed. It also states the axe dependency is absent with `MODULE_NOT_FOUND` evidence (`:14`) — a truthful non-claim.

### 7.5 The untracked-spec problem

`GATE-EXCLUSIONS.md:24-27` and `:12-13` repeatedly say `live-security.spec.ts`, `live-resync.spec.ts`, `live-tool-turn.spec.ts`, `live-trace-capture.spec.ts` were **"untracked at the 2026-11-06 audit"** with no recorded run. Those files **do exist now** (§1.4) and the tree is otherwise clean, so they appear tracked at `HEAD`. **The documentation is stale on the exact point that decides whether those gates are closed.** Until a run log + `docs/trace/live/live-security-*.json` exists, G-UI-3/W1 and G-UI-10/W2 remain open regardless of file presence — that part of the exclusion text is still correct.

---

## 8. `runtime-test/**`, `ledgers/**`, dist manifests

### 8.1 `runtime-test/` — hand-run package smokes (evidence, not a test suite)

**VERIFIED** by enumeration: 4 package dirs (`pkg-final`, `pkg-cli-final`, `pkg-smoke`), 9 data dirs (`data`..`data9`), 4 pairs of `server*.out.txt`/`server*.err.txt`, `s5`–`s9` out/err pairs, `root.html`, and `workspace/{note.txt,README.md}`.

- `pkg-final/smoke.out` shows a real boot: `environment check passed check="rustc" … check="node" …`, then `hiverouter 0.1.0 listening on http://127.0.0.1:8802 (db …\smoke-data\hiverouter.db, schema v3, contract hiverouter.v1)` and `server booted address=127.0.0.1:8802 … schema_version=3`.
  - **Two defects proven by this log:** (i) the product reports **`0.1.0`** while the workspace is `1.2.0-dev` (§2.3); (ii) the smoke binds **fixed port 8802** and writes a **DB into the package folder** (`smoke-data/`), i.e. it is not hermetic.
- `pkg-final/CHECKSUMS.txt` records `hiverouter.exe (sha256 184D51A7…)` — **exactly** the `tools/bin` binary hash. So `runtime-test/pkg-final` and `tools/bin` and the e2e README all describe the same **2026-11-06** build.
- `runtime-test/**/data*/hiverouter.db` (+ `-shm`, `-wal`) are **committed binary databases**, including `data4`/`data6`/`data9` snapshot JSON. They are inert evidence, but they are stale SQLite files inside the repo.
- **VERDICT:** `runtime-test/` is **UNVERIFIED as current-build evidence** and **VERIFIED as stale**: every hash in it points at the 2026-11-06 build, and nothing regenerates it. There is **no script** anywhere that produces these directories (grep found no writer for `runtime-test/` in `tools/*.ps1` or `tools/qa/*`), so they cannot be reproduced by command — which violates "evidence must be a command + output" (`AGENTS.md` §22).
- **PROPOSED:** replace with a scripted, hermetic install smoke (§10.2) writing to `docs/evidence/install-smoke-<commit>.json`.

### 8.2 `ledgers/`

**VERIFIED** `ledgers/ui-watchdog.md` (6,315 B) — a read-only UI asset audit dated 2026-11-06T05:23:01, `Overall: PASS`, `ALLOW-FRONTEND: true`, `FAIL count: 0`, with a 20-row table giving per-file SHA-256 and verdicts (e.g. `docs/ui/figcraft/A/A-ASSETS.json` `88832272…77F1`). `ledgers/ui-watchdog.json` (8,655 B) is the machine form. Method is stated (file existence, byte size, SHA-256, PNG IHDR/SVG viewBox, `figcraft-cli info --json`). **This is a real ledger with real hashes.** Gap: it covers **design assets only** — there is no ledger for test results, releases, or installs. **PROPOSED:** generalize to a `ledgers/release.json` and `ledgers/install.json` with the same hash discipline.

### 8.3 `tools/release-manifest.json` — the release registry

**VERIFIED** contents: `schema hivemind.release-manifest.v1`, `version 1.1.0`, `status SEALED_1.1.0`, `tag hivemind-1.1.0`, `source_commit 87079cec…c7fe`, `generated_at 2026-11-06T16:42:45-06:00`, toolchain (`rustc 1.99.1`, `cargo 1.99.1`, `node v26.7.0`, `npm 11.19.0`, `gcc 16.2.0`), provider block (`endpoint https://api.deepseek.com/chat/completions`, models `deepseek-flash`/`deepseek-v4-pro`, `max_output_tokens 393216`, `key_env DEEPSEEK_API_KEY`), `web.tree_sha256 7911b91c…3f10`, 3 web files, `artifacts` = 2 binaries with sha+bytes, and a `package` block with zip/sidecar/`CHECKSUMS.txt` hashes and a 6-member `contents` list.

- `status_note` is a **model of honest release hygiene**: "Authoritative sealed 1.1.0 record only. The working tree is an unreleased 1.2.0-dev pre-release line; no 1.2 manifest, artifact, tag, live proof, or release-gate PASS exists. Do not overwrite this registry with a 1.2 build until the seal topology and release gates pass."
- `sealed` is **absent** (`$m.sealed` is null) → `$sealedManifest = $false` in `run-gates.ps1:241` → the v2 sealed path (allowlist/tag-at-D) is **not** active for this manifest. So despite the `SEALED_1.1.0` status string, the gate treats it as a **v1 record**, and only the pre-release branch can accept it.
- **VERIFIED consistency check:** `dist/HiveMind-1.1.0-win64/hiverouter.exe` = 13,624,320 B, sha `9c3174fb…d7207` = **exactly** the manifest artifact → the **1.1.0 package is manifest-consistent**.
- **VERIFIED drift:** `dist/HiveMind-1.1.0-win64.zip` = 8,613,122 B = manifest `zip_bytes` ✓, but I did **not** recompute its sha256 → **UNVERIFIED** (G7 would).
- **VERIFIED** three stale backups `dist/.release-manifest-2026110616{3430,3848,4240}.json` (2,159 B each) sit in `dist/` outside the manifest's own hash tree.
- `max_output_tokens: 393216` in the manifest is an **unverified hard number** per `AGENTS.md` §13/§21 ("never adopt a hard number"). **PROPOSED:** mark it `UNVERIFIED` in the manifest or cite the official changelog line.

### 8.4 `docs/evidence/` gate convention

**VERIFIED** by enumeration (~100 files, no README):
- Gate records: `gate-<8hex>-offline.json` ×49, `scale-s0-<8hex>-offline.json`, `scale-s1-<8hex>-offline.json` (102 evidence files total).
- JSON shape (from `gate-033bcde0-offline.json`): `{commit, captured_at, cargo:{passed,failed,ignored,command,suites,exit}, web_typecheck:{exit,command}, web_tests:{passed,command,failed,exit}, web_build:{exit,command}, cli:{exit,command}}`. **The commit hash and timestamp are the convention's strength** — every gate record is pinned to a commit.
- Logs: `1.1-backend-tests.log`, `1.2-backend-tests{,-2..-14}.log`, `final{,2,3,4}-backend-tests.log`, `round{2,3,4,6,7}-backend-tests.log`, `wave{2,3,4}-backend-tests.log`, `1.2-web-tests{,-2,-3}.log`, `1.2-web-typecheck.log`, `final-gate-tests.log`, `1.2-compile-gate{,-2}.log`, `1.2-release-build*.log`, `final-web-{build,tests}.log`.
- Paid/live records: `live-1.1-paid.log` (5 passed, exit 0, `1.7m`), `live-provider-smoke.md`, `live-trace-1.1.md`, `key-validation.md` (HTTP 200, `HIVE-KEY-OK`, response id, usage 14/6/20, **explicitly "No key substring appears in this evidence file"** — correct).
- **The convention is real and disciplined.** Its gap is **staleness, not shape**: the newest `gate-*.json` is `gate-7969f5b9-offline.json` (2026-11-09T00:19, cargo **571 passed**/2 ignored, web **191 passed**) while `HEAD` is `6a41250` (2026-11-10) with **559 declared Rust test fns / 236 web tests**. So (a) no gate JSON exists for `HEAD`; (b) the counts moved in both directions between that record and today (571→559 Rust, 191→236 web), which is *expected* if tests were reorganized — but it means **no committed record attests to the current suite's composition**. Note 571 − 559 = 12, consistent with doc-tests contributing to `cargo test --workspace`; I did **not** verify the doc-test count (**UNVERIFIED**).

---

## 9. Gate integrity — a consolidated assessment

### 9.1 What the gate stack genuinely proves (VERIFIED)

1. **Fail-closed preflight** with named `E_GATE_*`/`E_SEAL_*` codes and no gate claimed when the preflight refuses (`run-gates.ps1:87-92`, `:463-467`).
2. **Version parity** web↔every crate (`:229-233`) — catches a partial version bump.
3. **Release-registry consistency**: tag ↔ `source_commit` ↔ ancestor-of-HEAD ↔ manifest version, plus a **fixed** seal allowlist that refuses widening (`:246-261`).
4. **Artifact and web-dist hash equality** including **inside the zip** (`:602-704`).
5. **Secret scan** with near-miss warnings over tracked text (`:536-571`).
6. **Docs-vs-product truth** including "must not present unreleased X as released" (`check-docs-truth.mjs:172-176`).
7. **No false green from skips** (`:718`, `:731`, `:735-739`, `:747-749`).
8. **HEAD stability checked after the run** (`:714`, `:724`).
9. **Paid calls are opt-in twice** (`:470-481` + `live-server.ts:132-158`).

### 9.2 What it does not prove (the honest gap list)

| # | Not proven | Why | Severity |
|---|---|---|---|
| 1 | That the current `HEAD` passes anything | No `gate-*.json` at `6a41250`; newest is `7969f5b9` (day-old) | **HIGH** |
| 2 | That `run-gates.ps1` can reach exit 0 today | D1 — dirty tree + G7 drift | **HIGH** |
| 3 | That the seal predicate's own tests pass | D2 — `node --test` never invoked | **HIGH** |
| 4 | That the e2e suite actually ran N tests | D4 + `.last-run.json` has no counts | **HIGH** |
| 5 | That a paid live run used the current build | D5 | MEDIUM |
| 6 | That the **installed Desktop** binary is the current build | No identity stamp (§6.3), no check (§6.4), no manifest coverage (D7) | **CRITICAL** |
| 7 | That untracked files hold no secrets | D6 | MEDIUM |
| 8 | That components render correctly | No DOM test env (§4.2) | MEDIUM |
| 9 | That `hivemind-cli.mjs` behaves | Syntax check only (§3.5) | MEDIUM |
| 10 | That clippy/fmt are clean | Never run by any script | MEDIUM |

---

## 10. The minimum battery to call the finish plan complete

Two tiers. **Tier A is the floor**: without all of it, no completion claim is honest. **Tier B** is what the gate harness would need to be able to say "green".

### 10.1 Tier A — mandatory, in order (PROPOSED, derived from what exists)

```powershell
# 0. Clean tree (G0 refuses otherwise). Commit or remove untracked files first.
cd C:\Users\Riley\Desktop\hiverouter-v1
git status --porcelain          # EXPECT: empty

# 1. Rust: unit + integration + doctests, whole workspace
cd backend
cargo test --workspace --locked
# EXPECT: 559 test fns + doctests, 0 failed, 2 ignored (kill_restore helper, provider_live)

# 2. Web: typecheck + 236 unit tests + production build
cd ..\web
npm.cmd run typecheck
npm.cmd test -- --run
npm.cmd run build

# 3. QA guard suites — NOT wired into run-gates today (defect D2); run explicitly
cd ..
node --test tools/qa/tests/
# EXPECT: seal-topology 5 cases + trace-tools cases, 0 failed

# 4. Static + docs gates
node tools/qa/verify-harness.mjs
node tools/qa/check-docs-truth.mjs

# 5. E2E (app must already be running; never start it from Playwright)
$env:E2E_BASE_URL='http://127.0.0.1:8799'; $env:E2E_APP_READY='1'
cd e2e
$env:PLAYWRIGHT_SKIP_BROWSER_DOWNLOAD='1'
node run-guarded.mjs --project=chrome
# EXPECT: 79+ passed, 0 failed, named skips only; exit 0
# Then verify the count, not just the exit code:
node -e "const r=require('./test-results/results.json');console.log(r.stats)"

# 6. Desktop install smoke (proves the installed build == current build)
#    See 10.2 — this is the only step that closes gap #6.

# 7. Release-integrity dry run (must NOT be red for a *release* claim)
cd ..
powershell -NoProfile -ExecutionPolicy Bypass -File tools\qa\run-gates.ps1 -DryRun
```

### 10.2 The exact install smoke that proves a Desktop-installed **current** build

The repo has no such smoke today (§8.1). **PROPOSED** — every step is a real command with a checkable output; it is deliberately byte-level so it cannot pass on a stale binary:

```powershell
# ---- 1. Build fresh, recording what we built ---------------------------------
cd C:\Users\Riley\Desktop\hiverouter-v1
git rev-parse HEAD                                   # record COMMIT
powershell -NoProfile -ExecutionPolicy Bypass -File tools\build-desktop.ps1
$server = "dist\HiveMind\hiverouter.exe"
$built  = (Get-FileHash -Algorithm SHA256 $server).Hash.ToLower()
$version = (Get-Content web\package.json -Raw | ConvertFrom-Json).version
"BUILT   commit=$COMMIT version=$version sha256=$built bytes=$((Get-Item $server).Length)"

# ---- 2. Prove the INSTALLED launcher resolves to that exact binary -----------
# Launcher prefers the exe beside itself, so this asserts the shipped pairing:
$a = (Get-FileHash -Algorithm SHA256 "dist\HiveMind\HiveMind.exe").Hash.ToLower()
$b = (Get-FileHash -Algorithm SHA256 "tools\desktop-launcher\HiveMind.exe").Hash.ToLower()
if ($a -ne $b) { throw "launcher drift: dist launcher $a != built launcher $b" }

# ---- 3. Prove the INSTALLED binary is the CURRENT build ----------------------
# No launcher help needed: run the shipped exe directly and read /health.version.
$data = Join-Path $env:TEMP ("hm-smoke-" + [guid]::NewGuid().ToString('N'))
$port = 8811
$p = Start-Process -FilePath (Resolve-Path $server) -PassThru -WindowStyle Hidden `
     -ArgumentList '--bind',"127.0.0.1:$port",'--data-dir',$data,'--workspace-root',$data
try {
  $h = $null
  $deadline = (Get-Date).AddSeconds(30)
  do { Start-Sleep -Milliseconds 250
       try { $h = Invoke-RestMethod "http://127.0.0.1:$port/health" -TimeoutSec 2 } catch {}
  } while (-not $h -and (Get-Date) -lt $deadline)
  if (-not $h) { throw "install smoke: /health never answered" }
  "HEALTH  status=$($h.status) schema=$($h.schema) version=$($h.version)"
  # (a) identity: the server must report the tree's version
  if ($h.version -ne $version) { throw "STALE INSTALL: /health.version=$($h.version) != tree $version" }
  # (b) contract identity
  if ($h.schema -ne 'hiverouter.v1') { throw "contract drift: $($h.schema)" }
  # (c) the embedded UI is really in this binary
  $ui = Invoke-WebRequest "http://127.0.0.1:$port/" -TimeoutSec 5
  if ($ui.Content -notmatch '<div id="root">') { throw "embedded UI missing" }
  # (d) session endpoint works (proves the WS path is live, not just HTTP)
  $s = Invoke-RestMethod "http://127.0.0.1:$port/api/session" -TimeoutSec 5
  if (-not $s.token -or $s.schema -ne 'hiverouter.v1') { throw "session shape wrong" }
  "OK      installed build $built == current $version (commit $COMMIT)"
} finally {
  if (-not $p.HasExited) { Stop-Process -Id $p.Id -Force }
  Remove-Item $data -Recurse -Force -ErrorAction SilentlyContinue
}
```

**Why this is the exact smoke that closes the gap:** steps 3(a)–(c) are precisely what `e2e/helpers/live-server.ts:239-311` already does for the *test-harness-spawned* binary, applied to the **shipped** artifact instead. Step 3(a) is the assertion that fails on today's `tools/bin/hiverouter.exe` (which would report `0.1.0`, per `runtime-test/pkg-final/smoke.out:2`) and on any stale `dist/HiveMind` copy. Step 2 catches a launcher/binary pairing drift between `tools/desktop-launcher/` and `dist/HiveMind/`.

**Note (VERIFIED, and it must be fixed first):** this smoke will fail today unless `/health` actually carries `version`. `e2e/helpers/live-server.ts:57` declares `version: string` on `LiveHealth` and `:234` reads `body.version`, so the field is expected to exist — but I did not observe a running server, so **whether `/health.version` equals the crate version is UNVERIFIED**. That single field is the cheapest hardening available: it turns "installed build identity" from a hash comparison into a self-report the gate can check.

### 10.3 What must be fixed before any "gates pass" claim (ordered, minimal)

1. `tools/qa/run-gates.ps1:531` — add a `G5b` step: `node --test tools/qa/tests/` (closes D2). 1 line + 3.
2. `e2e/run-guarded.mjs:29` — also fail when `passed+failed+skipped === 0` (closes D4 partially).
3. `run-gates.ps1:470-477` — before `-Live`, assert `health.version == $version` (closes D5).
4. `tools/build-desktop.ps1` — write `dist/HiveMind/BUILD.json` `{commit, version, sha256, built_at}` and verify it in `run-gates.ps1` G7 as a third artifact class (closes D7).
5. `HiveMindLauncher.cs:66-67` — drain or drop the stdout/stderr redirects (closes the deadlock).
6. `run-gates.ps1` — make `$preReleaseMode` (line `:577`) also true under `-Rehearsal`, or add a named `E_GATE_REHEARSAL_MANIFEST_DRIFT` refusal, so `-Rehearsal` has a defined outcome instead of an accidental G7 red (closes D1's ambiguity).
7. Delete or relocate `docs-analysis-tmp-index.txt` (the sole reason the tree is dirty today).

---

## 11. Top-10 quality / release risks

| # | Risk | Severity | Evidence | Fix location |
|---|---|---|---|---|
| 1 | **Installed Desktop build is unidentifiable.** Four `hiverouter.exe` builds coexist with four hashes: `dist/HiveMind` + `target/release` `f369d7ac…` (36,540,416 B, 2026-11-10), `dist/HiveMind-1.1.0-win64` `9c3174fb…` (13,624,320 B), `tools/bin` `184d51a7…` (9,329,152 B, 2026-11-06). Nothing verifies which one the launcher ran; the launcher has no identity check. | **CRITICAL** | §6.3, §6.4, §8.1 | `tools/build-desktop.ps1:56-88` (stamp `BUILD.json`); `HiveMindLauncher.cs:72` (verify `/health.version`) |
| 2 | **No gate record exists for the current HEAD, and the current tree cannot pass `run-gates.ps1`.** Newest `gate-*.json` = `7969f5b9` (2026-11-09); HEAD = `6a41250`. Strict → `E_GATE_DIRTY_TREE`; `-Rehearsal` → G7 artifact/web-dist hash mismatch; `-Release` → exit 3. | **CRITICAL** | §5.2 D1, §8.4 | `tools/qa/run-gates.ps1:577` (rehearsal-aware preReleaseMode) + re-seal or rehearse against a recorded manifest |
| 3 | **Launcher pipe-buffer deadlock.** Both stdio streams redirected, never drained (`RedirectStandardOutput/Error = true`, no readers) → child can block on a full pipe and the user sees a misleading 60 s boot timeout. | **HIGH** | `HiveMindLauncher.cs:66-67` | `HiveMindLauncher.cs:66-67` — drop redirects or attach `OutputDataReceived`/`ErrorDataReceived` |
| 4 | **Seal predicate tests never run in the gate.** `tools/qa/tests/seal-topology.test.mjs` (5 cases) and `tests/trace-tools.test.mjs` are the unit tests for the release-acceptance predicate and trace guard, and `run-gates.ps1` never invokes `node --test`. | **HIGH** | §5.2 D2 (zero grep hits) | `tools/qa/run-gates.ps1` — new step between G5 and G6 |
| 5 | **E2E "pass" is unverifiable.** `.last-run.json` has no counts/timestamp/commit; `run-guarded.mjs` cannot fail a zero-collected run; e2e README describes a 2026-11-06 binary (`184d51a7…`, `index-Cbfm1IuM.js`) that no longer exists in `web/dist` (`index-BNz-sfds.js`). | **HIGH** | §7.2, §7.4 | `e2e/run-guarded.mjs:29`; `e2e/README.md:3-8` (re-state or delete the stale run claim) |
| 6 | **Hardcoded port 8099 in three independent components** (`HiveMindLauncher.cs:16`, `tools/start-hivemind.ps1:4`, `tools/qa/ui-e2e.mjs:22`) plus `9223` for CDP; launcher refuses rather than searching, contradicting `build-desktop.ps1:69` ("free loopback port") and `HiveMindLauncher`'s own error dialog. | **HIGH** | §6.3, §6.4, §5.6 | `HiveMindLauncher.cs:44-56` (scan 8099→8199); `tools/qa/ui-e2e.mjs:22` |
| 7 | **`hivemind-cli.mjs` (80,075 B) has no behavioural test** — the only check anywhere is `node --check` in one evidence JSON. Largest untested surface in the repo. | **HIGH** | §3.5 | `docs/evidence/gate-033bcde0-offline.json` (`cli.command`); add `tools/qa/tests/cli.test.mjs` |
| 8 | **No React component is ever rendered in a unit test** (`environment: 'node'`, no testing-library) — the render path's only proof is Playwright against a possibly-stale bundle. | **MEDIUM** | §4.2 | `web/vite.config.ts:33-37` |
| 9 | **Documentation/code drift on facts that decide gate closure**: `e2e/README.md:34` names `tools/bin/hiverouter.exe` but `live-server.ts:24-28` cannot find it; `GATE-EXCLUSIONS.md:24-27` calls four specs "untracked" that are tracked at HEAD; `e2e/playwright.config.ts:17` defaults `E2E_BASE_URL` to `8080` in Playwright vs `8799` documented. | **MEDIUM** | §7.3, §7.5 | `e2e/README.md`, `e2e/GATE-EXCLUSIONS.md`, `e2e/playwright.config.ts:17` |
| 10 | **Secret-scan blind spot + loose key entry.** G6 scans only `git ls-files`, so untracked `.env`/logs are unscanned (`tools/qa/provider-test.log` already sits outside the evidence convention); `set-provider-key.ps1` accepts any non-empty string and cannot run non-interactively. | **MEDIUM** | §5.2 D6, §6.5 | `tools/qa/run-gates.ps1:537` (also scan untracked non-ignored files); `tools/set-provider-key.ps1:25-27` (shape check) |

**Runners-up (kept explicit, not hidden):** `-Rehearsal`/`-AllowDirty`/`-SkipWebTests` footguns (§6.2); `runtime-test/**` is unreproducible evidence (§8.1); `dist/.release-manifest-*.json` ×3 stale backups inside `dist/` (§8.3); `manifest.max_output_tokens: 393216` is an unverified hard number (§8.3); `tools/test.ps1` enumerates four crates by hand so a fifth crate is silently skipped (§6.1); `hivemind-cli.mjs`'s `node --check` is not a test (§3.5).

---

## 12. Concrete full-battery command list

Copy-paste, from the repo root, in this order. `# EXPECT:` lines are the acceptance condition for each step — do not accept a bare exit code.

```powershell
# ---------------------------------------------------------------- 0. preconditions
cd C:\Users\Riley\Desktop\hiverouter-v1
git status --porcelain                 # EXPECT: empty (else G0 refuses: E_GATE_DIRTY_TREE)
git rev-parse HEAD                     # record COMMIT

# ---------------------------------------------------------------- 1. backend
cd backend
cargo test --workspace --locked        # EXPECT: 559 test fns + doctests, 0 failed, 2 ignored
cargo clippy --workspace --all-targets --locked -- -D warnings   # EXPECT: clean (NOT currently in any gate)
cargo fmt --check                      # EXPECT: clean (NOT currently in any gate)
cd ..

# ---------------------------------------------------------------- 2. web
cd web
npm.cmd run typecheck                  # EXPECT: exit 0
npm.cmd test -- --run                  # EXPECT: 236 passed, 0 skipped
npm.cmd run build                      # EXPECT: exit 0, new web/dist
cd ..

# ---------------------------------------------------------------- 3. QA guard suites (D2: run explicitly)
node --test tools/qa/tests/            # EXPECT: seal-topology 5 + trace-tools, 0 failed

# ---------------------------------------------------------------- 4. static / docs gates
node tools/qa/verify-harness.mjs       # EXPECT: exit 0 (28 surfaces, 3 DIFF cmds, >=30 refusals)
node tools/qa/check-docs-truth.mjs     # EXPECT: exit 0 (version parity, sealed-version claims)

# ---------------------------------------------------------------- 5. e2e against a running app
# start the app FIRST (never from Playwright); e.g. tools\start-hivemind.ps1 or the built exe
$env:E2E_BASE_URL='http://127.0.0.1:8799'; $env:E2E_APP_READY='1'
$env:PLAYWRIGHT_SKIP_BROWSER_DOWNLOAD='1'
cd e2e
node run-guarded.mjs --project=chrome  # EXPECT: 79+ passed, 0 failed, named skips only, exit 0
node -e "const r=require('./test-results/results.json');console.log(JSON.stringify(r.stats))"
# EXPECT: expected>0 — a zero/skip-only run is not a pass
cd ..

# ---------------------------------------------------------------- 6. install smoke (section 10.2)
powershell -NoProfile -ExecutionPolicy Bypass -File tools\build-desktop.ps1
# then run the 10.2 block: EXPECT "OK installed build <sha> == current <version> (commit <COMMIT>)"

# ---------------------------------------------------------------- 7. release-integrity gate
powershell -NoProfile -ExecutionPolicy Bypass -File tools\qa\run-gates.ps1 -DryRun
# EXPECT: exit 0 with no FAIL lines (exit 1 == preflight would refuse; exit 3 == documented pre-release)
powershell -NoProfile -ExecutionPolicy Bypass -File tools\qa\run-gates.ps1 -Rehearsal
# EXPECT from the CURRENT tree: G7 FAIL on target/release + web/dist hash drift (defect D1) -> exit 1
# After re-sealing or on a sealed tree: EXPECT all PASS, exit 0
powershell -NoProfile -ExecutionPolicy Bypass -File tools\qa\run-gates.ps1 -Release
# EXPECT: exit 0 only on a sealed tree (HEAD == tag target); exit 3 == PRE-RELEASE (not a release)

# ---------------------------------------------------------------- 8. paid proof (operator only)
$env:E2E_BASE_URL='http://127.0.0.1:8799'; $env:E2E_APP_READY='1'   # app already running, key present
powershell -NoProfile -ExecutionPolicy Bypass -File tools\qa\run-gates.ps1 -Live -RequireE2E
# EXPECT: exit 0, LIVE_ONE_PROMPT=1, one short prompt per paid gate, key never printed

# ---------------------------------------------------------------- 9. local developer gate (subset)
powershell -NoProfile -ExecutionPolicy Bypass -File tools\test.ps1
# EXPECT: 4 crate gates + web typecheck/tests pass; NOTE it runs neither clippy, e2e, nor G6/G7
```

---

## 13. Coverage enumeration — complete file list

**Backend integration (28):** `approval_ledger.rs`, `chat_store.rs`, `context_switch.rs`, `daemon_tick_degraded.rs`, `graph_authority.rs`, `handshake.rs`, `http_boot.rs`, `hyperagent_governance.rs`, `kill_restore.rs`, `lead_bugcheck_boot.rs`, `mem_isolation.rs`, `prompt_assets.rs`, `prompt_assets_wiring.rs`, `proposal_authorization.rs`, `provider_adapter.rs`, `provider_live.rs`, `provider_streaming.rs`, `recovery_gates.rs`, `recovery_multi_run.rs`, `remote_control_plane.rs`, `run_lifecycle.rs`, `scale_s0.rs`, `scale_s1.rs`, `snapshot_history.rs`, `suspension.rs`, `tool_loop_engine.rs`, `tool_loop_matrix.rs`, `ws_transport.rs` + `support/mod.rs`.

**Unit-test modules (58):** contract 6 · core 23 · provider 8 · tools 21 (names + counts in §1.2).

**Web tests (31):** listed with counts in §1.3. Config: `web/vite.config.ts`, `web/tsconfig.json`, `web/package.json`.

**E2E (16 specs + 8 helpers + 3 config/data):** specs in §1.4; helpers `app.ts`, `contrast.ts`, `evidence.ts`, `live-server.ts`, `matrix.ts`, `replay-harness.ts`, `trace.ts`, `ws-tap.ts`; config `playwright.config.ts`, `tsconfig.json`, `package.json`, `run-guarded.mjs`, `GATE-EXCLUSIONS.md`, `README.md`, `data/control-matrix.json`, root probes `data-probe.mjs`, `live-check2.mjs`, `live-inspect.mjs`, `live-probe.mjs`, `live-shot.mjs`, `live-smoke.mjs`.

**tools/qa (14):** `run-gates.ps1`, `check-docs-truth.mjs`, `check-seal-topology.mjs`, `verify-harness.mjs`, `apply-live-trace.mjs`, `build-trace-seeds.mjs`, `figcraft-assets.mjs`, `ui-e2e.mjs`, `check-env.ps1`, `apply_wp01.py`, `provider-test.log`, `README.md`, `tests/seal-topology.test.mjs`, `tests/trace-tools.test.mjs`.

**Build/install (10):** `tools/build-release.ps1`, `tools/build-desktop.ps1`, `tools/test.ps1`, `tools/start-hivemind.ps1`, `tools/set-provider-key.ps1`, `tools/set-provider-profile.ps1`, `tools/hivemind.ps1`, `tools/hivemind.cmd`, `tools/desktop-launcher/{HiveMindLauncher.cs,HiveMind.exe,README.md}`, `tools/release-manifest.json`, `tools/runbook.md`, `tools/hivemind-cli.mjs`.

**Evidence/ledgers/dist:** `docs/evidence/**` (gate JSONs, scale JSONs, test logs, paid logs, `key-validation.md`), `ledgers/ui-watchdog.{md,json}`, `dist/HiveMind/`, `dist/HiveMind-1.1.0-win64{,.zip,.zip.sha256}`, `dist/HiveMind-v1-win64{,.zip}`, `dist/HiveMind.zip`, `dist/.release-manifest-*.json` ×3, `runtime-test/**`.

**Loader files:** `HiveMind.cmd`, `HiveMind-Agent.cmd`, `HiveMind-Agent.ps1`, `AGENTS.md`, `CLAUDE.md`, `rust-toolchain.toml`, `.cargo/`.

**Not inspected (declared):** `tools/w64devkit/**` (vendored toolchain), `e2e/node_modules/**`, `e2e/playwright-report/**`, `target/**` beyond hashes, `.gitnexus/**`, `skills/**`.

---

## 14. Gap register (explicit, no omissions)

| # | Gap | Kind | Where |
|---|---|---|---|
| G-1 | No committed gate record at HEAD | evidence | `docs/evidence/gate-*.json` (newest `7969f5b9`) |
| G-2 | `run-gates.ps1` cannot return 0 on this tree | gate | `run-gates.ps1:224-228`, `:577`, `:609-618` |
| G-3 | `tools/qa/tests/**` never executed | gate | nothing invokes `node --test` |
| G-4 | Zero-collected e2e run is not a failure | gate | `run-guarded.mjs:25-35` |
| G-5 | `-Live` does not pin the app build | gate | `run-gates.ps1:470-477` |
| G-6 | G6 skips untracked files | security | `run-gates.ps1:537` |
| G-7 | No install identity stamp / check | install | `build-desktop.ps1:56-88`, `HiveMindLauncher.cs:72` |
| G-8 | Launcher stdio deadlock | install | `HiveMindLauncher.cs:66-67` |
| G-9 | Hardcoded 8099/9223 in 4 places, no port search | install | `HiveMindLauncher.cs:16,44-56`, `start-hivemind.ps1:4`, `ui-e2e.mjs:22-23` |
| G-10 | Personal absolute paths in shipped source | hygiene | `HiveMindLauncher.cs:17`, `desktop-launcher/README.md:30` |
| G-11 | `hivemind-cli.mjs` untested | coverage | no test file |
| G-12 | No DOM test environment | coverage | `web/vite.config.ts:34` |
| G-13 | No clippy / fmt in any gate | quality | `tools/test.ps1`, `run-gates.ps1` |
| G-14 | No coverage instrumentation / no CI | quality | repo-wide (no `.github`) |
| G-15 | `runtime-test/**` unreproducible, stale hashes | evidence | `runtime-test/pkg-final/CHECKSUMS.txt` |
| G-16 | `tools/bin/hiverouter.exe` not in `BINARY_CANDIDATES` | tooling | `e2e/helpers/live-server.ts:24-28` vs `e2e/README.md:34` |
| G-17 | Stale e2e run claim (79 passed on a binary that no longer exists) | docs | `e2e/README.md:3-8` |
| G-18 | GATE-EXCLUSIONS says "untracked" for files tracked at HEAD | docs | `e2e/GATE-EXCLUSIONS.md:24-27` |
| G-19 | `build-desktop.ps1 -Configuration` cannot affect the build it copies | bug | `build-desktop.ps1:40` vs `:59` |
| G-20 | Non-reproducible desktop zip; no manifest coverage of `dist/HiveMind` | release | `build-desktop.ps1:96`, `run-gates.ps1:602-665` |
| G-21 | `manifest.max_output_tokens: 393216` is an unverified hard number | claims | `tools/release-manifest.json` |
| G-22 | `provider-test.log` in `tools/qa/` outside evidence convention | hygiene | `tools/qa/provider-test.log` |
| G-23 | Suspension suite carries an untracked open gap | coverage | `suspension.rs:39` |
| G-24 | `tools/test.ps1` enumerates crates by hand | coverage | `tools/test.ps1:38-49` |
| G-25 | Playwright `E2E_BASE_URL` default `8080` ≠ documented `8799` | config | `playwright.config.ts:17` |

---

## 15. Verification table (what I actually did)

| Claim | How established | Status |
|---|---|---|
| 186 integration test fns / 28 files / 2 ignores | regex count + read of `support/mod.rs` in full | **VERIFIED** |
| 373 unit test fns across 58 modules | regex over `src/**/*.rs` filtered by `#[cfg(test)]` | **VERIFIED** |
| 236 web test sites / 31 files / 0 skips | regex count per file | **VERIFIED** |
| 95 e2e test sites / 48 gated skip sites | regex count per spec | **VERIFIED** |
| Harness uses real server/DB/WS, ephemeral port | read `support/mod.rs:42-102`, `:156-318` | **VERIFIED** |
| `run-gates.ps1` is fail-closed with named codes | read `:87-92`, `:110-131`, `:178-467`, `:536-571`, `:602-711`, `:713-753` | **VERIFIED** |
| Skips never count as PASS | read `:128`, `:133`, `:718`, `:731`, `:735-749` | **VERIFIED** |
| Current tree is dirty (1 untracked path) | `git status --porcelain` | **VERIFIED** (command output) |
| Strict gate would refuse on dirty tree | code at `:224-228` + measured dirty count | **VERIFIED** (code path; not executed) |
| `-Rehearsal` would fail G7 | `:577` + measured hashes below | **VERIFIED** (hash measurement; gate not executed) |
| `target/release/hiverouter.exe` sha/bytes drift | `Get-FileHash` + `Get-Item` | **VERIFIED** |
| `target/release/hiverouter-cli.exe` sha/bytes drift | `Get-FileHash` + `Get-Item` | **VERIFIED** |
| `web/dist` asset drift vs manifest | `Get-FileHash` over 25 files | **VERIFIED** |
| Four distinct `hiverouter.exe` builds on disk | `Get-FileHash` on each | **VERIFIED** |
| 1.1.0 package binary matches manifest | sha `9c3174fb…` == manifest artifact | **VERIFIED** |
| `tools/bin` == `runtime-test/pkg-final` CHECKSUMS == e2e README binary | all three sha `184d51a7…` | **VERIFIED** |
| `node --test` never invoked by any gate | grep across `tools/qa/*.ps1`, `tools/*.ps1`, both `package.json`, `README.md` → 0 hits | **VERIFIED** |
| Launcher redirects stdio without draining | read `HiveMindLauncher.cs:59-71` in full | **VERIFIED** |
| Launcher hardcodes 8099, no port search | read `:16`, `:44-56` | **VERIFIED** |
| DPAPI key handling is correct (hidden input, CurrentUser, zeroing) | read `set-provider-key.ps1:17-50` in full | **VERIFIED** |
| `run-gates.ps1` G6 scans only tracked files | read `:536-571` | **VERIFIED** |
| e2e `.last-run.json` carries no counts | read the file | **VERIFIED** |
| Newest gate JSON is `7969f5b9` @ 571 cargo / 191 web | `ConvertFrom-Json` + `LastWriteTime` sort | **VERIFIED** |
| Any test suite passes right now | **not executed** — read-only lane, no servers started | **UNVERIFIED** |
| `/health` exposes a `version` equal to the crate version | `live-server.ts:57,234` models it; I never observed a live `/health` | **UNVERIFIED** |
| `dist/HiveMind-1.1.0-win64.zip` sha == manifest | not recomputed | **UNVERIFIED** |
| 571 (gate JSON) vs 559 (static count) explained by doctests | arithmetic consistent with +12, not proven | **UNVERIFIED** |
| G-1..G-25 fixes | my recommendations | **PROPOSED** |
| §10.2 install smoke, §12 additions (clippy/fmt/`node --test` step) | my recommendations | **PROPOSED** |

**Stub sweep (declared, per §23):** the sweep target for this lane is test/build tooling, and I read every in-scope script in full or in the cited line ranges. I found **no fabricated success path** and **no `NODE_ENV=test`-style branch** in any production file I read. Legitimate §7 test doubles observed and classified as **permitted**: `tempfile` dirs, temp SQLite DBs (`support/mod.rs:63`), `wiremock` HTTP doubles in the provider crate, tokio timeouts, Playwright's `routeWebSocket` client mock in the static-fixture specs (documented as such in `GATE-EXCLUSIONS.md:3` — and that mock is exactly why G-UI-3/10 are *excluded* rather than claimed). The e2e `E2E_ALLOW_ALL_SKIPS=1` and `tools/test.ps1`'s "recorded as not-run, not passed" branch are **honest non-claims**, not fabricated passes, though both are footguns (G-4, G-24).
