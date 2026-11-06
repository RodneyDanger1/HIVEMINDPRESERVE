# PLANNER-02 — Contract / Security / UI remediation plan (HiveMind 1.0)

**Planner:** P2 (Contract-Security-UI) · **Mode:** read-only planning; writes only `plans/planners/PLANNER-02/**`.
**Repo:** `C:\Users\Riley\Desktop\hiverouter-v1` · **HEAD observed:** `5c69b66` (dirty tree; 8+ paths — S1/S6/S7/S8).
**Evidence base:** all 8 seat REPORT/READLOG/EVIDENCE/GATE sets read in full (S1–S8, 48 files), audit paper
(HEAD `3988622` baseline) re-read in full, 4 official DeepSeek docs re-fetched (2 truncated tails recorded).
**Seat provenance for every finding below:** S2 (contract-security), S3 (UI state/replay), S6 (provenance/key
security), with S1 (provider wiring), S7 (verification), S8 (release-doc reality) as corroboration.
Gate cycles: `GATE-CY-1/2/3.md` (recon→plan→lead each; CY-2 = PASS-WITH-NOTES, no BLOCKED cycle).

---

## 1. Outcome

After the remediation wave: (a) the UI treats every terminal contract state — including `failed` — as terminal,
so no mutating control offers an action the server will refuse; (b) `HISTORY_SNAPSHOT` replay renders a
read-only graph whose shape is pinned by a parser and by e2e, and a malformed snapshot produces a named refusal
banner instead of a silent empty canvas; (c) `SETTINGS_STATUS` remains truthful (provider/model/key-presence
only, never the key) with its additive thinking/effort fields documented and its copy fallbacks correct;
(d) the UI carries all **65** contract refusal codes from one generated source, so every server refusal renders
its canonical code/message/how_to_fix; (e) `NODE_REWIRE` no longer lies about state; (f) the tool-execution
boundary cannot leak the DeepSeek key to spawned child processes; (g) every claim above is pinned by a named
test, and any claim that cannot be executed is marked UNVERIFIED.

## 2. Current behavior (verified — seat evidence)

| # | Behavior | Evidence (seat + path:line) |
|---|---|---|
| C1 | Contract/agent-state vocabulary = **16** incl `Interrupted`; terminal set includes `failed` | S2 §1.4 (`contract/state.rs:18-72,97-109`); S3-V1; S3 diagram `04-state-inventory.mmd:5` |
| C2 | UI vocabulary = 16 (labels+enum) but **`isTerminal` omits `failed`** → failed runs count as active | S3-D1 (`web/src/state/agentState.ts:111-118`; `store.ts:434-441,550-557,254`) |
| C3 | Replay arm exists: HISTORY_SNAPSHOT stored → `buildReplayTasks` → nodes/edges → ReactFlow | S3-V2 (`store.ts:385-393`, `Canvas.tsx:113-136`, `graph.ts:93-131`) |
| C4 | Replay is **not read-only**: nodes draggable/selectable; `onNodeClick` mutates live selection | S3-D2 (`Canvas.tsx:146`; `graph.ts:64-72`) |
| C5 | Snapshot shape is **unpinned** (`graph: unknown`), malformed → silent `{}`/empty canvas, no replay test | S3-D3 (`contract.ts:235-239`; `graph.ts:93-100`; `web/tests/*` no replay test) |
| C6 | SETTINGS_STATUS carries provider/model/`key_present` bool + additive thinking/effort; key never read | S2 §1.7 (`tools/src/settings.rs:18-26`; `core/ws.rs:397-415`); S3-V4/V5; S6-V1–V5 |
| C7 | UI refusal table = **45** codes; contract enum = **65**; wire `ERROR.code` is free String | S2 F-S2-01/09 (`contract.ts:332-378` vs `contract/refusal.rs:73-282`) |
| C8 | `NODE_REWIRE` broadcasts hardcoded `AgentState::Running` regardless of real task state | S2 F-S2-02 (`core/src/ws.rs:344-352,483-490`) |
| C9 | Tool subprocesses inherit the full server environment (no env_clear/env_remove) → key can leave process | S6-F-01 (`tools/src/shell.rs:201-208`; `cli_support.rs:41`; `jail.rs:389`) |
| C10 | First-party key custody clean (Zeroizing, `ApiKey(***)`, header-only, bool-only APIs, audit redaction) | S2 §2, S6-V1–V8 (`secret.rs:20-81`, `client.rs:269-279`, `settings.rs:18-26`, `health.rs:59-60`, `audit.rs:31,247`) |
| C11 | Contract parity counts: 14 frozen commands (+3 DIFF decision-gated), 21 events, 65 refusals, 16 states | S2 §1 (`command.rs:26-83`, `event.rs:18-87`, `refusal.rs:73-282`, `state.rs:18-72`) |
| C12 | HEAD source fixes paper P0s A-F01/F02/F03, but **no gate log is HEAD-bound**; tree dirty | S1 E-2/E-3; S3-V10; S6-F-03/F-04; S7-F01/F03/F04; S8 §1 |
| C13 | Release docs contradict the actual artifacts (zip 4,065,104 B `9B2ED5A7…` vs actual 6,571,928 B `BC7A2BAD…`; 15 vs 16 states; trace 260/5 vs 270/6) | S8-S-2/3/4/5/6 (`README.md:24`, `PHASE-12-EVIDENCE.md:139`, `CLAIM-REGISTER` C-340, `FINAL-REPORT.md:26,45`) |

## 3. Decisions (each with rejected alternative, contract/ownership impact)

- **D1 — `failed` is terminal locally (S3-D1).** Add `failed` to `isTerminal`; keep `interrupted`. Rejected:
  leave failed "active for retry" — no retry command exists in the frozen 14-command vocabulary, so the only
  effect is enabling commands the server refuses (fake affordance). Contract impact: none (UI predicate).
- **D2 — replay is a read-only projection (S3-D2).** New replay builders set `draggable:false,
  selectable:false`; replay click is a no-op; live selection untouched. Rejected: CSS-only disable (drag/select
  internals still mutate). Contract impact: none.
- **D3 — pin the snapshot graph shape (S3-D3).** Add `HistorySnapshotGraph` typing + `parseReplayGraph()`:
  require `nodes[]` entries with `task_id`, `state`, `rev`; skip unknown states with a counted report; a graph
  present but unparsable → named client refusal (`E_SCHEMA_INVALID`-class) + ReplayBanner error, never an empty
  canvas. Rejected: trust backend blindly (silent data loss on drift). Contract impact: additive client-side
  type; `HistorySnapshotPayload.graph` may remain `unknown` on the wire, parsed once.
- **D4 — settings stays boolean-truthful (S2/S3/S6).** Keep provider/model/`key_present`; document
  `thinking`/`reasoning_effort` as optional additive fields in the frozen event doc and UI contract comment;
  fix copy fallback (`provider`/`model` use `notReported`, not `keyUnknown`). Rejected: drop thinking/effort
  (UI already renders them and tests cover them); rejected: add balance/trust fields (needs a new provider
  endpoint; not frozen).
- **D5 — refund table from one generated source (S2 F-S2-01, P1).** Emit `refusal-codes.generated.json` from
  `Refusal::ALL` in a contract-crate test (code/message/how_to_fix/retryable) and have `contract.ts` import the
  generated table; a drift test fails the build when enum and table differ. Rejected: hand-edit 45→65 (drifts
  again); rejected: runtime fetch (no endpoint; adds failure mode). Contract impact: none (UI artifact only).
- **D6 — NODE_REWIRE carries the real state (S2 F-S2-02, P1).** After a successful rewire, read the task/run
  state from `GraphAuthority` and pass it to `publish_node_event`; remove the `AgentStateForEvent::running()`
  synthesis. Rejected: keep `running` (wire lie; UI state and audit disagree). Contract impact: none (existing
  `AgentState`); may need a read-only graph state accessor (core owner).
- **D7 — subprocess env isolation (S6-F-01, P1).** Before spawning any tool child: `env_clear()` then set an
  explicit allowlist (`PATH`, `SystemRoot`, `TEMP`/`TMP`, locale vars, `HIVEROUTER_*` non-secret) — never
  `DEEPSEEK_API_KEY`; add a regression test spawning a child that prints its env and asserting the key is
  absent; same seam for `cli_support::version_probe` and the jail test helper. If the Lead rejects
  `env_clear` (breaking legitimate tools), document the boundary as an accepted risk in the release notes and
  redact child output before ledger writes. Rejected: rely on audit redaction (does not stop exfiltration).
- **D8 — HEAD-bound evidence + release-doc truth (S7-F04, S8).** Every "closed" claim carries a gate log with
  HEAD + command + exit code (or JUnit XML); RELEASE.md/CLAIM-REGISTER/PHASE-12 hashes must be generated from
  the rebuilt artifact, and the pre-integration `184D51A7…` artifact is labelled superseded. Rejected: reuse
  pre-integration numbers. Contract impact: documentation/claims only.
- **D9 — no wire-vocabulary change** in this plan: 14/21/65/16 stay frozen (S2 §1 C11); only client-side
  mirrors, state truth, and documentation are added. Any need for a new refusal code goes through the contract
  change process (CHANGE-LEDGER-IMPL + matrix test), not this plan.

## 4. File-by-file change (owner + exact functions/edge cases/error paths)

### UI / contract (owner: frontend window)
1. `web/src/state/agentState.ts` — `isTerminal` (111-118): add `failed`; keep `interrupted`; keep
   `isRunActive` derivation and document `interrupted` inbound transitions; tests below. Edge: `failed` +
   streaming → streaming indicator cleared (`store.ts:254` already keys on `isTerminal`).
2. `web/src/flow/graph.ts` — export `buildReplayGraph(graph: unknown): {tasks,nodes,edges,malformed,skipped}`
   reusing `buildReplayTasks` validation (93-131); replay nodes `draggable:false, selectable:false`; never
   invent kind/label beyond documented fallback (S3 V3); malformed flag when graph present but no valid nodes.
3. `web/src/flow/Canvas.tsx` — replay branch (113-136) uses the read-only projection; `onNodeClick` (146) is a
   no-op in replay; banner error copy when `malformed` (StatePack ReplayBanner); no live `selectTask` call.
4. `web/src/api/contract.ts` — add `HistorySnapshotGraph`/`parseReplayGraph` (235-239 stays wire-unknown);
   replace `KNOWN_REFUSAL_CODES` (332-378) with the generated 65-code table; keep tolerant decoder but mark
   unknown codes as `unknown_code` in strict mode (507-519). Edge: future new state → skipped + counted, no throw.
5. `web/src/state/store.ts` — no API break; consume corrected `isTerminal`; expose `replayMalformed` flag;
   consolidate contract imports (S3-D11, P3, optional).
6. `web/src/panels/SettingsPanel.tsx` / `web/src/state/strings.ts` — provider/model fallback → `notReported`
   (S3-D4); ensure one SETTINGS_STATUS per open remains (S3-V5).
7. `web/src/api/client.ts` / `web/src/api/health.ts` — add `AbortSignal.timeout` to `/api/session` and
   `/health` fetches, map `AbortError` → named `E_WS_DISCONNECTED`/`E_SCHEMA_INVALID` (S3-D5, P2).
8. `web/src/chat/ChatCore.tsx` — clear draft only after accepted submit (S3-D9, P3).

### Backend / contract (owner: core window)
9. `backend/crates/hiverouter-contract/tests/*` — new test emitting `web/src/api/refusals.generated.json` from
   `Refusal::ALL` (65 rows); golden/pin test fails on drift with the OLD→NEW diff. Edge: duplicate code/message
   must fail the pin (S2 §1.3 unique test already exists).
10. `backend/crates/hiverouter-core/src/ws.rs` — NODE_REWIRE (344-352): fetch real state via graph accessor
    (add `GraphAuthority::task_state(&TaskId) -> Option<AgentState>` if absent) before `publish_node_event`;
    remove `AgentStateForEvent::running()` helper (483-490). Edge: unknown task after rewire → ERROR
    `E_TASK_NOT_FOUND`, no NODE_REWIRE (never lie).
11. `backend/crates/hiverouter-tools/src/shell.rs` / `cli_support.rs` / `jail.rs` — env allowlist helper
    (`commands::env_allowlist()` or shell-private fn) with `env_clear()` + explicit vars; regression test spawns
    `cmd.exe /C set` (or a tiny child) and asserts no `DEEPSEEK_API_KEY`; never log child env.
12. `backend/crates/hiverouter-tools/src/health.rs` / `core/src/http.rs` — `/health` DB path redaction
    (basename only, S2 F-S2-05, P2); token compare constant-time (S2 F-S2-04, P3) if cheap; rename
    "per-session" docs to boot token (S2 F-S2-03, P3).
13. `web/src/api/contract.ts` decoder for FILE_TREE_RESULT strictness (S2 F-S2-06, P3).

### Evidence / release docs (owner: Lead, per S8 plan)
14. `docs/RELEASE-EVIDENCE.md` (new) + `RELEASE.md`/`README.md`/`PHASE-12-EVIDENCE.md`/`CLAIM-REGISTER` zip
    hashes regenerated from the rebuilt artifact (S8-S-1…S-4); `audit/HIVEMIND-AUDIT-PAPER.md` supersession
    banner for A-F01/F02/F03 at HEAD (S8-S-9); gate logs must carry HEAD + exit code/JUnit (S7-F04).

## 5. Failure modes

- **Malformed/unknown snapshot:** replay shows named banner + keeps live graph; unknown states skipped and
  counted; no exception into React render.
- **`failed` event with stale streaming text:** streaming cleared via corrected terminality; controls show
  disabled reasons; no spawn/rewire offered on a failed run.
- **Settings field missing:** renders `notReported` (never 0/keyUnknown); key never crosses API (bool only).
- **Unknown server refusal code:** UI shows the raw code + generic fix and marks it unknown (no crash, no
  dropped refusal); generated-table drift fails the build before release.
- **NODE_REWIRE on missing/terminal task:** ERROR event with named refusal; no fabricated `running` state.
- **Spawned child attempts to read env:** `env_clear()` allowlist means the key is absent from the child
  environment (regression test); child stdout still passes through audit redaction.
- **Headless/no-browser planning:** runtime replay read-only/e2e claims stay UNVERIFIED until the e2e seat runs
  the commands in §6; the plan does not convert source-read into a pass (S3/S7 rule).

## 6. Tests (discriminating, named, with commands)

Contract/backend (`cd backend`; PATH includes cargo + tools/w64devkit/bin):
- `cargo test -p hiverouter-contract` — new `refusal_codes_generated_artifact_matches_enum` (65 rows; drift
  fails) + existing unique/roundtrip/matrix tests.
- `cargo test -p hiverouter-core --test ws_transport` — add `rewire_terminal_task_emits_real_state`
  (rewire a terminal task → NODE_REWIRE state ≠ running; unknown task → `E_TASK_NOT_FOUND`).
- `cargo test -p hiverouter-tools --lib shell::tests` — add `spawned_child_cannot_see_provider_key` (spawn child
  that prints env; assert no `DEEPSEEK_API_KEY`); keep timeout/cancel tests.
- `cargo test -p hiverouter-tools --lib health` — `session_token_not_in_health`/DB path redaction (P2).

UI (`cd web`; `npm.cmd run typecheck; npm.cmd test -- --run`):
- `agentState.test.ts` — assert exact **16** states, `isTerminal('failed') === true`,
  `isTerminal('interrupted') === true`, `isTerminal('running') === false`; `isRunActive('failed') === false`.
- `graph.replay.test.ts` (new) — valid snapshot → nodes/edges with replay nodes `draggable:false,
  selectable:false`; malformed graph → `malformed=true` + named banner state; unknown state skipped/counted;
  orphan parent detached (S3 V3 behaviour preserved); replay click does not call live `selectTask`.
- `contract.test.ts` — generated refusal table has 65 codes and equals `Refusal::ALL`; unknown code → marked
  unknown; SETTINGS_STATUS decode requires provider/model/key_present, optional thinking/effort.
- `store.test.ts` — SETTINGS_STATUS once per open + refusal attributed to its own toast (S3-V5 preserved);
  HISTORY_SNAPSHOT stored; failed run gates refuse spawn/rewire/cancel.
- `client.timeout.test.ts` — never-resolving `/api/session`/`/health` fetch rejects by named refusal within
  the timeout (fake timers).

E2E (`cd e2e`; `playwright test --project=chrome` against the rebuilt binary, `E2E_APP_READY=1`):
- replay scenario: scrub → HISTORY_SNAPSHOT → replay banner + non-empty nodes; drag attempt does not move the
  node and click does not change live selection (S3-D2 pin).
- state parity: inject/observe a failed run → replay/terminal affordances reflect terminality (or record
  UNVERIFIED if the run cannot be produced without a paid call; a stub-provider path is preferred).
- refusing commands on a failed run: spawn/rewire controls disabled with the reason copy (no fake affordance).

Release gate (Lead): HEAD-bound logs per S7 §5, rebuild `cargo build --release --locked`, regenerate
`RELEASE-EVIDENCE.md`, re-hash zip, re-run doc-truth checks from S8 §6.

## 7. Rollback

- Each item is an isolated commit: D1 one-line predicate + test; D2 replay projection (revert to shared
  builder); D3 parser (remove + restore `graph: unknown`); D5 generated table (restore hand table and its
  test); D6 graph accessor + ws rewire state (revert); D7 env allowlist (revert helper; restore inherited env
  only if the Lead explicitly accepts the risk — recorded in release notes); doc/claims changes are additive.
- No DB migrations and no wire-vocabulary changes, so rollback is `git revert` per commit; the old tests keep
  passing except the new invariants (which fail — that is the intended RED).
- Runtime inverses: replay banner state is ephemeral UI; env allowlist has no persistent state.

## 8. Out of scope / blockers

- **Out of scope (this planner):** implementing the changes; running builds/tests (read-only seat);
  e2e/browser execution; paid provider calls or key access; changing the frozen 14/21/65/16 vocabulary;
  provider cost math (owned by S5/backend); release packaging execution (Lead); FigCraft/pixel fidelity;
  UI-ADD/Diff decision surfaces (unfrozen).
- **Approval blockers:** (B1) D7 `env_clear` allowlist needs Lead security sign-off (could break tool
  commands that needs env; fallback = documented accepted risk + redaction). (B2) D3 `HistorySnapshotGraph`
  required-field set needs contract-owner agreement (which node fields are mandatory vs optional);
  (B3) D6 requires a read-only `GraphAuthority` state accessor (core owner) — otherwise the plan records
  NODE_REWIRE truth as BLOCKED rather than inventing state.
- **Evidence blockers:** no HEAD-bound gate log exists at HEAD `5c69b66` (S7-F01); tree dirty (S6-F-04); a
  browser/e2e seat must execute the §6 e2e commands before replay/failed-terminal claims leave UNVERIFIED.

## 9. Contract parity summary (verification contract for the wave)

| Vocabulary | Backend (current) | UI (current) | Target |
|---|---:|---:|---|
| Agent states | 16 | 16 (labels+enum) | 16 + **terminality parity** (`failed` terminal) |
| Frozen commands | 14 (+3 decision-gated) | 14 (+3) | unchanged |
| Events | 21 | 21 | unchanged |
| Refusal codes | 65 | **45** | 65 via generated artifact + drift test |
| Settings fields | provider/model/key_present + optional thinking/effort | same (bool key only) | documented additive fields; key never present |
| NODE_REWIRE state | hardcoded `running` | renders what arrives | real task state / named refusal |

## 10. UNVERIFIED (must not be read as pass)

- Runtime replay read-only behaviour under real pointer events (source-analysis only — S3 §6.2).
- 16-state parity and failed-terminality under a **live** RUN_STATUS stream (no live run; S3 §6.3).
- Settings against a live key (no key access; S2/S6 policy).
- Contract/backend tests in §6 were **not run** by this planner (read-only) — their last known green run is
  per-seat evidence at a different HEAD; gate logs are not HEAD-bound (S7-F01).
- e2e replay and failed-run scenarios require a rebuilt artifact + e2e seat (S7/S8).
- Paper A-F01/F02/F03 closure at HEAD is source-level (S1/S3/S8) but not release-artifact-proven; the shipped
  `184D51A7…` binary predates the fixes (S8 §2).
- Two official docs pages remain truncated at the tail (thinking-mode sample, streaming response schema);
  contract-relevant sections were obtained before truncation and no claim depends on the tails.
