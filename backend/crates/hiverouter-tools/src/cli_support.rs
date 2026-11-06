//! CLI support helpers: version probes, semver-triple parsing, HTTP JSON fetch.
//!
//! Kept separate from [`crate::cli`] so the dispatch logic is testable without
//! spawning processes or opening sockets; each helper is exercised directly by
//! unit tests here.

use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

/// Parses `major.minor.patch` from a version string (tolerates trailing text).
#[must_use]
pub fn parse_version_triple(raw: &str) -> Option<(u64, u64, u64)> {
    let trimmed = raw.trim().trim_start_matches('v');
    let mut parts = trimmed.split(|character: char| !character.is_ascii_digit() && character != '.');
    let first = parts.next()?;
    let mut numbers = first.split('.');
    let major = numbers.next()?.parse().ok()?;
    let minor = numbers.next()?.parse().ok()?;
    let patch = numbers.next()?.parse().ok()?;
    Some((major, minor, patch))
}

/// Required Node floor (FACT F3 / blueprint WP 1.10).
#[must_use]
pub fn node_floor() -> (u64, u64, u64) {
    (22, 12, 0)
}

/// Required Rust MSRV floor (FACT F9).
#[must_use]
pub fn rust_msrv_floor() -> (u64, u64, u64) {
    (1, 89, 0)
}

/// Runs `<program> --version` with a bounded wait and returns the raw version line.
///
/// # Errors
/// Returns a human-readable reason when the program is missing, exits non-zero
/// or does not answer within five seconds (it is then killed).
pub fn version_probe(program: &str) -> Result<(String, String), String> {
    let mut child = Command::new(program)
        .env_remove(hiverouter_provider::API_KEY_ENV)
        .arg("--version")
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|error| format!("{program} could not be started: {error}"))?;

    let deadline = Instant::now() + Duration::from_secs(5);
    loop {
        match child.try_wait() {
            Ok(Some(status)) => {
                let output = child.wait_with_output().map_err(|error| {
                    format!("{program} could not be waited on: {error}")
                })?;
                if !status.success() {
                    return Err(format!(
                        "{program} --version exited with {status}: {}",
                        String::from_utf8_lossy(&output.stderr).trim()
                    ));
                }
                let stdout = String::from_utf8_lossy(&output.stdout).trim().to_owned();
                let stderr = String::from_utf8_lossy(&output.stderr).trim().to_owned();
                let observed = if stdout.is_empty() { stderr } else { stdout };
                if observed.is_empty() {
                    return Err(format!("{program} --version printed nothing"));
                }
                return Ok((program.to_owned(), observed));
            }
            Ok(None) if Instant::now() >= deadline => {
                let _ = child.kill();
                let _ = child.wait();
                return Err(format!("{program} --version did not answer within 5s"));
            }
            Ok(None) => std::thread::sleep(Duration::from_millis(20)),
            Err(error) => {
                let _ = child.kill();
                return Err(format!("{program} wait failed: {error}"));
            }
        }
    }
}

/// Fetches a JSON document from a local HTTP endpoint with a request timeout.
///
/// # Errors
/// Returns a transport/status/parse description; the CLI maps it to a named
/// refusal. The returned error text never contains request headers.
pub fn fetch_json(url: &str, timeout: Duration) -> Result<serde_json::Value, String> {
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .map_err(|error| format!("cannot start the CLI runtime: {error}"))?;
    runtime.block_on(async {
        let client = reqwest::Client::builder()
            .timeout(timeout)
            .build()
            .map_err(|error| format!("cannot build the CLI HTTP client: {error}"))?;
        let response = client
            .get(url)
            .send()
            .await
            .map_err(|error| format!("local server unreachable at {url}: {error}"))?;
        let status = response.status();
        let body = response
            .text()
            .await
            .map_err(|error| format!("cannot read the {status} response: {error}"))?;
        if !status.is_success() {
            return Err(format!(
                "local server answered HTTP {status} for {url} (body bytes {})",
                body.len()
            ));
        }
        serde_json::from_str(&body)
            .map_err(|error| format!("local server returned non-JSON for {url}: {error}"))
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn version_triples_parse() {
        assert_eq!(parse_version_triple("22.12.0"), Some((22, 12, 0)));
        assert_eq!(parse_version_triple("v26.7.0"), Some((26, 7, 0)));
        assert_eq!(parse_version_triple("1.99.1 (58cf8f9c3 2026-10-13)"), Some((1, 99, 1)));
        assert_eq!(parse_version_triple("not-a-version"), None);
        assert_eq!(parse_version_triple("1.2"), None);

        let host_node = version_probe("node").expect("node present on this host");
        let node_version = parse_version_triple(host_node.1.trim_start_matches('v'))
            .expect("node version triple");
        assert!(node_version >= node_floor(), "host node meets the floor");

        let host_rustc = version_probe("rustc").expect("rustc present on this host");
        let rust_version = host_rustc
            .1
            .split_whitespace()
            .nth(1)
            .and_then(parse_version_triple)
            .expect("rustc version triple");
        assert!(rust_version >= rust_msrv_floor(), "host rustc meets the MSRV");
    }

    #[test]
    fn missing_program_is_reported_not_panicked() {
        let error = version_probe("definitely-not-a-program-hiverouter").expect_err("missing");
        assert!(error.contains("could not be started"));
    }

    #[test]
    fn fetch_json_against_unreachable_local_port_fails() {
        let error = fetch_json("http://127.0.0.1:1/health", Duration::from_millis(500))
            .expect_err("unreachable");
        assert!(error.contains("unreachable") || error.contains("HTTP"));
    }
}
