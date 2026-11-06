# GATE-CY-2 — Planner P2, cycle 2: recon → plan → lead (the six focus areas)

Date 2026-11-06 ~11:3x CST · HEAD `5c69b66` (tree dirty per S1/S6/S7/S8) · read-only.

## 1. RECON (per focus area; all claims trace to seat file:line)
1. **16-state parity + failed-terminal.** Contract has 16 `AgentState` incl `Interrupted`, 5 terminal incl
   `failed` (`contract/state.rs:18-72,97-109`; `audit/diagrams/04-state-inventory.mmd:5`). UI enum/labels are
   16 (S3-V1) but `isTerminal` omits `failed` (`web/src/state/agentState.ts:111-118`) → failed runs stay
   "active": spawn/rewire/cancel gates pass locally (S3-D1, `store.ts:434-441,550-557`) and streaming indicator
   never clears (`store.ts:254`). Contract/UI parity therefore holds on **count** but fails on **terminality**.
2. **Replay read-only + graph shape.** HISTORY_SNAPSHOT is stored and projected (S3-V2; `store.ts:385-393`,
   `Canvas.tsx:113-136`) and malformed/unknown nodes are guarded (S3-V3; `graph.ts:93-131`), but replay nodes
   are `draggable/selectable` and `onNodeClick` mutates live selection (`Canvas.tsx:146`, `graph.ts:64-72`,
   S3-D2), and the payload is `graph: unknown` with silent empty-canvas degradation and **no replay test**
   (`contract.ts:235-239`, `graph.ts:93-100`, S3-D3/D7).
3. **Settings truthfulness.** SETTINGS_STATUS returns provider/model/`key_present` bool + optional
   thinking/effort; key never read/printed (S2 §1.7; `tools/src/settings.rs:18-26`, `core/ws.rs:397-415`);
   UI decodes bool only and renders all five fields, one request per open (S3-V4/V5). Residual: additive
   fields not in the frozen doc (S2 F-S2-10, INFO); copy fallback bug only when server returns empty
   (S3-D4, P2).
4. **45/65 UI refusal codes.** Backend `Refusal::ALL` = 65 (`contract/refusal.rs:73-282`), UI
   `KNOWN_REFUSAL_CODES` = 45 (`web/src/api/contract.ts:332-378`) — 20 codes missing (S2 F-S2-01), so UI copy
   for those refusals cannot be canonical; wire ERROR.code is a free String and UI accepts unknown codes
   (S2 F-S2-09).
5. **NODE_REWIRE hardcoded state.** `core/src/ws.rs:344-352` publishes rewire events with
   `AgentStateForEvent::running()` (helper 483-490): a rewire of a queued/terminal task broadcasts `running`
   (S2 F-S2-02) — a state-truth defect on the same wire the UI trusts.
6. **Key/refusal security.** First-party key custody is clean: env-only, Zeroizing, `ApiKey(***)`,
   header-only use, bool-only APIs, audit redaction (S2 §2, S6 V1–V8; `secret.rs:20-81`,
   `client.rs:269-279`, `settings.rs:18-26`, `health.rs:59-60`, `audit.rs:31,247`). **But S6-F-01 (P1):**
   spawned tool children inherit the environment (`shell.rs:201-208`, `cli_support.rs:41`, `jail.rs:389`;
   zero `env_clear`/`env_remove`) → any executed child can read `DEEPSEEK_API_KEY`. Evidence docs are
   self-reported with no commit/binary hash (S6-F-02) and the paper/verifier baseline predates HEAD
   (S6-F-03/S7-F03). Residual LOW: token compare timing, `/health` DB path, decoder default (S2 F-02/03/06).

## 2. PLAN (decisions, each with rejected alternative)
- **P2-D1 failed-terminal:** make UI terminality match the contract (add `failed` to `isTerminal`; keep
  `interrupted`). Rejected: leave failed "active for retry" — there is no retry command in the frozen
  vocabulary, so the affordance only enables server-refused actions (fake affordance).
- **P2-D2 replay read-only:** build a replay-specific projection (`draggable:false, selectable:false`,
  no-op click) rather than toggling CSS on live nodes. Rejected: CSS `pointer-events:none` — drag/select
  state can still mutate.
- **P2-D3 graph-shape pin:** add a typed `HistorySnapshotGraph` + `parseReplayGraph` validator (nodes with
  required task_id/state/rev; unknown state skipped but reported; malformed graph → named client refusal + banner
  instead of silent empty canvas). Rejected: trust the backend blindly (drift hides a real failure).
- **P2-D4 settings truth:** keep key-presence boolean contract; reconcile additive `thinking`/`effort` fields
  in the frozen doc/contract comment (document as optional additive) and fix copy fallback to `notReported`.
  Rejected: remove thinking/effort from the event (UI already renders them and S3 tests cover them).
- **P2-D5 refusal-table parity:** generate the UI refusal table from the backend enum at contract-test time
  (a checked-in generated artifact) instead of hand-copying, and add a drift test. Rejected: hand-edit 45→65
  (will drift again); rejected: runtime fetch of codes (no such endpoint, adds failure mode).
- **P2-D6 NODE_REWIRE truth:** read the task's real state from `GraphAuthority` after a successful rewire and
  emit that state; if the graph has no state accessor, add a read-only accessor (core owner) rather than
  synthesising `running`. Rejected: keep `running` (wire lie).
- **P2-D7 subprocess env isolation:** `env_clear()` + explicit allowlist (`PATH`, `SystemRoot`, `TEMP`,
  `HIVEROUTER_*` non-secret) before spawning tool children, with a regression test that a child cannot see
  `DEEPSEEK_API_KEY`; document the boundary honestly if the Lead rejects env_clear for allowlist reasons.
  Rejected: rely on audit redaction (does not stop child exfiltration).
- **P2-D8 HEAD-bound evidence:** no "closed" claim without a gate log carrying HEAD + command + exit code
  (S7-F04); release must bind RELEASE.md/CLAIM-REGISTER zip hashes to the actual rebuilt artifact (S8-S-2/3/4).
  Rejected: reuse pre-integration numbers (S8 shows they are stale).
- **P2-D9 tests:** each decision has a discriminating test (below); a skipped e2e is not a pass; explicit
  UNVERIFIED where browser/live is unavailable.

## 3. LEAD (check)
- Every focus area now has a decision, a rejected alternative, an owner file list, and a test; no decision
  requires a wire-vocabulary change except the additive settings-field documentation note (contract owner).
- No product write planned in this seat; changes are owned by the future remediation wave (frontend, core,
  tools/security) with disjoint files.
- **Cycle 2 verdict: PASS-WITH-NOTES** — P2-D7 (env_clear) and P2-D3 (snapshot typing) are the two items whose
  approval needs the Lead's security/contract sign-off; they are recorded as blockers in PLANNER-02 §8.
