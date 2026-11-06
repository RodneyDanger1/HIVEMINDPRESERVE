"""Generate the 120-task, 6-phase, 8-seat audit/migration roadmap."""
from __future__ import annotations

import csv
from pathlib import Path

ROOT = Path(r"C:\Users\Riley\Desktop\hiverouter-v1")
OUT = ROOT / "audit"
SEATS = ["b-core", "b-provider", "watchdog-1", "watchdog-2", "figcraft-a", "figcraft-b", "ui-watchdog", "strip-watchdog"]

PHASES = [
    ("P1", "Reconnaissance & Full Inventory", [
        "Recursive directory walk and artifact census", "Tracked-vs-ignored class reconciliation",
        "Language and LOC distribution profile", "Dependency manifest extraction (Cargo/npm)",
        "Lockfile provenance and checksum census", "Git-history vs manifest reconciliation",
        "File hash integrity baseline", "Seat-assignment coverage check (507/507)",
        "Generated-artifact tracking audit (runtime-test DB/logs)", "Asset census (FigCraft/PDF/PNG/SVG)",
        "Binary artifact census (target/dist/exe)", "Directory ownership and module boundary map",
        "Untracked/ignored artifact policy", "Census reproducibility script", "Inventory sign-off report",
    ]),
    ("P2", "Micro-File Exhaustive Analysis", [
        "Core entrypoint and boot sequence analysis", "Contract envelope/command/event/refusal analysis",
        "Transport/WS security and resync analysis", "Graph authority/cycle analysis",
        "Run lifecycle/cancellation/caps analysis", "Snapshot/migration/kill-restore analysis",
        "Provider DTO/streaming/refusal analysis", "Budget/Decimal/pricing analysis",
        "Tool registry/schema/allowlist analysis", "Approval nonce/replay/TTL analysis",
        "Workspace jail escape analysis", "Audit ledger/observability analysis",
        "Web token/contrast/a11y analysis", "API client/ACK-before-mutation analysis",
        "State store/event reducer analysis", "Canvas/nodes/edge analysis", "Panels/chat/controls analysis",
        "E2E harness/trace-spec analysis", "CLI/agent-native contract analysis", "Quality evidence per file",
    ]),
    ("P3", "Architectural Mapping & Interaction Tracing", [
        "Cross-module import graph", "Function/command call graph", "Event emitter/subscriber parity",
        "HTTP route inventory and handler map", "WS command dispatch matrix", "Store→DB→event flow trace",
        "Provider call path and failure identity trace", "Tool execution/approval path trace",
        "Static-UI/serve path trace", "History/replay path trace", "Error/refusal propagation map",
        "Concurrency and backpressure map", "Boundary/ownership map", "Bottleneck and refactor-risk inventory",
        "Mermaid topology diagram set", "Mermaid data-flow diagram", "Mermaid state-machine diagram",
        "Mermaid dependency/provenance diagram", "Diagram validation/export receipts", "Architecture sign-off",
    ]),
    ("P4", "Upstream Provenance & Donor Repo Audit", [
        "Crate dependency lineage extraction", "npm dependency lineage extraction",
        "License compatibility matrix", "Vendored/toolchain provenance audit",
        "FigCraft asset provenance audit", "Blueprint/donor pattern lineage map",
        "Upstream repo currency and maintenance check", "Donor mechanism adoption map",
        "Candidate donor capability gaps", "Supply-chain security review (lockfiles/checksums)",
        "Provenance UNVERIFIED register", "Provenance Mermaid graph", "Third-party notice/attribution check",
        "Dependency upgrade risk list", "Provenance sign-off",
    ]),
    ("P5", "Code Quality, Mathematical Rigor & Tech Debt Scoring", [
        "Per-file correctness evidence score", "Maintainability/complexity score", "Modularity/coupling score",
        "Error-handling/refusal score", "Security/authority score", "Documentation score",
        "Test coverage mapping by module", "Runtime/behavioral evidence audit", "Mathematical/logic invariants audit",
        "Money/Decimal integrity audit", "Concurrency/timeout invariant audit", "Generated-artifact debt catalog",
        "Stale-doc/claim drift catalog", "Debt severity/priority ranking", "Quality matrix sign-off",
    ]),
    ("P6", "Scientific Research Paper Synthesis & Final Roadmap", [
        "Abstract/introduction drafting", "Architecture & topology chapter", "Empirical file-analysis chapter",
        "Provenance & donor-lineage chapter", "Quality/logic chapter", "Current-vs-target state specification",
        "Refactor wave 1 (P0 safety/correctness)", "Refactor wave 2 (contracts/observability)",
        "Refactor wave 3 (UI/UX + assets)", "Refactor wave 4 (packaging/toolchain)",
        "Acceptance gates per wave", "Risk/rollback plan", "Diagram finalization",
        "Zero-hallucination fact-check pass", "Final paper compile + appendix", "Final deliverable sign-off",
    ]),
]

TASK_DETAIL = {
    "P1": "Produce manifest-backed evidence; no claims without file/hash.",
    "P2": "Read the file; cite path:line; deep set 8-12 paragraphs.",
    "P3": "Trace callers/emit/subscribe; cite path:line; render Mermaid.",
    "P4": "Fetch official source or mark UNVERIFIED; cite URL+date.",
    "P5": "Score 1-5 with evidence; list debt with severity.",
    "P6": "Compile verified text only; reconcile every number with manifests.",
}

rows = []
idx = 1
for phase_id, phase_name, titles in PHASES:
    for i, title in enumerate(titles):
        seat = SEATS[(i + (0 if phase_id in ("P1", "P4") else 2)) % len(SEATS)]
        rows.append({
            "task_id": f"T{idx:03d}", "phase": f"{phase_id}: {phase_name}", "seat": seat,
            "title": title, "objective": f"{title} for the audited repository, with path/hash evidence.",
            "deliverable": {
                "P1": "inventory/manifest or census artifact", "P2": "audit/parts/<seat>/FILES.md section",
                "P3": "dependency/trace note or audit/diagrams/*.mmd", "P4": "PROVENANCE.md row + URL",
                "P5": "QUALITY.csv row + FINDINGS.md entry", "P6": "paper/roadmap section",
            }[phase_id],
            "acceptance": TASK_DETAIL[phase_id],
            "evidence_required": "command output, path:line, or official URL+date",
            "depends_on": "" if idx <= 15 else f"T{max(1, idx-15):03d}",
            "estimate": "15-40 min", "priority": "P1" if phase_id in ("P1", "P5", "P6") else "P2",
        })
        idx += 1

(OUT / "ROADMAP-TASKS.csv").write_text("", encoding="utf-8")
with (OUT / "ROADMAP-TASKS.csv").open("w", newline="", encoding="utf-8") as f:
    w = csv.DictWriter(f, fieldnames=list(rows[0].keys()))
    w.writeheader(); w.writerows(rows)
md = ["# EXECUTION ROADMAP — 120 tasks / 6 phases / 8 seats", "",
      f"Tasks: **{len(rows)}** · Phases: **{len(PHASES)}** · Seats: **{len(SEATS)}**", "",
      "| Phase | Tasks | Seats |", "|---|---:|---|"]
for pid, pname, titles in PHASES:
    ph = [r for r in rows if r["phase"].startswith(pid)]
    md.append(f"| {pid} {pname} | {len(ph)} | {', '.join(sorted({r['seat'] for r in ph}))} |")
md += ["", "## Task table", "", "| ID | Phase | Seat | Task | Acceptance |", "|---|---|---|---|---|"]
for r in rows:
    md.append(f"| {r['task_id']} | {r['phase'].split(':')[0]} | {r['seat']} | {r['title']} | {r['acceptance']} |")
(OUT / "ROADMAP.md").write_text("\n".join(md) + "\n", encoding="utf-8")
print("roadmap tasks", len(rows), "phases", len(PHASES))
