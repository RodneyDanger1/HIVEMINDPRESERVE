"""Pattern lab: validate every primitive the real build depends on."""
import json
import os
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import fig

B = os.path.join(fig.ROOT, "build")


def main():
    cmds = [
        {"command": "file.new", "params": {"name": "lab"}},
        {"command": "page.rename", "params": {"id": 1, "name": "Test"}},
        {"command": "frame.create", "params": {"name": "Board", "x": 0, "y": 0,
                                               "width": 600, "height": 400}},
        {"command": "variable.createCollection", "params": {"name": "HiveMind A"}},
        {"command": "variable.create", "params": {"collection": "HiveMind A",
                                                  "name": "surface/canvas",
                                                  "type": "color", "value": "#F8F4EB"}},
        {"command": "variable.create", "params": {"collection": "HiveMind A",
                                                  "name": "honey/300",
                                                  "type": "color", "value": "#F0C050"}},
        {"command": "shape.rectangle", "params": {
            "x": 20, "y": 20, "width": 100, "height": 60, "parent": 2, "name": "s1",
            "props": {"fill": "#F0C050", "cornerRadius": 14,
                      "stroke": "#D9A02F", "strokeWeight": 1.5}}},
        {"command": "text.create", "params": {
            "text": "Test SemiBold", "x": 20, "y": 110, "parent": 2, "width": 300,
            "name": "t1", "props": {"fontSize": 13, "fontFamily": "Inter",
                                    "fontStyle": "SemiBold", "fill": "#282010",
                                    "letterSpacing": "8%", "lineHeight": 20,
                                    "textAlign": "left"}}},
        {"command": "shape.polygon", "params": {
            "x": 200, "y": 110, "width": 40, "height": 40, "parent": 2, "name": "hex",
            "props": {"pointCount": 6, "fill": "#E8B850"}}},
        {"command": "shape.line", "params": {
            "x": 250, "y": 120, "width": 80, "parent": 2, "name": "ln",
            "props": {"rotation": 0, "stroke": "#B23A2E", "strokeWeight": 1.5,
                      "dashes": [6, 4]}}},
        {"command": "vector.create", "params": {
            "path": "M0 0 L20 10 L0 20 Z", "x": 300, "y": 200, "parent": 2,
            "name": "v1", "props": {"fill": "#3F7A55"}}},
        {"command": "shape.rectangle", "params": {
            "x": 400, "y": 20, "width": 80, "height": 80, "parent": 2, "name": "fx",
            "props": {"fill": "#FFFFFF", "cornerRadius": 12}}},
        {"command": "effect.add", "params": {"ids": [8], "type": "dropShadow"}},
        {"command": "object.group", "params": {"ids": [3, 4], "name": "grp"}},
        {"command": "style.create", "params": {"type": "paint", "name": "honey/300",
                                               "from": 3}},
        {"command": "style.create", "params": {"type": "text", "name": "label",
                                               "from": 4}},
        {"command": "style.create", "params": {"type": "effect", "name": "e1/node",
                                               "from": 8}},
        {"command": "node.get", "params": {"id": 2}},
        {"command": "node.get", "params": {"id": 8}},
        {"command": "variable.list", "params": {}},
        {"command": "style.list", "params": {}},
    ]
    out = os.path.join(B, "lab1.figcraft")
    results = fig.run_script(cmds, out_path=out)
    for i, r in enumerate(results):
        res = r.get("result")
        if isinstance(res, dict) and res.get("id") is not None:
            print("step %2d created id=%s  cmd=%s" % (i, res["id"], cmds[i]["command"]))
    print("--- node.get frame:", json.dumps(results[17]["result"], ensure_ascii=False)[:600])
    print("--- node.get fx:", json.dumps(results[18]["result"], ensure_ascii=False)[:600])
    print("--- variables:", json.dumps(results[19]["result"], ensure_ascii=False)[:600])
    print("--- styles:", json.dumps(results[20]["result"], ensure_ascii=False)[:600])

    # second, risk-isolated run: ids on node.set / select.byName / selection group
    cmds2 = [
        {"command": "node.set", "params": {"ids": [5], "fill": "#123456"}},
        {"command": "select.byName", "params": {"query": "Board"}},
        {"command": "node.get", "params": {"id": 9}},
        {"command": "node.set", "params": {"fill": "#654321"}},
    ]
    out2 = os.path.join(B, "lab2.figcraft")
    try:
        results2 = fig.run_script(cmds2, in_path=out, out_path=out2)
        for i, r in enumerate(results2):
            print("lab2 step %d ok result=%s" % (i, json.dumps(r.get("result"), ensure_ascii=False)[:300]))
    except RuntimeError as exc:
        print("LAB2 FAILED:", exc)


if __name__ == "__main__":
    main()
