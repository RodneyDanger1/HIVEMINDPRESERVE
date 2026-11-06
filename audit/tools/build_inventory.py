"""Build exhaustive repository inventory for the HiveMind v1 audit.

Scope: all git-tracked files (507 at HEAD 3988622). Ignored/generated trees (target/,
web/node_modules/, e2e/node_modules/, tools/w64devkit/, dist/, .git/) are censused as
classes with file counts/bytes, not re-hashed individually (they are build artifacts and
third-party vendor trees). Every tracked file gets sha256 + category + language + seat.
"""
from __future__ import annotations

import csv
import hashlib
import json
import subprocess
from collections import Counter, defaultdict
from pathlib import Path

ROOT = Path(r"C:\Users\Riley\Desktop\hiverouter-v1")
AUDIT = ROOT / "audit"

LANG = {
    ".rs": "Rust", ".ts": "TypeScript", ".tsx": "TypeScript/TSX", ".mjs": "JavaScript",
    ".js": "JavaScript", ".py": "Python", ".ps1": "PowerShell", ".c": "C",
    ".md": "Markdown", ".txt": "Text", ".json": "JSON", ".toml": "TOML",
    ".lock": "Lockfile", ".csv": "CSV", ".html": "HTML", ".css": "CSS",
    ".svg": "SVG", ".png": "PNG", ".figcraft": "FigCraft", ".mmd": "Mermaid",
    ".prompty": "Prompty", ".db": "SQLite", ".db-wal": "SQLite-WAL",
    ".db-shm": "SQLite-SHM", ".out": "Log", ".err": "Log", ".log": "Log",
    ".exe": "PE-binary", ".zip": "Zip", ".pyc": "Python-bytecode",
    ".gitignore": "Git-config",
}

SEATS = {
    "A": "b-core", "B": "b-provider", "C": "watchdog-1", "D": "watchdog-2",
    "E": "figcraft-a", "F": "figcraft-b", "G": "ui-watchdog", "H": "strip-watchdog",
}

DEEP_PATTERNS = [
    "backend/crates/hiverouter-contract/src/lib.rs",
    "backend/crates/hiverouter-contract/src/envelope.rs",
    "backend/crates/hiverouter-contract/src/command.rs",
    "backend/crates/hiverouter-contract/src/event.rs",
    "backend/crates/hiverouter-contract/src/refusal.rs",
    "backend/crates/hiverouter-contract/src/state.rs",
    "backend/crates/hiverouter-contract/src/schema.rs",
    "backend/crates/hiverouter-core/src/main.rs",
    "backend/crates/hiverouter-core/src/http.rs",
    "backend/crates/hiverouter-core/src/ws.rs",
    "backend/crates/hiverouter-core/src/graph.rs",
    "backend/crates/hiverouter-core/src/execution.rs",
    "backend/crates/hiverouter-core/src/db.rs",
    "backend/crates/hiverouter-core/src/snapshot.rs",
    "backend/crates/hiverouter-core/src/recovery.rs",
    "backend/crates/hiverouter-core/src/static_ui.rs",
    "backend/crates/hiverouter-core/src/tools_bridge.rs",
    "backend/crates/hiverouter-core/src/prompts.rs",
    "backend/crates/hiverouter-core/src/handshake.rs",
    "backend/crates/hiverouter-core/src/context.rs",
    "backend/crates/hiverouter-core/src/bus.rs",
    "backend/crates/hiverouter-provider/src/client.rs",
    "backend/crates/hiverouter-provider/src/dto.rs",
    "backend/crates/hiverouter-provider/src/budget.rs",
    "backend/crates/hiverouter-provider/src/accounting.rs",
    "backend/crates/hiverouter-provider/src/pricing.rs",
    "backend/crates/hiverouter-tools/src/approval.rs",
    "backend/crates/hiverouter-tools/src/jail.rs",
    "backend/crates/hiverouter-tools/src/shell.rs",
    "backend/crates/hiverouter-tools/src/cli.rs",
    "backend/crates/hiverouter-tools/src/tool.rs",
    "backend/crates/hiverouter-tools/src/ledger.rs",
    "backend/crates/hiverouter-tools/src/census.rs",
    "backend/crates/hiverouter-tools/src/health.rs",
    "web/src/App.tsx",
    "web/src/tokens/tokens.ts",
    "web/src/tokens/contrast.ts",
    "web/src/api/contract.ts",
    "web/src/api/client.ts",
    "web/src/state/store.ts",
    "web/src/state/agentState.ts",
    "web/src/state/strings.ts",
    "web/src/flow/Canvas.tsx",
    "web/src/flow/nodes.tsx",
    "web/src/chat/ChatCore.tsx",
    "web/src/panels/DataPanel.tsx",
    "web/src/panels/SettingsPanel.tsx",
    "web/src/a11y/keyboard.ts",
    "e2e/specs/live-app.spec.ts",
    "e2e/specs/trace-hops.spec.ts",
    "e2e/playwright.config.ts",
    "tools/qa/verify-harness.mjs",
    "backend/Cargo.toml",
    "backend/Cargo.lock",
    "web/package.json",
    "web/vite.config.ts",
]


def seat_for(path: str) -> str:
    p = path.replace("\\", "/")
    if p.startswith("backend/crates/hiverouter-contract/") or p.startswith("backend/crates/hiverouter-core/"):
        return "A"
    if p.startswith("backend/crates/hiverouter-provider/") or p.startswith("backend/crates/hiverouter-tools/"):
        return "B"
    if p in ("backend/Cargo.toml", "backend/Cargo.lock"):
        return "B"
    if p.startswith("web/"):
        return "C"
    if p.startswith("e2e/") or p.startswith("tools/") or p.startswith("runtime-test/"):
        return "D"
    if p.startswith("docs/ui/") or p.startswith("docs/evidence/") or p.startswith("ledgers/"):
        return "E"
    if p.startswith("docs/plans/") or p.startswith("docs/contracts/") or p.startswith("docs/claims/") or p.startswith("docs/reports/") or p.startswith("docs/audit/"):
        return "F"
    if p.startswith("docs/trace/") or p.startswith("docs/diagrams/") or p in (
        "README.md", "CHANGE-LEDGER-IMPL.md", ".gitignore", "rust-toolchain.toml", ".cargo/config.toml"
    ):
        return "G"
    return "H"


def category(path: str) -> tuple[str, str]:
    p = path.replace("\\", "/")
    ext = Path(p).suffix.lower()
    if p.startswith("runtime-test/"):
        return "runtime-artifact", "generated/test-output"
    if p.startswith(("docs/", "ledgers/")):
        if ext in (".png", ".svg", ".figcraft", ".zip"):
            return "documentation-asset", "design/evidence asset"
        return "documentation", "report/plan/evidence"
    if p.startswith(("web/", "backend/", "e2e/", "tools/")):
        if ext in (".rs", ".ts", ".tsx", ".mjs", ".js", ".py", ".ps1", ".c", ".css", ".html", ".prompty"):
            return "source", "first-party code"
        if ext in (".json", ".toml", ".lock", ".gitignore"):
            return "config-manifest", "build/dependency/config"
        if ext in (".png", ".svg", ".mmd", ".figcraft", ".db", ".db-wal", ".db-shm", ".exe", ".zip", ".pyc", ".out", ".err", ".log", ".txt"):
            return "generated-artifact", "test/evidence artifact"
    return "other", "uncategorized"


def main() -> None:
    tracked = subprocess.check_output(["git", "ls-files"], cwd=ROOT, text=True).splitlines()
    tracked = [t for t in tracked if t.strip()]
    rows = []
    seat_counts = Counter()
    for rel in tracked:
        fp = ROOT / rel
        data = fp.read_bytes() if fp.exists() else b""
        ext = Path(rel).suffix.lower()
        cat, sub = category(rel)
        seat = seat_for(rel)
        seat_counts[seat] += 1
        rows.append({
            "path": rel,
            "bytes": len(data),
            "sha256": hashlib.sha256(data).hexdigest().upper(),
            "ext": ext,
            "language": LANG.get(ext, "unknown"),
            "category": cat,
            "subcategory": sub,
            "seat": seat,
            "depth": "deep" if rel.replace("\\", "/") in DEEP_PATTERNS else "standard",
        })
    rows.sort(key=lambda r: r["path"])
    out = AUDIT / "inventory"
    out.mkdir(parents=True, exist_ok=True)
    with (out / "FILE-MANIFEST.csv").open("w", newline="", encoding="utf-8") as f:
        w = csv.DictWriter(f, fieldnames=list(rows[0].keys()))
        w.writeheader()
        w.writerows(rows)
    with (out / "FILE-MANIFEST.json").open("w", encoding="utf-8") as f:
        json.dump({"repo": str(ROOT), "head": subprocess.check_output(["git", "rev-parse", "HEAD"], cwd=ROOT, text=True).strip(),
                   "count": len(rows), "files": rows}, f, indent=2)

    assignments = defaultdict(list)
    for r in rows:
        assignments[SEATS[r["seat"]]].append({"path": r["path"], "depth": r["depth"], "language": r["language"], "category": r["category"]})
    (out / "ASSIGNMENTS.json").write_text(json.dumps(assignments, indent=2), encoding="utf-8")

    by_category = Counter(r["category"] for r in rows)
    by_lang = Counter(r["language"] for r in rows)
    by_seat = Counter(SEATS[r["seat"]] for r in rows)
    deep = [r["path"] for r in rows if r["depth"] == "deep"]
    lines = ["# DIRECTORY / CATEGORY CENSUS", "", f"Tracked files: **{len(rows)}**", ""]
    lines += ["## By top-level directory", "", "| Directory | Files |", "|---|---:|"]
    top = Counter(r["path"].split("/")[0] if "/" in r["path"] else "(root)" for r in rows)
    lines += [f"| `{k}` | {v} |" for k, v in top.most_common()]
    lines += ["", "## By category", "", "| Category | Files |", "|---|---:|"]
    lines += [f"| {k} | {v} |" for k, v in by_category.most_common()]
    lines += ["", "## By language", "", "| Language | Files |", "|---|---:|"]
    lines += [f"| {k} | {v} |" for k, v in by_lang.most_common()]
    lines += ["", "## By audit seat", "", "| Seat | Teammate | Files |", "|---|---|---:|"]
    lines += [f"| {k} | {v} |" for k, v in sorted(by_seat.items())]
    lines += ["", "## Deep-analysis set (8-12 paragraph target)", ""]
    lines += [f"- `{p}`" for p in deep]
    lines += ["", "## Generated/ignored classes (counted, not per-file prose)", "",
              "| Tree | Files | MB | Class |", "|---|---:|---:|---|",
              "| `target/` | 13,525 | 8,087.2 | Rust build artifacts |",
              "| `web/node_modules/` | 3,976 | 110.2 | npm vendor |",
              "| `e2e/node_modules/` | 194 | 18.2 | npm vendor |",
              "| `tools/w64devkit/` | 9,452 | 716.7 | third-party toolchain |",
              "| `dist/` | 6 | 20.3 | packaged release output |",
              "| `runtime-test/` | 82 (tracked) | 33.6 | runtime/test DB + logs (tracked by mistake: finding) |",
              "| `.git/` | not counted | - | VCS internals, excluded by convention |"]
    (out / "DIRECTORY-CENSUS.md").write_text("\n".join(lines) + "\n", encoding="utf-8")
    (out / "DEEP-LIST.md").write_text("# Deep-analysis files\n\n" + "\n".join(f"- `{p}`" for p in deep) + "\n", encoding="utf-8")
    print(f"tracked={len(rows)} deep={len(deep)} seats={dict(by_seat)} categories={dict(by_category)}")


if __name__ == "__main__":
    main()
