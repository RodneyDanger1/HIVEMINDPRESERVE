# PHASE 6 EVIDENCE — Tools, Approvals & Execution Safety

Owner: B-Provider. Date: 2026-11-06 (CST). Blueprint card: Phase 6 (WPs 6.1–6.14).
All tests offline; no live provider/network call; no Docker/Python/seccomp (D1 fence).

## Environment / preflight (observed)

```
$ rustc --version   →  rustc 1.99.1 (58cf8f9c3 2026-10-13)
$ cargo --version   →  cargo 1.99.1 (5f94df478 2026-08-27)
$ gcc --version     →  gcc.exe (GCC) 16.2.0 (tools/w64devkit/bin, prepended to PATH)
workspace root: backend/Cargo.toml (run cargo from C:\Users\Riley\Desktop\hiverouter-v1\backend)
```

Preflight (blueprint: `cargo check`):

```
cargo check -p hiverouter-tools --all-targets
  → TOOLS CHECK EXIT: 0
```

## Tests — command + observed output

```
cargo test -p hiverouter-tools
test result: ok. 77 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 3.87s   (lib)
test result: ok. 0 passed; 0 failed; ...                                                        (bin hiverouter-cli)
     Running tests\audit_jsonl.rs
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s
TEST EXIT: 0
```

Phase-6 acceptance tests actually executed (quoted `ok` lines):

```
test approval::tests::approval_suspend_resume_roundtrip ... ok
test approval::tests::approval_args_binding_refuses_different_arguments ... ok
test approval::tests::approval_replay_is_refused_after_one_execution ... ok
test approval::tests::approval_ttl_expiry_has_zero_side_effects ... ok
test approval::tests::deny_leaves_no_grant ... ok
test approval::tests::allow_session_is_scoped_to_run_tool_and_args ... ok
test approval::tests::wire_approval_resolve_uses_nonce_and_replay_refusal ... ok
test approval::tests::wire_approval_decision_is_validated ... ok
test approval::tests::read_only_tool_cannot_request_approval ... ok
test approval::tests::ledger_row_never_stores_argument_body ... ok
test jail::tests::escape_vector_parent_traversal_is_refused ... ok
test jail::tests::escape_vector_absolute_path_is_refused ... ok
test jail::tests::escape_vector_unc_and_device_prefixes_are_refused ... ok
test jail::tests::escape_vector_drive_change_is_refused ... ok
test jail::tests::escape_vector_short_name_alias_is_refused ... ok
test jail::tests::escape_vector_symlink_or_junction_is_refused ... ok
test jail::tests::read_cap_is_enforced ... ok
test jail::tests::reads_writes_and_lists_inside_the_root ... ok
test jail::tests::write_with_approved_relative_path_stays_inside ... ok
test shell::tests::non_allowlisted_program_is_refused_by_name ... ok
test shell::tests::allowlisted_command_runs_and_captures_stdout ... ok
test shell::tests::timeout_terminates_the_child ... ok
test shell::tests::stdout_cap_truncates_without_deadlock ... ok
test shell::tests::cancellation_returns_named_refusal ... ok
test shell::tests::allowlist_accepts_bare_names_only ... ok
test cancel::tests::cancel_broadcasts_to_all_tasks_and_records_terminal ... ok
test cancel::tests::kill_one_task_only_cancels_that_task ... ok
test cancel::tests::tasks_can_select_on_the_token ... ok
test cancel::tests::unregistered_run_and_task_refusals_are_named ... ok
test ledger::tests::jsonl_roundtrips_with_monotonic_seq_and_no_arg_body ... ok
test ledger::tests::refusing_outcome_without_code_is_rejected ... ok
test ledger::tests::raw_argument_body_cannot_be_smuggled_as_digest ... ok
test ledger::tests::append_only_file_survives_reopen ... ok
test tool::tests::every_tool_id_has_exactly_one_contract ... ok
test tool::tests::schemas_are_valid_and_forbid_unknown_fields ... ok
test tool::tests::tier_allowlist_grants_and_refusals_are_by_name ... ok
test tool::tests::resolve_call_validates_args_by_name ... ok
test tool::tests::mutating_and_executing_tools_require_approval ... ok
test tool::tests::allowlist_membership_is_always_a_registered_tool ... ok
test commands::tests::file_read_and_tree_go_through_the_jail ... ok
test commands::tests::file_read_refusals_are_the_patched_codes ... ok
test commands::tests::approval_resolve_command_uses_nonce ... ok
test commands::tests::settings_payload_is_secret_free ... ok
```

## Exit-gate mapping (blueprint Phase 6)

| Gate | Evidence |
|---|---|
| allowlisted tool succeeds; non-allowlisted returns `E_TOOL_NOT_ALLOWED` | `resolve_call_validates_args_by_name` (E_TOOL_NOT_ALLOWED for hyper tier/file_read), `tier_allowlist_grants_and_refusals_are_by_name` |
| mutating action without approval → `E_APPROVAL_REQUIRED`, changes nothing | `approval_args_binding_refuses_different_arguments` (different args → E_APPROVAL_REQUIRED), `deny_leaves_no_grant`; no execution is reachable without `verify_grant` |
| cancel/kill leaves no running task + Cancelled broadcast | `cancel_broadcasts_to_all_tasks_and_records_terminal`, `kill_one_task_only_cancels_that_task`, `tasks_can_select_on_the_token`; child process timeout kills via kill-on-close Job Object (`timeout_terminates_the_child`) |
| path escape (`..`, symlink, other drive) refused + logged | six escape-vector tests, all `E_PATH_OUTSIDE_JAIL`; symlink/junction vector exercised with real `mklink /J`; refusals carry the code that the ledger writes |
| every refusal appears in the run ledger | `refusing_outcome_without_code_is_rejected` (a refusing outcome cannot be recorded without a named code), `jsonl_roundtrips_with_monotonic_seq_and_no_arg_body` |
| approval nonce replay refused; TTL expiry leaves bytes unchanged; jail matrix passes | `wire_approval_resolve_uses_nonce_and_replay_refusal` (wrong nonce → E_APPROVAL_REPLAY; correct nonce resolves once; replay refused; unknown id → E_APPROVAL_NOT_FOUND), `approval_replay_is_refused_after_one_execution`, `approval_suspend_resume_roundtrip` (AllowOnce consumed exactly once), `approval_ttl_expiry_has_zero_side_effects` (canary side effect stays 0; late decision → E_APPROVAL_REPLAY), six-vector jail matrix |

Additionally implemented from the Lead's contract patch (frozen UI-COMMANDS-v1):
`APPROVAL_RESOLVE` (`approve|deny`, nonce+reason, E_APPROVAL_NOT_FOUND/E_APPROVAL_REPLAY/E_APPROVAL_TIMEOUT/
E_APPROVAL_DECISION_INVALID), `SETTINGS_STATUS` (boolean `key_present`, never the key), `FILE_TREE_LIST`,
`FILE_READ` (jailed, 1 MiB cap → E_FILE_TOO_LARGE, UTF-8 only) — handlers are in
`src/commands.rs` and tested by `file_read_and_tree_go_through_the_jail`,
`file_read_refusals_are_the_patched_codes`, `approval_resolve_command_uses_nonce`,
`settings_payload_is_secret_free`.

## Files

`backend/crates/hiverouter-tools/src/{tool,approval,jail,shell,cancel,ledger,sha256,commands,settings,error,lib}.rs`,
`backend/crates/hiverouter-tools/tests/audit_jsonl.rs`, `docs/contracts/ADAPTERS.md` (donor adapter records A2–A4, A8).

## Final-gate defect found and fixed (2026-11-06 08:4x): timeout test PATH shadowing

Symptom (reproduced on the Lead's request): `cargo test -p hiverouter-tools --lib
shell::tests::timeout_terminates_the_child` FAILED deterministically:

```
thread 'shell::tests::timeout_terminates_the_child' panicked at crates\hiverouter-tools\src\shell.rs:413:14:
timeout: ShellOutput { program: "cmd.exe", args: ["/C", "ping -n 4 127.0.0.1 > nul"], exit_code: Some(1),
success: false, stdout: "", stderr: "BusyBox v1.39.0.git-6354-g5f09fb446 ... Usage: ping [OPTIONS] HOST
... -c CNT ...", truncated: false, duration_ms: 87 }
```

Root cause (environment, not product logic): `where.exe ping` on the build PATH returns

```
C:\Users\Riley\Desktop\hiverouter-v1\tools\w64devkit\bin\ping.exe     <- BusyBox applet, first
C:\Windows\System32\PING.EXE                                          <- Windows ping, second
```

The BusyBox `ping` rejects the Windows `-n <count>` flag and exits in ~87 ms with a usage error, so the child
finished *before* the 300 ms timeout and the test asserted the wrong path. The product's timeout/cancel logic
was never exercised by that invocation.

Fix (minimal, test-only): run the **Windows system** ping by absolute path inside `cmd.exe`
(`%SystemRoot%\System32\PING.EXE -n <count> 127.0.0.1 > nul`) in both the timeout and cancellation tests via a
`system_ping_command` helper with the root cause documented. The allowlist still applies to the outer program
(`cmd.exe`); no production code path changed.

After fix (observed):

```
cargo test -p hiverouter-tools --lib shell::tests
test shell::tests::allowlist_accepts_bare_names_only ... ok
test shell::tests::non_allowlisted_program_is_refused_by_name ... ok
test shell::tests::allowlisted_command_runs_and_captures_stdout ... ok
test shell::tests::stdout_cap_truncates_without_deadlock ... ok
test shell::tests::cancellation_returns_named_refusal ... ok
test shell::tests::timeout_terminates_the_child ... ok
test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 71 filtered out; finished in 0.37s
SHELL MODULE EXIT: 0

cargo test -p hiverouter-tools
test result: ok. 77 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 2.26s   (lib)
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s    (tests/audit_jsonl.rs)
TOOLS SUITE EXIT: 0
```
## Not done / UNVERIFIED (honest list)

- **Core wiring pending**: `hiverouter-core` must call `hiverouter_tools::migrate_approvals(&Connection)`,
  `ApprovalGate`/`commands::*`, the `ProcessRegistry` kill hook and the run ledger. This crate is independent and
  tested, but end-to-end integration through core's run executor is B-Core's Phase 4 gate — not claimed here.
- **`win32job` memory ceiling**: the kill-on-close Job Object is proven (timeout/cancel kill the child); a
  memory-limit kill (`child_job_limits`) is NOT exercised — no memory ceiling is configured in v1, recorded as
  UNVERIFIED rather than claimed.
- **Grandchild reaping**: covered by the kill-on-close job semantics when the job is dropped, but no dedicated
  grandchild test exists (criterion: kill-on-close is the mechanism, not a claim about every descendant tree).
- **`cap-std 4.0.3`**: remains spike-gated/not in the lockfile (FACT F10); the jail is canonicalise+prefix based
  and refuses escapes rather than clamping. No cap-std claim is made.
- Approval decision UX/session grants are programmatic (`AllowSession` is internal; the frozen wire only exposes
  `approve|deny` per the contract patch).
