# APPENDIX D — GITIGNORED DESIGN DELIVERABLES (outside the 507 tracked manifest)

These **18** delivered design artifacts are ignored by `.gitignore` (`.figcraft` + `ASSETS/*.svg|png`)
and therefore are *not* in `FILE-MANIFEST.csv`. Their hashes are recorded in `A/A-ASSETS.json`, `B/B-ASSETS.json`,
`ASSETS-INDEX.json` and `ledgers/ui-watchdog.json`; the values below were recomputed from disk this audit.

- Count: **18** · total **22,009,234 B (21.0 MiB)**
- Verification: every listed file exists with the hash shown (recomputed).

| Artifact | Owner | Kind | Surface | Bytes | sha256 | Tracked |
|---|---|---|---|---:|---|---|
| `docs/ui/figcraft/A/A-design.figcraft` | figcraft-a | A design | all | 127,191 | `3C5531ABA40ABD2C98B28558…` | no (gitignored) |
| `docs/ui/figcraft/ASSETS/a-brand.png` | figcraft-a | A export | S23 | 628,963 | `47680984AA5DACB1BF6A3576…` | no (gitignored) |
| `docs/ui/figcraft/ASSETS/a-brand.svg` | figcraft-a | A export | S23 | 940,908 | `94CF946703C1435560181336…` | no (gitignored) |
| `docs/ui/figcraft/ASSETS/a-components.png` | figcraft-a | A export | S02 | 2,067,230 | `F4FCC4C63CAEC0E40FA59117…` | no (gitignored) |
| `docs/ui/figcraft/ASSETS/a-components.svg` | figcraft-a | A export | S02 | 3,761,056 | `2EB182185E22183C3473F137…` | no (gitignored) |
| `docs/ui/figcraft/ASSETS/a-foundations.png` | figcraft-a | A export | S24 | 1,054,806 | `E49B788A812E8A4962A29A92…` | no (gitignored) |
| `docs/ui/figcraft/ASSETS/a-foundations.svg` | figcraft-a | A export | S24 | 1,768,930 | `24536354E6FFE9BA4DDA2C96…` | no (gitignored) |
| `docs/ui/figcraft/ASSETS/a-specimens.png` | figcraft-a | A export | S01 | 886,858 | `DA0F4CB7D435E46CDABEA310…` | no (gitignored) |
| `docs/ui/figcraft/ASSETS/a-specimens.svg` | figcraft-a | A export | S01 | 1,800,657 | `E88AECA9A0F448FE87116837…` | no (gitignored) |
| `docs/ui/figcraft/ASSETS/b-canvas-default.png` | figcraft-b | B export | S01 | 1,027,268 | `AA6066D38F73D391B45C623C…` | no (gitignored) |
| `docs/ui/figcraft/ASSETS/b-canvas-default.svg` | figcraft-b | B export | S01 | 1,223,908 | `B5A5A824145A85848247A10A…` | no (gitignored) |
| `docs/ui/figcraft/ASSETS/b-hero.png` | figcraft-b | B export | multiple | 941,929 | `719A28BE8DE4926612ECFE55…` | no (gitignored) |
| `docs/ui/figcraft/ASSETS/b-hero.svg` | figcraft-b | B export | multiple | 780,682 | `2BF8D20FACBA986B2C148702…` | no (gitignored) |
| `docs/ui/figcraft/ASSETS/b-interaction.png` | figcraft-b | B export | S24 | 1,025,547 | `DE50112439898E88B8BBF334…` | no (gitignored) |
| `docs/ui/figcraft/ASSETS/b-interaction.svg` | figcraft-b | B export | S24 | 1,665,849 | `927514772AC5321C3771ED72…` | no (gitignored) |
| `docs/ui/figcraft/ASSETS/b-zoom-states.png` | figcraft-b | B export | multiple | 1,114,713 | `A02CB4A1963A8C3588AE3CC6…` | no (gitignored) |
| `docs/ui/figcraft/ASSETS/b-zoom-states.svg` | figcraft-b | B export | multiple | 966,005 | `AB1EE97AE2B9D042B9D8B78E…` | no (gitignored) |
| `docs/ui/figcraft/B/B-canvas.figcraft` | figcraft-b | B design | all | 226,734 | `DAE6068926E30C99E89A43A1…` | no (gitignored) |

**Implication:** the empirical corpus is 507 tracked files *plus* this 18-file ignored deliverable class;
assertions in the paper that say “every file” refer to tracked files only, and this appendix closes that gap.
**Migration:** decide a policy — commit final design sources+exports (with size budget) or generate them reproducibly
from scripts and store only hashes/manifests. Current mixed tracking is a reproducibility defect (A-F12).
