# Round 20 bug sweep (offline)

Skills applied: `/forge`, `/integration-logic`, `/logic-gatekeeper`,
`/evidence-check`, `/evidence-gap-audit`. No server/browser/test/provider run.

## Phase 1

- Architecture: run/task graph + scoped contexts + durable A2A rows; UI reads a server projection.
- Data flow: parent context facts -> child context; A2A rows -> recipient context; live run -> memory panel.
- Root cause: derived UI state and inherited context facts crossed run/recipient boundaries.

## Findings and fixes

| Finding | Root cause | Fix | Evidence |
|---|---|---|---|
| Memory panel could show previous run rows while a new run loaded | rows were neither cleared nor gated on status | clear on load; render only `ok` | source; typecheck clean |
| Replaying a historical graph showed live-run memory | panel keyed only on live `run.id` | replay hides live scoped memory and clears rows | source; typecheck clean |
| A2A inbox facts leaked from parent to child | child inherited all parent context facts | inherited facts now exclude `a2a:` and `roster:` keys | source |
| Clearing persona memory left run section stale | only role view refreshed | both views refresh | source |

## Claims

| Claim | Grade | Evidence |
|---|---|---|
| UI no longer shows stale previous-run rows | VERIFIED source | row clear + `ok` gate |
| Replay hides live scoped memory | VERIFIED source | `replaying` gate |
| Child cannot inherit parent A2A/roster facts | VERIFIED source | filtered inheritance |
| Typecheck | VERIFIED | `npm run typecheck` clean |
| Live behavior | UNVERIFIED | no runtime testing authorized |
| Rust compile/tests | UNVERIFIED | no test run |

## Gaps (not fixed)

- No full A2A answer/wake/deadline.
- A2A message privacy relies on context filtering, not per-recipient storage ACL.
- `/api/*` still Host-only, not token-authenticated.
- Scoped memory clear is persona-wide, not branch-specific.
- No live UI/render verification.
