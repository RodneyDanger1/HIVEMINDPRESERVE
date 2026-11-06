# FILES — watchdog-2 partition (122 tracked files; e2e/**, tools/**, runtime-test/**)

Read-only audit at HEAD `3988622` (wave sheet) — manifest `audit/inventory/FILE-MANIFEST.csv`, assignments `audit/inventory/ASSIGNMENTS.json` key `watchdog-2`.
Deep files: 4 (e2e/playwright.config.ts, e2e/specs/live-app.spec.ts, e2e/specs/trace-hops.spec.ts, tools/qa/verify-harness.mjs). Standard: 118. All metadata below is measured: bytes/sha256 re-computed from disk and compared to the manifest (matches: 122/122).

## Partition summary

- languages: {"unknown":1,"Markdown":10,"JavaScript":10,"JSON":7,"TypeScript":12,"SQLite":13,"SQLite-SHM":13,"SQLite-WAL":13,"Text":16,"PE-binary":8,"Log":12,"HTML":1,"PowerShell":3,"Python":1,"Rust":1,"C":1}
- categories: {"other":4,"source":28,"config-manifest":4,"runtime-artifact":82,"generated-artifact":4}
- depth: {"standard":118,"deep":4}
- git-tracked: 122/122; manifest-hash matches: 122/122; untracked: []
- questionable flags: {"skipped-test":6,"mock-data":1,"hardcoded-host":29,"console.log":10,"ad-hoc-probe":8,"generated-marker":7,"fs-write":6,"process.env":5,"runtime-artifact":27,"secret-ish":4}

---

## e2e/.gitignore

**Metadata:** 125 B · sha256 `EC621CAF388828C9A865137362779A41E7DCEB44D365D71A1DB6C974BC6DB333` (manifest MATCH) · unknown · category `other` · depth `standard` · git-tracked: yes.

**Purpose:** Ad-hoc Playwright probe script (exploration/smoke/screenshot), not wired into the test runner.

**Mechanics:** Starts installed Chrome headless and prints/records live-app state; overlaps specs in purpose.

**Dependencies/callers:** no in-partition imports; external: none detected.

**Quality (1–5):** correctness=4, maintainability=4, modularity=4, error_handling=4, security=4, docs=3, evidence=4. Flags: none.

**Evidence/UNVERIFIED:** read 7 lines.

---

## e2e/data-probe.mjs

**Metadata:** 1022 B · sha256 `E0E5A045390F6D6D5E5CA41DE83BA16C16325A8B5A01DAF9FFFB266326C336C6` (manifest MATCH) · JavaScript · category `source` · depth `standard` · git-tracked: yes.

**Purpose:** Ad-hoc Playwright probe script (exploration/smoke/screenshot), not wired into the test runner.

**Mechanics:** Starts installed Chrome headless and prints/records live-app state; overlaps specs in purpose.

**Dependencies/callers:** no in-partition imports; external: `@playwright/test`.

**Quality (1–5):** correctness=2, maintainability=2, modularity=2, error_handling=2, security=3, docs=1, evidence=2. Flags: console.log, hardcoded-host, ad-hoc-probe.

**Evidence/UNVERIFIED:** read 18 lines; flagged at console.log:16, hardcoded-host:9.

---

## e2e/data/control-matrix.json

**Metadata:** 14979 B · sha256 `D42E3342DC3175B8B7B73B67F6D3D039FD6C8091C0BED94767F2FDD10CA99375` (manifest MATCH) · JSON · category `config-manifest` · depth `standard` · git-tracked: yes.

**Purpose:** Machine-readable control matrix: 28 guide surfaces + S-APPROVAL supplemental control × frozen commands/events/refusals/ACK rules.

**Mechanics:** JSON consumed by helpers/matrix.ts and specs; validated statically by tools/qa/verify-harness.mjs.

**Dependencies/callers:** no in-partition imports; external: none detected.

**Quality (1–5):** correctness=4, maintainability=4, modularity=4, error_handling=4, security=4, docs=3, evidence=4. Flags: generated-marker.

**Evidence/UNVERIFIED:** read 67 lines; flagged at generated-marker:18.

---

## e2e/helpers/app.ts

**Metadata:** 2044 B · sha256 `CB9AE9F946471B6FA443CD22AD31E618127EFBAD7BD5070AC7E8A2FF1F798F8C` (manifest MATCH) · TypeScript · category `source` · depth `standard` · git-tracked: yes.

**Purpose:** HTTP probe helper for /health and /api/session reachability.

**Mechanics:** Exports probeApp()/wsUrl used by WS contract spec.

**Dependencies/callers:** no in-partition imports; external: `@playwright/test`.

**Quality (1–5):** correctness=5, maintainability=4, modularity=4, error_handling=4, security=4, docs=4, evidence=4. Flags: none.

**Evidence/UNVERIFIED:** read 63 lines.

---

## e2e/helpers/contrast.ts

**Metadata:** 2214 B · sha256 `646ED79DF15D2A303933194C4C8D9F51FFDD11D9B4FCCC1DCCCEC10C5ED0726A` (manifest MATCH) · TypeScript · category `source` · depth `standard` · git-tracked: yes.

**Purpose:** WCAG 2.1 relative-luminance contrast math and expected token pairs used by a11y/live tests.

**Mechanics:** Exports contrastRatio(), EXPECTED_CONTRAST and KNOWN_FAILURES; no DOM or network access.

**Dependencies/callers:** no in-partition imports; external: none detected.

**Quality (1–5):** correctness=5, maintainability=4, modularity=4, error_handling=4, security=4, docs=4, evidence=4. Flags: none.

**Evidence/UNVERIFIED:** read 49 lines.

---

## e2e/helpers/evidence.ts

**Metadata:** 2921 B · sha256 `066430F0A36E9F9A63B555AD9858A3FFCE108A068CCB8CEBF6AC7CF7C672E482` (manifest MATCH) · TypeScript · category `source` · depth `standard` · git-tracked: yes.

**Purpose:** Evidence capture helper: captures WS frames, waits for frames, writes JSON evidence/screenshots into docs/trace/live.

**Mechanics:** Exports captureFrames/parseFrames/waitForFrame/writeEvidence/shot and the LIVE_APP_DIR constants that specs use.

**Dependencies/callers:** no in-partition imports; external: `node:fs`, `node:url`, `@playwright/test`.

**Quality (1–5):** correctness=5, maintainability=4, modularity=4, error_handling=4, security=4, docs=4, evidence=4. Flags: fs-write.

**Evidence/UNVERIFIED:** read 83 lines; flagged at fs-write:1.

---

## e2e/helpers/matrix.ts

**Metadata:** 2004 B · sha256 `BC101B3505E1F98BB258CCA037790A6B660C4876316B60D73778870849C0D472` (manifest MATCH) · TypeScript · category `source` · depth `standard` · git-tracked: yes.

**Purpose:** Loads and types the 28-surface control matrix plus frozen-command vocabulary/frozen-command coverage helper.

**Mechanics:** Exports Surface types, loadMatrix(), appMissing() skip gate and FROZEN_COMMANDS list used by specs.

**Dependencies/callers:** no in-partition imports; external: `node:fs`, `node:url`.

**Quality (1–5):** correctness=5, maintainability=4, modularity=4, error_handling=4, security=4, docs=4, evidence=4. Flags: process.env.

**Evidence/UNVERIFIED:** read 74 lines; flagged at process.env:44.

---

## e2e/helpers/trace.ts

**Metadata:** 1299 B · sha256 `404CE7202A8C8A5557C88CA7F132D2C3C4CF8F05468F3F211A5E7F5396664E97` (manifest MATCH) · TypeScript · category `source` · depth `standard` · git-tracked: yes.

**Purpose:** Append-only trace-run writer for docs/trace/traces.json.

**Mechanics:** Exports appendTrace(run) which serialises captured 11-hop evidence.

**Dependencies/callers:** no in-partition imports; external: `node:fs`, `node:url`.

**Quality (1–5):** correctness=5, maintainability=4, modularity=4, error_handling=4, security=4, docs=4, evidence=4. Flags: fs-write.

**Evidence/UNVERIFIED:** read 38 lines; flagged at fs-write:1.

---

## e2e/helpers/ws-tap.ts

**Metadata:** 1790 B · sha256 `4E667F62D942B736C4F452B5A01F86B0DF21AFA49F36FF54E1F1E6FE3E1069F8` (manifest MATCH) · TypeScript · category `source` · depth `standard` · git-tracked: yes.

**Purpose:** Small WebSocket capture helper used by the WS contract spec.

**Mechanics:** Collects sockets/opening frames; close() releases listeners.

**Dependencies/callers:** no in-partition imports; external: `@playwright/test`.

**Quality (1–5):** correctness=5, maintainability=4, modularity=4, error_handling=4, security=4, docs=4, evidence=4. Flags: none.

**Evidence/UNVERIFIED:** read 61 lines.

---

## e2e/live-check2.mjs

**Metadata:** 712 B · sha256 `042EACC4F2919579B83E6A509CFA5B53321A9A5E8C0ED40B346101882F1C4E78` (manifest MATCH) · JavaScript · category `source` · depth `standard` · git-tracked: yes.

**Purpose:** Ad-hoc Playwright probe script (exploration/smoke/screenshot), not wired into the test runner.

**Mechanics:** Starts installed Chrome headless and prints/records live-app state; overlaps specs in purpose.

**Dependencies/callers:** no in-partition imports; external: `@playwright/test`.

**Quality (1–5):** correctness=2, maintainability=2, modularity=2, error_handling=2, security=3, docs=1, evidence=2. Flags: console.log, hardcoded-host, ad-hoc-probe.

**Evidence/UNVERIFIED:** read 12 lines; flagged at console.log:10, hardcoded-host:8.

---

## e2e/live-inspect.mjs

**Metadata:** 6568 B · sha256 `DDD46289FF4EB7601EBC1222CD92C8633D74F81E6AC403C4E4A6946117F0D74A` (manifest MATCH) · JavaScript · category `source` · depth `standard` · git-tracked: yes.

**Purpose:** Ad-hoc Playwright probe script (exploration/smoke/screenshot), not wired into the test runner.

**Mechanics:** Starts installed Chrome headless and prints/records live-app state; overlaps specs in purpose.

**Dependencies/callers:** no in-partition imports; external: `@playwright/test`, `node:fs`, `node:url`.

**Quality (1–5):** correctness=2, maintainability=2, modularity=2, error_handling=2, security=3, docs=1, evidence=2. Flags: console.log, process.env, hardcoded-host, fs-write, generated-marker, ad-hoc-probe.

**Evidence/UNVERIFIED:** read 128 lines; flagged at console.log:102, process.env:9, hardcoded-host:9, fs-write:6.

---

## e2e/live-probe.mjs

**Metadata:** 1019 B · sha256 `6C90EE5C0B7FDA7C179ADFB95EE6C55A9D67158BD7ECD77998291C54628AE285` (manifest MATCH) · JavaScript · category `source` · depth `standard` · git-tracked: yes.

**Purpose:** Ad-hoc Playwright probe script (exploration/smoke/screenshot), not wired into the test runner.

**Mechanics:** Starts installed Chrome headless and prints/records live-app state; overlaps specs in purpose.

**Dependencies/callers:** no in-partition imports; external: `@playwright/test`.

**Quality (1–5):** correctness=2, maintainability=2, modularity=2, error_handling=2, security=3, docs=1, evidence=2. Flags: console.log, hardcoded-host, ad-hoc-probe.

**Evidence/UNVERIFIED:** read 17 lines; flagged at console.log:8, hardcoded-host:7.

---

## e2e/live-shot.mjs

**Metadata:** 529 B · sha256 `349E459DBFB88EC740A695F652793AAACD26E53B3EF0CF467B90C86540DA1A0C` (manifest MATCH) · JavaScript · category `source` · depth `standard` · git-tracked: yes.

**Purpose:** Ad-hoc Playwright probe script (exploration/smoke/screenshot), not wired into the test runner.

**Mechanics:** Starts installed Chrome headless and prints/records live-app state; overlaps specs in purpose.

**Dependencies/callers:** no in-partition imports; external: `@playwright/test`.

**Quality (1–5):** correctness=2, maintainability=2, modularity=2, error_handling=2, security=3, docs=1, evidence=2. Flags: console.log, hardcoded-host, ad-hoc-probe.

**Evidence/UNVERIFIED:** read 9 lines; flagged at console.log:7, hardcoded-host:4.

---

## e2e/live-smoke.mjs

**Metadata:** 1064 B · sha256 `50E507618D831B6C2B4B6295E71BF3730A61746DF05F3A8404314674CEE20259` (manifest MATCH) · JavaScript · category `source` · depth `standard` · git-tracked: yes.

**Purpose:** Ad-hoc Playwright probe script (exploration/smoke/screenshot), not wired into the test runner.

**Mechanics:** Starts installed Chrome headless and prints/records live-app state; overlaps specs in purpose.

**Dependencies/callers:** no in-partition imports; external: `@playwright/test`.

**Quality (1–5):** correctness=2, maintainability=2, modularity=2, error_handling=2, security=3, docs=1, evidence=2. Flags: console.log, hardcoded-host, ad-hoc-probe.

**Evidence/UNVERIFIED:** read 20 lines; flagged at console.log:18, hardcoded-host:7.

---

## e2e/package-lock.json

**Metadata:** 1748 B · sha256 `883C7A729F40D01BA20B7C2FF3AF9D787B59FC070094F6CFE5C47F3A0EB3CB68` (manifest MATCH) · JSON · category `config-manifest` · depth `standard` · git-tracked: yes.

**Purpose:** Ad-hoc Playwright probe script (exploration/smoke/screenshot), not wired into the test runner.

**Mechanics:** Starts installed Chrome headless and prints/records live-app state; overlaps specs in purpose.

**Dependencies/callers:** no in-partition imports; external: none detected.

**Quality (1–5):** correctness=4, maintainability=4, modularity=4, error_handling=4, security=4, docs=3, evidence=4. Flags: none.

**Evidence/UNVERIFIED:** read 64 lines.

---

## e2e/package.json

**Metadata:** 659 B · sha256 `F5D3D35E1C3F4FCC395F2849E3F1F073EA4CE11BAF071CDB1F787122863C0A28` (manifest MATCH) · JSON · category `config-manifest` · depth `standard` · git-tracked: yes.

**Purpose:** Ad-hoc Playwright probe script (exploration/smoke/screenshot), not wired into the test runner.

**Mechanics:** Starts installed Chrome headless and prints/records live-app state; overlaps specs in purpose.

**Dependencies/callers:** no in-partition imports; external: none detected.

**Quality (1–5):** correctness=4, maintainability=4, modularity=4, error_handling=4, security=4, docs=3, evidence=4. Flags: none.

**Evidence/UNVERIFIED:** read 22 lines.

---

## e2e/playwright.config.ts

**Metadata:** 1917 B · sha256 `E414A391049EB04C8A2865188C9457185FF525919BCF19FF4E5C83EDB1EA3698` (manifest MATCH) · TypeScript · category `source` · depth `deep` · git-tracked: yes.

**Purpose:** Central Playwright configuration for the HiveMind QA harness (installed-browser channels only; no browser download).

**Mechanics:** Defines testDir, outputDir, 1 worker, 0 retries, 30 s timeout, list+HTML reporters, headless Chrome/Edge projects and the env-driven baseURL.

**Dependencies/callers:** no in-partition imports; external: `@playwright/test`.

**Quality (1–5):** correctness=5, maintainability=4, modularity=4, error_handling=4, security=3, docs=4, evidence=5. Flags: process.env, hardcoded-host.

**Deep analysis (8–12 paragraphs):**

1. Functional mechanics: the file is a single defineConfig() call (lines 19-46) that hands Playwright the test directory, output directory, timeout/expect budgets, worker/retry policy, reporters, and the default use{} block. There is no conditional logic; every knob is a literal except the env-driven BASE_URL on line 17. The chrome/msedge projects (lines 41-46) switch only the launch channel, so both share the same record/trace policy.

2. Browser policy: lines 2-8 are an explicit, auditable contract: launch the host-installed Chrome (Channel chrome) or Edge (msedge), never download a browser. This matches the shipped-testing constraint and is stronger than an npm-level pin because it makes the browser a host capability rather than a fetched dependency. The weakness is that the exact host versions live only in comments, so reproducibility is asserted by prose rather than by a captured version value.

3. Server coupling: lines 10-14 document that the release binary serves the embedded UI on 127.0.0.1:8799 and that webServer is intentionally absent. Consequently the suite cannot be self-contained: it fails (or skips) if the Lead has not started the artifact, and it tests whatever process is listening. The config does not verify a binary hash, route count, or /health version, so a stale process would be exercised and reported as the shipped app.

4. Base URL drift: line 17 defaults to http://127.0.0.1:8080, but every captured live run in this repository used http://127.0.0.1:8799 via E2E_BASE_URL (live-app.spec.ts:6, trace-hops.spec.ts:14). A bare invocation without env therefore targets a different port and a different (likely absent) server. The fallback should either be the shipped port or omitted so the missing env fails loudly instead of silently pointing elsewhere.

5. Determinism: workers=1 (line 25) serialises the suite so WebSocket frame ordering used by trace evidence is stable; retries=0 (line 26) and forbidOnly=true (line 27) prevent flake-hiding and accidental .only commits. The trade-off is runtime: the final 93-test run took 3.6 minutes. This is an intentional correctness-over-speed choice and is documented in the comment on line 25.

6. Timeout/expectation budget: timeout 30_000 ms per test and expect timeout 5_000 ms (lines 22-23) are compatible with the observed networkidle loads (~1-2 s) but tight for a slow cold start; a slow server boot would surface as a named timeout rather than a false pass. Since tests always wait for concrete locators/frames, 5 s expect is adequate for the current UI.

7. Artifact/report policy: outputDir ./test-results and the HTML reporter folder playwright-report (lines 21, 28-31) are generated and untracked; trace retain-on-failure and screenshot only-on-failure (lines 35-36) mean passing runs leave no screenshot evidence, only the JSON files written explicitly by specs. A consequence (observed at closeout) is that anything written into test-results is deleted by the next run — durable logs must live outside outputDir.

8. Execution isolation: there is no storageState, no auth bypass and no route interception (lines 39-40); the app's own /api/session token flow is exercised end-to-end, which is the right boundary for a WS/auth trace. acceptDownloads=false and video=off keep artifacts small. Because every test gets a fresh context but the same server, cross-test state on the server (e.g. persisted chat pin) must be handled by dynamic before/after reads rather than assumptions — the specs do this for CHAT_PIN.

9. Evidence: the config is directly exercised by the final artifact run (e2e/test-results/run-live-FINAL-artifact.log: 93 tests, 79 passed, 0 failed, 14 skipped, exit 0, 3.6 m) and by the earlier per-spec runs; there is no unit test for the config itself. Its correctness is therefore operational, not asserted: an environment without Chrome/Edge would fail at launch with a Playwright error rather than a custom named refusal, which is acceptable but not a receipt.

10. Risk summary for this file: S7 (baseURL default 8080 vs shipped 8799 and no artifact-identity preflight), S12 (host-pinned browser versions only in comments), S13 (no webServer + 1 worker trade-off), S3 (test-results wiped per run). No blocking defect found; the file is a compact, readable single source of truth for the harness runtime.

**Evidence/UNVERIFIED:** read 48 lines; flagged at process.env:17, hardcoded-host:11.

---

## e2e/README.md

**Metadata:** 4722 B · sha256 `566522894E805A08096C3F4D356672FB6D387430592AFDBFBCDAC455223D7ED3` (manifest MATCH) · Markdown · category `other` · depth `standard` · git-tracked: yes.

**Purpose:** Repository documentation for the e2e harness (run commands, evidence, limitations).

**Mechanics:** Markdown reference; not executed.

**Dependencies/callers:** `e2e/node_modules/@playwright/test/package.json`; external: none detected.

**Quality (1–5):** correctness=4, maintainability=4, modularity=4, error_handling=4, security=3, docs=3, evidence=4. Flags: skipped-test, mock-data, hardcoded-host.

**Evidence/UNVERIFIED:** read 80 lines; flagged at skipped-test:7, mock-data:19, hardcoded-host:5.

---

## e2e/specs/a11y-gates.spec.ts

**Metadata:** 2769 B · sha256 `9F5DE694757DCF044C2D6B553482F08290674CE2831AE56C5D7976C9190E8C4F` (manifest MATCH) · TypeScript · category `source` · depth `standard` · git-tracked: yes.

**Purpose:** Browser E2E test suite executed by Playwright against the shipped embedded UI (Chrome channel).

**Mechanics:** Registers 3 test(...) cases and asserts DOM/WS/contract behaviour; writes evidence JSON tied to each assertion (see imports/evidence helper).

**Dependencies/callers:** `e2e/helpers/contrast.ts`, `e2e/helpers/matrix.ts`; external: `@playwright/test`.

**Quality (1–5):** correctness=4, maintainability=4, modularity=4, error_handling=4, security=4, docs=3, evidence=5. Flags: skipped-test.

**Evidence/UNVERIFIED:** read 53 lines; flagged at skipped-test:44.

---

## e2e/specs/control-matrix.spec.ts

**Metadata:** 3755 B · sha256 `E4B16D3E05BD57AEAB2DC460032B9A304AE8EF9044A704122879487076B38407` (manifest MATCH) · TypeScript · category `source` · depth `standard` · git-tracked: yes.

**Purpose:** Browser E2E test suite executed by Playwright against the shipped embedded UI (Chrome channel).

**Mechanics:** Registers 5 test(...) cases and asserts DOM/WS/contract behaviour; writes evidence JSON tied to each assertion (see imports/evidence helper).

**Dependencies/callers:** `e2e/helpers/matrix.ts`; external: `@playwright/test`.

**Quality (1–5):** correctness=4, maintainability=4, modularity=4, error_handling=4, security=4, docs=3, evidence=5. Flags: skipped-test.

**Evidence/UNVERIFIED:** read 86 lines; flagged at skipped-test:75.

---

## e2e/specs/live-app.spec.ts

**Metadata:** 19833 B · sha256 `CB3A5E8D70EF35F937D4ECDC3266D6A91265329720E9C5B1F385B8299BC7C54C` (manifest MATCH) · TypeScript · category `source` · depth `deep` · git-tracked: yes.

**Purpose:** Browser E2E test suite executed by Playwright against the shipped embedded UI (Chrome channel).

**Mechanics:** Registers 15 test(...) cases and asserts DOM/WS/contract behaviour; writes evidence JSON tied to each assertion (see imports/evidence helper).

**Dependencies/callers:** `e2e/helpers/matrix.ts`, `e2e/helpers/evidence.ts`, `e2e/helpers/contrast.ts`; external: `@playwright/test`.

**Quality (1–5):** correctness=5, maintainability=4, modularity=4, error_handling=4, security=3, docs=3, evidence=5. Flags: skipped-test, process.env, hardcoded-host.

**Deep analysis (8–12 paragraphs):**

1. Functional mechanics: a 15-test Playwright suite (lines 14-334) organised into three describes — boot & surfaces, safe interactions & render-back, and contract wiring. It navigates the shipped embedded UI at BASE (env E2E_BASE_URL, fallback 127.0.0.1:8799), asserts concrete DOM/ARIA facts, performs only safe client-side or read commands (tabs, nav, zoom/Ctrl+0, CHAT_PIN, SETTINGS_STATUS, FILE_TREE_LIST, empty-prompt refusal), and writes a numbered JSON evidence file per test into docs/trace/live/live-app/. Every test starts with test.skip(appMissing(), skipReason()) so it is inert unless E2E_APP_READY=1.

2. Boot/basics tests: boot (15-28) asserts HTTP 200, exact document title, #root visibility, skip-link text/href and body length, then captures a screenshot; skip-link target (30-37) asserts #hive-canvas exists; console test (39-50) registers console/pageerror listeners before navigation and requires zero errors; census (52-95) asserts 6 nav items, 4 tabs, chat core, composer placeholder, the four control buttons, HUD (with honest NO METRICS), disabled scrubber, library + 4 templates, minimap and 4 zoom buttons. These are strong smoke assertions tied to captured evidence.

3. Interaction tests: tab switching (99-112) clicks each tab and asserts aria-selected, then verifies Routing empty copy; nav test (114-133) clicks all five non-workspace destinations asserting aria-current and expected panel copy, then returns to Workspace; zoom test (135-155) reads the React Flow viewport transform before/after Zoom in and Ctrl+0 and treats Fit as a safe no-op on an empty graph. These render-back checks are genuinely behavioural, not just presence checks.

4. WebSocket/contract tests: CHAT_PIN (157-171) captures frames, waits for SERVER_HELLO, records aria-pressed before, clicks pin, waits for CHAT_PIN command and the ACK whose correlation_id matches, then asserts the pressed state flipped — a correct ACK-before-render-back probe. Empty prompt (173-187) proves the client-side E_PROMPT_EMPTY toast appears while asserting zero SUBMIT_PROMPT frames were sent (no fake success). WS handshake (189-205) compares the WS URL token and last_seq with /api/session and asserts the hello frame plus SERVER_HELLO envelope fields.

5. Settings contract test (229-243): captures frames, opens Settings, then inspects commands/responses, asserting exactly one SETTINGS_STATUS, an ACK and a SETTINGS_STATUS_RESULT. It deliberately uses expect.soft for these three assertions so all three diagnostics appear at once while the test still fails if any fail (Playwright marks soft-assertion failures). The evidence JSON records commands count and response codes/correlation ids, which is exactly the forensic payload needed when the command regresses.

6. Data-panel tests (245-293, 295-333): the first asserts the shipped bundle no longer contains the stale 'no workspace-file API yet' / 'no approval-response command' claims by reading main text and fetching the module bundle; the second captures FILE_TREE_RESULT and asserts note.txt/README.md render; the third asserts the unknown-size em dash (README.md—, note.txt—) and that '0 B' appears neither in DOM nor in the shipped bundle. These tests found and then pinned the real contract-drift and null-size rendering defects.

7. Test isolation and state: each test gets a fresh Playwright context/page but they share one live server, so server-side state matters. CHAT_PIN reads before/after dynamically and flips the persisted pin, which is safe only if the test runs exactly once per run; two parallel workers would race. The config enforces workers=1, so this is intentional, but it is a coupling worth documenting beyond the config comment.

8. Time and flake behaviour: gotoApp() waits for networkidle plus 600 ms (lines 8-12); several tests add 350/450/600/1500 ms settle waits; WS waits use a polling helper with a 6 s timeout. On the shipped artifact the suite ran 15/15 green in the final run, but the design relies on generous fixed waits rather than event-driven synchronisation (e.g. waitForSelector/waitForResponse), so a slower machine could produce timeouts or flaky screenshots rather than deterministic failure evidence.

9. Evidence quality: every test writes a numbered JSON artefact (01-15) with the observed values that justify its assertion; screenshots are attached for boot/census. This is a strong audit trail. Limitations: evidence paths are hardcoded into docs/trace/live/live-app (dirtying the product tree on every run — observed untracked 14/15 files at closeout), the files carry no schema version or run id, and writeEvidence does not assert the write succeeded.

10. Coverage boundaries: the suite says nothing about node cards, streaming replies, approvals, model/provider calls, re-entry/terminal state or performance; those are UNVERIFIED by design because no paid run was started. It also relies on the shipped workspace fixture containing exactly note.txt/README.md and on copy strings from the product (e.g. 'Provider key status', 'No edges in the current graph.'). That is acceptable for pinning current behaviour but makes the suite sensitive to legitimate copy/fixture edits.

11. Risk summary for this file: S1-adjacent side effects (writes into docs/), S6 (fixture literals), S7 (hardcoded 8799 fallback and no artifact identity check), plus the skip gate S2. The assertions themselves are concrete, empirically validated on the shipped artifact (15/15 in the final run) and provide the most user-visible evidence in this partition.

**Evidence/UNVERIFIED:** read 335 lines; flagged at skipped-test:16, process.env:6, hardcoded-host:6.

---

## e2e/specs/trace-hops.spec.ts

**Metadata:** 17640 B · sha256 `03B8B6A4F1122F494BEF9DCFE8D227C189928771462942090B5639EF7D7E9270` (manifest MATCH) · TypeScript · category `source` · depth `deep` · git-tracked: yes.

**Purpose:** Browser E2E test suite executed by Playwright against the shipped embedded UI (Chrome channel).

**Mechanics:** Registers 1 test(...) cases and asserts DOM/WS/contract behaviour; writes evidence JSON tied to each assertion (see imports/evidence helper).

**Dependencies/callers:** `e2e/helpers/matrix.ts`, `e2e/helpers/evidence.ts`; external: `@playwright/test`, `node:fs`, `node:url`.

**Quality (1–5):** correctness=5, maintainability=4, modularity=4, error_handling=4, security=3, docs=3, evidence=5. Flags: skipped-test, process.env, hardcoded-host, fs-write, generated-marker.

**Deep analysis (8–12 paragraphs):**

1. Functional mechanics: the spec builds a data-driven suite of 28 per-surface tests (lines 28-57, 89-317) whose job is to replace the placeholder matrix with captured DOM/WS evidence. A module-level SurfaceRecord[] accumulates one record per surface, and test.afterAll (lines 76-87) serialises the whole array into docs/trace/live/trace-capture.json with a schema tag, timestamp, base URL and an explicit no-fabrication rule. Each test navigates the live app, runs a kind-specific branch, sets hop verdicts from what it observed, asserts the surface rendered, and returns.

2. Record contract: SurfaceRecord (lines 16-25) carries surface_id, human surface name, selector, present, free-text evidence, interactions, an 11-key hops map restricted to the five trace verdicts, and notes. hopBase(present) (lines 59-73) initialises every hop to UNVERIFIED except 1_render, which is WIRED-AND-WORKS only when the surface is present. This defaults-closed design is what makes the file honest: a hop is green only when a branch explicitly proves it.

3. Branch coverage: node-card surfaces S15-S19 (lines 106-113) intentionally record present=false/MISSING and note that a live run is required — no paid provider call is made, so this is an acknowledged gap rather than a fake pass. Settings (115-142) and Data (160-190) branches capture WebSocket frames and record ACK/result hops; tabs (217-232), nav (234-250) and zoom (252-272) perform real client interactions and derive 10_render_back from aria-selected/aria-current/computed transform changes; composer/control (274-297) prove render and gating only.

4. Settings under-claim: the Settings branch sets hops 2-5 and 9 from the observed ACK + SETTINGS_STATUS_RESULT, but never sets 10_render_back even though it separately waits for 'Provider key status' and the panel visibly renders provider/model/key values (proven in live-app.spec.ts:229-243). In the final capture S21 therefore has 9=WIRED-AND-WORKS but 10=UNVERIFIED — conservative, but inconsistent with the evidence the same branch already holds.

5. Data branch quality: it attaches WS capture before navigation (line 92), so frames emitted during page load are captured; it then clicks Data, waits up to 6 s for FILE_TREE_RESULT or ERROR, reads main text and regex-checks for note.txt/README.md before assigning 9_response and 10_render_back. This is the strongest chain in the file (client call → route → response → render-back) and it was the branch that proved the shipped S22 fix.

6. Completeness gap: SURFACES has 28 entries while e2e/data/control-matrix.json contains 29 controls including the supplemental S-APPROVAL (APPROVAL_RESOLVE). No S-APPROVAL record is written, and afterAll never asserts records.length===28 or that IDs are unique/ordered. If a test aborts early (page.goto timeout, worker crash), the afterAll still writes whatever survived, and apply-live-trace.mjs will merge a partial capture without complaint. The capture schema itself is a string tag, not validated by any tool.

7. Determinism and flake exposure: every test repeats page.goto with waitUntil networkidle plus a fixed 350 ms settle (line 93-94); branches use fixed waits of 300/350/450/800/1200 ms and a 6 s frame timeout. These are pragmatic but time-based: on a loaded host or slow restart they can under-wait (missing frame) or over-wait (slow suite). The final run passed 28/28 in ~2.1 min, so the current server is comfortably within budget, but the margins are not measured or asserted.

8. Fixture coupling: the Data branch hardcodes /note\.txt|README\.md/ (line 172) and states branch hardcodes the Agents/Scrubber/HUD copy (lines 204-206). These strings come from the runtime workspace and product copy, so a legitimate fixture or copy change can fail a trace capture unrelated to wiring. A better probe would derive expected entries from the captured FILE_TREE_RESULT payload and expected copy from a single source (strings/aria), as the spec already does for most selectors.

9. Evidence provenance: the spec writes only into docs/trace/live (ensureLiveDirs/LIVE_DIR from helpers/evidence.ts) and never into product source. It labels the capture base_url and generation time but does not attach per-surface timestamps, event seq ranges or screenshots; a reader cannot tell when each record was captured relative to server quiescence except by file mtime. For a paper-grade trace, per-record captured_at and seq would strengthen the evidence.

10. Dependency and execution evidence: imports are from @playwright/test, node:fs/node:url and the local helpers matrix/evidence (lines 1-5); it has no backend access and no direct DB/HTTP assertions, so hops 6-8 (service/store/provider) and 11 (re-entry) remain UNVERIFIED by design. Executed in the final artifact run: 28/28 passed, contributing the 43 WIRED-AND-WORKS hops that were later merged into docs/trace/controls.json by apply-live-trace.mjs.

11. Risk summary for this file: S4 (capture completeness not enforced; S-APPROVAL missing; partial captures accepted downstream), S5 (S21 render-back under-claimed), S6 (fixture/copy literals). The core design — data-driven, defaults-closed, evidence-per-branch, explicit UNVERIFIED for unexercised hops — is sound and is the best scientific artifact in this partition.

**Evidence/UNVERIFIED:** read 319 lines; flagged at skipped-test:91, process.env:14, hardcoded-host:14, fs-write:2.

---

## e2e/specs/ws-contract.spec.ts

**Metadata:** 2188 B · sha256 `025EB2901FF4DD490F468CE75FF7AFD13F6553500570D55FA5BB1F34CBEF8599` (manifest MATCH) · TypeScript · category `source` · depth `standard` · git-tracked: yes.

**Purpose:** Browser E2E test suite executed by Playwright against the shipped embedded UI (Chrome channel).

**Mechanics:** Registers 3 test(...) cases and asserts DOM/WS/contract behaviour; writes evidence JSON tied to each assertion (see imports/evidence helper).

**Dependencies/callers:** `e2e/helpers/app.ts`, `e2e/helpers/matrix.ts`, `e2e/helpers/ws-tap.ts`; external: `@playwright/test`.

**Quality (1–5):** correctness=4, maintainability=4, modularity=4, error_handling=4, security=4, docs=3, evidence=5. Flags: skipped-test.

**Evidence/UNVERIFIED:** read 45 lines; flagged at skipped-test:12.

---

## e2e/tsconfig.json

**Metadata:** 228 B · sha256 `6D92AE9D89B3F28E9EDE5F180059DB48D5571AAE49EA5696D203F71337C3730C` (manifest MATCH) · JSON · category `config-manifest` · depth `standard` · git-tracked: yes.

**Purpose:** Ad-hoc Playwright probe script (exploration/smoke/screenshot), not wired into the test runner.

**Mechanics:** Starts installed Chrome headless and prints/records live-app state; overlaps specs in purpose.

**Dependencies/callers:** no in-partition imports; external: none detected.

**Quality (1–5):** correctness=4, maintainability=4, modularity=4, error_handling=4, security=4, docs=3, evidence=4. Flags: none.

**Evidence/UNVERIFIED:** read 13 lines.

---

## runtime-test/data/hiverouter.db

**Metadata:** 4096 B · sha256 `89A2318C76452BE80E1D743F163F3DD96623B2B05604AA06985C6B5501238362` (manifest MATCH) · SQLite · category `runtime-artifact` · depth `standard` · git-tracked: yes.

**Purpose:** SQLite runtime database produced by the shipped server during live QA; schema_version 3 previously observed; contains runtime/seed rows.

**Mechanics:** Binary SQLite file (+SHM/WAL companions if present); generated, not authored source.

**Dependencies/callers:** no in-partition imports; external: none detected.

**Quality (1–5):** correctness=4, maintainability=4, modularity=4, error_handling=4, security=4, docs=3, evidence=4. Flags: none.

**Evidence/UNVERIFIED:** binary/large file — content not read (metadata + role only); behaviour UNVERIFIED at byte level.

---

## runtime-test/data/hiverouter.db-shm

**Metadata:** 32768 B · sha256 `F6FCFFAF91E43D08B3945DCDB332FB8E40AA14A7FEE0FF98F8F447AE8EF8ABE7` (manifest MATCH) · SQLite-SHM · category `runtime-artifact` · depth `standard` · git-tracked: yes.

**Purpose:** SQLite runtime database produced by the shipped server during live QA; schema_version 3 previously observed; contains runtime/seed rows.

**Mechanics:** Binary SQLite file (+SHM/WAL companions if present); generated, not authored source.

**Dependencies/callers:** no in-partition imports; external: none detected.

**Quality (1–5):** correctness=4, maintainability=4, modularity=4, error_handling=4, security=4, docs=3, evidence=4. Flags: none.

**Evidence/UNVERIFIED:** binary/large file — content not read (metadata + role only); behaviour UNVERIFIED at byte level.

---

## runtime-test/data/hiverouter.db-wal

**Metadata:** 94792 B · sha256 `90A60CD79BA2DA4C9C1963ABB0BBE8E3629535CD5EF23686D515CB0DD6BF516A` (manifest MATCH) · SQLite-WAL · category `runtime-artifact` · depth `standard` · git-tracked: yes.

**Purpose:** SQLite runtime database produced by the shipped server during live QA; schema_version 3 previously observed; contains runtime/seed rows.

**Mechanics:** Binary SQLite file (+SHM/WAL companions if present); generated, not authored source.

**Dependencies/callers:** no in-partition imports; external: none detected.

**Quality (1–5):** correctness=4, maintainability=4, modularity=4, error_handling=4, security=4, docs=3, evidence=4. Flags: none.

**Evidence/UNVERIFIED:** binary/large file — content not read (metadata + role only); behaviour UNVERIFIED at byte level.

---

## runtime-test/data2/hiverouter.db

**Metadata:** 4096 B · sha256 `89A2318C76452BE80E1D743F163F3DD96623B2B05604AA06985C6B5501238362` (manifest MATCH) · SQLite · category `runtime-artifact` · depth `standard` · git-tracked: yes.

**Purpose:** SQLite runtime database produced by the shipped server during live QA; schema_version 3 previously observed; contains runtime/seed rows.

**Mechanics:** Binary SQLite file (+SHM/WAL companions if present); generated, not authored source.

**Dependencies/callers:** no in-partition imports; external: none detected.

**Quality (1–5):** correctness=4, maintainability=4, modularity=4, error_handling=4, security=4, docs=3, evidence=4. Flags: none.

**Evidence/UNVERIFIED:** binary/large file — content not read (metadata + role only); behaviour UNVERIFIED at byte level.

---

## runtime-test/data2/hiverouter.db-shm

**Metadata:** 32768 B · sha256 `B86EDA07432C5F9282B06C85BCA16BE947D5D278F96CDE1A907454BC27CC34A2` (manifest MATCH) · SQLite-SHM · category `runtime-artifact` · depth `standard` · git-tracked: yes.

**Purpose:** SQLite runtime database produced by the shipped server during live QA; schema_version 3 previously observed; contains runtime/seed rows.

**Mechanics:** Binary SQLite file (+SHM/WAL companions if present); generated, not authored source.

**Dependencies/callers:** no in-partition imports; external: none detected.

**Quality (1–5):** correctness=4, maintainability=4, modularity=4, error_handling=4, security=4, docs=3, evidence=4. Flags: none.

**Evidence/UNVERIFIED:** binary/large file — content not read (metadata + role only); behaviour UNVERIFIED at byte level.

---

## runtime-test/data2/hiverouter.db-wal

**Metadata:** 94792 B · sha256 `66B34BF73C5CB37954B9BC8215013A4D2A0461F4FA9792248DD44B02213D875B` (manifest MATCH) · SQLite-WAL · category `runtime-artifact` · depth `standard` · git-tracked: yes.

**Purpose:** SQLite runtime database produced by the shipped server during live QA; schema_version 3 previously observed; contains runtime/seed rows.

**Mechanics:** Binary SQLite file (+SHM/WAL companions if present); generated, not authored source.

**Dependencies/callers:** no in-partition imports; external: none detected.

**Quality (1–5):** correctness=4, maintainability=4, modularity=4, error_handling=4, security=4, docs=3, evidence=4. Flags: none.

**Evidence/UNVERIFIED:** binary/large file — content not read (metadata + role only); behaviour UNVERIFIED at byte level.

---

## runtime-test/data3/hiverouter.db

**Metadata:** 4096 B · sha256 `89A2318C76452BE80E1D743F163F3DD96623B2B05604AA06985C6B5501238362` (manifest MATCH) · SQLite · category `runtime-artifact` · depth `standard` · git-tracked: yes.

**Purpose:** SQLite runtime database produced by the shipped server during live QA; schema_version 3 previously observed; contains runtime/seed rows.

**Mechanics:** Binary SQLite file (+SHM/WAL companions if present); generated, not authored source.

**Dependencies/callers:** no in-partition imports; external: none detected.

**Quality (1–5):** correctness=4, maintainability=4, modularity=4, error_handling=4, security=4, docs=3, evidence=4. Flags: none.

**Evidence/UNVERIFIED:** binary/large file — content not read (metadata + role only); behaviour UNVERIFIED at byte level.

---

## runtime-test/data3/hiverouter.db-shm

**Metadata:** 32768 B · sha256 `AC5ED03284D6ADB47B966A2CDEE315E341EE811FBF7E433EE6218B362270476E` (manifest MATCH) · SQLite-SHM · category `runtime-artifact` · depth `standard` · git-tracked: yes.

**Purpose:** SQLite runtime database produced by the shipped server during live QA; schema_version 3 previously observed; contains runtime/seed rows.

**Mechanics:** Binary SQLite file (+SHM/WAL companions if present); generated, not authored source.

**Dependencies/callers:** no in-partition imports; external: none detected.

**Quality (1–5):** correctness=4, maintainability=4, modularity=4, error_handling=4, security=4, docs=3, evidence=4. Flags: none.

**Evidence/UNVERIFIED:** binary/large file — content not read (metadata + role only); behaviour UNVERIFIED at byte level.

---

## runtime-test/data3/hiverouter.db-wal

**Metadata:** 94792 B · sha256 `DFDBC19177BD835C248BF45B48448FC6C4DE5428BB6DAD48FC9BD47226B82FBC` (manifest MATCH) · SQLite-WAL · category `runtime-artifact` · depth `standard` · git-tracked: yes.

**Purpose:** SQLite runtime database produced by the shipped server during live QA; schema_version 3 previously observed; contains runtime/seed rows.

**Mechanics:** Binary SQLite file (+SHM/WAL companions if present); generated, not authored source.

**Dependencies/callers:** no in-partition imports; external: none detected.

**Quality (1–5):** correctness=4, maintainability=4, modularity=4, error_handling=4, security=4, docs=3, evidence=4. Flags: none.

**Evidence/UNVERIFIED:** binary/large file — content not read (metadata + role only); behaviour UNVERIFIED at byte level.

---

## runtime-test/data4/hiverouter.db

**Metadata:** 4096 B · sha256 `89A2318C76452BE80E1D743F163F3DD96623B2B05604AA06985C6B5501238362` (manifest MATCH) · SQLite · category `runtime-artifact` · depth `standard` · git-tracked: yes.

**Purpose:** SQLite runtime database produced by the shipped server during live QA; schema_version 3 previously observed; contains runtime/seed rows.

**Mechanics:** Binary SQLite file (+SHM/WAL companions if present); generated, not authored source.

**Dependencies/callers:** no in-partition imports; external: none detected.

**Quality (1–5):** correctness=4, maintainability=4, modularity=4, error_handling=4, security=4, docs=3, evidence=4. Flags: none.

**Evidence/UNVERIFIED:** binary/large file — content not read (metadata + role only); behaviour UNVERIFIED at byte level.

---

## runtime-test/data4/hiverouter.db-shm

**Metadata:** 32768 B · sha256 `A7AD9AA038535D86B7EA98CA6ED07B932CC607072850A9DFB7F83DD1970C0F77` (manifest MATCH) · SQLite-SHM · category `runtime-artifact` · depth `standard` · git-tracked: yes.

**Purpose:** SQLite runtime database produced by the shipped server during live QA; schema_version 3 previously observed; contains runtime/seed rows.

**Mechanics:** Binary SQLite file (+SHM/WAL companions if present); generated, not authored source.

**Dependencies/callers:** no in-partition imports; external: none detected.

**Quality (1–5):** correctness=4, maintainability=4, modularity=4, error_handling=4, security=4, docs=3, evidence=4. Flags: none.

**Evidence/UNVERIFIED:** binary/large file — content not read (metadata + role only); behaviour UNVERIFIED at byte level.

---

## runtime-test/data4/hiverouter.db-wal

**Metadata:** 152472 B · sha256 `A258EBD0FF9F7F1DA7857BFE208D05192137218C380D17C7B217050B5F68F8CA` (manifest MATCH) · SQLite-WAL · category `runtime-artifact` · depth `standard` · git-tracked: yes.

**Purpose:** SQLite runtime database produced by the shipped server during live QA; schema_version 3 previously observed; contains runtime/seed rows.

**Mechanics:** Binary SQLite file (+SHM/WAL companions if present); generated, not authored source.

**Dependencies/callers:** no in-partition imports; external: none detected.

**Quality (1–5):** correctness=4, maintainability=4, modularity=4, error_handling=4, security=4, docs=3, evidence=4. Flags: none.

**Evidence/UNVERIFIED:** binary/large file — content not read (metadata + role only); behaviour UNVERIFIED at byte level.

---

## runtime-test/data4/snapshots/01a1b119-2f34-7228-974e-03b1c27856fc/0000000000.json

**Metadata:** 271 B · sha256 `B3384389368E413687C88F3C462AE81CE62E7AFD61AD07DC2B8B2B443C39E05B` (manifest MATCH) · JSON · category `runtime-artifact` · depth `standard` · git-tracked: yes.

**Purpose:** Generated runtime-test artifact/log from the live server.

**Mechanics:** Non-source artifact captured during runs; included for provenance of the live test session.

**Dependencies/callers:** no in-partition imports; external: none detected.

**Quality (1–5):** correctness=3, maintainability=1, modularity=1, error_handling=1, security=2, docs=1, evidence=2. Flags: runtime-artifact.

**Evidence/UNVERIFIED:** read 13 lines.

---

## runtime-test/data5/hiverouter.db

**Metadata:** 4096 B · sha256 `89A2318C76452BE80E1D743F163F3DD96623B2B05604AA06985C6B5501238362` (manifest MATCH) · SQLite · category `runtime-artifact` · depth `standard` · git-tracked: yes.

**Purpose:** SQLite runtime database produced by the shipped server during live QA; schema_version 3 previously observed; contains runtime/seed rows.

**Mechanics:** Binary SQLite file (+SHM/WAL companions if present); generated, not authored source.

**Dependencies/callers:** no in-partition imports; external: none detected.

**Quality (1–5):** correctness=4, maintainability=4, modularity=4, error_handling=4, security=4, docs=3, evidence=4. Flags: none.

**Evidence/UNVERIFIED:** binary/large file — content not read (metadata + role only); behaviour UNVERIFIED at byte level.

---

## runtime-test/data5/hiverouter.db-shm

**Metadata:** 32768 B · sha256 `48EDB60FE77A761302738713142C382846E8A863AC51A50F3A15C9E6F85F555D` (manifest MATCH) · SQLite-SHM · category `runtime-artifact` · depth `standard` · git-tracked: yes.

**Purpose:** SQLite runtime database produced by the shipped server during live QA; schema_version 3 previously observed; contains runtime/seed rows.

**Mechanics:** Binary SQLite file (+SHM/WAL companions if present); generated, not authored source.

**Dependencies/callers:** no in-partition imports; external: none detected.

**Quality (1–5):** correctness=4, maintainability=4, modularity=4, error_handling=4, security=4, docs=3, evidence=4. Flags: none.

**Evidence/UNVERIFIED:** binary/large file — content not read (metadata + role only); behaviour UNVERIFIED at byte level.

---

## runtime-test/data5/hiverouter.db-wal

**Metadata:** 94792 B · sha256 `6CB19A0BB68C37C6396828E0CEA5EC713C2550A8F1FAA09C64245FA30E5B4752` (manifest MATCH) · SQLite-WAL · category `runtime-artifact` · depth `standard` · git-tracked: yes.

**Purpose:** SQLite runtime database produced by the shipped server during live QA; schema_version 3 previously observed; contains runtime/seed rows.

**Mechanics:** Binary SQLite file (+SHM/WAL companions if present); generated, not authored source.

**Dependencies/callers:** no in-partition imports; external: none detected.

**Quality (1–5):** correctness=4, maintainability=4, modularity=4, error_handling=4, security=4, docs=3, evidence=4. Flags: none.

**Evidence/UNVERIFIED:** binary/large file — content not read (metadata + role only); behaviour UNVERIFIED at byte level.

---

## runtime-test/data6/hiverouter.db

**Metadata:** 4096 B · sha256 `89A2318C76452BE80E1D743F163F3DD96623B2B05604AA06985C6B5501238362` (manifest MATCH) · SQLite · category `runtime-artifact` · depth `standard` · git-tracked: yes.

**Purpose:** SQLite runtime database produced by the shipped server during live QA; schema_version 3 previously observed; contains runtime/seed rows.

**Mechanics:** Binary SQLite file (+SHM/WAL companions if present); generated, not authored source.

**Dependencies/callers:** no in-partition imports; external: none detected.

**Quality (1–5):** correctness=4, maintainability=4, modularity=4, error_handling=4, security=4, docs=3, evidence=4. Flags: none.

**Evidence/UNVERIFIED:** binary/large file — content not read (metadata + role only); behaviour UNVERIFIED at byte level.

---

## runtime-test/data6/hiverouter.db-shm

**Metadata:** 32768 B · sha256 `41607360C495452BD680C280BD6F0B16A437ECD344E92BA058F20266C61D2AD6` (manifest MATCH) · SQLite-SHM · category `runtime-artifact` · depth `standard` · git-tracked: yes.

**Purpose:** SQLite runtime database produced by the shipped server during live QA; schema_version 3 previously observed; contains runtime/seed rows.

**Mechanics:** Binary SQLite file (+SHM/WAL companions if present); generated, not authored source.

**Dependencies/callers:** no in-partition imports; external: none detected.

**Quality (1–5):** correctness=4, maintainability=4, modularity=4, error_handling=4, security=4, docs=3, evidence=4. Flags: none.

**Evidence/UNVERIFIED:** binary/large file — content not read (metadata + role only); behaviour UNVERIFIED at byte level.

---

## runtime-test/data6/hiverouter.db-wal

**Metadata:** 152472 B · sha256 `FCF13BB6D6C7F60774FE7F487AB9F38FE0DA67819E5A26F084308ABEEF3D0D08` (manifest MATCH) · SQLite-WAL · category `runtime-artifact` · depth `standard` · git-tracked: yes.

**Purpose:** SQLite runtime database produced by the shipped server during live QA; schema_version 3 previously observed; contains runtime/seed rows.

**Mechanics:** Binary SQLite file (+SHM/WAL companions if present); generated, not authored source.

**Dependencies/callers:** no in-partition imports; external: none detected.

**Quality (1–5):** correctness=4, maintainability=4, modularity=4, error_handling=4, security=4, docs=3, evidence=4. Flags: none.

**Evidence/UNVERIFIED:** binary/large file — content not read (metadata + role only); behaviour UNVERIFIED at byte level.

---

## runtime-test/data6/snapshots/01a1b11d-90f3-73b3-bca9-616a21cd2e54/0000000000.json

**Metadata:** 271 B · sha256 `208D3889264A830EE08C8D95ADE7E02E35C3C6D6F890721FBC4C9246F33AC820` (manifest MATCH) · JSON · category `runtime-artifact` · depth `standard` · git-tracked: yes.

**Purpose:** Generated runtime-test artifact/log from the live server.

**Mechanics:** Non-source artifact captured during runs; included for provenance of the live test session.

**Dependencies/callers:** no in-partition imports; external: none detected.

**Quality (1–5):** correctness=3, maintainability=1, modularity=1, error_handling=1, security=2, docs=1, evidence=2. Flags: runtime-artifact.

**Evidence/UNVERIFIED:** read 13 lines.

---

## runtime-test/data7/hiverouter.db

**Metadata:** 4096 B · sha256 `89A2318C76452BE80E1D743F163F3DD96623B2B05604AA06985C6B5501238362` (manifest MATCH) · SQLite · category `runtime-artifact` · depth `standard` · git-tracked: yes.

**Purpose:** SQLite runtime database produced by the shipped server during live QA; schema_version 3 previously observed; contains runtime/seed rows.

**Mechanics:** Binary SQLite file (+SHM/WAL companions if present); generated, not authored source.

**Dependencies/callers:** no in-partition imports; external: none detected.

**Quality (1–5):** correctness=4, maintainability=4, modularity=4, error_handling=4, security=4, docs=3, evidence=4. Flags: none.

**Evidence/UNVERIFIED:** binary/large file — content not read (metadata + role only); behaviour UNVERIFIED at byte level.

---

## runtime-test/data7/hiverouter.db-shm

**Metadata:** 32768 B · sha256 `6202D05B99FCCF258B1A56FED7260F8952A1E040AB46E0C719C2A08423F53BD8` (manifest MATCH) · SQLite-SHM · category `runtime-artifact` · depth `standard` · git-tracked: yes.

**Purpose:** SQLite runtime database produced by the shipped server during live QA; schema_version 3 previously observed; contains runtime/seed rows.

**Mechanics:** Binary SQLite file (+SHM/WAL companions if present); generated, not authored source.

**Dependencies/callers:** no in-partition imports; external: none detected.

**Quality (1–5):** correctness=4, maintainability=4, modularity=4, error_handling=4, security=4, docs=3, evidence=4. Flags: none.

**Evidence/UNVERIFIED:** binary/large file — content not read (metadata + role only); behaviour UNVERIFIED at byte level.

---

## runtime-test/data7/hiverouter.db-wal

**Metadata:** 127752 B · sha256 `DBFA36BC2423DD8C3F54B7FCF2EA3362D9AB5709C158AEC0B12F3FB23EED7B03` (manifest MATCH) · SQLite-WAL · category `runtime-artifact` · depth `standard` · git-tracked: yes.

**Purpose:** SQLite runtime database produced by the shipped server during live QA; schema_version 3 previously observed; contains runtime/seed rows.

**Mechanics:** Binary SQLite file (+SHM/WAL companions if present); generated, not authored source.

**Dependencies/callers:** no in-partition imports; external: none detected.

**Quality (1–5):** correctness=4, maintainability=4, modularity=4, error_handling=4, security=4, docs=3, evidence=4. Flags: none.

**Evidence/UNVERIFIED:** binary/large file — content not read (metadata + role only); behaviour UNVERIFIED at byte level.

---

## runtime-test/data8/hiverouter.db

**Metadata:** 4096 B · sha256 `89A2318C76452BE80E1D743F163F3DD96623B2B05604AA06985C6B5501238362` (manifest MATCH) · SQLite · category `runtime-artifact` · depth `standard` · git-tracked: yes.

**Purpose:** SQLite runtime database produced by the shipped server during live QA; schema_version 3 previously observed; contains runtime/seed rows.

**Mechanics:** Binary SQLite file (+SHM/WAL companions if present); generated, not authored source.

**Dependencies/callers:** no in-partition imports; external: none detected.

**Quality (1–5):** correctness=4, maintainability=4, modularity=4, error_handling=4, security=4, docs=3, evidence=4. Flags: none.

**Evidence/UNVERIFIED:** binary/large file — content not read (metadata + role only); behaviour UNVERIFIED at byte level.

---

## runtime-test/data8/hiverouter.db-shm

**Metadata:** 32768 B · sha256 `94D110727614C178AADC22EF676EFE5B4875B69653EBB59DC66E64038E88687E` (manifest MATCH) · SQLite-SHM · category `runtime-artifact` · depth `standard` · git-tracked: yes.

**Purpose:** SQLite runtime database produced by the shipped server during live QA; schema_version 3 previously observed; contains runtime/seed rows.

**Mechanics:** Binary SQLite file (+SHM/WAL companions if present); generated, not authored source.

**Dependencies/callers:** no in-partition imports; external: none detected.

**Quality (1–5):** correctness=4, maintainability=4, modularity=4, error_handling=4, security=4, docs=3, evidence=4. Flags: none.

**Evidence/UNVERIFIED:** binary/large file — content not read (metadata + role only); behaviour UNVERIFIED at byte level.

---

## runtime-test/data8/hiverouter.db-wal

**Metadata:** 127752 B · sha256 `705014ADDA043F550D06B9C742731FBD577C5475B50E7AD11A940B6DE6A43AF7` (manifest MATCH) · SQLite-WAL · category `runtime-artifact` · depth `standard` · git-tracked: yes.

**Purpose:** SQLite runtime database produced by the shipped server during live QA; schema_version 3 previously observed; contains runtime/seed rows.

**Mechanics:** Binary SQLite file (+SHM/WAL companions if present); generated, not authored source.

**Dependencies/callers:** no in-partition imports; external: none detected.

**Quality (1–5):** correctness=4, maintainability=4, modularity=4, error_handling=4, security=4, docs=3, evidence=4. Flags: none.

**Evidence/UNVERIFIED:** binary/large file — content not read (metadata + role only); behaviour UNVERIFIED at byte level.

---

## runtime-test/data9/hiverouter.db

**Metadata:** 4096 B · sha256 `89A2318C76452BE80E1D743F163F3DD96623B2B05604AA06985C6B5501238362` (manifest MATCH) · SQLite · category `runtime-artifact` · depth `standard` · git-tracked: yes.

**Purpose:** SQLite runtime database produced by the shipped server during live QA; schema_version 3 previously observed; contains runtime/seed rows.

**Mechanics:** Binary SQLite file (+SHM/WAL companions if present); generated, not authored source.

**Dependencies/callers:** no in-partition imports; external: none detected.

**Quality (1–5):** correctness=4, maintainability=4, modularity=4, error_handling=4, security=4, docs=3, evidence=4. Flags: none.

**Evidence/UNVERIFIED:** binary/large file — content not read (metadata + role only); behaviour UNVERIFIED at byte level.

---

## runtime-test/data9/hiverouter.db-shm

**Metadata:** 32768 B · sha256 `E8620D36D8403129F81BA7211A88B2B07BCC187034B614827C295147D62EE1DB` (manifest MATCH) · SQLite-SHM · category `runtime-artifact` · depth `standard` · git-tracked: yes.

**Purpose:** SQLite runtime database produced by the shipped server during live QA; schema_version 3 previously observed; contains runtime/seed rows.

**Mechanics:** Binary SQLite file (+SHM/WAL companions if present); generated, not authored source.

**Dependencies/callers:** no in-partition imports; external: none detected.

**Quality (1–5):** correctness=4, maintainability=4, modularity=4, error_handling=4, security=4, docs=3, evidence=4. Flags: none.

**Evidence/UNVERIFIED:** binary/large file — content not read (metadata + role only); behaviour UNVERIFIED at byte level.

---

## runtime-test/data9/hiverouter.db-wal

**Metadata:** 185432 B · sha256 `655543F8FBC42F1048CF01C0BC236E27E4FCF176CA05DC799CBB5FCCD49B6F60` (manifest MATCH) · SQLite-WAL · category `runtime-artifact` · depth `standard` · git-tracked: yes.

**Purpose:** SQLite runtime database produced by the shipped server during live QA; schema_version 3 previously observed; contains runtime/seed rows.

**Mechanics:** Binary SQLite file (+SHM/WAL companions if present); generated, not authored source.

**Dependencies/callers:** no in-partition imports; external: none detected.

**Quality (1–5):** correctness=4, maintainability=4, modularity=4, error_handling=4, security=4, docs=3, evidence=4. Flags: none.

**Evidence/UNVERIFIED:** binary/large file — content not read (metadata + role only); behaviour UNVERIFIED at byte level.

---

## runtime-test/data9/snapshots/01a1b16d-7530-75e4-b51e-83b6bc3fdcca/0000000000.json

**Metadata:** 271 B · sha256 `BD2270F596523038B41956A54C338D1842E10507083D3AD7B1D3E1543983F958` (manifest MATCH) · JSON · category `runtime-artifact` · depth `standard` · git-tracked: yes.

**Purpose:** Generated runtime-test artifact/log from the live server.

**Mechanics:** Non-source artifact captured during runs; included for provenance of the live test session.

**Dependencies/callers:** no in-partition imports; external: none detected.

**Quality (1–5):** correctness=3, maintainability=1, modularity=1, error_handling=1, security=2, docs=1, evidence=2. Flags: runtime-artifact.

**Evidence/UNVERIFIED:** read 13 lines.

---

## runtime-test/pkg-cli-final/CHECKSUMS.txt

**Metadata:** 712 B · sha256 `842E44C7439268B5114EE8A5A7F0E556DBB24D4AB958FD61ECD780D5F3380CB4` (manifest MATCH) · Text · category `runtime-artifact` · depth `standard` · git-tracked: yes.

**Purpose:** Generated runtime-test artifact/log from the live server.

**Mechanics:** Non-source artifact captured during runs; included for provenance of the live test session.

**Dependencies/callers:** no in-partition imports; external: none detected.

**Quality (1–5):** correctness=3, maintainability=1, modularity=1, error_handling=1, security=2, docs=1, evidence=2. Flags: hardcoded-host, runtime-artifact.

**Evidence/UNVERIFIED:** read 9 lines; flagged at hardcoded-host:5.

---

## runtime-test/pkg-cli-final/data/hiverouter.db

**Metadata:** 4096 B · sha256 `89A2318C76452BE80E1D743F163F3DD96623B2B05604AA06985C6B5501238362` (manifest MATCH) · SQLite · category `runtime-artifact` · depth `standard` · git-tracked: yes.

**Purpose:** SQLite runtime database produced by the shipped server during live QA; schema_version 3 previously observed; contains runtime/seed rows.

**Mechanics:** Binary SQLite file (+SHM/WAL companions if present); generated, not authored source.

**Dependencies/callers:** no in-partition imports; external: none detected.

**Quality (1–5):** correctness=4, maintainability=4, modularity=4, error_handling=4, security=4, docs=3, evidence=4. Flags: none.

**Evidence/UNVERIFIED:** binary/large file — content not read (metadata + role only); behaviour UNVERIFIED at byte level.

---

## runtime-test/pkg-cli-final/data/hiverouter.db-shm

**Metadata:** 32768 B · sha256 `B0C2485EC9EC9C17B8C858ED5FFB618F61E1DCB33880F6C50A846EE9E4E5726C` (manifest MATCH) · SQLite-SHM · category `runtime-artifact` · depth `standard` · git-tracked: yes.

**Purpose:** SQLite runtime database produced by the shipped server during live QA; schema_version 3 previously observed; contains runtime/seed rows.

**Mechanics:** Binary SQLite file (+SHM/WAL companions if present); generated, not authored source.

**Dependencies/callers:** no in-partition imports; external: none detected.

**Quality (1–5):** correctness=4, maintainability=4, modularity=4, error_handling=4, security=4, docs=3, evidence=4. Flags: none.

**Evidence/UNVERIFIED:** binary/large file — content not read (metadata + role only); behaviour UNVERIFIED at byte level.

---

## runtime-test/pkg-cli-final/data/hiverouter.db-wal

**Metadata:** 127752 B · sha256 `779BAE5FE7BF0A3471D6D623F7E029FD49AE0DF88FE98564F2802236E632424E` (manifest MATCH) · SQLite-WAL · category `runtime-artifact` · depth `standard` · git-tracked: yes.

**Purpose:** SQLite runtime database produced by the shipped server during live QA; schema_version 3 previously observed; contains runtime/seed rows.

**Mechanics:** Binary SQLite file (+SHM/WAL companions if present); generated, not authored source.

**Dependencies/callers:** no in-partition imports; external: none detected.

**Quality (1–5):** correctness=4, maintainability=4, modularity=4, error_handling=4, security=4, docs=3, evidence=4. Flags: none.

**Evidence/UNVERIFIED:** binary/large file — content not read (metadata + role only); behaviour UNVERIFIED at byte level.

---

## runtime-test/pkg-cli-final/hiverouter-cli.exe

**Metadata:** 5364224 B · sha256 `51E1ABA6DC7349BFB8904014D7007015E13642EA9710F04D380525DCCBD9B9AD` (manifest MATCH) · PE-binary · category `runtime-artifact` · depth `standard` · git-tracked: yes.

**Purpose:** Generated runtime-test artifact/log from the live server.

**Mechanics:** Non-source artifact captured during runs; included for provenance of the live test session.

**Dependencies/callers:** no in-partition imports; external: none detected.

**Quality (1–5):** correctness=4, maintainability=4, modularity=4, error_handling=4, security=4, docs=3, evidence=4. Flags: none.

**Evidence/UNVERIFIED:** binary/large file — content not read (metadata + role only); behaviour UNVERIFIED at byte level.

---

## runtime-test/pkg-cli-final/hiverouter.exe

**Metadata:** 9329152 B · sha256 `184D51A7C3CCA7B9D50954B677C5AD8A14472700A651FDF0D270441BE3EAC9A9` (manifest MATCH) · PE-binary · category `runtime-artifact` · depth `standard` · git-tracked: yes.

**Purpose:** Generated runtime-test artifact/log from the live server.

**Mechanics:** Non-source artifact captured during runs; included for provenance of the live test session.

**Dependencies/callers:** no in-partition imports; external: none detected.

**Quality (1–5):** correctness=4, maintainability=4, modularity=4, error_handling=4, security=4, docs=3, evidence=4. Flags: none.

**Evidence/UNVERIFIED:** binary/large file — content not read (metadata + role only); behaviour UNVERIFIED at byte level.

---

## runtime-test/pkg-cli-final/README.md

**Metadata:** 1276 B · sha256 `CFA567329EDC9E6C52EF02C8620E2FD27C5917BC92C3EFC1794EA26DBF0BEBA3` (manifest MATCH) · Markdown · category `runtime-artifact` · depth `standard` · git-tracked: yes.

**Purpose:** Generated runtime-test artifact/log from the live server.

**Mechanics:** Non-source artifact captured during runs; included for provenance of the live test session.

**Dependencies/callers:** no in-partition imports; external: none detected.

**Quality (1–5):** correctness=3, maintainability=1, modularity=1, error_handling=1, security=2, docs=1, evidence=2. Flags: runtime-artifact.

**Evidence/UNVERIFIED:** read 25 lines.

---

## runtime-test/pkg-cli-final/RUNBOOK.md

**Metadata:** 4396 B · sha256 `E7FBA10F3332CA65FD3F81873AA2916916ADA14EA7F889BE8172C805C5B72CFC` (manifest MATCH) · Markdown · category `runtime-artifact` · depth `standard` · git-tracked: yes.

**Purpose:** Generated runtime-test artifact/log from the live server.

**Mechanics:** Non-source artifact captured during runs; included for provenance of the live test session.

**Dependencies/callers:** no in-partition imports; external: none detected.

**Quality (1–5):** correctness=3, maintainability=1, modularity=1, error_handling=1, security=2, docs=1, evidence=2. Flags: hardcoded-host, secret-ish, runtime-artifact.

**Evidence/UNVERIFIED:** read 83 lines; flagged at hardcoded-host:46, secret-ish:45.

---

## runtime-test/pkg-final/CHECKSUMS.txt

**Metadata:** 477 B · sha256 `9193A5B4F365B2945BA31E984FAC41C5C6AB650EC31A78859DCFA2EB80834051` (manifest MATCH) · Text · category `runtime-artifact` · depth `standard` · git-tracked: yes.

**Purpose:** Generated runtime-test artifact/log from the live server.

**Mechanics:** Non-source artifact captured during runs; included for provenance of the live test session.

**Dependencies/callers:** no in-partition imports; external: none detected.

**Quality (1–5):** correctness=3, maintainability=1, modularity=1, error_handling=1, security=2, docs=1, evidence=2. Flags: hardcoded-host, runtime-artifact.

**Evidence/UNVERIFIED:** read 7 lines; flagged at hardcoded-host:4.

---

## runtime-test/pkg-final/data/hiverouter.db

**Metadata:** 4096 B · sha256 `89A2318C76452BE80E1D743F163F3DD96623B2B05604AA06985C6B5501238362` (manifest MATCH) · SQLite · category `runtime-artifact` · depth `standard` · git-tracked: yes.

**Purpose:** SQLite runtime database produced by the shipped server during live QA; schema_version 3 previously observed; contains runtime/seed rows.

**Mechanics:** Binary SQLite file (+SHM/WAL companions if present); generated, not authored source.

**Dependencies/callers:** no in-partition imports; external: none detected.

**Quality (1–5):** correctness=4, maintainability=4, modularity=4, error_handling=4, security=4, docs=3, evidence=4. Flags: none.

**Evidence/UNVERIFIED:** binary/large file — content not read (metadata + role only); behaviour UNVERIFIED at byte level.

---

## runtime-test/pkg-final/data/hiverouter.db-shm

**Metadata:** 32768 B · sha256 `400EBFD736D798597F39E7FB278881D6E26A9A26950B5BB90A43D5023329A97B` (manifest MATCH) · SQLite-SHM · category `runtime-artifact` · depth `standard` · git-tracked: yes.

**Purpose:** SQLite runtime database produced by the shipped server during live QA; schema_version 3 previously observed; contains runtime/seed rows.

**Mechanics:** Binary SQLite file (+SHM/WAL companions if present); generated, not authored source.

**Dependencies/callers:** no in-partition imports; external: none detected.

**Quality (1–5):** correctness=4, maintainability=4, modularity=4, error_handling=4, security=4, docs=3, evidence=4. Flags: none.

**Evidence/UNVERIFIED:** binary/large file — content not read (metadata + role only); behaviour UNVERIFIED at byte level.

---

## runtime-test/pkg-final/data/hiverouter.db-wal

**Metadata:** 127752 B · sha256 `3622E5BBF27033FD4D2B9D28BE69FF237A365495B8C8D5A99D4B21321CE057D7` (manifest MATCH) · SQLite-WAL · category `runtime-artifact` · depth `standard` · git-tracked: yes.

**Purpose:** SQLite runtime database produced by the shipped server during live QA; schema_version 3 previously observed; contains runtime/seed rows.

**Mechanics:** Binary SQLite file (+SHM/WAL companions if present); generated, not authored source.

**Dependencies/callers:** no in-partition imports; external: none detected.

**Quality (1–5):** correctness=4, maintainability=4, modularity=4, error_handling=4, security=4, docs=3, evidence=4. Flags: none.

**Evidence/UNVERIFIED:** binary/large file — content not read (metadata + role only); behaviour UNVERIFIED at byte level.

---

## runtime-test/pkg-final/err.txt

**Metadata:** 0 B · sha256 `E3B0C44298FC1C149AFBF4C8996FB92427AE41E4649B934CA495991B7852B855` (manifest MATCH) · Text · category `runtime-artifact` · depth `standard` · git-tracked: yes.

**Purpose:** Generated runtime-test artifact/log from the live server.

**Mechanics:** Non-source artifact captured during runs; included for provenance of the live test session.

**Dependencies/callers:** no in-partition imports; external: none detected.

**Quality (1–5):** correctness=4, maintainability=4, modularity=4, error_handling=4, security=4, docs=3, evidence=4. Flags: none.

**Evidence/UNVERIFIED:** binary/large file — content not read (metadata + role only); behaviour UNVERIFIED at byte level.

---

## runtime-test/pkg-final/hiverouter.exe

**Metadata:** 9329152 B · sha256 `184D51A7C3CCA7B9D50954B677C5AD8A14472700A651FDF0D270441BE3EAC9A9` (manifest MATCH) · PE-binary · category `runtime-artifact` · depth `standard` · git-tracked: yes.

**Purpose:** Generated runtime-test artifact/log from the live server.

**Mechanics:** Non-source artifact captured during runs; included for provenance of the live test session.

**Dependencies/callers:** no in-partition imports; external: none detected.

**Quality (1–5):** correctness=4, maintainability=4, modularity=4, error_handling=4, security=4, docs=3, evidence=4. Flags: none.

**Evidence/UNVERIFIED:** binary/large file — content not read (metadata + role only); behaviour UNVERIFIED at byte level.

---

## runtime-test/pkg-final/out.txt

**Metadata:** 571 B · sha256 `8BF54E0F7B765DA4C55F4FC80D2466067637307C94119CE5340CA751663E9737` (manifest MATCH) · Text · category `runtime-artifact` · depth `standard` · git-tracked: yes.

**Purpose:** Generated runtime-test artifact/log from the live server.

**Mechanics:** Non-source artifact captured during runs; included for provenance of the live test session.

**Dependencies/callers:** no in-partition imports; external: none detected.

**Quality (1–5):** correctness=3, maintainability=1, modularity=1, error_handling=1, security=2, docs=1, evidence=2. Flags: hardcoded-host, runtime-artifact.

**Evidence/UNVERIFIED:** read 5 lines; flagged at hardcoded-host:3.

---

## runtime-test/pkg-final/README.md

**Metadata:** 1276 B · sha256 `CFA567329EDC9E6C52EF02C8620E2FD27C5917BC92C3EFC1794EA26DBF0BEBA3` (manifest MATCH) · Markdown · category `runtime-artifact` · depth `standard` · git-tracked: yes.

**Purpose:** Generated runtime-test artifact/log from the live server.

**Mechanics:** Non-source artifact captured during runs; included for provenance of the live test session.

**Dependencies/callers:** no in-partition imports; external: none detected.

**Quality (1–5):** correctness=3, maintainability=1, modularity=1, error_handling=1, security=2, docs=1, evidence=2. Flags: runtime-artifact.

**Evidence/UNVERIFIED:** read 25 lines.

---

## runtime-test/pkg-final/RUNBOOK.md

**Metadata:** 4396 B · sha256 `E7FBA10F3332CA65FD3F81873AA2916916ADA14EA7F889BE8172C805C5B72CFC` (manifest MATCH) · Markdown · category `runtime-artifact` · depth `standard` · git-tracked: yes.

**Purpose:** Generated runtime-test artifact/log from the live server.

**Mechanics:** Non-source artifact captured during runs; included for provenance of the live test session.

**Dependencies/callers:** no in-partition imports; external: none detected.

**Quality (1–5):** correctness=3, maintainability=1, modularity=1, error_handling=1, security=2, docs=1, evidence=2. Flags: hardcoded-host, secret-ish, runtime-artifact.

**Evidence/UNVERIFIED:** read 83 lines; flagged at hardcoded-host:46, secret-ish:45.

---

## runtime-test/pkg-final/smoke-data/hiverouter.db

**Metadata:** 4096 B · sha256 `89A2318C76452BE80E1D743F163F3DD96623B2B05604AA06985C6B5501238362` (manifest MATCH) · SQLite · category `runtime-artifact` · depth `standard` · git-tracked: yes.

**Purpose:** SQLite runtime database produced by the shipped server during live QA; schema_version 3 previously observed; contains runtime/seed rows.

**Mechanics:** Binary SQLite file (+SHM/WAL companions if present); generated, not authored source.

**Dependencies/callers:** no in-partition imports; external: none detected.

**Quality (1–5):** correctness=4, maintainability=4, modularity=4, error_handling=4, security=4, docs=3, evidence=4. Flags: none.

**Evidence/UNVERIFIED:** binary/large file — content not read (metadata + role only); behaviour UNVERIFIED at byte level.

---

## runtime-test/pkg-final/smoke-data/hiverouter.db-shm

**Metadata:** 32768 B · sha256 `114C6549E7E78B0BBB2C584E966E8127F7353602F5248A79104789C0DD1FEAB5` (manifest MATCH) · SQLite-SHM · category `runtime-artifact` · depth `standard` · git-tracked: yes.

**Purpose:** SQLite runtime database produced by the shipped server during live QA; schema_version 3 previously observed; contains runtime/seed rows.

**Mechanics:** Binary SQLite file (+SHM/WAL companions if present); generated, not authored source.

**Dependencies/callers:** no in-partition imports; external: none detected.

**Quality (1–5):** correctness=4, maintainability=4, modularity=4, error_handling=4, security=4, docs=3, evidence=4. Flags: none.

**Evidence/UNVERIFIED:** binary/large file — content not read (metadata + role only); behaviour UNVERIFIED at byte level.

---

## runtime-test/pkg-final/smoke-data/hiverouter.db-wal

**Metadata:** 127752 B · sha256 `4C9089758F4F0F34FC70DB1D2B0A28DB9DFAC9EEA4E83007ADE7EE330DFFA22F` (manifest MATCH) · SQLite-WAL · category `runtime-artifact` · depth `standard` · git-tracked: yes.

**Purpose:** SQLite runtime database produced by the shipped server during live QA; schema_version 3 previously observed; contains runtime/seed rows.

**Mechanics:** Binary SQLite file (+SHM/WAL companions if present); generated, not authored source.

**Dependencies/callers:** no in-partition imports; external: none detected.

**Quality (1–5):** correctness=4, maintainability=4, modularity=4, error_handling=4, security=4, docs=3, evidence=4. Flags: none.

**Evidence/UNVERIFIED:** binary/large file — content not read (metadata + role only); behaviour UNVERIFIED at byte level.

---

## runtime-test/pkg-final/smoke.err

**Metadata:** 0 B · sha256 `E3B0C44298FC1C149AFBF4C8996FB92427AE41E4649B934CA495991B7852B855` (manifest MATCH) · Log · category `runtime-artifact` · depth `standard` · git-tracked: yes.

**Purpose:** Generated runtime-test artifact/log from the live server.

**Mechanics:** Non-source artifact captured during runs; included for provenance of the live test session.

**Dependencies/callers:** no in-partition imports; external: none detected.

**Quality (1–5):** correctness=4, maintainability=4, modularity=4, error_handling=4, security=4, docs=3, evidence=4. Flags: none.

**Evidence/UNVERIFIED:** binary/large file — content not read (metadata + role only); behaviour UNVERIFIED at byte level.

---

## runtime-test/pkg-final/smoke.out

**Metadata:** 583 B · sha256 `6208165E1BB6BFD66E0329C66B6543071729AFA2016472058383159CC83C3211` (manifest MATCH) · Log · category `runtime-artifact` · depth `standard` · git-tracked: yes.

**Purpose:** Generated runtime-test artifact/log from the live server.

**Mechanics:** Non-source artifact captured during runs; included for provenance of the live test session.

**Dependencies/callers:** no in-partition imports; external: none detected.

**Quality (1–5):** correctness=3, maintainability=1, modularity=1, error_handling=1, security=2, docs=1, evidence=2. Flags: hardcoded-host, ad-hoc-probe, runtime-artifact.

**Evidence/UNVERIFIED:** read 5 lines; flagged at hardcoded-host:3.

---

## runtime-test/pkg-smoke/CHECKSUMS.txt

**Metadata:** 451 B · sha256 `C95EF561BF047DBDD341DEFED1907D4E684F472A13DA92EFBBF66C838CAD96E2` (manifest MATCH) · Text · category `runtime-artifact` · depth `standard` · git-tracked: yes.

**Purpose:** Generated runtime-test artifact/log from the live server.

**Mechanics:** Non-source artifact captured during runs; included for provenance of the live test session.

**Dependencies/callers:** no in-partition imports; external: none detected.

**Quality (1–5):** correctness=3, maintainability=1, modularity=1, error_handling=1, security=2, docs=1, evidence=2. Flags: hardcoded-host, runtime-artifact.

**Evidence/UNVERIFIED:** read 8 lines; flagged at hardcoded-host:5.

---

## runtime-test/pkg-smoke/data/hiverouter.db

**Metadata:** 4096 B · sha256 `89A2318C76452BE80E1D743F163F3DD96623B2B05604AA06985C6B5501238362` (manifest MATCH) · SQLite · category `runtime-artifact` · depth `standard` · git-tracked: yes.

**Purpose:** SQLite runtime database produced by the shipped server during live QA; schema_version 3 previously observed; contains runtime/seed rows.

**Mechanics:** Binary SQLite file (+SHM/WAL companions if present); generated, not authored source.

**Dependencies/callers:** no in-partition imports; external: none detected.

**Quality (1–5):** correctness=4, maintainability=4, modularity=4, error_handling=4, security=4, docs=3, evidence=4. Flags: none.

**Evidence/UNVERIFIED:** binary/large file — content not read (metadata + role only); behaviour UNVERIFIED at byte level.

---

## runtime-test/pkg-smoke/data/hiverouter.db-shm

**Metadata:** 32768 B · sha256 `B58903D1BFC4664F3B7E0BBC3AF564B1CC1C619E4C4B53371BDFEB010E10A0A7` (manifest MATCH) · SQLite-SHM · category `runtime-artifact` · depth `standard` · git-tracked: yes.

**Purpose:** SQLite runtime database produced by the shipped server during live QA; schema_version 3 previously observed; contains runtime/seed rows.

**Mechanics:** Binary SQLite file (+SHM/WAL companions if present); generated, not authored source.

**Dependencies/callers:** no in-partition imports; external: none detected.

**Quality (1–5):** correctness=4, maintainability=4, modularity=4, error_handling=4, security=4, docs=3, evidence=4. Flags: none.

**Evidence/UNVERIFIED:** binary/large file — content not read (metadata + role only); behaviour UNVERIFIED at byte level.

---

## runtime-test/pkg-smoke/data/hiverouter.db-wal

**Metadata:** 94792 B · sha256 `277C9B5895EBBB6C7E99DDE864376EBF3FCFC198AFBC2714A84E364023E4B0E0` (manifest MATCH) · SQLite-WAL · category `runtime-artifact` · depth `standard` · git-tracked: yes.

**Purpose:** SQLite runtime database produced by the shipped server during live QA; schema_version 3 previously observed; contains runtime/seed rows.

**Mechanics:** Binary SQLite file (+SHM/WAL companions if present); generated, not authored source.

**Dependencies/callers:** no in-partition imports; external: none detected.

**Quality (1–5):** correctness=4, maintainability=4, modularity=4, error_handling=4, security=4, docs=3, evidence=4. Flags: none.

**Evidence/UNVERIFIED:** binary/large file — content not read (metadata + role only); behaviour UNVERIFIED at byte level.

---

## runtime-test/pkg-smoke/err.txt

**Metadata:** 0 B · sha256 `E3B0C44298FC1C149AFBF4C8996FB92427AE41E4649B934CA495991B7852B855` (manifest MATCH) · Text · category `runtime-artifact` · depth `standard` · git-tracked: yes.

**Purpose:** Generated runtime-test artifact/log from the live server.

**Mechanics:** Non-source artifact captured during runs; included for provenance of the live test session.

**Dependencies/callers:** no in-partition imports; external: none detected.

**Quality (1–5):** correctness=4, maintainability=4, modularity=4, error_handling=4, security=4, docs=3, evidence=4. Flags: none.

**Evidence/UNVERIFIED:** binary/large file — content not read (metadata + role only); behaviour UNVERIFIED at byte level.

---

## runtime-test/pkg-smoke/hiverouter.exe

**Metadata:** 9099776 B · sha256 `71B229B587A909A6D8C2BDB9B8978B6087CECBF0609B176A9D46CA69717C38FC` (manifest MATCH) · PE-binary · category `runtime-artifact` · depth `standard` · git-tracked: yes.

**Purpose:** Generated runtime-test artifact/log from the live server.

**Mechanics:** Non-source artifact captured during runs; included for provenance of the live test session.

**Dependencies/callers:** no in-partition imports; external: none detected.

**Quality (1–5):** correctness=4, maintainability=4, modularity=4, error_handling=4, security=4, docs=3, evidence=4. Flags: none.

**Evidence/UNVERIFIED:** binary/large file — content not read (metadata + role only); behaviour UNVERIFIED at byte level.

---

## runtime-test/pkg-smoke/out.txt

**Metadata:** 571 B · sha256 `3EF24FC0C9F6D7DF41A666CDDEE32CE6F88EB977124DFE8AB45FF55A932803C7` (manifest MATCH) · Text · category `runtime-artifact` · depth `standard` · git-tracked: yes.

**Purpose:** Generated runtime-test artifact/log from the live server.

**Mechanics:** Non-source artifact captured during runs; included for provenance of the live test session.

**Dependencies/callers:** no in-partition imports; external: none detected.

**Quality (1–5):** correctness=3, maintainability=1, modularity=1, error_handling=1, security=2, docs=1, evidence=2. Flags: hardcoded-host, runtime-artifact.

**Evidence/UNVERIFIED:** read 5 lines; flagged at hardcoded-host:3.

---

## runtime-test/pkg-smoke/README.md

**Metadata:** 1276 B · sha256 `CFA567329EDC9E6C52EF02C8620E2FD27C5917BC92C3EFC1794EA26DBF0BEBA3` (manifest MATCH) · Markdown · category `runtime-artifact` · depth `standard` · git-tracked: yes.

**Purpose:** Generated runtime-test artifact/log from the live server.

**Mechanics:** Non-source artifact captured during runs; included for provenance of the live test session.

**Dependencies/callers:** no in-partition imports; external: none detected.

**Quality (1–5):** correctness=3, maintainability=1, modularity=1, error_handling=1, security=2, docs=1, evidence=2. Flags: runtime-artifact.

**Evidence/UNVERIFIED:** read 25 lines.

---

## runtime-test/pkg-smoke/RUNBOOK.md

**Metadata:** 4396 B · sha256 `E7FBA10F3332CA65FD3F81873AA2916916ADA14EA7F889BE8172C805C5B72CFC` (manifest MATCH) · Markdown · category `runtime-artifact` · depth `standard` · git-tracked: yes.

**Purpose:** Generated runtime-test artifact/log from the live server.

**Mechanics:** Non-source artifact captured during runs; included for provenance of the live test session.

**Dependencies/callers:** no in-partition imports; external: none detected.

**Quality (1–5):** correctness=3, maintainability=1, modularity=1, error_handling=1, security=2, docs=1, evidence=2. Flags: hardcoded-host, secret-ish, runtime-artifact.

**Evidence/UNVERIFIED:** read 83 lines; flagged at hardcoded-host:46, secret-ish:45.

---

## runtime-test/root.html

**Metadata:** 46 B · sha256 `71BC055475BEE0BE10CBC6A5B23DA158D9E7DD3C9D5C90C5CB975CB699CA5757` (manifest MATCH) · HTML · category `runtime-artifact` · depth `standard` · git-tracked: yes.

**Purpose:** Generated runtime-test artifact/log from the live server.

**Mechanics:** Non-source artifact captured during runs; included for provenance of the live test session.

**Dependencies/callers:** no in-partition imports; external: none detected.

**Quality (1–5):** correctness=3, maintainability=1, modularity=1, error_handling=1, security=2, docs=1, evidence=2. Flags: runtime-artifact.

**Evidence/UNVERIFIED:** read 1 lines.

---

## runtime-test/s5.err

**Metadata:** 0 B · sha256 `E3B0C44298FC1C149AFBF4C8996FB92427AE41E4649B934CA495991B7852B855` (manifest MATCH) · Log · category `runtime-artifact` · depth `standard` · git-tracked: yes.

**Purpose:** Generated runtime-test artifact/log from the live server.

**Mechanics:** Non-source artifact captured during runs; included for provenance of the live test session.

**Dependencies/callers:** no in-partition imports; external: none detected.

**Quality (1–5):** correctness=4, maintainability=4, modularity=4, error_handling=4, security=4, docs=3, evidence=4. Flags: none.

**Evidence/UNVERIFIED:** binary/large file — content not read (metadata + role only); behaviour UNVERIFIED at byte level.

---

## runtime-test/s5.out

**Metadata:** 553 B · sha256 `CB540A4A3A9CD322E44F591C5E3ABAEB3BE77D061E4214AE2A4EEAE159B1F046` (manifest MATCH) · Log · category `runtime-artifact` · depth `standard` · git-tracked: yes.

**Purpose:** Generated runtime-test artifact/log from the live server.

**Mechanics:** Non-source artifact captured during runs; included for provenance of the live test session.

**Dependencies/callers:** no in-partition imports; external: none detected.

**Quality (1–5):** correctness=3, maintainability=1, modularity=1, error_handling=1, security=2, docs=1, evidence=2. Flags: hardcoded-host, runtime-artifact.

**Evidence/UNVERIFIED:** read 5 lines; flagged at hardcoded-host:3.

---

## runtime-test/s6.err

**Metadata:** 0 B · sha256 `E3B0C44298FC1C149AFBF4C8996FB92427AE41E4649B934CA495991B7852B855` (manifest MATCH) · Log · category `runtime-artifact` · depth `standard` · git-tracked: yes.

**Purpose:** Generated runtime-test artifact/log from the live server.

**Mechanics:** Non-source artifact captured during runs; included for provenance of the live test session.

**Dependencies/callers:** no in-partition imports; external: none detected.

**Quality (1–5):** correctness=4, maintainability=4, modularity=4, error_handling=4, security=4, docs=3, evidence=4. Flags: none.

**Evidence/UNVERIFIED:** binary/large file — content not read (metadata + role only); behaviour UNVERIFIED at byte level.

---

## runtime-test/s6.out

**Metadata:** 4836 B · sha256 `EFAA576CC176079CFFDED6CE758E9FF4B2ED8E40C8CE0AAB128577DBE8597143` (manifest MATCH) · Log · category `runtime-artifact` · depth `standard` · git-tracked: yes.

**Purpose:** Generated runtime-test artifact/log from the live server.

**Mechanics:** Non-source artifact captured during runs; included for provenance of the live test session.

**Dependencies/callers:** no in-partition imports; external: none detected.

**Quality (1–5):** correctness=3, maintainability=1, modularity=1, error_handling=1, security=2, docs=1, evidence=2. Flags: hardcoded-host, runtime-artifact.

**Evidence/UNVERIFIED:** read 43 lines; flagged at hardcoded-host:3.

---

## runtime-test/s7.err

**Metadata:** 0 B · sha256 `E3B0C44298FC1C149AFBF4C8996FB92427AE41E4649B934CA495991B7852B855` (manifest MATCH) · Log · category `runtime-artifact` · depth `standard` · git-tracked: yes.

**Purpose:** Generated runtime-test artifact/log from the live server.

**Mechanics:** Non-source artifact captured during runs; included for provenance of the live test session.

**Dependencies/callers:** no in-partition imports; external: none detected.

**Quality (1–5):** correctness=4, maintainability=4, modularity=4, error_handling=4, security=4, docs=3, evidence=4. Flags: none.

**Evidence/UNVERIFIED:** binary/large file — content not read (metadata + role only); behaviour UNVERIFIED at byte level.

---

## runtime-test/s7.out

**Metadata:** 553 B · sha256 `F5FA99B678B606C6FA99797BEF53D853764A8F27CEB28892DF598D16DAD8C7A0` (manifest MATCH) · Log · category `runtime-artifact` · depth `standard` · git-tracked: yes.

**Purpose:** Generated runtime-test artifact/log from the live server.

**Mechanics:** Non-source artifact captured during runs; included for provenance of the live test session.

**Dependencies/callers:** no in-partition imports; external: none detected.

**Quality (1–5):** correctness=3, maintainability=1, modularity=1, error_handling=1, security=2, docs=1, evidence=2. Flags: hardcoded-host, runtime-artifact.

**Evidence/UNVERIFIED:** read 5 lines; flagged at hardcoded-host:3.

---

## runtime-test/s8.err

**Metadata:** 0 B · sha256 `E3B0C44298FC1C149AFBF4C8996FB92427AE41E4649B934CA495991B7852B855` (manifest MATCH) · Log · category `runtime-artifact` · depth `standard` · git-tracked: yes.

**Purpose:** Generated runtime-test artifact/log from the live server.

**Mechanics:** Non-source artifact captured during runs; included for provenance of the live test session.

**Dependencies/callers:** no in-partition imports; external: none detected.

**Quality (1–5):** correctness=4, maintainability=4, modularity=4, error_handling=4, security=4, docs=3, evidence=4. Flags: none.

**Evidence/UNVERIFIED:** binary/large file — content not read (metadata + role only); behaviour UNVERIFIED at byte level.

---

## runtime-test/s8.out

**Metadata:** 553 B · sha256 `40442AAFD3EA9E42128F091BC2E545BCDDD516537FA4F716411EC11A551864B1` (manifest MATCH) · Log · category `runtime-artifact` · depth `standard` · git-tracked: yes.

**Purpose:** Generated runtime-test artifact/log from the live server.

**Mechanics:** Non-source artifact captured during runs; included for provenance of the live test session.

**Dependencies/callers:** no in-partition imports; external: none detected.

**Quality (1–5):** correctness=3, maintainability=1, modularity=1, error_handling=1, security=2, docs=1, evidence=2. Flags: hardcoded-host, runtime-artifact.

**Evidence/UNVERIFIED:** read 5 lines; flagged at hardcoded-host:3.

---

## runtime-test/s9.err

**Metadata:** 0 B · sha256 `E3B0C44298FC1C149AFBF4C8996FB92427AE41E4649B934CA495991B7852B855` (manifest MATCH) · Log · category `runtime-artifact` · depth `standard` · git-tracked: yes.

**Purpose:** Generated runtime-test artifact/log from the live server.

**Mechanics:** Non-source artifact captured during runs; included for provenance of the live test session.

**Dependencies/callers:** no in-partition imports; external: none detected.

**Quality (1–5):** correctness=4, maintainability=4, modularity=4, error_handling=4, security=4, docs=3, evidence=4. Flags: none.

**Evidence/UNVERIFIED:** binary/large file — content not read (metadata + role only); behaviour UNVERIFIED at byte level.

---

## runtime-test/s9.out

**Metadata:** 553 B · sha256 `7F7FA6E1C65C12B9C322E4440540934CB1DBED40D85FD6CEEEB05AE25DD374B4` (manifest MATCH) · Log · category `runtime-artifact` · depth `standard` · git-tracked: yes.

**Purpose:** Generated runtime-test artifact/log from the live server.

**Mechanics:** Non-source artifact captured during runs; included for provenance of the live test session.

**Dependencies/callers:** no in-partition imports; external: none detected.

**Quality (1–5):** correctness=3, maintainability=1, modularity=1, error_handling=1, security=2, docs=1, evidence=2. Flags: hardcoded-host, runtime-artifact.

**Evidence/UNVERIFIED:** read 5 lines; flagged at hardcoded-host:3.

---

## runtime-test/server.err.txt

**Metadata:** 0 B · sha256 `E3B0C44298FC1C149AFBF4C8996FB92427AE41E4649B934CA495991B7852B855` (manifest MATCH) · Text · category `runtime-artifact` · depth `standard` · git-tracked: yes.

**Purpose:** Generated runtime-test artifact/log from the live server.

**Mechanics:** Non-source artifact captured during runs; included for provenance of the live test session.

**Dependencies/callers:** no in-partition imports; external: none detected.

**Quality (1–5):** correctness=4, maintainability=4, modularity=4, error_handling=4, security=4, docs=3, evidence=4. Flags: none.

**Evidence/UNVERIFIED:** binary/large file — content not read (metadata + role only); behaviour UNVERIFIED at byte level.

---

## runtime-test/server.out.txt

**Metadata:** 551 B · sha256 `11B42121DD424B017C047FD5ACC03475B550E657077EC1E296329924D5C09F86` (manifest MATCH) · Text · category `runtime-artifact` · depth `standard` · git-tracked: yes.

**Purpose:** Generated runtime-test artifact/log from the live server.

**Mechanics:** Non-source artifact captured during runs; included for provenance of the live test session.

**Dependencies/callers:** no in-partition imports; external: none detected.

**Quality (1–5):** correctness=3, maintainability=1, modularity=1, error_handling=1, security=2, docs=1, evidence=2. Flags: hardcoded-host, runtime-artifact.

**Evidence/UNVERIFIED:** read 5 lines; flagged at hardcoded-host:3.

---

## runtime-test/server2.err.txt

**Metadata:** 0 B · sha256 `E3B0C44298FC1C149AFBF4C8996FB92427AE41E4649B934CA495991B7852B855` (manifest MATCH) · Text · category `runtime-artifact` · depth `standard` · git-tracked: yes.

**Purpose:** Generated runtime-test artifact/log from the live server.

**Mechanics:** Non-source artifact captured during runs; included for provenance of the live test session.

**Dependencies/callers:** no in-partition imports; external: none detected.

**Quality (1–5):** correctness=4, maintainability=4, modularity=4, error_handling=4, security=4, docs=3, evidence=4. Flags: none.

**Evidence/UNVERIFIED:** binary/large file — content not read (metadata + role only); behaviour UNVERIFIED at byte level.

---

## runtime-test/server2.out.txt

**Metadata:** 553 B · sha256 `838F1917D697F9EBC1E80EDDDD45FA5F9B18C71F3960CC7A70D5434492E35AD1` (manifest MATCH) · Text · category `runtime-artifact` · depth `standard` · git-tracked: yes.

**Purpose:** Generated runtime-test artifact/log from the live server.

**Mechanics:** Non-source artifact captured during runs; included for provenance of the live test session.

**Dependencies/callers:** no in-partition imports; external: none detected.

**Quality (1–5):** correctness=3, maintainability=1, modularity=1, error_handling=1, security=2, docs=1, evidence=2. Flags: hardcoded-host, runtime-artifact.

**Evidence/UNVERIFIED:** read 5 lines; flagged at hardcoded-host:3.

---

## runtime-test/server3.err.txt

**Metadata:** 0 B · sha256 `E3B0C44298FC1C149AFBF4C8996FB92427AE41E4649B934CA495991B7852B855` (manifest MATCH) · Text · category `runtime-artifact` · depth `standard` · git-tracked: yes.

**Purpose:** Generated runtime-test artifact/log from the live server.

**Mechanics:** Non-source artifact captured during runs; included for provenance of the live test session.

**Dependencies/callers:** no in-partition imports; external: none detected.

**Quality (1–5):** correctness=4, maintainability=4, modularity=4, error_handling=4, security=4, docs=3, evidence=4. Flags: none.

**Evidence/UNVERIFIED:** binary/large file — content not read (metadata + role only); behaviour UNVERIFIED at byte level.

---

## runtime-test/server3.out.txt

**Metadata:** 553 B · sha256 `6B953D16D09BC692295C22B2D23707E5DEE75D0CB79C1F12A4E13BD6A3F2BC26` (manifest MATCH) · Text · category `runtime-artifact` · depth `standard` · git-tracked: yes.

**Purpose:** Generated runtime-test artifact/log from the live server.

**Mechanics:** Non-source artifact captured during runs; included for provenance of the live test session.

**Dependencies/callers:** no in-partition imports; external: none detected.

**Quality (1–5):** correctness=3, maintainability=1, modularity=1, error_handling=1, security=2, docs=1, evidence=2. Flags: hardcoded-host, runtime-artifact.

**Evidence/UNVERIFIED:** read 5 lines; flagged at hardcoded-host:3.

---

## runtime-test/server4.err.txt

**Metadata:** 0 B · sha256 `E3B0C44298FC1C149AFBF4C8996FB92427AE41E4649B934CA495991B7852B855` (manifest MATCH) · Text · category `runtime-artifact` · depth `standard` · git-tracked: yes.

**Purpose:** Generated runtime-test artifact/log from the live server.

**Mechanics:** Non-source artifact captured during runs; included for provenance of the live test session.

**Dependencies/callers:** no in-partition imports; external: none detected.

**Quality (1–5):** correctness=4, maintainability=4, modularity=4, error_handling=4, security=4, docs=3, evidence=4. Flags: none.

**Evidence/UNVERIFIED:** binary/large file — content not read (metadata + role only); behaviour UNVERIFIED at byte level.

---

## runtime-test/server4.out.txt

**Metadata:** 1230 B · sha256 `8C42F12E13BD62C24A454FFA1454358DC20C934FC8887A351261AED43600216B` (manifest MATCH) · Text · category `runtime-artifact` · depth `standard` · git-tracked: yes.

**Purpose:** Generated runtime-test artifact/log from the live server.

**Mechanics:** Non-source artifact captured during runs; included for provenance of the live test session.

**Dependencies/callers:** no in-partition imports; external: none detected.

**Quality (1–5):** correctness=3, maintainability=1, modularity=1, error_handling=1, security=2, docs=1, evidence=2. Flags: hardcoded-host, runtime-artifact.

**Evidence/UNVERIFIED:** read 11 lines; flagged at hardcoded-host:3.

---

## runtime-test/workspace/note.txt

**Metadata:** 20 B · sha256 `987F6FD43C751A63D17CE41EAD84EF493A389D1E5F91BF655FE629117D630F71` (manifest MATCH) · Text · category `runtime-artifact` · depth `standard` · git-tracked: yes.

**Purpose:** Workspace fixture file served by FILE_TREE_LIST/FILE_READ to the Data panel tests.

**Mechanics:** Plain text fixture (note.txt/README.md) created for live QA; content drives test assertions.

**Dependencies/callers:** no in-partition imports; external: none detected.

**Quality (1–5):** correctness=3, maintainability=1, modularity=1, error_handling=1, security=2, docs=1, evidence=2. Flags: runtime-artifact.

**Evidence/UNVERIFIED:** read 2 lines.

---

## runtime-test/workspace/README.md

**Metadata:** 44 B · sha256 `B0BD9D5596577EF6DBFFAB3E859EB64895742C1411DA9CBC14C09ACAE80A3313` (manifest MATCH) · Markdown · category `runtime-artifact` · depth `standard` · git-tracked: yes.

**Purpose:** Workspace fixture file served by FILE_TREE_LIST/FILE_READ to the Data panel tests.

**Mechanics:** Plain text fixture (note.txt/README.md) created for live QA; content drives test assertions.

**Dependencies/callers:** no in-partition imports; external: none detected.

**Quality (1–5):** correctness=3, maintainability=1, modularity=1, error_handling=1, security=2, docs=1, evidence=2. Flags: runtime-artifact.

**Evidence/UNVERIFIED:** read 3 lines.

---

## tools/bin/hiverouter-cli.exe

**Metadata:** 5364224 B · sha256 `51E1ABA6DC7349BFB8904014D7007015E13642EA9710F04D380525DCCBD9B9AD` (manifest MATCH) · PE-binary · category `generated-artifact` · depth `standard` · git-tracked: yes.

**Purpose:** Repository file assigned to the QA partition.

**Mechanics:** Static content; see scan metadata.

**Dependencies/callers:** no in-partition imports; external: none detected.

**Quality (1–5):** correctness=4, maintainability=4, modularity=4, error_handling=4, security=4, docs=3, evidence=4. Flags: none.

**Evidence/UNVERIFIED:** binary/large file — content not read (metadata + role only); behaviour UNVERIFIED at byte level.

---

## tools/bin/hiverouter.exe

**Metadata:** 9329152 B · sha256 `184D51A7C3CCA7B9D50954B677C5AD8A14472700A651FDF0D270441BE3EAC9A9` (manifest MATCH) · PE-binary · category `generated-artifact` · depth `standard` · git-tracked: yes.

**Purpose:** Compiled release HiveMind server binary shipped to the host; serves embedded UI on loopback and implements the WS command/event contract.

**Mechanics:** PE executable (9,329,152 B); executed externally as http://127.0.0.1:8799 for the final e2e runs.

**Dependencies/callers:** no in-partition imports; external: none detected.

**Quality (1–5):** correctness=4, maintainability=4, modularity=4, error_handling=4, security=4, docs=3, evidence=4. Flags: none.

**Evidence/UNVERIFIED:** binary/large file — content not read (metadata + role only); behaviour UNVERIFIED at byte level.

---

## tools/build_assets_index.py

**Metadata:** 3349 B · sha256 `34A725D984F4A39A1BD065D886F26AF2DB129555FC33F521257ED1C31C090D42` (manifest MATCH) · Python · category `source` · depth `standard` · git-tracked: yes.

**Purpose:** Repository file assigned to the QA partition.

**Mechanics:** Static content; see scan metadata.

**Dependencies/callers:** no in-partition imports; external: none detected.

**Quality (1–5):** correctness=4, maintainability=4, modularity=4, error_handling=4, security=4, docs=3, evidence=4. Flags: generated-marker.

**Evidence/UNVERIFIED:** read 68 lines; flagged at generated-marker:45.

---

## tools/build-release.ps1

**Metadata:** 2051 B · sha256 `2B465B0D05F1C7B3F6689C2625F43B35AA747EBD379F6BAA5F83D5395EC375E5` (manifest MATCH) · PowerShell · category `source` · depth `standard` · git-tracked: yes.

**Purpose:** Repository file assigned to the QA partition.

**Mechanics:** Static content; see scan metadata.

**Dependencies/callers:** no in-partition imports; external: none detected.

**Quality (1–5):** correctness=4, maintainability=4, modularity=4, error_handling=4, security=4, docs=3, evidence=4. Flags: none.

**Evidence/UNVERIFIED:** read 52 lines.

---

## tools/cc-test/r.exe

**Metadata:** 5088156 B · sha256 `DB801A07B6FD5194E84C49E68E9E640EF9DA51ECA2CE1B74656E6A016340C660` (manifest MATCH) · PE-binary · category `generated-artifact` · depth `standard` · git-tracked: yes.

**Purpose:** Repository file assigned to the QA partition.

**Mechanics:** Static content; see scan metadata.

**Dependencies/callers:** no in-partition imports; external: none detected.

**Quality (1–5):** correctness=4, maintainability=4, modularity=4, error_handling=4, security=4, docs=3, evidence=4. Flags: none.

**Evidence/UNVERIFIED:** binary/large file — content not read (metadata + role only); behaviour UNVERIFIED at byte level.

---

## tools/cc-test/r.rs

**Metadata:** 47 B · sha256 `0D2CAAE627FD973CFDDE04AC64960E6634B7A03E849EFA6D8467DA6DF52C3236` (manifest MATCH) · Rust · category `source` · depth `standard` · git-tracked: yes.

**Purpose:** Repository file assigned to the QA partition.

**Mechanics:** Static content; see scan metadata.

**Dependencies/callers:** no in-partition imports; external: none detected.

**Quality (1–5):** correctness=4, maintainability=4, modularity=4, error_handling=4, security=4, docs=3, evidence=4. Flags: none.

**Evidence/UNVERIFIED:** read 2 lines.

---

## tools/cc-test/t.c

**Metadata:** 70 B · sha256 `29838A3AAB694B42F1BED72FEB531E83CE280952382CDDE2D9F331683A464990` (manifest MATCH) · C · category `source` · depth `standard` · git-tracked: yes.

**Purpose:** Repository file assigned to the QA partition.

**Mechanics:** Static content; see scan metadata.

**Dependencies/callers:** no in-partition imports; external: none detected.

**Quality (1–5):** correctness=4, maintainability=4, modularity=4, error_handling=4, security=4, docs=3, evidence=4. Flags: none.

**Evidence/UNVERIFIED:** read 3 lines.

---

## tools/cc-test/t.exe

**Metadata:** 49876 B · sha256 `2840568E7B7D4C227ED05E3C4CD41E2932E3692636D6D88DF2ACA5A510F83A81` (manifest MATCH) · PE-binary · category `generated-artifact` · depth `standard` · git-tracked: yes.

**Purpose:** Repository file assigned to the QA partition.

**Mechanics:** Static content; see scan metadata.

**Dependencies/callers:** no in-partition imports; external: none detected.

**Quality (1–5):** correctness=4, maintainability=4, modularity=4, error_handling=4, security=4, docs=3, evidence=4. Flags: none.

**Evidence/UNVERIFIED:** binary/large file — content not read (metadata + role only); behaviour UNVERIFIED at byte level.

---

## tools/qa/apply-live-trace.mjs

**Metadata:** 2483 B · sha256 `D5E49DA85ECF22929B9FD2B0D246D5F8615BFD4587D7225A080A4338462C3EF6` (manifest MATCH) · JavaScript · category `source` · depth `standard` · git-tracked: yes.

**Purpose:** Merge tool: applies captured live trace verdicts from docs/trace/live/trace-capture.json into docs/trace/controls.json (evidence-backed only).

**Mechanics:** Writes controls.json with verdict/evidence; refuses to run without a capture file.

**Dependencies/callers:** no in-partition imports; external: `node:fs`, `node:url`, `node:path`.

**Quality (1–5):** correctness=2, maintainability=2, modularity=2, error_handling=2, security=4, docs=1, evidence=2. Flags: console.log, fs-write, generated-marker, ad-hoc-probe.

**Evidence/UNVERIFIED:** read 60 lines; flagged at console.log:59, fs-write:7, generated-marker:51.

---

## tools/qa/build-trace-seeds.mjs

**Metadata:** 3608 B · sha256 `E1C67D22AD002CA22848F1C9F48890E4237DDF618CDDCB89CC5A6A12C22DA99C` (manifest MATCH) · JavaScript · category `source` · depth `standard` · git-tracked: yes.

**Purpose:** Generator for docs/trace/{controls,traces,events}.json seed files from the control matrix.

**Mechanics:** Reads e2e/data/control-matrix.json and writes 11-hop PENDING records + empty trace/event vocabularies.

**Dependencies/callers:** no in-partition imports; external: `node:fs`, `node:url`, `node:path`.

**Quality (1–5):** correctness=4, maintainability=4, modularity=4, error_handling=4, security=4, docs=3, evidence=4. Flags: console.log, fs-write, generated-marker.

**Evidence/UNVERIFIED:** read 88 lines; flagged at console.log:85, fs-write:9, generated-marker:35.

---

## tools/qa/check-env.ps1

**Metadata:** 3747 B · sha256 `469514E2777207D1858CB452550A321EDD0DD84C7F508E9335C6F1B4E39BC30F` (manifest MATCH) · PowerShell · category `source` · depth `standard` · git-tracked: yes.

**Purpose:** Read-only host/toolchain evidence collector (node/npm/Playwright/browser versions, pins, project state, disk).

**Mechanics:** PowerShell script; prints key=value facts and pin comparisons for QA evidence.

**Dependencies/callers:** no in-partition imports; external: none detected.

**Quality (1–5):** correctness=4, maintainability=4, modularity=4, error_handling=4, security=4, docs=3, evidence=4. Flags: none.

**Evidence/UNVERIFIED:** read 69 lines.

---

## tools/qa/figcraft-assets.mjs

**Metadata:** 3556 B · sha256 `48FFA47248259DA4148EFE91A734B8DC9B2529AAFAE8832188E2811EF097D49D` (manifest MATCH) · JavaScript · category `source` · depth `standard` · git-tracked: yes.

**Purpose:** Read-only FigCraft asset inventory/report tool (sizes, sha256, manifest/A-design presence).

**Mechanics:** Reads docs/ui/figcraft/** and emits JSON; never mutates assets.

**Dependencies/callers:** no in-partition imports; external: `node:crypto`, `node:fs`, `node:url`, `node:path`.

**Quality (1–5):** correctness=4, maintainability=4, modularity=4, error_handling=4, security=4, docs=3, evidence=4. Flags: console.log, generated-marker.

**Evidence/UNVERIFIED:** read 75 lines; flagged at console.log:74, generated-marker:19.

---

## tools/qa/README.md

**Metadata:** 1200 B · sha256 `F8F253AAE35FE5020CDECB933D80CF8587E515558C8ACEC3C8E1D8CA639E4BA0` (manifest MATCH) · Markdown · category `other` · depth `standard` · git-tracked: yes.

**Purpose:** README/index for the tools/qa scripts.

**Mechanics:** Markdown reference; not executed.

**Dependencies/callers:** no in-partition imports; external: none detected.

**Quality (1–5):** correctness=4, maintainability=4, modularity=4, error_handling=4, security=4, docs=3, evidence=4. Flags: none.

**Evidence/UNVERIFIED:** read 14 lines.

---

## tools/qa/verify-harness.mjs

**Metadata:** 8167 B · sha256 `08B84C782F0E76737CDCE5628A8733669FAEDB6683782D4A155687463EFDDBD3` (manifest MATCH) · JavaScript · category `source` · depth `deep` · git-tracked: yes.

**Purpose:** Static read-only validator: frozen contract ↔ control matrix ↔ trace seeds consistency; exits 0/1/2.

**Mechanics:** Parses docs/contracts/UI-COMMANDS-v1.md sections, validates matrix surfaces/commands/events/refusals/hops/statuses and docs/trace JSON seeds; prints PASS/FAIL lines and a summary.

**Dependencies/callers:** no in-partition imports; external: `node:fs`, `node:url`, `node:path`.

**Quality (1–5):** correctness=4, maintainability=4, modularity=4, error_handling=4, security=4, docs=4, evidence=5. Flags: console.log.

**Deep analysis (8–12 paragraphs):**

1. Functional mechanics: a dependency-free Node CLI that reads four artefacts (frozen contract, control matrix, and three trace seed files), performs 27 named checks, prints PASS/FAIL per check plus a summary, and exits 0 on success, 1 on any failure, 2 on an uncaught file/parse error (lines 40-120, 122-131). It never launches a browser, server or network call; it is fully deterministic and safe to run in a read-only tree.

2. Contract parsing: section() (lines 30-36) slices the markdown between exact heading strings; commands are taken only from the first table column of §2, with combos split, and DIFF_* names are separated as decision-gated (lines 42-54). Events and refusals are extracted by backtick regex from §3/§5. This is adequate for the current contract shape, but it is a string protocol: renaming a heading returns an empty slice, and a re-worded table could silently change the parsed sets.

3. Threshold semantics: the first four checks assert lower bounds — frozen commands ≥10, DIFF ==3, events ≥12, refusals ≥30 (lines 56-59). The exact counts observed at audit time were 14 frozen commands, 21 events and 46 refusal codes. Lower bounds let vocabulary shrink or drift without failing the validator, which weakens its value as a frozen-contract guard; exact-set comparison against a checked-in expectation would be stronger.

4. Matrix validation: it checks array shape, exactly 28 guide IDs in S01..S28 order, that supplemental IDs (S-APPROVAL) match _meta.supplemental_controls, that every matrix command is frozen and every frozen command is covered, refusals and success events are frozen, the 11 hop names are exactly the ladder, statuses stay pre-evidence, mutating-command controls require ACK in events, and DIFF_* is never assumed frozen (lines 61-92). These checks caught real drifts earlier (uncovered VIEWPORT_RECENTER, missing S22 commands) and are the strongest part of the tool.

5. Trace-seed validation: for controls.json it requires one record per matrix surface, 11 hop keys each, and that every non-PENDING verdict carries non-empty evidence (lines 94-109); traces.json must contain a traces array; events.json must contain delivered[] and it must be empty until captured (lines 110-111). This enforces the no-fabrication rule structurally rather than trusting prose. It does not validate the live capture file (trace-capture.json) or that controls verdicts came from it.

6. FigCraft handling: the only expected-delivery check is a WARN when the canonical docs/ui/figcraft/ASSETS-INDEX.json is absent (lines 114-116); the warning records that A/A-ASSETS.json is a candidate needing acceptance. It cannot fail the run, which is appropriate for an external dependency outside QA control, but it means the validator can be green while the design-asset manifest delivery is incomplete — exactly the state at audit time.

7. Error handling: the whole body is wrapped in one try/catch that converts unexpected exceptions to exit 2 (lines 117-120); individual checks accumulate failures and continue, so one broken area does not hide others. There is no timeout, no child process, no filesystem write and no network — the failure modes are limited to missing files and malformed JSON. This matches the tool's read-only remit and is defensible under the production-quality bar.

8. Evidence: the validator was run repeatedly during the QA phase; the last recorded run is 27 checks, 27 pass, 0 fail, exit 0 on the shipped tree (documented in docs/audit/QA-ENV.md, row 8, and docs/trace/live/README.md). It is also invoked by e2e/package.json script verify:harness and is therefore the static gate used next to the browser suite. No unit tests exercise its parser/edge cases, so a future contract heading or type change is the main regression risk.

9. Dependency surface: only node:fs, node:url and node:path (lines 9-11); it reads docs/contracts/UI-COMMANDS-v1.md, e2e/data/control-matrix.json and docs/trace/{controls,traces,events}.json, and probes docs/ui/figcraft/ASSETS-INDEX.json. It imports nothing from e2e helpers, so it remains runnable when node_modules is absent — a deliberate and useful property for a static gate.

10. Risk summary for this file: S8 (lower-bound thresholds and heading-string parsing weaken frozen-vocabulary enforcement), S11 (no unit tests), and a scope boundary — it cannot verify behavioural wiring or live captures, only static structure. It is otherwise a compact, honest gate whose no-fake-pass checks are exactly the right shape for the audit's claims discipline.

**Evidence/UNVERIFIED:** read 132 lines; flagged at console.log:24.

---

## tools/runbook.md

**Metadata:** 4396 B · sha256 `E7FBA10F3332CA65FD3F81873AA2916916ADA14EA7F889BE8172C805C5B72CFC` (manifest MATCH) · Markdown · category `other` · depth `standard` · git-tracked: yes.

**Purpose:** Repository file assigned to the QA partition.

**Mechanics:** Static content; see scan metadata.

**Dependencies/callers:** no in-partition imports; external: none detected.

**Quality (1–5):** correctness=4, maintainability=4, modularity=4, error_handling=4, security=3, docs=3, evidence=4. Flags: hardcoded-host, secret-ish.

**Evidence/UNVERIFIED:** read 83 lines; flagged at hardcoded-host:46, secret-ish:45.

---

## tools/test.ps1

**Metadata:** 2531 B · sha256 `210AD6393AFD743234ADDB41184895092C15A370E4C4C56960170915CBA979A5` (manifest MATCH) · PowerShell · category `source` · depth `standard` · git-tracked: yes.

**Purpose:** Repository file assigned to the QA partition.

**Mechanics:** Static content; see scan metadata.

**Dependencies/callers:** no in-partition imports; external: none detected.

**Quality (1–5):** correctness=4, maintainability=4, modularity=4, error_handling=4, security=4, docs=3, evidence=4. Flags: none.

**Evidence/UNVERIFIED:** read 77 lines.

---

