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

#[test]
fn npm_release_workflow_should_verify_registry_replacement_for_manifest_version() {
    let workflow =
        fs::read_to_string(".github/workflows/npm-release.yml").expect("read npm release workflow");

    assert!(
        workflow.contains("JSON.parse(require('fs').readFileSync('release-manifest/release-manifest.json', 'utf8')).packageVersion"),
        "registry replacement verifier must derive the npm version from release-manifest.json"
    );
    assert!(
        !workflow.contains("npm run verify:registry-replacement -- \"$(node -p \"require('./package.json').version\")\""),
        "registry replacement verifier must not derive the npm version from package.json"
    );
}

#[test]
fn npm_release_workflow_should_run_full_cargo_test_before_release() {
    let workflow =
        fs::read_to_string(".github/workflows/npm-release.yml").expect("read npm release workflow");

    assert!(
        workflow
            .lines()
            .any(|line| line.trim() == "- run: cargo test"),
        "release workflow must run the full Rust test suite before release"
    );
}

#[test]
fn npm_release_workflow_should_test_publish_artifact_verifier_before_publish() {
    let workflow =
        fs::read_to_string(".github/workflows/npm-release.yml").expect("read npm release workflow");

    assert!(
        workflow.contains("--test npm_publish_artifact_test"),
        "release workflow must test publish artifact verifier before publish"
    );
}

#[test]
fn npm_release_workflow_should_test_public_api_parity_before_release() {
    let workflow =
        fs::read_to_string(".github/workflows/npm-release.yml").expect("read npm release workflow");

    assert!(
        workflow.contains("--test npm_public_api_parity_test"),
        "release workflow must test public API parity verifier before release"
    );
    assert!(
        workflow.contains("node scripts/verify-public-api-parity.mjs"),
        "release workflow must run public API parity verifier before release"
    );
}

#[test]
fn npm_release_workflow_should_test_declaration_parity_before_release() {
    let workflow =
        fs::read_to_string(".github/workflows/npm-release.yml").expect("read npm release workflow");

    assert!(
        workflow.contains("--test npm_declaration_parity_test"),
        "release workflow must test declaration parity verifier before release"
    );
    assert!(
        workflow.contains("node scripts/verify-declaration-parity.mjs"),
        "release workflow must run declaration parity verifier before release"
    );
}

#[test]
fn npm_release_workflow_should_audit_typescript_test_surface_before_release() {
    let workflow =
        fs::read_to_string(".github/workflows/npm-release.yml").expect("read npm release workflow");

    assert!(
        workflow.contains("--test typescript_test_surface_audit_test"),
        "release workflow must test TypeScript test surface audit before release"
    );
    assert!(
        workflow.contains("node scripts/audit-typescript-test-surface.mjs"),
        "release workflow must run TypeScript test surface audit before release"
    );
}

#[test]
fn npm_release_workflow_should_verify_publish_result_after_registry_replacement() {
    let workflow =
        fs::read_to_string(".github/workflows/npm-release.yml").expect("read npm release workflow");

    assert!(
        workflow.contains("--test npm_publish_result_test"),
        "release workflow must test npm publish result verifier before publish"
    );
    assert!(
        workflow.contains(
            "npm run verify:publish-result -- release-manifest/release-manifest.json npm-publish.json npm-registry-replacement.json > npm-publish-result.json"
        ),
        "publish job must verify npm publish output against release manifest and registry metadata"
    );
    assert!(
        workflow.contains("npm-publish-result.json"),
        "publish job must upload npm publish result verifier output"
    );
}

#[test]
fn npm_release_workflow_should_verify_final_cutover_evidence_after_publish() {
    let workflow =
        fs::read_to_string(".github/workflows/npm-release.yml").expect("read npm release workflow");

    assert!(
        workflow.contains("--test npm_cutover_evidence_test"),
        "release workflow must test final cutover evidence verifier before publish"
    );
    assert!(
        workflow.contains(
            "npm run verify:cutover-evidence -- release-manifest/release-manifest.json release/release-signoff.json release-signoff-summary.json npm-publish-artifact.json npm-publish-result.json > npm-cutover-evidence.json"
        ),
        "publish job must verify the final cutover evidence bundle including release-signoff-summary.json"
    );
    assert!(
        workflow.contains("npm-cutover-evidence.json"),
        "publish job must upload final cutover evidence output"
    );
}

#[test]
fn npm_release_workflow_should_archive_cutover_source_evidence_after_publish() {
    let workflow =
        fs::read_to_string(".github/workflows/npm-release.yml").expect("read npm release workflow");
    let publish_job = workflow_section(&workflow, "publish-npm:", "");
    let publish_artifact = workflow_section(publish_job, "name: npm-publish", "if-no-files-found");

    assert!(
        publish_artifact.contains("release-manifest/release-manifest.json"),
        "publish job must archive the release manifest with final npm cutover evidence"
    );
    assert!(
        publish_artifact.contains("release/release-signoff.json"),
        "publish job must archive the aggregate release signoff with final npm cutover evidence"
    );
}

#[test]
fn npm_release_workflow_should_verify_signed_tarball_before_publish() {
    let workflow =
        fs::read_to_string(".github/workflows/npm-release.yml").expect("read npm release workflow");

    assert!(
        workflow.contains("name: release-manifest") && workflow.contains("path: release-manifest"),
        "publish job must download the release manifest before npm publish"
    );
    assert!(
        workflow.contains(
            "npm run verify:publish-artifact -- release-manifest/release-manifest.json dist"
        ),
        "publish job must verify the tarball SHA256 against release-manifest.json before npm publish"
    );
}

#[test]
fn npm_release_workflow_should_verify_release_signoff_summary_before_publish() {
    let workflow =
        fs::read_to_string(".github/workflows/npm-release.yml").expect("read npm release workflow");

    let signoff_index = workflow
        .find("npm run verify:release-signoff-summary -- release-manifest/release-manifest.json release/release-signoff.json")
        .expect("publish job should verify release signoff summary before npm publish");
    let publish_index = workflow
        .find("npm publish \"${TARBALL}\" --provenance --access public --json > npm-publish.json")
        .expect("publish job should publish the manifest tarball");

    assert!(
        signoff_index < publish_index,
        "publish job must verify release-signoff.json against release-manifest.json before npm publish"
    );
}

#[test]
fn npm_release_workflow_should_fail_fast_when_npm_token_is_missing() {
    let workflow =
        fs::read_to_string(".github/workflows/npm-release.yml").expect("read npm release workflow");

    let token_check_index = workflow
        .find("test -n \"${NODE_AUTH_TOKEN}\"")
        .expect("publish job should verify NPM_TOKEN before npm publish");
    let publish_index = workflow
        .find("npm publish \"${TARBALL}\" --provenance --access public --json > npm-publish.json")
        .expect("publish job should publish the manifest tarball");

    assert!(
        token_check_index < publish_index,
        "publish job must verify NODE_AUTH_TOKEN is populated before npm publish"
    );
}

#[test]
fn npm_release_workflow_should_verify_staged_binary_matrix_before_pack() {
    let workflow =
        fs::read_to_string(".github/workflows/npm-release.yml").expect("read npm release workflow");

    let verify_index = workflow
        .find("npm run build:npm-matrix -- --verify-staged --expect-complete --out build/npm-binaries")
        .expect("workflow should verify staged npm binaries before pack");
    let pack_index = workflow
        .find("CKC_NPM_BINARIES_DIR=build/npm-binaries npm pack")
        .expect("workflow should pack staged npm binaries");

    assert!(
        verify_index < pack_index,
        "workflow must verify the downloaded binary matrix before npm pack"
    );
}

#[test]
fn npm_release_workflow_should_publish_the_manifest_tarball() {
    let workflow =
        fs::read_to_string(".github/workflows/npm-release.yml").expect("read npm release workflow");

    assert!(
        workflow.contains("JSON.parse(require('fs').readFileSync('release-manifest/release-manifest.json', 'utf8')).tarball"),
        "publish job must derive the npm publish tarball from release-manifest.json"
    );
    assert!(
        !workflow.contains("TARBALL=\"$(ls dist/*.tgz | head -n 1)\"\n          npm publish"),
        "publish job must not choose the published tarball via ls dist/*.tgz"
    );
}

#[test]
fn npm_release_workflow_should_sign_off_the_manifest_tarball() {
    let workflow =
        fs::read_to_string(".github/workflows/npm-release.yml").expect("read npm release workflow");
    let signoff_job = workflow_section(&workflow, "platform-signoff:", "finalize-signoff:");

    assert!(
        signoff_job.contains("name: release-manifest")
            && signoff_job.contains("path: release-manifest"),
        "platform signoff job must download release-manifest.json before choosing the tarball"
    );
    assert!(
        signoff_job.contains("JSON.parse(require('fs').readFileSync('release-manifest/release-manifest.json', 'utf8')).tarball"),
        "platform signoff job must derive the signed-off tarball from release-manifest.json"
    );
    assert!(
        !signoff_job.contains("TARBALL=\"$(ls dist/*.tgz | head -n 1)\""),
        "platform signoff job must not choose the signed-off tarball via ls dist/*.tgz"
    );
}

#[test]
fn npm_release_workflow_audit_should_reject_platform_signoff_runner_mismatch() {
    if !node_available() {
        return;
    }

    let workflow =
        fs::read_to_string(".github/workflows/npm-release.yml").expect("read npm release workflow");
    let tampered = replace_in_workflow_section(
        &workflow,
        "platform-signoff:",
        "finalize-signoff:",
        "runner: ubuntu-24.04-arm",
        "runner: ubuntu-24.04",
    );
    let workflow_path = write_temp_workflow("platform-signoff-runner-mismatch", &tampered);

    let output = Command::new("node")
        .arg("scripts/audit-npm-release-workflow.mjs")
        .env("CKC_NPM_RELEASE_WORKFLOW", &workflow_path)
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .output()
        .expect("run npm release workflow audit against tampered platform signoff workflow");

    let _ = fs::remove_file(&workflow_path);

    assert!(
        !output.status.success(),
        "audit should reject a platform-signoff target bound to the wrong runner\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        String::from_utf8_lossy(&output.stderr).contains("linux-arm64")
            && String::from_utf8_lossy(&output.stderr).contains("ubuntu-24.04-arm"),
        "runner mismatch failure should identify the target and expected runner\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn npm_release_workflow_audit_should_reject_build_binary_runner_mismatch() {
    if !node_available() {
        return;
    }

    let workflow =
        fs::read_to_string(".github/workflows/npm-release.yml").expect("read npm release workflow");
    let tampered = replace_in_workflow_section(
        &workflow,
        "build-binary:",
        "pack-release:",
        "runner: windows-11-arm",
        "runner: windows-2025",
    );
    let workflow_path = write_temp_workflow("build-binary-runner-mismatch", &tampered);

    let output = Command::new("node")
        .arg("scripts/audit-npm-release-workflow.mjs")
        .env("CKC_NPM_RELEASE_WORKFLOW", &workflow_path)
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .output()
        .expect("run npm release workflow audit against tampered build binary workflow");

    let _ = fs::remove_file(&workflow_path);

    assert!(
        !output.status.success(),
        "audit should reject a build-binary target bound to the wrong runner\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        String::from_utf8_lossy(&output.stderr).contains("win32-arm64")
            && String::from_utf8_lossy(&output.stderr).contains("windows-11-arm"),
        "runner mismatch failure should identify the target and expected runner\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn npm_release_workflow_audit_should_reject_publish_without_final_signoff_dependency() {
    if !node_available() {
        return;
    }

    let workflow =
        fs::read_to_string(".github/workflows/npm-release.yml").expect("read npm release workflow");
    let tampered = replace_in_workflow_section(
        &workflow,
        "publish-npm:",
        "",
        "needs: finalize-signoff",
        "needs: pack-release",
    );
    let workflow_path = write_temp_workflow("publish-without-final-signoff", &tampered);

    let output = Command::new("node")
        .arg("scripts/audit-npm-release-workflow.mjs")
        .env("CKC_NPM_RELEASE_WORKFLOW", &workflow_path)
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .output()
        .expect("run npm release workflow audit against tampered publish workflow");

    let _ = fs::remove_file(&workflow_path);

    assert!(
        !output.status.success(),
        "audit should reject publish-npm when it bypasses finalize-signoff\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        String::from_utf8_lossy(&output.stderr).contains("publish-npm")
            && String::from_utf8_lossy(&output.stderr).contains("finalize-signoff"),
        "dependency failure should identify publish-npm and finalize-signoff\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn npm_release_workflow_audit_should_reject_publish_without_npm_token_preflight() {
    if !node_available() {
        return;
    }

    let workflow =
        fs::read_to_string(".github/workflows/npm-release.yml").expect("read npm release workflow");
    let tampered = workflow.replace(
        "      - run: test -n \"${NODE_AUTH_TOKEN}\"\n        env:\n          NODE_AUTH_TOKEN: ${{ secrets.NPM_TOKEN }}\n",
        "",
    );
    let workflow_path = write_temp_workflow("publish-without-npm-token-preflight", &tampered);

    let output = Command::new("node")
        .arg("scripts/audit-npm-release-workflow.mjs")
        .env("CKC_NPM_RELEASE_WORKFLOW", &workflow_path)
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .output()
        .expect("run npm release workflow audit against missing token preflight workflow");

    let _ = fs::remove_file(&workflow_path);

    assert!(
        !output.status.success(),
        "audit should reject publish job without an NPM_TOKEN preflight\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        String::from_utf8_lossy(&output.stderr).contains("NPM_TOKEN preflight"),
        "missing token preflight failure should identify the expected guard\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

fn workflow_section<'a>(workflow: &'a str, start: &str, end: &str) -> &'a str {
    let start_index = workflow
        .find(start)
        .unwrap_or_else(|| panic!("workflow should include {start}"));
    if end.is_empty() {
        return &workflow[start_index..];
    }
    let end_index = workflow[start_index..]
        .find(end)
        .map(|offset| start_index + offset)
        .unwrap_or_else(|| panic!("workflow should include {end} after {start}"));
    &workflow[start_index..end_index]
}

fn replace_in_workflow_section(
    workflow: &str,
    start: &str,
    end: &str,
    from: &str,
    to: &str,
) -> String {
    let start_index = workflow
        .find(start)
        .unwrap_or_else(|| panic!("workflow should include {start}"));
    let end_index = if end.is_empty() {
        workflow.len()
    } else {
        workflow[start_index..]
            .find(end)
            .map(|offset| start_index + offset)
            .unwrap_or_else(|| panic!("workflow should include {end} after {start}"))
    };
    let mut section = workflow[start_index..end_index].to_string();
    assert!(
        section.contains(from),
        "workflow section {start} should include {from}"
    );
    section = section.replacen(from, to, 1);

    let mut tampered = String::new();
    tampered.push_str(&workflow[..start_index]);
    tampered.push_str(&section);
    tampered.push_str(&workflow[end_index..]);
    tampered
}

fn write_temp_workflow(name: &str, contents: &str) -> std::path::PathBuf {
    let path =
        std::env::temp_dir().join(format!("rust-calckernel-{name}-{}.yml", std::process::id()));
    fs::write(&path, contents).expect("write temp workflow");
    path
}

fn node_available() -> bool {
    Command::new("node")
        .arg("--version")
        .output()
        .is_ok_and(|output| output.status.success())
}
