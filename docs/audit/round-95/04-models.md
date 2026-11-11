# Round 95 — Model comparison: DeepSeek V4.1 Flash 1017 vs Claude Fable 5.2

**Observed:** 2026-11-10 (America/Chicago). **Scope:** model-capability and price facts, then the HiveMind decision.
**Evidence labels:** `P` = primary vendor page fetched and quoted; `S` = vendor-adjacent/secondary; `U` = unverified for this decision; `ND` = not documented by a primary source.
**No benchmark number is treated as an independent evaluation.** Both vendors publish their own benchmark tables. There is no cross-vendor, same-task, blind evaluation in this repo or in the sources fetched.

## 1. Both names are real and currently documented

| Question | Answer | Evidence |
|---|---|---|
| Does “DeepSeek V4.1 Flash 1017” exist? | **Yes.** API model name `deepseek-flash`; model version `DeepSeek-V4.1-Flash-1017`; release page dated 2026-10-17. | [DeepSeek release note](https://api-docs.deepseek.com/news/news261017) `P`; [Models & Pricing](https://api-docs.deepseek.com/quick_start/pricing/) `P` |
| Does “Claude Fable 5.2” exist? | **Yes.** API id `claude-fable-5-2`; launch dated 2026-11-09. | [Anthropic Fable/Mythos 5.2](https://www.anthropic.com/claude-fable-and-mythos-5-2) `P`; [Claude Platform release notes](https://platform.claude.com/docs/en/release-notes/overview) `P`; [Models overview](https://platform.claude.com/docs/en/models/overview) `P` |

**Correction to the premise:** this is not a comparison against an imaginary model. It is a real, current comparison — but “as good as” is not established by either vendor page.

## 2. Documented specs and prices

| Fact | DeepSeek V4.1 Flash 1017 | Claude Fable 5.2 |
|---|---|---|
| API id | `deepseek-flash` (legacy aliases accepted, served by this model) | `claude-fable-5-2` |
| Context | 1M tokens `P` | 1M tokens `P` |
| Max output | 384K tokens `P` | 128K tokens `P` |
| Tools/JSON | Tool calls, JSON output, Responses API, Anthropic-format API `P` | Tool use; adaptive thinking always on; computer-use toolset `P` |
| Vision | Yes `P` | Text + image input, text output `P` |
| Price / 1M input | $0.15 off-peak, $0.30 peak; cache-hit $0.003/$0.006 `P` | $8.00 `P` |
| Price / 1M output | $0.60 off-peak, $1.20 peak `P` | $40.00 `P` |
| Fast option | — | `speed:"fast"` up to 2.5× output tokens/sec at $16/$80 `P` |
| Concurrency / availability | 2500 concurrency limit `P`; open weights link `P` | API, AWS, Google Cloud, Microsoft Foundry `P` |
| Vendor claim | “stronger benchmark performance, faster task completion, shorter outputs” than prior Flash `P` | “up to 30% faster” than Fable 5.1; vendor-run benchmark table (FrontierCode 2.0 49.1%, FrontierSWE v2 69.9%, OSWorld 2.1 89.3%) `P` |
| Vendor recommendation | Not specified for this decision | Anthropic recommends **Opus 5.5 as the daily driver for most coding/knowledge work**, with Fable for the hard tail `P` |

**Price ratio, list-token only (arithmetic, not measured):**

- input: `$8.00 / $0.15 = 53.3×` cheaper (off-peak); `$8.00 / $0.30 = 26.7×` (peak).
- output: `$40.00 / $0.60 = 66.7×` cheaper (off-peak); `$40.00 / $1.20 = 33.3×` (peak).
- against Fable fast mode: `$16/$80` → `53.3×` input off-peak and `66.7×` output off-peak.

These are **token list-price ratios**, not cost per successful task. Tool calls, retries, rework, cache hit rate, context size, and agent-loop length can change the latter by large factors.

## 3. What cannot be concluded from official pages

| Claim | Status | Why |
|---|---|---|
| “DeepSeek V4.1 Flash 1017 is as good as Claude Fable 5.2 for coding” | **UNVERIFIED** | No independent same-task evaluation; vendor benchmarks are self-reported and not comparable. |
| “It will need less bug fixing” | **UNVERIFIED** | Bug-fix rate is a task/harness property, not a model spec. Requires a pre-registered acceptance-test study. |
| “It is faster for this workload” | **UNVERIFIED** | Vendor latency claims are not cross-vendor wall-clock measurements on the same harness. |
| “Cost per task is 30–100× lower” | **UNVERIFIED** | Only token prices are documented. Per-task cost must be measured as `$ / successful task`, including rework. |
| “It can replace Claude for agentic coding” | **UNVERIFIED as a general claim** | Different harnesses, tool loops, retry behavior and acceptance tests decide this. |

## 4. What HiveMind can actually use today

- `ProviderExecutorAdapter` supports the configured provider types; the repo supports DeepSeek and TokenLab/OpenAI-compatible endpoints (`backend/crates/hiverouter-core/src/provider_executor.rs`, `config.rs`).
- **No Anthropic provider adapter is installed in this tree.** Claude Fable 5.2 is external to the current product; using it would require adding and testing a provider route, key custody, streaming/tool-call handling, refusal mapping, and cost accounting.
- The installed profile currently selects `deepseek` / `deepseek-flash` and has **no key**; every live prompt refuses `E_KEY_MISSING` (verified `/health` 2026-11-10).
- The model itself does not give HiveMind a learning loop or autonomy. HiveMind remains an approval-gated, capped, single-run HMAS. Model capability cannot substitute for missing harness capabilities (no vector memory, no automatic learning, no monitor actions).

## 5. Honest recommendation

1. **Do not promise parity with Fable 5.2.** The vendor pages do not support it, and HiveMind has no same-task eval.
2. **Use `deepseek-flash` as the default worker/planner model** when a key is configured and the work is bounded: token price is dramatically lower, context is 1M, tool calls/JSON are documented.
3. **Do not use it as a hands-off autonomous coder.** HiveMind is not a coding IDE or a continuous agent; it is a one-run hierarchy with approvals and caps.
4. **If Fable is used, use it as a budget-capped advisor/escalation**, not the default — Anthropic itself recommends Opus 5.5 for most daily coding. Adding Anthropic support is a provider-integration project, not a config toggle in this tree.
5. **To settle quality, run a blind same-task trial:** 30–50 real HiveMind tasks, pre-registered acceptance tests, success@1, rework cycles, wall-clock, and `$ / successful task`; require non-inferiority (e.g. success within 5 percentage points) before any “as good” claim.

## 6. Claim register

| ID | Claim | Grade | Evidence |
|---|---|---|---|
| M1 | DeepSeek-V4.1-Flash-1017 exists and is the `deepseek-flash` model | P | DeepSeek release + pricing |
| M2 | Fable 5.2 exists as `claude-fable-5-2` | P | Anthropic announcement + platform docs |
| M3 | DeepSeek 1M context / 384K output, Fable 1M / 128K | P | vendor model pages |
| M4 | DeepSeek token prices are ~26–67× lower than Fable standard list prices | P + arithmetic | pricing pages; ratio computed above |
| M5 | Quality, speed and bug-fix parity | U | no independent same-task study |
| M6 | Fable can be used by HiveMind today | FALSE / ND | no Anthropic adapter in tree |
| M7 | HiveMind learns automatically from model use | FALSE | proposal/lesson gate only; no weight/skill update |

## 7. Sources (fetched 2026-11-10)

- [DeepSeek V4.1 Flash 1017 release](https://api-docs.deepseek.com/news/news261017)
- [DeepSeek models & pricing](https://api-docs.deepseek.com/quick_start/pricing/)
- [Anthropic Claude Fable 5.2 and Mythos 5.2](https://www.anthropic.com/claude-fable-and-mythos-5-2)
- [Claude Platform release notes, 2026-11-09](https://platform.claude.com/docs/en/release-notes/overview)
- [Claude models overview](https://platform.claude.com/docs/en/models/overview)
