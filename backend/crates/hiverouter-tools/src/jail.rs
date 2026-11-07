//! Windows workspace jail (Phase 6 WPs 6.8/6.12/6.14, CANONICAL C18).
//!
//! One containment owner: a canonicalised workspace root plus refusal of every
//! escape vector — **refused, never clamped** (a clamped path is a silent
//! reinterpretation of agent input):
//! 1. `..` parent traversal;
//! 2. absolute paths / drive-qualified paths (`C:\...`, `D:file`);
//! 3. symlink / junction targets that canonicalise outside the root;
//! 4. UNC / device prefixes (`\\server\share`, `\\?\C:\...`);
//! 5. drive changes;
//! 6. 8.3 short-name aliases (`PROGRA~1`) — conservatively refused because they
//!    resolve to a different lexical path.
//!
//! `cap-std 4.0.3` is SPIKE-GATED and not adopted (FACT F10): this module is the
//! single jail primitive and is proven by the escape-matrix test on this
//! Windows target. Child-process containment is the win32job Job Object in
//! [`crate::shell`] (one containment owner: win32job, not cap-std).

use std::fs;
use std::io::Write as _;
use std::path::{Component, Path, PathBuf};

use hiverouter_contract::Refusal;
use serde::Serialize;

use crate::error::ToolError;

/// Maximum bytes a single `file_read` may return.
///
/// The Lead's UI-COMMANDS contract patch fixes the `FILE_READ` cap at 1 MiB
/// (`E_FILE_TOO_LARGE`); this constant is the single cap for both the command
/// surface and the jail helper.
pub const MAX_READ_BYTES: u64 = 1024 * 1024;

/// A symlink/junction-resolved path outside the workspace.
#[must_use]
pub struct WorkspaceJail {
    root: PathBuf,
}

impl std::fmt::Debug for WorkspaceJail {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("WorkspaceJail")
            .field("root", &self.root)
            .finish()
    }
}

impl WorkspaceJail {
    /// Opens (creating when missing) the canonical workspace root.
    ///
    /// # Errors
    /// [`Refusal::PathOutsideJail`] when `root` is not an absolute path;
    /// [`Refusal::InternalError`] when the directory cannot be created or
    /// canonicalised.
    pub fn open(root: impl AsRef<Path>) -> Result<Self, ToolError> {
        let root = root.as_ref();
        if !root.is_absolute() {
            return Err(ToolError::new(
                Refusal::PathOutsideJail,
                "workspace root must be an absolute path",
            ));
        }
        fs::create_dir_all(root).map_err(|error| {
            ToolError::new(
                Refusal::InternalError,
                format!("cannot create the workspace root: {error}"),
            )
        })?;
        let canonical = fs::canonicalize(root).map_err(|error| {
            ToolError::new(
                Refusal::InternalError,
                format!("cannot canonicalise the workspace root: {error}"),
            )
        })?;
        Ok(Self { root: canonical })
    }

    /// The canonical root.
    #[must_use]
    pub fn root(&self) -> &Path {
        &self.root
    }

    /// Resolves a workspace-relative path, refusing every escape vector.
    ///
    /// The returned path is under the canonical root; if the target exists its
    /// canonical form is checked (catching symlink/junction escapes), otherwise
    /// the nearest existing ancestor is checked.
    ///
    /// # Errors
    /// [`Refusal::PathOutsideJail`] for every escape vector (no silent clamping).
    pub fn resolve(&self, requested: &str) -> Result<PathBuf, ToolError> {
        if requested.contains('\0') {
            return Err(escape("path contains a NUL byte"));
        }
        if requested.is_empty() || requested == "." || requested == "./" {
            return Ok(self.root.clone());
        }
        if requested.starts_with(r"\\") || requested.starts_with("//") {
            return Err(escape("UNC or device path is outside the workspace jail"));
        }

        let path = Path::new(requested);
        let mut relative = PathBuf::new();
        for component in path.components() {
            match component {
                Component::Prefix(_) => {
                    return Err(escape("drive/device prefix is outside the workspace jail"));
                }
                Component::RootDir => {
                    return Err(escape(
                        "rooted (absolute) path is outside the workspace jail",
                    ));
                }
                Component::ParentDir => {
                    return Err(escape("parent traversal ('..') is refused, not clamped"));
                }
                Component::CurDir => {}
                Component::Normal(segment) => {
                    let text = segment.to_string_lossy();
                    if text.contains(':') {
                        return Err(escape(
                            "path segment contains ':' (drive or alternate data stream)",
                        ));
                    }
                    if text.contains('~') {
                        return Err(escape(
                            "path segment contains '~' (8.3 short-name alias) and is refused",
                        ));
                    }
                    if text == "." || text == ".." {
                        return Err(escape("dot segment is refused"));
                    }
                    relative.push(segment);
                }
            }
        }
        let joined = self.root.join(&relative);
        self.verify_containment(&joined)?;
        Ok(joined)
    }

    /// Verifies that `candidate` (or its nearest existing ancestor) is inside
    /// the canonical root, resolving symlinks and junctions.
    fn verify_containment(&self, candidate: &Path) -> Result<(), ToolError> {
        let mut probe = candidate;
        loop {
            if probe.exists() {
                let canonical = fs::canonicalize(probe).map_err(|error| {
                    ToolError::new(
                        Refusal::PathOutsideJail,
                        format!("cannot resolve the path for containment check: {error}"),
                    )
                })?;
                if !self.is_inside(&canonical) {
                    return Err(escape(
                        "resolved path (through a symlink/junction) is outside the workspace root",
                    ));
                }
                return Ok(());
            }
            match probe.parent() {
                Some(parent) => probe = parent,
                None => return Err(escape("path has no existing ancestor inside the jail")),
            }
        }
    }

    /// Whether a canonical path is the root or inside it (Windows:
    /// case-insensitive, component-boundary safe).
    #[must_use]
    pub fn is_inside(&self, canonical: &Path) -> bool {
        let root = self.root.to_string_lossy().to_lowercase();
        let candidate = canonical.to_string_lossy().to_lowercase();
        candidate == root
            || candidate.starts_with(&format!("{root}\\"))
            || candidate.starts_with(&format!("{root}/"))
    }

    /// Reads a workspace file.
    ///
    /// # Errors
    /// [`Refusal::PathOutsideJail`] for escapes, [`Refusal::PathNotFound`] when
    /// the file does not exist, [`Refusal::FileTooLarge`] when it exceeds
    /// [`MAX_READ_BYTES`], [`Refusal::ToolSchema`] for a non-regular file, and
    /// [`Refusal::InternalError`] for I/O failures.
    pub fn read(&self, requested: &str) -> Result<Vec<u8>, ToolError> {
        let path = self.resolve(requested)?;
        let metadata = fs::metadata(&path).map_err(|error| {
            ToolError::new(
                Refusal::PathNotFound,
                format!("cannot read {requested:?}: {error}"),
            )
        })?;
        if !metadata.is_file() {
            return Err(ToolError::new(
                Refusal::ToolSchema,
                format!("{requested:?} is not a regular file"),
            ));
        }
        if metadata.len() > MAX_READ_BYTES {
            return Err(ToolError::new(
                Refusal::FileTooLarge,
                format!(
                    "{requested:?} is {} bytes; the read cap is {MAX_READ_BYTES} bytes",
                    metadata.len()
                ),
            ));
        }
        fs::read(&path).map_err(|error| {
            ToolError::new(
                Refusal::InternalError,
                format!("I/O failure reading {requested:?}: {error}"),
            )
        })
    }

    /// Writes a workspace file (parents created inside the jail).
    ///
    /// The caller is responsible for the approval gate: this function is only
    /// reachable through an approved `file_write`.
    ///
    /// # Errors
    /// [`Refusal::PathOutsideJail`] for escapes; [`Refusal::InternalError`] for I/O.
    pub fn write(&self, requested: &str, bytes: &[u8]) -> Result<PathBuf, ToolError> {
        let path = self.resolve(requested)?;
        if let Some(parent) = path.parent() {
            if !self.is_inside(&fs::canonicalize(parent).unwrap_or_else(|_| parent.to_path_buf())) {
                return Err(escape("write parent resolves outside the workspace root"));
            }
            fs::create_dir_all(parent).map_err(|error| {
                ToolError::new(
                    Refusal::InternalError,
                    format!("cannot create parent directory for {requested:?}: {error}"),
                )
            })?;
        }
        fs::write(&path, bytes).map_err(|error| {
            ToolError::new(
                Refusal::InternalError,
                format!("I/O failure writing {requested:?}: {error}"),
            )
        })?;
        Ok(path)
    }

    /// Atomically replaces a workspace file: a unique temp file in the target's
    /// directory is written and fsynced, then renamed over the target.
    ///
    /// Refuses the workspace root, directories and non-regular existing targets,
    /// plus every escape vector [`Self::resolve`] already refuses. Parent
    /// directories are created inside the jail (same rule as [`Self::write`]).
    ///
    /// The caller owns both the approval gate and the content bound: this helper
    /// guarantees containment + atomic replacement only, so no partial file is
    /// ever visible under the target name.
    ///
    /// # Errors
    /// [`Refusal::PathOutsideJail`] for escapes, [`Refusal::ToolSchema`] for the
    /// root/directory/non-regular target, [`Refusal::InternalError`] for I/O.
    pub fn write_atomic(&self, requested: &str, bytes: &[u8]) -> Result<PathBuf, ToolError> {
        let target = self.resolve(requested)?;
        if target == self.root() {
            return Err(ToolError::new(
                Refusal::ToolSchema,
                "cannot atomically write the workspace root",
            ));
        }
        let parent = target.parent().ok_or_else(|| {
            ToolError::new(
                Refusal::InternalError,
                "write target has no parent directory",
            )
        })?;
        // Mirror `write`: check the parent's canonical containment, then create it.
        if !self.is_inside(&fs::canonicalize(parent).unwrap_or_else(|_| parent.to_path_buf())) {
            return Err(escape("write parent resolves outside the workspace root"));
        }
        fs::create_dir_all(parent).map_err(|error| {
            ToolError::new(
                Refusal::InternalError,
                format!("cannot create parent directory for {requested:?}: {error}"),
            )
        })?;
        match fs::symlink_metadata(&target) {
            Ok(metadata) if metadata.is_dir() => {
                return Err(ToolError::new(
                    Refusal::ToolSchema,
                    format!("{requested:?} is a directory"),
                ));
            }
            Ok(metadata) if !metadata.is_file() => {
                return Err(ToolError::new(
                    Refusal::ToolSchema,
                    format!("{requested:?} is not a regular file (symlink/non-file refused)"),
                ));
            }
            Ok(_) => {}
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
            Err(error) => return Err(error.into()),
        }

        // Same-directory temp keeps the rename on one volume; `create_new`
        // refuses to follow a pre-planted symlink and refuses collisions.
        let temp = parent.join(format!(".hive-write-{}.tmp", uuid::Uuid::now_v7()));
        let mut file = fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&temp)
            .map_err(|error| {
                ToolError::new(
                    Refusal::InternalError,
                    format!("cannot create the temporary write file: {error}"),
                )
            })?;
        let write_outcome = (|| -> Result<(), ToolError> {
            file.write_all(bytes).map_err(|error| {
                ToolError::new(
                    Refusal::InternalError,
                    format!("I/O failure writing the temporary file: {error}"),
                )
            })?;
            file.sync_all().map_err(|error| {
                ToolError::new(
                    Refusal::InternalError,
                    format!("cannot flush the temporary write file: {error}"),
                )
            })?;
            Ok(())
        })();
        drop(file);
        if let Err(error) = write_outcome {
            let _ = fs::remove_file(&temp);
            return Err(error);
        }
        // Windows `std::fs::rename` maps to MoveFileExW with replace-existing
        // semantics; the overwrite test proves it on this host. On failure the
        // stale temp is removed, never the target.
        if let Err(error) = fs::rename(&temp, &target) {
            let _ = fs::remove_file(&temp);
            return Err(ToolError::from(error));
        }
        Ok(target)
    }

    /// Lists a workspace directory (root by default).
    ///
    /// # Errors
    /// [`Refusal::PathOutsideJail`] for escapes; [`Refusal::ToolSchema`] when the
    /// path is not a directory.
    pub fn list(&self, requested: Option<&str>) -> Result<Vec<DirEntryInfo>, ToolError> {
        let path = self.resolve(requested.unwrap_or(""))?;
        let metadata = fs::metadata(&path).map_err(|error| {
            ToolError::new(
                Refusal::PathNotFound,
                format!("cannot list {}: {error}", requested.unwrap_or("<root>")),
            )
        })?;
        if !metadata.is_dir() {
            return Err(ToolError::new(
                Refusal::ToolSchema,
                "path is not a directory",
            ));
        }
        let mut entries = Vec::new();
        for entry in fs::read_dir(&path).map_err(|error| {
            ToolError::new(
                Refusal::InternalError,
                format!("I/O failure listing the directory: {error}"),
            )
        })? {
            let entry = entry.map_err(|error| {
                ToolError::new(
                    Refusal::InternalError,
                    format!("I/O failure reading a directory entry: {error}"),
                )
            })?;
            let metadata = entry.metadata().ok();
            let file_type = entry.file_type().ok();
            entries.push(DirEntryInfo {
                name: entry.file_name().to_string_lossy().into_owned(),
                is_dir: file_type.map(|kind| kind.is_dir()).unwrap_or(false),
                size: metadata.map(|meta| meta.len()).unwrap_or(0),
            });
        }
        entries.sort_by(|left, right| left.name.cmp(&right.name));
        Ok(entries)
    }
}

/// One directory entry returned by [`WorkspaceJail::list`].
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct DirEntryInfo {
    /// Entry name (no path; always inside the jail).
    pub name: String,
    /// Whether the entry is a directory.
    pub is_dir: bool,
    /// File size in bytes (0 for directories).
    pub size: u64,
}

fn escape(detail: &str) -> ToolError {
    ToolError::new(Refusal::PathOutsideJail, detail)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    fn jail() -> (tempfile::TempDir, WorkspaceJail) {
        let temp = tempfile::tempdir().expect("tempdir");
        let jail = WorkspaceJail::open(temp.path()).expect("jail");
        (temp, jail)
    }

    #[test]
    fn reads_writes_and_lists_inside_the_root() {
        let (_temp, jail) = jail();
        fs::create_dir_all(jail.root().join("docs")).expect("dir");
        jail.write("docs/note.txt", b"hello hive").expect("write");
        assert_eq!(jail.read("docs/note.txt").expect("read"), b"hello hive");
        let entries = jail.list(Some("docs")).expect("list");
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].name, "note.txt");
        assert!(!entries[0].is_dir);
    }

    #[test]
    fn escape_vector_parent_traversal_is_refused() {
        let (_temp, jail) = jail();
        for attempt in [
            "..",
            "../escape.txt",
            "a/../../escape.txt",
            "docs/../../../x",
        ] {
            let error = jail.resolve(attempt).expect_err(attempt);
            assert_eq!(error.code(), "E_PATH_OUTSIDE_JAIL", "vector: {attempt}");
        }
    }

    #[test]
    fn escape_vector_absolute_path_is_refused() {
        let (_temp, jail) = jail();
        for attempt in [
            "C:\\Windows\\notepad.exe",
            "C:/Windows/notepad.exe",
            "\\\\?\\C:\\Windows",
            "/etc/passwd",
        ] {
            let error = jail.resolve(attempt).expect_err(attempt);
            assert_eq!(error.code(), "E_PATH_OUTSIDE_JAIL", "vector: {attempt}");
        }
    }

    #[test]
    fn escape_vector_unc_and_device_prefixes_are_refused() {
        let (_temp, jail) = jail();
        for attempt in [
            r"\\server\share\file.txt",
            r"\\?\C:\Windows\win.ini",
            r"\\.\PhysicalDrive0",
        ] {
            let error = jail.resolve(attempt).expect_err(attempt);
            assert_eq!(error.code(), "E_PATH_OUTSIDE_JAIL", "vector: {attempt}");
        }
    }

    #[test]
    fn escape_vector_drive_change_is_refused() {
        let (_temp, jail) = jail();
        for attempt in ["D:\\data\\x.txt", "D:relative.txt", "E:/other/file"] {
            let error = jail.resolve(attempt).expect_err(attempt);
            assert_eq!(error.code(), "E_PATH_OUTSIDE_JAIL", "vector: {attempt}");
        }
    }

    #[test]
    fn escape_vector_short_name_alias_is_refused() {
        let (_temp, jail) = jail();
        for attempt in ["PROGRA~1\\evil.txt", "docs\\FILE~1.TXT", "a~b.txt"] {
            let error = jail.resolve(attempt).expect_err(attempt);
            assert_eq!(error.code(), "E_PATH_OUTSIDE_JAIL", "vector: {attempt}");
        }
    }

    #[cfg(windows)]
    #[test]
    fn escape_vector_symlink_or_junction_is_refused() {
        let (_temp, jail) = jail();
        let outside = tempfile::tempdir().expect("outside tempdir");
        let mut secret = fs::File::create(outside.path().join("secret.txt")).expect("secret");
        secret.write_all(b"outside the jail").expect("write secret");

        let link = jail.root().join("link");
        // Junction creation via cmd (no administrator privilege required on NTFS).
        let status = std::process::Command::new("cmd")
            .args([
                "/C",
                "mklink",
                "/J",
                &link.to_string_lossy(),
                &outside.path().to_string_lossy(),
            ])
            .output()
            .expect("run mklink");
        if !status.status.success() {
            // Fallback: directory symlink (needs developer mode); if it also
            // fails the vector cannot be exercised on this host.
            let symlink = std::os::windows::fs::symlink_dir(outside.path(), &link);
            assert!(
                symlink.is_ok(),
                "cannot create a junction or symlink to exercise the escape vector: {}",
                String::from_utf8_lossy(&status.stderr)
            );
        }

        let error = jail
            .resolve("link/secret.txt")
            .expect_err("symlink escape refused");
        assert_eq!(error.code(), "E_PATH_OUTSIDE_JAIL");
        let error = jail.read("link/secret.txt").expect_err("read refused");
        assert_eq!(error.code(), "E_PATH_OUTSIDE_JAIL");
        // The escaped content must never be returned.
        assert!(!format!("{error}").contains("outside the jail"));
    }

    #[test]
    fn read_cap_is_enforced() {
        let (_temp, jail) = jail();
        let bytes = vec![b'x'; usize::try_from(MAX_READ_BYTES).expect("fits") + 1];
        jail.write("big.bin", &bytes).expect("write");
        let error = jail.read("big.bin").expect_err("too large");
        assert_eq!(error.code(), "E_FILE_TOO_LARGE");
    }

    #[test]
    fn write_with_approved_relative_path_stays_inside() {
        let (_temp, jail) = jail();
        let path = jail.write("nested/dir/file.txt", b"ok").expect("write");
        let root = fs::canonicalize(jail.root()).expect("root canonical");
        assert!(path.starts_with(&root));
        assert_eq!(fs::read(path).expect("read back"), b"ok");
    }

    fn no_temp_residue(dir: &Path) -> bool {
        fs::read_dir(dir).expect("read dir").all(|entry| {
            !entry
                .expect("dir entry")
                .file_name()
                .to_string_lossy()
                .starts_with(".hive-write-")
        })
    }

    #[test]
    fn write_atomic_creates_and_replaces_file() {
        let (temp, jail) = jail();
        let first = jail
            .write_atomic("notes/a.txt", b"one")
            .expect("first write");
        assert_eq!(fs::read(&first).expect("read back"), b"one");
        let root = fs::canonicalize(jail.root()).expect("root canonical");
        assert!(first.starts_with(&root));

        let second = jail.write_atomic("notes/a.txt", b"two").expect("replace");
        assert_eq!(second, first, "same target path");
        assert_eq!(fs::read(&second).expect("read back"), b"two");
        assert!(
            no_temp_residue(&temp.path().join("notes")),
            "atomic write must not leave a .hive-write-*.tmp behind"
        );
    }

    #[test]
    fn write_atomic_creates_nested_parents_inside_jail() {
        let (_temp, jail) = jail();
        let path = jail
            .write_atomic("a/b/c.txt", b"nested")
            .expect("nested write");
        let root = fs::canonicalize(jail.root()).expect("root canonical");
        assert!(path.starts_with(&root));
        assert_eq!(fs::read(path).expect("read back"), b"nested");
    }

    #[test]
    fn write_atomic_refuses_root_and_directory_targets() {
        let (_temp, jail) = jail();
        for target in ["", ".", "./"] {
            let error = jail.write_atomic(target, b"x").expect_err(target);
            assert_eq!(error.code(), "E_TOOL_SCHEMA", "root target: {target:?}");
        }
        fs::create_dir_all(jail.root().join("dir")).expect("dir");
        let error = jail
            .write_atomic("dir", b"x")
            .expect_err("directory target");
        assert_eq!(error.code(), "E_TOOL_SCHEMA");
    }

    #[test]
    fn write_atomic_escape_vectors_stay_refused() {
        let (_temp, jail) = jail();
        for attempt in ["../x.txt", "C:\\x.txt", "PROGRA~1\\x.txt", "a\0b.txt"] {
            let error = jail.write_atomic(attempt, b"x").expect_err(attempt);
            assert_eq!(error.code(), "E_PATH_OUTSIDE_JAIL", "vector: {attempt:?}");
        }
    }
}
