use std::process::Command;

#[test]
fn host_npm_install_verifier_should_pass_without_ckc_bin_override() {
    if !node_available() || !npm_available() {
        return;
    }

    let output = Command::new("node")
        .arg("scripts/verify-host-npm-install.mjs")
        .env_remove("CKC_BIN")
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .output()
        .expect("run host npm install verifier");

    assert!(
        output.status.success(),
        "host npm install verifier failed\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        String::from_utf8_lossy(&output.stdout).contains("\"typeSmoke\": \"passed\""),
        "host npm install verifier should report TypeScript declaration smoke\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        String::from_utf8_lossy(&output.stdout).contains("\"targetName\":"),
        "host npm install verifier should report the npm target name for release sign-off\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        String::from_utf8_lossy(&output.stdout).contains("\"packageVersion\":"),
        "host npm install verifier should report the installed package version for release sign-off\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        String::from_utf8_lossy(&output.stdout).contains("\"tarballSha256\":"),
        "host npm install verifier should report the tarball SHA256 for release sign-off\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        String::from_utf8_lossy(&output.stdout).contains("\"installedBin\":"),
        "host npm install verifier should report the installed node_modules/.bin/ckc path\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        String::from_utf8_lossy(&output.stdout).contains("\"packagedBinary\":"),
        "host npm install verifier should report the packaged Rust binary path used by the wrapper\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        String::from_utf8_lossy(&output.stdout).contains("\"packagedBinarySha256\":"),
        "host npm install verifier should report the packaged Rust binary SHA256 for release sign-off\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        String::from_utf8_lossy(&output.stdout).contains("\"sourceFallback\": \"disabled\""),
        "host npm install verifier should report disabled source fallback for release sign-off\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn host_npm_install_verifier_should_reject_missing_tarball_argument() {
    if !node_available() || !npm_available() {
        return;
    }

    let missing_tarball = std::env::temp_dir().join("rust-calckernel-missing-host-smoke.tgz");
    let _ = std::fs::remove_file(&missing_tarball);

    let output = Command::new("node")
        .arg("scripts/verify-host-npm-install.mjs")
        .arg(&missing_tarball)
        .env_remove("CKC_BIN")
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .output()
        .expect("run host npm install verifier with missing tarball");

    assert!(
        !output.status.success(),
        "missing tarball should fail\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        String::from_utf8_lossy(&output.stderr).contains("does not exist"),
        "missing tarball failure should mention missing file\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn host_npm_install_verifier_should_prepare_typescript_for_ci_without_local_oracle_fallback() {
    let script =
        std::fs::read_to_string("scripts/verify-host-npm-install.mjs").expect("read verifier");

    assert!(
        script.contains("ensureTypeScriptCompiler(consumer, installedEnv)"),
        "host npm install verifier must prepare tsc inside the temporary consumer before type smoke"
    );
    assert!(
        script.contains("typescript@^5.8.0"),
        "host npm install verifier should install the TypeScript compiler range used by the oracle"
    );
    assert!(
        !script.contains("join(\"/Users/lynn/code/CalcKernel\""),
        "host npm install verifier must not depend on the developer-local TypeScript oracle path"
    );
}

#[test]
fn host_npm_install_verifier_should_disable_source_checkout_fallback_for_release_signoff() {
    let verifier =
        std::fs::read_to_string("scripts/verify-host-npm-install.mjs").expect("read verifier");
    let wrapper = std::fs::read_to_string("npm/ckc.js").expect("read npm wrapper");

    assert!(
        verifier.contains("installedEnv.CKC_DISABLE_SOURCE_FALLBACK = \"1\""),
        "host npm install verifier must disable npm wrapper source checkout fallback during release sign-off"
    );
    assert!(
        verifier.contains("sourceFallback: \"disabled\""),
        "host npm install verifier must report disabled source fallback in sign-off JSON"
    );
    assert!(
        wrapper.contains("CKC_DISABLE_SOURCE_FALLBACK"),
        "npm wrapper must support disabling source checkout fallback for release sign-off"
    );
}

#[test]
fn host_npm_install_verifier_should_run_backend_runtime_smokes_for_release_signoff() {
    let verifier =
        std::fs::read_to_string("scripts/verify-host-npm-install.mjs").expect("read verifier");

    for expected in [
        "ckc build smoke.ck -o build/smoke-c",
        "node smoke-c-runtime.mjs",
        "node smoke-wasm-runtime.mjs",
        "node smoke-llvm-object-runtime.mjs",
    ] {
        assert!(
            verifier.contains(expected),
            "host npm install verifier must record backend runtime smoke command {expected}"
        );
    }
}

fn node_available() -> bool {
    Command::new("node")
        .arg("--version")
        .output()
        .is_ok_and(|output| output.status.success())
}

fn npm_available() -> bool {
    Command::new("npm")
        .arg("--version")
        .output()
        .is_ok_and(|output| output.status.success())
}
