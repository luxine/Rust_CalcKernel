use std::{
    fs,
    path::PathBuf,
    process::Command,
    time::{SystemTime, UNIX_EPOCH},
};

const TARBALL_SHA256: &str = "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef";
const BINARY_SHA256: &str = "1111111111111111111111111111111111111111111111111111111111111111";
const VALID_SHASUM: &str = "0123456789abcdef0123456789abcdef01234567";
const VALID_INTEGRITY: &str = "sha512-AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA==";

#[test]
fn cutover_evidence_verifier_should_be_registered_as_npm_script() {
    let package_json = fs::read_to_string("package.json").expect("read package.json");

    assert!(
        package_json.contains(
            "\"verify:cutover-evidence\": \"node scripts/verify-npm-cutover-evidence.mjs\""
        ),
        "package.json must expose verify:cutover-evidence for final release artifact audits"
    );
}

#[test]
fn cutover_evidence_verifier_should_accept_matching_release_and_publish_evidence() {
    if !node_available() {
        return;
    }

    let temp = temp_dir("rust-calckernel-cutover-evidence-ok");
    fs::create_dir_all(&temp).expect("create temp dir");
    let manifest = temp.join("release-manifest.json");
    let signoff = temp.join("release-signoff.json");
    let publish_artifact = temp.join("npm-publish-artifact.json");
    let publish_result = temp.join("npm-publish-result.json");
    fs::write(&manifest, release_manifest_json(TARBALL_SHA256)).expect("write manifest");
    fs::write(&signoff, release_signoff_json(TARBALL_SHA256)).expect("write signoff");
    fs::write(&publish_artifact, publish_artifact_json(TARBALL_SHA256))
        .expect("write publish artifact");
    fs::write(&publish_result, publish_result_json()).expect("write publish result");

    let output = Command::new("node")
        .arg("scripts/verify-npm-cutover-evidence.mjs")
        .arg(&manifest)
        .arg(&signoff)
        .arg(&publish_artifact)
        .arg(&publish_result)
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .output()
        .expect("run cutover evidence verifier");

    let _ = fs::remove_dir_all(&temp);

    assert!(
        output.status.success(),
        "matching cutover evidence should pass\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        String::from_utf8_lossy(&output.stdout).contains("\"targetCount\": 6"),
        "cutover evidence verifier should report all six signed-off targets\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        String::from_utf8_lossy(&output.stdout).contains("\"signedTargets\": ["),
        "cutover evidence verifier should report signed target binary hashes\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        String::from_utf8_lossy(&output.stdout)
            .contains(&format!("\"sha256\": \"{BINARY_SHA256}\"")),
        "cutover evidence verifier should report signed target SHA256 values\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        String::from_utf8_lossy(&output.stdout).contains("\"registryStatus\": \"ok\""),
        "cutover evidence verifier should report successful registry replacement status\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        String::from_utf8_lossy(&output.stdout).contains(
            "\"registryTarball\": \"https://registry.npmjs.org/calckernel/-/calckernel-0.8.0.tgz\""
        ),
        "cutover evidence verifier should report the registry tarball URL\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        String::from_utf8_lossy(&output.stdout).contains("\"consumerInstallScripts\": []"),
        "cutover evidence verifier should report that registry metadata has no consumer install lifecycle scripts\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        String::from_utf8_lossy(&output.stdout)
            .contains(&format!("\"shasum\": \"{VALID_SHASUM}\"")),
        "cutover evidence verifier should report the registry shasum\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn cutover_evidence_verifier_should_reject_signoff_sha256_mismatch() {
    if !node_available() {
        return;
    }

    let temp = temp_dir("rust-calckernel-cutover-evidence-mismatch");
    fs::create_dir_all(&temp).expect("create temp dir");
    let manifest = temp.join("release-manifest.json");
    let signoff = temp.join("release-signoff.json");
    let publish_artifact = temp.join("npm-publish-artifact.json");
    let publish_result = temp.join("npm-publish-result.json");
    fs::write(&manifest, release_manifest_json(TARBALL_SHA256)).expect("write manifest");
    fs::write(
        &signoff,
        release_signoff_json("abcdef0123456789abcdef0123456789abcdef0123456789abcdef0123456789"),
    )
    .expect("write signoff");
    fs::write(&publish_artifact, publish_artifact_json(TARBALL_SHA256))
        .expect("write publish artifact");
    fs::write(&publish_result, publish_result_json()).expect("write publish result");

    let output = Command::new("node")
        .arg("scripts/verify-npm-cutover-evidence.mjs")
        .arg(&manifest)
        .arg(&signoff)
        .arg(&publish_artifact)
        .arg(&publish_result)
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .output()
        .expect("run cutover evidence verifier");

    let _ = fs::remove_dir_all(&temp);

    assert!(
        !output.status.success(),
        "mismatched signoff SHA256 should fail\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        String::from_utf8_lossy(&output.stderr).contains("release sign-off tarballSha256"),
        "failure should identify release sign-off SHA256 mismatch\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn cutover_evidence_verifier_should_reject_invalid_publish_result_shasum() {
    if !node_available() {
        return;
    }

    let temp = temp_dir("rust-calckernel-cutover-evidence-shasum");
    fs::create_dir_all(&temp).expect("create temp dir");
    let manifest = temp.join("release-manifest.json");
    let signoff = temp.join("release-signoff.json");
    let publish_artifact = temp.join("npm-publish-artifact.json");
    let publish_result = temp.join("npm-publish-result.json");
    fs::write(&manifest, release_manifest_json(TARBALL_SHA256)).expect("write manifest");
    fs::write(&signoff, release_signoff_json(TARBALL_SHA256)).expect("write signoff");
    fs::write(&publish_artifact, publish_artifact_json(TARBALL_SHA256))
        .expect("write publish artifact");
    fs::write(
        &publish_result,
        publish_result_json_with_shasum("not-a-sha1"),
    )
    .expect("write publish result");

    let output = Command::new("node")
        .arg("scripts/verify-npm-cutover-evidence.mjs")
        .arg(&manifest)
        .arg(&signoff)
        .arg(&publish_artifact)
        .arg(&publish_result)
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .output()
        .expect("run cutover evidence verifier");

    let _ = fs::remove_dir_all(&temp);

    assert!(
        !output.status.success(),
        "invalid publish result shasum should fail\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        String::from_utf8_lossy(&output.stderr).contains("publish result shasum"),
        "failure should identify publish result shasum\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn cutover_evidence_verifier_should_reject_signed_target_sha256_mismatch() {
    if !node_available() {
        return;
    }

    let temp = temp_dir("rust-calckernel-cutover-evidence-target-mismatch");
    fs::create_dir_all(&temp).expect("create temp dir");
    let manifest = temp.join("release-manifest.json");
    let signoff = temp.join("release-signoff.json");
    let publish_artifact = temp.join("npm-publish-artifact.json");
    let publish_result = temp.join("npm-publish-result.json");
    fs::write(&manifest, release_manifest_json(TARBALL_SHA256)).expect("write manifest");
    fs::write(
        &signoff,
        release_signoff_json_with_signed_target_sha256(
            TARBALL_SHA256,
            "abcdef0123456789abcdef0123456789abcdef0123456789abcdef0123456789",
        ),
    )
    .expect("write signoff");
    fs::write(&publish_artifact, publish_artifact_json(TARBALL_SHA256))
        .expect("write publish artifact");
    fs::write(&publish_result, publish_result_json()).expect("write publish result");

    let output = Command::new("node")
        .arg("scripts/verify-npm-cutover-evidence.mjs")
        .arg(&manifest)
        .arg(&signoff)
        .arg(&publish_artifact)
        .arg(&publish_result)
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .output()
        .expect("run cutover evidence verifier");

    let _ = fs::remove_dir_all(&temp);

    assert!(
        !output.status.success(),
        "mismatched signed target SHA256 should fail\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        String::from_utf8_lossy(&output.stderr).contains("signedTargets"),
        "failure should identify signed target SHA256 mismatch\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

fn release_manifest_json(tarball_sha256: &str) -> String {
    format!(
        r#"{{
  "packageName": "calckernel",
  "packageVersion": "0.8.0",
  "tarball": "calckernel-0.8.0.tgz",
  "tarballSha256": "{tarball_sha256}",
  "targets": [
    {{"name": "darwin-arm64", "sha256": "{BINARY_SHA256}"}},
    {{"name": "darwin-x64", "sha256": "{BINARY_SHA256}"}},
    {{"name": "linux-arm64", "sha256": "{BINARY_SHA256}"}},
    {{"name": "linux-x64", "sha256": "{BINARY_SHA256}"}},
    {{"name": "win32-arm64", "sha256": "{BINARY_SHA256}"}},
    {{"name": "win32-x64", "sha256": "{BINARY_SHA256}"}}
  ]
}}"#
    )
}

fn release_signoff_json(tarball_sha256: &str) -> String {
    release_signoff_json_with_signed_target_sha256(tarball_sha256, BINARY_SHA256)
}

fn release_signoff_json_with_signed_target_sha256(
    tarball_sha256: &str,
    signed_target_sha256: &str,
) -> String {
    format!(
        r#"{{
  "status": "ok",
  "package": "calckernel",
  "packageVersion": "0.8.0",
  "tarball": "calckernel-0.8.0.tgz",
  "tarballSha256": "{tarball_sha256}",
  "targetCount": 6,
  "targets": [
    "darwin-arm64",
    "darwin-x64",
    "linux-arm64",
    "linux-x64",
    "win32-arm64",
    "win32-x64"
  ],
  "signedTargets": [
    {{"name": "darwin-arm64", "sha256": "{BINARY_SHA256}"}},
    {{"name": "darwin-x64", "sha256": "{BINARY_SHA256}"}},
    {{"name": "linux-arm64", "sha256": "{BINARY_SHA256}"}},
    {{"name": "linux-x64", "sha256": "{signed_target_sha256}"}},
    {{"name": "win32-arm64", "sha256": "{BINARY_SHA256}"}},
    {{"name": "win32-x64", "sha256": "{BINARY_SHA256}"}}
  ]
}}"#
    )
}

fn publish_artifact_json(tarball_sha256: &str) -> String {
    format!(
        r#"{{
  "status": "ok",
  "package": "calckernel",
  "packageVersion": "0.8.0",
  "tarball": "calckernel-0.8.0.tgz",
  "tarballPath": "/tmp/dist/calckernel-0.8.0.tgz",
  "tarballSha256": "{tarball_sha256}"
}}"#
    )
}

fn publish_result_json() -> String {
    publish_result_json_with_shasum(VALID_SHASUM)
}

fn publish_result_json_with_shasum(shasum: &str) -> String {
    format!(
        r#"{{
  "status": "ok",
  "package": "calckernel",
  "version": "0.8.0",
  "tarball": "calckernel-0.8.0.tgz",
  "registryStatus": "ok",
  "registryTarball": "https://registry.npmjs.org/calckernel/-/calckernel-0.8.0.tgz",
  "shasum": "{shasum}",
  "consumerInstallScripts": [],
  "integrity": "{VALID_INTEGRITY}"
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
