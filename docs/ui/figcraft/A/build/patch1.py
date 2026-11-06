import io

# --- gen_common.py: shorter pair labels on the board (values unchanged) -------
pg = r"C:\Users\Riley\Desktop\hiverouter-v1\docs\ui\figcraft\A\build\gen_common.py"
g = io.open(pg, encoding="utf-8").read()
reps = [
    ('("ink-900 / canvas", "14.67:1", "AAA")', '("ink-900/canvas", "14.67:1", "AAA")'),
    ('("ink-900 / honey-400", "8.74:1", "AAA")', '("ink-900/honey-400", "8.74:1", "AAA")'),
    ('("white / ink-800", "12.64:1", "AAA")', '("white/ink-800", "12.64:1", "AAA")'),
    ('("amber / canvas", "3.32:1", "AA-LARGE")', '("amber/canvas", "3.32:1", "AA-LG")'),
    ('("error / canvas", "5.41:1", "AA")', '("error/canvas", "5.41:1", "AA")'),
    ('("verify / canvas", "4.64:1", "AA")', '("verify/canvas", "4.64:1", "AA")'),
]
for a, b in reps:
    assert a in g, a
    g = g.replace(a, b)
io.open(pg, "w", encoding="utf-8", newline="\n").write(g)

# --- pages_foundations.py: contrast table + radii specimen --------------------
p = r"C:\Users\Riley\Desktop\hiverouter-v1\docs\ui\figcraft\A\build\pages_foundations.py"
s = io.open(p, encoding="utf-8").read()

old = '''    b.section(48, 720, 330, "contrast · WCAG 2.1", note="PDF p2:80-92")
    for i, (pair, ratio, level) in enumerate(CONTRAST_PRINTED):
        col, row = i // 3, i % 3
        x = 48 + col * 165
        y = 748 + row * 24
        b.text(x, y, pair, 8.5, "Consolas", "Regular", C["ink700"], width=88,
               line=12, group="00-contrast")
        b.text(x + 90, y, ratio, 9.5, "Consolas", "Regular", C["ink900"], width=38,
               line=12, align="right", group="00-contrast")
        b.text(x + 130, y, level, 7.5, "Consolas", "Regular", C["ink500"], width=36,
               line=12, align="right", group="00-contrast")
'''
new = '''    b.section(48, 720, 330, "contrast · WCAG 2.1", note="PDF p2:80-92")
    for i, (pair, ratio, level) in enumerate(CONTRAST_PRINTED):
        col, row = i // 3, i % 3
        x = 48 + col * 166
        y = 748 + row * 24
        b.text(x, y, pair, 8.5, "Consolas", "Regular", C["ink700"], width=96,
               line=12, group="00-contrast")
        b.text(x + 98, y, ratio + " " + level, 8.5, "Consolas", "Regular",
               C["ink900"], width=66, line=12, align="right", group="00-contrast")
'''
assert old in s
s = s.replace(old, new)

old_radii = '''    b.section(740, 720, 300, "radius · 7 stops", note="PDF p4:135")
    for i, v in enumerate(RADII):
        rx = 740 + i * 42
        b.rect(rx, 748, 36, 36, fill=C["honey100"], r=(36 if v == 999 else v),
               stroke=C["honey500"], sw=1, group="00-radii")
        b.text(rx, 788, str(v), 8.5, "Consolas", "Regular", C["ink700"], width=36,
               line=12, align="center", group="00-radii")
'''
new_radii = '''    b.section(740, 720, 300, "radius · 7 stops", note="PDF p4:135")
    rad_layout = [(4, 34, 34), (8, 34, 34), (12, 34, 34), (16, 34, 34),
                  (24, 40, 40), (48, 40, 40), (999, 58, 26)]
    rx = 740
    for v, wdt, hgt in rad_layout:
        b.rect(rx, 750 + (40 - hgt), wdt, hgt, fill=C["honey100"],
               r=(hgt if v == 999 else v), stroke=C["honey500"], sw=1,
               group="00-radii")
        b.text(rx, 794, str(v), 8.5, "Consolas", "Regular", C["ink700"],
               width=wdt, line=12, align="center", group="00-radii")
        rx += wdt + 8
'''
assert old_radii in s
s = s.replace(old_radii, new_radii)
io.open(p, "w", encoding="utf-8", newline="\n").write(s)
print("patched")
