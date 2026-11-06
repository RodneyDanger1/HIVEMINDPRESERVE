"""FigCraft-B audit extractor — READ-ONLY over product docs.

Produces a per-file digest used to write FILES.md / QUALITY.csv / FINDINGS.md:
manifest metadata, sha256 re-computation, headings, marker counts, claim hints.
Writes only inside audit/parts/figcraft-b/.
"""
import csv
import hashlib
import io
import json
import os
import re
import subprocess

ROOT = r"C:\Users\Riley\Desktop\hiverouter-v1"
OUT = os.path.join(ROOT, "audit", "parts", "figcraft-b")
os.makedirs(OUT, exist_ok=True)


def sha256(path):
    h = hashlib.sha256()
    with open(path, "rb") as f:
        for chunk in iter(lambda: f.read(1 << 20), b""):
            h.update(chunk)
    return h.hexdigest().upper()


def main():
    seat = json.load(io.open(os.path.join(ROOT, "audit", "inventory",
                                          "ASSIGNMENTS.json"),
                             encoding="utf-8"))["figcraft-b"]
    man = {}
    with io.open(os.path.join(ROOT, "audit", "inventory",
                              "FILE-MANIFEST.csv"),
                 encoding="utf-8", newline="") as f:
        for row in csv.DictReader(f):
            man[row["path"]] = row

    # repo facts (read-only)
    git_head = subprocess.run(["git", "rev-parse", "HEAD"], cwd=ROOT,
                              capture_output=True, text=True).stdout.strip()
    git_short = subprocess.run(["git", "rev-parse", "--short", "HEAD"], cwd=ROOT,
                               capture_output=True, text=True).stdout.strip()
    git_status = subprocess.run(["git", "status", "--porcelain"], cwd=ROOT,
                                capture_output=True, text=True).stdout.strip()
    tracked = subprocess.run(["git", "ls-files"], cwd=ROOT,
                             capture_output=True, text=True).stdout.splitlines()

    TEXT_EXT = {".md", ".json", ".csv", ".txt", ".out", ".err", ".mjs", ".py",
                ".c", ".rs", ".js", ".yml", ".yaml", ".toml", ".sh"}
    BIN_EXT = {".exe", ".png", ".zip", ".jpg", ".webp"}
    digest = {
        "repo": {"head": git_head, "head_short": git_short,
                 "status_clean": git_status == "", "status": git_status,
                 "tracked_count": len(tracked)},
        "seat_count": len(seat), "files": [],
    }
    for item in seat:
        p = item["path"]
        full = os.path.join(ROOT, p)
        m = man.get(p, {})
        rec = {"path": p, "depth": item.get("depth"),
               "language": item.get("language"),
               "category": item.get("category"),
               "subcategory": m.get("subcategory"),
               "manifest_bytes": int(m["bytes"]) if m.get("bytes") else None,
               "manifest_sha256": (m.get("sha256") or "").upper() or None,
               "exists": os.path.exists(full)}
        if rec["exists"]:
            rec["actual_bytes"] = os.path.getsize(full)
            rec["actual_sha256"] = sha256(full)
            rec["hash_match"] = rec["actual_sha256"] == rec["manifest_sha256"]
            rec["size_match"] = rec["actual_bytes"] == rec["manifest_bytes"]
        else:
            rec["hash_match"] = None
            rec["size_match"] = None
        ext = os.path.splitext(p)[1].lower()
        rec["ext"] = ext
        rec["binary"] = ext in BIN_EXT
        if rec["exists"] and not rec["binary"]:
            try:
                text = io.open(full, encoding="utf-8", errors="replace").read()
            except Exception as ex:                       # noqa: BLE001
                text = ""
                rec["read_error"] = str(ex)
            lines = text.splitlines()
            rec["lines"] = len(lines)
            rec["headings"] = [ln.strip() for ln in lines
                               if re.match(r"^#{1,3}\s", ln)][:12]
            rec["first_lines"] = [ln.strip() for ln in lines[:14]
                                  if ln.strip()][:8]
            markers = {}
            for key in ("PASS", "FAIL", "UNVERIFIED", "PASS-WITH-NOTES",
                        "BLOCKED", "DECISION-GATED", "NOT IMPLEMENTED"):
                markers[key] = text.count(key)
            rec["markers"] = markers
            rec["status_words"] = re.findall(
                r"(PASS-WITH-NOTES|PASS|FAIL|UNVERIFIED|BLOCKED)", text)[:8]
            rec["dates"] = sorted(set(re.findall(r"20\d\d-\d\d-\d\d", text)))[:8]
            rec["sha_refs"] = sorted(set(re.findall(r"\b[0-9A-Fa-f]{40,64}\b",
                                                   text)))[:5]
            rec["number_claims"] = re.findall(
                r"\b\d[\d,]{0,6}\s*(?:tests?|files?|commands?|crates?|routes?|"
                r"artboards?|swatches?|rows?|cases?|evidence files?|"
                r"requirements?)\b", text)[:8]
            rec["provenance_hits"] = sorted(set(
                k for k in ("PDF", "figcraft", "blueprint", "upstream", "donor",
                            "source:", "VID", "UI-ADD", "MP-", "EXT-R", "CF-")
                if k in text))
        digest["files"].append(rec)

    with io.open(os.path.join(OUT, "_digest.json"), "w", encoding="utf-8",
                 newline="\n") as f:
        json.dump(digest, f, ensure_ascii=False, indent=1)

    # console digest for the auditor
    print("HEAD", git_short, "clean=", git_status == "",
          "tracked=", len(tracked), "seat=", len(seat))
    missing = [r["path"] for r in digest["files"] if not r["exists"]]
    badhash = [r["path"] for r in digest["files"] if r["hash_match"] is False]
    print("missing:", missing or "none")
    print("hash_mismatch:", badhash or "none")
    for r in digest["files"]:
        if r["binary"]:
            print("%-70s BIN %8d" % (r["path"].replace("docs/", ""),
                                     r["actual_bytes"]))
            continue
        h = r["headings"][0] if r["headings"] else "(no heading)"
        print("%-70s %5dL %6dB | %s | st=%s | %s"
              % (r["path"].replace("docs/", ""), r.get("lines", 0),
                 r["actual_bytes"], h[:44], ",".join(r["status_words"][:3]),
                 ",".join(r["dates"][-2:])))


if __name__ == "__main__":
    main()
