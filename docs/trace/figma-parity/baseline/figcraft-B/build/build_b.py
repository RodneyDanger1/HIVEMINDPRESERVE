"""Build orchestrator for the FigCraft-B canvas deliverable.

Pipeline (each `run` is one transactional headless session):
  stage0 pages -> stage1 frames -> one content stage per board -> effects stage
  -> export SVG(1x)/PNG(2x) per frame name -> B-canvas.figcraft + B-ASSETS.json

Ids: frames report their ids; content nodes report their ids; the effects stage
uses those real ids. Nothing is predicted and no command id is invented.
"""
import hashlib
import importlib
import json
import os
import shutil
import sys
import time

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, HERE)
import fig_b as fig
from boards_common import C, SHADOW_INK, SURFACES_B

B_ROOT = fig.ROOT
TMP = os.path.join(B_ROOT, "build", "tmp")
SCRIPTS = os.path.join(B_ROOT, "scripts")
ASSETS = os.path.abspath(os.path.join(B_ROOT, "..", "ASSETS"))
EPISODES = os.path.abspath(os.path.join(B_ROOT, "..", "EPISODES"))
STATE = os.path.join(B_ROOT, "build", "build-state.json")

BOARDS = {
    "00": {"page": "Canvas / default", "frame": "b-canvas-default",
           "module": "board_default", "fn": "build_default",
           "script": "02_canvas_default.json", "stem": "b-canvas-default"},
    "01": {"page": "Canvas / zoom states", "frame": "b-zoom-states",
           "module": "board_zoom", "fn": "build_zoom",
           "script": "03_zoom_states.json", "stem": "b-zoom-states"},
    "02": {"page": "Canvas / interaction states", "frame": "b-interaction",
           "module": "board_interaction", "fn": "build_interaction",
           "script": "04_interaction.json", "stem": "b-interaction"},
    "03": {"page": "Brand hero", "frame": "b-hero",
           "module": "board_hero", "fn": "build_hero",
           "script": "05_hero.json", "stem": "b-hero"},
}
ORDER = ["00", "01", "02", "03"]


def sha256(path):
    h = hashlib.sha256()
    with open(path, "rb") as f:
        for chunk in iter(lambda: f.read(1 << 20), b""):
            h.update(chunk)
    return h.hexdigest()


def png_size(path):
    from PIL import Image
    with Image.open(path) as im:
        return list(im.size)


def write_script(name, cmds):
    os.makedirs(SCRIPTS, exist_ok=True)
    path = os.path.join(SCRIPTS, name)
    with open(path, "w", encoding="utf-8", newline="\n") as f:
        json.dump(cmds, f, ensure_ascii=False, indent=1)
        f.write("\n")
    return path


def run_stage(name, cmds, in_path, out_path, note):
    script_path = write_script(name, cmds)
    t0 = time.time()
    results = fig.run_script(cmds, in_path=in_path, out_path=out_path)
    dur = int((time.time() - t0) * 1000)
    rec = {"stage": name, "script": script_path, "in": in_path, "out": out_path,
           "commands": len(cmds),
           "command_ids": sorted(set(c["command"] for c in cmds)),
           "duration_ms": dur, "note": note, "failures": []}
    print("stage %-28s cmds=%-4d %5dms -> %s" % (name, len(cmds), dur, out_path))
    return rec, results


def stage0():
    cmds = [
        {"command": "file.new", "params": {"name": "HiveMind v1 - Canvas (B)"}},
        {"command": "page.rename", "params": {"id": 1,
                                              "name": BOARDS["00"]["page"]}},
        {"command": "page.add", "params": {"name": BOARDS["01"]["page"]}},
        {"command": "page.add", "params": {"name": BOARDS["02"]["page"]}},
        {"command": "page.add", "params": {"name": BOARDS["03"]["page"]}},
        {"command": "page.list", "params": {}},
    ]
    out = os.path.join(TMP, "step0.figcraft")
    rec, res = run_stage("00_setup_pages.json", cmds, None, out,
                         "file.new + 4 pages (default page renamed)")
    pages = res[-1]["result"]
    if isinstance(pages, dict):
        pages = pages.get("pages", [])
    pages = sorted(pages, key=lambda p: p["id"])
    want = [BOARDS[c]["page"] for c in ORDER]
    got = [p["name"] for p in pages]
    if got != want:
        raise RuntimeError("page names mismatch: %s" % got)
    return rec, out, [p["id"] for p in pages]


def stage1(page_ids):
    cmds = []
    for pid in page_ids:
        cmds.append({"command": "page.select", "params": {"id": pid}})
        cmds.append({"command": "frame.create",
                     "params": {"name": BOARDS[ORDER[page_ids.index(pid)]]["frame"],
                                "x": 0, "y": 0, "width": 1440, "height": 900}})
        cmds.append({"command": "node.set", "params": {"fill": C["canvas"]}})
        cmds.append({"command": "page.setBackground",
                     "params": {"id": pid, "color": C["canvas"]}})
    out = os.path.join(TMP, "step1.figcraft")
    rec, res = run_stage("01_frames.json", cmds, os.path.join(TMP, "step0.figcraft"),
                         out, "one 1440x900 frame per page + canvas background")
    frame_ids = [r["result"]["id"] for r in res
                 if r["command"] == "frame.create"]
    if len(frame_ids) != 4:
        raise RuntimeError("expected 4 frames, got %s" % frame_ids)
    return rec, out, frame_ids


def effects_commands(boards):
    cmds = []
    for code, board in boards.items():
        for idx, tier in board.effects:
            nid = board.el[idx]["id"]
            cmds.append({"command": "effect.add",
                         "params": {"ids": [nid], "type": "dropShadow"}})
            cmds.append({"command": "effect.set",
                         "params": {"ids": [nid], "index": 0, "effect": {
                             "type": "dropShadow", "color": dict(SHADOW_INK),
                             "offset": [0, 8], "radius": 24, "spread": 0,
                             "visible": True, "showBehind": False,
                             "blend": "normal"}}})
    return cmds


def export_board(doc, code, board):
    info = BOARDS[code]
    svg = os.path.join(ASSETS, info["stem"] + ".svg")
    png = os.path.join(ASSETS, info["stem"] + ".png")
    fig.export(doc, svg, "svg", node=info["frame"], scale=1)
    fig.export(doc, png, "png", node=info["frame"], scale=2)
    return {"svg": os.path.relpath(svg, os.path.dirname(B_ROOT)),
            "png": os.path.relpath(png, os.path.dirname(B_ROOT)),
            "sha256_svg": sha256(svg), "sha256_png": sha256(png),
            "svg_bytes": os.path.getsize(svg), "png_bytes": os.path.getsize(png),
            "dimensions": {"board": [board.w, board.h], "svg": "vector",
                           "png_2x": png_size(png)}}


def main():
    want = [c for c in (sys.argv[1:] or ORDER) if c in BOARDS]
    os.makedirs(TMP, exist_ok=True)
    os.makedirs(ASSETS, exist_ok=True)
    os.makedirs(EPISODES, exist_ok=True)
    t_start = time.time()
    state = {"tool": {"name": "figcraft-cli", "version": "0.11.2",
                      "commands": 597},
             "stages": [], "pages": {}, "boards": {}, "exports": {},
             "surfaces": {}, "started": time.strftime("%Y-%m-%dT%H:%M:%S")}

    rec0, prev, page_ids = stage0()
    state["stages"].append(rec0)
    rec1, prev, frame_ids = stage1(page_ids)
    state["stages"].append(rec1)
    for i, code in enumerate(ORDER):
        state["pages"][code] = {"page": BOARDS[code]["page"],
                               "page_id": page_ids[i],
                               "frame": BOARDS[code]["frame"],
                               "frame_id": frame_ids[i], "size": [1440, 900]}

    boards = {}
    for code in ORDER:
        if code not in want:
            continue
        mod = importlib.import_module(BOARDS[code]["module"])
        board = getattr(mod, BOARDS[code]["fn"])(frame_ids[ORDER.index(code)])
        out = os.path.join(TMP, "board%s.figcraft" % code)
        rec, res = run_stage(BOARDS[code]["script"], board.cmd, prev, out,
                             "content: %s" % BOARDS[code]["page"])
        board.apply_ids(res)
        state["stages"].append(rec)
        boards[code] = board
        prev = out

    # effects stage (real ids from the content stages)
    cmds = effects_commands(boards)
    if cmds:
        out = os.path.join(TMP, "effects.figcraft")
        rec, _ = run_stage("06_effects.json", cmds, prev, out,
                           "warm-ink e1/e2/e3 shadows (printed #282010 @7-13%)")
        state["stages"].append(rec)
        prev = out

    # final document + exports
    final_doc = os.path.join(B_ROOT, "B-canvas.figcraft")
    shutil.copyfile(prev, final_doc)
    state["file"] = os.path.relpath(final_doc, os.path.dirname(B_ROOT))
    state["file_sha256"] = sha256(final_doc)
    if list(boards.keys()) == ORDER:
        for code, board in boards.items():
            state["exports"][code] = export_board(final_doc, code, board)
            print("exported %s %s" % (BOARDS[code]["stem"],
                                      state["exports"][code]["dimensions"]))
    for code, board in boards.items():
        state["boards"][code] = {
            "page": board.title, "frame": BOARDS[code]["frame"],
            "frame_id": board.frame, "elements": len(board.el),
            "effects": len(board.effects),
            "surfaces": list(board.surfaces.keys()),
            "element_names": dict((el["name"], el["id"]) for el in board.el),
        }
        for sid, ename in board.surfaces.items():
            eid = dict((el["name"], el["id"]) for el in board.el).get(ename)
            state["surfaces"].setdefault(sid, {
                "name": SURFACES_B.get(sid, ("", "", ""))[0],
                "wiring": SURFACES_B.get(sid, ("", "", ""))[1],
                "pdf_lines": SURFACES_B.get(sid, ("", "", ""))[2],
                "page": BOARDS[code]["page"], "node_name": ename,
                "node_id": eid, "stem": BOARDS[code]["stem"]})

    state["command_total"] = sum(s["commands"] for s in state["stages"])
    state["wall_ms"] = int((time.time() - t_start) * 1000)
    with open(STATE, "w", encoding="utf-8") as f:
        json.dump(state, f, ensure_ascii=False, indent=1)
    print("state:", STATE, "commands:", state["command_total"])


if __name__ == "__main__":
    main()
