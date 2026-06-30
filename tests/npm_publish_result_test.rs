use std::{
    fs,
    path::PathBuf,
    process::Command,
    time::{SystemTime, UNIX_EPOCH},
};

const VALID_SHASUM: &str = "0123456789abcdef0123456789abcdef01234567";
const VALID_INTEGRITY: &str = "sha512-AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA==";
const GITHUB_RUN_ID: &str = "1234567890";
const GITHUB_RUN_ATTEMPT: &str = "2";
const GITHUB_SHA: &str = "abcdef0123456789abcdef0123456789abcdef01";
const GITHUB_REPOSITORY: &str = "luxine/Rust_CalcKernel";
const GITHUB_WORKFLOW: &str = "npm release artifact";
const GITHUB_JOB: &str = "publish-npm";
const RUNNER_OS: &str = "Linux";
const RUNNER_ARCH: &str = "X64";

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

    let output = node_command()
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
        String::from_utf8_lossy(&output.stdout)
            .contains(&format!("\"sourceGitSha\": \"{GITHUB_SHA}\"")),
        "publish result verifier should report the manifest source checkout SHA\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        String::from_utf8_lossy(&output.stdout)
            .contains(&format!("\"sourceRepository\": \"{GITHUB_REPOSITORY}\"")),
        "publish result verifier should report the manifest source repository\nstdout:\n{}\nstderr:\n{}",
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
        String::from_utf8_lossy(&output.stdout).contains("\"publishId\": \"calckernel@0.8.0\""),
        "publish result verifier should preserve the npm publish package id\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        String::from_utf8_lossy(&output.stdout)
            .contains("\"publishFilename\": \"calckernel-0.8.0.tgz\""),
        "publish result verifier should preserve the npm publish tarball filename\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        String::from_utf8_lossy(&output.stdout)
            .contains(&format!("\"publishShasum\": \"{VALID_SHASUM}\"")),
        "publish result verifier should preserve the npm publish shasum\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        String::from_utf8_lossy(&output.stdout)
            .contains(&format!("\"publishIntegrity\": \"{VALID_INTEGRITY}\"")),
        "publish result verifier should preserve the npm publish integrity\nstdout:\n{}\nstderr:\n{}",
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
fn publish_result_verifier_should_preserve_github_actions_publish_provenance() {
    if !node_available() {
        return;
    }

    let temp = temp_dir("rust-calckernel-publish-result-github-provenance");
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

    let output = node_command()
        .arg("scripts/verify-npm-publish-result.mjs")
        .arg(&manifest)
        .arg(&publish)
        .arg(&registry)
        .env("GITHUB_ACTIONS", "true")
        .env("GITHUB_RUN_ID", GITHUB_RUN_ID)
        .env("GITHUB_RUN_ATTEMPT", GITHUB_RUN_ATTEMPT)
        .env("GITHUB_SHA", GITHUB_SHA)
        .env("GITHUB_REPOSITORY", GITHUB_REPOSITORY)
        .env("GITHUB_WORKFLOW", GITHUB_WORKFLOW)
        .env("GITHUB_JOB", GITHUB_JOB)
        .env("RUNNER_OS", RUNNER_OS)
        .env("RUNNER_ARCH", RUNNER_ARCH)
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .output()
        .expect("run npm publish result verifier with GitHub Actions provenance");

    let _ = fs::remove_dir_all(&temp);

    assert!(
        output.status.success(),
        "matching publish result with GitHub Actions provenance should pass\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        String::from_utf8_lossy(&output.stdout).contains("\"publishProvenance\": {")
            && String::from_utf8_lossy(&output.stdout)
                .contains("\"ciProvider\": \"github-actions\"")
            && String::from_utf8_lossy(&output.stdout)
                .contains(&format!("\"githubRunId\": \"{GITHUB_RUN_ID}\""))
            && String::from_utf8_lossy(&output.stdout)
                .contains(&format!("\"githubRepository\": \"{GITHUB_REPOSITORY}\""))
            && String::from_utf8_lossy(&output.stdout)
                .contains(&format!("\"githubJob\": \"{GITHUB_JOB}\""))
            && String::from_utf8_lossy(&output.stdout)
                .contains(&format!("\"runnerOs\": \"{RUNNER_OS}\""))
            && String::from_utf8_lossy(&output.stdout)
                .contains(&format!("\"runnerArch\": \"{RUNNER_ARCH}\"")),
        "publish result verifier should preserve npm publish job provenance\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn publish_result_verifier_should_reject_publish_sha_that_differs_from_manifest_source_sha() {
    if !node_available() {
        return;
    }

    let temp = temp_dir("rust-calckernel-publish-result-source-sha-mismatch");
    fs::create_dir_all(&temp).expect("create temp dir");
    let manifest = temp.join("release-manifest.json");
    let publish = temp.join("npm-publish.json");
    let registry = temp.join("npm-registry-replacement.json");
    fs::write(
        &manifest,
        release_manifest_json("calckernel-0.8.0.tgz").replace(
            &format!("\"sourceGitSha\": \"{GITHUB_SHA}\""),
            "\"sourceGitSha\": \"1111111111111111111111111111111111111111\"",
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

    let output = node_command()
        .arg("scripts/verify-npm-publish-result.mjs")
        .arg(&manifest)
        .arg(&publish)
        .arg(&registry)
        .env("GITHUB_ACTIONS", "true")
        .env("GITHUB_RUN_ID", GITHUB_RUN_ID)
        .env("GITHUB_RUN_ATTEMPT", GITHUB_RUN_ATTEMPT)
        .env("GITHUB_SHA", GITHUB_SHA)
        .env("GITHUB_REPOSITORY", GITHUB_REPOSITORY)
        .env("GITHUB_WORKFLOW", GITHUB_WORKFLOW)
        .env("GITHUB_JOB", GITHUB_JOB)
        .env("RUNNER_OS", RUNNER_OS)
        .env("RUNNER_ARCH", RUNNER_ARCH)
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .output()
        .expect("run npm publish result verifier with mismatched source SHA");

    let _ = fs::remove_dir_all(&temp);

    assert!(
        !output.status.success(),
        "publish SHA mismatch should fail\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        String::from_utf8_lossy(&output.stderr).contains("sourceGitSha"),
        "failure should identify manifest source SHA\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn publish_result_verifier_should_reject_source_repository_mismatch() {
    if !node_available() {
        return;
    }

    let temp = temp_dir("rust-calckernel-publish-result-source-repository-mismatch");
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

    let output = node_command()
        .arg("scripts/verify-npm-publish-result.mjs")
        .arg(&manifest)
        .arg(&publish)
        .arg(&registry)
        .env("GITHUB_ACTIONS", "true")
        .env("GITHUB_RUN_ID", GITHUB_RUN_ID)
        .env("GITHUB_RUN_ATTEMPT", GITHUB_RUN_ATTEMPT)
        .env("GITHUB_SHA", GITHUB_SHA)
        .env("GITHUB_REPOSITORY", "luxine/OtherCalcKernel")
        .env("GITHUB_WORKFLOW", GITHUB_WORKFLOW)
        .env("GITHUB_JOB", GITHUB_JOB)
        .env("RUNNER_OS", RUNNER_OS)
        .env("RUNNER_ARCH", RUNNER_ARCH)
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .output()
        .expect("run npm publish result verifier with mismatched source repository");

    let _ = fs::remove_dir_all(&temp);

    assert!(
        !output.status.success(),
        "publish repository mismatch should fail\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        String::from_utf8_lossy(&output.stderr).contains("sourceRepository"),
        "failure should identify manifest source repository\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn publish_result_verifier_should_reject_incomplete_github_actions_publish_provenance() {
    if !node_available() {
        return;
    }

    let temp = temp_dir("rust-calckernel-publish-result-incomplete-github-provenance");
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

    let output = node_command()
        .arg("scripts/verify-npm-publish-result.mjs")
        .arg(&manifest)
        .arg(&publish)
        .arg(&registry)
        .env("GITHUB_ACTIONS", "true")
        .env_remove("GITHUB_RUN_ID")
        .env("GITHUB_RUN_ATTEMPT", GITHUB_RUN_ATTEMPT)
        .env("GITHUB_SHA", GITHUB_SHA)
        .env("GITHUB_REPOSITORY", GITHUB_REPOSITORY)
        .env("GITHUB_WORKFLOW", GITHUB_WORKFLOW)
        .env("GITHUB_JOB", GITHUB_JOB)
        .env("RUNNER_OS", RUNNER_OS)
        .env("RUNNER_ARCH", RUNNER_ARCH)
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .output()
        .expect("run npm publish result verifier with incomplete GitHub provenance");

    let _ = fs::remove_dir_all(&temp);

    assert!(
        !output.status.success(),
        "incomplete GitHub Actions publish provenance should fail\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        String::from_utf8_lossy(&output.stderr).contains("githubRunId"),
        "failure should identify the missing publish GitHub run id\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn publish_result_verifier_should_reject_wrong_publish_workflow_job_in_github_actions() {
    if !node_available() {
        return;
    }

    let temp = temp_dir("rust-calckernel-publish-result-wrong-github-job");
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

    let output = node_command()
        .arg("scripts/verify-npm-publish-result.mjs")
        .arg(&manifest)
        .arg(&publish)
        .arg(&registry)
        .env("GITHUB_ACTIONS", "true")
        .env("GITHUB_RUN_ID", GITHUB_RUN_ID)
        .env("GITHUB_RUN_ATTEMPT", GITHUB_RUN_ATTEMPT)
        .env("GITHUB_SHA", GITHUB_SHA)
        .env("GITHUB_REPOSITORY", GITHUB_REPOSITORY)
        .env("GITHUB_WORKFLOW", "unit test workflow")
        .env("GITHUB_JOB", "verify-release-scripts")
        .env("RUNNER_OS", RUNNER_OS)
        .env("RUNNER_ARCH", RUNNER_ARCH)
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .output()
        .expect("run npm publish result verifier with wrong GitHub workflow/job");

    let _ = fs::remove_dir_all(&temp);

    assert!(
        !output.status.success(),
        "wrong GitHub Actions workflow/job should fail publish result verification\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        String::from_utf8_lossy(&output.stderr).contains("githubWorkflow")
            || String::from_utf8_lossy(&output.stderr).contains("githubJob"),
        "failure should identify the publish workflow/job mismatch\nstdout:\n{}\nstderr:\n{}",
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

    let output = node_command()
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

    let output = node_command()
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

    let output = node_command()
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

    let output = node_command()
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

    let output = node_command()
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

    let output = node_command()
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

    let output = node_command()
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

    let output = node_command()
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
    "consumerInstallScripts": [],
    "packageManager": null,
    "scriptNames": ["audit:release-workflow", "audit:typescript-test-surface", "build", "build:npm-matrix", "ckc", "postpack", "prepack", "test", "verify:cutover-evidence", "verify:declaration-parity", "verify:host-npm-install", "verify:npm-release", "verify:public-api-parity", "verify:publish-artifact", "verify:publish-result", "verify:registry-replacement", "verify:release-signoff", "verify:release-signoff-summary", "verify:typescript-oracle"]
  }},
  "tarball": "{tarball}",
  "tarballSha256": "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef",
  "sourceGitSha": "{GITHUB_SHA}",
  "sourceRepository": "{GITHUB_REPOSITORY}",
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

fn node_command() -> Command {
    let mut command = Command::new("node");
    clear_github_actions_env(&mut command);
    command
}

fn clear_github_actions_env(command: &mut Command) {
    for key in [
        "GITHUB_ACTIONS",
        "GITHUB_RUN_ID",
        "GITHUB_RUN_ATTEMPT",
        "GITHUB_SHA",
        "GITHUB_REPOSITORY",
        "GITHUB_WORKFLOW",
        "GITHUB_JOB",
        "RUNNER_OS",
        "RUNNER_ARCH",
    ] {
        command.env_remove(key);
    }
}
