# 09 — Verified Risks and Gaps

## Product/runtime gaps

1. **No chat-history endpoint.** `chat_messages` is persisted but no HTTP route exposes it; web hydration reports `messages-unavailable` and does not invent messages.
2. **Four Library templates missing assets.** Real lookup/insertion exists, but only `planner` has prompt/DAG assets; content-writer/data-analyst/code-reviewer/chatbot still refuse.
3. **Nested file listing absent.** `FILE_TREE_LIST` has no path payload; directories are disabled honestly, not falsely read.
4. **Persona loader absent.** Engine enforces task tier + scoped allowed_tool_ids; no Persona registry is wired.
5. **NODE `kind` absent.** Contract supports it; TaskRecord lacks authoritative tier, so producers omit rather than guess.
6. **Worker spawn policy.** `spawn_agent` is in Hyper/Super allowlists; worker-role tasks are correctly refused.
7. **Spawn-vs-terminal race.** A spawn accepted between queue-drain and terminal status can leave a queued task settled without execution; documented in code.
8. **Memory/research/learning loops absent.** No product code or tables.

## Proof/release gaps

- No 1.2 live paid proof, no live tool-turn/approval/security/resync run.
- No 1.2 tag, artifact, manifest, or seal; only sealed 1.1.
- Trace: 1 WIRED / 49 UNVERIFIED / 5 MISSING / 264 PENDING; raw capture empty.
- Axe packages absent; G-UI-3/G-UI-10/axe exclusions open.
- Key rotation/runtime-test purge are operator actions.
- `docs/CODEMAP` is documentation, not product verification.

## Data/security risks

- Tracked SQLite/WAL/SHM fixtures can contain prompts/approvals; redaction/reset policy needed.
- Three identical EXEs and 13 identical DBs inflate repo; provenance/hashes recorded.
- 11 fixture-token hits are fake test values; no real secrets found in pattern scan. `.env*` absent.
- No first-party LICENSE/NOTICE; dependency/license census incomplete.
- No `.gitattributes`; CRLF/LF makes hashes/citations unstable across checkouts.

## Documentation risks

- `audit/inventory/FILE-MANIFEST*.json` is stale (507 tracked at an older HEAD); current is 805 at `e021b12`.
- Many plans/reports describe intermediate states; only source + frozen inventory are current.
- Earlier “404/441 backend” and “5 failed web” claims are superseded by the committed final 475/0, 179/179, 106/0 receipts.
