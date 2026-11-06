# UI WATCHDOG LEDGER — HiveMind v1 FigCraft assets

**Checked at:** 2026-11-06T05:23:01.101154-06:00 · **Auditor:** ui-watchdog (read-only)
**Overall:** `PASS` · **ALLOW-FRONTEND: true** · **FAIL count: 0**
**Scope:** re-audit of A residual closure (spinner colour, REFUSED codes, DECISION-GATED) + full B manifest confirmation.
**Method:** file existence, byte size, SHA-256, PNG IHDR / SVG viewBox dimensions, `figcraft-cli info --json`, design token/variable extraction, `<svg>` hex-set audit.

## Verdicts (20 artifacts)

| File | SHA-256 | Verdict | Issues / evidence |
|---|---|---|---|
| `docs/ui/figcraft/A/A-design.figcraft` | `3C5531ABA40ABD2C98B2855818CDA85475FB46985FB9146EF97C537CC1D8D6C0` | PASS | clean: 4 artboards, 20/20 token variables exact, no off-token colours, refusals+DECISION-GATED printed, manifest hashes/dims verified |
| `docs/ui/figcraft/A/A-ASSETS.json` | `88832272CB6C81DDFF234D77F5ABBDF4D9811AE1E8D787B7B02F1C07F11E77F1` | PASS | document_sha256 matches; all 28 rows: files exist, sha256 + dimensions match; node ids resolve |
| `docs/ui/figcraft/ASSETS/a-brand.png` | `47680984AA5DACB1BF6A3576B42A11B01864C646068F33E5CA4ACFE124F4A918` | PASS | bytes non-zero; sha256/dimensions match manifest |
| `docs/ui/figcraft/ASSETS/a-brand.svg` | `94CF946703C14355601813367F9A5FA11105D4E744990DD66AC7B0E5AD0A9522` | PASS | bytes non-zero; sha256/dimensions match manifest |
| `docs/ui/figcraft/ASSETS/a-components.png` | `F4FCC4C63CAEC0E40FA5911709E34C8102EB976DA0864063BA7273CBAA85C65A` | PASS | sha256/dimensions match manifest; button-state table populated; refusal codes printed |
| `docs/ui/figcraft/ASSETS/a-components.svg` | `2EB182185E22183C3473F1370784E2834C7646E7648C4D2FEBA95BDF3ADBE0E9` | PASS | sha256/dimensions match manifest; button-state table populated; refusal codes printed |
| `docs/ui/figcraft/ASSETS/a-foundations.png` | `E49B788A812E8A4962A29A926D25970B0A39218ADE8DD34EE9C8C7079BDC53BC` | PASS | bytes non-zero; sha256/dimensions match manifest |
| `docs/ui/figcraft/ASSETS/a-foundations.svg` | `24536354E6FFE9BA4DDA2C9682B526BB44BEDEA6AEA7F1D21FD4E3AEB75E551A` | PASS | bytes non-zero; sha256/dimensions match manifest |
| `docs/ui/figcraft/ASSETS/a-specimens.png` | `DA0F4CB7D435E46CDABEA31074845F9E54EEEB828C274D661104810B6807814B` | PASS | bytes non-zero; sha256/dimensions match manifest |
| `docs/ui/figcraft/ASSETS/a-specimens.svg` | `E88AECA9A0F448FE87116837E11F560D202F1D25066D441A7B591E1F70DBE350` | PASS | bytes non-zero; sha256/dimensions match manifest |
| `docs/ui/figcraft/B/B-canvas.figcraft` | `DAE6068926E30C99E89A43A1B15257FF8F16B1FD6C943AE990A4F383C0418288` | PASS | design_file sha matches; 27 surfaces; pending=[]; built=['00', '01', '02', '03']; 4 export pairs sha/dims verified; no off-token colours |
| `docs/ui/figcraft/B/B-ASSETS.json` | `A8A81E74BFBFBB3B9BCA9150DB734E8A1444D9B9E04F82CEF5C71A17F55D1F4D` | PASS | 27 surfaces; pending=[]; 4 export pairs verified |
| `docs/ui/figcraft/ASSETS/b-canvas-default.svg` | `B5A5A824145A85848247A10A663FAB2D4783CC120D04DE51FB93316A9EE33D9F` | PASS | bytes non-zero; sha256/dimensions match manifest |
| `docs/ui/figcraft/ASSETS/b-canvas-default.png` | `AA6066D38F73D391B45C623C62E285BDD19D541F3443DB984279627F2DF7A694` | PASS | bytes non-zero; sha256/dimensions match manifest |
| `docs/ui/figcraft/ASSETS/b-zoom-states.svg` | `AB1EE97AE2B9D042B9D8B78EF22F10F1D4B594041FBC51ACD27C32112A9493AE` | PASS | bytes non-zero; sha256/dimensions match manifest |
| `docs/ui/figcraft/ASSETS/b-zoom-states.png` | `A02CB4A1963A8C3588AE3CC66FA74C0F7D415E1A46E95089D191794D4F79141C` | PASS | bytes non-zero; sha256/dimensions match manifest |
| `docs/ui/figcraft/ASSETS/b-interaction.svg` | `927514772AC5321C3771ED7293AE72B67E53526CB544222B21AF408550A2E945` | PASS | bytes non-zero; sha256/dimensions match manifest |
| `docs/ui/figcraft/ASSETS/b-interaction.png` | `DE50112439898E88B8BBF334DD8427F0F8AF2FA2FBCC6BED338334F34F0D00A4` | PASS | bytes non-zero; sha256/dimensions match manifest |
| `docs/ui/figcraft/ASSETS/b-hero.svg` | `2BF8D20FACBA986B2C148702A774187538886658C680F7D323AA0E785A678477` | PASS | bytes non-zero; sha256/dimensions match manifest |
| `docs/ui/figcraft/ASSETS/b-hero.png` | `719A28BE8DE4926612ECFE5537EC7BDB98E04EAC5338D34FCC6EFBB62B54834E` | PASS | bytes non-zero; sha256/dimensions match manifest |

## Required checks (re-run)

| Check | Result | Evidence |
|---|---|---|
| A-ASSETS rows: files exist + sha256 + dimensions | PASS | 28/28 rows verified; `document_sha256` matches A-design; PNG/SVG dimensions match declared |
| `figcraft-cli info` A-design | PASS | pages 00 Foundations 1440x1200, 01 Brand 1440x900, 02 Components 1440x2580, 03 Specimens 1440x1400; nodes=1372; missingFonts=[] |
| A SVG hex audit = only 20 printed tokens | PASS | a-brand.svg: 12 hexes, off-token=none; a-components.svg: 20 hexes, off-token=none; a-foundations.svg: 20 hexes, off-token=none; a-specimens.svg: 17 hexes, off-token=none |
| A token variables = 20 PDF values | PASS | 20/20 exact; mismatches=none; extras=none |
| A spinner residual closed | PASS | `03/load/spinner` = {"fills": null, "strokes": [{"type": "solid", "color": {"r": 0.8509804, "g": 0.627451, "b": 0.18431373, "a": 1.0}, "opacity": 1.0, "visible": true, "blend": "normal"}], "stroke_weight": 2.0}; design off-token colours=none |
| A refusal codes + DECISION-GATED | PASS | codes={'E_SPAWN_DEPTH': True, 'E_CYCLE': True, 'E_RUN_TERMINAL': True, 'E_KILL_DENIED': True}; DECISION-GATED texts=2 |
| B still matches manifest | PASS | design sha `DAE6068926E30C99E89A43A1B15257FF8F16B1FD6C943AE990A4F383C0418288`; 27 surfaces; pending=[]; built=['00', '01', '02', '03']; 4 export pairs sha/dims OK; off-token=none |
| Blueprint/source hashes unchanged | PASS | 11/11 recorded sha256 match `BUILD-CHECKS.json` |

## Blocking issues

- **None.** All 20 artifacts pass manifest truth, token truth and info parsing; no invented token hexes remain.

## UNVERIFIED (recorded, not FAIL)

- Full pixel-level visual pass still partial: A boards + B default were visually spot-checked; B zoom/interaction/hero verified by manifest hashes + text/lattice inventory.
- Bee direction verified visually against `R-U6` rules on rendered boards; per-edge vector not machine-measured.

