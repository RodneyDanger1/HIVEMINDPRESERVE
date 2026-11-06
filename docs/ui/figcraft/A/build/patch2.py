import io

# ---------------------------------------------------------------- gen_common.py
pg = r"C:\Users\Riley\Desktop\hiverouter-v1\docs\ui\figcraft\A\build\gen_common.py"
g = io.open(pg, encoding="utf-8").read()

old_rect = '''        props = {}
        if fill is not None:
            props["fill"] = fill
        if r:
            props["cornerRadius"] = r'''
new_rect = '''        props = {}
        if fill is not None:
            props["fill"] = fill
        elif stroke:
            props["fill"] = "none"
        if r:
            props["cornerRadius"] = r'''
assert old_rect in g
g = g.replace(old_rect, new_rect)

start = g.index("    def hex_bee(")
end = g.index("    def button(")
new_hex = '''    def hex_bee(self, x, y, s, hex_fill, bee_body, bee_head, wing, stripe=None,
                mono=False, group=None, name="mark", bee=True):
        """Hexagon mark (pointy-top) with an inner bee flying right (+x)."""
        self.poly(x, y, s, s, hex_fill, pointCount=6, group=group, name=name + "/hex")
        if not bee or s < 20:
            return
        u = s / 56.0
        cx, cy = x + s * 0.5, y + s * 0.5
        # wings sit behind the body, angled up and forward
        self.ellipse(cx - 7 * u, cy - 15 * u, 13 * u, 7 * u, fill=wing,
                     rotation=-14, group=group, name=name + "/wing-l")
        self.ellipse(cx + 3 * u, cy - 16 * u, 12 * u, 7 * u, fill=wing,
                     rotation=16, group=group, name=name + "/wing-r")
        # stinger trails the body (-x): the bee is a routing arrow flying +x
        self.poly(cx - 20 * u, cy - 2.5 * u, 7 * u, 5 * u, bee_head,
                  pointCount=3, rotation=270, group=group, name=name + "/stinger")
        # abdomen, bands, thorax, head
        self.ellipse(cx - 17 * u, cy - 6.5 * u, 18 * u, 13 * u, fill=bee_body,
                     group=group, name=name + "/abdomen")
        if stripe and not mono:
            self.rect(cx - 14.5 * u, cy - 6 * u, 2.0 * u, 12 * u, fill=stripe,
                      group=group, name=name + "/stripe-1")
            self.rect(cx - 11.0 * u, cy - 6 * u, 2.0 * u, 12 * u, fill=stripe,
                      group=group, name=name + "/stripe-2")
        self.ellipse(cx - 2.5 * u, cy - 5 * u, 10 * u, 10 * u, fill=bee_head,
                     group=group, name=name + "/thorax")
        self.ellipse(cx + 5.5 * u, cy - 4 * u, 8 * u, 8 * u, fill=bee_head,
                     group=group, name=name + "/head")
        self.line(cx + 8 * u, cy - 5 * u, 6 * u, bee_head, 1.2 * u, rotation=-42,
                  group=group, name=name + "/antenna-1")
        self.line(cx + 9 * u, cy - 3 * u, 5 * u, bee_head, 1.2 * u, rotation=-10,
                  group=group, name=name + "/antenna-2")

'''
g = g[:start] + new_hex + g[end:]
io.open(pg, "w", encoding="utf-8", newline="\n").write(g)

# ---------------------------------------------------------------- pages_brand.py
p = r"C:\Users\Riley\Desktop\hiverouter-v1\docs\ui\figcraft\A\build\pages_brand.py"
s = io.open(p, encoding="utf-8").read()

start = s.index("def bee_side(")
end = s.index("def lockup(")
new_bee = '''def bee_side(b, bx, by, group="01-anatomy", ink=C["ink900"]):
    """Side-view bee construction, head at +x (flies along the data direction)."""
    # wings behind the body
    b.ellipse(bx + 26, by - 4, 58, 22, fill=C["honey100"], stroke=C["ink700"],
              sw=1.2, rotation=-16, group=group, name="01/bee/wing-1")
    b.ellipse(bx + 42, by + 4, 50, 20, fill=C["paper"], stroke=C["ink700"],
              sw=1.2, rotation=18, group=group, name="01/bee/wing-2")
    # abdomen - thorax - head, left to right
    b.ellipse(bx + 2, by + 28, 58, 38, fill=C["honey300"], stroke=C["ink700"],
              sw=1.4, group=group, name="01/bee/abdomen")
    b.ellipse(bx + 52, by + 29, 34, 33, fill=C["honey500"], stroke=C["ink700"],
              sw=1.4, group=group, name="01/bee/thorax")
    b.ellipse(bx + 84, by + 32, 25, 25, fill=C["ink800"], group=group,
              name="01/bee/head")
    # abdomen bands (ink on honey)
    for i in range(2):
        b.rect(bx + 16 + i * 13, by + 11, 6, 36, fill=ink, r=3, group=group,
               name="01/bee/band-%d" % (i + 1))
    # antennae
    b.line(bx + 98, by + 30, 17, ink, 2, rotation=-55, group=group,
           name="01/bee/antenna-1")
    b.line(bx + 102, by + 34, 15, ink, 2, rotation=-20, group=group,
           name="01/bee/antenna-2")
    # stinger points back along -x: the bee is a routing arrow, it flies +x
    b.poly(bx - 12, by + 40, 15, 11, ink, pointCount=3, rotation=270, group=group,
           name="01/bee/stinger")
    # construction labels + leaders
    b.text(bx + 14, by - 30, "wings", 9, "Consolas", "Regular", C["ink700"],
           width=60, line=12, group=group)
    b.line(bx + 34, by - 14, 20, C["ink500"], 1, rotation=58, group=group,
           name="01/bee/leader-wing")
    b.text(bx + 108, by + 2, "antennae", 9, "Consolas", "Regular", C["ink700"],
           width=70, line=12, group=group)
    b.line(bx + 104, by + 18, 16, C["ink500"], 1, rotation=115, group=group,
           name="01/bee/leader-antenna")
    b.text(bx + 116, by + 44, "head", 9, "Consolas", "Regular", C["ink700"],
           width=50, line=12, group=group)
    b.line(bx + 110, by + 46, 6, C["ink500"], 1, group=group,
           name="01/bee/leader-head")
    b.text(bx + 58, by + 78, "thorax", 9, "Consolas", "Regular", C["ink700"],
           width=60, line=12, group=group)
    b.line(bx + 68, by + 64, 8, C["ink500"], 1, rotation=-80, group=group,
           name="01/bee/leader-thorax")
    b.text(bx - 78, by + 16, "abdomen", 9, "Consolas", "Regular", C["ink700"],
           width=70, line=12, group=group)
    b.line(bx - 32, by + 30, 40, C["ink500"], 1, rotation=6, group=group,
           name="01/bee/leader-abdomen")
    b.text(bx - 82, by + 70, "stinger", 9, "Consolas", "Regular", C["ink700"],
           width=60, line=12, group=group)
    b.line(bx - 26, by + 54, 56, C["ink500"], 1, rotation=-14, group=group,
           name="01/bee/leader-stinger")
    # direction rule
    b.text(bx - 82, by + 102, "BEE FLIES +x  ·  SOURCE → TARGET (±1°)", 8.5,
           "Consolas", "Regular", C["ink700"], width=320, line=12,
           tracking=8, group=group)


'''
s = s[:start] + new_bee + s[end:]

# inverse panel: drop the opaque cover rect (it painted the default grey fill)
old_inv = '''        b.rect(x, y, 420, 230, fill=C["ink900"], r=12, group="01-lockups",
               name="01/lockup/inverse/panel")
        b.rect(x, y, 420, 230, fill=None, r=12, stroke=C["ink800"], sw=1,
               group="01-lockups", name="01/lockup/inverse/edge")
'''
new_inv = '''        b.rect(x, y, 420, 230, fill=C["ink900"], r=12, group="01-lockups",
               name="01/lockup/inverse/panel")
'''
assert old_inv in s
s = s.replace(old_inv, new_inv)

# clear-space card layout: title/body/measure stacked without overlap
old_cs = '''    gx, gy = 932 + 46, 220 + 62
    b.rect(gx, gy, 96, 96, fill=None, r=4, stroke=C["ink500"], sw=1, dash=[4, 4],
           group="01-clearspace", name="01/clearspace/guide")
    b.hex_bee(gx + 16, gy + 16, 64, C["ink900"], C["honey300"], C["honey500"],
              C["paper"], stripe=C["ink900"], group="01-clearspace",
              name="01/clearspace/mark")
    b.text(gx + 18, gy + 104, "h", 9, "Consolas", "Regular", C["ink700"], width=60,
           line=12, group="01-clearspace")
    b.text(gx + 4, gy + 50, "0.25h", 8.5, "Consolas", "Regular", C["signal"],
           width=44, line=12, align="right", group="01-clearspace")
    b.text(gx - 10, gy + 46, "16", 8, "Consolas", "Regular", C["ink500"], width=24,
           line=12, align="right", group="01-clearspace")
    b.text(gx + 178, gy - 8, "Clear space = 0.25 × mark height on every side.", 11,
           "Inter", "SemiBold", C["ink900"], width=250, line=16, group="01-clearspace")
    b.text(gx + 178, gy + 14, "No type, rule, edge or other mark may enter the clear "
                              "space. The guide box above is 1.5h square for a 1h mark.",
           9.5, "Inter", "Regular", C["ink700"], width=240, line=15,
           group="01-clearspace")
    b.text(gx + 178, gy + 64, "MEASURE", 8.5, "Consolas", "Regular", C["ink700"],
           width=220, line=12, tracking=8, group="01-clearspace")
    b.text(gx + 178, gy + 80, "mark 64 px  ·  clear 16 px  ·  box 96 px", 9,
           "Consolas", "Regular", C["ink500"], width=240, line=13,
           group="01-clearspace")
'''
new_cs = '''    gx, gy = 932 + 40, 220 + 54
    b.rect(gx, gy, 96, 96, fill=None, r=4, stroke=C["ink500"], sw=1, dash=[4, 4],
           group="01-clearspace", name="01/clearspace/guide")
    b.hex_bee(gx + 16, gy + 16, 64, C["ink900"], C["honey300"], C["honey500"],
              C["paper"], stripe=C["ink900"], group="01-clearspace",
              name="01/clearspace/mark")
    b.text(gx, gy + 104, "h = 64  ·  clear space 0.25h = 16  ·  box 1.5h = 96", 8,
           "Consolas", "Regular", C["ink500"], width=200, line=12,
           group="01-clearspace")
    b.text(gx - 6, gy + 62, "16", 8, "Consolas", "Regular", C["signal"], width=30,
           line=12, align="right", group="01-clearspace")
    b.text(gx - 6, gy - 16, "16", 8, "Consolas", "Regular", C["signal"], width=30,
           line=12, align="right", group="01-clearspace")
    b.text(gx + 170, gy - 10, "Clear space = 0.25 × mark height on every side.",
           10.5, "Inter", "SemiBold", C["ink900"], width=250, line=14,
           group="01-clearspace")
    b.text(gx + 170, gy + 22, "No type, rule, edge or other mark may enter the clear "
                              "space. The guide box is 1.5h square around a 1h mark.",
           9, "Inter", "Regular", C["ink700"], width=250, line=14,
           group="01-clearspace")
    b.text(gx + 170, gy + 74, "MEASURE / mm-free", 8.5, "Consolas", "Regular",
           C["ink700"], width=250, line=12, tracking=8, group="01-clearspace")
    b.text(gx + 170, gy + 90, "0.25 × mark height, all four sides", 8.5,
           "Consolas", "Regular", C["ink500"], width=250, line=12,
           group="01-clearspace")
'''
assert old_cs in s
s = s.replace(old_cs, new_cs)
io.open(p, "w", encoding="utf-8", newline="\n").write(s)
print("patched common + brand")
