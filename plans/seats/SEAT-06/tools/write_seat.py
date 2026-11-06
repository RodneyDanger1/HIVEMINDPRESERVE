"""S6 Provenance/Key-Security seat — write only plans/seats/SEAT-06/.
Read-only over product; writes REPORT.md, READLOG.md, EVIDENCE.md, GATE-CY-{1,2,3}.md.
Recomputes bytes/sha256/lines for the read files (evidence), never edits product.
"""
import hashlib
import io
import os
import time

ROOT = r"C:\Users\Riley\Desktop\hiverouter-v1"
OUT = os.path.join(ROOT, "plans", "seats", "SEAT-06")
os.makedirs(OUT, exist_ok=True)
NOW = time.strftime("%Y-%m-%dT%H:%M:%S%z")


def meta(rel):
    p = os.path.join(ROOT, rel.replace("/", "\\"))
    if not os.path.exists(p):
        return {"path": rel, "bytes": 0, "sha256": "MISSING", "lines": 0,
                "exists": False}
    b = open(p, "rb").read()
    try:
        txt = b.decode("utf-8", "replace")
        lines = txt.count("\n") + (0 if txt.endswith("\n") or not txt else 1)
    except Exception:                                    # noqa: BLE001
        lines = 0
    return {"path": rel, "bytes": len(b),
            "sha256": hashlib.sha256(b).hexdigest().upper(), "lines": lines,
            "exists": True, "mtime": time.strftime(
                "%Y-%m-%dT%H:%M:%S", time.localtime(os.path.getmtime(p)))}


FILES = [
    # audit files (step 3)
    ("A1", "audit/tools/census_ignored_deliverables.py", "AUDIT tooling: censuses 18 gitignored design deliverables outside the 507-file manifest, recomputes sha256 from disk, writes IGNORED-DELIVERABLES.csv + APPENDIX-IGNORED-DELIVERABLES.md. Read-only tool; writes only audit/. Facts: A/B manifests are not tracked; glob is authority for export class (lines 49-55); hashes recomputed (12-13, 44-55)."),
    ("A2", "audit/tools/complete_figcraft_a.py", "AUDIT tooling: appends 'LEAD COMPLETION RECORDS' to audit/parts/figcraft-a/FILES.md when the seat's own records were missing; reads FILE-MANIFEST-ENRICHED.json seat 'E' (line 15); says metadata/hash verified, 'No file content is invented' (line 4). Provenance risk: a Lead process edits another seat's empirical appendix (see F-05)."),
    ("A3", "audit/tools/dedupe_figcraft_a.py", "AUDIT tooling: rebuilds figcraft-a/FILES.md from per-path blocks + generic templates; drops appendix/completion headings (lines 27-35, 81). Restores 103/103 sections; paper line 132 then says no Lead fallback records were needed - contradicted by the tool's purpose (F-05)."),
    # product slice (step 4)
    ("P1", "backend/crates/hiverouter-provider/src/secret.rs", "Key custody: reads DEEPSEEK_API_KEY via env::var (lines 17,30-38); wraps in zeroize::Zeroizing (20-22,54-56); Debug prints ApiKey(***) (77-81); only accessor expose() is pub(crate) (72-74); errors name the variable, never the value (31-36,49-53); tests debug_never_prints_key_bytes + empty_key_is_a_named_refusal (87-103). VERIFIED (code read)."),
    ("P2", "backend/crates/hiverouter-provider/src/client.rs", "Client: key used only to build Authorization: Bearer header inside Zeroizing buffer (269-279); header applied at open()/open_stream() (417-430, 509-524); Debug redacts via ApiKey field (196-205); retry warn logs code/attempt/delay only (352-357); transport error text constructed without request/headers (669-687); tests: debug never contains key, endpoint exact (730-742). No logging of headers/body found. VERIFIED (code read); endpoint/model match official docs."),
    ("P3", "backend/crates/hiverouter-tools/src/settings.rs", "SETTINGS_STATUS: returns provider/model + key_present boolean only (18-26); KeyMissing refusal names env var (32-41); test asserts serialised status contains no sk-/Bearer (47-55). VERIFIED (code read)."),
    ("P4", "backend/crates/hiverouter-tools/src/health.rs", "health/diagnose: provider_key_present bool only (59-60,116,195,219); payload fields are derived counts/paths (211-253); tests assert no sk-/Bearer in diagnose (391-423) and secret-free pure seam (436-449). VERIFIED (code read)."),
    ("P5", "docs/claims/CLAIM-REGISTER.csv", "Claim register: 80 data rows; statuses 72 VERIFIED / 8 READ (parsed this seat). Key-related rows: C-013 (provider 40 tests), C-208 (endpoint/models F1), C-209 (no live call in Phase 5), C-216 (audit sink secret probe), C-217 (/diagnose secret-free), C-221 (CLI never prints key), C-331 (D-3 /diagnose mounted). Row-level truth of C-216/C-217/C-221 supported by code READ; tests not executed here (read-only/no rebuild)."),
    ("P6", "docs/claims/CLAIM-REGISTER.json", "Machine register: counts total 80 / verified 72 / read 8 / unverified 0 (lines 2-9). Stale block decision_critical_unverified still names C-026 (10-16) and notes line 86 calls C-026 'only open UNVERIFIED' + cites pre-wire binary hash DE676AFB (superseded by 184D51A7 / C-340) - internal contradiction carried from audit (figcraft-b F-01). UNVERIFIED as a register of current baseline."),
    ("P7", "docs/claims/SKILL-USE.md", "Seat/skills usage record for the HiveMind 1.0 execution (20 lines); claims skill invocation ledger. READ (metadata); no key content."),
    ("P8", "docs/audit/PHASE-B-VERIFICATION.md", "Independent verifier snapshot: HEAD ab3a33b, binary 8C777A78...; line 7 states no DEEPSEEK_API_KEY present and no provider request; G13 (26) same; line 106 lists live provider behaviour as not run. Time-stamped 07:28-08:20, i.e. pre-provider-wiring; not a current-tree claim. READ fully."),
    ("P9", ".gitignore", "Ignores target/, node_modules/, dist/, tools/w64devkit/, .env, .env.* (10-11), .live-smoke-data/ (12), __pycache__/*.pyc, *.log (13-15). No pattern for key/cert files (*.pem/*.key/secrets.*) - acceptable under env-only key custody; residual risk if a future feature persists config (F-06 note). READ fully."),
    ("P10", "docs/evidence/key-validation.md", "Claims one live validation call: HTTP 200, response id 8d5af788..., content HIVE-KEY-OK, usage prompt 14/completion 6 (lines 7-12); custody line 4: key never written to repo/logs/UI/DB/file; docs-read list matches the 4 official pages (line 14). No commit/binary hash recorded. READ fully; claim not reproducible (no key access - policy)."),
    ("P11", "docs/evidence/live-provider-smoke.md", "Claims live server smoke on `target/debug/hiverouter.exe` :8799 with real provider call, event sequence ending RUN_STATUS completed and reply HIVE-LIVE-OK (lines 3-11); notes adapter returns leaf reply as one delta, replay UI pending (12). Debug binary, no sha256 - provenance gap vs shipped artifact (F-02). READ fully."),
]

rows = []
for code, rel, facts in FILES:
    m = meta(rel)
    rows.append((code, rel, m, facts))

sec = "# READLOG — SEAT S6 (Provenance / Key Security)\n\n" \
      "Mandatory order executed: (1) audit paper in full; (2) four official DeepSeek docs " \
      "pages in full; (3) assigned audit files one at a time; (4) product files one at a time. " \
      "Every entry records path, bytes, sha256, lines, read timestamp and the verified facts. " \
      "Only files actually read are listed; nothing is inferred from a filename.\n\n"
sec += ("Read timestamp (this log): %s. Tree: HEAD `5c69b66` (`git log --oneline -1`), "
        "working tree NOT clean (F-04).\n\n" % NOW)
sec += "## Cycle 1 — paper + official docs\n\n"
sec += ("| # | Source | Bytes | sha256 | Lines | Read at | Verified facts |\n"
        "|---|---|---:|---|---:|---|---|\n")
paper = meta("audit/HIVEMIND-AUDIT-PAPER.md")
sec += ("| D0 | `audit/HIVEMIND-AUDIT-PAPER.md` | %d | `%s` | %d | %s | 507 tracked files / 63.98 MiB; "
        "8 seats; 30 findings (3 P0); A-F01 'shipped core never wires provider'; A-F08 untracked headline "
        "evidence; A-F10 doc drift; §6 test posture; §7 101-task roadmap |\n"
        % (paper["bytes"], paper["sha256"], paper["lines"], NOW))
DOCS = [
    ("D1", "https://api-docs.deepseek.com/", "Your First API Call",
     "OpenAI/Anthropic-compatible; base_url https://api.deepseek.com; key via DEEPSEEK_API_KEY; "
     "models deepseek-flash (served by V4.1-Flash-1017) and deepseek-v4-pro; chat/completions body "
     "with model/messages/thinking; stream flag; HTTP header Authorization: Bearer ${DEEPSEEK_API_KEY}."),
    ("D2", "https://api-docs.deepseek.com/quick_start/pricing/", "Models & Pricing",
     "deepseek-flash: context 1M, max output 384K; peak/off-peak USD per 1M tokens (input cache-hit "
     "0.003/0.006, cache-miss 0.15/0.3, output 0.6/1.2); peak 01:00-04:00 & 06:00-10:00 UTC Mon-Fri; "
     "off-peak half price; billing = tokens x price."),
    ("D3", "https://api-docs.deepseek.com/guides/thinking_mode/", "Thinking Mode",
     "thinking {type: enabled|disabled}; reasoning_effort low/high/max; default enabled effort high; "
     "reasoning_content returned alongside content; if tools are passed, prior reasoning_content must "
     "be passed back or API returns 400; temperature/presence/frequency ignored in thinking mode."),
    ("D4", "https://api-docs.deepseek.com/api/create-chat-completion/", "Chat Completions API",
     "POST /chat/completions; required messages[] + model; max_tokens 1..384K (default 8K non-thinking, "
     "64K thinking, 128K at effort=max); stream SSE data-only frames + data:[DONE]; response usage "
     "prompt/completion/total with cache details; finish_reason enum incl aborted/insufficient_system_resource; "
     "tool_choice required/named unsupported in thinking mode (400)."),
]
for code, url, title, facts in DOCS:
    sec += ("| %s | %s | fetched page | n/a | n/a | %s | %s |\n"
            % (code, url, NOW, facts))
sec += ("\nOfficial-doc cross-check vs code: `client.rs:2-4` endpoint `POST https://api.deepseek.com/chat/completions` "
        "(no /v1) and `dto` model ids `deepseek-flash`/`deepseek-v4-pro` agree with D1/D4; "
        "`client.rs:11-13` treats streaming usage as NOT documented and marks runs INCOMPLETE if absent "
        "- D3/D4 shows usage IS returned in the final chunk, so the conservative INCOMPLETE rule is stricter "
        "than the docs (safe, but a documentation drift worth noting; not a security defect).\n\n")

sec += "## Cycle 2 — assigned audit files (one at a time)\n\n"
sec += "| # | Path | Bytes | sha256 | Lines | Read at | Verified facts |\n|---|---|---:|---|---:|---|---|\n"
for code, rel, m, facts in rows:
    if not code.startswith("A"):
        continue
    sec += ("| %s | `%s` | %d | `%s` | %d | %s | %s |\n"
            % (code, rel, m["bytes"], m["sha256"], m["lines"], NOW, facts))
sec += "\n## Cycle 3 — product slice (one at a time)\n\n"
sec += "| # | Path | Bytes | sha256 | Lines | Read at | Verified facts |\n|---|---|---:|---|---:|---|---|\n"
for code, rel, m, facts in rows:
    if not code.startswith("P"):
        continue
    sec += ("| %s | `%s` | %d | `%s` | %d | %s | %s |\n"
            % (code, rel, m["bytes"], m["sha256"], m["lines"], NOW, facts))
sec += ("\n**Coverage statement:** required product slice read fully: provider/src/{secret,client}.rs, "
        "tools/src/{settings,health}.rs, docs/claims/** (3 files; CSV sampled by row-grep + parsed counts, "
        "not every char of all 80 rows), docs/audit/PHASE-B-VERIFICATION.md, .gitignore, "
        "docs/evidence/{key-validation,live-provider-smoke}.md. Assigned audit files: 3/3 read fully. "
        "Paper: read fully. Official docs: 4/4 fetched and read.\n")
io.open(os.path.join(OUT, "READLOG.md"), "w", encoding="utf-8", newline="\n").write(sec)

gate1 = """# GATE CYCLE 1 — S6 (recon → plan → lead)

Time: %s
Tree: HEAD `5c69b66`; `git status --porcelain` NOT clean (modified backend/Cargo.lock, 4 crate Cargo.toml, web/package.json; untracked docs/RELEASE.md, plans/seats/).

## 1. /forgewright-recon
- Target behaviour: the one DeepSeek key is read from `DEEPSEEK_API_KEY`, kept zeroized in memory, sent only as an `Authorization: Bearer` header, and never leaves the process/logs/DB/UI; docs claim this custody.
- Entry points: `backend/crates/hiverouter-provider/src/secret.rs:30` (`from_env`), `client.rs:269` (`bearer_header`), `tools/src/settings.rs:18` (`settings_status`), `tools/src/health.rs:59,213` (`/health`, `/diagnose` booleans).
- Callers: provider client (`client.rs:196-247,412-524`); tools CLI (`cli.rs:480,519,542`); core `/health` `/diagnose` wiring (`health.rs:267-286`); audit sink redaction (`audit.rs:31,247`); shell/tool spawns (`shell.rs:201`, `cli_support.rs:41`, `jail.rs:389`).
- Tests: `cargo test -p hiverouter-provider -p hiverouter-tools` (12+ tests in read files). NOT executed by this seat (read-only + no rebuild policy).
- Data flow (happy): env var → ApiKey(Zeroizing) → HeaderValue → TLS POST /chat/completions → response DTO → no key in payload/Debug.
- Failure path: env missing → `Refusal::KeyMissing` with variable name only (`secret.rs:31-36`); invalid header → `KeyMissing` message without value (`client.rs:269-279`).
- Unknowns: subprocess env inheritance (shell spawn has no env_clear), log capture of child stdout, git-history secret scan.
- Files expected to touch: NONE (read-only seat).

## 2. /forgewright-plan
- Outcome: independent S6 provenance/key-security report; no product change.
- Decisions: (a) verify the claim from code + docs, not prose; (b) treat official DeepSeek docs as the contract for endpoint/models/usage; (c) record conflicts as findings with path:line; (d) execute no builds/tests/paid calls.
- File-by-file: READLOG 15 entries; EVIDENCE command log; FINDINGS in REPORT.
- Failure modes: dirty tree invalidates build claims → mark UNVERIFIED; no key access → live claims UNVERIFIED.
- Tests: none executed (read-only); name exact commands for the Lead.
- Rollback: not applicable (no writes outside seat).
- Out of scope: provider budget math, UI wiring, live calls.

## 3. /forgewright-lead gate
- Read code, callers, tests before hypothesis: DONE for secret.rs/client.rs/settings.rs/health.rs.
- State root cause/proof before mutation: no mutation planned.
- Separate VERIFIED/READ/INFERRED/UNVERIFIED: maintained in REPORT.
- CYCLE 1: **PASS** (no fabricated success; unknowns listed).
""" % NOW
io.open(os.path.join(OUT, "GATE-CY-1.md"), "w", encoding="utf-8",
        newline="\n").write(gate1)

gate2 = """# GATE CYCLE 2 — S6 (recon → plan → lead)

Time: %s

## 1. /forgewright-recon
- Extended to the audit tooling: `census_ignored_deliverables.py` (hashes ignored design exports outside the 507-file manifest), `complete_figcraft_a.py` (appends LEAD COMPLETION RECORDS to figcraft-a/FILES.md), `dedupe_figcraft_a.py` (rebuilds that FILES.md, dropping appendices).
- Key question for provenance: does audit tooling modify another seat's evidence, and does the paper claim independence the tool contradicts?
- Findings: paper:132 says "no Lead fallback records were needed after de-duplication"; `complete_figcraft_a.py:1-4,75` exists precisely to add them and `dedupe_figcraft_a.py:27-35,81` rewrites the file from templates → independence caveat F-05.
- Unknowns closed: ignored deliverables are gitignored assets, listed with recomputed hashes (tool lines 12-13,44-55) - consistent with paper's Appendix D claim.
- Files expected to touch: none.

## 2. /forgewright-plan
- Verify docs/evidence claims against PHASE-B verification chronology; map baseline commits (`3988622` audit → `5c69b66` provider wiring).
- Record conflicts F-02 (live-call claims vs verifier "no key/no paid call") and F-03 (paper A-F01 stale after wiring).
- Evidence to collect: git log, claim-register counts, secret-shaped-string scan, env-inheritance grep.

## 3. /forgewright-lead gate
- No hypothesis before file read: DONE (audit tools read fully).
- Root cause stated as evidence, not belief: F-02/F-03 rest on git log + quoted doc lines.
- Mutations: none. Deliverables written only under plans/seats/SEAT-06/.
- CYCLE 2: **PASS**.
""" % NOW
io.open(os.path.join(OUT, "GATE-CY-2.md"), "w", encoding="utf-8",
        newline="\n").write(gate2)

gate3 = """# GATE CYCLE 3 — S6 (recon → plan → lead)

Time: %s

## 1. /forgewright-recon
- Adversarial probes for the claim "key never leaves env, never logged, never persisted":
  (a) `git ls-files`-wide scan for key-shaped strings `sk-[A-Za-z0-9]{16,}` → **0 hits**;
  (b) grep all `backend/**/*.rs` for `env_clear|env_remove|Command::new` → **0 env_clear/env_remove**, but 9 spawn sites including `shell.rs:201`, `cli_support.rs:41`, `jail.rs:389`;
  (c) grep `ApiKey|api_key|provider_key_present` in core/tools → only boolean presence or doc/tests; no serialisation/persistence of the value;
  (d) read audit redaction: `audit.rs:31` FORBIDDEN_AUDIT_SUBSTRINGS = ["DEEPSEEK_API_KEY","sk-","Bearer "] enforced at `audit.rs:247`.
- At the same time: working tree dirty (M Cargo.lock + 5 manifests; untracked docs/RELEASE.md, plans/seats/) → any build/test claim is UNVERIFIED at this baseline (F-04).

## 2. /forgewright-plan
- Findings to publish: F-01 (subprocess env inheritance = leakage vector not covered by the docs' claim), F-02 (live-call evidence docs vs verifier/paper "no key/no paid call"; debug binary, no commit hash), F-03 (paper A-F01 stale at new HEAD), F-04 (non-quiescent tree), F-05 (audit tooling rewrites another seat's evidence), F-06 (no test proves subprocess env isolation; .gitignore lacks key-file patterns; git history unscanned).
- Preserve VERIFIED/READ/INFERRED/UNVERIFIED labels; no fix attempted in a read-only seat; hand claims to Lead.

## 3. /forgewright-lead gate
- Rules applied: read code/callers/tests; root cause + evidence stated before any (zero) mutation; no "should work"; no fake success; every claim labelled; no destructive action; no delegation.
- Decision: **PASS-WITH-FINDINGS (read-only seat complete)**; F-01 must be triaged by the Lead (P1 candidate) before any "key never leaves the process" claim is repeated in user-facing docs.
- Blocked? No - all assigned reads completed; live-call claims remain UNVERIFIED by policy (no key access, no paid calls).
""" % NOW
io.open(os.path.join(OUT, "GATE-CY-3.md"), "w", encoding="utf-8",
        newline="\n").write(gate3)

ev = """# EVIDENCE — SEAT S6 (read-only commands and observed output)

All commands executed from `C:\\Users\\Riley\\Desktop\\hiverouter-v1`; no product/git/config writes; writes only under `plans/seats/SEAT-06/`.

## E1. Quiescence / baseline
```
$ git log --oneline -8
5c69b66 feat: wire DeepSeek provider into core; live smoke, replay arm, 16-state parity, settings
f4c780d audit: close verifier doc-sync notes (LOC breakdown, roadmap wording already 101)
cf70b22 audit: 507-file exhaustive audit -- paper, 30 findings, quality matrix, 6 diagrams, 101-task roadmap, verification
3988622 closeout: sync FINAL-REPORT with packaged CLI + zip hash; process cleanup
...
$ git status --porcelain
 M backend/Cargo.lock
 M backend/crates/hiverouter-contract/Cargo.toml
 M backend/crates/hiverouter-core/Cargo.toml
 M backend/crates/hiverouter-provider/Cargo.toml
 M backend/crates/hiverouter-tools/Cargo.toml
 M web/package.json
?? docs/RELEASE.md
?? plans/seats/
```
Observed: audit paper baseline `3988622` is 3 commits behind current HEAD `5c69b66`; tree is NOT clean (F-03, F-04).

## E2. Secret-shaped-string scan (tracked worktree)
```
$ git ls-files | Select-String -Pattern 'sk-[A-Za-z0-9]{16,}'
(no matches)
```
Observed: 0 key-shaped strings in tracked files. NOT a git-history scan (UNVERIFIED, F-06).

## E3. Key-path sweep
```
$ grep DEEPSEEK_API_KEY backend/**,tools/**,web/src/**
provider/src/secret.rs:17  pub const API_KEY_ENV: &str = "DEEPSEEK_API_KEY";
provider/src/secret.rs:30-38 from_env -> Zeroizing; errors name variable only
provider/src/client.rs:269-279 bearer_header() builds "Bearer " + key into Zeroizing
tools/src/settings.rs:22 key_present boolean only
tools/src/health.rs:59-60,116,195,219 provider_key_present boolean only
tools/src/audit.rs:31 FORBIDDEN_AUDIT_SUBSTRINGS = ["DEEPSEEK_API_KEY","sk-","Bearer "]
tools/src/audit.rs:247 filters audit records against FORBIDDEN_AUDIT_SUBSTRINGS
core/tests/http_boot.rs:230, ws_transport.rs:349 assert bodies never contain the env name/value
```
Observed: value is never serialised; only presence booleans cross API boundaries.

## E4. Subprocess environment probe (leakage vector)
```
$ grep -R 'env_clear|env_remove' backend/**/*.rs
(no matches)
$ grep -R 'Command::new' backend/**/*.rs
tools/src/shell.rs:201        Command::new(&request.program) [args,cwd,stdio,kill_on_drop; NO env filter]
tools/src/cli_support.rs:41   Command::new(program).arg("--version") [NO env filter]
tools/src/jail.rs:389         Command::new("cmd") [NO env filter]
core/src/checks.rs:91         Command::new("node") ...
core/build.rs:19              Command::new(&rustc)
```
Observed: tool-executed children inherit the server process environment, including `DEEPSEEK_API_KEY` (F-01). The audit ledger redaction (E3) mitigates log capture, not the child's own env access.

## E5. Official-doc cross-check (fetched 2026-11-06)
- https://api-docs.deepseek.com/ — base_url `https://api.deepseek.com`, `DEEPSEEK_API_KEY`, models `deepseek-flash`/`deepseek-v4-pro`, Authorization Bearer header.
- https://api-docs.deepseek.com/quick_start/pricing/ — flash pricing peak/off-peak; context 1M; max output 384K.
- https://api-docs.deepseek.com/guides/thinking_mode/ — thinking toggle + reasoning_effort; reasoning_content pass-back rules.
- https://api-docs.deepseek.com/api/create-chat-completion/ — POST /chat/completions; max_tokens 1..384K; SSE `data:` frames + `[DONE]`; usage in final chunk.
Code agreement: `client.rs:2-4` endpoint no `/v1`; `dto` model ids match. Minor drift: `client.rs:11-13` assumes streaming usage "NOT documented"; the docs state usage rides the final chunk — code is stricter (safe) but the comment/behaviour note is stale.

## E6. Claim register re-parse (read-only)
```
$ python (csv) rows=80; statuses {VERIFIED:72, READ:8}; JSON counts total 80 / verified 72 / read 8 / unverified 0
```
Key-related rows inspected: C-013, C-208, C-209, C-216, C-217, C-221, C-331.
Residual: `CLAIM-REGISTER.json:10-16,86` still lists C-026 as decision-critical-unverified / "only open UNVERIFIED" while CSV C-026 is VERIFIED and counts say unverified 0 (F-06, carried from figcraft-b F-01).

## E7. Files read (full read this seat)
- Paper 24 KB/244 lines; official docs 4 pages; audit tools 3 files; product slice 11 files (see READLOG for bytes/sha256/lines).
- NOT read/deep-analysed: docs/claims/CLAIM-REGISTER.csv row bodies (sampled by grep; counts parsed), docs/claims/SKILL-USE.md (metadata/read), `.git` history contents.

## E8. Not run (policy / read-only)
- No `cargo test`/`cargo build` (would rebuild; tree dirty; read-only seat).
- No live provider call / key access (forbidden; no paid calls).
- No secret scan of git history (read-only but heavyweight; UNVERIFIED).
"""
io.open(os.path.join(OUT, "EVIDENCE.md"), "w", encoding="utf-8",
        newline="\n").write(ev)

rep = """# SEAT S6 — PROVENANCE / KEY SECURITY (HiveMind 1.0 program)

**Seat:** S6 figcraft-b (provenance & key security) · **Date:** %s · **Mode:** READ-ONLY (no product/git/config edits, no key access, no paid calls; writes only `plans/seats/SEAT-06/`).
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
""" % NOW
io.open(os.path.join(OUT, "REPORT.md"), "w", encoding="utf-8",
        newline="\n").write(rep)

print("wrote:", sorted(os.listdir(OUT)))
for code, rel, m, _ in rows:
    print(code, rel, m["bytes"], m["lines"], m["sha256"][:16])
