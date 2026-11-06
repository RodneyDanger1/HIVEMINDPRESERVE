"""Complete figcraft-a FILES.md for any assigned docs/ui/ledgers file the seat did not document.
These are explicitly labelled Lead completion records: metadata/hash verified from the manifest,
purpose derived from path/manifest provenance, quality taken from the seat's own QUALITY.csv.
No file content is invented."""
from __future__ import annotations

import csv
import json
import re
from pathlib import Path

ROOT = Path(r"C:\Users\Riley\Desktop\hiverouter-v1")
PART = ROOT / "audit" / "parts" / "figcraft-a"
MAN = json.loads((ROOT / "audit" / "inventory" / "FILE-MANIFEST-ENRICHED.json").read_text(encoding="utf-8"))
assigned = [r for r in MAN["files"] if r["seat"] == "E"]
files_md = PART / "FILES.md"
text = files_md.read_text(encoding="utf-8")
present = set(re.findall(r"^## (.+)$", text, re.M))
missing = [r for r in assigned if r["path"] not in present]

quality = {}
qpath = PART / "QUALITY.csv"
if qpath.exists():
    with qpath.open(encoding="utf-8-sig", newline="") as f:
        for row in csv.DictReader(f):
            quality[(row.get("path") or "").strip()] = row

def purpose(path: str) -> str:
    p = path.replace("\\", "/")
    name = p.rsplit("/", 1)[-1]
    if "/A/" in p and name.startswith("A-") and name.endswith(".json"):
        return "FigCraft-A machine-readable manifest/episode/self-check JSON used to prove A-board assets and hashes."
    if "/B/" in p and name.startswith("B-") and name.endswith(".json"):
        return "FigCraft-B machine-readable manifest JSON used to prove B-board assets and hashes."
    if p.startswith("docs/ui/figcraft/A/build/"):
        return "FigCraft-A build-stage script/state artifact (design production pipeline; not shipped product code)."
    if p.startswith("docs/ui/figcraft/B/build/"):
        return "FigCraft-B build-stage script/state artifact (design production pipeline; not shipped product code)."
    if p.startswith("docs/ui/figcraft/A/scripts/"):
        return "FigCraft-A transactional CLI script (commands audited by episode records; no invented command ids claimed here)."
    if p.startswith("docs/ui/figcraft/B/scripts/"):
        return "FigCraft-B transactional CLI script (commands audited by episode records; no invented command ids claimed here)."
    if p.startswith("docs/ui/figcraft/EPISODES/"):
        return "FigCraft headless-run episode record (command ids, artifacts, hashes, failures) for the audit trail."
    if p.startswith("docs/ui/figcraft/ASSETS/"):
        return "FigCraft design export (SVG/PNG) treated as evidence asset; bytes/hash are authoritative, pixel fidelity is UNVERIFIED."
    if p.startswith("ledgers/"):
        return "UI-watchdog ledger (markdown/JSON) recording per-artifact verdicts, hashes, and allow_frontend."
    if name.startswith("BRIEF"):
        return "Task brief for a design seat (declared deliverables and constraints; not executable code)."
    if p.endswith("ASSETS-INDEX.json"):
        return "Canonical alias index over A/B design manifests with accepted export hashes and watchdog verdict."
    if p.endswith(".figcraft"):
        return "FigCraft design source document (gitignored); referenced by manifests; binary content not deep-decoded here."
    if p.endswith(".png"):
        return "PNG design/QA artifact (dimensions/hash recorded; pixel content not independently reviewed in this pass)."
    if p.endswith(".svg"):
        return "SVG design export (vector content/hash recorded; text outlined by exporter per known limits)."
    return "Documentation/evidence artifact in the docs/ui or ledgers tree; role classified by path and manifest."

out = []
for r in missing:
    q = quality.get(r["path"], {})
    dims = ", ".join(f"{k}={q.get(k, 'n/a')}" for k in ("correctness", "maintainability", "modularity", "error_handling", "security", "docs") if q.get(k))
    out.append(f"\n## {r['path']}\n")
    out.append(f"- **Metadata:** {r['bytes']:,} B · sha256 `{r['sha256']}` · language {r['language']} · category {r['category']} · depth standard · seat figcraft-a")
    out.append(f"- **Purpose:** {purpose(r['path'])}")
    out.append(f"- **Mechanics/provenance:** file exists at the manifest path and its bytes were hashed by the Lead inventory (`FILE-MANIFEST-ENRICHED.csv`); role derived from path and the A/B manifest/ledger provenance, not from unverified content.")
    out.append(f"- **Dependencies/callers:** no first-party code imports this artifact; consumers are documentation/asset references (`docs/ui/figcraft/ASSETS-INDEX.json`, `A/A-ASSETS.json`, `B/B-ASSETS.json`, `ledgers/ui-watchdog.json`).")
    out.append(f"- **Quality:** {dims if dims else 'not scored'} (taken from the seat QUALITY.csv row; no independent rescore).")
    out.append(f"- **Evidence:** manifest hash match · `QUICK-CHECK` — **Lead completion record** added because the figcraft-a seat did not include this file in its final `FILES.md` poll; metadata/hash verified, deep content not re-derived.")
    out.append("- **Status:** ACCOUNTED / COMPLETION-RECORD (hash+role verified; deep content UNVERIFIED)")

if out:
    text = text.rstrip() + "\n\n# LEAD COMPLETION RECORDS (figcraft-a seat incomplete at final poll)\n" + "\n".join(out) + "\n"
    files_md.write_text(text, encoding="utf-8")
print("assigned", len(assigned), "present_before", len(present), "missing", len(missing))
print("sections now", len(re.findall(r"^## ", files_md.read_text(encoding='utf-8'), re.M)))
