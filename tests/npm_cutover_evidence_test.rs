use std::{
    fs,
    path::PathBuf,
    process::Command,
    time::{SystemTime, UNIX_EPOCH},
};

const TARBALL_SHA256: &str = "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef";
const BINARY_SHA256: &str = "1111111111111111111111111111111111111111111111111111111111111111";
const NODE_VERSION: &str = "v20.10.0";
const NPM_VERSION: &str = "10.2.0";
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
    let release_signoff_summary = temp.join("release-signoff-summary.json");
    let publish_artifact = temp.join("npm-publish-artifact.json");
    let publish_result = temp.join("npm-publish-result.json");
    fs::write(&manifest, release_manifest_json(TARBALL_SHA256)).expect("write manifest");
    fs::write(&signoff, release_signoff_json(TARBALL_SHA256)).expect("write signoff");
    fs::write(
        &release_signoff_summary,
        release_signoff_summary_json(TARBALL_SHA256),
    )
    .expect("write release signoff summary");
    fs::write(&publish_artifact, publish_artifact_json(TARBALL_SHA256))
        .expect("write publish artifact");
    fs::write(&publish_result, publish_result_json()).expect("write publish result");

    let output = Command::new("node")
        .arg("scripts/verify-npm-cutover-evidence.mjs")
        .arg(&manifest)
        .arg(&signoff)
        .arg(&release_signoff_summary)
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
        String::from_utf8_lossy(&output.stdout).contains("\"packageVersion\": \"0.8.0\""),
        "cutover evidence verifier should report the manifest packageVersion explicitly\nstdout:\n{}\nstderr:\n{}",
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
        String::from_utf8_lossy(&output.stdout).contains("\"platform\": \"linux\"")
            && String::from_utf8_lossy(&output.stdout).contains("\"arch\": \"x64\""),
        "cutover evidence verifier should preserve target platform and architecture evidence\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        String::from_utf8_lossy(&output.stdout)
            .contains(&format!("\"nodeVersion\": \"{NODE_VERSION}\""))
            && String::from_utf8_lossy(&output.stdout)
                .contains(&format!("\"npmVersion\": \"{NPM_VERSION}\"")),
        "cutover evidence verifier should preserve signed target Node/npm environment evidence\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        String::from_utf8_lossy(&output.stdout).contains("\"installedBin\":")
            && String::from_utf8_lossy(&output.stdout).contains("node_modules/.bin/ckc"),
        "cutover evidence verifier should preserve installed CLI path evidence\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        String::from_utf8_lossy(&output.stdout).contains("\"packagedBinary\":")
            && String::from_utf8_lossy(&output.stdout)
                .contains("node_modules/calckernel/npm/bin/ckc-linux-x64"),
        "cutover evidence verifier should preserve packaged binary path evidence\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        String::from_utf8_lossy(&output.stdout)
            .contains(&format!("\"packagedBinarySha256\": \"{BINARY_SHA256}\"")),
        "cutover evidence verifier should preserve packaged binary SHA256 evidence\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        String::from_utf8_lossy(&output.stdout).contains("\"releaseSignoffSummary\":"),
        "cutover evidence verifier should report the release signoff summary evidence path\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        String::from_utf8_lossy(&output.stdout)
            .contains("\"publishArtifactTarballPath\": \"/tmp/dist/calckernel-0.8.0.tgz\""),
        "cutover evidence verifier should preserve the pre-publish tarball path evidence\nstdout:\n{}\nstderr:\n{}",
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
        String::from_utf8_lossy(&output.stdout).contains("\"publishId\": \"calckernel@0.8.0\""),
        "cutover evidence verifier should preserve the npm publish package id\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        String::from_utf8_lossy(&output.stdout)
            .contains("\"publishFilename\": \"calckernel-0.8.0.tgz\""),
        "cutover evidence verifier should preserve the npm publish tarball filename\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        String::from_utf8_lossy(&output.stdout)
            .contains(&format!("\"publishShasum\": \"{VALID_SHASUM}\"")),
        "cutover evidence verifier should preserve the npm publish shasum\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        String::from_utf8_lossy(&output.stdout)
            .contains(&format!("\"publishIntegrity\": \"{VALID_INTEGRITY}\"")),
        "cutover evidence verifier should preserve the npm publish integrity\nstdout:\n{}\nstderr:\n{}",
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
    assert!(
        String::from_utf8_lossy(&output.stdout).contains("\"sourceFallback\": \"disabled\""),
        "cutover evidence verifier should report disabled source fallback evidence\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        String::from_utf8_lossy(&output.stdout).contains("\"ckcBinOverride\": \"unset\""),
        "cutover evidence verifier should preserve CKC_BIN unset evidence\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        String::from_utf8_lossy(&output.stdout).contains("\"commands\": [")
            && String::from_utf8_lossy(&output.stdout)
                .contains("\"ckc emit-llvm smoke.ck -o build/smoke.ll\""),
        "cutover evidence verifier should preserve CLI smoke command evidence\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        String::from_utf8_lossy(&output.stdout).contains("\"apiSymbols\": [")
            && String::from_utf8_lossy(&output.stdout).contains("\"emitCSource\""),
        "cutover evidence verifier should preserve package root API smoke evidence\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        String::from_utf8_lossy(&output.stdout).contains("\"typeSmoke\": \"passed\""),
        "cutover evidence verifier should preserve TypeScript declaration smoke evidence\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        String::from_utf8_lossy(&output.stdout).contains("\"backendRuntimeSmokes\": ["),
        "cutover evidence verifier should report backend runtime smoke evidence\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        String::from_utf8_lossy(&output.stdout).contains("\"node smoke-llvm-object-runtime.mjs\""),
        "cutover evidence verifier should report LLVM object runtime smoke evidence\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        String::from_utf8_lossy(&output.stdout).contains("\"description\": \"A small CK / CalcKernel integer-computation DSL compiler with C, WASM, and LLVM backends.\""),
        "cutover evidence verifier should report the public package description\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        String::from_utf8_lossy(&output.stdout).contains("\"keywords\": ["),
        "cutover evidence verifier should report the public package keywords\nstdout:\n{}\nstderr:\n{}",
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
    let release_signoff_summary = temp.join("release-signoff-summary.json");
    let publish_artifact = temp.join("npm-publish-artifact.json");
    let publish_result = temp.join("npm-publish-result.json");
    fs::write(&manifest, release_manifest_json(TARBALL_SHA256)).expect("write manifest");
    fs::write(
        &signoff,
        release_signoff_json("abcdef0123456789abcdef0123456789abcdef0123456789abcdef0123456789"),
    )
    .expect("write signoff");
    fs::write(
        &release_signoff_summary,
        release_signoff_summary_json(TARBALL_SHA256),
    )
    .expect("write release signoff summary");
    fs::write(&publish_artifact, publish_artifact_json(TARBALL_SHA256))
        .expect("write publish artifact");
    fs::write(&publish_result, publish_result_json()).expect("write publish result");

    let output = Command::new("node")
        .arg("scripts/verify-npm-cutover-evidence.mjs")
        .arg(&manifest)
        .arg(&signoff)
        .arg(&release_signoff_summary)
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
fn cutover_evidence_verifier_should_reject_release_signoff_summary_sha256_mismatch() {
    if !node_available() {
        return;
    }

    let temp = temp_dir("rust-calckernel-cutover-evidence-summary-mismatch");
    fs::create_dir_all(&temp).expect("create temp dir");
    let manifest = temp.join("release-manifest.json");
    let signoff = temp.join("release-signoff.json");
    let release_signoff_summary = temp.join("release-signoff-summary.json");
    let publish_artifact = temp.join("npm-publish-artifact.json");
    let publish_result = temp.join("npm-publish-result.json");
    fs::write(&manifest, release_manifest_json(TARBALL_SHA256)).expect("write manifest");
    fs::write(&signoff, release_signoff_json(TARBALL_SHA256)).expect("write signoff");
    fs::write(
        &release_signoff_summary,
        release_signoff_summary_json(
            "abcdef0123456789abcdef0123456789abcdef0123456789abcdef0123456789",
        ),
    )
    .expect("write release signoff summary");
    fs::write(&publish_artifact, publish_artifact_json(TARBALL_SHA256))
        .expect("write publish artifact");
    fs::write(&publish_result, publish_result_json()).expect("write publish result");

    let output = Command::new("node")
        .arg("scripts/verify-npm-cutover-evidence.mjs")
        .arg(&manifest)
        .arg(&signoff)
        .arg(&release_signoff_summary)
        .arg(&publish_artifact)
        .arg(&publish_result)
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .output()
        .expect("run cutover evidence verifier");

    let _ = fs::remove_dir_all(&temp);

    assert!(
        !output.status.success(),
        "mismatched release signoff summary SHA256 should fail\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        String::from_utf8_lossy(&output.stderr).contains("release sign-off summary tarballSha256"),
        "failure should identify release signoff summary SHA256 mismatch\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn cutover_evidence_verifier_should_reject_release_signoff_summary_source_fallback_mismatch() {
    if !node_available() {
        return;
    }

    let temp = temp_dir("rust-calckernel-cutover-evidence-source-fallback-mismatch");
    fs::create_dir_all(&temp).expect("create temp dir");
    let manifest = temp.join("release-manifest.json");
    let signoff = temp.join("release-signoff.json");
    let release_signoff_summary = temp.join("release-signoff-summary.json");
    let publish_artifact = temp.join("npm-publish-artifact.json");
    let publish_result = temp.join("npm-publish-result.json");
    fs::write(&manifest, release_manifest_json(TARBALL_SHA256)).expect("write manifest");
    fs::write(&signoff, release_signoff_json(TARBALL_SHA256)).expect("write signoff");
    fs::write(
        &release_signoff_summary,
        release_signoff_summary_json_with_source_fallback(TARBALL_SHA256, "enabled"),
    )
    .expect("write release signoff summary");
    fs::write(&publish_artifact, publish_artifact_json(TARBALL_SHA256))
        .expect("write publish artifact");
    fs::write(&publish_result, publish_result_json()).expect("write publish result");

    let output = Command::new("node")
        .arg("scripts/verify-npm-cutover-evidence.mjs")
        .arg(&manifest)
        .arg(&signoff)
        .arg(&release_signoff_summary)
        .arg(&publish_artifact)
        .arg(&publish_result)
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .output()
        .expect("run cutover evidence verifier");

    let _ = fs::remove_dir_all(&temp);

    assert!(
        !output.status.success(),
        "mismatched release signoff summary source fallback should fail\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        String::from_utf8_lossy(&output.stderr).contains("sourceFallback"),
        "failure should identify sourceFallback mismatch\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn cutover_evidence_verifier_should_reject_missing_release_signoff_summary_runtime_smokes() {
    if !node_available() {
        return;
    }

    let temp = temp_dir("rust-calckernel-cutover-evidence-summary-runtime-smokes");
    fs::create_dir_all(&temp).expect("create temp dir");
    let manifest = temp.join("release-manifest.json");
    let signoff = temp.join("release-signoff.json");
    let release_signoff_summary = temp.join("release-signoff-summary.json");
    let publish_artifact = temp.join("npm-publish-artifact.json");
    let publish_result = temp.join("npm-publish-result.json");
    fs::write(&manifest, release_manifest_json(TARBALL_SHA256)).expect("write manifest");
    fs::write(&signoff, release_signoff_json(TARBALL_SHA256)).expect("write signoff");
    fs::write(
        &release_signoff_summary,
        release_signoff_summary_json_without_runtime_smokes(TARBALL_SHA256),
    )
    .expect("write release signoff summary");
    fs::write(&publish_artifact, publish_artifact_json(TARBALL_SHA256))
        .expect("write publish artifact");
    fs::write(&publish_result, publish_result_json()).expect("write publish result");

    let output = Command::new("node")
        .arg("scripts/verify-npm-cutover-evidence.mjs")
        .arg(&manifest)
        .arg(&signoff)
        .arg(&release_signoff_summary)
        .arg(&publish_artifact)
        .arg(&publish_result)
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .output()
        .expect("run cutover evidence verifier");

    let _ = fs::remove_dir_all(&temp);

    assert!(
        !output.status.success(),
        "missing release signoff summary backend runtime smokes should fail\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        String::from_utf8_lossy(&output.stderr).contains("backendRuntimeSmokes"),
        "failure should identify release signoff summary backendRuntimeSmokes\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn cutover_evidence_verifier_should_reject_missing_release_signoff_type_smoke() {
    if !node_available() {
        return;
    }

    let temp = temp_dir("rust-calckernel-cutover-evidence-signoff-type-smoke");
    fs::create_dir_all(&temp).expect("create temp dir");
    let manifest = temp.join("release-manifest.json");
    let signoff = temp.join("release-signoff.json");
    let release_signoff_summary = temp.join("release-signoff-summary.json");
    let publish_artifact = temp.join("npm-publish-artifact.json");
    let publish_result = temp.join("npm-publish-result.json");
    fs::write(&manifest, release_manifest_json(TARBALL_SHA256)).expect("write manifest");
    fs::write(
        &signoff,
        release_signoff_json_without_type_smoke(TARBALL_SHA256),
    )
    .expect("write signoff");
    fs::write(
        &release_signoff_summary,
        release_signoff_summary_json(TARBALL_SHA256),
    )
    .expect("write release signoff summary");
    fs::write(&publish_artifact, publish_artifact_json(TARBALL_SHA256))
        .expect("write publish artifact");
    fs::write(&publish_result, publish_result_json()).expect("write publish result");

    let output = Command::new("node")
        .arg("scripts/verify-npm-cutover-evidence.mjs")
        .arg(&manifest)
        .arg(&signoff)
        .arg(&release_signoff_summary)
        .arg(&publish_artifact)
        .arg(&publish_result)
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .output()
        .expect("run cutover evidence verifier");

    let _ = fs::remove_dir_all(&temp);

    assert!(
        !output.status.success(),
        "missing release signoff typeSmoke should fail\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        String::from_utf8_lossy(&output.stderr).contains("typeSmoke"),
        "failure should identify release signoff typeSmoke evidence\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn cutover_evidence_verifier_should_reject_summary_type_smoke_mismatch() {
    if !node_available() {
        return;
    }

    let temp = temp_dir("rust-calckernel-cutover-evidence-summary-type-smoke");
    fs::create_dir_all(&temp).expect("create temp dir");
    let manifest = temp.join("release-manifest.json");
    let signoff = temp.join("release-signoff.json");
    let release_signoff_summary = temp.join("release-signoff-summary.json");
    let publish_artifact = temp.join("npm-publish-artifact.json");
    let publish_result = temp.join("npm-publish-result.json");
    fs::write(&manifest, release_manifest_json(TARBALL_SHA256)).expect("write manifest");
    fs::write(&signoff, release_signoff_json(TARBALL_SHA256)).expect("write signoff");
    fs::write(
        &release_signoff_summary,
        release_signoff_summary_json_with_type_smoke(TARBALL_SHA256, "skipped"),
    )
    .expect("write release signoff summary");
    fs::write(&publish_artifact, publish_artifact_json(TARBALL_SHA256))
        .expect("write publish artifact");
    fs::write(&publish_result, publish_result_json()).expect("write publish result");

    let output = Command::new("node")
        .arg("scripts/verify-npm-cutover-evidence.mjs")
        .arg(&manifest)
        .arg(&signoff)
        .arg(&release_signoff_summary)
        .arg(&publish_artifact)
        .arg(&publish_result)
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .output()
        .expect("run cutover evidence verifier");

    let _ = fs::remove_dir_all(&temp);

    assert!(
        !output.status.success(),
        "mismatched release signoff summary typeSmoke should fail\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        String::from_utf8_lossy(&output.stderr).contains("typeSmoke"),
        "failure should identify release signoff summary typeSmoke mismatch\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn cutover_evidence_verifier_should_reject_missing_release_signoff_commands() {
    if !node_available() {
        return;
    }

    let temp = temp_dir("rust-calckernel-cutover-evidence-signoff-commands");
    fs::create_dir_all(&temp).expect("create temp dir");
    let manifest = temp.join("release-manifest.json");
    let signoff = temp.join("release-signoff.json");
    let release_signoff_summary = temp.join("release-signoff-summary.json");
    let publish_artifact = temp.join("npm-publish-artifact.json");
    let publish_result = temp.join("npm-publish-result.json");
    fs::write(&manifest, release_manifest_json(TARBALL_SHA256)).expect("write manifest");
    fs::write(
        &signoff,
        release_signoff_json_without_commands(TARBALL_SHA256),
    )
    .expect("write signoff");
    fs::write(
        &release_signoff_summary,
        release_signoff_summary_json(TARBALL_SHA256),
    )
    .expect("write release signoff summary");
    fs::write(&publish_artifact, publish_artifact_json(TARBALL_SHA256))
        .expect("write publish artifact");
    fs::write(&publish_result, publish_result_json()).expect("write publish result");

    let output = Command::new("node")
        .arg("scripts/verify-npm-cutover-evidence.mjs")
        .arg(&manifest)
        .arg(&signoff)
        .arg(&release_signoff_summary)
        .arg(&publish_artifact)
        .arg(&publish_result)
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .output()
        .expect("run cutover evidence verifier");

    let _ = fs::remove_dir_all(&temp);

    assert!(
        !output.status.success(),
        "missing release signoff commands should fail\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        String::from_utf8_lossy(&output.stderr).contains("commands"),
        "failure should identify release signoff commands evidence\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn cutover_evidence_verifier_should_reject_summary_api_symbols_mismatch() {
    if !node_available() {
        return;
    }

    let temp = temp_dir("rust-calckernel-cutover-evidence-summary-api-symbols");
    fs::create_dir_all(&temp).expect("create temp dir");
    let manifest = temp.join("release-manifest.json");
    let signoff = temp.join("release-signoff.json");
    let release_signoff_summary = temp.join("release-signoff-summary.json");
    let publish_artifact = temp.join("npm-publish-artifact.json");
    let publish_result = temp.join("npm-publish-result.json");
    fs::write(&manifest, release_manifest_json(TARBALL_SHA256)).expect("write manifest");
    fs::write(&signoff, release_signoff_json(TARBALL_SHA256)).expect("write signoff");
    fs::write(
        &release_signoff_summary,
        release_signoff_summary_json_without_api_symbols(TARBALL_SHA256),
    )
    .expect("write release signoff summary");
    fs::write(&publish_artifact, publish_artifact_json(TARBALL_SHA256))
        .expect("write publish artifact");
    fs::write(&publish_result, publish_result_json()).expect("write publish result");

    let output = Command::new("node")
        .arg("scripts/verify-npm-cutover-evidence.mjs")
        .arg(&manifest)
        .arg(&signoff)
        .arg(&release_signoff_summary)
        .arg(&publish_artifact)
        .arg(&publish_result)
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .output()
        .expect("run cutover evidence verifier");

    let _ = fs::remove_dir_all(&temp);

    assert!(
        !output.status.success(),
        "missing release signoff summary apiSymbols should fail\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        String::from_utf8_lossy(&output.stderr).contains("apiSymbols"),
        "failure should identify release signoff summary apiSymbols evidence\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn cutover_evidence_verifier_should_reject_summary_ckc_bin_override_mismatch() {
    if !node_available() {
        return;
    }

    let temp = temp_dir("rust-calckernel-cutover-evidence-summary-ckc-bin");
    fs::create_dir_all(&temp).expect("create temp dir");
    let manifest = temp.join("release-manifest.json");
    let signoff = temp.join("release-signoff.json");
    let release_signoff_summary = temp.join("release-signoff-summary.json");
    let publish_artifact = temp.join("npm-publish-artifact.json");
    let publish_result = temp.join("npm-publish-result.json");
    fs::write(&manifest, release_manifest_json(TARBALL_SHA256)).expect("write manifest");
    fs::write(&signoff, release_signoff_json(TARBALL_SHA256)).expect("write signoff");
    fs::write(
        &release_signoff_summary,
        release_signoff_summary_json_with_ckc_bin_override(TARBALL_SHA256, "set"),
    )
    .expect("write release signoff summary");
    fs::write(&publish_artifact, publish_artifact_json(TARBALL_SHA256))
        .expect("write publish artifact");
    fs::write(&publish_result, publish_result_json()).expect("write publish result");

    let output = Command::new("node")
        .arg("scripts/verify-npm-cutover-evidence.mjs")
        .arg(&manifest)
        .arg(&signoff)
        .arg(&release_signoff_summary)
        .arg(&publish_artifact)
        .arg(&publish_result)
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .output()
        .expect("run cutover evidence verifier");

    let _ = fs::remove_dir_all(&temp);

    assert!(
        !output.status.success(),
        "mismatched release signoff summary ckcBinOverride should fail\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        String::from_utf8_lossy(&output.stderr).contains("ckcBinOverride"),
        "failure should identify release signoff summary ckcBinOverride mismatch\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn cutover_evidence_verifier_should_reject_missing_release_signoff_summary_package_version() {
    if !node_available() {
        return;
    }

    let temp = temp_dir("rust-calckernel-cutover-evidence-summary-package-version");
    fs::create_dir_all(&temp).expect("create temp dir");
    let manifest = temp.join("release-manifest.json");
    let signoff = temp.join("release-signoff.json");
    let release_signoff_summary = temp.join("release-signoff-summary.json");
    let publish_artifact = temp.join("npm-publish-artifact.json");
    let publish_result = temp.join("npm-publish-result.json");
    fs::write(&manifest, release_manifest_json(TARBALL_SHA256)).expect("write manifest");
    fs::write(&signoff, release_signoff_json(TARBALL_SHA256)).expect("write signoff");
    fs::write(
        &release_signoff_summary,
        release_signoff_summary_json_without_package_version(TARBALL_SHA256),
    )
    .expect("write release signoff summary");
    fs::write(&publish_artifact, publish_artifact_json(TARBALL_SHA256))
        .expect("write publish artifact");
    fs::write(&publish_result, publish_result_json()).expect("write publish result");

    let output = Command::new("node")
        .arg("scripts/verify-npm-cutover-evidence.mjs")
        .arg(&manifest)
        .arg(&signoff)
        .arg(&release_signoff_summary)
        .arg(&publish_artifact)
        .arg(&publish_result)
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .output()
        .expect("run cutover evidence verifier");

    let _ = fs::remove_dir_all(&temp);

    assert!(
        !output.status.success(),
        "missing release signoff summary packageVersion should fail\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        String::from_utf8_lossy(&output.stderr).contains("release sign-off summary packageVersion"),
        "failure should identify release signoff summary packageVersion\nstdout:\n{}\nstderr:\n{}",
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
    let release_signoff_summary = temp.join("release-signoff-summary.json");
    let publish_artifact = temp.join("npm-publish-artifact.json");
    let publish_result = temp.join("npm-publish-result.json");
    fs::write(&manifest, release_manifest_json(TARBALL_SHA256)).expect("write manifest");
    fs::write(&signoff, release_signoff_json(TARBALL_SHA256)).expect("write signoff");
    fs::write(
        &release_signoff_summary,
        release_signoff_summary_json(TARBALL_SHA256),
    )
    .expect("write release signoff summary");
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
        .arg(&release_signoff_summary)
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
fn cutover_evidence_verifier_should_reject_missing_publish_artifact_tarball_path() {
    if !node_available() {
        return;
    }

    let temp = temp_dir("rust-calckernel-cutover-evidence-publish-artifact-path");
    fs::create_dir_all(&temp).expect("create temp dir");
    let manifest = temp.join("release-manifest.json");
    let signoff = temp.join("release-signoff.json");
    let release_signoff_summary = temp.join("release-signoff-summary.json");
    let publish_artifact = temp.join("npm-publish-artifact.json");
    let publish_result = temp.join("npm-publish-result.json");
    fs::write(&manifest, release_manifest_json(TARBALL_SHA256)).expect("write manifest");
    fs::write(&signoff, release_signoff_json(TARBALL_SHA256)).expect("write signoff");
    fs::write(
        &release_signoff_summary,
        release_signoff_summary_json(TARBALL_SHA256),
    )
    .expect("write release signoff summary");
    fs::write(
        &publish_artifact,
        publish_artifact_json_without_tarball_path(TARBALL_SHA256),
    )
    .expect("write publish artifact");
    fs::write(&publish_result, publish_result_json()).expect("write publish result");

    let output = Command::new("node")
        .arg("scripts/verify-npm-cutover-evidence.mjs")
        .arg(&manifest)
        .arg(&signoff)
        .arg(&release_signoff_summary)
        .arg(&publish_artifact)
        .arg(&publish_result)
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .output()
        .expect("run cutover evidence verifier");

    let _ = fs::remove_dir_all(&temp);

    assert!(
        !output.status.success(),
        "missing publish artifact tarballPath should fail\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        String::from_utf8_lossy(&output.stderr).contains("publish artifact tarballPath"),
        "failure should identify publish artifact tarballPath\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn cutover_evidence_verifier_should_reject_publish_artifact_tarball_path_mismatch() {
    if !node_available() {
        return;
    }

    let temp = temp_dir("rust-calckernel-cutover-evidence-publish-artifact-path-mismatch");
    fs::create_dir_all(&temp).expect("create temp dir");
    let manifest = temp.join("release-manifest.json");
    let signoff = temp.join("release-signoff.json");
    let release_signoff_summary = temp.join("release-signoff-summary.json");
    let publish_artifact = temp.join("npm-publish-artifact.json");
    let publish_result = temp.join("npm-publish-result.json");
    fs::write(&manifest, release_manifest_json(TARBALL_SHA256)).expect("write manifest");
    fs::write(&signoff, release_signoff_json(TARBALL_SHA256)).expect("write signoff");
    fs::write(
        &release_signoff_summary,
        release_signoff_summary_json(TARBALL_SHA256),
    )
    .expect("write release signoff summary");
    fs::write(
        &publish_artifact,
        publish_artifact_json_with_tarball_path(TARBALL_SHA256, Some("/tmp/dist/other.tgz")),
    )
    .expect("write publish artifact");
    fs::write(&publish_result, publish_result_json()).expect("write publish result");

    let output = Command::new("node")
        .arg("scripts/verify-npm-cutover-evidence.mjs")
        .arg(&manifest)
        .arg(&signoff)
        .arg(&release_signoff_summary)
        .arg(&publish_artifact)
        .arg(&publish_result)
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .output()
        .expect("run cutover evidence verifier");

    let _ = fs::remove_dir_all(&temp);

    assert!(
        !output.status.success(),
        "mismatched publish artifact tarballPath should fail\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        String::from_utf8_lossy(&output.stderr).contains("publish artifact tarballPath"),
        "failure should identify publish artifact tarballPath mismatch\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn cutover_evidence_verifier_should_reject_missing_publish_result_package_version() {
    if !node_available() {
        return;
    }

    let temp = temp_dir("rust-calckernel-cutover-evidence-publish-result-package-version");
    fs::create_dir_all(&temp).expect("create temp dir");
    let manifest = temp.join("release-manifest.json");
    let signoff = temp.join("release-signoff.json");
    let release_signoff_summary = temp.join("release-signoff-summary.json");
    let publish_artifact = temp.join("npm-publish-artifact.json");
    let publish_result = temp.join("npm-publish-result.json");
    fs::write(&manifest, release_manifest_json(TARBALL_SHA256)).expect("write manifest");
    fs::write(&signoff, release_signoff_json(TARBALL_SHA256)).expect("write signoff");
    fs::write(
        &release_signoff_summary,
        release_signoff_summary_json(TARBALL_SHA256),
    )
    .expect("write release signoff summary");
    fs::write(&publish_artifact, publish_artifact_json(TARBALL_SHA256))
        .expect("write publish artifact");
    fs::write(
        &publish_result,
        publish_result_json_without_package_version(),
    )
    .expect("write publish result");

    let output = Command::new("node")
        .arg("scripts/verify-npm-cutover-evidence.mjs")
        .arg(&manifest)
        .arg(&signoff)
        .arg(&release_signoff_summary)
        .arg(&publish_artifact)
        .arg(&publish_result)
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .output()
        .expect("run cutover evidence verifier");

    let _ = fs::remove_dir_all(&temp);

    assert!(
        !output.status.success(),
        "missing publish result packageVersion should fail\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        String::from_utf8_lossy(&output.stderr).contains("publish result packageVersion"),
        "failure should identify publish result packageVersion\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn cutover_evidence_verifier_should_reject_missing_publish_result_public_identity() {
    if !node_available() {
        return;
    }

    let temp = temp_dir("rust-calckernel-cutover-evidence-publish-result-identity");
    fs::create_dir_all(&temp).expect("create temp dir");
    let manifest = temp.join("release-manifest.json");
    let signoff = temp.join("release-signoff.json");
    let release_signoff_summary = temp.join("release-signoff-summary.json");
    let publish_artifact = temp.join("npm-publish-artifact.json");
    let publish_result = temp.join("npm-publish-result.json");
    fs::write(&manifest, release_manifest_json(TARBALL_SHA256)).expect("write manifest");
    fs::write(&signoff, release_signoff_json(TARBALL_SHA256)).expect("write signoff");
    fs::write(
        &release_signoff_summary,
        release_signoff_summary_json(TARBALL_SHA256),
    )
    .expect("write release signoff summary");
    fs::write(&publish_artifact, publish_artifact_json(TARBALL_SHA256))
        .expect("write publish artifact");
    fs::write(
        &publish_result,
        publish_result_json_without_public_identity(),
    )
    .expect("write publish result");

    let output = Command::new("node")
        .arg("scripts/verify-npm-cutover-evidence.mjs")
        .arg(&manifest)
        .arg(&signoff)
        .arg(&release_signoff_summary)
        .arg(&publish_artifact)
        .arg(&publish_result)
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .output()
        .expect("run cutover evidence verifier");

    let _ = fs::remove_dir_all(&temp);

    assert!(
        !output.status.success(),
        "missing publish result public identity should fail\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        String::from_utf8_lossy(&output.stderr).contains("publish result description")
            || String::from_utf8_lossy(&output.stderr).contains("publish result keywords"),
        "failure should identify publish result public identity\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn cutover_evidence_verifier_should_reject_manifest_public_identity_mismatch() {
    if !node_available() {
        return;
    }

    let temp = temp_dir("rust-calckernel-cutover-evidence-manifest-identity");
    fs::create_dir_all(&temp).expect("create temp dir");
    let manifest = temp.join("release-manifest.json");
    let signoff = temp.join("release-signoff.json");
    let release_signoff_summary = temp.join("release-signoff-summary.json");
    let publish_artifact = temp.join("npm-publish-artifact.json");
    let publish_result = temp.join("npm-publish-result.json");
    fs::write(
        &manifest,
        release_manifest_json_with_description(TARBALL_SHA256, "Legacy TypeScript ckc package"),
    )
    .expect("write manifest");
    fs::write(&signoff, release_signoff_json(TARBALL_SHA256)).expect("write signoff");
    fs::write(
        &release_signoff_summary,
        release_signoff_summary_json(TARBALL_SHA256),
    )
    .expect("write release signoff summary");
    fs::write(&publish_artifact, publish_artifact_json(TARBALL_SHA256))
        .expect("write publish artifact");
    fs::write(&publish_result, publish_result_json()).expect("write publish result");

    let output = Command::new("node")
        .arg("scripts/verify-npm-cutover-evidence.mjs")
        .arg(&manifest)
        .arg(&signoff)
        .arg(&release_signoff_summary)
        .arg(&publish_artifact)
        .arg(&publish_result)
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .output()
        .expect("run cutover evidence verifier");

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
fn cutover_evidence_verifier_should_reject_signed_target_sha256_mismatch() {
    if !node_available() {
        return;
    }

    let temp = temp_dir("rust-calckernel-cutover-evidence-target-mismatch");
    fs::create_dir_all(&temp).expect("create temp dir");
    let manifest = temp.join("release-manifest.json");
    let signoff = temp.join("release-signoff.json");
    let release_signoff_summary = temp.join("release-signoff-summary.json");
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
    fs::write(
        &release_signoff_summary,
        release_signoff_summary_json(TARBALL_SHA256),
    )
    .expect("write release signoff summary");
    fs::write(&publish_artifact, publish_artifact_json(TARBALL_SHA256))
        .expect("write publish artifact");
    fs::write(&publish_result, publish_result_json()).expect("write publish result");

    let output = Command::new("node")
        .arg("scripts/verify-npm-cutover-evidence.mjs")
        .arg(&manifest)
        .arg(&signoff)
        .arg(&release_signoff_summary)
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

#[test]
fn cutover_evidence_verifier_should_reject_missing_summary_signed_target_platform_arch() {
    if !node_available() {
        return;
    }

    let temp = temp_dir("rust-calckernel-cutover-evidence-target-platform-arch");
    fs::create_dir_all(&temp).expect("create temp dir");
    let manifest = temp.join("release-manifest.json");
    let signoff = temp.join("release-signoff.json");
    let release_signoff_summary = temp.join("release-signoff-summary.json");
    let publish_artifact = temp.join("npm-publish-artifact.json");
    let publish_result = temp.join("npm-publish-result.json");
    fs::write(&manifest, release_manifest_json(TARBALL_SHA256)).expect("write manifest");
    fs::write(&signoff, release_signoff_json(TARBALL_SHA256)).expect("write signoff");
    fs::write(
        &release_signoff_summary,
        release_signoff_summary_json_without_signed_target_platform_arch(TARBALL_SHA256),
    )
    .expect("write release signoff summary");
    fs::write(&publish_artifact, publish_artifact_json(TARBALL_SHA256))
        .expect("write publish artifact");
    fs::write(&publish_result, publish_result_json()).expect("write publish result");

    let output = Command::new("node")
        .arg("scripts/verify-npm-cutover-evidence.mjs")
        .arg(&manifest)
        .arg(&signoff)
        .arg(&release_signoff_summary)
        .arg(&publish_artifact)
        .arg(&publish_result)
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .output()
        .expect("run cutover evidence verifier");

    let _ = fs::remove_dir_all(&temp);

    assert!(
        !output.status.success(),
        "missing summary signed target platform/arch should fail\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        String::from_utf8_lossy(&output.stderr).contains("platform")
            || String::from_utf8_lossy(&output.stderr).contains("arch"),
        "failure should identify signed target platform/arch evidence\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn cutover_evidence_verifier_should_reject_missing_signoff_signed_target_binary_paths() {
    if !node_available() {
        return;
    }

    let temp = temp_dir("rust-calckernel-cutover-evidence-signoff-target-binary-paths");
    fs::create_dir_all(&temp).expect("create temp dir");
    let manifest = temp.join("release-manifest.json");
    let signoff = temp.join("release-signoff.json");
    let release_signoff_summary = temp.join("release-signoff-summary.json");
    let publish_artifact = temp.join("npm-publish-artifact.json");
    let publish_result = temp.join("npm-publish-result.json");
    fs::write(&manifest, release_manifest_json(TARBALL_SHA256)).expect("write manifest");
    fs::write(
        &signoff,
        release_signoff_json_without_signed_target_binary_paths(TARBALL_SHA256),
    )
    .expect("write signoff");
    fs::write(
        &release_signoff_summary,
        release_signoff_summary_json(TARBALL_SHA256),
    )
    .expect("write release signoff summary");
    fs::write(&publish_artifact, publish_artifact_json(TARBALL_SHA256))
        .expect("write publish artifact");
    fs::write(&publish_result, publish_result_json()).expect("write publish result");

    let output = Command::new("node")
        .arg("scripts/verify-npm-cutover-evidence.mjs")
        .arg(&manifest)
        .arg(&signoff)
        .arg(&release_signoff_summary)
        .arg(&publish_artifact)
        .arg(&publish_result)
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .output()
        .expect("run cutover evidence verifier");

    let _ = fs::remove_dir_all(&temp);

    assert!(
        !output.status.success(),
        "missing signoff signed target binary path evidence should fail\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        String::from_utf8_lossy(&output.stderr).contains("installedBin")
            || String::from_utf8_lossy(&output.stderr).contains("packagedBinary"),
        "failure should identify signoff signed target binary path evidence\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn cutover_evidence_verifier_should_reject_missing_summary_signed_target_binary_paths() {
    if !node_available() {
        return;
    }

    let temp = temp_dir("rust-calckernel-cutover-evidence-summary-target-binary-paths");
    fs::create_dir_all(&temp).expect("create temp dir");
    let manifest = temp.join("release-manifest.json");
    let signoff = temp.join("release-signoff.json");
    let release_signoff_summary = temp.join("release-signoff-summary.json");
    let publish_artifact = temp.join("npm-publish-artifact.json");
    let publish_result = temp.join("npm-publish-result.json");
    fs::write(&manifest, release_manifest_json(TARBALL_SHA256)).expect("write manifest");
    fs::write(&signoff, release_signoff_json(TARBALL_SHA256)).expect("write signoff");
    fs::write(
        &release_signoff_summary,
        release_signoff_summary_json_without_signed_target_binary_paths(TARBALL_SHA256),
    )
    .expect("write release signoff summary");
    fs::write(&publish_artifact, publish_artifact_json(TARBALL_SHA256))
        .expect("write publish artifact");
    fs::write(&publish_result, publish_result_json()).expect("write publish result");

    let output = Command::new("node")
        .arg("scripts/verify-npm-cutover-evidence.mjs")
        .arg(&manifest)
        .arg(&signoff)
        .arg(&release_signoff_summary)
        .arg(&publish_artifact)
        .arg(&publish_result)
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .output()
        .expect("run cutover evidence verifier");

    let _ = fs::remove_dir_all(&temp);

    assert!(
        !output.status.success(),
        "missing summary signed target binary path evidence should fail\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        String::from_utf8_lossy(&output.stderr).contains("installedBin")
            || String::from_utf8_lossy(&output.stderr).contains("packagedBinary"),
        "failure should identify summary signed target binary path evidence\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn cutover_evidence_verifier_should_reject_missing_signoff_signed_target_runtime_environment() {
    if !node_available() {
        return;
    }

    let temp = temp_dir("rust-calckernel-cutover-evidence-signoff-target-runtime-env");
    fs::create_dir_all(&temp).expect("create temp dir");
    let manifest = temp.join("release-manifest.json");
    let signoff = temp.join("release-signoff.json");
    let release_signoff_summary = temp.join("release-signoff-summary.json");
    let publish_artifact = temp.join("npm-publish-artifact.json");
    let publish_result = temp.join("npm-publish-result.json");
    fs::write(&manifest, release_manifest_json(TARBALL_SHA256)).expect("write manifest");
    fs::write(
        &signoff,
        release_signoff_json_without_signed_target_runtime_environment(TARBALL_SHA256),
    )
    .expect("write signoff");
    fs::write(
        &release_signoff_summary,
        release_signoff_summary_json(TARBALL_SHA256),
    )
    .expect("write release signoff summary");
    fs::write(&publish_artifact, publish_artifact_json(TARBALL_SHA256))
        .expect("write publish artifact");
    fs::write(&publish_result, publish_result_json()).expect("write publish result");

    let output = Command::new("node")
        .arg("scripts/verify-npm-cutover-evidence.mjs")
        .arg(&manifest)
        .arg(&signoff)
        .arg(&release_signoff_summary)
        .arg(&publish_artifact)
        .arg(&publish_result)
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .output()
        .expect("run cutover evidence verifier");

    let _ = fs::remove_dir_all(&temp);

    assert!(
        !output.status.success(),
        "missing signoff signed target runtime environment evidence should fail\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        String::from_utf8_lossy(&output.stderr).contains("nodeVersion")
            || String::from_utf8_lossy(&output.stderr).contains("npmVersion"),
        "failure should identify signoff signed target Node/npm environment evidence\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn cutover_evidence_verifier_should_reject_missing_summary_signed_target_runtime_environment() {
    if !node_available() {
        return;
    }

    let temp = temp_dir("rust-calckernel-cutover-evidence-summary-target-runtime-env");
    fs::create_dir_all(&temp).expect("create temp dir");
    let manifest = temp.join("release-manifest.json");
    let signoff = temp.join("release-signoff.json");
    let release_signoff_summary = temp.join("release-signoff-summary.json");
    let publish_artifact = temp.join("npm-publish-artifact.json");
    let publish_result = temp.join("npm-publish-result.json");
    fs::write(&manifest, release_manifest_json(TARBALL_SHA256)).expect("write manifest");
    fs::write(&signoff, release_signoff_json(TARBALL_SHA256)).expect("write signoff");
    fs::write(
        &release_signoff_summary,
        release_signoff_summary_json_without_signed_target_runtime_environment(TARBALL_SHA256),
    )
    .expect("write release signoff summary");
    fs::write(&publish_artifact, publish_artifact_json(TARBALL_SHA256))
        .expect("write publish artifact");
    fs::write(&publish_result, publish_result_json()).expect("write publish result");

    let output = Command::new("node")
        .arg("scripts/verify-npm-cutover-evidence.mjs")
        .arg(&manifest)
        .arg(&signoff)
        .arg(&release_signoff_summary)
        .arg(&publish_artifact)
        .arg(&publish_result)
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .output()
        .expect("run cutover evidence verifier");

    let _ = fs::remove_dir_all(&temp);

    assert!(
        !output.status.success(),
        "missing summary signed target runtime environment evidence should fail\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        String::from_utf8_lossy(&output.stderr).contains("nodeVersion")
            || String::from_utf8_lossy(&output.stderr).contains("npmVersion"),
        "failure should identify summary signed target Node/npm environment evidence\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn cutover_evidence_verifier_should_reject_missing_publish_side_result_evidence() {
    if !node_available() {
        return;
    }

    let temp = temp_dir("rust-calckernel-cutover-evidence-publish-side-result");
    fs::create_dir_all(&temp).expect("create temp dir");
    let manifest = temp.join("release-manifest.json");
    let signoff = temp.join("release-signoff.json");
    let release_signoff_summary = temp.join("release-signoff-summary.json");
    let publish_artifact = temp.join("npm-publish-artifact.json");
    let publish_result = temp.join("npm-publish-result.json");
    fs::write(&manifest, release_manifest_json(TARBALL_SHA256)).expect("write manifest");
    fs::write(&signoff, release_signoff_json(TARBALL_SHA256)).expect("write signoff");
    fs::write(
        &release_signoff_summary,
        release_signoff_summary_json(TARBALL_SHA256),
    )
    .expect("write release signoff summary");
    fs::write(&publish_artifact, publish_artifact_json(TARBALL_SHA256))
        .expect("write publish artifact");
    fs::write(
        &publish_result,
        publish_result_json_without_publish_side_evidence(),
    )
    .expect("write publish result");

    let output = Command::new("node")
        .arg("scripts/verify-npm-cutover-evidence.mjs")
        .arg(&manifest)
        .arg(&signoff)
        .arg(&release_signoff_summary)
        .arg(&publish_artifact)
        .arg(&publish_result)
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .output()
        .expect("run cutover evidence verifier");

    let _ = fs::remove_dir_all(&temp);

    assert!(
        !output.status.success(),
        "missing publish-side result evidence should fail\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        String::from_utf8_lossy(&output.stderr).contains("publishId")
            || String::from_utf8_lossy(&output.stderr).contains("publishFilename")
            || String::from_utf8_lossy(&output.stderr).contains("publishShasum")
            || String::from_utf8_lossy(&output.stderr).contains("publishIntegrity"),
        "failure should identify missing publish-side result evidence\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

fn release_manifest_json(tarball_sha256: &str) -> String {
    release_manifest_json_with_description(
        tarball_sha256,
        "A small CK / CalcKernel integer-computation DSL compiler with C, WASM, and LLVM backends.",
    )
}

fn release_manifest_json_with_description(tarball_sha256: &str, description: &str) -> String {
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
    release_signoff_json_with_signed_target_sha256_and_type_smoke(
        tarball_sha256,
        signed_target_sha256,
        Some("passed"),
    )
}

fn release_signoff_json_without_type_smoke(tarball_sha256: &str) -> String {
    release_signoff_json_with_signed_target_sha256_and_type_smoke(
        tarball_sha256,
        BINARY_SHA256,
        None,
    )
}

fn release_signoff_json_without_commands(tarball_sha256: &str) -> String {
    release_signoff_json_with_evidence(
        tarball_sha256,
        BINARY_SHA256,
        Some("passed"),
        true,
        true,
        true,
        false,
        true,
    )
}

fn release_signoff_json_without_signed_target_binary_paths(tarball_sha256: &str) -> String {
    release_signoff_json_with_evidence(
        tarball_sha256,
        BINARY_SHA256,
        Some("passed"),
        false,
        true,
        true,
        true,
        true,
    )
}

fn release_signoff_json_without_signed_target_runtime_environment(tarball_sha256: &str) -> String {
    release_signoff_json_with_evidence(
        tarball_sha256,
        BINARY_SHA256,
        Some("passed"),
        true,
        false,
        true,
        true,
        true,
    )
}

fn release_signoff_json_with_signed_target_sha256_and_type_smoke(
    tarball_sha256: &str,
    signed_target_sha256: &str,
    type_smoke: Option<&str>,
) -> String {
    release_signoff_json_with_evidence(
        tarball_sha256,
        signed_target_sha256,
        type_smoke,
        true,
        true,
        true,
        true,
        true,
    )
}

// Test fixture builder keeps each release sign-off evidence toggle explicit at call sites.
#[expect(clippy::too_many_arguments)]
fn release_signoff_json_with_evidence(
    tarball_sha256: &str,
    signed_target_sha256: &str,
    type_smoke: Option<&str>,
    include_binary_paths: bool,
    include_runtime_environment: bool,
    include_ckc_bin_override: bool,
    include_commands: bool,
    include_api_symbols: bool,
) -> String {
    let signed_targets = signed_targets_json(
        signed_target_sha256,
        true,
        include_binary_paths,
        include_runtime_environment,
    );
    let type_smoke = type_smoke
        .map(|value| {
            format!(
                r#",
  "typeSmoke": "{value}""#
            )
        })
        .unwrap_or_default();
    let ckc_bin_override = if include_ckc_bin_override {
        r#",
  "ckcBinOverride": "unset""#
    } else {
        ""
    };
    let commands = if include_commands {
        r#",
  "commands": [
    "ckc --help",
    "ckc check smoke.ck",
    "ckc emit-mir smoke.ck -o build/smoke.mir",
    "ckc emit-c smoke.ck -o build/smoke.c",
    "ckc emit-wat smoke.ck -o build/smoke.wat",
    "ckc emit-wasm smoke.ck -o build/smoke.wasm",
    "ckc emit-llvm smoke.ck -o build/smoke.ll",
    "ckc build smoke.ck -o build/smoke-c",
    "node smoke-c-runtime.mjs",
    "node smoke-wasm-runtime.mjs",
    "ckc build-llvm smoke.ck --kind object -o build/smoke.o",
    "node smoke-llvm-object-runtime.mjs"
  ]"#
    } else {
        ""
    };
    let api_symbols = if include_api_symbols {
        r#",
  "apiSymbols": [
    "SourceFile",
    "TokenKind",
    "lex",
    "parse",
    "check",
    "getFunctionInfo",
    "emitCHeader",
    "emitCSource",
    "CKWasmArena",
    "createCKWasmArena"
  ]"#
    } else {
        ""
    };
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
{signed_targets}
  ],
  "sourceFallback": "disabled"{ckc_bin_override}{commands}{api_symbols}{type_smoke},
  "backendRuntimeSmokes": [
    "node smoke-c-runtime.mjs",
    "node smoke-wasm-runtime.mjs",
    "node smoke-llvm-object-runtime.mjs"
  ]
}}"#
    )
}

fn release_signoff_summary_json(tarball_sha256: &str) -> String {
    release_signoff_summary_json_with_source_fallback(tarball_sha256, "disabled")
}

fn release_signoff_summary_json_without_signed_target_platform_arch(
    tarball_sha256: &str,
) -> String {
    release_signoff_summary_json_with_package_version_source_fallback_runtime_smokes_and_platform_arch(
        true,
        tarball_sha256,
        "disabled",
        true,
        false,
        true,
    )
}

fn release_signoff_summary_json_without_signed_target_binary_paths(tarball_sha256: &str) -> String {
    release_signoff_summary_json_with_package_version_source_fallback_runtime_smokes_and_platform_arch(
        true,
        tarball_sha256,
        "disabled",
        true,
        true,
        false,
    )
}

fn release_signoff_summary_json_without_signed_target_runtime_environment(
    tarball_sha256: &str,
) -> String {
    release_signoff_summary_json_with_evidence(
        true,
        tarball_sha256,
        "disabled",
        true,
        "passed",
        true,
        true,
        false,
        Some("unset"),
        true,
        true,
    )
}

fn release_signoff_summary_json_with_source_fallback(
    tarball_sha256: &str,
    source_fallback: &str,
) -> String {
    release_signoff_summary_json_with_package_version_source_fallback_and_runtime_smokes(
        true,
        tarball_sha256,
        source_fallback,
        true,
    )
}

fn release_signoff_summary_json_without_package_version(tarball_sha256: &str) -> String {
    release_signoff_summary_json_with_package_version_source_fallback_and_runtime_smokes(
        false,
        tarball_sha256,
        "disabled",
        true,
    )
}

fn release_signoff_summary_json_without_runtime_smokes(tarball_sha256: &str) -> String {
    release_signoff_summary_json_with_package_version_source_fallback_and_runtime_smokes(
        true,
        tarball_sha256,
        "disabled",
        false,
    )
}

fn release_signoff_summary_json_with_type_smoke(tarball_sha256: &str, type_smoke: &str) -> String {
    release_signoff_summary_json_with_package_version_source_fallback_runtime_smokes_type_smoke_and_platform_arch(
        true,
        tarball_sha256,
        "disabled",
        true,
        type_smoke,
        true,
        true,
    )
}

fn release_signoff_summary_json_with_ckc_bin_override(
    tarball_sha256: &str,
    ckc_bin_override: &str,
) -> String {
    release_signoff_summary_json_with_evidence(
        true,
        tarball_sha256,
        "disabled",
        true,
        "passed",
        true,
        true,
        true,
        Some(ckc_bin_override),
        true,
        true,
    )
}

fn release_signoff_summary_json_without_api_symbols(tarball_sha256: &str) -> String {
    release_signoff_summary_json_with_evidence(
        true,
        tarball_sha256,
        "disabled",
        true,
        "passed",
        true,
        true,
        true,
        Some("unset"),
        true,
        false,
    )
}

fn release_signoff_summary_json_with_package_version_source_fallback_and_runtime_smokes(
    include_package_version: bool,
    tarball_sha256: &str,
    source_fallback: &str,
    include_runtime_smokes: bool,
) -> String {
    release_signoff_summary_json_with_package_version_source_fallback_runtime_smokes_and_platform_arch(
        include_package_version,
        tarball_sha256,
        source_fallback,
        include_runtime_smokes,
        true,
        true,
    )
}

fn release_signoff_summary_json_with_package_version_source_fallback_runtime_smokes_and_platform_arch(
    include_package_version: bool,
    tarball_sha256: &str,
    source_fallback: &str,
    include_runtime_smokes: bool,
    include_platform_arch: bool,
    include_binary_paths: bool,
) -> String {
    release_signoff_summary_json_with_package_version_source_fallback_runtime_smokes_type_smoke_and_platform_arch(
        include_package_version,
        tarball_sha256,
        source_fallback,
        include_runtime_smokes,
        "passed",
        include_platform_arch,
        include_binary_paths,
    )
}

fn release_signoff_summary_json_with_package_version_source_fallback_runtime_smokes_type_smoke_and_platform_arch(
    include_package_version: bool,
    tarball_sha256: &str,
    source_fallback: &str,
    include_runtime_smokes: bool,
    type_smoke: &str,
    include_platform_arch: bool,
    include_binary_paths: bool,
) -> String {
    release_signoff_summary_json_with_evidence(
        include_package_version,
        tarball_sha256,
        source_fallback,
        include_runtime_smokes,
        type_smoke,
        include_platform_arch,
        include_binary_paths,
        true,
        Some("unset"),
        true,
        true,
    )
}

// Test fixture builder keeps each release evidence toggle explicit at call sites.
#[expect(clippy::too_many_arguments)]
fn release_signoff_summary_json_with_evidence(
    include_package_version: bool,
    tarball_sha256: &str,
    source_fallback: &str,
    include_runtime_smokes: bool,
    type_smoke: &str,
    include_platform_arch: bool,
    include_binary_paths: bool,
    include_runtime_environment: bool,
    ckc_bin_override: Option<&str>,
    include_commands: bool,
    include_api_symbols: bool,
) -> String {
    let package_version = if include_package_version {
        r#",
  "packageVersion": "0.8.0""#
    } else {
        ""
    };
    let runtime_smokes = if include_runtime_smokes {
        r#",
  "backendRuntimeSmokes": [
    "node smoke-c-runtime.mjs",
    "node smoke-wasm-runtime.mjs",
    "node smoke-llvm-object-runtime.mjs"
  ]"#
    } else {
        ""
    };
    let ckc_bin_override = ckc_bin_override
        .map(|value| {
            format!(
                r#",
  "ckcBinOverride": "{value}""#
            )
        })
        .unwrap_or_default();
    let commands = if include_commands {
        r#",
  "commands": [
    "ckc --help",
    "ckc check smoke.ck",
    "ckc emit-mir smoke.ck -o build/smoke.mir",
    "ckc emit-c smoke.ck -o build/smoke.c",
    "ckc emit-wat smoke.ck -o build/smoke.wat",
    "ckc emit-wasm smoke.ck -o build/smoke.wasm",
    "ckc emit-llvm smoke.ck -o build/smoke.ll",
    "ckc build smoke.ck -o build/smoke-c",
    "node smoke-c-runtime.mjs",
    "node smoke-wasm-runtime.mjs",
    "ckc build-llvm smoke.ck --kind object -o build/smoke.o",
    "node smoke-llvm-object-runtime.mjs"
  ]"#
    } else {
        ""
    };
    let api_symbols = if include_api_symbols {
        r#",
  "apiSymbols": [
    "SourceFile",
    "TokenKind",
    "lex",
    "parse",
    "check",
    "getFunctionInfo",
    "emitCHeader",
    "emitCSource",
    "CKWasmArena",
    "createCKWasmArena"
  ]"#
    } else {
        ""
    };
    let signed_targets = signed_targets_json(
        BINARY_SHA256,
        include_platform_arch,
        include_binary_paths,
        include_runtime_environment,
    );
    format!(
        r#"{{
  "status": "ok",
  "package": "calckernel",
  "version": "0.8.0"{package_version},
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
{signed_targets}
  ],
  "sourceFallback": "{source_fallback}"{ckc_bin_override}{commands}{api_symbols},
  "typeSmoke": "{type_smoke}"{runtime_smokes}
}}"#
    )
}

fn signed_targets_json(
    linux_x64_sha256: &str,
    include_platform_arch: bool,
    include_binary_paths: bool,
    include_runtime_environment: bool,
) -> String {
    [
        ("darwin-arm64", "darwin", "arm64", BINARY_SHA256),
        ("darwin-x64", "darwin", "x64", BINARY_SHA256),
        ("linux-arm64", "linux", "arm64", BINARY_SHA256),
        ("linux-x64", "linux", "x64", linux_x64_sha256),
        ("win32-arm64", "win32", "arm64", BINARY_SHA256),
        ("win32-x64", "win32", "x64", BINARY_SHA256),
    ]
    .into_iter()
    .map(|(name, platform, arch, sha256)| {
        let platform_arch = if include_platform_arch {
            format!(r#", "platform": "{platform}", "arch": "{arch}""#)
        } else {
            String::new()
        };
        let binary_paths = if include_binary_paths {
            format!(
                r#", "installedBin": "{}", "packagedBinary": "{}", "packagedBinarySha256": "{sha256}""#,
                installed_bin_value(name),
                packaged_binary_value(name)
            )
        } else {
            String::new()
        };
        let runtime_environment = if include_runtime_environment {
            format!(r#", "nodeVersion": "{NODE_VERSION}", "npmVersion": "{NPM_VERSION}""#)
        } else {
            String::new()
        };
        format!(r#"    {{"name": "{name}"{platform_arch}, "sha256": "{sha256}"{runtime_environment}{binary_paths}}}"#)
    })
    .collect::<Vec<_>>()
    .join(",\n")
}

fn installed_bin_value(target: &str) -> String {
    if target.starts_with("win32-") {
        r#"C:\\consumer\\node_modules\\.bin\\ckc.cmd"#.to_string()
    } else {
        "/tmp/consumer/node_modules/.bin/ckc".to_string()
    }
}

fn packaged_binary_value(target: &str) -> String {
    let binary_file = match target {
        "win32-arm64" => "ckc-win32-arm64.exe".to_string(),
        "win32-x64" => "ckc-win32-x64.exe".to_string(),
        _ => format!("ckc-{target}"),
    };

    if target.starts_with("win32-") {
        format!(r#"C:\\consumer\\node_modules\\calckernel\\npm\\bin\\{binary_file}"#)
    } else {
        format!("/tmp/consumer/node_modules/calckernel/npm/bin/{binary_file}")
    }
}

fn publish_artifact_json(tarball_sha256: &str) -> String {
    publish_artifact_json_with_tarball_path(tarball_sha256, Some("/tmp/dist/calckernel-0.8.0.tgz"))
}

fn publish_artifact_json_without_tarball_path(tarball_sha256: &str) -> String {
    publish_artifact_json_with_tarball_path(tarball_sha256, None)
}

fn publish_artifact_json_with_tarball_path(
    tarball_sha256: &str,
    tarball_path: Option<&str>,
) -> String {
    let tarball_path = tarball_path.map_or_else(String::new, |path| {
        format!(
            r#",
  "tarballPath": "{path}""#
        )
    });
    format!(
        r#"{{
  "status": "ok",
  "package": "calckernel",
  "packageVersion": "0.8.0",
  "tarball": "calckernel-0.8.0.tgz"{tarball_path},
  "tarballSha256": "{tarball_sha256}"
}}"#
    )
}

fn publish_result_json() -> String {
    publish_result_json_with_shasum(VALID_SHASUM)
}

fn publish_result_json_with_shasum(shasum: &str) -> String {
    publish_result_json_with_package_version_shasum_and_publish_side_evidence(true, shasum, true)
}

fn publish_result_json_without_package_version() -> String {
    publish_result_json_with_package_version_shasum_and_publish_side_evidence(
        false,
        VALID_SHASUM,
        true,
    )
}

fn publish_result_json_without_publish_side_evidence() -> String {
    publish_result_json_with_package_version_shasum_and_publish_side_evidence(
        true,
        VALID_SHASUM,
        false,
    )
}

fn publish_result_json_with_package_version_shasum_and_publish_side_evidence(
    include_package_version: bool,
    shasum: &str,
    include_publish_side_evidence: bool,
) -> String {
    let package_version = if include_package_version {
        r#",
  "packageVersion": "0.8.0""#
    } else {
        ""
    };
    let publish_side_evidence = if include_publish_side_evidence {
        format!(
            r#",
  "publishId": "calckernel@0.8.0",
  "publishFilename": "calckernel-0.8.0.tgz",
  "publishShasum": "{shasum}",
  "publishIntegrity": "{VALID_INTEGRITY}""#
        )
    } else {
        String::new()
    };
    format!(
        r#"{{
  "status": "ok",
  "package": "calckernel",
  "version": "0.8.0"{package_version},
  "tarball": "calckernel-0.8.0.tgz",
  "publishPackage": "calckernel",
  "publishVersion": "0.8.0"{publish_side_evidence},
  "registryStatus": "ok",
  "registryTarball": "https://registry.npmjs.org/calckernel/-/calckernel-0.8.0.tgz",
  "shasum": "{shasum}",
  "description": "A small CK / CalcKernel integer-computation DSL compiler with C, WASM, and LLVM backends.",
  "keywords": ["calckernel", "ck", "compiler", "dsl", "c", "wasm", "llvm"],
  "license": "MIT",
  "engines": {{
    "node": ">=20"
  }},
  "consumerInstallScripts": [],
  "integrity": "{VALID_INTEGRITY}"
}}"#
    )
}

fn publish_result_json_without_public_identity() -> String {
    format!(
        r#"{{
  "status": "ok",
  "package": "calckernel",
  "version": "0.8.0",
  "packageVersion": "0.8.0",
  "tarball": "calckernel-0.8.0.tgz",
  "registryStatus": "ok",
  "registryTarball": "https://registry.npmjs.org/calckernel/-/calckernel-0.8.0.tgz",
  "shasum": "{VALID_SHASUM}",
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
