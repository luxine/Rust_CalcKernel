use std::{fs, path::Path};

fn repo_root() -> &'static Path {
    Path::new(env!("CARGO_MANIFEST_DIR"))
}

#[test]
fn repository_should_define_native_cargo_benchmark_harness() {
    let cargo_toml = fs::read_to_string(repo_root().join("Cargo.toml")).expect("read Cargo.toml");

    for required in [
        "[[bench]]",
        "name = \"ckc_perf\"",
        "path = \"benches/ckc_perf.rs\"",
        "harness = false",
    ] {
        assert!(
            cargo_toml.contains(required),
            "Cargo.toml must register a native ckc_perf benchmark with `{required}`"
        );
    }

    assert!(
        repo_root().join("benches/ckc_perf.rs").is_file(),
        "benches/ckc_perf.rs must contain the native performance harness"
    );
}

#[test]
fn benchmark_tree_should_not_keep_empty_placeholder_directories() {
    for relative in [
        "bench/perf/baselines",
        "bench/perf/cases",
        "bench/perf/fixtures",
        "bench/perf/lib",
        "bench/perf/tests",
    ] {
        let dir = repo_root().join(relative);
        assert!(dir.is_dir(), "{relative} must exist");
        assert!(
            fs::read_dir(&dir)
                .expect("read benchmark directory")
                .next()
                .is_some(),
            "{relative} must contain benchmark-owned files"
        );
    }
}

#[test]
fn benchmark_harness_should_cover_compiler_stages_and_backends() {
    let harness = fs::read_to_string(repo_root().join("benches/ckc_perf.rs"))
        .expect("read benchmark harness");

    for required in [
        "cargo bench --bench ckc_perf",
        "bench/perf/fixtures",
        "emit_c_module",
        "emit_wat_module_with_options",
        "emit_wasm_module_with_options",
        "EmitWasmOptions { opt_level: 3 }",
        "emit_llvm_module",
        "run_mir_pass_pipeline",
        "build/perf/latest.summary.json",
        "build/perf/latest.summary.md",
    ] {
        assert!(
            harness.contains(required),
            "native benchmark harness must mention `{required}`"
        );
    }
}

#[test]
fn benchmark_docs_should_explain_native_cargo_bench_workflow() {
    let docs = [
        fs::read_to_string(repo_root().join("docs/PERFORMANCE.md")).expect("read performance doc"),
        fs::read_to_string(repo_root().join("docs/zh-CN/PERFORMANCE.md"))
            .expect("read zh performance doc"),
        fs::read_to_string(repo_root().join("docs/bench/README.md")).expect("read bench doc"),
        fs::read_to_string(repo_root().join("docs/bench/README.zh-CN.md"))
            .expect("read zh bench doc"),
    ];

    for text in docs {
        for required in [
            "cargo bench --bench ckc_perf",
            "build/perf/latest.summary.json",
            "build/perf/latest.summary.md",
        ] {
            assert!(
                text.contains(required),
                "benchmark docs must describe `{required}`"
            );
        }
    }
}
