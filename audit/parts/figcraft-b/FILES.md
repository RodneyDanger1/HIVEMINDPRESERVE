# FILES — FigCraft-B seat (91 files: docs/plans, docs/contracts, docs/claims, docs/reports, docs/audit)

Audit wave: HEAD `3988622` (short), seat `figcraft-b`, read-only. Every record below is generated from the read-only extractor (`tools/extract.py`) over the actual file + `audit/inventory/FILE-MANIFEST.csv`; no behaviour is inferred from a filename.

Manifest hash verification: **91/91 files byte-identical to the manifest sha256** (0 missing, 0 mismatched, 0 size mismatches).

## Partition census

| Group | Files | Bytes | MD/CSV/JSON | Logs/out/err | Binary |
|---|---:|---:|---:|---:|---:|
| `docs/audit` | 59 | 5,364,955 | 12 | 38 | 4 |
| `docs/claims` | 3 | 28,541 | 3 | 0 | 0 |
| `docs/contracts` | 2 | 17,515 | 2 | 0 | 0 |
| `docs/plans` | 26 | 152,866 | 26 | 0 | 0 |
| `docs/reports` | 1 | 6,121 | 1 | 0 | 0 |

Standard depth for all 91 files (wave sheet: 0 deep files for this seat).

---

## docs/audit/BASELINE-VERIFICATION.md

- **Metadata:** 15,071 bytes · sha256 `DC11F4811F4E12563B74B63F5CCD58B0A3791BC39F1CD3100D8A55C66FE1692A` · Markdown · documentation · seat figcraft-b · depth standard
- **Manifest:** bytes MATCH · sha256 MATCH · category `documentation`/`report/plan/evidence`
- **Purpose (from content):** # BASELINE VERIFICATION — independent verifier seat (Phase A) - **Seat:** independent verifier (read-only on product code; writes only `docs/audit/**`, `docs/claims/VERIFIER-*.md`, `docs/trace/**`, `docs/diagrams/**`). - **Date:** 2026-11-06, America/Chicago (session window 04:05–04:30).
- **Mechanics / structure:** 123 lines; headings: 1. Source-of-truth hash gate — PASS (9/9); 2. Toolchain re-verified — PASS; 3. Claim-register validation (`docs/claims/CLAIM-REGISTER.csv/.json`) — PASS with two stale-status annotations; 4. `reports/verify/PRUNED-PLAN-CLAIMS.*` validation — PASS; 5. Baseline workspace state (snapshot, not a build verdict); 6. Contradictions / discrepancies found (recorded, not hidden); 7. Verdict; 8. Report shape · markers: PASS=21, UNVERIFIED=9
- **Dependencies (cited in text):** `audit/evidence/controls-verifier-skeleton.json`, `audit/evidence/toolchain/hello.c`, `audit/evidence/toolchain/hello.exe`, `audit/evidence/toolchain/hello.rs`, `audit/evidence/toolchain/hello_rs.exe`, `claims/CLAIM-REGISTER.csv`, `contracts/UI-COMMANDS-v1.md`, `plans/HASH-VERIFICATION.md`, `plans/TOOLCHAIN.md`
- **Quality:** correctness 4/5, maintainability 4/5, modularity 3/5, error_handling 3/5, security 5/5, docs 4/5 — 9 UNVERIFIED mentions.
- **Evidence:** manifest sha256 MATCH; 123 lines read; dates: 2026-08-27, 2026-10-13, 2026-11-06; status words: PASS, PASS, PASS, PASS, PASS, PASS, PASS, PASS.

## docs/audit/FINAL-CLOSEOUT.md

- **Metadata:** 3,218 bytes · sha256 `81D45A5C4615AF15A7C1895524A70013C7832FB02741B4E74C83FED9A8A85AEB` · Markdown · documentation · seat figcraft-b · depth standard
- **Manifest:** bytes MATCH · sha256 MATCH · category `documentation`/`report/plan/evidence`
- **Purpose (from content):** # FINAL CLOSEOUT — HiveMind v1 frozen landing Date: 2026-11-06 (America/Chicago). 8-seat closeout loop; all work landed read-only; tree frozen and clean. ## Frozen artifact
- **Mechanics / structure:** 42 lines; headings: Frozen artifact; Seat landing checks (all PASS); Process cleanup (this closeout); Honest residuals (not passed) · markers: PASS=3, FAIL=1, UNVERIFIED=2, PASS-WITH-NOTES=1
- **Dependencies (cited in text):** `audit/FINAL-DELTA-VERIFICATION.md`, `audit/evidence/final/pw-out/.last-run.json`
- **Quality:** correctness 3/5, maintainability 4/5, modularity 3/5, error_handling 3/5, security 5/5, docs 4/5 — self-declared FAIL/BLOCKED present; 2 UNVERIFIED mentions.
- **Evidence:** manifest sha256 MATCH; 42 lines read; dates: 2026-11-06; status words: PASS, PASS, FAIL, PASS-WITH-NOTES, UNVERIFIED, UNVERIFIED.

## docs/audit/FINAL-DELTA-VERIFICATION.md

- **Metadata:** 13,168 bytes · sha256 `A4ACBCBB1D17DA924B4B8B99C219B4F4DA3AA41700338C5380EE93E298F54EC3` · Markdown · documentation · seat figcraft-b · depth standard
- **Manifest:** bytes MATCH · sha256 MATCH · category `documentation`/`report/plan/evidence`
- **Purpose (from content):** # FINAL DELTA VERIFICATION — independent verifier (post-D-2/D-3/F-fix, frozen tree) - **Seat:** independent verifier (read-only on product code; writes only `docs/audit/**`). - **Date:** 2026-11-06, 09:13–09:31 America/Chicago.
- **Mechanics / structure:** 81 lines; headings: Verdict: **PASS-WITH-NOTES (product/live/release gates) + FAIL (final claim-register deliverable)**; Delta gates (independently re-run); Defect delta (all confirmed fixed); Open items (do NOT claim these as verified; nothing blocks the product gates); Report shape · markers: PASS=10, FAIL=5, UNVERIFIED=10, PASS-WITH-NOTES=2
- **Dependencies (cited in text):** `audit/evidence/final/cli-diagnose-final.out`, `audit/evidence/final/cli-health-final.out`, `audit/evidence/final/live-probe-final.json`, `audit/evidence/final/pw-out/.last-run.json`, `claims/CLAIM-REGISTER.csv`, `claims/CLAIM-REGISTER.json`
- **Quality:** correctness 3/5, maintainability 4/5, modularity 3/5, error_handling 3/5, security 5/5, docs 4/5 — self-declared FAIL/BLOCKED present; 10 UNVERIFIED mentions.
- **Evidence:** manifest sha256 MATCH; 81 lines read; dates: 2026-11-06; status words: PASS-WITH-NOTES, FAIL, PASS-WITH-NOTES, FAIL, PASS, PASS, PASS, PASS.

## docs/audit/PHASE-B-VERIFICATION.md

- **Metadata:** 19,881 bytes · sha256 `CA6E76D1C942EF8579C5572E9711122964CFA9C91D56B4B70E836C5893789E6C` · Markdown · documentation · seat figcraft-b · depth standard
- **Manifest:** bytes MATCH · sha256 MATCH · category `documentation`/`report/plan/evidence`
- **Purpose (from content):** # PHASE B VERIFICATION — independent verifier - **Seat:** independent verifier (read-only on product code; writes only `docs/audit/**`, `docs/claims/VERIFIER-*.md`, `docs/diagrams/**`). - **Date:** 2026-11-06, 07:28–08:20 America/Chicago.
- **Mechanics / structure:** 106 lines; headings: Gate verdicts; Defects (remaining, with fix seams); Open claim / gate that must not be claimed; V2 re-verification — artifact changed during this audit (read with the tables above); Verdict (V2 snapshot); Report shape · markers: PASS=6, FAIL=1, UNVERIFIED=11, BLOCKED=1
- **Dependencies (cited in text):** `audit/evidence/final/pw-out/.last-run.json`, `audit/evidence/phase-b/cli/preflight.err`, `audit/evidence/phase-b/live-probe-verifier-v2.json`, `audit/evidence/phase-b/live-probe-verifier-v2.out.txt`, `audit/evidence/phase-b/live-probe-verifier.json`, `claims/CLAIM-REGISTER.csv`, `claims/CLAIM-REGISTER.json`
- **Quality:** correctness 3/5, maintainability 4/5, modularity 3/5, error_handling 3/5, security 5/5, docs 4/5 — self-declared FAIL/BLOCKED present; 11 UNVERIFIED mentions.
- **Evidence:** manifest sha256 MATCH; 106 lines read; dates: 2026-11-06; status words: FAIL, UNVERIFIED, PASS, UNVERIFIED, UNVERIFIED, UNVERIFIED, PASS, PASS.

## docs/audit/QA-ENV.md

- **Metadata:** 6,608 bytes · sha256 `E3ED721740DAF0E204C78CD8B8138D4611C41F57A1A68B500A644670FE317FDE` · Markdown · documentation · seat figcraft-b · depth standard
- **Manifest:** bytes MATCH · sha256 MATCH · category `documentation`/`report/plan/evidence`
- **Purpose (from content):** ﻿# QA-ENV — QA/trace environment & artifact evidence audit | Field | Value | |---|---|
- **Mechanics / structure:** 54 lines; headings: Discrepancies / notes (owned elsewhere); Not verified / open · markers: PASS=1, FAIL=1, UNVERIFIED=1
- **Dependencies (cited in text):** `claims/SKILL-USE.md`, `contracts/UI-COMMANDS-v1.md`
- **Quality:** correctness 3/5, maintainability 4/5, modularity 3/5, error_handling 3/5, security 5/5, docs 4/5 — self-declared FAIL/BLOCKED present; 1 UNVERIFIED mentions.
- **Evidence:** manifest sha256 MATCH; 54 lines read; dates: 2026-11-06; status words: PASS, FAIL, UNVERIFIED.

## docs/audit/evidence/controls-verifier-skeleton.json

- **Metadata:** 39,735 bytes · sha256 `2AE3C3F74A8CD378EC42CF73D776742E90F39465AE68E9285290834411EFD2CF` · JSON · documentation · seat figcraft-b · depth standard
- **Manifest:** bytes MATCH · sha256 MATCH · category `documentation`/`report/plan/evidence`
- **Purpose (from content):** { "register": "docs/trace/controls.json", "kind": "skeleton inventory (Phase A baseline)",
- **Mechanics / structure:** 970 lines; headings: (none) · markers: UNVERIFIED=341
- **Dependencies (cited in text):** `contracts/UI-COMMANDS-v1.md`
- **Quality:** correctness 3/5, maintainability 4/5, modularity 3/5, error_handling 3/5, security 5/5, docs 3/5 — 341 UNVERIFIED mentions.
- **Evidence:** manifest sha256 MATCH; 970 lines read; dates: 2026-11-06; status words: UNVERIFIED, UNVERIFIED, UNVERIFIED, UNVERIFIED, UNVERIFIED, UNVERIFIED, UNVERIFIED, UNVERIFIED.

## docs/audit/evidence/final/cli-diagnose-final.err

- **Metadata:** 0 bytes · sha256 `E3B0C44298FC1C149AFBF4C8996FB92427AE41E4649B934CA495991B7852B855` · Log · documentation · seat figcraft-b · depth standard
- **Manifest:** bytes MATCH · sha256 MATCH · category `documentation`/`report/plan/evidence`
- **Purpose (from content):** (no markdown heading)
- **Mechanics / structure:** 0 lines; headings: (none)
- **Dependencies (cited in text):** none detected in assigned set
- **Quality:** correctness 5/5, maintainability 3/5, modularity 2/5, error_handling 4/5, security 5/5, docs 2/5 — empty stderr (expected for a clean command).
- **Evidence:** manifest sha256 MATCH; 0 lines read; dates: none; status words: none.

## docs/audit/evidence/final/cli-diagnose-final.out

- **Metadata:** 6,246 bytes · sha256 `6596AD0FDE6AE8C044B417902240A5267A9DF895E46CBC7035B8C0618590B78A` · Log · documentation · seat figcraft-b · depth standard
- **Manifest:** bytes MATCH · sha256 MATCH · category `documentation`/`report/plan/evidence`
- **Purpose (from content):** ��{ " s c h e m a " : " h i v e r o u t e r . c l i . v 1 " , " o k " : t r u e , " c o m m a n d " : " d i a g n o s e " , " d a t a " : { " u r l " : " h t t p : / / 1 2 7 . 0 . 0 . 1 : 8 7 9 9 / d i a g n o s e " , " r e s p o n s e " : { " s c h e m a " : " h i v e r o u t e r . d i a g n o s e . v 1 " , " v e r s i o n " : " 0 . 1 . 0 " , " u p t i m e _ s e c s " : 0 , " d b _ o k " : t r u e , " p r o v i d e 
- **Mechanics / structure:** 3 lines; headings: (none)
- **Dependencies (cited in text):** none detected in assigned set
- **Quality:** correctness 5/5, maintainability 3/5, modularity 2/5, error_handling 4/5, security 5/5, docs 2/5 — no failure markers.
- **Evidence:** manifest sha256 MATCH; 3 lines read; dates: none; status words: none.

## docs/audit/evidence/final/cli-health-final.out

- **Metadata:** 1,452 bytes · sha256 `FC5FE6F367A1D692FF4993CBF7C97DDEB2E143A0F648992AFE1BB69F200DCBD3` · Log · documentation · seat figcraft-b · depth standard
- **Manifest:** bytes MATCH · sha256 MATCH · category `documentation`/`report/plan/evidence`
- **Purpose (from content):** ��{ " s c h e m a " : " h i v e r o u t e r . c l i . v 1 " , " o k " : t r u e , " c o m m a n d " : " h e a l t h " , " d a t a " : { " u r l " : " h t t p : / / 1 2 7 . 0 . 0 . 1 : 8 7 9 9 / h e a l t h " , " r e s p o n s e " : { " s t a t u s " : " o k " , " v e r s i o n " : " 0 . 1 . 0 " , " s c h e m a " : " h i v e r o u t e r . v 1 " , " u p t i m e _ s e c s " : 6 4 6 , " c h e c k s " : [ { " n a m e " : 
- **Mechanics / structure:** 3 lines; headings: (none)
- **Dependencies (cited in text):** none detected in assigned set
- **Quality:** correctness 5/5, maintainability 3/5, modularity 2/5, error_handling 4/5, security 5/5, docs 2/5 — no failure markers.
- **Evidence:** manifest sha256 MATCH; 3 lines read; dates: none; status words: none.

## docs/audit/evidence/final/live-probe-final.json

- **Metadata:** 5,877 bytes · sha256 `5AE601760ACFA8FC9B145864B9B85450AAF532BCE6D0D003C91813D23B50E190` · JSON · documentation · seat figcraft-b · depth standard
- **Manifest:** bytes MATCH · sha256 MATCH · category `documentation`/`report/plan/evidence`
- **Purpose (from content):** { "generated_at": "2026-11-06T15:14:33.957Z", "base": "http://127.0.0.1:8799",
- **Mechanics / structure:** 223 lines; headings: (none) · markers: PASS=2, FAIL=2, UNVERIFIED=2, PASS-WITH-NOTES=2
- **Dependencies (cited in text):** none detected in assigned set
- **Quality:** correctness 3/5, maintainability 4/5, modularity 3/5, error_handling 3/5, security 5/5, docs 3/5 — self-declared FAIL/BLOCKED present; 2 UNVERIFIED mentions.
- **Evidence:** manifest sha256 MATCH; 223 lines read; dates: 2026-11-06; status words: PASS-WITH-NOTES, FAIL, UNVERIFIED, PASS-WITH-NOTES, FAIL, UNVERIFIED.

## docs/audit/evidence/final/live-probe-final.mjs

- **Metadata:** 6,788 bytes · sha256 `854F35D3F669663505EEC82AE21E9349AFD60DAF2ED948D0373C2DE6B368996A` · JavaScript · documentation · seat figcraft-b · depth standard
- **Manifest:** bytes MATCH · sha256 MATCH · category `documentation`/`report/plan/evidence`
- **Purpose (from content):** // FINAL DELTA verifier live probe — browser + WS command round-trips on the frozen binary. import { writeFileSync } from 'node:fs'; import { createRequire } from 'node:module';
- **Mechanics / structure:** 107 lines; headings: (none)
- **Dependencies (cited in text):** `audit/evidence/final/live-probe-final.json`
- **Quality:** correctness 4/5, maintainability 3/5, modularity 3/5, error_handling 3/5, security 5/5, docs 3/5 — no failure markers.
- **Evidence:** manifest sha256 MATCH; 107 lines read; dates: none; status words: none.

## docs/audit/evidence/final/live-probe-final.out.txt

- **Metadata:** 7,786 bytes · sha256 `9FF94FE9FEB7494E7F3D8E09BF172AB6952D3CACE030E9245CE7A74D73C17A54` · Text · documentation · seat figcraft-b · depth standard
- **Manifest:** bytes MATCH · sha256 MATCH · category `documentation`/`report/plan/evidence`
- **Purpose (from content):** ��{         " o k " :   t r u e , 
- **Mechanics / structure:** 419 lines; headings: (none)
- **Dependencies (cited in text):** none detected in assigned set
- **Quality:** correctness 4/5, maintainability 3/5, modularity 3/5, error_handling 3/5, security 5/5, docs 3/5 — no failure markers.
- **Evidence:** manifest sha256 MATCH; 419 lines read; dates: none; status words: none.

## docs/audit/evidence/final/pw-out/.last-run.json

- **Metadata:** 96 bytes · sha256 `9C10E541B3FF84B211BF2FF8DC0D531241D515BFFF256630609FFB09D745C5AA` · JSON · documentation · seat figcraft-b · depth standard
- **Manifest:** bytes MATCH · sha256 MATCH · category `documentation`/`report/plan/evidence`
- **Purpose (from content):** { "status": "failed", "failedTests": [
- **Mechanics / structure:** 6 lines; headings: (none)
- **Dependencies (cited in text):** none detected in assigned set
- **Quality:** correctness 4/5, maintainability 4/5, modularity 3/5, error_handling 3/5, security 5/5, docs 3/5 — no failure markers.
- **Evidence:** manifest sha256 MATCH; 6 lines read; dates: none; status words: none.

## docs/audit/evidence/final/pw-out/ws-contract-WS-transport-c-9a060-refusal-has-code-how-to-fix-chrome/error-context.md

- **Metadata:** 3,113 bytes · sha256 `BDF922489455595EEA0B3DD455122EAF7CA357FC2F8DC4ADFA738240597995BC` · Markdown · documentation · seat figcraft-b · depth standard
- **Manifest:** bytes MATCH · sha256 MATCH · category `documentation`/`report/plan/evidence`
- **Purpose (from content):** # Instructions - Following Playwright test failed. - Explain why, be concise, respect Playwright best practices.
- **Mechanics / structure:** 67 lines; headings: Test info; Error details; Test source
- **Dependencies (cited in text):** `contracts/UI-COMMANDS-v1.md`
- **Quality:** correctness 4/5, maintainability 4/5, modularity 3/5, error_handling 3/5, security 5/5, docs 4/5 — no failure markers.
- **Evidence:** manifest sha256 MATCH; 67 lines read; dates: none; status words: none.

## docs/audit/evidence/final/pw-out/ws-contract-WS-transport-c-9a060-refusal-has-code-how-to-fix-chrome/test-failed-1.png

- **Metadata:** 4,254 bytes · sha256 `3878570F4A62A0D65F57CF1754E0269E56EC797F4E36E26B44C172EB6E800401` · PNG · documentation-asset · seat figcraft-b · depth standard
- **Manifest:** bytes MATCH · sha256 MATCH · category `documentation-asset`/`design/evidence asset`
- **Purpose / provenance:** binary evidence artifact (`.png`), committed as raw build output evidence; content not prose-readable.
- **Mechanics:** not applicable (binary). SHA-256 verified against manifest; size recorded in metadata.
- **Dependencies:** none resolvable from content; see DEPENDENCIES.md for the referencing documents.
- **Quality:** correctness 3/5 (raw artifact, provenance unstated in-file); maintainability 2/5; evidence 3/5 (hash only).
- **Evidence / UNVERIFIED:** hash match proven; semantic content UNVERIFIED (binary).

## docs/audit/evidence/final/pw-out/ws-contract-WS-transport-c-9a060-refusal-has-code-how-to-fix-chrome/trace.zip

- **Metadata:** 11,204 bytes · sha256 `7835BF048A538279FD112967CAFE49CD827C9F44522755422E36CE22126E889C` · Zip · documentation-asset · seat figcraft-b · depth standard
- **Manifest:** bytes MATCH · sha256 MATCH · category `documentation-asset`/`design/evidence asset`
- **Purpose / provenance:** binary evidence artifact (`.zip`), committed as raw build output evidence; content not prose-readable.
- **Mechanics:** not applicable (binary). SHA-256 verified against manifest; size recorded in metadata.
- **Dependencies:** none resolvable from content; see DEPENDENCIES.md for the referencing documents.
- **Quality:** correctness 3/5 (raw artifact, provenance unstated in-file); maintainability 2/5; evidence 3/5 (hash only).
- **Evidence / UNVERIFIED:** hash match proven; semantic content UNVERIFIED (binary).

## docs/audit/evidence/gen_controls.py

- **Metadata:** 15,934 bytes · sha256 `8EBC94962F25917A6BBD72E902A3BD0B3DDC3E3B5C416AC87E5447F7BF830E88` · Python · documentation · seat figcraft-b · depth standard
- **Manifest:** bytes MATCH · sha256 MATCH · category `documentation`/`report/plan/evidence`
- **Purpose (from content):** ﻿import hashlib, json, datetime, os ROOT = r"C:\Users\Riley\Desktop\hiverouter-v1" WIRING = r"C:\Users\Riley\Desktop\hivemind-blueprint\blueprint\UI-WIRING-GUIDE.md"
- **Mechanics / structure:** 244 lines; headings: expected_events, acceptance, owner_phase, provenance) · markers: UNVERIFIED=17
- **Dependencies (cited in text):** `audit/evidence/controls-verifier-skeleton.json`, `contracts/UI-COMMANDS-v1.md`
- **Quality:** correctness 3/5, maintainability 3/5, modularity 3/5, error_handling 3/5, security 5/5, docs 3/5 — 17 UNVERIFIED mentions.
- **Evidence:** manifest sha256 MATCH; 244 lines read; dates: none; status words: UNVERIFIED, UNVERIFIED, UNVERIFIED, UNVERIFIED, UNVERIFIED, UNVERIFIED, UNVERIFIED, UNVERIFIED.

## docs/audit/evidence/phase-b/cli/census-verify.json

- **Metadata:** 4,968 bytes · sha256 `1E0139F367D7BD1A4D4D028C4908251BDD981CF172B21C2E71358E17C90AF755` · JSON · documentation · seat figcraft-b · depth standard
- **Manifest:** bytes MATCH · sha256 MATCH · category `documentation`/`report/plan/evidence`
- **Purpose (from content):** ��{ " s c h e m a " : " h i v e r o u t e r . c l i . v 1 " , " o k " : t r u e , " c o m m a n d " : " c a p a b i l i t i e s . c e n s u s " , " d a t a " : { " c e n s u s " : { " s c h e m a " : " h i v e r o u t e r . c e n s u s . v 1 " , " d e r i v e d _ f r o m " : " s t r u m : : E n u m I t e r   o v e r   T o o l I d / R e f u s a l / A g e n t T i e r / P e r m i s s i o n C l a s s   +   T o o l R e g 
- **Mechanics / structure:** 3 lines; headings: (none)
- **Dependencies (cited in text):** none detected in assigned set
- **Quality:** correctness 4/5, maintainability 4/5, modularity 3/5, error_handling 3/5, security 5/5, docs 3/5 — no failure markers.
- **Evidence:** manifest sha256 MATCH; 3 lines read; dates: none; status words: none.

## docs/audit/evidence/phase-b/cli/census.err

- **Metadata:** 0 bytes · sha256 `E3B0C44298FC1C149AFBF4C8996FB92427AE41E4649B934CA495991B7852B855` · Log · documentation · seat figcraft-b · depth standard
- **Manifest:** bytes MATCH · sha256 MATCH · category `documentation`/`report/plan/evidence`
- **Purpose (from content):** (no markdown heading)
- **Mechanics / structure:** 0 lines; headings: (none)
- **Dependencies (cited in text):** none detected in assigned set
- **Quality:** correctness 5/5, maintainability 3/5, modularity 2/5, error_handling 4/5, security 5/5, docs 2/5 — empty stderr (expected for a clean command).
- **Evidence:** manifest sha256 MATCH; 0 lines read; dates: none; status words: none.

## docs/audit/evidence/phase-b/cli/census.out

- **Metadata:** 4,968 bytes · sha256 `1E0139F367D7BD1A4D4D028C4908251BDD981CF172B21C2E71358E17C90AF755` · Log · documentation · seat figcraft-b · depth standard
- **Manifest:** bytes MATCH · sha256 MATCH · category `documentation`/`report/plan/evidence`
- **Purpose (from content):** ��{ " s c h e m a " : " h i v e r o u t e r . c l i . v 1 " , " o k " : t r u e , " c o m m a n d " : " c a p a b i l i t i e s . c e n s u s " , " d a t a " : { " c e n s u s " : { " s c h e m a " : " h i v e r o u t e r . c e n s u s . v 1 " , " d e r i v e d _ f r o m " : " s t r u m : : E n u m I t e r   o v e r   T o o l I d / R e f u s a l / A g e n t T i e r / P e r m i s s i o n C l a s s   +   T o o l R e g 
- **Mechanics / structure:** 3 lines; headings: (none)
- **Dependencies (cited in text):** none detected in assigned set
- **Quality:** correctness 5/5, maintainability 3/5, modularity 2/5, error_handling 4/5, security 5/5, docs 2/5 — no failure markers.
- **Evidence:** manifest sha256 MATCH; 3 lines read; dates: none; status words: none.

## docs/audit/evidence/phase-b/cli/diagnose_live.err

- **Metadata:** 0 bytes · sha256 `E3B0C44298FC1C149AFBF4C8996FB92427AE41E4649B934CA495991B7852B855` · Log · documentation · seat figcraft-b · depth standard
- **Manifest:** bytes MATCH · sha256 MATCH · category `documentation`/`report/plan/evidence`
- **Purpose (from content):** (no markdown heading)
- **Mechanics / structure:** 0 lines; headings: (none)
- **Dependencies (cited in text):** none detected in assigned set
- **Quality:** correctness 5/5, maintainability 3/5, modularity 2/5, error_handling 4/5, security 5/5, docs 2/5 — empty stderr (expected for a clean command).
- **Evidence:** manifest sha256 MATCH; 0 lines read; dates: none; status words: none.

## docs/audit/evidence/phase-b/cli/diagnose_live.out

- **Metadata:** 964 bytes · sha256 `45750E20D2529F12850F1130277A1BE24B411FB2188D715109C92F137E77F109` · Log · documentation · seat figcraft-b · depth standard
- **Manifest:** bytes MATCH · sha256 MATCH · category `documentation`/`report/plan/evidence`
- **Purpose (from content):** ��{ " s c h e m a " : " h i v e r o u t e r . c l i . v 1 " , " o k " : f a l s e , " c o m m a n d " : " d i a g n o s e " , " e r r o r " : { " c o d e " : " E _ W S _ D I S C O N N E C T E D " , " m e s s a g e " : " T h e   l o c a l   W e b S o c k e t   c o n n e c t i o n   i s   n o t   a v a i l a b l e .   ( l o c a l   s e r v e r   r e t u r n e d   n o n - J S O N   f o r   h t t p : / / 1 2 7 . 0 . 0 . 
- **Mechanics / structure:** 3 lines; headings: (none)
- **Dependencies (cited in text):** none detected in assigned set
- **Quality:** correctness 5/5, maintainability 3/5, modularity 2/5, error_handling 4/5, security 5/5, docs 2/5 — no failure markers.
- **Evidence:** manifest sha256 MATCH; 3 lines read; dates: none; status words: none.

## docs/audit/evidence/phase-b/cli/health_dryrun.err

- **Metadata:** 0 bytes · sha256 `E3B0C44298FC1C149AFBF4C8996FB92427AE41E4649B934CA495991B7852B855` · Log · documentation · seat figcraft-b · depth standard
- **Manifest:** bytes MATCH · sha256 MATCH · category `documentation`/`report/plan/evidence`
- **Purpose (from content):** (no markdown heading)
- **Mechanics / structure:** 0 lines; headings: (none)
- **Dependencies (cited in text):** none detected in assigned set
- **Quality:** correctness 5/5, maintainability 3/5, modularity 2/5, error_handling 4/5, security 5/5, docs 2/5 — empty stderr (expected for a clean command).
- **Evidence:** manifest sha256 MATCH; 0 lines read; dates: none; status words: none.

## docs/audit/evidence/phase-b/cli/health_dryrun.out

- **Metadata:** 568 bytes · sha256 `B687C7DF18B9EACE6E5FB96853F67FFFBE716ADDA97C3B20FE9BF6B097473243` · Log · documentation · seat figcraft-b · depth standard
- **Manifest:** bytes MATCH · sha256 MATCH · category `documentation`/`report/plan/evidence`
- **Purpose (from content):** ��{ " s c h e m a " : " h i v e r o u t e r . c l i . v 1 " , " o k " : t r u e , " c o m m a n d " : " h e a l t h " , " d a t a " : { " d r y _ r u n " : t r u e , " r e q u e s t " : { " m e t h o d " : " G E T " , " u r l " : " h t t p : / / 1 2 7 . 0 . 0 . 1 : 8 0 8 0 / h e a l t h " , " t i m e o u t _ s e c s " : 1 0 } } , " m e t a " : { " c l i _ s c h e m a " : " h i v e r o u t e r . c l i . v 1 " , " v e 
- **Mechanics / structure:** 3 lines; headings: (none)
- **Dependencies (cited in text):** none detected in assigned set
- **Quality:** correctness 5/5, maintainability 3/5, modularity 2/5, error_handling 4/5, security 5/5, docs 2/5 — no failure markers.
- **Evidence:** manifest sha256 MATCH; 3 lines read; dates: none; status words: none.

## docs/audit/evidence/phase-b/cli/health_live.err

- **Metadata:** 0 bytes · sha256 `E3B0C44298FC1C149AFBF4C8996FB92427AE41E4649B934CA495991B7852B855` · Log · documentation · seat figcraft-b · depth standard
- **Manifest:** bytes MATCH · sha256 MATCH · category `documentation`/`report/plan/evidence`
- **Purpose (from content):** (no markdown heading)
- **Mechanics / structure:** 0 lines; headings: (none)
- **Dependencies (cited in text):** none detected in assigned set
- **Quality:** correctness 5/5, maintainability 3/5, modularity 2/5, error_handling 4/5, security 5/5, docs 2/5 — empty stderr (expected for a clean command).
- **Evidence:** manifest sha256 MATCH; 0 lines read; dates: none; status words: none.

## docs/audit/evidence/phase-b/cli/health_live.out

- **Metadata:** 1,450 bytes · sha256 `594B46239D6A2183ABFC866E23E04C5B2BA08A43B6B692B306847A5358C99DE8` · Log · documentation · seat figcraft-b · depth standard
- **Manifest:** bytes MATCH · sha256 MATCH · category `documentation`/`report/plan/evidence`
- **Purpose (from content):** ��{ " s c h e m a " : " h i v e r o u t e r . c l i . v 1 " , " o k " : t r u e , " c o m m a n d " : " h e a l t h " , " d a t a " : { " u r l " : " h t t p : / / 1 2 7 . 0 . 0 . 1 : 8 7 9 9 / h e a l t h " , " r e s p o n s e " : { " s t a t u s " : " o k " , " v e r s i o n " : " 0 . 1 . 0 " , " s c h e m a " : " h i v e r o u t e r . v 1 " , " u p t i m e _ s e c s " : 3 8 3 , " c h e c k s " : [ { " n a m e " : 
- **Mechanics / structure:** 3 lines; headings: (none)
- **Dependencies (cited in text):** none detected in assigned set
- **Quality:** correctness 5/5, maintainability 3/5, modularity 2/5, error_handling 4/5, security 5/5, docs 2/5 — no failure markers.
- **Evidence:** manifest sha256 MATCH; 3 lines read; dates: none; status words: none.

## docs/audit/evidence/phase-b/cli/preflight-path.err

- **Metadata:** 0 bytes · sha256 `E3B0C44298FC1C149AFBF4C8996FB92427AE41E4649B934CA495991B7852B855` · Log · documentation · seat figcraft-b · depth standard
- **Manifest:** bytes MATCH · sha256 MATCH · category `documentation`/`report/plan/evidence`
- **Purpose (from content):** (no markdown heading)
- **Mechanics / structure:** 0 lines; headings: (none)
- **Dependencies (cited in text):** none detected in assigned set
- **Quality:** correctness 5/5, maintainability 3/5, modularity 2/5, error_handling 4/5, security 5/5, docs 2/5 — empty stderr (expected for a clean command).
- **Evidence:** manifest sha256 MATCH; 0 lines read; dates: none; status words: none.

## docs/audit/evidence/phase-b/cli/preflight-path.out

- **Metadata:** 1,374 bytes · sha256 `C39AF6AF2C1B26456B6E75B048D245FC147C9E5B8196F1BB4672A22F5D2091BA` · Log · documentation · seat figcraft-b · depth standard
- **Manifest:** bytes MATCH · sha256 MATCH · category `documentation`/`report/plan/evidence`
- **Purpose (from content):** ��{ " s c h e m a " : " h i v e r o u t e r . c l i . v 1 " , " o k " : t r u e , " c o m m a n d " : " p r e f l i g h t " , " d a t a " : { " r e a d y " : t r u e , " c h e c k s " : [ { " n a m e " : " n o d e " , " o k " : t r u e , " r e q u i r e d " : " > = 2 2 . 1 2 . 0 " , " o b s e r v e d " : " v 2 6 . 7 . 0 " , " c o d e " : n u l l } , { " n a m e " : " r u s t c " , " o k " : t r u e , " r e q u i r e 
- **Mechanics / structure:** 3 lines; headings: (none)
- **Dependencies (cited in text):** none detected in assigned set
- **Quality:** correctness 5/5, maintainability 3/5, modularity 2/5, error_handling 4/5, security 5/5, docs 2/5 — no failure markers.
- **Evidence:** manifest sha256 MATCH; 3 lines read; dates: none; status words: none.

## docs/audit/evidence/phase-b/cli/preflight.err

- **Metadata:** 0 bytes · sha256 `E3B0C44298FC1C149AFBF4C8996FB92427AE41E4649B934CA495991B7852B855` · Log · documentation · seat figcraft-b · depth standard
- **Manifest:** bytes MATCH · sha256 MATCH · category `documentation`/`report/plan/evidence`
- **Purpose (from content):** (no markdown heading)
- **Mechanics / structure:** 0 lines; headings: (none)
- **Dependencies (cited in text):** none detected in assigned set
- **Quality:** correctness 5/5, maintainability 3/5, modularity 2/5, error_handling 4/5, security 5/5, docs 2/5 — empty stderr (expected for a clean command).
- **Evidence:** manifest sha256 MATCH; 0 lines read; dates: none; status words: none.

## docs/audit/evidence/phase-b/cli/preflight.out

- **Metadata:** 1,640 bytes · sha256 `2457B48AB44CF0698EA734428169F8C097E2F74969A1CA7D8525AF02914E0189` · Log · documentation · seat figcraft-b · depth standard
- **Manifest:** bytes MATCH · sha256 MATCH · category `documentation`/`report/plan/evidence`
- **Purpose (from content):** ��{ " s c h e m a " : " h i v e r o u t e r . c l i . v 1 " , " o k " : f a l s e , " c o m m a n d " : " p r e f l i g h t " , " e r r o r " : { " c o d e " : " E _ T O O L C H A I N _ M S R V " , " m e s s a g e " : " T h e   R u s t   t o o l c h a i n   i s   o l d e r   t h a n   t h e   r e q u i r e d   M S R V   1 . 8 9 . 0 . " , " h o w _ t o _ f i x " : " I n s t a l l   R u s t   1 . 8 9 . 0   o r   n e w 
- **Mechanics / structure:** 3 lines; headings: (none)
- **Dependencies (cited in text):** none detected in assigned set
- **Quality:** correctness 5/5, maintainability 3/5, modularity 2/5, error_handling 4/5, security 5/5, docs 2/5 — no failure markers.
- **Evidence:** manifest sha256 MATCH; 3 lines read; dates: none; status words: none.

## docs/audit/evidence/phase-b/cli/provider_offpeak.err

- **Metadata:** 0 bytes · sha256 `E3B0C44298FC1C149AFBF4C8996FB92427AE41E4649B934CA495991B7852B855` · Log · documentation · seat figcraft-b · depth standard
- **Manifest:** bytes MATCH · sha256 MATCH · category `documentation`/`report/plan/evidence`
- **Purpose (from content):** (no markdown heading)
- **Mechanics / structure:** 0 lines; headings: (none)
- **Dependencies (cited in text):** none detected in assigned set
- **Quality:** correctness 5/5, maintainability 3/5, modularity 2/5, error_handling 4/5, security 5/5, docs 2/5 — empty stderr (expected for a clean command).
- **Evidence:** manifest sha256 MATCH; 0 lines read; dates: none; status words: none.

## docs/audit/evidence/phase-b/cli/provider_offpeak.out

- **Metadata:** 1,824 bytes · sha256 `D04DFCA68B01BB11F04C4C93B28B546FA1E83AD41D5B0AEB55BA828992829666` · Log · documentation · seat figcraft-b · depth standard
- **Manifest:** bytes MATCH · sha256 MATCH · category `documentation`/`report/plan/evidence`
- **Purpose (from content):** ��{ " s c h e m a " : " h i v e r o u t e r . c l i . v 1 " , " o k " : t r u e , " c o m m a n d " : " p r o v i d e r . c o s t " , " d a t a " : { " m o d e l " : " d e e p s e e k - f l a s h " , " u s a g e " : { " p r o m p t _ t o k e n s " : 2 0 0 0 0 0 0 , " c o m p l e t i o n _ t o k e n s " : 1 0 0 0 0 0 0 , " t o t a l _ t o k e n s " : 3 0 0 0 0 0 0 , " p r o m p t _ c a c h e _ h i t _ t o k e n s " : 
- **Mechanics / structure:** 3 lines; headings: (none)
- **Dependencies (cited in text):** none detected in assigned set
- **Quality:** correctness 5/5, maintainability 3/5, modularity 2/5, error_handling 4/5, security 5/5, docs 2/5 — no failure markers.
- **Evidence:** manifest sha256 MATCH; 3 lines read; dates: none; status words: none.

## docs/audit/evidence/phase-b/cli/provider_peak.err

- **Metadata:** 0 bytes · sha256 `E3B0C44298FC1C149AFBF4C8996FB92427AE41E4649B934CA495991B7852B855` · Log · documentation · seat figcraft-b · depth standard
- **Manifest:** bytes MATCH · sha256 MATCH · category `documentation`/`report/plan/evidence`
- **Purpose (from content):** (no markdown heading)
- **Mechanics / structure:** 0 lines; headings: (none)
- **Dependencies (cited in text):** none detected in assigned set
- **Quality:** correctness 5/5, maintainability 3/5, modularity 2/5, error_handling 4/5, security 5/5, docs 2/5 — empty stderr (expected for a clean command).
- **Evidence:** manifest sha256 MATCH; 0 lines read; dates: none; status words: none.

## docs/audit/evidence/phase-b/cli/provider_peak.out

- **Metadata:** 1,816 bytes · sha256 `0FBE228F6B9AF4BB027310ACE1A9ECD8D0DA06D310E6CEA514CD302D2D6FC3CC` · Log · documentation · seat figcraft-b · depth standard
- **Manifest:** bytes MATCH · sha256 MATCH · category `documentation`/`report/plan/evidence`
- **Purpose (from content):** ��{ " s c h e m a " : " h i v e r o u t e r . c l i . v 1 " , " o k " : t r u e , " c o m m a n d " : " p r o v i d e r . c o s t " , " d a t a " : { " m o d e l " : " d e e p s e e k - f l a s h " , " u s a g e " : { " p r o m p t _ t o k e n s " : 2 0 0 0 0 0 0 , " c o m p l e t i o n _ t o k e n s " : 1 0 0 0 0 0 0 , " t o t a l _ t o k e n s " : 3 0 0 0 0 0 0 , " p r o m p t _ c a c h e _ h i t _ t o k e n s " : 
- **Mechanics / structure:** 3 lines; headings: (none)
- **Dependencies (cited in text):** none detected in assigned set
- **Quality:** correctness 5/5, maintainability 3/5, modularity 2/5, error_handling 4/5, security 5/5, docs 2/5 — no failure markers.
- **Evidence:** manifest sha256 MATCH; 3 lines read; dates: none; status words: none.

## docs/audit/evidence/phase-b/cli/schema.err

- **Metadata:** 0 bytes · sha256 `E3B0C44298FC1C149AFBF4C8996FB92427AE41E4649B934CA495991B7852B855` · Log · documentation · seat figcraft-b · depth standard
- **Manifest:** bytes MATCH · sha256 MATCH · category `documentation`/`report/plan/evidence`
- **Purpose (from content):** (no markdown heading)
- **Mechanics / structure:** 0 lines; headings: (none)
- **Dependencies (cited in text):** none detected in assigned set
- **Quality:** correctness 5/5, maintainability 3/5, modularity 2/5, error_handling 4/5, security 5/5, docs 2/5 — empty stderr (expected for a clean command).
- **Evidence:** manifest sha256 MATCH; 0 lines read; dates: none; status words: none.

## docs/audit/evidence/phase-b/cli/schema.out

- **Metadata:** 4,000 bytes · sha256 `BB9F478733F49FE56EF845D46BFBCAB486BCB1E702349F874B9AB1F90FE2CE67` · Log · documentation · seat figcraft-b · depth standard
- **Manifest:** bytes MATCH · sha256 MATCH · category `documentation`/`report/plan/evidence`
- **Purpose (from content):** ��{ " s c h e m a " : " h i v e r o u t e r . c l i . v 1 " , " o k " : t r u e , " c o m m a n d " : " s c h e m a " , " d a t a " : { " s c h e m a " : " h i v e r o u t e r . c l i . v 1 " , " c o m m a n d s " : [ { " n a m e " : " v e r s i o n " , " r e s o u r c e " : " v e r s i o n " , " a c t i o n " : " " , " s u m m a r y " : " P r i n t   C L I / b i n a r y   v e r s i o n   a n d   s c h e m a   i d s 
- **Mechanics / structure:** 3 lines; headings: (none)
- **Dependencies (cited in text):** none detected in assigned set
- **Quality:** correctness 5/5, maintainability 3/5, modularity 2/5, error_handling 4/5, security 5/5, docs 2/5 — no failure markers.
- **Evidence:** manifest sha256 MATCH; 3 lines read; dates: none; status words: none.

## docs/audit/evidence/phase-b/cli/tool_missing.err

- **Metadata:** 0 bytes · sha256 `E3B0C44298FC1C149AFBF4C8996FB92427AE41E4649B934CA495991B7852B855` · Log · documentation · seat figcraft-b · depth standard
- **Manifest:** bytes MATCH · sha256 MATCH · category `documentation`/`report/plan/evidence`
- **Purpose (from content):** (no markdown heading)
- **Mechanics / structure:** 0 lines; headings: (none)
- **Dependencies (cited in text):** none detected in assigned set
- **Quality:** correctness 5/5, maintainability 3/5, modularity 2/5, error_handling 4/5, security 5/5, docs 2/5 — empty stderr (expected for a clean command).
- **Evidence:** manifest sha256 MATCH; 0 lines read; dates: none; status words: none.

## docs/audit/evidence/phase-b/cli/tool_missing.out

- **Metadata:** 834 bytes · sha256 `C9411A2CA776C312D565701B46A22B65461304408B024B2A70FA8E1A16AFB489` · Log · documentation · seat figcraft-b · depth standard
- **Manifest:** bytes MATCH · sha256 MATCH · category `documentation`/`report/plan/evidence`
- **Purpose (from content):** ��{ " s c h e m a " : " h i v e r o u t e r . c l i . v 1 " , " o k " : f a l s e , " c o m m a n d " : " t o o l s . d e s c r i b e " , " e r r o r " : { " c o d e " : " E _ T O O L _ N O T _ F O U N D " , " m e s s a g e " : " T h e   r e q u e s t e d   t o o l   i s   n o t   r e g i s t e r e d .   ( n o   t o o l   \ " r m _ r f \ "   i n   t h e   f r o z e n   v o c a b u l a r y ) " , " h o w _ t o _ f i x 
- **Mechanics / structure:** 3 lines; headings: (none)
- **Dependencies (cited in text):** none detected in assigned set
- **Quality:** correctness 5/5, maintainability 3/5, modularity 2/5, error_handling 4/5, security 5/5, docs 2/5 — no failure markers.
- **Evidence:** manifest sha256 MATCH; 3 lines read; dates: none; status words: none.

## docs/audit/evidence/phase-b/cli/tools_describe.err

- **Metadata:** 0 bytes · sha256 `E3B0C44298FC1C149AFBF4C8996FB92427AE41E4649B934CA495991B7852B855` · Log · documentation · seat figcraft-b · depth standard
- **Manifest:** bytes MATCH · sha256 MATCH · category `documentation`/`report/plan/evidence`
- **Purpose (from content):** (no markdown heading)
- **Mechanics / structure:** 0 lines; headings: (none)
- **Dependencies (cited in text):** none detected in assigned set
- **Quality:** correctness 5/5, maintainability 3/5, modularity 2/5, error_handling 4/5, security 5/5, docs 2/5 — empty stderr (expected for a clean command).
- **Evidence:** manifest sha256 MATCH; 0 lines read; dates: none; status words: none.

## docs/audit/evidence/phase-b/cli/tools_describe.out

- **Metadata:** 1,934 bytes · sha256 `3C3D6EFE7B6F3814FD9AF738269031214B06A3A5302D1CAA283C53009C7AC201` · Log · documentation · seat figcraft-b · depth standard
- **Manifest:** bytes MATCH · sha256 MATCH · category `documentation`/`report/plan/evidence`
- **Purpose (from content):** ��{ " s c h e m a " : " h i v e r o u t e r . c l i . v 1 " , " o k " : t r u e , " c o m m a n d " : " t o o l s . d e s c r i b e " , " d a t a " : { " i d " : " f i l e _ w r i t e " , " d e s c r i p t i o n " : " W r i t e   a   f i l e   i n s i d e   t h e   w o r k s p a c e   j a i l   ( r e q u i r e s   a p p r o v a l ) . " , " p e r m i s s i o n _ c l a s s " : " m u t a t e " , " r e q u i r e s _ a p 
- **Mechanics / structure:** 3 lines; headings: (none)
- **Dependencies (cited in text):** none detected in assigned set
- **Quality:** correctness 5/5, maintainability 3/5, modularity 2/5, error_handling 4/5, security 5/5, docs 2/5 — no failure markers.
- **Evidence:** manifest sha256 MATCH; 3 lines read; dates: none; status words: none.

## docs/audit/evidence/phase-b/cli/tools_list.err

- **Metadata:** 0 bytes · sha256 `E3B0C44298FC1C149AFBF4C8996FB92427AE41E4649B934CA495991B7852B855` · Log · documentation · seat figcraft-b · depth standard
- **Manifest:** bytes MATCH · sha256 MATCH · category `documentation`/`report/plan/evidence`
- **Purpose (from content):** (no markdown heading)
- **Mechanics / structure:** 0 lines; headings: (none)
- **Dependencies (cited in text):** none detected in assigned set
- **Quality:** correctness 5/5, maintainability 3/5, modularity 2/5, error_handling 4/5, security 5/5, docs 2/5 — empty stderr (expected for a clean command).
- **Evidence:** manifest sha256 MATCH; 0 lines read; dates: none; status words: none.

## docs/audit/evidence/phase-b/cli/tools_list.out

- **Metadata:** 2,432 bytes · sha256 `877888142359985676B4020697DB84FC27072240257E167EEABC4096A0CAD042` · Log · documentation · seat figcraft-b · depth standard
- **Manifest:** bytes MATCH · sha256 MATCH · category `documentation`/`report/plan/evidence`
- **Purpose (from content):** ��{ " s c h e m a " : " h i v e r o u t e r . c l i . v 1 " , " o k " : t r u e , " c o m m a n d " : " t o o l s . l i s t " , " d a t a " : { " c o u n t " : 7 , " t o o l s " : [ { " i d " : " s p a w n _ a g e n t " , " d e s c r i p t i o n " : " S p a w n   a   c h i l d   a g e n t   ( d e p t h / f a n - o u t / a g e n t   c a p s   e n f o r c e d   b e f o r e   a n y   s p a w n ) . " , " p e r m i s s i 
- **Mechanics / structure:** 3 lines; headings: (none)
- **Dependencies (cited in text):** none detected in assigned set
- **Quality:** correctness 5/5, maintainability 3/5, modularity 2/5, error_handling 4/5, security 5/5, docs 2/5 — no failure markers.
- **Evidence:** manifest sha256 MATCH; 3 lines read; dates: none; status words: none.

## docs/audit/evidence/phase-b/cli/unknown.err

- **Metadata:** 0 bytes · sha256 `E3B0C44298FC1C149AFBF4C8996FB92427AE41E4649B934CA495991B7852B855` · Log · documentation · seat figcraft-b · depth standard
- **Manifest:** bytes MATCH · sha256 MATCH · category `documentation`/`report/plan/evidence`
- **Purpose (from content):** (no markdown heading)
- **Mechanics / structure:** 0 lines; headings: (none)
- **Dependencies (cited in text):** none detected in assigned set
- **Quality:** correctness 5/5, maintainability 3/5, modularity 2/5, error_handling 4/5, security 5/5, docs 2/5 — empty stderr (expected for a clean command).
- **Evidence:** manifest sha256 MATCH; 0 lines read; dates: none; status words: none.

## docs/audit/evidence/phase-b/cli/unknown.out

- **Metadata:** 824 bytes · sha256 `F89A0CEB8515EA8D9D143D023521C623E386559C06268ECC9A1E1EBE84CE3D86` · Log · documentation · seat figcraft-b · depth standard
- **Manifest:** bytes MATCH · sha256 MATCH · category `documentation`/`report/plan/evidence`
- **Purpose (from content):** ��{ " s c h e m a " : " h i v e r o u t e r . c l i . v 1 " , " o k " : f a l s e , " c o m m a n d " : " f r o b n i c a t e " , " e r r o r " : { " c o d e " : " E _ C O M M A N D _ U N K N O W N " , " m e s s a g e " : " T h e   c o m m a n d   n a m e   i s   n o t   p a r t   o f   t h e   f r o z e n   U I - C O M M A N D S - v 1   v o c a b u l a r y . " , " h o w _ t o _ f i x " : " U s e   a   c o m m a n d 
- **Mechanics / structure:** 3 lines; headings: (none)
- **Dependencies (cited in text):** none detected in assigned set
- **Quality:** correctness 5/5, maintainability 3/5, modularity 2/5, error_handling 4/5, security 5/5, docs 2/5 — no failure markers.
- **Evidence:** manifest sha256 MATCH; 3 lines read; dates: none; status words: none.

## docs/audit/evidence/phase-b/cli/usage_error.err

- **Metadata:** 0 bytes · sha256 `E3B0C44298FC1C149AFBF4C8996FB92427AE41E4649B934CA495991B7852B855` · Log · documentation · seat figcraft-b · depth standard
- **Manifest:** bytes MATCH · sha256 MATCH · category `documentation`/`report/plan/evidence`
- **Purpose (from content):** (no markdown heading)
- **Mechanics / structure:** 0 lines; headings: (none)
- **Dependencies (cited in text):** none detected in assigned set
- **Quality:** correctness 5/5, maintainability 3/5, modularity 2/5, error_handling 4/5, security 5/5, docs 2/5 — empty stderr (expected for a clean command).
- **Evidence:** manifest sha256 MATCH; 0 lines read; dates: none; status words: none.

## docs/audit/evidence/phase-b/cli/usage_error.out

- **Metadata:** 704 bytes · sha256 `F621505E51474A1FD6CBBAD40A092B2913752A1E6A2E62CF7732D8C6486084D9` · Log · documentation · seat figcraft-b · depth standard
- **Manifest:** bytes MATCH · sha256 MATCH · category `documentation`/`report/plan/evidence`
- **Purpose (from content):** ��{ " s c h e m a " : " h i v e r o u t e r . c l i . v 1 " , " o k " : f a l s e , " c o m m a n d " : " t o o l s . d e s c r i b e " , " e r r o r " : { " c o d e " : " E _ S C H E M A _ I N V A L I D " , " m e s s a g e " : " m i s s i n g   < t o o l _ i d > " , " h o w _ t o _ f i x " : " R u n   ` h i v e r o u t e r   - - h e l p `   o r   ` h i v e r o u t e r   s c h e m a   < c o m m a n d > ` . " , " r e 
- **Mechanics / structure:** 3 lines; headings: (none)
- **Dependencies (cited in text):** none detected in assigned set
- **Quality:** correctness 5/5, maintainability 3/5, modularity 2/5, error_handling 4/5, security 5/5, docs 2/5 — no failure markers.
- **Evidence:** manifest sha256 MATCH; 3 lines read; dates: none; status words: none.

## docs/audit/evidence/phase-b/cli/version-human.err

- **Metadata:** 726 bytes · sha256 `23CF6824D5209A9717934E8CE3DDA1A752664252D008FD9B508010AB8CB44290` · Log · documentation · seat figcraft-b · depth standard
- **Manifest:** bytes MATCH · sha256 MATCH · category `documentation`/`report/plan/evidence`
- **Purpose (from content):** ��h i v e r o u t e r - c l i . e x e   :   h i v e r o u t e r   0 . 1 . 0     A t   l i n e : 1 3   c h a r : 9 2 8 
- **Mechanics / structure:** 15 lines; headings: (none)
- **Dependencies (cited in text):** none detected in assigned set
- **Quality:** correctness 5/5, maintainability 3/5, modularity 2/5, error_handling 4/5, security 5/5, docs 2/5 — no failure markers.
- **Evidence:** manifest sha256 MATCH; 15 lines read; dates: none; status words: none.

## docs/audit/evidence/phase-b/cli/version-human.out

- **Metadata:** 0 bytes · sha256 `E3B0C44298FC1C149AFBF4C8996FB92427AE41E4649B934CA495991B7852B855` · Log · documentation · seat figcraft-b · depth standard
- **Manifest:** bytes MATCH · sha256 MATCH · category `documentation`/`report/plan/evidence`
- **Purpose (from content):** (no markdown heading)
- **Mechanics / structure:** 0 lines; headings: (none)
- **Dependencies (cited in text):** none detected in assigned set
- **Quality:** correctness 5/5, maintainability 3/5, modularity 2/5, error_handling 4/5, security 5/5, docs 2/5 — no failure markers.
- **Evidence:** manifest sha256 MATCH; 0 lines read; dates: none; status words: none.

## docs/audit/evidence/phase-b/cli/version.err

- **Metadata:** 0 bytes · sha256 `E3B0C44298FC1C149AFBF4C8996FB92427AE41E4649B934CA495991B7852B855` · Log · documentation · seat figcraft-b · depth standard
- **Manifest:** bytes MATCH · sha256 MATCH · category `documentation`/`report/plan/evidence`
- **Purpose (from content):** (no markdown heading)
- **Mechanics / structure:** 0 lines; headings: (none)
- **Dependencies (cited in text):** none detected in assigned set
- **Quality:** correctness 5/5, maintainability 3/5, modularity 2/5, error_handling 4/5, security 5/5, docs 2/5 — empty stderr (expected for a clean command).
- **Evidence:** manifest sha256 MATCH; 0 lines read; dates: none; status words: none.

## docs/audit/evidence/phase-b/cli/version.out

- **Metadata:** 818 bytes · sha256 `84BEFDB61A976C7D26D3AD31787F698CBF80059E2D70785D92DD6B15FFFB2E09` · Log · documentation · seat figcraft-b · depth standard
- **Manifest:** bytes MATCH · sha256 MATCH · category `documentation`/`report/plan/evidence`
- **Purpose (from content):** ��{ " s c h e m a " : " h i v e r o u t e r . c l i . v 1 " , " o k " : t r u e , " c o m m a n d " : " v e r s i o n " , " d a t a " : { " v e r s i o n " : " 0 . 1 . 0 " , " c l i _ s c h e m a " : " h i v e r o u t e r . c l i . v 1 " , " h e a l t h _ s c h e m a " : " h i v e r o u t e r . h e a l t h . v 1 " , " d i a g n o s e _ s c h e m a " : " h i v e r o u t e r . d i a g n o s e . v 1 " , " c o n t r a c 
- **Mechanics / structure:** 3 lines; headings: (none)
- **Dependencies (cited in text):** none detected in assigned set
- **Quality:** correctness 5/5, maintainability 3/5, modularity 2/5, error_handling 4/5, security 5/5, docs 2/5 — no failure markers.
- **Evidence:** manifest sha256 MATCH; 3 lines read; dates: none; status words: none.

## docs/audit/evidence/phase-b/live-probe-verifier-v2.json

- **Metadata:** 4,890 bytes · sha256 `19D660FBEBB3C9EAC611F6E523C3A4657D1EA5891594112821D02A25E2DFA695` · JSON · documentation · seat figcraft-b · depth standard
- **Manifest:** bytes MATCH · sha256 MATCH · category `documentation`/`report/plan/evidence`
- **Purpose (from content):** { "generated_at": "2026-11-06T13:47:29.844Z", "base": "http://127.0.0.1:8799",
- **Mechanics / structure:** 176 lines; headings: (none)
- **Dependencies (cited in text):** none detected in assigned set
- **Quality:** correctness 4/5, maintainability 4/5, modularity 3/5, error_handling 3/5, security 5/5, docs 3/5 — no failure markers.
- **Evidence:** manifest sha256 MATCH; 176 lines read; dates: 2026-11-06; status words: none.

## docs/audit/evidence/phase-b/live-probe-verifier-v2.out.txt

- **Metadata:** 8,882 bytes · sha256 `8A87F5D00137028CD9EEBC438F874B0652F0587F48958C959D366D4095683C9A` · Text · documentation · seat figcraft-b · depth standard
- **Manifest:** bytes MATCH · sha256 MATCH · category `documentation`/`report/plan/evidence`
- **Purpose (from content):** ��{         " o k " :   t r u e , 
- **Mechanics / structure:** 323 lines; headings: (none)
- **Dependencies (cited in text):** none detected in assigned set
- **Quality:** correctness 4/5, maintainability 3/5, modularity 3/5, error_handling 3/5, security 5/5, docs 3/5 — no failure markers.
- **Evidence:** manifest sha256 MATCH; 323 lines read; dates: none; status words: none.

## docs/audit/evidence/phase-b/live-probe-verifier.json

- **Metadata:** 4,890 bytes · sha256 `19D660FBEBB3C9EAC611F6E523C3A4657D1EA5891594112821D02A25E2DFA695` · JSON · documentation · seat figcraft-b · depth standard
- **Manifest:** bytes MATCH · sha256 MATCH · category `documentation`/`report/plan/evidence`
- **Purpose (from content):** { "generated_at": "2026-11-06T13:47:29.844Z", "base": "http://127.0.0.1:8799",
- **Mechanics / structure:** 176 lines; headings: (none)
- **Dependencies (cited in text):** none detected in assigned set
- **Quality:** correctness 4/5, maintainability 4/5, modularity 3/5, error_handling 3/5, security 5/5, docs 3/5 — no failure markers.
- **Evidence:** manifest sha256 MATCH; 176 lines read; dates: 2026-11-06; status words: none.

## docs/audit/evidence/phase-b/live-probe-verifier.mjs

- **Metadata:** 6,414 bytes · sha256 `5C21CD473E430DE4D290591B7AD7730B7E2DA756BF75619387BA477DB6359964` · JavaScript · documentation · seat figcraft-b · depth standard
- **Manifest:** bytes MATCH · sha256 MATCH · category `documentation`/`report/plan/evidence`
- **Purpose (from content):** // Independent verifier live probe v2 — browser e2e evidence for C-026 / Phase B. // Uses installed Chrome via the e2e project's Playwright (no browser download). import { writeFileSync } from 'node:fs';
- **Mechanics / structure:** 130 lines; headings: (none)
- **Dependencies (cited in text):** `audit/evidence/phase-b/live-probe-verifier.json`
- **Quality:** correctness 4/5, maintainability 3/5, modularity 3/5, error_handling 3/5, security 5/5, docs 3/5 — no failure markers.
- **Evidence:** manifest sha256 MATCH; 130 lines read; dates: none; status words: none.

## docs/audit/evidence/phase-b/live-probe-verifier.out.txt

- **Metadata:** 9,416 bytes · sha256 `25B4CE4E8AEFF40D3529670463E3BBC31F7D988E7AE902AC9D0A9F83F6AA6DC8` · Text · documentation · seat figcraft-b · depth standard
- **Manifest:** bytes MATCH · sha256 MATCH · category `documentation`/`report/plan/evidence`
- **Purpose (from content):** ��{         " o k " :   t r u e , 
- **Mechanics / structure:** 345 lines; headings: (none)
- **Dependencies (cited in text):** none detected in assigned set
- **Quality:** correctness 4/5, maintainability 3/5, modularity 3/5, error_handling 3/5, security 5/5, docs 3/5 — no failure markers.
- **Evidence:** manifest sha256 MATCH; 345 lines read; dates: none; status words: none.

## docs/audit/evidence/toolchain/hello.c

- **Metadata:** 75 bytes · sha256 `1567B81E13E47B8AAFA199A1199022360B4B82279A84E56284503747AAF05DEB` · C · documentation · seat figcraft-b · depth standard
- **Manifest:** bytes MATCH · sha256 MATCH · category `documentation`/`report/plan/evidence`
- **Purpose (from content):** #include <stdio.h> int main(void) { printf("c-compile-ok\n"); return 0; }
- **Mechanics / structure:** 2 lines; headings: (none)
- **Dependencies (cited in text):** none detected in assigned set
- **Quality:** correctness 4/5, maintainability 3/5, modularity 3/5, error_handling 3/5, security 5/5, docs 3/5 — no failure markers.
- **Evidence:** manifest sha256 MATCH; 2 lines read; dates: none; status words: none.

## docs/audit/evidence/toolchain/hello.exe

- **Metadata:** 49,876 bytes · sha256 `8F186FF8B9B159DD6107C4D2F5C57362B0679733EB1E688BB4E995B3B257F119` · PE-binary · documentation · seat figcraft-b · depth standard
- **Manifest:** bytes MATCH · sha256 MATCH · category `documentation`/`report/plan/evidence`
- **Purpose / provenance:** binary evidence artifact (`.exe`), committed as raw build output evidence; content not prose-readable.
- **Mechanics:** not applicable (binary). SHA-256 verified against manifest; size recorded in metadata.
- **Dependencies:** none resolvable from content; see DEPENDENCIES.md for the referencing documents.
- **Quality:** correctness 3/5 (raw artifact, provenance unstated in-file); maintainability 2/5; evidence 3/5 (hash only).
- **Evidence / UNVERIFIED:** hash match proven; semantic content UNVERIFIED (binary).

## docs/audit/evidence/toolchain/hello.rs

- **Metadata:** 45 bytes · sha256 `913D97C057D169DA67B212EDC78D10AA472627FDC21C77B29E98C1060E4CC3D4` · Rust · documentation · seat figcraft-b · depth standard
- **Manifest:** bytes MATCH · sha256 MATCH · category `documentation`/`report/plan/evidence`
- **Purpose (from content):** fn main() { println!("rust-gnu-link-ok"); }
- **Mechanics / structure:** 1 lines; headings: (none)
- **Dependencies (cited in text):** none detected in assigned set
- **Quality:** correctness 4/5, maintainability 3/5, modularity 3/5, error_handling 3/5, security 5/5, docs 3/5 — no failure markers.
- **Evidence:** manifest sha256 MATCH; 1 lines read; dates: none; status words: none.

## docs/audit/evidence/toolchain/hello_rs.exe

- **Metadata:** 5,088,192 bytes · sha256 `8DF34E8A4D2C71099027DDD263FE52F040969F3F4E02CFAA26D252A173230ED3` · PE-binary · documentation · seat figcraft-b · depth standard
- **Manifest:** bytes MATCH · sha256 MATCH · category `documentation`/`report/plan/evidence`
- **Purpose / provenance:** binary evidence artifact (`.exe`), committed as raw build output evidence; content not prose-readable.
- **Mechanics:** not applicable (binary). SHA-256 verified against manifest; size recorded in metadata.
- **Dependencies:** none resolvable from content; see DEPENDENCIES.md for the referencing documents.
- **Quality:** correctness 3/5 (raw artifact, provenance unstated in-file); maintainability 2/5; evidence 3/5 (hash only).
- **Evidence / UNVERIFIED:** hash match proven; semantic content UNVERIFIED (binary).

## docs/claims/CLAIM-REGISTER.csv

- **Metadata:** 24,292 bytes · sha256 `1518294B77D253239069F26A19C9EFC2A3FB955E43E4F0ABEFF1C40BB61C9AD7` · CSV · documentation · seat figcraft-b · depth standard
- **Manifest:** bytes MATCH · sha256 MATCH · category `documentation`/`report/plan/evidence`
- **Purpose (from content):** claim_id,claim_text,location,type,source_doc,source_url,fetched_at,status,also_in,notes C-001,All nine blueprint source-of-truth hashes match the locked values,docs/plans/HASH-VERIFICATION.md,fact,EXECUTION-PROMPT.md §0,,2026-11-06,VERIFIED,HASH-VERIFICATION.md,Get-FileHash observed; full table recorded C-002,FigCraft headless CLI is installed and exposes 597 commands,C:\Program Files\FigCraft\figcraft-cli.exe,capabi
- **Mechanics / structure:** 81 lines; headings: (none) · markers: PASS=1, UNVERIFIED=3, PASS-WITH-NOTES=1
- **Dependencies (cited in text):** `plans/D2-D3-WIRING-EVIDENCE.md`, `plans/FRONTEND-STATUS.md`, `plans/HASH-VERIFICATION.md`, `plans/PHASE-1-EVIDENCE.md`, `plans/PHASE-11-EVIDENCE.md`, `plans/PHASE-12-EVIDENCE.md`, `plans/PHASE-2-EVIDENCE.md`, `plans/PHASE-3-EVIDENCE.md`, `plans/PHASE-4-EVIDENCE.md`, `plans/PHASE-5-EVIDENCE.md`
- **Quality:** correctness 4/5, maintainability 4/5, modularity 3/5, error_handling 3/5, security 5/5, docs 3/5 — 3 UNVERIFIED mentions.
- **Evidence:** manifest sha256 MATCH; 81 lines read; dates: 2026-11-06; status words: UNVERIFIED, PASS-WITH-NOTES, UNVERIFIED, UNVERIFIED.

## docs/claims/CLAIM-REGISTER.json

- **Metadata:** 2,717 bytes · sha256 `C58869DAE44D8476AEC722FCE324957390642A18D88CE4B7C05B605595EA5603` · JSON · documentation · seat figcraft-b · depth standard
- **Manifest:** bytes MATCH · sha256 MATCH · category `documentation`/`report/plan/evidence`
- **Purpose (from content):** { "counts": { "total": 80,
- **Mechanics / structure:** 87 lines; headings: (none) · markers: UNVERIFIED=1
- **Dependencies (cited in text):** `audit/BASELINE-VERIFICATION.md`, `plans/D2-D3-WIRING-EVIDENCE.md`, `plans/PHASE-1-EVIDENCE.md`, `plans/PHASE-3-EVIDENCE.md`, `plans/PHASE-4-EVIDENCE.md`, `plans/PHASE-7-8-10-EVIDENCE.md`, `plans/PHASE-9-EVIDENCE.md`, `plans/TOOLCHAIN.md`
- **Quality:** correctness 4/5, maintainability 4/5, modularity 3/5, error_handling 3/5, security 5/5, docs 3/5 — 1 UNVERIFIED mentions.
- **Evidence:** manifest sha256 MATCH; 87 lines read; dates: none; status words: UNVERIFIED.

## docs/claims/SKILL-USE.md

- **Metadata:** 1,532 bytes · sha256 `7D649CACB3A441DD1205343BAF40811B2F244B7BD9BD6281263BE4F86D9513A7` · Markdown · documentation · seat figcraft-b · depth standard
- **Manifest:** bytes MATCH · sha256 MATCH · category `documentation`/`report/plan/evidence`
- **Purpose (from content):** # SKILL USE RECORD — HiveMind v1 execution Skills were loaded from the session skill catalog at execution start (their instructions were present in the execution prompt/skill blocks). Substitutions: none at W0. Recorded per EXECUTION-PROMPT.md §2.
- **Mechanics / structure:** 20 lines; headings: (none)
- **Dependencies (cited in text):** `claims/CLAIM-REGISTER.csv`, `contracts/ADAPTERS.md`, `plans/PHASE-0-RECON.md`
- **Quality:** correctness 4/5, maintainability 4/5, modularity 3/5, error_handling 3/5, security 5/5, docs 4/5 — no failure markers.
- **Evidence:** manifest sha256 MATCH; 20 lines read; dates: none; status words: none.

## docs/contracts/ADAPTERS.md

- **Metadata:** 10,628 bytes · sha256 `ED7FEEE4747CF8980A915D9B9112190DD57A5BBA336D7736551C3EC81A4B5714` · Markdown · documentation · seat figcraft-b · depth standard
- **Manifest:** bytes MATCH · sha256 MATCH · category `documentation`/`report/plan/evidence`
- **Purpose (from content):** # ADAPTERS — donor mechanisms adapted into HiveMind v1 Owner: B-Provider (Phase 5/6/11/12). Written per `harness-code-adapter`: a donor repo is an architecture source, never a file to paste. Every row records the pinned version, licence, adapter file, the invariant
- **Mechanics / structure:** 113 lines; headings: A1. `wiremock 0.6.5` — HTTP provider test double (DEV-ONLY); A2. `win32job 2.0.3` — Windows containment owner (Phase 6); A3. `schemars 1.2.3` + `jsonschema 0.60.2` — one schema source (Phase 6); A4. `rusqlite 0.41.1` (bundled) — approval persistence (Phase 6); A5. `tracing-subscriber 0.3.23` + `tracing-appender 0.2.5` — local JSONL audit sink (Phase 11); A6. `sysinfo 0.40.1` — `/health` sampling (Phase 11); A7. `noti · markers: BLOCKED=1
- **Dependencies (cited in text):** `plans/PHASE-11-EVIDENCE.md`
- **Quality:** correctness 3/5, maintainability 4/5, modularity 3/5, error_handling 3/5, security 5/5, docs 4/5 — self-declared FAIL/BLOCKED present.
- **Evidence:** manifest sha256 MATCH; 113 lines read; dates: 2026-11-06; status words: BLOCKED.

## docs/contracts/UI-COMMANDS-v1.md

- **Metadata:** 6,887 bytes · sha256 `B7A50E8EAAF13F8EF8802B7AFD79DBF723CF46515F2CA8B97898E630123F757C` · Markdown · documentation · seat figcraft-b · depth standard
- **Manifest:** bytes MATCH · sha256 MATCH · category `documentation`/`report/plan/evidence`
- **Purpose (from content):** # UI-COMMANDS-v1 — FROZEN CONTRACT (Phase 1 freeze) Freeze date: 2026-11-06. Source: `blueprint/HIVEMIND-BLUEPRINT-v1.md` Phase 1 (MP-01/MP-02/MP-07, UI-ADD-14) + `blueprint/UI-WIRING-GUIDE.md` S01-S28 / §2 wiring ladder. This document is the single vocabulary for backend, frontend and
- **Mechanics / structure:** 79 lines; headings: 1. Envelope; 2. Commands (client → server) — frozen; 3. Events (server → client) — frozen; 4. Single agent-state enum (>=13; superset allowed); 5. Refusal codes (Phase 1 table; UI must display code + how_to_fix); 6. Commands ↔ surface matrix (FigCraft assets must label these); 7. Acceptance (Phase 1 gate)
- **Dependencies (cited in text):** none detected in assigned set
- **Quality:** correctness 4/5, maintainability 4/5, modularity 3/5, error_handling 3/5, security 5/5, docs 4/5 — no failure markers.
- **Evidence:** manifest sha256 MATCH; 79 lines read; dates: 2026-11-06; status words: none.

## docs/plans/D2-D3-WIRING-EVIDENCE.md

- **Metadata:** 7,031 bytes · sha256 `77F6D5CD0BFD350534F430C9E37CC10735E50CEB6B760BB71F19C1BDB1D1BCB8` · Markdown · documentation · seat figcraft-b · depth standard
- **Manifest:** bytes MATCH · sha256 MATCH · category `documentation`/`report/plan/evidence`
- **Purpose (from content):** # D-2 / D-3 WIRING EVIDENCE — tools commands over WS + live `/diagnose` Date: 2026-11-06 (America/Chicago). Executor: B-Core. Verifier defects: **D-2** (core WS dispatch returned `E_INTERNAL_ERROR` for `SETTINGS_STATUS`/`FILE_TREE_LIST`/`FILE_READ`/`APPROVAL_RESOLVE`) and **D-3** (`/diagnose` not
- **Mechanics / structure:** 106 lines; headings: 1. What changed; 2. Tests (observed); 3. Live release probe; 4. Done / Not done / Unverified
- **Dependencies (cited in text):** `contracts/UI-COMMANDS-v1.md`
- **Quality:** correctness 4/5, maintainability 4/5, modularity 3/5, error_handling 3/5, security 5/5, docs 4/5 — no failure markers.
- **Evidence:** manifest sha256 MATCH; 106 lines read; dates: 2026-11-06; status words: none.

## docs/plans/FRONTEND-STATUS.md

- **Metadata:** 10,244 bytes · sha256 `475A57BFDB9C3867F9FFC9AF5BC9C69D840A2CA59DFF894DA29CACEA5EF8BACD` · Markdown · documentation · seat figcraft-b · depth standard
- **Manifest:** bytes MATCH · sha256 MATCH · category `documentation`/`report/plan/evidence`
- **Purpose (from content):** # FRONTEND STATUS — HiveMind v1 web (owned `web/**`) Updated: 2026-11-06 04:27 CST · Owner: FRONTEND seat · Repo: `C:\Users\Riley\Desktop\hiverouter-v1` ## What exists now (milestone 1 — foundation)
- **Mechanics / structure:** 110 lines; headings: What exists now (milestone 1 — foundation); Commands run (quoted output); Provenance / constraint compliance so far; Next step (in progress, same run); Blockers / open items (not hidden); Milestone 2 (DONE — all green) — full 28-surface UI + contract patch + asset gate; Live-e2e fix set (2026-11-06) — all green · markers: PASS=1, FAIL=1, UNVERIFIED=5, PASS-WITH-NOTES=1
- **Dependencies (cited in text):** `plans/PHASE-7-8-10-EVIDENCE.md`, `plans/PHASE-7-8-10-RECON.md`
- **Quality:** correctness 3/5, maintainability 4/5, modularity 3/5, error_handling 3/5, security 5/5, docs 4/5 — self-declared FAIL/BLOCKED present; 5 UNVERIFIED mentions.
- **Evidence:** manifest sha256 MATCH; 110 lines read; dates: 2026-11-06; status words: UNVERIFIED, UNVERIFIED, UNVERIFIED, UNVERIFIED, PASS-WITH-NOTES, FAIL, UNVERIFIED.

## docs/plans/HASH-VERIFICATION.md

- **Metadata:** 2,978 bytes · sha256 `1CAECEA3CF33F0AD0BC7A392F69BF968E939E8B531FFE06F73C7C966AA528E7A` · Markdown · documentation · seat figcraft-b · depth standard
- **Manifest:** bytes MATCH · sha256 MATCH · category `documentation`/`report/plan/evidence`
- **Purpose (from content):** # HASH VERIFICATION — HiveMind v1 execution Date: 2026-11-06 (America/Chicago) Executed by: implementation Lead (W0)
- **Mechanics / structure:** 42 lines; headings: Environment probe (evidence); Toolchain remediation started (W0)
- **Dependencies (cited in text):** none detected in assigned set
- **Quality:** correctness 4/5, maintainability 4/5, modularity 3/5, error_handling 3/5, security 5/5, docs 4/5 — no failure markers.
- **Evidence:** manifest sha256 MATCH; 42 lines read; dates: 2026-11-06; status words: none.

## docs/plans/PHASE-0-RECON.md

- **Metadata:** 1,388 bytes · sha256 `D52E9EF05217D661766CD5AE0DAC4695C5A838536491B7E43E9CE404821107A3` · Markdown · documentation · seat figcraft-b · depth standard
- **Manifest:** bytes MATCH · sha256 MATCH · category `documentation`/`report/plan/evidence`
- **Purpose (from content):** # PHASE 0 RECON — new project, empty workspace - **Target behavior:** create HiveMind v1 from scratch (no existing product code). Entry point is the build root `C:\Users\Riley\Desktop\hiverouter-v1`; reference-only blueprint at `C:\Users\Riley\Desktop\hivemind-blueprint`.
- **Mechanics / structure:** 15 lines; headings: (none)
- **Dependencies (cited in text):** none detected in assigned set
- **Quality:** correctness 4/5, maintainability 4/5, modularity 3/5, error_handling 3/5, security 5/5, docs 4/5 — no failure markers.
- **Evidence:** manifest sha256 MATCH; 15 lines read; dates: none; status words: none.

## docs/plans/PHASE-1-EVIDENCE.md

- **Metadata:** 12,164 bytes · sha256 `AF6C28D6543CE2B259CF29E1295FACEE9C1F7929AF69DCE7C3D001929F93FF43` · Markdown · documentation · seat figcraft-b · depth standard
- **Manifest:** bytes MATCH · sha256 MATCH · category `documentation`/`report/plan/evidence`
- **Purpose (from content):** # PHASE 1 EVIDENCE — Contract, Toolchain & Boot Date: 2026-11-06 (America/Chicago). Executor: B-Core. Recon: `PHASE-1-RECON.md`. Every command below was run from `C:\Users\Riley\Desktop\hiverouter-v1\backend` with
- **Mechanics / structure:** 226 lines; headings: 1. Preflight (blueprint Phase 1 card); 2. Toolchain security audit (environment incident); 2a. Packaging defect and documented workaround (host toolchain, not product code); 3. Manifest corrections (FACT-REGISTER F2 API-diff checkpoint); 4. Contract acceptance (blueprint: `cargo test contract_schema_roundtrip`); 5. Core acceptance; 6. Boot + `/health` live smoke; 7. Error paths exercised · markers: UNVERIFIED=1
- **Dependencies (cited in text):** `plans/PHASE-1-RECON.md`
- **Quality:** correctness 4/5, maintainability 4/5, modularity 3/5, error_handling 3/5, security 5/5, docs 4/5 — 1 UNVERIFIED mentions.
- **Evidence:** manifest sha256 MATCH; 226 lines read; dates: 2026-08-27, 2026-10-13, 2026-11-06; status words: UNVERIFIED.

## docs/plans/PHASE-1-RECON.md

- **Metadata:** 4,990 bytes · sha256 `9745DB35A2055C239E617151BFBCCE20CF188935DDDFC1A02D155A39C3F10C82` · Markdown · documentation · seat figcraft-b · depth standard
- **Manifest:** bytes MATCH · sha256 MATCH · category `documentation`/`report/plan/evidence`
- **Purpose (from content):** # PHASE 1 RECON — Contract, Toolchain & Boot ForgeWright recon ledger (skill: `forgewright-recon`). Written BEFORE any Phase 1 edit. ## Recon
- **Mechanics / structure:** 55 lines; headings: Recon · markers: UNVERIFIED=2
- **Dependencies (cited in text):** `contracts/UI-COMMANDS-v1.md`, `plans/PHASE-1-EVIDENCE.md`
- **Quality:** correctness 4/5, maintainability 4/5, modularity 3/5, error_handling 3/5, security 5/5, docs 4/5 — 2 UNVERIFIED mentions.
- **Evidence:** manifest sha256 MATCH; 55 lines read; dates: 2026-11-06; status words: UNVERIFIED, UNVERIFIED.

## docs/plans/PHASE-11-EVIDENCE.md

- **Metadata:** 6,616 bytes · sha256 `2CE0F35EF15ABABD4A1C3332B414148F5CAD564419049FA9B145EB1F09F408C4` · Markdown · documentation · seat figcraft-b · depth standard
- **Manifest:** bytes MATCH · sha256 MATCH · category `documentation`/`report/plan/evidence`
- **Purpose (from content):** # PHASE 11 EVIDENCE — Diagnosis, Observability & Verification Owner: B-Provider. Date: 2026-11-06 (CST). Blueprint card: Phase 11 (WPs 11.1–11.15). Local-only: no OTel/Prometheus/collector, no cloud CI, no test double in a product path.
- **Mechanics / structure:** 93 lines; headings: Preflight (blueprint: `cargo test --no-run && npm run build`); Tests — command + observed output; What each exit-gate item is proven by; `/diagnose` payload (derived, secret-free); Local benchmark instrument (criterion 0.8.2, dev-only); Files; Not done / UNVERIFIED (honest list) · markers: UNVERIFIED=4
- **Dependencies (cited in text):** none detected in assigned set
- **Quality:** correctness 4/5, maintainability 4/5, modularity 3/5, error_handling 3/5, security 5/5, docs 4/5 — 4 UNVERIFIED mentions.
- **Evidence:** manifest sha256 MATCH; 93 lines read; dates: 2026-11-06; status words: UNVERIFIED, UNVERIFIED, UNVERIFIED, UNVERIFIED.

## docs/plans/PHASE-11-RECON.md

- **Metadata:** 3,989 bytes · sha256 `52113636EC3839E56C7441E033CF2A940DA32C66299DAE494308319F80AE743D` · Markdown · documentation · seat figcraft-b · depth standard
- **Manifest:** bytes MATCH · sha256 MATCH · category `documentation`/`report/plan/evidence`
- **Purpose (from content):** # PHASE 11 RECON — Diagnosis, Observability & Verification Written before Phase 11 implementation edits (2026-11-06 03:41 CST). Blueprint card: Phase 11 (WPs 11.1–11.15). ## Recon
- **Mechanics / structure:** 47 lines; headings: Recon; Scope check against the blueprint card · markers: UNVERIFIED=4
- **Dependencies (cited in text):** `plans/PHASE-11-EVIDENCE.md`
- **Quality:** correctness 4/5, maintainability 4/5, modularity 3/5, error_handling 3/5, security 5/5, docs 4/5 — 4 UNVERIFIED mentions.
- **Evidence:** manifest sha256 MATCH; 47 lines read; dates: 2026-11-06; status words: UNVERIFIED, UNVERIFIED, UNVERIFIED, UNVERIFIED.

## docs/plans/PHASE-12-EVIDENCE.md

- **Metadata:** 9,448 bytes · sha256 `74CA606808C286DF94D638A6AC15F937602332AEC275FAB45F8406BEA51D169F` · Markdown · documentation · seat figcraft-b · depth standard
- **Manifest:** bytes MATCH · sha256 MATCH · category `documentation`/`report/plan/evidence`
- **Purpose (from content):** # PHASE 12 EVIDENCE — Packaging, Release & Runbook Owner: B-Provider. Date: 2026-11-06 (CST). Blueprint card: Phase 12 (WPs 12.1–12.4). Truthful packaging statement: **v1 = localhost server + browser UI.** No installer, no Tauri/WebView2 desktop
- **Mechanics / structure:** 143 lines; headings: Preflight (blueprint: `cargo build --release --locked`); re-run after the final source edits (same command, incremental):; CLI smoke (observed; full transcript in `backend/cli-smoke.log`); Runbook + first-run checks; Files; Not done / UNVERIFIED (honest list); Final shipped artifact (frozen 2026-11-06) · markers: UNVERIFIED=5
- **Dependencies (cited in text):** `contracts/UI-COMMANDS-v1.md`
- **Quality:** correctness 4/5, maintainability 4/5, modularity 3/5, error_handling 3/5, security 5/5, docs 4/5 — 5 UNVERIFIED mentions.
- **Evidence:** manifest sha256 MATCH; 143 lines read; dates: 2026-10-13, 2026-11-06; status words: UNVERIFIED, UNVERIFIED, UNVERIFIED, UNVERIFIED, UNVERIFIED.

## docs/plans/PHASE-12-RECON.md

- **Metadata:** 3,738 bytes · sha256 `83DE50FDF69C33EC721C84D3C82F60F879CB9F1238DE0D0CCBA4F16B4DB56005` · Markdown · documentation · seat figcraft-b · depth standard
- **Manifest:** bytes MATCH · sha256 MATCH · category `documentation`/`report/plan/evidence`
- **Purpose (from content):** # PHASE 12 RECON — Packaging, Release & Runbook Written before Phase 12 implementation edits (2026-11-06 03:41 CST). Blueprint card: Phase 12 (WPs 12.1–12.4). ## Recon
- **Mechanics / structure:** 44 lines; headings: Recon; Scope check against the blueprint card · markers: UNVERIFIED=1
- **Dependencies (cited in text):** none detected in assigned set
- **Quality:** correctness 4/5, maintainability 4/5, modularity 3/5, error_handling 3/5, security 5/5, docs 4/5 — 1 UNVERIFIED mentions.
- **Evidence:** manifest sha256 MATCH; 44 lines read; dates: 2026-11-06; status words: UNVERIFIED.

## docs/plans/PHASE-2-EVIDENCE.md

- **Metadata:** 8,178 bytes · sha256 `5AB509605FA86CDF626F61811E410296829E1AD371104FE2125770323549D2DB` · Markdown · documentation · seat figcraft-b · depth standard
- **Manifest:** bytes MATCH · sha256 MATCH · category `documentation`/`report/plan/evidence`
- **Purpose (from content):** # PHASE 2 EVIDENCE — Transport & Graph Authority Date: 2026-11-06 (America/Chicago). Executor: B-Core. Recon: `PHASE-2-RECON.md`. Commands run from `C:\Users\Riley\Desktop\hiverouter-v1\backend` with the Phase 1 PATH and env described in
- **Mechanics / structure:** 132 lines; headings: 1. Preflight; 2. Acceptance tests (blueprint Phase 2 card); 3. Exit-gate mapping (blueprint Phase 2); 4. Implementation facts (read back from the code); 5. Deviations recorded (not silently resolved); 6. Done / Not done / Unverified; Addendum (after Phase 4 engine wiring + disk-wipe rebuild)
- **Dependencies (cited in text):** `contracts/UI-COMMANDS-v1.md`, `plans/PHASE-1-EVIDENCE.md`, `plans/PHASE-2-RECON.md`
- **Quality:** correctness 4/5, maintainability 4/5, modularity 3/5, error_handling 3/5, security 5/5, docs 4/5 — no failure markers.
- **Evidence:** manifest sha256 MATCH; 132 lines read; dates: 2026-11-06; status words: none.

## docs/plans/PHASE-2-RECON.md

- **Metadata:** 5,473 bytes · sha256 `F0FFC149B0BD0D036080AAC211682159DD4E2EBA303FF16E40BF95590B8C5B7E` · Markdown · documentation · seat figcraft-b · depth standard
- **Manifest:** bytes MATCH · sha256 MATCH · category `documentation`/`report/plan/evidence`
- **Purpose (from content):** # PHASE 2 RECON — Transport & Graph Authority ForgeWright recon ledger (skill: `forgewright-recon`). Written BEFORE any Phase 2 edit. ## Recon
- **Mechanics / structure:** 59 lines; headings: Recon · markers: UNVERIFIED=1
- **Dependencies (cited in text):** `contracts/UI-COMMANDS-v1.md`
- **Quality:** correctness 4/5, maintainability 4/5, modularity 3/5, error_handling 3/5, security 5/5, docs 4/5 — 1 UNVERIFIED mentions.
- **Evidence:** manifest sha256 MATCH; 59 lines read; dates: none; status words: UNVERIFIED.

## docs/plans/PHASE-3-EVIDENCE.md

- **Metadata:** 5,624 bytes · sha256 `E7E398223C8CE99DBE09AEBCF6F516881568151EBE70372F8D2F4288D7282648` · Markdown · documentation · seat figcraft-b · depth standard
- **Manifest:** bytes MATCH · sha256 MATCH · category `documentation`/`report/plan/evidence`
- **Purpose (from content):** # PHASE 3 EVIDENCE — Agent Context & Orchestration Handshake Date: 2026-11-06 (America/Chicago). Executor: B-Core. Recon: `PHASE-3-RECON.md`. Commands from `C:\Users\Riley\Desktop\hiverouter-v1\backend`, PATH = cargo + `tools\w64devkit\bin`,
- **Mechanics / structure:** 85 lines; headings: 1. Preflight; 2. Acceptance tests (blueprint Phase 3 card); 3. Implementation facts (read back from the code); 4. Deviations recorded (not silently resolved); 5. Done / Not done / Unverified
- **Dependencies (cited in text):** `plans/PHASE-3-RECON.md`
- **Quality:** correctness 4/5, maintainability 4/5, modularity 3/5, error_handling 3/5, security 5/5, docs 4/5 — no failure markers.
- **Evidence:** manifest sha256 MATCH; 85 lines read; dates: 2026-11-06; status words: none.

## docs/plans/PHASE-3-RECON.md

- **Metadata:** 4,563 bytes · sha256 `F21666ECF1CEA59F20E146F17A6010FC6AE6818FB3689B1514F0F6E5A6986413` · Markdown · documentation · seat figcraft-b · depth standard
- **Manifest:** bytes MATCH · sha256 MATCH · category `documentation`/`report/plan/evidence`
- **Purpose (from content):** # PHASE 3 RECON — Agent Context & Orchestration Handshake ForgeWright recon ledger (skill: `forgewright-recon`). Written BEFORE any Phase 3 edit. ## Recon
- **Mechanics / structure:** 51 lines; headings: Recon
- **Dependencies (cited in text):** none detected in assigned set
- **Quality:** correctness 4/5, maintainability 4/5, modularity 3/5, error_handling 3/5, security 5/5, docs 4/5 — no failure markers.
- **Evidence:** manifest sha256 MATCH; 51 lines read; dates: none; status words: none.

## docs/plans/PHASE-4-EVIDENCE.md

- **Metadata:** 6,243 bytes · sha256 `274EFBB4FB512A585F2796ED815FFE9CC16166171B77F8506106B7DC5BC24FE0` · Markdown · documentation · seat figcraft-b · depth standard
- **Manifest:** bytes MATCH · sha256 MATCH · category `documentation`/`report/plan/evidence`
- **Purpose (from content):** # PHASE 4 EVIDENCE — Scheduling, Delegation & Run Lifecycle Date: 2026-11-06 (America/Chicago). Executor: B-Core. Recon: `PHASE-4-RECON.md`. Commands from `backend/` with the Phase 1 PATH/env (see PHASE-1-EVIDENCE preamble).
- **Mechanics / structure:** 87 lines; headings: 1. Preflight + full suite; 2. Acceptance tests (blueprint Phase 4 card); 3. Implementation facts (read back from the code); 4. Deviations recorded (not silently resolved); 5. Done / Not done / Unverified
- **Dependencies (cited in text):** `plans/PHASE-4-RECON.md`
- **Quality:** correctness 4/5, maintainability 4/5, modularity 3/5, error_handling 3/5, security 5/5, docs 4/5 — no failure markers.
- **Evidence:** manifest sha256 MATCH; 87 lines read; dates: 2026-11-06; status words: none.

## docs/plans/PHASE-4-RECON.md

- **Metadata:** 4,161 bytes · sha256 `24F408B03AC901D769C5770D1D700F514C4063B164E679303DF1FDEB7DEADEA4` · Markdown · documentation · seat figcraft-b · depth standard
- **Manifest:** bytes MATCH · sha256 MATCH · category `documentation`/`report/plan/evidence`
- **Purpose (from content):** # PHASE 4 RECON — Scheduling, Delegation & Run Lifecycle ForgeWright recon ledger (skill: `forgewright-recon`). Written BEFORE any Phase 4 edit. ## Recon
- **Mechanics / structure:** 47 lines; headings: Recon
- **Dependencies (cited in text):** none detected in assigned set
- **Quality:** correctness 4/5, maintainability 4/5, modularity 3/5, error_handling 3/5, security 5/5, docs 4/5 — no failure markers.
- **Evidence:** manifest sha256 MATCH; 47 lines read; dates: none; status words: none.

## docs/plans/PHASE-5-EVIDENCE.md

- **Metadata:** 6,521 bytes · sha256 `4A176CD2BEFF7774CA37F02B0AA5E5B04FA5CA61734570F0D7D9D64C30A7C390` · Markdown · documentation · seat figcraft-b · depth standard
- **Manifest:** bytes MATCH · sha256 MATCH · category `documentation`/`report/plan/evidence`
- **Purpose (from content):** # PHASE 5 EVIDENCE — Provider Client, Budget & Cost Accounting Owner: B-Provider. Date: 2026-11-06 (CST). Blueprint card: Phase 5 (WPs 5.1–5.8). No live DeepSeek call was made and no `DEEPSEEK_API_KEY` was read: every provider test uses local
- **Mechanics / structure:** 106 lines; headings: Environment (observed); Preflight (blueprint: `cargo check && cargo test --no-run`); Tests — command + observed output; Exit-gate mapping (blueprint Phase 5); Files; Deliberate decisions recorded; Not done / UNVERIFIED (honest list) · markers: UNVERIFIED=2
- **Dependencies (cited in text):** `plans/PHASE-5-RECON.md`
- **Quality:** correctness 4/5, maintainability 4/5, modularity 3/5, error_handling 3/5, security 5/5, docs 4/5 — 2 UNVERIFIED mentions.
- **Evidence:** manifest sha256 MATCH; 106 lines read; dates: 2026-08-27, 2026-10-13, 2026-11-06; status words: UNVERIFIED, UNVERIFIED.

## docs/plans/PHASE-5-RECON.md

- **Metadata:** 4,101 bytes · sha256 `C930023A320BDE7F4191368BED8F095FD159DA3CD2C70888D28D576C66D1BB25` · Markdown · documentation · seat figcraft-b · depth standard
- **Manifest:** bytes MATCH · sha256 MATCH · category `documentation`/`report/plan/evidence`
- **Purpose (from content):** # PHASE 5 RECON — Provider Client, Budget & Cost Accounting Written before Phase 5 implementation edits (2026-11-06 03:41 CST). Blueprint card: Phase 5 (WPs 5.1–5.8). ## Recon
- **Mechanics / structure:** 49 lines; headings: Recon; Scope check against the blueprint card · markers: UNVERIFIED=1
- **Dependencies (cited in text):** `contracts/ADAPTERS.md`
- **Quality:** correctness 4/5, maintainability 4/5, modularity 3/5, error_handling 3/5, security 5/5, docs 4/5 — 1 UNVERIFIED mentions.
- **Evidence:** manifest sha256 MATCH; 49 lines read; dates: 2026-11-06; status words: UNVERIFIED.

## docs/plans/PHASE-6-EVIDENCE.md

- **Metadata:** 10,265 bytes · sha256 `DE6EEFB3E8040B061E5CEBCF8574E06B12AE6D43F6207ECB9E4BE5AF94AFDDB3` · Markdown · documentation · seat figcraft-b · depth standard
- **Manifest:** bytes MATCH · sha256 MATCH · category `documentation`/`report/plan/evidence`
- **Purpose (from content):** # PHASE 6 EVIDENCE — Tools, Approvals & Execution Safety Owner: B-Provider. Date: 2026-11-06 (CST). Blueprint card: Phase 6 (WPs 6.1–6.14). All tests offline; no live provider/network call; no Docker/Python/seccomp (D1 fence).
- **Mechanics / structure:** 164 lines; headings: Environment / preflight (observed); Tests — command + observed output; Exit-gate mapping (blueprint Phase 6); Files; Final-gate defect found and fixed (2026-11-06 08:4x): timeout test PATH shadowing; Not done / UNVERIFIED (honest list) · markers: FAIL=1, UNVERIFIED=2
- **Dependencies (cited in text):** `contracts/ADAPTERS.md`
- **Quality:** correctness 3/5, maintainability 4/5, modularity 3/5, error_handling 3/5, security 5/5, docs 4/5 — self-declared FAIL/BLOCKED present; 2 UNVERIFIED mentions.
- **Evidence:** manifest sha256 MATCH; 164 lines read; dates: 2026-08-27, 2026-10-13, 2026-11-06; status words: FAIL, UNVERIFIED, UNVERIFIED.

## docs/plans/PHASE-6-RECON.md

- **Metadata:** 4,254 bytes · sha256 `E43CE35A863126385FB12BB82A0282AA8D776F24FCE0BD6D1DE57DDB63790365` · Markdown · documentation · seat figcraft-b · depth standard
- **Manifest:** bytes MATCH · sha256 MATCH · category `documentation`/`report/plan/evidence`
- **Purpose (from content):** # PHASE 6 RECON — Tools, Approvals & Execution Safety Written before Phase 6 implementation edits (2026-11-06 03:41 CST). Blueprint card: Phase 6 (WPs 6.1–6.14). ## Recon
- **Mechanics / structure:** 48 lines; headings: Recon; Scope check against the blueprint card · markers: BLOCKED=2
- **Dependencies (cited in text):** `contracts/ADAPTERS.md`
- **Quality:** correctness 3/5, maintainability 4/5, modularity 3/5, error_handling 3/5, security 5/5, docs 4/5 — self-declared FAIL/BLOCKED present.
- **Evidence:** manifest sha256 MATCH; 48 lines read; dates: 2026-11-06; status words: BLOCKED, BLOCKED.

## docs/plans/PHASE-7-8-10-EVIDENCE.md

- **Metadata:** 11,738 bytes · sha256 `8714D474F62BFE79916D21F63B775FCF5D6AA59F4238395EC485E4CE561E490A` · Markdown · documentation · seat figcraft-b · depth standard
- **Manifest:** bytes MATCH · sha256 MATCH · category `documentation`/`report/plan/evidence`
- **Purpose (from content):** # PHASE 7 / 8 / 10 — FRONTEND EVIDENCE Date: 2026-11-06 · Owner: FRONTEND seat · Scope: `web/**` + this evidence doc + claim register update. Blueprint: Phase 7/8/10 + §5 (sha `501BA198…C5B1`), UI-WIRING-GUIDE (S01–S28, R-U1..R-U15, G-UI),
- **Mechanics / structure:** 157 lines; headings: 1. Commands run and observed output (quoted); 2. 28-surface implementation matrix (S01–S28); 3. Contract wiring (frozen `hiverouter.v1` + 2026-11-06 patch); 4. G-UI gate status; 5. FigCraft asset gate (wired after watchdog PASS-WITH-NOTES); 6. Known gaps / UNVERIFIED (not hidden); 7. Live-e2e fix set (post-audit, 2026-11-06) · markers: PASS=13, FAIL=2, UNVERIFIED=10, PASS-WITH-NOTES=3, DECISION-GATED=1
- **Dependencies (cited in text):** `contracts/UI-COMMANDS-v1.md`
- **Quality:** correctness 3/5, maintainability 4/5, modularity 3/5, error_handling 3/5, security 5/5, docs 4/5 — self-declared FAIL/BLOCKED present; 10 UNVERIFIED mentions.
- **Evidence:** manifest sha256 MATCH; 157 lines read; dates: 2026-11-06; status words: PASS-WITH-NOTES, FAIL, UNVERIFIED, PASS, PASS, UNVERIFIED, PASS, PASS.

## docs/plans/PHASE-7-8-10-RECON.md

- **Metadata:** 4,784 bytes · sha256 `A707E9314D4C867DD3382A40150657B90DCC34832BE5A8A900144CE801E8BCC3` · Markdown · documentation · seat figcraft-b · depth standard
- **Manifest:** bytes MATCH · sha256 MATCH · category `documentation`/`report/plan/evidence`
- **Purpose (from content):** # PHASE 7 / 8 / 10 — FRONTEND RECON LEDGER Date: 2026-11-06 · Owner: FRONTEND seat · Scope: `web/**` only (no backend/blueprint/ledger edits) ## Recon (forgewright-recon format)
- **Mechanics / structure:** 54 lines; headings: Recon (forgewright-recon format); Scope check against the blueprint cards · markers: UNVERIFIED=2
- **Dependencies (cited in text):** `contracts/UI-COMMANDS-v1.md`
- **Quality:** correctness 4/5, maintainability 4/5, modularity 3/5, error_handling 3/5, security 5/5, docs 4/5 — 2 UNVERIFIED mentions.
- **Evidence:** manifest sha256 MATCH; 54 lines read; dates: 2026-11-06; status words: UNVERIFIED, UNVERIFIED.

## docs/plans/PHASE-9-EVIDENCE.md

- **Metadata:** 5,612 bytes · sha256 `699E1DE3DDE3F669A10350EABF9C80596BD5F77611433ADDBD82AC670C5DB076` · Markdown · documentation · seat figcraft-b · depth standard
- **Manifest:** bytes MATCH · sha256 MATCH · category `documentation`/`report/plan/evidence`
- **Purpose (from content):** # PHASE 9 EVIDENCE — State, Persistence & Memory Date: 2026-11-06 (America/Chicago). Executor: B-Core. Recon: `PHASE-9-RECON.md`. Commands from `backend/` with the Phase 1 PATH/env (see PHASE-1-EVIDENCE preamble).
- **Mechanics / structure:** 78 lines; headings: 1. Preflight + full suite; 2. Acceptance tests (blueprint Phase 9 card); 3. Implementation facts (read back from the code); 4. Deviations recorded (not silently resolved); 5. Done / Not done / Unverified
- **Dependencies (cited in text):** `plans/PHASE-9-RECON.md`
- **Quality:** correctness 4/5, maintainability 4/5, modularity 3/5, error_handling 3/5, security 5/5, docs 4/5 — no failure markers.
- **Evidence:** manifest sha256 MATCH; 78 lines read; dates: 2026-11-06; status words: none.

## docs/plans/PHASE-9-RECON.md

- **Metadata:** 3,912 bytes · sha256 `920896015BD7C6019AC1ED4DD29ABF7F0B24354ECC169F1E2497610FBFEC2D1F` · Markdown · documentation · seat figcraft-b · depth standard
- **Manifest:** bytes MATCH · sha256 MATCH · category `documentation`/`report/plan/evidence`
- **Purpose (from content):** # PHASE 9 RECON — State, Persistence & Memory ForgeWright recon ledger (skill: `forgewright-recon`). Written BEFORE any Phase 9 edit. ## Recon
- **Mechanics / structure:** 45 lines; headings: Recon
- **Dependencies (cited in text):** none detected in assigned set
- **Quality:** correctness 4/5, maintainability 4/5, modularity 3/5, error_handling 3/5, security 5/5, docs 4/5 — no failure markers.
- **Evidence:** manifest sha256 MATCH; 45 lines read; dates: none; status words: none.

## docs/plans/SESSION-PLAN.md

- **Metadata:** 2,829 bytes · sha256 `839B691CE96AE57FA60742ACF8E638FF956A6EAC01FBF402BC1D55AA69F518C8` · Markdown · documentation · seat figcraft-b · depth standard
- **Manifest:** bytes MATCH · sha256 MATCH · category `documentation`/`report/plan/evidence`
- **Purpose (from content):** # SESSION PLAN — HiveMind v1 build (five-field plan) 1. **Task statement.** Build the entire verified HiveMind v1 blueprint from scratch: one Rust binary (axum/tokio/ reqwest/serde/rusqlite-bundled/tracing/rust-embed) serving a React 19 + TS + Vite 8 + Tailwind 4 + @xyflow/react 12.12.1
- **Mechanics / structure:** 33 lines; headings: Team / budgets; Phase order
- **Dependencies (cited in text):** `contracts/UI-COMMANDS-v1.md`
- **Quality:** correctness 4/5, maintainability 4/5, modularity 3/5, error_handling 3/5, security 5/5, docs 4/5 — no failure markers.
- **Evidence:** manifest sha256 MATCH; 33 lines read; dates: 2026-11-06; status words: none.

## docs/plans/TOOLCHAIN.md

- **Metadata:** 2,022 bytes · sha256 `86F2B9C21EB8C318CA5BAEA255FF02FEC6B2D9FA2D6A02A9857AE08214C74765` · Markdown · documentation · seat figcraft-b · depth standard
- **Manifest:** bytes MATCH · sha256 MATCH · category `documentation`/`report/plan/evidence`
- **Purpose (from content):** # TOOLCHAIN — verified evidence (2026-11-06) ## Rust | Item | Value | Evidence |
- **Mechanics / structure:** 33 lines; headings: Rust; C compiler / linker; Provenance / security note
- **Dependencies (cited in text):** `audit/evidence/toolchain/hello.rs`
- **Quality:** correctness 4/5, maintainability 4/5, modularity 3/5, error_handling 3/5, security 5/5, docs 4/5 — no failure markers.
- **Evidence:** manifest sha256 MATCH; 33 lines read; dates: 2026-10-13, 2026-11-06; status words: none.

## docs/reports/FINAL-REPORT.md

- **Metadata:** 6,121 bytes · sha256 `978271BE811ACDFD97FD55F5A9F3E694E18FA78CCC9DC8F4F893073B0148514D` · Markdown · documentation · seat figcraft-b · depth standard
- **Manifest:** bytes MATCH · sha256 MATCH · category `documentation`/`report/plan/evidence`
- **Purpose (from content):** ﻿# HiveMind v1 — FINAL REPORT (shipped artifact) Date: 2026-11-06 (America/Chicago). Workspace: `C:\Users\Riley\Desktop\hiverouter-v1`. Blueprint source of truth: `C:\Users\Riley\Desktop\hivemind-blueprint` (read-only; hashes verified 9/9 at start).
- **Mechanics / structure:** 75 lines; headings: What was built; Verification (all re-run on the final tree); Not done / deliberately not claimed; Open decisions (visible, not invented); Reproduce; UI is embedded at compile time: rebuild the Rust binary after web changes. · markers: PASS=2, FAIL=1, UNVERIFIED=3, DECISION-GATED=1
- **Dependencies (cited in text):** `audit/PHASE-B-VERIFICATION.md`, `claims/CLAIM-REGISTER.csv`, `plans/D2-D3-WIRING-EVIDENCE.md`
- **Quality:** correctness 3/5, maintainability 4/5, modularity 3/5, error_handling 3/5, security 5/5, docs 4/5 — self-declared FAIL/BLOCKED present; 3 UNVERIFIED mentions.
- **Evidence:** manifest sha256 MATCH; 75 lines read; dates: 2026-11-06; status words: PASS, FAIL, PASS, UNVERIFIED, UNVERIFIED, UNVERIFIED.
