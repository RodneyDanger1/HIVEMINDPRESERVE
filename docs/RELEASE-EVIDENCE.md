# HiveMind 1.0.0 — release evidence

Date: 2026-11-06 (America/Chicago). Product commit: sealed source `6a2b1c6`; artifacts committed after it. No key material appears in this file. Independent final audit: `plans/verification/FINAL-AUDIT.md` — PASS-WITH-NOTES; the web-dist manifest note was fixed by recording per-file dist hashes (no opaque aggregate hash).

## Artifacts

| Artifact | SHA-256 | Bytes |
|---|---|---:|
| `hiverouter.exe` | `b17eb3d6469c347f623a4dedc94537b15bb633a76215828742624fa6fe062a9a` | 13,253,120 |
| `hiverouter-cli.exe` | `49f46e2be488dd675280f3dae7e8d455ac5eeec2aa46d7fd44f1c2557c386623` | 5,373,952 |
| `dist/HiveMind-1.0.0-win64.zip` | `46305fcb1fe1bee23d3645a94a82635a68ee47fbcd3ce0537df747de3c6105ef` | 8,444,021 |

The zip contains both binaries, `README.md`, `RUNBOOK.md`, `RELEASE.md`, and `CHECKSUMS.txt`; an extraction smoke listed all six files and verified the archive opened.

## Release-binary live smoke (not the debug binary)

Started `target/release/hiverouter.exe` on `127.0.0.1:8800` with a fresh data directory and the key only in the process environment.

- `GET /health` → `ok`, version `1.0.0`.
- `GET /` → 200, embedded UI HTML present (`id="root"`), 774 bytes.
- `SETTINGS_STATUS` → `{"provider":"deepseek","model":"deepseek-flash","key_present":true,"thinking":"enabled","reasoning_effort":"high"}`.
- `SUBMIT_PROMPT` `Reply with exactly HIVE-ARTIFACT-OK` → `RUN_CREATED` → nodes/reply events → `REPLY_DONE` usage `{prompt:91, completion:8, total:99}` → `RUN_STATUS completed`; reply `HIVE-ARTIFACT-OK`.
- `HISTORY_REVISION_SET` revision 0 → `HISTORY_SNAPSHOT` with **2 graph nodes** (real snapshot; not fabricated).
- The key never appeared in the server log, settings payload, or evidence.
- Child-process isolation: shell/tool spawns remove `DEEPSEEK_API_KEY` before execution (`shell.rs`, `cli_support.rs`), pinned by the new `child_process_does_not_inherit_the_provider_key` test.
- Source commit for this build: `6a2b1c6d461a81e7ee66951d0d0eb6b1063bf540`; manifest: `tools/release-manifest.json`; web-dist hashes are recorded **per file** in the manifest (`web_dist_files`: `index.html` `8264ae92…8776d`, `index-6T84PJfk.js` `b771bc2e…49bc`, `index-BWOl__uM.css` `96ffa873…3a748`; the former opaque aggregate `86a1c088…` was retired by release commit `87dada4`).

## Test evidence at integration baseline

- Rust workspace on sealed commit `6a2b1c6`: **237 passed / 0 failed / 2 ignored** (`docs/evidence/final-gate-tests.log`, includes HEAD/command/exit); the ignored tests are the explicit live-provider test and one live-gated test.
- Web: **46/46** unit tests, typecheck 0 errors, Vite production build green (`docs/evidence/final-web-tests.log`, `final-web-build.log`).
- Live provider adapter: planner node + `HIVE-PROVIDER-OK`, usage 99 tokens (`docs/evidence/live-provider-smoke.md` has the server-level run).
- Key scan: strict credential pattern (`sk-[A-Za-z0-9]{32,}`) = **0 occurrences across 677 tracked non-vendor files**; literal `sk-` substrings exist only as obvious unit-test fixtures (`provider/src/client.rs`, `secret.rs`, `tests/refusal_matrix.rs`, `tools/src/shell.rs`) and embedded binary strings — no key material. Tracked-tree child spawns remove the key.

## Known limits (also in `docs/RELEASE.md`)

- Leaf reply is emitted as one delta after a non-streaming completion; true SSE chunk rendering is 1.1.
- Autonomous model tool-calling, full trace ladder, pixel fidelity and Tauri are 1.1.
- Normal tests never call the provider; live tests are explicit and budget-capped.
- The supplied key was exposed in chat; rotate after release if the transcript is shared.

## Final tag gate

Tag only after the independent audit and quiescent-tree check. Moving the previous private `v1.0.0` tag requires owner confirmation and records the superseded commit.
