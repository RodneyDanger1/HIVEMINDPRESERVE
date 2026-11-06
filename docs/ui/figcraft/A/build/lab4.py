"""Pattern lab 3: shadow alpha, 999 radius, dashes, rotation, wrapped text render."""
import json
import os
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import fig

B = os.path.join(fig.ROOT, "build")


def main():
    cmds = [
        {"command": "file.new", "params": {"name": "lab4"}},
        {"command": "page.rename", "params": {"id": 1, "name": "Wire"}},
        {"command": "frame.create", "params": {"name": "Wire", "x": 0, "y": 0,
                                               "width": 720, "height": 420}},
        {"command": "node.set", "params": {"ids": [2], "fill": "#F8F4EB"}},
        # A: effect color object with alpha
        {"command": "shape.rectangle", "params": {"x": 40, "y": 40, "width": 140, "height": 80,
                                                  "parent": 2, "name": "A",
                                                  "props": {"fill": "#FFFFFF", "cornerRadius": 12}}},
        {"command": "effect.add", "params": {"ids": [3], "type": "dropShadow"}},
        {"command": "effect.set", "params": {"ids": [3], "index": 0, "effect": {
            "type": "dropShadow", "color": {"r": 0.157, "g": 0.125, "b": 0.063, "a": 0.10},
            "offset": [0, 6], "radius": 18, "spread": 0, "visible": True}}},
        # B: hex8 alpha
        {"command": "shape.rectangle", "params": {"x": 220, "y": 40, "width": 140, "height": 80,
                                                  "parent": 2, "name": "B",
                                                  "props": {"fill": "#FFFFFF", "cornerRadius": 12}}},
        {"command": "effect.add", "params": {"ids": [4], "type": "dropShadow"}},
        {"command": "effect.set", "params": {"ids": [4], "index": 0, "effect": {
            "type": "dropShadow", "color": "#2820101A", "offset": [0, 6],
            "radius": 18, "spread": 0, "visible": True}}},
        # C: 999 radius pill
        {"command": "shape.rectangle", "params": {"x": 400, "y": 40, "width": 160, "height": 28,
                                                  "parent": 2, "name": "C-pill",
                                                  "props": {"fill": "#F0C050", "cornerRadius": 999}}},
        # D: dashed wire
        {"command": "shape.line", "params": {"x": 40, "y": 180, "width": 220, "parent": 2,
                                             "name": "D-wire",
                                             "props": {"stroke": "#B23A2E", "strokeWeight": 1.5,
                                                       "dashes": [6, 4]}}},
        # E: rotated polygon (hex)
        {"command": "shape.polygon", "params": {"x": 300, "y": 160, "width": 56, "height": 56,
                                                "parent": 2, "name": "E-hex",
                                                "props": {"pointCount": 6, "fill": "#E8B850",
                                                          "rotation": 30}}},
        # F: wrapped body text
        {"command": "text.create", "params": {
            "text": "Drop a repository into the core hive and the HyperAgent maps its architecture before a single worker spawns.",
            "x": 40, "y": 240, "width": 420, "parent": 2, "name": "F-body",
            "props": {"fontSize": 17, "fontFamily": "Inter", "fontStyle": "Regular",
                      "fill": "#282010", "lineHeight": 28, "textAlign": "left"}}},
        # G: 96px playfair
        {"command": "text.create", "params": {
            "text": "Aa 96", "x": 400, "y": 150, "parent": 2, "name": "G-big",
            "props": {"fontSize": 96, "fontFamily": "Playfair Display",
                      "fontStyle": "SemiBold", "fill": "#282010",
                      "letterSpacing": "-3%", "lineHeight": 100}}},
        # H: letter-spaced label
        {"command": "text.create", "params": {
            "text": "ACTIVE AGENTS", "x": 400, "y": 260, "parent": 2, "name": "H-label",
            "props": {"fontSize": 12, "fontFamily": "Inter", "fontStyle": "SemiBold",
                      "fill": "#75663F", "letterSpacing": "8%", "lineHeight": 16}}},
        {"command": "node.get", "params": {"id": 3}},
        {"command": "node.get", "params": {"id": 5}},
        {"command": "node.get", "params": {"id": 6}},
    ]
    out = os.path.join(B, "lab4.figcraft")
    results = fig.run_script(cmds, out_path=out)
    print("A effect:", json.dumps(results[15]["result"].get("effects"), ensure_ascii=False))
    print("C pill radius:", json.dumps(results[16]["result"].get("cornerRadius"), ensure_ascii=False))
    print("D dashes:", json.dumps(results[17]["result"].get("dashes"), ensure_ascii=False))
    png = os.path.join(B, "lab4.png")
    fig.export(out, png, "png", node="Wire", scale=2)
    svg = os.path.join(B, "lab4.svg")
    fig.export(out, svg, "svg", node="Wire", scale=1)
    print("png:", png, os.path.getsize(png))
    print("svg:", svg, os.path.getsize(svg))


if __name__ == "__main__":
    main()
