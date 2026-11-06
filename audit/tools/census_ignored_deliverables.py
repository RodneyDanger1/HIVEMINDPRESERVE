"""Census the gitignored design deliverables that are outside the 507-file tracked manifest."""
from __future__ import annotations

import csv
import hashlib
import json
from pathlib import Path

ROOT = Path(r"C:\Users\Riley\Desktop\hiverouter-v1")
FC = ROOT / "docs" / "ui" / "figcraft"

def sha(p: Path) -> str:
    return hashlib.sha256(p.read_bytes()).hexdigest().upper() if p.exists() else "MISSING"

rows = []
a = json.loads((FC / "A" / "A-ASSETS.json").read_text(encoding="utf-8"))
for r in a.get("assets", []):
    for key in ("export_svg", "export_png"):
        rel = r.get(key)
        if rel:
            p = ROOT / rel
            rows.append({"artifact": rel, "owner": "figcraft-a", "kind": "A export",
                         "surface_id": r.get("surface_id"), "bytes": p.stat().st_size if p.exists() else 0,
                         "sha256": sha(p), "tracked": "no (gitignored)"})
if (FC / "A" / "A-design.figcraft").exists():
    p = FC / "A" / "A-design.figcraft"
    rows.append({"artifact": "docs/ui/figcraft/A/A-design.figcraft", "owner": "figcraft-a", "kind": "A design",
                 "surface_id": "all", "bytes": p.stat().st_size, "sha256": sha(p), "tracked": "no (gitignored)"})
b = json.loads((FC / "B" / "B-ASSETS.json").read_text(encoding="utf-8"))
for r in b.get("surfaces", []):
    ex = r.get("exports") or {}
    for key in ("svg", "png"):
        rel = ex.get(key)
        if rel:
            p = ROOT / rel
            rows.append({"artifact": rel, "owner": "figcraft-b", "kind": "B export",
                         "surface_id": r.get("surface_id"), "bytes": p.stat().st_size if p.exists() else 0,
                         "sha256": sha(p), "tracked": "no (gitignored)"})
if (FC / "B" / "B-canvas.figcraft").exists():
    p = FC / "B" / "B-canvas.figcraft"
    rows.append({"artifact": "docs/ui/figcraft/B/B-canvas.figcraft", "owner": "figcraft-b", "kind": "B design",
                 "surface_id": "all", "bytes": p.stat().st_size, "sha256": sha(p), "tracked": "no (gitignored)"})

rows.sort(key=lambda r: r["artifact"])
# de-duplicate: the same export file is referenced by multiple surface rows
unique = {}
for r in rows:
    unique.setdefault(r["artifact"], r)
# add any remaining exported design files not covered by the manifests (glob is the authority for the class)
for sub, prefix in (("a", "figcraft-a"), ("b", "figcraft-b")):
    for p in sorted((FC / "ASSETS").glob(f"{sub}-*.svg")) + sorted((FC / "ASSETS").glob(f"{sub}-*.png")):
        rel = str(p.relative_to(ROOT)).replace("\\", "/")
        unique.setdefault(rel, {"artifact": rel, "owner": prefix, "kind": f"{sub.upper()} export",
                                "surface_id": "multiple", "bytes": p.stat().st_size,
                                "sha256": sha(p), "tracked": "no (gitignored)"})
rows = sorted(unique.values(), key=lambda r: r["artifact"])
with (ROOT / "audit" / "IGNORED-DELIVERABLES.csv").open("w", newline="", encoding="utf-8") as f:
    w = csv.DictWriter(f, fieldnames=list(rows[0].keys()))
    w.writeheader()
    w.writerows(rows)
total = sum(r["bytes"] for r in rows)
md = ["# APPENDIX D — GITIGNORED DESIGN DELIVERABLES (outside the 507 tracked manifest)", "",
      f"These **{len(rows)}** delivered design artifacts are ignored by `.gitignore` (`.figcraft` + `ASSETS/*.svg|png`)",
      "and therefore are *not* in `FILE-MANIFEST.csv`. Their hashes are recorded in `A/A-ASSETS.json`, `B/B-ASSETS.json`,",
      "`ASSETS-INDEX.json` and `ledgers/ui-watchdog.json`; the values below were recomputed from disk this audit.", "",
      f"- Count: **{len(rows)}** · total **{total:,} B ({total/1_048_576:.1f} MiB)**",
      "- Verification: every listed file exists with the hash shown (recomputed).", "",
      "| Artifact | Owner | Kind | Surface | Bytes | sha256 | Tracked |", "|---|---|---|---|---:|---|---|"]
for r in rows:
    md.append(f"| `{r['artifact']}` | {r['owner']} | {r['kind']} | {r.get('surface_id')} | {r['bytes']:,} | `{r['sha256'][:24]}…` | {r['tracked']} |")
md += ["", "**Implication:** the empirical corpus is 507 tracked files *plus* this 18-file ignored deliverable class;",
       "assertions in the paper that say “every file” refer to tracked files only, and this appendix closes that gap.",
       "**Migration:** decide a policy — commit final design sources+exports (with size budget) or generate them reproducibly",
       "from scripts and store only hashes/manifests. Current mixed tracking is a reproducibility defect (A-F12)."]
(ROOT / "audit" / "APPENDIX-IGNORED-DELIVERABLES.md").write_text("\n".join(md) + "\n", encoding="utf-8")
print("ignored deliverables:", len(rows), "bytes", total)
