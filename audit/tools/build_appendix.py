"""Generate Appendix A: complete 507-file census grouped by audit seat."""
from __future__ import annotations

import json
from collections import defaultdict
from pathlib import Path

ROOT = Path(r"C:\Users\Riley\Desktop\hiverouter-v1")
INV = ROOT / "audit" / "inventory"
manifest = json.loads((INV / "FILE-MANIFEST-ENRICHED.json").read_text(encoding="utf-8"))
rows = manifest["files"]
by_seat = defaultdict(list)
for r in rows:
    by_seat[r["seat"]].append(r)
SEAT_NAMES = {"A": "b-core", "B": "b-provider", "C": "watchdog-1", "D": "watchdog-2",
              "E": "figcraft-a", "F": "figcraft-b", "G": "ui-watchdog", "H": "strip-watchdog"}

lines = ["# APPENDIX A — COMPLETE FILE CENSUS (507 tracked files)", "",
         f"Repo: `C:\\Users\\Riley\\Desktop\\hiverouter-v1` · HEAD `{manifest['head']}`",
         f"Manifest: `audit/inventory/FILE-MANIFEST-ENRICHED.csv` · Count: **{len(rows)}**", "",
         "Every tracked file appears exactly once below. `sha256` is truncated to 16 hex chars for readability;",
         "the full 64-char value is in the manifest. `D` = deep-analysis file (8–12 paragraphs in seat `FILES.md`).", ""]
for code in "ABCDEFGH":
    seat = SEAT_NAMES[code]
    seat_rows = by_seat.get(code, [])
    lines += [f"## Seat {code} — {seat} ({len(seat_rows)} files)", "",
              "| Path | Bytes | sha256[:16] | Lang | Category | D |", "|---|---:|---|---|---|---|"]
    for r in sorted(seat_rows, key=lambda x: x["path"]):
        d = "D" if r.get("depth") == "deep" else ""
        lines.append(f"| `{r['path']}` | {r['bytes']:,} | `{r['sha256'][:16]}` | {r['language']} | {r['category']} | {d} |")
    lines.append("")
(ROOT / "audit" / "APPENDIX-FILE-CENSUS.md").write_text("\n".join(lines) + "\n", encoding="utf-8")
print("appendix rows", len(rows), "seats", {SEAT_NAMES[k]: len(v) for k, v in by_seat.items()})
