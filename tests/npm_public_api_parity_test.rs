use std::{
    fs,
    path::PathBuf,
    process::Command,
    time::{SystemTime, UNIX_EPOCH},
};

#[test]
fn public_api_parity_verifier_should_be_registered_as_npm_script() {
    let package_json = fs::read_to_string("package.json").expect("read package.json");

    assert!(
        package_json.contains(
            "\"verify:public-api-parity\": \"node scripts/verify-public-api-parity.mjs\""
        ),
        "package.json must expose verify:public-api-parity"
    );
}

#[test]
fn public_api_parity_verifier_should_accept_current_typescript_oracle_exports() {
    if !node_available() {
        return;
    }

    let output = Command::new("node")
        .arg("scripts/verify-public-api-parity.mjs")
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .output()
        .expect("run public API parity verifier");

    assert!(
        output.status.success(),
        "current Rust public API should match TypeScript oracle exports\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        String::from_utf8_lossy(&output.stdout).contains("\"status\": \"ok\""),
        "public API parity verifier should print JSON status\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn public_api_parity_verifier_should_reject_extra_or_missing_runtime_exports() {
    if !node_available() {
        return;
    }

    let temp = temp_dir("rust-calckernel-public-api-parity");
    fs::create_dir_all(&temp).expect("create temp dir");
    let rust_index = temp.join("rust-index.mjs");
    let typescript_index = temp.join("typescript-index.mjs");
    fs::write(
        &rust_index,
        "export const onlyRust = 1; export const shared = 2;\n",
    )
    .expect("write Rust mock index");
    fs::write(
        &typescript_index,
        "export const onlyTypescript = 1; export const shared = 2;\n",
    )
    .expect("write TypeScript mock index");

    let output = Command::new("node")
        .arg("scripts/verify-public-api-parity.mjs")
        .arg("--rust-index")
        .arg(&rust_index)
        .arg("--typescript-index")
        .arg(&typescript_index)
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .output()
        .expect("run public API parity verifier");

    let _ = fs::remove_dir_all(&temp);

    assert!(
        !output.status.success(),
        "mismatched public API exports should fail\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        String::from_utf8_lossy(&output.stderr).contains("extra Rust exports")
            && String::from_utf8_lossy(&output.stderr).contains("missing Rust exports"),
        "failure should identify both extra and missing Rust exports\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn public_api_parity_verifier_should_reject_runtime_export_kind_mismatch() {
    if !node_available() {
        return;
    }

    let temp = temp_dir("rust-calckernel-public-api-kind-parity");
    fs::create_dir_all(&temp).expect("create temp dir");
    let rust_index = temp.join("rust-index.mjs");
    let typescript_index = temp.join("typescript-index.mjs");
    fs::write(&rust_index, "export const shared = 1;\n").expect("write Rust mock index");
    fs::write(&typescript_index, "export function shared() {}\n")
        .expect("write TypeScript mock index");

    let output = Command::new("node")
        .arg("scripts/verify-public-api-parity.mjs")
        .arg("--rust-index")
        .arg(&rust_index)
        .arg("--typescript-index")
        .arg(&typescript_index)
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .output()
        .expect("run public API parity verifier");

    let _ = fs::remove_dir_all(&temp);

    assert!(
        !output.status.success(),
        "mismatched public API export kinds should fail\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        String::from_utf8_lossy(&output.stderr).contains("export kind mismatch for shared"),
        "failure should identify the mismatched export kind\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn public_api_parity_verifier_should_reject_runtime_object_property_mismatch() {
    if !node_available() {
        return;
    }

    let temp = temp_dir("rust-calckernel-public-api-object-parity");
    fs::create_dir_all(&temp).expect("create temp dir");
    let rust_index = temp.join("rust-index.mjs");
    let typescript_index = temp.join("typescript-index.mjs");
    fs::write(
        &rust_index,
        "export const shared = { Present: \"Present\" };\n",
    )
    .expect("write Rust mock index");
    fs::write(
        &typescript_index,
        "export const shared = { Present: \"Present\", Missing: \"Missing\" };\n",
    )
    .expect("write TypeScript mock index");

    let output = Command::new("node")
        .arg("scripts/verify-public-api-parity.mjs")
        .arg("--rust-index")
        .arg(&rust_index)
        .arg("--typescript-index")
        .arg(&typescript_index)
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .output()
        .expect("run public API parity verifier");

    let _ = fs::remove_dir_all(&temp);

    assert!(
        !output.status.success(),
        "mismatched runtime object properties should fail\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        String::from_utf8_lossy(&output.stderr)
            .contains("runtime object property mismatch for shared"),
        "failure should identify the mismatched runtime object property\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn public_api_parity_verifier_should_reject_non_enumerable_runtime_object_property_mismatch() {
    if !node_available() {
        return;
    }

    let temp = temp_dir("rust-calckernel-public-api-object-own-property-parity");
    fs::create_dir_all(&temp).expect("create temp dir");
    let rust_index = temp.join("rust-index.mjs");
    let typescript_index = temp.join("typescript-index.mjs");
    fs::write(
        &rust_index,
        "export const shared = { Present: \"Present\" };\n",
    )
    .expect("write Rust mock index");
    fs::write(
        &typescript_index,
        [
            "const shared = { Present: \"Present\" };",
            "Object.defineProperty(shared, \"Hidden\", { value: \"Hidden\", enumerable: false, configurable: true, writable: true });",
            "export { shared };",
            "",
        ]
        .join("\n"),
    )
    .expect("write TypeScript mock index");

    let output = Command::new("node")
        .arg("scripts/verify-public-api-parity.mjs")
        .arg("--rust-index")
        .arg(&rust_index)
        .arg("--typescript-index")
        .arg(&typescript_index)
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .output()
        .expect("run public API parity verifier");

    let _ = fs::remove_dir_all(&temp);

    assert!(
        !output.status.success(),
        "mismatched runtime object own properties should fail\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        String::from_utf8_lossy(&output.stderr)
            .contains("runtime object property mismatch for shared"),
        "failure should identify the mismatched runtime object own property\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn public_api_parity_verifier_should_reject_runtime_object_metadata_mismatch() {
    if !node_available() {
        return;
    }

    let temp = temp_dir("rust-calckernel-public-api-object-metadata-parity");
    fs::create_dir_all(&temp).expect("create temp dir");
    let rust_index = temp.join("rust-index.mjs");
    let typescript_index = temp.join("typescript-index.mjs");
    fs::write(&rust_index, "export const shared = {};\n").expect("write Rust mock index");
    fs::write(
        &typescript_index,
        [
            "const shared = {};",
            "Object.preventExtensions(shared);",
            "export { shared };",
            "",
        ]
        .join("\n"),
    )
    .expect("write TypeScript mock index");

    let output = Command::new("node")
        .arg("scripts/verify-public-api-parity.mjs")
        .arg("--rust-index")
        .arg(&rust_index)
        .arg("--typescript-index")
        .arg(&typescript_index)
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .output()
        .expect("run public API parity verifier");

    let _ = fs::remove_dir_all(&temp);

    assert!(
        !output.status.success(),
        "mismatched runtime object metadata should fail\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        String::from_utf8_lossy(&output.stderr)
            .contains("runtime object metadata mismatch for shared"),
        "failure should identify the mismatched runtime object metadata\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn public_api_parity_verifier_should_reject_runtime_object_property_descriptor_mismatch() {
    if !node_available() {
        return;
    }

    let temp = temp_dir("rust-calckernel-public-api-object-descriptor-parity");
    fs::create_dir_all(&temp).expect("create temp dir");
    let rust_index = temp.join("rust-index.mjs");
    let typescript_index = temp.join("typescript-index.mjs");
    fs::write(
        &rust_index,
        [
            "const shared = {};",
            "Object.defineProperty(shared, \"Present\", { value: \"Present\", enumerable: true, configurable: true, writable: true });",
            "export { shared };",
            "",
        ]
        .join("\n"),
    )
    .expect("write Rust mock index");
    fs::write(
        &typescript_index,
        [
            "const shared = {};",
            "Object.defineProperty(shared, \"Present\", { value: \"Present\", enumerable: true, configurable: true, writable: false });",
            "export { shared };",
            "",
        ]
        .join("\n"),
    )
    .expect("write TypeScript mock index");

    let output = Command::new("node")
        .arg("scripts/verify-public-api-parity.mjs")
        .arg("--rust-index")
        .arg(&rust_index)
        .arg("--typescript-index")
        .arg(&typescript_index)
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .output()
        .expect("run public API parity verifier");

    let _ = fs::remove_dir_all(&temp);

    assert!(
        !output.status.success(),
        "mismatched runtime object property descriptors should fail\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        String::from_utf8_lossy(&output.stderr)
            .contains("runtime object property mismatch for shared"),
        "failure should identify the mismatched runtime object property descriptor\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn public_api_parity_verifier_should_reject_runtime_property_null_kind_mismatch() {
    if !node_available() {
        return;
    }

    let temp = temp_dir("rust-calckernel-public-api-object-null-kind-parity");
    fs::create_dir_all(&temp).expect("create temp dir");
    let rust_index = temp.join("rust-index.mjs");
    let typescript_index = temp.join("typescript-index.mjs");
    fs::write(&rust_index, "export const shared = { value: null };\n")
        .expect("write Rust mock index");
    fs::write(&typescript_index, "export const shared = { value: {} };\n")
        .expect("write TypeScript mock index");

    let output = Command::new("node")
        .arg("scripts/verify-public-api-parity.mjs")
        .arg("--rust-index")
        .arg(&rust_index)
        .arg("--typescript-index")
        .arg(&typescript_index)
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .output()
        .expect("run public API parity verifier");

    let _ = fs::remove_dir_all(&temp);

    assert!(
        !output.status.success(),
        "mismatched runtime null/object property kinds should fail\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        String::from_utf8_lossy(&output.stderr)
            .contains("runtime object property mismatch for shared"),
        "failure should identify the mismatched runtime property kind\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn public_api_parity_verifier_should_reject_runtime_class_member_mismatch() {
    if !node_available() {
        return;
    }

    let temp = temp_dir("rust-calckernel-public-api-class-parity");
    fs::create_dir_all(&temp).expect("create temp dir");
    let rust_index = temp.join("rust-index.mjs");
    let typescript_index = temp.join("typescript-index.mjs");
    fs::write(
        &rust_index,
        "export class Shared { present() { return 1; } }\n",
    )
    .expect("write Rust mock index");
    fs::write(
        &typescript_index,
        "export class Shared { present() { return 1; } missing() { return 2; } }\n",
    )
    .expect("write TypeScript mock index");

    let output = Command::new("node")
        .arg("scripts/verify-public-api-parity.mjs")
        .arg("--rust-index")
        .arg(&rust_index)
        .arg("--typescript-index")
        .arg(&typescript_index)
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .output()
        .expect("run public API parity verifier");

    let _ = fs::remove_dir_all(&temp);

    assert!(
        !output.status.success(),
        "mismatched runtime class members should fail\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        String::from_utf8_lossy(&output.stderr)
            .contains("runtime class member mismatch for Shared"),
        "failure should identify the mismatched runtime class member\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn public_api_parity_verifier_should_reject_runtime_class_member_descriptor_mismatch() {
    if !node_available() {
        return;
    }

    let temp = temp_dir("rust-calckernel-public-api-class-descriptor-parity");
    fs::create_dir_all(&temp).expect("create temp dir");
    let rust_index = temp.join("rust-index.mjs");
    let typescript_index = temp.join("typescript-index.mjs");
    fs::write(
        &rust_index,
        "export class Shared { present(value) { return value; } }\n",
    )
    .expect("write Rust mock index");
    fs::write(
        &typescript_index,
        [
            "class Shared { present(value) { return value; } }",
            "Object.defineProperty(Shared.prototype, \"present\", { value: Shared.prototype.present, enumerable: true, configurable: true, writable: true });",
            "export { Shared };",
            "",
        ]
        .join("\n"),
    )
    .expect("write TypeScript mock index");

    let output = Command::new("node")
        .arg("scripts/verify-public-api-parity.mjs")
        .arg("--rust-index")
        .arg(&rust_index)
        .arg("--typescript-index")
        .arg(&typescript_index)
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .output()
        .expect("run public API parity verifier");

    let _ = fs::remove_dir_all(&temp);

    assert!(
        !output.status.success(),
        "mismatched runtime class member descriptors should fail\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        String::from_utf8_lossy(&output.stderr)
            .contains("runtime class member mismatch for Shared"),
        "failure should identify the mismatched runtime class member descriptor\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn public_api_parity_verifier_should_reject_runtime_class_constructor_metadata_mismatch() {
    if !node_available() {
        return;
    }

    let temp = temp_dir("rust-calckernel-public-api-class-constructor-parity");
    fs::create_dir_all(&temp).expect("create temp dir");
    let rust_index = temp.join("rust-index.mjs");
    let typescript_index = temp.join("typescript-index.mjs");
    fs::write(
        &rust_index,
        "export class Shared { constructor(value) { this.value = value; } }\n",
    )
    .expect("write Rust mock index");
    fs::write(
        &typescript_index,
        "export class Shared { constructor(left, right) { this.value = left + right; } }\n",
    )
    .expect("write TypeScript mock index");

    let output = Command::new("node")
        .arg("scripts/verify-public-api-parity.mjs")
        .arg("--rust-index")
        .arg(&rust_index)
        .arg("--typescript-index")
        .arg(&typescript_index)
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .output()
        .expect("run public API parity verifier");

    let _ = fs::remove_dir_all(&temp);

    assert!(
        !output.status.success(),
        "mismatched runtime class constructor metadata should fail\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        String::from_utf8_lossy(&output.stderr)
            .contains("runtime class constructor metadata mismatch for Shared"),
        "failure should identify the mismatched runtime class constructor metadata\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn public_api_parity_verifier_should_reject_runtime_class_object_metadata_mismatch() {
    if !node_available() {
        return;
    }

    let temp = temp_dir("rust-calckernel-public-api-class-object-metadata-parity");
    fs::create_dir_all(&temp).expect("create temp dir");
    let rust_index = temp.join("rust-index.mjs");
    let typescript_index = temp.join("typescript-index.mjs");
    fs::write(&rust_index, "export class Shared {}\n").expect("write Rust mock index");
    fs::write(
        &typescript_index,
        [
            "class Shared {}",
            "Object.preventExtensions(Shared.prototype);",
            "export { Shared };",
            "",
        ]
        .join("\n"),
    )
    .expect("write TypeScript mock index");

    let output = Command::new("node")
        .arg("scripts/verify-public-api-parity.mjs")
        .arg("--rust-index")
        .arg(&rust_index)
        .arg("--typescript-index")
        .arg(&typescript_index)
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .output()
        .expect("run public API parity verifier");

    let _ = fs::remove_dir_all(&temp);

    assert!(
        !output.status.success(),
        "mismatched runtime class object metadata should fail\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        String::from_utf8_lossy(&output.stderr)
            .contains("runtime class object metadata mismatch for Shared"),
        "failure should identify the mismatched runtime class object metadata\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn public_api_parity_verifier_should_reject_runtime_class_instance_property_mismatch() {
    if !node_available() {
        return;
    }

    let temp = temp_dir("rust-calckernel-public-api-class-instance-property-parity");
    fs::create_dir_all(&temp).expect("create temp dir");
    let rust_index = temp.join("rust-index.mjs");
    let typescript_index = temp.join("typescript-index.mjs");
    fs::write(
        &rust_index,
        "export class Shared { constructor() { this.present = 1; } }\n",
    )
    .expect("write Rust mock index");
    fs::write(
        &typescript_index,
        "export class Shared { constructor() { this.present = 1; this.missing = 2; } }\n",
    )
    .expect("write TypeScript mock index");

    let output = Command::new("node")
        .arg("scripts/verify-public-api-parity.mjs")
        .arg("--rust-index")
        .arg(&rust_index)
        .arg("--typescript-index")
        .arg(&typescript_index)
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .output()
        .expect("run public API parity verifier");

    let _ = fs::remove_dir_all(&temp);

    assert!(
        !output.status.success(),
        "mismatched runtime class instance properties should fail\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        String::from_utf8_lossy(&output.stderr)
            .contains("runtime class instance property mismatch for Shared"),
        "failure should identify the mismatched runtime class instance property\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn public_api_parity_verifier_should_reject_runtime_class_instance_metadata_mismatch() {
    if !node_available() {
        return;
    }

    let temp = temp_dir("rust-calckernel-public-api-class-instance-metadata-parity");
    fs::create_dir_all(&temp).expect("create temp dir");
    let rust_index = temp.join("rust-index.mjs");
    let typescript_index = temp.join("typescript-index.mjs");
    fs::write(&rust_index, "export class Shared { constructor() {} }\n")
        .expect("write Rust mock index");
    fs::write(
        &typescript_index,
        "export class Shared { constructor() { Object.preventExtensions(this); } }\n",
    )
    .expect("write TypeScript mock index");

    let output = Command::new("node")
        .arg("scripts/verify-public-api-parity.mjs")
        .arg("--rust-index")
        .arg(&rust_index)
        .arg("--typescript-index")
        .arg(&typescript_index)
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .output()
        .expect("run public API parity verifier");

    let _ = fs::remove_dir_all(&temp);

    assert!(
        !output.status.success(),
        "mismatched runtime class instance metadata should fail\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        String::from_utf8_lossy(&output.stderr)
            .contains("runtime class instance metadata mismatch for Shared"),
        "failure should identify the mismatched runtime class instance metadata\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn public_api_parity_verifier_should_reject_runtime_class_instance_property_descriptor_mismatch() {
    if !node_available() {
        return;
    }

    let temp = temp_dir("rust-calckernel-public-api-class-instance-descriptor-parity");
    fs::create_dir_all(&temp).expect("create temp dir");
    let rust_index = temp.join("rust-index.mjs");
    let typescript_index = temp.join("typescript-index.mjs");
    fs::write(
        &rust_index,
        [
            "export class Shared {",
            "  constructor() {",
            "    Object.defineProperty(this, \"present\", { value: 1, enumerable: true, configurable: true, writable: true });",
            "  }",
            "}",
            "",
        ]
        .join("\n"),
    )
    .expect("write Rust mock index");
    fs::write(
        &typescript_index,
        [
            "export class Shared {",
            "  constructor() {",
            "    Object.defineProperty(this, \"present\", { value: 1, enumerable: true, configurable: true, writable: false });",
            "  }",
            "}",
            "",
        ]
        .join("\n"),
    )
    .expect("write TypeScript mock index");

    let output = Command::new("node")
        .arg("scripts/verify-public-api-parity.mjs")
        .arg("--rust-index")
        .arg(&rust_index)
        .arg("--typescript-index")
        .arg(&typescript_index)
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .output()
        .expect("run public API parity verifier");

    let _ = fs::remove_dir_all(&temp);

    assert!(
        !output.status.success(),
        "mismatched runtime class instance property descriptors should fail\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        String::from_utf8_lossy(&output.stderr)
            .contains("runtime class instance property mismatch for Shared"),
        "failure should identify the mismatched runtime class instance property descriptor\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn public_api_parity_verifier_should_reject_runtime_class_member_metadata_mismatch() {
    if !node_available() {
        return;
    }

    let temp = temp_dir("rust-calckernel-public-api-class-member-metadata-parity");
    fs::create_dir_all(&temp).expect("create temp dir");
    let rust_index = temp.join("rust-index.mjs");
    let typescript_index = temp.join("typescript-index.mjs");
    fs::write(
        &rust_index,
        "export class Shared { present(value) { return value; } }\n",
    )
    .expect("write Rust mock index");
    fs::write(
        &typescript_index,
        "export class Shared { present(left, right) { return left + right; } }\n",
    )
    .expect("write TypeScript mock index");

    let output = Command::new("node")
        .arg("scripts/verify-public-api-parity.mjs")
        .arg("--rust-index")
        .arg(&rust_index)
        .arg("--typescript-index")
        .arg(&typescript_index)
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .output()
        .expect("run public API parity verifier");

    let _ = fs::remove_dir_all(&temp);

    assert!(
        !output.status.success(),
        "mismatched runtime class member metadata should fail\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        String::from_utf8_lossy(&output.stderr)
            .contains("runtime class member mismatch for Shared"),
        "failure should identify the mismatched runtime class member metadata\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn public_api_parity_verifier_should_reject_runtime_function_metadata_mismatch() {
    if !node_available() {
        return;
    }

    let temp = temp_dir("rust-calckernel-public-api-function-parity");
    fs::create_dir_all(&temp).expect("create temp dir");
    let rust_index = temp.join("rust-index.mjs");
    let typescript_index = temp.join("typescript-index.mjs");
    fs::write(
        &rust_index,
        "export function shared(value) { return value; }\n",
    )
    .expect("write Rust mock index");
    fs::write(
        &typescript_index,
        "export function shared(left, right) { return left + right; }\n",
    )
    .expect("write TypeScript mock index");

    let output = Command::new("node")
        .arg("scripts/verify-public-api-parity.mjs")
        .arg("--rust-index")
        .arg(&rust_index)
        .arg("--typescript-index")
        .arg(&typescript_index)
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .output()
        .expect("run public API parity verifier");

    let _ = fs::remove_dir_all(&temp);

    assert!(
        !output.status.success(),
        "mismatched runtime function metadata should fail\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        String::from_utf8_lossy(&output.stderr)
            .contains("runtime function metadata mismatch for shared"),
        "failure should identify the mismatched runtime function metadata\nstdout:\n{}\nstderr:\n{}",
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
