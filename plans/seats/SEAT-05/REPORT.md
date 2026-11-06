# SEAT-05 (Release / KeyOps) — REPORT.md

HiveMind 1.0 · seat S5 · read-only audit · repo `C:\Users\Riley\Desktop\hiverouter-v1`
Evidence timestamp: 2026-11-06 11:00–11:40 CST · HEAD `5c69b66e363e9f1d0801ea5c2e66a1fc12c8943c` · tree **DIRTY (8 files)**
Writes: `plans/seats/SEAT-05/**` only (no product/git/config edits, no key access, no paid calls, no builds).

## 1. Mandated inputs actually read (READLOG.md is the receipt)

1. `audit/HIVEMIND-AUDIT-PAPER.md` — **FULL** (24,141 B read as 25,141 B file / 244 lines).
2. Four official DeepSeek docs — fetched HTTP 200 and read: `/` (full), `/quick_start/pricing/` (full),
   `/guides/thinking_mode/` (partial, truncated at tool-call sample), `/api/create-chat-completion/`
   (partial, truncated at streaming response schema). Truncated tails are UNVERIFIED, not invented.
3. Assigned audit files — each read fully, one at a time: `audit/diagrams/06-migration-target.mmd` (7 lines),
   `audit/tools/build_inventory.py` (209 lines), `audit/tools/enrich_manifest.py` (68 lines).
4. Assigned product files — read one at a time (full text where textual; binary metadata only):
   `backend/Cargo.toml` (54 lines), 4 crate `Cargo.toml` (20/30/51/41 lines), `backend/Cargo.lock`
   (4,047 lines, two passes), `.gitignore` (15), `tools/runbook.md` (82), `tools/build-release.ps1` (51),
   `web/package.json` (37), `dist/HiveMind-v1-win64/CHECKSUMS.txt` (8), `dist/HiveMind-v1-win64/README.md`
   (24), `dist/HiveMind-v1-win64/RUNBOOK.md` (hash-identical to `tools/runbook.md`, sha `e7fba10f…`,
   not re-read separately), `docs/evidence/key-validation.md` (15), `docs/evidence/live-provider-smoke.md`
   (13). `tools/release-manifest.json` **does not exist** (Test-Path false; `git ls-files` empty).
   `dist/HiveMind-v1-win64.zip` and two `.exe` files: binary class — metadata/hash only, not decoded.

## 2. VERIFIED FACTS (command or path:line)

| # | Fact | Evidence |
|---|---|---|
| V1 | HEAD = `5c69b66e…`, `git describe` = `v1.0.0-3-g5c69b66`; tag `v1.0.0` points at `3988622d…` — **3 commits behind HEAD** | `git rev-parse HEAD`, `git rev-list -n1 v1.0.0`, `git describe --tags` |
| V2 | HEAD commit message: `feat: wire DeepSeek provider into core; live smoke, replay arm, 16-state parity, settings` — i.e. the paper's P0 fixes A-F01/A-F02/A-F03 landed **after** tag v1.0.0 | `git log --oneline -4` |
| V3 | Working tree **dirty, 8 files**: `README.md`, `backend/Cargo.lock`, 4× `backend/crates/hiverouter-*/Cargo.toml`, `tools/runbook.md`, `web/package.json`; diffstat `+46/−26` | `git status --porcelain`, `git diff --stat` |
| V4 | Release outputs exist under ignored `dist/` (6 files, 16,542,476 B): zip 6,571,928 B; `hiverouter.exe` 9,329,152 B; `hiverouter-cli.exe` 5,364,224 B; README 1,276 B; RUNBOOK 4,396 B; CHECKSUMS 712 B | dir listing + sizes |
| V5 | `dist/` is **NOT tracked** and is ignored: `git ls-files dist` → empty; `.gitignore:3 dist/` | commands above |
| V6 | Package checksums verify: `CHECKSUMS.txt` server `184D51A7…` == actual `hiverouter.exe`; cli `51E1ABA6…` == actual `hiverouter-cli.exe` (both recomputed) | `Get-FileHash` |
| V7 | Zip member set = {CHECKSUMS.txt, hiverouter.exe, hiverouter-cli.exe, README.md, RUNBOOK.md} — 5 members, no nested directory | `.NET ZipFile.OpenRead` entry listing (read-only, no extraction) |
| V8 | `tools/release-manifest.json` **MISSING** although `build-release.ps1:45` writes it and `runbook.md:40` promises it ("(prints artifact paths + SHA-256; writes tools/release-manifest.json)") | Test-Path false; `git ls-files` empty; file reads |
| V9 | Build script gap: `build-release.ps1` runs only `cargo build --release --locked` (line 21) — it does **not** run the web build, does not copy artifacts to `dist/`, does not write `CHECKSUMS.txt`, does not verify the tag or version | full read of 51-line script |
| V10 | Shipped binaries are **older than the current web build**: `hiverouter.exe` mtime 09:10:59, zip 09:33:17, while `web/dist/index.html`+assets mtime **10:58:47** — `rust-embed` embeds `web/dist` at compile time, so the packaged server cannot contain the 10:58 UI build | `Get-Item LastWriteTime`; `Cargo.toml:37 rust-embed` |
| V11 | All workspace crates version `1.0.0`; `web/package.json` version `1.0.0`; `web/package-lock.json` exists; `node>=22.12.0` engine matches runbook requirement | Cargo.toml lines 3, package.json lines 4/8, Test-Path |
| V12 | `hiverouter-core/Cargo.toml:21` now depends on `hiverouter-provider`; `Cargo.lock` hiverouter-core deps include `hiverouter-provider` (line 1000) → provider is wired in the current tree (paper A-F01 fixed post-tag) | reads |
| V13 | Dependency pinning is strict: workspace deps use exact `=x.y.z` in `backend/Cargo.toml:17-43`; release profile `lto=thin, codegen-units=1, strip=true, panic=abort`; lockfile `version = 4` | reads |
| V14 | Duplicate-major debt remains in `Cargo.lock`: `thiserror 1.0.69` + `2.0.21` (lines 2950-2988), `tower-http 0.6.11` + `0.7.1` (3165-3208), multiple `getrandom`/`rand`/`windows`/`windows-sys` majors — matches paper A-F07; `cargo tree -d` NOT run (no build allowed) → UNVERIFIED verdict | lock lines |
| V15 | Key custody structurally sound in the release surface: `.gitignore:10-11` ignores `.env` and `.env.*`; no key values in the scanned release docs/scripts — only the placeholder `$env:DEEPSEEK_API_KEY = "<your-key>"` in both runbooks (`runbook.md:45`, `dist/…/RUNBOOK.md:45`); `key-validation.md` states the key never leaves process env and cites a live call | `Select-String` scan + reads |
| V16 | Evidence honesty: `live-provider-smoke.md:12-13` explicitly records open gaps ("adapter returns the leaf reply as one delta after the non-streaming completion; true SSE delta publishing and the replay UI are still being completed") — a truthful limitation, not fake success | read |
| V17 | `dist/…/README.md:3` hardcodes a host path `C:\Users\Riley\Desktop\hivemind-blueprint` inside the **shipped** README (machine-specific, non-portable provenance) | read |
| V18 | Paper §5.3 line 149 claims "`dist/` is release output (6 files, 20 MB) and is tracked" — **REFUTED** by V5; the paper's own inventory correctly classifies `dist/` as an ignored generated class (`build_inventory.py:200`). This is a paper documentation defect, not a repo defect | V5 + build_inventory.py |
| V19 | `audit/tools/build_inventory.py` builds the manifest from `git ls-files` only (line 140) and hardcodes ignored-class counts (lines 194-202) → no release artefacts are hashed by the audit inventory; `enrich_manifest.py` has a duplicated condition at line 64 (harmless but sloppy) | reads |
| V20 | Audit diagram `06-migration-target.mmd` has **no release/packaging/version wave** — W1 mentions only "generated-artifact tracking cleanup"; no tag sealing, no release manifest, no checksum gate | read (7 lines) |

## 3. Defects / risks (release-focused register)

| ID | Sev | Defect | Evidence | Fix (owner) |
|---|---|---|---|---|
| S5-01 | **HIGH** | Tag `v1.0.0` predates the P0 provider/replay/parity commits; HEAD is 3 commits past the tag and the tree is dirty (8 files incl. lockfile + crate manifests + web package.json) ⇒ there is **no sealed, reproducible v1.0.0 source state** for the shipped binaries | V1-V3 | Lead: cut a new tag only from a clean tree after the remaining dirty edits land; document `git describe --dirty` rule |
| S5-02 | **HIGH** | `tools/release-manifest.json` missing although promised by runbook and written by the release script ⇒ shipped binaries have **no sealed manifest**; dist assembly (copy + CHECKSUMS) is manual/undocumented | V8 | Add a `release` step that produces manifest + CHECKSUMS into `dist/` and tracks the manifest (`tools/release-manifest.json` is small, should be committed); add version/tag fields |
| S5-03 | **HIGH** | Packaged server predates current `web/dist` (09:10 vs 10:58) and the release script never builds the web ⇒ the shipped UI can be stale even when the checkout is correct; no embed-freshness gate | V9-V10 | Release script must run `npm ci && npm run build` (or assert `web/dist` mtime/hash) before `cargo build --release`; record web dist hash in the manifest |
| S5-04 | MED | CHECKSUMS checks the two binaries but not the zip itself; the zip hash (6,571,928 B) is not anchored in any tracked file | V6-V7 | Manifest should include zip sha256 + build metadata (rustc/cargo versions, git commit+tag, web dist hash) |
| S5-05 | MED | Version/consistency gates absent: no step asserts `cargo metadata` versions == tag, no binary `--version` check, no tag-vs-HEAD check | V9, V11 | Add `cargo metadata --no-deps`, `hiverouter.exe version`, `hiverouter-cli.exe version` checks + tag match to the release script |
| S5-06 | MED | Paper §5.3 claims `dist/` tracked — refuted; documentation drift in the audit artefact itself | V18 | Paper correction: `dist/` is ignored/untracked (6 files, 16.5 MB); cite `git ls-files dist` |
| S5-07 | LOW | Line-ending nondeterminism: git warns LF→CRLF pending for the 8 dirty files; no `.gitattributes` seen in slice | git stderr | Add `.gitattributes` (`* text=auto eol=lf`) for reproducible bytes |
| S5-08 | LOW | Shipped README hardcodes host path `C:\Users\Riley\Desktop\hivemind-blueprint` | V17 | Use a relative/omitted blueprint reference in shipped docs |
| S5-09 | LOW | `build-release.ps1` captures no toolchain identity and no SOURCE_DATE_EPOCH; `Set-Content -Encoding utf8` BOM behaviour is host-dependent; no exit-code gate after `Set-Content` | V9 | Record rustc/cargo versions; use deterministic zip (fixed timestamps); gate on `$LASTEXITCODE`/`$?` |
| S5-10 | INFO | Key custody acceptable: env-only key, `.env*` ignored, no key material in scanned release surface; evidence docs are self-reported (no independent paid call allowed) | V15-V16 | Keep; add a CI check that scans tracked docs/scripts for key-shaped strings |
| S5-11 | INFO | Provider now wired in current tree (A-F01 fixed post-tag) but tag/release evidence still reflects pre-fix release — release notes must state this | V2, V12 | Release notes/CHANGELOG: v1.0.0 shipped shell; provider wiring is post-tag |

## 4. Key-custody assessment (no key access performed)

- **Custody model (READ)**: one DeepSeek key in the server process environment (`DEEPSEEK_API_KEY`), reported by
  `hiverouter-cli preflight` only as `present: true/false` (`tools/runbook.md:18,27-28`), never printed/logged
  (`docs/evidence/key-validation.md:4`), never written to repo/logs/UI/DB.
- **Repo hygiene (VERIFIED)**: `.gitignore:10-11` excludes `.env`/`.env.*`; scan of release docs/scripts found
  only the placeholder `<your-key>`; no key value in `CHECKSUMS.txt`, README, RUNBOOK, package.json.
- **Evidence quality (READ/UNVERIFIED)**: `key-validation.md` reports HTTP 200 with response id/usage;
  `live-provider-smoke.md` reports a completed live run with event order. Both are self-reported receipts;
  independently re-running them requires a paid provider call and the key → **UNVERIFIED by this seat** (by
  rule). No fake-success pattern found; limitations are stated honestly (V16).
- **Operational risk**: `dist/` is ignored yet contains key-independent binaries; no secret material ships.
  Highest key-related risk is operational (release manifest missing), not leakage.

## 5. File-by-file plan (current → action → acceptance)

| File | Current state (verified) | Planned action | Acceptance |
|---|---|---|---|
| `backend/Cargo.toml` | workspace 4 crates, exact pins, release profile hardened | no change to deps; add `[workspace.metadata.release]` tag/version pin optional | `cargo build --release --locked` on clean tree; versions == tag |
| `backend/crates/hiverouter-contract/Cargo.toml` | v1.0.0; clean deps (+schemars uuid1) | keep | `cargo metadata` version 1.0.0 |
| `backend/crates/hiverouter-provider/Cargo.toml` | v1.0.0; contract+reqwest+rust_decimal+zeroize | keep | same |
| `backend/crates/hiverouter-tools/Cargo.toml` | v1.0.0; provider+axum+win32job; CLI bin | keep | CLI `--version` prints 1.0.0 |
| `backend/crates/hiverouter-core/Cargo.toml` | v1.0.0; **now depends on provider**; rust-embed | keep; release gate asserts embed | server `/` serves a UI whose `web/dist` hash is in manifest |
| `backend/Cargo.lock` | lock v4, 4,047 lines, 405-ish pkgs, duplicate majors | keep; add `cargo tree -d` gate (allow-list documented) | duplicate-major report attached to release |
| `.gitignore` | excludes dist/, target/, .env*, __pycache__, *.pyc | keep; add `!tools/release-manifest.json` exception | manifest committable while dist stays ignored |
| `tools/runbook.md` | truthful packaging statement; guarantees release-manifest.json | fix promise or implement it (S5-02); add tag/version + web-build steps | runbook matches actual release script behaviour |
| `tools/build-release.ps1` | cargo build only; writes manifest; prints hashes | add web build, tag/version assertions, artifact copy→dist, CHECKSUMS+zip, manifest with git/toolchain/web hashes, deterministic zip | one command produces sealed dist + manifest; re-run yields identical hashes except documented timestamp |
| `tools/release-manifest.json` | **MISSING** | create (script output, committed) | file exists, schema includes version/tag/commit/toolchain/web-dist hash/artefact hashes |
| `dist/**` (6 files) | present, ignored; checksums for 2 binaries verify; zip 5 members | keep ignored; regenerate only via release script; publish manifest+checksums as tracked artefacts | `sha256` of dist files == manifest; zip hash in manifest |
| `web/package.json` | React/Vite pinned exact, node>=22.12, version 1.0.0 | keep; release script runs `npm ci && npm run build` | embedded UI == `web/dist` hash in manifest |
| `docs/evidence/key-validation.md` | live-call receipt, no key material | keep; add commit hash + binary hash used | receipt reproducible from sealed manifest |
| `docs/evidence/live-provider-smoke.md` | live-server smoke receipt; open gaps stated | keep; append closure when SSE/replay land | gaps closed or explicitly gated |

## 6. Tests / commands (release gates to add; all read-only today)

```
# today (executed read-only by this seat)
git rev-parse HEAD ; git rev-list -n1 v1.0.0 ; git describe --tags ; git status --porcelain ; git diff --stat
git ls-files dist ; git ls-files tools/release-manifest.json
Get-FileHash dist/HiveMind-v1-win64/*.exe -Algorithm SHA256      # == CHECKSUMS.txt (verified)
[System.IO.Compression.ZipFile]::OpenRead("dist/HiveMind-v1-win64.zip").Entries   # 5 members (verified)
Select-String -Path <release docs/scripts> -Pattern 'sk-[A-Za-z0-9]|DEEPSEEK_API_KEY\s*='  # only placeholder
# proposed gates (NOT run here: would build/network/paid)
npm.cmd ci && npm.cmd run build            # in web/
cargo metadata --no-deps --format-version 1   # versions == tag
cargo build --release --locked ; cargo tree -d
hiverouter.exe / hiverouter-cli.exe version   # binaries report 1.0.0
sha256sum dist/** ; diff against tools/release-manifest.json
```

## 7. Rollback (this seat + release process)

- **Seat rollback**: delete `plans/seats/SEAT-05/**` — no product/git/config/key/tool state was modified, so
  rollback is a folder delete (verified: only writes are these files).
- **Release-process rollback** (if a proposed gate proves wrong): the release script/manifest changes are
  additive and statically revertible (`git revert` of the `tools/build-release.ps1` +
  `tools/release-manifest.json` commit); ignored `dist/` can be regenerated from the script; the existing
  `CHECKSUMS.txt` remains valid for the current binaries.
- **Tag rollback**: do not move `v1.0.0` (immutability); cut `v1.0.1`/`v1.1.0` from a clean tree instead and
  record the supersession in CHANGE-LEDGER.

## 8. UNVERIFIED (honest limits)

1. That the shipped `dist/` binaries were built from HEAD `5c69b66` or from the dirty tree — no build metadata in
   the binaries/manifest; proven only by a rebuild-and-compare (forbidden here: no rebuilds).
2. Binary version strings (`hiverouter.exe version`) — not executed (would run a product binary; no need/no
   execution allowed); claimed 1.0.0 from manifests only.
3. `cargo tree -d` duplicate-major verdict — not run (no build/lock resolution executed); lock shows duplicate
   majors as READ evidence only.
4. Live/provider evidence (`key-validation.md` HTTP 200, `live-provider-smoke.md` completed run) — self-reported;
   independent proof needs a paid provider call and key (prohibited by this seat).
5. Truncated tails of `thinking_mode` and `create-chat-completion` docs (fetch-tool truncation) — the
   contract-relevant sections were obtained; tails recorded as UNVERIFIED.
6. `web/dist` asset contents/hashes (built 10:58) — files exist; not hashed into any sealed manifest.
7. Exact per-hunk content of the 8 dirty files — diffstat captured; individual hunks not line-audited (timebox).

## 9. Quiescence addendum — tree moved during the audit (2026-11-06 11:41 CST re-check)

The audit tree was **not quiescent**: the baseline snapshot (11:00–11:22) recorded HEAD `5c69b66` with
**8 dirty files**; the 11:41 re-check shows **15 modified files** (adds `hiverouter-core/src/execution.rs`,
`provider_executor.rs`, `hiverouter-tools/src/cli_support.rs`, `shell.rs`, `web/src/flow/Canvas.tsx`,
`graph.ts`, `web/src/state/agentState.ts`) and new untracked paths `docs/RELEASE.md`,
`docs/RELEASE-EVIDENCE.md`, `tools/release-manifest.json`. Per the auditor quiescence discipline, all findings
above are timestamped to the earlier snapshot; the release verdict must be re-run on the sealed tree. Two new
timestamped facts:

- **S5-12 (HIGH, new at 11:41): `tools/release-manifest.json` now exists but contradicts the shipped dist.**
  It records `target\release\hiverouter.exe` sha `6b6f1d0b…` / 13,251,584 B and `hiverouter-cli.exe` sha
  `d76bc64a…` / 5,364,224 B, while the shipped `dist/` binaries are `184D51A7…` / 9,329,152 B and
  `51E1ABA6…` / 5,364,224 B (CHECKSUMS.txt, recomputed V6). Same-size CLI with a different hash ⇒ rebuilt
  content; server differs in size and hash ⇒ **manifest and dist describe different builds**, and only
  `dist/CHECKSUMS.txt` describes the shipped pair. The manifest also uses absolute host paths
  (`C:\Users\Riley\Desktop\…`) and carries no version/tag/commit/toolchain/web-dist metadata.
  Fix: one release command must generate manifest+CHECKSUMS+zip from the same build and be committed
  (with `!tools/release-manifest.json` added to `.gitignore`).
- **S5-13 (process): quiescence violation** — the 8→15-file drift confirms the "product under active
  implementation" baseline note; S5-02 ("manifest missing") was true at 11:18 and is superseded by S5-12
  (manifest present but inconsistent). Re-verify S5-01/02/03/12 after the tree is sealed for release.
