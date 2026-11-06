# SEAT-08 EVIDENCE — commands, hashes, fetch log, contradiction matrix

Seat S8 (Docs/Plan Compiler), read-only. Repo `C:\Users\Riley\Desktop\hiverouter-v1`. Window 2026-11-06 10:59–11:32 CT.
No product/git/config writes; no key access; no paid calls. Deliverable writes confined to `plans/seats/SEAT-08/`.

## 1. Repository state at audit time

```
$ git log -3 --format="%h %ci %s"
5c69b66 2026-11-06 10:59:06 -0600 feat: wire DeepSeek provider into core; live smoke, replay arm, 16-state parity, settings
f4c780d 2026-11-06 09:52:40 -0600 audit: close verifier doc-sync notes (LOC breakdown, roadmap wording already 101)
cf70b22 2026-11-06 09:52:14 -0600 audit: 507-file exhaustive audit -- paper, 30 findings, quality matrix, 6 diagrams, 101-task roadmap, verification

$ git status --short
 M backend/Cargo.lock
 M backend/crates/hiverouter-contract/Cargo.toml
 M backend/crates/hiverouter-core/Cargo.toml
 M backend/crates/hiverouter-provider/Cargo.toml
 M backend/crates/hiverouter-tools/Cargo.toml
 M web/package.json
?? docs/RELEASE.md
?? plans/seats/
?? tools/release-manifest.json

$ git diff --stat
6 files changed, 20 insertions(+), 15 deletions(-)
```

**Quiescence verdict: FAIL** — product tree dirty at audit time (S8 did not cause it; the only S8 writes are `plans/seats/SEAT-08/**`).

## 2. Artifact hashes/sizes/timestamps (Get-Item + Get-FileHash)

| Artifact | Bytes | sha256 | mtime (CT) |
|---|---:|---|---|
| `tools/bin/hiverouter.exe` | 9,329,152 | `184D51A7C3CCA7B9D50954B677C5AD8A14472700A651FDF0D270441BE3EAC9A9` | 09:10:59 |
| `dist/HiveMind-v1-win64.zip` | 6,571,928 | `BC7A2BAD98FAB1C849E3340D195C26F165581350B630F7BA37D89EE7CB27FEE7` | 09:33:17 |
| `web/dist/index.html` | 774 | (served index) | 10:58:47 |
| `backend/crates/hiverouter-core/src/provider_executor.rs` | 10,589 | (new source) | 10:56:06 |
| `backend/crates/hiverouter-core/tests/provider_live.rs` | 2,491 | (new test) | 10:41:04 |
| `web/src/state/agentState.ts` | 4,671 | (16-state enum) | 10:44:27 |
| `docs/RELEASE.md` (untracked draft) | 3,287 | `10D32112813D58E120B1F7D8E2E8F9C963C51DB9E9DCCD95E0006AFAF1E8F014` | 10:59:58 |
| `docs/reports/FINAL-REPORT.md` | 6,121 | `978271BE811ACDFD97FD55F5A9F3E694E18FA78CCC9DC8F4F893073B0148514D` | 09:34:52 |

**Conclusion:** both shipped artifacts (09:10/09:33) predate the provider commit (10:59:06) and the final web build (10:58:47).

## 3. Docs slice hashes (full table in `READLOG.md` §4)

`docs/RELEASE.md` 3,287 B / `10D32112…1E8F014`; `FINAL-REPORT.md` 6,121 B / `978271BE…48514D`; `key-validation.md` 1,038 B / `71EA6868…23A2A6`; `live-provider-smoke.md` 1,312 B / `9ED04F45…E18EEB`; `README.md` 1,742 B / `B60FAC5A…10852`; `tools/runbook.md` 4,396 B / `E7FBA10F…72CFC`; `PHASE-12-EVIDENCE.md` 9,448 B / `74CA6068…D169F`; `D2-D3-WIRING-EVIDENCE.md` 7,031 B / `77F6D5CD…BCB8`; `audit/VERIFICATION.md` 9,937 B / `99C3A398…B32C45`; `audit/ROADMAP.md` 13,804 B / `4D69A25A…E98756`; `audit/ROADMAP-TASKS.csv` 35,038 B / `06869F6C…6042`; `docs/claims/CLAIM-REGISTER.csv` 24,292 B / `1518294B…C9AD7`; `plans/HIVEMIND-1.0-SEAT-ASSIGNMENTS.md` 3,629 B / `4F688B24…951EC`; `audit/HIVEMIND-AUDIT-PAPER.md` 25,141 B / `3BAAD198…3D495`; `audit/tools/build_appendix.py` 1,859 B / `F918D362…54DBC`; `audit/tools/synthesize_paper.py` 10,176 B / `18D20042…8CD4`.

## 4. Official DeepSeek docs fetch log (web_fetch, HTTP 200; date 2026-11-06)

| URL | Fetched (CT) | Facts used |
|---|---|---|
| https://api-docs.deepseek.com/ | 11:01 | base_url, model ids, env key, chat endpoint, thinking/reasoning params |
| https://api-docs.deepseek.com/quick_start/pricing/ | 11:01 | flash/v4-pro per-1M prices (cache hit/miss, output, peak/off-peak), concurrency 2500/500, peak windows |
| https://api-docs.deepseek.com/guides/thinking_mode/ | 11:02 | thinking toggle, reasoning_effort mapping, parameter interactions |
| https://api-docs.deepseek.com/api/create-chat-completion/ | 11:02 | endpoint, max_tokens bounds/defaults, usage fields incl cache hit/miss and reasoning_tokens |

Long pages were partially truncated at the tail by the fetch tool; all sections used were present. Content treated as external/untrusted data.

## 5. Code reality checks (read-only)

```
backend/crates/hiverouter-core/Cargo.toml:21  hiverouter-provider = { path = "../hiverouter-provider" }
backend/crates/hiverouter-core/src/http.rs:25 use crate::provider_executor::ProviderExecutorAdapter;
backend/crates/hiverouter-core/src/http.rs:67 fn build_executor(...) -> Arc<dyn ProviderExecutor> {
backend/crates/hiverouter-core/src/http.rs:68     if !config.provider.enabled { ... UnconfiguredExecutor }
backend/crates/hiverouter-core/src/http.rs:72     match ProviderExecutorAdapter::from_env(...)
web/src/state/agentState.ts                16 states incl 'interrupted'; header "16 states"
web/src/flow/Canvas.tsx:54,113-122         replay from history.replayGraph -> buildReplayTasks -> buildNodes/buildEdges
```

## 6. Contradiction matrix (release docs vs reality)

| Claim (file:line) | Observed | Source |
|---|---|---|
| RELEASE.md:5,9-17 "verified in the build", provider path live | shipped binary/zip predate provider commit; `docs/RELEASE-EVIDENCE.md` missing | §2 + `Test-Path` false |
| README.md:24 zip `9B2ED5A7…` | actual `BC7A2BAD…`, 6,571,928 B | §2 |
| PHASE-12-EVIDENCE.md:139 zip 4,065,104 B `9B2ED5A7…` | actual 6,571,928 B `BC7A2BAD…` | §2 |
| CLAIM-REGISTER C-340 note zip 4,065,104 B `9B2ED5A7…` | actual 6,571,928 B `BC7A2BAD…` | §2 + CSV parse |
| FINAL-REPORT.md:26 "15-state enum" | HEAD has 16 states | §5 |
| FINAL-REPORT.md:45 trace 260 UNVERIFIED / 5 MISSING | controls.json: 270 PENDING / 6 MISSING / 43 W&W of 319 | controls.json parse |
| FINAL-REPORT.md:51 + README.md:26 "no paid provider run" | live-provider evidence docs dated 10:57 | `docs/evidence/*` (READ) |
| FINAL-REPORT.md:32-47 tests "on the final tree" | logs 08:41/09:21 predate provider commit 10:59 | file mtimes |
| Paper A-F01/F02/F03 P0 | true for shipped artifact; fixed in HEAD source | §5 |

## 7. Non-claims / limits

- Live provider call **not re-executed** (no key access / no paid-API authorization); evidence docs are READ, not VERIFIED-by-S8.
- No build/test/e2e executed (read-only seat; timebox).
- No git write, no product write, no config write; `git status` shows only pre-existing product modifications plus the new `plans/seats/` (this seat's outputs) and `docs/RELEASE.md` (not created by S8).

## 8. Deliverables (this seat)

`plans/seats/SEAT-08/{REPORT.md, READLOG.md, EVIDENCE.md, GATE-CY-1.md, GATE-CY-2.md, GATE-CY-3.md}` — created 11:24–11:32 CT. No other writes.
