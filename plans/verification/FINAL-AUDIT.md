# FINAL INDEPENDENT AUDIT — HiveMind 1.0 release

- **Status: FINAL — all 7 mandated checks executed and evidenced below (interim request satisfied by this file; no further repo-wide re-read needed).** Core checks re-confirmed on disk at write time (see §1–§8).
- **Auditor:** independent verification seat (read-only; this file is the only write).
- **Repo:** `C:\Users\Riley\Desktop\hiverouter-v1`.
- **Window:** 2026-11-06 11:28–11:55 America/Chicago. No product/git writes, no key access, no paid calls.
- **Verdict: PASS-WITH-NOTES.** Every mandated hash, count, tag, status, security property, source claim, manifest field-presence, evidence claim and release-plan claim verified against the tree with quoted output; **no contradiction of the artifact hashes/counts/tags/key material was found**. Notes are documentation/provenance items, the single material one being that the manifest's `web_dist_sha256` is not reproducible from `web/dist` with any documented or tested method (N1).

## 1. Git identity, tags, quiescence

| Check | Observed (quoted) | Verdict |
|---|---|---|
| `git rev-parse HEAD` | `74be0f46dc3dc349c238dc6fbc50fd182f68ce0f` (= expected `74be0f4…`) | PASS |
| `git rev-parse hivemind-1.0.0` | `88f109bbc5a60abcb482fcbaa03566f1c57fbbfb` — this is the **annotated tag object**; `git rev-parse hivemind-1.0.0^{commit}` = `74be0f46dc3dc349c238dc6fbc50fd182f68ce0f` = HEAD (tag message: "HiveMind 1.0.0 official build: provider wired, live-proven, key-isolated, replay/16-state; artifacts b17eb3d6/49f46e2b/a49dbb2b") | PASS (note N2: raw rev-parse differs by tag object only) |
| `git status --porcelain` | **0 lines — clean tree** | PASS |
| Old `v1.0.0` | `3988622d6ef65ff39a0c8bb126862a8e8405ffd9` (lightweight commit tag) — matches the execution plan's recorded private checkpoint (`3988622`, pre-fix) and was not moved by this release (plan §1/§8: "old private `v1.0.0` tag is left untouched"); the older FINAL-CLOSEOUT record (`6815dbe`) is historical and superseded | PASS (note N5) |
| Tags present | `hivemind-1.0.0` (annotated), `v1.0.0` (commit) | PASS |

## 2. Artifacts — all three hashes match exactly

| Artifact | Bytes | SHA-256 (observed) | Expected | Verdict |
|---|---:|---|---|---|
| `target/release/hiverouter.exe` | 13,253,120 | `b17eb3d6469c347f623a4dedc94537b15bb633a76215828742624fa6fe062a9a` | same | **MATCH** |
| `target/release/hiverouter-cli.exe` | 5,373,952 | `49f46e2be488dd675280f3dae7e8d455ac5eeec2aa46d7fd44f1c2557c386623` | same | **MATCH** |
| `dist/HiveMind-1.0.0-win64.zip` | 8,444,038 | `a49dbb2bda65d0c3ad500157e04960638c54e2b08df8c8da5885583cf0e895e8` | same | **MATCH** |

- Zip opens with **6 members**: `CHECKSUMS.txt` (401 B), `hiverouter-cli.exe` (5,373,952), `hiverouter.exe` (13,253,120), `README.md` (2,113), `RELEASE.md` (3,287), `RUNBOOK.md` (5,386) — **PASS**.
- Every member hash recomputed from inside the archive equals `CHECKSUMS.txt`: exe `b17e…`, cli `49f4…`, README `16cf03b7…`, RUNBOOK `40cb431c…`, RELEASE `10d32112…` — **all MATCH**.
- Sidecar `dist/HiveMind-1.0.0-win64.zip.sha256` = `a49dbb2b…e895e8` = actual zip — **MATCH**.
- Zip members equal the repo files: `README.md` repo sha256 `16cf03b7…`; `tools/runbook.md` `40cb431c…`; `docs/RELEASE.md` `10d32112…` — identical bytes — **PASS**.
- Binary truth: the release binary embeds the **current** dist asset names `index-6T84PJfk.js` + `index-BWOl__uM.css` (ASCII scan `True/True`) and does **not** contain the old `index-Cbfm1IuM.js` (`False`) — **PASS**.

## 3. Source at sealed commit `6a2b1c6`

| Check | Observed | Verdict |
|---|---|---|
| Sealed commit | `6a2b1c6d461a81e7ee66951d0d0eb6b1063bf540` exists (`git cat-file -t` = commit; 2026-11-06 11:20:33 -0600) | PASS |
| Version | `version = "1.0.0"` in all four `backend/crates/*/Cargo.toml` | PASS |
| Provider dep in core | `backend/crates/hiverouter-core/Cargo.toml:21` → `hiverouter-provider = { path = "../hiverouter-provider" }` | PASS |
| `build_executor` wiring | `core/src/http.rs:21` imports `ProviderExecutor`/`UnconfiguredExecutor`; `:25` `ProviderExecutorAdapter`; `:67 fn build_executor`; `:68` `if !config.provider.enabled`; `:70` returns `UnconfiguredExecutor`; `:72` `ProviderExecutorAdapter::from_env(...)`; `:79` honest fallback; called at `:101,:141,:178` | PASS |
| Replay arm | `web/src/flow/Canvas.tsx:54` `history.replayGraph`; `:114` `buildReplayTasks(...)`; `:118` `replayNodes`; `:135-136` nodes/edges switch to replay when snapshot present; read-only (`noopSelect`) | PASS |
| 16 states incl `interrupted` | `web/src/state/agentState.ts` = healthy, running, queued, failed, completed, cancelled, killed, planning, thinking, awaiting_approval, streaming, reconnecting, replay, empty, loading, **interrupted** — count **16** | PASS |
| Failed is terminal | `agentState.ts:111-119 isTerminal()` returns true for `failed`, `completed`, `cancelled`, `killed`, `interrupted` | PASS |
| Child spawn removes key | `hiverouter-tools/src/shell.rs:203` `.env_remove(hiverouter_provider::API_KEY_ENV)`; `cli_support.rs:42` same; gate log `test shell::tests::child_process_does_not_inherit_the_provider_key ... ok`; test at `shell.rs:405` | PASS |
| HEAD vs sealed commit | `git diff --name-only 6a2b1c6..HEAD` = 73 files, **none under `backend/` or `web/src/`**; non-doc entries are `.gitignore`, `dist/HiveMind-1.0.0-win64.zip`, `dist/HiveMind-1.0.0-win64.zip.sha256` → **no product-source drift between sealed commit and HEAD** | PASS |

## 4. Logs — counts recomputed from the logs, not quoted from prose

- `docs/evidence/final-gate-tests.log`: `HEAD=6a2b1c6d461a81e7ee66951d0d0eb6b1063bf540`; `EXIT=0`; 26 `test result` lines → **237 passed / 0 failed / 2 ignored** (sum recomputed). Suites include contract 18+3+6+5, core lib 37, http_boot 6, kill_restore 2(+1 ignored), live-provider 0(+1 ignored), ws_transport 13, provider 29+11, tools 78+1 — **matches claim 237/2**.
- `docs/evidence/final-web-tests.log`: `HEAD=6a2b1c6…`; 8 files / **46 passed (46)**; `EXIT=0` — **matches**.
- `docs/evidence/final-web-build.log`: `HEAD=6a2b1c6…`; `TYPECHECK_EXIT=0`; `BUILD_EXIT=0` (vite 8.3.6 build green, index 0.77 kB, css 35.75 kB, js 672.26 kB) — **matches**.

## 5. Security

- **Strict credential-pattern scan** (`sk-[A-Za-z0-9]{32,}` over all 677 tracked non-vendor files): **0 hits**.
- No key material in `tools/release-manifest.json`, `docs/RELEASE-EVIDENCE.md`, `docs/evidence/*`, release docs, or the **text members of the zip**: all `key_hits=0`.
- Child-process isolation: code + green regression test (see §3); release docs state child spawns remove the key and **do not imply inheritance** (`RELEASE-EVIDENCE.md:25`, plan §1/§4; `live-provider-smoke.md:5` refers only to the server process environment).
- **Note N3:** a **broad literal `sk-` substring scan** is *not* zero — 8 non-secret files: `provider/src/client.rs` (`sk-debug-test-value`, `sk-validation-test`), `provider/src/secret.rs` (`sk-unit-test-only-value`), `provider/tests/refusal_matrix.rs` (`sk-wiremock-offline-only`), `tools/src/shell.rs` (`sk-shell-env-test-not-a-real-key`), plus tracked binaries containing embedded test/token substrings. All are obvious fixtures; **no credential**. Release wording "`key scan … 0 occurrences of a sk-… credential`"/"tracked-tree key scan = 0 hits" is true for credentials but should be stated as "strict credential pattern".

## 6. Release manifest

`tools/release-manifest.json` fields: `source_commit = 6a2b1c6d461a81e7ee66951d0d0eb6b1063bf540` ✓ · `toolchain = rustc 1.99.1 (58cf8f9c3 2026-10-13)` ✓ · `web_dist_sha256 = 86a1c088dc912517b7abf135f71f0b18526d36bc4e0bde8aa160972353673844` · `artifacts` = 3 rows with exact hashes/bytes (§2) ✓ · `artifact_built_at = 2026-11-06T11:25:01…` · `note` claims artifacts rebuilt from source commit ✓ · **absolute-path scan = 0** ✓.

- **Note N1 (only unverified item):** `web_dist_sha256` **could not be reproduced** from `web/dist` — tested raw byte concatenation in multiple orders, path+hash line lists (LF/CRLF, relative/absolute paths), hash-only concatenations, JSON lists: **0 matches**. The tracked `tools/build-release.ps1` emits only artifact path/hash rows and cannot generate `source_commit`/`toolchain`/`web_dist_sha256` (the manifest was written by a step not committed). No contradiction was proven (dist mtime 11:21:13 < binary 11:24:14 < manifest 11:25:01, and the binary embeds current dist names), but the hash's referent/algorithm is undocumented and therefore **UNVERIFIED**. Fix: document the exact web-dist algorithm and regenerate via a committed script (already roadmapped as W7), or replace the field with per-file dist hashes.

## 7. Evidence docs and release plan

- `docs/RELEASE-EVIDENCE.md` claims match artifacts/logs/code: artifact table exact (§2); "237 passed / 0 failed / 2 ignored" matches recomputed sums (§4); 46/46 + build/typecheck match; provider wiring matches §3; child-isolation matches §3; source commit `6a2b1c6` matches manifest; tag gate text consistent.
- Live-provider claims (`RELEASE-EVIDENCE.md` §"Release-binary live smoke"; `docs/evidence/live-provider-smoke.md`; `key-validation.md`) are **operator-recorded READ-level evidence**: no key material present, no raw HTTP/WS capture with independent identifiers beyond the recorded response id/usage; they cannot be re-executed by a read-only auditor without key access. **No claim is mislabelled as independently VERIFIED.** (Note N4.)
- **Note N6:** `docs/RELEASE.md` (also shipped in the zip, hash `10d32112…`) still carries the header **"(draft; update hashes at final build)"** even though the build is sealed; its content is otherwise final-consistent (it points to `docs/RELEASE-EVIDENCE.md` for exact counts/hashes). Update the header/labels.
- Release plan `plans/HIVEMIND-1.0-EXECUTION-PLAN.md` present; **8** seat dirs `plans/seats/SEAT-01..08` each with `REPORT.md`, `READLOG.md`, `EVIDENCE.md`, `GATE-CY-1/2/3.md`; **4** planner dirs `plans/planners/PLANNER-01..04` with reports + gates (+ READLOG for some). Spot-checked **5** claims, all verify: (1) artifact hashes in the plan match §2 exactly; (2) old `v1.0.0` = `3988622`; (3) sealed-commit test counts 237/0/2 and 46/46; (4) child spawns remove the key (test green); (5) replay snapshot (2 nodes, read-only canvas) — READ evidence from `RELEASE-EVIDENCE.md:23` plus code §3.

## 8. Contradictions

None found for artifact hashes, sizes, zip members/checksums, tag/HEAD identity, git cleanliness, version, provider wiring, replay, state count, child-key isolation, log sums, or manifest fields. The items below are **notes**, not contradictions:

| # | Note | Evidence | Action |
|---|---|---|---|
| N1 | `web_dist_sha256` unreproducible/undocumented; manifest not generated by tracked script | §6 attempts; `tools/build-release.ps1` content | Document algorithm + committed generator, or per-file dist hashes |
| N2 | Raw `git rev-parse hivemind-1.0.0` = tag object `88f109b`; expected `74be0f4` only via `^{commit}` | §1 | Use `^{commit}` in gate commands/docs |
| N3 | Broad `sk-` substring scan ≠0 (test fixtures/binaries); strict credential pattern =0 | §5 | Reword "0 occurrences of a sk-… credential" precisely |
| N4 | Live-provider claims are operator-READ, not independently re-runnable | §7 | Keep labelled READ/UNVERIFIED in any external claim |
| N5 | Old `v1.0.0` = `3988622` (intended private checkpoint; earlier FINAL-CLOSEOUT `6815dbe` superseded) | §1; plan §1/§8 | None (documented) |
| N6 | Shipped `docs/RELEASE.md` still titled "draft; update hashes at final build" | §7; zip member hash `10d32112…` | Update header before publishing externally |

## 9. What was NOT independently verified

- The live provider calls themselves (no key access/no paid calls allowed): key-validation + live smoke are READ evidence only.
- `web_dist_sha256` content (N1) — the only hash in the release package whose referent could not be reproduced from the tree.
- Tests/e2e were **not re-executed** (read-only audit; no rebuilds). Logs were verified for internal consistency, HEAD binding (`HEAD=6a2b1c6`), exit codes, and recomputed sum counts; they are not a substitute for a fresh run on another machine.
- Streaming/tool-call/pixel-fidelity claims are explicitly out of scope/1.1 per release docs and were not assessed here.

**Final: PASS-WITH-NOTES.** Release artifacts, source seal, tag/HEAD, clean tree, test/web log counts, provider wiring, replay arm, 16-state parity, key isolation and child-key removal, manifest fields and plan/seat corpus all verify. The one non-verifying item is the manifest's undocumented `web_dist_sha256` (N1); the other notes are documentation hygiene. No hash, count, tag, or key-material contradiction was found.
