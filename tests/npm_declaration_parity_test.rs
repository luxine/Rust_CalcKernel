use std::{
    fs,
    path::PathBuf,
    process::Command,
    time::{SystemTime, UNIX_EPOCH},
};

#[test]
fn declaration_parity_verifier_should_be_registered_as_npm_script() {
    let package_json = fs::read_to_string("package.json").expect("read package.json");

    assert!(
        package_json.contains(
            "\"verify:declaration-parity\": \"node scripts/verify-declaration-parity.mjs\""
        ),
        "package.json must expose verify:declaration-parity"
    );
}

#[test]
fn declaration_parity_verifier_should_accept_current_typescript_oracle_exports() {
    if !node_available() {
        return;
    }

    let output = Command::new("node")
        .arg("scripts/verify-declaration-parity.mjs")
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .output()
        .expect("run declaration parity verifier");

    assert!(
        output.status.success(),
        "current Rust declarations should match TypeScript oracle exports\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        String::from_utf8_lossy(&output.stdout).contains("\"status\": \"ok\""),
        "declaration parity verifier should print JSON status\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn declaration_parity_verifier_should_reject_extra_or_missing_declaration_exports() {
    if !node_available() {
        return;
    }

    let temp = temp_dir("rust-calckernel-declaration-parity");
    fs::create_dir_all(&temp).expect("create temp dir");
    let rust_dts = temp.join("rust-index.d.ts");
    let typescript_dts = temp.join("typescript-index.d.ts");
    fs::write(
        &rust_dts,
        "export interface Shared {}\nexport type OnlyRust = string;\n",
    )
    .expect("write Rust mock declaration");
    fs::write(
        &typescript_dts,
        "export interface Shared {}\nexport type OnlyTypescript = string;\n",
    )
    .expect("write TypeScript mock declaration");

    let output = Command::new("node")
        .arg("scripts/verify-declaration-parity.mjs")
        .arg("--rust-dts")
        .arg(&rust_dts)
        .arg("--typescript-dts")
        .arg(&typescript_dts)
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .output()
        .expect("run declaration parity verifier");

    let _ = fs::remove_dir_all(&temp);

    assert!(
        !output.status.success(),
        "mismatched declaration exports should fail\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        String::from_utf8_lossy(&output.stderr).contains("extra Rust declaration exports")
            && String::from_utf8_lossy(&output.stderr).contains("missing Rust declaration exports"),
        "failure should identify both extra and missing Rust declaration exports\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
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
