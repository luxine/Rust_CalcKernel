use std::{
    fs,
    path::PathBuf,
    process::Command,
    time::{SystemTime, UNIX_EPOCH},
};

const VALID_SHASUM: &str = "0123456789abcdef0123456789abcdef01234567";
const VALID_INTEGRITY: &str = "sha512-AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA==";

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
        npm_publish_json("calckernel-0.8.0.tgz", VALID_INTEGRITY),
    )
    .expect("write publish output");
    fs::write(
        &registry,
        registry_replacement_json("calckernel-0.8.0.tgz", VALID_INTEGRITY),
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
    assert!(
        String::from_utf8_lossy(&output.stdout).contains("\"registryStatus\": \"ok\""),
        "publish result verifier should report successful registry replacement status\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        String::from_utf8_lossy(&output.stdout).contains("\"packageVersion\": \"0.8.0\""),
        "publish result verifier should report the manifest packageVersion explicitly\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        String::from_utf8_lossy(&output.stdout).contains("\"consumerInstallScripts\": []"),
        "publish result verifier should report that registry metadata has no consumer install lifecycle scripts\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        String::from_utf8_lossy(&output.stdout)
            .contains(&format!("\"shasum\": \"{VALID_SHASUM}\"")),
        "publish result verifier should report the registry shasum\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        String::from_utf8_lossy(&output.stdout).contains("\"description\": \"A small CK / CalcKernel integer-computation DSL compiler with C, WASM, and LLVM backends.\""),
        "publish result verifier should report the public package description\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        String::from_utf8_lossy(&output.stdout).contains("\"keywords\": ["),
        "publish result verifier should report the public package keywords\nstdout:\n{}\nstderr:\n{}",
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
        npm_publish_json("calckernel-0.8.0.tgz", VALID_INTEGRITY),
    )
    .expect("write publish output");
    fs::write(
        &registry,
        registry_replacement_json(
            "calckernel-0.8.0.tgz",
            "sha512-BBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBB==",
        ),
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

#[test]
fn publish_result_verifier_should_reject_registry_shasum_mismatch() {
    if !node_available() {
        return;
    }

    let temp = temp_dir("rust-calckernel-publish-result-shasum");
    fs::create_dir_all(&temp).expect("create temp dir");
    let manifest = temp.join("release-manifest.json");
    let publish = temp.join("npm-publish.json");
    let registry = temp.join("npm-registry-replacement.json");
    fs::write(&manifest, release_manifest_json("calckernel-0.8.0.tgz")).expect("write manifest");
    fs::write(
        &publish,
        npm_publish_json_with_shasum("calckernel-0.8.0.tgz", VALID_SHASUM, VALID_INTEGRITY),
    )
    .expect("write publish output");
    fs::write(
        &registry,
        registry_replacement_json_with_status_and_shasum(
            "ok",
            "calckernel-0.8.0.tgz",
            "abcdef0123456789abcdef0123456789abcdef01",
            VALID_INTEGRITY,
        ),
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
        "mismatched registry shasum should fail\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        String::from_utf8_lossy(&output.stderr).contains("registry shasum"),
        "failure should identify registry shasum mismatch\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn publish_result_verifier_should_reject_failed_registry_replacement_status() {
    if !node_available() {
        return;
    }

    let temp = temp_dir("rust-calckernel-publish-result-registry-status");
    fs::create_dir_all(&temp).expect("create temp dir");
    let manifest = temp.join("release-manifest.json");
    let publish = temp.join("npm-publish.json");
    let registry = temp.join("npm-registry-replacement.json");
    fs::write(&manifest, release_manifest_json("calckernel-0.8.0.tgz")).expect("write manifest");
    fs::write(
        &publish,
        npm_publish_json("calckernel-0.8.0.tgz", VALID_INTEGRITY),
    )
    .expect("write publish output");
    fs::write(
        &registry,
        registry_replacement_json_with_status("failed", "calckernel-0.8.0.tgz", VALID_INTEGRITY),
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
        "failed registry replacement status should fail\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        String::from_utf8_lossy(&output.stderr).contains("registry replacement status"),
        "failure should identify registry replacement status\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn publish_result_verifier_should_reject_missing_registry_package_version() {
    if !node_available() {
        return;
    }

    let temp = temp_dir("rust-calckernel-publish-result-registry-package-version");
    fs::create_dir_all(&temp).expect("create temp dir");
    let manifest = temp.join("release-manifest.json");
    let publish = temp.join("npm-publish.json");
    let registry = temp.join("npm-registry-replacement.json");
    fs::write(&manifest, release_manifest_json("calckernel-0.8.0.tgz")).expect("write manifest");
    fs::write(
        &publish,
        npm_publish_json("calckernel-0.8.0.tgz", VALID_INTEGRITY),
    )
    .expect("write publish output");
    fs::write(
        &registry,
        registry_replacement_json_without_package_version("calckernel-0.8.0.tgz", VALID_INTEGRITY),
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
        "missing registry packageVersion should fail\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        String::from_utf8_lossy(&output.stderr).contains("registry packageVersion"),
        "failure should identify registry packageVersion\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn publish_result_verifier_should_reject_missing_registry_public_identity() {
    if !node_available() {
        return;
    }

    let temp = temp_dir("rust-calckernel-publish-result-registry-identity");
    fs::create_dir_all(&temp).expect("create temp dir");
    let manifest = temp.join("release-manifest.json");
    let publish = temp.join("npm-publish.json");
    let registry = temp.join("npm-registry-replacement.json");
    fs::write(&manifest, release_manifest_json("calckernel-0.8.0.tgz")).expect("write manifest");
    fs::write(
        &publish,
        npm_publish_json("calckernel-0.8.0.tgz", VALID_INTEGRITY),
    )
    .expect("write publish output");
    fs::write(
        &registry,
        registry_replacement_json_without_public_identity("calckernel-0.8.0.tgz", VALID_INTEGRITY),
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
        "missing registry public identity should fail\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        String::from_utf8_lossy(&output.stderr).contains("registry description")
            || String::from_utf8_lossy(&output.stderr).contains("registry keywords"),
        "failure should identify registry public identity\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn publish_result_verifier_should_reject_manifest_public_identity_mismatch() {
    if !node_available() {
        return;
    }

    let temp = temp_dir("rust-calckernel-publish-result-manifest-identity");
    fs::create_dir_all(&temp).expect("create temp dir");
    let manifest = temp.join("release-manifest.json");
    let publish = temp.join("npm-publish.json");
    let registry = temp.join("npm-registry-replacement.json");
    fs::write(
        &manifest,
        release_manifest_json_with_description(
            "calckernel-0.8.0.tgz",
            "Legacy TypeScript ckc package",
        ),
    )
    .expect("write manifest");
    fs::write(
        &publish,
        npm_publish_json("calckernel-0.8.0.tgz", VALID_INTEGRITY),
    )
    .expect("write publish output");
    fs::write(
        &registry,
        registry_replacement_json("calckernel-0.8.0.tgz", VALID_INTEGRITY),
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
        "mismatched release manifest public identity should fail\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        String::from_utf8_lossy(&output.stderr).contains("release manifest packageMetadata")
            || String::from_utf8_lossy(&output.stderr).contains("release manifest description"),
        "failure should identify release manifest public identity\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn publish_result_verifier_should_reject_incomplete_release_manifest() {
    if !node_available() {
        return;
    }

    let temp = temp_dir("rust-calckernel-publish-result-incomplete-manifest");
    fs::create_dir_all(&temp).expect("create temp dir");
    let manifest = temp.join("release-manifest.json");
    let publish = temp.join("npm-publish.json");
    let registry = temp.join("npm-registry-replacement.json");
    fs::write(&manifest, incomplete_release_manifest_json()).expect("write manifest");
    fs::write(
        &publish,
        npm_publish_json("calckernel-0.8.0.tgz", VALID_INTEGRITY),
    )
    .expect("write publish output");
    fs::write(
        &registry,
        registry_replacement_json("calckernel-0.8.0.tgz", VALID_INTEGRITY),
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
        "incomplete release manifest should fail\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        String::from_utf8_lossy(&output.stderr).contains("release manifest tarballSha256"),
        "failure should identify missing release manifest tarballSha256\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn publish_result_verifier_should_reject_invalid_integrity_format() {
    if !node_available() {
        return;
    }

    let temp = temp_dir("rust-calckernel-publish-result-integrity-format");
    fs::create_dir_all(&temp).expect("create temp dir");
    let manifest = temp.join("release-manifest.json");
    let publish = temp.join("npm-publish.json");
    let registry = temp.join("npm-registry-replacement.json");
    fs::write(&manifest, release_manifest_json("calckernel-0.8.0.tgz")).expect("write manifest");
    fs::write(
        &publish,
        npm_publish_json("calckernel-0.8.0.tgz", "sha512-not-a-real-digest"),
    )
    .expect("write publish output");
    fs::write(
        &registry,
        registry_replacement_json("calckernel-0.8.0.tgz", "sha512-not-a-real-digest"),
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
        "invalid npm integrity should fail\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        String::from_utf8_lossy(&output.stderr).contains("integrity"),
        "failure should identify integrity format\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

fn release_manifest_json(tarball: &str) -> String {
    release_manifest_json_with_description(
        tarball,
        "A small CK / CalcKernel integer-computation DSL compiler with C, WASM, and LLVM backends.",
    )
}

fn release_manifest_json_with_description(tarball: &str, description: &str) -> String {
    format!(
        r#"{{
  "packageName": "calckernel",
  "packageVersion": "0.8.0",
  "packageMetadata": {{
    "description": "{description}",
    "keywords": ["calckernel", "ck", "compiler", "dsl", "c", "wasm", "llvm"],
    "license": "MIT",
    "engines": {{
      "node": ">=20"
    }},
    "type": "module",
    "main": "./npm/index.js",
    "types": "./npm/index.d.ts",
    "exports": {{
      ".": {{
        "types": "./npm/index.d.ts",
        "import": "./npm/index.js"
      }}
    }},
    "bin": {{
      "ckc": "./npm/ckc.js"
    }},
    "dependencyFields": {{}},
    "consumerInstallScripts": []
  }},
  "tarball": "{tarball}",
  "tarballSha256": "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef",
  "targets": []
}}"#
    )
}

fn incomplete_release_manifest_json() -> String {
    r#"{
  "packageName": "calckernel",
  "packageVersion": "0.8.0",
  "tarball": "calckernel-0.8.0.tgz"
}"#
    .to_string()
}

fn npm_publish_json(filename: &str, integrity: &str) -> String {
    npm_publish_json_with_shasum(filename, VALID_SHASUM, integrity)
}

fn npm_publish_json_with_shasum(filename: &str, shasum: &str, integrity: &str) -> String {
    format!(
        r#"{{
  "id": "calckernel@0.8.0",
  "name": "calckernel",
  "version": "0.8.0",
  "filename": "{filename}",
  "shasum": "{shasum}",
  "integrity": "{integrity}"
}}"#
    )
}

fn registry_replacement_json(tarball: &str, integrity: &str) -> String {
    registry_replacement_json_with_status("ok", tarball, integrity)
}

fn registry_replacement_json_with_status(status: &str, tarball: &str, integrity: &str) -> String {
    registry_replacement_json_with_package_version_status_and_shasum(
        true,
        status,
        tarball,
        VALID_SHASUM,
        integrity,
    )
}

fn registry_replacement_json_with_status_and_shasum(
    status: &str,
    tarball: &str,
    shasum: &str,
    integrity: &str,
) -> String {
    registry_replacement_json_with_package_version_status_and_shasum(
        true, status, tarball, shasum, integrity,
    )
}

fn registry_replacement_json_without_package_version(tarball: &str, integrity: &str) -> String {
    registry_replacement_json_with_package_version_status_and_shasum(
        false,
        "ok",
        tarball,
        VALID_SHASUM,
        integrity,
    )
}

fn registry_replacement_json_with_package_version_status_and_shasum(
    include_package_version: bool,
    status: &str,
    tarball: &str,
    shasum: &str,
    integrity: &str,
) -> String {
    let package_version = if include_package_version {
        r#",
  "packageVersion": "0.8.0""#
    } else {
        ""
    };
    format!(
        r#"{{
  "status": "{status}",
  "package": "calckernel",
  "version": "0.8.0"{package_version},
  "description": "A small CK / CalcKernel integer-computation DSL compiler with C, WASM, and LLVM backends.",
  "keywords": ["calckernel", "ck", "compiler", "dsl", "c", "wasm", "llvm"],
  "license": "MIT",
  "engines": {{
    "node": ">=20"
  }},
  "tarball": "https://registry.npmjs.org/calckernel/-/{tarball}",
  "shasum": "{shasum}",
  "consumerInstallScripts": [],
  "integrity": "{integrity}"
}}"#
    )
}

fn registry_replacement_json_without_public_identity(tarball: &str, integrity: &str) -> String {
    format!(
        r#"{{
  "status": "ok",
  "package": "calckernel",
  "version": "0.8.0",
  "packageVersion": "0.8.0",
  "tarball": "https://registry.npmjs.org/calckernel/-/{tarball}",
  "shasum": "{VALID_SHASUM}",
  "consumerInstallScripts": [],
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
