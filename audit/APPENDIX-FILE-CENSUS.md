# APPENDIX A — COMPLETE FILE CENSUS (507 tracked files)

Repo: `C:\Users\Riley\Desktop\hiverouter-v1` · HEAD `3988622d6ef65ff39a0c8bb126862a8e8405ffd9`
Manifest: `audit/inventory/FILE-MANIFEST-ENRICHED.csv` · Count: **507**

Every tracked file appears exactly once below. `sha256` is truncated to 16 hex chars for readability;
the full 64-char value is in the manifest. `D` = deep-analysis file (8–12 paragraphs in seat `FILES.md`).

## Seat A — b-core (54 files)

| Path | Bytes | sha256[:16] | Lang | Category | D |
|---|---:|---|---|---|---|
| `backend/crates/hiverouter-contract/Cargo.toml` | 517 | `643E9584B91078F8` | TOML | config-manifest |  |
| `backend/crates/hiverouter-contract/schemas/contract-v1.json` | 48,340 | `F139CDEC67365725` | JSON | config-manifest |  |
| `backend/crates/hiverouter-contract/src/command.rs` | 21,334 | `2EEAFCDD4CE7BEFB` | Rust | source | D |
| `backend/crates/hiverouter-contract/src/envelope.rs` | 6,352 | `A8C647E94B9CBBFD` | Rust | source | D |
| `backend/crates/hiverouter-contract/src/event.rs` | 18,156 | `359B0D4A87BE316A` | Rust | source | D |
| `backend/crates/hiverouter-contract/src/ids.rs` | 3,251 | `2AB28A180DC0225C` | Rust | source |  |
| `backend/crates/hiverouter-contract/src/lib.rs` | 1,845 | `E9C7C87F61048644` | Rust | source | D |
| `backend/crates/hiverouter-contract/src/refusal.rs` | 17,229 | `9C99C4475DF6CBCB` | Rust | source | D |
| `backend/crates/hiverouter-contract/src/schema.rs` | 13,690 | `5734E4B19316C977` | Rust | source | D |
| `backend/crates/hiverouter-contract/src/state.rs` | 4,791 | `DEA156DD9764D00E` | Rust | source | D |
| `backend/crates/hiverouter-contract/src/timestamp.rs` | 11,106 | `43BB8EC910F258A2` | Rust | source |  |
| `backend/crates/hiverouter-contract/src/usage.rs` | 3,698 | `C2A945135482FE2B` | Rust | source |  |
| `backend/crates/hiverouter-contract/tests/common/mod.rs` | 13,211 | `CB6BF9136203369E` | Rust | source |  |
| `backend/crates/hiverouter-contract/tests/golden_schemas.rs` | 3,873 | `E2ED3D34BBB89EF5` | Rust | source |  |
| `backend/crates/hiverouter-contract/tests/schema_roundtrip.rs` | 6,636 | `9BE68275D5737699` | Rust | source |  |
| `backend/crates/hiverouter-contract/tests/ui_command_matrix.rs` | 4,931 | `DFEDBE3BACB9C006` | Rust | source |  |
| `backend/crates/hiverouter-core/Cargo.toml` | 1,020 | `8A0DCCE75A0FEF2C` | TOML | config-manifest |  |
| `backend/crates/hiverouter-core/assets/prompts/manifest.json` | 192 | `487C00282A9B9955` | JSON | config-manifest |  |
| `backend/crates/hiverouter-core/assets/prompts/personas.json` | 214 | `CA16628EBABE80EC` | JSON | config-manifest |  |
| `backend/crates/hiverouter-core/assets/prompts/planner.prompty` | 462 | `15239626AA4AE2A9` | Prompty | source |  |
| `backend/crates/hiverouter-core/build.rs` | 2,097 | `AA5EA8BD665FCA0B` | Rust | source |  |
| `backend/crates/hiverouter-core/src/bus.rs` | 9,135 | `5E8E966806DCC49C` | Rust | source | D |
| `backend/crates/hiverouter-core/src/caps.rs` | 2,918 | `D21061F2FA3C5A53` | Rust | source |  |
| `backend/crates/hiverouter-core/src/checks.rs` | 5,824 | `FFECF02B43CD9B51` | Rust | source |  |
| `backend/crates/hiverouter-core/src/config.rs` | 10,707 | `D4112CC86721A3CF` | Rust | source |  |
| `backend/crates/hiverouter-core/src/context.rs` | 9,310 | `B09D11F3AF246223` | Rust | source | D |
| `backend/crates/hiverouter-core/src/db.rs` | 15,761 | `1822162A2D11E89D` | Rust | source | D |
| `backend/crates/hiverouter-core/src/entities.rs` | 2,735 | `8F6DC673E06F1AB0` | Rust | source |  |
| `backend/crates/hiverouter-core/src/execution.rs` | 20,243 | `A7E0F403AA6D437E` | Rust | source | D |
| `backend/crates/hiverouter-core/src/graph.rs` | 12,281 | `16F3AB2B4C8C8927` | Rust | source | D |
| `backend/crates/hiverouter-core/src/handshake.rs` | 8,430 | `D7647B54EAB33122` | Rust | source | D |
| `backend/crates/hiverouter-core/src/history.rs` | 3,630 | `FC236BCBB0D4DCE3` | Rust | source |  |
| `backend/crates/hiverouter-core/src/http.rs` | 14,376 | `A1E819DD1B800042` | Rust | source | D |
| `backend/crates/hiverouter-core/src/lib.rs` | 1,493 | `30887021C00C7E84` | Rust | source |  |
| `backend/crates/hiverouter-core/src/main.rs` | 6,588 | `3A03F2219BF577B3` | Rust | source | D |
| `backend/crates/hiverouter-core/src/planner.rs` | 6,053 | `4BAD5AAC1CD69284` | Rust | source |  |
| `backend/crates/hiverouter-core/src/prompts.rs` | 11,326 | `FF6A40E5F3C26D8B` | Rust | source | D |
| `backend/crates/hiverouter-core/src/recovery.rs` | 2,787 | `4B3C7098B71143F0` | Rust | source | D |
| `backend/crates/hiverouter-core/src/snapshot.rs` | 10,033 | `4811CC72A354E50C` | Rust | source | D |
| `backend/crates/hiverouter-core/src/static_ui.rs` | 3,795 | `C53A13CAF93B8745` | Rust | source | D |
| `backend/crates/hiverouter-core/src/store.rs` | 15,733 | `CA2BC6DDE0663F6C` | Rust | source |  |
| `backend/crates/hiverouter-core/src/tools_bridge.rs` | 6,788 | `4767E3E083ED7741` | Rust | source | D |
| `backend/crates/hiverouter-core/src/ws.rs` | 19,462 | `D4679001A0D99748` | Rust | source | D |
| `backend/crates/hiverouter-core/tests/chat_store.rs` | 5,209 | `42CEDF2FBBFF7322` | Rust | source |  |
| `backend/crates/hiverouter-core/tests/context_switch.rs` | 2,326 | `284BB4B96503945B` | Rust | source |  |
| `backend/crates/hiverouter-core/tests/graph_authority.rs` | 4,265 | `DAF11FD613C19715` | Rust | source |  |
| `backend/crates/hiverouter-core/tests/handshake.rs` | 2,906 | `954B65E0AAA239D6` | Rust | source |  |
| `backend/crates/hiverouter-core/tests/http_boot.rs` | 8,847 | `7E7D597443DDF98E` | Rust | source |  |
| `backend/crates/hiverouter-core/tests/kill_restore.rs` | 4,803 | `B76470D6BF83023A` | Rust | source |  |
| `backend/crates/hiverouter-core/tests/prompt_assets.rs` | 4,523 | `BDA3C600C1E7CED5` | Rust | source |  |
| `backend/crates/hiverouter-core/tests/run_lifecycle.rs` | 10,846 | `8B2B780FCD2FD56C` | Rust | source |  |
| `backend/crates/hiverouter-core/tests/snapshot_history.rs` | 7,833 | `A348BC8DE1A5DBC5` | Rust | source |  |
| `backend/crates/hiverouter-core/tests/support/mod.rs` | 12,697 | `6457B80F8DBED03B` | Rust | source |  |
| `backend/crates/hiverouter-core/tests/ws_transport.rs` | 19,177 | `2B21D3EFA093AEE0` | Rust | source |  |

## Seat B — b-provider (33 files)

| Path | Bytes | sha256[:16] | Lang | Category | D |
|---|---:|---|---|---|---|
| `backend/Cargo.lock` | 100,124 | `7B704C18E67F56DA` | Lockfile | config-manifest | D |
| `backend/Cargo.toml` | 1,765 | `323A5865A3ED81F1` | TOML | config-manifest | D |
| `backend/crates/hiverouter-provider/Cargo.toml` | 752 | `6F32DD277F25B262` | TOML | config-manifest |  |
| `backend/crates/hiverouter-provider/src/accounting.rs` | 17,056 | `7BDA95F2A98177ED` | Rust | source | D |
| `backend/crates/hiverouter-provider/src/budget.rs` | 23,965 | `0B685A5ED36B97DC` | Rust | source | D |
| `backend/crates/hiverouter-provider/src/client.rs` | 27,461 | `A8B0A5F88C86F294` | Rust | source | D |
| `backend/crates/hiverouter-provider/src/dto.rs` | 24,305 | `A1C0E05878140585` | Rust | source | D |
| `backend/crates/hiverouter-provider/src/error.rs` | 4,483 | `56E7116E9155C315` | Rust | source |  |
| `backend/crates/hiverouter-provider/src/lib.rs` | 2,408 | `83343C8A7FA13A58` | Rust | source |  |
| `backend/crates/hiverouter-provider/src/pricing.rs` | 9,959 | `28E627929595016A` | Rust | source | D |
| `backend/crates/hiverouter-provider/src/secret.rs` | 3,349 | `297E6A7E4A783DD2` | Rust | source |  |
| `backend/crates/hiverouter-provider/tests/refusal_matrix.rs` | 11,475 | `80389102FB8A2CC3` | Rust | source |  |
| `backend/crates/hiverouter-tools/Cargo.toml` | 1,198 | `179D5B83474FB430` | TOML | config-manifest |  |
| `backend/crates/hiverouter-tools/benches/engine_benches.rs` | 3,270 | `2CE34ADF5B6C3B57` | Rust | source |  |
| `backend/crates/hiverouter-tools/src/approval.rs` | 34,944 | `E32411E62FFEC925` | Rust | source | D |
| `backend/crates/hiverouter-tools/src/audit.rs` | 10,695 | `960E793A5369765F` | Rust | source |  |
| `backend/crates/hiverouter-tools/src/bin/hiverouter.rs` | 579 | `6A0FA448CA080530` | Rust | source |  |
| `backend/crates/hiverouter-tools/src/cancel.rs` | 10,411 | `61C76D60BBF75E97` | Rust | source |  |
| `backend/crates/hiverouter-tools/src/census.rs` | 9,205 | `3F67B9D0C66155E7` | Rust | source | D |
| `backend/crates/hiverouter-tools/src/cli.rs` | 37,923 | `A9F583EECAFB1961` | Rust | source | D |
| `backend/crates/hiverouter-tools/src/cli_support.rs` | 6,270 | `54AAF4ABE2872C28` | Rust | source |  |
| `backend/crates/hiverouter-tools/src/commands.rs` | 7,350 | `639E2376892FD462` | Rust | source |  |
| `backend/crates/hiverouter-tools/src/error.rs` | 2,717 | `D987C3E066AA08C6` | Rust | source |  |
| `backend/crates/hiverouter-tools/src/health.rs` | 15,752 | `91AFD4D1FA92A032` | Rust | source | D |
| `backend/crates/hiverouter-tools/src/jail.rs` | 16,815 | `1F6072AD7622971B` | Rust | source | D |
| `backend/crates/hiverouter-tools/src/ledger.rs` | 13,381 | `F5878DA79A93C394` | Rust | source | D |
| `backend/crates/hiverouter-tools/src/lib.rs` | 3,397 | `406B7D19BCD24328` | Rust | source |  |
| `backend/crates/hiverouter-tools/src/settings.rs` | 2,027 | `BE807D709B756401` | Rust | source |  |
| `backend/crates/hiverouter-tools/src/sha256.rs` | 5,379 | `8135BCDC28EA1215` | Rust | source |  |
| `backend/crates/hiverouter-tools/src/shell.rs` | 16,201 | `00E19826DC6970B0` | Rust | source | D |
| `backend/crates/hiverouter-tools/src/tool.rs` | 19,612 | `353AB4E67695C7AA` | Rust | source | D |
| `backend/crates/hiverouter-tools/src/watcher.rs` | 7,947 | `1D894543C1755722` | Rust | source |  |
| `backend/crates/hiverouter-tools/tests/audit_jsonl.rs` | 2,163 | `336D4431A0545AE8` | Rust | source |  |

## Seat C — watchdog-1 (55 files)

| Path | Bytes | sha256[:16] | Lang | Category | D |
|---|---:|---|---|---|---|
| `web/index.html` | 676 | `BC8E7E4D9AFEFAC2` | HTML | source |  |
| `web/package-lock.json` | 78,338 | `2F141BD0E8CBE08A` | JSON | config-manifest |  |
| `web/package.json` | 929 | `A80E90B9F0182EDA` | JSON | config-manifest | D |
| `web/src/App.tsx` | 3,961 | `BC93C009D101C040` | TypeScript/TSX | source | D |
| `web/src/a11y/keyboard.ts` | 2,976 | `72BAE334D9D7A4AA` | TypeScript | source | D |
| `web/src/a11y/reducedMotion.ts` | 1,051 | `45231D4FEB659362` | TypeScript | source |  |
| `web/src/api/client.ts` | 15,930 | `6EFA1C7D1DEE8D04` | TypeScript | source | D |
| `web/src/api/contract.ts` | 28,868 | `4C43F563D0967E37` | TypeScript | source | D |
| `web/src/api/health.ts` | 1,885 | `84963F8C645ED6C0` | TypeScript | source |  |
| `web/src/api/ids.ts` | 1,607 | `9D7D8D755A5EB863` | TypeScript | source |  |
| `web/src/api/index.ts` | 751 | `6D6E3618DC4918CC` | TypeScript | source |  |
| `web/src/api/mock.ts` | 12,437 | `A149A4E36350E0CB` | TypeScript | source |  |
| `web/src/api/refusals.ts` | 1,582 | `DC715B3AE2480144` | TypeScript | source |  |
| `web/src/assets/manifest.ts` | 4,888 | `C0DA46D37948D5C6` | TypeScript | source |  |
| `web/src/chat/ChatCore.tsx` | 6,270 | `E924BB44FA76476B` | TypeScript/TSX | source | D |
| `web/src/flow/BeeEdge.tsx` | 1,686 | `96C4E65A6BB24120` | TypeScript/TSX | source |  |
| `web/src/flow/Canvas.tsx` | 8,823 | `5751824B68466EB5` | TypeScript/TSX | source | D |
| `web/src/flow/graph.ts` | 3,232 | `2357DCB03F698A6B` | TypeScript | source |  |
| `web/src/flow/nodes.tsx` | 6,451 | `5DE1969AE23615BC` | TypeScript/TSX | source | D |
| `web/src/main.tsx` | 404 | `2BC1AA6AAF5A25FD` | TypeScript/TSX | source |  |
| `web/src/panels/AgentsPanel.tsx` | 2,038 | `0A452EE172371E85` | TypeScript/TSX | source |  |
| `web/src/panels/ApprovalOverlay.tsx` | 2,857 | `EB1595B3150B400D` | TypeScript/TSX | source |  |
| `web/src/panels/ControlBar.tsx` | 5,023 | `A22B87B6D259C2FB` | TypeScript/TSX | source |  |
| `web/src/panels/DataPanel.tsx` | 3,077 | `58A77EDED98E0677` | TypeScript/TSX | source | D |
| `web/src/panels/DevGallery.tsx` | 1,044 | `5D6645467E54865F` | TypeScript/TSX | source |  |
| `web/src/panels/DiffDecisionModal.tsx` | 1,265 | `6C2DDD6BFD6F800C` | TypeScript/TSX | source |  |
| `web/src/panels/Header.tsx` | 3,268 | `8285642357B7B8F6` | TypeScript/TSX | source |  |
| `web/src/panels/LibraryPanel.tsx` | 2,994 | `85B059387AC942AD` | TypeScript/TSX | source |  |
| `web/src/panels/ModelsPanel.tsx` | 1,195 | `231A22A7228E632C` | TypeScript/TSX | source |  |
| `web/src/panels/NavRail.tsx` | 2,251 | `DC54427F99FC0097` | TypeScript/TSX | source |  |
| `web/src/panels/RoutingPanel.tsx` | 1,888 | `F0269705A18A93B5` | TypeScript/TSX | source |  |
| `web/src/panels/Scrubber.tsx` | 1,782 | `1D491793C3F81799` | TypeScript/TSX | source |  |
| `web/src/panels/SettingsPanel.tsx` | 5,186 | `DFBF40970F72B05C` | TypeScript/TSX | source | D |
| `web/src/panels/StatePack.tsx` | 1,816 | `ED3FBDC6EBE78C27` | TypeScript/TSX | source |  |
| `web/src/panels/ToastRegion.tsx` | 2,072 | `403D24C553BF9F07` | TypeScript/TSX | source |  |
| `web/src/state/agentState.ts` | 4,509 | `88CFB66BBAAE9F07` | TypeScript | source | D |
| `web/src/state/context.ts` | 811 | `2CE91B7243F15660` | TypeScript | source |  |
| `web/src/state/metrics.ts` | 1,710 | `15FC2EB1729032A8` | TypeScript | source |  |
| `web/src/state/store.ts` | 23,407 | `6289459A98C9BD6E` | TypeScript | source | D |
| `web/src/state/strings.ts` | 12,692 | `D6C9E9420D9280D1` | TypeScript | source | D |
| `web/src/state/useAppReducedMotion.ts` | 388 | `CB9A8F53F90D2BA1` | TypeScript | source |  |
| `web/src/styles/index.css` | 3,601 | `E3925C3E6BFEA2A5` | CSS | source |  |
| `web/src/styles/tokens.css` | 2,858 | `43A0B8E4C07E5DB7` | CSS | source |  |
| `web/src/tokens/contrast.ts` | 1,683 | `C3193DB4403CC32A` | TypeScript | source | D |
| `web/src/tokens/tokens.ts` | 7,070 | `078354A37D8AC754` | TypeScript | source | D |
| `web/tests/agentState.test.ts` | 1,819 | `D2B3849B5F282D83` | TypeScript | source |  |
| `web/tests/apiClient.test.ts` | 4,998 | `1BBAB1FE55DEDCA8` | TypeScript | source |  |
| `web/tests/contract.test.ts` | 4,472 | `7790F09690295F6F` | TypeScript | source |  |
| `web/tests/keyboard.test.ts` | 2,409 | `FC05F6443800BF2C` | TypeScript | source |  |
| `web/tests/metrics.test.ts` | 1,484 | `16AAAC10CA47EE6A` | TypeScript | source |  |
| `web/tests/noHardcoded.test.ts` | 2,228 | `45D5B407FBA4108F` | TypeScript | source |  |
| `web/tests/store.test.ts` | 7,331 | `8945EF17B632EDB3` | TypeScript | source |  |
| `web/tests/tokens.test.ts` | 2,170 | `80B68A13EA77F61A` | TypeScript | source |  |
| `web/tsconfig.json` | 618 | `A50AC07820F94752` | JSON | config-manifest |  |
| `web/vite.config.ts` | 798 | `4C0BF654D4C48B24` | TypeScript | source | D |

## Seat D — watchdog-2 (122 files)

| Path | Bytes | sha256[:16] | Lang | Category | D |
|---|---:|---|---|---|---|
| `e2e/.gitignore` | 125 | `EC621CAF388828C9` | unknown | other |  |
| `e2e/README.md` | 4,722 | `566522894E805A08` | Markdown | other |  |
| `e2e/data-probe.mjs` | 1,022 | `E0E5A045390F6D6D` | JavaScript | source |  |
| `e2e/data/control-matrix.json` | 14,979 | `D42E3342DC3175B8` | JSON | config-manifest |  |
| `e2e/helpers/app.ts` | 2,044 | `CB9AE9F946471B6F` | TypeScript | source |  |
| `e2e/helpers/contrast.ts` | 2,214 | `646ED79DF15D2A30` | TypeScript | source |  |
| `e2e/helpers/evidence.ts` | 2,921 | `066430F0A36E9F9A` | TypeScript | source |  |
| `e2e/helpers/matrix.ts` | 2,004 | `BC101B3505E1F98B` | TypeScript | source |  |
| `e2e/helpers/trace.ts` | 1,299 | `404CE7202A8C8A55` | TypeScript | source |  |
| `e2e/helpers/ws-tap.ts` | 1,790 | `4E667F62D942B736` | TypeScript | source |  |
| `e2e/live-check2.mjs` | 712 | `042EACC4F2919579` | JavaScript | source |  |
| `e2e/live-inspect.mjs` | 6,568 | `DDD46289FF4EB760` | JavaScript | source |  |
| `e2e/live-probe.mjs` | 1,019 | `6C90EE5C0B7FDA7C` | JavaScript | source |  |
| `e2e/live-shot.mjs` | 529 | `349E459DBFB88EC7` | JavaScript | source |  |
| `e2e/live-smoke.mjs` | 1,064 | `50E507618D831B6C` | JavaScript | source |  |
| `e2e/package-lock.json` | 1,748 | `883C7A729F40D01B` | JSON | config-manifest |  |
| `e2e/package.json` | 659 | `F5D3D35E1C3F4FCC` | JSON | config-manifest |  |
| `e2e/playwright.config.ts` | 1,917 | `E414A391049EB04C` | TypeScript | source | D |
| `e2e/specs/a11y-gates.spec.ts` | 2,769 | `9F5DE694757DCF04` | TypeScript | source |  |
| `e2e/specs/control-matrix.spec.ts` | 3,755 | `E4B16D3E05BD57AE` | TypeScript | source |  |
| `e2e/specs/live-app.spec.ts` | 19,833 | `CB3A5E8D70EF35F9` | TypeScript | source | D |
| `e2e/specs/trace-hops.spec.ts` | 17,640 | `03B8B6A4F1122F49` | TypeScript | source | D |
| `e2e/specs/ws-contract.spec.ts` | 2,188 | `025EB2901FF4DD49` | TypeScript | source |  |
| `e2e/tsconfig.json` | 228 | `6D92AE9D89B3F28E` | JSON | config-manifest |  |
| `runtime-test/data/hiverouter.db` | 4,096 | `89A2318C76452BE8` | SQLite | runtime-artifact |  |
| `runtime-test/data/hiverouter.db-shm` | 32,768 | `F6FCFFAF91E43D08` | SQLite-SHM | runtime-artifact |  |
| `runtime-test/data/hiverouter.db-wal` | 94,792 | `90A60CD79BA2DA4C` | SQLite-WAL | runtime-artifact |  |
| `runtime-test/data2/hiverouter.db` | 4,096 | `89A2318C76452BE8` | SQLite | runtime-artifact |  |
| `runtime-test/data2/hiverouter.db-shm` | 32,768 | `B86EDA07432C5F92` | SQLite-SHM | runtime-artifact |  |
| `runtime-test/data2/hiverouter.db-wal` | 94,792 | `66B34BF73C5CB379` | SQLite-WAL | runtime-artifact |  |
| `runtime-test/data3/hiverouter.db` | 4,096 | `89A2318C76452BE8` | SQLite | runtime-artifact |  |
| `runtime-test/data3/hiverouter.db-shm` | 32,768 | `AC5ED03284D6ADB4` | SQLite-SHM | runtime-artifact |  |
| `runtime-test/data3/hiverouter.db-wal` | 94,792 | `DFDBC19177BD835C` | SQLite-WAL | runtime-artifact |  |
| `runtime-test/data4/hiverouter.db` | 4,096 | `89A2318C76452BE8` | SQLite | runtime-artifact |  |
| `runtime-test/data4/hiverouter.db-shm` | 32,768 | `A7AD9AA038535D86` | SQLite-SHM | runtime-artifact |  |
| `runtime-test/data4/hiverouter.db-wal` | 152,472 | `A258EBD0FF9F7F1D` | SQLite-WAL | runtime-artifact |  |
| `runtime-test/data4/snapshots/01a1b119-2f34-7228-974e-03b1c27856fc/0000000000.json` | 271 | `B3384389368E4136` | JSON | runtime-artifact |  |
| `runtime-test/data5/hiverouter.db` | 4,096 | `89A2318C76452BE8` | SQLite | runtime-artifact |  |
| `runtime-test/data5/hiverouter.db-shm` | 32,768 | `48EDB60FE77A7613` | SQLite-SHM | runtime-artifact |  |
| `runtime-test/data5/hiverouter.db-wal` | 94,792 | `6CB19A0BB68C37C6` | SQLite-WAL | runtime-artifact |  |
| `runtime-test/data6/hiverouter.db` | 4,096 | `89A2318C76452BE8` | SQLite | runtime-artifact |  |
| `runtime-test/data6/hiverouter.db-shm` | 32,768 | `41607360C495452B` | SQLite-SHM | runtime-artifact |  |
| `runtime-test/data6/hiverouter.db-wal` | 152,472 | `FCF13BB6D6C7F607` | SQLite-WAL | runtime-artifact |  |
| `runtime-test/data6/snapshots/01a1b11d-90f3-73b3-bca9-616a21cd2e54/0000000000.json` | 271 | `208D3889264A830E` | JSON | runtime-artifact |  |
| `runtime-test/data7/hiverouter.db` | 4,096 | `89A2318C76452BE8` | SQLite | runtime-artifact |  |
| `runtime-test/data7/hiverouter.db-shm` | 32,768 | `6202D05B99FCCF25` | SQLite-SHM | runtime-artifact |  |
| `runtime-test/data7/hiverouter.db-wal` | 127,752 | `DBFA36BC2423DD8C` | SQLite-WAL | runtime-artifact |  |
| `runtime-test/data8/hiverouter.db` | 4,096 | `89A2318C76452BE8` | SQLite | runtime-artifact |  |
| `runtime-test/data8/hiverouter.db-shm` | 32,768 | `94D110727614C178` | SQLite-SHM | runtime-artifact |  |
| `runtime-test/data8/hiverouter.db-wal` | 127,752 | `705014ADDA043F55` | SQLite-WAL | runtime-artifact |  |
| `runtime-test/data9/hiverouter.db` | 4,096 | `89A2318C76452BE8` | SQLite | runtime-artifact |  |
| `runtime-test/data9/hiverouter.db-shm` | 32,768 | `E8620D36D8403129` | SQLite-SHM | runtime-artifact |  |
| `runtime-test/data9/hiverouter.db-wal` | 185,432 | `655543F8FBC42F10` | SQLite-WAL | runtime-artifact |  |
| `runtime-test/data9/snapshots/01a1b16d-7530-75e4-b51e-83b6bc3fdcca/0000000000.json` | 271 | `BD2270F596523038` | JSON | runtime-artifact |  |
| `runtime-test/pkg-cli-final/CHECKSUMS.txt` | 712 | `842E44C7439268B5` | Text | runtime-artifact |  |
| `runtime-test/pkg-cli-final/README.md` | 1,276 | `CFA567329EDC9E6C` | Markdown | runtime-artifact |  |
| `runtime-test/pkg-cli-final/RUNBOOK.md` | 4,396 | `E7FBA10F3332CA65` | Markdown | runtime-artifact |  |
| `runtime-test/pkg-cli-final/data/hiverouter.db` | 4,096 | `89A2318C76452BE8` | SQLite | runtime-artifact |  |
| `runtime-test/pkg-cli-final/data/hiverouter.db-shm` | 32,768 | `B0C2485EC9EC9C17` | SQLite-SHM | runtime-artifact |  |
| `runtime-test/pkg-cli-final/data/hiverouter.db-wal` | 127,752 | `779BAE5FE7BF0A34` | SQLite-WAL | runtime-artifact |  |
| `runtime-test/pkg-cli-final/hiverouter-cli.exe` | 5,364,224 | `51E1ABA6DC7349BF` | PE-binary | runtime-artifact |  |
| `runtime-test/pkg-cli-final/hiverouter.exe` | 9,329,152 | `184D51A7C3CCA7B9` | PE-binary | runtime-artifact |  |
| `runtime-test/pkg-final/CHECKSUMS.txt` | 477 | `9193A5B4F365B294` | Text | runtime-artifact |  |
| `runtime-test/pkg-final/README.md` | 1,276 | `CFA567329EDC9E6C` | Markdown | runtime-artifact |  |
| `runtime-test/pkg-final/RUNBOOK.md` | 4,396 | `E7FBA10F3332CA65` | Markdown | runtime-artifact |  |
| `runtime-test/pkg-final/data/hiverouter.db` | 4,096 | `89A2318C76452BE8` | SQLite | runtime-artifact |  |
| `runtime-test/pkg-final/data/hiverouter.db-shm` | 32,768 | `400EBFD736D79859` | SQLite-SHM | runtime-artifact |  |
| `runtime-test/pkg-final/data/hiverouter.db-wal` | 127,752 | `3622E5BBF27033FD` | SQLite-WAL | runtime-artifact |  |
| `runtime-test/pkg-final/err.txt` | 0 | `E3B0C44298FC1C14` | Text | runtime-artifact |  |
| `runtime-test/pkg-final/hiverouter.exe` | 9,329,152 | `184D51A7C3CCA7B9` | PE-binary | runtime-artifact |  |
| `runtime-test/pkg-final/out.txt` | 571 | `8BF54E0F7B765DA4` | Text | runtime-artifact |  |
| `runtime-test/pkg-final/smoke-data/hiverouter.db` | 4,096 | `89A2318C76452BE8` | SQLite | runtime-artifact |  |
| `runtime-test/pkg-final/smoke-data/hiverouter.db-shm` | 32,768 | `114C6549E7E78B0B` | SQLite-SHM | runtime-artifact |  |
| `runtime-test/pkg-final/smoke-data/hiverouter.db-wal` | 127,752 | `4C9089758F4F0F34` | SQLite-WAL | runtime-artifact |  |
| `runtime-test/pkg-final/smoke.err` | 0 | `E3B0C44298FC1C14` | Log | runtime-artifact |  |
| `runtime-test/pkg-final/smoke.out` | 583 | `6208165E1BB6BFD6` | Log | runtime-artifact |  |
| `runtime-test/pkg-smoke/CHECKSUMS.txt` | 451 | `C95EF561BF047DBD` | Text | runtime-artifact |  |
| `runtime-test/pkg-smoke/README.md` | 1,276 | `CFA567329EDC9E6C` | Markdown | runtime-artifact |  |
| `runtime-test/pkg-smoke/RUNBOOK.md` | 4,396 | `E7FBA10F3332CA65` | Markdown | runtime-artifact |  |
| `runtime-test/pkg-smoke/data/hiverouter.db` | 4,096 | `89A2318C76452BE8` | SQLite | runtime-artifact |  |
| `runtime-test/pkg-smoke/data/hiverouter.db-shm` | 32,768 | `B58903D1BFC4664F` | SQLite-SHM | runtime-artifact |  |
| `runtime-test/pkg-smoke/data/hiverouter.db-wal` | 94,792 | `277C9B5895EBBB6C` | SQLite-WAL | runtime-artifact |  |
| `runtime-test/pkg-smoke/err.txt` | 0 | `E3B0C44298FC1C14` | Text | runtime-artifact |  |
| `runtime-test/pkg-smoke/hiverouter.exe` | 9,099,776 | `71B229B587A909A6` | PE-binary | runtime-artifact |  |
| `runtime-test/pkg-smoke/out.txt` | 571 | `3EF24FC0C9F6D7DF` | Text | runtime-artifact |  |
| `runtime-test/root.html` | 46 | `71BC055475BEE0BE` | HTML | runtime-artifact |  |
| `runtime-test/s5.err` | 0 | `E3B0C44298FC1C14` | Log | runtime-artifact |  |
| `runtime-test/s5.out` | 553 | `CB540A4A3A9CD322` | Log | runtime-artifact |  |
| `runtime-test/s6.err` | 0 | `E3B0C44298FC1C14` | Log | runtime-artifact |  |
| `runtime-test/s6.out` | 4,836 | `EFAA576CC176079C` | Log | runtime-artifact |  |
| `runtime-test/s7.err` | 0 | `E3B0C44298FC1C14` | Log | runtime-artifact |  |
| `runtime-test/s7.out` | 553 | `F5FA99B678B606C6` | Log | runtime-artifact |  |
| `runtime-test/s8.err` | 0 | `E3B0C44298FC1C14` | Log | runtime-artifact |  |
| `runtime-test/s8.out` | 553 | `40442AAFD3EA9E42` | Log | runtime-artifact |  |
| `runtime-test/s9.err` | 0 | `E3B0C44298FC1C14` | Log | runtime-artifact |  |
| `runtime-test/s9.out` | 553 | `7F7FA6E1C65C12B9` | Log | runtime-artifact |  |
| `runtime-test/server.err.txt` | 0 | `E3B0C44298FC1C14` | Text | runtime-artifact |  |
| `runtime-test/server.out.txt` | 551 | `11B42121DD424B01` | Text | runtime-artifact |  |
| `runtime-test/server2.err.txt` | 0 | `E3B0C44298FC1C14` | Text | runtime-artifact |  |
| `runtime-test/server2.out.txt` | 553 | `838F1917D697F9EB` | Text | runtime-artifact |  |
| `runtime-test/server3.err.txt` | 0 | `E3B0C44298FC1C14` | Text | runtime-artifact |  |
| `runtime-test/server3.out.txt` | 553 | `6B953D16D09BC692` | Text | runtime-artifact |  |
| `runtime-test/server4.err.txt` | 0 | `E3B0C44298FC1C14` | Text | runtime-artifact |  |
| `runtime-test/server4.out.txt` | 1,230 | `8C42F12E13BD62C2` | Text | runtime-artifact |  |
| `runtime-test/workspace/README.md` | 44 | `B0BD9D5596577EF6` | Markdown | runtime-artifact |  |
| `runtime-test/workspace/note.txt` | 20 | `987F6FD43C751A63` | Text | runtime-artifact |  |
| `tools/bin/hiverouter-cli.exe` | 5,364,224 | `51E1ABA6DC7349BF` | PE-binary | generated-artifact |  |
| `tools/bin/hiverouter.exe` | 9,329,152 | `184D51A7C3CCA7B9` | PE-binary | generated-artifact |  |
| `tools/build-release.ps1` | 2,051 | `2B465B0D05F1C7B3` | PowerShell | source |  |
| `tools/build_assets_index.py` | 3,349 | `34A725D984F4A39A` | Python | source |  |
| `tools/cc-test/r.exe` | 5,088,156 | `DB801A07B6FD5194` | PE-binary | generated-artifact |  |
| `tools/cc-test/r.rs` | 47 | `0D2CAAE627FD973C` | Rust | source |  |
| `tools/cc-test/t.c` | 70 | `29838A3AAB694B42` | C | source |  |
| `tools/cc-test/t.exe` | 49,876 | `2840568E7B7D4C22` | PE-binary | generated-artifact |  |
| `tools/qa/README.md` | 1,200 | `F8F253AAE35FE502` | Markdown | other |  |
| `tools/qa/apply-live-trace.mjs` | 2,483 | `D5E49DA85ECF2292` | JavaScript | source |  |
| `tools/qa/build-trace-seeds.mjs` | 3,608 | `E1C67D22AD002CA2` | JavaScript | source |  |
| `tools/qa/check-env.ps1` | 3,747 | `469514E2777207D1` | PowerShell | source |  |
| `tools/qa/figcraft-assets.mjs` | 3,556 | `48FFA47248259DA4` | JavaScript | source |  |
| `tools/qa/verify-harness.mjs` | 8,167 | `08B84C782F0E7673` | JavaScript | source | D |
| `tools/runbook.md` | 4,396 | `E7FBA10F3332CA65` | Markdown | other |  |
| `tools/test.ps1` | 2,531 | `210AD6393AFD7432` | PowerShell | source |  |

## Seat E — figcraft-a (103 files)

| Path | Bytes | sha256[:16] | Lang | Category | D |
|---|---:|---|---|---|---|
| `docs/ui/figcraft/A/A-ASSETS.json` | 18,534 | `88832272CB6C81DD` | JSON | documentation |  |
| `docs/ui/figcraft/A/A-EPISODES.json` | 3,895 | `BD7593F710483C82` | JSON | documentation |  |
| `docs/ui/figcraft/A/A-SELF-CHECK.json` | 11,246 | `063066BF47E14579` | JSON | documentation |  |
| `docs/ui/figcraft/A/build/build-state.json` | 103,383 | `7874F3459D50AEA7` | JSON | documentation |  |
| `docs/ui/figcraft/A/build/c4.png` | 86,238 | `956DD5BAFFDE6849` | PNG | documentation-asset |  |
| `docs/ui/figcraft/A/build/fig.py` | 3,380 | `7793BF9656FA40DA` | Python | documentation |  |
| `docs/ui/figcraft/A/build/gen_build.py` | 10,259 | `156CC41465354353` | Python | documentation |  |
| `docs/ui/figcraft/A/build/gen_common.py` | 21,202 | `697CB7B80527C027` | Python | documentation |  |
| `docs/ui/figcraft/A/build/gen_manifest.py` | 17,760 | `61D800CF7E0A4929` | Python | documentation |  |
| `docs/ui/figcraft/A/build/page00.figcraft` | 78,099 | `1AEBF14DF36AAB0D` | FigCraft | documentation-asset |  |
| `docs/ui/figcraft/A/build/page00g.figcraft` | 78,540 | `2E17E137D3219120` | FigCraft | documentation-asset |  |
| `docs/ui/figcraft/A/build/page01.figcraft` | 88,448 | `8FA909A270925F44` | FigCraft | documentation-asset |  |
| `docs/ui/figcraft/A/build/page01g.figcraft` | 88,784 | `3B1FAF4B0E788CF3` | FigCraft | documentation-asset |  |
| `docs/ui/figcraft/A/build/page02.figcraft` | 115,696 | `5A23DFA3B9D9DB04` | FigCraft | documentation-asset |  |
| `docs/ui/figcraft/A/build/page02g.figcraft` | 116,787 | `08E11CDB797F1845` | FigCraft | documentation-asset |  |
| `docs/ui/figcraft/A/build/page03.figcraft` | 126,837 | `F22935BB1D259DF7` | FigCraft | documentation-asset |  |
| `docs/ui/figcraft/A/build/page03g.figcraft` | 127,191 | `3C5531ABA40ABD2C` | FigCraft | documentation-asset |  |
| `docs/ui/figcraft/A/build/pages_brand.py` | 16,379 | `6A0DBFC4C77E98D6` | Python | documentation |  |
| `docs/ui/figcraft/A/build/pages_components.py` | 43,816 | `BFE7DED25A8CBEC9` | Python | documentation |  |
| `docs/ui/figcraft/A/build/pages_foundations.py` | 13,873 | `AA365521DD750549` | Python | documentation |  |
| `docs/ui/figcraft/A/build/pages_specimens.py` | 18,950 | `3FEDB4165D41CEDF` | Python | documentation |  |
| `docs/ui/figcraft/A/build/probe-s04c.png` | 23,088 | `28EB4E30D1CDFB5A` | PNG | documentation-asset |  |
| `docs/ui/figcraft/A/build/stage0.figcraft` | 1,086 | `4E7F14ECD87D467E` | FigCraft | documentation-asset |  |
| `docs/ui/figcraft/A/build/stage1.figcraft` | 4,967 | `475466F7CF702EBE` | FigCraft | documentation-asset |  |
| `docs/ui/figcraft/A/build/styles.figcraft` | 80,405 | `3132F03047E7F204` | FigCraft | documentation-asset |  |
| `docs/ui/figcraft/A/scripts/00_setup_pages.json` | 3,681 | `622B84004114282A` | JSON | documentation |  |
| `docs/ui/figcraft/A/scripts/01_frames.json` | 1,473 | `2CD4F67CD7D82D9A` | JSON | documentation |  |
| `docs/ui/figcraft/A/scripts/02_foundations.json` | 76,384 | `6886D2EE933F861C` | JSON | documentation |  |
| `docs/ui/figcraft/A/scripts/03_foundations_groups.json` | 2,879 | `9D9D0034A550CC0E` | JSON | documentation |  |
| `docs/ui/figcraft/A/scripts/04_styles.json` | 4,911 | `3D1E3EDC382832ED` | JSON | documentation |  |
| `docs/ui/figcraft/A/scripts/05_brand.json` | 49,111 | `DDAE3EC1779C62E9` | JSON | documentation |  |
| `docs/ui/figcraft/A/scripts/06_brand_groups.json` | 1,867 | `7FC718D847F507C4` | JSON | documentation |  |
| `docs/ui/figcraft/A/scripts/07_components.json` | 200,344 | `1401820BF674C6CC` | JSON | documentation |  |
| `docs/ui/figcraft/A/scripts/08_components_groups.json` | 7,306 | `40CFB4D040DBD8FA` | JSON | documentation |  |
| `docs/ui/figcraft/A/scripts/09_specimens.json` | 68,015 | `5B968CDA478FAF53` | JSON | documentation |  |
| `docs/ui/figcraft/A/scripts/10_specimens_groups.json` | 3,539 | `0FA384767F117478` | JSON | documentation |  |
| `docs/ui/figcraft/ASSETS-INDEX.json` | 35,998 | `C5763BFA74A82C1D` | JSON | documentation |  |
| `docs/ui/figcraft/B/B-ASSETS.json` | 26,809 | `A8A81E74BFBFBB3B` | JSON | documentation |  |
| `docs/ui/figcraft/B/build/__pycache__/board_default.cpython-314.pyc` | 29,494 | `2F6955F9A0A59371` | Python-bytecode | documentation |  |
| `docs/ui/figcraft/B/build/__pycache__/board_hero.cpython-314.pyc` | 15,308 | `3E36B8B717F04FB1` | Python-bytecode | documentation |  |
| `docs/ui/figcraft/B/build/__pycache__/board_interaction.cpython-314.pyc` | 25,009 | `90B8E94E740D88EA` | Python-bytecode | documentation |  |
| `docs/ui/figcraft/B/build/__pycache__/board_zoom.cpython-314.pyc` | 16,839 | `A8BFF67F3F04AB58` | Python-bytecode | documentation |  |
| `docs/ui/figcraft/B/build/__pycache__/boards_common.cpython-314.pyc` | 26,082 | `BA2199207293EAF6` | Python-bytecode | documentation |  |
| `docs/ui/figcraft/B/build/__pycache__/fig_b.cpython-314.pyc` | 4,996 | `3E2FD13EF12BE71C` | Python-bytecode | documentation |  |
| `docs/ui/figcraft/B/build/_run.json` | 17,355 | `55FAD1FBD94AF2CF` | JSON | documentation |  |
| `docs/ui/figcraft/B/build/board_default.py` | 22,086 | `1C30304D871CF883` | Python | documentation |  |
| `docs/ui/figcraft/B/build/board_hero.py` | 11,816 | `8B1A2393C1CE3462` | Python | documentation |  |
| `docs/ui/figcraft/B/build/board_interaction.py` | 19,172 | `B371F88C0FB81680` | Python | documentation |  |
| `docs/ui/figcraft/B/build/board_zoom.py` | 11,922 | `6BB6352EC2BD21E1` | Python | documentation |  |
| `docs/ui/figcraft/B/build/boards_common.py` | 19,072 | `1AD83890997DF604` | Python | documentation |  |
| `docs/ui/figcraft/B/build/build-state.json` | 29,301 | `44E3EF35F9A03932` | JSON | documentation |  |
| `docs/ui/figcraft/B/build/build_b.py` | 9,962 | `DF43373E99874076` | Python | documentation |  |
| `docs/ui/figcraft/B/build/fig_b.py` | 2,776 | `0E0BF626F49FE0EC` | Python | documentation |  |
| `docs/ui/figcraft/B/build/info-B.json` | 1,468 | `A6C32597C3BF666D` | JSON | documentation |  |
| `docs/ui/figcraft/B/build/make_manifest.py` | 14,329 | `46AC706599D47302` | Python | documentation |  |
| `docs/ui/figcraft/B/build/patch3.py` | 15,487 | `C94F86CFBDE18B0B` | Python | documentation |  |
| `docs/ui/figcraft/B/build/patch4.py` | 8,394 | `625EE45B7E2EAC9A` | Python | documentation |  |
| `docs/ui/figcraft/B/build/patch5.py` | 2,467 | `1B6CB7C71FC3DF3C` | Python | documentation |  |
| `docs/ui/figcraft/B/build/patch_default.py` | 7,268 | `A5CC8AF47EF7C9B5` | Python | documentation |  |
| `docs/ui/figcraft/B/build/patch_default2.py` | 3,177 | `BBB639AC2891DF0B` | Python | documentation |  |
| `docs/ui/figcraft/B/build/patch_manifest.py` | 832 | `85BC4269A30C0041` | Python | documentation |  |
| `docs/ui/figcraft/B/build/patch_manifest2.py` | 1,387 | `9898A90C0F842073` | Python | documentation |  |
| `docs/ui/figcraft/B/build/patch_verify.py` | 2,801 | `35866B160EBC1A32` | Python | documentation |  |
| `docs/ui/figcraft/B/build/tmp/board00.figcraft` | 96,529 | `CB67EF889CE96AB7` | FigCraft | documentation-asset |  |
| `docs/ui/figcraft/B/build/tmp/board01.figcraft` | 154,725 | `30B82B71DFCB74F9` | FigCraft | documentation-asset |  |
| `docs/ui/figcraft/B/build/tmp/board02.figcraft` | 184,409 | `E15F74A99F70E651` | FigCraft | documentation-asset |  |
| `docs/ui/figcraft/B/build/tmp/board03.figcraft` | 220,688 | `43CB51073086B629` | FigCraft | documentation-asset |  |
| `docs/ui/figcraft/B/build/tmp/effects.figcraft` | 226,734 | `DAE6068926E30C99` | FigCraft | documentation-asset |  |
| `docs/ui/figcraft/B/build/tmp/step0.figcraft` | 479 | `E1A337C46B6F9579` | FigCraft | documentation-asset |  |
| `docs/ui/figcraft/B/build/tmp/step1.figcraft` | 3,500 | `55F239BC554356C5` | FigCraft | documentation-asset |  |
| `docs/ui/figcraft/B/build/verify_b.py` | 8,954 | `F1CA91F75590597A` | Python | documentation |  |
| `docs/ui/figcraft/B/scripts/00_setup_pages.json` | 483 | `B4CB27577B250AEE` | JSON | documentation |  |
| `docs/ui/figcraft/B/scripts/01_frames.json` | 1,471 | `12FC72E51658F428` | JSON | documentation |  |
| `docs/ui/figcraft/B/scripts/02_canvas_default.json` | 98,178 | `B7E3B8506254CCDE` | JSON | documentation |  |
| `docs/ui/figcraft/B/scripts/03_zoom_states.json` | 104,367 | `72BA962F41293411` | JSON | documentation |  |
| `docs/ui/figcraft/B/scripts/04_interaction.json` | 83,317 | `12279E3A5F9A24A6` | JSON | documentation |  |
| `docs/ui/figcraft/B/scripts/05_hero.json` | 74,872 | `912F15B627B251B7` | JSON | documentation |  |
| `docs/ui/figcraft/B/scripts/06_effects.json` | 17,356 | `68FD956DDC8E23D6` | JSON | documentation |  |
| `docs/ui/figcraft/BRIEF-A.md` | 5,806 | `F8EBA08A45485465` | Markdown | documentation |  |
| `docs/ui/figcraft/BRIEF-B.md` | 3,225 | `1028F4548476E3F1` | Markdown | documentation |  |
| `docs/ui/figcraft/EPISODES/a-a-build-00-00_setup_pages.json` | 3,011 | `A9074BAC5EDE735B` | JSON | documentation |  |
| `docs/ui/figcraft/EPISODES/a-a-build-01-01_frames.json` | 1,702 | `9BC9EA665C31F958` | JSON | documentation |  |
| `docs/ui/figcraft/EPISODES/a-a-build-02-02_foundations.json` | 1,704 | `D3BD839378EE6C55` | JSON | documentation |  |
| `docs/ui/figcraft/EPISODES/a-a-build-03-03_foundations_groups.json` | 1,694 | `7212B7DDD0CF7CF9` | JSON | documentation |  |
| `docs/ui/figcraft/EPISODES/a-a-build-04-04_styles.json` | 1,679 | `1CAC392CAFB69A4A` | JSON | documentation |  |
| `docs/ui/figcraft/EPISODES/a-a-build-05-05_brand.json` | 1,720 | `62E14A06C79E9E3D` | JSON | documentation |  |
| `docs/ui/figcraft/EPISODES/a-a-build-06-06_brand_groups.json` | 1,668 | `C63AA5DC342790A6` | JSON | documentation |  |
| `docs/ui/figcraft/EPISODES/a-a-build-07-07_components.json` | 1,741 | `823CB7DED9BF9E81` | JSON | documentation |  |
| `docs/ui/figcraft/EPISODES/a-a-build-08-08_components_groups.json` | 1,689 | `917168E3D7FAD732` | JSON | documentation |  |
| `docs/ui/figcraft/EPISODES/a-a-build-09-09_specimens.json` | 1,737 | `2F0D52054E04BE6F` | JSON | documentation |  |
| `docs/ui/figcraft/EPISODES/a-a-build-10-10_specimens_groups.json` | 1,685 | `2799C4B396BC169E` | JSON | documentation |  |
| `docs/ui/figcraft/EPISODES/a-a-exports.json` | 14,280 | `958BCE12BFC978EA` | JSON | documentation |  |
| `docs/ui/figcraft/EPISODES/b-00_setup_pages.json` | 1,559 | `3E4901E24386D9D2` | JSON | documentation |  |
| `docs/ui/figcraft/EPISODES/b-01_frames.json` | 1,658 | `D23F056CE12BBCF4` | JSON | documentation |  |
| `docs/ui/figcraft/EPISODES/b-02_canvas_default.json` | 1,681 | `9FB241E8A755345D` | JSON | documentation |  |
| `docs/ui/figcraft/EPISODES/b-03_zoom_states.json` | 1,638 | `9FE70D6AE9414785` | JSON | documentation |  |
| `docs/ui/figcraft/EPISODES/b-04_interaction.json` | 1,688 | `BB8B9B393232EB44` | JSON | documentation |  |
| `docs/ui/figcraft/EPISODES/b-05_hero.json` | 1,636 | `2BF94673AC84148B` | JSON | documentation |  |
| `docs/ui/figcraft/EPISODES/b-06_effects.json` | 1,624 | `D2ACA9B04DF4C327` | JSON | documentation |  |
| `docs/ui/figcraft/EPISODES/b-exports.json` | 2,929 | `CE54FCBC5A52FA40` | JSON | documentation |  |
| `docs/ui/figcraft/WATCHDOG-BRIEF.md` | 3,074 | `916C743FF1A69534` | Markdown | documentation |  |
| `ledgers/ui-watchdog.json` | 8,655 | `810A5AA01E648857` | JSON | documentation |  |
| `ledgers/ui-watchdog.md` | 6,315 | `3EA59E7CDE69DA53` | Markdown | documentation |  |

## Seat F — figcraft-b (91 files)

| Path | Bytes | sha256[:16] | Lang | Category | D |
|---|---:|---|---|---|---|
| `docs/audit/BASELINE-VERIFICATION.md` | 15,071 | `DC11F4811F4E1256` | Markdown | documentation |  |
| `docs/audit/FINAL-CLOSEOUT.md` | 3,218 | `81D45A5C4615AF15` | Markdown | documentation |  |
| `docs/audit/FINAL-DELTA-VERIFICATION.md` | 13,168 | `A4ACBCBB1D17DA92` | Markdown | documentation |  |
| `docs/audit/PHASE-B-VERIFICATION.md` | 19,881 | `CA6E76D1C942EF85` | Markdown | documentation |  |
| `docs/audit/QA-ENV.md` | 6,608 | `E3ED721740DAF0E2` | Markdown | documentation |  |
| `docs/audit/evidence/controls-verifier-skeleton.json` | 39,735 | `2AE3C3F74A8CD378` | JSON | documentation |  |
| `docs/audit/evidence/final/cli-diagnose-final.err` | 0 | `E3B0C44298FC1C14` | Log | documentation |  |
| `docs/audit/evidence/final/cli-diagnose-final.out` | 6,246 | `6596AD0FDE6AE8C0` | Log | documentation |  |
| `docs/audit/evidence/final/cli-health-final.out` | 1,452 | `FC5FE6F367A1D692` | Log | documentation |  |
| `docs/audit/evidence/final/live-probe-final.json` | 5,877 | `5AE601760ACFA8FC` | JSON | documentation |  |
| `docs/audit/evidence/final/live-probe-final.mjs` | 6,788 | `854F35D3F6696635` | JavaScript | documentation |  |
| `docs/audit/evidence/final/live-probe-final.out.txt` | 7,786 | `9FF94FE9FEB7494E` | Text | documentation |  |
| `docs/audit/evidence/final/pw-out/.last-run.json` | 96 | `9C10E541B3FF84B2` | JSON | documentation |  |
| `docs/audit/evidence/final/pw-out/ws-contract-WS-transport-c-9a060-refusal-has-code-how-to-fix-chrome/error-context.md` | 3,113 | `BDF922489455595E` | Markdown | documentation |  |
| `docs/audit/evidence/final/pw-out/ws-contract-WS-transport-c-9a060-refusal-has-code-how-to-fix-chrome/test-failed-1.png` | 4,254 | `3878570F4A62A0D6` | PNG | documentation-asset |  |
| `docs/audit/evidence/final/pw-out/ws-contract-WS-transport-c-9a060-refusal-has-code-how-to-fix-chrome/trace.zip` | 11,204 | `7835BF048A538279` | Zip | documentation-asset |  |
| `docs/audit/evidence/gen_controls.py` | 15,934 | `8EBC94962F25917A` | Python | documentation |  |
| `docs/audit/evidence/phase-b/cli/census-verify.json` | 4,968 | `1E0139F367D7BD1A` | JSON | documentation |  |
| `docs/audit/evidence/phase-b/cli/census.err` | 0 | `E3B0C44298FC1C14` | Log | documentation |  |
| `docs/audit/evidence/phase-b/cli/census.out` | 4,968 | `1E0139F367D7BD1A` | Log | documentation |  |
| `docs/audit/evidence/phase-b/cli/diagnose_live.err` | 0 | `E3B0C44298FC1C14` | Log | documentation |  |
| `docs/audit/evidence/phase-b/cli/diagnose_live.out` | 964 | `45750E20D2529F12` | Log | documentation |  |
| `docs/audit/evidence/phase-b/cli/health_dryrun.err` | 0 | `E3B0C44298FC1C14` | Log | documentation |  |
| `docs/audit/evidence/phase-b/cli/health_dryrun.out` | 568 | `B687C7DF18B9EACE` | Log | documentation |  |
| `docs/audit/evidence/phase-b/cli/health_live.err` | 0 | `E3B0C44298FC1C14` | Log | documentation |  |
| `docs/audit/evidence/phase-b/cli/health_live.out` | 1,450 | `594B46239D6A2183` | Log | documentation |  |
| `docs/audit/evidence/phase-b/cli/preflight-path.err` | 0 | `E3B0C44298FC1C14` | Log | documentation |  |
| `docs/audit/evidence/phase-b/cli/preflight-path.out` | 1,374 | `C39AF6AF2C1B2645` | Log | documentation |  |
| `docs/audit/evidence/phase-b/cli/preflight.err` | 0 | `E3B0C44298FC1C14` | Log | documentation |  |
| `docs/audit/evidence/phase-b/cli/preflight.out` | 1,640 | `2457B48AB44CF069` | Log | documentation |  |
| `docs/audit/evidence/phase-b/cli/provider_offpeak.err` | 0 | `E3B0C44298FC1C14` | Log | documentation |  |
| `docs/audit/evidence/phase-b/cli/provider_offpeak.out` | 1,824 | `D04DFCA68B01BB11` | Log | documentation |  |
| `docs/audit/evidence/phase-b/cli/provider_peak.err` | 0 | `E3B0C44298FC1C14` | Log | documentation |  |
| `docs/audit/evidence/phase-b/cli/provider_peak.out` | 1,816 | `0FBE228F6B9AF4BB` | Log | documentation |  |
| `docs/audit/evidence/phase-b/cli/schema.err` | 0 | `E3B0C44298FC1C14` | Log | documentation |  |
| `docs/audit/evidence/phase-b/cli/schema.out` | 4,000 | `BB9F478733F49FE5` | Log | documentation |  |
| `docs/audit/evidence/phase-b/cli/tool_missing.err` | 0 | `E3B0C44298FC1C14` | Log | documentation |  |
| `docs/audit/evidence/phase-b/cli/tool_missing.out` | 834 | `C9411A2CA776C312` | Log | documentation |  |
| `docs/audit/evidence/phase-b/cli/tools_describe.err` | 0 | `E3B0C44298FC1C14` | Log | documentation |  |
| `docs/audit/evidence/phase-b/cli/tools_describe.out` | 1,934 | `3C3D6EFE7B6F3814` | Log | documentation |  |
| `docs/audit/evidence/phase-b/cli/tools_list.err` | 0 | `E3B0C44298FC1C14` | Log | documentation |  |
| `docs/audit/evidence/phase-b/cli/tools_list.out` | 2,432 | `8778881423599856` | Log | documentation |  |
| `docs/audit/evidence/phase-b/cli/unknown.err` | 0 | `E3B0C44298FC1C14` | Log | documentation |  |
| `docs/audit/evidence/phase-b/cli/unknown.out` | 824 | `F89A0CEB8515EA8D` | Log | documentation |  |
| `docs/audit/evidence/phase-b/cli/usage_error.err` | 0 | `E3B0C44298FC1C14` | Log | documentation |  |
| `docs/audit/evidence/phase-b/cli/usage_error.out` | 704 | `F621505E51474A1F` | Log | documentation |  |
| `docs/audit/evidence/phase-b/cli/version-human.err` | 726 | `23CF6824D5209A97` | Log | documentation |  |
| `docs/audit/evidence/phase-b/cli/version-human.out` | 0 | `E3B0C44298FC1C14` | Log | documentation |  |
| `docs/audit/evidence/phase-b/cli/version.err` | 0 | `E3B0C44298FC1C14` | Log | documentation |  |
| `docs/audit/evidence/phase-b/cli/version.out` | 818 | `84BEFDB61A976C7D` | Log | documentation |  |
| `docs/audit/evidence/phase-b/live-probe-verifier-v2.json` | 4,890 | `19D660FBEBB3C9EA` | JSON | documentation |  |
| `docs/audit/evidence/phase-b/live-probe-verifier-v2.out.txt` | 8,882 | `8A87F5D00137028C` | Text | documentation |  |
| `docs/audit/evidence/phase-b/live-probe-verifier.json` | 4,890 | `19D660FBEBB3C9EA` | JSON | documentation |  |
| `docs/audit/evidence/phase-b/live-probe-verifier.mjs` | 6,414 | `5C21CD473E430DE4` | JavaScript | documentation |  |
| `docs/audit/evidence/phase-b/live-probe-verifier.out.txt` | 9,416 | `25B4CE4E8AEFF40D` | Text | documentation |  |
| `docs/audit/evidence/toolchain/hello.c` | 75 | `1567B81E13E47B8A` | C | documentation |  |
| `docs/audit/evidence/toolchain/hello.exe` | 49,876 | `8F186FF8B9B159DD` | PE-binary | documentation |  |
| `docs/audit/evidence/toolchain/hello.rs` | 45 | `913D97C057D169DA` | Rust | documentation |  |
| `docs/audit/evidence/toolchain/hello_rs.exe` | 5,088,192 | `8DF34E8A4D2C7109` | PE-binary | documentation |  |
| `docs/claims/CLAIM-REGISTER.csv` | 24,292 | `1518294B77D25323` | CSV | documentation |  |
| `docs/claims/CLAIM-REGISTER.json` | 2,717 | `C58869DAE44D8476` | JSON | documentation |  |
| `docs/claims/SKILL-USE.md` | 1,532 | `7D649CACB3A441DD` | Markdown | documentation |  |
| `docs/contracts/ADAPTERS.md` | 10,628 | `ED7FEEE4747CF898` | Markdown | documentation |  |
| `docs/contracts/UI-COMMANDS-v1.md` | 6,887 | `B7A50E8EAAF13F8E` | Markdown | documentation |  |
| `docs/plans/D2-D3-WIRING-EVIDENCE.md` | 7,031 | `77F6D5CD0BFD3505` | Markdown | documentation |  |
| `docs/plans/FRONTEND-STATUS.md` | 10,244 | `475A57BFDB9C3867` | Markdown | documentation |  |
| `docs/plans/HASH-VERIFICATION.md` | 2,978 | `1CAECEA3CF33F0AD` | Markdown | documentation |  |
| `docs/plans/PHASE-0-RECON.md` | 1,388 | `D52E9EF05217D661` | Markdown | documentation |  |
| `docs/plans/PHASE-1-EVIDENCE.md` | 12,164 | `AF6C28D6543CE2B2` | Markdown | documentation |  |
| `docs/plans/PHASE-1-RECON.md` | 4,990 | `9745DB35A2055C23` | Markdown | documentation |  |
| `docs/plans/PHASE-11-EVIDENCE.md` | 6,616 | `2CE0F35EF15ABABD` | Markdown | documentation |  |
| `docs/plans/PHASE-11-RECON.md` | 3,989 | `52113636EC3839E5` | Markdown | documentation |  |
| `docs/plans/PHASE-12-EVIDENCE.md` | 9,448 | `74CA606808C286DF` | Markdown | documentation |  |
| `docs/plans/PHASE-12-RECON.md` | 3,738 | `83DE50FDF69C33EC` | Markdown | documentation |  |
| `docs/plans/PHASE-2-EVIDENCE.md` | 8,178 | `5AB509605FA86CDF` | Markdown | documentation |  |
| `docs/plans/PHASE-2-RECON.md` | 5,473 | `F0FFC149B0BD0D03` | Markdown | documentation |  |
| `docs/plans/PHASE-3-EVIDENCE.md` | 5,624 | `E7E398223C8CE99D` | Markdown | documentation |  |
| `docs/plans/PHASE-3-RECON.md` | 4,563 | `F21666ECF1CEA59F` | Markdown | documentation |  |
| `docs/plans/PHASE-4-EVIDENCE.md` | 6,243 | `274EFBB4FB512A58` | Markdown | documentation |  |
| `docs/plans/PHASE-4-RECON.md` | 4,161 | `24F408B03AC901D7` | Markdown | documentation |  |
| `docs/plans/PHASE-5-EVIDENCE.md` | 6,521 | `4A176CD2BEFF7774` | Markdown | documentation |  |
| `docs/plans/PHASE-5-RECON.md` | 4,101 | `C930023A320BDE7F` | Markdown | documentation |  |
| `docs/plans/PHASE-6-EVIDENCE.md` | 10,265 | `DE6EEFB3E8040B06` | Markdown | documentation |  |
| `docs/plans/PHASE-6-RECON.md` | 4,254 | `E43CE35A86312638` | Markdown | documentation |  |
| `docs/plans/PHASE-7-8-10-EVIDENCE.md` | 11,738 | `8714D474F62BFE79` | Markdown | documentation |  |
| `docs/plans/PHASE-7-8-10-RECON.md` | 4,784 | `A707E9314D4C867D` | Markdown | documentation |  |
| `docs/plans/PHASE-9-EVIDENCE.md` | 5,612 | `699E1DE3DDE3F669` | Markdown | documentation |  |
| `docs/plans/PHASE-9-RECON.md` | 3,912 | `920896015BD7C601` | Markdown | documentation |  |
| `docs/plans/SESSION-PLAN.md` | 2,829 | `839B691CE96AE57F` | Markdown | documentation |  |
| `docs/plans/TOOLCHAIN.md` | 2,022 | `86F2B9C21EB8C318` | Markdown | documentation |  |
| `docs/reports/FINAL-REPORT.md` | 6,121 | `978271BE811ACDFD` | Markdown | documentation |  |

## Seat G — ui-watchdog (49 files)

| Path | Bytes | sha256[:16] | Lang | Category | D |
|---|---:|---|---|---|---|
| `.cargo/config.toml` | 407 | `7E2C7429B2BFB246` | TOML | other |  |
| `.gitignore` | 158 | `FCA124B10A7E08A0` | unknown | other |  |
| `CHANGE-LEDGER-IMPL.md` | 2,393 | `3995499955B942F3` | Markdown | other |  |
| `README.md` | 1,742 | `B60FAC5A2A7DDFB2` | Markdown | other |  |
| `docs/diagrams/agent-lifecycle-state.mmd` | 1,660 | `E8DCD06FB84CEB22` | Mermaid | documentation |  |
| `docs/diagrams/agent-lifecycle-state.png` | 136,871 | `97F3984D1F22D64A` | PNG | documentation-asset |  |
| `docs/diagrams/agent-lifecycle-state.svg` | 72,503 | `559347BD2DD440DD` | SVG | documentation-asset |  |
| `docs/diagrams/architecture.mmd` | 2,080 | `4A1F683DC05275F5` | Mermaid | documentation |  |
| `docs/diagrams/architecture.png` | 120,208 | `05475658BC89C592` | PNG | documentation-asset |  |
| `docs/diagrams/architecture.svg` | 41,481 | `89E8A0C76C25577E` | SVG | documentation-asset |  |
| `docs/diagrams/er-schema.mmd` | 2,725 | `B6E82A96FD19AEF1` | Mermaid | documentation |  |
| `docs/diagrams/er-schema.png` | 119,367 | `075A77FC69CABF60` | PNG | documentation-asset |  |
| `docs/diagrams/er-schema.svg` | 218,998 | `47A4C5844B6B426F` | SVG | documentation-asset |  |
| `docs/diagrams/run-sequence.mmd` | 2,201 | `18FF2A7FE8AFBA17` | Mermaid | documentation |  |
| `docs/diagrams/run-sequence.png` | 149,891 | `E9703E3E6B0B3A1C` | PNG | documentation-asset |  |
| `docs/diagrams/run-sequence.svg` | 47,817 | `F4AB892C62692347` | SVG | documentation-asset |  |
| `docs/diagrams/ui-control-event-sequence.mmd` | 1,880 | `1CB4689E29310FCE` | Mermaid | documentation |  |
| `docs/diagrams/ui-control-event-sequence.png` | 98,011 | `0C71C776FFC17A9C` | PNG | documentation-asset |  |
| `docs/diagrams/ui-control-event-sequence.svg` | 43,918 | `6CC5DA36A603A2DE` | SVG | documentation-asset |  |
| `docs/trace/A11Y-CONTRAST-PLAN.md` | 5,837 | `8DB0269DA711AE2C` | Markdown | documentation |  |
| `docs/trace/QA-HARNESS.md` | 19,296 | `09734D2A94E73373` | Markdown | documentation |  |
| `docs/trace/TRACE-SCHEMA.md` | 6,984 | `836CA29B8976A406` | Markdown | documentation |  |
| `docs/trace/controls.json` | 73,765 | `D364512D98EEEF50` | JSON | documentation |  |
| `docs/trace/events.json` | 1,272 | `1037643C7D56A6CF` | JSON | documentation |  |
| `docs/trace/live/README.md` | 6,081 | `DE5A193B6D24ED3E` | Markdown | documentation |  |
| `docs/trace/live/app-boot-full.png` | 112,923 | `6368078AE15D3B99` | PNG | documentation-asset |  |
| `docs/trace/live/app-boot.png` | 112,876 | `E0C1B0AD6C9815BF` | PNG | documentation-asset |  |
| `docs/trace/live/inspect.json` | 10,462 | `CA23AD61B0E8E569` | JSON | documentation |  |
| `docs/trace/live/live-app/01-boot.json` | 231 | `B3DED1CD14658DAF` | JSON | documentation |  |
| `docs/trace/live/live-app/02-skip-link.json` | 131 | `1246C1EA54B02384` | JSON | documentation |  |
| `docs/trace/live/live-app/03-console.json` | 46 | `4E768728E3A67852` | JSON | documentation |  |
| `docs/trace/live/live-app/04-census.json` | 639 | `B2E5124A529DC8C4` | JSON | documentation |  |
| `docs/trace/live/live-app/05-tabs.json` | 56 | `81DE340B8378D8A1` | JSON | documentation |  |
| `docs/trace/live/live-app/06-nav.json` | 122 | `B92DEBDBB0B57E3B` | JSON | documentation |  |
| `docs/trace/live/live-app/07-zoom.json` | 197 | `8E27D5EF0DBEBC52` | JSON | documentation |  |
| `docs/trace/live/live-app/08-chat-pin.json` | 334 | `892CC160997F0901` | JSON | documentation |  |
| `docs/trace/live/live-app/09-empty-prompt.json` | 50 | `1CED73CEF3179D73` | JSON | documentation |  |
| `docs/trace/live/live-app/10-ws-handshake.json` | 200 | `9F4B564EA9CE146B` | JSON | documentation |  |
| `docs/trace/live/live-app/11-colours.json` | 265 | `70B0F44C187C3CC5` | JSON | documentation |  |
| `docs/trace/live/live-app/12-settings-status.json` | 266 | `5BA61E4C91D46141` | JSON | documentation |  |
| `docs/trace/live/live-app/13-contract-drift.json` | 443 | `4D93E38D8F022589` | JSON | documentation |  |
| `docs/trace/live/live-app/14-file-tree.json` | 504 | `7D8A09750959415C` | JSON | documentation |  |
| `docs/trace/live/live-app/15-data-size.json` | 678 | `4E026FD90D1A53EA` | JSON | documentation |  |
| `docs/trace/live/live-app/boot.png` | 99,367 | `9EA23060505B5C65` | PNG | documentation-asset |  |
| `docs/trace/live/live-app/surface-census.png` | 99,367 | `9EA23060505B5C65` | PNG | documentation-asset |  |
| `docs/trace/live/trace-capture.json` | 22,492 | `2D22C75E3A6F2D2E` | JSON | documentation |  |
| `docs/trace/terminal-truth.md` | 2,548 | `BBC7C40EDE3E7038` | Markdown | documentation |  |
| `docs/trace/traces.json` | 279 | `D0BA5A066CD07A4D` | JSON | documentation |  |
| `rust-toolchain.toml` | 108 | `6F03E2211329AC20` | TOML | other |  |

## Seat H — strip-watchdog (0 files)

| Path | Bytes | sha256[:16] | Lang | Category | D |
|---|---:|---|---|---|---|

