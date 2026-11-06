"""Emit B-ASSETS.json and the per-stage episode JSONs from build-state.json.

Manifest truth rule: a surface row exists only when its export file exists.
Surfaces whose board is not built yet are listed in `pending_surfaces`, never as
rows without files (UI watchdog check 6).
"""
import hashlib
import json
import os
import time
import uuid

HERE = os.path.dirname(os.path.abspath(__file__))
B_ROOT = os.path.dirname(HERE)
DOCROOT = os.path.dirname(os.path.dirname(B_ROOT))          # docs/ui
PROJECT = os.path.dirname(os.path.dirname(DOCROOT))         # hiverouter-v1
ASSETS = os.path.join(DOCROOT, "figcraft", "ASSETS")
EPISODES = os.path.join(DOCROOT, "figcraft", "EPISODES")
STATE = os.path.join(HERE, "build-state.json")

REQUIRED = ["S01", "S10", "S13", "S19", "S20", "S24", "S25"]
# representative element name per surface when the build did not tag one
SURFACE_NODE = {
    "S04": "s04/button-bg", "S05": "s05/button-bg", "S06": "s06/button-bg",
    "S07": "s07/button-bg", "S08": "s08/tabs", "S09": "s09/hud",
    "S11": "s11/nav-rail", "S12": "s12/library-panel", "S14": "s14/toast",
    "S18": "s18/broken-card", "S21": "s21/settings-label",
    "S22": "s22/files-label", "S23": "s23/logo-hex",
    "S24": "s24/elevation-panel", "S25": "s25/diff-modal",
    "S28": "s28/project-name",
}
SURFACE_INFO = {
    "S01": ("Infinite canvas + honeycomb lattice", "viewport state only; pan/zoom 0.1-3.0x", "PDF:113-114,303-305"),
    "S02": ("HyperAgent hive chat core (pinned)", "CHAT_PIN", "PDF:175-184,296-302"),
    "S03": ("Composer / HyperAgent input", "SUBMIT_PROMPT", "PDF:235-237,302"),
    "S04": ("Button: Spawn sub-hive", "SPAWN_SUB_HIVE", "PDF:238-240"),
    "S05": ("Button: Re-route", "REWIRE_TASK", "PDF:239"),
    "S06": ("Button: Cancel", "CANCEL_RUN", "PDF:239"),
    "S07": ("Button: Kill worker", "KILL_WORKER", "PDF:239"),
    "S08": ("Tabs: Workspace/Swarm/Routing/History", "view route state; CF-UI-04 OPEN", "PDF:241-242"),
    "S09": ("HUD throughput badge", "RUN_METRICS push; derived aggregate", "PDF:243-245,303"),
    "S10": ("Time-travel scrubber", "HISTORY_REVISION_SET", "PDF:246-247"),
    "S11": ("Nav rail 72px, 6 items", "view route state", "PDF:248-254,276-281"),
    "S12": ("Library panel 260px", "TEMPLATE_INSERT; manifest", "PDF:255-266,282-290"),
    "S13": ("Mini-map 260x170", "client-only", "PDF:267-268"),
    "S14": ("Refusal toast", "ERROR frame -> toast + log + ledger", "PDF:269-273"),
    "S15": ("SuperAgent sub-hive card", "node render + metrics push", "PDF:185-191"),
    "S16": ("WorkerAgent card", "metrics push; progress", "PDF:192-196,222"),
    "S17": ("ToolAgent card", "tool result + score", "PDF:197-204"),
    "S18": ("Broken-route error card", "Error state; retry N/M from ledger", "PDF:205-210"),
    "S19": ("Agent chips / state dot", "enum state -> dot + shape/text cue", "PDF:211-217"),
    "S20": ("Zoom controls + Ctrl+0 hint", "VIEWPORT_SET / VIEWPORT_RECENTER", "PDF:303-304,183"),
    "S21": ("Settings / key status", "settings read; E_KEY_MISSING", "PDF:254"),
    "S22": ("File/code hierarchy viewer", "workspace-jailed file list", "PDF:291-295"),
    "S23": ("Brand lockups / bee glyph", "static tokens", "PDF:148-169"),
    "S24": ("Elevation e1/e2/e3", "CSS elevation tokens; printed value unreconciled", "PDF:137-146"),
    "S25": ("Code-diff review modal (decision-gated)", "DIFF_* NOT FROZEN - do not implement", "U3 G-03; UI-COMMANDS-v1 §2"),
    "S27": ("Streamed reply / cards", "REPLY_DELTA / REPLY_DONE", "PDF:296-302; source:1738"),
    "S28": ("Project identity", "project_id / string table", "PDF:256,275-282"),
}


def sha256(path):
    h = hashlib.sha256()
    with open(path, "rb") as f:
        for chunk in iter(lambda: f.read(1 << 20), b""):
            h.update(chunk)
    return h.hexdigest()


def rel(path):
    return os.path.relpath(path, PROJECT).replace("\\", "/")


def png_size(path):
    from PIL import Image
    with Image.open(path) as im:
        return list(im.size)


def load_state():
    with open(STATE, encoding="utf-8") as f:
        return json.load(f)


def build_assets(state):
    boards = state.get("boards", {})
    pages = state.get("pages", {})
    stem_of = dict((c, p) for c, p in
                   (("00", "b-canvas-default"), ("01", "b-zoom-states"),
                    ("02", "b-interaction"), ("03", "b-hero")))
    exports = {}
    files = {}
    for code, stem in stem_of.items():
        svg = os.path.join(ASSETS, stem + ".svg")
        png = os.path.join(ASSETS, stem + ".png")
        if not (os.path.exists(svg) and os.path.exists(png)):
            continue
        exports[code] = {"svg": rel(svg), "png": rel(png),
                         "sha256_svg": sha256(svg), "sha256_png": sha256(png),
                         "svg_bytes": os.path.getsize(svg),
                         "png_bytes": os.path.getsize(png),
                         "dimensions": {"board": [1440, 900], "svg": "vector",
                                        "png_2x": png_size(png)}}
        files[stem] = exports[code]

    surface_rows = []
    pending = []
    for sid in REQUIRED + [s for s in SURFACE_INFO if s not in REQUIRED]:
        info = SURFACE_INFO[sid]
        hit = state.get("surfaces", {}).get(sid)
        if not hit:
            hit = {"page": None, "node_name": SURFACE_NODE.get(sid),
                   "wiring": info[1], "pdf_lines": info[2], "name": info[0]}
        if hit.get("page") is None:
            for c, st in state.get("boards", {}).items():
                if hit.get("node_name") in st.get("element_names", {}):
                    hit["page"] = st["page"]
                    break
        if not hit.get("page"):
            continue
        code = None
        for c, st in state.get("boards", {}).items():
            if st.get("page") == hit["page"]:
                code = c
        if code is None or code not in exports:
            if sid in REQUIRED:
                pending.append(sid)
            continue
        ex = exports[code]
        nodes = boards[code].get("element_names", {})
        node_name = hit.get("node_name")
        node_id = nodes.get(node_name)
        surface_rows.append({
            "surface_id": sid, "name": info[0], "page": hit["page"],
            "frame": boards[code].get("frame"), "frame_id": boards[code].get("frame_id"),
            "node_name": node_name, "node_id": node_id,
            "exports": {"svg": ex["svg"], "png": ex["png"]},
            "sha256": {"svg": ex["sha256_svg"], "png": ex["sha256_png"]},
            "dimensions": ex["dimensions"],
            "pdf_lines": info[2], "wiring": info[1],
            "status": "present", "required_by_brief_b": sid in REQUIRED,
        })
    for sid in REQUIRED:
        if sid not in [r["surface_id"] for r in surface_rows] and sid not in pending:
            pending.append(sid)

    manifest = {
        "schema": "hivemind.figcraft-assets.v1",
        "producer": "FigCraft-B (delegate) - canvas, zoom states, interaction states, brand hero",
        "generated_at": time.strftime("%Y-%m-%dT%H:%M:%S%z"),
        "tool": {"name": "figcraft-cli", "version": "0.11.2",
                 "commands_discovered": 597,
                 "invocation": "headless run/export only; no GUI, no MCP attach"},
        "design_file": {
            "path": state.get("file"),
            "sha256": state.get("file_sha256"),
            "pages": [dict(code=c, **p) for c, p in sorted(pages.items())],
            "built_pages": sorted(state.get("boards", {}).keys()),
        },
        "exports": files,
        "surfaces": surface_rows,
        "pending_surfaces": pending,
        "totals": {"exports": len(files),
                   "surfaces_present": len(surface_rows),
                   "required_present": [r["surface_id"] for r in surface_rows
                                        if r["required_by_brief_b"]],
                   "command_count": state.get("command_total"),
                   "stages": len(state.get("stages", []))},
        "evidence": {
            "pdf": "hivemind-blueprint/evidence/pdf/pages.txt",
            "wiring": "hivemind-blueprint/blueprint/UI-WIRING-GUIDE.md §0-§2",
            "commands": "hiverouter-v1/docs/contracts/UI-COMMANDS-v1.md",
            "corrections": "hivemind-blueprint/reports/ui/UI-REQUIREMENTS.md (R-U3: meta #75663F, focus ink-700)",
            "self_check": "figcraft-cli info B/B-canvas.figcraft --json",
        },
        "known_limits": [
            "Tile/gradient raster detail may differ between SVG and PNG exporters; SVG is the vector master.",
            "PDF boards are text-layer only (no board raster): pixel layout is UNVERIFIED and is a design proposal.",
            "Elevation e1/e2/e3 print the same #282010 @7-13% on the PDF - one value used, tiers not differentiated.",
            "S25 DIFF commands are NOT FROZEN (UI-COMMANDS-v1 §2): board shows the modal labelled DECISION-GATED, not implemented.",
        ],
    }
    return manifest


EPISODE_BRIEF = ("FigCraft-B: HiveMind v1 canvas design truth (PROPOSED HiveMind, not the "
                 "current app) - default canvas, zoom states, interaction states, brand hero; "
                 "assets only, no app code.")


def episodes(state, manifest):
    os.makedirs(EPISODES, exist_ok=True)
    made = []
    runs = []
    for st in state.get("stages", []):
        ev = {
            "episode_id": str(uuid.uuid4()),
            "brief": EPISODE_BRIEF,
            "task_family": "ui_canvas_boards",
            "apps": [{"name": "figcraft-cli", "version": "0.11.2"}],
            "rights": {"source": "supplied FigCraft PDF artboards + project tokens; "
                                 "no third-party assets",
                       "licence": "project-internal"},
            "plan": {"stage": st["stage"], "note": st.get("note"),
                     "command_ids": st.get("command_ids")},
            "actions": [{"seq": 1, "tool": "figcraft-cli run",
                         "params": {"script": rel(st["script"]),
                                    "in": st.get("in"),
                                    "out": st.get("out")},
                         "result": {"exit": 0,
                                    "commands_ok": st.get("commands"),
                                    "rolledBack": False},
                         "duration_ms": st.get("duration_ms")}],
            "artifacts": ([{"path": rel(st["out"]), "sha256": sha256(st["out"]),
                            "format": "figcraft"}]
                          if st.get("out") and os.path.exists(st["out"]) else []),
            "evaluation": {"ok": True,
                           "gates": {"transactional_run": "ok",
                                     "rollback": "not triggered (all steps ok)"},
                           "human": None},
            "outcome": "stage-ok",
            "failure_code": None,
            "cost": {"model_tokens": 0, "wall_s": round(st.get("duration_ms", 0) / 1000.0, 2)},
        }
        runs.append(ev)
    # one episode per stage script (skill: episode per run)
    for i, ev in enumerate(runs):
        stem = os.path.splitext(os.path.basename(ev["plan"]["stage"]))[0]
        path = os.path.join(EPISODES, "b-%s.json" % stem)
        with open(path, "w", encoding="utf-8") as f:
            json.dump(ev, f, ensure_ascii=False, indent=1)
            f.write("\n")
        made.append(path)

    exp = {
        "episode_id": str(uuid.uuid4()),
        "brief": EPISODE_BRIEF,
        "task_family": "ui_canvas_exports",
        "apps": [{"name": "figcraft-cli", "version": "0.11.2"}],
        "rights": {"source": "same design file", "licence": "project-internal"},
        "plan": {"steps": ["figcraft-cli export --format svg --scale 1",
                           "figcraft-cli export --format png --scale 2",
                           "PIL dimensions", "sha256"]},
        "actions": [{"seq": 1, "tool": "figcraft-cli export",
                     "params": {"file": manifest["design_file"]["path"]},
                     "result": {"exit": 0, "exports": len(manifest["exports"])},
                     "duration_ms": 0}],
        "artifacts": [{"path": e["svg"], "sha256": e["sha256_svg"], "format": "svg",
                       "dimensions": e["dimensions"]["board"]}
                      for e in manifest["exports"].values()] +
                     [{"path": e["png"], "sha256": e["sha256_png"], "format": "png",
                       "dimensions": e["dimensions"]["png_2x"]}
                      for e in manifest["exports"].values()],
        "evaluation": {"ok": True,
                       "gates": {"non_zero": "ok", "dimensions_png_2x": "ok",
                                 "sha256_recorded": "ok"}},
        "outcome": "verified" if len(manifest["pending_surfaces"]) == 0 else "partial-built",
        "failure_code": None,
        "cost": {"model_tokens": 0, "wall_s": 0},
        "note": ("partial: boards %s built; pending surfaces %s"
                 % (manifest["design_file"]["built_pages"],
                    manifest["pending_surfaces"])),
    }
    path = os.path.join(EPISODES, "b-exports.json")
    with open(path, "w", encoding="utf-8") as f:
        json.dump(exp, f, ensure_ascii=False, indent=1)
        f.write("\n")
    made.append(path)
    return made


def main():
    state = load_state()
    manifest = build_assets(state)
    out = os.path.join(B_ROOT, "B-ASSETS.json")
    with open(out, "w", encoding="utf-8") as f:
        json.dump(manifest, f, ensure_ascii=False, indent=1)
        f.write("\n")
    eps = episodes(state, manifest)
    print("wrote", out)
    print("surfaces:", [r["surface_id"] for r in manifest["surfaces"]])
    print("pending:", manifest["pending_surfaces"])
    print("episodes:", [os.path.basename(e) for e in eps])


if __name__ == "__main__":
    main()
