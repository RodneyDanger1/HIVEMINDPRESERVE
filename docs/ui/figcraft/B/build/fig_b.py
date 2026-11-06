"""FigCraft headless driver for the B (canvas) deliverable.

Only the documented CLI is used: `run` (transactional), `info`, `export`.
Node ids are never invented: stage 1 frames report their ids, and the effects
stage reads ids back from the per-step JSON results of the content stage.
"""
import json
import os
import subprocess

CLI = r"C:\Program Files\FigCraft\figcraft-cli.exe"
ROOT = r"C:\Users\Riley\Desktop\hiverouter-v1\docs\ui\figcraft\B"
BUILD = os.path.join(ROOT, "build")
SCRIPTS = os.path.join(ROOT, "scripts")
ASSETS = os.path.join(ROOT, "..", "ASSETS")
TMP = os.path.join(BUILD, "tmp")
_SCRATCH = os.path.join(BUILD, "_run.json")


def _run_proc(argv):
    return subprocess.run(argv, capture_output=True, text=True, encoding="utf-8",
                          errors="replace")


def run_script(commands, in_path=None, out_path=None):
    """Execute a JSON array of {"command","params"} in one headless transaction."""
    os.makedirs(SCRIPTS, exist_ok=True)
    with open(_SCRATCH, "w", encoding="utf-8", newline="\n") as f:
        json.dump(commands, f, ensure_ascii=False, indent=1)
    argv = [CLI, "run", _SCRATCH]
    if in_path:
        argv += ["--in", in_path]
    if out_path:
        argv += ["--out", out_path]
    proc = _run_proc(argv)
    if proc.returncode != 0:
        raise RuntimeError("run failed exit=%s\nstdout:\n%s\nstderr:\n%s"
                           % (proc.returncode, proc.stdout[-4000:], proc.stderr[-4000:]))
    results = []
    for line in proc.stdout.splitlines():
        line = line.strip()
        if not line:
            continue
        try:
            results.append(json.loads(line))
        except json.JSONDecodeError:
            raise RuntimeError("non-JSON stdout line: %r" % line)
    bad = [r for r in results if not r.get("ok")]
    if bad:
        raise RuntimeError("step rejected: %s" % json.dumps(bad[0], ensure_ascii=False))
    if len(results) != len(commands):
        raise RuntimeError("result count %d != command count %d"
                           % (len(results), len(commands)))
    return results


def info(path):
    proc = _run_proc([CLI, "info", path, "--json"])
    if proc.returncode != 0:
        raise RuntimeError("info failed: %s %s" % (proc.stdout, proc.stderr))
    return json.loads(proc.stdout)


def export(path, out, fmt, node=None, scale=None):
    os.makedirs(os.path.dirname(os.path.abspath(out)), exist_ok=True)
    argv = [CLI, "export", path, "-o", out, "--format", fmt]
    if node:
        argv += ["--node", node]
    if scale is not None:
        argv += ["--scale", str(scale)]
    proc = _run_proc(argv)
    if proc.returncode != 0:
        raise RuntimeError("export failed: %s %s" % (proc.stdout[-3000:], proc.stderr[-3000:]))
    return proc.stdout.strip()
