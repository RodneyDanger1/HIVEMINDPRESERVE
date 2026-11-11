//! Detect and run workspace test suites through the bounded shell seam.
//!
//! Detection is marker-based and never guesses:
//! - `Cargo.toml` → `cargo test`;
//! - `package.json` → `npm test` (only runnable when `scripts.test` exists);
//! - pytest configuration (`pytest.ini`, `pyproject.toml` `[tool.pytest`,
//!   `setup.cfg` `[tool:pytest]`, `tox.ini` `[pytest]`, or a `tests/` directory)
//!   → `python -m pytest`.
//!
//! The search walks from the requested directory up to the workspace root, so a
//! call from a crate subdirectory finds the workspace manifest. Every execution
//! goes through [`crate::shell::execute`] with an explicit program allowlist,
//! the jail working-directory check, a timeout and stdout/stderr caps. Test
//! commands are arbitrary code execution for the repository (scripts, build
//! scripts, test code), so the caller must treat the `run_tests` tool as an
//! approval-gated `Execute` capability, not as a sandbox.

use std::fs;
use std::path::Path;
use std::time::Duration;

use hiverouter_contract::Refusal;
use serde::Serialize;
use serde_json::Value;

use crate::error::ToolError;
use crate::jail::WorkspaceJail;
use crate::shell::{self, ShellPolicy, ShellRequest};

/// Default test timeout (120 s).
pub const DEFAULT_TEST_TIMEOUT: Duration = Duration::from_secs(120);
/// Minimum configurable test timeout.
pub const TEST_MIN_TIMEOUT: Duration = Duration::from_secs(1);
/// Maximum configurable test timeout (10 min).
pub const TEST_MAX_TIMEOUT: Duration = Duration::from_secs(600);
/// Default stdout cap (1 MiB).
pub const DEFAULT_TEST_STDOUT_BYTES: usize = 1024 * 1024;
/// Maximum configurable stdout cap (8 MiB).
pub const TEST_HARD_STDOUT_CAP: usize = 8 * 1024 * 1024;
/// Maximum extra arguments per test invocation.
pub const MAX_TEST_ARGS: usize = 16;
/// Maximum bytes of one test argument.
pub const MAX_TEST_ARG_BYTES: usize = 4096;
/// Maximum combined bytes of extra arguments.
pub const MAX_TEST_ARGV_BYTES: usize = 16 * 1024;

/// A supported test runner.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize)]
pub enum TestRunner {
    /// Cargo test (`cargo test`).
    Cargo,
    /// npm test (`npm test`).
    Npm,
    /// pytest (`python -m pytest`).
    Pytest,
}

impl TestRunner {
    /// Stable wire label.
    #[must_use]
    pub fn label(self) -> &'static str {
        match self {
            TestRunner::Cargo => "cargo",
            TestRunner::Npm => "npm",
            TestRunner::Pytest => "pytest",
        }
    }

    /// Bare executable name (allowlisted by [`TestRunPolicy`]).
    #[must_use]
    pub fn program(self) -> &'static str {
        match self {
            TestRunner::Cargo => {
                if cfg!(windows) {
                    "cargo.exe"
                } else {
                    "cargo"
                }
            }
            TestRunner::Npm => {
                if cfg!(windows) {
                    "npm.cmd"
                } else {
                    "npm"
                }
            }
            TestRunner::Pytest => {
                if cfg!(windows) {
                    "python.exe"
                } else {
                    "python3"
                }
            }
        }
    }

    /// Base arguments before the caller's extra arguments.
    #[must_use]
    pub fn base_args(self) -> Vec<String> {
        match self {
            TestRunner::Cargo => vec!["test".to_owned()],
            TestRunner::Npm => vec!["test".to_owned()],
            TestRunner::Pytest => vec!["-m".to_owned(), "pytest".to_owned()],
        }
    }
}

/// Caller selection: auto-detect or force one runner.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TestSelection {
    /// Detect from the nearest marker.
    Auto,
    /// Force cargo.
    Cargo,
    /// Force npm.
    Npm,
    /// Force pytest.
    Pytest,
}

impl TestSelection {
    /// Resolves the selection to a runner (`None` for auto).
    #[must_use]
    pub fn runner(self) -> Option<TestRunner> {
        match self {
            TestSelection::Auto => None,
            TestSelection::Cargo => Some(TestRunner::Cargo),
            TestSelection::Npm => Some(TestRunner::Npm),
            TestSelection::Pytest => Some(TestRunner::Pytest),
        }
    }
}

/// Detection result.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct TestDetection {
    /// Chosen runner (the first runnable marker, else the first marker).
    pub runner: TestRunner,
    /// Workspace-relative directory holding the marker.
    pub cwd: String,
    /// Whether the chosen runner has a usable test entry point.
    pub runnable: bool,
    /// All markers found at that directory, in priority order.
    pub candidates: Vec<TestRunner>,
    /// Human-readable detection reason.
    pub reason: String,
}

/// Test execution caps. Builders clamp to the hard bounds.
#[derive(Debug, Clone)]
pub struct TestRunPolicy {
    shell: ShellPolicy,
}

impl Default for TestRunPolicy {
    fn default() -> Self {
        Self::new()
    }
}

impl TestRunPolicy {
    /// Default policy with the allowlisted runner executables.
    #[must_use]
    pub fn new() -> Self {
        Self {
            shell: ShellPolicy::new([
                TestRunner::Cargo.program().to_owned(),
                TestRunner::Npm.program().to_owned(),
                TestRunner::Pytest.program().to_owned(),
            ])
            .with_timeout(DEFAULT_TEST_TIMEOUT)
            .with_stdout_cap(DEFAULT_TEST_STDOUT_BYTES),
        }
    }

    /// Overrides the timeout, clamped to
    /// [`TEST_MIN_TIMEOUT`]..=[`TEST_MAX_TIMEOUT`].
    #[must_use]
    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.shell = self
            .shell
            .with_timeout(timeout.clamp(TEST_MIN_TIMEOUT, TEST_MAX_TIMEOUT));
        self
    }

    /// Overrides the stdout cap, clamped to 1 byte..=[`TEST_HARD_STDOUT_CAP`].
    #[must_use]
    pub fn with_stdout_cap(mut self, max_stdout_bytes: usize) -> Self {
        self.shell = self
            .shell
            .with_stdout_cap(max_stdout_bytes.clamp(1, TEST_HARD_STDOUT_CAP));
        self
    }

    /// Configured timeout (observable for tests and the ledger).
    #[must_use]
    pub fn timeout(&self) -> Duration {
        self.shell.timeout()
    }
}

/// One test-run request.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TestRunRequest {
    /// Workspace-relative working directory (defaults to the jail root).
    pub cwd: Option<String>,
    /// Auto-detect or force a runner.
    pub selection: TestSelection,
    /// Extra arguments appended to the runner's base arguments.
    pub args: Vec<String>,
    /// Optional per-call timeout override (clamped to the policy bounds).
    pub timeout: Option<Duration>,
}

impl TestRunRequest {
    /// Builds an auto-detecting request.
    #[must_use]
    pub fn auto() -> Self {
        Self {
            cwd: None,
            selection: TestSelection::Auto,
            args: Vec::new(),
            timeout: None,
        }
    }

    /// Sets the working directory.
    #[must_use]
    pub fn with_cwd(mut self, cwd: impl Into<String>) -> Self {
        self.cwd = Some(cwd.into());
        self
    }

    /// Forces a runner.
    #[must_use]
    pub fn with_selection(mut self, selection: TestSelection) -> Self {
        self.selection = selection;
        self
    }

    /// Sets extra arguments.
    #[must_use]
    pub fn with_args(mut self, args: impl IntoIterator<Item = String>) -> Self {
        self.args = args.into_iter().collect();
        self
    }

    /// Sets a timeout override.
    #[must_use]
    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = Some(timeout);
        self
    }
}

/// Result of one test run. A non-zero exit code means "tests failed" and is
/// returned as data; it is never converted into a fake success.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct TestRunOutcome {
    /// Runner that executed.
    pub runner: TestRunner,
    /// Workspace-relative working directory.
    pub cwd: String,
    /// Executed program.
    pub program: String,
    /// Executed arguments.
    pub args: Vec<String>,
    /// Exit code (`None` when terminated by a signal).
    pub exit_code: Option<i32>,
    /// Whether the process exited 0.
    pub success: bool,
    /// Capped stdout.
    pub stdout: String,
    /// Capped stderr.
    pub stderr: String,
    /// A cap truncated stdout/stderr.
    pub truncated: bool,
    /// Wall-clock duration in milliseconds.
    pub duration_ms: u128,
    /// Detection reason for an auto-detected run.
    pub detection_reason: Option<String>,
}

/// Detects the nearest supported test runner from `cwd` up to the jail root.
///
/// # Errors
/// [`Refusal::PathOutsideJail`] for an escaping cwd, [`Refusal::PathNotFound`]
/// when the cwd does not exist, [`Refusal::ToolSchema`] when it is not a
/// directory or no supported marker exists, [`Refusal::InternalError`] for I/O.
pub fn detect(jail: &WorkspaceJail, cwd: Option<&str>) -> Result<TestDetection, ToolError> {
    let start = jail.resolve(cwd.unwrap_or(""))?;
    let metadata = fs::metadata(&start).map_err(|error| {
        ToolError::new(
            Refusal::PathNotFound,
            format!("cannot inspect the test working directory: {error}"),
        )
    })?;
    if !metadata.is_dir() {
        return Err(ToolError::new(
            Refusal::ToolSchema,
            "test working directory is not a directory",
        ));
    }

    let mut directory = start;
    loop {
        if let Some(found) = detect_in(jail, &directory)? {
            return Ok(found);
        }
        if directory == jail.root() {
            break;
        }
        match directory.parent() {
            Some(parent) if parent.starts_with(jail.root()) => {
                directory = parent.to_path_buf();
            }
            _ => break,
        }
    }
    Err(ToolError::new(
        Refusal::ToolSchema,
        "no supported test runner marker (Cargo.toml, package.json scripts.test, \
         or pytest configuration) was found from the working directory up to the \
         workspace root",
    ))
}

/// Runs a test suite through the bounded shell seam.
///
/// # Errors
/// [`Refusal::ToolSchema`] for an invalid selection/argument/no runnable marker;
/// path/jail failures as [`detect`]; [`Refusal::ToolNotAllowed`] when the
/// runner executable is not allowlisted; [`Refusal::ToolTimeout`] when the
/// child exceeds the timeout; [`Refusal::InternalError`] for spawn/I/O.
pub async fn run_tests(
    jail: &WorkspaceJail,
    policy: &TestRunPolicy,
    request: &TestRunRequest,
) -> Result<TestRunOutcome, ToolError> {
    validate_extra_args(&request.args)?;
    let (runner, cwd, detection_reason) = match request.selection.runner() {
        Some(runner) => (runner, request.cwd.clone().unwrap_or_default(), None),
        None => {
            let detection = detect(jail, request.cwd.as_deref())?;
            if !detection.runnable {
                return Err(ToolError::new(
                    Refusal::ToolSchema,
                    format!(
                        "{} was detected at {:?} but it has no runnable test entry point: {}",
                        detection.runner.label(),
                        detection.cwd,
                        detection.reason
                    ),
                ));
            }
            (detection.runner, detection.cwd, Some(detection.reason))
        }
    };

    let mut args = runner.base_args();
    if runner == TestRunner::Npm && !request.args.is_empty() {
        args.push("--".to_owned());
    }
    args.extend(request.args.iter().cloned());

    let mut shell_policy = policy.shell.clone();
    if let Some(timeout) = request.timeout {
        shell_policy = shell_policy.with_timeout(timeout.clamp(TEST_MIN_TIMEOUT, TEST_MAX_TIMEOUT));
    }
    let shell_request = ShellRequest::new(runner.program(), args.clone()).with_cwd(&cwd);
    let output = shell::execute(jail, &shell_policy, &shell_request, None).await?;
    Ok(TestRunOutcome {
        runner,
        cwd,
        program: output.program,
        args,
        exit_code: output.exit_code,
        success: output.success,
        stdout: output.stdout,
        stderr: output.stderr,
        truncated: output.truncated,
        duration_ms: output.duration_ms,
        detection_reason,
    })
}

fn detect_in(jail: &WorkspaceJail, directory: &Path) -> Result<Option<TestDetection>, ToolError> {
    let relative = relative_path(jail, directory)?;
    let join = |name: &str| {
        if relative.is_empty() {
            name.to_owned()
        } else {
            format!("{relative}/{name}")
        }
    };
    let mut found: Vec<(TestRunner, bool, String)> = Vec::new();

    if directory.join("Cargo.toml").is_file() {
        found.push((
            TestRunner::Cargo,
            true,
            format!("Cargo.toml found in {:?}", display_dir(&relative)),
        ));
    }

    if directory.join("package.json").is_file() {
        let package = read_optional_text(jail, &join("package.json"))?;
        let (has_test_script, unparsable) = match package {
            Some(text) => match serde_json::from_str::<Value>(&text) {
                Ok(value) => (
                    value
                        .get("scripts")
                        .and_then(|scripts| scripts.get("test"))
                        .is_some_and(Value::is_string),
                    false,
                ),
                Err(_) => (false, true),
            },
            None => (false, false),
        };
        let reason = if has_test_script {
            format!(
                "package.json with scripts.test found in {:?}",
                display_dir(&relative)
            )
        } else if unparsable {
            format!(
                "package.json found in {:?} but it is not valid JSON",
                display_dir(&relative)
            )
        } else {
            format!(
                "package.json found in {:?} but it has no scripts.test entry",
                display_dir(&relative)
            )
        };
        found.push((TestRunner::Npm, has_test_script, reason));
    }

    if let Some(marker) = pytest_marker(jail, directory, &join)? {
        found.push((
            TestRunner::Pytest,
            true,
            format!("{marker} found in {:?}", display_dir(&relative)),
        ));
    }

    let Some((runner, runnable, reason)) = found
        .iter()
        .find(|(_, runnable, _)| *runnable)
        .cloned()
        .or_else(|| found.first().cloned())
    else {
        return Ok(None);
    };
    Ok(Some(TestDetection {
        runner,
        cwd: relative,
        runnable,
        candidates: found.iter().map(|(runner, _, _)| *runner).collect(),
        reason,
    }))
}

fn pytest_marker(
    jail: &WorkspaceJail,
    directory: &Path,
    join: &impl Fn(&str) -> String,
) -> Result<Option<String>, ToolError> {
    for name in ["pytest.ini", "pyproject.toml", "setup.cfg", "tox.ini"] {
        let path = directory.join(name);
        if !path.is_file() {
            continue;
        }
        let needle = match name {
            "pytest.ini" => return Ok(Some(name.to_owned())),
            "pyproject.toml" => "[tool.pytest",
            "setup.cfg" => "[tool:pytest]",
            "tox.ini" => "[pytest]",
            _ => unreachable!("fixed marker list"),
        };
        if let Some(text) = read_optional_text(jail, &join(name))? {
            if text.contains(needle) {
                return Ok(Some(name.to_owned()));
            }
        }
    }
    if directory.join("tests").is_dir() {
        return Ok(Some("tests/ directory".to_owned()));
    }
    Ok(None)
}

fn read_optional_text(jail: &WorkspaceJail, relative: &str) -> Result<Option<String>, ToolError> {
    match jail.read(relative) {
        Ok(bytes) => Ok(String::from_utf8(bytes).ok()),
        Err(error)
            if matches!(
                error.refusal(),
                Refusal::PathNotFound | Refusal::FileTooLarge
            ) =>
        {
            Ok(None)
        }
        Err(error) => Err(error),
    }
}

fn relative_path(jail: &WorkspaceJail, absolute: &Path) -> Result<String, ToolError> {
    let relative = absolute.strip_prefix(jail.root()).map_err(|_| {
        ToolError::new(
            Refusal::PathOutsideJail,
            "test working directory is outside the workspace root",
        )
    })?;
    let text = relative
        .to_str()
        .ok_or_else(|| ToolError::new(Refusal::ToolSchema, "workspace path is not valid UTF-8"))?;
    Ok(text.replace('\\', "/"))
}

fn display_dir(relative: &str) -> &str {
    if relative.is_empty() {
        "<workspace root>"
    } else {
        relative
    }
}

fn validate_extra_args(args: &[String]) -> Result<(), ToolError> {
    if args.len() > MAX_TEST_ARGS {
        return Err(ToolError::new(
            Refusal::ToolSchema,
            format!(
                "test runner has {} extra arguments; the cap is {MAX_TEST_ARGS}",
                args.len()
            ),
        ));
    }
    let mut total = 0usize;
    for argument in args {
        if argument.contains('\0') {
            return Err(ToolError::new(
                Refusal::ToolSchema,
                "test argument must not contain a NUL byte",
            ));
        }
        if argument.len() > MAX_TEST_ARG_BYTES {
            return Err(ToolError::new(
                Refusal::ToolSchema,
                format!(
                    "test argument is {} bytes; the cap is {MAX_TEST_ARG_BYTES}",
                    argument.len()
                ),
            ));
        }
        total += argument.len();
    }
    if total > MAX_TEST_ARGV_BYTES {
        return Err(ToolError::new(
            Refusal::ToolSchema,
            format!("test arguments total {total} bytes; the cap is {MAX_TEST_ARGV_BYTES}"),
        ));
    }
    // The shared shell guard refuses interpreter/escape forms before spawn. It
    // runs again inside `shell::execute`; doing it here keeps the refusal before
    // any policy/allowlist check and gives the same stable code.
    shell::validate_arguments(args)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn jail() -> (tempfile::TempDir, WorkspaceJail) {
        let temp = tempfile::tempdir().expect("tempdir");
        let jail = WorkspaceJail::open(temp.path()).expect("jail");
        (temp, jail)
    }

    fn cargo_project(jail: &WorkspaceJail, body: &str) {
        fs::create_dir_all(jail.root().join("src")).expect("src");
        fs::write(
            jail.root().join("Cargo.toml"),
            "[package]\nname = \"cap3-probe\"\nversion = \"0.1.0\"\nedition = \"2021\"\n",
        )
        .expect("manifest");
        fs::write(jail.root().join("src/lib.rs"), body).expect("lib");
    }

    #[test]
    fn detects_cargo_npm_and_pytest_markers() {
        let (_temp, cargo_jail) = jail();
        fs::write(cargo_jail.root().join("Cargo.toml"), "[package]\n").expect("cargo");
        let detection = detect(&cargo_jail, None).expect("cargo");
        assert_eq!(detection.runner, TestRunner::Cargo);
        assert!(detection.runnable);
        assert_eq!(detection.candidates, [TestRunner::Cargo]);

        let (_temp, npm_jail) = jail();
        fs::write(
            npm_jail.root().join("package.json"),
            r#"{"scripts":{"test":"node -e \"process.exit(0)\""}}"#,
        )
        .expect("package");
        let detection = detect(&npm_jail, None).expect("npm");
        assert_eq!(detection.runner, TestRunner::Npm);
        assert!(detection.runnable);

        let (_temp, pytest_jail) = jail();
        fs::write(
            pytest_jail.root().join("pyproject.toml"),
            "[tool.pytest.ini_options]\naddopts = \"-q\"\n",
        )
        .expect("pyproject");
        let detection = detect(&pytest_jail, None).expect("pytest");
        assert_eq!(detection.runner, TestRunner::Pytest);
        assert!(detection.runnable);
    }

    #[test]
    fn detection_prefers_cargo_and_walks_up_from_a_subdirectory() {
        let (_temp, jail) = jail();
        fs::write(jail.root().join("Cargo.toml"), "[package]\n").expect("cargo");
        fs::write(
            jail.root().join("package.json"),
            r#"{"scripts":{"test":"true"}}"#,
        )
        .expect("package");
        fs::create_dir_all(jail.root().join("crates/inner")).expect("inner");
        let detection = detect(&jail, Some("crates/inner")).expect("detect");
        assert_eq!(detection.runner, TestRunner::Cargo);
        assert_eq!(detection.cwd, "");
        assert_eq!(detection.candidates, [TestRunner::Cargo, TestRunner::Npm]);
    }

    #[test]
    fn npm_without_a_test_script_is_detected_but_not_runnable() {
        let (_temp, jail) = jail();
        fs::write(jail.root().join("package.json"), r#"{"scripts":{}}"#).expect("package");
        let detection = detect(&jail, None).expect("npm");
        assert_eq!(detection.runner, TestRunner::Npm);
        assert!(!detection.runnable);
        assert!(detection.reason.contains("no scripts.test"));
    }

    #[test]
    fn no_marker_is_a_named_refusal() {
        let (_temp, jail) = jail();
        let error = detect(&jail, None).expect_err("no marker");
        assert_eq!(error.code(), "E_TOOL_SCHEMA");
    }

    #[test]
    fn detection_cwd_escape_is_refused() {
        let (_temp, jail) = jail();
        let error = detect(&jail, Some("../outside")).expect_err("escape");
        assert_eq!(error.code(), "E_PATH_OUTSIDE_JAIL");
    }

    #[test]
    fn policy_builders_clamp_to_hard_bounds() {
        let policy = TestRunPolicy::new()
            .with_timeout(Duration::from_millis(1))
            .with_stdout_cap(0);
        assert_eq!(policy.timeout(), TEST_MIN_TIMEOUT);
        let policy = TestRunPolicy::new()
            .with_timeout(Duration::from_secs(10_000))
            .with_stdout_cap(usize::MAX);
        assert_eq!(policy.timeout(), TEST_MAX_TIMEOUT);
        assert_eq!(TestRunPolicy::new().timeout(), DEFAULT_TEST_TIMEOUT);
    }

    #[test]
    fn extra_argument_bounds_are_enforced() {
        let too_many = vec!["--quiet".to_owned(); MAX_TEST_ARGS + 1];
        assert_eq!(
            validate_extra_args(&too_many).expect_err("too many").code(),
            "E_TOOL_SCHEMA"
        );
        let huge = vec!["x".repeat(MAX_TEST_ARG_BYTES + 1)];
        assert_eq!(
            validate_extra_args(&huge).expect_err("huge").code(),
            "E_TOOL_SCHEMA"
        );
        let escape = vec!["a&b".to_owned()];
        assert_eq!(
            validate_extra_args(&escape).expect_err("escape").code(),
            "E_TOOL_NOT_ALLOWED"
        );
    }

    #[tokio::test]
    async fn runs_a_real_cargo_suite_with_a_bounded_output_cap() {
        let (_temp, jail) = jail();
        cargo_project(
            &jail,
            "#[cfg(test)]\nmod tests { #[test] fn passes() { assert_eq!(2 + 2, 4); } }\n",
        );
        let request = TestRunRequest::auto();
        let outcome = run_tests(&jail, &TestRunPolicy::new(), &request)
            .await
            .expect("cargo test runs");
        assert_eq!(outcome.runner, TestRunner::Cargo);
        assert!(
            outcome.success,
            "cargo test failed: exit {:?}\nstdout: {}\nstderr: {}",
            outcome.exit_code, outcome.stdout, outcome.stderr
        );

        let capped = TestRunPolicy::new().with_stdout_cap(16);
        let outcome = run_tests(&jail, &capped, &request)
            .await
            .expect("cargo test runs capped");
        assert!(outcome.truncated);
        assert!(outcome.stdout.len() <= 16);
    }

    #[tokio::test]
    async fn timeout_terminates_the_test_process() {
        let (_temp, jail) = jail();
        cargo_project(
            &jail,
            "#[cfg(test)]\nmod tests { #[test] fn slow() { \
             std::thread::sleep(std::time::Duration::from_secs(30)); } }\n",
        );
        let request = TestRunRequest::auto().with_timeout(Duration::from_secs(1));
        let error = run_tests(&jail, &TestRunPolicy::new(), &request)
            .await
            .expect_err("timeout");
        assert_eq!(error.code(), "E_TOOL_TIMEOUT");
    }

    #[tokio::test]
    async fn auto_run_refuses_an_unrunnable_marker_without_spawning() {
        let (_temp, jail) = jail();
        fs::write(jail.root().join("package.json"), r#"{"scripts":{}}"#).expect("package");
        let error = run_tests(&jail, &TestRunPolicy::new(), &TestRunRequest::auto())
            .await
            .expect_err("not runnable");
        assert_eq!(error.code(), "E_TOOL_SCHEMA");
        assert!(error.detail().contains("no runnable test entry point"));
    }

    #[test]
    fn explicit_selection_maps_to_the_expected_command() {
        assert_eq!(TestSelection::Cargo.runner(), Some(TestRunner::Cargo));
        assert_eq!(TestSelection::Npm.runner(), Some(TestRunner::Npm));
        assert_eq!(TestSelection::Pytest.runner(), Some(TestRunner::Pytest));
        assert_eq!(TestRunner::Cargo.base_args(), ["test"]);
        assert_eq!(TestRunner::Pytest.base_args(), ["-m", "pytest"]);
    }
}
