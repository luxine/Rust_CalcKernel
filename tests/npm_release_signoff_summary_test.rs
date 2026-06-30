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
const CI_PROVIDER: &str = "github-actions";
const GITHUB_RUN_ID: &str = "1234567890";
const GITHUB_RUN_ATTEMPT: &str = "2";
const GITHUB_SHA: &str = "abcdef0123456789abcdef0123456789abcdef01";
const GITHUB_WORKFLOW: &str = "npm release artifact";
const GITHUB_JOB: &str = "platform-signoff";

#[test]
fn release_signoff_summary_verifier_should_be_registered_as_npm_script() {
    let package_json = fs::read_to_string("package.json").expect("read package.json");

    assert!(
        package_json.contains(
            "\"verify:release-signoff-summary\": \"node scripts/verify-npm-release-signoff-summary.mjs\""
        ),
        "package.json must expose verify:release-signoff-summary for pre-publish signoff validation"
    );
}

#[test]
fn release_signoff_summary_verifier_should_accept_matching_manifest_and_summary() {
    if !node_available() {
        return;
    }

    let temp = temp_dir("rust-calckernel-release-signoff-summary-ok");
    fs::create_dir_all(&temp).expect("create temp dir");
    let manifest = temp.join("release-manifest.json");
    let signoff = temp.join("release-signoff.json");
    fs::write(&manifest, release_manifest_json(TARBALL_SHA256)).expect("write manifest");
    fs::write(&signoff, release_signoff_json(TARBALL_SHA256)).expect("write signoff");

    let output = Command::new("node")
        .arg("scripts/verify-npm-release-signoff-summary.mjs")
        .arg(&manifest)
        .arg(&signoff)
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .output()
        .expect("run release signoff summary verifier");

    let _ = fs::remove_dir_all(&temp);

    assert!(
        output.status.success(),
        "matching release signoff summary should pass\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        String::from_utf8_lossy(&output.stdout).contains("\"targetCount\": 6"),
        "release signoff summary verifier should report all six targets\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        String::from_utf8_lossy(&output.stdout).contains("\"packageVersion\": \"0.8.0\""),
        "release signoff summary verifier should report the manifest packageVersion explicitly\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        String::from_utf8_lossy(&output.stdout).contains("\"signedTargets\": ["),
        "release signoff summary verifier should report signed target binary hashes\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        String::from_utf8_lossy(&output.stdout)
            .contains(&format!("\"sha256\": \"{BINARY_SHA256}\"")),
        "release signoff summary verifier should report each signed target SHA256\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        String::from_utf8_lossy(&output.stdout).contains("\"platform\": \"linux\"")
            && String::from_utf8_lossy(&output.stdout).contains("\"arch\": \"x64\""),
        "release signoff summary verifier should preserve target platform and architecture evidence\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        String::from_utf8_lossy(&output.stdout)
            .contains(&format!("\"nodeVersion\": \"{NODE_VERSION}\""))
            && String::from_utf8_lossy(&output.stdout)
                .contains(&format!("\"npmVersion\": \"{NPM_VERSION}\"")),
        "release signoff summary verifier should preserve signed target Node/npm environment evidence\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        String::from_utf8_lossy(&output.stdout)
            .contains(&format!("\"ciProvider\": \"{CI_PROVIDER}\""))
            && String::from_utf8_lossy(&output.stdout)
                .contains(&format!("\"githubRunId\": \"{GITHUB_RUN_ID}\""))
            && String::from_utf8_lossy(&output.stdout).contains("\"runnerOs\": \"Linux\"")
            && String::from_utf8_lossy(&output.stdout).contains("\"runnerArch\": \"X64\""),
        "release signoff summary verifier should preserve GitHub Actions provenance and runner evidence\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        String::from_utf8_lossy(&output.stdout).contains("\"installedBin\":")
            && String::from_utf8_lossy(&output.stdout).contains("node_modules/.bin/ckc"),
        "release signoff summary verifier should preserve installed CLI path evidence\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        String::from_utf8_lossy(&output.stdout).contains("\"packagedBinary\":")
            && String::from_utf8_lossy(&output.stdout)
                .contains("node_modules/calckernel/npm/bin/ckc-linux-x64"),
        "release signoff summary verifier should preserve packaged binary path evidence\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        String::from_utf8_lossy(&output.stdout)
            .contains(&format!("\"packagedBinarySha256\": \"{BINARY_SHA256}\"")),
        "release signoff summary verifier should preserve packaged binary SHA256 evidence\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        String::from_utf8_lossy(&output.stdout).contains("\"sourceFallback\": \"disabled\""),
        "release signoff summary verifier should report disabled source fallback evidence\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        String::from_utf8_lossy(&output.stdout).contains("\"ckcBinOverride\": \"unset\""),
        "release signoff summary verifier should preserve CKC_BIN unset evidence\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        String::from_utf8_lossy(&output.stdout).contains("\"commands\": [")
            && String::from_utf8_lossy(&output.stdout)
                .contains("\"ckc emit-llvm smoke.ck -o build/smoke.ll\""),
        "release signoff summary verifier should preserve CLI smoke command evidence\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        String::from_utf8_lossy(&output.stdout).contains("\"apiSymbols\": [")
            && String::from_utf8_lossy(&output.stdout).contains("\"emitCSource\""),
        "release signoff summary verifier should preserve package root API smoke evidence\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        String::from_utf8_lossy(&output.stdout).contains("\"typeSmoke\": \"passed\""),
        "release signoff summary verifier should preserve TypeScript declaration smoke evidence\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        String::from_utf8_lossy(&output.stdout).contains("\"backendRuntimeSmokes\": ["),
        "release signoff summary verifier should report backend runtime smoke evidence\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn release_signoff_summary_verifier_should_reject_missing_source_fallback_evidence() {
    if !node_available() {
        return;
    }

    let temp = temp_dir("rust-calckernel-release-signoff-summary-source-fallback");
    fs::create_dir_all(&temp).expect("create temp dir");
    let manifest = temp.join("release-manifest.json");
    let signoff = temp.join("release-signoff.json");
    fs::write(&manifest, release_manifest_json(TARBALL_SHA256)).expect("write manifest");
    fs::write(
        &signoff,
        release_signoff_json_without_source_fallback(TARBALL_SHA256),
    )
    .expect("write signoff");

    let output = Command::new("node")
        .arg("scripts/verify-npm-release-signoff-summary.mjs")
        .arg(&manifest)
        .arg(&signoff)
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .output()
        .expect("run release signoff summary verifier");

    let _ = fs::remove_dir_all(&temp);

    assert!(
        !output.status.success(),
        "missing source fallback evidence should fail\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        String::from_utf8_lossy(&output.stderr).contains("sourceFallback"),
        "failure should identify missing sourceFallback evidence\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn release_signoff_summary_verifier_should_reject_missing_backend_runtime_smoke_evidence() {
    if !node_available() {
        return;
    }

    let temp = temp_dir("rust-calckernel-release-signoff-summary-runtime-smokes");
    fs::create_dir_all(&temp).expect("create temp dir");
    let manifest = temp.join("release-manifest.json");
    let signoff = temp.join("release-signoff.json");
    fs::write(&manifest, release_manifest_json(TARBALL_SHA256)).expect("write manifest");
    fs::write(
        &signoff,
        release_signoff_json_without_backend_runtime_smokes(TARBALL_SHA256),
    )
    .expect("write signoff");

    let output = Command::new("node")
        .arg("scripts/verify-npm-release-signoff-summary.mjs")
        .arg(&manifest)
        .arg(&signoff)
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .output()
        .expect("run release signoff summary verifier");

    let _ = fs::remove_dir_all(&temp);

    assert!(
        !output.status.success(),
        "missing backend runtime smoke evidence should fail\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        String::from_utf8_lossy(&output.stderr).contains("backendRuntimeSmokes"),
        "failure should identify missing backendRuntimeSmokes evidence\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn release_signoff_summary_verifier_should_reject_missing_type_smoke_evidence() {
    if !node_available() {
        return;
    }

    let temp = temp_dir("rust-calckernel-release-signoff-summary-type-smoke");
    fs::create_dir_all(&temp).expect("create temp dir");
    let manifest = temp.join("release-manifest.json");
    let signoff = temp.join("release-signoff.json");
    fs::write(&manifest, release_manifest_json(TARBALL_SHA256)).expect("write manifest");
    fs::write(
        &signoff,
        release_signoff_json_without_type_smoke(TARBALL_SHA256),
    )
    .expect("write signoff");

    let output = Command::new("node")
        .arg("scripts/verify-npm-release-signoff-summary.mjs")
        .arg(&manifest)
        .arg(&signoff)
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .output()
        .expect("run release signoff summary verifier");

    let _ = fs::remove_dir_all(&temp);

    assert!(
        !output.status.success(),
        "missing TypeScript declaration smoke evidence should fail\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        String::from_utf8_lossy(&output.stderr).contains("typeSmoke"),
        "failure should identify missing typeSmoke evidence\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn release_signoff_summary_verifier_should_reject_missing_ckc_bin_override_evidence() {
    if !node_available() {
        return;
    }

    let temp = temp_dir("rust-calckernel-release-signoff-summary-ckc-bin");
    fs::create_dir_all(&temp).expect("create temp dir");
    let manifest = temp.join("release-manifest.json");
    let signoff = temp.join("release-signoff.json");
    fs::write(&manifest, release_manifest_json(TARBALL_SHA256)).expect("write manifest");
    fs::write(
        &signoff,
        release_signoff_json_without_ckc_bin_override(TARBALL_SHA256),
    )
    .expect("write signoff");

    let output = Command::new("node")
        .arg("scripts/verify-npm-release-signoff-summary.mjs")
        .arg(&manifest)
        .arg(&signoff)
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .output()
        .expect("run release signoff summary verifier");

    let _ = fs::remove_dir_all(&temp);

    assert!(
        !output.status.success(),
        "missing CKC_BIN override evidence should fail\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        String::from_utf8_lossy(&output.stderr).contains("ckcBinOverride"),
        "failure should identify missing ckcBinOverride evidence\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn release_signoff_summary_verifier_should_reject_missing_cli_smoke_commands() {
    if !node_available() {
        return;
    }

    let temp = temp_dir("rust-calckernel-release-signoff-summary-commands");
    fs::create_dir_all(&temp).expect("create temp dir");
    let manifest = temp.join("release-manifest.json");
    let signoff = temp.join("release-signoff.json");
    fs::write(&manifest, release_manifest_json(TARBALL_SHA256)).expect("write manifest");
    fs::write(
        &signoff,
        release_signoff_json_without_commands(TARBALL_SHA256),
    )
    .expect("write signoff");

    let output = Command::new("node")
        .arg("scripts/verify-npm-release-signoff-summary.mjs")
        .arg(&manifest)
        .arg(&signoff)
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .output()
        .expect("run release signoff summary verifier");

    let _ = fs::remove_dir_all(&temp);

    assert!(
        !output.status.success(),
        "missing CLI smoke commands should fail\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        String::from_utf8_lossy(&output.stderr).contains("commands"),
        "failure should identify missing commands evidence\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn release_signoff_summary_verifier_should_reject_missing_api_symbol_evidence() {
    if !node_available() {
        return;
    }

    let temp = temp_dir("rust-calckernel-release-signoff-summary-api-symbols");
    fs::create_dir_all(&temp).expect("create temp dir");
    let manifest = temp.join("release-manifest.json");
    let signoff = temp.join("release-signoff.json");
    fs::write(&manifest, release_manifest_json(TARBALL_SHA256)).expect("write manifest");
    fs::write(
        &signoff,
        release_signoff_json_without_api_symbols(TARBALL_SHA256),
    )
    .expect("write signoff");

    let output = Command::new("node")
        .arg("scripts/verify-npm-release-signoff-summary.mjs")
        .arg(&manifest)
        .arg(&signoff)
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .output()
        .expect("run release signoff summary verifier");

    let _ = fs::remove_dir_all(&temp);

    assert!(
        !output.status.success(),
        "missing package root API symbol evidence should fail\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        String::from_utf8_lossy(&output.stderr).contains("apiSymbols"),
        "failure should identify missing apiSymbols evidence\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn release_signoff_summary_verifier_should_reject_sha256_mismatch() {
    if !node_available() {
        return;
    }

    let temp = temp_dir("rust-calckernel-release-signoff-summary-mismatch");
    fs::create_dir_all(&temp).expect("create temp dir");
    let manifest = temp.join("release-manifest.json");
    let signoff = temp.join("release-signoff.json");
    fs::write(&manifest, release_manifest_json(TARBALL_SHA256)).expect("write manifest");
    fs::write(
        &signoff,
        release_signoff_json("abcdef0123456789abcdef0123456789abcdef0123456789abcdef0123456789"),
    )
    .expect("write signoff");

    let output = Command::new("node")
        .arg("scripts/verify-npm-release-signoff-summary.mjs")
        .arg(&manifest)
        .arg(&signoff)
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .output()
        .expect("run release signoff summary verifier");

    let _ = fs::remove_dir_all(&temp);

    assert!(
        !output.status.success(),
        "mismatched release signoff summary should fail\nstdout:\n{}\nstderr:\n{}",
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
fn release_signoff_summary_verifier_should_reject_signed_target_sha256_mismatch() {
    if !node_available() {
        return;
    }

    let temp = temp_dir("rust-calckernel-release-signoff-summary-target-mismatch");
    fs::create_dir_all(&temp).expect("create temp dir");
    let manifest = temp.join("release-manifest.json");
    let signoff = temp.join("release-signoff.json");
    fs::write(&manifest, release_manifest_json(TARBALL_SHA256)).expect("write manifest");
    fs::write(
        &signoff,
        release_signoff_json_with_signed_target_sha256(
            TARBALL_SHA256,
            "abcdef0123456789abcdef0123456789abcdef0123456789abcdef0123456789",
        ),
    )
    .expect("write signoff");

    let output = Command::new("node")
        .arg("scripts/verify-npm-release-signoff-summary.mjs")
        .arg(&manifest)
        .arg(&signoff)
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .output()
        .expect("run release signoff summary verifier");

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
fn release_signoff_summary_verifier_should_reject_missing_signed_target_platform_arch() {
    if !node_available() {
        return;
    }

    let temp = temp_dir("rust-calckernel-release-signoff-summary-target-platform-arch");
    fs::create_dir_all(&temp).expect("create temp dir");
    let manifest = temp.join("release-manifest.json");
    let signoff = temp.join("release-signoff.json");
    fs::write(&manifest, release_manifest_json(TARBALL_SHA256)).expect("write manifest");
    fs::write(
        &signoff,
        release_signoff_json_without_signed_target_platform_arch(TARBALL_SHA256),
    )
    .expect("write signoff");

    let output = Command::new("node")
        .arg("scripts/verify-npm-release-signoff-summary.mjs")
        .arg(&manifest)
        .arg(&signoff)
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .output()
        .expect("run release signoff summary verifier");

    let _ = fs::remove_dir_all(&temp);

    assert!(
        !output.status.success(),
        "missing signed target platform/arch should fail\nstdout:\n{}\nstderr:\n{}",
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
fn release_signoff_summary_verifier_should_reject_missing_signed_target_binary_paths() {
    if !node_available() {
        return;
    }

    let temp = temp_dir("rust-calckernel-release-signoff-summary-target-binary-paths");
    fs::create_dir_all(&temp).expect("create temp dir");
    let manifest = temp.join("release-manifest.json");
    let signoff = temp.join("release-signoff.json");
    fs::write(&manifest, release_manifest_json(TARBALL_SHA256)).expect("write manifest");
    fs::write(
        &signoff,
        release_signoff_json_without_signed_target_binary_paths(TARBALL_SHA256),
    )
    .expect("write signoff");

    let output = Command::new("node")
        .arg("scripts/verify-npm-release-signoff-summary.mjs")
        .arg(&manifest)
        .arg(&signoff)
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .output()
        .expect("run release signoff summary verifier");

    let _ = fs::remove_dir_all(&temp);

    assert!(
        !output.status.success(),
        "missing signed target binary path evidence should fail\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        String::from_utf8_lossy(&output.stderr).contains("installedBin")
            || String::from_utf8_lossy(&output.stderr).contains("packagedBinary"),
        "failure should identify signed target binary path evidence\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn release_signoff_summary_verifier_should_reject_missing_signed_target_runtime_environment() {
    if !node_available() {
        return;
    }

    let temp = temp_dir("rust-calckernel-release-signoff-summary-target-runtime-env");
    fs::create_dir_all(&temp).expect("create temp dir");
    let manifest = temp.join("release-manifest.json");
    let signoff = temp.join("release-signoff.json");
    fs::write(&manifest, release_manifest_json(TARBALL_SHA256)).expect("write manifest");
    fs::write(
        &signoff,
        release_signoff_json_without_signed_target_runtime_environment(TARBALL_SHA256),
    )
    .expect("write signoff");

    let output = Command::new("node")
        .arg("scripts/verify-npm-release-signoff-summary.mjs")
        .arg(&manifest)
        .arg(&signoff)
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .output()
        .expect("run release signoff summary verifier");

    let _ = fs::remove_dir_all(&temp);

    assert!(
        !output.status.success(),
        "missing signed target runtime environment evidence should fail\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        String::from_utf8_lossy(&output.stderr).contains("nodeVersion")
            || String::from_utf8_lossy(&output.stderr).contains("npmVersion"),
        "failure should identify signed target Node/npm runtime environment evidence\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn release_signoff_summary_verifier_should_reject_missing_signed_target_ci_provenance() {
    if !node_available() {
        return;
    }

    let temp = temp_dir("rust-calckernel-release-signoff-summary-target-ci-provenance");
    fs::create_dir_all(&temp).expect("create temp dir");
    let manifest = temp.join("release-manifest.json");
    let signoff = temp.join("release-signoff.json");
    fs::write(&manifest, release_manifest_json(TARBALL_SHA256)).expect("write manifest");
    fs::write(
        &signoff,
        release_signoff_json_without_signed_target_ci_provenance(TARBALL_SHA256),
    )
    .expect("write signoff");

    let output = Command::new("node")
        .arg("scripts/verify-npm-release-signoff-summary.mjs")
        .arg(&manifest)
        .arg(&signoff)
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .output()
        .expect("run release signoff summary verifier");

    let _ = fs::remove_dir_all(&temp);

    assert!(
        !output.status.success(),
        "missing signed target CI provenance evidence should fail\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        String::from_utf8_lossy(&output.stderr).contains("ciProvider")
            || String::from_utf8_lossy(&output.stderr).contains("runnerOs")
            || String::from_utf8_lossy(&output.stderr).contains("runnerArch"),
        "failure should identify signed target CI provenance evidence\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn release_signoff_summary_verifier_should_reject_wrong_signed_target_github_workflow() {
    if !node_available() {
        return;
    }

    let temp = temp_dir("rust-calckernel-release-signoff-summary-target-workflow");
    fs::create_dir_all(&temp).expect("create temp dir");
    let manifest = temp.join("release-manifest.json");
    let signoff = temp.join("release-signoff.json");
    fs::write(&manifest, release_manifest_json(TARBALL_SHA256)).expect("write manifest");
    fs::write(
        &signoff,
        release_signoff_json(TARBALL_SHA256).replace(GITHUB_WORKFLOW, "unit test workflow"),
    )
    .expect("write signoff");

    let output = Command::new("node")
        .arg("scripts/verify-npm-release-signoff-summary.mjs")
        .arg(&manifest)
        .arg(&signoff)
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .output()
        .expect("run release signoff summary verifier");

    let _ = fs::remove_dir_all(&temp);

    assert!(
        !output.status.success(),
        "wrong signed target GitHub workflow should fail\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        String::from_utf8_lossy(&output.stderr).contains("githubWorkflow")
            && String::from_utf8_lossy(&output.stderr).contains(GITHUB_WORKFLOW),
        "failure should identify the required GitHub workflow\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn release_signoff_summary_verifier_should_reject_wrong_signed_target_github_job() {
    if !node_available() {
        return;
    }

    let temp = temp_dir("rust-calckernel-release-signoff-summary-target-job");
    fs::create_dir_all(&temp).expect("create temp dir");
    let manifest = temp.join("release-manifest.json");
    let signoff = temp.join("release-signoff.json");
    fs::write(&manifest, release_manifest_json(TARBALL_SHA256)).expect("write manifest");
    fs::write(
        &signoff,
        release_signoff_json(TARBALL_SHA256).replace(GITHUB_JOB, "verify-release-scripts"),
    )
    .expect("write signoff");

    let output = Command::new("node")
        .arg("scripts/verify-npm-release-signoff-summary.mjs")
        .arg(&manifest)
        .arg(&signoff)
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .output()
        .expect("run release signoff summary verifier");

    let _ = fs::remove_dir_all(&temp);

    assert!(
        !output.status.success(),
        "wrong signed target GitHub job should fail\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        String::from_utf8_lossy(&output.stderr).contains("githubJob")
            && String::from_utf8_lossy(&output.stderr).contains(GITHUB_JOB),
        "failure should identify the required GitHub job\nstdout:\n{}\nstderr:\n{}",
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

fn release_signoff_json_without_signed_target_platform_arch(tarball_sha256: &str) -> String {
    release_signoff_json_with_signed_target_sha256_source_fallback_runtime_smokes_and_platform_arch(
        tarball_sha256,
        BINARY_SHA256,
        Some("disabled"),
        true,
        false,
        true,
    )
}

fn release_signoff_json_without_signed_target_binary_paths(tarball_sha256: &str) -> String {
    release_signoff_json_with_signed_target_sha256_source_fallback_runtime_smokes_and_platform_arch(
        tarball_sha256,
        BINARY_SHA256,
        Some("disabled"),
        true,
        true,
        false,
    )
}

fn release_signoff_json_without_signed_target_runtime_environment(tarball_sha256: &str) -> String {
    release_signoff_json_with_evidence(
        tarball_sha256,
        BINARY_SHA256,
        Some("disabled"),
        true,
        Some("passed"),
        true,
        true,
        false,
        true,
        true,
        true,
    )
}

fn release_signoff_json_without_signed_target_ci_provenance(tarball_sha256: &str) -> String {
    let mut json = release_signoff_json(tarball_sha256);
    for target in [
        "darwin-arm64",
        "darwin-x64",
        "linux-arm64",
        "linux-x64",
        "win32-arm64",
        "win32-x64",
    ] {
        json = json.replace(&ci_provenance_fields(target), "");
    }
    json
}

fn release_signoff_json_without_backend_runtime_smokes(tarball_sha256: &str) -> String {
    release_signoff_json_with_signed_target_sha256_source_fallback_and_runtime_smokes(
        tarball_sha256,
        BINARY_SHA256,
        Some("disabled"),
        false,
    )
}

fn release_signoff_json_without_type_smoke(tarball_sha256: &str) -> String {
    release_signoff_json_with_signed_target_sha256_source_fallback_runtime_smokes_type_smoke_and_platform_arch(
        tarball_sha256,
        BINARY_SHA256,
        Some("disabled"),
        true,
        None,
        true,
        true,
    )
}

fn release_signoff_json_without_ckc_bin_override(tarball_sha256: &str) -> String {
    release_signoff_json_with_evidence(
        tarball_sha256,
        BINARY_SHA256,
        Some("disabled"),
        true,
        Some("passed"),
        true,
        true,
        true,
        false,
        true,
        true,
    )
}

fn release_signoff_json_without_commands(tarball_sha256: &str) -> String {
    release_signoff_json_with_evidence(
        tarball_sha256,
        BINARY_SHA256,
        Some("disabled"),
        true,
        Some("passed"),
        true,
        true,
        true,
        true,
        false,
        true,
    )
}

fn release_signoff_json_without_api_symbols(tarball_sha256: &str) -> String {
    release_signoff_json_with_evidence(
        tarball_sha256,
        BINARY_SHA256,
        Some("disabled"),
        true,
        Some("passed"),
        true,
        true,
        true,
        true,
        true,
        false,
    )
}

fn release_signoff_json_without_source_fallback(tarball_sha256: &str) -> String {
    release_signoff_json_with_signed_target_sha256_source_fallback_and_runtime_smokes(
        tarball_sha256,
        BINARY_SHA256,
        None,
        true,
    )
}

fn release_signoff_json_with_signed_target_sha256(
    tarball_sha256: &str,
    signed_target_sha256: &str,
) -> String {
    release_signoff_json_with_signed_target_sha256_source_fallback_and_runtime_smokes(
        tarball_sha256,
        signed_target_sha256,
        Some("disabled"),
        true,
    )
}

fn release_signoff_json_with_signed_target_sha256_source_fallback_and_runtime_smokes(
    tarball_sha256: &str,
    signed_target_sha256: &str,
    source_fallback: Option<&str>,
    include_runtime_smokes: bool,
) -> String {
    release_signoff_json_with_signed_target_sha256_source_fallback_runtime_smokes_and_platform_arch(
        tarball_sha256,
        signed_target_sha256,
        source_fallback,
        include_runtime_smokes,
        true,
        true,
    )
}

fn release_signoff_json_with_signed_target_sha256_source_fallback_runtime_smokes_and_platform_arch(
    tarball_sha256: &str,
    signed_target_sha256: &str,
    source_fallback: Option<&str>,
    include_runtime_smokes: bool,
    include_platform_arch: bool,
    include_binary_paths: bool,
) -> String {
    release_signoff_json_with_signed_target_sha256_source_fallback_runtime_smokes_platform_arch_and_binary_paths(
        tarball_sha256,
        signed_target_sha256,
        source_fallback,
        include_runtime_smokes,
        include_platform_arch,
        include_binary_paths,
    )
}

fn release_signoff_json_with_signed_target_sha256_source_fallback_runtime_smokes_platform_arch_and_binary_paths(
    tarball_sha256: &str,
    signed_target_sha256: &str,
    source_fallback: Option<&str>,
    include_runtime_smokes: bool,
    include_platform_arch: bool,
    include_binary_paths: bool,
) -> String {
    release_signoff_json_with_signed_target_sha256_source_fallback_runtime_smokes_type_smoke_and_platform_arch(
        tarball_sha256,
        signed_target_sha256,
        source_fallback,
        include_runtime_smokes,
        Some("passed"),
        include_platform_arch,
        include_binary_paths,
    )
}

fn release_signoff_json_with_signed_target_sha256_source_fallback_runtime_smokes_type_smoke_and_platform_arch(
    tarball_sha256: &str,
    signed_target_sha256: &str,
    source_fallback: Option<&str>,
    include_runtime_smokes: bool,
    type_smoke: Option<&str>,
    include_platform_arch: bool,
    include_binary_paths: bool,
) -> String {
    release_signoff_json_with_evidence(
        tarball_sha256,
        signed_target_sha256,
        source_fallback,
        include_runtime_smokes,
        type_smoke,
        include_platform_arch,
        include_binary_paths,
        true,
        true,
        true,
        true,
    )
}

// Test fixture builder keeps each release evidence toggle explicit at call sites.
#[expect(clippy::too_many_arguments)]
fn release_signoff_json_with_evidence(
    tarball_sha256: &str,
    signed_target_sha256: &str,
    source_fallback: Option<&str>,
    include_runtime_smokes: bool,
    type_smoke: Option<&str>,
    include_platform_arch: bool,
    include_binary_paths: bool,
    include_runtime_environment: bool,
    include_ckc_bin_override: bool,
    include_commands: bool,
    include_api_symbols: bool,
) -> String {
    let source_fallback = source_fallback
        .map(|value| {
            format!(
                r#",
  "sourceFallback": "{value}""#
            )
        })
        .unwrap_or_default();
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
    let signed_targets = signed_targets_json(
        signed_target_sha256,
        include_platform_arch,
        include_binary_paths,
        include_runtime_environment,
    );
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
  ]{source_fallback}{ckc_bin_override}{commands}{api_symbols}{type_smoke}{runtime_smokes}
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
        let ci_provenance = ci_provenance_fields(name);
        format!(r#"    {{"name": "{name}"{platform_arch}, "sha256": "{sha256}"{runtime_environment}{ci_provenance}{binary_paths}}}"#)
    })
    .collect::<Vec<_>>()
    .join(",\n")
}

fn ci_provenance_fields(target: &str) -> String {
    let (runner_os, runner_arch) = runner_os_arch_for_target(target);
    format!(
        r#", "ciProvider": "{CI_PROVIDER}", "githubRunId": "{GITHUB_RUN_ID}", "githubRunAttempt": "{GITHUB_RUN_ATTEMPT}", "githubSha": "{GITHUB_SHA}", "githubWorkflow": "{GITHUB_WORKFLOW}", "githubJob": "{GITHUB_JOB}", "runnerOs": "{runner_os}", "runnerArch": "{runner_arch}""#
    )
}

fn runner_os_arch_for_target(target: &str) -> (&'static str, &'static str) {
    let (platform, arch) = target
        .split_once('-')
        .expect("target includes platform and arch");
    let runner_os = match platform {
        "darwin" => "macOS",
        "linux" => "Linux",
        "win32" => "Windows",
        _ => panic!("unsupported target platform: {platform}"),
    };
    let runner_arch = match arch {
        "arm64" => "ARM64",
        "x64" => "X64",
        _ => panic!("unsupported target arch: {arch}"),
    };
    (runner_os, runner_arch)
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
