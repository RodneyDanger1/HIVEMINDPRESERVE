# Deep Foresight — Run 1 trace and lessons

**Date:** 2026-11-06 · **Repo:** `C:\Users\Riley\Desktop\hiverouter-v1`
**Starting HEAD:** `37f0e7c` · **Source version:** `1.2.0-dev` · **1.1 tag:** `87079ce`
**Objective:** code two Lead-owned runtime gaps (reasoning passback, approval boot expiry/recovery) while four planners produce code-level blueprints for the remaining 1.2 work; independently bug-check the Lead diff; compile a ledger; leave a repeatable workflow.

## Selected Lead items (2–3)

1. Reasoning-content passback: `StreamedReply.reasoning`, `ToolTurnTranscript.reasoning`, adapter continuation `with_reasoning_content`.
2. Approval expiry/recovery: `expire_pending`, boot `expire_orphaned_pending`, run recovery `E_APPROVAL_TIMEOUT`.
3. Production workspace-jail wiring in `http.rs` (already needed a constructor-seam correction).

## Trace (actual commands/artifacts)

| Step | Command / artifact | Invariant? |
|---|---|---|
| Freeze ledger | HEAD `37f0e7c`, version `1.2.0-dev`, clean at start | yes |
| Four planners | `plans/foresight/{CODE-APPROVAL-ENGINE,CODE-SIDE-EFFECTS,CODE-APPROVAL-UX,CODE-PROOF}.md` + four source plans | yes |
| Lead code | `execution.rs`, `provider_executor.rs`, `approval.rs`, `tools_bridge.rs`, `http.rs`, `recovery.rs`, two test files | yes |
| Compile | `cargo check --workspace --tests --locked` → exit 0 | yes |
| Full gate | `cargo test --workspace --locked` → **396 passed / 0 failed** | yes |
| Web gate | `npm.cmd run typecheck` → 0; `npm.cmd test -- --run` → **147/147**; build → green | yes |
| Browser gate | tool-loop + approval-flow specs → **16 passed** | yes |
| Docs/QA | docs checker 19/19; harness 27/27; strict key scan 0 | yes |
| Independent bug-check | `plans/verification/LEAD-BUGCHECK.md`; RED→GREEN twice for boot expiry; approved-resolution timeout mapping | yes |
| Compiler | `plans/foresight/COMPILED-LEDGER.md`; 12 WPs, 13 contradictions, 14 UNKNOWNs, handoff prompt | yes |
| Skill | `C:\Users\Riley\.dsh\skills\deep-foresight\SKILL.md` loads via `skill deep-foresight` | yes |

## Confirmed defects found by the run

1. **`StreamedReply` Eq derive** — `AssembledToolCall` contains `serde_json::Value` and was not `Eq`; compile failed. Fixed by removing the incorrect `Eq` derive (and provider now also carries `Eq` where valid). Lesson: compile before more blind waves.
2. **Test seam jail leak** — wiring the production workspace jail through all three `AppState` constructors made the test-only constructor expose file access; `ws_transport` failed. Fixed with `wire_workspace_jail: bool` (production true, test seam false).
3. **Boot orphan approval** — future-TTL pending approval survived restart after the run was marked `interrupted`; fixed with unconditional `expire_orphaned_pending` at boot. RED: `expired == 0`; GREEN: test passes twice.
4. **Expired refusal mapping** — a row consumed as `expired` returned `E_APPROVAL_REPLAY`; now `E_APPROVAL_TIMEOUT`.
5. **Untrusted teammate edits** — an over-eager UI seat modified three web files outside its remit; typecheck/tests passed but the feature was half-integrated, so it was reverted. Lesson: enforce write scopes; typecheck does not prove feature integration.
6. **SSR zustand test harness** — `renderToStaticMarkup` uses `getInitialState`; seeding via `setState` did not render. Fixed by overriding `getInitialState` in the test helper.
7. **Playwright word-boundary matcher** — `\bOK\b` failed against concatenated DOM text; substring regex fixed the test, not the product.

## Compiler contradictions that need Lead decisions

- Waiter design: `ApprovalWaiters` vs `ApprovalBroker`.
- Side effects default-off vs always-wired; one-shot grant location (engine vs tools).
- `args_preview` needs an additive contract patch; explicitly out of current 1.2 boundary.
- Seal topology: source commit vs seal commit; stricter manifest model required before 1.2 seal.
- Test/status semantics for timeout/cancel/restart and duplicate resolve.

## Next runs

- **Run 2:** implement WP-01/02/03 (approval primitives, side-effect executor, engine suspension) from `COMPILED-LEDGER.md`; then deterministic suspension tests.
- **Run 3:** backend replay + web replay/retry + live tool-turn proof.
- **Run 4:** trace/G-UI/axe closure.
- **Run 5:** strict gate + 1.2 seal + evidence.
- Repeat 3–8 runs; each run must reduce the residual list or name a blocker.

## Deep-skillify handoff

Trace is above. Invariant steps: freeze ledger → four read-only planners → Lead codes 2–3 → independent bug-check → compile reported → next pass. Preconditions: repo with a residual list and file paths, four idle teammates, one independent checker, explicit paid/rebuild policy, no overlapping write scopes. Authoring handoff: `deep-skills` (do not duplicate the skill-authoring path). Lessons appended to `C:\Users\Riley\.dsh\skills\deep-foresight\references\RUN-1-LESSONS.md`.
