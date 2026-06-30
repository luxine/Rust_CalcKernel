use std::fs;
use std::process::Command;

#[test]
fn rust_replacement_readiness_audit_should_not_require_typescript_checkout_edits() {
    if !node_available() {
        return;
    }

    let output = Command::new("node")
        .arg("scripts/audit-rust-replacement-readiness.mjs")
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .output()
        .expect("run Rust replacement readiness audit");

    assert!(
        output.status.success(),
        "Rust replacement readiness audit failed\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn rust_replacement_readiness_audit_should_require_final_publish_evidence_verifiers() {
    let audit =
        fs::read_to_string("scripts/audit-rust-replacement-readiness.mjs").expect("read audit");

    for expected in [
        r#"packageJson.scripts?.["verify:publish-result"]"#,
        r#"packageJson.scripts?.["verify:cutover-evidence"]"#,
        "scripts/verify-npm-publish-result.mjs",
        "scripts/verify-npm-cutover-evidence.mjs",
        r#"expectIncludes(npmRelease, "verify:publish-result", "npm release docs")"#,
        r#"expectIncludes(npmRelease, "verify:cutover-evidence", "npm release docs")"#,
        r#"expectIncludes(npmRelease, "npm-cutover-evidence.json", "npm release docs")"#,
        r#"expectIncludes(npmRelease, "release-signoff-summary.json", "npm release docs")"#,
        r#"expectIncludes(npmRelease, "registry replacement status", "npm release docs")"#,
        r#"expectIncludes(npmRelease, "registry tarball URL", "npm release docs")"#,
        r#"expectIncludes(npmRelease, "sha512 npm integrity", "npm release docs")"#,
        r#"expectIncludes(npmRelease, "sha1 shasum", "npm release docs")"#,
        r#"expectIncludes(npmRelease, "consumer install lifecycle scripts", "npm release docs")"#,
        r#"expectIncludes(npmRelease, "scriptNames", "npm release docs")"#,
        r#"expectIncludes(npmRelease, "packageManager", "npm release docs")"#,
        r#"expectIncludes(npmRelease, "publishArtifactTarballPath", "npm release docs")"#,
        r#"expectIncludes(npmRelease, "publishId", "npm release docs")"#,
        r#"expectIncludes(npmRelease, "publishFilename", "npm release docs")"#,
        r#"expectIncludes(npmRelease, "publishShasum", "npm release docs")"#,
        r#"expectIncludes(npmRelease, "publishIntegrity", "npm release docs")"#,
        r#"expectIncludes(npmRelease, "publishProvenance", "npm release docs")"#,
        r#"expectIncludes(npmRelease, "publish-npm", "npm release docs")"#,
        "README.zh-CN.md",
    ] {
        assert!(
            audit.contains(expected),
            "readiness audit must require {expected}"
        );
    }
}

#[test]
fn rust_replacement_readiness_audit_should_require_exact_package_script_surface() {
    let audit =
        fs::read_to_string("scripts/audit-rust-replacement-readiness.mjs").expect("read audit");

    for expected in [
        "const EXPECTED_PACKAGE_SCRIPT_NAMES = Object.freeze",
        r#"expectNoPackageManager(packageJson, "Rust package")"#,
        r#"expectPackageScriptNames(packageJson, EXPECTED_PACKAGE_SCRIPT_NAMES, "Rust package")"#,
        "function expectNoPackageManager(packageJson, label)",
        "function expectPackageScriptNames(packageJson, expected, label)",
    ] {
        assert!(
            audit.contains(expected),
            "readiness audit must require {expected}"
        );
    }
}

#[test]
fn rust_replacement_readiness_audit_should_require_self_contained_cutover_artifact_docs() {
    let audit =
        fs::read_to_string("scripts/audit-rust-replacement-readiness.mjs").expect("read audit");

    for expected in [
        r#"expectIncludes(npmRelease, "release-manifest/release-manifest.json", "npm release docs")"#,
        r#"expectIncludes(npmRelease, "release/release-signoff.json", "npm release docs")"#,
        r#"expectIncludes(npmRelease, "npm-publish.json", "npm release docs")"#,
        r#"expectIncludes(npmRelease, "npm-registry-replacement.json", "npm release docs")"#,
        r#"expectIncludes(npmRelease, "release-signoff-summary.json", "npm release docs")"#,
        r#"expectIncludes(npmRelease, "npm-publish-artifact.json", "npm release docs")"#,
        r#"expectIncludes(npmRelease, "npm-publish-result.json", "npm release docs")"#,
        r#"expectIncludes(npmRelease, "npm-cutover-evidence.json", "npm release docs")"#,
    ] {
        assert!(
            audit.contains(expected),
            "readiness audit must require {expected}"
        );
    }
}

#[test]
fn rust_replacement_readiness_audit_should_require_final_manifest_evidence_surface() {
    let audit =
        fs::read_to_string("scripts/audit-rust-replacement-readiness.mjs").expect("read audit");

    for expected in [
        r#"expectIncludes(npmRelease, "fileSurface", "npm release docs")"#,
        r#"expectIncludes(npmRelease, "required/forbidden/allowed file lists", "npm release docs")"#,
        r#"expectIncludes(npmRelease, "target Rust triples", "npm release docs")"#,
        r#"expectIncludes(npmRelease, "binary paths", "npm release docs")"#,
        r#"expectIncludes(npmRelease, "file modes", "npm release docs")"#,
        r#"expectIncludes(npmRelease, "binary formats", "npm release docs")"#,
        r#"expectIncludes(npmRelease, "architectures", "npm release docs")"#,
        r#"expectIncludes(npmRelease, "sizes", "npm release docs")"#,
        r#"expectIncludes(npmRelease, "SHA256 values", "npm release docs")"#,
    ] {
        assert!(
            audit.contains(expected),
            "readiness audit must require {expected}"
        );
    }
}

#[test]
fn rust_replacement_readiness_audit_should_require_public_api_parity_verifier() {
    let audit =
        fs::read_to_string("scripts/audit-rust-replacement-readiness.mjs").expect("read audit");

    for expected in [
        r#"packageJson.scripts?.["verify:public-api-parity"]"#,
        "scripts/verify-public-api-parity.mjs",
        r#"expectIncludes(npmRelease, "verify:public-api-parity", "npm release docs")"#,
        r#"expectIncludes(npmRelease, "runtime export kind", "npm release docs")"#,
    ] {
        assert!(
            audit.contains(expected),
            "readiness audit must require {expected}"
        );
    }
}

#[test]
fn rust_replacement_readiness_audit_should_require_declaration_parity_verifier() {
    let audit =
        fs::read_to_string("scripts/audit-rust-replacement-readiness.mjs").expect("read audit");

    for expected in [
        r#"packageJson.scripts?.["verify:declaration-parity"]"#,
        "scripts/verify-declaration-parity.mjs",
        r#"expectIncludes(npmRelease, "verify:declaration-parity", "npm release docs")"#,
        r#"expectIncludes(npmRelease, "declaration export kind", "npm release docs")"#,
        r#"expectIncludes(npmRelease, "function signature compatibility", "npm release docs")"#,
        r#"expectIncludes(npmRelease, "class member surface", "npm release docs")"#,
        r#"expectIncludes(npmRelease, "interface member surface", "npm release docs")"#,
        r#"expectIncludes(npmRelease, "enum member surface", "npm release docs")"#,
        r#"expectIncludes(npmRelease, "type alias surface", "npm release docs")"#,
    ] {
        assert!(
            audit.contains(expected),
            "readiness audit must require {expected}"
        );
    }
}

#[test]
fn rust_replacement_readiness_audit_should_require_host_signoff_type_smoke_compiler_setup() {
    let audit =
        fs::read_to_string("scripts/audit-rust-replacement-readiness.mjs").expect("read audit");

    for expected in [
        r#"expectIncludes(npmRelease, "TypeScript declaration smoke", "npm release docs")"#,
        r#"expectIncludes(npmRelease, "typeSmoke", "npm release docs")"#,
        r#"expectIncludes(npmRelease, "typescript@^5.8.0", "npm release docs")"#,
        r#"expectIncludes(npmRelease, "packagedBinarySha256", "npm release docs")"#,
        r#"expectIncludes(npmRelease, "installedBin", "npm release docs")"#,
        r#"expectIncludes(npmRelease, "packagedBinary", "npm release docs")"#,
        r#"expectIncludes(npmRelease, "signed target binary SHA256", "npm release docs")"#,
        r#"expectIncludes(npmRelease, "platform / arch", "npm release docs")"#,
        r#"expectIncludes(npmRelease, "nodeVersion", "npm release docs")"#,
        r#"expectIncludes(npmRelease, "npmVersion", "npm release docs")"#,
        r#"expectIncludes(npmRelease, "ciProvider", "npm release docs")"#,
        r#"expectIncludes(npmRelease, "githubRunId", "npm release docs")"#,
        r#"expectIncludes(npmRelease, "githubRunAttempt", "npm release docs")"#,
        r#"expectIncludes(npmRelease, "githubSha", "npm release docs")"#,
        r#"expectIncludes(npmRelease, "githubWorkflow", "npm release docs")"#,
        r#"expectIncludes(npmRelease, "githubJob", "npm release docs")"#,
        r#"expectIncludes(npmRelease, "npm release artifact", "npm release docs")"#,
        r#"expectIncludes(npmRelease, "platform-signoff", "npm release docs")"#,
        r#"expectIncludes(npmRelease, "runnerOs", "npm release docs")"#,
        r#"expectIncludes(npmRelease, "runnerArch", "npm release docs")"#,
        r#"expectIncludes(npmRelease, "ckcBinOverride", "npm release docs")"#,
        r#"expectIncludes(npmRelease, "commands", "npm release docs")"#,
        r#"expectIncludes(npmRelease, "apiSymbols", "npm release docs")"#,
        r#"expectIncludes(npmRelease, "sourceFallback", "npm release docs")"#,
        r#"expectIncludes(npmRelease, "backend runtime smoke", "npm release docs")"#,
        r#"expectIncludes(npmRelease, "backendRuntimeSmokes", "npm release docs")"#,
        r#"expectIncludes(npmRelease, "node smoke-c-runtime.mjs", "npm release docs")"#,
        r#"expectIncludes(npmRelease, "node smoke-wasm-runtime.mjs", "npm release docs")"#,
        r#"expectIncludes(npmRelease, "node smoke-llvm-object-runtime.mjs", "npm release docs")"#,
    ] {
        assert!(
            audit.contains(expected),
            "readiness audit must require {expected}"
        );
    }
}

#[test]
fn rust_replacement_readiness_audit_should_require_typescript_test_surface_audit() {
    let audit =
        fs::read_to_string("scripts/audit-rust-replacement-readiness.mjs").expect("read audit");

    for expected in [
        r#"packageJson.scripts?.["audit:typescript-test-surface"]"#,
        "scripts/audit-typescript-test-surface.mjs",
        "docs/typescript-test-surface.json",
    ] {
        assert!(
            audit.contains(expected),
            "readiness audit must require {expected}"
        );
    }
}

#[test]
fn rust_replacement_readiness_audit_should_require_tests_fixture_backend_coverage_docs() {
    let audit =
        fs::read_to_string("scripts/audit-rust-replacement-readiness.mjs").expect("read audit");

    for expected in [
        r#"expectIncludes(architectureReview, "tests/fixtures", "architecture review")"#,
        r#"expectIncludes(zhArchitectureReview, "tests/fixtures", "Chinese architecture review")"#,
        r#"expectIncludes(architectureReview, "f64 edge fixture C dynamic-library runtime parity", "architecture review")"#,
        r#"expectIncludes(zhArchitectureReview, "f64 edge fixture C dynamic-library runtime parity", "Chinese architecture review")"#,
        r#"expectIncludes(architectureReview, "f64-array C dynamic-library runtime parity", "architecture review")"#,
        r#"expectIncludes(zhArchitectureReview, "f64-array C dynamic-library runtime parity", "Chinese architecture review")"#,
        r#"expectIncludes(architectureReview, "f64-axpy/f64-sum/pricing-SoA C dynamic-library runtime parity", "architecture review")"#,
        r#"expectIncludes(zhArchitectureReview, "f64-axpy/f64-sum/pricing-SoA C dynamic-library runtime parity", "Chinese architecture review")"#,
        r#"expectIncludes(architectureReview, "WASM scalar/calls/control-flow/memory/short-circuit C dynamic-library runtime parity", "architecture review")"#,
        r#"expectIncludes(zhArchitectureReview, "WASM scalar/calls/control-flow/memory/short-circuit C dynamic-library runtime parity", "Chinese architecture review")"#,
        r#"expectIncludes(architectureReview, "LLVM scalar/calls/control-flow/memory/short-circuit/bool C dynamic-library runtime parity", "architecture review")"#,
        r#"expectIncludes(zhArchitectureReview, "LLVM scalar/calls/control-flow/memory/short-circuit/bool C dynamic-library runtime parity", "Chinese architecture review")"#,
        r#"expectIncludes(architectureReview, "dijkstra C dynamic-library runtime parity", "architecture review")"#,
        r#"expectIncludes(zhArchitectureReview, "dijkstra C dynamic-library runtime parity", "Chinese architecture review")"#,
        r#"expectIncludes(architectureReview, "f64 edge fixture WASM runtime parity", "architecture review")"#,
        r#"expectIncludes(zhArchitectureReview, "f64 edge fixture WASM runtime parity", "Chinese architecture review")"#,
        r#"expectIncludes(architectureReview, "dijkstra WASM runtime parity", "architecture review")"#,
        r#"expectIncludes(zhArchitectureReview, "dijkstra WASM runtime parity", "Chinese architecture review")"#,
        r#"expectIncludes(architectureReview, "dijkstra LLVM object/dynamic runtime parity", "architecture review")"#,
        r#"expectIncludes(zhArchitectureReview, "dijkstra LLVM object/dynamic runtime parity", "Chinese architecture review")"#,
        r#"expectIncludes(architectureReview, "f64-array LLVM object/dynamic runtime parity", "architecture review")"#,
        r#"expectIncludes(zhArchitectureReview, "f64-array LLVM object/dynamic runtime parity", "Chinese architecture review")"#,
        r#"expectIncludes(architectureReview, "f64-axpy/f64-sum/pricing-SoA LLVM object/dynamic runtime parity", "architecture review")"#,
        r#"expectIncludes(zhArchitectureReview, "f64-axpy/f64-sum/pricing-SoA LLVM object/dynamic runtime parity", "Chinese architecture review")"#,
    ] {
        assert!(
            audit.contains(expected),
            "readiness audit must require {expected}"
        );
    }
}

fn node_available() -> bool {
    Command::new("node")
        .arg("--version")
        .output()
        .is_ok_and(|output| output.status.success())
}
