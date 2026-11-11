//! Bounded Git wrappers for tools and the agent loop.
//!
//! Every call goes through [`crate::shell::execute`], so it inherits the jail
//! working-directory check, the `git.exe` allowlist, the sanitized child
//! environment, the stdout/stderr caps, the timeout and the kill-on-close Job
//! Object. Arguments are passed directly to `git.exe` (never through a shell)
//! and every path argument is resolved through [`WorkspaceJail`] before the
//! child starts.
//!
//! Read helpers use `--no-optional-locks` so status/diff/log do not refresh or
//! lock the index. Every call also passes `-c core.fsmonitor=false` and
//! `-c core.hooksPath=` so a repository configuration cannot make Git run the
//! fsmonitor hook or any other hook. Diff uses `--no-ext-diff` and
//! `--no-textconv` so it cannot become an external program execution. Commit
//! uses `--no-verify` and `--no-gpg-sign` for the same reason: hooks and
//! signing helpers are repository/user-controlled programs. This is a
//! deliberate capability reduction, documented rather than implicit.
//!
//! Argument note: Git is launched directly (never through a shell), but the
//! shared [`crate::shell`] guard still refuses arguments containing `/c`, `/k`,
//! `&`, `|`, `>` or CR/LF. That applies to commit messages too, so a commit
//! message is one line without those characters.
//!
//! A non-zero Git exit code is **data**, not a refusal: the outcome carries
//! `success`, `exit_code` and bounded `stderr` so a caller can report "not a
//! repository" or "nothing to commit" without a fabricated success. Only spawn,
//! timeout, jail and I/O failures are [`ToolError`]s.

use std::path::{Component, Path};
use std::time::Duration;

use hiverouter_contract::Refusal;
use serde::Serialize;

use crate::error::ToolError;
use crate::jail::WorkspaceJail;
use crate::shell::{self, ShellOutput, ShellPolicy, ShellRequest};

/// Default Git timeout (30 s).
pub const GIT_TIMEOUT: Duration = Duration::from_secs(30);
/// Minimum configurable Git timeout.
pub const GIT_MIN_TIMEOUT: Duration = Duration::from_secs(1);
/// Maximum configurable Git timeout (5 min).
pub const GIT_MAX_TIMEOUT: Duration = Duration::from_secs(300);
/// Default stdout cap (1 MiB).
pub const GIT_MAX_STDOUT_BYTES: usize = 1024 * 1024;
/// Maximum configurable stdout cap (8 MiB).
pub const GIT_HARD_STDOUT_CAP: usize = 8 * 1024 * 1024;
/// Maximum entries returned by one `log` call.
pub const MAX_GIT_LOG_ENTRIES: usize = 100;
/// Maximum paths staged in one call.
pub const MAX_GIT_STAGE_PATHS: usize = 256;
/// Maximum bytes of one repository-relative path argument.
pub const MAX_GIT_PATH_BYTES: usize = 4096;
/// Maximum commit message bytes (single line).
pub const MAX_GIT_COMMIT_MESSAGE_BYTES: usize = 8192;

/// Git execution caps. Constructed from safe defaults; builders clamp to the
/// hard bounds so a caller cannot disable the timeout or the output cap.
#[derive(Debug, Clone)]
pub struct GitPolicy {
    shell: ShellPolicy,
}

impl Default for GitPolicy {
    fn default() -> Self {
        Self::new()
    }
}

impl GitPolicy {
    /// Builds the default policy: `git.exe` only, 30 s timeout, 1 MiB stdout.
    #[must_use]
    pub fn new() -> Self {
        Self {
            shell: ShellPolicy::new(["git.exe".to_owned()])
                .with_timeout(GIT_TIMEOUT)
                .with_stdout_cap(GIT_MAX_STDOUT_BYTES),
        }
    }

    /// Overrides the timeout, clamped to [`GIT_MIN_TIMEOUT`]..=[`GIT_MAX_TIMEOUT`].
    #[must_use]
    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.shell = self
            .shell
            .with_timeout(timeout.clamp(GIT_MIN_TIMEOUT, GIT_MAX_TIMEOUT));
        self
    }

    /// Overrides the stdout cap, clamped to 1 byte..=[`GIT_HARD_STDOUT_CAP`].
    #[must_use]
    pub fn with_stdout_cap(mut self, max_stdout_bytes: usize) -> Self {
        self.shell = self
            .shell
            .with_stdout_cap(max_stdout_bytes.clamp(1, GIT_HARD_STDOUT_CAP));
        self
    }

    /// Configured timeout (observable for tests and the ledger).
    #[must_use]
    pub fn timeout(&self) -> Duration {
        self.shell.timeout()
    }
}

/// One status entry parsed from `git status --porcelain=v1 -z`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct GitStatusEntry {
    /// Index (staged) status character (` `, `M`, `A`, `D`, `R`, `C`, `?`, ...).
    pub index: String,
    /// Worktree status character.
    pub worktree: String,
    /// Workspace-relative path.
    pub path: String,
    /// Original path for a rename/copy record.
    pub original_path: Option<String>,
}

/// Result of `git status`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct GitStatus {
    /// Parsed entries (empty when the command failed or output was unusable).
    pub entries: Vec<GitStatusEntry>,
    /// Capped raw porcelain output (`\0`-separated).
    pub raw: String,
    /// The stdout cap truncated the output.
    pub truncated: bool,
    /// Whether Git exited 0.
    pub success: bool,
    /// Git exit code (`None` when terminated by a signal).
    pub exit_code: Option<i32>,
    /// Capped stderr.
    pub stderr: String,
}

/// Request for [`Git::diff`].
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct GitDiffRequest {
    /// Workspace-relative working directory (defaults to the jail root).
    pub cwd: Option<String>,
    /// Show the staged diff instead of the worktree diff.
    pub staged: bool,
    /// Optional repository-relative path filter (literal pathspec).
    pub path: Option<String>,
}

/// Result of `git diff`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct GitDiff {
    /// Unified diff text (capped).
    pub text: String,
    /// The stdout cap truncated the diff.
    pub truncated: bool,
    /// Whether Git exited 0.
    pub success: bool,
    /// Git exit code.
    pub exit_code: Option<i32>,
    /// Capped stderr.
    pub stderr: String,
}

/// Request for [`Git::log`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GitLogRequest {
    /// Workspace-relative working directory.
    pub cwd: Option<String>,
    /// Maximum entries (clamped to 1..=[`MAX_GIT_LOG_ENTRIES`]).
    pub limit: usize,
}

impl Default for GitLogRequest {
    fn default() -> Self {
        Self {
            cwd: None,
            limit: 20,
        }
    }
}

/// One parsed `git log` entry.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct GitLogEntry {
    /// Full commit hash.
    pub hash: String,
    /// Author name.
    pub author: String,
    /// Author date (ISO 8601 strict).
    pub date: String,
    /// Commit subject.
    pub subject: String,
}

/// Result of `git log`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct GitLog {
    /// Parsed entries.
    pub entries: Vec<GitLogEntry>,
    /// The stdout cap truncated the output.
    pub truncated: bool,
    /// Whether Git exited 0.
    pub success: bool,
    /// Git exit code.
    pub exit_code: Option<i32>,
    /// Capped stderr.
    pub stderr: String,
}

/// Request for [`Git::stage`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GitStageRequest {
    /// Workspace-relative working directory.
    pub cwd: Option<String>,
    /// Repository-relative paths to add.
    pub paths: Vec<String>,
    /// Stage all tracked and untracked changes (`git add --all`).
    pub all: bool,
}

impl GitStageRequest {
    /// Stages exactly these paths.
    #[must_use]
    pub fn paths(paths: impl IntoIterator<Item = String>) -> Self {
        Self {
            cwd: None,
            paths: paths.into_iter().collect(),
            all: false,
        }
    }

    /// Stages every change.
    #[must_use]
    pub fn all() -> Self {
        Self {
            cwd: None,
            paths: Vec::new(),
            all: true,
        }
    }
}

/// Result of `git add`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct GitStageOutcome {
    /// Whether Git exited 0.
    pub success: bool,
    /// Git exit code.
    pub exit_code: Option<i32>,
    /// Capped stdout.
    pub stdout: String,
    /// Capped stderr.
    pub stderr: String,
    /// The caps truncated output.
    pub truncated: bool,
}

/// Optional commit identity; passed as `-c user.name` / `-c user.email` so a
/// commit does not depend on global Git configuration.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GitAuthor {
    /// Author name.
    pub name: String,
    /// Author email.
    pub email: String,
}

/// Request for [`Git::commit`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GitCommitRequest {
    /// Workspace-relative working directory.
    pub cwd: Option<String>,
    /// Single-line commit message.
    pub message: String,
    /// Optional explicit identity.
    pub author: Option<GitAuthor>,
}

impl GitCommitRequest {
    /// Builds a commit request for one message.
    #[must_use]
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            cwd: None,
            message: message.into(),
            author: None,
        }
    }

    /// Sets the explicit identity.
    #[must_use]
    pub fn with_author(mut self, name: impl Into<String>, email: impl Into<String>) -> Self {
        self.author = Some(GitAuthor {
            name: name.into(),
            email: email.into(),
        });
        self
    }
}

/// Result of `git commit`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct GitCommitOutcome {
    /// Whether Git exited 0.
    pub success: bool,
    /// Git exit code.
    pub exit_code: Option<i32>,
    /// Commit hash from `git rev-parse HEAD` when the commit succeeded.
    pub hash: Option<String>,
    /// Capped commit stdout.
    pub stdout: String,
    /// Capped stderr.
    pub stderr: String,
    /// The caps truncated output.
    pub truncated: bool,
}

/// A Git facade bound to one workspace jail and one policy.
#[derive(Debug, Clone)]
pub struct Git<'a> {
    jail: &'a WorkspaceJail,
    policy: GitPolicy,
}

impl<'a> Git<'a> {
    /// Builds a facade with the default policy.
    #[must_use]
    pub fn new(jail: &'a WorkspaceJail) -> Self {
        Self {
            jail,
            policy: GitPolicy::new(),
        }
    }

    /// Builds a facade with an explicit policy.
    #[must_use]
    pub fn with_policy(jail: &'a WorkspaceJail, policy: GitPolicy) -> Self {
        Self { jail, policy }
    }

    /// The configured policy (tests/ledger).
    #[must_use]
    pub fn policy(&self) -> &GitPolicy {
        &self.policy
    }

    /// Runs `git status --porcelain=v1 -z` in `cwd`.
    ///
    /// # Errors
    /// [`Refusal::PathOutsideJail`] for an escaping cwd, [`Refusal::ToolTimeout`]
    /// when the child exceeds the policy timeout, [`Refusal::InternalError`] for
    /// spawn/I/O failures.
    pub async fn status(&self, cwd: Option<&str>) -> Result<GitStatus, ToolError> {
        let output = self
            .run(
                cwd,
                vec![
                    "--no-optional-locks".to_owned(),
                    "status".to_owned(),
                    "--porcelain=v1".to_owned(),
                    "-z".to_owned(),
                    "--untracked-files=all".to_owned(),
                ],
            )
            .await?;
        let mut raw = output.stdout;
        if output.truncated && !raw.ends_with('\0') {
            // Drop the final fragment so a truncated record is never parsed.
            if let Some(last) = raw.rfind('\0') {
                raw.truncate(last + 1);
            } else {
                raw.clear();
            }
        }
        let entries = parse_status(&raw);
        Ok(GitStatus {
            entries,
            raw,
            truncated: output.truncated,
            success: output.success,
            exit_code: output.exit_code,
            stderr: output.stderr,
        })
    }

    /// Runs `git diff` (worktree or staged) in `cwd`.
    ///
    /// # Errors
    /// Path/jail, timeout and spawn failures as [`Git::status`]; an invalid path
    /// filter is [`Refusal::ToolSchema`].
    pub async fn diff(&self, request: &GitDiffRequest) -> Result<GitDiff, ToolError> {
        let mut args = vec![
            "--no-optional-locks".to_owned(),
            "diff".to_owned(),
            "--no-color".to_owned(),
            "--no-ext-diff".to_owned(),
            "--no-textconv".to_owned(),
        ];
        if request.staged {
            args.push("--staged".to_owned());
        }
        if let Some(path) = request.path.as_deref() {
            validate_repo_path(self.jail, request.cwd.as_deref(), path)?;
            args.push("--".to_owned());
            args.push(path.to_owned());
        }
        let output = self.run(request.cwd.as_deref(), args).await?;
        Ok(GitDiff {
            text: output.stdout,
            truncated: output.truncated,
            success: output.success,
            exit_code: output.exit_code,
            stderr: output.stderr,
        })
    }

    /// Runs `git log` with a bounded entry count in `cwd`.
    ///
    /// # Errors
    /// Path/jail, timeout and spawn failures as [`Git::status`].
    pub async fn log(&self, request: &GitLogRequest) -> Result<GitLog, ToolError> {
        let limit = request.limit.clamp(1, MAX_GIT_LOG_ENTRIES);
        let output = self
            .run(
                request.cwd.as_deref(),
                vec![
                    "--no-optional-locks".to_owned(),
                    "log".to_owned(),
                    "--no-color".to_owned(),
                    format!("-n{limit}"),
                    "--format=%H%x1f%an%x1f%aI%x1f%s".to_owned(),
                ],
            )
            .await?;
        let entries = parse_log(&output.stdout);
        Ok(GitLog {
            entries,
            truncated: output.truncated,
            success: output.success,
            exit_code: output.exit_code,
            stderr: output.stderr,
        })
    }

    /// Runs `git add -- <paths>` or `git add --all` in `cwd`.
    ///
    /// # Errors
    /// [`Refusal::ToolSchema`] for an empty/ambiguous selection or an invalid
    /// path; path/jail, timeout and spawn failures as [`Git::status`].
    pub async fn stage(&self, request: &GitStageRequest) -> Result<GitStageOutcome, ToolError> {
        if request.all && !request.paths.is_empty() {
            return Err(ToolError::new(
                Refusal::ToolSchema,
                "git stage takes either `all` or explicit paths, not both",
            ));
        }
        if !request.all && request.paths.is_empty() {
            return Err(ToolError::new(
                Refusal::ToolSchema,
                "git stage needs `all: true` or at least one path",
            ));
        }
        if request.paths.len() > MAX_GIT_STAGE_PATHS {
            return Err(ToolError::new(
                Refusal::ToolSchema,
                format!(
                    "git stage has {} paths; the cap is {MAX_GIT_STAGE_PATHS}",
                    request.paths.len()
                ),
            ));
        }
        let mut args = vec![
            "--no-optional-locks".to_owned(),
            "--literal-pathspecs".to_owned(),
            "add".to_owned(),
        ];
        if request.all {
            args.push("--all".to_owned());
        } else {
            args.push("--".to_owned());
            for path in &request.paths {
                validate_repo_path(self.jail, request.cwd.as_deref(), path)?;
                args.push(path.clone());
            }
        }
        let output = self.run(request.cwd.as_deref(), args).await?;
        Ok(GitStageOutcome {
            success: output.success,
            exit_code: output.exit_code,
            stdout: output.stdout,
            stderr: output.stderr,
            truncated: output.truncated,
        })
    }

    /// Runs `git commit --no-verify --no-gpg-sign -m <message>` in `cwd`.
    ///
    /// A non-zero exit (for example "nothing to commit") is returned as an
    /// `Ok` [`GitCommitOutcome`] with `success: false`; it is not faked into an
    /// error or a success.
    ///
    /// # Errors
    /// [`Refusal::ToolSchema`] for an empty/multi-line/oversize message or an
    /// invalid identity; path/jail, timeout and spawn failures as [`Git::status`].
    pub async fn commit(&self, request: &GitCommitRequest) -> Result<GitCommitOutcome, ToolError> {
        let message = validate_commit_message(&request.message)?;
        let mut args = vec!["--no-optional-locks".to_owned()];
        if let Some(author) = &request.author {
            let name = validate_author_field("author.name", &author.name)?;
            let email = validate_author_field("author.email", &author.email)?;
            args.push("-c".to_owned());
            args.push(format!("user.name={name}"));
            args.push("-c".to_owned());
            args.push(format!("user.email={email}"));
        }
        args.push("commit".to_owned());
        args.push("--no-verify".to_owned());
        args.push("--no-gpg-sign".to_owned());
        args.push("-m".to_owned());
        args.push(message);

        let output = self.run(request.cwd.as_deref(), args).await?;
        let mut hash = None;
        if output.success {
            let rev = self
                .run(
                    request.cwd.as_deref(),
                    vec![
                        "--no-optional-locks".to_owned(),
                        "rev-parse".to_owned(),
                        "HEAD".to_owned(),
                    ],
                )
                .await?;
            if rev.success {
                hash = rev
                    .stdout
                    .lines()
                    .next()
                    .map(str::trim)
                    .map(ToOwned::to_owned);
                if hash.as_deref() == Some("") {
                    hash = None;
                }
            }
        }
        Ok(GitCommitOutcome {
            success: output.success,
            exit_code: output.exit_code,
            hash,
            stdout: output.stdout,
            stderr: output.stderr,
            truncated: output.truncated,
        })
    }

    async fn run(&self, cwd: Option<&str>, args: Vec<String>) -> Result<ShellOutput, ToolError> {
        // Global safety overrides precede the subcommand: no fsmonitor hook and
        // no hooks path. `args` already carries `--no-optional-locks` and the
        // per-command hardening flags.
        let mut hardened = Vec::with_capacity(args.len() + 4);
        hardened.push("-c".to_owned());
        hardened.push("core.fsmonitor=false".to_owned());
        hardened.push("-c".to_owned());
        hardened.push("core.hooksPath=".to_owned());
        hardened.extend(args);
        let mut request = ShellRequest::new("git.exe", hardened);
        if let Some(cwd) = cwd {
            request = request.with_cwd(cwd);
        }
        shell::execute(self.jail, &self.policy.shell, &request, None).await
    }
}

fn parse_status(raw: &str) -> Vec<GitStatusEntry> {
    let mut records: Vec<&str> = raw.split('\0').collect();
    if raw.ends_with('\0') {
        records.pop();
    }
    let mut entries = Vec::new();
    let mut index = 0usize;
    while index < records.len() {
        let record = records[index];
        let mut chars = record.chars();
        let (Some(index_status), Some(worktree), Some(separator)) =
            (chars.next(), chars.next(), chars.next())
        else {
            index += 1;
            continue;
        };
        if separator != ' ' {
            index += 1;
            continue;
        }
        let path = chars.as_str().to_owned();
        if path.is_empty() {
            index += 1;
            continue;
        }
        let mut original_path = None;
        if matches!(index_status, 'R' | 'C') || matches!(worktree, 'R' | 'C') {
            if let Some(original) = records.get(index + 1) {
                if !original.is_empty() {
                    original_path = Some((*original).to_owned());
                    index += 1;
                }
            }
        }
        entries.push(GitStatusEntry {
            index: index_status.to_string(),
            worktree: worktree.to_string(),
            path,
            original_path,
        });
        index += 1;
    }
    entries
}

fn parse_log(raw: &str) -> Vec<GitLogEntry> {
    let mut entries = Vec::new();
    for line in raw.lines() {
        let fields: Vec<&str> = line.split('\x1f').collect();
        if fields.len() != 4 {
            continue;
        }
        entries.push(GitLogEntry {
            hash: fields[0].to_owned(),
            author: fields[1].to_owned(),
            date: fields[2].to_owned(),
            subject: fields[3].to_owned(),
        });
    }
    entries
}

fn validate_repo_path(
    jail: &WorkspaceJail,
    cwd: Option<&str>,
    path: &str,
) -> Result<(), ToolError> {
    if path.is_empty() {
        return Err(ToolError::new(
            Refusal::ToolSchema,
            "git path argument must not be empty",
        ));
    }
    if path.len() > MAX_GIT_PATH_BYTES {
        return Err(ToolError::new(
            Refusal::ToolSchema,
            format!(
                "git path is {} bytes; the cap is {MAX_GIT_PATH_BYTES}",
                path.len()
            ),
        ));
    }
    if path.contains('\0') {
        return Err(ToolError::new(
            Refusal::ToolSchema,
            "git path argument must not contain a NUL byte",
        ));
    }
    let candidate = Path::new(path);
    if candidate.is_absolute() {
        return Err(ToolError::new(
            Refusal::ToolSchema,
            "git path must be repository-relative, not absolute",
        ));
    }
    for component in candidate.components() {
        match component {
            Component::Normal(_) | Component::CurDir => {}
            Component::ParentDir => {
                return Err(ToolError::new(
                    Refusal::ToolSchema,
                    "git path must not contain `..`",
                ));
            }
            Component::Prefix(_) | Component::RootDir => {
                return Err(ToolError::new(
                    Refusal::ToolSchema,
                    "git path must be repository-relative",
                ));
            }
        }
    }
    let joined = match cwd {
        Some(cwd) if !cwd.is_empty() => format!("{cwd}/{path}"),
        _ => path.to_owned(),
    };
    // The jail owns containment and denied-subtree checks; this also refuses a
    // cwd that escapes.
    jail.resolve(&joined)?;
    Ok(())
}

fn validate_commit_message(message: &str) -> Result<String, ToolError> {
    if message.trim().is_empty() {
        return Err(ToolError::new(
            Refusal::ToolSchema,
            "commit message must not be empty",
        ));
    }
    if message.len() > MAX_GIT_COMMIT_MESSAGE_BYTES {
        return Err(ToolError::new(
            Refusal::ToolSchema,
            format!(
                "commit message is {} bytes; the cap is {MAX_GIT_COMMIT_MESSAGE_BYTES}",
                message.len()
            ),
        ));
    }
    if message.contains('\0') || message.contains('\n') || message.contains('\r') {
        return Err(ToolError::new(
            Refusal::ToolSchema,
            "commit message must be one line without NUL or CR/LF; the shared shell guard \
             refuses those argument forms even though git is launched without a shell",
        ));
    }
    Ok(message.to_owned())
}

fn validate_author_field(field: &str, value: &str) -> Result<String, ToolError> {
    let value = value.trim();
    if value.is_empty() {
        return Err(ToolError::new(
            Refusal::ToolSchema,
            format!("git {field} must not be empty"),
        ));
    }
    if value.len() > 256 {
        return Err(ToolError::new(
            Refusal::ToolSchema,
            format!("git {field} is {} bytes; the cap is 256", value.len()),
        ));
    }
    if value.contains('\0')
        || value.contains('\n')
        || value.contains('\r')
        || value.contains('&')
        || value.contains('|')
        || value.contains('>')
    {
        return Err(ToolError::new(
            Refusal::ToolSchema,
            format!("git {field} contains a control character or shell metacharacter"),
        ));
    }
    Ok(value.to_owned())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path as StdPath;
    use std::process::Command;

    fn jail() -> (tempfile::TempDir, WorkspaceJail) {
        let temp = tempfile::tempdir().expect("tempdir");
        let jail = WorkspaceJail::open(temp.path()).expect("jail");
        (temp, jail)
    }

    fn run_git(directory: &StdPath, args: &[&str]) -> std::process::Output {
        Command::new("git")
            .args(args)
            .current_dir(directory)
            .output()
            .expect("run git")
    }

    fn init_repo(jail: &WorkspaceJail) {
        let output = run_git(jail.root(), &["init", "-q", "-b", "main"]);
        assert!(
            output.status.success(),
            "git init failed: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }

    fn author() -> GitAuthor {
        GitAuthor {
            name: "Hive Test".to_owned(),
            email: "hive@example.invalid".to_owned(),
        }
    }

    #[tokio::test]
    async fn status_stage_commit_log_round_trip() {
        let (_temp, jail) = jail();
        init_repo(&jail);
        std::fs::write(jail.root().join("note.txt"), b"hello\n").expect("seed");
        let git = Git::new(&jail);

        let status = git.status(None).await.expect("status");
        assert!(status.success, "stderr: {}", status.stderr);
        assert_eq!(status.entries.len(), 1);
        assert_eq!(status.entries[0].index, "?");
        assert_eq!(status.entries[0].worktree, "?");
        assert_eq!(status.entries[0].path, "note.txt");

        let staged = git
            .stage(&GitStageRequest::paths(["note.txt".to_owned()]))
            .await
            .expect("stage");
        assert!(staged.success, "stderr: {}", staged.stderr);
        let status = git.status(None).await.expect("status");
        assert_eq!(status.entries[0].index, "A");

        let mut request = GitCommitRequest::new("add note.txt");
        request.author = Some(author());
        let commit = git.commit(&request).await.expect("commit");
        assert!(commit.success, "stderr: {}", commit.stderr);
        let hash = commit.hash.expect("commit hash");
        assert_eq!(hash.len(), 40, "unexpected hash: {hash}");

        let log = git
            .log(&GitLogRequest {
                limit: 1,
                ..GitLogRequest::default()
            })
            .await
            .expect("log");
        assert!(log.success, "stderr: {}", log.stderr);
        assert_eq!(log.entries.len(), 1);
        assert_eq!(log.entries[0].hash, hash);
        assert_eq!(log.entries[0].subject, "add note.txt");
        assert_eq!(log.entries[0].author, "Hive Test");
    }

    #[tokio::test]
    async fn diff_returns_unified_text_and_honours_caps() {
        let (_temp, jail) = jail();
        init_repo(&jail);
        std::fs::write(jail.root().join("note.txt"), b"hello\n").expect("seed");
        let git = Git::new(&jail);
        git.stage(&GitStageRequest::paths(["note.txt".to_owned()]))
            .await
            .expect("stage");
        let mut request = GitCommitRequest::new("seed");
        request.author = Some(author());
        assert!(git.commit(&request).await.expect("commit").success);
        std::fs::write(jail.root().join("note.txt"), b"hello world\n").expect("edit");

        let diff = git.diff(&GitDiffRequest::default()).await.expect("diff");
        assert!(diff.success, "stderr: {}", diff.stderr);
        assert!(diff.text.contains("-hello"));
        assert!(diff.text.contains("+hello world"));

        let capped = Git::with_policy(&jail, GitPolicy::new().with_stdout_cap(8));
        let diff = capped
            .diff(&GitDiffRequest::default())
            .await
            .expect("capped diff");
        assert!(diff.truncated);
        assert!(diff.text.len() <= 8);
    }

    #[tokio::test]
    async fn commit_message_is_passed_literally_without_shell_interpolation() {
        let (_temp, jail) = jail();
        init_repo(&jail);
        std::fs::write(jail.root().join("note.txt"), b"x\n").expect("seed");
        let git = Git::new(&jail);
        git.stage(&GitStageRequest::all()).await.expect("stage");
        let literal = "literal ; $(not-a-command) `also-not`";
        let mut request = GitCommitRequest::new(literal);
        request.author = Some(author());
        let commit = git.commit(&request).await.expect("commit");
        assert!(commit.success, "stderr: {}", commit.stderr);
        let log = git
            .log(&GitLogRequest {
                limit: 1,
                ..GitLogRequest::default()
            })
            .await
            .expect("log");
        assert_eq!(log.entries[0].subject, literal);
    }

    #[tokio::test]
    async fn non_repository_and_nothing_to_commit_are_results_not_fakes() {
        let (_temp, jail) = jail();
        let git = Git::new(&jail);
        let status = git.status(None).await.expect("status runs");
        assert!(!status.success);
        assert!(status.entries.is_empty());
        assert!(!status.stderr.is_empty());

        init_repo(&jail);
        let mut request = GitCommitRequest::new("empty");
        request.author = Some(author());
        let commit = git.commit(&request).await.expect("commit runs");
        assert!(!commit.success);
        assert!(commit.hash.is_none());
    }

    #[tokio::test]
    async fn escaping_paths_and_cwd_are_refused_before_git_runs() {
        let (_temp, jail) = jail();
        init_repo(&jail);
        let git = Git::new(&jail);
        for path in [
            "../outside.txt",
            "C:\\Windows\\win.ini",
            "/etc/passwd",
            "a\0b",
        ] {
            let error = git
                .stage(&GitStageRequest::paths([path.to_owned()]))
                .await
                .expect_err(path);
            assert!(
                error.code() == "E_TOOL_SCHEMA" || error.code() == "E_PATH_OUTSIDE_JAIL",
                "{path}: {}",
                error.code()
            );
        }
        let status = git
            .status(Some("../outside"))
            .await
            .expect_err("cwd escape");
        assert_eq!(status.code(), "E_PATH_OUTSIDE_JAIL");
    }

    #[tokio::test]
    async fn invalid_commit_message_is_refused_without_running_git() {
        let (_temp, jail) = jail();
        init_repo(&jail);
        let git = Git::new(&jail);
        for message in ["", "   ", "two\nlines"] {
            let error = git
                .commit(&GitCommitRequest::new(message))
                .await
                .expect_err(message);
            assert_eq!(error.code(), "E_TOOL_SCHEMA", "{message:?}");
        }
    }

    #[test]
    fn policy_builders_clamp_to_hard_bounds() {
        let policy = GitPolicy::new()
            .with_timeout(Duration::from_millis(1))
            .with_stdout_cap(0);
        assert_eq!(policy.timeout(), GIT_MIN_TIMEOUT);
        let policy = GitPolicy::new()
            .with_timeout(Duration::from_secs(10_000))
            .with_stdout_cap(usize::MAX);
        assert_eq!(policy.timeout(), GIT_MAX_TIMEOUT);
        assert_eq!(GitPolicy::new().timeout(), GIT_TIMEOUT);
    }

    #[test]
    fn status_parser_handles_rename_records_and_truncation() {
        let entries = parse_status("R  new.txt\0old.txt\0?? other.txt\0");
        assert_eq!(entries.len(), 2);
        assert_eq!(entries[0].index, "R");
        assert_eq!(entries[0].path, "new.txt");
        assert_eq!(entries[0].original_path.as_deref(), Some("old.txt"));
        assert_eq!(entries[1].path, "other.txt");

        let log = parse_log("abc\u{1f}Ada\u{1f}2026-01-01T00:00:00+00:00\u{1f}subject\n");
        assert_eq!(log.len(), 1);
        assert_eq!(log[0].hash, "abc");
        assert_eq!(log[0].subject, "subject");
    }
}
