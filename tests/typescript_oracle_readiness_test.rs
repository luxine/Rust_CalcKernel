use std::{
    fs,
    path::PathBuf,
    process::Command,
    time::{SystemTime, UNIX_EPOCH},
};

#[test]
fn typescript_oracle_verifier_should_confirm_default_cli_oracle_is_available() {
    if !node_available() {
        return;
    }

    let output = Command::new("node")
        .arg("scripts/verify-typescript-oracle.mjs")
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .output()
        .expect("run TypeScript oracle verifier");

    assert!(
        output.status.success(),
        "TypeScript oracle verifier failed\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        String::from_utf8_lossy(&output.stdout).contains("\"cliStatus\": \"ok\""),
        "TypeScript oracle verifier should report CLI health\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn typescript_oracle_verifier_should_reject_missing_dist_cli() {
    if !node_available() {
        return;
    }

    let temp = temp_dir("rust-calckernel-missing-ts-cli");
    fs::create_dir_all(&temp).expect("create temp TypeScript oracle root");
    fs::write(
        temp.join("package.json"),
        r#"{
  "name": "calckernel",
  "main": "./dist/src/index.js",
  "bin": {
    "ckc": "./dist/src/cli.js"
  },
  "dependencies": {
    "wabt": "^1.0.39"
  }
}"#,
    )
    .expect("write package json");

    let output = Command::new("node")
        .arg("scripts/verify-typescript-oracle.mjs")
        .env("CALCKERNEL_TS_ROOT", &temp)
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .output()
        .expect("run TypeScript oracle verifier");

    let _ = fs::remove_dir_all(&temp);

    assert!(
        !output.status.success(),
        "missing dist CLI should fail\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        String::from_utf8_lossy(&output.stderr).contains("dist/src/cli.js"),
        "missing dist CLI failure should identify the oracle CLI\nstdout:\n{}\nstderr:\n{}",
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
