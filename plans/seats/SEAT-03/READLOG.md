# READLOG — SEAT S3 (UI-State/Replay)

Read-only seat. Clock sample used for timestamps: `2026-11-06T11:04:25-06:00` (local). Every entry was read fully, in the mandated order: paper → official docs → audit diagram → each product file one-by-one → tests. Hash/line counts were computed locally on the same bytes that were read.

## A. Mandatory order

| # | Item | Bytes | Lines | SHA-256 / URL status | Read at | Verified facts recorded |
|---:|---|---:|---:|---|---|---|
| 1 | `audit/HIVEMIND-AUDIT-PAPER.md` | 25,141 | 244 | `3BAAD198337564E1BD92ED23580A926426F92E2D4A9D64F260DBAF4D6713D495` | 11:00 | 8-seat/507-file audit; P0 A-F02 dead replay; P0 A-F03 16-vs-15 parity; P1 A-F05 settings refusal; P2 mock-import/timeouts/favicon; paper checkpoint = HEAD 3988622 (stale vs current HEAD 5c69b66) |
| 2 | `https://api-docs.deepseek.com/` (first API call) | — | — | HTTP 200 | 11:01 | OpenAI/Anthropic-compatible base URLs; models `deepseek-flash`, `deepseek-v4-pro`; OpenAI SDK sends `thinking` via `extra_body`; stream flag; DEEPSEEK_API_KEY env |
| 3 | `https://api-docs.deepseek.com/quick_start/pricing` | — | — | HTTP 200 | 11:01 | flash vs v4-pro: context 1M, max output 384K; thinking default on; off-peak/peak pricing table (flash $0.003/$0.006 cache-hit, $0.15/$0.3 cache-miss, $0.6/$1.2 output per 1M); concurrency 2500/500 |
| 4 | `https://api-docs.deepseek.com/guides/thinking_mode` | — | — | HTTP 200 (fetch truncated at ~tool-calls sample) | 11:02 | `thinking {type: enabled/disabled}`; `reasoning_effort` low/high/max with real mappings (minimal→low, medium→high, xhigh→high, ultra→max); default high; `reasoning_content` returned; temperature/penalties ignored; tools mode requires reasoning_content replay |
| 5 | `https://api-docs.deepseek.com/api/create-chat-completion` | — | — | HTTP 200 (fetch truncated after usage schema) | 11:03 | `thinking` object nullable with `type required`; `reasoning_effort` values none/low/high/max (minimal/medium/xhigh accepted aliases); `max_tokens` 1–384K, defaults 8K non-thinking/64K thinking/128K at max; streaming SSE w/ usage on last chunk; `finish_reason` includes `insufficient_system_resource`, `aborted` |
| 6 | `audit/diagrams/04-state-inventory.mmd` | 619 | 9 | `ADF5BBA937A1561D362D6E1B70BBAA5DA444EEB3E59C61B097130A2B36C4E6F6` | 11:03 | Declared inventory **16 variants**; five terminal (`failed·completed·cancelled·killed·interrupted`); transitions not enumerated in `state.rs` (diagram note line 8) |

## B. Product slice (read one-by-one; assigned order)

| # | Path | Bytes | Lines | SHA-256 | Read at | Verified facts recorded |
|---:|---|---:|---:|---|---|---|
| 7 | `web/src/flow/Canvas.tsx` | 9,176 | 256 | `03153FB57D694E810D168A69B12438D5804BAF2DBE60A17381CECF9C83C0C9DB` | 11:04 | Replay branch lines 113-136 (`buildReplayTasks`→`buildNodes`/`buildEdges`→ReactFlow); minimap a11y 96-110; zoom 0.1/3 139-140; header claims read-only replay (5-6); live `onNodeClick` at 146; banner 183-189 |
| 8 | `web/src/flow/graph.ts` | 4,818 | 147 | `6C0F055768AFD5DE8FE10789E9B70D26F01B4702A66D6A140BCA534824CCA842` | 11:04 | `buildReplayTasks` 93-131 expects `graph.nodes[].{task_id,parent_task_id,state,rev}`; silently `{}` on malformed; forces kind worker/label `task <id8>`; orphan parent dropped; `buildNodes` sets draggable/selectable true 64-72 |
| 9 | `web/src/state/store.ts` | 23,799 | 748 | `8B3E95A9E2D03874060237BD4689E90B9FC065A76EBD018F5B872F0054A03374` | 11:05 | HISTORY_SNAPSHOT stores graph 385-393; settings result thinking/reasoning_effort 346-361; probeSettings guard+refusal attribution 662-684; guardActiveRun 434-441; pending/timeouts 401-432; replay type 69-72 |
| 10 | `web/src/state/agentState.ts` | 4,671 | 134 | `CE54BC65357E7DE846644AB1F150F428F3CA39D3745FCCF8F245D400A005ED03` | 11:05 | **16 states incl interrupted** 9-26; four dots 30; shapes incl interrupted triangle 73; `isTerminal` 111-118 omits `failed`; isRunActive 120-122; transitions 87-104 |
| 11 | `web/src/state/strings.ts` | 12,830 | 378 | `951FFC64743C246FC0C94939EC12C3334C30F8D67A095A6E2796A524B9ED9E19` | 11:05 | 16 state labels 166-183 (interrupted:182); settings thinking/effort labels 155-156; mock label 54; seed/demo strings 239-249; client refusal copy 251-331; pure formatters 337-378 |
| 12 | `web/src/panels/SettingsPanel.tsx` | 5,615 | 128 | `EB270A7C3DF66E25C7BA08B48EFE3E088DCB271002EA571A0F76273032B9C948` | 11:05 | Renders provider/model/thinking/effort/key 65-78; typed refusal 51-64; single effect guard 15-24; health section 82-105; provider/model fallback uses keyUnknown 67/69 (finding) |
| 13 | `web/src/panels/Chat.tsx` | — | — | **DOES NOT EXIST** | 11:05 | Assigned path absent in repo/HEAD; substituted the real chat component (next row); recorded as D8 |
| 14 | `web/src/chat/ChatCore.tsx` | 6,270 | 154 | `E924BB44FA76476B2342EF7554E9026D7523DC432C8A869F07FFF4869FA78FBE` | 11:05 | Pinned chat/composer/streaming; submit clears draft before validation result 51-56/131-137 (finding); pending disable 25-28; aria-live 95-99 |
| 15 | `web/src/api/contract.ts` | 29,353 | 875 | `ECEC7A5AB52E11C38C916FD370DCBA4EC99EB74670C971ED086AEAB5A7B8A275` | 11:06 | 14 commands 102-117; 12 server commands 120-133; 21 events 294-316; 45 documented refusal codes 332-378; `HistorySnapshotPayload.graph: unknown` 235-239; settings result optional thinking/reasoning_effort 242-248; approval nonce optional 223-229; NODE metadata optional 208-216; parse/decode 413-776; jail/prompt/zoom validators 823-866 |
| 16 | `web/src/api/index.ts` | 751 | 19 | `6D6E3618DC4918CC974344FB3DCA84A1E7FFF0695FCC5485D93D391DA5906A76` | 11:06 | Real client default; mock only when `DEV && VITE_UI_MOCK==='1'` 14-19; static import 8 + re-export 12 (finding D6) |
| 17 | `web/src/api/client.ts` | 15,930 | 497 | `6EFA1C7D1DEE8D045C31AA3F326DA1E6B9080DEDB913973ECF03A5F3082E8CA7` | 11:06 | Session fetch no abort/timeout 235-238; handshake timeout 8s 389-419; command timeout 15s 443-452; ACK-only resolve 339-359; typed ERROR reject 360-379; hello control frame 283; reopen hello 289-294; close fails pending 295-305 |
| 18 | `web/src/api/health.ts` | 1,885 | 54 | `84963F8C645ED6C04272C69977A343EBF1E9151F3FC89FE23EE89A31739058F8` | 11:06 | `/health` fetch 15-24 no abort/timeout (finding D5); status normalisation 36-39; checks parsing 40-51; never throws schema as success (throws E_SCHEMA_INVALID 32) |

## C. Tests (all 8 files read fully, one-by-one)

| # | Path | Bytes | Lines | SHA-256 | Read at | Verified facts |
|---:|---|---:|---:|---|---|---|
| 19 | `web/tests/agentState.test.ts` | 1,819 | 54 | `D2B3849B5F282D8353298AD83C3DB1BB7945B5899A3B60D62DBBC13BA56CF433` | 11:06 | 5 tests; asserts `>=13` not 16; no `interrupted`/`failed`-terminal test |
| 20 | `web/tests/apiClient.test.ts` | 4,998 | 145 | `1BBAB1FE55DEDCA8BFC210098ED60BDD19BD9CE6834BFC347DF982C36D947F75` | 11:06 | 5 tests; hello/ACK/ERROR/malformed/jail; no HTTP timeout test |
| 21 | `web/tests/contract.test.ts` | 4,472 | 137 | `7790F09690295F6FE6693E24B4403D5CB214915949C56AFD90BEE9D10817FDC3` | 11:06 | 8 tests; pins 14/21/codes/validators; no HISTORY graph test |
| 22 | `web/tests/keyboard.test.ts` | 2,409 | 80 | `FC05F6443800BF2C2F192D5319D5F1DB1B2155A43021DC3C24327B517A16A934` | 11:06 | 7 tests; Ctrl+0/Meta/Digit0/clamp/bee/roving |
| 23 | `web/tests/metrics.test.ts` | 1,484 | 47 | `16AAAC10CA47EE6ACE0B5AF9F7EF379DBE0F2CA75CBFAD65230789504210CFB3` | 11:06 | 3 tests; null-vs-derived-vs-server |
| 24 | `web/tests/noHardcoded.test.ts` | 2,228 | 63 | `45D5B407FBA4108F2857D318AD71F78F2B02F9279A4BECA487084101AD553B66` | 11:06 | 4 tests; token isolation/no-demo literals; scans src only |
| 25 | `web/tests/store.test.ts` | 7,331 | 188 | `8945EF17B632EDB3717557406BEA6E0D4F10267576A090E78D8AA3BA38D7C3BC` | 11:06 | 8 tests; ACK-before-mutation, refusals, approvals/nonce, single SETTINGS_STATUS, metadata fallback; no replay/failed/timeout tests |
| 26 | `web/tests/tokens.test.ts` | 2,170 | 57 | `80B68A13EA77F61A5423342DE67C5AE178CA5F3E3D770AACFF16CD579E238646` | 11:06 | 6 tests; 20 swatches/contrast/corrections |

**Test count corroboration:** source `it(` count = **46** across 8 files; `docs/reports/FINAL-REPORT.md:37` records “46 passed / 8 files, typecheck + build exit 0”. No test execution was performed by this seat (read-only; cited log only).

## D. Gate records written by this seat
`plans/seats/SEAT-03/GATE-CY-{1,2,3}.md` — each contains Recon → Plan → Lead decision in the mandated order (no cycle failed; no BLOCKED state).
