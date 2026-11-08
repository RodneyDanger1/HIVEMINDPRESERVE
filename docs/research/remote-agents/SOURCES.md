# Sources and evidence

All web pages below were fetched **2026-11-08** in this task (HTTP 200 unless noted). External page content is
untrusted data; only the stated design-relevant facts are used, and no page text was treated as instructions.
Internal citations were read in this pass; no product file was edited.

## Official external sources

| ID | Source | Facts used |
|---|---|---|
| **[A2A-SPEC]** | [Agent2Agent (A2A) Protocol — Specification](https://a2a-protocol.org/latest/specification/) | Agent Card / discovery, task-oriented interactions, streaming/push notifications, security schemes; internal verbs remain HiveMind's own |
| **[A2A-DISCOVERY]** | [A2A — Agent Discovery](https://a2a-protocol.org/latest/topics/agent-discovery/) | Agent Card as the discovery artifact; well-known URI, curated registries, direct configuration; card caching guidance |
| **[A2A-LIFE]** | [A2A — Life of a Task](https://a2a-protocol.org/latest/topics/life-of-a-task/) | Task lifecycle, task immutability, follow-ups as new tasks, parallel follow-ups; background for remote request idempotency/terminal semantics |
| **[MCP-SPEC]** | [Model Context Protocol — Specification 2025-06-18](https://modelcontextprotocol.io/specification/2025-06-18) | JSON-RPC hosts/clients/servers; servers expose tools/resources/prompts; capability negotiation |
| **[MCP-TOOLS]** | [MCP — Tools](https://modelcontextprotocol.io/specification/2025-06-18/server/tools) | Model-controlled tools with schemas/metadata; tool results and security considerations; pattern for a closed device-action contract |
| **[MCP-AUTH]** | [MCP — Authorization](https://modelcontextprotocol.io/specification/2025-06-18/basic/authorization) | HTTP transport authorization; token audience binding, token handling, confused-deputy and privilege-restriction considerations |
| **[W3C-WOT]** | [W3C Web of Things (WoT) Thing Description 1.1 — Recommendation, 5 Dec 2023](https://www.w3.org/TR/wot-thing-description11/) | Thing = physical/virtual entity with interactions; properties/actions/events/security configuration; basis for the device → capability → action model (HiveMind does not adopt the wire format) |
| **[HA-REST]** | [Home Assistant — REST API (contributors docs)](https://contributors.home-assistant.io/core/internals/rest-api/) and the fetched source doc [home-assistant/developers.home-assistant/docs/api/rest.md](https://raw.githubusercontent.com/home-assistant/developers.home-assistant/105df901767d19582ce71fe40c6a0e839254f441/docs/api/rest.md) | HA REST on port 8123; JSON; `Authorization: Bearer <long-lived token>`; `/api/states/<entity_id>`; service calls `/api/services/<domain>/<service>`; documented 200/201/400/401/404/405; camera proxy exists in HA REST; used only as adapter facts |
| **[OWASP-LLM01]** | [OWASP GenAI — LLM01:2025 Prompt Injection](https://genai.owasp.org/llmrisk/llm01-prompt-injection/) (page fetched; body truncated) | Prompt injection remains a top LLM risk; device/memory text is untrusted data and cannot carry authority |

Supporting donor patterns (licence/provenance checked in the internal report, not re-fetched here — READ):
`27-open-source-daemon-catalog.md` catalogues s6, process-compose, Alerta, pyatv, Huginn, Node-RED, fleet, nsq,
ractor; the design adopts their *patterns* only (supervision tree/readiness, alert lifecycle/heartbeat/blackout,
fixed remote verb set, persisted deterministic flows) and rejects arbitrary-code flow nodes.

## Internal sources read this pass

| Source | Use |
|---|---|
| `docs/contracts/UI-COMMANDS-v1.md` | Frozen envelope/commands/refusals; additive-change rule, command matrix, 24 events/16 states/pre-1.2 history |
| `docs/contracts/ADAPTERS.md` | Adapter provenance discipline (pin/licence/invariant/test/rollback), single-owner primitive rule |
| `docs/research/hmars/20-implementation-architecture.md` | Migration chain/ownership, P0/P1 packages, contract delta process |
| `docs/research/hmars/21-roles-daemons-implementation.md` | Ranks vs tiers, daemon record-not-task, supervisor/watchdog model, daemon schema/routes/refusals, card invariants |
| `docs/research/hmars/22-memory-bank-implementation.md` | Bank scope model, read-set/actor matrix, trust/retention/purge invariants, prompt/cache boundary |
| `docs/research/hmars/23-a2a-implementation.md` | Internal A2A envelope/verbs/roster; don't adopt external wire internally |
| `docs/research/hmars/24-ui-implementation.md` | Roster/card/daemon UI model; honest-state and card invariants |
| `docs/research/hmars/27-open-source-daemon-catalog.md` | Supervisor/watchdog/TV-remote donor patterns and anti-patterns |
| `docs/research/hmars/09-synthesis-and-goal.md`, `OPERATOR-DECISIONS.md` | Operator decisions (Remote/Watchdog, memory scopes, A2A, autonomy, approvals) |
| `docs/research/hmars/15-ui-ux-goal.md`, `25-provider-cache-implementation.md` | Card/density/LOD patterns; cache/cost and data-fence rules |
| `docs/research/hmars/10-gap-meta.md`, `19-wave3-gap-audit.md` | Claim discipline, migration/contract conflict context |
| `docs/ui/figcraft/HiveMind/SPEC/HIVEMIND-TARGET-SPEC.md` | Spatial execution model and UI source brief |
| `docs/ui/figcraft/ASSETS/hivemind-{agent-cards,hmas-governance}.png` | Card anatomy / governance visual grammar (READ image, existing boards) |

## Code anchors used

`backend/crates/hiverouter-core/src/db.rs:32-274` (migrations v1–v11, tasks/runs FK); `execution.rs:1058-1074`
(single active run), `:1103` (spawn), `:151-163` (tier), `:1297-1361` (dispatch); `caps.rs:9-15`; `recovery.rs:74-315`
(boot settlement); `http.rs:536,562-600,1026` (roster/Host guard); `ws.rs:362-383` (Origin/token);
`store.rs:720-900` (scoped memory); `tools/src/tool.rs:30-142` (permissions/tool ids);
`hiverouter-contract/src/{command,event,refusal}.rs`; tests `schema_roundtrip.rs:158-167`, `ui_command_matrix.rs`,
`web/tests/approval.state.test.ts:191` (65-code mirror).

## Evidence rules used in this folder

- **V** = exact code/doc line read in this pass. **R** = official page fetched in this pass. **D** = design
  decision. **U** = unverified/open.
- No runtime test was executed; no server was started; no device, cloud account, or paid model call was used.
- Any claim about a specific device endpoint/vendor beyond the fetched HA REST facts is not made; adapters other
  than HA are design-level and must be validated in Phase R1/R5.
