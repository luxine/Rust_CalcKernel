use std::{
    fs,
    path::{Path, PathBuf},
};

#[test]
fn typescript_oracle_fixtures_should_be_covered_by_rust_backend_tests() {
    let report = audit_typescript_oracle_fixture_coverage()
        .expect("run TypeScript oracle fixture coverage audit");

    assert!(
        report.failures.is_empty(),
        "TypeScript oracle fixture coverage audit failed:\n{}",
        report.failures.join("\n")
    );

    assert!(
        report
            .generated_output_fixtures
            .iter()
            .any(|fixture| fixture == "tests/fixtures/f64_edges.ck"),
        "f64 edge fixture should be part of cross-backend generated output coverage"
    );
}

struct FixtureCoverageReport {
    generated_output_fixtures: Vec<String>,
    failures: Vec<String>,
}

fn audit_typescript_oracle_fixture_coverage() -> Result<FixtureCoverageReport, String> {
    let ts_root = typescript_root();
    let fixture_roots = ["examples", "bench/perf/fixtures", "tests/fixtures"];
    let backend_coverage = [
        ("MIR", "tests/mir_test.rs"),
        ("C", "tests/c_backend_test.rs"),
        ("WASM", "tests/wasm_backend_test.rs"),
        ("LLVM", "tests/llvm_backend_test.rs"),
    ];
    let mut failures = Vec::new();

    if !ts_root.exists() {
        failures.push(format!(
            "TypeScript oracle root is missing: {}",
            ts_root.display()
        ));
    }

    let mut fixtures = Vec::new();
    for fixture_root in fixture_roots {
        let root = ts_root.join(fixture_root);
        if root.exists() {
            fixtures.extend(list_ck_files(&ts_root, &root)?);
        } else {
            failures.push(format!(
                "TypeScript fixture directory is missing: {}",
                root.display()
            ));
        }
    }
    fixtures.sort();

    let mut backend_contents = Vec::new();
    for (label, path) in backend_coverage {
        let absolute = repo_root().join(path);
        match fs::read_to_string(&absolute) {
            Ok(text) => backend_contents.push((label, text)),
            Err(error) => failures.push(format!("Rust test file is missing: {path}: {error}")),
        }
    }

    let all_rust_tests = list_files(&repo_root().join("tests"))?
        .into_iter()
        .filter(|path| path.extension().and_then(|value| value.to_str()) == Some("rs"))
        .map(|path| {
            fs::read_to_string(&path).map_err(|error| format!("read {}: {error}", path.display()))
        })
        .collect::<Result<Vec<_>, _>>()?
        .join("\n");

    for fixture in &fixtures {
        for (label, contents) in &backend_contents {
            if !contents.contains(fixture) {
                failures.push(format!(
                    "{fixture} is missing from {label} backend oracle coverage"
                ));
            }
        }

        if !all_rust_tests.contains(fixture) {
            failures.push(format!(
                "{fixture} is not referenced by any Rust oracle test"
            ));
        }
    }

    Ok(FixtureCoverageReport {
        generated_output_fixtures: fixtures,
        failures,
    })
}

fn list_ck_files(base: &Path, dir: &Path) -> Result<Vec<String>, String> {
    Ok(list_files(dir)?
        .into_iter()
        .filter(|path| path.extension().and_then(|value| value.to_str()) == Some("ck"))
        .map(|path| normalize_relative(base, &path))
        .collect())
}

fn list_files(dir: &Path) -> Result<Vec<PathBuf>, String> {
    let mut files = Vec::new();
    for entry in fs::read_dir(dir).map_err(|error| format!("read {}: {error}", dir.display()))? {
        let path = entry
            .map_err(|error| format!("read entry in {}: {error}", dir.display()))?
            .path();
        if path.is_dir() {
            files.extend(list_files(&path)?);
        } else if path.is_file() {
            files.push(path);
        }
    }
    Ok(files)
}

fn normalize_relative(base: &Path, path: &Path) -> String {
    path.strip_prefix(base)
        .expect("fixture under TypeScript root")
        .to_string_lossy()
        .replace(std::path::MAIN_SEPARATOR, "/")
}

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn typescript_root() -> PathBuf {
    std::env::var_os("CALCKERNEL_TS_ROOT")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("/Users/lynn/code/CalcKernel"))
}
