import io
P = 'make_manifest.py'
s = io.open(P, encoding='utf-8').read()
old = '''        hit = state.get("surfaces", {}).get(sid)
        if not hit:
            continue'''
new = '''        hit = state.get("surfaces", {}).get(sid)
        if not hit:
            hit = {"page": None, "node_name": SURFACE_NODE.get(sid),
                   "wiring": info[1], "pdf_lines": info[2], "name": info[0]}
        if hit.get("page") is None:
            for c, st in state.get("boards", {}).items():
                if hit.get("node_name") in st.get("element_names", {}):
                    hit["page"] = st["page"]
                    break
        if not hit.get("page"):
            continue'''
assert s.count(old) == 1
s = s.replace(old, new)
io.open(P, 'w', encoding='utf-8', newline='\n').write(s)
print('patched make_manifest.py')
