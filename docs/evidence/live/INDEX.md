# Live / evidence index (2026-11-12, current)

**Current source/package:** `941ca2b`, schema **32**, `dirty:false`; server SHA-256 `263f033d6a9c4f78dee13f7cf73d120772f1f5cfac7d12472adfdc81e45c99c4`, launcher SHA-256 `993237f73f570784e9feb4f57192bf2d040ef81558ee4f58db7e3ce6a3c7f89d`; no provider key configured.

**Historical snapshot:** the 2026-11-11 rows below were taken at HEAD `1129349` / product `806251b`, schema 24, and are not current-release evidence.

**Rule:** a file here proves only what its receipt says. A PASS on a dirty tree is not a release proof; a scale harness is not a live provider run; a screenshot is not a click-through.

| File | Identity | Proves | Does not prove |
|---|---|---|---|
| `full-battery-20261110-234515.json` | HEAD `1129349`, `working_tree_dirty:true` (audit docs) | Offline battery final: 8/8 executed, 0 failed | Clean HEAD; provider behavior; release seal |
| `full-battery-20261110-232942.json` | HEAD `1129349`, dirty with discarded endpoint candidate | Prior offline battery: 8/8 | Clean HEAD; provider behavior; release seal |
| `full-battery-20261110-225940.json` | product `806251b`, dirty at `c49f90c` | Prior offline battery PASS | Current dirty/HEAD; live model |
| `install-1.2.0-dev.json` | installed `806251b`, `dirty:false` | Installed 1.2.0-dev, schema 24, shortcuts, 1440/375 screenshots, key missing | Live provider; Edge-window screenshot; clean-profile install |
| `crash-drill-20261110-230629.json` | release binary, temp data | Process kill → restart, boot_id changed, schema 24, data preserved | Mid-run crash; installed binary crash |
| `scale-drill-20261110-230653.json` | repo tests | 41-task serial and bounded-parallel probes pass | Live 40-node provider run |
| `install-ui-1440x900.png`, `install-ui-375x812.png` | release server UI | Rendering at both viewports; 8-template library visible at 1440 | Installed-launcher window; every interaction |
| `ui-splash.png`, `ui-splash-375.png`, `ui-polished-final.png` | earlier UI receipts | Splash/connected UI rendering | Current build behavior |

**Historical build identity:** product commit `806251b`; package SHA-256 `B4BAC18D29560D5C63404649C2953E359FFC6170C1214A3AA1B54DE127811061`; server SHA-256 `D9D7690287E5D9D84CF655525690983A1F109D625EFD7CFD7168CD2529732C78`.
**Truth audit:** `docs/audit/HIVEMIND-TRUTH-AND-ROADMAP-2026-11-10.md`.
