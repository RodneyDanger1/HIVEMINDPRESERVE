"""Enrich the file manifest with LOC/size stats and produce code statistics for the paper."""
from __future__ import annotations

import csv
import json
from collections import Counter, defaultdict
from pathlib import Path

ROOT = Path(r"C:\Users\Riley\Desktop\hiverouter-v1")
INV = ROOT / "audit" / "inventory"

TEXT = {".rs", ".ts", ".tsx", ".mjs", ".js", ".py", ".ps1", ".c", ".md", ".txt", ".json",
        ".toml", ".html", ".css", ".mmd", ".prompty", ".csv", ".lock", ".gitignore"}
CODE = {".rs", ".ts", ".tsx", ".mjs", ".js", ".py", ".ps1", ".c", ".html", ".css"}

manifest = json.loads((INV / "FILE-MANIFEST.json").read_text(encoding="utf-8"))
rows = manifest["files"]
for r in rows:
    p = ROOT / r["path"]
    ext = r["ext"]
    loc = 0
    if ext in TEXT and p.exists() and r["bytes"] < 5_000_000:
        try:
            loc = p.read_text(encoding="utf-8", errors="replace").count("\n") + 1
        except OSError:
            loc = 0
    r["loc"] = loc
    r["is_code"] = ext in CODE

with (INV / "FILE-MANIFEST-ENRICHED.csv").open("w", newline="", encoding="utf-8") as f:
    w = csv.DictWriter(f, fieldnames=list(rows[0].keys()))
    w.writeheader()
    w.writerows(rows)
(INV / "FILE-MANIFEST-ENRICHED.json").write_text(json.dumps(manifest, indent=2), encoding="utf-8")

by_dir = defaultdict(lambda: {"files": 0, "loc": 0, "bytes": 0, "source_files": 0, "source_loc": 0})
by_lang = defaultdict(lambda: {"files": 0, "loc": 0})
for r in rows:
    top = r["path"].split("/")[0] if "/" in r["path"] else "(root)"
    d = by_dir[top]
    d["files"] += 1; d["loc"] += r["loc"]; d["bytes"] += r["bytes"]
    if r["is_code"]:
        d["source_files"] += 1; d["source_loc"] += r["loc"]
    l = by_lang[r["language"]]
    l["files"] += 1; l["loc"] += r["loc"]

code = [r for r in rows if r["is_code"]]
lines = ["# CODE STATISTICS", "",
         f"- Tracked files: **{len(rows)}**",
         f"- First-party code files: **{code.__len__()}**",
         f"- Total source LOC: **{sum(r['loc'] for r in code):,}**",
         f"- Total tracked LOC (all text incl. docs/data): **{sum(r['loc'] for r in rows):,}**",
         f"- Total tracked bytes: **{sum(r['bytes'] for r in rows)/1_048_576:.2f} MiB**", "",
         "## By directory", "", "| Directory | Files | Source files | Source LOC | Total LOC |", "|---|---:|---:|---:|---:|"]
for k, v in sorted(by_dir.items(), key=lambda kv: -kv[1]["source_loc"]):
    lines.append(f"| `{k}` | {v['files']} | {v['source_files']} | {v['source_loc']:,} | {v['loc']:,} |")
lines += ["", "## By language", "", "| Language | Files | LOC |", "|---|---:|---:|"]
for k, v in sorted(by_lang.items(), key=lambda kv: -kv[1]["loc"]):
    lines.append(f"| {k} | {v['files']} | {v['loc']:,} |")
lines += ["", "## Largest first-party source files (LOC)", "", "| File | LOC | Bytes |", "|---|---:|---:|"]
for r in sorted(code, key=lambda x: -x["loc"])[:25]:
    lines.append(f"| `{r['path']}` | {r['loc']:,} | {r['bytes']:,} |")
lines += ["", "## Test-related files", ""]
tests = [r["path"] for r in rows if "/tests/" in r["path"] or r["path"].startswith("e2e/") or "/tests/" in r["path"] or r["path"].endswith(".spec.ts")]
lines += [f"- `{t}`" for t in tests]
(INV / "CODE-STATS.md").write_text("\n".join(lines) + "\n", encoding="utf-8")
print("code files", len(code), "source loc", sum(r["loc"] for r in code), "total rows", len(rows))
print("tests", len(tests), "top", [r["path"] for r in sorted(code, key=lambda x: -x["loc"])[:5]])
