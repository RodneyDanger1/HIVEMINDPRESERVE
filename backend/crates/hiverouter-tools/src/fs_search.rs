//! Bounded workspace search helpers: glob, literal grep and line-range reads.
//!
//! Every path enters through [`WorkspaceJail::resolve`], so the six escape
//! vectors are refused, not clamped. Directory walks are bounded on four axes:
//! visited entries, recursion depth, result count and total scanned bytes.
//! Symlinks/junctions are never followed by the walk (the entry is skipped);
//! each visited relative path is additionally re-resolved by the jail so a
//! configured denied subtree is skipped even when the walk reaches it.
//!
//! Grep is deliberately a **literal substring** search (no regex dependency and
//! no pattern engine): `case_sensitive: false` uses ASCII case folding, so byte
//! columns stay stable. Files over the 1 MiB read cap, binary files (a NUL in
//! the first 8 KiB) and non-UTF-8 files are counted as skipped, never returned
//! as partial matches.

use std::fs;
use std::path::Path;

use hiverouter_contract::Refusal;
use serde::Serialize;

use crate::error::ToolError;
use crate::jail::{WorkspaceJail, MAX_READ_BYTES};

/// Maximum glob results returned in one call.
pub const MAX_GLOB_RESULTS: usize = 10_000;
/// Maximum directory entries visited by one walk.
pub const MAX_WALK_ENTRIES: usize = 50_000;
/// Maximum recursion depth below the requested root.
pub const MAX_WALK_DEPTH: usize = 64;
/// Maximum glob pattern length.
pub const MAX_GLOB_PATTERN_BYTES: usize = 1024;
/// Maximum literal grep matches returned in one call.
pub const MAX_GREP_MATCHES: usize = 1000;
/// Maximum files a single grep call will read.
pub const MAX_GREP_FILES: usize = 5000;
/// Maximum bytes a single grep call will scan.
pub const MAX_GREP_SCAN_BYTES: u64 = 64 * 1024 * 1024;
/// Maximum bytes of one matching line returned by grep.
pub const MAX_GREP_LINE_BYTES: usize = 4096;
/// Maximum bytes returned by one line-range read.
pub const MAX_RANGE_BYTES: usize = 256 * 1024;
/// Maximum lines one line-range read may request.
pub const MAX_RANGE_LINES: u64 = 2000;

/// One glob request.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GlobRequest {
    /// Glob pattern (`*`, `?`, `**`, `[...]`); matched against the path relative
    /// to the jail root.
    pub pattern: String,
    /// Optional workspace-relative root directory (defaults to the jail root).
    pub root: Option<String>,
    /// Desired result count; clamped to [`MAX_GLOB_RESULTS`].
    pub max_results: usize,
}

impl GlobRequest {
    /// Builds a request with the default result cap.
    #[must_use]
    pub fn new(pattern: impl Into<String>) -> Self {
        Self {
            pattern: pattern.into(),
            root: None,
            max_results: 200,
        }
    }

    /// Sets the search root.
    #[must_use]
    pub fn with_root(mut self, root: impl Into<String>) -> Self {
        self.root = Some(root.into());
        self
    }

    /// Sets the result cap.
    #[must_use]
    pub fn with_max_results(mut self, max_results: usize) -> Self {
        self.max_results = max_results;
        self
    }
}

/// One glob hit.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct GlobHit {
    /// Workspace-relative path with `/` separators.
    pub path: String,
    /// Whether the hit is a directory.
    pub is_dir: bool,
    /// File size in bytes (0 for directories).
    pub size: u64,
}

/// Bounded glob result.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct GlobResult {
    /// Matching entries, sorted by directory then name (deterministic walk).
    pub matches: Vec<GlobHit>,
    /// A cap stopped the walk or the result list.
    pub truncated: bool,
    /// Entries visited before stopping.
    pub walked: usize,
    /// Entries skipped (unreadable, non-UTF-8, symlink, or jail-refused).
    pub skipped: usize,
}

/// One literal grep request.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GrepRequest {
    /// Literal (non-regex) substring to find.
    pub pattern: String,
    /// Optional workspace-relative root (defaults to the jail root).
    pub root: Option<String>,
    /// Optional file glob filter (defaults to `**/*`).
    pub glob: Option<String>,
    /// Case-sensitive by default; `false` uses ASCII case folding.
    pub case_sensitive: bool,
    /// Desired match count; clamped to [`MAX_GREP_MATCHES`].
    pub max_matches: usize,
}

impl GrepRequest {
    /// Builds a case-sensitive request with the default caps.
    #[must_use]
    pub fn new(pattern: impl Into<String>) -> Self {
        Self {
            pattern: pattern.into(),
            root: None,
            glob: None,
            case_sensitive: true,
            max_matches: 100,
        }
    }

    /// Sets the search root.
    #[must_use]
    pub fn with_root(mut self, root: impl Into<String>) -> Self {
        self.root = Some(root.into());
        self
    }

    /// Sets the file glob filter.
    #[must_use]
    pub fn with_glob(mut self, glob: impl Into<String>) -> Self {
        self.glob = Some(glob.into());
        self
    }

    /// Sets case sensitivity.
    #[must_use]
    pub fn with_case_sensitive(mut self, case_sensitive: bool) -> Self {
        self.case_sensitive = case_sensitive;
        self
    }

    /// Sets the match cap.
    #[must_use]
    pub fn with_max_matches(mut self, max_matches: usize) -> Self {
        self.max_matches = max_matches;
        self
    }
}

/// One grep match.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct GrepMatch {
    /// Workspace-relative file path.
    pub path: String,
    /// 1-based line number.
    pub line: u64,
    /// 1-based byte column of the first match on the line.
    pub column: u64,
    /// Matching line text (bounded by [`MAX_GREP_LINE_BYTES`], no terminator).
    pub text: String,
    /// The line text was clipped at the line cap.
    pub line_truncated: bool,
}

/// Bounded grep result.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct GrepResult {
    /// Matches in walk order, then line order.
    pub matches: Vec<GrepMatch>,
    /// Files whose content was searched.
    pub files_scanned: usize,
    /// Files skipped (oversize, binary, non-UTF-8, or unreadable).
    pub files_skipped: usize,
    /// A cap (walk, match, time/byte budget) stopped the search.
    pub truncated: bool,
}

/// One line-range read request.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReadRangeRequest {
    /// Workspace-relative file path.
    pub path: String,
    /// 1-based first line.
    pub start_line: u64,
    /// Number of lines to return (1..=[`MAX_RANGE_LINES`]).
    pub line_count: u64,
}

impl ReadRangeRequest {
    /// Builds a line-range request.
    #[must_use]
    pub fn new(path: impl Into<String>, start_line: u64, line_count: u64) -> Self {
        Self {
            path: path.into(),
            start_line,
            line_count,
        }
    }
}

/// Result of a bounded line-range read.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ReadRange {
    /// Workspace-relative file path.
    pub path: String,
    /// 1-based first requested line.
    pub start_line: u64,
    /// 1-based last line whose text is present (`start_line - 1` when the range
    /// starts past the end of the file).
    pub end_line: u64,
    /// Total lines in the file.
    pub total_lines: u64,
    /// Selected line text, including original terminators.
    pub text: String,
    /// The output byte cap clipped the last line or dropped later lines.
    pub truncated: bool,
}

/// Bounded glob over the jail.
///
/// # Errors
/// [`Refusal::ToolSchema`] for an invalid pattern/root, [`Refusal::PathOutsideJail`]
/// for an escaping root, [`Refusal::PathNotFound`] when the root does not exist,
/// [`Refusal::InternalError`] for a root read failure.
pub fn glob(jail: &WorkspaceJail, request: &GlobRequest) -> Result<GlobResult, ToolError> {
    let pattern = validate_pattern(&request.pattern)?;
    let max_results = request.max_results.clamp(1, MAX_GLOB_RESULTS);
    let root = jail.resolve(request.root.as_deref().unwrap_or(""))?;
    let metadata = fs::metadata(&root).map_err(|error| {
        ToolError::new(
            Refusal::PathNotFound,
            format!("cannot inspect the glob root: {error}"),
        )
    })?;
    let root_rel = relative_path(jail, &root)?;
    if metadata.is_file() {
        let hit = GlobHit {
            path: root_rel.clone(),
            is_dir: false,
            size: metadata.len(),
        };
        let matches = if match_relative(&pattern, &root_rel, cfg!(windows)) {
            vec![hit]
        } else {
            Vec::new()
        };
        return Ok(GlobResult {
            matches,
            truncated: false,
            walked: 1,
            skipped: 0,
        });
    }
    if !metadata.is_dir() {
        return Err(ToolError::new(
            Refusal::ToolSchema,
            "glob root is not a regular file or directory",
        ));
    }

    let mut state = WalkState {
        pattern,
        case_insensitive: cfg!(windows),
        max_results,
        matches: Vec::new(),
        walked: 0,
        skipped: 0,
        truncated: false,
    };
    walk_glob(jail, &root, &root_rel, 0, &mut state)?;
    Ok(GlobResult {
        matches: state.matches,
        truncated: state.truncated,
        walked: state.walked,
        skipped: state.skipped,
    })
}

struct WalkState {
    pattern: String,
    case_insensitive: bool,
    max_results: usize,
    matches: Vec<GlobHit>,
    walked: usize,
    skipped: usize,
    truncated: bool,
}

fn walk_glob(
    jail: &WorkspaceJail,
    directory: &Path,
    relative: &str,
    depth: usize,
    state: &mut WalkState,
) -> Result<(), ToolError> {
    let reader = match fs::read_dir(directory) {
        Ok(reader) => reader,
        Err(_) => {
            state.skipped += 1;
            return Ok(());
        }
    };
    let mut entries = Vec::new();
    for entry in reader {
        let Ok(entry) = entry else {
            state.skipped += 1;
            continue;
        };
        entries.push(entry);
    }
    entries.sort_by_key(|entry| entry.file_name());

    for entry in entries {
        if state.walked >= MAX_WALK_ENTRIES {
            state.truncated = true;
            return Ok(());
        }
        let Some(name) = entry.file_name().to_str().map(ToOwned::to_owned) else {
            state.skipped += 1;
            continue;
        };
        let child_relative = join_relative(relative, &name);
        // Re-resolve every visited path: this applies the jail's containment and
        // denied-subtree rules even though the walk itself started inside it.
        match jail.resolve(&child_relative) {
            Ok(_) => {}
            Err(error) if error.refusal() == Refusal::PathOutsideJail => {
                state.skipped += 1;
                continue;
            }
            Err(error) => return Err(error),
        }
        let Ok(file_type) = entry.file_type() else {
            state.skipped += 1;
            continue;
        };
        if is_link_like(&file_type) {
            // Never follow a link during a walk; a link can point outside the jail.
            state.skipped += 1;
            continue;
        }
        state.walked += 1;
        let is_dir = file_type.is_dir();
        let size = entry.metadata().map(|metadata| metadata.len()).unwrap_or(0);
        let matched = match_relative(&state.pattern, &child_relative, state.case_insensitive);
        if matched {
            state.matches.push(GlobHit {
                path: child_relative.clone(),
                is_dir,
                size,
            });
            if state.matches.len() >= state.max_results {
                state.truncated = true;
                return Ok(());
            }
        }
        if is_dir {
            if depth + 1 > MAX_WALK_DEPTH {
                state.skipped += 1;
                continue;
            }
            walk_glob(
                jail,
                &directory.join(&name),
                &child_relative,
                depth + 1,
                state,
            )?;
            if state.truncated {
                return Ok(());
            }
        }
    }
    Ok(())
}

/// Bounded literal grep over the jail.
///
/// # Errors
/// [`Refusal::ToolSchema`] for an empty pattern or invalid root/glob,
/// [`Refusal::PathOutsideJail`] for an escaping root,
/// [`Refusal::PathNotFound`] when the root does not exist.
pub fn grep(jail: &WorkspaceJail, request: &GrepRequest) -> Result<GrepResult, ToolError> {
    if request.pattern.is_empty() {
        return Err(ToolError::new(
            Refusal::ToolSchema,
            "grep pattern must not be empty",
        ));
    }
    if request.pattern.contains('\0') {
        return Err(ToolError::new(
            Refusal::ToolSchema,
            "grep pattern must not contain a NUL byte",
        ));
    }
    let max_matches = request.max_matches.clamp(1, MAX_GREP_MATCHES);
    let root = jail.resolve(request.root.as_deref().unwrap_or(""))?;
    let metadata = fs::metadata(&root).map_err(|error| {
        ToolError::new(
            Refusal::PathNotFound,
            format!("cannot inspect the grep root: {error}"),
        )
    })?;
    let root_rel = relative_path(jail, &root)?;

    let (files, walk_truncated) = if metadata.is_file() {
        (vec![(root_rel, metadata.len())], false)
    } else if metadata.is_dir() {
        let filter = request.glob.as_deref().unwrap_or("**/*");
        let pattern = validate_pattern(filter)?;
        let mut state = WalkState {
            pattern,
            case_insensitive: cfg!(windows),
            max_results: MAX_GREP_FILES,
            matches: Vec::new(),
            walked: 0,
            skipped: 0,
            truncated: false,
        };
        walk_glob(jail, &root, &root_rel, 0, &mut state)?;
        let files = state
            .matches
            .into_iter()
            .filter(|hit| !hit.is_dir)
            .map(|hit| (hit.path, hit.size))
            .collect();
        (files, state.truncated)
    } else {
        return Err(ToolError::new(
            Refusal::ToolSchema,
            "grep root is not a regular file or directory",
        ));
    };

    let needle = if request.case_sensitive {
        request.pattern.clone()
    } else {
        request.pattern.to_ascii_lowercase()
    };
    let mut result = GrepResult {
        matches: Vec::new(),
        files_scanned: 0,
        files_skipped: 0,
        truncated: walk_truncated,
    };
    let mut scanned_bytes: u64 = 0;

    'files: for (path, size) in files {
        if result.matches.len() >= max_matches {
            result.truncated = true;
            break;
        }
        if scanned_bytes >= MAX_GREP_SCAN_BYTES {
            result.truncated = true;
            break;
        }
        if size > MAX_READ_BYTES {
            result.files_skipped += 1;
            continue;
        }
        let bytes = match jail.read(&path) {
            Ok(bytes) => bytes,
            Err(error) if error.refusal() == Refusal::FileTooLarge => {
                result.files_skipped += 1;
                continue;
            }
            Err(error) => return Err(error),
        };
        scanned_bytes = scanned_bytes.saturating_add(bytes.len() as u64);
        if looks_binary(&bytes) {
            result.files_skipped += 1;
            continue;
        }
        let Ok(text) = std::str::from_utf8(&bytes) else {
            result.files_skipped += 1;
            continue;
        };
        result.files_scanned += 1;
        for (line_index, line) in text.lines().enumerate() {
            let found = if request.case_sensitive {
                line.find(&needle)
            } else {
                line.to_ascii_lowercase().find(&needle)
            };
            let Some(column) = found else {
                continue;
            };
            let (text, line_truncated) = clip_line(line);
            result.matches.push(GrepMatch {
                path: path.clone(),
                line: line_index as u64 + 1,
                column: column as u64 + 1,
                text,
                line_truncated,
            });
            if result.matches.len() >= max_matches {
                result.truncated = true;
                break 'files;
            }
        }
        if scanned_bytes >= MAX_GREP_SCAN_BYTES {
            result.truncated = true;
            break;
        }
    }
    Ok(result)
}

/// Reads a bounded line range from one jail-contained UTF-8 file.
///
/// # Errors
/// [`Refusal::ToolSchema`] for `start_line == 0`, `line_count == 0`, a request
/// over [`MAX_RANGE_LINES`], or a non-UTF-8 target; the same path errors as
/// [`WorkspaceJail::read`].
pub fn read_range(
    jail: &WorkspaceJail,
    request: &ReadRangeRequest,
) -> Result<ReadRange, ToolError> {
    if request.start_line == 0 {
        return Err(ToolError::new(
            Refusal::ToolSchema,
            "start_line is 1-based; 0 is not a valid line",
        ));
    }
    if request.line_count == 0 {
        return Err(ToolError::new(
            Refusal::ToolSchema,
            "line_count must be at least 1",
        ));
    }
    if request.line_count > MAX_RANGE_LINES {
        return Err(ToolError::new(
            Refusal::ToolSchema,
            format!(
                "line_count {} exceeds the {MAX_RANGE_LINES}-line range cap",
                request.line_count
            ),
        ));
    }
    let bytes = jail.read(&request.path)?;
    let text = std::str::from_utf8(&bytes).map_err(|_| {
        ToolError::new(
            Refusal::ToolSchema,
            format!(
                "file {:?} is not valid UTF-8; the range reader returns text only",
                request.path
            ),
        )
    })?;
    let lines: Vec<&str> = text.split_inclusive('\n').collect();
    let total_lines = lines.len() as u64;
    if request.start_line > total_lines {
        return Ok(ReadRange {
            path: request.path.clone(),
            start_line: request.start_line,
            end_line: request.start_line - 1,
            total_lines,
            text: String::new(),
            truncated: false,
        });
    }
    let start_index = (request.start_line - 1) as usize;
    let requested_end = start_index.saturating_add(request.line_count as usize);
    let end_index = requested_end.min(lines.len());
    let mut output = String::new();
    let mut end_line = request.start_line - 1;
    let mut truncated = false;
    for (offset, line) in lines[start_index..end_index].iter().enumerate() {
        if output.len() >= MAX_RANGE_BYTES {
            truncated = true;
            break;
        }
        let remaining = MAX_RANGE_BYTES - output.len();
        if line.len() <= remaining {
            output.push_str(line);
            end_line = request.start_line + offset as u64;
            continue;
        }
        let mut cut = remaining;
        while cut > 0 && !line.is_char_boundary(cut) {
            cut -= 1;
        }
        output.push_str(&line[..cut]);
        end_line = request.start_line + offset as u64;
        truncated = true;
        break;
    }
    Ok(ReadRange {
        path: request.path.clone(),
        start_line: request.start_line,
        end_line,
        total_lines,
        text: output,
        truncated,
    })
}

fn relative_path(jail: &WorkspaceJail, absolute: &Path) -> Result<String, ToolError> {
    let relative = absolute.strip_prefix(jail.root()).map_err(|_| {
        ToolError::new(
            Refusal::PathOutsideJail,
            "path is not inside the workspace root",
        )
    })?;
    let text = relative
        .to_str()
        .ok_or_else(|| ToolError::new(Refusal::ToolSchema, "workspace path is not valid UTF-8"))?;
    Ok(text.replace('\\', "/"))
}

fn join_relative(parent: &str, name: &str) -> String {
    if parent.is_empty() {
        name.to_owned()
    } else {
        format!("{parent}/{name}")
    }
}

fn validate_pattern(pattern: &str) -> Result<String, ToolError> {
    let pattern = pattern.trim();
    if pattern.is_empty() {
        return Err(ToolError::new(
            Refusal::ToolSchema,
            "glob pattern must not be empty",
        ));
    }
    if pattern.len() > MAX_GLOB_PATTERN_BYTES {
        return Err(ToolError::new(
            Refusal::ToolSchema,
            format!(
                "glob pattern is {} bytes; the cap is {MAX_GLOB_PATTERN_BYTES}",
                pattern.len()
            ),
        ));
    }
    if pattern.contains('\0') {
        return Err(ToolError::new(
            Refusal::ToolSchema,
            "glob pattern must not contain a NUL byte",
        ));
    }
    let normalized = pattern.replace('\\', "/");
    if normalized.starts_with('/') {
        return Err(ToolError::new(
            Refusal::ToolSchema,
            "glob pattern must be relative (leading `/` is refused)",
        ));
    }
    let mut chars = normalized.chars();
    if let (Some(first), Some(':')) = (chars.next(), chars.next()) {
        if first.is_ascii_alphabetic() {
            return Err(ToolError::new(
                Refusal::ToolSchema,
                "glob pattern must be relative (drive prefix is refused)",
            ));
        }
    }
    if normalized.split('/').any(|segment| segment == "..") {
        return Err(ToolError::new(
            Refusal::ToolSchema,
            "glob pattern must not contain `..`",
        ));
    }
    let normalized = normalized
        .trim_start_matches("./")
        .trim_end_matches('/')
        .to_owned();
    if normalized.is_empty() {
        return Err(ToolError::new(
            Refusal::ToolSchema,
            "glob pattern does not name a path",
        ));
    }
    Ok(normalized)
}

fn match_relative(pattern: &str, path: &str, case_insensitive: bool) -> bool {
    let (pattern, path) = if case_insensitive {
        (pattern.to_ascii_lowercase(), path.to_ascii_lowercase())
    } else {
        (pattern.to_owned(), path.to_owned())
    };
    let pattern_segments: Vec<&str> = pattern.split('/').filter(|s| !s.is_empty()).collect();
    let path_segments: Vec<&str> = path.split('/').filter(|s| !s.is_empty()).collect();
    match_segments(&pattern_segments, &path_segments)
}

fn match_segments(pattern: &[&str], path: &[&str]) -> bool {
    let mut memo = vec![vec![0u8; path.len() + 1]; pattern.len() + 1];
    fn go(pi: usize, si: usize, pattern: &[&str], path: &[&str], memo: &mut [Vec<u8>]) -> bool {
        if memo[pi][si] != 0 {
            return memo[pi][si] == 2;
        }
        let result = if pi == pattern.len() {
            si == path.len()
        } else if pattern[pi] == "**" {
            go(pi + 1, si, pattern, path, memo)
                || (si < path.len() && go(pi, si + 1, pattern, path, memo))
        } else {
            si < path.len()
                && match_segment(pattern[pi], path[si])
                && go(pi + 1, si + 1, pattern, path, memo)
        };
        memo[pi][si] = if result { 2 } else { 1 };
        result
    }
    go(0, 0, pattern, path, &mut memo)
}

fn match_segment(pattern: &str, text: &str) -> bool {
    let pattern: Vec<char> = pattern.chars().collect();
    let text: Vec<char> = text.chars().collect();
    let mut memo = vec![vec![0u8; text.len() + 1]; pattern.len() + 1];
    go_segment(&pattern, &text, 0, 0, &mut memo)
}

fn go_segment(pattern: &[char], text: &[char], pi: usize, ti: usize, memo: &mut [Vec<u8>]) -> bool {
    if memo[pi][ti] != 0 {
        return memo[pi][ti] == 2;
    }
    let result = if pi == pattern.len() {
        ti == text.len()
    } else {
        match pattern[pi] {
            '*' => {
                go_segment(pattern, text, pi + 1, ti, memo)
                    || (ti < text.len() && go_segment(pattern, text, pi, ti + 1, memo))
            }
            '?' => ti < text.len() && go_segment(pattern, text, pi + 1, ti + 1, memo),
            '[' => match class_match(pattern, pi, text.get(ti).copied()) {
                Some((matches, next)) => matches && go_segment(pattern, text, next, ti + 1, memo),
                None => {
                    ti < text.len()
                        && text[ti] == '['
                        && go_segment(pattern, text, pi + 1, ti + 1, memo)
                }
            },
            literal => {
                ti < text.len()
                    && text[ti].eq_ignore_ascii_case(&literal)
                    && go_segment(pattern, text, pi + 1, ti + 1, memo)
            }
        }
    };
    memo[pi][ti] = if result { 2 } else { 1 };
    result
}

/// Parses a `[...]` class starting at `pattern[start] == '['`. Returns whether
/// `target` matches and the index after the closing `]`; `None` when the class
/// is unterminated (the caller then treats `[` as a literal).
fn class_match(pattern: &[char], start: usize, target: Option<char>) -> Option<(bool, usize)> {
    let mut index = start + 1;
    let mut negated = false;
    if index < pattern.len() && (pattern[index] == '!' || pattern[index] == '^') {
        negated = true;
        index += 1;
    }
    let mut matched = false;
    let mut first = true;
    while index < pattern.len() {
        let current = pattern[index];
        if current == ']' && !first {
            let result = if negated { !matched } else { matched };
            return Some((result, index + 1));
        }
        first = false;
        if index + 2 < pattern.len() && pattern[index + 1] == '-' && pattern[index + 2] != ']' {
            if let Some(target) = target {
                let target = target.to_ascii_lowercase();
                if current.to_ascii_lowercase() <= target
                    && target <= pattern[index + 2].to_ascii_lowercase()
                {
                    matched = true;
                }
            }
            index += 3;
            continue;
        }
        if let Some(target) = target {
            if target.eq_ignore_ascii_case(&current) {
                matched = true;
            }
        }
        index += 1;
    }
    None
}

fn is_link_like(file_type: &fs::FileType) -> bool {
    #[cfg(windows)]
    {
        use std::os::windows::fs::FileTypeExt;
        file_type.is_symlink() || file_type.is_symlink_dir() || file_type.is_symlink_file()
    }
    #[cfg(not(windows))]
    {
        file_type.is_symlink()
    }
}

fn looks_binary(bytes: &[u8]) -> bool {
    bytes.iter().take(8192).any(|byte| *byte == 0)
}

fn clip_line(line: &str) -> (String, bool) {
    if line.len() <= MAX_GREP_LINE_BYTES {
        return (line.to_owned(), false);
    }
    let mut end = MAX_GREP_LINE_BYTES;
    while end > 0 && !line.is_char_boundary(end) {
        end -= 1;
    }
    (line[..end].to_owned(), true)
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

    #[test]
    fn matcher_supports_star_globstar_question_and_classes() {
        assert!(match_relative("*.txt", "a.txt", false));
        assert!(!match_relative("*.txt", "nested/a.txt", false));
        assert!(match_relative("**/*.txt", "nested/deep/a.txt", false));
        assert!(match_relative("**/*.txt", "a.txt", false));
        assert!(match_relative("src/**/mod.rs", "src/a/b/mod.rs", false));
        assert!(match_relative("file?.[ch]", "file1.c", false));
        assert!(match_relative("[a-c]*", "bob", false));
        assert!(!match_relative("[!a-c]*", "bob", false));
        assert!(match_relative("[!a-c]*", "dave", false));
        assert!(match_relative("**", "any/thing", false));
    }

    #[test]
    fn glob_walks_and_matches_inside_the_jail() {
        let (_temp, jail) = jail();
        fs::create_dir_all(jail.root().join("src/nested")).expect("dirs");
        fs::write(jail.root().join("top.txt"), b"top").expect("top");
        fs::write(jail.root().join("src/a.txt"), b"a").expect("a");
        fs::write(jail.root().join("src/nested/b.txt"), b"b").expect("b");
        fs::write(jail.root().join("src/nested/c.rs"), b"c").expect("c");

        let result = glob(&jail, &GlobRequest::new("**/*.txt")).expect("glob");
        let paths: Vec<&str> = result.matches.iter().map(|hit| hit.path.as_str()).collect();
        assert_eq!(paths, ["src/a.txt", "src/nested/b.txt", "top.txt"]);
        assert!(!result.truncated);
        assert!(result.walked >= 4);
    }

    #[test]
    fn glob_respects_result_and_walk_caps() {
        let (_temp, jail) = jail();
        for index in 0..5 {
            fs::write(jail.root().join(format!("f{index}.txt")), b"x").expect("seed");
        }
        let result = glob(&jail, &GlobRequest::new("*.txt").with_max_results(2)).expect("glob");
        assert_eq!(result.matches.len(), 2);
        assert!(result.truncated);
    }

    #[test]
    fn glob_refuses_escaping_roots_and_patterns() {
        let (_temp, jail) = jail();
        let error =
            glob(&jail, &GlobRequest::new("**").with_root("../outside")).expect_err("escape");
        assert_eq!(error.code(), "E_PATH_OUTSIDE_JAIL");
        for pattern in ["/etc/passwd", "C:\\Windows\\*.dll", "../secret", ""] {
            let error = glob(&jail, &GlobRequest::new(pattern)).expect_err(pattern);
            assert_eq!(error.code(), "E_TOOL_SCHEMA", "{pattern:?}");
        }
    }

    #[test]
    fn glob_skips_symlinks_instead_of_following_them() {
        let (_temp, jail) = jail();
        let outside = tempfile::tempdir().expect("outside");
        fs::write(outside.path().join("secret.txt"), b"secret").expect("secret");
        #[cfg(windows)]
        {
            let link = jail.root().join("link");
            let status = std::process::Command::new("cmd")
                .args([
                    "/C",
                    "mklink",
                    "/J",
                    &link.to_string_lossy(),
                    &outside.path().to_string_lossy(),
                ])
                .output()
                .expect("mklink");
            if status.status.success() {
                let result = glob(&jail, &GlobRequest::new("**/*.txt")).expect("glob");
                assert!(
                    result
                        .matches
                        .iter()
                        .all(|hit| hit.path != "link/secret.txt"),
                    "walk followed a junction outside the jail: {:?}",
                    result.matches
                );
                assert!(result.skipped >= 1);
            }
        }
    }

    #[test]
    fn grep_finds_lines_with_line_and_column() {
        let (_temp, jail) = jail();
        fs::create_dir_all(jail.root().join("src")).expect("dir");
        fs::write(
            jail.root().join("src/lib.rs"),
            b"fn main() {}\n// TODO: fix\nlet x = \"TODO\";\n",
        )
        .expect("seed");
        fs::write(jail.root().join("src/skip.txt"), b"TODO\n").expect("seed");

        let result = grep(&jail, &GrepRequest::new("TODO").with_glob("**/*.rs")).expect("grep");
        assert_eq!(result.matches.len(), 2);
        assert_eq!(result.matches[0].path, "src/lib.rs");
        assert_eq!(result.matches[0].line, 2);
        assert_eq!(result.matches[0].column, 4);
        assert_eq!(result.matches[0].text, "// TODO: fix");
        assert_eq!(result.matches[1].line, 3);
        assert_eq!(result.files_scanned, 1);
    }

    #[test]
    fn grep_is_case_sensitive_by_default_and_folds_on_request() {
        let (_temp, jail) = jail();
        fs::write(jail.root().join("a.txt"), b"Hello\nhello\n").expect("seed");
        assert!(grep(&jail, &GrepRequest::new("HELLO"))
            .expect("grep")
            .matches
            .is_empty());
        let folded =
            grep(&jail, &GrepRequest::new("HELLO").with_case_sensitive(false)).expect("grep");
        assert_eq!(folded.matches.len(), 2);
        assert_eq!(folded.matches[0].line, 1);
    }

    #[test]
    fn grep_skips_oversize_binary_and_non_utf8_files() {
        let (_temp, jail) = jail();
        fs::write(jail.root().join("bin.dat"), [0u8, 1, 2]).expect("seed");
        fs::write(jail.root().join("bad.txt"), [0xff, 0xfe]).expect("seed");
        fs::write(
            jail.root().join("big.txt"),
            vec![b'a'; usize::try_from(MAX_READ_BYTES).expect("fits") + 1],
        )
        .expect("seed");
        fs::write(jail.root().join("ok.txt"), b"needle\n").expect("seed");
        let result = grep(&jail, &GrepRequest::new("needle")).expect("grep");
        assert_eq!(result.matches.len(), 1);
        assert_eq!(result.matches[0].path, "ok.txt");
        assert_eq!(result.files_skipped, 3);
    }

    #[test]
    fn grep_caps_matches() {
        let (_temp, jail) = jail();
        fs::write(jail.root().join("a.txt"), b"x\nx\nx\n").expect("seed");
        let result = grep(&jail, &GrepRequest::new("x").with_max_matches(2)).expect("grep");
        assert_eq!(result.matches.len(), 2);
        assert!(result.truncated);
    }

    #[test]
    fn read_range_selects_lines_and_reports_bounds() {
        let (_temp, jail) = jail();
        jail.write("f.txt", b"one\ntwo\nthree\nfour\n")
            .expect("seed");
        let range = read_range(&jail, &ReadRangeRequest::new("f.txt", 2, 2)).expect("range");
        assert_eq!(range.text, "two\nthree\n");
        assert_eq!(range.start_line, 2);
        assert_eq!(range.end_line, 3);
        assert_eq!(range.total_lines, 4);
        assert!(!range.truncated);

        let past = read_range(&jail, &ReadRangeRequest::new("f.txt", 9, 2)).expect("range");
        assert_eq!(past.text, "");
        assert_eq!(past.end_line, 8);
        assert_eq!(past.total_lines, 4);
    }

    #[test]
    fn read_range_rejects_invalid_windows_and_escapes() {
        let (_temp, jail) = jail();
        jail.write("f.txt", b"one\n").expect("seed");
        for (start, count) in [(0, 1), (1, 0), (1, MAX_RANGE_LINES + 1)] {
            let error = read_range(&jail, &ReadRangeRequest::new("f.txt", start, count))
                .expect_err("invalid range");
            assert_eq!(error.code(), "E_TOOL_SCHEMA", "{start},{count}");
        }
        let error =
            read_range(&jail, &ReadRangeRequest::new("../f.txt", 1, 1)).expect_err("escape");
        assert_eq!(error.code(), "E_PATH_OUTSIDE_JAIL");
    }

    #[test]
    fn read_range_clips_at_the_byte_cap() {
        let (_temp, jail) = jail();
        let line = "x".repeat(MAX_RANGE_BYTES + 100);
        jail.write("big.txt", line.as_bytes()).expect("seed");
        let range = read_range(&jail, &ReadRangeRequest::new("big.txt", 1, 1)).expect("range");
        assert!(range.truncated);
        assert_eq!(range.text.len(), MAX_RANGE_BYTES);
        assert_eq!(range.end_line, 1);
    }
}
