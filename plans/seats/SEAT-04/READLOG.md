# READLOG — SEAT S4 (Live UX/E2E)

- Session start: 2026-11-06T17:06:44.002Z
- Mandatory order honored: paper (full) → 4 DeepSeek docs → 2 audit files → product slice, one file at a time. Metadata measured from disk at read time.

## audit/HIVEMIND-AUDIT-PAPER.md
- bytes: 25141 · sha256: `3BAAD198337564E1BD92ED23580A926426F92E2D4A9D64F260DBAF4D6713D495` · lines: 245 · read_at: 2026-11-06T17:06:44.002Z · tracked: true
- **Facts:** Subject HEAD 3988622/tag v1.0.0; 507 tracked files/63.98 MiB; 30 findings (3 P0/10 P1/16 P2/1 info); 101-task roadmap; e2e 79 pass/0 fail/14 skips. P0s: A-F01 provider unwired in shipped core (http.rs UnconfiguredExecutor), A-F02 dead replay (Canvas replayNodes=[]), A-F03 16 vs 15 state parity (missing interrupted). A-F05 Settings refusal source; A-F08 headline e2e receipts untracked; A-F09 82 tracked runtime-test artifacts; A-F13+ skip-as-green env guard, wiped test logs, trace completeness gaps, tests writing docs/, default baseURL mismatch. Seat watchdog-2 mean correctness 3.74/docs 2.51.

## DeepSeek docs — "Your First API Call" (quick start)
- source: https://api-docs.deepseek.com/ (HTTP 200, full body) · fetched_at: 2026-11-06T17:06:44.002Z
- **Facts:** base_url OpenAI https://api.deepseek.com, Anthropic /anthropic; models `deepseek-flash` (served by DeepSeek-V4.1-Flash-1017) and `deepseek-v4-pro`; API key from platform/env DEEPSEEK_API_KEY; POST /chat/completions example with messages, thinking{type:enabled}, reasoning_effort:high, stream:false; OpenAI/Anthropic-compatible.

## DeepSeek docs — "Models & Pricing" (pricing)
- source: https://api-docs.deepseek.com/quick_start/pricing (HTTP 200; fetched body contained the quick-start page, not a pricing table) · fetched_at: 2026-11-06T17:06:44.002Z
- **Facts:** No price numbers present in fetched body → pricing UNVERIFIED (needs a rendering fetch/official JSON). Model IDs deepseek-flash/deepseek-v4-pro restated.

## DeepSeek docs — "Thinking Mode"
- source: https://api-docs.deepseek.com/guides/thinking_mode (HTTP 200; body truncated mid tool-call sample) · fetched_at: 2026-11-06T17:06:44.002Z
- **Facts:** thinking toggle {"thinking":{"type":"enabled|disabled"}}; reasoning_effort low/high/max (minimal→low, medium/xhigh→high, ultra→max); thinking enabled by default effort high; reasoning_content alongside content; temperature/presence/frequency have no effect; top_p mode/model-dependent; with tools, previous reasoning_content must be passed back or 400; without tools it is ignored.

## DeepSeek docs — "Chat Completions API"
- source: https://api-docs.deepseek.com/api/create-chat-completion (HTTP 200; body truncated near end of response schema) · fetched_at: 2026-11-06T17:06:44.002Z
- **Facts:** POST /chat/completions; messages required (system/user/assistant/tool); model deepseek-flash|deepseek-v4-pro; thinking{type}; reasoning_effort none/low/high/max; max_tokens 1..384K (defaults 8K non-thinking, 64K thinking, 128K at max); response_format text|json_object; stream SSE data-only + [DONE]; stream_options.include_usage; finish_reason stop|length|content_filter|tool_calls|insufficient_system_resource|aborted; usage prompt/completion/total + cache hit/miss + reasoning_tokens; tool_choice required/named unsupported in thinking mode (400).

## audit/inventory/DEEP-LIST.md (context read)
- 56 deep files total; S4 assignment: `e2e/playwright.config.ts`, `e2e/specs/live-app.spec.ts`, `e2e/specs/trace-hops.spec.ts`, `tools/qa/verify-harness.mjs`.

## audit/diagrams/05-dependency-lineage.mmd
- bytes: 1200 · sha256: `93DB866C60C259770787D34C8F6C84E4717B19911766C62C16AFAAF0D878C350` · lines: 33 · read_at: 2026-11-06T17:06:44.002Z · tracked: true
- **Facts:** Rust deps (tokio 1.54.1/axum 0.8.9/reqwest 0.13.5/rusqlite 0.41.1/petgraph 0.8.3/rust_decimal/schemars/jsonschema/tracing/zeroize/win32job/prompty; wiremock dev-only) + Web deps (React 19.3.1, @xyflow/react 12.12.1, zustand 5.0.16, partysocket 1.3.0, motion, Vite 8.3.6/TS 7.0.2/Tailwind 4.3.4/vitest) feed shipped hiverouter.exe + hiverouter-cli.exe; labels Cargo.lock 405 packages and npm-lock 147.

## audit/tools/build_roadmap.py
- bytes: 7302 · sha256: `8FB5FF89F109D4E691D681EF6BF3B75E713CD86DBC7063797DC4897436FE1B10` · lines: 111 · read_at: 2026-11-06T17:06:44.002Z · tracked: true
- **Facts:** Generates ROADMAP-TASKS.csv + ROADMAP.md: PHASES (P1 15 tasks, P2 20, P3 20, P4 15, P5 15, P6 16 = 101 titles), SEATS 8; ROOT hardcoded absolute line 7; seat assigned by formula line 80; priority rule line 92; deliverable/acceptance maps lines 84-90.

## docs/evidence/live-provider-smoke.md
- bytes: 1312 · sha256: `9ED04F4581BA7500FD5398E046CEAADA9887C88AA8906E61F1145B1263E18EEB` · lines: 14 · read_at: 2026-11-06T17:06:44.002Z · tracked: true
- **Facts:** Tracked manual live evidence 2026-11-06: debug binary 127.0.0.1:8799 fresh data; key inherited env never printed; SUBMIT_PROMPT "Reply with exactly HIVE-LIVE-OK"; events RUN_CREATED→ACK→NODE_SPAWN×2→NODE_STATE→REPLY_DELTA→REPLY_DONE→NODE_STATE→RUN_STATUS; completed, no error; reply HIVE-LIVE-OK; usage prompt 89/completion 6/total 95 (cache miss 89, reasoning absent); real provider, no mock; planner hardened after empty-content; remaining gaps: leaf reply returned as one delta after non-streaming completion; true SSE delta publishing and replay UI still being completed (lines 12-13).

## e2e/playwright.config.ts
- bytes: 1917 · sha256: `E414A391049EB04C8A2865188C9457185FF525919BCF19FF4E5C83EDB1EA3698` · lines: 48 · read_at: 2026-11-06T17:06:44.002Z · tracked: true
- **Facts:** testDir ./specs, outputDir ./test-results, timeout 30s, expect 5s, fullyParallel false, workers=1, retries=0, forbidOnly, list+html reporters, headless installed Chrome/Edge channels only (no download; comments lines 2-8), trace retain-on-failure, screenshot only-on-failure, video off, no storageState; BASE_URL env default http://127.0.0.1:8080 line 17; comments document shipped :8799 + E2E_APP_READY=1 and no webServer lines 10-14.

## e2e/package.json
- bytes: 659 · sha256: `F5D3D35E1C3F4FCC395F2849E3F1F073EA4CE11BAF071CDB1F787122863C0A28` · lines: 22 · read_at: 2026-11-06T17:06:44.002Z · tracked: true
- **Facts:** private ESM; devDependency @playwright/test 1.65.0; scripts e2e/e2e:chrome/e2e:msedge + verify:harness/verify:figcraft/seeds calling ../tools/qa scripts; engines node >=22.12.0.

## e2e/specs/live-app.spec.ts
- bytes: 19833 · sha256: `CB3A5E8D70EF35F937D4ECDC3266D6A91265329720E9C5B1F385B8299BC7C54C` · lines: 335 · read_at: 2026-11-06T17:06:44.002Z · tracked: true
- **Facts:** 15 tests / 3 describes: boot/title/#root/skip-link; #hive-canvas target; zero console/pageerror; census nav6/tabs4/chat/composer/4 controls/HUD NO METRICS/scrubber disabled/library4/minimap/4 zoom; tabs+nav render-back; zoom+Ctrl0 transforms; CHAT_PIN ACK correlation; empty prompt E_PROMPT_EMPTY with zero SUBMIT frames; WS token/last_seq/hello/SERVER_HELLO; computed token colours contrast>=4.5; SETTINGS_STATUS soft asserts; no stale Data/Approval copy via bundle grep; FILE_TREE_RESULT note.txt/README.md render; unknown-size em dash not 0 B. Writes numbered evidence JSON/screenshots to docs/trace/live/live-app.

## e2e/specs/trace-hops.spec.ts
- bytes: 17640 · sha256: `03B8B6A4F1122F494BEF9DCFE8D227C189928771462942090B5639EF7D7E9270` · lines: 319 · read_at: 2026-11-06T17:06:44.002Z · tracked: true
- **Facts:** 28 per-surface capture (S01-S28; no S-APPROVAL), afterAll writes docs/trace/live/trace-capture.json (76-87); defaults-closed hopBase (59-73); node cards S15-S19 forced MISSING/UNVERIFIED (no paid run, 106-113); Settings branch captures ACK+SETTINGS_STATUS_RESULT but leaves 10_render_back UNVERIFIED (115-142); Data branch FILE_TREE_RESULT + render-back (160-190); tabs/nav/zoom assert render-back (217-272); composer/control render+gating only (274-297); fixed waits 350/800/1200/6000ms; BASE fallback 8799 line 14.

## e2e/specs/control-matrix.spec.ts
- bytes: 3755 · sha256: `E4B16D3E05BD57AEAB2DC460032B9A304AE8EF9044A704122879487076B38407` · lines: 86 · read_at: 2026-11-06T17:06:44.002Z · tracked: true
- **Facts:** Static integrity: 28 guide IDs + supplemental consistency, 11-hop vocabulary exact, frozen command/refusal cross-check, no pre-evidence PASS (6-70); 29 boot probes navigate / assert 200 + #root + body contains HiveMind (72-85).

## e2e/specs/a11y-gates.spec.ts
- bytes: 2769 · sha256: `9F5DE694757DCF044C2D6B553482F08290674CE2831AE56C5D7976C9190E8C4F` · lines: 53 · read_at: 2026-11-06T17:06:44.002Z · tracked: true
- **Facts:** 2 static contrast tests execute (six rows ±0.01, four recorded failures 9-24); 12 G-UI DOM gate tests are test.fixme, not executed (26-51).

## e2e/specs/ws-contract.spec.ts
- bytes: 2188 · sha256: `025EB2901FF4DD490F468CE75FF7AFD13F6553500570D55FA5BB1F34CBEF8599` · lines: 45 · read_at: 2026-11-06T17:06:44.002Z · tracked: true
- **Facts:** health/session ready+token real test (11-16); foreign-Origin refusal test fixme needing synthetic non-allowlisted page (18-28); hello/ACK/refusal scenario fixme duplicate of live-app (30-43).

## e2e/helpers/matrix.ts
- bytes: 2004 · sha256: `BC101B3505E1F98BB258CCA037790A6B660C4876316B60D73778870849C0D472` · lines: 74 · read_at: 2026-11-06T17:06:44.002Z · tracked: true
- **Facts:** Types + loadMatrix from e2e/data/control-matrix.json (36-40); appMissing() = process.env.E2E_APP_READY !== "1" (43-45) skip-as-green gate; skipReason stale "web/src is a milestone-1 shell" (47-52); FROZEN_COMMANDS 14 + NOT_FROZEN DIFF 3 (55-73).

## e2e/helpers/evidence.ts
- bytes: 2921 · sha256: `066430F0A36E9F9A63B555AD9858A3FFCE108A068CCB8CEBF6AC7CF7C672E482` · lines: 83 · read_at: 2026-11-06T17:06:44.002Z · tracked: true
- **Facts:** Evidence writers to docs/trace/live[/live-app] (LIVE_DIR/LIVE_APP_DIR lines 5-6; ensureLiveDirs/writeEvidence/shot 8-27); captureFrames/parseFrames/waitForFrame polling 50ms/6s (35-82); silently ignores non-JSON frames in waitForFrame (72-74) by design.

## e2e/helpers/contrast.ts
- bytes: 2214 · sha256: `646ED79DF15D2A303933194C4C8D9F51FFDD11D9B4FCCC1DCCCEC10C5ED0726A` · lines: 49 · read_at: 2026-11-06T17:06:44.002Z · tracked: true
- **Facts:** WCAG relative-luminance/contrast math (3-30); EXPECTED_CONTRAST 6 token rows (33-40); KNOWN_FAILURES 4 below-threshold pairs (43-48); throws on invalid hex.

## e2e/helpers/app.ts
- bytes: 2044 · sha256: `CB9AE9F946471B6FA443CD22AD31E618127EFBAD7BD5070AC7E8A2FF1F798F8C` · lines: 63 · read_at: 2026-11-06T17:06:44.002Z · tracked: true
- **Facts:** probeApp GET /health (200, status ok/degraded) then /api/session (200; token presence only, never logs token) (18-54); wsUrl builder (57-62).

## e2e/helpers/ws-tap.ts
- bytes: 1790 · sha256: `4E667F62D942B736C4F452B5A01F86B0DF21AFA49F36FF54E1F1E6FE3E1069F8` · lines: 61 · read_at: 2026-11-06T17:06:44.002Z · tracked: true
- **Facts:** tapWebSockets captures frames+sockets, waitFor polling default 5s, close() teardown ignoring close errors (21-60); comment states ACK-or-typed-ERROR before UI mutation (16-20).

## e2e/helpers/trace.ts
- bytes: 1299 · sha256: `404CE7202A8C8A5557C88CA7F132D2C3C4CF8F05468F3F211A5E7F5396664E97` · lines: 38 · read_at: 2026-11-06T17:06:44.002Z · tracked: true
- **Facts:** appendTrace appends TraceRun to docs/trace/traces.json with caller-supplied evidence (27-33); HopRecord/TraceRun types (6-19); readControls (35-37); currently no caller found in specs (UNUSED helper risk).

## web/src/App.tsx
- bytes: 3961 · sha256: `BC93C009D101C0409458ABB758E606C4219A8CD7ACA9149452587CE457175224` · lines: 122 · read_at: 2026-11-06T17:06:44.002Z · tracked: true
- **Facts:** Root: one store per mount; AppFrame selects nav content (Agents/Data/Models/Routing/Settings/workspace) and tab view (routing panel; history=Canvas only; default Canvas+ChatCore) (47-74); skip link #hive-canvas (78-80); NavRail/Header/mock banner only apiKind==="mock"/connection alert role=alert/ControlBar+Scrubber only workspace/LibraryPanel/ToastRegion/ApprovalOverlay/DiffDecisionModal (76-103); init()+dispose (106-121).

## web/src/App.tsx
- bytes: 3961 · sha256: `BC93C009D101C0409458ABB758E606C4219A8CD7ACA9149452587CE457175224` · lines: 122 · read_at: 2026-11-06T17:06:44.002Z · tracked: true
- **Facts:** Root: one store per mount; AppFrame selects nav content (Agents/Data/Models/Routing/Settings/workspace) and tab view (routing panel; history=Canvas only; default Canvas+ChatCore) (47-74); skip link #hive-canvas (78-80); NavRail/Header/mock banner only apiKind==="mock"/connection alert role=alert/ControlBar+Scrubber only workspace/LibraryPanel/ToastRegion/ApprovalOverlay/DiffDecisionModal (76-103); init()+dispose (106-121).

## web/src/panels/AgentsPanel.tsx
- bytes: 2038 · sha256: `0A452EE172371E85A59D28BE600A0D0817CCB9B826E51EFAEE8CC99A06B8210F` · lines: 48 · read_at: 2026-11-06T17:06:44.002Z · tracked: true
- **Facts:** S15-S19 registry from store.tasks; per-task button aria-pressed selected (23-41); state glyph; progress null => em dash (39); empty copy (19); no fake data.

## web/src/panels/ApprovalOverlay.tsx
- bytes: 2857 · sha256: `EB1595B3150B400D9CF7FB0BE3DCBD189B242B6FA7D7CC8DD965FC87030A37BF` · lines: 67 · read_at: 2026-11-06T17:06:44.002Z · tracked: true
- **Facts:** Pending approvals modal dialogs; nonce===null shows named gap and disables Approve/Deny fail-closed (30-55); resolveApproval(approvalId, decision, reason); comment "patched contract: APPROVAL_RESOLVE with server nonce" (line 1).

## web/src/panels/ControlBar.tsx
- bytes: 5023 · sha256: `A22B87B6D259C2FB9B4DC34D0449AE5D7CA9DB8AA2F720FCD230FC5346FACF92` · lines: 126 · read_at: 2026-11-06T17:06:44.002Z · tracked: true
- **Facts:** S04-S07 spawn/reroute/cancel/kill gated by connected && runActive && !pending plus worker selection (24-37); disabled unless preconditions; dispatch store actions; pending/terminal status line aria-live (116-122).

## web/src/panels/DataPanel.tsx
- bytes: 3077 · sha256: `58A77EDED98E06776F43F50D039F1E4FA2029C522BDE9EC145B51A85FB383F12` · lines: 71 · read_at: 2026-11-06T17:06:44.002Z · tracked: true
- **Facts:** FILE_TREE_LIST on mount; file preview via FILE_READ; size null => em dash, kind dir "/" (line 44); loading/error/empty copy; no fake tree.

## web/src/panels/DevGallery.tsx
- bytes: 1044 · sha256: `5D6645467E54865F50E989613708D5171BC4DB0B0E26761FCCDA27F53140D7FC` · lines: 26 · read_at: 2026-11-06T17:06:44.002Z · tracked: true
- **Facts:** DEV-only (import.meta.env.DEV line 8) state-pack gallery over AGENT_STATES; never in prod build.

## web/src/panels/DiffDecisionModal.tsx
- bytes: 1265 · sha256: `6C2DDD6BFD6F800C57F01CBEE0F3C0A16C10E902D7E95ECF8A749A33337412D0` · lines: 35 · read_at: 2026-11-06T17:06:44.002Z · tracked: true
- **Facts:** Decision-gated S25 modal; shows decisionRequired copy and close only (5-32); no DIFF_* command call.

## web/src/panels/Header.tsx
- bytes: 3268 · sha256: `8285642357B7B8F660075742A3E8CA58E5AAD991545777E3B9BEBAA0241394BA` · lines: 81 · read_at: 2026-11-06T17:06:44.002Z · tracked: true
- **Facts:** S08 tabs (4) role=tablist aria-selected; S09 HUD derived counts/percent + DerivedBadge; S28 project identity; connection pill; no fabricated metrics.

## web/src/panels/LibraryPanel.tsx
- bytes: 2994 · sha256: `85B059387AC942ADC44980CB698AE338522B94927F9709382CF98B24429D114E` · lines: 74 · read_at: 2026-11-06T17:06:44.002Z · tracked: true
- **Facts:** S12 project identity, derived active agents, 4 template buttons dispatching insertTemplate (50-58), FigCraft gate copy, tip; xl-only aside.

## web/src/panels/ModelsPanel.tsx
- bytes: 1195 · sha256: `231A22A7228E632CF4DF6E937A1448031EAB6220A1855B44EFB105F92A5F4858` · lines: 25 · read_at: 2026-11-06T17:06:44.002Z · tracked: true
- **Facts:** Read-only provider/model from settings; empty state when both null (11-13); contract note labelled; no invented models.

## web/src/panels/NavRail.tsx
- bytes: 2251 · sha256: `DC54427F99FC00972A99DAA8C824C67AA5BFD1FA8F366C87F7BD74D3C91BF94E` · lines: 62 · read_at: 2026-11-06T17:06:44.002Z · tracked: true
- **Facts:** S11 72px nav, 6 destinations, aria-current, roving ArrowUp/Down focus (22-59); client-state only.

## web/src/panels/RoutingPanel.tsx
- bytes: 1888 · sha256: `F0269705A18A93B587EB5C9B887737E4DEFE3F5C7453DCBEB14F06103FFEB1B9` · lines: 40 · read_at: 2026-11-06T17:06:44.002Z · tracked: true
- **Facts:** Derived table from task.parentId edges (9); empty copy (13); from/to/kind/state columns (14-36); no synthetic routes.

## web/src/panels/Scrubber.tsx
- bytes: 1782 · sha256: `1D491793C3F8179994BE7C58AE9909C7168CBED670DE81071166B90AC2225824` · lines: 53 · read_at: 2026-11-06T17:06:44.002Z · tracked: true
- **Facts:** S10 range input commits HISTORY_REVISION_SET on pointer/key release (11-24); disabled when latest==0 (32); live button (42-48); aria-label Time-travel scrubber.

## web/src/panels/SettingsPanel.tsx
- bytes: 5615 · sha256: `EB270A7C3DF66E25C7BA08B48EFE3E088DCB271002EA571A0F76273032B9C948` · lines: 129 · read_at: 2026-11-06T17:06:44.002Z · tracked: true
- **Facts:** S21 probes SETTINGS_STATUS + health on mount; renders provider/model/key present-missing-unknown; reducedMotion override; opens code-diff modal; DEV gallery; StatePack loading/error.

## web/src/panels/StatePack.tsx
- bytes: 1816 · sha256: `ED3FBDC6EBE78C27BA69CB454C38E7B2A19803EC5F6649F016B71EADF730C677` · lines: 53 · read_at: 2026-11-06T17:06:44.002Z · tracked: true
- **Facts:** Shared PanelTitle/EmptyState/LoadingState(role=status)/ErrorState(role=alert)/ReplayBanner/DerivedBadge primitives (5-52); text from STRINGS.

## web/src/panels/ToastRegion.tsx
- bytes: 2072 · sha256: `403D24C553BF9F07E7EBA5A94671AA00F98D20CE31DC7EA71B8D4FA3ED6E8A4A` · lines: 50 · read_at: 2026-11-06T17:06:44.002Z · tracked: true
- **Facts:** S14 assertive live region aria-label "Refusals and errors"; renders REFUSED BY NAME · NOT SILENT, code, message, how_to_fix, dismiss (12-47); role=alert entries.

## web/src/panels/ChatCore.tsx
- **STATUS: MISSING** — assigned by slice; no such path on disk as of 2026-11-06T17:06:44.002Z. Recorded, not inferred.

## tools/qa/verify-harness.mjs
- bytes: 8167 · sha256: `08B84C782F0E76737CDCE5628A8733669FAEDB6683782D4A155687463EFDDBD3` · lines: 132 · read_at: 2026-11-06T17:06:44.002Z · tracked: true
- **Facts:** Read-only static validator: parses contract §2/§3/§5; validates matrix (28 guide IDs, supplemental, command/refusal/event coverage, 11 hops, statuses, ACK rule, DIFF forbidden), trace seeds (controls count/hops/evidence-backed verdicts, traces array, delivered empty), WARN-only absent canonical FigCraft ASSETS-INDEX; exit 0/1/2; lower-bound thresholds >=10 commands/>=12 events/>=30 refusals (56-59); heading-substring section() (30-36).

## tools/qa/build-trace-seeds.mjs
- bytes: 3608 · sha256: `E1C67D22AD002CA22848F1C9F48890E4237DDF618CDDCB89CC5A6A12C22DA99C` · lines: 88 · read_at: 2026-11-06T17:06:44.002Z · tracked: true
- **Facts:** Regenerates docs/trace/controls.json (from matrix surfaces, all 11 hops PENDING, 29-47), traces.json empty (49-57), events.json vocabulary + parity skeleton (59-79); overwrites seeds only; comment says 28 controls (4-5) though matrix now has 29 incl S-APPROVAL.

## tools/qa/apply-live-trace.mjs
- bytes: 2483 · sha256: `D5E49DA85ECF22929B9FD2B0D246D5F8615BFD4587D7225A080A4338462C3EF6` · lines: 60 · read_at: 2026-11-06T17:06:44.002Z · tracked: true
- **Facts:** Merges docs/trace/live/trace-capture.json into controls.json: only captured non-UNVERIFIED hops get verdict+evidence (24-45); status PARTIALLY_VERIFIED when present else PENDING; writes JSON summary; depends on capture and PENDING seed ordering.

## tools/qa/figcraft-assets.mjs
- bytes: 3556 · sha256: `48FFA47248259DA4148EFE91A734B8DC9B2529AAFAE8832188E2811EF097D49D` · lines: 75 · read_at: 2026-11-06T17:06:44.002Z · tracked: true
- **Facts:** Read-only FigCraft inventory: exports/boards/manifest candidates/A-design presence/hash/CLI present (18-72); status COMPLETE/PARTIAL_MANIFEST_CANDIDATE/BLOCKED_PARTIAL; hardcoded CLI path line 14 and board names line 44.

## tools/qa/check-env.ps1
- bytes: 3747 · sha256: `469514E2777207D1858CB452550A321EDD0DD84C7F508E9335C6F1B4E39BC30F` · lines: 69 · read_at: 2026-11-06T17:06:44.002Z · tracked: true
- **Facts:** Read-only evidence dump: node/npm/playwright versions, installed Chrome/Edge versions, web src count, package/node_modules presence, crate list, git HEAD, disk, blueprint pin hashes, FigCraft assets/manifest/design/CLI presence, docs/trace+docs/audit file names (1-68).

## tools/qa/README.md
- bytes: 1200 · sha256: `F8F253AAE35FE5020CDECB933D80CF8587E515558C8ACEC3C8E1D8CA639E4BA0` · lines: 14 · read_at: 2026-11-06T17:06:44.002Z · tracked: true
- **Facts:** Documents 5 scripts + run commands; claims scripts run before app exists and never fabricate evidence (3,12-13); claims build-trace-seeds is the only docs/trace writer (line 12) while apply-live-trace also writes controls; exit codes documented.

## web/src/hooks/useAppReducedMotion.ts
- **STATUS: MISSING** — assigned by slice; no such path on disk as of 2026-11-06T17:06:44.002Z. Recorded, not inferred.

## Coverage summary
- Product/audit files recorded with measured metadata: 40 (+1 context list + 1 missing directory web/src/hooks).
- 4 DeepSeek docs pages fetched; 2 bodies truncated by web_fetch (pricing absent, thinking/chat tails cut) → tails UNVERIFIED.
- No product/git/config/key/paid writes; all writes under plans/seats/SEAT-04/.
