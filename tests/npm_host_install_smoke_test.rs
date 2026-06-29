use std::process::Command;

#[test]
fn host_npm_install_verifier_should_pass_without_ckc_bin_override() {
    if !node_available() || !npm_available() {
        return;
    }

    let output = Command::new("node")
        .arg("scripts/verify-host-npm-install.mjs")
        .env_remove("CKC_BIN")
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .output()
        .expect("run host npm install verifier");

    assert!(
        output.status.success(),
        "host npm install verifier failed\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        String::from_utf8_lossy(&output.stdout).contains("\"typeSmoke\": \"passed\""),
        "host npm install verifier should report TypeScript declaration smoke\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        String::from_utf8_lossy(&output.stdout).contains("\"targetName\":"),
        "host npm install verifier should report the npm target name for release sign-off\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        String::from_utf8_lossy(&output.stdout).contains("\"tarballSha256\":"),
        "host npm install verifier should report the tarball SHA256 for release sign-off\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn host_npm_install_verifier_should_reject_missing_tarball_argument() {
    if !node_available() || !npm_available() {
        return;
    }

    let missing_tarball = std::env::temp_dir().join("rust-calckernel-missing-host-smoke.tgz");
    let _ = std::fs::remove_file(&missing_tarball);

    let output = Command::new("node")
        .arg("scripts/verify-host-npm-install.mjs")
        .arg(&missing_tarball)
        .env_remove("CKC_BIN")
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .output()
        .expect("run host npm install verifier with missing tarball");

    assert!(
        !output.status.success(),
        "missing tarball should fail\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        String::from_utf8_lossy(&output.stderr).contains("does not exist"),
        "missing tarball failure should mention missing file\nstdout:\n{}\nstderr:\n{}",
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

fn npm_available() -> bool {
    Command::new("npm")
        .arg("--version")
        .output()
        .is_ok_and(|output| output.status.success())
}
