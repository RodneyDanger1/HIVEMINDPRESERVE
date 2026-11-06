# figcraft-a - EVIDENCE.md

Read-only audit; only writes are the five seat files + diagram under `audit/parts/figcraft-a/`.

## E1 inventory integrity
```
ASSIGNMENTS["figcraft-a"] = 103 paths ; FILE-MANIFEST.csv = 507 rows
assigned present = 103/103 ; missing = []
sha256(actual) vs manifest = 103/103 match ; byte mismatches = 0
actual docs/ui+ledgers files = 121 ; assigned 103 ; unassigned 18
git ls-files docs/ui ledgers = 103 ; assigned-not-tracked = [] ; unassigned-tracked = []
git status --porcelain -- docs/ui ledgers = (empty)
git check-ignore -v A/A-design.figcraft  -> .gitignore:7:docs/ui/figcraft/*/*.figcraft
git check-ignore -v ASSETS/a-foundations.svg -> .gitignore:9:docs/ui/figcraft/ASSETS/*.svg
```

## E2 A manifest vs disk (independent recompute)
```
A-ASSETS.json sha256 = 88832272cb6c81ddff234d77f5abbdf4d9811ae1e8d787b7b02f1c07f11e77f1 (18534 B)
rows=28 ; unique surfaces=28 (S01..S28) ; generated 2026-11-06T05:12:30-0600
document_sha256 == sha256(A-design.figcraft) = 3c5531aba40abd2c98b2855818cda85475fb46985fb9146ef97c537cc1d8d6c0
export sha mismatches = [] ; dimension mismatches = []
node.get on A-design = 28/28 resolve
A-SELF-CHECK.json sha256 = 063066bf47e14579a4ec0ebc49926752d7aab813842b82e2a0c86ab07f710f3d ; ok=true
A-EPISODES.json sha256 = bd7593f710483c82fcc221eb714e05c4eca3868e2fe3ea6ad83cc26b0d4a1730 ; 12 episodes ; 19 command ids ; rejected=[]
figcraft-cli commands --json = 597 rows ; A ids invalid = []
```

## E3 B manifest vs disk
```
B-ASSETS.json sha256 = a8a81e74bfbfbb3b9bca9150db734e8a1444d9b9e04f82cef5c71a17f55d1f4d (26809 B)
27 surfaces ; pending=[] ; 4 export pairs ; export sha mismatches = [] ; surface->export links = 27/27
design sha == sha256(B-canvas.figcraft) = dae6068926e30c99e89a43a1b15257ff8f16b1fd6c943ae990a4f383c0418288
B scripts = 7 files, 690 commands, invalid ids = [] ; B episodes = 8 (outcome stage-ok=7, verified=1)
```

## E4 ledger + index cross-check
```
ledgers/ui-watchdog.json sha256 = 810a5aa01e648857bac89b2faa563ac8517c57205ee988f09704f7fe4067ea39
overall=PASS allow_frontend=true checked_at=2026-11-06T05:23:01-06:00 artifacts=20
recomputed sha mismatches = 0 ; verdict FAIL = 0 ; blueprint_hashes_ok=true (external claim: UNVERIFIED)
ledgers/ui-watchdog.md sha256 = 3ea59e7cde69da53814fa002abcde69ba5633ed96761cef1617a2b1b8ccc1580
ASSETS-INDEX.json sha256 = c5763bfa74a82c1d4e0bdd454786d0e46eec10c543f37b5a99f0df487fac8e41
source-manifest shas match actual A/B = true/true ; design shas match = true/true
counts 55 assets (28 A + 27 B), 28 surfaces ; A rows with sha 28/28 ; B rows with sha 0/27
watchdog.non_blocking[0] stale: "spinner #D9D9D9 (fix in progress)" vs ledger spinner fills=null, off_token=0
```

## E5 asset measurements
```
A SVG/PNG(2x): foundations 2880x2400 ; brand 2880x1800 ; components 2880x5160 ; specimens 2880x2800
A SVG hex audit: 20-token palette, 0 off-token in 4/4 ; <text> nodes = 0 (exporter outlines text)
B SVG/PNG(2x): 4 boards 2880x1800 each ; sha match B-ASSETS + ledger
18 tracked .figcraft parsed by `figcraft-cli info --json` (all ok; final docs 4 pages, missingFonts=[])
6 B .pyc: magic 2b0e0d0a..., filename tag cpython-314 (audit python 3.12.14)
```

## E6 limitations
- All 103 assigned files read/parsed. `.pyc` not decompiled; `.figcraft` internals viewed only through documented `info` output.
- 18 gitignored deliverables outside FILE-MANIFEST by construction (F1); hashes computed directly and cross-checked to A/B manifests + ledger.
- Pixel-level fidelity vs PDF UNVERIFIED (text-layer PDF); carried known limits (elevation tiers identical, Ctrl+0 WebView2).

## E7 commands used
PowerShell Get-FileHash/Get-ChildItem/Get-CimInstance; git ls-files/check-ignore/status; figcraft-cli commands/info/run(read-only node.get);
Python hashlib/csv/json/PIL for hashing, parsing, PNG dims. No builds, installs, network or paid calls.
