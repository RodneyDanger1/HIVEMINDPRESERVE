"""FigCraft headless driver for the C (HiveMind shell/HMAS/states) deliverable.

Only the documented CLI is used: `run` (transactional), `info`, `export`.
Node ids are never invented: stage 1 frames report their ids and every content
stage reads back the ids of the elements it created.
"""
import json
import os
import subprocess

CLI = os.environ.get("FIGCRAFT_CLI", r"C:\Program Files\FigCraft\figcraft-cli.exe")


def _project_root(start):
    path = os.path.abspath(start)
    while True:
        if os.path.isdir(os.path.join(path, "backend")):
            return path
        parent = os.path.dirname(path)
        if parent == path:
            return os.path.abspath(start)
        path = parent


HERE = os.path.dirname(os.path.abspath(__file__))
C_ROOT = os.path.dirname(HERE)
FIG_ROOT = os.path.dirname(C_ROOT)
PROJECT = _project_root(FIG_ROOT)
BUILD = os.path.join(C_ROOT, "build")
SCRIPTS = os.path.join(C_ROOT, "scripts")
EPISODES = os.path.join(C_ROOT, "episodes")
ASSETS = os.path.join(FIG_ROOT, "ASSETS")
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


def tool_info():
    """Captured CLI identity for the manifest (never invented)."""
    version = _run_proc([CLI, "--version"])
    commands = _run_proc([CLI, "commands", "--json"])
    count = None
    try:
        rows = json.loads(commands.stdout)
        count = len(rows) if isinstance(rows, list) else len(rows.get("commands", []))
    except Exception:
        count = None
    return {
        "name": "figcraft-cli",
        "cli_path": CLI,
        "version_stdout": (version.stdout or version.stderr).strip(),
        "version_ok": version.returncode == 0,
        "command_count": count,
    }
