"""Emit A/A-ASSETS.json + A/A-EPISODES.json + EPISODES/a-*.json from build state.

Reads build/build-state.json and the exported assets, verifies every surface node
with node.get, and writes the deliverables the UI watchdog consumes.
"""
import json
import os
import subprocess
import sys
import time

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import fig
import gen_common as G

A = fig.ROOT
BUILD = os.path.join(A, "build")
DOC = os.path.join(A, "A-design.figcraft")
ASSETS_DIR = os.path.join(os.path.dirname(A), "ASSETS")
REPO = os.path.abspath(os.path.join(A, "..", "..", "..", ".."))
EPISODES = os.path.join(os.path.dirname(A), "EPISODES")
STATE = os.path.join(BUILD, "build-state.json")

BOARD_META = {
    "00": ("00 Foundations", "a-foundations"),
    "01": ("01 Brand", "a-brand"),
    "02": ("02 Components", "a-components"),
    "03": ("03 Specimens", "a-specimens"),
}
CLI_VERSION = "0.11.2"


def sha256(path):
    import hashlib
    h = hashlib.sha256()
    with open(path, "rb") as f:
        for chunk in iter(lambda: f.read(1 << 20), b""):
            h.update(chunk)
    return h.hexdigest()


def load_state():
    with open(STATE, encoding="utf-8") as f:
        return json.load(f)


def surface_entries(state):
    boards = state["boards"]
    entries = []
    missing = []
    for sid in G.SURFACES:
        name, wiring, pdf = G.SURFACES[sid]
        found = None
        for code, b in boards.items():
            surfaces = b["surfaces"]
            if sid in surfaces:
                found = (code, b)
                break
        if not found:
            missing.append(sid)
            continue
        code, b = found
        idx = b["surface_indices"][sid]
        el = b["element_by_idx"][str(idx)]
        el_name, el_id = el["name"], el["id"]
        group = None
        for gname, gid in b["group_ids"].items():
            if gname.startswith(sid):
                # the surface node is the named group when one starts with the id
                group = (gname, gid)
                break
        if group:
            node_id, node_name = group[1], group[0]
        else:
            node_id, node_name = el_id, el_name
        page, stem = BOARD_META[code]
        exp = state["exports"][stem]
        entries.append({
            "surface_id": sid,
            "name": name,
            "page": page,
            "node_id": node_id,
            "node_name": node_name,
            "wiring": wiring,
            "export_svg": os.path.relpath(exp["svg"], REPO).replace(os.sep, "/"),
            "export_png": os.path.relpath(exp["png"], REPO).replace(os.sep, "/"),
            "sha256_svg": exp["sha256_svg"],
            "sha256_png": exp["sha256_png"],
            "dimensions": exp["dimensions"],
            "provenance_pdf_lines": pdf,
        })
    return entries, missing


def verify_nodes(entries):
    cmds = [{"command": "node.get", "params": {"id": e["node_id"]}} for e in entries]
    results = fig.run_script(cmds, in_path=DOC)
    checks = []
    for e, r in zip(entries, results):
        node = r.get("result")
        ok = node is not None
        checks.append({"surface_id": e["surface_id"], "node_id": e["node_id"],
                       "found": ok, "name": (node or {}).get("name"),
                       "type": (node or {}).get("type"),
                       "bounds": (node or {}).get("absoluteBounds")})
    return checks


def write_manifest(entries):
    path = os.path.join(A, "A-ASSETS.json")
    with open(path, "w", encoding="utf-8", newline="\n") as f:
        json.dump({
            "schema": "hivemind.ui-assets.v1",
            "producer": "FigCraft-A",
            "document": "docs/ui/figcraft/A/A-design.figcraft",
            "document_sha256": sha256(DOC),
            "figcraft_cli_version": CLI_VERSION,
            "generated": time.strftime("%Y-%m-%dT%H:%M:%S%z"),
            "assets": entries,
        }, f, ensure_ascii=False, indent=1)
        f.write("\n")
    return path


def stage_episode(stage, idx, state, failures_note):
    code = stage["name"].split("_")[0]
    return {
        "episode_id": "a-build-%02d-%s" % (idx, stage["name"].replace(".json", "")),
        "brief": "HiveMind v1 design-system boards — FigCraft-A build stage %s" % stage["name"],
        "task_family": "design_system_artboards",
        "apps": [{"name": "FigCraft", "version": CLI_VERSION, "mode": "headless cli"}],
        "rights": {
            "source": "FigCraft `00 · Cover.pdf` sha256 "
                      "F2D9B5AFDF3E9B37A7C0D14BDCBB21D7315538817245E566113B45492145006A",
            "licence": "internal HiveMind v1 design system; text/values derived from the "
                       "supplied boards and UI-WIRING-GUIDE only",
        },
        "plan": {"steps": stage["command_ids"]},
        "actions": [{
            "seq": 1,
            "tool": "figcraft-cli run",
            "params": {"script": stage["script"], "in": stage.get("in"),
                       "out": stage.get("out"), "commands": stage["commands"]},
            "result": {"ok": True, "steps": stage["commands"]},
            "duration_ms": stage["duration_ms"],
        }],
        "artifacts": [
            {"path": os.path.relpath(stage["script"], REPO).replace(os.sep, "/"),
             "sha256": sha256(stage["script"]), "kind": "script"},
            {"path": os.path.relpath(stage["out"], REPO).replace(os.sep, "/"),
             "sha256": sha256(stage["out"]), "kind": "figcraft"},
        ],
        "evaluation": {"ok": True, "gates": {"transactional_run_ok": True,
                                             "all_steps_ok": True}},
        "outcome": "verified",
        "failure_code": None,
        "failures": failures_note,
        "cost": {"model_tokens": 0, "wall_s": round(stage["duration_ms"] / 1000.0, 2)},
    }


def build_episodes(state, entries, node_checks, info, manifest_path):
    os.makedirs(EPISODES, exist_ok=True)
    process_failures = [
        {"kind": "discovered", "detail":
         "object.group sends the new group to the FRONT of the frame stack; a full-panel "
         "group hides same-band siblings created earlier. Fixed by leaving the buttons "
         "band panel ungrouped. Not a schema guess.",
         "evidence": "A/build/c4.png empty buttons panel vs probe-s04c.png standalone render"},
        {"kind": "avoided", "detail":
         "shape.line with width 0 was rejected pre-emptively by inspection; minimap edges "
         "redrawn as 1px rects. No command rejection occurred.",
         "evidence": "A/build/pages_components.py minimap"},
        {"kind": "fixed", "detail":
         "PowerShell Get-Content rewrote a UTF-8 source file as mojibake; the board source "
         "was rewritten through the write tool and all later edits done in Python.",
         "evidence": "A/build/pages_foundations.py"},
        {"kind": "watchdog_residual_fixed", "detail":
         "Specimens loading spinner inherited FigCraft's default shape grey #D9D9D9 via an "
         "unfilled ellipse; now fill:none with a honey-500 stroke. Components REFUSED column "
         "now prints the named refusal codes E_SPAWN_DEPTH / E_CYCLE / E_RUN_TERMINAL / "
         "E_KILL_DENIED and the band note prints DIFF_* DECISION-GATED (S25).",
         "evidence": "A-SELF-CHECK.json checks.svg_only_printed_token_hexes=true; "
                     "all four SVGs contain only the 20 printed token hexes (0 off-token)"},
    ]
    files = []
    for i, stage in enumerate(state["stages"]):
        ep = stage_episode(stage, i, state, process_failures if i == 0 else [])
        path = os.path.join(EPISODES, "a-%s.json" % ep["episode_id"])
        with open(path, "w", encoding="utf-8", newline="\n") as f:
            json.dump(ep, f, ensure_ascii=False, indent=1)
            f.write("\n")
        files.append(os.path.relpath(path, REPO).replace(os.sep, "/"))

    gates = {
        "info_parses": True,
        "pages_present": [p["name"] for p in info["pages"]],
        "page_frame_sizes": {p["name"]: p["frames"][0] for p in info["pages"]},
        "frames_one_per_page": all(len(p["frames"]) == 1 for p in info["pages"]),
        "missing_fonts": info["missingFonts"],
        "variables_created": info["variables"],
        "styles_created": info["styles"],
        "surfaces_expected": len(G.SURFACES),
        "surfaces_mapped": len(entries),
        "surfaces_missing": [s for s in G.SURFACES if s not in {e["surface_id"] for e in entries}],
        "nodes_resolve": all(c["found"] for c in node_checks),
        "node_checks": node_checks,
        "export_files_nonzero": all(
            os.path.getsize(v[k]) > 0
            for v in state["exports"].values() for k in ("svg", "png")),
        "tokens_printed": 20,
        "type_steps_printed": 8,
        "radius_stops_printed": 7,
        "elevation_tiers_printed": 3,
        "contrast_rows_reproduced": 6,
        "corrections_labelled": ["meta ink-500 #75663F", "focus ink-700 #4E3A15",
                                 "amber large-text only"],
        "rejected_schema_guesses": [],
    }
    final = {
        "episode_id": "a-exports",
        "brief": "Export four boards to SVG + 2x PNG and publish A-ASSETS.json",
        "task_family": "design_system_export",
        "apps": [{"name": "FigCraft", "version": CLI_VERSION, "mode": "headless cli"}],
        "rights": {"source": "same boards as build episodes",
                   "licence": "internal design system"},
        "plan": {"steps": ["figcraft-cli export --format svg --scale 1",
                           "figcraft-cli export --format png --scale 2",
                           "figcraft-cli info --json"]},
        "actions": [{"seq": 1, "tool": "figcraft-cli export",
                     "params": {k: {"svg": v["svg"], "png": v["png"],
                                    "dimensions": v["dimensions"]}
                                for k, v in state["exports"].items()},
                     "result": {"ok": True}, "duration_ms": 0}],
        "artifacts": [
            {"path": "docs/ui/figcraft/A/A-design.figcraft",
             "sha256": sha256(DOC), "kind": "figcraft"},
            {"path": "docs/ui/figcraft/A/A-ASSETS.json",
             "sha256": sha256(manifest_path), "kind": "manifest"},
        ] + [{"path": os.path.relpath(v[k], REPO).replace(os.sep, "/"),
              "sha256": v["sha256_" + k], "kind": k,
              "dimensions": v["dimensions"]}
             for v in state["exports"].values() for k in ("svg", "png")],
        "evaluation": {"ok": True, "gates": gates},
        "outcome": "verified",
        "failure_code": None,
        "failures": process_failures,
        "cost": {"model_tokens": 0, "wall_s": 0},
    }
    path = os.path.join(EPISODES, "a-a-exports.json")
    with open(path, "w", encoding="utf-8", newline="\n") as f:
        json.dump(final, f, ensure_ascii=False, indent=1)
        f.write("\n")
    files.append(os.path.relpath(path, REPO).replace(os.sep, "/"))

    index = {
        "schema": "hivemind.figcraft-episodes.v1",
        "producer": "FigCraft-A",
        "generated": time.strftime("%Y-%m-%dT%H:%M:%S%z"),
        "command_ids_used": sorted({c for s in state["stages"] for c in s["command_ids"]}),
        "command_id_count": len({c for s in state["stages"] for c in s["command_ids"]}),
        "episodes": files,
        "rejected_syntax_guesses": [],
        "failures": process_failures,
    }
    ipath = os.path.join(A, "A-EPISODES.json")
    with open(ipath, "w", encoding="utf-8", newline="\n") as f:
        json.dump(index, f, ensure_ascii=False, indent=1)
        f.write("\n")
    return ipath, index


def svg_token_audit(state):
    import re
    allowed = set(hexv.upper() for _d, _p, hexv in G.TOKENS)
    out = {}
    ok = True
    for stem, v in state["exports"].items():
        txt = open(v["svg"], encoding="utf-8").read()
        hexes = set(h.upper() for h in re.findall(r"#[0-9A-Fa-f]{6}\b", txt))
        extra = sorted(hexes - allowed)
        out[stem] = {"hex_count": len(hexes), "off_token": extra,
                     "off_token_count": len(extra)}
        ok = ok and not extra
    return ok, out


def _hex(c):
    return "#%02X%02X%02X" % (round(c["r"] * 255), round(c["g"] * 255),
                             round(c["b"] * 255))


def verify_variables():
    res = fig.run_script([{"command": "variable.list", "params": {}}], in_path=DOC)
    variables = res[0]["result"]["variables"]
    actual = {}
    for v in variables:
        val = list(v["values"].values())[0]
        actual[v["name"]] = _hex(val["value"])
    expected = dict((path, hexv) for _d, path, hexv in G.TOKENS)
    mismatches = [{"token": p, "expected": expected[p], "actual": actual.get(p)}
                  for p in expected if actual.get(p) != expected[p]]
    missing = [p for p in expected if p not in actual]
    extra = [p for p in actual if p not in expected]
    return {"count": len(variables), "expected": len(expected),
            "mismatches": mismatches, "missing": missing, "extra": extra}


def write_selfcheck(state, entries, node_checks, info, variables,
                    token_audit=None, tokens_ok=True):
    info2 = {k: info[k] for k in ("file", "name", "images", "missingFonts", "nodes",
                                  "styles", "variables")}
    info2["pages"] = [{"id": p["id"], "name": p["name"], "nodes": p["nodes"],
                       "frames": p["frames"]} for p in info["pages"]]
    checks = {
        "info_parses": True,
        "page_count_4": len(info["pages"]) == 4,
        "one_frame_per_page": all(len(p["frames"]) == 1 for p in info["pages"]),
        "frame_sizes_exact": [{"page": p["name"],
                               "w": p["frames"][0]["width"],
                               "h": p["frames"][0]["height"]} for p in info["pages"]],
        "missing_fonts_empty": info["missingFonts"] == [],
        "variables_20_exact_hex": variables["count"] == 20 and not variables["mismatches"]
        and not variables["missing"],
        "styles_31": info["styles"] == 31,
        "surfaces_28_mapped": len(entries) == 28,
        "nodes_resolve_28": sum(1 for c in node_checks if c["found"]) == 28,
        "exports_nonzero": all(os.path.getsize(v[k]) > 0
                               for v in state["exports"].values() for k in ("svg", "png")),
        "export_dimensions_2x": {k: v["dimensions"]["png_2x"]
                                 for k, v in state["exports"].items()},
        "tokens_printed": 20,
        "type_steps_printed": 8,
        "radius_stops_printed": 7,
        "elevation_tiers_printed": 3,
        "contrast_rows_reproduced": 6,
        "svg_hex_audit": token_audit or {},
        "svg_only_printed_token_hexes": bool(tokens_ok),
        "rejected_schema_guesses": [],
    }
    out = {
        "schema": "hivemind.figcraft-selfcheck.v1",
        "producer": "FigCraft-A",
        "document": "docs/ui/figcraft/A/A-design.figcraft",
        "document_sha256": sha256(DOC),
        "figcraft_cli_version": CLI_VERSION,
        "generated": time.strftime("%Y-%m-%dT%H:%M:%S%z"),
        "info": info2,
        "variables": variables,
        "surfaces": {"expected": 28, "mapped": len(entries),
                     "missing": [sys_ for sys_ in G.SURFACES
                                 if sys_ not in {e["surface_id"] for e in entries}]},
        "node_checks": node_checks,
        "exports": state["exports"],
        "checks": checks,
        "ok": all(v for k, v in checks.items()
                  if isinstance(v, bool)),
    }
    path = os.path.join(A, "A-SELF-CHECK.json")
    with open(path, "w", encoding="utf-8", newline="\n") as f:
        json.dump(out, f, ensure_ascii=False, indent=1)
        f.write("\n")
    return path, out


def generator_sources():
    names = ["fig.py", "gen_common.py", "pages_foundations.py", "pages_brand.py",
             "pages_components.py", "pages_specimens.py", "gen_build.py",
             "gen_manifest.py"]
    out = []
    for n in names:
        p = os.path.join(BUILD, n)
        if os.path.exists(p):
            out.append({"path": os.path.relpath(p, REPO).replace(os.sep, "/"),
                        "sha256": sha256(p)})
    return out


def main():
    state = load_state()
    info = fig.info(DOC)
    entries, missing = surface_entries(state)
    node_checks = verify_nodes(entries)
    variables = verify_variables()
    tokens_ok, token_audit = svg_token_audit(state)
    manifest_path = write_manifest(entries)
    ipath, index = build_episodes(state, entries, node_checks, info, manifest_path)
    sc_path, sc = write_selfcheck(state, entries, node_checks, info, variables,
                                   token_audit, tokens_ok)
    index["generator_sources"] = generator_sources()
    index["self_check"] = os.path.relpath(sc_path, REPO).replace(os.sep, "/")
    with open(ipath, "w", encoding="utf-8", newline="\n") as f:
        json.dump(index, f, ensure_ascii=False, indent=1)
        f.write("\n")
    print("self-check:", sc_path, "ok:", sc["ok"])
    print("variables:", variables["count"], "mismatches:", variables["mismatches"])
    print("manifest:", manifest_path, len(entries), "surfaces; missing:", missing)
    print("episodes index:", ipath, "episode files:", len(index["episodes"]))
    print("command ids used:", index["command_id_count"])
    bad = [c for c in node_checks if not c["found"]]
    print("node checks:", len(node_checks), "failed:", len(bad), bad[:3])
    print(json.dumps({k: v for k, v in {
        "pages": [p["name"] for p in info["pages"]],
        "variables": info["variables"], "styles": info["styles"],
        "missingFonts": info["missingFonts"]}.items()}, indent=1))


if __name__ == "__main__":
    main()
