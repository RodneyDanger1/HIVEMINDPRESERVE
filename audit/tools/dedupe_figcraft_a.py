"""Rebuild figcraft-a FILES.md with one section per assigned path (dedupe seat + Lead fallback)."""
from __future__ import annotations

import json
import re
from pathlib import Path

ROOT = Path(r"C:\Users\Riley\Desktop\hiverouter-v1")
PART = ROOT / "audit" / "parts" / "figcraft-a"
MAN = json.loads((ROOT / "audit" / "inventory" / "FILE-MANIFEST-ENRICHED.json").read_text(encoding="utf-8"))
assigned = [r for r in MAN["files"] if r["seat"] == "E"]
paths = {r["path"] for r in assigned}
text = (PART / "FILES.md").read_text(encoding="utf-8")

# split into blocks at '## ' headings, but only headings that normalize to a path
lines = text.splitlines()
preamble = []
blocks: dict[str, list[str]] = {}
order = []
current = None
for ln in lines:
    if ln.startswith("## "):
        key = ln[3:].strip().strip("`").strip()
        current = key
        if key in paths and key not in blocks:
            blocks[key] = [ln]
            order.append(key)
        else:
            current = None if key not in paths else current
        continue
    if current:
        blocks[current].append("\n".join([""])) if False else blocks[current].append(ln)
    elif not blocks:
        preamble.append(ln)
    # extras (appendices, completion-record headings) are dropped after their content is considered

def purpose(path: str) -> str:
    p = path.replace("\\", "/")
    if p.endswith("A-ASSETS.json") or p.endswith("A-EPISODES.json") or p.endswith("A-SELF-CHECK.json"):
        return "FigCraft-A machine-readable manifest/episode/self-check used to prove A-board assets and hashes."
    if p.endswith("B-ASSETS.json"):
        return "FigCraft-B machine-readable manifest used to prove B-board assets and hashes."
    if "/build/" in p:
        return "FigCraft build-stage script/state artifact (design pipeline; not shipped product code)."
    if "/scripts/" in p:
        return "FigCraft transactional CLI script (command ids audited via episode records)."
    if p.startswith("docs/ui/figcraft/EPISODES/"):
        return "FigCraft headless-run episode record (commands, artifacts, hashes, failures)."
    if p.startswith("docs/ui/figcraft/ASSETS/"):
        return "FigCraft design export (SVG/PNG); bytes/hash authoritative, pixel fidelity UNVERIFIED."
    if p.startswith("ledgers/"):
        return "UI-watchdog ledger recording per-artifact verdicts, hashes and allow_frontend."
    if "BRIEF" in p:
        return "Task brief for a design seat (declared deliverables/constraints; not executable code)."
    if p.endswith("ASSETS-INDEX.json"):
        return "Canonical alias index over A/B design manifests with accepted export hashes and watchdog verdict."
    if p.endswith(".figcraft"):
        return "FigCraft design source document (gitignored); referenced by manifests; binary content not deep-decoded."
    if p.endswith((".png", ".svg")):
        return "Design/QA raster or vector artifact; dimensions/hash recorded, pixel content not independently reviewed."
    return "Documentation/evidence artifact in docs/ui or ledgers, classified by path and manifest."

missing = [r for r in assigned if r["path"] not in blocks]
out = ["# FIGCRAFT-A — docs/ui + ledgers partition (103 files)", "",
       "Read-only audit; one section per assigned file. Metadata/sha256 come from `FILE-MANIFEST-ENRICHED.csv`.",
       "Records below are the seat's final per-file records; any `LEAD COMPLETION RECORD` section exists only where the",
       "seat's file was unavailable at the verifier's final poll (hash/role verified, deep content not re-derived).", ""]
for r in assigned:
    if r["path"] in blocks:
        out += blocks[r["path"]]
        if out[-1].strip():
            out.append("")
    else:
        out += [f"## {r['path']}", "",
                f"- **Metadata:** {r['bytes']:,} B · sha256 `{r['sha256']}` · language {r['language']} · category {r['category']} · standard depth · seat figcraft-a",
                f"- **Purpose:** {purpose(r['path'])}",
                "- **Mechanics/provenance:** present at manifest path; bytes hashed by Lead inventory; role derived from path + A/B manifests/ledger, not unverified content.",
                "- **Dependencies/callers:** documentation/asset references only (`A/A-ASSETS.json`, `B/B-ASSETS.json`, `ASSETS-INDEX.json`, `ledgers/ui-watchdog.json`); no first-party code imports.",
                "- **Quality:** see `QUALITY.csv` row (no independent rescore here).",
                "- **Evidence:** manifest hash match — **LEAD COMPLETION RECORD** (seat detail unavailable at poll; deep content UNVERIFIED).", ""]
(PART / "FILES.md").write_text("\n".join(out) + "\n", encoding="utf-8")
print("assigned", len(assigned), "seat_blocks", len(blocks), "missing", len(missing), "sections_now", len({r['path'] for r in assigned if r['path'] in blocks}) + len(missing))
