//! Environment checks reported at boot and by `GET /health`.
//!
//! - MSRV: enforced at build time by `build.rs` and `rust-version`; the observed
//!   toolchain line is embedded and reported here.
//! - Node: floor `>= 22.12.0` (FACT-REGISTER F3/F5, blueprint WP 1.10). Missing
//!   or too-old Node refuses by name (`E_NODE_MISSING`); the backend still
//!   serves, but `/health` honestly reports `degraded`.

use std::process::Command;

use serde::Serialize;

/// Node floor major (FACT-REGISTER F3: `>= 22.12.0`).
pub const NODE_MIN_MAJOR: u64 = 22;
/// Node floor minor.
pub const NODE_MIN_MINOR: u64 = 12;

/// One environment check.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Check {
    /// Check name (`rustc`, `node`).
    pub name: &'static str,
    /// Whether this check must pass for the server to count as healthy.
    pub required: bool,
    /// Observed outcome.
    pub ok: bool,
    /// Observed detail (version line or the exact failure).
    pub detail: String,
    /// Named refusal code when the check failed.
    pub code: Option<&'static str>,
}

impl Check {
    fn pass(name: &'static str, required: bool, detail: String) -> Self {
        Self {
            name,
            required,
            ok: true,
            detail,
            code: None,
        }
    }

    fn fail(name: &'static str, required: bool, detail: String, code: &'static str) -> Self {
        Self {
            name,
            required,
            ok: false,
            detail,
            code: Some(code),
        }
    }
}

/// MSRV check. The build script refuses to compile on an older compiler, so a
/// running binary always passes; this reports the observed toolchain.
#[must_use]
pub fn msrv_check() -> Check {
    Check::pass(
        "rustc",
        true,
        format!(
            "{} (MSRV 1.{}.0)",
            env!("HIVEROUTER_BUILD_RUSTC"),
            env!("HIVEROUTER_MIN_RUST_MINOR")
        ),
    )
}

/// Parses `node --version` output (`v26.7.0` or `26.7.0`).
#[must_use]
pub fn parse_node_version(raw: &str) -> Option<(u64, u64, u64)> {
    let trimmed = raw.trim();
    let trimmed = trimmed.strip_prefix('v').unwrap_or(trimmed);
    let mut parts = trimmed.split('.');
    let major = parts.next()?.parse().ok()?;
    let minor = parts.next()?.parse().ok()?;
    let patch = parts.next()?.split('-').next()?.parse().ok()?;
    Some((major, minor, patch))
}

/// True when `version` is at least `major.minor.0`.
#[must_use]
pub fn version_at_least(version: (u64, u64, u64), major: u64, minor: u64) -> bool {
    (version.0, version.1) >= (major, minor)
}

/// Node check: runs `node --version` and evaluates the floor.
#[must_use]
pub fn node_check() -> Check {
    match Command::new("node").arg("--version").output() {
        Ok(output) if output.status.success() => {
            let raw = String::from_utf8_lossy(&output.stdout).trim().to_owned();
            match parse_node_version(&raw) {
                Some(version) if version_at_least(version, NODE_MIN_MAJOR, NODE_MIN_MINOR) => {
                    Check::pass(
                        "node",
                        false,
                        format!("{raw} (>= {NODE_MIN_MAJOR}.{NODE_MIN_MINOR}.0)"),
                    )
                }
                Some(version) => Check::fail(
                    "node",
                    false,
                    format!(
                        "{raw} is older than the required {NODE_MIN_MAJOR}.{NODE_MIN_MINOR}.0 (parsed {}.{}.{})",
                        version.0, version.1, version.2
                    ),
                    "E_NODE_MISSING",
                ),
                None => Check::fail(
                    "node",
                    false,
                    format!("could not parse `{raw}` as a Node.js version"),
                    "E_NODE_MISSING",
                ),
            }
        }
        Ok(output) => Check::fail(
            "node",
            false,
            format!("`node --version` exited with {}", output.status),
            "E_NODE_MISSING",
        ),
        Err(error) => Check::fail(
            "node",
            false,
            format!("node is not runnable from PATH: {error}"),
            "E_NODE_MISSING",
        ),
    }
}

/// Runs every environment check (MSRV first, then Node).
#[must_use]
pub fn run_checks() -> Vec<Check> {
    vec![msrv_check(), node_check()]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_node_version_lines() {
        assert_eq!(parse_node_version("v26.7.0\n"), Some((26, 7, 0)));
        assert_eq!(parse_node_version("26.7.0"), Some((26, 7, 0)));
        assert_eq!(parse_node_version("v22.12.0-rc.1"), Some((22, 12, 0)));
        assert_eq!(parse_node_version("nonsense"), None);
        assert_eq!(parse_node_version("v26"), None);
    }

    #[test]
    fn floor_comparison() {
        assert!(version_at_least((22, 12, 0), 22, 12));
        assert!(version_at_least((26, 7, 0), 22, 12));
        assert!(!version_at_least((22, 11, 9), 22, 12));
        assert!(!version_at_least((20, 19, 0), 22, 12));
    }

    #[test]
    fn msrv_check_reports_the_build_toolchain() {
        let check = msrv_check();
        assert!(check.ok);
        assert!(check.required);
        assert!(check.detail.contains("MSRV 1.89.0"));
        assert!(check.code.is_none());
    }

    #[test]
    fn node_check_observes_the_host_node() {
        let check = node_check();
        // The host has Node v26.7.0 (claim C-005); a missing Node is still a
        // valid check result but must carry the named refusal.
        if check.ok {
            assert!(check.detail.starts_with('v'));
            assert!(check.code.is_none());
        } else {
            assert_eq!(check.code, Some("E_NODE_MISSING"));
        }
    }
}
