use std::{
    fs,
    path::{Path, PathBuf},
    process::Command,
    time::{SystemTime, UNIX_EPOCH},
};

#[test]
fn publish_artifact_verifier_should_accept_manifest_tarball_with_matching_sha256() {
    if !node_available() {
        return;
    }

    let temp = temp_dir("rust-calckernel-publish-artifact-ok");
    let dist = temp.join("dist");
    fs::create_dir_all(&dist).expect("create dist");
    let tarball = dist.join("calckernel-0.8.0.tgz");
    fs::write(&tarball, b"release tarball bytes").expect("write tarball");
    let manifest = temp.join("release-manifest.json");
    fs::write(
        &manifest,
        release_manifest_json("calckernel-0.8.0.tgz", &sha256_file(&tarball)),
    )
    .expect("write release manifest");

    let output = Command::new("node")
        .arg("scripts/verify-npm-publish-artifact.mjs")
        .arg(&manifest)
        .arg(&dist)
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .output()
        .expect("run publish artifact verifier");

    let _ = fs::remove_dir_all(&temp);

    assert!(
        output.status.success(),
        "matching publish artifact should pass\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        String::from_utf8_lossy(&output.stdout).contains("\"tarballSha256\""),
        "publish artifact verifier should report tarball SHA256\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn publish_artifact_verifier_should_reject_tarball_sha256_mismatch() {
    if !node_available() {
        return;
    }

    let temp = temp_dir("rust-calckernel-publish-artifact-mismatch");
    let dist = temp.join("dist");
    fs::create_dir_all(&dist).expect("create dist");
    let tarball = dist.join("calckernel-0.8.0.tgz");
    fs::write(&tarball, b"different tarball bytes").expect("write tarball");
    let manifest = temp.join("release-manifest.json");
    fs::write(
        &manifest,
        release_manifest_json(
            "calckernel-0.8.0.tgz",
            "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef",
        ),
    )
    .expect("write release manifest");

    let output = Command::new("node")
        .arg("scripts/verify-npm-publish-artifact.mjs")
        .arg(&manifest)
        .arg(&dist)
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .output()
        .expect("run publish artifact verifier");

    let _ = fs::remove_dir_all(&temp);

    assert!(
        !output.status.success(),
        "mismatched publish artifact should fail\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        String::from_utf8_lossy(&output.stderr).contains("tarballSha256 does not match"),
        "mismatch failure should identify tarball SHA256\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn publish_artifact_verifier_should_reject_incomplete_release_manifest() {
    if !node_available() {
        return;
    }

    let temp = temp_dir("rust-calckernel-publish-artifact-incomplete-manifest");
    let dist = temp.join("dist");
    fs::create_dir_all(&dist).expect("create dist");
    let tarball = dist.join("calckernel-0.8.0.tgz");
    fs::write(&tarball, b"release tarball bytes").expect("write tarball");
    let manifest = temp.join("release-manifest.json");
    fs::write(
        &manifest,
        incomplete_release_manifest_json("calckernel-0.8.0.tgz", &sha256_file(&tarball)),
    )
    .expect("write incomplete release manifest");

    let output = Command::new("node")
        .arg("scripts/verify-npm-publish-artifact.mjs")
        .arg(&manifest)
        .arg(&dist)
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .output()
        .expect("run publish artifact verifier");

    let _ = fs::remove_dir_all(&temp);

    assert!(
        !output.status.success(),
        "incomplete release manifest should fail\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        String::from_utf8_lossy(&output.stderr).contains("release manifest packageMetadata")
            || String::from_utf8_lossy(&output.stderr).contains("release manifest fileSurface"),
        "failure should identify missing full release manifest evidence\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

fn release_manifest_json(tarball: &str, tarball_sha256: &str) -> String {
    format!(
        r#"{{
  "packageName": "calckernel",
  "packageVersion": "0.8.0",
  "packageMetadata": {{
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
    "dependencyFields": {{}},
    "consumerInstallScripts": []
  }},
  "tarball": "{tarball}",
  "tarballSha256": "{tarball_sha256}",
  "fileSurface": {{
    "packageJsonFiles": [
      "npm",
      "README.md",
      "README.zh-CN.md",
      "docs/npm-release.md",
      "docs/architecture-review.md",
      "docs/zh-CN/architecture-review.md"
    ],
    "requiredFiles": [
      "package/package.json",
      "package/npm/ckc.js",
      "package/npm/platform.js",
      "package/npm/index.js",
      "package/npm/index.d.ts",
      "package/docs/npm-release.md",
      "package/docs/architecture-review.md",
      "package/docs/zh-CN/architecture-review.md",
      "package/README.md",
      "package/README.zh-CN.md"
    ],
    "forbiddenPrefixes": [
      "package/docs/superpowers/",
      "package/src/",
      "package/target/"
    ],
    "allowedEntries": [
      "package/README.md",
      "package/README.zh-CN.md",
      "package/docs/architecture-review.md",
      "package/docs/npm-release.md",
      "package/docs/zh-CN/architecture-review.md",
      "package/npm/bin/ckc-darwin-arm64",
      "package/npm/bin/ckc-darwin-x64",
      "package/npm/bin/ckc-linux-arm64",
      "package/npm/bin/ckc-linux-x64",
      "package/npm/bin/ckc-win32-arm64.exe",
      "package/npm/bin/ckc-win32-x64.exe",
      "package/npm/ckc.js",
      "package/npm/index.d.ts",
      "package/npm/index.js",
      "package/npm/platform.js",
      "package/package.json"
    ]
  }},
  "targets": [
    {{
      "name": "darwin-arm64",
      "rustTarget": "aarch64-apple-darwin",
      "binaryPath": "package/npm/bin/ckc-darwin-arm64",
      "fileMode": "-rwxr-xr-x",
      "binaryFormat": "mach-o",
      "binaryArchitecture": "arm64",
      "sizeBytes": 1,
      "sha256": "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef"
    }},
    {{
      "name": "darwin-x64",
      "rustTarget": "x86_64-apple-darwin",
      "binaryPath": "package/npm/bin/ckc-darwin-x64",
      "fileMode": "-rwxr-xr-x",
      "binaryFormat": "mach-o",
      "binaryArchitecture": "x86_64",
      "sizeBytes": 1,
      "sha256": "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef"
    }},
    {{
      "name": "linux-arm64",
      "rustTarget": "aarch64-unknown-linux-gnu",
      "binaryPath": "package/npm/bin/ckc-linux-arm64",
      "fileMode": "-rwxr-xr-x",
      "binaryFormat": "elf",
      "binaryArchitecture": "aarch64",
      "sizeBytes": 1,
      "sha256": "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef"
    }},
    {{
      "name": "linux-x64",
      "rustTarget": "x86_64-unknown-linux-gnu",
      "binaryPath": "package/npm/bin/ckc-linux-x64",
      "fileMode": "-rwxr-xr-x",
      "binaryFormat": "elf",
      "binaryArchitecture": "x86_64",
      "sizeBytes": 1,
      "sha256": "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef"
    }},
    {{
      "name": "win32-arm64",
      "rustTarget": "aarch64-pc-windows-msvc",
      "binaryPath": "package/npm/bin/ckc-win32-arm64.exe",
      "fileMode": "-rw-r--r--",
      "binaryFormat": "pe",
      "binaryArchitecture": "arm64",
      "sizeBytes": 1,
      "sha256": "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef"
    }},
    {{
      "name": "win32-x64",
      "rustTarget": "x86_64-pc-windows-msvc",
      "binaryPath": "package/npm/bin/ckc-win32-x64.exe",
      "fileMode": "-rw-r--r--",
      "binaryFormat": "pe",
      "binaryArchitecture": "x86_64",
      "sizeBytes": 1,
      "sha256": "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef"
    }}
  ]
}}"#
    )
}

fn incomplete_release_manifest_json(tarball: &str, tarball_sha256: &str) -> String {
    format!(
        r#"{{
  "packageName": "calckernel",
  "packageVersion": "0.8.0",
  "tarball": "{tarball}",
  "tarballSha256": "{tarball_sha256}"
}}"#
    )
}

fn sha256_file(path: &Path) -> String {
    let output = Command::new("shasum")
        .arg("-a")
        .arg("256")
        .arg(path)
        .output()
        .expect("run shasum");
    assert!(
        output.status.success(),
        "shasum failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8_lossy(&output.stdout)
        .split_whitespace()
        .next()
        .expect("sha256")
        .to_string()
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
