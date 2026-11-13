"""Build the canonical FigCraft asset + board acceptance index.

Inputs (read-only):
  docs/ui/figcraft/A/A-ASSETS.json
  docs/ui/figcraft/B/B-ASSETS.json
  docs/ui/figcraft/HiveMind/HiveMind-ASSETS.json
  ledgers/ui-watchdog.json
  docs/ui/figcraft/HiveMind/build/verify-report.json

Output:
  docs/ui/figcraft/ASSETS-INDEX.json

The index is the single input for tools/build-figcraft-catalog.mjs. Each board
record carries its accepted manifest sha256 and an explicit audit status so the
generated catalog cannot silently present a proposed or self-verified board as
watchdog-audited. No pixels are copied here; build-figcraft-catalog.mjs copies
the served SVG bytes and re-verifies every hash.
"""
import hashlib
import json
import sys
from datetime import datetime, timezone
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
FC = ROOT / "docs" / "ui" / "figcraft"
WATCHDOG = ROOT / "ledgers" / "ui-watchdog.json"
HM_VERIFY = FC / "HiveMind" / "build" / "verify-report.json"

AUDIT_WATCHDOG = "watchdog-audited"
AUDIT_SELF = "self-verified"
AUDIT_PROPOSED = "proposed"
AUDIT_UNVERIFIED = "unverified"
AUDIT_STATUSES = {AUDIT_WATCHDOG, AUDIT_SELF, AUDIT_PROPOSED, AUDIT_UNVERIFIED}

# HiveMind board codes that the set's own README marks as PROPOSED-ADDITION
# (HMAS hierarchy/governance, states, motion) or as a proposal carrying
# SAMPLE/READ values (run detail). Boards 00-07 are the self-verified spatial
# style system. None of the HiveMind boards are in the A/B watchdog ledger.
HIVE_PROPOSED_CODES = {"08", "09", "10", "11", "12"}


def sha(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest().upper()


def rel(path: Path) -> str:
    return str(path.relative_to(ROOT)).replace("\\", "/")


def load(path: Path) -> dict:
    if not path.is_file():
        raise SystemExit("missing required manifest: %s" % rel(path))
    return json.loads(path.read_text(encoding="utf-8"))


def audit_status(record: dict) -> str:
    status = (record or {}).get("status")
    if status not in AUDIT_STATUSES:
        raise SystemExit("invalid audit status %r in board record" % (status,))
    return status


def asset_row(row: dict, owner: str) -> dict:
    """Keep the canonical alias fields stable; add sha fields only when known."""
    out = {
        "surface_id": row.get("surface_id"),
        "owner": owner,
        "name": row.get("name"),
        "page": row.get("page"),
        "node_id": row.get("node_id"),
        "export_svg": row.get("export_svg"),
        "export_png": row.get("export_png"),
        "dimensions": row.get("dimensions"),
        "provenance_pdf_lines": row.get("provenance_pdf_lines"),
    }
    for key in ("sha256_svg", "sha256_png", "export_png_2x", "set"):
        if row.get(key) is not None:
            out[key] = row[key]
    return out


def main() -> int:
    a = load(FC / "A" / "A-ASSETS.json")
    b = load(FC / "B" / "B-ASSETS.json")
    hm = load(FC / "HiveMind" / "HiveMind-ASSETS.json")
    wd = load(WATCHDOG)
    verify = load(HM_VERIFY) if HM_VERIFY.is_file() else None

    watchdog_by_file = {
        artifact.get("file"): artifact
        for artifact in wd.get("artifacts", [])
        if artifact.get("file")
    }
    hm_verify_sha = sha(HM_VERIFY) if verify is not None else None
    hm_verify_status = verify.get("status") if verify is not None else None

    # ---- assets: the canonical A + B + HiveMind surface aliases (114 rows) ----
    assets = [asset_row(row, "figcraft-a") for row in a.get("assets", [])]
    b_export_by_svg = {
        export.get("svg"): (slug, export)
        for slug, export in (b.get("exports") or {}).items()
    }
    for row in b.get("surfaces", []):
        exports = row.get("exports") or row.get("export") or {}
        svg = exports.get("svg") if isinstance(exports, dict) else row.get("export_svg")
        png = exports.get("png") if isinstance(exports, dict) else row.get("export_png")
        dims = (exports.get("dimensions") if isinstance(exports, dict) else None) or row.get("dimensions")
        match = b_export_by_svg.get(svg)
        row_export = match[1] if match else {}
        assets.append(
            {
                "surface_id": row.get("surface_id"),
                "owner": "figcraft-b",
                "name": row.get("name"),
                "page": row.get("page"),
                "node_id": row.get("node_id"),
                "export_svg": svg,
                "export_png": png,
                "sha256_svg": row_export.get("sha256_svg"),
                "sha256_png": row_export.get("sha256_png"),
                "dimensions": dims,
                "provenance_pdf_lines": row.get("pdf_lines") or row.get("provenance_pdf_lines"),
            }
        )
    for surface in hm.get("surfaces", []):
        assets.append(
            {
                "surface_id": surface.get("surface_id"),
                "owner": "figcraft-hivemind",
                "name": surface.get("name"),
                "page": surface.get("board"),
                "node_id": None,
                "export_svg": surface.get("export_svg"),
                "export_png": surface.get("export_png"),
                "export_png_2x": surface.get("export_png_2x"),
                "sha256_svg": surface.get("sha256_svg"),
                "sha256_png": surface.get("sha256_png"),
                "dimensions": surface.get("dimensions"),
                "provenance_pdf_lines": surface.get("provenance"),
                "set": "HiveMind",
            }
        )

    # ---- boards: one record per served SVG, with per-board sha + audit ----
    boards = []

    def add_board(record: dict) -> None:
        required = ("id", "owner", "title", "width", "height", "svg_file",
                    "source_svg", "sha256_svg", "bytes", "manifest")
        for key in required:
            if record.get(key) in (None, ""):
                raise SystemExit("board record missing %s: %r" % (key, record.get("id")))
        audit_status(record.get("audit", {}))
        source = ROOT / record["source_svg"]
        if not source.is_file():
            raise SystemExit("board source missing: %s" % record["source_svg"])
        actual = sha(source)
        if actual != str(record["sha256_svg"]).upper():
            raise SystemExit(
                "board sha mismatch for %s: manifest %s actual %s"
                % (record["id"], record["sha256_svg"], actual)
            )
        if record["bytes"] != source.stat().st_size:
            raise SystemExit("board byte count mismatch for %s" % record["id"])
        boards.append(record)

    def watchdog_audit(source_svg: str) -> dict:
        artifact = watchdog_by_file.get(source_svg)
        if artifact is None:
            return {
                "status": AUDIT_UNVERIFIED,
                "verdict": None,
                "ledger": rel(WATCHDOG),
                "basis": "not present in ledgers/ui-watchdog.json",
            }
        verdict = artifact.get("verdict")
        if verdict not in ("PASS", "PASS-WITH-NOTES"):
            return {
                "status": AUDIT_UNVERIFIED,
                "verdict": verdict,
                "ledger": rel(WATCHDOG),
                "basis": "watchdog verdict is %s" % verdict,
            }
        recorded = str(artifact.get("sha256") or "").upper()
        actual = sha(ROOT / source_svg)
        if recorded != actual:
            raise SystemExit(
                "watchdog sha mismatch for %s: ledger %s actual %s"
                % (source_svg, recorded, actual)
            )
        return {
            "status": AUDIT_WATCHDOG,
            "verdict": verdict,
            "ledger": rel(WATCHDOG),
            "basis": "watchdog-audited export, sha verified against the ledger",
        }

    # HiveMind boards, in design-file order (00..12).
    for board in hm.get("boards", []):
        slug = board.get("slug")
        export = (hm.get("exports") or {}).get(slug)
        if not export:
            raise SystemExit("HiveMind board %s has no export record" % slug)
        code = str(board.get("code", ""))
        if code in HIVE_PROPOSED_CODES:
            audit = {
                "status": AUDIT_PROPOSED,
                "verdict": None,
                "ledger": None,
                "basis": (
                    "PROPOSED-ADDITION / motion proposal per "
                    "docs/ui/figcraft/HiveMind/README.md; not watchdog-audited"
                ),
            }
        else:
            if verify is None or hm_verify_status != "PASS":
                audit = {
                    "status": AUDIT_UNVERIFIED,
                    "verdict": hm_verify_status,
                    "ledger": rel(HM_VERIFY),
                    "basis": "HiveMind verify report is missing or not PASS",
                }
            else:
                audit = {
                    "status": AUDIT_SELF,
                    "verdict": "PASS",
                    "ledger": rel(HM_VERIFY),
                    "basis": (
                        "HiveMind verify_hivemind.py report PASS; "
                        "not a watchdog audit"
                    ),
                }
        add_board(
            {
                "id": slug,
                "owner": "figcraft-hivemind",
                "title": board.get("frame"),
                "width": board.get("width"),
                "height": board.get("height"),
                "svg_file": Path(export.get("svg")).name,
                "source_svg": export.get("svg"),
                "source_png": export.get("png"),
                "sha256_svg": export.get("sha256_svg"),
                "bytes": export.get("svg_bytes"),
                "manifest": rel(FC / "HiveMind" / "HiveMind-ASSETS.json"),
                "surfaces": [
                    {"surfaceId": surface.get("surface_id"), "name": surface.get("name")}
                    for surface in hm.get("surfaces", [])
                    if surface.get("export_svg") == export.get("svg")
                ],
                "audit": audit,
                "served": True,
            }
        )

    # A boards, in first-appearance order of the accepted asset rows.
    a_groups = {}
    for row in a.get("assets", []):
        svg = row.get("export_svg")
        if not svg:
            continue
        a_groups.setdefault(svg, []).append(row)
    for svg, rows in a_groups.items():
        first = rows[0]
        board_id = Path(svg).stem
        dims = first.get("dimensions") or {}
        width, height = (dims.get("board") or [None, None])[:2]
        add_board(
            {
                "id": board_id,
                "owner": "figcraft-a",
                "title": first.get("page"),
                "width": width,
                "height": height,
                "svg_file": "%s.svg" % board_id,
                "source_svg": svg,
                "source_png": first.get("export_png"),
                "sha256_svg": first.get("sha256_svg"),
                "bytes": (ROOT / svg).stat().st_size,
                "manifest": rel(FC / "A" / "A-ASSETS.json"),
                "surfaces": [
                    {"surfaceId": row.get("surface_id"), "name": row.get("name")}
                    for row in rows
                ],
                "audit": watchdog_audit(svg),
                "served": True,
            }
        )

    # B boards, in export order; title is the humanised export slug.
    for slug, export in (b.get("exports") or {}).items():
        svg = export.get("svg")
        if not svg:
            continue
        dims = export.get("dimensions") or {}
        width, height = (dims.get("board") or [None, None])[:2]
        title = slug.replace("b-", "").replace("-", " ").strip().title()
        surfaces = []
        for row in b.get("surfaces", []):
            exports = row.get("exports") or {}
            if exports.get("svg") == svg:
                surfaces.append(
                    {"surfaceId": row.get("surface_id"), "name": row.get("name")}
                )
        add_board(
            {
                "id": slug,
                "owner": "figcraft-b",
                "title": title,
                "width": width,
                "height": height,
                "svg_file": Path(export.get("svg")).name,
                "source_svg": svg,
                "source_png": export.get("png"),
                "sha256_svg": export.get("sha256_svg"),
                "bytes": export.get("svg_bytes") or (ROOT / svg).stat().st_size,
                "manifest": rel(FC / "B" / "B-ASSETS.json"),
                "surfaces": surfaces,
                "audit": watchdog_audit(svg),
                "served": True,
            }
        )

    if len({board["id"] for board in boards}) != len(boards):
        raise SystemExit("duplicate board id in index")
    if len({board["svg_file"] for board in boards}) != len(boards):
        raise SystemExit("duplicate served svg file name in index")

    served = [board for board in boards if board.get("served") is not False]
    distinct_surfaces = {
        surface["surfaceId"]
        for board in served
        for surface in board.get("surfaces", [])
    }
    counts = {
        "assets": len(assets),
        "surfaces": len({row["surface_id"] for row in assets if row.get("surface_id")}),
        "surface_mappings": sum(len(board.get("surfaces", [])) for board in served),
        "boards": len(served),
        "watchdog_audited_boards": sum(
            1 for board in served if board["audit"]["status"] == AUDIT_WATCHDOG
        ),
        "self_verified_boards": sum(
            1 for board in served if board["audit"]["status"] == AUDIT_SELF
        ),
        "proposed_boards": sum(
            1 for board in served if board["audit"]["status"] == AUDIT_PROPOSED
        ),
        "unverified_boards": sum(
            1 for board in served if board["audit"]["status"] == AUDIT_UNVERIFIED
        ),
        "distinct_surfaces": len(distinct_surfaces),
        "owners": len({board["owner"] for board in served}),
    }

    index = {
        "schema": "hiverouter.assets-index.v2",
        "generated_at": datetime.now(timezone.utc).isoformat(timespec="seconds"),
        "generated_by": (
            "tools/build_assets_index.py - canonical alias of the A/B/HiveMind "
            "manifests; per-board sha and audit status; no pixels copied"
        ),
        "source_manifests": {
            "A": {"path": rel(FC / "A" / "A-ASSETS.json"), "sha256": sha(FC / "A" / "A-ASSETS.json")},
            "B": {"path": rel(FC / "B" / "B-ASSETS.json"), "sha256": sha(FC / "B" / "B-ASSETS.json")},
            "HiveMind": {
                "path": rel(FC / "HiveMind" / "HiveMind-ASSETS.json"),
                "sha256": sha(FC / "HiveMind" / "HiveMind-ASSETS.json"),
            },
        },
        "design_files": {
            "A": {"path": rel(FC / "A" / "A-design.figcraft"), "sha256": sha(FC / "A" / "A-design.figcraft")},
            "B": {"path": rel(FC / "B" / "B-canvas.figcraft"), "sha256": sha(FC / "B" / "B-canvas.figcraft")},
            "HiveMind": {
                "path": rel(FC / "HiveMind" / "HiveMind-Style-System.figcraft"),
                "sha256": sha(FC / "HiveMind" / "HiveMind-Style-System.figcraft"),
            },
        },
        "watchdog": {
            "ledger": rel(WATCHDOG),
            "overall": wd.get("overall"),
            "allow_frontend": wd.get("allow_frontend"),
            "fail_count": sum(
                1 for artifact in wd.get("artifacts", []) if artifact.get("verdict") == "FAIL"
            ),
            "scope": wd.get("scope"),
            "non_blocking": [
                "A specimens spinner #D9D9D9 (fix in progress)",
                "pixel-layout fidelity UNVERIFIED",
                "S25 DECISION-GATED",
            ],
        },
        "hivemind_verify": {
            "report": rel(HM_VERIFY),
            "sha256": hm_verify_sha,
            "status": hm_verify_status,
            "failed_checks": verify.get("failed_checks") if verify is not None else None,
            "scope": "HiveMind CLI build evidence only; not a watchdog audit",
        },
        "counts": counts,
        "boards": boards,
        "assets": assets,
    }
    out = FC / "ASSETS-INDEX.json"
    out.write_text(json.dumps(index, indent=2, ensure_ascii=False) + "\n", encoding="utf-8")
    print(
        "wrote %s boards=%d served=%d watchdog=%d self=%d proposed=%d assets=%d surfaces=%d sha=%s"
        % (
            rel(out),
            len(boards),
            counts["boards"],
            counts["watchdog_audited_boards"],
            counts["self_verified_boards"],
            counts["proposed_boards"],
            counts["assets"],
            counts["surfaces"],
            sha(out),
        )
    )
    return 0


if __name__ == "__main__":
    sys.exit(main())
