# 12 — GitHub Publication Receipt

## Destination

- Repository: <https://github.com/RodneyDanger1/HiveMind>
- Default branch: `main`
- Remote name in the source checkout: `hive`
- First pushed commit: `cf51e12e388738ab2df6235801b9cd7f98811701`
- Parent state: source history was joined with the remote's initial `2a7b12e Initial commit` via a merge. The remote initial commit was **not** force-overwritten.

## What was pushed

| Item | Value |
|---|---:|
| Tracked files on remote `main` | **822** |
| Files frozen in the census (pre-CODEMAP) | 805 |
| CODEMAP files added | 18 |
| Product-tree changes in the publication commit | none |
| Ignored `target/`, `node_modules/`, `tools/w64devkit/` | not pushed |
| Binary assets | pushed as tracked repository files; none >100 MB |

The 18 CODEMAP files are:
`00-START-HERE.md`, `01-SCOPE-AND-CENSUS.md`, `02-ARCHITECTURE.md`, `03-BACKEND.md`, `04-WEB.md`,
`05-E2E-TOOLS.md`, `06-DOCS-PLANS-AUDIT.md`, `07-RUNTIME-BINARIES.md`, `08-FILE-CATALOG.md`,
`09-RISKS-GAPS.md`, `10-VERIFICATION.md`, `11-AUDITOR-FINDINGS.md`, `12-GITHUB-PUSH.md`,
`_tools/build_codemap.py`, `_tools/build_docs.py`, `_data/summary.json`, `_data/inventory.csv`, `_data/inventory.json`.

## Exact commands used

```powershell
git remote add hive https://github.com/RodneyDanger1/HiveMind.git
git fetch --depth=1 hive main
git merge FETCH_HEAD --allow-unrelated-histories -X ours -m "chore: join HiveMind GitHub initial commit"
git push hive master:main
git ls-remote hive refs/heads/main
git fetch --depth=1 hive main
git ls-tree -r --name-only FETCH_HEAD | Measure-Object
```

## Verification

- `git ls-remote hive refs/heads/main` → `cf51e12e388738ab2df6235801b9cd7f98811701`.
- Remote `git ls-tree -r --name-only FETCH_HEAD | Measure-Object` → **822**.
- Remote CODEMAP paths after the follow-up push → **18**.
- Local `HEAD` at push → `cf51e12`; local worktree clean after the commit.
- No force push, no tag change, no product-code change.

## Self-reference note

The frozen census describes the repository **before** `docs/CODEMAP/` existed: 805 tracked / 36,480 working-tree / 35,675 ignored. Publishing the encyclopedia adds tracked documentation files, so the current repository has **822** tracked files. Both numbers are true at their stated commit; they must not be mixed.


## Follow-up: historical inputs integrated

The supplied Git-ingest dump, duplicate dump, and GitDiagram export were integrated into:
`13-HISTORICAL-AUDIT.md`, `14-GITDIAGRAM-ARCHITECTURE.md`, `15-INPUT-PROVENANCE.md`, and
`_incoming/{DUMP-ANALYSIS,DUMP-DELTA,DIAGRAM-ANALYSIS}.md`.

That follow-up adds 6 tracked files, taking the repository from 823 to **829** tracked files.
The frozen 805-file census is unchanged; it describes `e021b12` before CODEMAP existed.
