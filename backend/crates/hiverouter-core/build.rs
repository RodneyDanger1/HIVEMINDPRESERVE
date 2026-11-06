//! Build-time gates for the HiveMind v1 backend.
//!
//! 1. MSRV gate (blueprint WP 1.9, refusal `E_TOOLCHAIN_MSRV`): the compiler
//!    that builds this crate must be at least 1.89.0. Cargo already enforces the
//!    `rust-version` field; this gate fails the build with the named refusal if
//!    the workspace is ever built by a toolchain that ignores that field.
//! 2. Toolchain provenance: the observed `rustc --version` line is embedded in
//!    the binary and reported by `GET /health`.

use std::process::Command;

const MIN_RUST_MINOR: u32 = 89;

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-env-changed=RUSTC");

    let rustc = std::env::var("RUSTC").unwrap_or_else(|_| String::from("rustc"));
    let output = Command::new(&rustc)
        .arg("--version")
        .output()
        .unwrap_or_else(|error| {
            panic!("E_TOOLCHAIN_MSRV: could not run `{rustc} --version`: {error}");
        });
    if !output.status.success() {
        panic!("E_TOOLCHAIN_MSRV: `{rustc} --version` exited with {}", output.status);
    }
    let version_line = String::from_utf8_lossy(&output.stdout).trim().to_owned();
    let (major, minor) = parse_major_minor(&version_line).unwrap_or_else(|| {
        panic!("E_TOOLCHAIN_MSRV: could not parse rustc version from `{version_line}`");
    });
    if major < 1 || (major == 1 && minor < MIN_RUST_MINOR) {
        panic!(
            "E_TOOLCHAIN_MSRV: rustc {major}.{minor} is older than the required MSRV 1.{MIN_RUST_MINOR}.0 \
             (`{version_line}`); install Rust 1.89.0 or newer and rebuild"
        );
    }
    println!("cargo:rustc-env=HIVEROUTER_BUILD_RUSTC={version_line}");
    println!("cargo:rustc-env=HIVEROUTER_MIN_RUST_MINOR={MIN_RUST_MINOR}");
}

/// Parses `rustc 1.99.1 (hash date)` into `(1, 99)`.
fn parse_major_minor(line: &str) -> Option<(u32, u32)> {
    let version = line.split_whitespace().nth(1)?;
    let mut parts = version.split('.');
    let major = parts.next()?.parse().ok()?;
    let minor = parts.next()?.parse().ok()?;
    Some((major, minor))
}
