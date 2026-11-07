"""Build orchestrator for the FigCraft-C deliverable (HiveMind shell/HMAS/states).

Pipeline (each `run` is one transactional headless session):
  stage0 pages -> stage1 frames (3 x 1440x900) -> one content stage per board
  -> C-design.figcraft -> export SVG(1x)/PNG(2x) per frame -> build-state.json

No tag, no key, no absolute path in emitted manifests. A/B are never touched.
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
import fig_c as fig
from boards_common import C, C_SURFACES

C_ROOT = fig.C_ROOT
TMP = fig.TMP
SCRIPTS = fig.SCRIPTS
EPISODES = fig.EPISODES
ASSETS = fig.ASSETS
STATE = os.path.join(C_ROOT, "build", "build-state.json")

BOARDS = {
    "00": {"page": "HiveMind / shell", "frame": "c-shell",
           "module": "boards_c", "fn": "build_shell",
           "script": "02_c_shell.json", "stem": "c-shell"},
    "01": {"page": "HiveMind / hmas", "frame": "c-hmas",
           "module": "boards_c", "fn": "build_hmas",
           "script": "03_c_hmas.json", "stem": "c-hmas"},
    "02": {"page": "HiveMind / states", "frame": "c-states",
           "module": "boards_c", "fn": "build_states",
           "script": "04_c_states.json", "stem": "c-states"},
}
ORDER = ["00", "01", "02"]


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


def rel_to_figroot(path):
    return os.path.relpath(path, fig.FIG_ROOT).replace("\\", "/")


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
    rec = {"stage": name, "script": rel_to_figroot(script_path),
           "in": rel_to_figroot(in_path) if in_path else None,
           "out": rel_to_figroot(out_path), "commands": len(cmds),
           "command_ids": sorted(set(c["command"] for c in cmds)),
           "duration_ms": dur, "note": note, "failures": []}
    print("stage %-24s cmds=%-4d %6dms -> %s"
          % (name, len(cmds), dur, rel_to_figroot(out_path)))
    return rec, results


def stage0():
    cmds = [
        {"command": "file.new", "params": {"name": "HiveMind v1 - Design C"}},
        {"command": "page.rename", "params": {"id": 1, "name": BOARDS["00"]["page"]}},
        {"command": "page.add", "params": {"name": BOARDS["01"]["page"]}},
        {"command": "page.add", "params": {"name": BOARDS["02"]["page"]}},
        {"command": "page.list", "params": {}},
    ]
    out = os.path.join(TMP, "step0.figcraft")
    rec, res = run_stage("00_setup_pages.json", cmds, None, out,
                         "file.new + 3 pages (default page renamed)")
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
    for code, pid in zip(ORDER, page_ids):
        cmds += [
            {"command": "page.select", "params": {"id": pid}},
            {"command": "frame.create",
             "params": {"name": BOARDS[code]["frame"], "x": 0, "y": 0,
                        "width": 1440, "height": 900}},
            {"command": "node.set", "params": {"fill": C["canvas"]}},
            {"command": "page.setBackground", "params": {"id": pid, "color": C["canvas"]}},
        ]
    out = os.path.join(TMP, "step1.figcraft")
    rec, res = run_stage("01_frames.json", cmds, os.path.join(TMP, "step0.figcraft"),
                         out, "one 1440x900 frame per page + canvas background")
    frame_ids = [r["result"]["id"] for r in res if r["command"] == "frame.create"]
    if len(frame_ids) != 3:
        raise RuntimeError("expected 3 frames, got %s" % frame_ids)
    return rec, out, frame_ids


def export_board(doc, code, board):
    info = BOARDS[code]
    svg = os.path.join(ASSETS, info["stem"] + ".svg")
    png = os.path.join(ASSETS, info["stem"] + ".png")
    fig.export(doc, svg, "svg", node=info["frame"], scale=1)
    fig.export(doc, png, "png", node=info["frame"], scale=2)
    return {"svg": rel_to_figroot(svg), "png": rel_to_figroot(png),
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
    tool = fig.tool_info()
    state = {"tool": tool, "stages": [], "pages": {}, "boards": {},
             "exports": {}, "surfaces": {},
             "started": time.strftime("%Y-%m-%dT%H:%M:%S")}

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
        board = getattr(importlib.import_module(BOARDS[code]["module"]),
                        BOARDS[code]["fn"])(frame_ids[ORDER.index(code)])
        out = os.path.join(TMP, "board%s.figcraft" % code)
        rec, res = run_stage(BOARDS[code]["script"], board.cmd, prev, out,
                             "content: %s" % BOARDS[code]["page"])
        board.apply_ids(res)
        state["stages"].append(rec)
        boards[code] = board
        prev = out

    final_doc = os.path.join(C_ROOT, "C-design.figcraft")
    shutil.copyfile(prev, final_doc)
    state["file"] = rel_to_figroot(final_doc)
    state["file_sha256"] = sha256(final_doc)
    state["file_bytes"] = os.path.getsize(final_doc)
    if list(boards.keys()) == ORDER:
        for code, board in boards.items():
            state["exports"][code] = export_board(final_doc, code, board)
            print("exported %s %s" % (BOARDS[code]["stem"],
                                      state["exports"][code]["dimensions"]))
    for code, board in boards.items():
        names = [{"name": el["name"], "id": el["id"], "command": el["command"]}
                 for el in board.el]
        state["boards"][code] = {
            "page": BOARDS[code]["page"], "frame": BOARDS[code]["frame"],
            "frame_id": board.frame, "elements": len(board.el),
            "surfaces": list(board.surfaces.keys()),
            "element_names": dict((el["name"], el["id"]) for el in board.el),
            "elements_detail": names,
        }
        ids = dict((el["name"], el["id"]) for el in board.el)
        for sid, ename in board.surfaces.items():
            surface_row = C_SURFACES.get(sid, ("", "", ""))
            state["surfaces"].setdefault(sid, {
                "name": surface_row[0], "wiring": surface_row[1],
                "pdf_lines": surface_row[2],
                "page": BOARDS[code]["page"], "frame": BOARDS[code]["frame"],
                "node_name": ename, "node_id": ids.get(ename),
                "stem": BOARDS[code]["stem"],
            })
    state["command_total"] = sum(s["commands"] for s in state["stages"])
    state["command_ids"] = sorted(set(i for s in state["stages"]
                                      for i in s["command_ids"]))
    state["wall_ms"] = int((time.time() - t_start) * 1000)
    state["finished"] = time.strftime("%Y-%m-%dT%H:%M:%S")
    with open(STATE, "w", encoding="utf-8", newline="\n") as f:
        json.dump(state, f, ensure_ascii=False, indent=1)
        f.write("\n")
    print("state:", rel_to_figroot(STATE), "commands:", state["command_total"])
    print("design:", rel_to_figroot(final_doc), state["file_sha256"])


if __name__ == "__main__":
    main()
