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

#[test]
fn declaration_parity_verifier_should_reject_declaration_export_kind_mismatch() {
    if !node_available() {
        return;
    }

    let temp = temp_dir("rust-calckernel-declaration-kind-parity");
    fs::create_dir_all(&temp).expect("create temp dir");
    let rust_dts = temp.join("rust-index.d.ts");
    let typescript_dts = temp.join("typescript-index.d.ts");
    fs::write(&rust_dts, "export const Shared: number;\n").expect("write Rust mock declaration");
    fs::write(&typescript_dts, "export function Shared(): void;\n")
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
        "mismatched declaration export kinds should fail\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        String::from_utf8_lossy(&output.stderr).contains("declaration kind mismatch for Shared"),
        "failure should identify the mismatched declaration kind\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn declaration_parity_verifier_should_resolve_reexported_declaration_kinds() {
    if !node_available() {
        return;
    }

    let temp = temp_dir("rust-calckernel-declaration-reexport-kind-parity");
    fs::create_dir_all(&temp).expect("create temp dir");
    let rust_dts = temp.join("rust-index.d.ts");
    let typescript_dts = temp.join("typescript-index.d.ts");
    let typescript_source_dts = temp.join("typescript-source.d.ts");
    fs::write(&rust_dts, "export function Shared(): void;\n").expect("write Rust mock declaration");
    fs::write(&typescript_source_dts, "export function Shared(): void;\n")
        .expect("write TypeScript source declaration");
    fs::write(
        &typescript_dts,
        "export { Shared } from \"./typescript-source.js\";\n",
    )
    .expect("write TypeScript re-export declaration");

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
        output.status.success(),
        "re-exported declaration kinds should resolve before comparison\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn declaration_parity_verifier_should_reject_function_signature_mismatch() {
    if !node_available() {
        return;
    }

    let temp = temp_dir("rust-calckernel-declaration-signature-parity");
    fs::create_dir_all(&temp).expect("create temp dir");
    let rust_dts = temp.join("rust-index.d.ts");
    let typescript_dts = temp.join("typescript-index.d.ts");
    fs::write(&rust_dts, "export function Shared(value: number): void;\n")
        .expect("write Rust mock declaration");
    fs::write(
        &typescript_dts,
        "export function Shared(value: string): void;\n",
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
        "mismatched declaration function signatures should fail\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        String::from_utf8_lossy(&output.stderr).contains("function signature mismatch for Shared"),
        "failure should identify the mismatched function signature\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn declaration_parity_verifier_should_reject_class_member_mismatch() {
    if !node_available() {
        return;
    }

    let temp = temp_dir("rust-calckernel-declaration-class-member-parity");
    fs::create_dir_all(&temp).expect("create temp dir");
    let rust_dts = temp.join("rust-index.d.ts");
    let typescript_dts = temp.join("typescript-index.d.ts");
    fs::write(
        &rust_dts,
        "export declare class Shared { present(): number; }\n",
    )
    .expect("write Rust mock declaration");
    fs::write(
        &typescript_dts,
        "export declare class Shared { present(): number; missing(): string; }\n",
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
        "mismatched declaration class members should fail\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        String::from_utf8_lossy(&output.stderr).contains("declaration member mismatch for Shared"),
        "failure should identify the mismatched declaration class member\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn declaration_parity_verifier_should_reject_interface_member_mismatch() {
    if !node_available() {
        return;
    }

    let temp = temp_dir("rust-calckernel-declaration-interface-member-parity");
    fs::create_dir_all(&temp).expect("create temp dir");
    let rust_dts = temp.join("rust-index.d.ts");
    let typescript_dts = temp.join("typescript-index.d.ts");
    fs::write(&rust_dts, "export interface Shared { present: number; }\n")
        .expect("write Rust mock declaration");
    fs::write(
        &typescript_dts,
        "export interface Shared { present: number; missing: string; }\n",
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
        "mismatched declaration interface members should fail\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        String::from_utf8_lossy(&output.stderr)
            .contains("declaration interface member mismatch for Shared"),
        "failure should identify the mismatched declaration interface member\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn declaration_parity_verifier_should_reject_type_alias_mismatch() {
    if !node_available() {
        return;
    }

    let temp = temp_dir("rust-calckernel-declaration-type-alias-parity");
    fs::create_dir_all(&temp).expect("create temp dir");
    let rust_dts = temp.join("rust-index.d.ts");
    let typescript_dts = temp.join("typescript-index.d.ts");
    fs::write(&rust_dts, "export type Shared = { present: number };\n")
        .expect("write Rust mock declaration");
    fs::write(
        &typescript_dts,
        "export type Shared = { present: string };\n",
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
        "mismatched declaration type aliases should fail\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        String::from_utf8_lossy(&output.stderr)
            .contains("declaration type alias mismatch for Shared"),
        "failure should identify the mismatched declaration type alias\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn declaration_parity_verifier_should_reject_enum_member_mismatch() {
    if !node_available() {
        return;
    }

    let temp = temp_dir("rust-calckernel-declaration-enum-member-parity");
    fs::create_dir_all(&temp).expect("create temp dir");
    let rust_dts = temp.join("rust-index.d.ts");
    let typescript_dts = temp.join("typescript-index.d.ts");
    fs::write(
        &rust_dts,
        "export declare enum Shared { Present = \"Present\" }\n",
    )
    .expect("write Rust mock declaration");
    fs::write(
        &typescript_dts,
        "export declare enum Shared { Present = \"Present\", Missing = \"Missing\" }\n",
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
        "mismatched declaration enum members should fail\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        String::from_utf8_lossy(&output.stderr)
            .contains("declaration enum member mismatch for Shared"),
        "failure should identify the mismatched declaration enum member\nstdout:\n{}\nstderr:\n{}",
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
