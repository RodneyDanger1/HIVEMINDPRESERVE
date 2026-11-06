# FILES.md — ui-watchdog seat (49/49 assigned files)

**Seat:** ui-watchdog · **Partition:** `docs/trace/**` (27 files), `docs/diagrams/**` (15), root configs (5), plus dependency provenance research. **Coverage:** 49/49 files in `audit/inventory/ASSIGNMENTS.json`; all bytes/sha256 match `FILE-MANIFEST.csv`. **Deep files:** 0 (per WAVE-SHEET depth table).

File classes: 5 Mermaid sources (`.mmd`) + 5 SVG renders + 5 PNG renders; 17 trace docs/data (`docs/trace/**`) incl. 15 per-test JSON captures + 4 PNG screenshots; 5 root/config files.

Records are structured per WAVE-SHEET §3: metadata, purpose, mechanics, quality, evidence/UNVERIFIED. Pure docs/assets get purpose/provenance/role/defect records instead of fictional code mechanics.

## .cargo/config.toml

**Metadata:** 407 bytes · sha256 `7E2C7429B2BFB246661EBA6BA6B07D2DB7B4D0A89085A5AA11699A59D3CC00A8` · language TOML · category other · seat ui-watchdog · depth standard

**Purpose:** Cargo build/env config for this host.

**Mechanics:** [build] target-dir=target; [env] CARGO_PROFILE_DEV_DEBUG=0, CARGO_INCREMENTAL=0, RUST_BACKTRACE=1, CC/AR x86_64-pc-windows-gnu = C:/Users/Riley/Desktop/hiverouter-v1/tools/w64devkit/bin/{gcc,ar}.exe; target linker=gcc.exe.

**Observation:** Machine-specific absolute paths → portability defect (F-09).

**Quality:** 4/5 configuration/doc (clear; see FINDINGS for specifics).

**Evidence:** file read (`read`); metadata cross-checked against `audit/inventory/FILE-MANIFEST.csv` (bytes + sha256 match).

## .gitignore

**Metadata:** 158 bytes · sha256 `FCA124B10A7E08A0A6F07BDA4ACB829A2A985D09D5C75FED12D20C087A1BC60A` · language unknown · category other · seat ui-watchdog · depth standard

**Purpose:** Repo ignore rules.

**Mechanics:** Full content: ignores `target/`, `node_modules/`, `dist/`, `tools/w64devkit/`, `*.pdb`, `*.tmp`, `docs/ui/figcraft/*/*.figcraft`, `docs/ui/figcraft/ASSETS/*.png`, `docs/ui/figcraft/ASSETS/*.svg`, `.env`, `*.log`. Note: FigCraft design files and export assets are deliberately untracked -> assets are not in the git tree/manifest (see F-13).

**Quality:** 4/5 configuration/doc (clear; see FINDINGS for specifics).

**Evidence:** file read (`read`); metadata cross-checked against `audit/inventory/FILE-MANIFEST.csv` (bytes + sha256 match).

## CHANGE-LEDGER-IMPL.md

**Metadata:** 2393 bytes · sha256 `3995499955B942F397F072A848130BB4A1E50B3551DD7F052A4800D474637307` · language Markdown · category other · seat ui-watchdog · depth standard

**Purpose:** Implementation deviations + contract-change ledger.

**Mechanics:** 6 rows CL-I-001..006: CL-I-001 adds APPROVAL_RESOLVE/SETTINGS_STATUS/FILE_TREE_LIST/FILE_READ + 4 events + 5 refusal codes (applied, evidence contract crate tests); CL-I-002 GNU toolchain/w64devkit + reqwest rustls feature correction; CL-I-003 FigCraft A/B assets; CL-I-004 web milestone 1; CL-I-005 mermaid diagrams; CL-I-006 claim register. Status values applied/applied-pending-evidence/blocked.

**Observation:** CL-I-003 says 'B: 24 + 2 pending' superseded by final B 27/0 pending (F-10 low).

**Quality:** 4/5 configuration/doc (clear; see FINDINGS for specifics).

**Evidence:** file read (`read`); metadata cross-checked against `audit/inventory/FILE-MANIFEST.csv` (bytes + sha256 match).

## README.md

**Metadata:** 1742 bytes · sha256 `B60FAC5A2A7DDFB22B4ED50A0C2AD265DA3D7687E659ABDCA3FABB6542010852` · language Markdown · category other · seat ui-watchdog · depth standard

**Purpose:** Workspace README / shipped-status summary.

**Mechanics:** Layout backend/web/tools/docs/ledgers; toolchain stable-x86_64-pc-windows-gnu + w64devkit, Node >=22.12.0, no Python/Docker/cloud; build commands; status: final binary sha 184D51A7…C9A9, dist zip sha 9B2ED5A7…B144, 233 backend tests passed/0 failed/1 ignored, 46 web unit tests, live e2e 79 passed/0 failed/14 fixme skips.

**Observation:** Claims need cross-check; trace shows 260 UNVERIFIED hops (F-02).

**Quality:** 4/5 configuration/doc (clear; see FINDINGS for specifics).

**Evidence:** file read (`read`); metadata cross-checked against `audit/inventory/FILE-MANIFEST.csv` (bytes + sha256 match).

## docs/diagrams/agent-lifecycle-state.mmd

**Metadata:** 1660 bytes · sha256 `E8DCD06FB84CEB22A389656EB7288A469F53E8B4E7B74EFF1F5803A642E35317` · language Mermaid · category documentation · seat ui-watchdog · depth standard

**Purpose:** Agent lifecycle state machine (one ≥13-state enum).

**Mechanics:** stateDiagram-v2; 6 not-terminal states, 4 terminal states, healthy→reconnecting→healthy loop, replay view state; notes mark empty/loading as surface states and awaiting_approval nonce replay → E_APPROVAL_REPLAY.

**Quality:** Documentation quality: 4/5 (clear, sourced, machine-friendly Mermaid; one stale baseline note). Evidence: full Mermaid source read; PNG/SVG rendered pairs present.

**Evidence:** file read (`read`); metadata cross-checked against `audit/inventory/FILE-MANIFEST.csv` (bytes + sha256 match).

## docs/diagrams/agent-lifecycle-state.png

**Metadata:** 136871 bytes · sha256 `97F3984D1F22D64A8F6D0CDD688EBD5C038A31286AC6D90F51A63E50AB1571BF` · language PNG · category documentation-asset · seat ui-watchdog · depth standard

**Purpose:** Raster export of agent-lifecycle-state.mmd.

**Mechanics:** Raster PNG 1839×1122px, 136871 bytes; binary asset (pixels not prose-analysed). Aspect ratio matches its SVG/Mermaid source where applicable.

**Quality:** 4/5 rendered evidence asset; binary so content UNVERIFIED beyond dimensions/hash. Evidence: PNG IHDR dimensions + sha256 from manifest.

**UNVERIFIED:** per-pixel fidelity and text legibility at final render size (would need image review).

**Evidence:** file read (`read`); metadata cross-checked against `audit/inventory/FILE-MANIFEST.csv` (bytes + sha256 match).

## docs/diagrams/agent-lifecycle-state.svg

**Metadata:** 72503 bytes · sha256 `559347BD2DD440DDC6E8EFD14B8F4CB5568BA3D11C3F90073014B58DFAC34352` · language SVG · category documentation-asset · seat ui-watchdog · depth standard

**Purpose:** Rendered SVG for `agent-lifecycle-state.mmd` (Mermaid CLI output).

**Mechanics:** Self-contained SVG with embedded CSS + `<style>` + graph geometry; viewBox 0 0 1838.04 1122 (statediagram); content matched to its `.mmd` source by node/actor labels.

**Quality:** 4/5 generated artifact (deterministic mmdc output); no hand-edit drift found. Evidence: header + viewBox + labels read; paired PNG aspect ratio matches.

**Evidence:** file read (`read`); metadata cross-checked against `audit/inventory/FILE-MANIFEST.csv` (bytes + sha256 match).

## docs/diagrams/architecture.mmd

**Metadata:** 2080 bytes · sha256 `4A1F683DC05275F5AE95F13DADBFE42CFB45C3D482F2B56FD1C704EAAC9571FD` · language Mermaid · category documentation · seat ui-watchdog · depth standard

**Purpose:** System architecture flowchart.

**Mechanics:** flowchart TD; Browser subgraph (React 19, Vite 8, xyflow 12.12.1, token module, WS client), Server subgraph (single Rust binary: http/ws/bus/graph/orchestrator/scheduler/provider/tools/db/snapshots/observability), external DeepSeek + wiremock. Edges show COMMAND/EVENT WS frames, ACK/refuse bus, provider HTTPS.

**Quality:** Documentation quality: 4/5 (clear, sourced, machine-friendly Mermaid; one stale baseline note). Evidence: full Mermaid source read; PNG/SVG rendered pairs present.

**Evidence:** file read (`read`); metadata cross-checked against `audit/inventory/FILE-MANIFEST.csv` (bytes + sha256 match).

## docs/diagrams/architecture.png

**Metadata:** 120208 bytes · sha256 `05475658BC89C592BA59AE60EC8D58B4F1BB1C004CFD59084922CA4B7DFE4B52` · language PNG · category documentation-asset · seat ui-watchdog · depth standard

**Purpose:** Raster export of architecture.mmd.

**Mechanics:** Raster PNG 1733×1676px, 120208 bytes; binary asset (pixels not prose-analysed). Aspect ratio matches its SVG/Mermaid source where applicable.

**Quality:** 4/5 rendered evidence asset; binary so content UNVERIFIED beyond dimensions/hash. Evidence: PNG IHDR dimensions + sha256 from manifest.

**UNVERIFIED:** per-pixel fidelity and text legibility at final render size (would need image review).

**Evidence:** file read (`read`); metadata cross-checked against `audit/inventory/FILE-MANIFEST.csv` (bytes + sha256 match).

## docs/diagrams/architecture.svg

**Metadata:** 41481 bytes · sha256 `89E8A0C76C25577EECF6CB5CDF0453F3A3527798CF66395FA6126606E9FFBE55` · language SVG · category documentation-asset · seat ui-watchdog · depth standard

**Purpose:** Rendered SVG for `architecture.mmd` (Mermaid CLI output).

**Mechanics:** Self-contained SVG with embedded CSS + `<style>` + graph geometry; viewBox 0 0 1732.46 1676 (flowchart-v2); content matched to its `.mmd` source by node/actor labels.

**Quality:** 4/5 generated artifact (deterministic mmdc output); no hand-edit drift found. Evidence: header + viewBox + labels read; paired PNG aspect ratio matches.

**Evidence:** file read (`read`); metadata cross-checked against `audit/inventory/FILE-MANIFEST.csv` (bytes + sha256 match).

## docs/diagrams/er-schema.mmd

**Metadata:** 2725 bytes · sha256 `B6E82A96FD19AEF19E62A45215F1531FF734936F76387221F4B58F2A4D86F80C` · language Mermaid · category documentation · seat ui-watchdog · depth standard

**Purpose:** Target ER schema for SQLite + versioned JSON snapshots.

**Mechanics:** erDiagram with 10 entities (RUNS, RUN_TASKS, MESSAGES, APPROVALS, TOOL_CALLS, USAGE_RECORDS, HISTORY_REVISIONS, AUDIT_LEDGER, SCHEMA_MIGRATIONS, APP_META); NULL-not-zero semantics annotated for cost/tokens; approvals bind args_sha256 + UNIQUE nonce.

**Quality:** Documentation quality: 4/5 (clear, sourced, machine-friendly Mermaid; one stale baseline note). Evidence: full Mermaid source read; PNG/SVG rendered pairs present.

**Evidence:** file read (`read`); metadata cross-checked against `audit/inventory/FILE-MANIFEST.csv` (bytes + sha256 match).

## docs/diagrams/er-schema.png

**Metadata:** 119367 bytes · sha256 `075A77FC69CABF60CB066CE66A5F3CCDF31358654E863034CB77DE7FDE4BBF58` · language PNG · category documentation-asset · seat ui-watchdog · depth standard

**Purpose:** Raster export of er-schema.mmd.

**Mechanics:** Raster PNG 2032×735px, 119367 bytes; binary asset (pixels not prose-analysed). Aspect ratio matches its SVG/Mermaid source where applicable.

**Quality:** 4/5 rendered evidence asset; binary so content UNVERIFIED beyond dimensions/hash. Evidence: PNG IHDR dimensions + sha256 from manifest.

**UNVERIFIED:** per-pixel fidelity and text legibility at final render size (would need image review).

**Evidence:** file read (`read`); metadata cross-checked against `audit/inventory/FILE-MANIFEST.csv` (bytes + sha256 match).

## docs/diagrams/er-schema.svg

**Metadata:** 218998 bytes · sha256 `47A4C5844B6B426F2D1EE6AD43DF94B7165BA37FB7E4B946D40FC84093D49AA7` · language SVG · category documentation-asset · seat ui-watchdog · depth standard

**Purpose:** Rendered SVG for `er-schema.mmd` (Mermaid CLI output).

**Mechanics:** Self-contained SVG with embedded CSS + `<style>` + graph geometry; viewBox 0 0 3793.95 1372.25 (er); content matched to its `.mmd` source by node/actor labels.

**Quality:** 4/5 generated artifact (deterministic mmdc output); no hand-edit drift found. Evidence: header + viewBox + labels read; paired PNG aspect ratio matches.

**Evidence:** file read (`read`); metadata cross-checked against `audit/inventory/FILE-MANIFEST.csv` (bytes + sha256 match).

## docs/diagrams/run-sequence.mmd

**Metadata:** 2201 bytes · sha256 `18FF2A7FE8AFBA17D40CBED182DE7D4EDC3C39185977C8638B03519D25CE1566` · language Mermaid · category documentation · seat ui-watchdog · depth standard

**Purpose:** Prompt→terminal-run sequence (offline/mock provider path).

**Mechanics:** sequenceDiagram, 9 participants; SUBMIT_PROMPT→validate→DB run row→ACK/RUN_CREATED→plan DAG→Decimal budget preflight→scheduler caps→provider→NODE_STATE/RUN_METRICS→REPLY_DELTA→terminal; refusal branches E_SCHEMA_INVALID/E_PROMPT_EMPTY/E_BUDGET_EXCEEDED; failure-path note 429/401/402/timeout.

**Quality:** Documentation quality: 4/5 (clear, sourced, machine-friendly Mermaid; one stale baseline note). Evidence: full Mermaid source read; PNG/SVG rendered pairs present.

**Evidence:** file read (`read`); metadata cross-checked against `audit/inventory/FILE-MANIFEST.csv` (bytes + sha256 match).

## docs/diagrams/run-sequence.png

**Metadata:** 149891 bytes · sha256 `E9703E3E6B0B3A1C89B555359864F2D1C61FC3CE99A2E6F69791C46F8F92F638` · language PNG · category documentation-asset · seat ui-watchdog · depth standard

**Purpose:** Raster export of run-sequence.mmd.

**Mechanics:** Raster PNG 2032×1092px, 149891 bytes; binary asset (pixels not prose-analysed). Aspect ratio matches its SVG/Mermaid source where applicable.

**Quality:** 4/5 rendered evidence asset; binary so content UNVERIFIED beyond dimensions/hash. Evidence: PNG IHDR dimensions + sha256 from manifest.

**UNVERIFIED:** per-pixel fidelity and text legibility at final render size (would need image review).

**Evidence:** file read (`read`); metadata cross-checked against `audit/inventory/FILE-MANIFEST.csv` (bytes + sha256 match).

## docs/diagrams/run-sequence.svg

**Metadata:** 47817 bytes · sha256 `F4AB892C62692347FB4D7C1442696AE5DEDD37FD8D5B75A67BC7593FB8056E3C` · language SVG · category documentation-asset · seat ui-watchdog · depth standard

**Purpose:** Rendered SVG for `run-sequence.mmd` (Mermaid CLI output).

**Mechanics:** Self-contained SVG with embedded CSS + `<style>` + graph geometry; viewBox -50 -10 2920.5 1569 (sequence); content matched to its `.mmd` source by node/actor labels.

**Quality:** 4/5 generated artifact (deterministic mmdc output); no hand-edit drift found. Evidence: header + viewBox + labels read; paired PNG aspect ratio matches.

**Evidence:** file read (`read`); metadata cross-checked against `audit/inventory/FILE-MANIFEST.csv` (bytes + sha256 match).

## docs/diagrams/ui-control-event-sequence.mmd

**Metadata:** 1880 bytes · sha256 `1CB4689E29310FCE2DD0588AAF19A9A1F9B1EFE05893F54129C9600571ED2595` · language Mermaid · category documentation · seat ui-watchdog · depth standard

**Purpose:** Mutating-control ACK-or-refuse sequence (S04/S05).

**Mechanics:** sequenceDiagram; SPAWN_SUB_HIVE validate Origin/token→depth<3 fanout<8→transaction insert task+ledger→ACK+NODE_SPAWN→render-back only after ACK; over-cap → ERROR E_FANOUT_LIMIT→toast→un-mutated. Then REWIRE_TASK stale expected_rev→E_RESYNC_REQUIRED resync.

**Quality:** Documentation quality: 4/5 (clear, sourced, machine-friendly Mermaid; one stale baseline note). Evidence: full Mermaid source read; PNG/SVG rendered pairs present.

**Evidence:** file read (`read`); metadata cross-checked against `audit/inventory/FILE-MANIFEST.csv` (bytes + sha256 match).

## docs/diagrams/ui-control-event-sequence.png

**Metadata:** 98011 bytes · sha256 `0C71C776FFC17A9C4B4CF4E74093CB55D0757176E078CB60F59ED9556CBA8CC3` · language PNG · category documentation-asset · seat ui-watchdog · depth standard

**Purpose:** Raster export of ui-control-event-sequence.mmd.

**Mechanics:** Raster PNG 2032×813px, 98011 bytes; binary asset (pixels not prose-analysed). Aspect ratio matches its SVG/Mermaid source where applicable.

**Quality:** 4/5 rendered evidence asset; binary so content UNVERIFIED beyond dimensions/hash. Evidence: PNG IHDR dimensions + sha256 from manifest.

**UNVERIFIED:** per-pixel fidelity and text legibility at final render size (would need image review).

**Evidence:** file read (`read`); metadata cross-checked against `audit/inventory/FILE-MANIFEST.csv` (bytes + sha256 match).

## docs/diagrams/ui-control-event-sequence.svg

**Metadata:** 43918 bytes · sha256 `6CC5DA36A603A2DE2503F28C98E55C923CE48A41F1977B3801C8F392D3B5D9D1` · language SVG · category documentation-asset · seat ui-watchdog · depth standard

**Purpose:** Rendered SVG for `ui-control-event-sequence.mmd` (Mermaid CLI output).

**Mechanics:** Self-contained SVG with embedded CSS + `<style>` + graph geometry; viewBox -50 -10 3186 1274 (sequence); content matched to its `.mmd` source by node/actor labels.

**Quality:** 4/5 generated artifact (deterministic mmdc output); no hand-edit drift found. Evidence: header + viewBox + labels read; paired PNG aspect ratio matches.

**Evidence:** file read (`read`); metadata cross-checked against `audit/inventory/FILE-MANIFEST.csv` (bytes + sha256 match).

## docs/trace/A11Y-CONTRAST-PLAN.md

**Metadata:** 5837 bytes · sha256 `8DB0269DA711AE2C9924753EB39BC2B9D8A2FB5A207ED6F5FD472072F0037094` · language Markdown · category documentation · seat ui-watchdog · depth standard

**Purpose:** G-UI-1..12 accessibility/contrast harness plan.

**Mechanics:** Status: PLAN + static math only. 10 contrast pairs recomputed (node, WCAG 2.1): six PDF rows reproduce exactly (14.6677/8.7427/12.6391/3.3167/5.4079/4.6364), four recorded failures (card-meta 4.21/3.84, honey ring 1.70/1.55); maps each of 12 gates to harness implementation; states DOM gates blocked until wired UI/Playwright exist.

**Observation:** Stale: file still says 'Wired UI not delivered yet (web/src/App.tsx is milestone-1 chrome)' but QA-HARNESS reports harness LIVE final run (F-03).

**Quality:** 4/5 documentation (evidence-rich, honest UNVERIFIED sections; status drift noted in FINDINGS).

**Evidence:** file read (`read`); metadata cross-checked against `audit/inventory/FILE-MANIFEST.csv` (bytes + sha256 match).

## docs/trace/QA-HARNESS.md

**Metadata:** 19296 bytes · sha256 `09734D2A94E73373606269B52FD5B8CFA40D70CB3F69A7DB64CE8608DA5BB75C` · language Markdown · category documentation · seat ui-watchdog · depth standard

**Purpose:** QA/trace harness status + layout + measurements.

**Mechanics:** 232 lines, 12 sections. Status: 93-test Chrome suite = 79 passed/0 failed/14 skipped against binary 184D51A7…C9A9, asset index-Cbfm1IuM.js, PID 33580; static validator 27/27; documents backend routes (GET /health, /api/session, /api/…), harness layout, frozen-count parsing, evidence + honest limits; earlier defects listed as fixed by owners.

**Observation:** Contradicts terminal-truth.md/A11Y plan legacy 'app does not exist' status (F-03/F-04).

**Quality:** 4/5 documentation (evidence-rich, honest UNVERIFIED sections; status drift noted in FINDINGS).

**Evidence:** file read (`read`); metadata cross-checked against `audit/inventory/FILE-MANIFEST.csv` (bytes + sha256 match).

## docs/trace/TRACE-SCHEMA.md

**Metadata:** 6984 bytes · sha256 `836CA29B8976A406AF18724E2C7B4616C02986531BC4A6D003D3706BE835BC86` · language Markdown · category documentation · seat ui-watchdog · depth standard

**Purpose:** 11-hop control-trace data contract.

**Mechanics:** Defines controls.json (29 records = 28 surfaces + S-APPROVAL × 11 hops), traces.json (append-only runs), events.json (emitted/subscribed/delivered parity, 21-name vocabulary), terminal-truth.md; exact hop keys 1_render..11_reentry with required proof type and verdict-leaving rule (evidence required).

**Observation:** Status line: live capture applied; 43 WIRED-AND-WORKS, 260 UNVERIFIED, 5 MISSING, 0 WIRED-BUT-DEAD.

**Quality:** 4/5 documentation (evidence-rich, honest UNVERIFIED sections; status drift noted in FINDINGS).

**Evidence:** file read (`read`); metadata cross-checked against `audit/inventory/FILE-MANIFEST.csv` (bytes + sha256 match).

## docs/trace/controls.json

**Metadata:** 73765 bytes · sha256 `D364512D98EEEF50D50A0855307DB65E6FB7876FE59BD131EC89EBFE9DCD2856` · language JSON · category documentation · seat ui-watchdog · depth standard

**Purpose:** 29 controls (28 surfaces + S-APPROVAL) × 11 hops.

**Mechanics:** schema hiverouter.trace.controls/v1; _meta status PENDING_APP_NOT_BUILT, source control-matrix.json, live_capture link to trace-capture; 23 PARTIALLY_VERIFIED / 6 PENDING_APP_NOT_BUILT; kinds {derived 9, mutating-command 8, client-state 5, consume-only 3, static-asset 2, consume+client-state 1, decision-gated 1}; hop verdicts {PENDING 270, WIRED-AND-WORKS 43, MISSING 6}; commands cover 12 frozen names.

**Observation:** Meta.status stale vs per-control statuses (F-07).

**Quality:** 3/5 machine data (schema-compliant, honest status; completeness limited by missing live runs).

**Evidence:** file read (`read`); metadata cross-checked against `audit/inventory/FILE-MANIFEST.csv` (bytes + sha256 match).

## docs/trace/events.json

**Metadata:** 1272 bytes · sha256 `1037643C7D56A6CF8975AEA6ECE545C55B42D42234C6E9B7093AA3AA74326350` · language JSON · category documentation · seat ui-watchdog · depth standard

**Purpose:** Event parity ledger (emitted/subscribed/delivered).

**Mechanics:** schema hiverouter.trace.events/v1; status NOT_COLLECTED; arrays empty (emitted/subscribed/delivered/payload_parity/terminal_records all []); 21-name frozen event vocabulary + 5 parity rules incl. failed-run cost/tokens NULL never 0.

**Observation:** Honest not-collected; event parity remains UNVERIFIED (F-05).

**Quality:** 3/5 machine data (schema-compliant, honest status; completeness limited by missing live runs).

**Evidence:** file read (`read`); metadata cross-checked against `audit/inventory/FILE-MANIFEST.csv` (bytes + sha256 match).

## docs/trace/live/README.md

**Metadata:** 6081 bytes · sha256 `DE5A193B6D24ED3E3FDA7B60BE3B706ABE44F59E09E3E8B6B4A5F282303C247C` · language Markdown · category documentation · seat ui-watchdog · depth standard

**Purpose:** Live browser-QA evidence index.

**Mechanics:** Base http://127.0.0.1:8799; Playwright 1.65.0; final run 93 tests = 79 passed/0 failed/14 skipped (3.6m); table of per-area evidence (boot, skip-link, console, census, tabs/nav, zoom, CHAT_PIN ACK, empty prompt E_PROMPT_EMPTY with 0 frames, WS token+hello, token contrast 14.6677, SETTINGS_STATUS ACK+result, contract drift, FILE_TREE_LIST/READ, 28-surface trace capture); trace totals 43/260/5/0 and controls 23 partial/6 pending.

**Observation:** Provenance: e2e harness partition; source of trace verdicts.

**Quality:** 4/5 documentation (evidence-rich, honest UNVERIFIED sections; status drift noted in FINDINGS).

**Evidence:** file read (`read`); metadata cross-checked against `audit/inventory/FILE-MANIFEST.csv` (bytes + sha256 match).

## docs/trace/live/app-boot-full.png

**Metadata:** 112923 bytes · sha256 `6368078AE15D3B99310BF68771E44E2529EE2D4887CDEF038EFCEEAF11436D24` · language PNG · category documentation-asset · seat ui-watchdog · depth standard

**Purpose:** Live-app boot screenshot (full page).

**Mechanics:** Raster PNG 1440×900px, 112923 bytes; binary asset (pixels not prose-analysed). Aspect ratio matches its SVG/Mermaid source where applicable.

**Quality:** 4/5 rendered evidence asset; binary so content UNVERIFIED beyond dimensions/hash. Evidence: PNG IHDR dimensions + sha256 from manifest.

**UNVERIFIED:** per-pixel fidelity and text legibility at final render size (would need image review).

**Evidence:** file read (`read`); metadata cross-checked against `audit/inventory/FILE-MANIFEST.csv` (bytes + sha256 match).

## docs/trace/live/app-boot.png

**Metadata:** 112876 bytes · sha256 `E0C1B0AD6C9815BF00FD958AF9975E401C1A881FF10996E1AD4F4FEA064FE855` · language PNG · category documentation-asset · seat ui-watchdog · depth standard

**Purpose:** Live-app boot screenshot.

**Mechanics:** Raster PNG 1440×900px, 112876 bytes; binary asset (pixels not prose-analysed). Aspect ratio matches its SVG/Mermaid source where applicable.

**Quality:** 4/5 rendered evidence asset; binary so content UNVERIFIED beyond dimensions/hash. Evidence: PNG IHDR dimensions + sha256 from manifest.

**UNVERIFIED:** per-pixel fidelity and text legibility at final render size (would need image review).

**Evidence:** file read (`read`); metadata cross-checked against `audit/inventory/FILE-MANIFEST.csv` (bytes + sha256 match).

## docs/trace/live/inspect.json

**Metadata:** 10462 bytes · sha256 `CA23AD61B0E8E56957F80727C19D5ED8715483F816BB3A01FF845B90A5739C8A` · language JSON · category documentation · seat ui-watchdog · depth standard

**Purpose:** Live browser inspection snapshot (2026-11-06T13:37:42Z).

**Mechanics:** status 200; session token present schema hiverouter.v1 last_seq 0 ws /ws; consoleErrors ['Failed to load resource: …404']; websockets: one /ws with token-in-url, sent hello{last_seq:0}, received SERVER_HELLO schema hiverouter.v1; info title/root/bodyText.

**Observation:** Single 404 console error recorded — minor (F-08).

**Quality:** 3/5 machine data (schema-compliant, honest status; completeness limited by missing live runs).

**Evidence:** file read (`read`); metadata cross-checked against `audit/inventory/FILE-MANIFEST.csv` (bytes + sha256 match).

## docs/trace/live/live-app/01-boot.json

**Metadata:** 231 bytes · sha256 `B3DED1CD14658DAFE2DB5D98B118B9E780F80054141AE1403EE936D8928292FE` · language JSON · category documentation · seat ui-watchdog · depth standard

**Purpose:** (unclassified) — see EVIDENCE.

**Quality:** UNVERIFIED

**Evidence:** file read (`read`); metadata cross-checked against `audit/inventory/FILE-MANIFEST.csv` (bytes + sha256 match).

## docs/trace/live/live-app/02-skip-link.json

**Metadata:** 131 bytes · sha256 `1246C1EA54B02384737A77E35A4F9665F391563EAAFD07E98EE05AA381846F66` · language JSON · category documentation · seat ui-watchdog · depth standard

**Purpose:** (unclassified) — see EVIDENCE.

**Quality:** UNVERIFIED

**Evidence:** file read (`read`); metadata cross-checked against `audit/inventory/FILE-MANIFEST.csv` (bytes + sha256 match).

## docs/trace/live/live-app/03-console.json

**Metadata:** 46 bytes · sha256 `4E768728E3A6785221F219D056CF4EA9120CACCE6A06CB227A52F422EC9C27FD` · language JSON · category documentation · seat ui-watchdog · depth standard

**Purpose:** (unclassified) — see EVIDENCE.

**Quality:** UNVERIFIED

**Evidence:** file read (`read`); metadata cross-checked against `audit/inventory/FILE-MANIFEST.csv` (bytes + sha256 match).

## docs/trace/live/live-app/04-census.json

**Metadata:** 639 bytes · sha256 `B2E5124A529DC8C4200A5B6D56253849E24E2789EC9B3E51BA4D93C8A06A6B7E` · language JSON · category documentation · seat ui-watchdog · depth standard

**Purpose:** (unclassified) — see EVIDENCE.

**Quality:** UNVERIFIED

**Evidence:** file read (`read`); metadata cross-checked against `audit/inventory/FILE-MANIFEST.csv` (bytes + sha256 match).

## docs/trace/live/live-app/05-tabs.json

**Metadata:** 56 bytes · sha256 `81DE340B8378D8A1FBF8386E1E2B7D1BF78D556B360E4629D074BCCA4342094A` · language JSON · category documentation · seat ui-watchdog · depth standard

**Purpose:** (unclassified) — see EVIDENCE.

**Quality:** UNVERIFIED

**Evidence:** file read (`read`); metadata cross-checked against `audit/inventory/FILE-MANIFEST.csv` (bytes + sha256 match).

## docs/trace/live/live-app/06-nav.json

**Metadata:** 122 bytes · sha256 `B92DEBDBB0B57E3BD80FDC2DBE1EB8EF60621763CC37DE98D152F7B20D38D27B` · language JSON · category documentation · seat ui-watchdog · depth standard

**Purpose:** (unclassified) — see EVIDENCE.

**Quality:** UNVERIFIED

**Evidence:** file read (`read`); metadata cross-checked against `audit/inventory/FILE-MANIFEST.csv` (bytes + sha256 match).

## docs/trace/live/live-app/07-zoom.json

**Metadata:** 197 bytes · sha256 `8E27D5EF0DBEBC52080BCC02F714F449837152BBF7039A848E920221AA96CB0C` · language JSON · category documentation · seat ui-watchdog · depth standard

**Purpose:** (unclassified) — see EVIDENCE.

**Quality:** UNVERIFIED

**Evidence:** file read (`read`); metadata cross-checked against `audit/inventory/FILE-MANIFEST.csv` (bytes + sha256 match).

## docs/trace/live/live-app/08-chat-pin.json

**Metadata:** 334 bytes · sha256 `892CC160997F0901D54A508A32E70E8A3CD873AF186A87FAB23CB216AC81C1DF` · language JSON · category documentation · seat ui-watchdog · depth standard

**Purpose:** (unclassified) — see EVIDENCE.

**Quality:** UNVERIFIED

**Evidence:** file read (`read`); metadata cross-checked against `audit/inventory/FILE-MANIFEST.csv` (bytes + sha256 match).

## docs/trace/live/live-app/09-empty-prompt.json

**Metadata:** 50 bytes · sha256 `1CED73CEF3179D73F6459E3483DAC6EE43911F2571BBDCBAC7C4B4C50BBA4F54` · language JSON · category documentation · seat ui-watchdog · depth standard

**Purpose:** (unclassified) — see EVIDENCE.

**Quality:** UNVERIFIED

**Evidence:** file read (`read`); metadata cross-checked against `audit/inventory/FILE-MANIFEST.csv` (bytes + sha256 match).

## docs/trace/live/live-app/10-ws-handshake.json

**Metadata:** 200 bytes · sha256 `9F4B564EA9CE146BAFB4C263D28C9E27161270DA412678ACF5180E1FBDAB6CF9` · language JSON · category documentation · seat ui-watchdog · depth standard

**Purpose:** (unclassified) — see EVIDENCE.

**Quality:** UNVERIFIED

**Evidence:** file read (`read`); metadata cross-checked against `audit/inventory/FILE-MANIFEST.csv` (bytes + sha256 match).

## docs/trace/live/live-app/11-colours.json

**Metadata:** 265 bytes · sha256 `70B0F44C187C3CC5679529510C90ABBE649810CA1E461085846C948FF4C72BF6` · language JSON · category documentation · seat ui-watchdog · depth standard

**Purpose:** (unclassified) — see EVIDENCE.

**Quality:** UNVERIFIED

**Evidence:** file read (`read`); metadata cross-checked against `audit/inventory/FILE-MANIFEST.csv` (bytes + sha256 match).

## docs/trace/live/live-app/12-settings-status.json

**Metadata:** 266 bytes · sha256 `5BA61E4C91D46141EE6EADCC0CFCCDB548697319E9A3C7F7B7A09EF0440885C2` · language JSON · category documentation · seat ui-watchdog · depth standard

**Purpose:** (unclassified) — see EVIDENCE.

**Quality:** UNVERIFIED

**Evidence:** file read (`read`); metadata cross-checked against `audit/inventory/FILE-MANIFEST.csv` (bytes + sha256 match).

## docs/trace/live/live-app/13-contract-drift.json

**Metadata:** 443 bytes · sha256 `4D93E38D8F022589B9AC56207E1BB4BE3AA29736FBAF5808F7D39264DD08A978` · language JSON · category documentation · seat ui-watchdog · depth standard

**Purpose:** (unclassified) — see EVIDENCE.

**Quality:** UNVERIFIED

**Evidence:** file read (`read`); metadata cross-checked against `audit/inventory/FILE-MANIFEST.csv` (bytes + sha256 match).

## docs/trace/live/live-app/14-file-tree.json

**Metadata:** 504 bytes · sha256 `7D8A09750959415C88569D78D0C142BAE9C8D28908AA4EAFF345ADEE1CF14306` · language JSON · category documentation · seat ui-watchdog · depth standard

**Purpose:** (unclassified) — see EVIDENCE.

**Quality:** UNVERIFIED

**Evidence:** file read (`read`); metadata cross-checked against `audit/inventory/FILE-MANIFEST.csv` (bytes + sha256 match).

## docs/trace/live/live-app/15-data-size.json

**Metadata:** 678 bytes · sha256 `4E026FD90D1A53EACEC3AA3B7706A156A5A95A0F8DE9585A8FC6D16C91D932CF` · language JSON · category documentation · seat ui-watchdog · depth standard

**Purpose:** (unclassified) — see EVIDENCE.

**Quality:** UNVERIFIED

**Evidence:** file read (`read`); metadata cross-checked against `audit/inventory/FILE-MANIFEST.csv` (bytes + sha256 match).

## docs/trace/live/live-app/boot.png

**Metadata:** 99367 bytes · sha256 `9EA23060505B5C65697A2DD735035FFE844FD17F1BCB76DC9372D8D6CD8D6DE7` · language PNG · category documentation-asset · seat ui-watchdog · depth standard

**Purpose:** Live-app boot capture (per-test evidence).

**Mechanics:** Raster PNG 1280×720px, 99367 bytes; binary asset (pixels not prose-analysed). Aspect ratio matches its SVG/Mermaid source where applicable.

**Quality:** 4/5 rendered evidence asset; binary so content UNVERIFIED beyond dimensions/hash. Evidence: PNG IHDR dimensions + sha256 from manifest.

**UNVERIFIED:** per-pixel fidelity and text legibility at final render size (would need image review).

**Evidence:** file read (`read`); metadata cross-checked against `audit/inventory/FILE-MANIFEST.csv` (bytes + sha256 match).

## docs/trace/live/live-app/surface-census.png

**Metadata:** 99367 bytes · sha256 `9EA23060505B5C65697A2DD735035FFE844FD17F1BCB76DC9372D8D6CD8D6DE7` · language PNG · category documentation-asset · seat ui-watchdog · depth standard

**Purpose:** Live-app surface census capture (per-test evidence).

**Mechanics:** Raster PNG 1280×720px, 99367 bytes; binary asset (pixels not prose-analysed). Aspect ratio matches its SVG/Mermaid source where applicable.

**Quality:** 4/5 rendered evidence asset; binary so content UNVERIFIED beyond dimensions/hash. Evidence: PNG IHDR dimensions + sha256 from manifest.

**UNVERIFIED:** per-pixel fidelity and text legibility at final render size (would need image review).

**Evidence:** file read (`read`); metadata cross-checked against `audit/inventory/FILE-MANIFEST.csv` (bytes + sha256 match).

## docs/trace/live/trace-capture.json

**Metadata:** 22492 bytes · sha256 `2D22C75E3A6F2D2E1CC96525FFD017281108DAB44CD9241FB591AF81A898236D` · language JSON · category documentation · seat ui-watchdog · depth standard

**Purpose:** 28-surface live trace capture (2026-11-06T15:24:24Z).

**Mechanics:** schema hiverouter.trace.live-capture/v1; base http://127.0.0.1:8799; rule: only evidence-backed hops count; 28 surface entries feeding controls.json verdicts.

**Observation:** Backs 43 WIRED-AND-WORKS / 5-6 MISSING claims.

**Quality:** 3/5 machine data (schema-compliant, honest status; completeness limited by missing live runs).

**Evidence:** file read (`read`); metadata cross-checked against `audit/inventory/FILE-MANIFEST.csv` (bytes + sha256 match).

## docs/trace/terminal-truth.md

**Metadata:** 2548 bytes · sha256 `BBC7C40EDE3E70383E7F51001686E062EC08A57706E3FFFC5B47BDD5FEF69A69` · language Markdown · category documentation · seat ui-watchdog · depth standard

**Purpose:** Terminal-run truth template.

**Mechanics:** Per-run evidence block: run row status/error, terminal event payload, snapshot/cost/token NULL semantics, live-vs-persisted event parity, header/actions, re-entry, refusal accounting, no-secret-leak; all rows PENDING and status NOT_COLLECTED ('app does not exist yet' — stale).

**Observation:** Template only; no runs captured (F-04).

**Quality:** 4/5 documentation (evidence-rich, honest UNVERIFIED sections; status drift noted in FINDINGS).

**Evidence:** file read (`read`); metadata cross-checked against `audit/inventory/FILE-MANIFEST.csv` (bytes + sha256 match).

## docs/trace/traces.json

**Metadata:** 279 bytes · sha256 `D0BA5A066CD07A4D90F2937842EF7CA3EA149FBD46C8CE927555E2890CDF9A43` · language JSON · category documentation · seat ui-watchdog · depth standard

**Purpose:** Append-only captured TraceRun[].

**Mechanics:** schema hiverouter.trace.runs/v1; status NO_RUNS; traces []; populated by e2e/helpers/trace.ts when a Playwright run captures evidence.

**Observation:** No runs means terminal truth uncollected (F-05).

**Quality:** 3/5 machine data (schema-compliant, honest status; completeness limited by missing live runs).

**Evidence:** file read (`read`); metadata cross-checked against `audit/inventory/FILE-MANIFEST.csv` (bytes + sha256 match).

## rust-toolchain.toml

**Metadata:** 108 bytes · sha256 `6F03E2211329AC20362187D0BBD913279569E4F45F3D60C2F43F09851468F7E0` · language TOML · category other · seat ui-watchdog · depth standard

**Purpose:** Rust toolchain pin.

**Mechanics:** channel=stable-x86_64-pc-windows-gnu; profile=minimal; components rustfmt, clippy.

**Quality:** 4/5 configuration/doc (clear; see FINDINGS for specifics).

**Evidence:** file read (`read`); metadata cross-checked against `audit/inventory/FILE-MANIFEST.csv` (bytes + sha256 match).
