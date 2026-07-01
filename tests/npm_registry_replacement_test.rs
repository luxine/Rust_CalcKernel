use std::{
    fs,
    path::Path,
    path::PathBuf,
    process::Command,
    time::{SystemTime, UNIX_EPOCH},
};

#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;

const VALID_SHASUM: &str = "0123456789abcdef0123456789abcdef01234567";
const VALID_INTEGRITY: &str = "sha512-AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA==";

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
    assert!(
        String::from_utf8_lossy(&output.stdout).contains("\"packageVersion\": \"0.8.0\""),
        "success output should report the registry packageVersion explicitly\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        String::from_utf8_lossy(&output.stdout)
            .contains(&format!("\"shasum\": \"{VALID_SHASUM}\"")),
        "success output should report the registry dist.shasum\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        String::from_utf8_lossy(&output.stdout).contains("\"description\": \"A small CK / CalcKernel integer-computation DSL compiler with C, WASM, and LLVM backends.\""),
        "success output should report the public package description\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        String::from_utf8_lossy(&output.stdout).contains("\"keywords\": ["),
        "success output should report the public package keywords\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        String::from_utf8_lossy(&output.stdout).contains("\"repository\": {"),
        "success output should report the public package repository\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn registry_replacement_verifier_should_accept_npm_normalized_metadata() {
    if !node_available() {
        return;
    }

    let temp = temp_dir("rust-calckernel-registry-normalized");
    fs::create_dir_all(&temp).expect("create temp dir");
    let metadata = temp.join("metadata.json");
    fs::write(&metadata, rust_registry_metadata_normalized()).expect("write metadata");

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
        "npm-normalized Rust package metadata should pass\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

#[cfg(unix)]
#[test]
fn registry_replacement_verifier_should_retry_transient_registry_lookup_failure() {
    if !node_available() {
        return;
    }

    let temp = temp_dir("rust-calckernel-registry-retry");
    fs::create_dir_all(&temp).expect("create temp dir");
    let bin_dir = temp.join("bin");
    fs::create_dir_all(&bin_dir).expect("create fake npm dir");
    let counter = temp.join("counter.txt");
    let fake_npm = bin_dir.join("npm");
    fs::write(&fake_npm, fake_npm_retry_script()).expect("write fake npm");
    let mut permissions = fs::metadata(&fake_npm)
        .expect("fake npm metadata")
        .permissions();
    permissions.set_mode(0o755);
    fs::set_permissions(&fake_npm, permissions).expect("make fake npm executable");

    let output = Command::new("node")
        .arg("scripts/verify-npm-registry-replacement.mjs")
        .arg("0.8.0")
        .env("PATH", prepend_path(&bin_dir))
        .env("CK_FAKE_NPM_COUNTER", &counter)
        .env("CK_NPM_REGISTRY_RETRY_ATTEMPTS", "2")
        .env("CK_NPM_REGISTRY_RETRY_DELAY_MS", "1")
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .output()
        .expect("run npm registry replacement verifier with retrying fake npm");

    let counter_value = fs::read_to_string(&counter).unwrap_or_default();
    let _ = fs::remove_dir_all(&temp);

    assert!(
        output.status.success(),
        "transient npm view failure should be retried\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(
        counter_value.trim(),
        "2",
        "fake npm should be called twice after a transient failure"
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
fn registry_replacement_verifier_should_reject_invalid_dist_shasum() {
    if !node_available() {
        return;
    }

    let temp = temp_dir("rust-calckernel-registry-shasum");
    fs::create_dir_all(&temp).expect("create temp dir");
    let metadata = temp.join("metadata.json");
    fs::write(&metadata, rust_metadata_with_shasum("not-a-sha1")).expect("write metadata");

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
        "invalid registry shasum should fail\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        String::from_utf8_lossy(&output.stderr).contains("dist.shasum"),
        "failure should identify dist.shasum\nstdout:\n{}\nstderr:\n{}",
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

#[test]
fn registry_replacement_verifier_should_reject_public_identity_mismatch() {
    if !node_available() {
        return;
    }

    let temp = temp_dir("rust-calckernel-registry-identity");
    fs::create_dir_all(&temp).expect("create temp dir");
    let metadata = temp.join("metadata.json");
    fs::write(
        &metadata,
        rust_metadata_with_public_identity(
            "Rust rewrite of the CK / CalcKernel compiler with C, WASM, and LLVM backends.",
            r#"["calckernel", "ck", "compiler", "dsl", "c", "wasm", "llvm", "rust"]"#,
        ),
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
        "public identity mismatch should fail\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        String::from_utf8_lossy(&output.stderr).contains("description")
            || String::from_utf8_lossy(&output.stderr).contains("keywords"),
        "failure should identify package identity metadata\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

fn rust_metadata() -> String {
    rust_metadata_with_integrity(VALID_INTEGRITY)
}

fn rust_metadata_with_integrity(integrity: &str) -> String {
    rust_metadata_with_integrity_and_shasum(integrity, VALID_SHASUM)
}

fn rust_metadata_with_shasum(shasum: &str) -> String {
    rust_metadata_with_integrity_and_shasum(VALID_INTEGRITY, shasum)
}

fn rust_metadata_with_integrity_and_shasum(integrity: &str, shasum: &str) -> String {
    rust_metadata_with_public_identity_and_dist(
        "A small CK / CalcKernel integer-computation DSL compiler with C, WASM, and LLVM backends.",
        r#"["calckernel", "ck", "compiler", "dsl", "c", "wasm", "llvm"]"#,
        integrity,
        shasum,
    )
}

fn rust_metadata_with_public_identity(description: &str, keywords: &str) -> String {
    rust_metadata_with_public_identity_and_dist(
        description,
        keywords,
        VALID_INTEGRITY,
        VALID_SHASUM,
    )
}

fn rust_metadata_with_public_identity_and_dist(
    description: &str,
    keywords: &str,
    integrity: &str,
    shasum: &str,
) -> String {
    format!(
        r#"{{
  "name": "calckernel",
  "version": "0.8.0",
  "description": "{description}",
  "keywords": {keywords},
  "repository": {{
    "type": "git",
    "url": "https://github.com/luxine/Rust_CalcKernel"
  }},
  "license": "MIT",
  "engines": {{
    "node": ">=20"
  }},
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
  "dist": {{
    "tarball": "https://registry.npmjs.org/calckernel/-/calckernel-0.8.0.tgz",
    "shasum": "{shasum}",
    "integrity": "{integrity}"
  }}
}}"#
    )
}

fn rust_metadata_with_scripts(scripts: &str) -> String {
    format!(
        r#"{{
  "name": "calckernel",
  "version": "0.8.0",
  "description": "A small CK / CalcKernel integer-computation DSL compiler with C, WASM, and LLVM backends.",
  "keywords": ["calckernel", "ck", "compiler", "dsl", "c", "wasm", "llvm"],
  "repository": {{
    "type": "git",
    "url": "https://github.com/luxine/Rust_CalcKernel"
  }},
  "license": "MIT",
  "engines": {{
    "node": ">=20"
  }},
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
    "shasum": "{VALID_SHASUM}",
    "integrity": "{VALID_INTEGRITY}"
  }}
}}"#
    )
}

fn rust_registry_metadata_normalized() -> String {
    format!(
        r#"{{
  "name": "calckernel",
  "version": "0.8.0",
  "description": "A small CK / CalcKernel integer-computation DSL compiler with C, WASM, and LLVM backends.",
  "keywords": ["calckernel", "ck", "compiler", "dsl", "c", "wasm", "llvm"],
  "repository": {{
    "type": "git",
    "url": "git+https://github.com/luxine/Rust_CalcKernel.git"
  }},
  "license": "MIT",
  "engines": {{
    "node": ">=20"
  }},
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
    "ckc": "npm/ckc.js"
  }},
  "dist": {{
    "tarball": "https://registry.npmjs.org/calckernel/-/calckernel-0.8.0.tgz",
    "shasum": "{VALID_SHASUM}",
    "integrity": "{VALID_INTEGRITY}"
  }}
}}"#
    )
}

#[cfg(unix)]
fn prepend_path(bin_dir: &Path) -> String {
    let current_path = std::env::var("PATH").unwrap_or_default();
    format!("{}:{}", bin_dir.display(), current_path)
}

#[cfg(unix)]
fn fake_npm_retry_script() -> &'static str {
    r#"#!/bin/sh
count_file="$CK_FAKE_NPM_COUNTER"
count=0
if [ -f "$count_file" ]; then
  count="$(cat "$count_file")"
fi
count=$((count + 1))
printf '%s\n' "$count" > "$count_file"
if [ "$count" -eq 1 ]; then
  cat <<'JSON'
{"error":{"code":"E404","summary":"Not Found"}}
JSON
  exit 1
fi
cat <<'JSON'
{
  "name": "calckernel",
  "version": "0.8.0",
  "description": "A small CK / CalcKernel integer-computation DSL compiler with C, WASM, and LLVM backends.",
  "keywords": ["calckernel", "ck", "compiler", "dsl", "c", "wasm", "llvm"],
  "repository": {
    "type": "git",
    "url": "git+https://github.com/luxine/Rust_CalcKernel.git"
  },
  "license": "MIT",
  "engines": {
    "node": ">=20"
  },
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
    "ckc": "npm/ckc.js"
  },
  "dist": {
    "tarball": "https://registry.npmjs.org/calckernel/-/calckernel-0.8.0.tgz",
    "shasum": "0123456789abcdef0123456789abcdef01234567",
    "integrity": "sha512-AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA=="
  }
}
JSON
"#
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
