use std::fs;

#[test]
fn rust_oracle_tests_should_not_hardcode_local_typescript_fixture_paths() {
    for path in [
        "tests/c_backend_test.rs",
        "tests/cli_test.rs",
        "tests/llvm_backend_test.rs",
        "tests/mir_test.rs",
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
