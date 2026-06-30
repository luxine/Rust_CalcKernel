use std::{
    fs,
    path::PathBuf,
    process::Command,
    time::{SystemTime, UNIX_EPOCH},
};

#[test]
fn registry_replacement_verifier_should_accept_rust_package_metadata() {
    if !node_available() {
        return;
    }

    let temp = temp_dir("rust-calckernel-registry-ok");
    fs::create_dir_all(&temp).expect("create temp dir");
    let metadata = temp.join("metadata.json");
    fs::write(&metadata, rust_metadata()).expect("write metadata");

    let output = Command::new("node")
        .arg("scripts/verify-npm-registry-replacement.mjs")
        .arg("--metadata-file")
        .arg(&metadata)
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .output()
        .expect("run npm registry replacement verifier");

    let _ = fs::remove_dir_all(&temp);

    assert!(
        output.status.success(),
        "Rust package metadata should pass\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        String::from_utf8_lossy(&output.stdout).contains("\"status\": \"ok\""),
        "success output should be a JSON status\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn registry_replacement_verifier_should_reject_typescript_package_metadata() {
    if !node_available() {
        return;
    }

    let temp = temp_dir("rust-calckernel-registry-ts");
    fs::create_dir_all(&temp).expect("create temp dir");
    let metadata = temp.join("metadata.json");
    fs::write(&metadata, typescript_metadata()).expect("write metadata");

    let output = Command::new("node")
        .arg("scripts/verify-npm-registry-replacement.mjs")
        .arg("--metadata-file")
        .arg(&metadata)
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .output()
        .expect("run npm registry replacement verifier");

    let _ = fs::remove_dir_all(&temp);

    assert!(
        !output.status.success(),
        "TypeScript package metadata should fail\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        String::from_utf8_lossy(&output.stderr).contains("main must be ./npm/index.js"),
        "failure should identify stale TypeScript package metadata\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn registry_replacement_verifier_should_reject_invalid_dist_integrity() {
    if !node_available() {
        return;
    }

    let temp = temp_dir("rust-calckernel-registry-integrity");
    fs::create_dir_all(&temp).expect("create temp dir");
    let metadata = temp.join("metadata.json");
    fs::write(
        &metadata,
        rust_metadata_with_integrity("not-a-valid-npm-integrity"),
    )
    .expect("write metadata");

    let output = Command::new("node")
        .arg("scripts/verify-npm-registry-replacement.mjs")
        .arg("--metadata-file")
        .arg(&metadata)
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .output()
        .expect("run npm registry replacement verifier");

    let _ = fs::remove_dir_all(&temp);

    assert!(
        !output.status.success(),
        "invalid registry integrity should fail\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        String::from_utf8_lossy(&output.stderr).contains("dist.integrity"),
        "failure should identify dist.integrity\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn registry_replacement_verifier_should_reject_consumer_install_lifecycle_scripts() {
    if !node_available() {
        return;
    }

    let temp = temp_dir("rust-calckernel-registry-install-script");
    fs::create_dir_all(&temp).expect("create temp dir");
    let metadata = temp.join("metadata.json");
    fs::write(
        &metadata,
        rust_metadata_with_scripts(r#""postinstall": "node-gyp rebuild""#),
    )
    .expect("write metadata");

    let output = Command::new("node")
        .arg("scripts/verify-npm-registry-replacement.mjs")
        .arg("--metadata-file")
        .arg(&metadata)
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .output()
        .expect("run npm registry replacement verifier");

    let _ = fs::remove_dir_all(&temp);

    assert!(
        !output.status.success(),
        "consumer install lifecycle scripts should fail\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        String::from_utf8_lossy(&output.stderr)
            .contains("consumer install lifecycle script postinstall"),
        "failure should identify the forbidden install lifecycle script\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

fn rust_metadata() -> String {
    rust_metadata_with_integrity(VALID_INTEGRITY)
}

const VALID_INTEGRITY: &str = "sha512-AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA==";

fn rust_metadata_with_integrity(integrity: &str) -> String {
    r#"{
  "name": "calckernel",
  "version": "0.8.0",
  "type": "module",
  "main": "./npm/index.js",
  "types": "./npm/index.d.ts",
  "exports": {
    ".": {
      "types": "./npm/index.d.ts",
      "import": "./npm/index.js"
    }
  },
  "bin": {
    "ckc": "./npm/ckc.js"
  },
  "dist": {
    "tarball": "https://registry.npmjs.org/calckernel/-/calckernel-0.8.0.tgz",
    "integrity": "__INTEGRITY__"
  }
}"#
    .replace("__INTEGRITY__", integrity)
}

fn rust_metadata_with_scripts(scripts: &str) -> String {
    format!(
        r#"{{
  "name": "calckernel",
  "version": "0.8.0",
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
  "scripts": {{
    {scripts}
  }},
  "dist": {{
    "tarball": "https://registry.npmjs.org/calckernel/-/calckernel-0.8.0.tgz",
    "integrity": "{VALID_INTEGRITY}"
  }}
}}"#
    )
}

fn typescript_metadata() -> &'static str {
    r#"{
  "name": "calckernel",
  "version": "0.8.0",
  "main": "./dist/src/index.js",
  "types": "./dist/src/index.d.ts",
  "bin": {
    "ckc": "./dist/src/cli.js"
  },
  "dependencies": {
    "wabt": "^1.0.39"
  },
  "dist": {
    "tarball": "https://registry.npmjs.org/calckernel/-/calckernel-0.8.0.tgz",
    "integrity": "sha512-test"
  }
}"#
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
