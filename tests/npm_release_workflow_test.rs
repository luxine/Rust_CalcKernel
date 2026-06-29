use std::{fs, process::Command};

#[test]
fn npm_release_workflow_should_build_pack_and_sign_off_all_targets() {
    if !node_available() {
        return;
    }

    let output = Command::new("node")
        .arg("scripts/audit-npm-release-workflow.mjs")
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .output()
        .expect("run npm release workflow audit");

    assert!(
        output.status.success(),
        "npm release workflow audit failed\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        String::from_utf8_lossy(&output.stdout).contains("\"publishJob\": true"),
        "npm release workflow audit should confirm a gated npm publish job\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn npm_release_workflow_should_test_registry_replacement_verifier_before_publish() {
    let workflow =
        fs::read_to_string(".github/workflows/npm-release.yml").expect("read npm release workflow");

    assert!(
        workflow.contains("--test npm_registry_replacement_test"),
        "release workflow must test registry replacement verifier before publish"
    );
}

fn node_available() -> bool {
    Command::new("node")
        .arg("--version")
        .output()
        .is_ok_and(|output| output.status.success())
}
