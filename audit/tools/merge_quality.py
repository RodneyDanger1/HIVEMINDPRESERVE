"""Merge per-seat QUALITY.csv files into audit/QUALITY-MATRIX.csv and emit aggregate stats."""
from __future__ import annotations

import csv
import json
from collections import defaultdict
from pathlib import Path

ROOT = Path(r"C:\Users\Riley\Desktop\hiverouter-v1")
PARTS = ROOT / "audit" / "parts"
OUT = ROOT / "audit" / "QUALITY-MATRIX.csv"
DIMS = ["correctness", "maintainability", "modularity", "error_handling", "security", "docs"]
rows_out = []
per_seat = defaultdict(lambda: {"rows": 0, "sums": defaultdict(float), "evidence_rows": 0})
for seat_dir in sorted(p for p in PARTS.iterdir() if p.is_dir()):
    q = seat_dir / "QUALITY.csv"
    if not q.exists():
        continue
    with q.open(encoding="utf-8-sig", newline="") as f:
        reader = csv.DictReader(f)
        for r in reader:
            path = (r.get("path") or "").strip()
            if not path:
                continue
            clean = {k.strip().lower().replace(" ", "_"): (v or "").strip() for k, v in r.items()}
            row = {"path": path, "seat": seat_dir.name}
            for d in DIMS:
                try:
                    row[d] = int(float(clean.get(d, 0)))
                except ValueError:
                    row[d] = 0
            row["evidence"] = clean.get("evidence", "")
            row["notes"] = clean.get("notes", "")
            rows_out.append(row)
            ps = per_seat[seat_dir.name]
            ps["rows"] += 1
            for d in DIMS:
                ps["sums"][d] += row[d]
            if row["evidence"]:
                ps["evidence_rows"] += 1
rows_out.sort(key=lambda r: (r["seat"], r["path"]))
with OUT.open("w", newline="", encoding="utf-8") as f:
    w = csv.DictWriter(f, fieldnames=["path", "seat", *DIMS, "evidence", "notes"])
    w.writeheader()
    w.writerows(rows_out)

summary = {"seats": {}, "total_files_scored": len(rows_out), "dimension_means": {}}
for seat, v in per_seat.items():
    summary["seats"][seat] = {
        "files_scored": v["rows"],
        "means": {d: round(v["sums"][d] / v["rows"], 2) if v["rows"] else None for d in DIMS},
        "evidence_cells": v["evidence_rows"],
    }
for d in DIMS:
    vals = [r[d] for r in rows_out if r[d] > 0]
    summary["dimension_means"][d] = round(sum(vals) / len(vals), 2) if vals else None
(ROOT / "audit" / "QUALITY-SUMMARY.json").write_text(json.dumps(summary, indent=2), encoding="utf-8")
print("scored files:", len(rows_out), "seats:", list(per_seat), "means:", {d: summary["dimension_means"][d] for d in DIMS})
