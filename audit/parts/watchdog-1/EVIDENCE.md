# EVIDENCE — watchdog-1 partition (`web/**`, 55 files)

Read-only audit; no product/git/rebuild writes. All outputs written only under `audit/parts/watchdog-1/`. Checkpoint HEAD `3988622` (git status for `web/` clean at audit time). Skills loaded: `forgewright-lead`, `integration-logic`, `smartfix`, `forgewright-recon`.

## 1. Inputs read

| Input | Evidence |
|---|---|
| `audit/WAVE-SHEET.md` | read (56 lines; scope, 55-file partition, 16-deep requirement, output schema) |
| `audit/inventory/DEEP-LIST.md` | read (58 lines; 16 `web/**` deep files extracted: package.json, App.tsx, keyboard.ts, client.ts, contract.ts, ChatCore.tsx, Canvas.tsx, nodes.tsx, DataPanel.tsx, SettingsPanel.tsx, agentState.ts, store.ts, strings.ts, contrast.ts, tokens.ts, vite.config.ts) |
| `audit/inventory/ASSIGNMENTS.json` | parsed seat key `watchdog-1` → 55 entries, all `web/**`, disjoint from other seats |
| `audit/inventory/FILE-MANIFEST.csv` | parsed `web/*` rows (55) with bytes/sha256/language/category/depth; used for every metadata block |
| `audit/inventory/FILE-MANIFEST.json` | not separately parsed (same inventory; CSV used as source of record) |
| `DEEP-LIST.md` + WAVE-SHEET | drive the 16×8-paragraph depth requirement met in `FILES.md` |
| all 55 assigned files | content scanned line-by-line (line counts, imports, symbol maps, pattern scans); deep files additionally get symbol maps and 8-paragraph analysis |

## 2. Commands run and observed output (read-only)

```
> git -C <repo> rev-parse HEAD
3988622d6ef65ff39a0c8bb126862a8e8405ffd9
> git -C <repo> status --short -- web
(no output = clean)
> git -C <repo> log --oneline -3
3988622 closeout: sync FINAL-REPORT with packaged CLI + zip hash; process cleanup
2817326 closeout: final landing checks PASS, CLI packaged, processes cleaned; FINAL-CLOSEOUT evidence
6815dbe package: ship hiverouter-cli.exe in tools/bin + deployment zip; refresh checksums/zip hash
```

**Manifest integrity re-verified (read-only `Get-FileHash`):**

```
web files = 55; drift = 0
=> verified sha256 + bytes 55/55 against audit/inventory/FILE-MANIFEST.csv
```

**Dependency/import scan (all of web/src + web/tests):**

```
npm external modules referenced: 8
local import edges: 127
=> see DEPENDENCIES.md §2-§3 for the full matrix
```

**Deep-file symbol maps** (line-numbered export/function/case indices) were produced for the 16 deep files and are reflected in `FILES.md` mechanics claims (e.g. `web/src/flow/Canvas.tsx:113` replayNodes, `web/src/state/store.ts:656` probeSettings, `web/src/api/client.ts:389` waitForOpen).

**Content-pattern scan (all 55 files):**

```
lines scanned: full text of 55 files
any-token occurrences: 3  (all in comments; 0 TypeScript `any` annotations)
TODO/FIXME/XXX/HACK:   0
console.* calls:       0
bare `catch {}`:       0
```

## 3. Green gate evidence (cited, not re-run — audit is read-only/no rebuild)

- `docs/reports/FINAL-REPORT.md:37` — “Frontend unit tests | 46 passed / 8 files, typecheck + build exit 0 | `web/` (vitest)”.
- `docs/plans/FRONTEND-STATUS.md:26-31` — recorded `tsc --noEmit` exit 0 and `tsc --noEmit && vite build` green (latest live-e2e fix set).
- `docs/plans/PHASE-7-8-10-EVIDENCE.md:17,27,30,36` — commands/outputs: typecheck exit 0; “Test Files 8 passed (8) · Tests 44 passed (44)”; `✓ built in 1.15s`; later FINAL-REPORT records 46/8 after two added tests.
- Latest code state matches those gates: `web/src` + `web/tests` files are byte-identical to HEAD manifest hashes (0 drift), and HEAD contains the final commits.
- Additional closeout evidence (read-only hash identity, noted for context): served bundle at `http://127.0.0.1:8799/` equals `web/dist`: index `AD1BD1E69D8F615FCC8CE25D6E6EC6455C7E34C8D7277FA0D5074011584A8E57`; JS `index-Cbfm1IuM.js` `056ECEEFC794AA9C6B5941AD5F2BA0D6F42818B2CE10A60099302F97F9478EAC`; CSS `index-BWOl__uM.css` `96FFA8737F8A6123F364EB2E4F7082D448B470C85FBC7E1A2656A2629443A748`.

## 4. Outputs produced (this seat)

| File | Rows/lines | Content |
|---|---:|---|
| `FILES.md` | 849 lines | 55 file sections (16 deep × 8 paragraphs; 39 standard 1–2 paragraph records), manifest metadata + re-hash status, purpose, mechanics, dependency/caller matrix, quality classification, evidence/UNVERIFIED |
| `DEPENDENCIES.md` | 190 lines | 15 pinned npm deps table, 8-module import matrix, 127 local edges, cross-partition links, runtime/env deps, 6 dependency risks |
| `QUALITY.csv` | 55 data rows | `path,correctness,maintainability,modularity,error_handling,security,docs,evidence,notes`; totals corr 246, maint 233, mod 240, err 236, sec 208, docs 219, ev 253; avg correctness 4.47 / maintainability 4.24 |
| `FINDINGS.md` | 18 findings | F-01/F-02 HIGH; F-03..F-11 MEDIUM; F-12..F-18 LOW; cross-partition notes; each with path:line evidence + recommendation |
| `EVIDENCE.md` | this file | commands, outputs, gates, limits, unverified list |

## 5. Files not read / limits

- **Unreadable in partition: none.** All 55 are UTF-8 text and were content-scanned; no binaries.
- `web/package-lock.json` (78 KB) was verified by manifest sha256 but not hand-analyzed line-by-line (generated lockfile; dependency truth taken from the pinned `web/package.json` + import scan).
- **Generated/ignored classes excluded (per WAVE-SHEET):** `web/node_modules/` (3,976 files/110 MB), `web/dist/` (3 shipped files), repo-wide `target/`, `e2e/node_modules/`, `tools/w64devkit/`, `runtime-test/`.
- An incidental scratch recon file was created under `audit/` during recon and **removed**; final writes are only the five required files under `audit/parts/watchdog-1/`.

## 6. UNVERIFIED (explicit)

1. **Browser runtime rendering** of Canvas/panels/chat is not exercised by this seat (no browser run; e2e seat owns Playwright). Unit tests cover pure modules/store with a fake API, not DOM mounts.
2. **Live socket reconnection timing** (partysocket close→reopen→hello replay) — unit-tested only via a fake transport; real timing unverified.
3. **Font rendering** (F-04) on the target machine — no font assets bundled; fallback appearance unverified.
4. **Pixel-layout fidelity** to FigCraft boards — explicitly UNVERIFIED/needs_human per the asset gate; token values are PDF-derived, never sampled.
5. **Production bundle composition** (whether MockApiClient is tree-shaken, F-05) — would require `vite build` + bundle/source-map inspection, which the read-only audit does not run.
6. **Server-side presence/absence of the optional nonce** (F-08) — contract doc/event struct review only; live APPROVAL_REQUEST payload not observed in this seat.
