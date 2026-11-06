"""Build orchestrator: runs the deliverable stage scripts through figcraft-cli.

Each stage script is written to A/scripts/<name>.json and executed with the
documented `run` subcommand (transactional). Real node ids are read back from the
per-step JSON results, so no id is ever predicted. Exports use `export --node`.
"""
import hashlib
import importlib
import json
import os
import sys
import time

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, HERE)
import fig
import gen_common as G
import pages_foundations as PF

A = fig.ROOT
SCRIPTS = os.path.join(A, "scripts")
BUILD = os.path.join(A, "build")
ASSETS = os.path.join(os.path.dirname(A), "ASSETS")
STATE = os.path.join(BUILD, "build-state.json")

PAGE_MODS = {
    "00": ("pages_foundations", "build_foundations", "02_foundations.json",
           "03_foundations_groups.json", "a-foundations"),
    "01": ("pages_brand", "build_brand", "05_brand.json",
           "06_brand_groups.json", "a-brand"),
    "02": ("pages_components", "build_components", "07_components_a.json",
           "09_components_groups.json", "a-components"),
    "03": ("pages_specimens", "build_specimens", "10_specimens.json",
           "11_specimens_groups.json", "a-specimens"),
}


def write_script(name, cmds):
    path = os.path.join(SCRIPTS, name)
    with open(path, "w", encoding="utf-8", newline="\n") as f:
        json.dump(cmds, f, ensure_ascii=False, indent=1)
        f.write("\n")
    return path


def run_stage(name, cmds, in_path, out_path):
    script_path = write_script(name, cmds)
    t0 = time.time()
    results = fig.run_script(cmds, in_path=in_path, out_path=out_path)
    rec = {"name": name, "script": script_path, "in": in_path, "out": out_path,
           "commands": len(cmds), "command_ids": sorted(set(c["command"] for c in cmds)),
           "duration_ms": int((time.time() - t0) * 1000), "failures": []}
    return rec, results


def apply_created_ids(board, results):
    assert len(results) == len(board.el), (len(results), len(board.el))
    for el, res in zip(board.el, results):
        rid = (res.get("result") or {}).get("id")
        if rid is None:
            raise RuntimeError("no id returned for %s" % el["name"])
        el["id"] = rid
    return board.el


def group_commands(board):
    cmds, specs = [], []
    for gname, idxs in board.groups.items():
        ids = [board.el[i]["id"] for i in idxs]
        if len(ids) >= 2:
            cmds.append({"command": "object.group",
                         "params": {"ids": ids, "name": gname}})
            specs.append((gname, "multi", idxs))
        else:
            specs.append((gname, "single", idxs))
    return cmds, specs


def apply_group_ids(board, specs, results):
    it = iter(results)
    for gname, kind, idxs in specs:
        if kind == "multi":
            res = next(it)
            gid = (res.get("result") or {}).get("id")
            if gid is None:
                raise RuntimeError("no group id for %s" % gname)
            board.group_ids[gname] = gid
        else:
            board.group_ids[gname] = board.el[idxs[0]]["id"]


def style_commands(boards):
    """20 paint styles + 8 text styles + 3 effect styles, created from real nodes."""
    cmds = []
    for b in boards:
        for name, idx in b.paint_styles:
            cmds.append({"command": "style.create",
                         "params": {"type": "paint", "name": name,
                                    "from": b.el[idx]["id"]}})
        for name, idx in b.text_styles:
            cmds.append({"command": "style.create",
                         "params": {"type": "text", "name": name,
                                    "from": b.el[idx]["id"]}})
        for name, idx in b.effect_styles:
            cmds.append({"command": "style.create",
                         "params": {"type": "effect", "name": name,
                                    "from": b.el[idx]["id"]}})
    return cmds


def effect_commands(board):
    """Warm-ink shadow: printed #282010 @7-13% (single printed value, unresolved)."""
    cmds = []
    for name, idx in board.effect_styles:
        nid = board.el[idx]["id"]
        cmds.append({"command": "effect.add",
                     "params": {"ids": [nid], "type": "dropShadow"}})
        cmds.append({"command": "effect.set",
                     "params": {"ids": [nid], "index": 0, "effect": {
                         "type": "dropShadow",
                         "color": {"r": 0.157, "g": 0.125, "b": 0.063, "a": 0.10},
                         "offset": [0, 8], "radius": 24, "spread": 0,
                         "visible": True, "showBehind": False, "blend": "normal"}}})
    return cmds


def export_page(doc, frame_name, stem):
    svg = os.path.join(ASSETS, stem + ".svg")
    png = os.path.join(ASSETS, stem + ".png")
    fig.export(doc, svg, "svg", node=frame_name, scale=1)
    fig.export(doc, png, "png", node=frame_name, scale=2)
    return svg, png


def sha256(path):
    h = hashlib.sha256()
    with open(path, "rb") as f:
        for chunk in iter(lambda: f.read(1 << 20), b""):
            h.update(chunk)
    return h.hexdigest()


def sizes_for(board):
    return list(board.surfaces.keys())


def main():
    only = sys.argv[1:] or None
    want = [c for c in (only or ["00", "01", "02", "03"])]
    os.makedirs(SCRIPTS, exist_ok=True)
    os.makedirs(ASSETS, exist_ok=True)
    state = {"stages": [], "pages": {}, "exports": {}, "surfaces": {},
             "boards": {}, "started": time.time()}

    st0 = os.path.join(BUILD, "stage0.figcraft")
    st1 = os.path.join(BUILD, "stage1.figcraft")

    # ---- stage 0: pages + 20 colour variables -----------------------------
    cmds = PF.setup_a_commands()
    cmds.append({"command": "page.list", "params": {}})
    rec0, res0 = run_stage("00_setup_pages.json", cmds, None, st0)
    state["stages"].append(rec0)
    pages = res0[-1]["result"]
    if isinstance(pages, dict):
        pages = pages.get("pages", [])
    pages = sorted(pages, key=lambda p: p["id"])
    page_ids = [p["id"] for p in pages]
    if [p["name"] for p in pages] != [n for n, _w, _h in PF.PAGES]:
        raise RuntimeError("page names mismatch: %s" % pages)
    state["pages"] = dict((p["name"], {"page_id": p["id"]}) for p in pages)

    # ---- stage 1: frames ---------------------------------------------------
    cmds = PF.setup_b_commands(page_ids)
    rec1, res1 = run_stage("01_frames.json", cmds, st0, st1)
    state["stages"].append(rec1)
    frame_ids = [r["result"]["id"] for r in res1
                 if r["command"] == "frame.create"]
    if len(frame_ids) != 4:
        raise RuntimeError("expected 4 frames, got %s" % frame_ids)
    for (name, w, h), nid in zip(PF.PAGES, frame_ids):
        state["pages"][name].update({"frame_id": nid, "width": w, "height": h})

    prev = st1
    boards = {}
    for code, (mod_name, fn_name, create_stage, group_stage, stem) in PAGE_MODS.items():
        if code not in want:
            continue
        mod = importlib.import_module(mod_name)
        board = getattr(mod, fn_name)(frame_ids[int(code)])
        board.group_ids = {}
        out_create = os.path.join(BUILD, "page%s.figcraft" % code)
        rec, res = run_stage(create_stage, board.cmd, prev, out_create)
        state["stages"].append(rec)
        apply_created_ids(board, res)
        prev = out_create

        gcmds, gspecs = group_commands(board)
        out_group = os.path.join(BUILD, "page%sg.figcraft" % code)
        recg, resg = run_stage(group_stage, gcmds, prev, out_group)
        state["stages"].append(recg)
        apply_group_ids(board, gspecs, resg)
        prev = out_group

        if code == "00":
            scmds = effect_commands(board) + style_commands([board])
            out_styles = os.path.join(BUILD, "styles.figcraft")
            recs, ress = run_stage("04_styles.json", scmds, prev, out_styles)
            state["stages"].append(recs)
            state["styles"] = {"count": len(scmds) - 6,
                               "paint": len(board.paint_styles),
                               "text": len(board.text_styles),
                               "effect": len(board.effect_styles)}
            prev = out_styles

        boards[code] = board
        state["boards"][code] = {
            "page": board.title, "frame_id": board.frame,
            "elements": len(board.el), "groups": len(board.groups),
            "surfaces": list(board.surfaces.keys()),
            "group_ids": board.group_ids,
            "element_names": dict((el["name"], el["id"]) for el in board.el),
            "element_surface": dict((sid, board.el[i]["name"])
                                    for sid, i in board.surfaces.items()),
        }

        # export after the last requested page
        if code == want[-1]:
            final_doc = prev
            if len(want) == 4:
                final_copy = os.path.join(A, "A-design.figcraft")
                import shutil
                shutil.copyfile(final_doc, final_copy)
                state["final_doc"] = final_copy
            for c2 in want:
                b2 = boards.get(c2)
                if b2 is None:
                    continue
                frame = PF.PAGES[int(c2)][0]
                stem2 = PAGE_MODS[c2][4]
                svg, png = export_page(final_doc, frame, stem2)
                from PIL import Image
                im = Image.open(png)
                state["exports"][stem2] = {
                    "svg": svg, "png": png, "sha256_svg": sha256(svg),
                    "sha256_png": sha256(png),
                    "dimensions": {"board": [b2.w, b2.h], "png_2x": list(im.size),
                                   "svg": "vector"},
                    "svg_bytes": os.path.getsize(svg),
                    "png_bytes": os.path.getsize(png),
                }
                print("exported", stem2, im.size, os.path.getsize(svg),
                      os.path.getsize(png))

    with open(STATE, "w", encoding="utf-8") as f:
        json.dump(state, f, ensure_ascii=False, indent=1, default=str)
    print("state:", STATE)
    print(json.dumps({"final_doc": state.get("final_doc"),
                      "exports": list(state["exports"])}, indent=1))


if __name__ == "__main__":
    main()
