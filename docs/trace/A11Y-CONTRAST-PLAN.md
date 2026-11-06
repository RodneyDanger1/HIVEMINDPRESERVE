# A11Y / CONTRAST PLAN — G-UI-1..12 (harness plan, NOT a pass record)

Sources: `reports/ui/UI-REQUIREMENTS.md` §1/§4/§5, `blueprint/UI-WIRING-GUIDE.md` §0/§3/§4,
`blueprint/HIVEMIND-BLUEPRINT-v1.md` Phase 10, `docs/contracts/UI-COMMANDS-v1.md` §5.
Status: **PLAN + static math only.** DOM gates are blocked until the wired UI exists and
`@playwright/test` is installed (not installed; install needs Lead approval — no browser download).

## 1. Evidence of static math (recomputed today, node, WCAG 2.1 relative luminance)

| Pair | Hex (PDF pages.txt:20-79) | Expected (UI-R-02) | Recomputed 2026-11-06 | Match |
|---|---|---|---|---|
| ink-900/canvas | `#282010` / `#F8F4EB` | 14.67 | **14.6677** | yes |
| ink-900/honey-400 | `#282010` / `#E8B850` | 8.74 | **8.7427** | yes |
| white/ink-800 | `#FFFFFF` / `#3F3115` | 12.64 | **12.6391** | yes |
| amber/canvas | `#B7791F` / `#F8F4EB` | 3.32 | **3.3167** | yes (AA-large only) |
| error/canvas | `#B23A2E` / `#F8F4EB` | 5.41 | **5.4079** | yes |
| verify/canvas | `#3F7A55` / `#F8F4EB` | 4.64 | **4.6364** | yes |
| card-meta/white | `#8A7A52` / `#FFFFFF` | fails 4.5 | **4.2115** | recorded failure |
| card-meta/canvas | `#8A7A52` / `#F8F4EB` | fails 4.5 | **3.8367** | recorded failure |
| focus-ring/white | `#F0C050` / `#FFFFFF` | fails 3:1 | **1.6989** | recorded failure |
| focus-ring/canvas | `#F0C050` / `#F8F4EB` | fails 3:1 | **1.5477** | recorded failure |

Implementation: `e2e/helpers/contrast.ts` (`contrastRatio`, `EXPECTED_CONTRAST`, `KNOWN_FAILURES`).
`e2e/specs/a11y-gates.spec.ts` asserts the six rows within ±0.01 and the four recorded failures.
The implemented CSS token mirror (`web/src/styles/tokens.css`) contains the matching hexes
(`--color-ink-900:#282010`, `--color-canvas:#f8f4eb`, `--color-honey-400:#e8b850`,
`--color-ink-800:#3f3115`, `--color-honey-300:#f0c050`, `--color-ink-700:#4e3a15`,
`--color-amber:#b7791f`, `--color-signal:#b23a2e`, `--color-verify:#3f7a55`) — token/DOM
verification still needs the app to render.

## 2. G-UI-1..12 harness plan

| Gate | What the harness must prove | Implementation | Status |
|---|---|---|---|
| G-UI-1 Contrast | six rows reproduce; meta/focus replaced; no small amber text | static math (above) + live computed-style scan of every text pair | math **VERIFIED**; DOM blocked |
| G-UI-2 Keyboard recentre | Ctrl+0 recentres ≤1 rAF; composer focus not hijacked; Meta+0 alias only macOS; page zoom unchanged | Playwright `keyboard.press('Control+0')` + viewport transform read; `E2E_CHANNEL` | blocked (app) |
| G-UI-3 WS origin refusal | foreign Origin → close 1008 + `E_ORIGIN_REJECTED`, zero state change; allowed origin+token accepted in first frame | synthetic non-allowlisted page (route intercept) + `tapWebSockets` | blocked (app) |
| G-UI-4 State vocabulary | one ≥13-state enum; enum↔token↔label total/injective; 0 state nouns outside enum | static source grep + runtime enum/table test | blocked (app); source grep possible later |
| G-UI-5 Missing-state coverage | each state reachable with copy + dismissal (loading, empty×3, app error, budget refusal, streaming, replay, offline, missing-key, thinking, cancelled/killed, reconnecting) | flag each state, screenshot each; no spinner without timeout | blocked (app) |
| G-UI-6 Reduced motion | 0 running animations under `prefers-reduced-motion`; state still updates; toggle persists | Playwright `emulateMedia({reducedMotion:'reduce'})` | blocked (app) |
| G-UI-7 Zoom clamp | clamps exactly 0.1/3.0, both reachable; no React Flow 0.5/2 default | wheel/drag sweep + store transform read | blocked (app) |
| G-UI-8 Focus & boundaries | every interactive element focus ring ≥3:1; no hairline-only boundary (1.19); axe serious = 0 | keyboard tab sweep + computed style; axe-core run TBD | blocked (app) |
| G-UI-9 Refusal by name | three fixtures (429, budget, rewire-while-streaming) render code+how_to_fix+`aria-live`, pre-mutation, one audit row | mock provider fixtures + `tapWebSockets` + toast DOM | blocked (app) |
| G-UI-10 Reconnect/resnapshot | kill socket mid-stream → `hello{last_seq}` → snapshot-or-diff; no duplicated text; `Reconnecting`; stale mutation `E_RESYNC_REQUIRED` | Playwright `context.setOffline`/route abort + WS tap | blocked (app) |
| G-UI-11 Token provenance | 0 hex outside token module; every token row has PDF page:line | `grep -rn '#[0-9A-Fa-f]{6}' web/src --exclude tokens*` (allow tokens.css/tokens.ts only) | runnable as static grep once web source stabilizes |
| G-UI-12 Performance | P95 frame time measured at N nodes/zoom extremes on this host, else UNVERIFIED | in-page rAF sampler + report hardware/command | blocked (app); no fps claim allowed |

## 3. Tooling decision (no install now)

- Blueprint pins `jest-axe 11.0.0` (axe-core 4.14.0, test-only) for unit tests; `@playwright/test 1.65.0` for e2e.
- E2E axe integration would need `@axe-core/playwright` (not in the blueprint pin list). **Decision needed from
  Lead**: either inject axe-core 4.14.0 from `web/node_modules` (jest-axe dependency) into Playwright pages, or add
  `@axe-core/playwright` as a devDependency with a new provenance row. No install performed by QA.
- No live paid API anywhere in these gates: provider fixtures are wiremock/mock only (blueprint Phase 5/11).

## 4. Blockers

1. Wired UI not delivered yet (`web/src/App.tsx` is milestone-1 chrome).
2. `@playwright/test` not installed (pinned 1.65.0; npx cache 1.62.1); install requires Lead approval — package
   install only, `PLAYWRIGHT_SKIP_BROWSER_DOWNLOAD=1`, never a browser download.
3. No axe dependency in Playwright context — decision needed (see §3).
4. No mock-provider e2e fixture for G-UI-9 yet (backend wiremock is dev-only Rust; a browser-visible mock mode or
   a fixture server must be agreed with the backend owner).
