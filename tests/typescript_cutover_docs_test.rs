use std::process::Command;

#[test]
fn rust_replacement_readiness_audit_should_not_require_typescript_checkout_edits() {
    if !node_available() {
        return;
    }

    let output = Command::new("node")
        .arg("scripts/audit-rust-replacement-readiness.mjs")
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .output()
        .expect("run Rust replacement readiness audit");

    assert!(
        output.status.success(),
        "Rust replacement readiness audit failed\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

fn node_available() -> bool {
    Command::new("node")
        .arg("--version")
        .output()
        .is_ok_and(|output| output.status.success())
}
