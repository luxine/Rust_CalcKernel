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
            "npm run verify:cutover-evidence -- release-manifest/release-manifest.json release/release-signoff.json npm-publish-artifact.json npm-publish-result.json > npm-cutover-evidence.json"
        ),
        "publish job must verify the final cutover evidence bundle"
    );
    assert!(
        workflow.contains("npm-cutover-evidence.json"),
        "publish job must upload final cutover evidence output"
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

fn node_available() -> bool {
    Command::new("node")
        .arg("--version")
        .output()
        .is_ok_and(|output| output.status.success())
}
