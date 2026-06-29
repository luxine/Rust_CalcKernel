use std::{
    fs,
    path::PathBuf,
    process::Command,
    time::{SystemTime, UNIX_EPOCH},
};

const TARGETS: [&str; 6] = [
    "darwin-arm64",
    "darwin-x64",
    "linux-arm64",
    "linux-x64",
    "win32-arm64",
    "win32-x64",
];

const TARBALL_SHA256: &str = "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef";
const REQUIRED_COMMANDS: [&str; 8] = [
    "ckc --help",
    "ckc check smoke.ck",
    "ckc emit-mir smoke.ck -o build/smoke.mir",
    "ckc emit-c smoke.ck -o build/smoke.c",
    "ckc emit-wat smoke.ck -o build/smoke.wat",
    "ckc emit-wasm smoke.ck -o build/smoke.wasm",
    "ckc emit-llvm smoke.ck -o build/smoke.ll",
    "ckc build-llvm smoke.ck --kind object -o build/smoke.o",
];

#[test]
fn release_signoff_verifier_should_reject_missing_target_smoke() {
    if !node_available() {
        return;
    }

    let temp = temp_dir("rust-calckernel-release-signoff-missing");
    let manifest = temp.join("release-manifest.json");
    let signoffs = temp.join("signoffs");
    fs::create_dir_all(&signoffs).expect("create signoff dir");
    fs::write(&manifest, release_manifest_json()).expect("write release manifest");
    for target in TARGETS.iter().filter(|target| **target != "win32-x64") {
        fs::write(
            signoffs.join(format!("{target}.json")),
            signoff_json(target),
        )
        .expect("write signoff");
    }

    let output = Command::new("node")
        .arg("scripts/verify-npm-release-signoff.mjs")
        .arg(&manifest)
        .arg(&signoffs)
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .output()
        .expect("run release signoff verifier");

    let _ = fs::remove_dir_all(&temp);

    assert!(
        !output.status.success(),
        "missing target signoff should fail\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        String::from_utf8_lossy(&output.stderr).contains("missing platform sign-off for win32-x64"),
        "missing target failure should identify win32-x64\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn release_signoff_verifier_should_reject_missing_build_llvm_smoke() {
    if !node_available() {
        return;
    }

    let temp = temp_dir("rust-calckernel-release-signoff-missing-build-llvm");
    let manifest = temp.join("release-manifest.json");
    let signoffs = temp.join("signoffs");
    fs::create_dir_all(&signoffs).expect("create signoff dir");
    fs::write(&manifest, release_manifest_json()).expect("write release manifest");
    let commands_without_build_llvm = REQUIRED_COMMANDS
        .iter()
        .copied()
        .filter(|command| !command.starts_with("ckc build-llvm "))
        .collect::<Vec<_>>();
    for target in TARGETS {
        fs::write(
            signoffs.join(format!("{target}.json")),
            signoff_json_with_commands(target, &commands_without_build_llvm),
        )
        .expect("write signoff");
    }

    let output = Command::new("node")
        .arg("scripts/verify-npm-release-signoff.mjs")
        .arg(&manifest)
        .arg(&signoffs)
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .output()
        .expect("run release signoff verifier");

    let _ = fs::remove_dir_all(&temp);

    assert!(
        !output.status.success(),
        "missing build-llvm signoff should fail\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        String::from_utf8_lossy(&output.stderr)
            .contains("ckc build-llvm smoke.ck --kind object -o build/smoke.o"),
        "missing build-llvm failure should identify the required command\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn release_signoff_verifier_should_reject_missing_packaged_binary_evidence() {
    if !node_available() {
        return;
    }

    let temp = temp_dir("rust-calckernel-release-signoff-missing-package-binary");
    let manifest = temp.join("release-manifest.json");
    let signoffs = temp.join("signoffs");
    fs::create_dir_all(&signoffs).expect("create signoff dir");
    fs::write(&manifest, release_manifest_json()).expect("write release manifest");
    for target in TARGETS {
        fs::write(
            signoffs.join(format!("{target}.json")),
            signoff_json_without_binary_evidence(target),
        )
        .expect("write signoff");
    }

    let output = Command::new("node")
        .arg("scripts/verify-npm-release-signoff.mjs")
        .arg(&manifest)
        .arg(&signoffs)
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .output()
        .expect("run release signoff verifier");

    let _ = fs::remove_dir_all(&temp);

    assert!(
        !output.status.success(),
        "missing packaged binary evidence should fail\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        String::from_utf8_lossy(&output.stderr).contains("packagedBinary"),
        "missing packaged binary evidence failure should identify packagedBinary\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn release_signoff_verifier_should_accept_complete_target_smokes() {
    if !node_available() {
        return;
    }

    let temp = temp_dir("rust-calckernel-release-signoff-complete");
    let manifest = temp.join("release-manifest.json");
    let signoffs = temp.join("signoffs");
    fs::create_dir_all(&signoffs).expect("create signoff dir");
    fs::write(&manifest, release_manifest_json()).expect("write release manifest");
    for target in TARGETS {
        fs::write(
            signoffs.join(format!("{target}.json")),
            signoff_json(target),
        )
        .expect("write signoff");
    }

    let output = Command::new("node")
        .arg("scripts/verify-npm-release-signoff.mjs")
        .arg(&manifest)
        .arg(&signoffs)
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .output()
        .expect("run release signoff verifier");

    let _ = fs::remove_dir_all(&temp);

    assert!(
        output.status.success(),
        "complete target signoffs should pass\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        String::from_utf8_lossy(&output.stdout).contains("\"targetCount\": 6"),
        "complete target signoff should report all six targets\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

fn release_manifest_json() -> String {
    let targets = TARGETS
        .iter()
        .map(|target| format!("{{\"name\":\"{target}\"}}"))
        .collect::<Vec<_>>()
        .join(",");
    format!(
        "{{\"packageName\":\"calckernel\",\"packageVersion\":\"0.8.0\",\"tarball\":\"calckernel-0.8.0.tgz\",\"tarballSha256\":\"{TARBALL_SHA256}\",\"targets\":[{targets}]}}"
    )
}

fn signoff_json(target: &str) -> String {
    signoff_json_with_commands(target, &REQUIRED_COMMANDS)
}

fn signoff_json_with_commands(target: &str, commands: &[&str]) -> String {
    let installed_bin = installed_bin_evidence(target);
    let packaged_binary = packaged_binary_evidence(target);
    signoff_json_with_commands_and_binary_evidence(
        target,
        commands,
        &format!("{installed_bin}{packaged_binary}"),
    )
}

fn signoff_json_without_binary_evidence(target: &str) -> String {
    signoff_json_with_commands_and_binary_evidence(
        target,
        &REQUIRED_COMMANDS,
        &installed_bin_evidence(target),
    )
}

fn signoff_json_with_commands_and_binary_evidence(
    target: &str,
    commands: &[&str],
    binary_evidence: &str,
) -> String {
    let commands_json = commands
        .iter()
        .map(|command| format!("    {command:?}"))
        .collect::<Vec<_>>()
        .join(",\n");
    format!(
        r#"{{
  "package": "calckernel",
  "targetName": "{target}",
  "tarball": "calckernel-0.8.0.tgz",
  "tarballSha256": "{TARBALL_SHA256}",
  "commands": [
{commands_json}
  ],
  "apiSymbols": [
    "SourceFile",
    "TokenKind",
    "lex",
    "parse",
    "check",
    "getFunctionInfo",
    "emitCHeader",
    "emitCSource",
    "CKWasmArena",
    "createCKWasmArena"
  ],
  "typeSmoke": "passed",
  "ckcBinOverride": "unset"{binary_evidence}
}}"#
    )
}

fn installed_bin_evidence(target: &str) -> String {
    if target.starts_with("win32-") {
        r#",
  "installedBin": "C:\\consumer\\node_modules\\.bin\\ckc.cmd""#
            .to_string()
    } else {
        r#",
  "installedBin": "/tmp/consumer/node_modules/.bin/ckc""#
            .to_string()
    }
}

fn packaged_binary_evidence(target: &str) -> String {
    let binary_file = match target {
        "win32-arm64" => "ckc-win32-arm64.exe",
        "win32-x64" => "ckc-win32-x64.exe",
        _ => target,
    };
    let binary_file = if target.starts_with("win32-") {
        binary_file.to_string()
    } else {
        format!("ckc-{binary_file}")
    };

    if target.starts_with("win32-") {
        format!(
            r#",
  "packagedBinary": "C:\\consumer\\node_modules\\calckernel\\npm\\bin\\{binary_file}""#
        )
    } else {
        format!(
            r#",
  "packagedBinary": "/tmp/consumer/node_modules/calckernel/npm/bin/{binary_file}""#
        )
    }
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
