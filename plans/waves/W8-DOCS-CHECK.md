# W8 — DOCS TRUTH / RELEASE NOTES (seat 5)

Repo `C:\Users\Riley\Desktop\hiverouter-v1` · HEAD `c9390f5` (`hivemind-1.0.0-3-gc9390f5`) · 2026-11-06.
Write scope: new `tools/qa/check-docs-truth.mjs`, `README.md`, `docs/RELEASE.md`, `docs/RELEASE-EVIDENCE.md`,
`docs/claims/**` (+ this plan file). No commit. Skills applied: forgewright-recon, forgewright-lead,
logic-gatekeeper, integration-logic, logic-debugger.

## Recon (forgewright-recon; every claim has a command/path)

- **Current version = 1.1.0, uncommitted in the working tree:** `web/package.json:4` `"version": "1.1.0"` and all four
  `backend/crates/*/Cargo.toml` `version = "1.1.0"` (`git status` shows them modified). HEAD describes the wave:
  `8a5b583 feat: 1.1 wave 2 — true SSE streaming, replay browser proof + layout fix, deterministic release script, docs truth`
  and `c9390f5 feat: 1.1 wave 3 — G-UI gate conversion + working skip guard, trace truth, UI fetch bounds, tool-loop decision`.
- **Streaming is now real (SSE), not one-delta:** `backend/crates/hiverouter-core/src/provider_executor.rs:96-156`
  implements `stream_reply` ("forwards every text fragment to `on_delta` as it arrives", settles usage from the final
  documented chunk, refuses by name on malformed/incomplete streams); offline matrix
  `backend/crates/hiverouter-core/tests/provider_streaming.rs` (new, wiremock, no key) pins ordered SSE fragments,
  cancellation and pre-first-byte retry. `docs/evidence/1.1-backend-tests.log` (47,508 B, 15:55) is the 1.1 run log.
- **No 1.1.0 artifacts exist yet:** `dist/` holds only `HiveMind-1.0.0-win64.zip` (8,444,021 B) + sidecar and the
  stale pre-integration `HiveMind-v1-win64.zip`; `tools/release-manifest.json` still records the 1.0.0 build
  (`b17eb3d6…`/`49f46e2b…`/`46305fcb…`, source `6a2b1c6`) → **1.1.0 artifact hashes must not be invented**; the
  authoritative hash registry is `tools/release-manifest.json` once a sealed 1.1.0 build runs
  (`tools/build-release.ps1`, W7).
- **Stale claims in current-facing docs:** `README.md:22` ("Status — HiveMind 1.0"), `README.md:25` (1.0.0 hashes
  presented as current), `README.md:28` ("leaf replies are returned as one delta after a non-streaming completion");
  `docs/RELEASE.md:1,5` ("HiveMind 1.0"), `docs/RELEASE.md:21` (one `REPLY_DELTA` after non-streaming completion,
  "true SSE… not claimed in 1.0"); `docs/RELEASE-EVIDENCE.md:1,9-11,19,37` (1.0.0 evidence presented as current,
  one-delta limit stated as current); `docs/claims/CLAIM-REGISTER.json:80` notes 1.0.0 hashes as current;
  `docs/claims/CLAIM-REGISTER.csv` rows `C-026` (one-delta/streaming "remain 1.1/UNVERIFIED", old binary hashes) and
  `C-340` (shipped-v1 hashes as final) need historical/superseded framing.
- **Historical content is legitimate:** the full 1.0.0 release record (hashes, smoke, limits) stays in
  `docs/RELEASE-EVIDENCE.md` and claims registers under an explicit historical/supersession section — the checker
  must allow it there and only there.
- **Tooling conventions:** `tools/qa/*.mjs` are dependency-free Node ESM scripts (`#!/usr/bin/env node`,
  `ROOT = path.resolve(dirname(fileURLToPath(import.meta.url)), '..','..')`, exit 0 pass / 1 failure / 2 file-parse).
  Node v26.7.0 available.
- Unknowns: exact 1.1.0 artifact hashes and sealed commit (no 1.1 build yet) → recorded, not invented; whether the
  release tag will be re-cut is an owner decision.

## Phase 1 (logic-gatekeeper — written BEFORE any mutation)

```json
{
  "phase1": {
    "architecture": "Truth authorities: version = web/package.json + backend/crates/*/Cargo.toml; artifact hashes = tools/release-manifest.json (sealed build); streaming behavior = hiverouter-core/src/provider_executor.rs + tests/provider_streaming.rs. Documentation surfaces: README.md and docs/RELEASE.md are current-facing; docs/RELEASE-EVIDENCE.md is a registry that must separate CURRENT (1.1.0) from HISTORICAL (1.0.0); docs/claims/* are row-level provenance records.",
    "dataFlow": "Source truth files (package.json, Cargo.toml, release-manifest.json, provider_executor.rs) -> docs authors/checker read them -> current-facing docs state version/streaming/hash pointers -> check-docs-truth.mjs masks historical regions, evaluates current-facing rules, exits 0/1/2 -> CI/acceptance consumes exit code; humans read corrected README/RELEASE.",
    "edgeCases": [
      "Old 1.0.0 hashes must remain visible in docs/RELEASE-EVIDENCE.md under a historical section without failing the checker, but must fail outside it.",
      "A claims CSV row may mention 1.0.0 hashes if the row is marked superseded/historical; a current VERIFIED row with one-delta/15-state text must fail.",
      "Version bump to 1.1.0 while tools/release-manifest.json still records 1.0.0: checker must not demand invented 1.1 hashes, only an authoritative pointer, and must not treat the manifest as a doc violation.",
      "Non-streaming/one-delta and 15-state phrases inside fenced code blocks or historical sections must not fail; the same phrases in README/RELEASE current text must fail.",
      "Malformed/missing claims JSON or missing required doc -> exit 2 with a named file error (no silent pass)."
    ],
    "rootCause": "The 1.1.0 version bump and the true-SSE streaming landing changed the product truth, but README, RELEASE, RELEASE-EVIDENCE and the claims registers still assert the 1.0 non-streaming behavior and present 1.0 artifact hashes as current with no automated docs-truth gate; the contradiction is docs claiming a superseded behavior/version while code and version fields say 1.1.0."
  }
}
```

## Decisions

- **D1 Checker is the gate, not a lint hint.** `tools/qa/check-docs-truth.mjs` exits 0 pass / 1 violations / 2 file error;
  it prints `file:line [RULE] message` per violation and a summary. `--root <dir>` lets the corrupt-test run against an
  isolated fixture without touching the repo.
- **D2 Historical masking** (three mechanisms): HTML markers
  `<!-- docs-truth:historical:start -->` … `<!-- docs-truth:historical:end -->`; markdown headings matching
  `/historical|supersed|pre-1\.1|1\.0(\.0)? (release|record)/i` (mask until next heading); claims CSV rows containing
  `supersed|historical|pre-1\.1`; claims JSON entries/notes with the same markers.
- **D3 Rules** (all scoped to current-facing/unmasked text):
  - `VERSION`: web + all crate versions agree; README and RELEASE contain the current version; current-facing docs do not
    call a different major release current (`HiveMind 1.0`, `1.0.0 is current`).
  - `STREAMING`: current-facing docs must state real SSE/token streaming; the one-delta/non-streaming claim fails.
  - `HASHES`: the 1.0.0 hashes (`b17eb3d6…`, `49f46e2b…`, `46305fcb…`, legacy `184D51A7…`/`9B2ED5A7…`/`BC7A2BAD…`)
    must not appear unmasked in current-facing docs; `docs/RELEASE-EVIDENCE.md` current section must point to
    `tools/release-manifest.json` as the exact-hash authority and must not invent 1.1 hashes.
  - `STATE`: `15[ -]states?`/`fifteen states` fails unmasked; 16-state parity is the current truth.
  - `CLAIMS`: claim rows/objects that assert stale behavior/hashes must be marked historical or updated.
- **D4 Docs corrections (no invented 1.1 hashes):** README → 1.1.0 status, SSE streaming real, exact-hash pointer to
  `docs/RELEASE-EVIDENCE.md`/`tools/release-manifest.json`, historical 1.0 block marked; RELEASE → 1.1 release notes with
  true streaming/replay/16-state claims and a historical 1.0 section; RELEASE-EVIDENCE → CURRENT 1.1.0 section (pointer,
  no hashes yet) + HISTORICAL 1.0.0 section (existing hashes/smoke/limits) with markers; claims JSON/CSV → notes/hash rows
  moved to historical supersession with a current pointer.
- **D5 RED→GREEN→corrupt proof (logic-debugger):** build checker first, run on the stale tree (RED, exit 1), edit docs,
  run again (GREEN, exit 0), then run against a corrupted temp fixture (exit 1) and against an untouched copy (exit 0).

## File-by-file

| File | Change |
|---|---|
| `tools/qa/check-docs-truth.mjs` | new: ESM checker (no deps) with VERSION/STREAMING/HASHES/STATE/CLAIMS rules, historical masking, `--root`, exit 0/1/2 |
| `README.md` | 1.1.0 status; SSE streaming real; no 1.0 hashes as current; pointer to `docs/RELEASE-EVIDENCE.md`; historical 1.0 subsection marked |
| `docs/RELEASE.md` | 1.1 release notes: true SSE streaming, replay browser proof, 16-state parity; hash pointer; historical 1.0 section marked; stale one-delta limit removed |
| `docs/RELEASE-EVIDENCE.md` | CURRENT 1.1.0 section (authoritative manifest pointer, exact hashes pending sealed build) + HISTORICAL 1.0.0 section with markers around existing evidence |
| `docs/claims/CLAIM-REGISTER.json` | notes: 1.1.0 current, 1.0.0 hashes into `historical_supersedes`, streaming truth; counts unchanged |
| `docs/claims/CLAIM-REGISTER.csv` | C-026 note: streaming now 1.1-real (no longer "remain 1.1/UNVERIFIED"), old hashes marked superseded; C-340 row: historical 1.0.0 record + current pointer; statuses preserved |

## Failure modes

Missing doc/claims file → exit 2 named; malformed JSON → exit 2; historical markers unbalanced → exit 2; current-facing
stale claim → exit 1; old hashes outside historical section → exit 1; version mismatch (web vs Cargo) → exit 1;
write scope respected (only the five paths above + plan); no commit; no build.

## Tests

```powershell
node tools/qa/check-docs-truth.mjs                      # RED before edits (exit 1), GREEN after (exit 0)
node tools/qa/check-docs-truth.mjs --root <corruptDir>  # deliberate corruption: exit 1
node tools/qa/check-docs-truth.mjs --root <cleanDir>    # control copy: exit 0
git status --porcelain                                  # scope check: only authorized files changed
```

## Rollback

Revert the six edited/created paths (`git checkout -- README.md docs/RELEASE.md docs/RELEASE-EVIDENCE.md docs/claims`
and remove the new checker/plan); no product/runtime code touched; no commit made; the checker is additive and
self-contained.

## UNVERIFIED / out of scope

- Exact 1.1.0 artifact hashes and sealed commit: **do not exist yet** (no 1.1 build); docs point to the authoritative
  manifest and state this explicitly — inventing values is forbidden.
- `tools/build-release.ps1` sealed 1.1.0 build execution (W7, owner-run); claims that depend on its output stay
  `READ`/pending until then.
- Live paid-provider streaming smoke on a 1.1.0 binary: not run (no 1.1 artifact · no key access · no paid calls).
- Historical 1.0.0 counts/hashes are retained as recorded evidence, not re-verified here.

## 9. Execution evidence (2026-11-06; no commit)

| file | bytes | sha256 |
|---|---:|---|
| `tools/qa/check-docs-truth.mjs` | 12916 | `d154f1db922484290f8b412ce793afca8259d750149d282bc02b8b4b7d40fb08` |
| `README.md` | 3739 | `e892f995602b5f0e39d56f4a860d78e0da4d216ae2dd3332ab21191330d6da17` |
| `docs/RELEASE.md` | 4270 | `4d4e3d29e84fb552584420a39d9f88a21ec93e6a78ff9f92b5656ed37614020b` |
| `docs/RELEASE-EVIDENCE.md` | 5230 | `2fc92dc59a964e35a9f43c252c3d52f7be5ac0d590584d312da1d72dfd52effa` |
| `docs/claims/CLAIM-REGISTER.json` | 3715 | `0a0b876cd6ed86c705fe37f5e4eabb7dc16f7f6cbd770d2c326ea0f6a2c9a56c` |
| `docs/claims/CLAIM-REGISTER.csv` | 25079 | `09119e6fef17100e27a53ffb503eefa3d06d4fb64c57297192556946678cdc60` |

**RED → GREEN → corrupt proof (logic-debugger):**
- RED on the stale tree: `node tools/qa/check-docs-truth.mjs` → exit **1**, 20 violations (README:22/25/28, RELEASE:1/5/21, RELEASE-EVIDENCE:1/9-11/19/37, claims JSON notes, claims CSV C-026/C-340).
- GREEN after corrections: `node tools/qa/check-docs-truth.mjs` → exit **0**, `checks: 11/11 pass; violations: 0`, `DOCS TRUTH: PASS` (`--quiet` variant also exit 0).
- Corrupt test (mandated): temp fixture root with a deliberately injected one-delta sentence in README → exit **1**, `VIOLATION README.md:56 [STREAMING] ...`; clean control fixture → exit **0**.
- Extra edge cases: injected `15-state` claim → exit **1** `[STATE]`; stale 1.0 hash + web/Cargo version drift (1.0.0 vs 1.1.0) → exit **1** with 4× `[VERSION]` + 1× `[HASHES]`; the same stale phrases placed inside `<!-- docs-truth:historical:start/end -->` markers → exit **0** (historical allowance, no false positives).
- Fixture isolation: checker `--root <dir>`; corrupt tests never touch the repo tree.

**Scope proof:** `git status --porcelain` shows only the authorized paths changed by this seat: `README.md`, `docs/RELEASE.md`, `docs/RELEASE-EVIDENCE.md`, `docs/claims/CLAIM-REGISTER.{json,csv}`, new `tools/qa/check-docs-truth.mjs`, new `plans/waves/W8-DOCS-CHECK.md`; no other file written; **no commit**.

**Not done / out of scope:** sealed 1.1.0 build + exact 1.1.0 hashes (no 1.1 artifact exists; docs point to `tools/release-manifest.json` and state values are pending); paid live streaming smoke on a 1.1.0 binary (no artifact/key/paid call); `tools/qa/README.md` was not edited (outside write scope), so the new checker is documented here and in its header only.

**UNVERIFIED:** 1.1.0 artifact hashes/commit; live paid streaming behavior; re-run of the full backend/web suites on a sealed 1.1.0 build; whether the owner re-cuts the release tag.
