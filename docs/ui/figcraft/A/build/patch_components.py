import io

# PAGES height for components must match the board
p0 = r"C:\Users\Riley\Desktop\hiverouter-v1\docs\ui\figcraft\A\build\pages_foundations.py"
s0 = io.open(p0, encoding="utf-8").read()
s0 = s0.replace('("02 Components", 1440, 2500),', '("02 Components", 1440, 2580),')
io.open(p0, "w", encoding="utf-8", newline="\n").write(s0)

p = r"C:\Users\Riley\Desktop\hiverouter-v1\docs\ui\figcraft\A\build\pages_components.py"
s = io.open(p, encoding="utf-8").read()

# 1. buttons matrix: record surface -> first element of each row group
old = '''    for r, (label, variant, g, surf, cmd) in enumerate(rows):
        ry = y + 36 + r * 68
        b.name_text(x + 16, ry + 8, label, 12, group=g)'''
new = '''    for r, (label, variant, g, surf, cmd) in enumerate(rows):
        ry = y + 36 + r * 68
        first_el = len(b.el)
        b.surfaces[surf] = first_el
        b.name_text(x + 16, ry + 8, label, 12, group=g)'''
assert old in s
s = s.replace(old, new)

# 2. settings card height / section clash
old = '''def settings_and_states(b):
    x, y, w = 936, 1396, 456
    b.section(936, 1370, 456, "settings · key status", note="PDF p7:254")
    g = "S21-settings"
    b.rect(x, y, w, 180, fill=C["paper"], r=12, stroke=C["hairline"], sw=1,
           group=g, surface="S21", name="02/settings/panel")'''
new = '''def settings_and_states(b):
    x, y, w = 936, 1396, 456
    b.section(936, 1370, 456, "settings · key status", note="PDF p7:254")
    g = "S21-settings"
    b.rect(x, y, w, 160, fill=C["paper"], r=12, stroke=C["hairline"], sw=1,
           group=g, surface="S21", name="02/settings/panel")'''
assert old in s
s = s.replace(old, new)
old = '''    b.text(x, y + 188, "S21 · SETTINGS / KEY STATUS · PDF p7:254", 9, "Consolas",
           "Regular", C["ink500"], width=w, line=13, group=g)

    # state vocabulary (compact, pointer to specimens)
    y2 = 1596
    b.section(936, 1570, 456, "state enum · 15 members", note="UI-COMMANDS-v1 §4")
    b.rect(x, y2, w, 130, fill=C["paper"], r=12, stroke=C["hairline"], sw=1,
           group="02-stateenum", name="02/stateenum/panel")'''
new = '''    b.text(x, y + 168, "S21 · SETTINGS / KEY STATUS · PDF p7:254", 9, "Consolas",
           "Regular", C["ink500"], width=w, line=13, group=g)

    # state vocabulary (compact, pointer to specimens)
    y2 = 1606
    b.section(936, 1580, 456, "state enum · 15 members", note="UI-COMMANDS-v1 §4")
    b.rect(x, y2, w, 120, fill=C["paper"], r=12, stroke=C["hairline"], sw=1,
           group="02-stateenum", name="02/stateenum/panel")'''
assert old in s
s = s.replace(old, new)

# 3. minimap: drop zero-length edges, use thin rect connectors
old = '''    nodes = [(x + 60, y + 40), (x + 110, y + 84), (x + 168, y + 46),
             (x + 196, y + 104), (x + 96, y + 122)]
    edges = [(0, 1), (0, 2), (1, 3), (1, 4), (2, 3)]
    for a, bb in edges:
        ax, ay = nodes[a]
        bx, by = nodes[bb]
        b.line(ax + 8, ay + 8, 0, C["honey600"], 1, name="02/minimap/edge")'''
new = '''    nodes = [(x + 60, y + 40), (x + 110, y + 84), (x + 168, y + 46),
             (x + 196, y + 104), (x + 96, y + 122)]
    b.rect(x + 78, y + 58, 1, 48, fill=C["honey600"], group=g,
           name="02/minimap/edge-a")
    b.rect(x + 78, y + 58, 120, 1, fill=C["honey600"], group=g,
           name="02/minimap/edge-b")
    b.rect(x + 128, y + 100, 1, 40, fill=C["honey600"], group=g,
           name="02/minimap/edge-c")'''
assert old in s
s = s.replace(old, new)

io.open(p, "w", encoding="utf-8", newline="\n").write(s)
print("patched components")
