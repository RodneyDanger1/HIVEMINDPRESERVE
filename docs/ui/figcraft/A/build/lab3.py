"""Pattern lab 2: shadows, 999 radius, page bg, export-by-name, long group."""
import json
import os
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import fig

B = os.path.join(fig.ROOT, "build")


def main():
    cmds = [
        {"command": "file.new", "params": {"name": "lab3"}},
        {"command": "page.rename", "params": {"id": 1, "name": "00 Foundations"}},
        {"command": "frame.create", "params": {"name": "00 Foundations", "x": 0, "y": 0,
                                               "width": 400, "height": 300}},
        {"command": "page.setBackground", "params": {"id": 1, "color": "#F8F4EB"}},
        {"command": "node.set", "params": {"ids": [2], "fill": "#F8F4EB"}},
        {"command": "shape.rectangle", "params": {"x": 20, "y": 20, "width": 100, "height": 40,
                                                  "parent": 2, "name": "pill",
                                                  "props": {"fill": "#FBF3DC", "cornerRadius": 999}}},
        {"command": "shape.rectangle", "params": {"x": 20, "y": 80, "width": 100, "height": 60,
                                                  "parent": 2, "name": "shadow",
                                                  "props": {"fill": "#FFFFFF", "cornerRadius": 14}}},
        {"command": "effect.add", "params": {"ids": [4], "type": "dropShadow"}},
        {"command": "effect.set", "params": {"ids": [4], "index": 0, "effect": {
            "type": "dropShadow", "color": "#282010", "offset": [0, 8],
            "radius": 24, "spread": 0, "visible": True, "showBehind": False,
            "blend": "normal"}}},
        {"command": "text.create", "params": {
            "text": "Centered", "x": 20, "y": 160, "width": 200, "parent": 2,
            "name": "t-center", "props": {"fontSize": 17, "fontFamily": "Inter",
                                          "fontStyle": "Regular", "fill": "#282010",
                                          "textAlign": "center", "lineHeight": 28}}},
        {"command": "text.create", "params": {
            "text": "PLAYFAIR 96", "x": 20, "y": 210, "parent": 2,
            "name": "t-big", "props": {"fontSize": 96, "fontFamily": "Playfair Display",
                                       "fontStyle": "SemiBold", "fill": "#282010",
                                       "letterSpacing": "-3%", "lineHeight": 100}}},
        {"command": "node.get", "params": {"id": 4}},
        {"command": "node.get", "params": {"id": 5}},
        {"command": "page.list", "params": {}},
    ]
    out = os.path.join(B, "lab3.figcraft")
    results = fig.run_script(cmds, out_path=out)
    print("pg bg:", json.dumps(results[3]["result"], ensure_ascii=False))
    print("shadow:", json.dumps(results[11]["result"].get("effects"), ensure_ascii=False))
    print("pill:", json.dumps(results[12]["result"].get("cornerRadius"), ensure_ascii=False))
    print("pages:", json.dumps(results[13]["result"], ensure_ascii=False)[:400])
    # export by frame name
    svg = os.path.join(B, "lab3-by-name.svg")
    fig.export(out, svg, "svg", node="00 Foundations", scale=1)
    print("svg bytes:", os.path.getsize(svg))
    png = os.path.join(B, "lab3-by-name.png")
    fig.export(out, png, "png", node="00 Foundations", scale=2)
    from PIL import Image
    im = Image.open(png)
    print("png:", im.size, im.mode)
    print((open(svg, encoding="utf-8").read())[:300])


if __name__ == "__main__":
    main()
