import io

# 1) unique element names for the surfaces that used defaults
p = r"C:\Users\Riley\Desktop\hiverouter-v1\docs\ui\figcraft\A\build\pages_components.py"
s = io.open(p, encoding="utf-8").read()
s = s.replace(
'''        b.surfaces[surf] = len(b.el)
        b.name_text(x + 16, ry + 8, label, 12)''',
'''        b.surfaces[surf] = len(b.el)
        b.name_text(x + 16, ry + 8, label, 12,
                    name="02/row/%s/label" % surf)''')
s = s.replace(
'''    b.text(x + 14, y + 14, "Project: HiveMind AI", 12.5, "Inter", "SemiBold",
           C["ink900"], width=230, line=16, group=g, surface="S28")''',
'''    b.text(x + 14, y + 14, "Project: HiveMind AI", 12.5, "Inter", "SemiBold",
           C["ink900"], width=230, line=16, group=g, surface="S28",
           name="02/library/project-name")''')
io.open(p, "w", encoding="utf-8", newline="\n").write(s)

# 2) build state: keep surface element indices (ids are resolved from those, not names)
p2 = r"C:\Users\Riley\Desktop\hiverouter-v1\docs\ui\figcraft\A\build\gen_build.py"
g = io.open(p2, encoding="utf-8").read()
old = '''            "surfaces": list(board.surfaces.keys()),
            "group_ids": board.group_ids,
            "element_names": dict((el["name"], el["id"]) for el in board.el),
            "element_surface": dict((sid, board.el[i]["name"])
                                    for sid, i in board.surfaces.items()),'''
new = '''            "surfaces": list(board.surfaces.keys()),
            "surface_indices": dict(board.surfaces),
            "group_ids": board.group_ids,
            "element_by_idx": dict((str(i), {"name": el["name"], "id": el["id"]})
                                   for i, el in enumerate(board.el)),'''
assert old in g
g = g.replace(old, new)
io.open(p2, "w", encoding="utf-8", newline="\n").write(g)

# 3) manifest: resolve by index, prefer a surface-prefixed group when present
p3 = r"C:\Users\Riley\Desktop\hiverouter-v1\docs\ui\figcraft\A\build\gen_manifest.py"
m = io.open(p3, encoding="utf-8").read()
old = '''        code, b = found
        el_name = b["element_surface"][sid]
        el_id = b["element_names"][el_name]
        group = None
        for gname, gid in b["group_ids"].items():
            if gname.startswith(sid):
                # element is inside that group only if the surface was assigned to a
                # group member; the group id is the stable manifest node.
                group = (gname, gid)
                break'''
new = '''        code, b = found
        idx = b["surface_indices"][sid]
        el = b["element_by_idx"][str(idx)]
        el_name, el_id = el["name"], el["id"]
        group = None
        for gname, gid in b["group_ids"].items():
            if gname.startswith(sid):
                # the surface node is the named group when one starts with the id
                group = (gname, gid)
                break'''
assert old in m
m = m.replace(old, new)
io.open(p3, "w", encoding="utf-8", newline="\n").write(m)
print("patched unique ids")
