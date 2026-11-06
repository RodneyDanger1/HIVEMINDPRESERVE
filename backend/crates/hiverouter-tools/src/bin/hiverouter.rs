//! `hiverouter` CLI entry point (Phase 12).
//!
//! Thin wrapper: all behaviour lives in [`hiverouter_tools::cli`] so it is
//! unit-testable; this binary only forwards argv and the process exit code.
//! stdout carries the machine JSON envelope, stderr carries human lines.

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    std::process::exit(hiverouter_tools::cli::run_main(args));
}
