//! Runtime configuration for the local server.

use std::net::SocketAddr;
use std::path::{Path, PathBuf};

use thiserror::Error;

/// Default bind address: loopback only, port 8080 (blueprint Phase 2).
pub const DEFAULT_BIND: &str = "127.0.0.1:8080";
/// Default data directory, relative to the working directory.
pub const DEFAULT_DATA_DIR: &str = "data";
/// Database file name inside the data directory.
pub const DEFAULT_DB_FILE: &str = "hiverouter.db";

/// Resolved server configuration.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Config {
    /// Loopback bind address.
    pub bind: SocketAddr,
    /// Directory for the SQLite database and graph snapshots.
    pub data_dir: PathBuf,
    /// SQLite database file.
    pub db_path: PathBuf,
    /// WebSocket Origin allowlist (`http://127.0.0.1:<port>` etc.).
    pub allowed_origins: Vec<String>,
    /// Workspace root for the jailed file tools (`FILE_TREE_LIST`/`FILE_READ`).
    pub workspace_root: PathBuf,
}

/// Environment variable that appends comma-separated extra Origins.
pub const ALLOWED_ORIGINS_ENV: &str = "HIVEROUTER_ALLOWED_ORIGINS";
/// Environment variable selecting the workspace root for the jailed file tools.
pub const WORKSPACE_ROOT_ENV: &str = "HIVEROUTER_WORKSPACE_ROOT";

/// Default workspace root: the process working directory.
#[must_use]
pub fn default_workspace_root() -> PathBuf {
    std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."))
}

/// Default allowlist for a bound port (`*` for ephemeral test ports).
#[must_use]
pub fn default_allowed_origins(bind: SocketAddr) -> Vec<String> {
    if bind.port() == 0 {
        return Vec::new();
    }
    vec![
        format!("http://127.0.0.1:{}", bind.port()),
        format!("http://localhost:{}", bind.port()),
    ]
}

/// Configuration errors (fail loudly before the server starts).
#[derive(Debug, Error)]
pub enum ConfigError {
    /// A bind address was requested that is not loopback.
    #[error(
        "E_ORIGIN_REJECTED: bind address {0} is not a loopback address; the v1 server must bind 127.0.0.1 or ::1"
    )]
    NotLoopback(SocketAddr),
    /// The bind address could not be parsed.
    #[error("invalid bind address `{0}`: {1}")]
    InvalidBind(String, String),
    /// The data directory could not be created.
    #[error("could not create data directory `{path}`: {source}")]
    DataDir {
        /// Path that failed.
        path: String,
        /// Underlying I/O error.
        #[source]
        source: std::io::Error,
    },
    /// The database path exists but is a directory.
    #[error("database path `{0}` is a directory, not a database file")]
    DbPathIsDirectory(String),
    /// A CLI flag was given without its value.
    #[error("argument `{0}` requires a value")]
    MissingValue(String),
    /// A CLI flag is not recognised.
    #[error("unknown argument `{0}` (try --help)")]
    UnknownArgument(String),
}

impl Config {
    /// Builds configuration from process arguments (excluding argv[0]).
    ///
    /// # Errors
    /// Returns [`ConfigError`] for malformed flags or a non-loopback bind.
    pub fn from_args<I: IntoIterator<Item = String>>(args: I) -> Result<Self, ConfigError> {
        let mut bind = std::env::var("HIVEROUTER_BIND").unwrap_or_else(|_| DEFAULT_BIND.to_owned());
        let mut data_dir =
            std::env::var("HIVEROUTER_DATA_DIR").unwrap_or_else(|_| DEFAULT_DATA_DIR.to_owned());
        let mut db_path: Option<PathBuf> = std::env::var("HIVEROUTER_DB").ok().map(PathBuf::from);
        let mut workspace_root: Option<PathBuf> = std::env::var(WORKSPACE_ROOT_ENV)
            .ok()
            .map(PathBuf::from);

        let mut iter = args.into_iter();
        while let Some(argument) = iter.next() {
            match argument.as_str() {
                "--bind" => {
                    bind = iter.next().ok_or(ConfigError::MissingValue(argument))?;
                }
                "--data-dir" => {
                    data_dir = iter.next().ok_or(ConfigError::MissingValue(argument))?;
                }
                "--db" => {
                    let value = iter.next().ok_or(ConfigError::MissingValue(argument))?;
                    db_path = Some(PathBuf::from(value));
                }
                "--workspace-root" => {
                    let value = iter.next().ok_or(ConfigError::MissingValue(argument))?;
                    workspace_root = Some(PathBuf::from(value));
                }
                other if other.starts_with("--bind=") => {
                    bind = other["--bind=".len()..].to_owned();
                }
                other if other.starts_with("--data-dir=") => {
                    data_dir = other["--data-dir=".len()..].to_owned();
                }
                other if other.starts_with("--db=") => {
                    db_path = Some(PathBuf::from(&other["--db=".len()..]));
                }
                other if other.starts_with("--workspace-root=") => {
                    workspace_root = Some(PathBuf::from(&other["--workspace-root=".len()..]));
                }
                other => return Err(ConfigError::UnknownArgument(other.to_owned())),
            }
        }
        Self::resolve_with_workspace(&bind, &data_dir, db_path, workspace_root.as_deref())
    }

    /// Validates and resolves the configuration, creating the data directory.
    ///
    /// # Errors
    /// Returns [`ConfigError`] when the bind address is invalid/non-loopback or
    /// the data directory cannot be created.
    pub fn resolve(
        bind: &str,
        data_dir: &str,
        db_path: Option<PathBuf>,
    ) -> Result<Self, ConfigError> {
        Self::resolve_with_workspace(bind, data_dir, db_path, None)
    }

    /// Validates and resolves the configuration with an explicit workspace root.
    ///
    /// # Errors
    /// Returns [`ConfigError`] when the bind address is invalid/non-loopback or
    /// the data directory cannot be created.
    pub fn resolve_with_workspace(
        bind: &str,
        data_dir: &str,
        db_path: Option<PathBuf>,
        workspace_root: Option<&Path>,
    ) -> Result<Self, ConfigError> {
        let bind: SocketAddr = bind
            .parse()
            .map_err(|error: std::net::AddrParseError| {
                ConfigError::InvalidBind(bind.to_owned(), error.to_string())
            })?;
        if !bind.ip().is_loopback() {
            return Err(ConfigError::NotLoopback(bind));
        }
        let data_dir = PathBuf::from(data_dir);
        std::fs::create_dir_all(&data_dir).map_err(|source| ConfigError::DataDir {
            path: data_dir.display().to_string(),
            source,
        })?;
        let db_path = match db_path {
            Some(path) => path,
            None => data_dir.join(DEFAULT_DB_FILE),
        };
        if db_path.is_dir() {
            return Err(ConfigError::DbPathIsDirectory(db_path.display().to_string()));
        }
        if let Some(parent) = db_path.parent() {
            if !parent.as_os_str().is_empty() {
                std::fs::create_dir_all(parent).map_err(|source| ConfigError::DataDir {
                    path: parent.display().to_string(),
                    source,
                })?;
            }
        }
        Ok(Self {
            bind,
            data_dir,
            db_path,
            allowed_origins: resolve_allowed_origins(bind),
            workspace_root: workspace_root
                .map(Path::to_path_buf)
                .unwrap_or_else(default_workspace_root),
        })
    }

    /// Path of the database file.
    #[must_use]
    pub fn db_file(&self) -> &Path {
        &self.db_path
    }
}

fn resolve_allowed_origins(bind: SocketAddr) -> Vec<String> {
    let mut origins = default_allowed_origins(bind);
    if let Ok(extra) = std::env::var(ALLOWED_ORIGINS_ENV) {
        for origin in extra.split(',').map(str::trim).filter(|origin| !origin.is_empty()) {
            if !origins.iter().any(|existing| existing.eq_ignore_ascii_case(origin)) {
                origins.push(origin.to_owned());
            }
        }
    }
    origins
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_bind_loopback_and_derive_db_path() {
        let temp = tempfile::tempdir().expect("temp dir");
        let config = Config::resolve(DEFAULT_BIND, temp.path().to_str().expect("utf8"), None)
            .expect("valid config");
        assert!(config.bind.ip().is_loopback());
        assert_eq!(config.db_path, temp.path().join(DEFAULT_DB_FILE));
        assert!(config
            .allowed_origins
            .contains(&"http://127.0.0.1:8080".to_owned()));
        assert!(config.allowed_origins.contains(&"http://localhost:8080".to_owned()));
        assert!(config.workspace_root.is_absolute() || config.workspace_root.exists());
    }

    #[test]
    fn non_loopback_bind_is_refused_by_name() {
        let temp = tempfile::tempdir().expect("temp dir");
        let error = Config::resolve("0.0.0.0:8080", temp.path().to_str().expect("utf8"), None)
            .expect_err("non-loopback must be refused");
        assert!(matches!(error, ConfigError::NotLoopback(_)));
        assert!(error.to_string().contains("E_ORIGIN_REJECTED"));
    }

    #[test]
    fn flags_parse_and_unknown_flags_are_refused() {
        let temp = tempfile::tempdir().expect("temp dir");
        let args = vec![
            "--bind".to_owned(),
            "127.0.0.1:0".to_owned(),
            "--data-dir".to_owned(),
            temp.path().display().to_string(),
            format!("--db={}", temp.path().join("custom.db").display()),
            "--workspace-root".to_owned(),
            temp.path().display().to_string(),
        ];
        let config = Config::from_args(args).expect("parses");
        assert_eq!(config.bind.port(), 0);
        assert_eq!(config.db_path, temp.path().join("custom.db"));
        assert_eq!(config.workspace_root, temp.path());

        let error = Config::from_args(vec!["--nope".to_owned()]).expect_err("unknown flag");
        assert!(matches!(error, ConfigError::UnknownArgument(_)));

        let error = Config::from_args(vec!["--bind".to_owned()]).expect_err("missing value");
        assert!(matches!(error, ConfigError::MissingValue(_)));
    }

    #[test]
    fn db_path_that_is_a_directory_is_refused() {
        let temp = tempfile::tempdir().expect("temp dir");
        let error = Config::resolve(
            "127.0.0.1:0",
            temp.path().to_str().expect("utf8"),
            Some(temp.path().to_path_buf()),
        )
        .expect_err("directory db path must be refused");
        assert!(matches!(error, ConfigError::DbPathIsDirectory(_)));
    }
}
