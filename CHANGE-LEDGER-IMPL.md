# CHANGE-LEDGER-IMPL — implementation deviations and contract changes

Every deviation from the blueprint or the frozen UI contract gets a row here before it can gate a phase
(EXECUTION-PROMPT §5.6). Status values: `applied`, `applied-pending-evidence`, `blocked`.

| ID | Date | Change | Reason / source | Owner | Status |
|---|---|---|---|---|---|
| CL-I-001 | 2026-11-06 | UI-COMMANDS-v1 adds `APPROVAL_RESOLVE`, `SETTINGS_STATUS`, `FILE_TREE_LIST`, `FILE_READ`; events `APPROVAL_RESOLVED`, `SETTINGS_STATUS_RESULT`, `FILE_TREE_RESULT`, `FILE_CONTENT`; refusal codes `E_APPROVAL_NOT_FOUND`, `E_APPROVAL_DECISION_INVALID`, `E_PATH_OUTSIDE_JAIL`, `E_PATH_NOT_FOUND`, `E_FILE_TOO_LARGE` | Frontend surfaced real surface needs (approval overlay S25 / settings S21 / file tree S22); UI-REQUIREMENTS UI-R-49/50; blueprint Phase 6 approval gate + Phase 10 MP-17 | Lead (doc) + B-Core (Rust contract/matrix) | applied (evidence: contract crate 32 tests incl. 5 matrix; CL-I-001 closed) |
| CL-I-002 | 2026-11-06 | Rust target `stable-x86_64-pc-windows-gnu` (1.99.1) with w64devkit GCC 16.2.0; `libgcc_eh.a` repacked locally; reqwest feature corrected from `rustls-tls` to `rustls` (F2 API diff) | No MSVC/SDK on host; Windows Defender false-positive on w64devkit (skeeto #185); reqwest 0.13.5 no longer exposes `rustls-tls` | Lead + B-Core | applied (toolchain/tests verified) |
| CL-I-003 | 2026-11-06 | FigCraft A/B produced design assets + manifests (A: 28/28 surfaces; B: 24 + 2 pending) and exports under `docs/ui/figcraft/ASSETS/` | UI-WIRING-GUIDE S01-S28 + PDF design authority; UI watchdog gate | FigCraft-A / FigCraft-B | applied-pending-watchdog |
| CL-I-004 | 2026-11-06 | Web milestone 1: tokens (20 swatches, corrected contrast), 15-state enum, frozen contract types, string table, App shell; `typecheck` + `vite build` pass | Blueprint Phase 7/8/10; UI-WIRING-GUIDE | Frontend | applied (quoted in FRONTEND-STATUS.md) |
| CL-I-005 | 2026-11-06 | Five Mermaid diagrams (architecture, agent lifecycle, run sequence, UI event, ER) exported `.mmd`+png+svg | EXECUTION-PROMPT §2 (mermaid-skill), blueprint §6/§9 | Verifier | applied (mmdc exit 0 each) |
| CL-I-006 | 2026-11-06 | Claim register extended with toolchain/contract/UI/build claims; W0 stale environment rows marked superseded | Verifier baseline D-8/C-003/C-006 findings | Lead | applied-pending-register-evidence |
