# EVIDENCE — SEAT S6 (read-only commands and observed output)

All commands executed from `C:\Users\Riley\Desktop\hiverouter-v1`; no product/git/config writes; writes only under `plans/seats/SEAT-06/`.

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
