//! Dev-only benchmark instrument (Phase 11 WP 11.14, `criterion 0.8.2`).
//!
//! Measures the hot paths this crate owns: schema validation of a tool call, the
//! argument-binding digest, jail path resolution, and F1 price lookup + cost
//! arithmetic. `criterion` is a `[dev-dependencies]` entry only and is never
//! linked into the release binary.
//!
//! Every number printed here is MEASURED on the host that ran the command; no
//! result is a product gate and none may be quoted as a capacity claim.

use std::hint::black_box;

use criterion::{criterion_group, criterion_main, Criterion};
use hiverouter_contract::Usage;
use hiverouter_provider::{Model, PricingTable, RunAccounting};
use hiverouter_tools::{sha256_hex, AgentTier, ToolRegistry, WorkspaceJail};

fn bench_tool_schema_validation(criterion: &mut Criterion) {
    let registry = ToolRegistry::standard();
    let args = serde_json::json!({"path": "docs/architecture.md", "content": "hello"});
    criterion.bench_function("tool_schema_validate_file_write", |bencher| {
        bencher.iter(|| {
            registry
                .resolve_call(
                    AgentTier::WorkerAgent,
                    black_box("file_write"),
                    black_box(&args),
                )
                .expect("valid call");
        });
    });
}

fn bench_args_digest(criterion: &mut Criterion) {
    let args = serde_json::json!({
        "path": "src/main.rs",
        "content": "fn main() { println!(\"hivemind\"); }\n",
        "expected_sha256": null
    })
    .to_string();
    criterion.bench_function("sha256_args_binding", |bencher| {
        bencher.iter(|| sha256_hex(black_box(args.as_bytes())));
    });
}

fn bench_jail_resolution(criterion: &mut Criterion) {
    let temp = tempfile::tempdir().expect("tempdir");
    std::fs::create_dir_all(temp.path().join("docs")).expect("dir");
    std::fs::write(temp.path().join("docs/note.txt"), b"body").expect("file");
    let jail = WorkspaceJail::open(temp.path()).expect("jail");
    criterion.bench_function("jail_resolve_relative_path", |bencher| {
        bencher.iter(|| jail.resolve(black_box("docs/note.txt")).expect("resolves"));
    });
}

fn bench_cost_math(criterion: &mut Criterion) {
    let table = PricingTable::documented();
    let usage = Usage {
        prompt_tokens: Some(120_000),
        completion_tokens: Some(8_000),
        total_tokens: Some(128_000),
        prompt_cache_hit_tokens: Some(40_000),
        prompt_cache_miss_tokens: Some(80_000),
        reasoning_tokens: None,
    };
    // 2026-11-06T01:00:00Z → documented UTC peak window.
    let peak_at = 1_792_702_000_u64;
    criterion.bench_function("decimal_cost_one_call_peak", |bencher| {
        bencher.iter(|| {
            let mut accounting = RunAccounting::new(table.clone());
            accounting
                .record(
                    black_box(Model::Flash),
                    Some(black_box(&usage)),
                    black_box(peak_at),
                )
                .expect("settles");
            accounting
        });
    });
}

criterion_group!(
    engine_benches,
    bench_tool_schema_validation,
    bench_args_digest,
    bench_jail_resolution,
    bench_cost_math
);
criterion_main!(engine_benches);
