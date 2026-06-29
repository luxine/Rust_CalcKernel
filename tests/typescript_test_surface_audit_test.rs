use std::{
    fs,
    path::PathBuf,
    process::Command,
    time::{SystemTime, UNIX_EPOCH},
};

#[test]
fn typescript_test_surface_audit_should_be_registered_as_npm_script() {
    let package_json = fs::read_to_string("package.json").expect("read package.json");

    assert!(
        package_json.contains(
            "\"audit:typescript-test-surface\": \"node scripts/audit-typescript-test-surface.mjs\""
        ),
        "package.json must expose audit:typescript-test-surface"
    );
}

#[test]
fn typescript_test_surface_audit_should_accept_current_oracle_test_mapping() {
    if !node_available() {
        return;
    }

    let output = Command::new("node")
        .arg("scripts/audit-typescript-test-surface.mjs")
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .output()
        .expect("run TypeScript test surface audit");

    assert!(
        output.status.success(),
        "current TypeScript test surface should be fully mapped\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        String::from_utf8_lossy(&output.stdout).contains("\"status\": \"ok\""),
        "test surface audit should print JSON status\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn typescript_test_surface_audit_should_reject_unmapped_oracle_tests() {
    if !node_available() {
        return;
    }

    let temp = temp_dir("rust-calckernel-typescript-test-surface");
    let ts_tests = temp.join("ts").join("tests");
    let rust_tests = temp.join("rust").join("tests");
    fs::create_dir_all(&ts_tests).expect("create TypeScript test dir");
    fs::create_dir_all(&rust_tests).expect("create Rust test dir");
    fs::write(ts_tests.join("mapped.test.ts"), "it('mapped', () => {});\n")
        .expect("write mapped TypeScript test");
    fs::write(
        ts_tests.join("unmapped.test.ts"),
        "it('unmapped', () => {});\n",
    )
    .expect("write unmapped TypeScript test");
    fs::write(
        rust_tests.join("mapped_test.rs"),
        "#[test]\nfn mapped() {}\n",
    )
    .expect("write mapped Rust test");

    let mapping = temp.join("mapping.json");
    fs::write(
        &mapping,
        r#"[
  {
    "typescriptTest": "tests/mapped.test.ts",
    "rustTests": ["tests/mapped_test.rs"],
    "coverage": "mapped smoke coverage"
  }
]
"#,
    )
    .expect("write partial mapping");

    let output = Command::new("node")
        .arg("scripts/audit-typescript-test-surface.mjs")
        .arg("--typescript-root")
        .arg(temp.join("ts"))
        .arg("--rust-root")
        .arg(temp.join("rust"))
        .arg("--mapping")
        .arg(&mapping)
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .output()
        .expect("run TypeScript test surface audit");

    let _ = fs::remove_dir_all(&temp);

    assert!(
        !output.status.success(),
        "unmapped TypeScript tests should fail the audit\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        String::from_utf8_lossy(&output.stderr)
            .contains("missing migration mapping for tests/unmapped.test.ts"),
        "failure should identify the unmapped TypeScript test file\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

fn temp_dir(prefix: &str) -> PathBuf {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system time")
        .as_nanos();
    std::env::temp_dir().join(format!("{prefix}-{unique}"))
}

fn node_available() -> bool {
    Command::new("node")
        .arg("--version")
        .output()
        .is_ok_and(|output| output.status.success())
}
