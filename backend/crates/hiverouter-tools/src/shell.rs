//! Allowlisted shell execution inside the workspace jail (WP 6.8).
//!
//! Safety contract:
//! - the program must be on the explicit allowlist (by bare executable name);
//!   paths with separators are refused so a call cannot re-point the allowlist;
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

    /// A conservative default allowlist for v1 developer utilities.
    ///
    /// Only programs that operate on the workspace are listed; the approval
    /// gate (`Execute` permission class) still runs before execution.
    #[must_use]
    pub fn conservative_default() -> Self {
        Self::new([
            "cmd.exe".to_owned(),
            "git.exe".to_owned(),
            "cargo.exe".to_owned(),
        ])
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
    #[must_use]
    pub fn allows(&self, program: &str) -> bool {
        if program.contains('\\') || program.contains('/') {
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

    #[test]
    fn allowlist_accepts_bare_names_only() {
        let policy = ShellPolicy::conservative_default();
        assert!(policy.allows("cmd.exe"));
        assert!(policy.allows("CMD.EXE"));
        assert!(!policy.allows("C:\\Windows\\System32\\cmd.exe"));
        assert!(!policy.allows("..\\cmd.exe"));
        assert!(!policy.allows("powershell.exe"));
    }

    #[tokio::test]
    async fn non_allowlisted_program_is_refused_by_name() {
        let (_temp, jail) = jail();
        let policy = ShellPolicy::conservative_default();
        let request = ShellRequest::new("powershell.exe", vec![]);
        let error = execute(&jail, &policy, &request, None)
            .await
            .expect_err("not allowed");
        assert_eq!(error.code(), "E_TOOL_NOT_ALLOWED");
    }

    #[cfg(windows)]
    #[tokio::test]
    async fn allowlisted_command_runs_and_captures_stdout() {
        let (_temp, jail) = jail();
        let policy = ShellPolicy::conservative_default();
        let request = ShellRequest::new("cmd.exe", vec!["/C".into(), "echo hi-hive".into()]);
        let output = execute(&jail, &policy, &request, None)
            .await
            .expect("executes");
        assert!(output.success, "exit: {:?}", output.exit_code);
        assert!(output.stdout.contains("hi-hive"), "stdout: {}", output.stdout);
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
        let policy = ShellPolicy::conservative_default();
        let request = ShellRequest::new(
            "cmd.exe",
            vec!["/C".into(), "echo %DEEPSEEK_API_KEY%".into()],
        );
        let result = execute(&jail, &policy, &request, None).await;
        match previous {
            Some(value) => std::env::set_var(hiverouter_provider::API_KEY_ENV, value),
            None => std::env::remove_var(hiverouter_provider::API_KEY_ENV),
        }
        let output = result.expect("executes");
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
        let policy = ShellPolicy::conservative_default().with_timeout(Duration::from_millis(300));
        let request = ShellRequest::new("cmd.exe", vec!["/C".into(), system_ping_command(4)]);
        let error = execute(&jail, &policy, &request, None)
            .await
            .expect_err("timeout");
        assert_eq!(error.code(), "E_TOOL_TIMEOUT");
    }

    #[cfg(windows)]
    #[tokio::test]
    async fn stdout_cap_truncates_without_deadlock() {
        let (_temp, jail) = jail();
        let policy = ShellPolicy::conservative_default().with_stdout_cap(4);
        let request = ShellRequest::new(
            "cmd.exe",
            vec!["/C".into(), "echo 1234567890".into()],
        );
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
        let policy = ShellPolicy::conservative_default().with_timeout(Duration::from_secs(30));
        let request = ShellRequest::new("cmd.exe", vec!["/C".into(), system_ping_command(10)]);
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

    /// Builds a `cmd.exe` command line that runs the **Windows system** ping by
    /// absolute path.
    ///
    /// Root cause of the former flake: the test environment puts
    /// `tools/w64devkit/bin` on `PATH`, which ships a BusyBox `ping` that rejects
    /// the Windows `-n <count>` flag and exits in ~90 ms with a usage error. The
    /// child then finished *before* the 300 ms timeout and the test asserted the
    /// wrong path. Using `%SystemRoot%\System32\PING.EXE` makes the duration
    /// deterministic and independent of PATH shadowing; the allowlist still
    /// applies to the outer program (`cmd.exe`) only.
    #[cfg(windows)]
    fn system_ping_command(count: u32) -> String {
        let system_root = std::env::var("SystemRoot").unwrap_or_else(|_| "C:\\Windows".to_owned());
        format!("{system_root}\\System32\\PING.EXE -n {count} 127.0.0.1 > nul")
    }
}
