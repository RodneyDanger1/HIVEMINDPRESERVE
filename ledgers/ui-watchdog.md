# UI WATCHDOG LEDGER — HiveMind v1 FigCraft assets

**Checked at:** 2026-11-06T04:55:00-06:00 (America/Chicago) · **Auditor:** ui-watchdog (read-only)
**Overall:** `PASS-WITH-NOTES` · **ALLOW-FRONTEND: true** · **FAIL count: 0**
**Scope:** `docs/ui/figcraft/A/*` and `docs/ui/figcraft/B/*` vs `A-ASSETS.json` (28 rows) and `B-ASSETS.json` (27 rows).
**Method:** file existence, byte size, SHA-256 (hashlib, equivalent to `Get-FileHash -Algorithm SHA256`), PNG IHDR / SVG viewBox dimensions, `figcraft-cli info --json` on both `.figcraft`, design-document token/variable extraction. Machine ledger: `ledgers/ui-watchdog.json`.

## Verdicts (20 artifacts)

| File | SHA-256 | Verdict | Issues / evidence |
|---|---|---|---|
| `A/A-design.figcraft` | `93226DDFC3297274AAD2B649852B53A60321D8A2F4FF280DE05349858276CD74` | PASS-WITH-NOTES | `info --json` parses: 4 pages 1440×1200 / 1440×900 / 1440×2580 / 1440×1400, 1368 nodes, `missingFonts: []`, 20 colour variables. 20/20 PDF token hexes exact; no renamed/extra tokens. One off-token colour `#D9D9D9` on `03/load/spinner` (03 Specimens loading state) — see must-fix. |
| `A/A-ASSETS.json` | `011D5AC08C1B6E2875F2E5CF643DB6624E211B9B0AA9556A0CFAB5868057C1B3` | PASS | 28 rows, 28 unique surface ids S01–S28. `document_sha256` matches A-design. All 56 listed export paths exist, non-zero, SHA-256 + dimensions match declared values. |
| `A/ASSETS/a-foundations.svg` | `24536354E6FFE9BA4DDA2C9682B526BB44BEDEA6AEA7F1D21FD4E3AEB75E551A` | PASS | 1440×1200 board (PNG @2x 2880×2400). 20/20 printed token hexes present; zero non-token hexes. |
| `A/ASSETS/a-foundations.png` | `E49B788A812E8A4962A29A926D25970B0A39218ADE8DD34EE9C8C7079BDC53BC` | PASS | 2880×2400, non-zero. |
| `A/ASSETS/a-brand.svg` | `94CF946703C14355601813367F9A5FA11105D4E744990DD66AC7B0E5AD0A9522` | PASS | 1440×900 board (PNG @2x 2880×1800). Token-clean. |
| `A/ASSETS/a-brand.png` | `47680984AA5DACB1BF6A3576B42A11B01864C646068F33E5CA4ACFE124F4A918` | PASS | 2880×1800, non-zero. |
| `A/ASSETS/a-components.svg` | `553E64308A50162F95F22F29C16DDB5555AD543CAC426E248A95D1ADAA53D0CE` | PASS | 1440×2580 board (PNG @2x 2880×5160). 20/20 token hexes, zero off-token hexes. **Buttons state table verified populated** (4 buttons × 5 states DEFAULT/HOVER/PRESSED/DISABLED/REFUSED + frozen ids SPAWN_SUB_HIVE/REWIRE_TASK/CANCEL_RUN/KILL_WORKER) — the earlier blank row is fixed; `A/A-EPISODES.json` records the cause (`object.group` front-of-stack ordering, fixes confirmed against `build/c4.png` vs `probe-s04c.png`). |
| `A/ASSETS/a-components.png` | `BC71B15999593B6081FDBD2087432D9C80DFAA2C1430CA5A8DD4173BDA786C58` | PASS | 2880×5160; button band visually re-checked, all 20 cells rendered. |
| `A/ASSETS/a-specimens.svg` | `D8128E43D7726A39D3BE07CF5D579A050FE50945296098E717D2BF346AF61563` | PASS-WITH-NOTES | 1440×1400 board (PNG @2x 2880×2800); exports the same off-token `#D9D9D9` spinner as the design file. |
| `A/ASSETS/a-specimens.png` | `99321E1FDEB2446CA2457BABF865D6353FCC693C3AB3E29361A5D8E4D1027FE7` | PASS-WITH-NOTES | 2880×2800; carries off-token spinner colour (see must-fix). |
| `B/B-canvas.figcraft` | `DAE6068926E30C99E89A43A1B15257FF8F16B1FD6C943AE990A4F383C0418288` | PASS | `info --json` parses: 4/4 boards built and non-empty — Canvas/default (223 nodes), Canvas/zoom states (94), Canvas/interaction states (174), Brand hero (107); 602 nodes; `missingFonts: []`; 0 token violations. Zooms `0.1x / 1.0x / 3.0x` explicit; S25 banner reads `DECISION-GATED · NOT FROZEN · NOT IMPLEMENTED`. |
| `B/B-ASSETS.json` | `C8D9C3DBD156FF81663D9D6EEBCE670CCA2D81BF5BAA776357380BDFDB673F92` | PASS | 27 surface ids (≥24), `pending_surfaces: []`, `built_pages: ["00","01","02","03"]`. `design_file.sha256` matches B-canvas. 4 export pairs verified. |
| `B/ASSETS/b-canvas-default.svg` | `3176D156B2E94005E5985E1ACC94F6FA3FC038A17C3EF3762368AB50CE0A6562` | PASS | 1440×900 (PNG @2x 2880×1800); token-clean; sha matches manifest. |
| `B/ASSETS/b-canvas-default.png` | `229407C272BA8EE4A14F0F10595470CDC0BEAFC4A5E8710702E8AD48052914F8` | PASS | 2880×1800, non-zero; visually checked: nav rail 72, library 260, minimap 260×170, HUD, scrubber, pinned chat core, 4 node types + error, bee-line edges. |
| `B/ASSETS/b-zoom-states.svg` | `AB1EE97AE2B9D042B9D8B78EF22F10F1D4B594041FBC51ACD27C32112A9493AE` | PASS | 1440×900; 0.1x/1.0x/3.0x frames present; token-clean; sha matches. |
| `B/ASSETS/b-zoom-states.png` | `A02CB4A1963A8C3588AE3CC66FA74C0F7D415E1A46E95089D191794D4F79141C` | PASS | 2880×1800, non-zero. |
| `B/ASSETS/b-interaction.svg` | `927514772AC5321C3771ED7293AE72B67E53526CB544222B21AF408550A2E945` | PASS | 1440×900; selected/drag/reroute/reconnect/awaiting-approval/refusal/kill/replay + `DECISION-GATED`; token-clean; sha matches. |
| `B/ASSETS/b-interaction.png` | `DE50112439898E88B8BBF334DD8427F0F8AF2FA2FBCC6BED338334F34F0D00A4` | PASS | 2880×1800, non-zero. |
| `B/ASSETS/b-hero.svg` | `2BF8D20FACBA986B2C148702A774187538886658C680F7D323AA0E785A678477` | PASS | 1440×900; token-clean; sha matches. |
| `B/ASSETS/b-hero.png` | `719A28BE8DE4926612ECFE5537EC7BDB98E04EAC5338D34FCC6EFBB62B54834E` | PASS | 2880×1800, non-zero. |

## Required checks

| Check | Result | Evidence |
|---|---|---|
| A has **28 unique** surface ids | PASS | `S01…S28`, duplicates = 0 (`A-ASSETS.json`, machine-verified). |
| B has **≥24** surface ids + pending list | PASS | 27 ids, `pending_surfaces: []`, `built_pages: ["00","01","02","03"]`. |
| PDF 20 token hexes match A | PASS | 20/20 exact: `#F8F4EB #FFFFFF #FBF8F1 #F1E9CF #E5E1D8 #FBF3DC #F6E7BE #F0C050 #E8B850 #D9A02F #C88030 #8A5A1C #604818 #282010 #3F3115 #4E3A15 #75663F #B7791F #B23A2E #3F7A55` (PDF p2:20-79); zero missing, zero renamed, zero extra tokens. |
| `.figcraft` info parses (A & B) | PASS | `figcraft-cli info --json` exit 0 both; pages/artboards as listed above; `missingFonts: []` both. |
| Manifest truth (no missing file, no orphan row) | PASS | 56 A file refs + 8 B file refs all exist; 20/20 ASSETS files are manifest-listed (16 deliverable + 0 orphan). |
| Blueprint/source hashes unchanged | PASS | All 11 recorded sha256 in `hivemind-blueprint/blueprint/BUILD-CHECKS.json` match (incl. source of record `7B85760E…D713`); frozen UI docs byte-identical (`UI-COMMANDS-v1.md`, `WATCHDOG-BRIEF`, `BRIEF-A/B`, `UI-REQUIREMENTS.md`, `pages.txt`). |

## Must-fix / blocking issues

- **None blocking frontend consumption.** Allow flag is `true`.
- **A (must-fix before A close, non-blocking):** one off-token colour `#D9D9D9` on node `03/load/spinner` (03 Specimens loading state) appears in `A-design.figcraft` and `a-specimens.svg/png`. Fix: use token `ink/500 #75663F` (or `border/hairline #E5E1D8`) for the spinner, regenerate `a-specimens.svg/png` + `A-ASSETS.json` hashes. Frontend must not adopt the grey as a token.
- **A (minor labels):** Components page S25 row reads `DIFF_* gated` / `Diff review (no board)` rather than the literal `DECISION-GATED` (the literal exists on 03 Specimens and on B's interaction board); `E_CYCLE` (REWIRE_TASK refusal) is not printed on the components refusals row (toast prints `E_REWIRE_IN_FLIGHT`). Add literal label if strict.

## UNVERIFIED (recorded, not FAIL)

- Full pixel-level visual pass was partial: all 4 A boards and `b-canvas-default` checked visually at key bands; B zoom/interaction/hero verified by text + lattice inventory and manifest hashes, not a full raster review.
- Bee direction: B boards state the rule (`R-U6`, `source→target`) and bees render along edges; exact per-edge vector was not machine-measured — visual only.
- Episodes exist (A: 12/12 files, 19 valid command ids, 3 documented failures; B: 8 episode JSONs) but no `B-EPISODES.json` summary file (BRIEF-B only requires episodes under `EPISODES/`).
- B interaction board has no reduced-motion tile (A 03 Specimens covers reduced motion; not a BRIEF-B interaction requirement).
