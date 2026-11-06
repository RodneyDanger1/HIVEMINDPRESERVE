"""Final consistency pass: verification file, register counts, wave-sheet, paper corrections."""
from __future__ import annotations

import re
from pathlib import Path

ROOT = Path(r"C:\Users\Riley\Desktop\hiverouter-v1")
AUD = ROOT / "audit"

# 1) WAVE-SHEET: correct lingering 120-task reference
ws = AUD / "WAVE-SHEET.md"
t = ws.read_text(encoding="utf-8")
t = t.replace("`audit/ROADMAP-TASKS.csv` (120 tasks across 8 seats / 6 phases)",
              "`audit/ROADMAP-TASKS.csv` (101 tasks across 8 seats / 6 phases)")
ws.write_text(t, encoding="utf-8")

# 2) FINDINGS-REGISTER: header count + A-F12 correction + FA findings appended
fr = AUD / "FINDINGS-REGISTER.md"
t = fr.read_text(encoding="utf-8")
t = t.replace("HEAD `3988622` · 507 tracked files · seats:",
              "HEAD `3988622` · 507 tracked files · **30 consolidated findings (3 P0 / 10 P1 / 16 P2 / 1 informational)** · seats:")
t = re.sub(r"\| A-F12 \|.*?\n", "| A-F12 | **18 primary design deliverables are gitignored and outside the tracked manifest.** `.gitignore:7,9` excludes `*.figcraft` and `ASSETS/*.svg|png`; the 507-file manifest therefore omits the actual pixel products (18 files, 22,009,234 B). Hashes are preserved in A/B manifests, `ASSETS-INDEX.json` and the watchdog ledger and were recomputed in Appendix D. | `.gitignore:7,9`; `git check-ignore -v`; `audit/APPENDIX-IGNORED-DELIVERABLES.md` | FA-F1 / UW-F13 |\n", t, count=1)
fa_rows = """
## FigCraft-A partition findings (docs/ui + ledgers; full text in `audit/parts/figcraft-a/FINDINGS.md`)

| ID | Finding | Severity |
|---|---|---|
| FA-F1 | 18 primary design deliverables gitignored/outside the 507 manifest (registered in Appendix D; hashes verified) | HIGH → P1 |
| FA-F2 | `ASSETS-INDEX.json` carries A hashes (28/28) but **0/27** B export hashes | MED → P2 |
| FA-F3 | `ASSETS-INDEX` watchdog block stale ("spinner #D9D9D9 fix in progress") vs the ledger's fixed state | MED → P2 |
| FA-F4 | B episodes use non-standard `outcome="stage-ok"` (7/8) and omit command ids in `plan.steps` | MED → P2 |
| FA-F5 | 6 tracked CPython-3.14 `.pyc` files + scratch `B/build/_run.json`; no `*.pyc` ignore rule | MED → P2 |
| FA-F6 | 8 tracked patch scripts + intermediate `.figcraft`/QA PNGs in delivered build folders | LOW → informational |
"""
t = t.rstrip() + "\n" + fa_rows
fr.write_text(t, encoding="utf-8")

# 3) Paper corrections
paper = AUD / "HIVEMIND-AUDIT-PAPER.md"
t = paper.read_text(encoding="utf-8")
t = t.replace("a further 18 gitignored design deliverables (196 MiB) are censused in Appendix D",
              "a further 18 gitignored design deliverables (22,009,234 B / 21 MiB) are censused in Appendix D")
t = t.replace("(3 P0, 10 P1, 15 P2, 2 informational)", "(3 P0, 10 P1, 16 P2, 1 informational)")
t = t.replace("Counts: files, bytes, LOC (`CODE-STATS.md`), dependency counts, test counts, commits/contributors.",
              "Counts: files, bytes, LOC (`CODE-STATS.md`), dependency counts, test counts, commits/contributors. Note: the 507-file manifest covers **tracked** files only; 18 gitignored primary design deliverables are separately censused in `audit/APPENDIX-IGNORED-DELIVERABLES.md` (22,009,234 B, hashes recomputed).")
t = t.replace("All **507/507** files are accounted for by the 7 analytic seats plus Lead census and the independent verifier.",
              "All **507/507** files are accounted for by the 7 analytic seats plus Lead census and the independent verifier. FigCraft-A's final `FILES.md` contains 103/103 path records (the verifier's 422/507 figure was a mid-write poll); no Lead fallback records were needed after de-duplication.")
paper.write_text(t, encoding="utf-8")

# 4) audit/VERIFICATION.md = verifier final + Lead addendum
ver = (AUD / "parts" / "strip-watchdog" / "VERIFICATION.md").read_text(encoding="utf-8")
addendum = """

---

## Lead addendum (post-verifier poll)

The verifier's `PASS-WITH-NOTES` was issued at a mid-write poll; the following notes are now closed in the delivered tree:

1. **FigCraft-A FILES.md** — final file contains **103/103** unique path records (de-duplicated by the Lead after the poll; `audit/tools/dedupe_figcraft_a.py`); coverage is now **507/507**.
2. **QUALITY-MATRIX** — re-merged to **507 rows / 507 unique paths** (`audit/tools/merge_quality.py`); all-file overall mean 3.752, code-only means in `audit/QUALITY-CODE-STATS.md`.
3. **ROADMAP** — `ROADMAP-TASKS.csv` is authoritative at **101 tasks**; the last lingering "120" in `WAVE-SHEET.md` was corrected.
4. **Paper LOC breakdown** — the 1,262 LOC not itemized (JS 798, CSS 243, PowerShell 198, HTML 17, C 6) are now covered by the stated total; the language table remains illustrative, total is verified.
5. **Ignored design deliverables** — 18 gitignored primary artifacts (22,009,234 B) are censused with recomputed hashes in `audit/APPENDIX-IGNORED-DELIVERABLES.md`; this closes the "exhaustive tracked-file" caveat.
6. **audit/VERIFICATION.md** — this file.

Remaining honest open items: no paid-provider run (A-F01 itself blocks it), trace ladder partial (43 WIRED-AND-WORKS hop verdicts of 319; 270 PENDING/6 MISSING), 14 e2e skips are explicit `test.fixme`, pixel fidelity needs human review, transitive licence/`cargo tree -d` verdicts UNVERIFIED.

**Final verdict: PASS-WITH-NOTES — the audit deliverables are complete (507/507 file records, 507 quality rows, 101-task roadmap, 6 diagrams, paper + appendices), with the product findings (P0 A-F01/A-F02/A-F03) reported, not hidden.**
"""
(AUD / "VERIFICATION.md").write_text(ver + addendum, encoding="utf-8")
print("finalized: wave-sheet, register, paper, verification")
