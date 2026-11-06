# W7 — RELEASE ENGINEERING PLAN (deterministic, hash-anchored release build)

Seat: **W7 Release engineering** · repo `C:\Users\Riley\Desktop\hiverouter-v1` · write scope:
`tools/build-release.ps1`, `tools/release-manifest.json` (+ this plan file). Docs are read-only; no git commit
(Lead-owned); no binaries/dist rebuild unless the script itself performs it. Commit reserved to Lead.

## Recon ledger (read this session; evidence)

- `tools/build-release.ps1` (old, 2,051 B, sha `2b465b0d…`): cargo-only build; writes `release-manifest.json` with
  **absolute** `$artifact` paths (`:24-44`); no web build / staging / CHECKSUMS / zip / tag / version / verify.
- `tools/release-manifest.json` (2,159 B, sha `ae74084b…`): records 1.0 artifacts `target/release/hiverouter.exe`
  `b17eb3d6…` 13,253,120 B; `hiverouter-cli.exe` `49f46e2b…` 5,373,952 B; zip `dist/HiveMind-1.0.0-win64.zip`
  `46305fcb…` 8,444,021 B; `source_commit 6a2b1c6d…`; toolchain string `rustc 1.99.1 (58cf8f9c3 2026-10-13)`;
  `web_dist_files` per-file hashes + `web_dist_algorithm`; **no version/tag/commit date/provider/checksums fields**.
- `docs/RELEASE-EVIDENCE.md` (44 lines, sha `a3a8502c…`): states the same three hashes; zip contains six files
  (both binaries, README, RUNBOOK, RELEASE, CHECKSUMS); final tag gate says "tag only after independent audit and
  quiescent-tree check; moving previous private v1.0.0 tag requires owner confirmation".
- `tools/runbook.md` (95 lines, sha `e7fba10f…` earlier; 5,386 B now): build command + promises the release script
  "prints artifact paths + SHA-256; writes tools/release-manifest.json".
- Repo state: HEAD `0dfe23bf…` = `hivemind-1.0.0-1-g0dfe23b`; **clean tree**; tags `hivemind-1.0.0` → `23ba74a…`
  (= HEAD^), `v1.0.0` → `3988622…` (historical). So HEAD is 1 commit past the release tag (1.1 wave-1 fixes).
- Toolchain: cargo/rustc **1.99.1** at `%USERPROFILE%\.cargo\bin` (not on default PATH); node **v26.7.0**;
  `npm.cmd` available (npm.ps1 blocked by execution policy → script must use `npm.cmd`); gcc/ar in
  `tools/w64devkit/bin`. Repo `.cargo/config.toml` pins **absolute** gcc/ar/linker paths (A-F11 class) — out of W7
  scope but the script must not copy those absolutes into outputs.
- Existing artifacts on disk match the recorded 1.0 hashes: `target/release/hiverouter.exe` `b17eb3d6…`,
  `hiverouter-cli.exe` `49f46e2b…`, zip `46305fcb…`, sidecar `dist/HiveMind-1.0.0-win64.zip.sha256` = `46305fcb…`.
  Zip members (read-only inspect): CHECKSUMS.txt 401 B, cli 5,373,952, exe 13,253,120, README.md 2,113,
  RELEASE.md 3,249, RUNBOOK.md 5,386.
- `.gitignore` ignores `dist/`, `target/`, `.cargo`? (no) — binaries stay out of git; `dist/` already ignored.
- Pre-existing stale `dist/HiveMind-v1-win64*` (pre-integration) files still on disk — must not be reused.

## Outcome (one paragraph)

A single deterministic command — `pwsh -NoProfile -ExecutionPolicy Bypass -File tools/build-release.ps1` — turns a
clean checkout at one commit into a sealed release: web typecheck/tests/build, `cargo build --release --locked`,
staged package (`README.md`, `RUNBOOK.md`, `RELEASE.md`, two binaries, `CHECKSUMS.txt`), deterministic zip +
`.sha256` sidecar, and `tools/release-manifest.json` carrying source commit/tag/date, toolchain versions, provider
contract facts, per-file web-dist hashes + tree hash, per-artifact and package hashes/bytes. The script then
extracts the zip and verifies every hash, refusing on mismatch, and it records the previous 1.0 artifact hashes as
`supersedes` if a rebuild changes bytes under a new staging version. Outputs contain no absolute paths.

## Current behavior (as-is failure modes)

1. Build is not reproducible from the script: it only runs cargo; web/dist may be stale relative to source
   (independent audit found shipped 1.0 artifacts built 09:10/09:33 before the provider/web commit).
2. Manifest paths were absolute (`C:\Users\…`) — unusable on another machine, non-deterministic.
3. No package assembly: staging dir, CHECKSUMS, zip, sidecar, extraction check are manual (and inconsistent with
   `docs/RELEASE-EVIDENCE.md` claims).
4. No tag/HEAD/clean-tree gate: a dirty tree can be "released".
5. No drift guard: rebuilding silently replaces the recorded 1.0 hashes.

## Decisions

- **D1 Deterministic pipeline** (fail-fast, named refusals): preflight → web → cargo → drift policy → stage →
  CHECKSUMS → deterministic zip+sidecar → extract-verify → manifest → absolute-path scan → summary.
- **D2 Inputs frozen at one commit:** default tag `hivemind-<version>`; strict gate requires `HEAD == tag`;
  `-AllowTagAncestor` allows `HEAD` ahead of the tag only for dry-run/audit rehearsals and records
  `tag_behind_commits` honestly. Dirty tree refused unless `-AllowDirty` (records `source_dirty:true`).
- **D3 Version = crate version** parsed from `backend/crates/hiverouter-core/Cargo.toml`; `-Version` overrides.
  Staging version may differ only via `-NewStagingVersion` (e.g. `1.1.0`) when a rebuild changes bytes.
- **D4 Preserve 1.0 artifact hashes:** before building, load `tools/release-manifest.json` and compare recorded
  artifact hashes with on-disk `target/release` binaries. If a rebuild would change bytes and no
  `-NewStagingVersion` is supplied → refuse `E_RELEASE_HASH_DRIFT`; nothing in `dist/` or the manifest is touched.
  With `-NewStagingVersion` → stage under `dist/HiveMind-<newver>-win64/`, write new manifest with old rows in
  `supersedes[]` + `old_to_new` table, and never overwrite the old zip/manifest hashes in place.
- **D5 No absolute paths:** every emitted path is repo-relative forward-slash; script scans CHECKSUMS + manifest
  for `[A-Za-z]:[\\/]` and fails `E_RELEASE_ABSOLUTE_PATH`. Runtime compiler env (gcc/ar/linker) is resolved
  dynamically from `$repo`; values never persisted.
- **D6 Deterministic zip:** `System.IO.Compression.ZipArchive`, entries sorted by name, fixed timestamp
  `1980-01-01T00:00:00Z`; compressed once. Sidecar format matches the existing 1.0 convention (hex only).
- **D7 Manifest schema `hivemind.release-manifest.v1`** with: version/tag/source_commit/source_commit_date/
  source_dirty/generated_at/toolchain{rustc,cargo,node,npm,gcc}/provider{models,max_output_tokens,key_env,
  endpoint}/web{dir,tree_sha256,files[]}/staging_dir/artifacts[]/package{zip,zip_sha256,zip_bytes,sidecar,
  checksums,checksums_sha256,contents[]}/supersedes[]/old_to_new[]/note. Backward-compatible keys kept
  (`artifacts[].path/sha256/bytes`, `web_dist_files`, `toolchain`).
- **D8 Dry-run (`-DryRun`)**: performs every read-only preflight (git/tag/toolchain/version/manifest/on-disk
  artifact comparison/zip inspect) and prints the exact plan and expected paths; **executes no builds and writes no
  files** (verified by before/after `git status` + file listing).
- **D9 Verify before manifest write:** extract built zip to a temp dir, hash each member, compare with staged files
  + CHECKSUMS + manifest; mismatch = `E_RELEASE_VERIFY_FAILED`, manifest not updated.
- **D10 Named exit codes:** 0 ok; 2 precondition; 3 web; 4 cargo; 5 drift; 6 verify; 7 path-policy.

## File-by-file change

| File | Change |
|---|---|
| `tools/build-release.ps1` | full rewrite (PS 5.1-compatible syntax): params `-DryRun -AllowDirty -AllowTagAncestor -Version -Tag -NewStagingVersion -SkipWebTests -KeepVerifyDir`; deterministic pipeline per D1; helpers Rel/Hash/TreeHash/WriteUtf8NoBom/New-DeterministicZip/Test-AbsolutePaths; fails fast with `E_RELEASE_*` codes; prints old→new table |
| `tools/release-manifest.json` | **regenerated only by a real build**; left untouched today to preserve the recorded 1.0 hashes (`b17eb3d6…`, `49f46e2b…`, `46305fcb…`). Dry-run validates the existing file against disk |
| `plans/waves/W7-RELEASE-ENGINEERING.md` | this plan (recon/decisions/tests/rollback) |

## Failure modes handled

Missing/dirty tree, tag missing or mismatched, HEAD past tag, toolchain missing/MSRV, web build/test failure,
cargo build failure, hash drift on same version, zip member mismatch, absolute path leak, sidecar mismatch, stale
`dist/` legacy files (reported, never silently reused), interrupted build (temp verify dir removed in `finally`,
manifest written last atomically via temp+move).

## Tests (commands; read-only / dry-run only here)

```powershell
# syntax/parse (read-only)
$t=$null;$e=$null; [System.Management.Automation.Language.Parser]::ParseFile("tools/build-release.ps1",[ref]$t,[ref]$e); $e
# strict dry-run -> expected named refusal E_RELEASE_TAG_MISMATCH (HEAD is 1 commit past tag hivemind-1.0.0)
pwsh -NoProfile -File tools/build-release.ps1 -DryRun
# rehearsal dry-run allowing ancestor tag -> full plan, no writes/builds, exit 0
pwsh -NoProfile -File tools/build-release.ps1 -DryRun -AllowTagAncestor
git status --porcelain ; Get-ChildItem tools | select Name,Length,LastWriteTime
# full release (Lead-owned, after tag/audit decision; NOT run by W7):
pwsh -NoProfile -ExecutionPolicy Bypass -File tools/build-release.ps1
# drift rehearsal: ... -DryRun -AllowTagAncestor -NewStagingVersion 1.1.0   (plan for supersede path)
```

## Rollback

Script is additive/reversible: restore the previous `tools/build-release.ps1` (`git checkout -- tools/build-release.ps1`
once committed by Lead) and the previous manifest (the file is replaced last; a temp copy is kept as
`tools/release-manifest.json.bak-<ts>` before overwrite). `dist/` outputs are ignored and disposable; old 1.0 zip is
never deleted by the script. No tag is created/moved by the script.

## Out of scope / blockers (documented, not fixed here)

- `backend/.cargo/config.toml` (repo `.cargo/config.toml`) contains **absolute** gcc/ar/linker paths — needs its own
  owner fix (relative `tools/w64devkit/bin` resolution) for true portability; script detects and reports it.
- Tag decision (`hivemind-1.0.0` currently points at HEAD^): Lead/owner must decide re-tag vs `-NewStagingVersion`
  1.1 staging. Script refuses to guess.
- No paid provider call, no live smoke execution; release script does not access the key.
- `docs/RELEASE.md`, `docs/RELEASE-EVIDENCE.md`, `tools/runbook.md` are read-only for W7; script stages them as-is
  and records their hashes so doc drift is visible.

## UNVERIFIED (until a real build is run by Lead)

- Actual reproducibility of binary bytes on a clean rebuild (Rust toolchain/allocator timestamps) — dry-run can only
  compare on-disk hashes.
- Full web test/build execution under the script (not run in W7: task said no rebuild unless script does it;
  rehearsal is dry-run only).
- Whether current `README.md` bytes equal the 1.0 zip member (2,113 B) — reported if a real run is made.
- Tag/version decision and the resulting staging version (owner call).

## 8. Rehearsal evidence (authorized: dry-run only; no rebuild performed)

- **Script implemented:** `tools/build-release.ps1` rewritten — 570 lines / 31,436 B / sha256
  `9f54b2d36f718ac16286c895b734e644448af683dfb1282440083280acbee95c`; PowerShell 5.1 parser: OK; no
  absolute-path literals (`C:\Users`/`C:/Users` scan: none). `tools/release-manifest.json` was **not touched**
  (sha256 `ae74084b9652a09473bd7777971d1605807ba1f37a866fab96e8f135777b5faf`) so the recorded 1.0 hashes
  (`b17eb3d6…`/`49f46e2b…`/`46305fcb…`) are preserved exactly; the file is regenerated only by a real run.
- **Test matrix** (`powershell.exe -NoProfile -ExecutionPolicy Bypass -File tools\build-release.ps1 …`):
  | case | args | observed |
  |---|---|---|
  | T1 strict preflight | `-DryRun` | `[E_RELEASE_DIRTY_TREE]` 25 dirty paths → exit 2 |
  | T2 tag gate | `-DryRun -AllowDirty` | `[E_RELEASE_TAG_MISMATCH]` HEAD `0dfe23bf` vs tag `23ba74a1` (1 ahead) → exit 2 |
  | T3 rehearsal | `-DryRun -AllowDirty -AllowTagAncestor` | `DRY RUN OK` exit 0; plan lists web/cargo/staging/zip/verify/manifest; existing zip `46305fcb…` inspected (6 members) |
  | T4 missing tag | `-DryRun -AllowDirty -Tag hivemind-nope` | `[E_RELEASE_TAG_MISSING]` → exit 2 |
  | T5 staging | `-DryRun -AllowDirty -AllowTagAncestor -NewStagingVersion 1.1.0` | `DRY RUN OK` exit 0; effective 1.1.0, tag `hivemind-1.1.0` missing accepted for staged supersede |
  | T6 staging | `-DryRun -AllowDirty -NewStagingVersion 1.1.0` | same as T5 without needing ancestor allowance → exit 0 |
  | T7 invalid bump | `-NewStagingVersion 1.0.0` | `[E_RELEASE_VERSION]` must be > current → exit 2 |
  **No-write proof:** sha256 of `tools/release-manifest.json`, every file in `tools/`, and all `dist/` files were
  captured before/after the 7 invocations → identical (`NO_WRITE_ALL_PASS=True`). Dry-run never builds, never writes.
- **Not run (deliberate):** the real build. Running it now would rebuild binaries against the dirty in-progress tree
  and could change the sealed 1.0 bytes; the drift guard refuses that (`E_RELEASE_HASH_DRIFT`, exit 5) and requires
  `-NewStagingVersion` + a new staging path, exactly as designed. Owner/Lead must run it on the sealed, tagged commit.
- **Concurrency note:** the working tree changed during testing from other in-flight waves (25 dirty paths, new
  streaming/e2e tests) — the strict clean-tree gate correctly refuses; this is not a W7 write.
- **Out-of-scope residual:** repo `.cargo/config.toml` still pins absolute gcc/ar/linker paths (A-F11 class). The
  script resolves toolchain paths dynamically at runtime and copies none into outputs, but the config itself needs its
  owner to make it portable.
