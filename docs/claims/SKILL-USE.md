# SKILL USE RECORD — HiveMind v1 execution

Skills were loaded from the session skill catalog at execution start (their instructions were present in the execution
prompt/skill blocks). Substitutions: none at W0. Recorded per EXECUTION-PROMPT.md §2.

| Skill | Loaded by | Applied at | Evidence / output |
|---|---|---|---|
| `forgewright-recon` | Lead | W0, per phase | `docs/plans/PHASE-0-RECON.md`, phase recon files |
| `forgewright-lead` | Lead | whole run | this session plan, phase evidence, final gate |
| `research` (`forgewright-research`) | Lead | toolchain/FigCraft feasibility | HASH-VERIFICATION environment probe; FIGCRAFT-COMMANDREPORT read |
| `claim-register-builder` | Lead | W0, per phase | `docs/claims/CLAIM-REGISTER.csv/.json` |
| `mermaid-skill` | Lead | W0 and final | `docs/diagrams/*.mmd` + exports |
| `capability-auditor` | Lead + verifier | Phase 6/10/11 + final | planned `docs/audit/CAPABILITY-CENSUS.md` |
| `agent-native-design` | Lead | Phase 1 + 11/12 | `docs/contracts/CLI-CONTRACT.md` |
| `craft-agent-operator` | FigCraft A/B | UI workstream | `docs/ui/figcraft/EPISODES/*.json`, ASSETS-INDEX |
| `harness-code-adapter` | Lead + backend | any donor mechanism | `docs/contracts/ADAPTERS.md` |
| `trace` | Lead + verifier | after each UI slice, final | `docs/trace/*` |

Skill catalog availability: forgewright-recon, forgewright-lead, research, claim-register-builder, mermaid-skill,
capability-auditor, agent-native-design, craft-agent-operator, harness-code-adapter, trace — all present.
