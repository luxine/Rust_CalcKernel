use std::process::Command;

#[test]
fn typescript_oracle_fixtures_should_be_covered_by_rust_backend_tests() {
    if !node_available() {
        return;
    }

    let output = Command::new("node")
        .arg("scripts/audit-typescript-oracle-fixtures.mjs")
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .output()
        .expect("run TypeScript oracle fixture coverage audit");

    assert!(
        output.status.success(),
        "TypeScript oracle fixture coverage audit failed\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    let generated_output_section = stdout
        .split("\"generatedOutputFixtures\": [")
        .nth(1)
        .and_then(|section| section.split("\"auxiliaryFixtures\":").next())
        .expect("fixture coverage audit should print generated output fixture JSON");

    assert!(
        generated_output_section.contains("\"tests/fixtures/f64_edges.ck\""),
        "f64 edge fixture should be part of cross-backend generated output coverage\nstdout:\n{}\nstderr:\n{}",
        stdout,
        String::from_utf8_lossy(&output.stderr)
    );
}

fn node_available() -> bool {
    Command::new("node")
        .arg("--version")
        .output()
        .is_ok_and(|output| output.status.success())
}
