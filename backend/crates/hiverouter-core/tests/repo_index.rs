//! Behavioral tests for the local repository index.
//!
//! Coverage:
//! * deterministic walk and ignore rules (`.git`, `target`, `node_modules`);
//! * refusal of binary, invalid-UTF-8, oversized, and symlinked files;
//! * a real `CREATE VIRTUAL TABLE ... USING fts5` probe plus the fail-closed
//!   branch for an absent FTS5 module;
//! * index/search round trips, re-indexing, and chunk line ranges;
//! * bounded glob/grep/read_range helpers;
//! * workspace-jail rejection of traversal, absolute paths, ADS, devices, and
//!   escaping symlinks.

use std::fs;
use std::path::Path;

use hiverouter_core::repo_index::{
    self, RepoIndexError, WalkOptions, Workspace, MAX_FILE_BYTES, MAX_GLOB_RESULTS,
    MAX_GREP_LINE_BYTES, MAX_GREP_MATCHES, MAX_INDEXED_BYTES, MAX_READ_BYTES, MAX_READ_LINES,
    MAX_SEARCH_LIMIT, MAX_SEARCH_QUERY_BYTES,
};
use rusqlite::Connection;
use tempfile::TempDir;

fn database() -> Connection {
    Connection::open_in_memory().expect("in-memory SQLite")
}

fn open_workspace() -> (TempDir, Workspace) {
    let directory = tempfile::tempdir().expect("temporary workspace");
    let workspace = Workspace::open(directory.path()).expect("workspace");
    (directory, workspace)
}

fn write_file(root: &Path, rel_path: &str, contents: &str) {
    let path = root.join(rel_path);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).expect("parent directory");
    }
    fs::write(path, contents).expect("write file");
}

fn write_bytes(root: &Path, rel_path: &str, contents: &[u8]) {
    let path = root.join(rel_path);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).expect("parent directory");
    }
    fs::write(path, contents).expect("write bytes");
}

fn indexed_paths(connection: &Connection) -> Vec<String> {
    let mut statement = connection
        .prepare("SELECT rel_path FROM repo_files ORDER BY rel_path")
        .expect("prepare path query");
    statement
        .query_map([], |row| row.get(0))
        .expect("query paths")
        .map(|row| row.expect("path row"))
        .collect()
}

fn count(connection: &Connection, sql: &str) -> i64 {
    connection
        .query_row(sql, [], |row| row.get(0))
        .expect("count query")
}

#[cfg(unix)]
fn try_symlink(target: &Path, link: &Path) -> bool {
    std::os::unix::fs::symlink(target, link).is_ok()
}

#[cfg(windows)]
fn try_symlink(target: &Path, link: &Path) -> bool {
    std::os::windows::fs::symlink_file(target, link).is_ok()
}

#[test]
fn walk_ignores_vcs_build_and_dependency_trees_in_deterministic_order() {
    let (directory, workspace) = open_workspace();
    let root = directory.path();
    write_file(root, "README.md", "readme");
    write_file(root, "src/main.rs", "fn main() {}");
    write_file(root, "src/lib.rs", "pub fn lib() {}");
    write_file(root, ".git/config", "[core]");
    write_file(root, "target/debug/app", "artifact");
    write_file(root, "node_modules/pkg/index.js", "module.exports = 1;");
    write_file(root, "src/.git/HEAD", "ref");
    write_file(root, "src/node_modules/nested.js", "ignored");
    write_file(root, "src/Target/obj.txt", "ignored case-insensitively");

    let connection = database();
    let report = repo_index::index_workspace(&connection, root).expect("index workspace");

    assert_eq!(
        indexed_paths(&connection),
        vec!["README.md", "src/lib.rs", "src/main.rs"],
        "only source files outside ignored trees may be indexed"
    );
    assert_eq!(report.files_indexed, 3);
    assert_eq!(
        report.dirs_skipped, 6,
        ".git/target/node_modules at both levels"
    );
    assert!(!report.truncated);

    // Re-indexing must not duplicate rows and must keep the same order.
    repo_index::index_workspace(&connection, root).expect("re-index workspace");
    assert_eq!(
        indexed_paths(&connection),
        vec!["README.md", "src/lib.rs", "src/main.rs"]
    );
    assert_eq!(count(&connection, "SELECT COUNT(*) FROM repo_files"), 3);

    // The filesystem helpers use the same ignore rules.
    let globbed = workspace.glob("**/*.rs", 10).expect("glob");
    assert_eq!(globbed.paths, vec!["src/lib.rs", "src/main.rs"]);
}

#[test]
fn binary_invalid_utf8_and_oversized_files_are_refused() {
    let (directory, workspace) = open_workspace();
    let root = directory.path();
    write_file(root, "ok.txt", "plain text");
    write_bytes(root, "blob.bin", &[0x00, 0x01, 0x02, 0x03]);
    write_bytes(root, "latin1.txt", &[0xC3, 0x28]); // invalid UTF-8, no NUL
    write_bytes(
        root,
        "huge.txt",
        &vec![b'a'; usize::try_from(MAX_FILE_BYTES).unwrap() + 1],
    );

    let connection = database();
    let report = repo_index::index_workspace(&connection, root).expect("index workspace");

    assert_eq!(indexed_paths(&connection), vec!["ok.txt"]);
    assert_eq!(report.files_indexed, 1);
    assert_eq!(report.files_skipped_binary, 2, "NUL and invalid UTF-8");
    assert_eq!(report.files_skipped_large, 1);

    assert!(matches!(
        workspace.read_text("blob.bin"),
        Err(RepoIndexError::NotText { .. })
    ));
    assert!(matches!(
        workspace.read_text("latin1.txt"),
        Err(RepoIndexError::NotText { .. })
    ));
    assert!(matches!(
        workspace.read_text("huge.txt"),
        Err(RepoIndexError::FileTooLarge { .. })
    ));
    assert!(matches!(
        workspace.read_range("blob.bin", 1, 1),
        Err(RepoIndexError::NotText { .. })
    ));
}

#[test]
fn fts5_probe_is_real_and_fail_closed_for_an_absent_module() {
    let connection = database();

    // Real probe: the test itself creates an FTS5 virtual table and queries it,
    // so this can only pass when the linked SQLite build actually has FTS5.
    repo_index::probe_fts5(&connection).expect("bundled SQLite must provide FTS5");
    connection
        .execute_batch(
            "CREATE VIRTUAL TABLE temp.manual_probe USING fts5(body);
             INSERT INTO temp.manual_probe(body) VALUES ('hive mind');",
        )
        .expect("manual FTS5 table");
    let hits: i64 = connection
        .query_row(
            "SELECT COUNT(*) FROM temp.manual_probe WHERE manual_probe MATCH 'hive'",
            [],
            |row| row.get(0),
        )
        .expect("manual FTS5 query");
    assert_eq!(hits, 1);

    // Fail closed: SQLite reports "no such module" for an unknown module and
    // the probe must surface that as Fts5Unavailable, never as a soft fallback.
    let error = repo_index::probe_fts5_module(&connection, "fts5_absent_probe")
        .expect_err("absent FTS5 module must fail closed");
    assert!(
        matches!(error, RepoIndexError::Fts5Unavailable { .. }),
        "unexpected probe error: {error}"
    );

    // Search without schema/indexing also fails closed instead of scanning text.
    let error = repo_index::search_chunks(&connection, "workspace", "hive", 5)
        .expect_err("search before schema must fail");
    assert!(
        matches!(error, RepoIndexError::SchemaMismatch(_)),
        "unexpected search error: {error}"
    );
}

#[test]
fn index_and_search_round_trip_reindexes_and_removes_stale_rows() {
    let (directory, _workspace) = open_workspace();
    let root = directory.path();
    write_file(
        root,
        "src/main.rs",
        "fn main() {\n    let alpha = 1;\n    // beta marker\n}\n",
    );
    write_file(root, "docs/notes.txt", "beta marker in documentation\n");

    let connection = database();
    let report = repo_index::index_workspace(&connection, root).expect("index");
    assert_eq!(report.files_indexed, 2);
    assert_eq!(
        count(&connection, "SELECT COUNT(*) FROM repo_chunks_fts"),
        i64::try_from(report.chunks_indexed).unwrap()
    );

    let hits = repo_index::search_workspace(&connection, root, "beta", 10).expect("search");
    assert_eq!(hits.len(), 2);
    assert!(hits
        .iter()
        .all(|hit| hit.snippet.to_lowercase().contains("beta")));
    assert!(hits.iter().any(|hit| hit.rel_path == "src/main.rs"));

    let phrase = repo_index::fts_phrase("beta marker");
    let phrase_hits =
        repo_index::search_workspace(&connection, root, &phrase, 10).expect("phrase search");
    assert!(!phrase_hits.is_empty());

    assert!(matches!(
        repo_index::search_workspace(&connection, root, "   ", 5),
        Err(RepoIndexError::InvalidQuery(_))
    ));
    assert!(matches!(
        repo_index::search_workspace(&connection, root, "\"unterminated", 5),
        Err(RepoIndexError::InvalidQuery(_))
    ));
    let oversized_query = "a".repeat(MAX_SEARCH_QUERY_BYTES + 1);
    assert!(matches!(
        repo_index::search_workspace(&connection, root, &oversized_query, 5),
        Err(RepoIndexError::InvalidQuery(_))
    ));
    assert!(matches!(
        repo_index::search_chunks(&connection, "workspace", "beta", 0),
        Err(RepoIndexError::InvalidLimit { .. })
    ));
    assert!(matches!(
        repo_index::search_chunks(&connection, "workspace", "beta", MAX_SEARCH_LIMIT + 1),
        Err(RepoIndexError::InvalidLimit { .. })
    ));

    // Remove one file and re-index: stale rows and FTS entries must disappear.
    fs::remove_file(root.join("docs/notes.txt")).expect("remove file");
    let report = repo_index::index_workspace(&connection, root).expect("re-index");
    assert_eq!(report.files_indexed, 1);
    assert_eq!(indexed_paths(&connection), vec!["src/main.rs"]);
    assert!(
        repo_index::search_workspace(&connection, root, "documentation", 10)
            .expect("search removed content")
            .is_empty()
    );
}

#[test]
fn long_files_are_chunked_with_exact_line_ranges() {
    let (directory, _workspace) = open_workspace();
    let root = directory.path();
    let mut body = String::new();
    for line in 1..=300 {
        body.push_str(&format!("line {line} filler\n"));
    }
    body.push_str("needle250 unique\n");
    write_file(root, "long.txt", &body);

    let connection = database();
    let report = repo_index::index_workspace(&connection, root).expect("index");
    assert_eq!(report.files_indexed, 1);
    assert!(
        report.chunks_indexed > 1,
        "300 lines must span multiple chunks"
    );
    assert_eq!(
        count(&connection, "SELECT COUNT(*) FROM repo_chunks"),
        i64::try_from(report.chunks_indexed).unwrap()
    );
    let (first, last): (i64, i64) = connection
        .query_row(
            "SELECT MIN(start_line), MAX(end_line) FROM repo_chunks",
            [],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .expect("chunk range");
    assert_eq!(first, 1);
    assert_eq!(last, 301);

    let hits = repo_index::search_workspace(&connection, root, "needle250", 5).expect("search");
    assert_eq!(hits.len(), 1);
    assert!(hits[0].start_line <= 301 && hits[0].end_line >= 301);
}

#[test]
fn grep_is_regex_aware_and_capped() {
    let (directory, workspace) = open_workspace();
    let root = directory.path();
    write_file(
        root,
        "src/a.txt",
        "needle 1\nneedle 2\nneedle 3\nother\nneedle 4\n",
    );
    write_file(root, ".git/secret.txt", "needle hidden\n");
    write_file(root, "node_modules/pkg/x.js", "needle hidden\n");
    write_bytes(root, "blob.bin", &[0x00, b'n', b'e']);

    let capped = workspace.grep(r"needle \d+", 2).expect("bounded grep");
    assert_eq!(capped.matches.len(), 2);
    assert!(capped.truncated, "remaining matches must be reported");
    assert_eq!(capped.matches[0].rel_path, "src/a.txt");
    assert_eq!(capped.matches[0].line_number, 1);
    assert_eq!(capped.matches[0].column, 1);
    assert!(capped.matches[0].line.contains("needle 1"));

    let all = workspace.grep(r"needle \d+", 10).expect("grep");
    assert_eq!(all.matches.len(), 4);
    assert!(!all.truncated);
    assert_eq!(
        all.files_scanned, 1,
        "ignored and binary files are not scanned"
    );

    let insensitive = workspace
        .grep("(?i)NEEDLE 4", 10)
        .expect("case-insensitive grep");
    assert_eq!(insensitive.matches.len(), 1);
    assert_eq!(insensitive.matches[0].line_number, 5);

    let long_line = format!("needle {}", "x".repeat(MAX_GREP_LINE_BYTES * 3));
    write_file(root, "long.txt", &format!("{long_line}\n"));
    let windowed = workspace.grep("needle", 10).expect("windowed grep");
    let hit = windowed
        .matches
        .iter()
        .find(|hit| hit.rel_path == "long.txt")
        .expect("long line hit");
    assert!(
        hit.line.len() <= MAX_GREP_LINE_BYTES + 6,
        "line window must be bounded"
    );
    assert!(hit.line.contains("needle"));

    assert!(matches!(
        workspace.grep("(", 10),
        Err(RepoIndexError::InvalidRegex(_))
    ));
    assert!(matches!(
        workspace.grep("needle", 0),
        Err(RepoIndexError::InvalidLimit { .. })
    ));
    assert!(matches!(
        workspace.grep("needle", MAX_GREP_MATCHES + 1),
        Err(RepoIndexError::InvalidLimit { .. })
    ));
}

#[test]
fn glob_supports_double_star_and_is_bounded() {
    let (directory, workspace) = open_workspace();
    let root = directory.path();
    write_file(root, "top.rs", "top");
    write_file(root, "src/a.rs", "a");
    write_file(root, "src/c.txt", "c");
    write_file(root, "src/nested/b.rs", "b");
    write_file(root, "src/deep/nested/d.rs", "d");
    write_file(root, ".git/hidden.rs", "hidden");
    write_file(root, "node_modules/pkg/index.js", "ignored");

    assert_eq!(
        workspace.glob("**/*.rs", 10).expect("glob").paths,
        vec![
            "src/a.rs",
            "src/deep/nested/d.rs",
            "src/nested/b.rs",
            "top.rs"
        ]
    );
    assert_eq!(
        workspace.glob("src/*.rs", 10).expect("glob").paths,
        vec!["src/a.rs"]
    );
    assert_eq!(
        workspace.glob("src/**/b.rs", 10).expect("glob").paths,
        vec!["src/nested/b.rs"]
    );
    assert_eq!(
        workspace.glob("src/**", 10).expect("glob").paths,
        vec![
            "src/a.rs",
            "src/c.txt",
            "src/deep/nested/d.rs",
            "src/nested/b.rs"
        ]
    );

    let bounded = workspace.glob("**/*", 1).expect("bounded glob");
    assert_eq!(bounded.paths.len(), 1);
    assert!(bounded.truncated);

    assert!(matches!(
        workspace.glob("[unclosed", 10),
        Err(RepoIndexError::InvalidGlob(_))
    ));
    assert!(matches!(
        workspace.glob("../*.rs", 10),
        Err(RepoIndexError::PathEscape(_))
    ));
    assert!(matches!(
        workspace.glob("/etc/*.rs", 10),
        Err(RepoIndexError::PathEscape(_))
    ));
    assert!(matches!(
        workspace.glob("a:b", 10),
        Err(RepoIndexError::InvalidGlob(_))
    ));
    assert!(matches!(
        workspace.glob("*.rs", 0),
        Err(RepoIndexError::InvalidLimit { .. })
    ));
    assert!(matches!(
        workspace.glob("*.rs", MAX_GLOB_RESULTS + 1),
        Err(RepoIndexError::InvalidLimit { .. })
    ));
}

#[test]
fn read_range_is_line_addressed_and_bounded() {
    let (directory, workspace) = open_workspace();
    let root = directory.path();
    write_file(root, "five.txt", "one\ntwo\nthree\nfour\nfive");
    write_file(root, "blank.txt", "a\n\nb");

    let middle = workspace.read_range("five.txt", 2, 2).expect("range");
    assert_eq!(middle.content, "two\nthree");
    assert_eq!(middle.start_line, 2);
    assert_eq!(middle.end_line, 3);
    assert_eq!(middle.total_lines, 5);
    assert!(middle.truncated, "lines remain after the requested range");

    let complete = workspace.read_range("five.txt", 1, 5).expect("range");
    assert_eq!(complete.content, "one\ntwo\nthree\nfour\nfive");
    assert!(!complete.truncated);
    assert_eq!(complete.end_line, 5);

    let past_end = workspace.read_range("five.txt", 10, 2).expect("range");
    assert!(past_end.content.is_empty());
    assert_eq!(past_end.end_line, 9);
    assert!(!past_end.truncated);

    let blank = workspace.read_range("blank.txt", 1, 3).expect("range");
    assert_eq!(blank.content, "a\n\nb");

    assert!(matches!(
        workspace.read_range("five.txt", 0, 1),
        Err(RepoIndexError::InvalidRange(_))
    ));
    assert!(matches!(
        workspace.read_range("five.txt", 1, 0),
        Err(RepoIndexError::InvalidLimit { .. })
    ));
    assert!(matches!(
        workspace.read_range("five.txt", 1, MAX_READ_LINES + 1),
        Err(RepoIndexError::InvalidLimit { .. })
    ));

    // The byte cap cuts inside a single oversized line without splitting UTF-8.
    write_file(root, "wide.txt", &"é".repeat(MAX_READ_BYTES / 2 + 128));
    let wide = workspace.read_range("wide.txt", 1, 1).expect("wide range");
    assert!(wide.content.len() <= MAX_READ_BYTES);
    assert!(wide.truncated);
    assert!(wide.content.chars().all(|character| character == 'é'));
}

#[test]
fn workspace_jail_rejects_traversal_absolute_ads_and_escaping_symlinks() {
    let (directory, workspace) = open_workspace();
    let root = directory.path();
    write_file(root, "inside.txt", "inside");
    let outside = tempfile::tempdir().expect("outside directory");
    write_file(outside.path(), "secret.txt", "outside secret");

    assert!(matches!(
        workspace.read_text("../secret.txt"),
        Err(RepoIndexError::PathEscape(_))
    ));
    assert!(matches!(
        workspace.read_range("src/../../inside.txt", 1, 1),
        Err(RepoIndexError::PathEscape(_))
    ));
    assert!(matches!(
        workspace.glob("../*.rs", 10),
        Err(RepoIndexError::PathEscape(_))
    ));

    let absolute = if cfg!(windows) {
        "C:\\Windows\\win.ini"
    } else {
        "/etc/passwd"
    };
    assert!(matches!(
        workspace.read_text(absolute),
        Err(RepoIndexError::InvalidRelativePath(_)) | Err(RepoIndexError::PathEscape(_))
    ));

    #[cfg(windows)]
    {
        assert!(matches!(
            workspace.read_text("inside.txt:stream"),
            Err(RepoIndexError::InvalidRelativePath(_))
        ));
        assert!(matches!(
            workspace.read_text("CON"),
            Err(RepoIndexError::InvalidRelativePath(_))
        ));
    }

    let link = root.join("escape.txt");
    if try_symlink(&outside.path().join("secret.txt"), &link) {
        assert!(
            matches!(
                workspace.read_text("escape.txt"),
                Err(RepoIndexError::PathEscape(_))
            ),
            "symlinks resolving outside the root must be rejected"
        );
        let connection = database();
        let report = repo_index::index_workspace(&connection, root).expect("index");
        assert_eq!(indexed_paths(&connection), vec!["inside.txt"]);
        assert!(report.symlinks_skipped >= 1, "walk must skip symlinks");
    }
}

#[test]
fn index_bounds_are_reported_not_silently_truncated() {
    let (directory, _workspace) = open_workspace();
    let root = directory.path();
    write_file(root, "a.txt", "a");
    write_file(root, "b.txt", "b");
    write_file(root, "c.txt", "c");

    let connection = database();
    let first = repo_index::index_workspace_with(
        &connection,
        root,
        &WalkOptions {
            max_files: 2,
            max_total_bytes: MAX_INDEXED_BYTES,
        },
    )
    .expect("bounded index");
    assert_eq!(first.files_indexed, 2);
    assert_eq!(first.files_skipped_limit, 1);
    assert!(first.truncated);
    assert_eq!(indexed_paths(&connection), vec!["a.txt", "b.txt"]);

    assert!(matches!(
        repo_index::index_workspace_with(
            &connection,
            root,
            &WalkOptions {
                max_files: 0,
                max_total_bytes: MAX_INDEXED_BYTES,
            },
        ),
        Err(RepoIndexError::InvalidLimit { .. })
    ));
    assert!(matches!(
        repo_index::index_workspace_with(
            &connection,
            root,
            &WalkOptions {
                max_files: 1,
                max_total_bytes: 0,
            },
        ),
        Err(RepoIndexError::InvalidLimit { .. })
    ));
}

#[test]
fn schema_creation_is_idempotent_and_root_errors_are_typed() {
    let connection = database();
    repo_index::ensure_schema(&connection).expect("first schema");
    repo_index::ensure_schema(&connection).expect("second schema");
    assert_eq!(
        count(
            &connection,
            "SELECT COUNT(*) FROM sqlite_master WHERE name IN ('repo_files', 'repo_chunks', 'repo_chunks_fts')"
        ),
        3
    );

    let (directory, _workspace) = open_workspace();
    write_file(directory.path(), "a.txt", "a");
    let first = repo_index::index_workspace(&connection, directory.path()).expect("index");
    let second = repo_index::index_workspace(&connection, directory.path()).expect("re-index");
    assert_eq!(first.files_indexed, second.files_indexed);
    assert_eq!(first.chunks_indexed, second.chunks_indexed);
    assert_eq!(count(&connection, "SELECT COUNT(*) FROM repo_files"), 1);
    assert_eq!(
        count(&connection, "SELECT COUNT(*) FROM repo_chunks"),
        i64::try_from(second.chunks_indexed).unwrap()
    );

    let file = directory.path().join("a.txt");
    assert!(matches!(
        Workspace::open(&file),
        Err(RepoIndexError::RootNotDirectory { .. })
    ));
    assert!(matches!(
        Workspace::open(directory.path().join("missing")),
        Err(RepoIndexError::RootNotDirectory { .. })
    ));
}
