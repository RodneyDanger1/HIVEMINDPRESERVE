# SEAT-08 READLOG — docs/plan compiler (every source read one at a time)

Read-only seat. Timeline: 2026-11-06 10:59–11:30 America/Chicago. Order enforced: paper → 4 official docs → assigned audit files → product slice.

## §1 Mandatory first read

| # | Source | Bytes | Lines | sha256 / fetch | Read at (CT) | Verified facts |
|---|---|---:|---:|---|---|---|
| 1 | `audit/HIVEMIND-AUDIT-PAPER.md` | 25,141 | 167 | `3BAAD198337564E1BD92ED23580A926426F92E2D4A9D64F260DBAF4D6713D495` | 11:00 | 507 tracked files/63.98 MiB/188 code/56 deep/36,802 LOC; 101-task/6-phase/8-seat roadmap; 30 consolidated findings (3 P0, 10 P1, 16 P2, 1 info); P0: A-F01 core lacked provider dependency (live run impossible), A-F02 replay nodes hardcoded `[]`, A-F03 16 contract states vs 15 UI; 233 backend/46 frontend/79 e2e pass/14 skips; no paid-provider run at paper time; subject HEAD `3988622` |

## §2 Official DeepSeek docs (fetched in full; untrusted data treated as facts only)

| # | URL | HTTP | Fetched (CT) | Verified facts captured |
|---|---|---|---:|---|
| 2 | <https://api-docs.deepseek.com/> | 200 | 11:01 | base_url `https://api.deepseek.com`; models `deepseek-flash` (DeepSeek-V4.1-Flash-1017) / `deepseek-v4-pro`; `DEEPSEEK_API_KEY`; `POST /chat/completions`; `thinking{type:enabled}` + `reasoning_effort` |
| 3 | <https://api-docs.deepseek.com/quick_start/pricing/> | 200 | 11:01 | per-1M flash: cache-hit 0.003 off-peak/0.006 peak; miss 0.15/0.3; output 0.6/1.2; v4-pro 0.022/0.044, 0.66/1.32, 1.98/3.96; concurrency 2500/500; peak 01–04 & 06–10 UTC Mon–Fri excl. CN holidays |
| 4 | <https://api-docs.deepseek.com/guides/thinking_mode/> | 200 | 11:02 | thinking default on, effort high; `thinking.type` enabled/disabled; `reasoning_effort` none/low/high/max (minimal→low, medium/xhigh→high); temperature/presence/frequency ignored in thinking; `top_p` rules |
| 5 | <https://api-docs.deepseek.com/api/create-chat-completion/> | 200 | 11:02 | `POST /chat/completions`; `max_tokens` 1..384K (defaults 8K/64K/128K); response usage incl `prompt_cache_hit_tokens`, `prompt_cache_miss_tokens`, `completion_tokens`, `reasoning_tokens` |

Note: very long pages were partially truncated at the tail by the fetch tool; every section used for the claims above was present. No paid call made; no key material accessed.

## §3 Assigned audit files (read in full, one at a time)

| # | Source | Bytes | Lines | sha256 | Read at | Verified facts |
|---|---|---:|---:|---|---|---|
| 6 | `audit/tools/build_appendix.py` | 1,859 | 30 | `F918D3625FFFDB521FE57CC8FA0BC6989396AA5138A8B4F6F263A7A18EA54DBC` | 11:03 | reads `FILE-MANIFEST-ENRICHED.json`; seat A–H→names; writes `APPENDIX-FILE-CENSUS.md`; deterministic doc generator only |
| 7 | `audit/tools/synthesize_paper.py` | 10,176 | 86 | `18D200424493161ACD2F98E8AFEFAAC396CF2B904EBEE91BD7A0792829AE8CD4` | 11:04 | applies string-replace synthesis to paper (abstract 26→30 findings, §4.2/4.3, 6.3, limits, appendix rows); fragile if paper text drifts; no product facts |

## §4 Product slice (read in full, one at a time, then analysis)

| # | Source | Bytes | Lines | sha256 | mtime | Read at | Facts / claims |
|---|---|---:|---:|---|---:|---|---|
| 8 | `plans/HIVEMIND-1.0-SEAT-ASSIGNMENTS.md` | 3,629 | 25 | `4F688B242C961044B7C34E0117A23632D7623EC475169D2FAB68D144A65951EC` | 10:57 | 11:05 | program baseline: HEAD includes provider wiring, live smoke, replay arm, 16-state web vocab; mandatory protocol; S8 = these 2 audit files + docs slice; read-only/no-key rule |
| 9 | `docs/RELEASE.md` | 3,287 | 25 | `10D32112813D58E120B1F7D8E2E8F9C963C51DB9E9DCCD95E0006AFAF1E8F014` | 10:59:58 | 11:06 | **draft** release notes; claims provider wiring + live key validation + live server path + 16/16 state parity + replay backend + tests green "after change"; references `docs/RELEASE-EVIDENCE.md` which does **not exist**; says update hashes at final build |
| 10 | `docs/reports/FINAL-REPORT.md` | 6,121 | 62 | `978271BE811ACDFD97FD55F5A9F3E694E18FA78CCC9DC8F4F893073B0148514D` | 09:34:52 | 11:07 | shipped-artifact report: binary `184D51A7…`; "15-state enum"; trace "260 UNVERIFIED; 5 MISSING"; "No paid provider run"; claim register 80 rows 72/8/0; trace/e2e counts pre-integration |
| 11 | `docs/evidence/key-validation.md` | 1,038 | 14 | `71EA686846CC5055576C6FC86C388C296A81AB003B5D027AA556178D8323A2A6` | 10:57:17 | 11:08 | live key validation HTTP 200, content `HIVE-KEY-OK`, usage 20; key custody statement; docs-read list matches §2 |
| 12 | `docs/evidence/live-provider-smoke.md` | 1,312 | 12 | `9ED04F4581BA7500FD5398E046CEAADA9887C88AA8906E61F1145B1263E18EEB` | 10:57:18 | 11:08 | live server `SUBMIT_PROMPT` → real provider → `RUN_STATUS completed`, reply `HIVE-LIVE-OK`, usage 95; single-delta limitation; replay UI still pending at that point |
| 13 | `README.md` | 1,742 | 21 | `B60FAC5A2A7DDFB22B4ED50A0C2AD265DA3D7687E659ABDCA3FABB6542010852` | 09:25:44 | 11:09 | shipped-v1 status: binary `184D51A7…`; zip `9B2ED5A7…`; "no paid provider run" in e2e line; points to FINAL-REPORT |
| 14 | `tools/runbook.md` | 4,396 | 65 | `E7FBA10F3332CA65FD3F81873AA2916916ADA14EA7F889BE8172C805C5B72CFC` | 04:54:41 | 11:10 | prerequisites/build/first-run/stop; named refusals; no provider live-verify step; no installer claim |
| 15 | `docs/plans/PHASE-12-EVIDENCE.md` | 9,448 | 115 | `74CA606808C286DF94D638A6AC15F937602332AEC275FAB45F8406BEA51D169F` | 09:25:44 | 11:11 | phase-12 evidence; final section: binary `184D51A7…`; **zip 4,065,104 B `9B2ED5A7…`**; e2e 79/0/14; backend 233; live probes |
| 16 | `docs/plans/D2-D3-WIRING-EVIDENCE.md` | 7,031 | 86 | `77F6D5CD0BFD350534F430C9E37CC10735E50CEB6B760BB71F19C1BDB1D1BCB8` | 08:26:28 | 11:12 | D-2/D-3 wiring evidence; intermediate release hash `DE676AFB…`; WS/HTTP probe transcript |
| 17 | `audit/VERIFICATION.md` | 9,937 | 67 | `99C3A398CEAF73369135FCF2B64382CD264684C2544EA51D662C000019B32C45` | 09:52:13 | 11:13 | Lead-compiled audit verification PASS-WITH-NOTES; covers 507/507, paper numbers, audit gaps |
| 18 | `audit/ROADMAP.md` | 13,804 | 114 | `4D69A25AD60BE517799256B0CC84E549395986A270D2378D6DB4F98C79E98756` | 09:44:01 | 11:14 | 101 tasks / 6 phases / 8 seats; P1 15, P2 20, P3 20, P4 15, P5 15, P6 16; task acceptance wording |
| 19 | `audit/ROADMAP-TASKS.csv` | 35,038 | 102 | `06869F6C6E8BD7A848004FF0E1D90D8643BD4EE5D1D64727741522F2B6FF6042` | 09:39:29 | 11:14 | 101 rows (parsed fully); seat counts 10–14; 11 columns incl acceptance/evidence_required |
| 20 | `docs/claims/CLAIM-REGISTER.csv` | 24,292 | 81 | `1518294B77D253239069F26A19C9EFC2A3FB955E43E4F0ABEFF1C40BB61C9AD7` | 09:29:23 | 11:15 | 80 claim rows parsed fully: 72 VERIFIED / 8 READ / 0 UNVERIFIED; C-026 VERIFIED; C-340 records **zip 4,065,104 B `9B2ED5A7…`** (contradicts actual zip) |

## §5 Code-reality spot-checks (read-only, for release-doc verification)

| Source | Evidence | Read at |
|---|---|---|
| `backend/crates/hiverouter-core/Cargo.toml` | line 21 `hiverouter-provider = { path = "../hiverouter-provider" }` (A-F01 fixed at HEAD) | 11:16 |
| `backend/crates/hiverouter-core/src/http.rs` | `:25` import `ProviderExecutorAdapter`; `:67 build_executor`; `:68` provider.enabled gate; `:72 from_env`; `:79` `UnconfiguredExecutor` fallback | 11:16 |
| `backend/crates/hiverouter-core/src/provider_executor.rs` | 10,589 B, mtime 10:56:06 (new live adapter) | 11:17 |
| `backend/crates/hiverouter-core/tests/provider_live.rs` | 2,491 B, mtime 10:41:04 (live-path test) | 11:17 |
| `web/src/state/agentState.ts` | 16 states incl `interrupted`; header "16 states" (A-F03 fixed at HEAD) | 11:18 |
| `web/src/flow/Canvas.tsx` | `:54` `history.replayGraph`; `:113-122` replay tasks→nodes/edges (A-F02 fixed at HEAD) | 11:18 |
| `web/dist/index.html` | mtime 10:58:47 (newer than shipped binary) | 11:19 |

No product/git/config writes were made by this seat.
