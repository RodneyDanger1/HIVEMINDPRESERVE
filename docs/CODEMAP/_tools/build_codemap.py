#!/usr/bin/env python3
"""Build the frozen file census for docs/CODEMAP.

Read-only with respect to product files. Writes only docs/CODEMAP/_data.
"""
from __future__ import annotations

import csv
import hashlib
import json
import os
import subprocess
import sys
from pathlib import Path

ROOT = Path(r"C:\Users\Riley\Desktop\hiverouter-v1")
OUT = ROOT / "docs" / "CODEMAP" / "_data"
OUT.mkdir(parents=True, exist_ok=True)


def tracked_paths() -> set[str]:
    raw = subprocess.check_output(
        ["git", "-C", str(ROOT), "-c", "core.quotePath=false", "ls-files", "-z"]
    )
    return {
        p.decode("utf-8", "surrogateescape").replace("/", "\\")
        for p in raw.split(b"\0")
        if p
    }


def sha256(path: Path) -> str:
    h = hashlib.sha256()
    with path.open("rb") as fh:
        for chunk in iter(lambda: fh.read(1024 * 1024), b""):
            h.update(chunk)
    return h.hexdigest()


def classify_encoding(path: Path) -> str:
    try:
        head = path.read_bytes()[:8192]
    except OSError:
        return "unreadable"
    if head.startswith(b"\xff\xfe"):
        return "utf-16le-bom"
    if head.startswith(b"\xfe\xff"):
        return "utf-16be-bom"
    if head.startswith(b"\xef\xbb\xbf"):
        return "utf-8-bom"
    if b"\x00" in head:
        return "nul-binary"
    try:
        head.decode("utf-8")
        return "utf-8"
    except UnicodeDecodeError:
        return "non-utf8"


def area_of(rel: str) -> str:
    parts = rel.split("\\")
    return parts[0] if len(parts) > 1 else "(root)"


def main() -> int:
    tracked = tracked_paths()
    rows = []
    for dirpath, dirnames, filenames in os.walk(ROOT):
        dirnames[:] = [d for d in dirnames if d != ".git"]
        rel_dir = Path(dirpath).relative_to(ROOT)
        if rel_dir == Path("docs") / "CODEMAP":
            dirnames[:] = []
            continue
        for name in filenames:
            full = Path(dirpath) / name
            rel = str(full.relative_to(ROOT))
            try:
                size = full.stat().st_size
            except OSError:
                size = -1
            ext = Path(name).suffix.lower() or "(none)"
            is_tracked = rel in tracked
            rows.append(
                {
                    "path": rel,
                    "area": area_of(rel),
                    "tracked": is_tracked,
                    "bytes": size,
                    "extension": ext,
                    "sha256": sha256(full) if is_tracked else "",
                    "encoding": classify_encoding(full) if is_tracked else "",
                }
            )
    rows.sort(key=lambda r: r["path"].lower())
    (OUT / "inventory.json").write_text(
        json.dumps(rows, ensure_ascii=False, separators=(",", ":")), encoding="utf-8"
    )
    with (OUT / "inventory.csv").open("w", newline="", encoding="utf-8") as fh:
        w = csv.DictWriter(fh, fieldnames=list(rows[0]))
        w.writeheader()
        w.writerows(rows)

    def agg(key):
        out = {}
        for r in rows:
            k = key(r)
            cur = out.setdefault(k, {"files": 0, "tracked": 0, "ignored": 0, "bytes": 0, "tracked_bytes": 0})
            cur["files"] += 1
            cur["bytes"] += max(0, r["bytes"])
            if r["tracked"]:
                cur["tracked"] += 1
                cur["tracked_bytes"] += max(0, r["bytes"])
            else:
                cur["ignored"] += 1
        return dict(sorted(out.items(), key=lambda kv: kv[1]["files"], reverse=True))

    areas = agg(lambda r: r["area"])
    exts = agg(lambda r: r["extension"])
    tracked_exts = agg(lambda r: r["extension"] if r["tracked"] else "(ignored)")
    biggest = sorted(rows, key=lambda r: r["bytes"], reverse=True)[:30]
    tracked_rows = [r for r in rows if r["tracked"]]
    ignored_rows = [r for r in rows if not r["tracked"]]
    summary = {
        "head": subprocess.check_output(["git", "-C", str(ROOT), "rev-parse", "HEAD"], text=True).strip(),
        "status_porcelain": subprocess.check_output(
            ["git", "-C", str(ROOT), "status", "--porcelain"], text=True
        ).strip(),
        "working_tree_files": len(rows),
        "tracked_files": len(tracked_rows),
        "ignored_files": len(ignored_rows),
        "working_tree_bytes": sum(max(0, r["bytes"]) for r in rows),
        "tracked_bytes": sum(max(0, r["bytes"]) for r in tracked_rows),
        "ignored_bytes": sum(max(0, r["bytes"]) for r in ignored_rows),
        "areas": areas,
        "extensions": exts,
        "biggest": biggest,
        "encoding": agg(lambda r: r["encoding"] if r["tracked"] else "(ignored)"),
        "tracked_areas": agg(lambda r: r["area"] if r["tracked"] else "(ignored)"),
        "tracked_extension_census": tracked_exts,
    }
    (OUT / "summary.json").write_text(json.dumps(summary, indent=2), encoding="utf-8")
    print(json.dumps({k: summary[k] for k in ("head", "working_tree_files", "tracked_files", "ignored_files", "working_tree_bytes", "tracked_bytes", "ignored_bytes")}, indent=2))
    return 0


if __name__ == "__main__":
    sys.exit(main())
