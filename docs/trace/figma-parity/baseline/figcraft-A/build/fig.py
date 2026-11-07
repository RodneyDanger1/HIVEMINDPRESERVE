"""FigCraft headless driver — only the documented CLI, no invented commands.

Exposes: run_script(commands, in_path=None, out_path=None) -> list of results
Each command dict: {"command": "<catalogue id>", "params": {...}}
The CLI prints one JSON result line per step; a rejected param aborts the whole
transaction (run rolls back), so callers treat any non-ok step as fatal.
"""
import json
import os
import subprocess
import sys

CLI = r"C:\Program Files\FigCraft\figcraft-cli.exe"
ROOT = r"C:\Users\Riley\Desktop\hiverouter-v1\docs\ui\figcraft\A"


def run_script(commands, in_path=None, out_path=None, jsonl=False):
    script_path = os.path.join(ROOT, "build", "_run.json")
    if jsonl:
        script_path = os.path.join(ROOT, "build", "_run.jsonl")
        with open(script_path, "w", encoding="utf-8", newline="\n") as f:
            for c in commands:
                f.write(json.dumps(c, ensure_ascii=False) + "\n")
    else:
        with open(script_path, "w", encoding="utf-8", newline="\n") as f:
            json.dump(commands, f, ensure_ascii=False, indent=1)
    argv = [CLI, "run", script_path]
    if in_path:
        argv += ["--in", in_path]
    if out_path:
        argv += ["--out", out_path]
    proc = subprocess.run(argv, capture_output=True, text=True, encoding="utf-8")
    if proc.returncode != 0:
        raise RuntimeError("run failed exit=%s\nstdout:\n%s\nstderr:\n%s"
                           % (proc.returncode, proc.stdout, proc.stderr))
    results = []
    for line in proc.stdout.splitlines():
        line = line.strip()
        if not line:
            continue
        try:
            results.append(json.loads(line))
        except json.JSONDecodeError:
            raise RuntimeError("non-JSON stdout line: %r\nstderr: %s" % (line, proc.stderr))
    bad = [r for r in results if not r.get("ok")]
    if bad:
        raise RuntimeError("step rejected: %s" % json.dumps(bad[0], ensure_ascii=False))
    if len(results) != len(commands):
        raise RuntimeError("result count %d != command count %d" % (len(results), len(commands)))
    return results


def info(path):
    proc = subprocess.run([CLI, "info", path, "--json"], capture_output=True,
                          text=True, encoding="utf-8")
    if proc.returncode != 0:
        raise RuntimeError("info failed: %s" % proc.stderr)
    return json.loads(proc.stdout)


def export(path, out, fmt, node=None, page=None, scale=None):
    argv = [CLI, "export", path, "-o", out, "--format", fmt]
    if node:
        argv += ["--node", node]
    if page:
        argv += ["--page", page]
    if scale is not None:
        argv += ["--scale", str(scale)]
    proc = subprocess.run(argv, capture_output=True, text=True, encoding="utf-8")
    if proc.returncode != 0:
        raise RuntimeError("export failed: %s %s" % (proc.stdout, proc.stderr))
    got = proc.stdout.strip()
    return got


class Ids:
    """Maps the Nth created node (per page build) to its real id from run results."""

    def __init__(self, results):
        self.results = results

    def created(self):
        out = []
        for i, r in enumerate(self.results):
            res = r.get("result") or {}
            if isinstance(res, dict) and "id" in res:
                out.append((i, res["id"]))
        return out


if __name__ == "__main__":
    print(json.dumps(info(sys.argv[1]), indent=2))
