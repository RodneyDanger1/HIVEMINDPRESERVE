# GATE-CY-3 — Test-tree coverage assessment + proof verdict

**Protocol:** /forgewright-recon → /forgewright-plan → /forgewright-lead (exact order). Failed cycle = BLOCKED.
**Status:** PASS (verdict: baseline PASS-WITH-NOTES / HEAD UNVERIFIED)

## 1. RECON

**VERIFIED** file inventory: 18 backend test files (~110 `fn` tests incl. helpers), 8 web test files = 46 `it()` (matches claim), 5 e2e specs + helpers (93 executed per log: 79 pass/14 skip). **VERIFIED** error-path coverage present in code: 401 (`E_TOKEN_INVALID` in ws_transport.rs; 401 in refusal_matrix.rs ×7), 402/429/timeout/budget/malformed in `hiverouter-provider/tests/refusal_matrix.rs` (401×7, 402×5, 429×6, timeout×7, budget×24, malformed×3), replay in handshake.rs/ws_transport.rs (E_APPROVAL_REPLAY/E_RESYNC_REQUIRED keywords), nonce/approval in ws_transport.rs, kill/interrupted in kill_restore.rs, history gap in snapshot_history.rs. **VERIFIED** missing: no landed test/log for HEAD `5c69b66` provider-wiring + replay-arm + 16-state parity change; no paid-run terminal evidence; 14 e2e skips explicit.

## 2. PLAN

Plan: build file-by-file test plan with exact commands/acceptance; rank gaps P0-equivalent (no gate for HEAD; provider live path unproven) vs P1 (terminal usage parity, replay render, state parity pin) vs P2 (skip inventory, log provenance). Define the minimal gate command set the Lead can run on a clean checkout of `5c69b66`.

## 3. LEAD

Lead decision: report `NOT PROVEN for current HEAD; PASS-WITH-NOTES for baseline 3988622`. Gate files written; no product edits; deliver REPORT with commands + UNVERIFIED list.
