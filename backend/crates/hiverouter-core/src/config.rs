//! Runtime configuration for the local server.

use std::net::SocketAddr;
use std::path::{Path, PathBuf};

use hiverouter_provider::{Model, ReasoningEffort, Thinking, MAX_OUTPUT_TOKENS};
use thiserror::Error;

/// Default bind address: loopback only, port 8080 (blueprint Phase 2).
pub const DEFAULT_BIND: &str = "127.0.0.1:8080";
/// Default data directory, relative to the working directory.
pub const DEFAULT_DATA_DIR: &str = "data";
/// Database file name inside the data directory.
pub const DEFAULT_DB_FILE: &str = "hiverouter.db";
/// Default model id (documented `deepseek-flash` = DeepSeek-V4.1-Flash-1017).
pub const DEFAULT_MODEL: &str = "deepseek-flash";
/// Default planner output cap (documented max is 393216; we cap cost locally).
pub const DEFAULT_PLANNER_MAX_TOKENS: u32 = 16_384;
/// Default leaf output cap.
pub const DEFAULT_LEAF_MAX_TOKENS: u32 = 8_192;

/// Environment variables for provider behaviour (never for the key itself).
pub const PROVIDER_ENV: &str = "HIVEROUTER_PROVIDER";
/// Model id environment variable.
pub const MODEL_ENV: &str = "HIVEROUTER_MODEL";
/// Planner thinking mode (`enabled`/`disabled`).
pub const THINKING_ENV: &str = "HIVEROUTER_THINKING";
/// Leaf thinking mode (`enabled`/`disabled`); default disabled to bound cost.
pub const LEAF_THINKING_ENV: &str = "HIVEROUTER_LEAF_THINKING";
/// Reasoning effort (`none`/`low`/`high`/`max`, with documented aliases).
pub const REASONING_EFFORT_ENV: &str = "HIVEROUTER_REASONING_EFFORT";
/// Leaf output cap.
pub const MAX_OUTPUT_TOKENS_ENV: &str = "HIVEROUTER_MAX_OUTPUT_TOKENS";
/// Planner output cap.
pub const PLANNER_MAX_OUTPUT_TOKENS_ENV: &str = "HIVEROUTER_PLANNER_MAX_OUTPUT_TOKENS";
/// Optional OpenAI-compatible base URL (for example AMD Token Lab's `/v1`).
pub const BASE_URL_ENV: &str = "HIVEROUTER_BASE_URL";
/// Optional wire model id when the endpoint's catalog name differs from the
/// documented DeepSeek id used for local pricing/accounting.
pub const WIRE_MODEL_ENV: &str = "HIVEROUTER_WIRE_MODEL";
/// Optional path to a saved provider profile JSON.
pub const PROVIDER_FILE_ENV: &str = "HIVEROUTER_PROVIDER_FILE";

/// Provider behaviour resolved from the environment at boot.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProviderSettings {
    /// When false, the server uses the honest `UnconfiguredExecutor`.
    pub enabled: bool,
    /// Documented model id.
    pub model: Model,
    /// Planner thinking switch.
    pub planner_thinking: Thinking,
    /// Leaf thinking switch.
    pub leaf_thinking: Thinking,
    /// Planner reasoning effort.
    pub planner_reasoning_effort: ReasoningEffort,
    /// Planner max output tokens.
    pub planner_max_tokens: u32,
    /// Leaf max output tokens.
    pub leaf_max_tokens: u32,
    /// OpenAI-compatible base URL; `None` means the documented DeepSeek URL.
    pub base_url: String,
    /// Wire model id override; accounting still uses `model`.
    pub wire_model: Option<String>,
}

impl Default for ProviderSettings {
    fn default() -> Self {
        Self {
            enabled: true,
            model: Model::Flash,
            planner_thinking: Thinking::enabled(),
            leaf_thinking: Thinking::disabled(),
            planner_reasoning_effort: ReasoningEffort::High,
            planner_max_tokens: DEFAULT_PLANNER_MAX_TOKENS,
            leaf_max_tokens: DEFAULT_LEAF_MAX_TOKENS,
            base_url: hiverouter_provider::DEFAULT_BASE_URL.to_owned(),
            wire_model: None,
        }
    }
}

impl ProviderSettings {
    /// True when the configured base URL is not the documented DeepSeek API.
    /// Such endpoints are OpenAI-compatible transports, not DeepSeek accounts.
    #[must_use]
    pub fn is_external_endpoint(&self) -> bool {
        self.base_url != hiverouter_provider::DEFAULT_BASE_URL
    }

    /// Wire model id: the endpoint catalogue id, or the documented model id.
    #[must_use]
    pub fn wire_model_id(&self) -> &str {
        self.wire_model.as_deref().unwrap_or(self.model.id())
    }

    /// Resolves provider settings from the process environment.
    ///
    /// # Errors
    /// Returns [`ConfigError::ProviderSetting`] for an unknown model, mode,
    /// effort or an out-of-range output cap. Unknown values never silently
    /// become defaults.
    pub fn from_env() -> Result<Self, ConfigError> {
        let default = Self::default();
        let profile = load_provider_profile();
        let profile_str = |key: &str| {
            profile
                .as_ref()
                .and_then(|value| value.get(key))
                .and_then(serde_json::Value::as_str)
                .map(str::trim)
                .filter(|value| !value.is_empty())
        };
        let enabled = match std::env::var(PROVIDER_ENV) {
            Ok(value) => match value.trim().to_ascii_lowercase().as_str() {
                "auto" | "on" | "enabled" | "true" | "1" => true,
                "off" | "disabled" | "false" | "0" => false,
                other => {
                    return Err(ConfigError::ProviderSetting(format!(
                        "unknown {PROVIDER_ENV} value `{other}` (expected auto|disabled)"
                    )))
                }
            },
            Err(_) => default.enabled,
        };
        let model = match std::env::var(MODEL_ENV) {
            Ok(value) => {
                let trimmed = value.trim();
                Model::from_id(trimmed).ok_or_else(|| {
                    ConfigError::ProviderSetting(format!(
                        "unknown {MODEL_ENV} value `{trimmed}` (documented: deepseek-flash|deepseek-v4-pro)"
                    ))
                })?
            }
            Err(_) => match profile_str("model") {
                Some(value) => Model::from_id(value).ok_or_else(|| {
                    ConfigError::ProviderSetting(format!(
                        "provider profile model `{value}` is not documented"
                    ))
                })?,
                None => default.model,
            },
        };
        let planner_thinking = match std::env::var(THINKING_ENV) {
            Ok(value) => parse_thinking(&value, THINKING_ENV)?,
            Err(_) => default.planner_thinking,
        };
        let leaf_thinking = match std::env::var(LEAF_THINKING_ENV) {
            Ok(value) => parse_thinking(&value, LEAF_THINKING_ENV)?,
            Err(_) => default.leaf_thinking,
        };
        let planner_reasoning_effort = match std::env::var(REASONING_EFFORT_ENV) {
            Ok(value) => parse_reasoning_effort(&value)?,
            Err(_) => default.planner_reasoning_effort,
        };
        let planner_max_tokens = match std::env::var(PLANNER_MAX_OUTPUT_TOKENS_ENV) {
            Ok(value) => parse_max_tokens(&value, PLANNER_MAX_OUTPUT_TOKENS_ENV)?,
            Err(_) => default.planner_max_tokens,
        };
        let leaf_max_tokens = match std::env::var(MAX_OUTPUT_TOKENS_ENV) {
            Ok(value) => parse_max_tokens(&value, MAX_OUTPUT_TOKENS_ENV)?,
            Err(_) => default.leaf_max_tokens,
        };
        let base_url = match std::env::var(BASE_URL_ENV) {
            Ok(value) => validate_base_url(&value)?,
            Err(_) => match profile_str("base_url") {
                Some(value) => validate_base_url(value)?,
                None => default.base_url,
            },
        };
        let wire_model = match std::env::var(WIRE_MODEL_ENV) {
            Ok(value) => parse_wire_model(&value, WIRE_MODEL_ENV)?,
            Err(_) => match profile_str("wire_model") {
                Some(value) => parse_wire_model(value, "provider profile wire_model")?,
                None => None,
            },
        };
        if base_url != hiverouter_provider::DEFAULT_BASE_URL && wire_model.is_none() {
            return Err(ConfigError::ProviderSetting(
                "an external OpenAI-compatible base URL requires a wire model id; set \
                 HIVEROUTER_WIRE_MODEL or the provider profile's wire_model"
                    .to_owned(),
            ));
        }
        Ok(Self {
            enabled,
            model,
            planner_thinking,
            leaf_thinking,
            planner_reasoning_effort,
            planner_max_tokens,
            leaf_max_tokens,
            base_url,
            wire_model,
        })
    }
}

/// Loads a saved non-secret provider profile. The key is never in this file;
/// it is supplied separately (DPAPI file via the launcher, or the process env).
fn load_provider_profile() -> Option<serde_json::Value> {
    let path = std::env::var(PROVIDER_FILE_ENV)
        .ok()
        .map(std::path::PathBuf::from)
        .or_else(|| {
            std::env::var("LOCALAPPDATA")
                .ok()
                .map(std::path::PathBuf::from)
                .map(|root| root.join("HiveMind").join("provider.json"))
        })?;
    let text = std::fs::read_to_string(path).ok()?;
    serde_json::from_str(&text).ok()
}

fn validate_base_url(raw: &str) -> Result<String, ConfigError> {
    let trimmed = raw.trim().trim_end_matches('/').to_owned();
    if trimmed.is_empty()
        || !trimmed.starts_with("http://") && !trimmed.starts_with("https://")
    {
        return Err(ConfigError::ProviderSetting(
            "provider base URL must be an http(s) URL".to_owned(),
        ));
    }
    Ok(trimmed)
}

fn parse_wire_model(raw: &str, key: &str) -> Result<Option<String>, ConfigError> {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return Err(ConfigError::ProviderSetting(format!(
            "{key} is present but empty"
        )));
    }
    Ok(Some(trimmed.to_owned()))
}

/// Parses one documented thinking toggle value.
fn parse_thinking(raw: &str, key: &str) -> Result<Thinking, ConfigError> {
    match raw.trim().to_ascii_lowercase().as_str() {
        "enabled" | "on" | "true" | "1" => Ok(Thinking::enabled()),
        "disabled" | "off" | "false" | "0" => Ok(Thinking::disabled()),
        other => Err(ConfigError::ProviderSetting(format!(
            "unknown {key} value `{other}` (expected enabled|disabled)"
        ))),
    }
}

/// Parses the documented effort values and their compatibility aliases.
fn parse_reasoning_effort(raw: &str) -> Result<ReasoningEffort, ConfigError> {
    match raw.trim().to_ascii_lowercase().as_str() {
        "none" => Ok(ReasoningEffort::None),
        "low" | "minimal" => Ok(ReasoningEffort::Low),
        "high" | "medium" | "xhigh" => Ok(ReasoningEffort::High),
        "max" | "ultra" => Ok(ReasoningEffort::Max),
        other => Err(ConfigError::ProviderSetting(format!(
            "unknown {REASONING_EFFORT_ENV} value `{other}` (expected none|low|high|max)"
        ))),
    }
}

/// Parses an output-token cap inside the documented range.
fn parse_max_tokens(raw: &str, key: &str) -> Result<u32, ConfigError> {
    let value: u32 = raw.trim().parse().map_err(|_| {
        ConfigError::ProviderSetting(format!("{key} must be an integer, got `{raw}`"))
    })?;
    if value == 0 || value > MAX_OUTPUT_TOKENS {
        return Err(ConfigError::ProviderSetting(format!(
            "{key} must be between 1 and {MAX_OUTPUT_TOKENS}, got {value}"
        )));
    }
    Ok(value)
}

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
    /// Provider behaviour (model, thinking, effort, caps).
    pub provider: ProviderSettings,
    /// Whether approved file/shell side effects may execute.
    pub side_effects_enabled: bool,
}

/// Environment variable that appends comma-separated extra Origins.
pub const ALLOWED_ORIGINS_ENV: &str = "HIVEROUTER_ALLOWED_ORIGINS";
/// Environment variable selecting the workspace root for the jailed file tools.
pub const WORKSPACE_ROOT_ENV: &str = "HIVEROUTER_WORKSPACE_ROOT";
/// Environment variable disabling approved side effects (`0`/`false`/`off`).
pub const SIDE_EFFECTS_ENV: &str = "HIVEROUTER_SIDE_EFFECTS";

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
    /// A provider setting is malformed or undocumented.
    #[error("E_SCHEMA_INVALID: {0}")]
    ProviderSetting(String),
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
        // Test/helper resolution defaults to the offline executor so `cargo test`
        // never performs a live provider call just because a key is present in
        // the developer environment. Production uses `from_args`.
        let mut config = Self::resolve_with_workspace(bind, data_dir, db_path, None)?;
        config.provider.enabled = false;
        Ok(config)
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
        let side_effects_enabled = match std::env::var(SIDE_EFFECTS_ENV) {
            Ok(value) => !matches!(
                value.trim().to_ascii_lowercase().as_str(),
                "0" | "false" | "off" | "disabled" | "no"
            ),
            Err(_) => true,
        };
        Ok(Self {
            bind,
            data_dir,
            db_path,
            allowed_origins: resolve_allowed_origins(bind),
            workspace_root: workspace_root
                .map(Path::to_path_buf)
                .unwrap_or_else(default_workspace_root),
            provider: ProviderSettings::from_env()?,
            side_effects_enabled,
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
