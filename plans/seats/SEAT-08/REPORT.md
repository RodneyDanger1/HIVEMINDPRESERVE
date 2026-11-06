# SEAT S8 — Docs/Plan Compiler REPORT (HiveMind 1.0 release documentation vs reality)

Seat: S8 strip-watchdog (Docs/Plan Compiler). Read-only; writes only `plans/seats/SEAT-08/`.
Repo `C:\Users\Riley\Desktop\hiverouter-v1`; baseline HEAD **`5c69b66`** (2026-11-06 10:59:06 CT) + **uncommitted product edits present**.
Scope: audit tools (`build_appendix.py`, `synthesize_paper.py`); product docs slice (release/claims/runbooks); `audit/ROADMAP.md`; `audit/ROADMAP-TASKS.csv`; `README.md`; `docs/evidence/*`; `plans/HIVEMIND-1.0-SEAT-ASSIGNMENTS.md`.

## 1. Verified facts (evidence-backed)

| # | Fact | Evidence (path:line / command / URL+date) | Label |
|---|---|---|---|
| 1 | Provider is wired into core **at HEAD source**: core depends on `hiverouter-provider`; `http.rs` builds `ProviderExecutorAdapter` from env with honest `UnconfiguredExecutor` fallback | `backend/crates/hiverouter-core/Cargo.toml:21`; `core/src/http.rs:25,67,68,72,79`; `core/src/provider_executor.rs` (10,589 B, 10:56:06) | VERIFIED (read) |
| 2 | Live provider evidence exists for the in-progress integration: key check HTTP 200 `HIVE-KEY-OK` usage 20; live `SUBMIT_PROMPT`→provider→`RUN_STATUS completed`, reply `HIVE-LIVE-OK`, usage 95 | `docs/evidence/key-validation.md:6-13`; `docs/evidence/live-provider-smoke.md:5-12` | READ (self-reported evidence, key/paid call not re-run by S8) |
| 3 | UI state parity is 16/16 at HEAD (incl. `interrupted`); replay path now builds nodes/edges from `history.replayGraph` | `web/src/state/agentState.ts:8-24`; `web/src/flow/Canvas.tsx:54,113-122` | VERIFIED (read) |
| 4 | **Shipped artifacts predate the provider integration**: `tools/bin/hiverouter.exe` 9,329,152 B sha256 `184D51A7…EAC9A9`, mtime **09:10:59**; `dist/HiveMind-v1-win64.zip` 6,571,928 B sha256 `BC7A2BAD…27FEE7`, mtime **09:33:17**; provider commit **10:59:06**; `web/dist/index.html` **10:58:47** | `Get-Item`/`Get-FileHash` (EVIDENCE §2); `git log -1 --format=%ci` | VERIFIED |
| 5 | **Tree is not quiescent**: 6 modified product files (+20/−15) — `backend/Cargo.lock`, 4 crate `Cargo.toml`, `web/package.json` — plus untracked `docs/RELEASE.md`, `tools/release-manifest.json` | `git status --short`; `git diff --stat` | VERIFIED |
| 6 | Claims register is internally consistent per row count: 80 rows = 72 VERIFIED / 8 READ / 0 UNVERIFIED / 0 CONTRADICTED; C-026 VERIFIED | `docs/claims/CLAIM-REGISTER.csv` (parsed fully, 81 lines) | VERIFIED |
| 7 | Roadmap is 101 tasks / 6 phases / 8 seats | `audit/ROADMAP.md:1-14`; `audit/ROADMAP-TASKS.csv` (101 rows parsed) | VERIFIED |
| 8 | Trace ladder is honestly partial: 29 controls (23 PARTIALLY_VERIFIED, 6 PENDING), 43 WIRED-AND-WORKS / 270 PENDING / 6 MISSING across 319 hops | `docs/trace/controls.json` (parsed) | VERIFIED |
| 9 | Official DeepSeek API facts match the product's provider contract (endpoint, model ids, usage fields, thinking controls) and the product's peak/off-peak cost arithmetic (1.506 peak, 0.753 off-peak) | 4 docs pages (§READLOG §2); `hiverouter-cli provider cost` observed in PHASE-12-EVIDENCE:78-82 | VERIFIED (docs) / READ (CLI transcript) |
| 10 | Audit paper is explicitly based on HEAD `3988622` and still reports A-F01/F02/F03 as P0 | `audit/HIVEMIND-AUDIT-PAPER.md:4,11` | VERIFIED (read) |

## 2. Stale / contradictory statements in release documentation (must fix before release claim)

Severity: **B** = blocking release claim; **M** = materially misleading; **L** = hygiene.

| ID | File:line | Claim as written | Reality (evidence) | Sev |
|---|---|---|---|---|
| S-1 | `docs/RELEASE.md:5,9-17` | "real DeepSeek provider path… `SUBMIT_PROMPT` reaches the provider"; "verified in the build"; exact counts/hashes "in docs/RELEASE-EVIDENCE.md" | Source wiring exists only at HEAD (uncommitted, post-10:59 commit); **shipped binary `184D51A7…` (09:10) and zip (09:33) predate it**; `docs/RELEASE-EVIDENCE.md` **does not exist** | **B** |
| S-2 | `README.md:24` | package `dist/HiveMind-v1-win64.zip` sha256 `9B2ED5A7…` | actual zip **6,571,928 B `BC7A2BAD…`** (09:33:17) — different size and hash | **B** |
| S-3 | `docs/plans/PHASE-12-EVIDENCE.md:139` | zip 4,065,104 B sha256 `9B2ED5A7…` | actual zip 6,571,928 B `BC7A2BAD…`; FINAL-REPORT:12 already carries the correct value → docs contradict each other | **B** |
| S-4 | `docs/claims/CLAIM-REGISTER.csv` row C-340 | "shipped v1 artifact hash and package are final…"; notes zip 4,065,104 B `9B2ED5A7…` | actual zip 6,571,928 B `BC7A2BAD…`; claim VERIFIED on stale data | **B** |
| S-5 | `docs/reports/FINAL-REPORT.md:26` | UI has "15-state enum" | HEAD has **16** states incl `interrupted` (`agentState.ts`); RELEASE.md:16 says 16 → internal contradiction | M |
| S-6 | `docs/reports/FINAL-REPORT.md:45` | trace "43 WIRED-AND-WORKS; 260 UNVERIFIED; 5 MISSING" | `docs/trace/controls.json`: 43 WIRED-AND-WORKS / **270 PENDING** / **6 MISSING** of 319 (paper itself already noted this drift as A-F10) | M |
| S-7 | `docs/reports/FINAL-REPORT.md:51`, `README.md:26` | "No paid provider run was made… UNVERIFIED" | live-provider evidence docs dated **10:57** now claim a real provider call/run; FINAL-REPORT predates them → stale statement (and still no *release-artifact* live run) | M |
| S-8 | `docs/reports/FINAL-REPORT.md:32-47` | 233 backend / 46 frontend / 79 e2e / 14 skips "all re-run on the final tree" | those logs predate provider commit (`final-gate-tests.log` 08:41; e2e log 09:21); no post-integration test/e2e run exists | M |
| S-9 | `audit/HIVEMIND-AUDIT-PAPER.md:11,26-28` | A-F01 "shipped server cannot call the model", A-F02 replay dead, A-F03 16 vs 15 | true of the **shipped artifact**; fixed in **HEAD source** (verified fact 1–3). Paper is not wrong, but must not be read as current-tree state; needs a supersession banner | M |
| S-10 | `docs/RELEASE.md:16` | "State parity: backend 16 and web 16" | true at HEAD source but not in shipped embedded UI (dist rebuilt 10:58:47 after binary) | M |
| S-11 | `docs/RELEASE.md:5` "first official build"; `:33-35` release gate | Draft title/date/version and "annotated tag after audit" gate; `v1.0.0` tag currently points at `3988622` (pre-integration), repo HEAD is `5c69b66` + dirty | M (tag/commit mismatch; versioning must be re-cut) |
| S-12 | `docs/RELEASE.md:24` | "The key was supplied in a chat transcript; rotate it after this release" | security note is correct/needed; ensure the transcript-adjacent key is rotated (status UNVERIFIED here) | L |
| S-13 | `docs/plans/PHASE-12-EVIDENCE.md:25-29,138` | historical hashes 8,384,512/31aea… alongside final 9,329,152/184D51A7 (and stale zip) | history is fine, but the "Final shipped artifact" block mixes two eras; add "superseded/pre-integration" labels | L |
| S-14 | `FINAL-REPORT.md:1-4` / docs generally | "shipped v1" framing | no document states the shipped artifact is pre-provider-integration; readers will assume RELEASE.md claims describe the shipped binary | **B** (consequence of S-1) |

**Verified non-contradictions:** binary hash `184D51A7…` is consistent across README/FINAL-REPORT/PHASE-12 and matches the file; claims-register row counts are internally consistent; roadmap 101/6/8 consistent across paper, ROADMAP.md, ROADMAP-TASKS.csv; audit tools are deterministic doc generators without product claims.

## 3. File-by-file plan (Lead-owned edits; S8 read-only)

1. **`docs/RELEASE.md`** — replace draft with final structure (§4); title/version/date fixed to the commit actually built; artifacts table generated from `docs/RELEASE-EVIDENCE.md`; explicitly mark shipped `184D51A7…` as the **pre-integration** artifact until rebuild; remove "verified in the build" if evidence paths absent; add config table and known limitations.
2. **`docs/RELEASE-EVIDENCE.md` (new)** — create from the final build: commit C, `git status` clean, toolchain versions, build command + exit, sha256/size for server/CLI/zip, four-crate test counts, web typecheck/unit/build, e2e exact-binary result, live probe outputs, live-provider run (no key material), trace/claim snapshots; one file the release notes cite.
3. **`docs/reports/FINAL-REPORT.md`** — fix 15→16 states (L26), trace 260/5→270/6 (L45), replace "no paid provider run" (L51) with "live provider evidence exists pre-release; release-artifact live run requires rebuild+re-run", correct/remove stale zip hash (already correct at L12 but ensure consistency with README/PHASE-12/C-340), add "artifact is pre-integration; rebuild required" banner, regenerate test/e2e counts after rebuild.
4. **`README.md`** — correct zip hash/size (L24) to the rebuilt artifact; replace "no paid provider run" l; add release-commit + evidence pointer.
5. **`tools/runbook.md`** — add live-provider first-run verification (key present → `SUBMIT_PROMPT` smoke, expected `REPLY_DONE`/`RUN_STATUS completed`; key absent → named `E_KEY_MISSING`), and env-var table from RELEASE.md.
6. **`docs/plans/PHASE-12-EVIDENCE.md`** — label L25-29 as historical pre-integration builds; fix L138-143 "Final shipped artifact" to the rebuilt artifact or mark superseded with pointer to RELEASE-EVIDENCE.
7. **`docs/claims/CLAIM-REGISTER.csv/.json`** — fix C-340 zip hash/size (or mark superseded); update S-4/S-7-related rows with the new evidence; add claims for provider wiring/live run at the release commit; keep VERIFIED/READ/UNVERIFIED labels.
8. **`audit/HIVEMIND-AUDIT-PAPER.md`** — add a dated supersession note (after the abstract) that A-F01/F02/F03 are fixed in source at `5c69b66` and in the next rebuilt artifact; paper stays valid for shipped `3988622`-era artifact.
9. **`docs/evidence/*`** — keep key-validation/live-provider-smoke as provenance; add `docs/evidence/release-smoke.md` captured on the rebuilt artifact (no key material).
10. **`plans/HIVEMIND-1.0-SEAT-ASSIGNMENTS.md` / `audit/ROADMAP*`** — no change needed (program artifacts; 101/6/8 consistent).

## 4. Final `RELEASE.md` structure (exact section skeleton)

```
# HiveMind 1.0.0 — Release Notes
1. Release identity        (version, release date, commit C + tag, build host/OS, operator)
2. Artifacts & checksums   (server exe, CLI, deployment zip: bytes + sha256 + build command; link to RELEASE-EVIDENCE)
3. What 1.0 is             (localhost server + browser UI; one DeepSeek key; no installer/Tauri/Python/Docker/cloud)
4. What is new in 1.0      (provider wiring; replay backend+UI; 16-state parity; settings/approval/file commands; evidence)
5. Verification            (table: gate | command | observed result | evidence path — Rust 4-crate, web, e2e, live HTTP, WS security, live provider, trace, claims)
6. Live-provider evidence  (key-validation + live server run; key custody, no key material)
7. Configuration           (DEEPSEEK_API_KEY, HIVEROUTER_* env table incl. provider on/off, model, thinking, effort, max tokens, bind/data-dir/workspace-root)
8. Install & first run     (runbook pointer; preflight; health/diagnose)
9. Known limitations       (non-streaming single-delta reply; no model-driven tool calls; trace partial; 14 explicit e2e skips; pixel fidelity needs_human; no Tauri)
10. Security & key custody (env-only, never logged; rotate transcript-exposed key; loopback+Origin+token)
11. Upgrade / rollback     (revert to previous tag; data dir migration; delete data dir for reset)
12. Evidence index         (RELEASE-EVIDENCE.md, keys docs, trace/claims snapshots)
13. Changelog / supersession (previous v1.0.0 tag at 3988622; re-cut to C)
```

## 5. Exact 1.0 acceptance wording (proposed canonical text)

> **HiveMind 1.0.0 acceptance gate.** Release is accepted only on a quiescent tree at a single commit `C` (tag re-cut at `C`, `git status` clean, no product edits after `C`), with every artifact rebuilt from `C` by the documented toolchain and `cargo build --release --locked` (exit 0). The release is accepted if and only if all of the following hold, each with an evidence path recorded in `docs/RELEASE-EVIDENCE.md`:
> 1. `hiverouter.exe`, `hiverouter-cli.exe`, and `dist/HiveMind-v1-win64.zip` are built from `C`; bytes + sha256 are recorded and the zip's `CHECKSUMS.txt` matches the binaries.
> 2. `cargo test -p hiverouter-contract -p hiverouter-core -p hiverouter-provider -p hiverouter-tools` is green with quoted counts (0 failed; ignored helpers named).
> 3. Web `typecheck`, unit tests, and production build are green, and the binary's embedded UI asset index matches the built `web/dist` bytes.
> 4. Live-browser e2e against the **exact shipped binary** is run with 0 failures; every skip is an explicit `test.fixme`/`test.skip` with a reason.
> 5. A live-provider proof (one key, no key material in logs/evidence) shows `SUBMIT_PROMPT → REPLY_DONE → RUN_STATUS completed` on the exact shipped binary; its absence means the provider path is not release-proven.
> 6. Live surface probes pass on the exact binary: `/health` 200, `/diagnose` 200 JSON, `/api/session` 200, `/` 200 with app root, assets 200, SPA fallback 200, traversal vectors 404, foreign Origin 403 `E_ORIGIN_REJECTED`, bad token 401.
> 7. Trace and claims snapshots are captured at `C`: trace verdicts remain `PENDING`/`UNVERIFIED` where evidence is absent (no fabricated passes); claim register rows are VERIFIED only with cited evidence and its row counts match the CSV/JSON.
> 8. `RELEASE.md` contains no statement lacking an evidence path; anything not proven is explicitly `UNVERIFIED` with the test that would settle it.
> Any failed or missing item ⇒ **release blocked** (not "PASS with notes"); the previous tag stays published and is marked superseded with its commit id.

## 6. UNVERIFIED / not claimed

- No post-integration **build** exists in this audit window: shipped binary/zip predate provider commit; test/e2e/live-provider evidence was captured on source/debug runs, not on a rebuilt release artifact.
- No independent re-run of the live provider call (no paid-API authorization for this seat; key custody respected); key-validation/live-provider-smoke are **READ** evidence only.
- Uncommitted product diff (6 files, +20/−15) content's final intent is not verified; release cannot be accepted on a dirty tree.
- `docs/RELEASE-EVIDENCE.md`, `e2e/playwright-report/index.html` presence/contents not verified (first missing at 11:20; report dir not checked).
- Streaming behavior (single-delta limitation), model-driven tool calls, trace ladder completeness, DOM G-UI gates, pixel fidelity, installer/Tauri remain out of scope/unverified per the audit.
- Key rotation after transcript exposure is the operator's action; not verifiable from the repo.
