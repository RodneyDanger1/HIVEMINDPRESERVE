//! Atomic unified-diff application inside the workspace jail.
//!
//! Contract:
//! - one target file per call; the `---`/`+++` headers must name that file
//!   (renames, creations and deletions are refused, never silently dropped);
//! - the whole patch is parsed and every hunk is applied in memory before any
//!   write happens, so a context mismatch, a stale `expected_sha256`, a malformed
//!   hunk or an over-size result leaves the target **byte-identical**;
//! - the only write is one [`WorkspaceJail::write_atomic`] rename of a completed
//!   result (no partial file can be observed under the target name);
//! - hunks are applied against the original file with a running line delta, so
//!   multi-hunk patches with earlier line-count changes land on the right lines;
//! - a context/removal mismatch returns [`PatchOutcome::Rejected`] with the
//!   1-based failed hunk index and the target line, never a clamped guess.
//!
//! Supported syntax is the standard unified diff body: `---`/`+++` headers,
//! `@@ -old_start[,old_count] +new_start[,new_count] @@` headers and
//! ` `/`+`/`-`/`\ No newline at end of file` body lines. `\r\n` targets keep
//! their line endings; added lines use the target's first observed ending.

use hiverouter_contract::Refusal;
use serde::Serialize;

use crate::error::ToolError;
use crate::jail::WorkspaceJail;
use crate::sha256::sha256_hex;
use crate::side_effects::MAX_WRITE_BYTES;

/// Maximum accepted patch text (1 MiB), the same order as the read/write caps.
pub const MAX_PATCH_BYTES: usize = 1024 * 1024;
/// Maximum hunks in one patch (prevents quadratic parse work).
pub const MAX_HUNKS: usize = 1000;
/// Maximum individual diff operations (` `, `+`, `-` lines) in one patch.
pub const MAX_PATCH_OPERATIONS: usize = 100_000;
/// Maximum bytes quoted from a mismatching line in a rejection reason.
const MAX_QUOTED_BYTES: usize = 200;

/// One apply request. `path` is workspace-relative and resolved through the jail.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PatchRequest {
    /// Workspace-relative target file.
    pub path: String,
    /// Unified diff text.
    pub patch: String,
    /// Optional 64-hex SHA-256 of the target the caller expects to patch.
    pub expected_sha256: Option<String>,
}

impl PatchRequest {
    /// Builds a request without a stale-write guard.
    #[must_use]
    pub fn new(path: impl Into<String>, patch: impl Into<String>) -> Self {
        Self {
            path: path.into(),
            patch: patch.into(),
            expected_sha256: None,
        }
    }

    /// Adds the stale-write guard.
    #[must_use]
    pub fn with_expected_sha256(mut self, expected_sha256: impl Into<String>) -> Self {
        self.expected_sha256 = Some(expected_sha256.into());
        self
    }
}

/// Result of one patch call.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub enum PatchOutcome {
    /// Every hunk applied and the result was atomically written.
    Applied {
        /// Workspace-relative target.
        path: String,
        /// Number of hunks applied.
        hunks: usize,
        /// Target size before the patch.
        old_bytes: usize,
        /// Target size after the patch.
        new_bytes: usize,
        /// SHA-256 of the written content.
        sha256: String,
    },
    /// The patch was parsed but a hunk did not match. Nothing was written, so
    /// the target is byte-identical to its pre-call content.
    Rejected {
        /// Workspace-relative target.
        path: String,
        /// 1-based index of the first failed hunk.
        failed_hunk: usize,
        /// Bounded explanation (expected vs actual context).
        reason: String,
        /// 1-based target line where matching failed, when known.
        target_line: Option<usize>,
    },
}

/// Parses and atomically applies one unified diff to one jail-contained file.
///
/// # Errors
/// [`Refusal::ToolSchema`] / [`Refusal::SchemaInvalid`] for a malformed patch,
/// mismatched headers, an invalid or stale `expected_sha256`, a binary target or
/// a target that changed while the patch was being prepared;
/// [`Refusal::PathOutsideJail`] for an escaping path; [`Refusal::PathNotFound`]
/// when the target does not exist; [`Refusal::FileTooLarge`] when the patch or
/// the result exceeds its cap; [`Refusal::InternalError`] for I/O failures.
///
/// A context mismatch is **not** an error: it is [`PatchOutcome::Rejected`] and
/// the target is untouched.
pub fn apply_patch(
    jail: &WorkspaceJail,
    request: &PatchRequest,
) -> Result<PatchOutcome, ToolError> {
    if request.path.trim().is_empty() {
        return Err(schema("patch target path must not be empty"));
    }
    // Containment first: an escaping target is refused before the patch is
    // parsed, so no patch text can influence path handling.
    let _ = jail.resolve(&request.path)?;
    if request.patch.len() > MAX_PATCH_BYTES {
        return Err(ToolError::new(
            Refusal::FileTooLarge,
            format!(
                "patch is {} bytes; the patch cap is {MAX_PATCH_BYTES} bytes",
                request.patch.len()
            ),
        ));
    }
    let hunks = parse_patch(&request.patch, &request.path)?;
    let current = jail.read(&request.path)?;
    let current_text = std::str::from_utf8(&current).map_err(|_| {
        ToolError::new(
            Refusal::ToolSchema,
            format!(
                "patch target {:?} is not valid UTF-8; binary patch targets are refused",
                request.path
            ),
        )
    })?;
    if let Some(expected) = request.expected_sha256.as_deref() {
        verify_expected_sha(expected, &sha256_hex(&current))?;
    }

    let new_text = match apply_hunks(current_text, &hunks) {
        Ok(text) => text,
        Err(reject) => {
            return Ok(PatchOutcome::Rejected {
                path: request.path.clone(),
                failed_hunk: reject.failed_hunk,
                reason: reject.reason,
                target_line: reject.target_line,
            });
        }
    };
    if new_text.len() > MAX_WRITE_BYTES {
        return Err(ToolError::new(
            Refusal::FileTooLarge,
            format!(
                "patched result is {} bytes; the write cap is {MAX_WRITE_BYTES} bytes",
                new_text.len()
            ),
        ));
    }

    // Narrow the TOCTOU window: a target mutated after the first read is not
    // silently overwritten with a patch applied to stale content.
    let before_write = jail.read(&request.path)?;
    if before_write != current {
        return Err(ToolError::new(
            Refusal::SchemaInvalid,
            format!(
                "patch target {:?} changed while the patch was being prepared; \
                 nothing was written",
                request.path
            ),
        ));
    }

    let sha256 = sha256_hex(new_text.as_bytes());
    jail.write_atomic(&request.path, new_text.as_bytes())?;
    Ok(PatchOutcome::Applied {
        path: request.path.clone(),
        hunks: hunks.len(),
        old_bytes: current.len(),
        new_bytes: new_text.len(),
        sha256,
    })
}

fn schema(detail: impl Into<String>) -> ToolError {
    ToolError::new(Refusal::ToolSchema, detail)
}

fn verify_expected_sha(expected: &str, actual: &str) -> Result<(), ToolError> {
    if expected.len() != 64 || !expected.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        return Err(ToolError::new(
            Refusal::SchemaInvalid,
            "expected_sha256 must be 64 hexadecimal characters",
        ));
    }
    if !actual.eq_ignore_ascii_case(expected) {
        return Err(ToolError::new(
            Refusal::SchemaInvalid,
            format!(
                "stale patch refused: current sha256 {actual} does not match \
                 expected_sha256 {expected}; nothing was written"
            ),
        ));
    }
    Ok(())
}

/// A target line, split into content and the exact terminator to preserve.
#[derive(Debug, Clone, PartialEq, Eq)]
struct Line {
    text: String,
    ending: Ending,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Ending {
    Lf,
    CrLf,
    None,
}

impl Ending {
    fn as_str(self) -> &'static str {
        match self {
            Ending::Lf => "\n",
            Ending::CrLf => "\r\n",
            Ending::None => "",
        }
    }
}

fn split_lines(content: &str) -> Vec<Line> {
    content
        .split_inclusive('\n')
        .map(|raw| match raw.strip_suffix('\n') {
            Some(body) => match body.strip_suffix('\r') {
                Some(body) => Line {
                    text: body.to_owned(),
                    ending: Ending::CrLf,
                },
                None => Line {
                    text: body.to_owned(),
                    ending: Ending::Lf,
                },
            },
            None => Line {
                text: raw.to_owned(),
                ending: Ending::None,
            },
        })
        .collect()
}

fn dominant_ending(lines: &[Line]) -> Ending {
    lines
        .iter()
        .map(|line| line.ending)
        .find(|ending| *ending != Ending::None)
        .unwrap_or(Ending::Lf)
}

fn render_lines(lines: &[Line], eol: Ending) -> String {
    let mut out = String::new();
    for (index, line) in lines.iter().enumerate() {
        out.push_str(&line.text);
        if index + 1 < lines.len() {
            let ending = if line.ending == Ending::None {
                eol
            } else {
                line.ending
            };
            out.push_str(ending.as_str());
        } else {
            out.push_str(line.ending.as_str());
        }
    }
    out
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum OpKind {
    Context,
    Add,
    Remove,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Op {
    kind: OpKind,
    text: String,
    no_newline: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Hunk {
    old_start: usize,
    old_count: usize,
    new_start: usize,
    new_count: usize,
    ops: Vec<Op>,
}

fn diff_lines(patch: &str) -> Vec<&str> {
    if patch.is_empty() {
        return Vec::new();
    }
    patch
        .split_inclusive('\n')
        .map(|raw| {
            let raw = raw.strip_suffix('\n').unwrap_or(raw);
            raw.strip_suffix('\r').unwrap_or(raw)
        })
        .collect()
}

fn parse_patch(patch: &str, target: &str) -> Result<Vec<Hunk>, ToolError> {
    let lines = diff_lines(patch);
    let mut index = 0usize;
    // Tolerate `diff --git`/`index` preamble lines produced by `git diff`, but
    // name the unsupported operations explicitly instead of failing later with
    // a missing header.
    while index < lines.len() {
        let line = lines[index];
        if line.starts_with("rename from ")
            || line.starts_with("rename to ")
            || line.starts_with("copy from ")
            || line.starts_with("copy to ")
            || line.starts_with("new file mode ")
            || line.starts_with("deleted file mode ")
        {
            return Err(schema(
                "rename/copy/create/delete patches are not supported; apply a \
                 modification patch or use file_write for a new file",
            ));
        }
        if line.starts_with("diff --git ")
            || line.starts_with("index ")
            || line.starts_with("old mode ")
            || line.starts_with("new mode ")
            || line.starts_with("similarity index ")
            || line.starts_with("dissimilarity index ")
        {
            index += 1;
            continue;
        }
        break;
    }
    let old_header = lines
        .get(index)
        .and_then(|line| line.strip_prefix("--- "))
        .ok_or_else(|| schema("patch is missing the `---` file header"))?;
    index += 1;
    let new_header = lines
        .get(index)
        .and_then(|line| line.strip_prefix("+++ "))
        .ok_or_else(|| schema("patch is missing the `+++` file header"))?;
    index += 1;
    let old_path = header_path(old_header)?;
    let new_path = header_path(new_header)?;
    if !same_path(&old_path, &new_path) {
        return Err(schema(format!(
            "patch headers name different files ({old_path:?} vs {new_path:?}); \
             renames are not supported"
        )));
    }
    if !same_path(&old_path, target) {
        return Err(schema(format!(
            "patch headers name {old_path:?} but the requested target is {target:?}; \
             nothing was written"
        )));
    }

    let mut hunks = Vec::new();
    let mut operations = 0usize;
    while index < lines.len() {
        let line = lines[index];
        if line.trim().is_empty() {
            index += 1;
            continue;
        }
        if line.starts_with("--- ") {
            return Err(schema(
                "patch contains more than one file; one target file per call is supported",
            ));
        }
        if !line.starts_with("@@") {
            return Err(schema(format!(
                "unexpected patch line {index}; expected a `@@` hunk header"
            )));
        }
        let (old_start, old_count, new_start, new_count) = parse_hunk_header(line)?;
        index += 1;
        let mut ops: Vec<Op> = Vec::new();
        let mut old_seen = 0usize;
        let mut new_seen = 0usize;
        while old_seen < old_count || new_seen < new_count {
            let body = lines.get(index).ok_or_else(|| {
                schema(format!(
                    "hunk at {old_start},{old_count} ended before its declared line counts"
                ))
            })?;
            index += 1;
            if let Some(marker) = body.strip_prefix('\\') {
                let previous = ops.last_mut().ok_or_else(|| {
                    schema(format!(
                        "`\\ No newline` marker for hunk at {old_start} has no preceding line"
                    ))
                })?;
                if !marker.starts_with(" No newline at end of file") {
                    return Err(schema(format!(
                        "unrecognised patch marker {body:?}; only \
                         `\\ No newline at end of file` is supported"
                    )));
                }
                previous.no_newline = true;
                continue;
            }
            let (kind, text) = match body.chars().next() {
                Some(' ') => (OpKind::Context, &body[1..]),
                Some('+') => (OpKind::Add, &body[1..]),
                Some('-') => (OpKind::Remove, &body[1..]),
                // Some hand-edited patches leave the leading space off a blank
                // context line. Accept the empty line as empty context.
                None => (OpKind::Context, ""),
                Some(other) => {
                    return Err(schema(format!(
                        "unrecognised hunk line starting with {other:?}; expected ` `, `+` or `-`"
                    )));
                }
            };
            match kind {
                OpKind::Context => {
                    old_seen += 1;
                    new_seen += 1;
                }
                OpKind::Add => new_seen += 1,
                OpKind::Remove => old_seen += 1,
            }
            if old_seen > old_count || new_seen > new_count {
                return Err(schema(format!(
                    "hunk at {old_start},{old_count} contains more lines than its header declares"
                )));
            }
            operations += 1;
            if operations > MAX_PATCH_OPERATIONS {
                return Err(schema(format!(
                    "patch contains more than {MAX_PATCH_OPERATIONS} diff operations"
                )));
            }
            ops.push(Op {
                kind,
                text: text.to_owned(),
                no_newline: false,
            });
        }
        // A marker after the final declared line applies to that final line.
        while index < lines.len() {
            let Some(marker) = lines[index].strip_prefix('\\') else {
                break;
            };
            if !marker.starts_with(" No newline at end of file") {
                return Err(schema(format!(
                    "unrecognised patch marker {:?}",
                    lines[index]
                )));
            }
            let previous = ops
                .last_mut()
                .ok_or_else(|| schema("`\\ No newline` marker has no preceding hunk line"))?;
            previous.no_newline = true;
            index += 1;
        }
        if ops.is_empty() {
            return Err(schema(format!("hunk at {old_start},{old_count} is empty")));
        }
        hunks.push(Hunk {
            old_start,
            old_count,
            new_start,
            new_count,
            ops,
        });
        if hunks.len() > MAX_HUNKS {
            return Err(schema(format!(
                "patch contains more than {MAX_HUNKS} hunks"
            )));
        }
    }
    if hunks.is_empty() {
        return Err(schema("patch contains no hunks"));
    }
    Ok(hunks)
}

fn header_path(header: &str) -> Result<String, ToolError> {
    let name = header.split('\t').next().unwrap_or(header).trim();
    if name.is_empty() {
        return Err(schema("patch file header has an empty path"));
    }
    if name == "/dev/null" {
        return Err(schema(
            "file creation/deletion patches are not supported; use file_write instead",
        ));
    }
    let name = name.replace('\\', "/");
    let name = name.strip_prefix("./").unwrap_or(&name);
    let name = name
        .strip_prefix("a/")
        .or_else(|| name.strip_prefix("b/"))
        .unwrap_or(name);
    Ok(name.to_owned())
}

fn same_path(left: &str, right: &str) -> bool {
    let left = normalize_path(left);
    let right = normalize_path(right);
    if cfg!(windows) {
        left.eq_ignore_ascii_case(&right)
    } else {
        left == right
    }
}

fn normalize_path(text: &str) -> String {
    let text = text.replace('\\', "/");
    text.strip_prefix("./").unwrap_or(&text).to_owned()
}

fn parse_hunk_header(line: &str) -> Result<(usize, usize, usize, usize), ToolError> {
    let rest = line
        .strip_prefix("@@")
        .ok_or_else(|| schema(format!("malformed hunk header {line:?}")))?;
    let end = rest
        .find("@@")
        .ok_or_else(|| schema(format!("hunk header is missing its closing `@@`: {line:?}")))?;
    let middle = rest[..end].trim();
    let mut parts = middle.split_whitespace();
    let old = parts
        .next()
        .ok_or_else(|| schema(format!("hunk header has no old range: {line:?}")))?;
    let new = parts
        .next()
        .ok_or_else(|| schema(format!("hunk header has no new range: {line:?}")))?;
    if parts.next().is_some() {
        return Err(schema(format!(
            "hunk header has unexpected trailing fields: {line:?}"
        )));
    }
    let (old_start, old_count) = parse_range(old, '-')?;
    let (new_start, new_count) = parse_range(new, '+')?;
    Ok((old_start, old_count, new_start, new_count))
}

fn parse_range(spec: &str, sign: char) -> Result<(usize, usize), ToolError> {
    let spec = spec
        .strip_prefix(sign)
        .ok_or_else(|| schema(format!("hunk range {spec:?} does not start with {sign:?}")))?;
    let (start, count) = match spec.split_once(',') {
        Some((start, count)) => (start, count),
        None => (spec, "1"),
    };
    let start = start
        .parse::<usize>()
        .map_err(|_| schema(format!("hunk range start {start:?} is not a number")))?;
    let count = count
        .parse::<usize>()
        .map_err(|_| schema(format!("hunk range count {count:?} is not a number")))?;
    if start == 0 && count > 0 {
        return Err(schema(
            "hunk range start 0 requires count 0 (insertion point form)",
        ));
    }
    Ok((start, count))
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Reject {
    failed_hunk: usize,
    reason: String,
    target_line: Option<usize>,
}

fn apply_hunks(old: &str, hunks: &[Hunk]) -> Result<String, Reject> {
    let old_lines = split_lines(old);
    let eol = dominant_ending(&old_lines);
    let mut out: Vec<Line> = Vec::new();
    let mut cursor = 0usize;
    let mut delta: isize = 0;

    for (index, hunk) in hunks.iter().enumerate() {
        let base = if hunk.old_count == 0 {
            if hunk.old_start == 0 {
                0
            } else {
                hunk.old_start
            }
        } else {
            hunk.old_start.saturating_sub(1)
        };
        let position = base as isize + delta;
        if position < 0 || position as usize > old_lines.len() {
            return Err(Reject {
                failed_hunk: index + 1,
                reason: format!(
                    "hunk starts at line {} but the target has {} lines",
                    hunk.old_start,
                    old_lines.len()
                ),
                target_line: Some(hunk.old_start),
            });
        }
        let position = position as usize;
        if position < cursor {
            return Err(Reject {
                failed_hunk: index + 1,
                reason: format!(
                    "hunk starts before the end of the previous hunk (line {} < {})",
                    position + 1,
                    cursor + 1
                ),
                target_line: Some(position + 1),
            });
        }
        out.extend_from_slice(&old_lines[cursor..position]);
        cursor = position;

        for op in &hunk.ops {
            match op.kind {
                OpKind::Context | OpKind::Remove => {
                    let Some(line) = old_lines.get(cursor) else {
                        return Err(Reject {
                            failed_hunk: index + 1,
                            reason: format!(
                                "context mismatch: the hunk expects {:?} but the target ended at line {}",
                                quote(&op.text),
                                cursor
                            ),
                            target_line: Some(cursor + 1),
                        });
                    };
                    if line.text != op.text {
                        return Err(Reject {
                            failed_hunk: index + 1,
                            reason: format!(
                                "context mismatch: expected {:?} but found {:?}",
                                quote(&op.text),
                                quote(&line.text)
                            ),
                            target_line: Some(cursor + 1),
                        });
                    }
                    if op.no_newline && line.ending != Ending::None {
                        return Err(Reject {
                            failed_hunk: index + 1,
                            reason: format!(
                                "context mismatch: {:?} is declared without a trailing newline \
                                 but the target line has one",
                                quote(&op.text)
                            ),
                            target_line: Some(cursor + 1),
                        });
                    }
                    if op.kind == OpKind::Context {
                        out.push(line.clone());
                    }
                    cursor += 1;
                }
                OpKind::Add => {
                    let ending = if op.no_newline { Ending::None } else { eol };
                    out.push(Line {
                        text: op.text.clone(),
                        ending,
                    });
                }
            }
        }
        delta += hunk.new_count as isize - hunk.old_count as isize;
    }

    out.extend_from_slice(&old_lines[cursor..]);
    Ok(render_lines(&out, eol))
}

fn quote(text: &str) -> String {
    if text.len() <= MAX_QUOTED_BYTES {
        return text.to_owned();
    }
    let mut end = MAX_QUOTED_BYTES;
    while end > 0 && !text.is_char_boundary(end) {
        end -= 1;
    }
    format!("{}…", &text[..end])
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

    fn read(jail: &WorkspaceJail, path: &str) -> Vec<u8> {
        fs::read(jail.resolve(path).expect("resolve")).expect("read")
    }

    const MULTI_HUNK: &str = "\
--- a/notes.txt
+++ b/notes.txt
@@ -1,3 +1,3 @@
 one
-two
+TWO
 three
@@ -5,3 +5,4 @@
 five
 six
+six-and-a-half
 seven
";

    #[test]
    fn applies_multi_hunk_patch_with_offsets() {
        let (_temp, jail) = jail();
        jail.write("notes.txt", b"one\ntwo\nthree\nfour\nfive\nsix\nseven\n")
            .expect("seed");
        let outcome =
            apply_patch(&jail, &PatchRequest::new("notes.txt", MULTI_HUNK)).expect("apply");
        match outcome {
            PatchOutcome::Applied {
                hunks, new_bytes, ..
            } => {
                assert_eq!(hunks, 2);
                assert_eq!(
                    new_bytes,
                    "one\nTWO\nthree\nfour\nfive\nsix\nsix-and-a-half\nseven\n".len()
                );
            }
            other => panic!("expected apply, got {other:?}"),
        }
        assert_eq!(
            read(&jail, "notes.txt"),
            b"one\nTWO\nthree\nfour\nfive\nsix\nsix-and-a-half\nseven\n"
        );
    }

    #[test]
    fn context_mismatch_returns_failed_hunk_and_is_byte_identical() {
        let (_temp, jail) = jail();
        let original = b"one\ntwo\nthree\nfour\nfive\nsix\nseven\n";
        jail.write("notes.txt", original).expect("seed");
        let bad = MULTI_HUNK.replace("-two", "-nope");
        let outcome = apply_patch(&jail, &PatchRequest::new("notes.txt", bad)).expect("result");
        match outcome {
            PatchOutcome::Rejected {
                failed_hunk,
                target_line,
                ..
            } => {
                assert_eq!(failed_hunk, 1);
                assert_eq!(target_line, Some(2));
            }
            other => panic!("expected rejection, got {other:?}"),
        }
        assert_eq!(read(&jail, "notes.txt"), original);
    }

    #[test]
    fn expected_sha_matches_and_mismatch_writes_nothing() {
        let (_temp, jail) = jail();
        let original = b"one\ntwo\nthree\nfour\nfive\nsix\nseven\n";
        jail.write("notes.txt", original).expect("seed");
        let good = sha256_hex(original);
        let patch = PatchRequest::new("notes.txt", MULTI_HUNK).with_expected_sha256(good);
        assert!(matches!(
            apply_patch(&jail, &patch).expect("apply"),
            PatchOutcome::Applied { .. }
        ));

        jail.write("notes.txt", original).expect("reseed");
        let stale = PatchRequest::new("notes.txt", MULTI_HUNK).with_expected_sha256("0".repeat(64));
        let error = apply_patch(&jail, &stale).expect_err("stale");
        assert_eq!(error.code(), "E_SCHEMA_INVALID");
        assert_eq!(read(&jail, "notes.txt"), original);

        let malformed = PatchRequest::new("notes.txt", MULTI_HUNK).with_expected_sha256("not-hex");
        let error = apply_patch(&jail, &malformed).expect_err("malformed sha");
        assert_eq!(error.code(), "E_SCHEMA_INVALID");
        assert_eq!(read(&jail, "notes.txt"), original);
    }

    #[test]
    fn malformed_hunk_never_touches_the_file() {
        let (_temp, jail) = jail();
        let original = b"a\nb\nc\n";
        jail.write("f.txt", original).expect("seed");
        for bad in [
            "@@ -1,5 +1,5 @@\n a\n-b\n+B\n",
            "@@ -1 +1 @@\n-a\n",
            "@@ -1,1 +1,1 @@\n? a\n",
            "--- a/f.txt\n+++ b/f.txt\n@@ -1,1 +1,1 @@\n",
        ] {
            let error = apply_patch(&jail, &PatchRequest::new("f.txt", bad)).expect_err(bad);
            assert_eq!(error.code(), "E_TOOL_SCHEMA", "{bad:?}");
            assert_eq!(read(&jail, "f.txt"), original, "{bad:?}");
        }
    }

    #[test]
    fn header_paths_must_match_the_target() {
        let (_temp, jail) = jail();
        jail.write("f.txt", b"a\n").expect("seed");
        let patch = "--- a/other.txt\n+++ b/other.txt\n@@ -1 +1 @@\n-a\n+b\n";
        let error = apply_patch(&jail, &PatchRequest::new("f.txt", patch)).expect_err("mismatch");
        assert_eq!(error.code(), "E_TOOL_SCHEMA");
        assert_eq!(read(&jail, "f.txt"), b"a\n");
    }

    #[test]
    fn multi_file_and_create_delete_patches_are_refused() {
        let (_temp, jail) = jail();
        jail.write("f.txt", b"a\n").expect("seed");
        let two_files = "--- a/f.txt\n+++ b/f.txt\n@@ -1 +1 @@\n-a\n+b\n\
                         --- a/g.txt\n+++ b/g.txt\n@@ -1 +1 @@\n-c\n+d\n";
        let error = apply_patch(&jail, &PatchRequest::new("f.txt", two_files)).expect_err("two");
        assert_eq!(error.code(), "E_TOOL_SCHEMA");

        let created = "--- /dev/null\n+++ b/new.txt\n@@ -0,0 +1 @@\n+new\n";
        let error = apply_patch(&jail, &PatchRequest::new("new.txt", created)).expect_err("new");
        assert_eq!(error.code(), "E_TOOL_SCHEMA");
    }

    #[test]
    fn crlf_and_missing_final_newline_are_preserved() {
        let (_temp, jail) = jail();
        jail.write("crlf.txt", b"a\r\nb\r\nc\r\n").expect("seed");
        let crlf_patch = "--- a/crlf.txt\n+++ b/crlf.txt\n@@ -1,3 +1,3 @@\n a\n-b\n+B\n c\n";
        apply_patch(&jail, &PatchRequest::new("crlf.txt", crlf_patch)).expect("apply");
        assert_eq!(read(&jail, "crlf.txt"), b"a\r\nB\r\nc\r\n");

        jail.write("eof.txt", b"a\nb").expect("seed");
        let eof_patch = "--- a/eof.txt\n+++ b/eof.txt\n@@ -1,2 +1,2 @@\n a\n-b\n+B\n\\ No newline at end of file\n";
        apply_patch(&jail, &PatchRequest::new("eof.txt", eof_patch)).expect("apply");
        assert_eq!(read(&jail, "eof.txt"), b"a\nB");
    }

    #[test]
    fn insertion_hunk_uses_the_zero_count_position() {
        let (_temp, jail) = jail();
        jail.write("f.txt", b"a\nb\n").expect("seed");
        let patch = "--- a/f.txt\n+++ b/f.txt\n@@ -2,0 +3,1 @@\n+inserted\n";
        apply_patch(&jail, &PatchRequest::new("f.txt", patch)).expect("apply");
        assert_eq!(read(&jail, "f.txt"), b"a\nb\ninserted\n");
    }

    #[test]
    fn binary_target_and_escaping_paths_are_refused() {
        let (_temp, jail) = jail();
        fs::write(jail.root().join("bin.dat"), [0xff, 0xfe, 0x00]).expect("seed");
        let patch = "--- a/bin.dat\n+++ b/bin.dat\n@@ -1 +1 @@\n-a\n+b\n";
        let error = apply_patch(&jail, &PatchRequest::new("bin.dat", patch)).expect_err("binary");
        assert_eq!(error.code(), "E_TOOL_SCHEMA");

        let error =
            apply_patch(&jail, &PatchRequest::new("../escape.txt", patch)).expect_err("escape");
        assert_eq!(error.code(), "E_PATH_OUTSIDE_JAIL");
    }

    #[test]
    fn patch_without_hunks_and_oversize_patch_are_refused() {
        let (_temp, jail) = jail();
        jail.write("f.txt", b"a\n").expect("seed");
        let error = apply_patch(
            &jail,
            &PatchRequest::new("f.txt", "--- a/f.txt\n+++ b/f.txt\n"),
        )
        .expect_err("no hunks");
        assert_eq!(error.code(), "E_TOOL_SCHEMA");

        let mut huge = String::with_capacity(MAX_PATCH_BYTES + 1);
        huge.push_str("--- a/f.txt\n+++ b/f.txt\n");
        huge.push_str(&"x".repeat(MAX_PATCH_BYTES));
        let error = apply_patch(&jail, &PatchRequest::new("f.txt", huge)).expect_err("huge");
        assert_eq!(error.code(), "E_FILE_TOO_LARGE");
        assert_eq!(read(&jail, "f.txt"), b"a\n");
    }

    #[test]
    fn hunk_counts_are_validated() {
        let (_temp, jail) = jail();
        jail.write("f.txt", b"a\nb\n").expect("seed");
        let short = "@@ -1,2 +1,2 @@\n a\n";
        let error = apply_patch(&jail, &PatchRequest::new("f.txt", short)).expect_err("short");
        assert_eq!(error.code(), "E_TOOL_SCHEMA");
        assert_eq!(read(&jail, "f.txt"), b"a\nb\n");
    }
}
