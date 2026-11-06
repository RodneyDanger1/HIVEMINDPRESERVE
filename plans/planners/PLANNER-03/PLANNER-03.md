# PLANNER-03 — Sealed, Reproducible 1.0.0 Release Plan (Release / KeyOps / Reproducibility)

**Planner:** P3 · **Program:** HiveMind 1.0 · **Mode:** read-only planning (writes only `plans/planners/PLANNER-03/`)
**Inputs (all read in full, in order):** 8 seat reports (SEAT-01..08 × REPORT/READLOG/EVIDENCE/GATE-CY-1..3),
`audit/HIVEMIND-AUDIT-PAPER.md` (244 lines), 4 official DeepSeek docs (2 tails fetch-truncated → UNVERIFIED).
**Baseline observed:** HEAD `5c69b66e363e9f1d0801ea5c2e66a1fc12c8943c` (`v1.0.0-3-g5c69b66`), tag `v1.0.0` =
`3988622d…`; tree **dirty (quiescence FAIL in S5/S6/S7/S8)**; shipped binary `184D51A7…` (09:10:59) and zip
`BC7A2BAD…` (09:33:17) predate the provider commit (10:59:06) and final web build (10:58:47).

## 0. Outcome (definition of done)

One commit **C** on a clean tree, one annotated tag **v1.0.1**, and one deterministic command
`pwsh -NoProfile -File tools/build-release.ps1` that rebuilds **web + binaries + zip + CHECKSUMS + release-manifest**
from C and emits a machine-checkable evidence record. A consumer can verify the shipped artifact against the
committed manifest alone (repo-relative paths, commit/toolchain hashes), the key-custody claim is true including
child processes, and every stale/superseded release claim has an OLD→NEW correction. No acceptance on a dirty tree.

## 1. Current behavior (evidence; no assumptions)

| Fact | Evidence |
|---|---|
| Source fixed the audit P0s but the artifact did not | HEAD `5c69b66` message = provider wiring/live smoke/replay arm/16-state parity/settings; paper P0 A-F01/A-F02/A-F03 (paper:11) still true of `184D51A7…`/`BC7A2BAD…` (S8 §2; S5 V2,V10) |
| Shipped artifacts are stale | binary mtime 09:10:59, zip 09:33:17 < provider commit 10:59:06 / web build 10:58:47 (S8 E2; S5 V10) |
| Release manifest is absent or contradicts dist | missing at 11:18 (S5 V8); reappeared untracked with `6b6f1d0b…/13,251,584` (server) and `d76bc64a…` (cli) vs shipped `184D51A7…/9,329,152` / `51E1ABA6…` (S5-12) |
| Build script cannot produce a sealed release | `build-release.ps1:21` cargo-only; no web build, no dist copy, no CHECKSUMS, no tag/version check; manifest written with absolute `$artifact` paths (`:36-39`) |
| Key custody has a process-boundary hole | child spawns without `env_clear`/`env_remove`: `shell.rs:201`, `cli_support.rs:41`, `jail.rs:389` (S6 F-01; 0 matches repo-wide) |
| Release docs contradict each other and the artifact | README zip `9B2ED5A7…` vs `BC7A2BAD…`; PHASE-12 zip `4,065,104 B`; FINAL-REPORT 15-state; trace 260/5 vs 270/6; "no paid run" vs evidence 10:57; C-340 mismatch (S8 §2) |
| No gate log binds to current HEAD | 233/46/79 counts pre-integration; logs lack HEAD/exit/JUnit (S7 §0,E4) |
| Quiescence | tree dirty throughout audits (S5 S5-13; S6 F-04; S7 E1; S8 E1) — acceptance must be post-seal |

## 2. Decisions (with rejected alternatives)

- **D1 Seal target.** Release = one commit C on a clean tree + annotated tag **`v1.0.1`**. *Rejected:* moving
  `v1.0.0` (immutability violated); *rejected:* shipping 1.0.0 semantics on a post-tag tree with no new tag
  (unverifiable provenance). C = current HEAD + reviewed working-tree changes committed (provider wiring, replay arm,
  16-state parity, settings, Cargo.toml/lock, `web/package.json`, README, runbook, release docs).
- **D2 One build command, deterministic.** Rewrite `tools/build-release.ps1` to run, in order: (1) preflight asserts
  (`git status --porcelain` empty; HEAD == tag target; `cargo metadata --no-deps` versions == version field);
  (2) web: `npm ci --no-audit --no-fund` → `npm run typecheck` → `npm run build`; compute `web/dist` tree hash
  (sorted path+sha256 list); (3) `cargo build --release --locked` in a fresh `CARGO_TARGET_DIR`; capture
  `rustc -Vv`/`cargo -V`; (4) stage `dist/HiveMind-<ver>-win64/` with binaries + `README.md`/`RUNBOOK.md` copied from
  source; (5) write `CHECKSUMS.txt` (binaries + zip); (6) deterministic zip (sorted entries, fixed timestamp /
  `SOURCE_DATE_EPOCH`); (7) write `tools/release-manifest.json`; (8) recompute and compare every hash → fail loudly.
  *Rejected:* PowerShell multi-step manual packaging (current failure mode); *rejected:* relying on cargo only.
- **D3 Manifest schema (fixed keys).** `schema`, `version`, `tag`, `commit`, `commit_date`, `dirty:false`,
  `toolchain{rustc,cargo,node,npm}`, `web_dist_sha256`, `provider{models:["deepseek-flash","deepseek-v4-pro"],
  max_output_tokens:393216, key_env:"DEEPSEEK_API_KEY"}`, `artifacts[{path(repo-relative),sha256,bytes}]`,
  `zip{path,sha256,bytes}`, `checksums_sha256`, `generated_by`. *Rejected:* free-form/absent manifest (current).
- **D4 No absolute paths.** Script derives `$repo = Split-Path -Parent $PSScriptRoot`; manifest paths
  repo-relative; shipped README drops `C:\Users\Riley\Desktop\hivemind-blueprint` (S5-08/S8); CI grep gate:
  `Select-String -Pattern 'C:\\\\Users\\\\'` over manifest/scripts/shipped docs must be empty.
- **D5 Tracked/ignored policy.** Tracked: source, `tools/*.ps1`, `tools/release-manifest.json` (`.gitignore`
  exception `!tools/release-manifest.json`), `release/CHECKSUMS.txt`, `docs/RELEASE-EVIDENCE.md`, docs. Ignored:
  `target/`, `node_modules/`, `dist/` (binaries+zip stay out of git), `.env*`, `__pycache__`, `*.pyc`, logs,
  `*.figcraft`, design `ASSETS/*.svg|png`, `.live-smoke-data/`. Remove tracked generated junk (S8/A-F09 class:
  `tools/bin/hiverouter.exe` duplicate). *Rejected:* tracking PE binaries (repo bloat + provenance drift).
- **D6 Key custody + child-process isolation (BLOCKING for any "key never leaves the process" claim).** In
  `tools/src/shell.rs:201`, `tools/src/cli_support.rs:41`, `tools/src/jail.rs:389` (and `core/src/checks.rs:91`
  node spawn): `Command::new(...).env_clear()` then set only allowlisted env (`PATH`, `SystemRoot`, `TEMP`, `TMP`,
  `USERPROFILE`, `PATHEXT`, locale). Add regression test: child prints env keys → assert no `DEEPSEEK_API_KEY`,
  no `sk-` value, and stdout passes through the existing audit redaction (`audit.rs:31,247`). Correct runbook/docs
  wording to the scoped truth. CI secret scan over tracked files (`sk-[A-Za-z0-9]{16,}`, `DEEPSEEK_API_KEY\s*=\s*\S`).
  *Rejected:* documenting the hole only (release-blocking claim); *rejected:* removing the key from server env
  (breaks provider calls).
- **D7 Superseded artifacts + docs corrections (OLD→NEW).** Create tracked `docs/RELEASE-EVIDENCE.md`:
  `v1.0.0` tag 3988622 / binary 184D51A7 / zip BC7A2BAD = **PRE-INTEGRATION, SUPERSEDED**; `v1.0.1` tag C /
  new hashes = current. Apply corrections with OLD→NEW rows: README:24 zip hash; `docs/plans/PHASE-12-EVIDENCE.md`
  zip row; `docs/reports/FINAL-REPORT.md` state count (15→16) and trace counts (260/5→270/6 per live
  `docs/trace/controls.json`); `docs/RELEASE.md` "verified in the build" → cite new manifest + evidence file that
  must exist; `docs/claims/CLAIM-REGISTER.csv` C-340; delete/mark duplicate `tools/bin/hiverouter.exe`.
  *Rejected:* silent overwrite (provenance loss); *rejected:* leaving doc-vs-doc contradictions.
- **D8 Verification binding.** The release run writes `release-evidence.json` + console log with command lines,
  exit codes, HEAD, tag, toolchain and all hashes (addresses S7: no HEAD-bound evidence). Re-run the full gate on
  the sealed checkout; previous logs are historical only.
- **D9 Acceptance = no-ship on any failure.** Gates G1–G12 below; partial acceptance is not allowed.

## 3. File-by-file change plan

| Path | Change |
|---|---|
| `tools/build-release.ps1` | full rewrite per D2; repo-relative; fail-fast on every step; no absolute paths; writes manifest+CHECKSUMS+zip; deterministic zip; emits `release-evidence.json` |
| `tools/release-manifest.json` | generated by script; committed (`.gitignore` exception); schema per D3 |
| `.gitignore` | add `!tools/release-manifest.json`; keep `dist/`, `.env*`, `__pycache__`, `*.pyc`, `*.figcraft`, `ASSETS/*.svg|png` ignored; no binary exceptions |
| `backend/crates/hiverouter-tools/src/shell.rs` | line ~201 spawn: `env_clear()` + allowlisted env; comment names the key boundary |
| `backend/crates/hiverouter-tools/src/cli_support.rs` | line ~41: same env policy |
| `backend/crates/hiverouter-tools/src/jail.rs` | line ~389: same env policy |
| `backend/crates/hiverouter-core/src/checks.rs` | line ~91 node spawn: same env policy |
| `backend/crates/hiverouter-tools/tests/` (new) | `child_env_isolation.rs`: child prints env; assert no `DEEPSEEK_API_KEY`; assert normal tools still work (`--version`) |
| `README.md` | v1.0.1 status; new manifest/zip hashes; no absolute blueprint path; link `docs/RELEASE-EVIDENCE.md` |
| `tools/runbook.md` (+ shipped copy) | build steps match script; key-custody wording scoped to env_clear allowlist; add verify command (`Get-FileHash` vs manifest) |
| `docs/RELEASE-EVIDENCE.md` (new) | authoritative pre/post-integration artifact table (tag, commit, binary/zip/CHECKSUMS sha256, toolchain, web-dist hash) |
| `docs/RELEASE.md` | rewrite as release notes for `v1.0.1` citing manifest/evidence; delete unverifiable "verified in the build" wording |
| `docs/plans/PHASE-12-EVIDENCE.md`, `docs/reports/FINAL-REPORT.md`, `docs/claims/CLAIM-REGISTER.csv` | OLD→NEW corrections (zip hash/size, 16 states, trace counts, C-340) with date/reason |
| `dist/**`, `tools/bin/hiverouter.exe` | rebuilt; dist stays ignored; duplicates removed; publish via release assets only |
| `backend/Cargo.toml`, crate `Cargo.toml`, `web/package.json` | version → `1.0.1` in C; lockfile refreshed with `--locked` verification |
| `release/CHECKSUMS.txt` (new, tracked) | copy of generated checksums so hashes are committed while binaries are not |

## 4. Failure modes handled

- Missing web build / stale embed → gate G8 (web-dist hash == embedded hash; script rebuilds web before cargo).
- Missing/partial manifest → schema validation gate G2; any hash mismatch = fail (no warning-only).
- Key leakage to child → regression test G5; if child needs an env var, allowlist explicitly (documented).
- Absolute path leakage → grep gate G4.
- Non-deterministic zip → fixed timestamps/sorted entries; two consecutive runs of the same commit must produce
  identical zip sha (else gate G3 fails and records which input drifted).
- Dirty tree during build → G1 refuses; `dirty` field never true in manifest.
- Doc drift → G7 doc consistency scan; old values only allowed inside `RELEASE-EVIDENCE.md` OLD rows.
- Missing tag → annotated tag created only after build passes; tag must point at C.
- Provider unavailable during smoke → packaged smoke goes key-less (`E_KEY_MISSING` by name); paid smoke is gated and
  optional (never CI default).

## 5. Tests / commands (acceptance gates)

```
G1  quiescence+tag : git status --porcelain (empty) ; git rev-parse HEAD ; git describe --exact-match --tags
G2  manifest       : schema valid; artifacts/zip/checksums hashes recomputed; no absolute paths
G3  determinism    : run build twice on same commit → binary/zip/check-sums hashes identical (or documented diff set)
G4  cert           : node/cargo/rustc/npm versions recorded; npm ci; cargo build --release --locked
G5  key isolation  : cargo test -p hiverouter-tools child_env_isolation ; secret scan over tracked files
G6  packaged smoke : dist exe on 127.0.0.1 high port with no key → /health 200, UI index served, SUBMIT_PROMPT →
                      E_KEY_MISSING by name (no fake success) ; with operator key (optional, gated) → HIVE-LIVE-OK
G7  docs           : README/RELEASE/PHASE-12/FINAL-REPORT/claims hashes == manifest ; only RELEASE-EVIDENCE has OLD rows
G8  embed freshness: web-dist tree hash recorded == served asset hash ; binary rebuilt after web build (mtime/order)
G9  checksums      : CHECKSUMS.txt == recomputed binaries + zip ; committed release/CHECKSUMS.txt == generated
G10 zip integrity  : members = {CHECKSUMS.txt, hiverouter.exe, hiverouter-cli.exe, README.md, RUNBOOK.md}; zip sha == manifest
G11 key-claim text : retire unscoped "never leaves the process" strings; runbook states env_clear allowlist
G12 release evidence: release-evidence.json present, HEAD/tag/toolchain/hashes/exit codes; no stale count quoted
```

## 6. Rollback

- **Plan/seat rollback:** delete `plans/planners/PLANNER-03/` (read-only planner; no product state touched).
- **Release rollback:** revert the release commit (script/docs/manifest) — old tag stays immutable; publish a
  retraction in `docs/RELEASE-EVIDENCE.md` and cut `v1.0.2` from the fixed tree; never delete/move tags; old
  artifacts remain hash-pinned and marked SUPERSEDED. Build script is idempotent: re-running on the same C reproduces
  the same manifest (gate G3), so a failed publish can be retried without new provenance.
- **Key-isolation rollback:** if an allowlist breaks a tool, add the exact variable to the allowlist (documented
  justification) — never revert to `env_clear`-less spawning.

## 7. Out of scope / owned elsewhere

- Contract/UI defects from other seats (S3 D1 failed-not-terminal, D2 replay tests, D5 HTTP timeouts; S2 UI refusal
  table 45/65; S1 non-streaming leaf) — release plan records them, owners fix separately.
- Paid provider run authorization, key rotation (operator action), Win32 job-object proof, byte-identical Rust
  reproducibility across hosts (toolchain-dependent; gate records same-host determinism only).

## 8. UNVERIFIED (must remain so until proven)

- Any current-HEAD test/e2e/paid counts (no gate bound to `5c69b66`; S7).
- That `dist/` binaries were built from HEAD or from the dirty tree (no build metadata; S5).
- Byte-identical binary reproducibility across machines; only same-commit/same-toolchain determinism is required.
- Live provider behaviour under real 402/429/streaming on the sealed artifact (needs operator key; S6/S7).
- Child-env isolation effect until the new regression test runs (planned, not executed).
- Truncated official-doc tails (`thinking_mode`, `create-chat-completion`).

## 9. Post-plan state delta (quiescence note, written 11:40 CST)

While this plan was being written the tree moved again: a new commit **`6a2b1c6`** — "release: HiveMind 1.0.0
provider wiring, key isolation, replay, parity, release docs" — landed on top of `5c69b66` (`git describe` now
`v1.0.0-4-g6a2b1c6`; `git status` shows only untracked `plans/`, `tools/release-manifest.json`, `.artifact-smoke/`).
Consequences: (a) the working-tree dirt described in §1 no longer exists, so the seal step D1 is partly done by that
commit; (b) "key isolation" in its message may address S6 F-01 — this must be **re-verified** against
`shell.rs/cli_support.rs/jail.rs` (gate G5) rather than assumed; (c) all §1 facts remain true for the audit window
(HEAD `5c69b66` / shipped `184D51A7…`/`BC7A2BAD…`) but acceptance gates G1–G12 must be re-run on `6a2b1c6` (or its
successor) and a new tag cut; `v1.0.0` still points at `3988622` and remains immutable. No plan decision is withdrawn;
the plan's gate structure is the deliverable, and it is now one commit closer to executable.

