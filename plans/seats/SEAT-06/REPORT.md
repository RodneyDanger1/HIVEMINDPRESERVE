# SEAT S6 — PROVENANCE / KEY SECURITY (HiveMind 1.0 program)

**Seat:** S6 figcraft-b (provenance & key security) · **Date:** 2026-11-06T11:06:22-0600 · **Mode:** READ-ONLY (no product/git/config edits, no key access, no paid calls; writes only `plans/seats/SEAT-06/`).
**Baseline:** HEAD `5c69b66` — "feat: wire DeepSeek provider into core; live smoke, replay arm, 16-state parity, settings". **Tree not clean** (F-04), so build/test claims are not re-verified here.

## 1. VERIFIED facts (code read, path:line)
1. Key source is env-only: `DEEPSEEK_API_KEY` (`provider/src/secret.rs:17,30-38`); missing/blank → `E_KEY_MISSING` naming the variable, never the value (`secret.rs:31-36,46-56`).
2. Key lives in `zeroize::Zeroizing<String>`; `Debug` prints `ApiKey(***)` (`secret.rs:20-22,77-81`); only accessor `expose()` is `pub(crate)` (`secret.rs:72-74`).
3. Key is used in exactly one place — the bearer header buffer (`client.rs:269-279`) applied to requests (`client.rs:417-430,509-524`); client `Debug` redacts via the key type (`client.rs:196-205`).
4. No logging of headers/key: retry log carries code/attempt/delay only (`client.rs:352-357`); transport error text is constructed without request data (`client.rs:669-687`).
5. API surfaces expose boolean presence only: `SETTINGS_STATUS.key_present` (`tools/src/settings.rs:18-26`), `/health` + `/diagnose` `provider_key_present` (`tools/src/health.rs:59-60,195,219`); diagnose payload is derived counts/paths (`health.rs:211-253`).
6. Audit sink has explicit secret filters: `FORBIDDEN_AUDIT_SUBSTRINGS = ["DEEPSEEK_API_KEY","sk-","Bearer "]` (`tools/src/audit.rs:31`) enforced at `audit.rs:247`.
7. Tests pin the non-leak claims (read, not executed): `secret.rs:87-103`, `client.rs:730-742`, `settings.rs:47-55`, `health.rs:391-423,436-449`, `core/tests/http_boot.rs:230`, `core/tests/ws_transport.rs:349`.
8. Worktree secret scan: 0 `sk-…`-shaped strings across tracked files (E2); `.gitignore` ignores `.env`, `.env.*`, `.live-smoke-data/`, `*.log` (`.gitignore:10-15`).
9. Official DeepSeek docs (4 pages, fetched 2026-11-06) confirm the code's endpoint (`https://api.deepseek.com/chat/completions`, no `/v1`), model ids (`deepseek-flash`, `deepseek-v4-pro`), Bearer auth, streaming SSE + usage-in-final-chunk semantics.

## 2. FINDINGS (security / provenance; severity for Lead triage)

**F-01 P1 — tool subprocesses inherit the environment carrying the key.** `tools/src/shell.rs:201-208` spawns user/agent-chosen programs with `args/current_dir/stdio/kill_on_drop` and **no `env_clear()`/`env_remove()`**; grep over `backend/**/*.rs` finds 0 occurrences of either. Same pattern at `tools/src/cli_support.rs:41` and `tools/src/jail.rs:389`. The key is in the server process env (`secret.rs:17,31`), so any executed child can read/print it; the audit-ledger redaction (`audit.rs:31,247`) only filters recorded text, not the child's ability to exfiltrate. The custody claim in `docs/evidence/key-validation.md:4` ("never written to the repository, logs, UI, database…") and `secret.rs:5-7` ("never logged…") are true of *first-party* code paths but do not cover this process-boundary vector. Fix seam (Lead-owned): `env_clear()` + explicit allowlisted env (`PATH`, `SystemRoot`, temp dirs) before `.args()`, regression test that a child cannot see `DEEPSEEK_API_KEY`; or document the boundary honestly.

**F-02 P1 — mutually exclusive live-provider claims across documents.** `docs/evidence/key-validation.md:7-12` claims a real HTTP 200 call (id `8d5af788…`, `HIVE-KEY-OK`); `docs/evidence/live-provider-smoke.md:7-11` claims a real provider call on `target/debug/hiverouter.exe` :8799 ending `RUN_STATUS completed` / `HIVE-LIVE-OK`; while `docs/audit/PHASE-B-VERIFICATION.md:7,26,106` states `DEEPSEEK_API_KEY` absent, no provider request made, and live provider behaviour not run, and the audit paper (`audit/HIVEMIND-AUDIT-PAPER.md:38,242`) states no paid call was made. Both can be temporally true (PHASE-B snapshot HEAD `ab3a33b`, 07:28-08:20; wiring commit `5c69b66` later), but the evidence docs record **no commit/binary sha256** (live smoke uses a debug binary, not the shipped artifact) so which baseline is "real" cannot be proven from the documents. Fix: add commit hash + artifact sha256 + UTC timestamp to each evidence note; mark historical notes superseded.

**F-03 P2 — audit paper baseline is stale.** Paper §1/§4/A-F01 (`audit/HIVEMIND-AUDIT-PAPER.md:4,11,104,208`) asserts P0 "shipped core never wires the provider"; current HEAD `5c69b66` is exactly the provider-wiring feature commit (E1). The paper is an accurate snapshot of `3988622` but is being cited as the program baseline; no note says A-F01 was resolved by `5c69b66`. Fix: add a baseline-delta note / re-audit A-F01 before the paper is finalised.

**F-04 P2 — tree not quiescent at verification time.** `git status --porcelain` shows 6 modified product manifests (`backend/Cargo.lock`, 4 crate `Cargo.toml`s, `web/package.json`) and untracked `docs/RELEASE.md` + `plans/seats/` (E1). Any test/build/hash claim made now is against a moving tree; S6 marks all such claims UNVERIFIED (quiescence rule).

**F-05 P2 — audit tooling edits another seat's evidence.** `audit/tools/complete_figcraft_a.py:1-4,75` appends "LEAD COMPLETION RECORDS" into `audit/parts/figcraft-a/FILES.md`, and `audit/tools/dedupe_figcraft_a.py:27-35,81` rebuilds that file from templates while dropping appendices, while `audit/HIVEMIND-AUDIT-PAPER.md:132` states "no Lead fallback records were needed after de-duplication". Independence caveat: figcraft-a per-file records were modified by a Lead-owned tool; the paper's independence statement should cite the tooling.

**F-06 P3 — residual key-hardening gaps.** (a) No test asserts subprocess env isolation (`env_clear` absent); (b) `.gitignore` has no key/cert-file patterns (`*.pem`, `*.key`, `secrets.*`) — acceptable while custody is env-only, but a future settings-persistence feature would silently leak; (c) `CLAIM-REGISTER.json:10-16,86` still carries the pre-wire C-026 "unverified" block and cites pre-wire binary `DE676AFB…` (F-06 carried from the figcraft-b audit); (d) git history was not scanned for secrets (UNVERIFIED).

## 3. FILE-BY-FILE PLAN (S6 scope; all read, none edited)
| File | Read | Verdict / action |
|---|---|---|
| `provider/src/secret.rs` | full | PASS — custody claims supported; subprocess boundary covered by F-01 (file-scope OK). |
| `provider/src/client.rs` | full | PASS — header-only key use, redacted Debug, no secret logging; note stale streaming-usage comment vs official docs. |
| `tools/src/settings.rs` | full | PASS — boolean only. |
| `tools/src/health.rs` | full | PASS — health/diagnose secret-free (tests read). |
| `docs/claims/CLAIM-REGISTER.csv/.json` | counts + key rows | CSV 80=72V+8R consistent; JSON stale C-026 block (F-06c). |
| `docs/claims/SKILL-USE.md` | metadata | READ — no key content. |
| `docs/audit/PHASE-B-VERIFICATION.md` | full | Historical snapshot; "no key present" is timeline-bound, not current (F-02). |
| `.gitignore` | full | PASS for `.env`; P3 suggestion key-file patterns (F-06b). |
| `docs/evidence/key-validation.md` | full | Claim READ; no commit/artifact hash → F-02. |
| `docs/evidence/live-provider-smoke.md` | full | Claim READ; debug binary, no hash → F-02. |
| `audit/tools/census_ignored_deliverables.py` | full | PASS — recomputes hashes, writes only audit/. |
| `audit/tools/complete_figcraft_a.py` | full | F-05 — writes another seat's evidence. |
| `audit/tools/dedupe_figcraft_a.py` | full | F-05 — rebuild/drops appendices. |

## 4. TESTS (not executed — read-only seat, dirty tree)
- `cargo test -p hiverouter-provider --lib` → secret.rs 2 + client.rs 3 tests (incl. debug-never-contains-key).
- `cargo test -p hiverouter-tools --lib` → settings 2 + health 5 tests (incl. diagnose secret-free).
- Proposed pin for F-01 (Lead implementation): spawn a child that prints its env from `shell.rs` path and assert `DEEPSEEK_API_KEY` absent.
- Live claims (`key-validation`, `live-provider-smoke`) require a user-approved key + budget; out of seat policy.

## 5. UNVERIFIED
- Whether any tool stdout/stderr capture can carry a child-printed key into the JSONL audit ledger (redaction exists but end-to-end path untested here).
- Git-history secret scan (worktree clean ≠ history clean).
- Live provider calls themselves (no key access, no paid calls) — evidence docs READ only.
- Unit tests above (not run; tree dirty; read-only).
- Claim-register row-level truth for C-216/C-217/C-221 beyond code READ (tests not executed).

## 6. Receipt
Read: paper (full), 4/4 official docs, 3/3 audit files, 11/11 product slice files. Writes: `plans/seats/SEAT-06/` only. Gate cycles 1-3 PASS / PASS / PASS-WITH-FINDINGS. Top item: **F-01 tool-subprocess env inheritance** (P1 candidate) — the key can leave the server process through any executed child; fix seam `shell.rs:201` + `env_clear` allowlist + regression pin.
