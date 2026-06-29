use std::{
    fs,
    path::PathBuf,
    process::Command,
    time::{SystemTime, UNIX_EPOCH},
};

#[test]
fn publish_result_verifier_should_be_registered_as_npm_script() {
    let package_json = fs::read_to_string("package.json").expect("read package.json");

    assert!(
        package_json
            .contains("\"verify:publish-result\": \"node scripts/verify-npm-publish-result.mjs\""),
        "package.json must expose verify:publish-result for the release workflow"
    );
}

#[test]
fn publish_result_verifier_should_accept_matching_manifest_publish_and_registry_outputs() {
    if !node_available() {
        return;
    }

    let temp = temp_dir("rust-calckernel-publish-result-ok");
    fs::create_dir_all(&temp).expect("create temp dir");
    let manifest = temp.join("release-manifest.json");
    let publish = temp.join("npm-publish.json");
    let registry = temp.join("npm-registry-replacement.json");
    fs::write(&manifest, release_manifest_json("calckernel-0.8.0.tgz")).expect("write manifest");
    fs::write(
        &publish,
        npm_publish_json("calckernel-0.8.0.tgz", "sha512-test"),
    )
    .expect("write publish output");
    fs::write(
        &registry,
        registry_replacement_json("calckernel-0.8.0.tgz", "sha512-test"),
    )
    .expect("write registry output");

    let output = Command::new("node")
        .arg("scripts/verify-npm-publish-result.mjs")
        .arg(&manifest)
        .arg(&publish)
        .arg(&registry)
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .output()
        .expect("run npm publish result verifier");

    let _ = fs::remove_dir_all(&temp);

    assert!(
        output.status.success(),
        "matching publish result should pass\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        String::from_utf8_lossy(&output.stdout).contains("\"status\": \"ok\""),
        "publish result verifier should print JSON status\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn publish_result_verifier_should_reject_registry_integrity_mismatch() {
    if !node_available() {
        return;
    }

    let temp = temp_dir("rust-calckernel-publish-result-integrity");
    fs::create_dir_all(&temp).expect("create temp dir");
    let manifest = temp.join("release-manifest.json");
    let publish = temp.join("npm-publish.json");
    let registry = temp.join("npm-registry-replacement.json");
    fs::write(&manifest, release_manifest_json("calckernel-0.8.0.tgz")).expect("write manifest");
    fs::write(
        &publish,
        npm_publish_json("calckernel-0.8.0.tgz", "sha512-published"),
    )
    .expect("write publish output");
    fs::write(
        &registry,
        registry_replacement_json("calckernel-0.8.0.tgz", "sha512-registry"),
    )
    .expect("write registry output");

    let output = Command::new("node")
        .arg("scripts/verify-npm-publish-result.mjs")
        .arg(&manifest)
        .arg(&publish)
        .arg(&registry)
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .output()
        .expect("run npm publish result verifier");

    let _ = fs::remove_dir_all(&temp);

    assert!(
        !output.status.success(),
        "mismatched registry integrity should fail\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        String::from_utf8_lossy(&output.stderr).contains("registry integrity"),
        "failure should identify registry integrity mismatch\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

fn release_manifest_json(tarball: &str) -> String {
    format!(
        r#"{{
  "packageName": "calckernel",
  "packageVersion": "0.8.0",
  "tarball": "{tarball}",
  "tarballSha256": "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef",
  "targets": []
}}"#
    )
}

fn npm_publish_json(filename: &str, integrity: &str) -> String {
    format!(
        r#"{{
  "id": "calckernel@0.8.0",
  "name": "calckernel",
  "version": "0.8.0",
  "filename": "{filename}",
  "integrity": "{integrity}"
}}"#
    )
}

fn registry_replacement_json(tarball: &str, integrity: &str) -> String {
    format!(
        r#"{{
  "status": "ok",
  "package": "calckernel",
  "version": "0.8.0",
  "tarball": "https://registry.npmjs.org/calckernel/-/{tarball}",
  "integrity": "{integrity}"
}}"#
    )
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
