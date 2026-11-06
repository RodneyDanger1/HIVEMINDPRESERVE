import io

# ---------------------------------------------------------------- gen_common.py
pg = r"C:\Users\Riley\Desktop\hiverouter-v1\docs\ui\figcraft\A\build\gen_common.py"
g = io.open(pg, encoding="utf-8").read()
start = g.index("    def hex_bee(")
end = g.index("    def button(")
new_hex = '''    def hex_bee(self, x, y, s, hex_fill, bee_body, bee_head, wing, stripe=None,
                mono=False, group=None, name="mark", bee=True, thorax=None):
        """Hexagon mark (pointy-top) with an inner bee flying right (+x)."""
        self.poly(x, y, s, s, hex_fill, pointCount=6, group=group, name=name + "/hex")
        if not bee or s < 20:
            return
        u = s / 56.0
        cx, cy = x + s * 0.5, y + s * 0.5
        # wings behind the body, angled out from the thorax
        self.ellipse(cx - 10 * u, cy - 15 * u, 14 * u, 6.5 * u, fill=wing,
                     rotation=-24, group=group, name=name + "/wing-l")
        self.ellipse(cx + 1 * u, cy - 16 * u, 13 * u, 6.5 * u, fill=wing,
                     rotation=20, group=group, name=name + "/wing-r")
        # stinger trails the body (-x): the bee is a routing arrow flying +x
        self.poly(cx - 22 * u, cy - 2.8 * u, 8 * u, 5.5 * u, bee_head,
                  pointCount=3, rotation=90, group=group, name=name + "/stinger")
        # abdomen, bands, thorax, head
        self.ellipse(cx - 17 * u, cy - 6.5 * u, 19 * u, 13 * u, fill=bee_body,
                     group=group, name=name + "/abdomen")
        if stripe and not mono:
            self.rect(cx - 14.5 * u, cy - 6 * u, 2.0 * u, 12 * u, fill=stripe,
                      group=group, name=name + "/stripe-1")
            self.rect(cx - 11.0 * u, cy - 6 * u, 2.0 * u, 12 * u, fill=stripe,
                      group=group, name=name + "/stripe-2")
        self.ellipse(cx - 2.5 * u, cy - 5 * u, 10 * u, 10 * u,
                     fill=thorax or bee_head, group=group, name=name + "/thorax")
        self.ellipse(cx + 5.5 * u, cy - 4 * u, 8.5 * u, 8.5 * u, fill=bee_head,
                     group=group, name=name + "/head")
        self.line(cx + 8 * u, cy - 5.5 * u, 6.5 * u, bee_head, 1.3 * u,
                  rotation=-45, group=group, name=name + "/antenna-1")
        self.line(cx + 9 * u, cy - 3 * u, 5 * u, bee_head, 1.3 * u, rotation=-12,
                  group=group, name=name + "/antenna-2")

'''
g = g[:start] + new_hex + g[end:]
io.open(pg, "w", encoding="utf-8", newline="\n").write(g)

# ---------------------------------------------------------------- pages_brand.py
p = r"C:\Users\Riley\Desktop\hiverouter-v1\docs\ui\figcraft\A\build\pages_brand.py"
s = io.open(p, encoding="utf-8").read()

# primary lockup mark (paper) - light head so it reads on ink hex
s = s.replace(
'''        b.hex_bee(x + 34, y + 68, 56, C["ink900"], C["honey300"], C["honey500"],
                  C["paper"], stripe=C["ink900"], group="01-lockups",
                  name="01/lockup/primary/mark")''',
'''        b.hex_bee(x + 34, y + 68, 56, C["ink900"], C["honey300"], C["honey100"],
                  C["paper"], stripe=C["ink900"], thorax=C["honey500"],
                  group="01-lockups", name="01/lockup/primary/mark")''')
# stacked mark
s = s.replace(
'''        b.hex_bee(x + 178, y + 46, 64, C["ink900"], C["honey300"], C["honey500"],
                  C["paper"], stripe=C["ink900"], group="01-lockups",
                  name="01/lockup/stacked/mark")''',
'''        b.hex_bee(x + 178, y + 46, 64, C["ink900"], C["honey300"], C["honey100"],
                  C["paper"], stripe=C["ink900"], thorax=C["honey500"],
                  group="01-lockups", name="01/lockup/stacked/mark")''')
# inverse mark: dark bee on honey hex
s = s.replace(
'''        b.hex_bee(x + 34, y + 68, 56, C["honey300"], C["ink900"], C["ink800"],
                  C["honey100"], stripe=C["honey300"], group="01-lockups",
                  name="01/lockup/inverse/mark")''',
'''        b.hex_bee(x + 34, y + 68, 56, C["honey300"], C["ink900"], C["ink900"],
                  C["honey100"], stripe=C["honey300"], thorax=C["ink800"],
                  group="01-lockups", name="01/lockup/inverse/mark")''')
# clear-space mark
s = s.replace(
'''    b.hex_bee(gx + 16, gy + 16, 64, C["ink900"], C["honey300"], C["honey500"],
              C["paper"], stripe=C["ink900"], group="01-clearspace",
              name="01/clearspace/mark")''',
'''    b.hex_bee(gx + 16, gy + 16, 64, C["ink900"], C["honey300"], C["honey100"],
              C["paper"], stripe=C["ink900"], thorax=C["honey500"],
              group="01-clearspace", name="01/clearspace/mark")''')
# 56 min size
s = s.replace(
'''    b.hex_bee(72, 772, 56, C["ink900"], C["honey300"], C["honey500"], C["paper"],
              stripe=C["ink900"], group="01-minsizes", name="01/minsizes/56")''',
'''    b.hex_bee(72, 772, 56, C["ink900"], C["honey300"], C["honey100"], C["paper"],
              stripe=C["ink900"], thorax=C["honey500"], group="01-minsizes",
              name="01/minsizes/56")''')
# 32 min size
s = s.replace(
'''    b.hex_bee(232, 784, 32, C["ink900"], C["honey300"], C["honey500"], C["paper"],
              stripe=C["ink900"], group="01-minsizes", name="01/minsizes/32")''',
'''    b.hex_bee(232, 784, 32, C["ink900"], C["honey300"], C["honey100"], C["paper"],
              stripe=C["ink900"], thorax=C["honey500"], group="01-minsizes",
              name="01/minsizes/32")''')

# anatomy: wings separation, bands inside abdomen, stinger rotation, leaders
s = s.replace(
'''    b.ellipse(bx + 26, by - 4, 58, 22, fill=C["honey100"], stroke=C["ink700"],
              sw=1.2, rotation=-16, group=group, name="01/bee/wing-1")
    b.ellipse(bx + 42, by + 4, 50, 20, fill=C["paper"], stroke=C["ink700"],
              sw=1.2, rotation=18, group=group, name="01/bee/wing-2")''',
'''    b.ellipse(bx + 14, by + 2, 48, 18, fill=C["honey100"], stroke=C["ink700"],
              sw=1.2, rotation=-30, group=group, name="01/bee/wing-1")
    b.ellipse(bx + 58, by - 2, 44, 16, fill=C["paper"], stroke=C["ink700"],
              sw=1.2, rotation=24, group=group, name="01/bee/wing-2")''')
s = s.replace(
'''    for i in range(2):
        b.rect(bx + 16 + i * 13, by + 11, 6, 36, fill=ink, r=3, group=group,
               name="01/bee/band-%d" % (i + 1))''',
'''    for i in range(2):
        b.rect(bx + 16 + i * 13, by + 34, 6, 26, fill=ink, r=3, group=group,
               name="01/bee/band-%d" % (i + 1))''')
s = s.replace(
'''    b.poly(bx - 12, by + 40, 15, 11, ink, pointCount=3, rotation=270, group=group,
           name="01/bee/stinger")''',
'''    b.poly(bx - 14, by + 40, 15, 11, ink, pointCount=3, rotation=90, group=group,
           name="01/bee/stinger")''')
s = s.replace(
'''    b.line(bx + 34, by - 14, 20, C["ink500"], 1, rotation=58, group=group,
           name="01/bee/leader-wing")''',
'''    b.line(bx + 34, by - 2, 20, C["ink500"], 1, rotation=42, group=group,
           name="01/bee/leader-wing")''')
io.open(p, "w", encoding="utf-8", newline="\n").write(s)
print("patched bee marks")
