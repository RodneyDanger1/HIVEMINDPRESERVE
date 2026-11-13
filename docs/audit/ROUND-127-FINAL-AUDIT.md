# Round 127 — consolidated audit, fixes, and roadmap

**Source:** `integration/capability-absorb` @ `4a3a0ff` (product package `4a3a0ff`).
**Schema:** v32 (`session_archive`). **Auditors:** four read-only seats (screen/UI, plan-vs-code,
test evidence, truthseeker). No seat committed or edited the repo.

This report is point-in-time evidence. Re-run the cited commands before treating any status as
current.

## 1. Verified product facts

| Fact | Evidence |
|---|---|
| Schema head v32, single head, v24→v32 migration | full backend gate PASS; session truth 21 passed |
| Sessions: list/create/detail/policy/todos/archive/unarchive | `http.rs` session routes; live archive 200 / unknown archive 404 |
| Eval: executed receipts, suites, metrics | full gate; live walkthrough eval 1/1 |
| Lessons: two comparable passing receipts, activation/rollback | session truth tests |
| Approvals: persisted bounded preview for replay | tools/core tests; 18 web preview tests |
| Jobs: allowlisted start/kill, UI console, CLI read | live `git --version` in UI, exit 0 |
| CLI: start/stop/run/submit/approve/deny/jobs/sessions | packaged `hivemind.cmd`; live session create/policy/archive |
| UI: splash, workspace, sessions, jobs, mobile bottom nav | Playwright 1440 + 375, zero console errors |
| Package: installer/uninstaller shipped, checksum manifest excludes itself | 12 manifest entries, 0 bad |

## 2. Audit findings and dispositions

### Screen/UI (all fixed)
- Mobile composer covered by canvas ControlBar → ControlBar hidden while chat open; composer
  hit-test now returns `#composer`.
- Fixed-height chat clipped content → mobile auto-height/max-dvh; controls remain reachable.
- Scrubber under bottom nav → mobile `bottom-14`.
- Seven bottom-nav buttons unnamed → `aria-label`; names verified.
- Archive control disabled with no reason → title/description.

### Plan-vs-code (partially fixed; remainder roadmap)
- CAP-4 workbench PARTIAL: FileTree/Editor/Timeline mounted; DiffView/Terminal/TodoPanel/
  SessionRail/CostBar/CommandPalette still export-only. No CodeMirror/xterm/PTY.
- CAP-5 PARTIAL: sessions/checkpoints/jobs real; no mid-run resume; no terminal session tables.
- CAP-6 PARTIAL: reasoning captured in memory, not persisted or displayed per turn.
- CAP-7 PARTIAL (weakest): no compaction, no verify-after-mutation, retry cap is prompt-only,
  no todo_write/update_plan tool.
- CAP-8 PARTIAL: skills/CLI/MCP-JS real; no Rust MCP module; no per-session model column.
- CAP-9 PARTIAL: eval/lessons/suites real; no non-inferiority statistic; vectors deferred.
- CAP-10 IMPLEMENTED: DB/WAL/SHM denial, jail, plan/trusted policy tests.

### Test evidence (gaps corrected)
- Batteries previously SKIPPED the backend suite. Final gate was run and passed at `4a3a0ff`;
  offline battery receipt committed for v32 web/docs/QA/health.
- Walkthrough schema pin raised to `>= 32`.
- Archive UI now has a regression pin (`workbenchMount` archive/nav source assertions).
- INDEX refreshed to current identity; stale hashes removed.
- `CHECKSUMS.txt` no longer hashes itself; 12 entries validate.

### Truthseeker (all fixed)
- Unknown-session archive returned 503 → now 404.
- CLI could not list archived sessions → `sessions --include-archived`.
- README/INDEX still said v31/44207bf → v32/308f357→4a3a0ff.
- Plan table claimed unshipped v27-v32 tables → corrected to shipped DDL.
- `dist/HiveMind.zip` stale (806251b): rebuild zip when a release archive is required.
- Archive is last-write-wins (repeated archive rewrites timestamp); documented, not treated as
  idempotent.

## 3. Remaining roadmap (priority order)

1. **CAP-7 loop quality (P1):** compaction module; enforced retry counter; verify-after-mutation
   evidence; todo/update_plan tool ids. This is the largest gap between “coding harness” and
   HiveMind's current loop.
2. **Terminal/PTY (P1/P2):** ConPTY design and pinned xterm; keep the current command console
   explicitly non-PTY until then.
3. **CAP-4 workbench wiring (P2):** mount DiffView and TodoPanel against existing APIs; add
   hunk-level review for patch approvals (preview is currently bounded JSON).
4. **CAP-8 MCP (P2):** local stdio MCP over the existing registry/policy; add CLI job start/kill.
5. **CAP-9 non-inferiority (P2):** compare baseline/candidate eval statistics before lesson
   activation; vectors optional.
6. **CAP-12 release:** rebuild `HiveMind.zip`, fresh-profile install with a real key, live keyed
   coding run and approval click-through.

## 4. Not done / unverified

- No keyed provider run in this session (`E_KEY_MISSING`); approval overlay with a real patch,
  non-inferiority, and full keyed coding remain unverified.
- No PTY, no per-turn reasoning persistence/UI, no compaction/retry enforcement.
- Three earlier audits from the prior batch terminated before returning; this round's four seats
  all returned.
