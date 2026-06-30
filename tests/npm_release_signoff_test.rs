use std::{
    fs,
    path::PathBuf,
    process::Command,
    time::{SystemTime, UNIX_EPOCH},
};

const TARGETS: [&str; 6] = [
    "darwin-arm64",
    "darwin-x64",
    "linux-arm64",
    "linux-x64",
    "win32-arm64",
    "win32-x64",
];

const TARBALL_SHA256: &str = "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef";
const BINARY_SHA256: &str = "1111111111111111111111111111111111111111111111111111111111111111";
const NODE_VERSION: &str = "v20.10.0";
const NPM_VERSION: &str = "10.2.0";
const CI_PROVIDER: &str = "github-actions";
const GITHUB_RUN_ID: &str = "1234567890";
const GITHUB_RUN_ATTEMPT: &str = "2";
const GITHUB_SHA: &str = "abcdef0123456789abcdef0123456789abcdef01";
const GITHUB_REPOSITORY: &str = "luxine/Rust_CalcKernel";
const GITHUB_WORKFLOW: &str = "npm release artifact";
const GITHUB_JOB: &str = "platform-signoff";
const REQUIRED_COMMANDS: [&str; 8] = [
    "ckc --help",
    "ckc check smoke.ck",
    "ckc emit-mir smoke.ck -o build/smoke.mir",
    "ckc emit-c smoke.ck -o build/smoke.c",
    "ckc emit-wat smoke.ck -o build/smoke.wat",
    "ckc emit-wasm smoke.ck -o build/smoke.wasm",
    "ckc emit-llvm smoke.ck -o build/smoke.ll",
    "ckc build-llvm smoke.ck --kind object -o build/smoke.o",
];
const REQUIRED_RUNTIME_COMMANDS: [&str; 4] = [
    "ckc build smoke.ck -o build/smoke-c",
    "node smoke-c-runtime.mjs",
    "node smoke-wasm-runtime.mjs",
    "node smoke-llvm-object-runtime.mjs",
];

#[test]
fn release_signoff_verifier_should_reject_missing_target_smoke() {
    if !node_available() {
        return;
    }

    let temp = temp_dir("rust-calckernel-release-signoff-missing");
    let manifest = temp.join("release-manifest.json");
    let signoffs = temp.join("signoffs");
    fs::create_dir_all(&signoffs).expect("create signoff dir");
    fs::write(&manifest, release_manifest_json()).expect("write release manifest");
    for target in TARGETS.iter().filter(|target| **target != "win32-x64") {
        fs::write(
            signoffs.join(format!("{target}.json")),
            signoff_json(target),
        )
        .expect("write signoff");
    }

    let output = Command::new("node")
        .arg("scripts/verify-npm-release-signoff.mjs")
        .arg(&manifest)
        .arg(&signoffs)
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .output()
        .expect("run release signoff verifier");

    let _ = fs::remove_dir_all(&temp);

    assert!(
        !output.status.success(),
        "missing target signoff should fail\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        String::from_utf8_lossy(&output.stderr).contains("missing platform sign-off for win32-x64"),
        "missing target failure should identify win32-x64\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn release_signoff_verifier_should_reject_non_canonical_manifest_targets() {
    if !node_available() {
        return;
    }

    let temp = temp_dir("rust-calckernel-release-signoff-non-canonical-manifest");
    let manifest = temp.join("release-manifest.json");
    let signoffs = temp.join("signoffs");
    fs::create_dir_all(&signoffs).expect("create signoff dir");
    fs::write(&manifest, release_manifest_json_with_extra_target())
        .expect("write release manifest");
    for target in TARGETS {
        fs::write(
            signoffs.join(format!("{target}.json")),
            signoff_json(target),
        )
        .expect("write signoff");
    }

    let output = Command::new("node")
        .arg("scripts/verify-npm-release-signoff.mjs")
        .arg(&manifest)
        .arg(&signoffs)
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .output()
        .expect("run release signoff verifier");

    let _ = fs::remove_dir_all(&temp);

    assert!(
        !output.status.success(),
        "release manifest with extra targets should fail\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        String::from_utf8_lossy(&output.stderr).contains("release manifest targets"),
        "failure should identify non-canonical release manifest targets\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn release_signoff_verifier_should_reject_missing_build_llvm_smoke() {
    if !node_available() {
        return;
    }

    let temp = temp_dir("rust-calckernel-release-signoff-missing-build-llvm");
    let manifest = temp.join("release-manifest.json");
    let signoffs = temp.join("signoffs");
    fs::create_dir_all(&signoffs).expect("create signoff dir");
    fs::write(&manifest, release_manifest_json()).expect("write release manifest");
    let commands_without_build_llvm = REQUIRED_COMMANDS
        .iter()
        .chain(REQUIRED_RUNTIME_COMMANDS.iter())
        .copied()
        .filter(|command| !command.starts_with("ckc build-llvm "))
        .collect::<Vec<_>>();
    for target in TARGETS {
        fs::write(
            signoffs.join(format!("{target}.json")),
            signoff_json_with_commands(target, &commands_without_build_llvm),
        )
        .expect("write signoff");
    }

    let output = Command::new("node")
        .arg("scripts/verify-npm-release-signoff.mjs")
        .arg(&manifest)
        .arg(&signoffs)
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .output()
        .expect("run release signoff verifier");

    let _ = fs::remove_dir_all(&temp);

    assert!(
        !output.status.success(),
        "missing build-llvm signoff should fail\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        String::from_utf8_lossy(&output.stderr)
            .contains("ckc build-llvm smoke.ck --kind object -o build/smoke.o"),
        "missing build-llvm failure should identify the required command\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn release_signoff_verifier_should_reject_missing_backend_runtime_smoke() {
    if !node_available() {
        return;
    }

    let temp = temp_dir("rust-calckernel-release-signoff-missing-runtime-smoke");
    let manifest = temp.join("release-manifest.json");
    let signoffs = temp.join("signoffs");
    fs::create_dir_all(&signoffs).expect("create signoff dir");
    fs::write(&manifest, release_manifest_json()).expect("write release manifest");
    let commands_without_runtime = REQUIRED_COMMANDS
        .iter()
        .chain(REQUIRED_RUNTIME_COMMANDS.iter())
        .copied()
        .filter(|command| *command != "node smoke-wasm-runtime.mjs")
        .collect::<Vec<_>>();
    for target in TARGETS {
        fs::write(
            signoffs.join(format!("{target}.json")),
            signoff_json_with_commands(target, &commands_without_runtime),
        )
        .expect("write signoff");
    }

    let output = Command::new("node")
        .arg("scripts/verify-npm-release-signoff.mjs")
        .arg(&manifest)
        .arg(&signoffs)
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .output()
        .expect("run release signoff verifier");

    let _ = fs::remove_dir_all(&temp);

    assert!(
        !output.status.success(),
        "missing backend runtime smoke should fail\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        String::from_utf8_lossy(&output.stderr).contains("node smoke-wasm-runtime.mjs"),
        "missing runtime smoke failure should identify the required command\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn release_signoff_verifier_should_reject_missing_packaged_binary_evidence() {
    if !node_available() {
        return;
    }

    let temp = temp_dir("rust-calckernel-release-signoff-missing-package-binary");
    let manifest = temp.join("release-manifest.json");
    let signoffs = temp.join("signoffs");
    fs::create_dir_all(&signoffs).expect("create signoff dir");
    fs::write(&manifest, release_manifest_json()).expect("write release manifest");
    for target in TARGETS {
        fs::write(
            signoffs.join(format!("{target}.json")),
            signoff_json_without_binary_evidence(target),
        )
        .expect("write signoff");
    }

    let output = Command::new("node")
        .arg("scripts/verify-npm-release-signoff.mjs")
        .arg(&manifest)
        .arg(&signoffs)
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .output()
        .expect("run release signoff verifier");

    let _ = fs::remove_dir_all(&temp);

    assert!(
        !output.status.success(),
        "missing packaged binary evidence should fail\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        String::from_utf8_lossy(&output.stderr).contains("packagedBinary"),
        "missing packaged binary evidence failure should identify packagedBinary\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn release_signoff_verifier_should_reject_missing_package_version_evidence() {
    if !node_available() {
        return;
    }

    let temp = temp_dir("rust-calckernel-release-signoff-missing-package-version");
    let manifest = temp.join("release-manifest.json");
    let signoffs = temp.join("signoffs");
    fs::create_dir_all(&signoffs).expect("create signoff dir");
    fs::write(&manifest, release_manifest_json()).expect("write release manifest");
    for target in TARGETS {
        fs::write(
            signoffs.join(format!("{target}.json")),
            signoff_json_without_package_version(target),
        )
        .expect("write signoff");
    }

    let output = Command::new("node")
        .arg("scripts/verify-npm-release-signoff.mjs")
        .arg(&manifest)
        .arg(&signoffs)
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .output()
        .expect("run release signoff verifier");

    let _ = fs::remove_dir_all(&temp);

    assert!(
        !output.status.success(),
        "missing packageVersion evidence should fail\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        String::from_utf8_lossy(&output.stderr).contains("packageVersion"),
        "missing packageVersion failure should identify packageVersion\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn release_signoff_verifier_should_reject_missing_platform_arch_evidence() {
    if !node_available() {
        return;
    }

    let temp = temp_dir("rust-calckernel-release-signoff-missing-platform-arch");
    let manifest = temp.join("release-manifest.json");
    let signoffs = temp.join("signoffs");
    fs::create_dir_all(&signoffs).expect("create signoff dir");
    fs::write(&manifest, release_manifest_json()).expect("write release manifest");
    for target in TARGETS {
        fs::write(
            signoffs.join(format!("{target}.json")),
            signoff_json_without_platform_arch(target),
        )
        .expect("write signoff");
    }

    let output = Command::new("node")
        .arg("scripts/verify-npm-release-signoff.mjs")
        .arg(&manifest)
        .arg(&signoffs)
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .output()
        .expect("run release signoff verifier");

    let _ = fs::remove_dir_all(&temp);

    assert!(
        !output.status.success(),
        "missing platform/arch evidence should fail\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        String::from_utf8_lossy(&output.stderr).contains("platform")
            || String::from_utf8_lossy(&output.stderr).contains("arch"),
        "missing platform/arch failure should identify target platform metadata\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn release_signoff_verifier_should_reject_missing_runtime_environment_evidence() {
    if !node_available() {
        return;
    }

    let temp = temp_dir("rust-calckernel-release-signoff-runtime-environment");
    let manifest = temp.join("release-manifest.json");
    let signoffs = temp.join("signoffs");
    fs::create_dir_all(&signoffs).expect("create signoff dir");
    fs::write(&manifest, release_manifest_json()).expect("write release manifest");
    for target in TARGETS {
        fs::write(
            signoffs.join(format!("{target}.json")),
            signoff_json_without_runtime_environment(target),
        )
        .expect("write signoff");
    }

    let output = Command::new("node")
        .arg("scripts/verify-npm-release-signoff.mjs")
        .arg(&manifest)
        .arg(&signoffs)
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .output()
        .expect("run release signoff verifier");

    let _ = fs::remove_dir_all(&temp);

    assert!(
        !output.status.success(),
        "missing runtime environment evidence should fail\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        String::from_utf8_lossy(&output.stderr).contains("nodeVersion")
            || String::from_utf8_lossy(&output.stderr).contains("npmVersion"),
        "missing runtime environment failure should identify Node/npm version evidence\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn release_signoff_verifier_should_reject_missing_ci_provenance_evidence() {
    if !node_available() {
        return;
    }

    let temp = temp_dir("rust-calckernel-release-signoff-ci-provenance");
    let manifest = temp.join("release-manifest.json");
    let signoffs = temp.join("signoffs");
    fs::create_dir_all(&signoffs).expect("create signoff dir");
    fs::write(&manifest, release_manifest_json()).expect("write release manifest");
    for target in TARGETS {
        fs::write(
            signoffs.join(format!("{target}.json")),
            signoff_json_without_ci_provenance(target),
        )
        .expect("write signoff");
    }

    let output = Command::new("node")
        .arg("scripts/verify-npm-release-signoff.mjs")
        .arg(&manifest)
        .arg(&signoffs)
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .output()
        .expect("run release signoff verifier");

    let _ = fs::remove_dir_all(&temp);

    assert!(
        !output.status.success(),
        "missing CI provenance evidence should fail\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        String::from_utf8_lossy(&output.stderr).contains("ciProvider")
            || String::from_utf8_lossy(&output.stderr).contains("githubRunId")
            || String::from_utf8_lossy(&output.stderr).contains("runnerOs"),
        "missing CI provenance failure should identify CI/runner evidence\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn release_signoff_verifier_should_reject_non_github_actions_signoff() {
    if !node_available() {
        return;
    }

    let temp = temp_dir("rust-calckernel-release-signoff-local-ci-provider");
    let manifest = temp.join("release-manifest.json");
    let signoffs = temp.join("signoffs");
    fs::create_dir_all(&signoffs).expect("create signoff dir");
    fs::write(&manifest, release_manifest_json()).expect("write release manifest");
    for target in TARGETS {
        fs::write(
            signoffs.join(format!("{target}.json")),
            signoff_json_with_ci_provider(target, "local"),
        )
        .expect("write signoff");
    }

    let output = Command::new("node")
        .arg("scripts/verify-npm-release-signoff.mjs")
        .arg(&manifest)
        .arg(&signoffs)
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .output()
        .expect("run release signoff verifier");

    let _ = fs::remove_dir_all(&temp);

    assert!(
        !output.status.success(),
        "local signoff provenance should fail\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        String::from_utf8_lossy(&output.stderr).contains("ciProvider"),
        "local provenance failure should identify ciProvider\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn release_signoff_verifier_should_reject_wrong_github_workflow() {
    if !node_available() {
        return;
    }

    let temp = temp_dir("rust-calckernel-release-signoff-wrong-workflow");
    let manifest = temp.join("release-manifest.json");
    let signoffs = temp.join("signoffs");
    fs::create_dir_all(&signoffs).expect("create signoff dir");
    fs::write(&manifest, release_manifest_json()).expect("write release manifest");
    for target in TARGETS {
        fs::write(
            signoffs.join(format!("{target}.json")),
            signoff_json_with_github_workflow(target, "unit test workflow"),
        )
        .expect("write signoff");
    }

    let output = Command::new("node")
        .arg("scripts/verify-npm-release-signoff.mjs")
        .arg(&manifest)
        .arg(&signoffs)
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .output()
        .expect("run release signoff verifier");

    let _ = fs::remove_dir_all(&temp);

    assert!(
        !output.status.success(),
        "wrong GitHub workflow provenance should fail\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        String::from_utf8_lossy(&output.stderr).contains("githubWorkflow")
            && String::from_utf8_lossy(&output.stderr).contains(GITHUB_WORKFLOW),
        "wrong workflow failure should identify the required workflow\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn release_signoff_verifier_should_reject_wrong_github_job() {
    if !node_available() {
        return;
    }

    let temp = temp_dir("rust-calckernel-release-signoff-wrong-job");
    let manifest = temp.join("release-manifest.json");
    let signoffs = temp.join("signoffs");
    fs::create_dir_all(&signoffs).expect("create signoff dir");
    fs::write(&manifest, release_manifest_json()).expect("write release manifest");
    for target in TARGETS {
        fs::write(
            signoffs.join(format!("{target}.json")),
            signoff_json_with_github_job(target, "verify-release-scripts"),
        )
        .expect("write signoff");
    }

    let output = Command::new("node")
        .arg("scripts/verify-npm-release-signoff.mjs")
        .arg(&manifest)
        .arg(&signoffs)
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .output()
        .expect("run release signoff verifier");

    let _ = fs::remove_dir_all(&temp);

    assert!(
        !output.status.success(),
        "wrong GitHub job provenance should fail\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        String::from_utf8_lossy(&output.stderr).contains("githubJob")
            && String::from_utf8_lossy(&output.stderr).contains(GITHUB_JOB),
        "wrong job failure should identify the required job\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn release_signoff_verifier_should_reject_runner_platform_mismatch() {
    if !node_available() {
        return;
    }

    let temp = temp_dir("rust-calckernel-release-signoff-runner-mismatch");
    let manifest = temp.join("release-manifest.json");
    let signoffs = temp.join("signoffs");
    fs::create_dir_all(&signoffs).expect("create signoff dir");
    fs::write(&manifest, release_manifest_json()).expect("write release manifest");
    for target in TARGETS {
        let signoff = if target == "darwin-arm64" {
            signoff_json_with_runner(target, "Linux", "ARM64")
        } else {
            signoff_json(target)
        };
        fs::write(signoffs.join(format!("{target}.json")), signoff).expect("write signoff");
    }

    let output = Command::new("node")
        .arg("scripts/verify-npm-release-signoff.mjs")
        .arg(&manifest)
        .arg(&signoffs)
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .output()
        .expect("run release signoff verifier");

    let _ = fs::remove_dir_all(&temp);

    assert!(
        !output.status.success(),
        "runner platform mismatch should fail\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        String::from_utf8_lossy(&output.stderr).contains("runnerOs")
            || String::from_utf8_lossy(&output.stderr).contains("runnerArch"),
        "runner mismatch failure should identify runner OS/arch evidence\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn release_signoff_verifier_should_reject_signoff_repository_mismatch() {
    if !node_available() {
        return;
    }

    let temp = temp_dir("rust-calckernel-release-signoff-source-repository");
    let manifest = temp.join("release-manifest.json");
    let signoffs = temp.join("signoffs");
    fs::create_dir_all(&signoffs).expect("create signoff dir");
    fs::write(&manifest, release_manifest_json()).expect("write release manifest");
    for target in TARGETS {
        let mut json = signoff_json(target);
        if target == "linux-x64" {
            json = json.replace(
                &format!("\"githubRepository\": \"{GITHUB_REPOSITORY}\""),
                "\"githubRepository\": \"luxine/OtherCalcKernel\"",
            );
        }
        fs::write(signoffs.join(format!("{target}.json")), json).expect("write signoff");
    }

    let output = Command::new("node")
        .arg("scripts/verify-npm-release-signoff.mjs")
        .arg(&manifest)
        .arg(&signoffs)
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .output()
        .expect("run release signoff verifier with mismatched repository");

    let _ = fs::remove_dir_all(&temp);

    assert!(
        !output.status.success(),
        "signoff repository mismatch should fail\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        String::from_utf8_lossy(&output.stderr).contains("sourceRepository"),
        "failure should identify source repository mismatch\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn release_signoff_verifier_should_reject_signoff_sha_that_differs_from_manifest_source_sha() {
    if !node_available() {
        return;
    }

    let temp = temp_dir("rust-calckernel-release-signoff-source-sha-mismatch");
    let manifest = temp.join("release-manifest.json");
    let signoffs = temp.join("signoffs");
    fs::create_dir_all(&signoffs).expect("create signoff dir");
    fs::write(
        &manifest,
        release_manifest_json().replace(
            &format!("\"sourceGitSha\":\"{GITHUB_SHA}\""),
            "\"sourceGitSha\":\"1111111111111111111111111111111111111111\"",
        ),
    )
    .expect("write release manifest");
    for target in TARGETS {
        fs::write(
            signoffs.join(format!("{target}.json")),
            signoff_json(target),
        )
        .expect("write signoff");
    }

    let output = Command::new("node")
        .arg("scripts/verify-npm-release-signoff.mjs")
        .arg(&manifest)
        .arg(&signoffs)
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .output()
        .expect("run release signoff verifier");

    let _ = fs::remove_dir_all(&temp);

    assert!(
        !output.status.success(),
        "source SHA mismatch should fail\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        String::from_utf8_lossy(&output.stderr).contains("sourceGitSha"),
        "source SHA mismatch should identify manifest source SHA\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn release_signoff_verifier_should_reject_source_checkout_fallback_smokes() {
    if !node_available() {
        return;
    }

    let temp = temp_dir("rust-calckernel-release-signoff-source-fallback");
    let manifest = temp.join("release-manifest.json");
    let signoffs = temp.join("signoffs");
    fs::create_dir_all(&signoffs).expect("create signoff dir");
    fs::write(&manifest, release_manifest_json()).expect("write release manifest");
    for target in TARGETS {
        fs::write(
            signoffs.join(format!("{target}.json")),
            signoff_json_with_source_fallback(target, "enabled"),
        )
        .expect("write signoff");
    }

    let output = Command::new("node")
        .arg("scripts/verify-npm-release-signoff.mjs")
        .arg(&manifest)
        .arg(&signoffs)
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .output()
        .expect("run release signoff verifier");

    let _ = fs::remove_dir_all(&temp);

    assert!(
        !output.status.success(),
        "source checkout fallback signoff should fail\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        String::from_utf8_lossy(&output.stderr).contains("source fallback must be disabled"),
        "source checkout fallback failure should identify disabled source fallback requirement\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn release_signoff_verifier_should_reject_packaged_binary_sha256_mismatch() {
    if !node_available() {
        return;
    }

    let temp = temp_dir("rust-calckernel-release-signoff-binary-sha-mismatch");
    let manifest = temp.join("release-manifest.json");
    let signoffs = temp.join("signoffs");
    fs::create_dir_all(&signoffs).expect("create signoff dir");
    fs::write(&manifest, release_manifest_json()).expect("write release manifest");
    for target in TARGETS {
        let signoff = if target == "linux-x64" {
            signoff_json_with_packaged_binary_sha256(
                target,
                "abcdef0123456789abcdef0123456789abcdef0123456789abcdef0123456789",
            )
        } else {
            signoff_json(target)
        };
        fs::write(signoffs.join(format!("{target}.json")), signoff).expect("write signoff");
    }

    let output = Command::new("node")
        .arg("scripts/verify-npm-release-signoff.mjs")
        .arg(&manifest)
        .arg(&signoffs)
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .output()
        .expect("run release signoff verifier");

    let _ = fs::remove_dir_all(&temp);

    assert!(
        !output.status.success(),
        "mismatched packaged binary SHA256 should fail\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        String::from_utf8_lossy(&output.stderr).contains("packagedBinarySha256"),
        "mismatch failure should identify packagedBinarySha256\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn release_signoff_verifier_should_accept_complete_target_smokes() {
    if !node_available() {
        return;
    }

    let temp = temp_dir("rust-calckernel-release-signoff-complete");
    let manifest = temp.join("release-manifest.json");
    let signoffs = temp.join("signoffs");
    fs::create_dir_all(&signoffs).expect("create signoff dir");
    fs::write(&manifest, release_manifest_json()).expect("write release manifest");
    for target in TARGETS {
        fs::write(
            signoffs.join(format!("{target}.json")),
            signoff_json(target),
        )
        .expect("write signoff");
    }

    let output = Command::new("node")
        .arg("scripts/verify-npm-release-signoff.mjs")
        .arg(&manifest)
        .arg(&signoffs)
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .output()
        .expect("run release signoff verifier");

    let _ = fs::remove_dir_all(&temp);

    assert!(
        output.status.success(),
        "complete target signoffs should pass\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        String::from_utf8_lossy(&output.stdout).contains("\"targetCount\": 6"),
        "complete target signoff should report all six targets\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        String::from_utf8_lossy(&output.stdout).contains("\"signedTargets\": ["),
        "complete target signoff should report signed target binary hashes\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        String::from_utf8_lossy(&output.stdout)
            .contains(&format!("\"sha256\": \"{BINARY_SHA256}\"")),
        "complete target signoff should report each packaged binary SHA256\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        String::from_utf8_lossy(&output.stdout).contains("\"platform\": \"linux\"")
            && String::from_utf8_lossy(&output.stdout).contains("\"arch\": \"x64\""),
        "complete target signoff should preserve target platform and architecture evidence\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        String::from_utf8_lossy(&output.stdout)
            .contains(&format!("\"nodeVersion\": \"{NODE_VERSION}\""))
            && String::from_utf8_lossy(&output.stdout)
                .contains(&format!("\"npmVersion\": \"{NPM_VERSION}\"")),
        "complete target signoff should preserve Node/npm runtime environment evidence per target\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        String::from_utf8_lossy(&output.stdout)
            .contains(&format!("\"ciProvider\": \"{CI_PROVIDER}\""))
            && String::from_utf8_lossy(&output.stdout)
                .contains(&format!("\"githubRunId\": \"{GITHUB_RUN_ID}\""))
            && String::from_utf8_lossy(&output.stdout)
                .contains(&format!("\"githubRepository\": \"{GITHUB_REPOSITORY}\""))
            && String::from_utf8_lossy(&output.stdout).contains("\"runnerOs\": \"Linux\"")
            && String::from_utf8_lossy(&output.stdout).contains("\"runnerArch\": \"X64\""),
        "complete target signoff should preserve CI run and runner provenance per target\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        String::from_utf8_lossy(&output.stdout).contains("\"installedBin\":")
            && String::from_utf8_lossy(&output.stdout).contains("node_modules/.bin/ckc"),
        "complete target signoff should preserve installed CLI path evidence per target\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        String::from_utf8_lossy(&output.stdout).contains("\"packagedBinary\":")
            && String::from_utf8_lossy(&output.stdout)
                .contains("node_modules/calckernel/npm/bin/ckc-linux-x64"),
        "complete target signoff should preserve packaged binary path evidence per target\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        String::from_utf8_lossy(&output.stdout)
            .contains(&format!("\"packagedBinarySha256\": \"{BINARY_SHA256}\"")),
        "complete target signoff should preserve packaged binary SHA256 evidence per target\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        String::from_utf8_lossy(&output.stdout)
            .contains(&format!("\"sourceGitSha\": \"{GITHUB_SHA}\"")),
        "complete target signoff should preserve manifest source checkout SHA evidence\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        String::from_utf8_lossy(&output.stdout).contains("\"sourceFallback\": \"disabled\""),
        "complete target signoff should report disabled source fallback evidence\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        String::from_utf8_lossy(&output.stdout).contains("\"ckcBinOverride\": \"unset\""),
        "complete target signoff should preserve CKC_BIN unset evidence\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        String::from_utf8_lossy(&output.stdout).contains("\"commands\": [")
            && String::from_utf8_lossy(&output.stdout)
                .contains("\"ckc emit-llvm smoke.ck -o build/smoke.ll\""),
        "complete target signoff should preserve CLI smoke command evidence\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        String::from_utf8_lossy(&output.stdout).contains("\"apiSymbols\": [")
            && String::from_utf8_lossy(&output.stdout).contains("\"emitCSource\""),
        "complete target signoff should preserve package root API smoke evidence\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        String::from_utf8_lossy(&output.stdout).contains("\"typeSmoke\": \"passed\""),
        "complete target signoff should preserve aggregate TypeScript declaration smoke evidence\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        String::from_utf8_lossy(&output.stdout).contains("\"backendRuntimeSmokes\": ["),
        "complete target signoff should report backend runtime smoke evidence\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        String::from_utf8_lossy(&output.stdout).contains("\"node smoke-wasm-runtime.mjs\""),
        "complete target signoff should report WASM runtime smoke evidence\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

fn release_manifest_json() -> String {
    let targets = TARGETS
        .iter()
        .map(|target| format!("{{\"name\":\"{target}\",\"sha256\":\"{BINARY_SHA256}\"}}"))
        .collect::<Vec<_>>()
        .join(",");
    format!(
        "{{\"packageName\":\"calckernel\",\"packageVersion\":\"0.8.0\",\"tarball\":\"calckernel-0.8.0.tgz\",\"tarballSha256\":\"{TARBALL_SHA256}\",\"sourceGitSha\":\"{GITHUB_SHA}\",\"sourceRepository\":\"{GITHUB_REPOSITORY}\",\"targets\":[{targets}]}}"
    )
}

fn release_manifest_json_with_extra_target() -> String {
    let mut targets = TARGETS
        .iter()
        .map(|target| format!("{{\"name\":\"{target}\",\"sha256\":\"{BINARY_SHA256}\"}}"))
        .collect::<Vec<_>>();
    targets.push(format!(
        "{{\"name\":\"freebsd-x64\",\"sha256\":\"{BINARY_SHA256}\"}}"
    ));
    format!(
        "{{\"packageName\":\"calckernel\",\"packageVersion\":\"0.8.0\",\"tarball\":\"calckernel-0.8.0.tgz\",\"tarballSha256\":\"{TARBALL_SHA256}\",\"sourceGitSha\":\"{GITHUB_SHA}\",\"sourceRepository\":\"{GITHUB_REPOSITORY}\",\"targets\":[{}]}}",
        targets.join(",")
    )
}

fn signoff_json(target: &str) -> String {
    let commands = REQUIRED_COMMANDS
        .iter()
        .chain(REQUIRED_RUNTIME_COMMANDS.iter())
        .copied()
        .collect::<Vec<_>>();
    signoff_json_with_commands(target, &commands)
}

fn signoff_json_without_platform_arch(target: &str) -> String {
    let installed_bin = installed_bin_evidence(target);
    let packaged_binary = packaged_binary_evidence(target);
    let packaged_binary_sha = packaged_binary_sha256_evidence(BINARY_SHA256);
    let source_fallback = source_fallback_evidence("disabled");
    signoff_json_with_commands_and_binary_evidence(
        target,
        &REQUIRED_COMMANDS
            .iter()
            .chain(REQUIRED_RUNTIME_COMMANDS.iter())
            .copied()
            .collect::<Vec<_>>(),
        &format!("{installed_bin}{packaged_binary}{packaged_binary_sha}{source_fallback}"),
    )
}

fn signoff_json_without_package_version(target: &str) -> String {
    let platform_arch = platform_arch_evidence(target);
    let installed_bin = installed_bin_evidence(target);
    let packaged_binary = packaged_binary_evidence(target);
    let packaged_binary_sha = packaged_binary_sha256_evidence(BINARY_SHA256);
    let source_fallback = source_fallback_evidence("disabled");
    signoff_json_with_commands_and_binary_evidence_without_package_version(
        target,
        &REQUIRED_COMMANDS,
        &format!(
            "{platform_arch}{installed_bin}{packaged_binary}{packaged_binary_sha}{source_fallback}"
        ),
    )
}

fn signoff_json_without_runtime_environment(target: &str) -> String {
    let platform_arch = platform_arch_evidence(target);
    let installed_bin = installed_bin_evidence(target);
    let packaged_binary = packaged_binary_evidence(target);
    let packaged_binary_sha = packaged_binary_sha256_evidence(BINARY_SHA256);
    let source_fallback = source_fallback_evidence("disabled");
    signoff_json_with_commands_and_binary_evidence_inner(
        target,
        &REQUIRED_COMMANDS
            .iter()
            .chain(REQUIRED_RUNTIME_COMMANDS.iter())
            .copied()
            .collect::<Vec<_>>(),
        &format!(
            "{platform_arch}{installed_bin}{packaged_binary}{packaged_binary_sha}{source_fallback}"
        ),
        true,
        false,
        true,
    )
}

fn signoff_json_without_ci_provenance(target: &str) -> String {
    let platform_arch = platform_arch_evidence(target);
    let installed_bin = installed_bin_evidence(target);
    let packaged_binary = packaged_binary_evidence(target);
    let packaged_binary_sha = packaged_binary_sha256_evidence(BINARY_SHA256);
    let source_fallback = source_fallback_evidence("disabled");
    signoff_json_with_commands_and_binary_evidence_inner(
        target,
        &REQUIRED_COMMANDS
            .iter()
            .chain(REQUIRED_RUNTIME_COMMANDS.iter())
            .copied()
            .collect::<Vec<_>>(),
        &format!(
            "{platform_arch}{installed_bin}{packaged_binary}{packaged_binary_sha}{source_fallback}"
        ),
        true,
        true,
        false,
    )
}

fn signoff_json_with_ci_provider(target: &str, ci_provider: &str) -> String {
    signoff_json_with_ci_provenance(target, ci_provider, None, None)
}

fn signoff_json_with_runner(target: &str, runner_os: &str, runner_arch: &str) -> String {
    signoff_json_with_ci_provenance(target, CI_PROVIDER, Some(runner_os), Some(runner_arch))
}

fn signoff_json_with_github_workflow(target: &str, github_workflow: &str) -> String {
    signoff_json(target).replace(
        &format!("\"githubWorkflow\": \"{GITHUB_WORKFLOW}\""),
        &format!("\"githubWorkflow\": \"{github_workflow}\""),
    )
}

fn signoff_json_with_github_job(target: &str, github_job: &str) -> String {
    signoff_json(target).replace(
        &format!("\"githubJob\": \"{GITHUB_JOB}\""),
        &format!("\"githubJob\": \"{github_job}\""),
    )
}

fn signoff_json_with_ci_provenance(
    target: &str,
    ci_provider: &str,
    runner_os: Option<&str>,
    runner_arch: Option<&str>,
) -> String {
    let (default_runner_os, default_runner_arch) = runner_os_arch_for_target(target);
    signoff_json(target)
        .replace(
            &format!("\"ciProvider\": \"{CI_PROVIDER}\""),
            &format!("\"ciProvider\": \"{ci_provider}\""),
        )
        .replace(
            &format!("\"runnerOs\": \"{default_runner_os}\""),
            &format!(
                "\"runnerOs\": \"{}\"",
                runner_os.unwrap_or(default_runner_os)
            ),
        )
        .replace(
            &format!("\"runnerArch\": \"{default_runner_arch}\""),
            &format!(
                "\"runnerArch\": \"{}\"",
                runner_arch.unwrap_or(default_runner_arch)
            ),
        )
}

fn signoff_json_with_commands(target: &str, commands: &[&str]) -> String {
    let platform_arch = platform_arch_evidence(target);
    let installed_bin = installed_bin_evidence(target);
    let packaged_binary = packaged_binary_evidence(target);
    let packaged_binary_sha = packaged_binary_sha256_evidence(BINARY_SHA256);
    let source_fallback = source_fallback_evidence("disabled");
    signoff_json_with_commands_and_binary_evidence(
        target,
        commands,
        &format!(
            "{platform_arch}{installed_bin}{packaged_binary}{packaged_binary_sha}{source_fallback}"
        ),
    )
}

fn signoff_json_with_packaged_binary_sha256(target: &str, packaged_binary_sha256: &str) -> String {
    let platform_arch = platform_arch_evidence(target);
    let installed_bin = installed_bin_evidence(target);
    let packaged_binary = packaged_binary_evidence(target);
    let packaged_binary_sha = packaged_binary_sha256_evidence(packaged_binary_sha256);
    let source_fallback = source_fallback_evidence("disabled");
    signoff_json_with_commands_and_binary_evidence(
        target,
        &REQUIRED_COMMANDS,
        &format!(
            "{platform_arch}{installed_bin}{packaged_binary}{packaged_binary_sha}{source_fallback}"
        ),
    )
}

fn signoff_json_without_binary_evidence(target: &str) -> String {
    let platform_arch = platform_arch_evidence(target);
    let source_fallback = source_fallback_evidence("disabled");
    signoff_json_with_commands_and_binary_evidence(
        target,
        &REQUIRED_COMMANDS,
        &format!(
            "{}{}{}",
            platform_arch,
            installed_bin_evidence(target),
            source_fallback
        ),
    )
}

fn signoff_json_with_source_fallback(target: &str, source_fallback: &str) -> String {
    let platform_arch = platform_arch_evidence(target);
    let installed_bin = installed_bin_evidence(target);
    let packaged_binary = packaged_binary_evidence(target);
    let packaged_binary_sha = packaged_binary_sha256_evidence(BINARY_SHA256);
    let source_fallback = source_fallback_evidence(source_fallback);
    signoff_json_with_commands_and_binary_evidence(
        target,
        &REQUIRED_COMMANDS,
        &format!(
            "{platform_arch}{installed_bin}{packaged_binary}{packaged_binary_sha}{source_fallback}"
        ),
    )
}

fn platform_arch_evidence(target: &str) -> String {
    let (platform, arch) = platform_arch_for_target(target);
    format!(
        r#",
  "platform": "{platform}",
  "arch": "{arch}""#
    )
}

fn platform_arch_for_target(target: &str) -> (&'static str, &'static str) {
    match target {
        "darwin-arm64" => ("darwin", "arm64"),
        "darwin-x64" => ("darwin", "x64"),
        "linux-arm64" => ("linux", "arm64"),
        "linux-x64" => ("linux", "x64"),
        "win32-arm64" => ("win32", "arm64"),
        "win32-x64" => ("win32", "x64"),
        _ => panic!("unknown target {target}"),
    }
}

fn source_fallback_evidence(source_fallback: &str) -> String {
    format!(
        r#",
  "sourceFallback": "{source_fallback}""#
    )
}

fn signoff_json_with_commands_and_binary_evidence(
    target: &str,
    commands: &[&str],
    binary_evidence: &str,
) -> String {
    signoff_json_with_commands_and_binary_evidence_inner(
        target,
        commands,
        binary_evidence,
        true,
        true,
        true,
    )
}

fn signoff_json_with_commands_and_binary_evidence_without_package_version(
    target: &str,
    commands: &[&str],
    binary_evidence: &str,
) -> String {
    signoff_json_with_commands_and_binary_evidence_inner(
        target,
        commands,
        binary_evidence,
        false,
        true,
        true,
    )
}

fn signoff_json_with_commands_and_binary_evidence_inner(
    target: &str,
    commands: &[&str],
    binary_evidence: &str,
    include_package_version: bool,
    include_runtime_environment: bool,
    include_ci_provenance: bool,
) -> String {
    let commands_json = commands
        .iter()
        .map(|command| format!("    {command:?}"))
        .collect::<Vec<_>>()
        .join(",\n");
    let package_version = if include_package_version {
        r#",
  "packageVersion": "0.8.0""#
    } else {
        ""
    };
    let runtime_environment = if include_runtime_environment {
        runtime_environment_evidence()
    } else {
        String::new()
    };
    let ci_provenance = if include_ci_provenance {
        ci_provenance_evidence(target)
    } else {
        String::new()
    };
    format!(
        r#"{{
  "package": "calckernel",
  "targetName": "{target}"{package_version}{runtime_environment}{ci_provenance},
  "tarball": "calckernel-0.8.0.tgz",
  "tarballSha256": "{TARBALL_SHA256}",
  "commands": [
{commands_json}
  ],
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
  ],
  "typeSmoke": "passed",
  "ckcBinOverride": "unset"{binary_evidence}
}}"#
    )
}

fn runtime_environment_evidence() -> String {
    format!(
        r#",
  "nodeVersion": "{NODE_VERSION}",
  "npmVersion": "{NPM_VERSION}""#
    )
}

fn ci_provenance_evidence(target: &str) -> String {
    ci_provenance_evidence_with(target, CI_PROVIDER, None, None)
}

fn ci_provenance_evidence_with(
    target: &str,
    ci_provider: &str,
    runner_os: Option<&str>,
    runner_arch: Option<&str>,
) -> String {
    let (expected_runner_os, expected_runner_arch) = runner_os_arch_for_target(target);
    let runner_os = runner_os.unwrap_or(expected_runner_os);
    let runner_arch = runner_arch.unwrap_or(expected_runner_arch);
    format!(
        r#",
  "ciProvider": "{ci_provider}",
  "githubRunId": "{GITHUB_RUN_ID}",
  "githubRunAttempt": "{GITHUB_RUN_ATTEMPT}",
  "githubSha": "{GITHUB_SHA}",
  "githubRepository": "{GITHUB_REPOSITORY}",
  "githubWorkflow": "{GITHUB_WORKFLOW}",
  "githubJob": "{GITHUB_JOB}",
  "runnerOs": "{runner_os}",
  "runnerArch": "{runner_arch}""#
    )
}

fn runner_os_arch_for_target(target: &str) -> (&'static str, &'static str) {
    match target {
        "darwin-arm64" => ("macOS", "ARM64"),
        "darwin-x64" => ("macOS", "X64"),
        "linux-arm64" => ("Linux", "ARM64"),
        "linux-x64" => ("Linux", "X64"),
        "win32-arm64" => ("Windows", "ARM64"),
        "win32-x64" => ("Windows", "X64"),
        _ => panic!("unknown target {target}"),
    }
}

fn installed_bin_evidence(target: &str) -> String {
    if target.starts_with("win32-") {
        r#",
  "installedBin": "C:\\consumer\\node_modules\\.bin\\ckc.cmd""#
            .to_string()
    } else {
        r#",
  "installedBin": "/tmp/consumer/node_modules/.bin/ckc""#
            .to_string()
    }
}

fn packaged_binary_evidence(target: &str) -> String {
    let binary_file = match target {
        "win32-arm64" => "ckc-win32-arm64.exe",
        "win32-x64" => "ckc-win32-x64.exe",
        _ => target,
    };
    let binary_file = if target.starts_with("win32-") {
        binary_file.to_string()
    } else {
        format!("ckc-{binary_file}")
    };

    if target.starts_with("win32-") {
        format!(
            r#",
  "packagedBinary": "C:\\consumer\\node_modules\\calckernel\\npm\\bin\\{binary_file}""#
        )
    } else {
        format!(
            r#",
  "packagedBinary": "/tmp/consumer/node_modules/calckernel/npm/bin/{binary_file}""#
        )
    }
}

fn packaged_binary_sha256_evidence(packaged_binary_sha256: &str) -> String {
    format!(
        r#",
  "packagedBinarySha256": "{packaged_binary_sha256}""#
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
