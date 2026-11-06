//! Child-process environment isolation.
//!
//! Tool and CLI children are given an explicit allowlist instead of the parent
//! process environment. This prevents credentials (including
//! `DEEPSEEK_API_KEY`) and unrelated operator state from leaking into a child
//! while preserving the variables Windows and the toolchain need to start.

use std::process::Command as StdCommand;

/// Variables a child may inherit. Everything else is removed.
pub const ALLOWED_ENV: &[&str] = &[
    "PATH",
    "PATHEXT",
    "SystemRoot",
    "WINDIR",
    "TEMP",
    "TMP",
    "USERPROFILE",
    "HOME",
    "COMSPEC",
    "OS",
    "PROCESSOR_ARCHITECTURE",
    "NUMBER_OF_PROCESSORS",
];

/// Applies the allowlist to a standard-library command.
pub fn sanitize_std(command: &mut StdCommand) {
    command.env_clear();
    for key in ALLOWED_ENV {
        if let Some(value) = std::env::var_os(key) {
            command.env(key, value);
        }
    }
}

/// Applies the allowlist to a Tokio command.
pub fn sanitize_tokio(command: &mut tokio::process::Command) {
    command.env_clear();
    for key in ALLOWED_ENV {
        if let Some(value) = std::env::var_os(key) {
            command.env(key, value);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(windows)]
    #[test]
    fn arbitrary_parent_environment_is_not_inherited() {
        let previous = std::env::var_os("HIVEROUTER_ENV_CANARY");
        std::env::set_var("HIVEROUTER_ENV_CANARY", "must-not-reach-child");
        let mut command = StdCommand::new("cmd.exe");
        sanitize_std(&mut command);
        let output = command
            .args(["/C", "echo", "%HIVEROUTER_ENV_CANARY%"])
            .output()
            .expect("cmd runs");
        match previous {
            Some(value) => std::env::set_var("HIVEROUTER_ENV_CANARY", value),
            None => std::env::remove_var("HIVEROUTER_ENV_CANARY"),
        }
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(
            !stdout.contains("must-not-reach-child"),
            "child inherited a non-allowlisted variable: {stdout}"
        );
    }

    #[test]
    fn path_is_allowlisted() {
        assert!(ALLOWED_ENV.contains(&"PATH"));
        assert!(!ALLOWED_ENV
            .iter()
            .any(|key| key.eq_ignore_ascii_case("DEEPSEEK_API_KEY")));
    }
}
