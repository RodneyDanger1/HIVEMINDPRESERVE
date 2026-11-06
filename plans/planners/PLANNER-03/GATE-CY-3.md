# GATE-CY-3 — P3 planner cycle 3/3: recon → plan → lead (release plan + acceptance/rollback)

Planner P3 · 2026-11-06 · read-only · writes only `plans/planners/PLANNER-03/`.
Cycle object: convert the cross-seat evidence into a decision-complete plan for a sealed, reproducible 1.0.0
release; define acceptance gates and rollback; check every decision against S5/S6/S7/S8 evidence.

## 1. RECON — synthesis constraints that shape the plan

- **Source/tag/artifact discontinuity (S5 V1–V10, S8 V4/E2):** HEAD `5c69b66` (10:59:06) → 3 commits after tag
  `v1.0.0` (`3988622`); shipped binary `184D51A7…` 09:10:59, zip `BC7A2BAD…` 09:33:17, web build 10:58:47; provider
  commit 10:59:06 ⇒ shipped artifacts **cannot contain** the provider/replay/16-state fixes; paper P0s still describe
  the artifact, not the HEAD source.
- **Manifest/tooling gaps (S5 V8–V13, S5-12):** `tools/release-manifest.json` absent at 11:18; reappeared 11:41 with
  hashes (`6b6f1d0b…/13,251,584`; `d76bc64a…`) **contradicting** shipped dist (`184D51A7…/9,329,152`; `51E1ABA6…`);
  absolute host paths; no version/tag/commit/toolchain/web-hash fields. `build-release.ps1` builds only cargo — no web
  build, no dist assembly, no CHECKSUMS, no tag/version gate.
- **Key custody gap (S6 F-01):** first-party paths are clean (zeroize, redaction, booleans), but **child processes
  inherit `DEEPSEEK_API_KEY`** (`shell.rs:201`, `cli_support.rs:41`, `jail.rs:389`; no `env_clear`/`env_remove`
  anywhere); docs/runbook claim core claim "never leaves the process" is therefore too strong.
- **Documentation drift (S8 §2):** README zip `9B2ED5A7…` vs actual `BC7A2BAD…`; PHASE-12 zip `4,065,104 B`;
  FINAL-REPORT 15-state vs 16; trace 260/5 vs 270/6; "no paid run" vs evidence docs 10:57; RELEASE.md references
  missing RELEASE-EVIDENCE.md; claims register C-340 zip mismatch.
- **Verification provenance (S7):** no landed gate log is bound to HEAD; all 233/46/79 counts are pre-integration;
  finalizer self-attestation risk; current HEAD UNVERIFIED by tests.
- **Quiescence (all seats):** tree dirty throughout (6→15 files); no acceptance on a moving tree.
- **Docs contract facts (cycle 2):** model ids, 384K output cap, key env name, peak windows must be asserted by the
  release gate, not just quoted.

## 2. PLAN — decisions (final; detail in PLANNER-03.md)

- **D10 (seal target):** release is defined on one commit **C** (HEAD after committing the reviewed working-tree
  changes: provider wiring, replay arm, 16-state parity, settings, Cargo.toml/lock, web/package.json, README, runbook,
  release docs) on a **clean tree**. Existing tag `v1.0.0` must **not** be moved (immutability); cut **`v1.0.1`**
  annotated tag on C and record the supersession (`v1.0.0` → 3988622/184D51A7 pre-integration; `v1.0.1` → C/new
  hashes). If product owners insist on 1.0.0 semantics, ship `v1.0.1` and document it as the sealed 1.0 line.
- **D11 (one build command):** rewrite `tools/build-release.ps1` to a deterministic pipeline:
  clean web build (`npm ci`, `typecheck`, `vite build`, record web-dist tree hash) → `cargo build --release --locked`
  (fresh target dir; record rustc/cargo versions) → stage dist (binaries + README/RUNBOOK copied from source) →
  write `CHECKSUMS.txt` (binaries + zip) → deterministic zip (sorted entries, fixed timestamps/SOURCE_DATE_EPOCH) →
  write `tools/release-manifest.json` (version, tag, commit sha, commit date, toolchain versions, web-dist hash,
  artifact list with repo-relative path/sha256/bytes, zip sha) → **verify** (recompute every hash, fail on mismatch).
  All paths **repo-relative**; no `C:\Users\...` in manifest/scripts/docs.
- **D12 (artifact policy):** binaries/zip remain **ignored** (`dist/` in `.gitignore`) and are published as release
  assets; the **tracked** release artefacts are the small text set: `tools/release-manifest.json` (explicit
  `!` exception), `CHECKSUMS.txt` copy under tracked `release/` (or committed `release/CHECKSUMS.txt`), and
  `docs/RELEASE-EVIDENCE.md` (new authoritative OLD→NEW hash ledger). No PE binaries in git.
- **D13 (key custody + child env isolation — blocking):** before any spawn: `env_clear()` + allowlist
  (`PATH`, `SystemRoot`, `TEMP`/`TMP`, `USERPROFILE` minimal, `PATHEXT`, locale) in `shell.rs:201`,
  `cli_support.rs:41`, `jail.rs:389` (and `checks.rs:91` for node); regression test starts a child that prints its
  env and asserts `DEEPSEEK_API_KEY`/`sk-` absent; runbook/docs wording corrected from "never leaves the process" to
  "never leaves the process except by explicit child-process isolation (env_clear + allowlist)". Release gate scans
  tracked files for key-shaped strings and stale claim strings.
- **D14 (supersede old artifacts):** do not delete history; publish `docs/RELEASE-EVIDENCE.md` mapping old vs new:
  binary 184D51A7 → new sha; zip BC7A2BAD → new sha; PRE-INTEGRATION notice; update README:24, PHASE-12-EVIDENCE
  zip row, FINAL-REPORT state/trace counts, claims register C-340, and remove/repair stale `tools/bin/hiverouter.exe`
  duplicate (same hash as dist) — all with OLD→NEW text.
- **D15 (acceptance + rollback):** acceptance = 12 gates (quiescence/tag, manifest schema+hash, checksums, no
  absolute paths, key-isolation test, secret scan, docs consistency, packaged smoke: `/health` 200 + UI asset hash +
  no-key `E_KEY_MISSING` by name, web-dist == embedded, git status clean after build, optional paid smoke gated by
  operator authorization). Any failure = NO SHIP (no partial acceptance). Rollback = revert release commit; never
  move/delete tags; publish retraction + patch tag; old artifacts remain hash-pinned in RELEASE-EVIDENCE.
- **D16 (verification evidence):** the release run must emit a machine-readable `release-evidence.json` with command
  lines, exit codes, toolchain versions, all hashes — the gap S7 identified (logs without HEAD/exit/JUnit).

## 3. LEAD — cycle verdict

- Decision completeness: seal target, build command, artifact policy, key isolation, supersession, acceptance and
  rollback are all specified with commands/evidence — **PASS**.
- Cross-seat incorporation: S5 (release/artifacts), S6 (key/child-env), S8 (docs/staleness) explicitly drive D11–D16;
  S1/S2/S3/S4/S7 evidence used for scope and UNVERIFIED — **PASS**.
- No product/git/config writes proposed for this seat; all edits are owned by Lead/owners; no key access; no paid
  call — **PASS**. **Cycle 3: PASS (not blocked).** Deliverable: `PLANNER-03.md`.
