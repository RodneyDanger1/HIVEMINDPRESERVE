# GAP AUDIT 2 — UI / design docs / evidence receipts / census

**Artifact class:** audit (read-only lane; no source, test, config, lane report, ledger or plan was edited by this work).
**Repo:** `C:\Users\Riley\Desktop\hiverouter-v1` — HEAD `6a4125070186da1bbe3f088b3634f3f6d5402fab`, branch `master`, `git status --porcelain` = `?? docs/analysis/` only.
**Inputs reopened:** 7 lane reports (`docs/analysis/lanes/L1..L7`), `docs/analysis/WATCHDOG-LEDGER.md` + `.jsonl` (247 / 100 rows), `docs/analysis/CODEBASE-CENSUS.md` / `.json` / `.mmd`, `docs/analysis/LEAD-RESEARCH-PSEUDOCODE-LANGUAGES.md`.
**Writes:** this file and `docs/analysis/GAP-PATCH-2.md` only.
**Not run:** no test, build, server, provider call, browser, DB open. No key read or printed. Every count below was produced by a command in this session and is stated with its method.

> **Relation to the watchdog.** This lane closes three holes the watchdog itself declared: `§4.4` live-receipt field audit, `§4.5` `web/src/api/mock.ts` body and `FigCraftGallery.tsx` beyond line 140, and it independently re-derives the census numbers. Where the watchdog already ruled, this report extends or corroborates it and says so; it does not restate confirmed findings as new.

---

## 0. Verdict

**`FIXABLE`** — the UI/design/evidence surface is materially more honest than its docs; the defects are concentrated in **stale prose and counts**, not in fabricated UI behaviour or fabricated design provenance.

| Counts (this lane) | CRITICAL | HIGH | MEDIUM | LOW | NOT-CHECKABLE |
|---|---:|---:|---:|---:|---:|
| Findings (F1–F27) | 0 | 6 | 13 | 8 | 3 |

(HIGH = F1, F6, F7, F8, F13, F19 · MEDIUM = F2, F3, F9, F10, F11, F12, F14, F15, F16, F17, F20, F25, F27 · LOW = F4, F5, F18, F21, F22, F23, F24, F26 · the 3 NOT-CHECKABLE items are the residual class in §7: provider identity, executed e2e counts, binary-embedded dist.)

Headline results:

1. **The FigCraft design provenance is real.** All 21 board `sha256` values in `web/src/figcraft/catalog.ts` match **both** the accepted source `docs/ui/figcraft/ASSETS/*.svg` and the served copy `web/public/figcraft/*.svg`; all 4 source-manifest hashes match. `web/public/figcraft/` is **22 files = 21 SVG boards + `SOURCES.json`** — the brief's "22 FigCraft SVG boards" is off by one (F19).
2. **The dev-mock boundary holds.** Full read of `web/src/api/mock.ts:1-368` plus a byte-scan of the shipped `web/dist/assets/index-BNz-sfds.js`: zero occurrences of `MockApiClient`, `dev-mock-session-token`, `VITE_UI_MOCK` or the mock's scenario strings. The only matching string in the bundle is the UI banner `mock:"DEV MOCK · simulated data"` (`web/src/state/strings.ts` via `App.tsx:175-179`), i.e. an honest label, not the double.
3. **The census is complete on paths but arithmetically wrong on features.** 1,573/1,573 tracked paths reproduce exactly, and the 25-path absent list reproduces exactly — but §15.4's "Total **216**" contradicts its own rows, which sum to **249**. The 216 is the 8-family count from §0; migrations (23) and prompt assets (10) are listed as rows but are outside it, and refusals (78) and caps (4) are enumerated in the JSON but excluded from the total entirely (F1).
4. **Two docs are actively contradicted by files in the same tree.** `e2e/README.md:79` says `ASSETS-INDEX.json` is "still absent" — it exists, is tracked, and its hash is in the catalog (F12). `e2e/README.md:67-68` reports a 43-WIRED hop ladder that `docs/trace/controls.json` explicitly invalidated and replaced with 1 WIRED (F11).
5. **`README.md:306`'s web-fetch numbers have no receipt behind them.** "577 bytes" and "Example Domain" occur nowhere in the repository except that README line (F13).

---

## 1. Method and reproducibility

| Claim | Command / read | Result |
|---|---|---|
| Tracked paths | `git ls-files` | **1,573** |
| JSON index | `ConvertFrom-Json` → `.index.Count` | **1,573** |
| Absent-path list | JSON `omissions.absent_from_all_lane_reports` vs `CODEBASE-CENSUS.md:480-484` | identical, **25** |
| Absent paths are tracked | `git ls-files web/index.html web/package-lock.json .gitignore` | all 3 return; 22 figcraft files return |
| Literal lane mentions | substring scan of the 7 reports (506,864 chars) for each absent path and its basename | **0** file references; `index.html` appears only as prose (F18) |
| FigCraft served set | `git ls-files web/public/figcraft` + `Get-ChildItem` | **22** = 21 `.svg` + `SOURCES.json` (dir also 22) |
| Board provenance | 21-entry regex over `catalog.ts` → `Get-FileHash` on `sourcePath` and `web/public/figcraft/<svgFile>` | **21/21 match**, 0 missing sources |
| Manifest provenance | `Get-FileHash` on the 4 paths in `FIGCRAFT_SOURCE_MANIFESTS` | **4/4 match** (case-insensitive) |
| Production bundle mock scan | `[IO.File]::ReadAllText(web/dist/assets/index-BNz-sfds.js)` | 0 mock markers; 1 banner string |
| Trace ladder | `controls.json` → flatten `controls[].hops.*.verdict` | **319 = 264 PENDING + 49 UNVERIFIED + 5 MISSING + 1 WIRED** (29 controls × 11 hops) |
| E2E declarations | title-string regex `^\s*test(\.fixme|\.skip|\.only)?\s*\(\s*['"\`]` over `e2e/specs/*.spec.ts` | **81** sites in **16** specs, **20** `describe`, **0** `test.fixme(` |
| Web file/line counts | `Get-Content .Count` and LF byte count | `web/src` **78 files / 19,977 lines**; `web/tests` **32 files** (31 test files = 4,916 lines; 32 = 5,013) |
| Portable zip members | `System.IO.Compression.ZipFile` | **3**: `HiveMind.exe` 10,240 B, `hiverouter.exe` 36,540,416 B, `README.txt` 1,038 B |
| Package/binary hashes | `Get-FileHash` | `dist/HiveMind.zip` = `15BF4E61…CA5DA` (README:171-172 ✅); `dist/HiveMind/hiverouter.exe` = `F369D7AC…CF871` (README:173-174 ✅); `tools/bin/hiverouter.exe` = **`184D51A7…C9A9`** (e2e README:5 prefix ✅) |

Encoding note (false-positive avoided): console output renders `·`/`—` as mojibake in this shell. `web/public/figcraft/SOURCES.json` was byte-checked — no `0xBD` byte, and the UTF-8 decode contains `00 · Cover`; the file is **not** corrupted. Do not "fix" that encoding.

---

## 2. Verified claims (no action)

| # | Claim | Evidence | Status |
|---|---|---|---|
| V1 | 1,573 tracked = 1,573 indexed; 0 unclassified | `git ls-files`; JSON `.index`; `omissions.unclassified` = 0 | **VERIFIED** |
| V2 | Coverage arithmetic closes | 410 named + 863 scope-specific + 275 scope-broad + 25 absent = 1,573; JSON tier counts match `CODEBASE-CENSUS.md:465-474` | **VERIFIED** |
| V3 | The 25 absent paths are exactly the 22 figcraft public files + `web/index.html` + `web/package-lock.json` + `.gitignore` | JSON list; `git ls-files`; §15.3 (`:480-484`) | **VERIFIED** |
| V4 | These 25 are build inputs, not dead files | `static_ui.rs:13-16` embeds `../../../web/dist`; `web/dist` contains `index.html` + `figcraft/` (22 files) | **VERIFIED** |
| V5 | 7 nav × 4 tabs = 28 addressable hash surfaces; only `workspace` varies by tab | `App.tsx:32-33,47-81,109-163` | **VERIFIED** |
| V6 | 36 panel components, all mounted; NavRail docstring drift ("six" vs seven items) | 36 files under `web/src/panels/`; `NavRail.tsx:1` vs `:9-17`; `App.tsx:32` | **VERIFIED** (count nuance F4) |
| V7 | `DevGallery` is reachable only in dev and renders nothing in production | `SettingsPanel.tsx:540-547` (settings → `figcraft` tab), `DevGallery.tsx:8-10` (`if (!import.meta.env.DEV) return null`) | **VERIFIED** — census wording is what is wrong (F3) |
| V8 | Dev mock is compile-time gated, dynamically imported, and refuses construction outside dev | `api/index.ts:32,62-80`; `mock.ts:67-76`; production bundle scan | **VERIFIED** |
| V9 | Mock emits only declared event names and fails loudly on unknown commands | `mock.ts:98-121` (`EVENT_NAMES` typed), `:336-342` (ERROR + `throw RefusalError`); `isMockEventName` used by tests | **VERIFIED** |
| V10 | FigCraftGallery's a11y contract is implemented, and a failed SVG load degrades to an explicit note (never a broken image or invented preview) | `FigCraftGallery.tsx:240-255` (roving Arrow/Home/End), `:64` (`aria-pressed`), `:74-81,:90,:153-157` (unavailable state), `:324-331` (labelled region + polite status), `:113-119,:257-264` (explicit empty states) | **VERIFIED** |
| V11 | Each gallery card shows real provenance (manifest path, source path, sha256) that matches disk | `FigCraftGallery.tsx:169-184`; hashes 21/21 + 4/4 verified | **VERIFIED** |
| V12 | Catalog counts 21 boards / 114 surfaces / 3 owners / 60 distinct surfaces | `catalog.ts` `FIGCRAFT_COUNTS`; `SOURCES.json` `counts`; regex count 114 `surfaceId` rows | **VERIFIED** |
| V13 | Splash motion is finite and the reduced-motion switch disables it | `splash.css` has no `infinite` (only the comment at `:10`); `index.css:101-119` sets `animation-duration:.001ms!important` for `[data-reduced-motion="true"] *` and `prefers-reduced-motion` | **VERIFIED** |
| V14 | Splash is a real React surface with a bounded wait, not a fake progress loop | `HiveMindSplash.tsx:41-60` (1.6 s floor, 4.4 s cap, 420 ms leave), `:119-125` (`role="progressbar"`), `:28-33` (connection-derived label) | **VERIFIED** |
| V15 | 3 MISSING command rows are real and correctly located | `command.rs:77-82`; no `Diff` arm in `ws.rs`; `tool.rs:115-140` = 12 tool ids | **VERIFIED** (classification nuance F2) |
| V16 | Trace ladder 1/264/5/49 of 319 | independent flatten of `controls.json` | **VERIFIED** (census citation correct; e2e README stale — F11) |
| V17 | 81 declared e2e sites in 16 specs, 20 `describe`, 0 `test.fixme(` | title-string scan; `test.fixme(` grep = 0; 49 `test.skip(` call sites | **VERIFIED** (census §17 #5's 81 correct *for declarations*; L6's "95 tests / 48 skips" is a different counting rule — its 48 gated skips ≈ my 49 `test.skip(` sites — not a typo; e2e README's 93/79/**14 fixme** is the part that fails) |
| V18 | Release hashes in README §6 are exact | `Get-FileHash` on both artifacts | **VERIFIED** |

---

## 3. Findings

### F1 — HIGH — census §15.4 "Total 216" contradicts its own rows (249), and 4 feature families sit outside the verdict
**Files:** `docs/analysis/CODEBASE-CENSUS.md:490-502` (table + `| **Total** | **216** |`), `:23` (the 216 enumeration), `CODEBASE-CENSUS.json` → `.features` (`http` 63, `ws` 17+24, `tools` 12, `personas` 8, `persistence` 28+23, `ui` 28+36, `refusals` 78, `caps` 4).

Arithmetic: `63+17+24+12+8+28+23+36+28+10 = 249`, not 216. 216 = `63+17+24+12+8+28+36+28` (the eight families named in §0 line 23). So **migrations (23) and prompt assets (10) are presented as members of a total they are not in**, and the JSON has **no `prompt_assets` family at all** (only the MD row claims 10). Excluded from the 216 and from §15.4 entirely: **refusals 78** and **caps 4** — both are product-surface vocabularies present in JSON (`features.refusals.rows`, `features.caps`) with owner lanes, but no row and no owner statement in the report.

**Correction (exact):** retitle the table "216 owned families" and split it:

```
HTTP method+path 63 · WS commands 17 · WS events 24 · Tools 12 · Personas 8
DB tables 28 · UI panels 36 · UI hash surfaces 28            = 216 (verdict scope)
Additional enumerated, not in the 216:
DB migrations 23 · Prompt assets 10                           = 33
Enumerated in JSON, outside both totals:
Refusal variants 78 · Caps 4                                  = 82
```
and change the verdict sentence to either `216 / 216` **with that scope stated**, or `249 / 249`, or `331 / 331` if refusals+caps are claimed. Do not leave "Total 216" above rows that sum to 249.

### F2 — MEDIUM — the 3 MISSING rows are accurate but under-cited; a finisher could "implement" them and break two tests
**Files:** `command.rs:77-82`; `tests/ui_command_matrix.rs:113` (`fn decision_gated_diff_commands_never_decode()`); `tests/common/mod.rs:411-427` (expects `Refusal::DecisionRequired` for all three); `refusal.rs:197` (`E_DIFF_DECISION_REQUIRED`).

The census says "no `ws.rs` arm; no handler" — true — but the three names are **pinned by two tests as intentionally undecodable**, not merely forgotten. **Correction:** in §12 add the two citations and reclassify the rows as `DECLARED + DECODE-REFUSED BY DESIGN (decision-gated)` or keep `MISSING` but add "do not add a handler without changing `ui_command_matrix.rs:113` and `common/mod.rs:411-427`".

### F3 — MEDIUM — census calls `DevGallery` "i.e. shipped"; it is dev-gated to a no-op in production
**Files:** `CODEBASE-CENSUS.md:379`; `DevGallery.tsx:1,8-10`; `SettingsPanel.tsx:540-547`.

The docstring says "Never rendered in production builds", and `:8-10` enforces exactly that (`if (!import.meta.env.DEV) return null`). The census reads it as a discrepancy and as "shipped". **Correction:** "DevGallery is mounted in Settings → FigCraft (`SettingsPanel.tsx:540-547`) but returns `null` unless `import.meta.env.DEV` (`DevGallery.tsx:8-10`); it is bundled, not rendered, in production. The docstring is accurate; no discrepancy."

### F4 — LOW — "10 surfaces render distinct content" is 9 distinct compositions
`App.tsx:112-113` renders `RoutingPanel` for `#/workspace/routing`, and `:155-156` renders the same `RoutingPanel` for `#/routing/*`. **Correction:** "10 surface *groups*; 9 distinct panel compositions (RoutingPanel is reachable from two nav sections); 18 duplicate addresses."

### F5 — LOW — census `#/workspace/history` row omits the empty-state condition
`App.tsx:118-138`: `Canvas` renders only when `run !== null`; otherwise an explicit "Nothing to replay yet" card. Row should read "HistoryPanel + RunDetailPanel + RunTranscriptPanel + Scrubber + Canvas (only when a run snapshot exists; otherwise a named empty state)".

### F6 — HIGH — `e2e/README.md` status line is unreconcilable with the specs it describes
**Files:** `e2e/README.md:3-10` ("93 tests = 79 passed, 0 failed, 14 skipped"; "The 14 skips are explicit `test.fixme`/UNDONE (12 a11y DOM gates, 2 WS stubs)").

Measured: **81** title-string declarations in 16 specs (0 `test.fixme(`, 49 conditional `test.skip(...)` sites). The specs themselves state the opposite of the README: `a11y-gates.spec.ts:26` — "the 12 previously-fixme DOM gates now live in `specs/gui-gates.spec.ts` and run"; `ws-contract.spec.ts:8` — "The two former `test.fixme` stubs … are now"; `gui-gates.spec.ts:356-361` — three assertions that these specs **must not** carry fixme/skip. Two sites are parameterized (`control-matrix.spec.ts:74` per surface, 29 surfaces; `trace-hops.spec.ts:90` per surface), so *executed* count ≠ 81 — but "93 / 79 / 14" cannot be derived from this tree by any count I can reproduce, and "14 explicit fixme" is contradicted by zero fixmes plus a test that forbids them.
**Correction:** replace `:3-10` with a dated measured run line (`<command> → <collected/passed/skipped>`, log path that exists) or mark the status `UNVERIFIED — stale receipt from an earlier spec set`. Delete "explicit `test.fixme`"; if the 14 skips still occur at runtime they are `test.skip(appMissing(), …)` conditional skips, not fixmes.

### F7 — HIGH — `e2e/README.md:5` cites a bundle file that does not exist
It names the served artifact as `index-Cbfm1IuM.js`. `web/dist/assets/` holds `index-BNz-sfds.js` (906,532 B) and `index-Dlq9NyYb.css` (72,711 B); `Test-Path web/dist/assets/index-Cbfm1IuM.js` = **False**. The binary hash prefix `184D51A7…C9A9` **is** correct for `tools/bin/hiverouter.exe`, so the receipt is from a real run against an **older embed**. **Correction:** restate as "run against `tools/bin/hiverouter.exe` (`184D51A7…`) embedding the then-current `web/dist`; the current `web/dist` asset names are `index-BNz-sfds.js` / `index-Dlq9NyYb.css`, so re-run before quoting a bundle name."

### F8 — HIGH — `e2e/README.md:79` "ASSETS-INDEX.json still absent" is false
`docs/ui/figcraft/ASSETS-INDEX.json` exists (89,885 B), is tracked, and its sha256 `FE0F6039F29DFB84BD48E593889984680F64DAB25BE7D54556C9B1EFA22E8A5F` is recorded in `catalog.ts` `FIGCRAFT_SOURCE_MANIFESTS`. **Correction:** delete the open item; replace with "A/B manifests accepted and hashed (`A-ASSETS.json` 8883…77F1, `B-ASSETS.json` A8A8…1F4D); pixel-layout fidelity remains UNVERIFIED" — which is the claim `SOURCES.json.watchdog.non_blocking` actually supports.

### F9 — MEDIUM — `e2e/README.md:24` cites a nonexistent blueprint path
`blueprint/UI-WIRING-GUIDE.md` does not exist anywhere in the repo (recursive filename search = 0). The live source of the 28-surface/29-row matrix is `e2e/data/control-matrix.json` (**29** surfaces) plus `docs/contracts/UI-COMMANDS-v1.md` (exists). **Correction:** replace the citation; drop "from `blueprint/UI-WIRING-GUIDE.md` S01-S28".

### F10 — MEDIUM — e2e "28/28 pass" and "15/15 pass" are site counts, not executed-test receipts
`e2e/README.md:59-63` tabulates per-spec pass counts. Those specs are parameterized (`trace-hops.spec.ts:90`, `control-matrix.spec.ts:74`), so the numbers are *instances*, and no tracked run log backs them: `e2e/test-results/` exists but `test-results/run-live-FINAL-artifact.log` (cited at `:6`) is **absent**; `e2e/playwright-report/` exists. `e2e/test-results/.last-run.json` (per watchdog `L6-005`) cannot distinguish passed from zero-collected.
**Correction:** mark the per-spec table `UNVERIFIED (no tracked log)` or attach the run log; state whether counts are declarations or instances.

### F11 — MEDIUM — `e2e/README.md:67-68` reports a hop ladder that `controls.json` invalidated
README: "43 WIRED-AND-WORKS / 260 UNVERIFIED / 5 MISSING; 23 surfaces present". `docs/trace/controls.json` `_meta.counts` = **1 WIRED / 264 PENDING / 5 MISSING / 49 UNVERIFIED of 319**, and `_meta.live_capture` names the 43/260 ladder as `prior_reference`, `status: EMPTY_RAW_CAPTURE`, `invalidated_wired_hops: 43`. My independent flatten reproduces 1/264/5/49 and 29×11=319. The README's `23 PARTIALLY_VERIFIED / 6 PENDING_APP_NOT_BUILT` **are** current (`_meta.counts.controls_*`).
**Correction:** replace the ladder with 1/264/5/49 and add "the earlier 43/260 ladder was invalidated by the empty raw capture (`controls.json` `live_capture.w4_correction`)".

### F12 — MEDIUM — `dist/HiveMind/README.txt` key instruction is unexecutable from the package (extends `L7-004`)
`dist/HiveMind/README.txt:14-15`: "Create or rotate it with: `powershell -ExecutionPolicy Bypass -File tools\set-provider-key.ps1`". The zip contains exactly three members (verified above); `tools/` is not shipped. Watchdog `§3.1 #2` already corrected L7's line citation (`:18-19` → `:14-15`); this audit confirms `:14-15` is the key paragraph and that the substance stands. **Correction:** ship the script (or a `set-provider-key.cmd` shim) in the package, or rewrite `:14-15` to the in-package path; also add version + commit + "1.2.0-dev, unreleased" to `:1-3` and a `CHECKSUMS.txt` (the 1.1.0 package had one: `dist/HiveMind-1.1.0-win64/CHECKSUMS.txt` — all five listed hashes verified against the files).

### F13 — HIGH — `README.md:306`'s "577 bytes" and "Example Domain" have no receipt
Cited evidence: `docs/evidence/live/live-tokenlab-web-fetch.json` — fields are `url`, `usage`, `http_status: 200`, `approval{decision: approve, tool: web_fetch}`, `observed_at`, `tools: ["web_fetch:ok"]`, `grant`, `status`, `run_id`, `provider`, `tasks: 2`. There is **no byte count and no body/reply text**. Repo-wide, the strings `577` (as a byte claim) and `Example Domain` appear **only** at `README.md:306`. **Correction:** either attach a receipt carrying `bytes` and a `body_excerpt`/reply snippet, or rewrite to what the receipt proves: "approval-gated `web_fetch` to `https://example.com` completed with HTTP 200 and one `web_fetch:ok` tool outcome; response body size/content not captured."

### F14 — MEDIUM — `live-tokenlab-lesson-retrieval.json` note claims a field the record does not have
`note`: "Seeded authorized lesson file; final and child answers both returned HONEYCOMB." Fields present: `lesson_token`, `final_answer: "HONEYCOMB"`, `run_id`, `usage`, `status`, `provider`, `lesson_proposal_id`. No child answer field. **Correction:** either add the child's answer text or downgrade the note to "final answer returned HONEYCOMB". Same pattern check on `live-tokenlab-lesson-retrieval-parsed.json:3` (says "new parser/fence path"; record has no parser identifier) — note only.

### F15 — MEDIUM — `live-tokenlab-kill-usage.json` `hard_kill` is an empty object
`"hard_kill": {}` carries no pid, timestamp, exit code or signal. What the receipt proves: after restart, `usage_after_restart.total_tokens = 449` equals `checkpoint_before_kill.total_tokens = 449`, `recovered_status: interrupted`, `recovered_tasks: 7`. That is **checkpoint durability across a restart**, not evidence that a *hard kill* occurred. **Correction:** record `{pid, signal, killed_at, exit_code}` or retitle the key `kill_note` and state "kill mechanism not receipted".

### F16 — MEDIUM — cross-receipt contradiction on the evolution/apply state
`live-tokenlab-proposal-votes.json` (`observed_at 2026-11-10T18:37:00Z`) → `vote_path: "superagent_votes_recorded; operator_apply_not_implemented"`, `vote_state: awaiting_votes`, `votes: 0`; `live-tokenlab-proposal-dedupe.json` (18:02Z) → `vote_path: "not_implemented"`. But `live-tokenlab-auto-voters-five-seat.json` (21:56Z) → `authorize_http: 200`, `apply_http: 200`, `lesson_count_after_apply: 1`, `rollback_http: 200`, `lesson_count_after_rollback: 0`; `live-tokenlab-applied-lesson-retrieval.json` (23:14Z) → `apply_http: 200`, `lesson_count_after_apply: 1`. The receipts are unversioned point-in-time summaries: aggregating them without timestamps yields a direct contradiction (`L7 §3.3` reaches the same conclusion from code).
**Correction:** add `head_commit` (and `observed_at`) to every receipt, or ship a `docs/evidence/live/INDEX.md` that orders them and marks 18:02/18:37 as pre-apply-implementation. Do not cite the 18:xx receipts for current state.

### F17 — MEDIUM — readme `2 of 3` stability needs two receipts, and the cited one is 1 of 2
`README.md:191` says "**2 of 3 completed**". `live-tokenlab-exact-two-stability.json` is an **array of 2**: one completed 7-task (`counts` 2/2/2/1) and one `failed` with `E_PROVIDER_TIMEOUT`, `tasks: 0`, `counts: {}`. The third is `live-tokenlab-final-exact-two.json` (completed, 7 tasks, `counts` 2/2/2/1). **Correction:** cite both files for "2 of 3"; a reader who opens only `exact-two-stability.json` sees 1 of 2. The failed entry's own note ("planner provider timeout; no tasks; not a topology failure") is honest and should be preserved.

### F18 — LOW — `web/index.html` is "absent" from the lane reports as a *file*, present as prose about the router
All 7 reports contain no `web/index.html` reference; `index.html` appears as un-backticked prose in `L4` and `L6` describing `static_ui.rs` SPA fallback (`L1` mentions `index.html` not at all). The census's token-based method is therefore defensible, but its §18 §5 note ("phantom references should not be read as errors") should also say "a bare `index.html` mention is not a citation of `web/index.html`". No data change needed; add one sentence.

### F19 — HIGH (brief correction) — there are 21 FigCraft SVG boards, not 22
`web/public/figcraft/` = **22 files**: 21 `.svg` + `SOURCES.json`; `catalog.ts` has **21** board objects each with an `svgFile`; `SOURCES.json.counts.boards = 21`. The census itself says "21 served board SVGs" at `:484` but "22 served FigCraft assets" at `:520` (the 22 includes `SOURCES.json`). Any plan line that says "22 SVG boards" must be corrected to **21 boards + 1 manifest**.

### F20 — MEDIUM — census line totals for `web/src` and `web/tests` do not reproduce (file counts do)
`CODEBASE-CENSUS.md:58-59` claims `web/src` **78 files / 18,955 lines** and web tests **32 files (31 with tests) / 4,572 lines**. Measured: 78 files ✅; lines = **19,977** (Get-Content, 78 files), **19,976** (LF bytes), **18,603** (`.ts`+`.tsx` only, excluding the 4 CSS = 1,374) — 18,955 matches none. Web tests: **32 files** ✅ (31 `.ts` + `helpers/fakeUiApi.ts`), lines = **4,916** (31 test files), **5,013** (all 32) — 4,572 matches neither. **Correction:** recompute with a stated definition (e.g. "all tracked files under the directory, `Get-Content .Count`") or delete the line totals; do not publish 18,955 / 4,572 as measured.

### F21 — LOW — splash.css header claim "Every color is a FigCraft token" is overbroad
`splash.css:9-10` vs the file body: **19** `var(--color-*)` references but **33** raw `rgb(...)` literals (0 hex) — e.g. `:36-38`, `:46-54`, `:87`, `:108-111`, `:146-149`, `:163`, `:198`, `:224-226`, `:252`. The literals are alpha/glass variants of token hues (`rgb(240 192 80 / .34)` = honey-500 at 34%), so the palette is token-derived, but the sentence as written is false and no test catches it: `web/tests/noHardcoded.test.ts:26` bans only 6-digit **hex** (`/#[0-9a-fA-F]{6}\b/`), so `rgb()` literals pass. **Correction:** either state "token palette; alpha variants written as literal `rgb()` because no alpha token exists", or extend the guard to reject non-tokenized `rgb(`/`oklch(` outside `styles/tokens.css`.

### F22 — LOW — mock timers are not cancelled on `close()`
`mock.ts:163` and `:183` schedule `globalThis.setTimeout` callbacks; `close()` (`:83-85`) only emits a status. A dev-mode unmount can still deliver `REPLY_DELTA`/`APPROVAL_REQUEST` into a disposed store. Dev-only, so severity is LOW (AGENTS §26.3 resource release), but the file's own claim "so the UI is exercised end-to-end" should not ship with an uncancelled timer path. **Correction:** track timer ids and clear them in `close()`.

### F23 — LOW — `catalog.ts` fallback group is unreachable and would be unsafe if reached
`FigCraftGallery.tsx:22-26` fabricates a `FIRST_GROUP` when `FIGCRAFT_GROUPS` is empty, while `:241` computes `count = FIGCRAFT_GROUPS.length` and `:244` does `% count` (0 → `NaN` index). Today `FIGCRAFT_GROUPS` is a 3-element const (`catalog.ts:13-29`), so the path is dead. **Correction:** delete the fallback and guard `FIGCRAFT_GROUPS.length === 0` before the tablist (the empty-catalog message at `:257-264` already covers the real case).

### F24 — LOW — `.gitignore` contradicts the tracked FigCraft asset set
`.gitignore:8-9` ignore `docs/ui/figcraft/ASSETS/*.png` and `*.svg`; `:12` un-ignores only `hivemind-*`. All **21** source SVGs are tracked today (added before/around the rule), so nothing is lost, but a **new** `a-*`/`b-*` board would be silently ignored and `git add -f` would be required. **Correction:** add `!docs/ui/figcraft/ASSETS/[ab]-*.svg` (and the PNG equivalent if those are meant to be tracked) or delete the two ignore lines.

### F25 — MEDIUM — `tools/release-manifest.json` records a `web/dist` tree that no longer exists on disk, and a provider token ceiling the harness marks unverified
`release-manifest.json` `web.web_dist_files` = `index-DRvC4Qb-.js` (675,204 B) + `index-D8k8-PfA.css` (35,807 B) + `index.html` (774 B, hash `01F31347…`). On disk: `index-BNz-sfds.js` (906,532 B) + `index-Dlq9NyYb.css` (72,711 B) + `index.html` (774 B). This is independent corroboration of watchdog `§5 #10` (rehearsal G7 mismatch) with concrete numbers. Separately, `provider.max_output_tokens: 393216` is recorded as sealed metadata with source string "official DeepSeek docs (api-docs.deepseek.com)" but no URL/version, while the operating contract's claims register marks **393,216 UNVERIFIED** (`~/.dsh/AGENTS.md §13, §21`). Also `provider.models = ["deepseek-flash","deepseek-v4-pro"]` does not include the receipts' `wire_model: "DeepSeek-V4.1-Flash"` (`live-tokenlab-hierarchy.json`, `live-tokenlab-simple.json`).
**Correction:** re-seal the manifest against a fresh build or add `"stale_web_dist": true` + the measured hashes; cite a versioned URL for the token ceiling or drop the field; reconcile the model list with the wire model the provider actually reports.

### F26 — LOW — CODEMAP's "805 tracked files" is exact **for its freeze commit**; the census's 805-vs-1,573 discrepancy is snapshot-vs-current, not an error
**Files:** `docs/CODEMAP/00-START-HERE.md:3` ("Frozen at commit `e021b12e5bb7`"), `:11` ("Git-tracked project files **805**"), `:17`; `docs/CODEMAP/01-SCOPE-AND-CENSUS.md:8-9,15`; census framing at `CODEBASE-CENSUS.md:539`.

Verified: `git rev-parse e021b12e5bb7` resolves; `git ls-tree -r --name-only e021b12e5bb7` returns **805** — the doc's number is exact for that commit. HEAD `6a41250` tracks **1,573**. The doc prints its freeze label one line above the table (`:3`), so the census's "claims 805 tracked files" is a fair reading of the table but an unfair verdict on the document. **Correction:** do not rewrite 805; add a "superseded snapshot" banner naming HEAD and 1,573, and register CODEMAP with the other dated snapshots (L7 §3.7).

### F27 — MEDIUM (plan hazard) — the workspace is a **shallow clone**, so git ancestry/`describe` evidence cannot support release-lineage reasoning
**Evidence:** `git rev-parse --is-shallow-repository` = **true**. `git describe --always --tags --dirty` = `6a41250` (no tag reachable); `git merge-base 87079ce… HEAD` returns nothing and `--is-ancestor` fails for both `87079ce` (`hivemind-1.1.0` tag target per `tools/release-manifest.json`) and `e021b12` (CODEMAP freeze); `git rev-list --max-parents=0` shows two different graft points (`aa16e13` on the HEAD line, `d1638f2` on the tag line); counts 363 vs 40.

**This is not evidence that the sealed 1.1.0 is unrelated to HEAD** — it is what a shallow clone with two fetch boundaries looks like. The defensible statements are: this clone **cannot answer** tag-ancestry questions, and `git describe` here is not evidence that HEAD is untagged. **Correction:** any finish-plan gate that reasons about "commits behind the tag", ancestry, or "history rewritten" must either `git fetch --unshallow` (network + write, out of scope for this lane) or use the sealed manifest record (`tag_target: 87079ce…`, `source_commit: 87079ce…`, `tag_behind_commits: 0`) and say that is the basis. Do not publish a lineage claim from this clone.

---

## 4. Live evidence field-level audit (`docs/evidence/live/*.json`, 26 JSON + 11 PNG)

This closes watchdog `§4.4` / `L7-009` ("filed count confirmed; per-file field audit not performed").

| Receipt | Fields that actually prove something | What it does **not** prove | Grade |
|---|---|---|---|
| `live-tokenlab-simple.json` | `answer:"hello hive"`, `tasks:2`, `task_roles:[planner,superagent]`, usage 1701 | model identity beyond the provider-reported `wire_model`; billing (`billing:"external_unverified"`) | READ (single synthetic run) |
| `live-tokenlab-hierarchy.json` | 7 tasks, `hierarchy` 2/2/2/1, `task_states.completed:7`, `wall_seconds:106` | that the topology was *planned* that way (post-hoc counts); model identity | READ |
| `live-tokenlab-web-fetch.json` | `http_status:200`, `approval.decision:approve`, `tools:["web_fetch:ok"]` | **byte count, body text, reply content, prompt** (F13) | READ |
| `live-tokenlab-web-fetch-killswitch.json` | `fetched:false`, `tool_outcome:"refused"`, `code:"E_TOOL_NOT_ALLOWED"`, `side_effects:"0"` | that the same call would have fetched without the switch (no paired raw frame) | READ — claim supported |
| `live-tokenlab-file-write-approval.json` | `artifact.content:"APPROVED_WRITE_OK"`, `artifact.exists:true`, `approval_id`, `decision:approve`; honest `usage:null` + note | artifact sha256, workspace path ("temp isolated workspace" is self-description), scope of the jail | READ/READ |
| `live-tokenlab-shell-exec.json` | `program:"cmd.exe"`, `args:["/c","echo shell-ok"]`, `stdout:"shell-ok"`, `exit_code:0` | that the allowlist was consulted (no allowlist field); jail scope | READ |
| `live-tokenlab-a2a.json` | `a2a.status:"answered"`, `attempts:1`, `answer_actor:"parent:<task>"`, answer text | answer latency/deadline behaviour (no timestamps per attempt) | READ |
| `live-tokenlab-remote-daemon.json` | `remote.transport_kind:"in_process"`, `state:"enrolled"`, `daemon.job.check:"db"`, `created_by:"hyper root-control"` | any daemon **tick** (no tick row/timestamp) — README:27 correctly says no live tick proof | READ |
| `live-tokenlab-request-id.json` / `-request-id-final.json` | `audit` rows incl. `remote_created:request_id=…`, `created_by:"agent:<task>"`, `grant_count:1`, `device_count:1` | that a *second* identical request was replayed idempotently (only one attempt) | READ |
| `live-tokenlab-detached-request.json` | admission audit rows `…:system:agent:allow`, remote state | absence of a conflicting second row (no negative case) | READ |
| `live-tokenlab-idempotent-attempt.json` | honest failure: `status:"running_when_stopped"`, `usage:null`, `remotes:0`, `E_TOOL_TIMEOUT` on `ask_parent`, `finding` says the idempotency path was **not** exercised | anything about idempotency | READ — **negative** receipt, do not cite as idempotency proof |
| `live-tokenlab-exact-two-stability.json` | array of 2: one 7-task completed; one `failed` `E_PROVIDER_TIMEOUT`, `tasks:0` | "2 of 3" without `final-exact-two` (F17) | READ |
| `live-tokenlab-final-exact-two.json` | third attempt: completed, 7 tasks, counts 2/2/2/1 | ordering vs the stability array (no shared timestamp) | READ |
| `live-tokenlab-kill-usage.json` | `usage_after_restart` == `checkpoint_before_kill` (449), `recovered_status:"interrupted"`, `recovered_tasks:7` | that a hard kill happened (`hard_kill:{}`, F15) | READ |
| `live-tokenlab-rate-limit.json` | `provider_call_made:false`, `E_LLM_RATE_LIMIT`, `tasks:0`, `observed_wall_seconds:0` | the embedded claim "retry after one window succeeded" (no second receipt in the file) | READ |
| `live-tokenlab-proposal-votes.json` | `proposal_count:1`, `applied:false`, `votes:0`, `vote_state:"awaiting_votes"` | current state (stale `vote_path`, F16) | READ — historical only |
| `live-tokenlab-proposal-dedupe.json` | `proposal_count:1` (dedupe), `governance` block 5-seat/`can_apply:false` | current state (`vote_path:"not_implemented"` is stale, F16) | READ — historical only |
| `live-tokenlab-authorize-refusal.json` | `authorize_http:409`, `superagent_approvals:0` | that four approvals would have succeeded (different receipt does that) | READ |
| `live-tokenlab-lesson-apply-refusal.json` | `apply_http:409`, `authorize_state:"not_authorized"` | no lesson write — only stated in `note`, no ledger count field | READ |
| `live-tokenlab-lessons-empty.json` | `count:0`, `endpoint:"/api/evolution/lessons"`, `state:"read_only"` | that the endpoint rejects writes (no write attempt) | READ |
| `live-tokenlab-auto-voters-five-seat.json` | `auto_spawned_voters:4`, `superagent_approvals:4`, `authorize_http:200`, `apply_http:200`, `lesson_count_after_apply:1`, `rollback_http:200`, `lesson_count_after_rollback:0` | that the voters were distinct *reasoning* agents (only a count), code application | READ — strongest evolution receipt |
| `live-tokenlab-voter-dissent.json` | `rejections:1`, `approvals:1`, `authorize_http:409`, `apply_http:409`, `vote_state:"rejected_by_superagent"` | that the two non-voters were asked | READ |
| `live-tokenlab-applied-lesson-retrieval.json` | `apply_http:200`, `lesson_count_after_apply:1`, `retrieval_answer` quotes the lesson text, `usage_apply`/`usage_retrieval` | that the *prompt* contained the lesson block and its sha256 (no prompt/frame); scope limited to `apply_scope:"lesson_artifact_only"` | READ |
| `live-tokenlab-lesson-retrieval.json` / `-parsed.json` | `lesson_token:"HONEYCOMB"`, `final_answer:"HONEYCOMB"` | the child answer (F14); the "parser/fence path" identity | READ |
| 11 `ui-*.png` | 1440×900-class UI screenshots referenced by README §5 | that the UI is the current HEAD build (no hash/commit in the PNGs) | READ |

**Aggregate finding (EV-A):** every receipt is a *summary written by the harness after the run*. None contains a raw frame, prompt, HTTP body or log line. They therefore support **state claims** ("a run completed with N tasks and this usage") and cannot support **content claims** (bytes, text, ordering, kill mechanics). This is consistent with `L7 §3.9` and the watchdog's `L7-009`; it should be stated once, at the top of `docs/evidence/live/`, instead of being re-litigated per claim.

---

## 5. The 25 absent paths — independent re-derivation

| Check | Result |
|---|---|
| Count in MD (`:480-484`) = count in JSON `omissions.absent_from_all_lane_reports` | **25 = 25** |
| All 25 tracked (`git ls-files`) | ✅ (`.gitignore`, `web/index.html`, `web/package-lock.json`, 22 figcraft files) |
| Literal file reference in any of the 7 reports | **0** for all 25 (my substring scan over 506,864 chars) |
| Composition | 21 SVG boards + `SOURCES.json` + `web/index.html` + `web/package-lock.json` + `.gitignore` |
| Ownership recorded | `.gitignore` → L7 (`root/repo`), `web/index.html` + `web/package-lock.json` → L5 (`web/build`), figcraft → L5 — matches JSON `index[].lane` |
| Product impact | `static_ui.rs:13-16` embeds `web/dist`; `web/dist` carries `index.html` + `figcraft/` (22 files) → all 22 figcraft files are served bytes; `web/index.html` is the Vite entry; `web/package-lock.json` is the pinned tree; `.gitignore:3` is why a stale `web/dist` can ship |

Corrections to the brief, not the census: **21 SVG boards**, not 22 (F19); and `web/index.html`'s only textual mentions are prose about the SPA fallback (F18).

---

## 6. Owner / feature-map gaps (census §15.4 + `features`)

| Gap | Detail | Correction |
|---|---|---|
| Total arithmetic | rows sum to 249, total says 216 (F1) | split the table into 216 / +33 / +82 |
| Prompt assets | MD row "Prompt assets 10" has **no JSON family** (`features` keys = http, ws, tools, persistence, refusals, personas, ui, caps, missing, absent_by_design) | add `features.prompts` with the 10 rows, or drop the row |
| Refusals 78 | enumerated in JSON with lane L4, absent from §15.4 and from the 216 | add a row `Refusal variants 78 — refusal.rs — L4 — 0 missing` outside the 216 |
| Caps 4 | enumerated in JSON (`features.caps`), absent from §15.4 | add row or note "caps are constants, not features" |
| Migrations 23 | row inside a 216 it is not part of | move to "additional enumerated" |
| 3 MISSING rows | correct, but the decode-refusal tests are not cited (F2) | add `ui_command_matrix.rs:113`, `common/mod.rs:411-427` |
| `APPROVAL_HYDRATE` | JSON `absent_by_design` has owner `ws.rs`, status `ABSENT-BY-DESIGN`; MD `:400` says "ABSENT BY DESIGN" ✅ | none |
| UI surfaces | 28 count correct; "10 distinct" → 9 compositions (F4) | wording |
| `web/index.html` / lockfile features | `web:build:index`, `web:build:package-lock` — plausible | none |

---

## 7. Residual unknowns (stated, not passed)

1. **No runtime observation.** The 28 hash surfaces, splash timing, gallery keyboard behaviour and mock banner are read from source. Only a browser run can confirm them; none was performed (read-only mandate).
2. **No tests executed.** `apiBoundary.test.ts` (which pins the mock boundary) was read, not run; `web/tests` count and `noHardcoded` guard are static reads.
3. **Provider identity.** `wire_model: "DeepSeek-V4.1-Flash"` and `endpoint https://api.tokenlab.amd.com/v1` are provider-reported; verifying them requires a provider call, which is out of scope. `provider_call_made:false` receipts are trusted only as summaries.
4. **E2E executed counts.** The 81 declarations are static; the executed count depends on `e2e/data/control-matrix.json` (29 surfaces) and skip conditions. No run log exists at the cited path.
5. **`web/dist` freshness.** The bundle scanned is the on-disk build; whether `dist/HiveMind/hiverouter.exe` embeds *this* bundle cannot be established without running/strings-diffing a 36 MB binary (not done).
6. **Census `report_mentions`/`report_scopes` per path** were not re-derived path-by-path; the absent set and the tier arithmetic were, and both reproduce.
7. **Git lineage.** The clone is shallow (F27); whether `hivemind-1.1.0` is an ancestor of HEAD is unresolved here and must not be asserted either way from this workspace. Closing it needs `git fetch --unshallow` or the upstream remote.
8. **CODEMAP body.** Only the freeze line and the 805 table were checked (F26); the encyclopedia's 378-file appendix was not re-read (watchdog `§0.3` already excludes that long tail).

---

## 8. Exact corrections checklist (for the Lead)

| # | File:line | Replace with |
|---|---|---|
| 1 | `CODEBASE-CENSUS.md:502` | `**Total**` → split: `216 (8 owned families)` + `33 additional enumerated (23 migrations + 10 prompt assets)`; add refusal 78 / caps 4 rows outside the 216 |
| 2 | `CODEBASE-CENSUS.md:23` | "216 / 216" → "216 owned-family features / 216; 3 explicit MISSING (`DIFF_*`); 78 refusals + 4 caps enumerated separately" |
| 3 | `CODEBASE-CENSUS.md:379` | delete "i.e. shipped … discrepancy"; use F3 text |
| 4 | `CODEBASE-CENSUS.md:520` | "22 served FigCraft assets" → "21 served SVG boards + `SOURCES.json`" |
| 5 | `CODEBASE-CENSUS.md:362` | "10 surfaces render distinct content" → "10 surface groups; 9 distinct panel compositions; 18 duplicate addresses" |
| 6 | `CODEBASE-CENSUS.md:58-59` | recompute or delete `18,955` / `4,572` (measured 19,977 / 4,916) |
| 7 | `CODEBASE-CENSUS.md:397-399` | append the two decode-refusal test citations |
| 8 | `e2e/README.md:3-10` | measured run line or `UNVERIFIED — stale`; delete "explicit test.fixme" |
| 9 | `e2e/README.md:5` | current bundle names or "embedded dist at run time" |
| 10 | `e2e/README.md:24` | `e2e/data/control-matrix.json` + `docs/contracts/UI-COMMANDS-v1.md` |
| 11 | `e2e/README.md:59-63` | mark per-spec pass counts UNVERIFIED (no tracked log) |
| 12 | `e2e/README.md:67-68` | 1 WIRED / 264 PENDING / 5 MISSING / 49 UNVERIFIED + invalidation note |
| 13 | `e2e/README.md:79` | delete (ASSETS-INDEX exists, hash recorded) |
| 14 | `README.md:306` | drop "577 bytes"/"Example Domain" or attach a receipt with `bytes` + excerpt |
| 15 | `README.md:191` | cite both exact-two receipts for "2 of 3" |
| 16 | `dist/HiveMind/README.txt:1-3` | add "1.2.0-dev, unreleased, not the sealed 1.1.0" |
| 17 | `dist/HiveMind/README.txt:14-15` | ship the script or point at an in-package path |
| 18 | `web/src/styles/splash.css:9-10` | "token palette; alpha variants are literal `rgb()`" (or extend `noHardcoded.test.ts:26`) |
| 19 | `docs/evidence/live/` | add `INDEX.md` with `head_commit` + ordering; add `bytes`/`body_excerpt` where content is claimed |
| 20 | `.gitignore:8-12` | add `!docs/ui/figcraft/ASSETS/[ab]-*.svg` or delete the ignore block |
| 21 | `docs/CODEMAP/00-START-HERE.md:3,11` | keep 805; add "superseded snapshot; HEAD `6a41250` tracks 1,573" |
| 22 | finish-plan gates using `git describe`/`merge-base` | use the sealed manifest fields; note the clone is **shallow** (`git rev-parse --is-shallow-repository` = true) |

*End of audit. No repository file other than this one and `GAP-PATCH-2.md` was written.*
