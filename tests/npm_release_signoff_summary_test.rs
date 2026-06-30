use std::{
    fs,
    path::PathBuf,
    process::Command,
    time::{SystemTime, UNIX_EPOCH},
};

const TARBALL_SHA256: &str = "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef";
const BINARY_SHA256: &str = "1111111111111111111111111111111111111111111111111111111111111111";

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
        String::from_utf8_lossy(&output.stdout).contains("\"sourceFallback\": \"disabled\""),
        "release signoff summary verifier should report disabled source fallback evidence\nstdout:\n{}\nstderr:\n{}",
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
    )
}

fn release_signoff_json_without_backend_runtime_smokes(tarball_sha256: &str) -> String {
    release_signoff_json_with_signed_target_sha256_source_fallback_and_runtime_smokes(
        tarball_sha256,
        BINARY_SHA256,
        Some("disabled"),
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
    )
}

fn release_signoff_json_with_signed_target_sha256_source_fallback_runtime_smokes_and_platform_arch(
    tarball_sha256: &str,
    signed_target_sha256: &str,
    source_fallback: Option<&str>,
    include_runtime_smokes: bool,
    include_platform_arch: bool,
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
    let signed_targets = signed_targets_json(signed_target_sha256, include_platform_arch);
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
  ]{source_fallback}{runtime_smokes}
}}"#
    )
}

fn signed_targets_json(linux_x64_sha256: &str, include_platform_arch: bool) -> String {
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
        if include_platform_arch {
            format!(
                r#"    {{"name": "{name}", "platform": "{platform}", "arch": "{arch}", "sha256": "{sha256}"}}"#
            )
        } else {
            format!(r#"    {{"name": "{name}", "sha256": "{sha256}"}}"#)
        }
    })
    .collect::<Vec<_>>()
    .join(",\n")
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
