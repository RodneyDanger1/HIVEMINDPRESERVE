"""Compute code-only quality statistics from the merged per-file matrix."""
from __future__ import annotations

import csv
import json
from collections import defaultdict
from pathlib import Path

ROOT = Path(r"C:\Users\Riley\Desktop\hiverouter-v1")
MATRIX = ROOT / "audit" / "QUALITY-MATRIX.csv"
CODE_EXT = (".rs", ".ts", ".tsx", ".mjs", ".js", ".py", ".ps1", ".c")
DIMS = ["correctness", "maintainability", "modularity", "error_handling", "security", "docs"]

with MATRIX.open(encoding="utf-8-sig", newline="") as f:
    rows = [r for r in csv.DictReader(f) if r.get("path")]

code = [r for r in rows if r["path"].lower().endswith(CODE_EXT)]
by_seat = defaultdict(lambda: defaultdict(list))
for r in code:
    for d in DIMS:
        try:
            by_seat[r["seat"]][d].append(int(r[d]))
        except (ValueError, KeyError):
            pass

def mean(vals):
    return round(sum(vals) / len(vals), 2) if vals else None

summary = {
    "total_scored": len(rows),
    "code_scored": len(code),
    "non_code_scored": len(rows) - len(code),
    "overall_code_means": {d: mean([int(r[d]) for r in code if str(r.get(d, "")).isdigit() and int(r[d]) > 0]) for d in DIMS},
    "by_seat": {
        seat: {"files": sum(1 for r in code if r["seat"] == seat),
               "means": {d: mean(v) for d, v in dims.items()}}
        for seat, dims in sorted(by_seat.items())
    },
}
(ROOT / "audit" / "QUALITY-CODE-STATS.json").write_text(json.dumps(summary, indent=2), encoding="utf-8")

lines = ["# CODE-ONLY QUALITY STATISTICS", "",
         f"- Scored files total: **{len(rows)}** · code files scored: **{len(code)}** · non-code scored: **{len(rows)-len(code)}**", "",
         "## Overall code means (1–5)", "", "| Dimension | Mean |", "|---|---:|"]
for d in DIMS:
    lines.append(f"| {d.replace('_', ' ')} | {summary['overall_code_means'][d]} |")
lines += ["", "## By seat (code files only)", "",
          "| Seat | Files | " + " | ".join(d.replace("_", " ") for d in DIMS) + " |",
          "|---|---:|" + "---:|" * len(DIMS)]
for seat, s in summary["by_seat"].items():
    lines.append("| " + seat + " | " + str(s["files"]) + " | " + " | ".join(str(s["means"][d]) for d in DIMS) + " |")
(ROOT / "audit" / "QUALITY-CODE-STATS.md").write_text("\n".join(lines) + "\n", encoding="utf-8")
print("total", len(rows), "code", len(code), "overall", summary["overall_code_means"])
