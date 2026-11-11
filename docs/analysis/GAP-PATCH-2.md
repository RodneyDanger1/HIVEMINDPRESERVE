# GAP PATCH 2 — paste-ready finish-plan section (UI / design docs / evidence / census)

**Source audit:** `docs/analysis/GAP-AUDIT-2.md` (findings F1–F25).
**Baseline:** HEAD `6a41250`; tree dirty only by `?? docs/analysis/`.
**Applies to:** finish-plan sections for **UI surfaces**, **design/FigCraft docs**, **live evidence**, **install/release docs**, **census corrections**.
**Does not apply to** anything in `WATCHDOG-LEDGER.md`/`.jsonl` (already ruled) or the final plan text (Lead merges). This file is a patch section, not the plan.

---

## 0. One-paragraph disposition

The UI is **not** the weak point: the 28-surface router, 36 mounted panels, dev-mock boundary, splash reduced-motion path and FigCraft provenance all hold up under full-file reads and hash checks. The weak point is **prose that outlived its evidence**: the e2e harness README reports a run set the specs no longer contain, the census's feature total contradicts its own rows, one README claim (577 bytes / "Example Domain") has no receipt, and the shipped portable package documents a script it does not ship. Patch the prose, add one evidence index, and re-derive two census counts; do **not** re-architect the UI.

**Estimated patch shape:** 22 one-line/text edits + 2 new files (`docs/evidence/live/INDEX.md`, a Downloads section) + 1 optional guard test. No Rust changes required by this lane.

---

## 1. Gaps this lane closed (state as closed, with evidence)

| # | Gap (watchdog ref) | What was proved | Evidence |
|---|---|---|---|
| C1 | `mock.ts` body unaudited (`§4.5`) | Dev-only double cannot reach a production bundle; it fails loudly; it emits only typed event names | `web/src/api/mock.ts:1-368` read; `api/index.ts:32,62-80`; byte-scan of `web/dist/assets/index-BNz-sfds.js` = 0 hits for `MockApiClient` / `dev-mock-session-token` / `VITE_UI_MOCK` / `no scenario for` |
| C2 | `FigCraftGallery.tsx:140-344` unaudited (`§4.5`) | A11y contract implemented; no fabricated preview; failed SVG degrades to a named note; provenance shown is real | `FigCraftGallery.tsx:64,74-81,90,113-119,153-157,240-255,257-264,324-331`; 21/21 board hashes + 4/4 manifest hashes match disk |
| C3 | Live receipts aggregate-only (`§4.4`, `L7-009`) | Per-file field table: what each of the 26 receipts proves and does not; 4 overclaims located | `GAP-AUDIT-2.md §4` |
| C4 | 25 absent paths never independently checked | List, count, tracked-status and zero-literal-mention all reproduce; composition is 21 SVG + `SOURCES.json` + `index.html` + lockfile + `.gitignore` | `GAP-AUDIT-2.md §5` |
| C5 | Census numbers taken on trust | 1,573/1,573 and the 410/863/275/25 arithmetic reproduce; two line totals and the 216 total do **not** | `GAP-AUDIT-2.md §2, §6` |
| C6 | Trace ladder conflict open | `controls.json` = 1 WIRED / 264 PENDING / 5 MISSING / 49 UNVERIFIED (319 = 29×11), so `e2e/README.md:67-68` is stale, not authoritative | independent flatten of `docs/trace/controls.json` |

---

## 2. Work packages (exact files, exact edit, acceptance gate)

### WP-UI-1 — Splash/CSS claim truth (LOW, 1 edit + optional guard)
- **File:** `web/src/styles/splash.css:9-10`.
- **Edit:** replace "Every color is a FigCraft token; no new palette is introduced." with "Token palette; glass/alpha shades are written as literal `rgb()` values because no alpha tokens exist (19 `var(--color-*)` + 33 `rgb()` in this file)."
- **Optional guard:** extend `web/tests/noHardcoded.test.ts:20-31` from hex-only (`/#[0-9a-fA-F]{6}\b/`) to also flag `rgb(`/`oklch(` outside `styles/tokens.css`, then either fix the 33 literals or allowlist `splash.css` with a comment.
- **Acceptance:** `Select-String -Path web/src/styles/splash.css -Pattern 'rgb\('` count matches the sentence; if the guard is extended, `npm test -- --run tests/noHardcoded.test.ts` is executed and its output quoted (currently unrun).
- **Do not:** rewrite the material recipe or the 33 literals to tokens you invent — no alpha token exists in `tokens.css`.

### WP-UI-2 — Gallery robustness (LOW, 1 edit)
- **File:** `web/src/figcraft/FigCraftGallery.tsx:22-26` + `:241-254`.
- **Edit:** delete the synthetic `FIRST_GROUP` fallback; before rendering the tablist, return the existing empty-catalog message (`:257-264`) when `FIGCRAFT_GROUPS.length === 0`, so `% count` can never be `NaN`.
- **Acceptance:** source diff only; no runtime claim. If a test exists for the gallery, add one empty-groups case.
- **Do not:** change `catalog.ts` — it is generated (`catalog.ts:1-6`).

### WP-UI-3 — Mock lifecycle (LOW, 1 edit)
- **File:** `web/src/api/mock.ts:62-85,163,183`.
- **Edit:** keep timer ids in a set, clear them in `close()`; emit nothing after close.
- **Acceptance:** `web/tests/apiBoundary.test.ts` still passes **when executed** (it is currently only read) and a new dev-only test asserts no event is delivered after `close()`.
- **Do not:** remove the constructor guard at `:67-76` or the dynamic import at `api/index.ts:67-69` — those are the boundary.

### WP-DOCS-1 — e2e harness README truth (HIGH, rewrite of 4 blocks)
- **File:** `e2e/README.md`.
- **Edits:**
  1. `:3-10` — replace the "93 tests = 79 passed … 14 skipped … explicit `test.fixme`" block with a **dated measured run** (`command → collected/passed/skipped`, log path that exists) **or** `Status: UNVERIFIED — receipt predates the current spec set`. Delete "explicit `test.fixme`": the specs contain 0 `test.fixme(` and `gui-gates.spec.ts:356-361` asserts they must not.
  2. `:5` — replace `index-Cbfm1IuM.js` with the run-time dist identity or "embedded `web/dist` at build time"; keep the verified binary hash `184D51A7…C9A9`.
  3. `:24` — replace `blueprint/UI-WIRING-GUIDE.md` (does not exist) with `e2e/data/control-matrix.json` (29 surfaces) + `docs/contracts/UI-COMMANDS-v1.md`.
  4. `:67-68` — replace "43 WIRED-AND-WORKS / 260 UNVERIFIED / 5 MISSING" with "1 WIRED-AND-WORKS / 264 PENDING / 5 MISSING / 49 UNVERIFIED of 319 (29 controls × 11 hops)" + one line that the 43/260 ladder was invalidated by the empty raw capture (`controls.json` `_meta.live_capture`).
  5. `:79` — **delete** "FigCraft canonical manifest `ASSETS-INDEX.json` still absent"; it exists (89,885 B, sha256 `FE0F6039…EA5F`). Replace with "A/B manifests accepted and hashed; pixel-layout fidelity UNVERIFIED".
  6. `:59-63` — mark per-spec pass counts `UNVERIFIED (no tracked run log)` or attach the log.
- **Acceptance:** every path in the rewritten README resolves (`Test-Path` for each referenced file), every count is either a measured number or explicitly labelled UNVERIFIED, and no claim references a missing artifact.
- **Do not:** delete the honest "Known UNVERIFIED / open" section — it is the best part of the file.

### WP-DOCS-2 — README evidence claims (HIGH, 2 edits)
- **File:** `README.md`.
- **Edits:**
  1. `:306` — drop "577 bytes" and "Agent reported `Example Domain`" unless a receipt with `bytes`/`body_excerpt` is added to `docs/evidence/live/`. Replacement text: "approval-gated `web_fetch` to `https://example.com` completed with HTTP 200 and one `web_fetch:ok` tool outcome (`live-tokenlab-web-fetch.json`); response body size/content not captured."
  2. `:191` — cite both `live-tokenlab-exact-two-stability.json` (1 completed + 1 failed) and `live-tokenlab-final-exact-two.json` (1 completed) for "2 of 3"; the stability receipt alone is 1 of 2.
- **Acceptance:** each number in §11/§7 has a receipt or is labelled UNVERIFIED; `grep` shows "577" no longer appears unbacked.
- **Do not:** touch `:171-174` hashes — they are exact (`15BF4E61…`, `F369D7AC…`).

### WP-DOCS-3 — Portable package install truth (HIGH, package + doc)
- **Files:** `dist/HiveMind/README.txt`, package contents, `docs/DOWNLOADS.md` (new) or `README.md` §6.1.
- **Edits:**
  1. `README.txt:14-15` currently points at `tools\set-provider-key.ps1`, which is **not in the zip** (zip = `HiveMind.exe` 10,240 B, `hiverouter.exe` 36,540,416 B, `README.txt` 1,038 B). Ship the script/shim in the package or repoint the instruction.
  2. `README.txt:1-3` — add "1.2.0-dev (unreleased); sealed release is 1.1.0" + commit + build date.
  3. Add `CHECKSUMS.txt` to the package (the 1.1.0 staging dir has one; all five of its listed hashes verify) and a manifest hash for the dev zip.
  4. Downloads section must state: version + commit + build date, zip name/size/SHA-256, contents list, prerequisites (Win10/11 x64 + WebView2), first-run, key location/rotation, data dir `%LOCALAPPDATA%\HiveMind`, **workspace root semantics** (launcher hard-codes a dev root — `tools/desktop-launcher/HiveMindLauncher.cs:17,244-264`), CLI availability, SmartScreen, uninstall/rollback, dev-vs-sealed label.
- **Acceptance:** a fresh extract of the zip can perform every documented step without the repo; state that as the test and quote the extract listing.
- **Do not:** publish the package as a release; `tools/release-manifest.json` is `SEALED_1.1.0` and explicitly says no 1.2 manifest/artifact exists.

### WP-EV-1 — Live evidence index + field completion (HIGH, 1 new file + 4 receipts)
- **Files:** `docs/evidence/live/INDEX.md` (new); `live-tokenlab-web-fetch.json`; `live-tokenlab-kill-usage.json`; `live-tokenlab-lesson-retrieval.json`; `live-tokenlab-proposal-votes.json` / `live-tokenlab-proposal-dedupe.json`.
- **Edits:**
  1. `INDEX.md`: one row per receipt with `observed_at`, `head_commit`, what it proves, what it does not, and a grade (`READ` not `VERIFIED`). Mark 18:02Z/18:37Z receipts as **pre-apply-implementation** (their `vote_path: "…not_implemented"` is stale versus 21:56Z/23:14Z apply receipts).
  2. `live-tokenlab-web-fetch.json`: add `bytes` and `body_excerpt` (or `body_not_captured: true`) so `README.md:306` has a field to cite (see WP-DOCS-2).
  3. `live-tokenlab-kill-usage.json`: replace `"hard_kill": {}` with `{pid, signal, killed_at, exit_code}` or rename to `kill_note` + `kill_mechanism_not_receipted: true`.
  4. `live-tokenlab-lesson-retrieval.json`: add the child answer field or soften the `note` (it claims "final and child answers both returned HONEYCOMB" but only `final_answer` exists).
  5. Every new receipt carries `head_commit` and `observed_at`.
- **Acceptance:** each README/evidence claim maps to a named field in a named receipt; `INDEX.md` states the aggregate-only limitation once (no raw frames/prompts/bodies).
- **Do not:** re-run any provider call to "fill" fields — mark `not_captured` instead.

### WP-CENSUS-1 — Arithmetic and classification corrections (MEDIUM, 5 edits)
- **File:** `docs/analysis/CODEBASE-CENSUS.md` (the census is an input, not the final plan; the Lead applies).
- **Edits:**
  1. `:490-502` — split the feature table: **216** = 63 routes + 17 commands + 24 events + 12 tools + 8 personas + 28 tables + 36 panels + 28 surfaces; **+33** = 23 migrations + 10 prompt assets; **outside both** = 78 refusal variants + 4 caps. Delete "Total 216" over rows that sum to 249.
  2. `:23` — restate the verdict to `216/216` **with that scope**, and mention that refusals (78) and caps (4) are enumerated in JSON but not feature-counted.
  3. `:379` — delete "i.e. shipped … UI-census discrepancy"; `DevGallery.tsx:8-10` returns `null` in production, so the docstring is correct.
  4. `:362` — "10 surfaces render distinct content" → "10 surface groups; 9 distinct panel compositions (RoutingPanel at `#/workspace/routing` and `#/routing/*`); 18 duplicate addresses".
  5. `:520` — "22 served FigCraft assets" → "21 served SVG boards + `SOURCES.json`".
  6. `:397-399` — append the decode-refusal pins: `tests/ui_command_matrix.rs:113`, `tests/common/mod.rs:411-427`, `refusal.rs:197`. **Do not implement `DIFF_*`** without changing those two tests.
  7. `:58-59` — recompute or delete the `18,955` / `4,572` line totals (measured: 19,977 for 78 `web/src` files; 4,916 for the 31 web test files). File counts 78/32 are correct.
- **Acceptance:** every total in the census is reproducible by the stated command; no row sits under a total it is not part of; the 3 MISSING rows carry the test pins.

### WP-HYGIENE-1 — Ignore rule vs tracked FigCraft sources (LOW, 1 edit)
- **File:** `.gitignore:8-12`.
- **Edit:** add `!docs/ui/figcraft/ASSETS/[ab]-*.svg` (and the PNG equivalent if those are meant to be tracked) or delete `docs/ui/figcraft/ASSETS/*.svg` / `*.png`. Today all 21 source SVGs are tracked but a **new** A/B board would be silently ignored.
- **Acceptance:** `git check-ignore -v docs/ui/figcraft/ASSETS/a-brand.svg` returns nothing for a tracked-or-new board; `git ls-files docs/ui/figcraft/ASSETS/*.svg` still 21.

### WP-RELEASE-1 — Manifest drift note (MEDIUM, 1 edit or 1 re-seal)
- **File:** `tools/release-manifest.json` (or a dated note beside it).
- **Edit:** `web.web_dist_files` records `index-DRvC4Qb-.js` (675,204 B) / `index-D8k8-PfA.css` (35,807 B); on disk `web/dist` is `index-BNz-sfds.js` (906,532 B) / `index-Dlq9NyYb.css` (72,711 B). Either re-seal from a fresh build or add `"stale_web_dist": true` + measured hashes + date. Also: `provider.max_output_tokens: 393216` has no versioned source — cite a URL+version or drop it; and `provider.models` does not list the receipts' `wire_model: "DeepSeek-V4.1-Flash"`.
- **Acceptance:** manifest hashes match `web/dist` or the drift is explicitly recorded; every numeric provider fact carries `version_source`.

### WP-DOCS-4 — CODEMAP snapshot banner + shallow-clone caution (LOW/MEDIUM, 1 edit + 1 plan note)
- **Files:** `docs/CODEMAP/00-START-HERE.md:3,11,17`; any finish-plan gate that calls `git describe`, `git merge-base` or `git rev-list --count`.
- **Edits:**
  1. Keep the number **805** — it is exact for the freeze commit (`git ls-tree -r --name-only e021b12e5bb7` = 805). Add one banner: "Superseded snapshot: frozen at `e021b12` (2026-11-06). HEAD `6a41250` tracks **1,573** paths; see `CODEBASE-CENSUS.md`."
  2. Plan note: this workspace is a **shallow clone** (`git rev-parse --is-shallow-repository` = true), so `git describe` returns `6a41250` with no tag and `git merge-base 87079ce… HEAD` returns nothing. That is a clone artifact, **not** evidence of rewritten history or an unrelated sealed release. Gates that need ancestry must `git fetch --unshallow` (network + write — not in this lane's scope) or cite `tools/release-manifest.json` `tag_target` / `source_commit` / `tag_behind_commits: 0` and say that is the basis.
- **Acceptance:** no plan or gate asserts tag ancestry from this clone; CODEMAP carries the superseded banner; the 805 figure is not "corrected" to 1,573 in place.
- **Do not:** rewrite CODEMAP's historical counts — it is a dated snapshot like `docs/audit/CAPABILITY-MATRIX.md` (L7 §3.7).

---

## 3. Corrections ledger (one-line, apply in this order)

| Order | File:line | Action |
|---:|---|---|
| 1 | `e2e/README.md:79` | delete the false "ASSETS-INDEX absent" item (highest misinformation per line) |
| 2 | `e2e/README.md:67-68` | replace the invalidated 43/260 ladder with 1/264/5/49 of 319 |
| 3 | `e2e/README.md:3-10` | measured run or UNVERIFIED; delete "explicit test.fixme" |
| 4 | `e2e/README.md:5` | remove the nonexistent `index-Cbfm1IuM.js` |
| 5 | `e2e/README.md:24` | replace `blueprint/UI-WIRING-GUIDE.md` |
| 6 | `README.md:306` | drop "577 bytes"/"Example Domain" or attach fields |
| 7 | `README.md:191` | cite both exact-two receipts |
| 8 | `dist/HiveMind/README.txt:14-15` | ship the key script or repoint |
| 9 | `dist/HiveMind/README.txt:1-3` | add version/commit/dev label |
| 10 | `web/src/styles/splash.css:9-10` | token-palette wording |
| 11 | `CODEBASE-CENSUS.md:490-502` + `:23` | split the 216 / +33 / +82 totals |
| 12 | `CODEBASE-CENSUS.md:379` | DevGallery correction |
| 13 | `CODEBASE-CENSUS.md:362`, `:520` | composition wording; 21 SVG + 1 manifest |
| 14 | `CODEBASE-CENSUS.md:397-399` | add decode-refusal test pins |
| 15 | `CODEBASE-CENSUS.md:58-59` | recompute/delete line totals |
| 16 | `web/src/figcraft/FigCraftGallery.tsx:22-26` | delete dead fallback |
| 17 | `web/src/api/mock.ts:62-85` | clear timers on close |
| 18 | `.gitignore:8-12` | un-ignore A/B source SVGs |
| 19 | `tools/release-manifest.json` | stale-dist note + provider source |
| 20 | `docs/evidence/live/INDEX.md` (new) | ordering + head_commit + grade |
| 21 | `docs/CODEMAP/00-START-HERE.md:3,11` | keep 805; add "superseded; HEAD tracks 1,573" |
| 22 | finish-plan gates using `git describe`/`merge-base` | cite the sealed manifest; record that the clone is shallow |

---

## 4. Residual unknowns (carry into the plan's Unverified table)

| # | Unknown | What would close it | Owner |
|---|---|---|---|
| U1 | Whether the 28 hash surfaces, gallery keyboard path, splash timing and mock banner behave as read | One Playwright/browser run against the shipped binary with a captured trace; quote the run log path | UI / e2e |
| U2 | Executed test counts (81 declarations vs 93 claimed vs 95 in L6) and skip set | `npx playwright test --project=chrome` run log with `--reporter=list`; count collected/passed/skipped | e2e |
| U3 | Whether `dist/HiveMind/hiverouter.exe` embeds the current `web/dist` | Rebuild + hash the embedded dist tree, or diff embedded asset names from the binary | release |
| U4 | Provider-side identity `DeepSeek-V4.1-Flash` / `api.tokenlab.amd.com` | Provider call with a response dump (out of scope here; requires approval) | provider lane |
| U5 | Content of the web-fetch response (bytes, text) | A receipt with `bytes` + `body_excerpt`, or leave `not_captured` | evidence lane |
| U6 | Whether a hard kill occurred in the kill-usage receipt | Receipt fields `pid/signal/killed_at/exit_code` | evidence lane |
| U7 | Exact 216-vs-249 scope the Lead wants as the "feature total" | A decision, not a measurement: pick 216 (8 families) or 331 (all JSON vocabularies) and state it once | Lead |
| U8 | Whether `hivemind-1.1.0` (`87079ce`) is an ancestor of HEAD `6a41250` | `git fetch --unshallow` (network + write), then `git merge-base --is-ancestor`; until then cite the manifest record only. **Do not** describe the histories as unrelated — this clone cannot tell | release / Lead |
| U9 | Whether CODEMAP's 378-file appendix still matches its own freeze commit | Re-run its inventory against `e021b12` (read-only `git ls-tree`) — out of scope here | docs |

---

## 5. Explicit do-not list

1. **Do not** "fix" the 3 `DIFF_*` rows by adding a `ws.rs` arm — `tests/ui_command_matrix.rs:113` and `tests/common/mod.rs:411-427` pin them as decision-gated/undecodable.
2. **Do not** rewrite the seven lane reports to match this audit — the Lead merges; the census corrections live in the census.
3. **Do not** regenerate `web/src/figcraft/catalog.ts` by hand — it is generated (`catalog.ts:1-6`); if you touch it, re-run the acceptance step and re-verify all 21 hashes.
4. **Do not** change `web/src/api/mock.ts`'s DEV guard or the dynamic import to "simplify" — that is the boundary proven in C1.
5. **Do not** treat the 26 JSON receipts as VERIFIED — they are READ-grade summaries; the only honest upgrade is a stored raw capture.
6. **Do not** re-seal `tools/release-manifest.json` to 1.2 while `run-gates.ps1` cannot pass (watchdog §5 #10); record the drift instead.
7. **Do not** claim from this workspace that `hivemind-1.1.0` is unrelated to HEAD, that history was rewritten, or that HEAD is untagged — the clone is shallow (`git rev-parse --is-shallow-repository` = true) and every such statement is unsupported here (F27).

---

*End of patch section. Read-only lane; only this file and `GAP-AUDIT-2.md` were written.*
