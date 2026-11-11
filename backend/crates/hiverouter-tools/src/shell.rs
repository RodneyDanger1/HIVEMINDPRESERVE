//! Allowlisted child-process execution inside the workspace jail (WP 6.8).
//!
//! # Truth: this is not an OS sandbox
//!
//! `shell_exec` is **approval-gated arbitrary code execution**. The approved
//! child runs with the same user privileges as the server. Even the
//! `conservative_default` allowlist is not a containment boundary: `git.exe` and
//! `cargo.exe` execute repository-controlled code (hooks, aliases, build
//! scripts) which can in turn run any program the user can run. The checks in
//! this module reduce accidental command-string injection and refuse command
//! interpreters, but they do **not** sandbox code: an operator who approves
//! `shell_exec` is approving arbitrary code execution until an OS sandbox
//! exists.
//!
//! Mechanical contract:
//! - the program must be on the explicit allowlist (by bare executable name);
//!   paths with separators are refused so a call cannot re-point the allowlist;
//! - command-interpreter hosts ([`SHELL_INTERPRETER_HOSTS`]) are refused even
//!   when a policy lists them;
//! - argument escape forms (`/c`, `/k`, `&`, `|`, `>`, CR/LF) are refused
//!   before any grant is consumed or any child is spawned;
//! - no shell interpolation: arguments are passed directly to the program;
//! - working directory is resolved through [`WorkspaceJail`];
//! - stdout/stderr are capped; the child is bounded by a timeout and killed
//!   through a kill-on-close Windows Job Object (single containment owner:
//!   win32job, never cap-std), which also reaps grandchildren;
//! - run cancellation returns `E_TOOL_CANCELLED` after terminating the child.

use std::path::PathBuf;
use std::time::{Duration, Instant};

use hiverouter_contract::Refusal;
use serde::Serialize;
use tokio::io::{AsyncRead, AsyncReadExt};
use tokio::process::{Child, Command};
use tokio_util::sync::CancellationToken;

use crate::error::ToolError;
use crate::jail::WorkspaceJail;

/// Default stdout cap (1 MiB) and stderr cap (256 KiB).
pub const DEFAULT_MAX_STDOUT_BYTES: usize = 1024 * 1024;
/// Default stderr cap.
pub const DEFAULT_MAX_STDERR_BYTES: usize = 256 * 1024;
/// Default tool timeout (60 s).
pub const DEFAULT_TIMEOUT: Duration = Duration::from_secs(60);

/// Command-interpreter/script hosts refused regardless of the allowlist.
///
/// Denying these by name keeps a policy (or a future default) from turning one
/// approved call into an unconstrained command string. The list is deliberately
/// exact and documented rather than heuristic; it is **not** a complete
/// arbitrary-code-execution boundary (see the module docs).
pub const SHELL_INTERPRETER_HOSTS: &[&str] = &[
    // Windows also resolves bare names through PATHEXT, so both the bare and
    // `.exe` spellings are denied.
    "cmd",
    "cmd.exe",
    "command",
    "command.com",
    "powershell",
    "powershell.exe",
    "pwsh",
    "pwsh.exe",
    "cscript",
    "cscript.exe",
    "wscript",
    "wscript.exe",
    "mshta",
    "mshta.exe",
    "wsl",
    "wsl.exe",
    "busybox",
    "busybox.exe",
    "sh",
    "sh.exe",
    "bash",
    "bash.exe",
    "zsh",
    "zsh.exe",
    "dash",
    "dash.exe",
    "ksh",
    "ksh.exe",
    "fish",
    "fish.exe",
];

/// Whether a bare program name is a denied command-interpreter host.
#[must_use]
pub fn is_shell_interpreter(program: &str) -> bool {
    // Windows strips trailing dots and spaces from executable names; normalize
    // both so `cmd.exe.` cannot slip past the name check if a policy lists it.
    let program = program.trim().trim_end_matches('.').to_lowercase();
    SHELL_INTERPRETER_HOSTS
        .iter()
        .any(|host| *host == program.as_str())
}

/// Refuses argument forms that a command interpreter would treat as syntax.
///
/// Arguments are normally passed directly to a non-interpreter program, but the
/// listed forms are refused before the approval grant is consumed so a mistaken
/// `cmd /c`-style launch can never become a command string. The exact denied
/// forms are `/c`, `/k`, `&`, `|`, `>` and CR/LF.
///
/// # Errors
/// [`Refusal::ToolNotAllowed`] naming the offending form (the argument value is
/// not echoed into the refusal detail beyond a bounded quoted excerpt).
pub fn validate_arguments(args: &[String]) -> Result<(), ToolError> {
    for argument in args {
        let trimmed = argument.trim();
        if trimmed.eq_ignore_ascii_case("/c") || trimmed.eq_ignore_ascii_case("/k") {
            return Err(ToolError::new(
                Refusal::ToolNotAllowed,
                format!(
                    "shell_exec argument {argument:?} is a command-interpreter switch; \
                     interpreter switches (/c, /k) are refused before execution"
                ),
            ));
        }
        if argument.contains('&')
            || argument.contains('|')
            || argument.contains('>')
            || argument.contains('\n')
            || argument.contains('\r')
        {
            return Err(ToolError::new(
                Refusal::ToolNotAllowed,
                format!(
                    "shell_exec argument {argument:?} contains a shell escape/metacharacter \
                     (&, |, >, or a newline); such arguments are refused before execution"
                ),
            ));
        }
    }
    Ok(())
}

/// The shell allowlist and resource caps.
#[derive(Debug, Clone)]
pub struct ShellPolicy {
    allowed_programs: Vec<String>,
    max_stdout_bytes: usize,
    max_stderr_bytes: usize,
    timeout: Duration,
}

impl ShellPolicy {
    /// Creates a policy from the program allowlist (comparison is
    /// case-insensitive on the bare executable name).
    #[must_use]
    pub fn new(allowed_programs: impl IntoIterator<Item = String>) -> Self {
        let mut allowed: Vec<String> = allowed_programs
            .into_iter()
            .map(|program| program.trim().to_lowercase())
            .filter(|program| !program.is_empty())
            .collect();
        allowed.sort();
        allowed.dedup();
        Self {
            allowed_programs: allowed,
            max_stdout_bytes: DEFAULT_MAX_STDOUT_BYTES,
            max_stderr_bytes: DEFAULT_MAX_STDERR_BYTES,
            timeout: DEFAULT_TIMEOUT,
        }
    }

    /// The default allowlist for v1 developer utilities.
    ///
    /// `cmd.exe` is intentionally **absent**: a command interpreter turns one
    /// approved call into an arbitrary command string. This is not a sandbox —
    /// `git.exe` and `cargo.exe` can execute repository-controlled code with the
    /// user's privileges (hooks, aliases, build scripts) — so `shell_exec`
    /// remains approval-gated arbitrary code execution. The approval gate
    /// (`Execute` permission class) still runs before execution, and every
    /// interpreter host is refused regardless of this list.
    #[must_use]
    pub fn conservative_default() -> Self {
        Self::new(["git.exe".to_owned(), "cargo.exe".to_owned()])
    }

    /// Overrides the tool timeout.
    #[must_use]
    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    /// Overrides the stdout/stderr caps.
    #[must_use]
    pub fn with_stdout_cap(mut self, max_stdout_bytes: usize) -> Self {
        self.max_stdout_bytes = max_stdout_bytes;
        self
    }

    /// The allowlist.
    #[must_use]
    pub fn allowed_programs(&self) -> &[String] {
        &self.allowed_programs
    }

    /// Whether a program name is allowed (bare name only).
    ///
    /// Command-interpreter hosts ([`is_shell_interpreter`]) are never allowed,
    /// even when an explicit policy lists them.
    #[must_use]
    pub fn allows(&self, program: &str) -> bool {
        if program.contains('\\') || program.contains('/') {
            return false;
        }
        if is_shell_interpreter(program) {
            return false;
        }
        self.allowed_programs
            .iter()
            .any(|allowed| allowed == &program.trim().to_lowercase())
    }

    /// Configured timeout.
    #[must_use]
    pub fn timeout(&self) -> Duration {
        self.timeout
    }
}

/// One shell request (arguments passed directly, no shell interpolation).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ShellRequest {
    /// Bare program name.
    pub program: String,
    /// Arguments.
    pub args: Vec<String>,
    /// Workspace-relative working directory (`None` = workspace root).
    pub cwd: Option<String>,
}

impl ShellRequest {
    /// Builds a request.
    #[must_use]
    pub fn new(program: impl Into<String>, args: Vec<String>) -> Self {
        Self {
            program: program.into(),
            args,
            cwd: None,
        }
    }

    /// Sets the working directory.
    #[must_use]
    pub fn with_cwd(mut self, cwd: impl Into<String>) -> Self {
        self.cwd = Some(cwd.into());
        self
    }
}

/// Result of an executed child process (non-zero exit is a result, not a
/// refusal: the program ran and its exit code is data).
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ShellOutput {
    /// Program executed.
    pub program: String,
    /// Arguments passed.
    pub args: Vec<String>,
    /// Exit code (`None` when terminated by signal).
    pub exit_code: Option<i32>,
    /// Whether the process exited successfully (exit code 0).
    pub success: bool,
    /// Captured stdout (capped).
    pub stdout: String,
    /// Captured stderr (capped).
    pub stderr: String,
    /// Stdout or stderr was truncated at the cap.
    pub truncated: bool,
    /// Wall-clock duration in milliseconds.
    pub duration_ms: u128,
}

/// Executes an allowlisted program inside the jail.
///
/// # Errors
/// [`Refusal::ToolNotAllowed`] for a program outside the allowlist or a
/// separator-bearing program string; [`Refusal::JailEscape`] for an escaping
/// working directory; [`Refusal::ToolTimeout`] when the child exceeds the
/// timeout (child terminated, grandchildren killed by the Job Object);
/// [`Refusal::ToolCancelled`] when the run is cancelled; [`Refusal::InternalError`]
/// for spawn/I/O failures.
pub async fn execute(
    jail: &WorkspaceJail,
    policy: &ShellPolicy,
    request: &ShellRequest,
    cancellation: Option<&CancellationToken>,
) -> Result<ShellOutput, ToolError> {
    if is_shell_interpreter(&request.program) {
        return Err(ToolError::new(
            Refusal::ToolNotAllowed,
            format!(
                "program {:?} is a command-interpreter host; interpreter hosts are denied \
                 regardless of the allowlist",
                request.program
            ),
        ));
    }
    if !policy.allows(&request.program) {
        return Err(ToolError::new(
            Refusal::ToolNotAllowed,
            format!(
                "program {:?} is not on the shell allowlist ({}); path-qualified programs are refused",
                request.program,
                policy.allowed_programs().join(", ")
            ),
        ));
    }
    validate_arguments(&request.args)?;
    let cwd: PathBuf = match &request.cwd {
        Some(cwd) => jail.resolve(cwd)?,
        None => jail.root().to_path_buf(),
    };
    if !cwd.is_dir() {
        return Err(ToolError::new(
            Refusal::ToolSchema,
            "shell working directory is not a directory inside the jail",
        ));
    }

    let started = Instant::now();
    let mut command = Command::new(&request.program);
    command
        .args(&request.args)
        .current_dir(&cwd)
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .kill_on_drop(true);
    crate::env_safety::sanitize_tokio(&mut command);

    let mut child = command.spawn().map_err(|error| {
        ToolError::new(
            Refusal::InternalError,
            format!("failed to spawn {:?}: {error}", request.program),
        )
    })?;

    // Windows containment owner: kill-on-close Job Object. Dropping the job at
    // the end of this function kills any descendant still running.
    #[cfg(windows)]
    let job = create_kill_on_close_job()?;
    #[cfg(windows)]
    assign_to_job(&job, &child)?;

    let stdout = child.stdout.take().ok_or_else(|| {
        ToolError::new(Refusal::InternalError, "child stdout pipe is unavailable")
    })?;
    let stderr = child.stderr.take().ok_or_else(|| {
        ToolError::new(Refusal::InternalError, "child stderr pipe is unavailable")
    })?;
    let stdout_cap = policy.max_stdout_bytes;
    let stderr_cap = policy.max_stderr_bytes;

    let work = async move {
        let (stdout, stdout_truncated) = read_capped(stdout, stdout_cap).await?;
        let (stderr, stderr_truncated) = read_capped(stderr, stderr_cap).await?;
        let status = child.wait().await?;
        Ok::<_, std::io::Error>((stdout, stdout_truncated, stderr, stderr_truncated, status))
    };

    let timed = async {
        if let Some(token) = cancellation {
            tokio::select! {
                () = token.cancelled() => Outcome::Cancelled,
                result = tokio::time::timeout(policy.timeout, work) => outcome_from(result),
            }
        } else {
            outcome_from(tokio::time::timeout(policy.timeout, work).await)
        }
    }
    .await;

    match timed {
        Outcome::Finished(stdout, stdout_truncated, stderr, stderr_truncated, status) => {
            Ok(ShellOutput {
                program: request.program.clone(),
                args: request.args.clone(),
                exit_code: status.code(),
                success: status.success(),
                stdout: String::from_utf8_lossy(&stdout).into_owned(),
                stderr: String::from_utf8_lossy(&stderr).into_owned(),
                truncated: stdout_truncated || stderr_truncated,
                duration_ms: started.elapsed().as_millis(),
            })
        }
        Outcome::TimedOut => Err(ToolError::new(
            Refusal::ToolTimeout,
            format!(
                "{:?} exceeded the {} ms tool timeout and was terminated",
                request.program,
                policy.timeout.as_millis()
            ),
        )),
        Outcome::Cancelled => Err(ToolError::new(
            Refusal::ToolCancelled,
            format!("{:?} was cancelled by run cancellation", request.program),
        )),
        Outcome::Io(detail) => Err(ToolError::new(
            Refusal::InternalError,
            format!("I/O failure while running {:?}: {detail}", request.program),
        )),
    }
}

enum Outcome {
    Finished(Vec<u8>, bool, Vec<u8>, bool, std::process::ExitStatus),
    TimedOut,
    Cancelled,
    Io(String),
}

fn outcome_from(
    result: Result<Result<(Vec<u8>, bool, Vec<u8>, bool, std::process::ExitStatus), std::io::Error>, tokio::time::error::Elapsed>,
) -> Outcome {
    match result {
        Err(_) => Outcome::TimedOut,
        Ok(Err(error)) => Outcome::Io(error.to_string()),
        Ok(Ok((stdout, stdout_truncated, stderr, stderr_truncated, status))) => {
            Outcome::Finished(stdout, stdout_truncated, stderr, stderr_truncated, status)
        }
    }
}

/// Reads at most `cap` bytes while draining the pipe (no deadlock, no
/// unbounded buffering); reports truncation.
async fn read_capped<R: AsyncRead + Unpin>(
    mut reader: R,
    cap: usize,
) -> std::io::Result<(Vec<u8>, bool)> {
    let mut chunk = [0u8; 8192];
    let mut stored = Vec::new();
    let mut truncated = false;
    loop {
        let read = reader.read(&mut chunk).await?;
        if read == 0 {
            break;
        }
        let remaining = cap.saturating_sub(stored.len());
        let take = read.min(remaining);
        if take > 0 {
            stored.extend_from_slice(&chunk[..take]);
        }
        if take < read {
            truncated = true;
        }
    }
    Ok((stored, truncated))
}

#[cfg(windows)]
fn create_kill_on_close_job() -> Result<win32job::Job, ToolError> {
    let mut info = win32job::ExtendedLimitInfo::new();
    info.limit_kill_on_job_close();
    win32job::Job::create_with_limit_info(&mut info).map_err(|error| {
        ToolError::new(
            Refusal::InternalError,
            format!("failed to create the kill-on-close Job Object: {error}"),
        )
    })
}

#[cfg(windows)]
fn assign_to_job(job: &win32job::Job, child: &Child) -> Result<(), ToolError> {
    let handle = child.raw_handle().ok_or_else(|| {
        ToolError::new(
            Refusal::InternalError,
            "child process handle is unavailable for Job Object assignment",
        )
    })?;
    job.assign_process(handle as isize).map_err(|error| {
        ToolError::new(
            Refusal::InternalError,
            format!("failed to assign the child process to the Job Object: {error}"),
        )
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn jail() -> (tempfile::TempDir, WorkspaceJail) {
        let temp = tempfile::tempdir().expect("tempdir");
        let jail = WorkspaceJail::open(temp.path()).expect("jail");
        (temp, jail)
    }

    /// Explicit non-interpreter test policy. Tests must never depend on a
    /// command interpreter being allowlisted; `git.exe` is safe to execute on
    /// this host (a version banner, or a bounded external `sleep` alias that
    /// Git for Windows supplies through its own shell).
    fn test_policy() -> ShellPolicy {
        ShellPolicy::new(["git.exe".to_owned()])
    }

    /// `git` alias that runs the Git-for-Windows shell's `sleep` in the
    /// foreground; used to exercise timeout/cancellation deterministically
    /// without depending on PATH-shadowed `ping.exe` flavours.
    fn sleep_args(seconds: &str) -> Vec<String> {
        vec![
            "-c".to_owned(),
            format!("alias.hive-sleep=!sleep {seconds}"),
            "hive-sleep".to_owned(),
        ]
    }

    #[test]
    fn allowlist_accepts_bare_names_only() {
        let policy = ShellPolicy::conservative_default();
        assert!(policy.allows("git.exe"));
        assert!(policy.allows("GIT.EXE"));
        assert!(
            !policy.allows("cmd.exe"),
            "cmd.exe was removed from the default allowlist"
        );
        assert!(
            policy.allows("git.exe") && !policy.allows("powershell.exe"),
            "shell interpreter hosts are never allowed"
        );
        assert!(!policy.allows("C:\\Windows\\System32\\git.exe"));
        assert!(!policy.allows("..\\git.exe"));

        // Even a policy that explicitly lists an interpreter cannot enable it.
        let explicit = ShellPolicy::new(["cmd.exe".to_owned(), "git.exe".to_owned()]);
        assert!(
            !explicit.allows("cmd.exe"),
            "interpreter hosts are denied regardless of the allowlist"
        );
        assert!(explicit.allows("git.exe"));
    }

    #[tokio::test]
    async fn non_allowlisted_program_is_refused_by_name() {
        let (_temp, jail) = jail();
        let policy = test_policy();
        let request = ShellRequest::new("powershell.exe", vec![]);
        let error = execute(&jail, &policy, &request, None)
            .await
            .expect_err("not allowed");
        assert_eq!(error.code(), "E_TOOL_NOT_ALLOWED");
    }

    #[tokio::test]
    async fn interpreter_host_is_refused_even_when_explicitly_allowlisted() {
        let (_temp, jail) = jail();
        let policy = ShellPolicy::new(["cmd.exe".to_owned()]);
        for program in [
            "cmd",
            "cmd.exe",
            "CMD.EXE",
            "cmd.exe.",
            "powershell.exe",
            "pwsh.exe",
            "wsl.exe",
            "sh.exe",
        ] {
            let request = ShellRequest::new(program, vec![]);
            let error = execute(&jail, &policy, &request, None)
                .await
                .expect_err(program);
            assert_eq!(error.code(), "E_TOOL_NOT_ALLOWED", "{program}");
            assert!(
                error.detail().contains("interpreter"),
                "the refusal names the interpreter policy: {program}: {error}"
            );
        }
    }

    #[tokio::test]
    async fn argument_escape_forms_are_refused_before_spawn() {
        let (_temp, jail) = jail();
        let policy = test_policy();
        for argument in ["/c", "/C", "/k", "a&b", "a|b", "a>b", "a\nb", "a\rb"] {
            let request = ShellRequest::new("git.exe", vec![argument.to_owned()]);
            let error = execute(&jail, &policy, &request, None)
                .await
                .expect_err(argument);
            assert_eq!(error.code(), "E_TOOL_NOT_ALLOWED", "{argument:?}");
        }
        // Well-formed direct arguments are not rejected by the escape check.
        validate_arguments(&["--version".to_owned()]).expect("plain argument");
    }

    #[cfg(windows)]
    #[tokio::test]
    async fn allowlisted_command_runs_and_captures_stdout() {
        let (_temp, jail) = jail();
        let policy = test_policy();
        let request = ShellRequest::new("git.exe", vec!["--version".into()]);
        let output = execute(&jail, &policy, &request, None)
            .await
            .expect("executes");
        assert!(output.success, "exit: {:?}", output.exit_code);
        assert!(
            output.stdout.to_lowercase().contains("git version"),
            "stdout: {}",
            output.stdout
        );
        assert_eq!(output.exit_code, Some(0));
    }

    #[cfg(windows)]
    #[tokio::test]
    async fn child_process_does_not_inherit_the_provider_key() {
        let previous = std::env::var_os(hiverouter_provider::API_KEY_ENV);
        std::env::set_var(
            hiverouter_provider::API_KEY_ENV,
            "sk-shell-env-test-not-a-real-key",
        );
        let (_temp, jail) = jail();
        let policy = test_policy();
        // `git` runs the external alias through its own shell; the environment
        // printed is the sanitized environment the shell child inherited.
        let request = ShellRequest::new(
            "git.exe",
            vec![
                "-c".into(),
                "alias.hive-env-probe=!env".into(),
                "hive-env-probe".into(),
            ],
        );
        let result = execute(&jail, &policy, &request, None).await;
        match previous {
            Some(value) => std::env::set_var(hiverouter_provider::API_KEY_ENV, value),
            None => std::env::remove_var(hiverouter_provider::API_KEY_ENV),
        }
        let output = result.expect("executes");
        assert!(
            output.success,
            "env probe must run: exit {:?}, stderr {}",
            output.exit_code, output.stderr
        );
        assert!(
            output.stdout.contains("PATH="),
            "env probe must print the child environment: {}",
            output.stdout
        );
        assert!(
            !output.stdout.contains("sk-shell-env-test-not-a-real-key"),
            "child inherited the provider key: {}",
            output.stdout
        );
    }

    #[cfg(windows)]
    #[tokio::test]
    async fn timeout_terminates_the_child() {
        let (_temp, jail) = jail();
        let policy = test_policy().with_timeout(Duration::from_millis(300));
        let request = ShellRequest::new("git.exe", sleep_args("10"));
        let error = execute(&jail, &policy, &request, None)
            .await
            .expect_err("timeout");
        assert_eq!(error.code(), "E_TOOL_TIMEOUT");
    }

    #[cfg(windows)]
    #[tokio::test]
    async fn stdout_cap_truncates_without_deadlock() {
        let (_temp, jail) = jail();
        let policy = test_policy().with_stdout_cap(4);
        let request = ShellRequest::new("git.exe", vec!["--version".into()]);
        let output = execute(&jail, &policy, &request, None)
            .await
            .expect("executes");
        assert!(output.truncated);
        assert!(output.stdout.len() <= 4, "stdout: {:?}", output.stdout);
    }

    #[cfg(windows)]
    #[tokio::test]
    async fn cancellation_returns_named_refusal() {
        let (_temp, jail) = jail();
        let policy = test_policy().with_timeout(Duration::from_secs(30));
        // Long foreground sleep so the only exit path is cancellation.
        let request = ShellRequest::new("git.exe", sleep_args("10"));
        let token = CancellationToken::new();
        let cancel = token.clone();
        tokio::spawn(async move {
            tokio::time::sleep(Duration::from_millis(200)).await;
            cancel.cancel();
        });
        let error = execute(&jail, &policy, &request, Some(&token))
            .await
            .expect_err("cancelled");
        assert_eq!(error.code(), "E_TOOL_CANCELLED");
    }
}
