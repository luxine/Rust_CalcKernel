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

fn workflow_section<'a>(workflow: &'a str, start: &str, end: &str) -> &'a str {
    let start_index = workflow
        .find(start)
        .unwrap_or_else(|| panic!("workflow should include {start}"));
    let end_index = workflow[start_index..]
        .find(end)
        .map(|offset| start_index + offset)
        .unwrap_or_else(|| panic!("workflow should include {end} after {start}"));
    &workflow[start_index..end_index]
}

fn node_available() -> bool {
    Command::new("node")
        .arg("--version")
        .output()
        .is_ok_and(|output| output.status.success())
}
