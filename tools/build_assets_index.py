"""Build the canonical FigCraft asset acceptance index from A/B manifests + watchdog verdict."""
import hashlib, json
from datetime import datetime, timezone
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
FC = ROOT / "docs" / "ui" / "figcraft"

def sha(p: Path) -> str:
    return hashlib.sha256(p.read_bytes()).hexdigest().upper()

def rel(p: Path) -> str:
    return str(p.relative_to(ROOT)).replace("\\", "/")

a = json.loads((FC / "A" / "A-ASSETS.json").read_text(encoding="utf-8"))
b = json.loads((FC / "B" / "B-ASSETS.json").read_text(encoding="utf-8"))
wd = json.loads((ROOT / "ledgers" / "ui-watchdog.json").read_text(encoding="utf-8"))

assets = []
for row in a.get("assets", []):
    assets.append({
        "surface_id": row.get("surface_id"), "owner": "figcraft-a", "name": row.get("name"),
        "page": row.get("page"), "node_id": row.get("node_id"),
        "export_svg": row.get("export_svg"), "export_png": row.get("export_png"),
        "sha256_svg": row.get("sha256_svg"), "sha256_png": row.get("sha256_png"),
        "dimensions": row.get("dimensions"), "provenance_pdf_lines": row.get("provenance_pdf_lines"),
    })
for row in b.get("surfaces", []):
    exports = row.get("exports") or row.get("export") or {}
    if isinstance(exports, dict):
        svg = exports.get("svg") or exports.get("export_svg")
        png = exports.get("png") or exports.get("export_png")
        dims = exports.get("dimensions") or row.get("dimensions")
    else:
        svg = row.get("export_svg"); png = row.get("export_png"); dims = row.get("dimensions")
    assets.append({
        "surface_id": row.get("surface_id"), "owner": "figcraft-b", "name": row.get("name"),
        "page": row.get("page"), "node_id": row.get("node_id"),
        "export_svg": svg, "export_png": png, "dimensions": dims,
        "provenance_pdf_lines": row.get("pdf_lines") or row.get("provenance_pdf_lines"),
    })

index = {
    "schema": "hiverouter.assets-index.v1",
    "generated_at": datetime.now(timezone.utc).isoformat(),
    "generated_by": "Lead acceptance (canonical alias of A/B manifests; no pixels copied)",
    "source_manifests": {
        "A": {"path": "docs/ui/figcraft/A/A-ASSETS.json", "sha256": sha(FC / "A" / "A-ASSETS.json")},
        "B": {"path": "docs/ui/figcraft/B/B-ASSETS.json", "sha256": sha(FC / "B" / "B-ASSETS.json")},
    },
    "design_files": {
        "A": {"path": "docs/ui/figcraft/A/A-design.figcraft", "sha256": sha(FC / "A" / "A-design.figcraft")},
        "B": {"path": "docs/ui/figcraft/B/B-canvas.figcraft", "sha256": sha(FC / "B" / "B-canvas.figcraft")},
    },
    "watchdog": {
        "ledger": "ledgers/ui-watchdog.json",
        "overall": wd.get("overall"),
        "allow_frontend": wd.get("allow_frontend"),
        "fail_count": sum(1 for x in wd.get("artifacts", []) if x.get("verdict") == "FAIL"),
        "non_blocking": ["A specimens spinner #D9D9D9 (fix in progress)", "pixel-layout fidelity UNVERIFIED", "S25 DECISION-GATED"],
    },
    "counts": {"assets": len(assets), "surfaces": len({x["surface_id"] for x in assets if x.get("surface_id")})},
    "assets": assets,
}
out = FC / "ASSETS-INDEX.json"
out.write_text(json.dumps(index, indent=2), encoding="utf-8")
print("wrote", out, "assets:", len(assets), "surfaces:", index["counts"]["surfaces"], "sha:", sha(out))
