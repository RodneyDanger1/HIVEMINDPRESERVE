"""Apply final seat synthesis to the audit paper (sections 4.2/4.3, 6.3, 7.3, abstract/limits)."""
from __future__ import annotations

from pathlib import Path

P = Path(r"C:\Users\Riley\Desktop\hiverouter-v1\audit\HIVEMIND-AUDIT-PAPER.md")
t = P.read_text(encoding="utf-8")

def replace_between(text: str, start: str, end: str, new: str) -> str:
    i = text.index(start)
    j = text.index(end, i + len(start))
    return text[:i] + new + text[j:]

# Abstract counts
t = t.replace(
    "A total of 26 consolidated findings is registered (3 P0, 9 P1, 14 P2), each with file/line evidence.",
    "A total of **30 consolidated findings** is registered (3 P0, 10 P1, 15 P2, 2 informational), each with file/line evidence; "
    "a further 18 gitignored design deliverables (196 MiB) are censused in Appendix D because they sit outside the tracked manifest.",
)

# 4.2 + 4.3 replacement
new_42 = """### 4.2 Empirical findings (consolidated register: `audit/FINDINGS-REGISTER.md`)

Thirty findings were registered (3 P0, 10 P1, 15 P2, 2 informational). Headline items:

- **A-F01 (P0, live-run blocker): the shipped server cannot call the model.** `hiverouter-core/Cargo.toml` depends only on `hiverouter-contract` and `hiverouter-tools`; `core/src/http.rs:77,117,154` constructs an `UnconfiguredExecutor`, so `SUBMIT_PROMPT` never reaches `hiverouter-provider`. Provider crate tests pass in isolation, but the binary run path is unconfigured. Evidence: `backend/crates/hiverouter-core/Cargo.toml`; `core/src/http.rs`.
- **A-F02 (P0, dead replay):** `web/src/flow/Canvas.tsx:113-127` passes `replayNodes = []`; `HISTORY_SNAPSHOT` is stored (`store.ts:379-385`) but never rendered, so the time-travel scrubber cannot display the graph.
- **A-F03 (P0, contract parity):** `contract/state.rs:18-51` declares 16 `AgentState` variants (including `interrupted`); `web/src/state/agentState.ts:8-24` declares 15 and its header wrongly calls the contract a "14-state list".
- **A-F04 (P1):** prompt/persona assets never loaded by the production server (`http.rs:82` `TemplateRegistry::default()`); handshake/context-switch code has no live caller (`ws.rs:338-353`).
- **A-F05 (P1):** Settings renders a refusal from `refusals[0]` instead of the `SETTINGS_STATUS` error (`store.ts:666`).
- **A-F06 (P1):** money overflow silently yields `Decimal::ZERO` in `price()` (`provider/src/accounting.rs:330`).
- **A-F07 (P1):** duplicate major versions resolved in `Cargo.lock` (`thiserror` 1.0.69+2.0.21; `tower-http` 0.6.11+0.7.1); `cargo tree -d` verdict remains `UNVERIFIED`.
- **A-F08 (P1):** headline verification receipts (`final-gate-tests.log`, `run-live-FINAL-artifact.log`, etc.) are **untracked** and therefore absent from the 507-file manifest.
- **A-F09 (P1):** 82 tracked `runtime-test/` artifacts (SQLite DB/WAL/SHM, snapshots, logs), including the shipped binary three times (identical hash), a stale build and a duplicate CLI.
- **A-F10 (P1):** documentation drift: FINAL-REPORT trace counts (260 UNVERIFIED/5 MISSING) disagree with `docs/trace/controls.json` (270 PENDING/6 MISSING); FINAL-DELTA's claim-register FAIL predates its repair; one committed `.last-run.json` says `failed` while closeout says passed.
- **A-F11 (P1):** `.cargo/config.toml` hardcodes absolute host paths for linker/CC/AR.
- **A-F12 (P1):** 18 primary design deliverables are gitignored and outside the tracked manifest (Appendix D censuses them; hashes preserved in A/B manifests and the watchdog ledger).
- **A-F13…A-F24 (P2):** unwired ad-hoc e2e probe scripts; skip-as-green env guard; wiped test logs; trace completeness gaps; tests writing into `docs/`; default baseURL mismatch; unbundled fonts; dev mock statically imported; missing HTTP timeouts; favicon hex; optional approval nonce; stale docs.
- **FA-F2…FA-F6 (P2/info):** ASSETS-INDEX drops all B export hashes; index watchdog block stale vs ledger; B episode schema non-standard; tracked CPython-3.14 `.pyc`/scratch files; iterative patch scripts tracked.

### 4.3 Seat results (empirical)

| Seat | Coverage | Deep | Quality mean (correctness / docs) | Top finding |
|---|---:|---:|---|---|
| b-core | 54/54 (sha 54/54) | 21/21 | 4.85 / 4.81 | A-F01 provider not wired |
| b-provider | 33/33 (sha 33/33) | 15/15 | 4.55 / 3.58 | A-F06 silent zero cost overflow |
| watchdog-1 | 55/55 (sha 55/55) | 16/16 | 4.47 / 3.98 | A-F02 dead replay; A-F05 wrong setting refusal |
| watchdog-2 | 122/122 (sha 122/122) | 4/4 | 3.74 / 2.51 | A-F09 runtime artifacts; harness gaps |
| figcraft-b | 91/91 (sha 91/91) | 0 | 4.20 / 2.97 | A-F10 docs/claims drift |
| figcraft-a | 103/103 (sha 103/103) | 0 | 4.50 / 3.31 | A-F12 ignored deliverables; index hash gaps |
| ui-watchdog | 49/49 (sha 49/49) | 0 | 3.92 / 3.69 | A-F11 absolute paths; provenance/licence gaps |
| strip-watchdog | independent 507/507 hashes verified | — | — | verification report in `audit/parts/strip-watchdog/VERIFICATION.md` |

All **507/507** files are accounted for by the 7 analytic seats plus Lead census and the independent verifier. Per-seat records: `audit/parts/<seat>/FILES.md`.
"""
t = replace_between(t, "### 4.2 Empirical findings", "---\n\n## 5. Upstream Provenance", new_42)

# 6.3 quality replacement
new_63 = """### 6.3 Quality scoring (final)

Per-file scores were collected for all **507** files and merged into `audit/QUALITY-MATRIX.csv` (`audit/tools/merge_quality.py`). Code-only aggregates (184 of 188 code-classified files scored; 4 binary/C fixtures not scored) are:

| Dimension | Mean /5 |
|---|---:|
| correctness | 4.43 |
| maintainability | 4.25 |
| modularity | 4.32 |
| error handling | 4.15 |
| security | 4.11 |
| documentation | 3.84 |

Seat means (all files) are in `audit/QUALITY-SUMMARY.json`; code-only per-seat table in `audit/QUALITY-CODE-STATS.md`. **No score is assigned without cited evidence**, and the scores measure the audited tree, not a coverage guarantee: docs/assets drag the all-file means down (3.34 documentation), while core Rust scores highest (b-core 4.85/4.81). The scoring does not mask the P0/P1 findings above — a high maintainability score is not a correctness claim.
"""
t = replace_between(t, "### 6.3 Quality scoring", "---\n\n## 7. Refactoring", new_63)

# 7.3 waves replacement
new_73 = """### 7.3 Prioritised waves (driven by the audit findings)

- **Wave 1 (P0 — correctness):** wire `hiverouter-provider` into `hiverouter-core` (A-F01) and add a live/mocked run-path test; render `HISTORY_SNAPSHOT` replay (A-F02); fix `interrupted` state parity in the UI + contract drift test (A-F03).
- **Wave 2 (P1 — contract/hygiene):** prompt/persona loading + handshake wiring (A-F04); settings refusal source (A-F05); Decimal overflow refusal (A-F06); duplicate-major gate `cargo tree -d` (A-F07); track or regenerate headline evidence (A-F08); remove/relocate 82 runtime artifacts + duplicate binaries (A-F09); reconcile stale docs/claims (A-F10); parameterise `.cargo/config.toml` (A-F11); fix design-artifact tracking policy (A-F12).
- **Wave 3 (P2 — verification/UX):** wire the six ad-hoc probe scripts or delete them; make skip conditions fail-closed; persist test logs; implement/declare the 12 DOM G-UI gates; normalise baseURL/artifact preflight; bundle fonts; remove dev mock from production bundle; HTTP timeouts; token-isolation scope; approval nonce in the frozen contract; regenerate ASSETS-INDEX with all B hashes.
- **Wave 4 (P3 — maintainability):** remove tracked `.pyc`/scratch/patch intermediates, consolidate documentation, decide S25 code-diff and Tauri/installer futures.

Acceptance gates per wave are recorded in `ROADMAP.md`; every task requires a command or `path:line` receipt before it can be marked complete. The 101-task roadmap table (`ROADMAP-TASKS.csv`) remains the execution schedule; the findings above are the prioritized backlog it should be sequenced against.
"""
t = replace_between(t, "### 7.3 Prioritised waves", "---\n\n## 8. Conclusion", new_73)

# Conclusion
old_concl_start = t.index("HiveMind v1 is a genuinely working")
old_concl_end = t.index("\n\n---\n\n## Appendix A", old_concl_start)
new_concl = """HiveMind v1 is a compact, unusually well-instrumented localhost application: a frozen Rust protocol, a real React UI, a packaged CLI, 233 backend tests, 79 live-browser tests, and an audit trail that names its own unverified regions. This audit makes the claim checkable: every one of the 507 tracked files is hashed and assigned, all 56 deep files are analysed, 507 files are quality-scored, dependencies and provenance are mapped, and the defects are stated plainly rather than polished away. The decisive finding is that **the shipped binary does not yet wire the provider run path** (A-F01), so v1.0.0 is a verified shell/transport/UI release rather than a verified end-to-end model runtime; the second is that replay rendering (A-F02) and state parity (A-F03) are incomplete. The right sequence is therefore: wire the provider, revive replay, fix parity, then clean the verification/hygiene debt. The codebase's strongest asset — its refusal-first, no-fake-success discipline — is exactly what makes this honest audit possible and what the roadmap must preserve."""
t = t[:old_concl_start] + new_concl + t[old_concl_end:]

# Limitations appendix add ignored class
t = t.replace(
    "- Ignored/generated trees are censused, not individually hashed or prose-analysed.",
    "- Ignored/generated trees are censused, not individually hashed or prose-analysed; 18 gitignored primary design deliverables (196 MiB) are separately censused in `audit/APPENDIX-IGNORED-DELIVERABLES.md` with recomputed hashes.",
)
# Appendix A add new artifacts
t = t.replace(
    "| `audit/QUALITY-MATRIX.csv` | merged quality scores |",
    "| `audit/QUALITY-MATRIX.csv`, `QUALITY-SUMMARY.json`, `QUALITY-CODE-STATS.md` | merged quality scores and code-only aggregates |\n| `audit/FINDINGS-REGISTER.md` | consolidated 30-finding register (P0/P1/P2) |\n| `audit/APPENDIX-FILE-CENSUS.md`, `APPENDIX-IGNORED-DELIVERABLES.md` | 507-file census; 18 gitignored design deliverables |",
)
P.write_text(t, encoding="utf-8")
print("paper synthesis applied; bytes", len(t))
