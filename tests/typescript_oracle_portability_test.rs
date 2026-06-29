use std::fs;

#[test]
fn rust_oracle_tests_should_not_hardcode_local_typescript_fixture_paths() {
    for path in [
        "tests/c_backend_test.rs",
        "tests/cli_test.rs",
        "tests/llvm_backend_test.rs",
        "tests/mir_test.rs",
        "tests/package_smoke_test.rs",
        "tests/wasm_backend_test.rs",
    ] {
        let text = fs::read_to_string(path).unwrap_or_else(|error| panic!("read {path}: {error}"));

        assert!(
            !text.contains("PathBuf::from(\"/Users/lynn/code/CalcKernel\").join"),
            "{path} must use CALCKERNEL_TS_ROOT-aware fixture paths instead of joining the local oracle path directly"
        );
        assert!(
            !text.contains("PathBuf::from(\"/Users/lynn/code/CalcKernel/"),
            "{path} must use CALCKERNEL_TS_ROOT-aware fixture paths instead of embedding local absolute fixture paths"
        );
        assert!(
            !text.contains("const tsIndexPath = \"/Users/lynn/code/CalcKernel/"),
            "{path} must use CALCKERNEL_TS_ROOT-aware package oracle paths instead of embedding local absolute fixture paths"
        );
    }
}

#[test]
fn npm_release_workflow_should_prepare_typescript_oracle_for_ci_parity() {
    let workflow =
        fs::read_to_string(".github/workflows/npm-release.yml").expect("read npm release workflow");

    for expected in [
        "typescript_oracle_repository:",
        "default: \"luxine/CalcKernel\"",
        "typescript_oracle_ref:",
        "repository: ${{ inputs.typescript_oracle_repository }}",
        "ref: ${{ inputs.typescript_oracle_ref }}",
        "path: typescript-oracle",
        "CALCKERNEL_TS_ROOT: ${{ github.workspace }}/typescript-oracle",
        "corepack enable",
        "pnpm install --frozen-lockfile",
        "pnpm build",
        "npm run verify:typescript-oracle",
    ] {
        assert!(
            workflow.contains(expected),
            "release workflow must include {expected}"
        );
    }

    let checkout_index = workflow
        .find("path: typescript-oracle")
        .expect("workflow should checkout the TypeScript oracle");
    let verify_index = workflow
        .find("npm run verify:typescript-oracle")
        .expect("workflow should verify the TypeScript oracle");
    let parity_index = workflow
        .find("node scripts/verify-declaration-parity.mjs")
        .expect("workflow should run declaration parity");

    assert!(
        checkout_index < verify_index && verify_index < parity_index,
        "workflow must checkout and verify the TypeScript oracle before oracle-dependent parity checks"
    );
}
