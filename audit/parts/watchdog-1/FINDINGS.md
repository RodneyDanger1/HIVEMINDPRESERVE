# FINDINGS — watchdog-1 partition (`web/**`, 55 files)

Read-only audit at HEAD `3988622`; 55/55 manifest hashes match. Severity: **HIGH** = advertised surface/contract broken or unsafe; **MEDIUM** = wrong/risky behaviour or maintainability with real cost; **LOW** = debt/coverage/hygiene. No product/git/rebuild writes were made.

## High

**F-01 (HIGH) — History replay surface is dead: HISTORY_SNAPSHOT graph is stored but never rendered.**
`web/src/flow/Canvas.tsx:113` defines `const replayNodes = useMemo<Node[]>(() => [], []);` and lines 126-127 switch to `nodes={history.replayGraph ? replayNodes : nodes}` / `edges=[]`; the SCRUBBER sends `HISTORY_REVISION_SET` and the store stores the graph (`web/src/state/store.ts:379-385`), but no projection to React Flow exists. Result: the S10 time-travel scrubber shows a banner and an empty canvas — the surface is not wired-and-working.
*Recommendation:* parse `HISTORY_SNAPSHOT.graph` in a store selector (validated schema), build replay nodes/edges via `flow/graph.ts` equivalents, render read-only (no drag/rewire), add a unit test for the projection and an e2e replay assertion.

**F-02 (HIGH) — Settings error can display an unrelated refusal.**
`web/src/state/store.ts:656-670` (`probeSettings`) sets `refusal: get().refusals[0]?.refusal ?? null` after a failed SETTINGS_STATUS; `refusals[0]` is the most recent toast of *any* action (e.g. E_KEY_MISSING from a submit), not necessarily the settings probe's ERROR. The panel renders that refusal verbatim (`web/src/panels/SettingsPanel.tsx:78-91`).
*Recommendation:* capture the refusal from the failing command itself (pending Promise rejection payload) instead of reading the toast list; test with two concurrent failures.

## Medium

**F-03 (MEDIUM) — Contract duplication across TypeScript/Rust can drift.**
`web/src/api/contract.ts` re-implements the wire vocabulary owned by `backend/crates/hiverouter-contract/src/{envelope,command,event,refusal,state}.rs`; the shared authority is only the doc `docs/contracts/UI-COMMANDS-v1.md`. No generated schema/types bridge exists. Evidence: duplicate constants (40 refusal codes client-side vs Rust `Refusal`), 14/21 name unions.
*Recommendation:* generate TS types/name unions from `backend/crates/hiverouter-contract/schemas/contract-v1.json` (already shipped) and gate in CI.

**F-04 (MEDIUM) — Font faces are declared but not bundled (R-U12 unmet).**
`web/src/styles/tokens.css:30-32` names "Playfair Display Variable"/"Inter Variable"/Consolas, but `web/package.json:17-34` contains no @fontsource/font package and `web/index.html` loads none. On most hosts the display face falls back to Georgia/system-serif, so brand/type fidelity is UNVERIFIED; no provenance claim should be made.
*Recommendation:* add `@fontsource-variable/playfair-display` + `@fontsource-variable/inter` (OFL) with licence notes, import in `main.tsx`, or drop the R-U12 bundled-face claim explicitly.

**F-05 (MEDIUM) — Dev-only mock is statically imported by the production entry.**
`web/src/api/index.ts:8` imports `MockApiClient`; lines 15-16 construct it only under `import.meta.env.DEV && VITE_UI_MOCK === '1'`, and `web/src/api/mock.ts:68-69` throws in PROD. Runtime is safe (no fabricated data can be produced in production), but the mock code is likely included in the production chunk (no dynamic import), increasing bundle weight and audit surface.
*Recommendation:* `const { MockApiClient } = await import('./mock')` inside a DEV-only branch; verify absence in the built bundle.

**F-06 (MEDIUM) — No HTTP timeout/abort for session or health fetch.**
`web/src/api/client.ts:235` awaits `/api/session` with no AbortController/timeout, and `web/src/api/health.ts:15-22` does the same for `/health`; a hung HTTP connection blocks `open()` or the Settings health probe indefinitely (WS commands have timeouts at `client.ts:443`, HTTP does not).
*Recommendation:* AbortSignal.timeout with a named `E_WS_DISCONNECTED` refusal and a Settings-specific failure state; unit-test the timeout path.

**F-07 (MEDIUM) — Token-isolation test does not cover `index.html`; HTML contains hex literals.**
`web/index.html:9` embeds a data-URI favicon with `#282010`/`#F0C050`; `web/tests/noHardcoded.test.ts:4-15` globs only `src/**/*.{ts,tsx}` and `src/**/*.css` with the token allowlist, so the shipped HTML shell bypasses G-UI-11. (The favicon was added by a later closeout seat; this audit only records it.)
*Recommendation:* move the favicon to a token-derived SVG asset or extend the isolation gate to HTML with the same allowlist; keep G-UI-11 scope explicit.

**F-08 (MEDIUM) — Approval nonce is an optional-extension field, not in the frozen APPROVAL_REQUEST shape.**
`web/src/api/contract.ts` `ApprovalRequestPayload` declares optional `nonce`, and `web/src/panels/ApprovalOverlay.tsx` disables resolution when absent; the patched frozen doc lists `APPROVAL_RESOLVE(nonce)` but the event row (`docs/contracts/UI-COMMANDS-v1.md:44-45`) does not list a nonce field. Without a server-provided nonce the overlay can never resolve — honest, but the contract gap must be closed server-side.
*Recommendation:* add `nonce` to the frozen APPROVAL_REQUEST schema + Rust event struct and keep the UI gap copy as the fallback.

**F-09 (MEDIUM) — No component/DOM tests: UI behaviour behind panels is UNVERIFIED by unit tests.**
`web/tests/*` cover pure modules/store with a fake API (`web/tests/store.test.ts`, `apiClient.test.ts`, etc.); no jsdom/RTL test mounts `ChatCore`, `Canvas`, panels or overlays. Browser behaviour is asserted only by the separate `e2e/**` seat and the FINAL-REPORT gate row.
*Recommendation:* add RTL/jsdom mount tests for the composer, refusal toast, approval overlay and settings probe; keep e2e as the integration layer.

**F-10 (MEDIUM) — Settings status can remain `pending` forever if the result event never arrives.**
`web/src/state/store.ts:656-664` sets `pending` before ACK and only `error` on failure; a successful ACK without SETTINGS_STATUS_RESULT leaves the panel's loading state until remount. There is no per-command result timeout.
*Recommendation:* clear to a named `E_RESYNC_REQUIRED`/timeout state after N seconds and test the missing-result path.

**F-11 (MEDIUM) — Composer clears the draft before knowing whether the send was accepted.**
`web/src/chat/ChatCore.tsx:51-55` and `:131-136` do `setDraft('')` then `void submitPrompt(text)`; client-side refusals (E_PROMPT_EMPTY/E_PROMPT_TOO_LONG) leave the user with an empty composer and only a toast.
*Recommendation:* make `submitPrompt` return a result (accepted/refused) and only clear on acceptance, or restore the draft on refusal.

## Low

**F-12 (LOW) — Canvas minimap a11y and shortcut are imperative DOM bridges.**
`web/src/flow/Canvas.tsx:91` adds a document capture keydown listener, `:100-102` queries `.react-flow__minimap` and sets attributes directly; there is no React ref/component wrapper, so the bridge bypasses the test suite and would break under a MiniMap DOM change.
*Recommendation:* wrap MiniMap in a ref'd component and test the attribute application in jsdom.

**F-13 (LOW) — Data panel error copy promises a Refresh control it does not render.**
`web/src/panels/DataPanel.tsx:23` shows `STRINGS.panels.dataError` ("retry with Refresh") but the panel exposes no refresh button; `requestFileTree` is wired only on idle mount (`:12-16`). After an error, remaining re-entry does not refetch.
*Recommendation:* add a visible Retry button and relax the idle-only effect guard.

**F-14 (LOW) — Client listener leak in the WS open path.**
`web/src/api/client.ts:389-407` (`waitForOpen`) registers an `onOpen` listener and never removes it; `attach` also registers an onOpen listener on every open (`:289`). Bounded by connection count but unnecessary.
*Recommendation:* return a cleanup from waitForOpen/once-semantics in Transport, or use a single listener with a resolved flag.

**F-15 (LOW) — Bundle is one 669 kB JS chunk; no code-splitting configured.**
`web/vite.config.ts:20-27` has no `manualChunks`/dynamic imports; build evidence (FINAL-REPORT) records a >500 kB warning. React Flow + motion + dagre dominate the single chunk.
*Recommendation:* lazy-load React Flow canvas and the mock, split vendor chunks; re-run the size gate.

**F-16 (LOW) — No lint/format gate.**
`web/package.json:11-15` defines only dev/typecheck/build/preview/test; no eslint/prettier/biome script or config in the 55 files, so style/dead-code rules are not machine-enforced (typecheck strictness is the only static gate).
*Recommendation:* add ESLint (typescript-eslint) with a no-unused/no-explicit-any gate; wire into typecheck CI.

**F-17 (LOW) — App init/dispose race under StrictMode double-mount (DEV).**
`web/src/App.tsx:109-112` calls `init()` then cleanup `dispose()`; `store.init()` sets `initialized` synchronously and `dispose()` closes the API, so the remount early-returns while the first handshake may still be in flight (subscribers removed, transport closed). Production is unaffected (StrictMode double-invoke is dev-only), hence UNVERIFIED/low.
*Recommendation:* ref-count init/dispose or move connection ownership above StrictMode remounts; verify in dev.

**F-18 (LOW) — TypeScript `any`-free but 3 occurrences of the word only in comments; no explicit-any policy.**
Content scan found 0 `any` type annotations in the 55 files (word matches only in comments such as `web/src/api/contract.ts:410`); nevertheless there is no lint rule pinning this property.
*Recommendation:* add `@typescript-eslint/no-explicit-any` so the property is enforced, not incidental.

## Cross-partition notes (not defects in this seat)

- Contract/runtime truth is owned by `backend/**`; `web/**` only consumes the frozen contract. The served bundle was hash-identical to `web/dist` at closeout (index `AD1BD1E6…`, JS `056ECEEF…`, CSS `96FFA873…`), so shipped UI == source build for the audited checkpoint.
- `e2e/**` owns browser-level verification (Playwright); this audit did not run a browser.
- `web/dist`, `web/node_modules` are generated/ignored classes and excluded from the 55-file prose analysis (WAVE-SHEET scope rule).
