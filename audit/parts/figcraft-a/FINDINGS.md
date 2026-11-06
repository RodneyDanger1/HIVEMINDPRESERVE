# figcraft-a - FINDINGS.md

Read-only audit; every claim has path evidence in EVIDENCE.md. Severity: HIGH blocks audit integrity; MEDIUM correctness/provenance risk; LOW hygiene.

## F1 - HIGH - the 18 primary deliverables are gitignored, so the inventory/assignments omit them
- Inventory = 507 git-tracked files; `docs/ui/**`+`ledgers/**` holds 121 files but only 103 are tracked/assigned.
- Unassigned: `A/A-design.figcraft`, 8 `ASSETS/a-*.{svg,png}`, `B/B-canvas.figcraft`, 8 `ASSETS/b-*.{svg,png}` (~17.4 MB, the actual pixel products). Full table in FILES.md appendix.
- Cause: `.gitignore:7 docs/ui/figcraft/*/*.figcraft` and `.gitignore:9 docs/ui/figcraft/ASSETS/*.png|svg`; confirmed by `git check-ignore -v`; `git status` clean.
- WAVE-SHEET §4 censuses target/, node_modules, dist/, runtime-test/ but not this class, so the paper's exhaustive claim silently excludes the delivered artworks.
- Mitigation exists: hashes registered in A-ASSETS/B-ASSETS/ASSETS-INDEX/ledger; all recomputed here and match (18/18 + 20/20 ledger artifacts).
- Recommend: add the appendix census to the paper; or commit final 4 exports + 2 design docs if manifest tracking is required.

## F2 - MEDIUM - ASSETS-INDEX drops all B export hashes
- A rows: 28/28 carry `sha256_svg`+`sha256_png`; B rows: 0/27 carry them. B hashes exist in B-ASSETS.json and are ledger-verified, so data is not lost, but a consumer of the canonical index alone cannot verify B pixels.
- Evidence: `docs/ui/figcraft/ASSETS-INDEX.json` (owner=="figcraft-b" rows) vs `B/B-ASSETS.json` exports.

## F3 - MEDIUM - ASSETS-INDEX watchdog block stale vs the ledger it cites
- `watchdog.non_blocking[0]` says "A specimens spinner #D9D9D9 (fix in progress)"; the cited ledger (checked_at 05:23:01) records the spinner fixed (`checks.A_design.spinner.fills=null`, honey stroke) and the specimens SVG has 0 off-token colours.
- Index `generated_at` (11:26Z / 05:26 local) is AFTER the ledger, yet carries earlier wording -> stale claim propagated.
- Recommend regenerate index watchdog block from ledger.

## F4 - MEDIUM - B episodes use non-standard outcome and omit command ids
- `EPISODES/b-00..b-06*.json`: outcome="stage-ok" (7/8); only b-exports is "verified" - A's episode contract uses verified/failed/blocked.
- B `plan.steps` carry no command ids, so B's 690-command census is not reconstructable from episodes; verified here directly from `B/scripts/*.json` = 690 commands, 0 invalid ids.
- Recommend normalise B episode schema or document the divergence.

## F5 - MEDIUM - generated CPython-3.14 bytecode + scratch files are git-tracked
- `B/build/__pycache__/*.pyc` (6 files) tracked; filename tag cpython-314 while audit env Python is 3.12.14 -> stale/foreign generated artifacts in VCS.
- `B/build/_run.json` tracked (leftover effect.add/effect.set run script).
- Root .gitignore lacks `__pycache__/` and `*.pyc`; recommend ignore + `git rm --cached`.

## F6 - LOW - patch scripts and intermediates in tracked build folders
- 8 iterative `patch*.py` tracked under `B/build/` alongside final generators.
- 15 intermediate `.figcraft` tracked (A 11, B 7 tmp) superseded by final design docs; 2 A QA PNGs (`c4.png`, `probe-s04c.png`) tracked without a README naming them as evidence.
- Legitimate provenance, but should be declared in the paper's appendix.

## F7 - INFO - verified positives (no defect)
- 103/103 assigned files: sha256+bytes match FILE-MANIFEST (0 mismatches).
- A: 28/28 rows, document sha == actual design sha, export hashes+dims match, node.get resolves 28/28.
- B: 27/27 surfaces, pending=[], design sha == actual, 4/4 export pairs, 27/27 surface->export sha links, command_count 690 == scripts.
- Ledger: 20/20 artifact hashes recomputed and match; PASS, fail_count 0, allow_frontend=true; token audit 20 exact, 0 off-token in 4/4 A SVGs.
- Catalogue: 19 A ids valid / 597 catalogue; 0 rejected syntax guesses; A/B `info` missingFonts=[].

## F8 - INFO - UNVERIFIED in this pass
- Ledger `blueprint_hashes_ok:true` refers to external `hivemind-blueprint/blueprint/BUILD-CHECKS.json`; not re-read -> UNVERIFIED (prove by hashing its 11 listed artifacts).
- Raw `.figcraft` internals and `.pyc` bytecode not parsed beyond documented `info` interface (binary/generated class); no invented content claimed.
- Pixel fidelity vs PDF remains UNVERIFIED (PDF is text-layer only); elevation tiers intentionally identical; Ctrl+0 WebView2 unverified - carried from A/B known limits.
