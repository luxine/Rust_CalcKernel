use std::{
    ffi::OsString,
    fs,
    path::PathBuf,
    process::Command,
    time::{SystemTime, UNIX_EPOCH},
};

#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;

#[test]
fn cli_should_check_and_emit_core_outputs() {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system time")
        .as_nanos();
    let dir = std::env::temp_dir().join(format!("rust_calckernel_cli_{unique}"));
    fs::create_dir_all(&dir).expect("create temp dir");
    let source = dir.join("sample.ck");
    fs::write(
        &source,
        r#"
          export fn add_i64(a: i64, b: i64) -> i64 {
            return a + b;
          }
        "#,
    )
    .expect("write source");

    let bin = env!("CARGO_BIN_EXE_ckc");

    let check = Command::new(bin)
        .arg("check")
        .arg(&source)
        .output()
        .expect("run check");
    assert!(check.status.success());
    assert!(String::from_utf8_lossy(&check.stdout).contains("OK:"));

    let mir = Command::new(bin)
        .arg("emit-mir")
        .arg(&source)
        .output()
        .expect("run emit-mir");
    assert!(mir.status.success());
    assert!(String::from_utf8_lossy(&mir.stdout).contains("export fn add_i64"));

    let c_path = dir.join("sample.c");
    let emit_c = Command::new(bin)
        .arg("emit-c")
        .arg("--out")
        .arg(&c_path)
        .arg(&source)
        .output()
        .expect("run emit-c");
    assert!(emit_c.status.success());
    assert!(
        fs::read_to_string(&c_path)
            .expect("read c")
            .contains("int64_t add_i64")
    );
    assert!(
        fs::read_to_string(dir.join("sample.h"))
            .expect("read default header")
            .contains("CK_API int64_t add_i64")
    );

    let checked_c_path = dir.join("checked.c");
    let checked_header_path = dir.join("api.h");
    let emit_checked_c = Command::new(bin)
        .arg("emit-c")
        .arg("--overflow")
        .arg("checked")
        .arg("--out")
        .arg(&checked_c_path)
        .arg("--header")
        .arg(&checked_header_path)
        .arg(&source)
        .output()
        .expect("run checked emit-c");
    assert!(
        emit_checked_c.status.success(),
        "stderr:\n{}",
        String::from_utf8_lossy(&emit_checked_c.stderr)
    );
    assert!(
        fs::read_to_string(&checked_c_path)
            .expect("read checked c")
            .contains("#include \"api.h\"")
    );
    assert!(
        fs::read_to_string(&checked_header_path)
            .expect("read checked header")
            .contains("CK_API CK_Status add_i64")
    );

    let wat = Command::new(bin)
        .arg("emit-wat")
        .arg(&source)
        .output()
        .expect("run emit-wat");
    assert!(wat.status.success());
    assert!(String::from_utf8_lossy(&wat.stdout).contains("(func $add_i64"));

    let llvm = Command::new(bin)
        .arg("emit-llvm")
        .arg(&source)
        .output()
        .expect("run emit-llvm");
    assert!(llvm.status.success());
    assert!(String::from_utf8_lossy(&llvm.stdout).contains("define i64 @add_i64"));

    let wasm_path = dir.join("sample.wasm");
    let wasm = Command::new(bin)
        .arg("emit-wasm")
        .arg("--out")
        .arg(&wasm_path)
        .arg(&source)
        .output()
        .expect("run emit-wasm");
    assert!(
        wasm.status.success(),
        "stderr:\n{}",
        String::from_utf8_lossy(&wasm.stderr)
    );
    let wasm_bytes = fs::read(&wasm_path).expect("read wasm");
    assert_eq!(&wasm_bytes[..4], b"\0asm");
    assert_eq!(&wasm_bytes[4..8], &[1, 0, 0, 0]);
}

#[test]
fn cli_should_build_native_c_library_and_llvm_object() {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system time")
        .as_nanos();
    let dir = std::env::temp_dir().join(format!("rust_calckernel_cli_build_{unique}"));
    fs::create_dir_all(&dir).expect("create temp dir");
    let source = dir.join("sample.ck");
    fs::write(
        &source,
        "export fn add_i64(a: i64, b: i64) -> i64 { return a + b; }",
    )
    .expect("write source");

    let bin = env!("CARGO_BIN_EXE_ckc");

    let library_base = dir.join("libsample");
    let build = Command::new(bin)
        .arg("build")
        .arg("--out")
        .arg(&library_base)
        .arg(&source)
        .output()
        .expect("run build");
    assert!(
        build.status.success(),
        "stderr:\n{}",
        String::from_utf8_lossy(&build.stderr)
    );
    assert!(shared_library_path(&library_base).exists());
    assert!(library_base.with_extension("c").exists());
    assert!(library_base.with_extension("h").exists());

    let checked_library_base = dir.join("libsample_checked");
    let build_checked = Command::new(bin)
        .arg("build")
        .arg("--overflow")
        .arg("checked")
        .arg("--out")
        .arg(&checked_library_base)
        .arg(&source)
        .output()
        .expect("run checked build");
    assert!(
        build_checked.status.success(),
        "stderr:\n{}",
        String::from_utf8_lossy(&build_checked.stderr)
    );
    assert!(shared_library_path(&checked_library_base).exists());
    assert!(checked_library_base.with_extension("h").exists());
    assert!(
        fs::read_to_string(checked_library_base.with_extension("c"))
            .expect("read checked c")
            .contains("#include \"libsample_checked.h\"")
    );
    assert!(
        fs::read_to_string(checked_library_base.with_extension("h"))
            .expect("read checked header")
            .contains("CK_Status add_i64")
    );

    let object_path = dir.join("sample.o");
    let build_llvm = Command::new(bin)
        .arg("build-llvm")
        .arg("--kind")
        .arg("object")
        .arg("--out")
        .arg(&object_path)
        .arg(&source)
        .output()
        .expect("run build-llvm");
    assert!(
        build_llvm.status.success(),
        "stderr:\n{}",
        String::from_utf8_lossy(&build_llvm.stderr)
    );
    assert!(object_path.exists());
    assert!(object_path.with_extension("ll").exists());
}

#[test]
fn emit_c_should_not_leave_c_output_when_header_path_cannot_be_created() {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system time")
        .as_nanos();
    let dir = std::env::temp_dir().join(format!("rust_calckernel_emit_c_failure_{unique}"));
    fs::create_dir_all(&dir).expect("create temp dir");
    let source = dir.join("sample.ck");
    fs::write(
        &source,
        "export fn add_i64(a: i64, b: i64) -> i64 { return a + b; }",
    )
    .expect("write source");
    let c_path = dir.join("build").join("sample.c");
    let header_parent = dir.join("headers");
    fs::write(&header_parent, "not a directory").expect("write header parent file");
    let header_path = header_parent.join("sample.h");

    let output = Command::new(env!("CARGO_BIN_EXE_ckc"))
        .arg("emit-c")
        .arg("--out")
        .arg(&c_path)
        .arg("--header")
        .arg(&header_path)
        .arg(&source)
        .output()
        .expect("run emit-c with invalid header path");
    assert!(!output.status.success());
    assert!(
        !c_path.exists(),
        "emit-c must not leave C output when header creation fails\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(!header_path.exists());
}

#[cfg(unix)]
#[test]
fn build_should_pass_typescript_compatible_ck_build_dll_define_to_clang() {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system time")
        .as_nanos();
    let dir = std::env::temp_dir().join(format!("rust_calckernel_fake_clang_{unique}"));
    fs::create_dir_all(&dir).expect("create temp dir");
    let fake_bin_dir = dir.join("bin");
    fs::create_dir_all(&fake_bin_dir).expect("create fake bin dir");
    let log_path = dir.join("clang-args.log");
    let fake_clang = fake_bin_dir.join("clang");
    fs::write(
        &fake_clang,
        r#"#!/bin/sh
printf '%s\n' "$*" >> "$CKC_CLANG_LOG"
if [ "$1" = "--version" ]; then
  echo "fake clang"
  exit 0
fi
out=""
previous=""
for arg in "$@"; do
  if [ "$previous" = "-o" ]; then
    out="$arg"
  fi
  previous="$arg"
done
if [ -n "$out" ]; then
  mkdir -p "$(dirname "$out")"
  : > "$out"
fi
exit 0
"#,
    )
    .expect("write fake clang");
    fs::set_permissions(&fake_clang, fs::Permissions::from_mode(0o755)).expect("chmod fake clang");

    let source = dir.join("sample.ck");
    fs::write(
        &source,
        "export fn add_i64(a: i64, b: i64) -> i64 { return a + b; }",
    )
    .expect("write source");
    let original_path = std::env::var_os("PATH").unwrap_or_default();
    let mut path_entries = vec![fake_bin_dir];
    path_entries.extend(std::env::split_paths(&original_path));
    let path = std::env::join_paths(path_entries).expect("join PATH");

    let library_base = dir.join("libsample");
    let output = Command::new(env!("CARGO_BIN_EXE_ckc"))
        .env("PATH", path)
        .env("CKC_CLANG_LOG", &log_path)
        .arg("build")
        .arg("--out")
        .arg(&library_base)
        .arg(&source)
        .output()
        .expect("run build with fake clang");
    assert!(
        output.status.success(),
        "stdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let log = fs::read_to_string(&log_path).expect("read clang args log");
    let compile_args = log
        .lines()
        .find(|line| line.contains("-shared"))
        .unwrap_or_else(|| panic!("missing compile clang invocation:\n{log}"));
    assert!(
        compile_args.contains("-DCK_BUILD_DLL"),
        "clang invocation must match TypeScript build macro contract:\n{log}"
    );
}

#[test]
fn build_llvm_should_print_typescript_compatible_missing_clang_message() {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system time")
        .as_nanos();
    let dir = std::env::temp_dir().join(format!("rust_calckernel_missing_clang_{unique}"));
    fs::create_dir_all(&dir).expect("create temp dir");
    let empty_path = dir.join("empty_path");
    fs::create_dir_all(&empty_path).expect("create empty PATH dir");
    let source = dir.join("sample.ck");
    fs::write(
        &source,
        "export fn add_i64(a: i64, b: i64) -> i64 { return a + b; }",
    )
    .expect("write source");

    let output = Command::new(env!("CARGO_BIN_EXE_ckc"))
        .arg("build-llvm")
        .arg("--out")
        .arg(dir.join("libsample"))
        .arg(&source)
        .env("PATH", &empty_path)
        .output()
        .expect("run build-llvm");

    assert!(!output.status.success());
    assert_eq!(String::from_utf8_lossy(&output.stdout), "");
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr
            .contains("clang was not found. Install clang and make sure it is available on PATH.")
    );
    assert!(stderr.contains("You can still run emit-llvm to generate LLVM IR without clang."));
}

#[test]
fn build_should_match_typescript_oracle_for_generated_c_header_and_stdout() {
    let Some(ts_cli) = typescript_cli() else {
        return;
    };
    if !clang_available() {
        return;
    }
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system time")
        .as_nanos();
    let dir = std::env::temp_dir().join(format!("rust_calckernel_build_oracle_{unique}"));
    fs::create_dir_all(&dir).expect("create temp dir");
    let source = typescript_root().join("examples/scalar.ck");

    for overflow in ["unchecked", "checked"] {
        let output_base = dir.join(format!("libscalar_{overflow}"));
        let args = vec![
            os("build"),
            os("--out"),
            output_base.clone().into_os_string(),
            os("--overflow"),
            os(overflow),
            source.clone().into_os_string(),
        ];

        let ts = run_typescript_cli(&ts_cli, &args);
        assert_eq!(ts.status_code, Some(0), "TS stderr: {}", ts.stderr);
        let ts_c = fs::read_to_string(format!("{}.c", output_base.display())).expect("read TS C");
        let ts_h = fs::read_to_string(format!("{}.h", output_base.display())).expect("read TS H");

        let rust = run_rust_cli(&args);
        assert_eq!(rust.status_code, ts.status_code, "overflow={overflow}");
        assert_eq!(rust.stdout, ts.stdout, "overflow={overflow} stdout");
        assert_eq!(rust.stderr, ts.stderr, "overflow={overflow} stderr");
        assert_eq!(
            fs::read_to_string(format!("{}.c", output_base.display())).expect("read Rust C"),
            ts_c,
            "overflow={overflow} C"
        );
        assert_eq!(
            fs::read_to_string(format!("{}.h", output_base.display())).expect("read Rust H"),
            ts_h,
            "overflow={overflow} H"
        );
    }
}

#[test]
fn build_should_match_typescript_oracle_for_pricing_dynamic_library_runtime_behavior() {
    let Some(ts_cli) = typescript_cli() else {
        return;
    };
    if !clang_available() || !python3_available() {
        return;
    }
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system time")
        .as_nanos();
    let dir = std::env::temp_dir().join(format!("rust_calckernel_build_runtime_{unique}"));
    fs::create_dir_all(&dir).expect("create temp dir");
    let runner = dir.join("run_pricing_dynamic.py");
    fs::write(&runner, pricing_dynamic_library_runner()).expect("write dynamic library runner");
    let source = typescript_root().join("examples/pricing.ck");

    for overflow in ["unchecked", "checked"] {
        let ts_base = dir.join(format!("ts_pricing_{overflow}"));
        let rust_base = dir.join(format!("rust_pricing_{overflow}"));
        let ts_args = vec![
            os("build"),
            os("--out"),
            ts_base.clone().into_os_string(),
            os("--overflow"),
            os(overflow),
            source.clone().into_os_string(),
        ];
        let rust_args = vec![
            os("build"),
            os("--out"),
            rust_base.clone().into_os_string(),
            os("--overflow"),
            os(overflow),
            source.clone().into_os_string(),
        ];

        let ts_build = run_typescript_cli(&ts_cli, &ts_args);
        assert_eq!(
            ts_build.status_code,
            Some(0),
            "{overflow} TS stderr: {}",
            ts_build.stderr
        );
        let rust_build = run_rust_cli(&rust_args);
        assert_eq!(
            rust_build.status_code, ts_build.status_code,
            "{overflow} build status"
        );

        let mode = if overflow == "checked" {
            "checked"
        } else {
            "unchecked"
        };
        let ts_run =
            run_python_pricing_dynamic_library(&runner, mode, &shared_library_path(&ts_base));
        let rust_run =
            run_python_pricing_dynamic_library(&runner, mode, &shared_library_path(&rust_base));

        assert_eq!(
            rust_run.status_code, ts_run.status_code,
            "{overflow} runtime status"
        );
        assert_eq!(rust_run.stdout, ts_run.stdout, "{overflow} runtime stdout");
        assert_eq!(rust_run.stderr, ts_run.stderr, "{overflow} runtime stderr");
    }
}

#[test]
fn build_should_match_typescript_oracle_for_official_c_dynamic_library_runtime_behavior() {
    let Some(ts_cli) = typescript_cli() else {
        return;
    };
    if !clang_available() || !python3_available() {
        return;
    }
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system time")
        .as_nanos();
    let dir =
        std::env::temp_dir().join(format!("rust_calckernel_build_official_c_runtime_{unique}"));
    fs::create_dir_all(&dir).expect("create temp dir");
    let runner = dir.join("run_official_c_dynamic.py");
    fs::write(&runner, official_c_dynamic_library_runner())
        .expect("write official C dynamic library runner");
    let cases = [
        ("c-scalar", "unchecked", "examples/scalar.ck"),
        ("c-casts", "unchecked", "examples/explicit_casts.ck"),
        ("c-scalar-checked", "checked", "examples/scalar_checked.ck"),
        (
            "c-control-checked",
            "checked",
            "examples/scalar_control_checked.ck",
        ),
        (
            "c-logical-checked",
            "checked",
            "examples/scalar_logical_checked.ck",
        ),
        (
            "c-calls-checked",
            "checked",
            "examples/scalar_calls_checked.ck",
        ),
    ];

    for (case_name, overflow, example) in cases {
        let source = typescript_root().join(example);
        let ts_base = dir.join(format!("ts_{case_name}"));
        let rust_base = dir.join(format!("rust_{case_name}"));
        let ts_args = vec![
            os("build"),
            os("--out"),
            ts_base.clone().into_os_string(),
            os("--overflow"),
            os(overflow),
            source.clone().into_os_string(),
        ];
        let rust_args = vec![
            os("build"),
            os("--out"),
            rust_base.clone().into_os_string(),
            os("--overflow"),
            os(overflow),
            source.clone().into_os_string(),
        ];

        let ts_build = run_typescript_cli(&ts_cli, &ts_args);
        assert_eq!(
            ts_build.status_code,
            Some(0),
            "{case_name} TS stderr: {}",
            ts_build.stderr
        );
        let rust_build = run_rust_cli(&rust_args);
        assert_eq!(
            rust_build.status_code, ts_build.status_code,
            "{case_name} build status"
        );

        let ts_run = run_python_official_c_dynamic_library(
            &runner,
            case_name,
            &shared_library_path(&ts_base),
        );
        let rust_run = run_python_official_c_dynamic_library(
            &runner,
            case_name,
            &shared_library_path(&rust_base),
        );

        assert_eq!(
            ts_run.status_code,
            Some(0),
            "{case_name} TS runtime stderr: {}",
            ts_run.stderr
        );
        assert_eq!(
            rust_run.status_code,
            Some(0),
            "{case_name} Rust runtime stderr: {}",
            rust_run.stderr
        );
        assert_eq!(
            rust_run.status_code, ts_run.status_code,
            "{case_name} runtime status"
        );
        assert_eq!(rust_run.stdout, ts_run.stdout, "{case_name} runtime stdout");
        assert_eq!(rust_run.stderr, ts_run.stderr, "{case_name} runtime stderr");
    }
}

#[test]
fn build_should_match_typescript_oracle_for_perf_c_dynamic_library_runtime_behavior() {
    let Some(ts_cli) = typescript_cli() else {
        return;
    };
    if !clang_available() || !python3_available() {
        return;
    }
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system time")
        .as_nanos();
    let dir = std::env::temp_dir().join(format!("rust_calckernel_build_perf_c_runtime_{unique}"));
    fs::create_dir_all(&dir).expect("create temp dir");
    let runner = dir.join("run_perf_c_dynamic.py");
    fs::write(&runner, official_c_dynamic_library_runner())
        .expect("write perf C dynamic library runner");
    let cases = [
        (
            "c-perf-pricing-helpers-o0",
            "bench/perf/fixtures/pricing_helpers.ck",
            "-O0",
        ),
        (
            "c-perf-pricing-helpers-o2",
            "bench/perf/fixtures/pricing_helpers.ck",
            "-O2",
        ),
        (
            "c-perf-pricing-soa-o3",
            "bench/perf/fixtures/pricing_soa.ck",
            "-O3",
        ),
        (
            "c-perf-f64-kernels-o3",
            "bench/perf/fixtures/f64_kernels.ck",
            "-O3",
        ),
    ];

    for (case_name, fixture, opt_level) in cases {
        let source = typescript_root().join(fixture);
        let ts_base = dir.join(format!("ts_{case_name}"));
        let rust_base = dir.join(format!("rust_{case_name}"));
        let ts_args = vec![
            os("build"),
            os("--out"),
            ts_base.clone().into_os_string(),
            os("--overflow"),
            os("unchecked"),
            os(opt_level),
            source.clone().into_os_string(),
        ];
        let rust_args = vec![
            os("build"),
            os("--out"),
            rust_base.clone().into_os_string(),
            os("--overflow"),
            os("unchecked"),
            os(opt_level),
            source.clone().into_os_string(),
        ];

        let ts_build = run_typescript_cli(&ts_cli, &ts_args);
        assert_eq!(
            ts_build.status_code,
            Some(0),
            "{case_name} TS stderr: {}",
            ts_build.stderr
        );
        let rust_build = run_rust_cli(&rust_args);
        assert_eq!(
            rust_build.status_code, ts_build.status_code,
            "{case_name} build status"
        );

        let ts_run = run_python_official_c_dynamic_library(
            &runner,
            case_name,
            &shared_library_path(&ts_base),
        );
        let rust_run = run_python_official_c_dynamic_library(
            &runner,
            case_name,
            &shared_library_path(&rust_base),
        );

        assert_eq!(
            ts_run.status_code,
            Some(0),
            "{case_name} TS runtime stderr: {}",
            ts_run.stderr
        );
        assert_eq!(
            rust_run.status_code,
            Some(0),
            "{case_name} Rust runtime stderr: {}",
            rust_run.stderr
        );
        assert_eq!(
            rust_run.status_code, ts_run.status_code,
            "{case_name} runtime status"
        );
        assert_eq!(rust_run.stdout, ts_run.stdout, "{case_name} runtime stdout");
        assert_eq!(rust_run.stderr, ts_run.stderr, "{case_name} runtime stderr");
    }
}

#[test]
fn cli_should_print_mir_optimization_debug_output_to_stderr() {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system time")
        .as_nanos();
    let dir = std::env::temp_dir().join(format!("rust_calckernel_cli_debug_{unique}"));
    fs::create_dir_all(&dir).expect("create temp dir");
    let source = dir.join("sample.ck");
    fs::write(
        &source,
        "export fn add_i64(a: i64, b: i64) -> i64 { return a + b; }",
    )
    .expect("write source");

    let output = Command::new(env!("CARGO_BIN_EXE_ckc"))
        .arg("emit-mir")
        .arg("-O3")
        .arg("--print-pass-pipeline")
        .arg("--print-mir-before-opt")
        .arg("--print-mir-after-opt")
        .arg(&source)
        .output()
        .expect("run emit-mir debug");
    assert!(
        output.status.success(),
        "stderr:\n{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stdout.contains("export fn add_i64"));
    assert!(stderr.contains("MIR pass pipeline: O3:"));
    assert!(stderr.contains("MIR before optimization:"));
    assert!(stderr.contains("MIR after optimization:"));
    assert!(stderr.contains("%t0: i64 = add a, b"));
}

#[test]
fn cli_should_print_c_emission_pass_pipeline_to_stderr() {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system time")
        .as_nanos();
    let dir = std::env::temp_dir().join(format!("rust_calckernel_cli_c_debug_{unique}"));
    fs::create_dir_all(&dir).expect("create temp dir");
    let source = dir.join("sample.ck");
    fs::write(
        &source,
        "export fn add_i64(a: i64, b: i64) -> i64 { return a + b; }",
    )
    .expect("write source");
    let c_path = dir.join("sample.c");

    let output = Command::new(env!("CARGO_BIN_EXE_ckc"))
        .arg("emit-c")
        .arg("-O3")
        .arg("--print-pass-pipeline")
        .arg("--out")
        .arg(&c_path)
        .arg(&source)
        .output()
        .expect("run emit-c debug");
    assert!(
        output.status.success(),
        "stderr:\n{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stdout.contains("OK: emitted C with overflow=unchecked"));
    assert!(stderr.starts_with("MIR pass pipeline: O3:"));
    assert!(stderr.contains("loop-invariant-code-motion"));
}

#[test]
fn cli_should_match_typescript_oracle_for_public_error_cases() {
    let Some(ts_cli) = typescript_cli() else {
        return;
    };
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system time")
        .as_nanos();
    let dir = std::env::temp_dir().join(format!("rust_calckernel_cli_oracle_{unique}"));
    fs::create_dir_all(&dir).expect("create temp dir");
    let source = dir.join("sample.ck");
    fs::write(
        &source,
        "export fn add_i64(a: i64, b: i64) -> i64 { return a + b; }",
    )
    .expect("write source");
    let legacy_source = dir.join("sample.ik");

    let cases = vec![
        vec![
            os("emit-wat"),
            os("--overflow"),
            os("checked"),
            source.clone().into_os_string(),
        ],
        vec![
            os("emit-wasm"),
            os("--overflow"),
            os("checked"),
            os("--out"),
            dir.join("sample.wasm").into_os_string(),
            source.clone().into_os_string(),
        ],
        vec![
            os("emit-llvm"),
            os("--overflow"),
            os("checked"),
            source.clone().into_os_string(),
        ],
        vec![os("emit-mir"), os("-O9"), source.clone().into_os_string()],
        vec![
            os("emit-c"),
            os("--overflow"),
            os("strict"),
            os("--out"),
            dir.join("sample.c").into_os_string(),
            source.into_os_string(),
        ],
        vec![os("check"), legacy_source.into_os_string()],
    ];

    for args in cases {
        let ts = run_typescript_cli(&ts_cli, &args);
        let rust = run_rust_cli(&args);
        assert_eq!(rust.status_code, ts.status_code, "args: {args:?}");
        assert_eq!(rust.stdout, ts.stdout, "args: {args:?}");
        assert_eq!(rust.stderr, ts.stderr, "args: {args:?}");
    }
}

#[test]
fn cli_should_match_typescript_oracle_for_help_check_and_mir_debug() {
    let Some(ts_cli) = typescript_cli() else {
        return;
    };
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system time")
        .as_nanos();
    let dir = std::env::temp_dir().join(format!("rust_calckernel_cli_success_oracle_{unique}"));
    fs::create_dir_all(&dir).expect("create temp dir");
    let source = dir.join("sample.ck");
    fs::write(
        &source,
        "export fn add_i64(a: i64, b: i64) -> i64 { return a + b; }",
    )
    .expect("write source");

    let cases = vec![
        vec![os("--help")],
        vec![os("check"), source.clone().into_os_string()],
        vec![
            os("emit-mir"),
            os("-O3"),
            os("--print-pass-pipeline"),
            os("--print-mir-before-opt"),
            os("--print-mir-after-opt"),
            source.into_os_string(),
        ],
    ];

    for args in cases {
        let ts = run_typescript_cli(&ts_cli, &args);
        let rust = run_rust_cli(&args);
        assert_eq!(rust.status_code, ts.status_code, "args: {args:?}");
        assert_eq!(rust.stdout, ts.stdout, "args: {args:?}");
        assert_eq!(rust.stderr, ts.stderr, "args: {args:?}");
    }
}

#[test]
fn cli_should_match_typescript_oracle_for_successful_emit_commands() {
    let Some(ts_cli) = typescript_cli() else {
        return;
    };
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system time")
        .as_nanos();
    let dir = std::env::temp_dir().join(format!("rust_calckernel_cli_emit_oracle_{unique}"));
    fs::create_dir_all(&dir).expect("create temp dir");
    let source = dir.join("sample.ck");
    fs::write(
        &source,
        "export fn add_i64(a: i64, b: i64) -> i64 { return a + b; }",
    )
    .expect("write source");

    let cases = vec![
        vec![
            os("emit-c"),
            os("--out"),
            dir.join("sample.c").into_os_string(),
            source.clone().into_os_string(),
        ],
        vec![
            os("emit-c"),
            os("--overflow"),
            os("checked"),
            os("--out"),
            dir.join("sample_checked.c").into_os_string(),
            source.clone().into_os_string(),
        ],
        vec![
            os("emit-wat"),
            os("--out"),
            dir.join("sample.wat").into_os_string(),
            source.clone().into_os_string(),
        ],
        vec![os("emit-wat"), source.clone().into_os_string()],
        vec![
            os("emit-wasm"),
            os("--out"),
            dir.join("sample.wasm").into_os_string(),
            source.clone().into_os_string(),
        ],
        vec![
            os("emit-llvm"),
            os("--target"),
            os("ck-test-target"),
            os("--out"),
            dir.join("sample.ll").into_os_string(),
            source.clone().into_os_string(),
        ],
        vec![
            os("emit-llvm"),
            os("--target"),
            os("ck-test-target"),
            source.into_os_string(),
        ],
    ];

    for args in cases {
        let ts = run_typescript_cli(&ts_cli, &args);
        let rust = run_rust_cli(&args);
        assert_eq!(rust.status_code, ts.status_code, "args: {args:?}");
        assert_eq!(rust.stdout, ts.stdout, "args: {args:?}");
        assert_eq!(rust.stderr, ts.stderr, "args: {args:?}");
    }
}

#[test]
fn cli_should_match_typescript_oracle_for_diagnostics() {
    let Some(ts_cli) = typescript_cli() else {
        return;
    };
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system time")
        .as_nanos();
    let dir = std::env::temp_dir().join(format!("rust_calckernel_cli_diag_oracle_{unique}"));
    fs::create_dir_all(&dir).expect("create temp dir");

    let cases = [
        ("lex.ck", "export fn bad() -> i32 {\n  return @;\n}\n"),
        (
            "parse.ck",
            "export fn bad() -> i32 {\n  let x: i32 = 1\n  return x;\n}\n",
        ),
        (
            "unknown_variable.ck",
            "export fn bad() -> i32 {\n  return missing;\n}\n",
        ),
        (
            "return_mismatch.ck",
            "export fn bad() -> i32 {\n  return true;\n}\n",
        ),
        (
            "non_bmp_unknown_character.ck",
            "export fn bad() -> i32 { return 🙂; }\n",
        ),
        (
            "multiline_unicode_marker_width.ck",
            "export fn bad() -> i32 { // 中文🙂\n  let x: i32 = 1;\n}\n",
        ),
    ];

    for (file_name, source_text) in cases {
        let source = dir.join(file_name);
        fs::write(&source, source_text).expect("write diagnostic source");
        let args = vec![os("check"), source.into_os_string()];
        let ts = run_typescript_cli(&ts_cli, &args);
        let rust = run_rust_cli(&args);
        assert_eq!(rust.status_code, ts.status_code, "file: {file_name}");
        assert_eq!(rust.stdout, ts.stdout, "file: {file_name}");
        assert_eq!(rust.stderr, ts.stderr, "file: {file_name}");
    }
}

#[test]
fn cli_should_match_typescript_oracle_for_usage_errors() {
    let Some(ts_cli) = typescript_cli() else {
        return;
    };
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system time")
        .as_nanos();
    let dir = std::env::temp_dir().join(format!("rust_calckernel_cli_usage_oracle_{unique}"));
    fs::create_dir_all(&dir).expect("create temp dir");
    let source = dir.join("sample.ck");
    fs::write(
        &source,
        "export fn add_i64(a: i64, b: i64) -> i64 { return a + b; }",
    )
    .expect("write source");

    let cases = vec![
        vec![],
        vec![os("check")],
        vec![
            os("check"),
            source.clone().into_os_string(),
            source.clone().into_os_string(),
        ],
        vec![os("emit-c"), source.clone().into_os_string()],
        vec![os("emit-wasm"), source.clone().into_os_string()],
        vec![os("emit-c"), os("--out")],
        vec![os("emit-c"), os("-o")],
        vec![os("emit-c"), os("--overflow")],
    ];

    for args in cases {
        let ts = run_typescript_cli(&ts_cli, &args);
        let rust = run_rust_cli(&args);
        assert_eq!(rust.status_code, ts.status_code, "args: {args:?}");
        assert_eq!(rust.stdout, ts.stdout, "args: {args:?}");
        assert_eq!(rust.stderr, ts.stderr, "args: {args:?}");
    }
}

#[test]
fn cli_should_match_typescript_oracle_for_ignored_unknown_long_flags() {
    let Some(ts_cli) = typescript_cli() else {
        return;
    };
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system time")
        .as_nanos();
    let dir =
        std::env::temp_dir().join(format!("rust_calckernel_cli_unknown_flag_oracle_{unique}"));
    fs::create_dir_all(&dir).expect("create temp dir");
    let source = dir.join("sample.ck");
    fs::write(
        &source,
        "export fn add_i64(a: i64, b: i64) -> i64 { return a + b; }",
    )
    .expect("write source");

    let cases = vec![
        vec![
            os("check"),
            os("--unused"),
            os("value"),
            source.clone().into_os_string(),
        ],
        vec![
            os("emit-c"),
            os("--unused"),
            os("value"),
            os("--out"),
            dir.join("sample.c").into_os_string(),
            source.into_os_string(),
        ],
    ];

    for args in cases {
        let ts = run_typescript_cli(&ts_cli, &args);
        let rust = run_rust_cli(&args);
        assert_eq!(rust.status_code, ts.status_code, "args: {args:?}");
        assert_eq!(rust.stdout, ts.stdout, "args: {args:?}");
        assert_eq!(rust.stderr, ts.stderr, "args: {args:?}");
    }
}

#[test]
fn cli_should_match_typescript_oracle_for_unknown_short_flags_as_positionals() {
    let Some(ts_cli) = typescript_cli() else {
        return;
    };
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system time")
        .as_nanos();
    let dir =
        std::env::temp_dir().join(format!("rust_calckernel_cli_unknown_short_oracle_{unique}"));
    fs::create_dir_all(&dir).expect("create temp dir");
    let source = dir.join("sample.ck");
    fs::write(
        &source,
        "export fn add_i64(a: i64, b: i64) -> i64 { return a + b; }",
    )
    .expect("write source");

    let args = vec![os("check"), os("-x"), source.into_os_string()];
    let ts = run_typescript_cli(&ts_cli, &args);
    let rust = run_rust_cli(&args);
    assert_eq!(rust.status_code, ts.status_code, "args: {args:?}");
    assert_eq!(rust.stdout, ts.stdout, "args: {args:?}");
    assert_eq!(rust.stderr, ts.stderr, "args: {args:?}");
}

#[test]
fn cli_should_defer_semantic_flag_validation_until_command_uses_flag() {
    let Some(ts_cli) = typescript_cli() else {
        return;
    };
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system time")
        .as_nanos();
    let dir = std::env::temp_dir().join(format!(
        "rust_calckernel_deferred_flag_validation_oracle_{unique}"
    ));
    fs::create_dir_all(&dir).expect("create temp dir");
    let source = dir.join("sample.ck");
    fs::write(
        &source,
        "export fn add_i64(a: i64, b: i64) -> i64 { return a + b; }",
    )
    .expect("write source");

    let cases = vec![
        vec![
            os("check"),
            os("--overflow"),
            os("bogus"),
            source.clone().into_os_string(),
        ],
        vec![os("check"), os("-O4"), source.clone().into_os_string()],
        vec![
            os("check"),
            os("--opt-level"),
            os("-O2"),
            source.clone().into_os_string(),
        ],
        vec![
            os("emit-mir"),
            os("--overflow"),
            os("bogus"),
            source.clone().into_os_string(),
        ],
        vec![
            os("emit-mir"),
            os("--out"),
            dir.join("sample.mir").into_os_string(),
            os("--overflow"),
            os("bogus"),
            source.into_os_string(),
        ],
    ];

    for args in cases {
        let ts = run_typescript_cli(&ts_cli, &args);
        let rust = run_rust_cli(&args);
        assert_eq!(rust.status_code, ts.status_code, "args: {args:?}");
        assert_eq!(rust.stdout, ts.stdout, "args: {args:?}");
        assert_eq!(rust.stderr, ts.stderr, "args: {args:?}");
    }
}

#[test]
fn cli_should_match_typescript_oracle_for_argument_error_precedence() {
    let Some(ts_cli) = typescript_cli() else {
        return;
    };
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system time")
        .as_nanos();
    let dir =
        std::env::temp_dir().join(format!("rust_calckernel_error_precedence_oracle_{unique}"));
    fs::create_dir_all(&dir).expect("create temp dir");
    let source = dir.join("sample.ck");
    fs::write(
        &source,
        "export fn add_i64(a: i64, b: i64) -> i64 { return a + b; }",
    )
    .expect("write source");

    let cases = vec![
        vec![os("emit-c"), os("--opt-level"), os("4")],
        vec![
            os("build"),
            os("--overflow"),
            os("bogus"),
            source.clone().into_os_string(),
        ],
        vec![os("emit-wat"), os("--overflow"), os("checked")],
        vec![
            os("emit-wasm"),
            os("--overflow"),
            os("checked"),
            source.clone().into_os_string(),
        ],
        vec![os("emit-llvm"), os("--overflow"), os("checked")],
        vec![
            os("build-llvm"),
            os("--overflow"),
            os("checked"),
            source.into_os_string(),
        ],
    ];

    for args in cases {
        let ts = run_typescript_cli(&ts_cli, &args);
        let rust = run_rust_cli(&args);
        assert_eq!(rust.status_code, ts.status_code, "args: {args:?}");
        assert_eq!(rust.stdout, ts.stdout, "args: {args:?}");
        assert_eq!(rust.stderr, ts.stderr, "args: {args:?}");
    }
}

#[test]
fn cli_should_match_typescript_oracle_for_unknown_command_precedence() {
    let Some(ts_cli) = typescript_cli() else {
        return;
    };
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system time")
        .as_nanos();
    let dir = std::env::temp_dir().join(format!("rust_calckernel_unknown_command_oracle_{unique}"));
    fs::create_dir_all(&dir).expect("create temp dir");
    let source = dir.join("sample.ck");
    fs::write(
        &source,
        "export fn add_i64(a: i64, b: i64) -> i64 { return a + b; }",
    )
    .expect("write source");

    let cases = vec![
        vec![os("unknown"), os("--out"), os("--other"), os("--other")],
        vec![
            os("unknown"),
            source.clone().into_os_string(),
            os("-o"),
            os("--header"),
            os("ck-test-target"),
        ],
        vec![os("checked"), os("value"), os("--overflow")],
        vec![os("4"), os("--target")],
    ];

    for args in cases {
        let ts = run_typescript_cli(&ts_cli, &args);
        let rust = run_rust_cli(&args);
        assert_eq!(rust.status_code, ts.status_code, "args: {args:?}");
        assert_eq!(rust.stdout, ts.stdout, "args: {args:?}");
        assert_eq!(rust.stderr, ts.stderr, "args: {args:?}");
    }
}

#[test]
fn cli_should_match_typescript_oracle_for_missing_input_file_errors() {
    let Some(ts_cli) = typescript_cli() else {
        return;
    };
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system time")
        .as_nanos();
    let dir = std::env::temp_dir().join(format!("rust_calckernel_missing_file_oracle_{unique}"));
    fs::create_dir_all(&dir).expect("create temp dir");
    let missing = dir.join("missing.ck");

    let cases = vec![
        vec![os("check"), missing.clone().into_os_string()],
        vec![
            os("emit-mir"),
            os("--header"),
            os("value"),
            missing.into_os_string(),
            os("--other"),
            dir.join("api.h").into_os_string(),
        ],
    ];

    for args in cases {
        let ts = run_typescript_cli(&ts_cli, &args);
        let rust = run_rust_cli(&args);
        assert_eq!(rust.status_code, ts.status_code, "args: {args:?}");
        assert_eq!(rust.stdout, ts.stdout, "args: {args:?}");
        assert_eq!(rust.stderr, ts.stderr, "args: {args:?}");
    }
}

#[test]
fn cli_should_match_typescript_oracle_for_input_file_read_edge_cases() {
    let Some(ts_cli) = typescript_cli() else {
        return;
    };
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system time")
        .as_nanos();
    let dir = std::env::temp_dir().join(format!("rust_calckernel_file_read_oracle_{unique}"));
    fs::create_dir_all(&dir).expect("create temp dir");
    let directory_input = dir.join("directory.ck");
    fs::create_dir_all(&directory_input).expect("create input directory");
    let invalid_utf8 = dir.join("invalid_utf8.ck");
    fs::write(&invalid_utf8, [0xff, b'\n']).expect("write invalid utf8 source");

    let cases = vec![
        vec![os("check"), directory_input.into_os_string()],
        vec![os("check"), invalid_utf8.into_os_string()],
    ];

    for args in cases {
        let ts = run_typescript_cli(&ts_cli, &args);
        let rust = run_rust_cli(&args);
        assert_eq!(rust.status_code, ts.status_code, "args: {args:?}");
        assert_eq!(rust.stdout, ts.stdout, "args: {args:?}");
        assert_eq!(rust.stderr, ts.stderr, "args: {args:?}");
    }
}

#[test]
fn cli_should_match_typescript_oracle_for_direct_output_write_errors() {
    let Some(ts_cli) = typescript_cli() else {
        return;
    };
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system time")
        .as_nanos();
    let dir = std::env::temp_dir().join(format!("rust_calckernel_direct_write_oracle_{unique}"));
    fs::create_dir_all(&dir).expect("create temp dir");
    let source = dir.join("sample.ck");
    fs::write(
        &source,
        "export fn add_i64(a: i64, b: i64) -> i64 { return a + b; }",
    )
    .expect("write source");
    let output_directory = dir.join("out.mir");
    fs::create_dir_all(&output_directory).expect("create output directory");

    let args = vec![
        os("emit-mir"),
        source.into_os_string(),
        os("--out"),
        output_directory.into_os_string(),
    ];
    let ts = run_typescript_cli(&ts_cli, &args);
    let rust = run_rust_cli(&args);
    assert_eq!(rust.status_code, ts.status_code, "args: {args:?}");
    assert_eq!(rust.stdout, ts.stdout, "args: {args:?}");
    assert_eq!(rust.stderr, ts.stderr, "args: {args:?}");
}

#[test]
fn cli_should_match_typescript_oracle_for_atomic_output_write_errors() {
    let Some(ts_cli) = typescript_cli() else {
        return;
    };
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system time")
        .as_nanos();
    let dir = std::env::temp_dir().join(format!("rust_calckernel_atomic_write_oracle_{unique}"));
    fs::create_dir_all(&dir).expect("create temp dir");
    let source = dir.join("sample.ck");
    fs::write(
        &source,
        "export fn add_i64(a: i64, b: i64) -> i64 { return a + b; }",
    )
    .expect("write source");
    let output_directory = dir.join("out.wat");
    fs::create_dir_all(&output_directory).expect("create output directory");

    let args = vec![
        os("emit-wat"),
        source.into_os_string(),
        os("--out"),
        output_directory.into_os_string(),
    ];
    let ts = run_typescript_cli(&ts_cli, &args);
    let rust = run_rust_cli(&args);
    assert_eq!(rust.status_code, ts.status_code, "args: {args:?}");
    assert_eq!(rust.stdout, ts.stdout, "args: {args:?}");
    assert_eq!(
        normalize_atomic_temp_paths(&rust.stderr),
        normalize_atomic_temp_paths(&ts.stderr),
        "args: {args:?}"
    );
}

#[test]
fn cli_should_match_typescript_oracle_for_output_parent_directory_creation_errors() {
    let Some(ts_cli) = typescript_cli() else {
        return;
    };
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system time")
        .as_nanos();
    let dir = std::env::temp_dir().join(format!("rust_calckernel_parent_dir_oracle_{unique}"));
    fs::create_dir_all(&dir).expect("create temp dir");
    let source = dir.join("sample.ck");
    fs::write(
        &source,
        "export fn add_i64(a: i64, b: i64) -> i64 { return a + b; }",
    )
    .expect("write source");
    let parent_file = dir.join("parent");
    fs::write(&parent_file, "not a directory").expect("write parent file");

    let cases = vec![
        vec![
            os("emit-mir"),
            source.clone().into_os_string(),
            os("--out"),
            parent_file.join("out.mir").into_os_string(),
        ],
        vec![
            os("emit-wat"),
            source.clone().into_os_string(),
            os("--out"),
            parent_file.join("out.wat").into_os_string(),
        ],
        vec![
            os("emit-c"),
            source.clone().into_os_string(),
            os("--out"),
            dir.join("ok.c").into_os_string(),
            os("--header"),
            parent_file.join("out.h").into_os_string(),
        ],
        vec![
            os("emit-mir"),
            source.clone().into_os_string(),
            os("--out"),
            parent_file.join("child").join("out.mir").into_os_string(),
        ],
        vec![
            os("emit-wat"),
            source.clone().into_os_string(),
            os("--out"),
            parent_file.join("child").join("out.wat").into_os_string(),
        ],
        vec![
            os("emit-c"),
            source.clone().into_os_string(),
            os("--out"),
            dir.join("ok_nested.c").into_os_string(),
            os("--header"),
            parent_file.join("child").join("out.h").into_os_string(),
        ],
    ];

    for args in cases {
        let ts = run_typescript_cli(&ts_cli, &args);
        let rust = run_rust_cli(&args);
        assert_eq!(rust.status_code, ts.status_code, "args: {args:?}");
        assert_eq!(rust.stdout, ts.stdout, "args: {args:?}");
        assert_eq!(rust.stderr, ts.stderr, "args: {args:?}");
    }
}

#[derive(Debug)]
struct CapturedOutput {
    status_code: Option<i32>,
    stdout: String,
    stderr: String,
}

fn os(value: &str) -> OsString {
    OsString::from(value)
}

fn normalize_atomic_temp_paths(text: &str) -> String {
    let mut normalized = String::new();
    let mut rest = text;
    while let Some(index) = rest.find(".tmp-") {
        let (before, after) = rest.split_at(index);
        normalized.push_str(before);
        normalized.push_str(".tmp-<id>");
        let Some(end_quote) = after.find('\'') else {
            return normalized;
        };
        rest = &after[end_quote..];
    }
    normalized.push_str(rest);
    normalized
}

fn typescript_cli() -> Option<PathBuf> {
    let cli = typescript_root().join("dist/src/cli.js");
    cli.exists().then_some(cli)
}

fn typescript_root() -> PathBuf {
    std::env::var_os("CALCKERNEL_TS_ROOT")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("/Users/lynn/code/CalcKernel"))
}

fn run_rust_cli(args: &[OsString]) -> CapturedOutput {
    let output = Command::new(env!("CARGO_BIN_EXE_ckc"))
        .args(args)
        .output()
        .expect("run Rust ckc");
    capture(output)
}

fn run_typescript_cli(ts_cli: &PathBuf, args: &[OsString]) -> CapturedOutput {
    let output = Command::new("node")
        .arg(ts_cli)
        .args(args)
        .output()
        .expect("run TypeScript ckc");
    capture(output)
}

fn clang_available() -> bool {
    Command::new("clang")
        .arg("--version")
        .output()
        .is_ok_and(|output| output.status.success())
}

fn python3_available() -> bool {
    Command::new("python3")
        .arg("--version")
        .output()
        .is_ok_and(|output| output.status.success())
}

fn run_python_pricing_dynamic_library(
    runner: &std::path::Path,
    mode: &str,
    library_path: &std::path::Path,
) -> CapturedOutput {
    let output = Command::new("python3")
        .arg(runner)
        .arg(mode)
        .arg(library_path)
        .output()
        .expect("run pricing dynamic library runner");
    capture(output)
}

fn run_python_official_c_dynamic_library(
    runner: &std::path::Path,
    case_name: &str,
    library_path: &std::path::Path,
) -> CapturedOutput {
    let output = Command::new("python3")
        .arg(runner)
        .arg(case_name)
        .arg(library_path)
        .output()
        .expect("run official C dynamic library runner");
    capture(output)
}

fn capture(output: std::process::Output) -> CapturedOutput {
    CapturedOutput {
        status_code: output.status.code(),
        stdout: String::from_utf8(output.stdout).expect("stdout should be utf8"),
        stderr: String::from_utf8(output.stderr).expect("stderr should be utf8"),
    }
}

fn shared_library_path(base: &std::path::Path) -> std::path::PathBuf {
    if cfg!(target_os = "macos") {
        base.with_extension("dylib")
    } else if cfg!(target_os = "windows") {
        base.with_extension("dll")
    } else {
        base.with_extension("so")
    }
}

fn pricing_dynamic_library_runner() -> &'static str {
    r#"
from __future__ import annotations

import ctypes
import sys


CK_OK = 0
CK_ERR_OVERFLOW = 1


class Item(ctypes.Structure):
    _fields_ = [
        ("price", ctypes.c_int64),
        ("qty", ctypes.c_int64),
        ("discount", ctypes.c_int64),
        ("tax_rate_ppm", ctypes.c_int64),
    ]


def pricing_items():
    return (Item * 3)(
        Item(price=10000, qty=2, discount=1000, tax_rate_ppm=82500),
        Item(price=2500, qty=4, discount=0, tax_rate_ppm=100000),
        Item(price=1200, qty=5, discount=500, tax_rate_ppm=100000),
    )


def run_unchecked(library_path: str) -> str:
    lib = ctypes.CDLL(library_path)
    lib.calc_items.argtypes = [
        ctypes.POINTER(Item),
        ctypes.c_int32,
        ctypes.POINTER(ctypes.c_int64),
    ]
    lib.calc_items.restype = ctypes.c_int32

    items = pricing_items()
    out = (ctypes.c_int64 * len(items))(0, 0, 0)
    status = lib.calc_items(items, ctypes.c_int32(len(items)), out)
    expected = [20567, 11000, 6050]
    actual = list(out)
    if status != 0 or actual != expected:
        raise AssertionError(f"unchecked mismatch status={status} actual={actual} expected={expected}")
    return f"pricing:status={status};out={','.join(str(value) for value in actual)}"


def run_checked(library_path: str) -> str:
    lib = ctypes.CDLL(library_path)
    lib.calc_items.argtypes = [
        ctypes.POINTER(Item),
        ctypes.c_int32,
        ctypes.POINTER(ctypes.c_int64),
        ctypes.POINTER(ctypes.c_int32),
    ]
    lib.calc_items.restype = ctypes.c_int32

    items = pricing_items()
    out = (ctypes.c_int64 * len(items))(0, 0, 0)
    ck_return = ctypes.c_int32(-1)
    status = lib.calc_items(items, ctypes.c_int32(len(items)), out, ctypes.byref(ck_return))
    expected = [20567, 11000, 6050]
    actual = list(out)
    if status != CK_OK or ck_return.value != 0 or actual != expected:
        raise AssertionError(
            f"checked mismatch status={status} ck_return={ck_return.value} actual={actual} expected={expected}"
        )

    overflow_items = (Item * 1)(
        Item(price=ctypes.c_int64(9223372036854775807).value, qty=2, discount=0, tax_rate_ppm=0),
    )
    overflow_out = (ctypes.c_int64 * 1)(0)
    overflow_return = ctypes.c_int32(-1)
    overflow_status = lib.calc_items(
        overflow_items,
        ctypes.c_int32(len(overflow_items)),
        overflow_out,
        ctypes.byref(overflow_return),
    )
    if overflow_status != CK_ERR_OVERFLOW:
        raise AssertionError(f"expected CK_ERR_OVERFLOW, got CK_Status {overflow_status}")

    return (
        f"pricing-checked:status={status};ck_return={ck_return.value};"
        f"out={','.join(str(value) for value in actual)};overflow_status={overflow_status}"
    )


def main() -> None:
    mode, library_path = sys.argv[1:3]
    if mode == "unchecked":
        print(run_unchecked(library_path))
    elif mode == "checked":
        print(run_checked(library_path))
    else:
        raise RuntimeError(f"unknown mode: {mode}")


if __name__ == "__main__":
    main()
"#
}

fn official_c_dynamic_library_runner() -> &'static str {
    r#"
from __future__ import annotations

import ctypes
import math
import sys


CK_OK = 0
CK_ERR_OVERFLOW = 1
CK_ERR_DIV_BY_ZERO = 2


def checked_i64_call(fn, *args: int) -> tuple[int, int]:
    result = ctypes.c_int64(-1)
    status = fn(*args, ctypes.byref(result))
    return status, result.value


def checked_bool_call(fn, *args) -> tuple[int, bool]:
    result = ctypes.c_bool(False)
    status = fn(*args, ctypes.byref(result))
    return status, bool(result.value)


def close(actual: float, expected: float) -> bool:
    return math.isclose(actual, expected, rel_tol=0.0, abs_tol=0.0000001)


def format_float(value: float) -> str:
    return format(value, ".17g")


def run_c_scalar(library_path: str) -> str:
    lib = ctypes.CDLL(library_path)
    lib.add_i64.argtypes = [ctypes.c_int64, ctypes.c_int64]
    lib.add_i64.restype = ctypes.c_int64
    lib.max_i32.argtypes = [ctypes.c_int32, ctypes.c_int32]
    lib.max_i32.restype = ctypes.c_int32

    add = lib.add_i64(2, 3)
    high = lib.max_i32(10, 3)
    low = lib.max_i32(1, 3)
    if add != 5 or high != 10 or low != 3:
        raise AssertionError(f"c-scalar mismatch add={add} high={high} low={low}")
    return f"c-scalar:add={add};max={high},{low}"


def run_c_casts(library_path: str) -> str:
    lib = ctypes.CDLL(library_path)
    lib.avg_i32.argtypes = [ctypes.c_int32, ctypes.c_int32]
    lib.avg_i32.restype = ctypes.c_double
    lib.ratio_u32.argtypes = [ctypes.c_uint32, ctypes.c_uint32]
    lib.ratio_u32.restype = ctypes.c_double

    avg = lib.avg_i32(7, 2)
    ratio = lib.ratio_u32(9, 4)
    if not close(avg, 3.5) or not close(ratio, 2.25):
        raise AssertionError(f"c-casts mismatch avg={avg} ratio={ratio}")
    return f"c-casts:avg={avg};ratio={ratio}"


def run_c_scalar_checked(library_path: str) -> str:
    lib = ctypes.CDLL(library_path)
    for name in ["add_i64", "mul_i64", "div_i64"]:
        fn = getattr(lib, name)
        fn.argtypes = [ctypes.c_int64, ctypes.c_int64, ctypes.POINTER(ctypes.c_int64)]
        fn.restype = ctypes.c_int32
    lib.neg_i64.argtypes = [ctypes.c_int64, ctypes.POINTER(ctypes.c_int64)]
    lib.neg_i64.restype = ctypes.c_int32
    lib.less_i64.argtypes = [ctypes.c_int64, ctypes.c_int64, ctypes.POINTER(ctypes.c_bool)]
    lib.less_i64.restype = ctypes.c_int32

    add_status, add = checked_i64_call(lib.add_i64, 2, 3)
    mul_status, product = checked_i64_call(lib.mul_i64, 3, 4)
    div_status, quotient = checked_i64_call(lib.div_i64, 10, 2)
    less_status, less = checked_bool_call(lib.less_i64, 1, 2)
    overflow_status, _ = checked_i64_call(lib.add_i64, 9223372036854775807, 1)
    div_zero_status, _ = checked_i64_call(lib.div_i64, 10, 0)
    neg_overflow_status, _ = checked_i64_call(lib.neg_i64, -9223372036854775808)
    if (
        add_status != CK_OK
        or add != 5
        or mul_status != CK_OK
        or product != 12
        or div_status != CK_OK
        or quotient != 5
        or less_status != CK_OK
        or less is not True
        or overflow_status != CK_ERR_OVERFLOW
        or div_zero_status != CK_ERR_DIV_BY_ZERO
        or neg_overflow_status != CK_ERR_OVERFLOW
    ):
        raise AssertionError("c-scalar-checked mismatch")
    return (
        f"c-scalar-checked:add={add};mul={product};div={quotient};less={int(less)};"
        f"overflow={overflow_status};div_zero={div_zero_status};neg_overflow={neg_overflow_status}"
    )


def run_c_control_checked(library_path: str) -> str:
    lib = ctypes.CDLL(library_path)
    for name in ["sum_to_n", "choose", "condition_overflow"]:
        fn = getattr(lib, name)
        if name == "sum_to_n":
            fn.argtypes = [ctypes.c_int64, ctypes.POINTER(ctypes.c_int64)]
        else:
            fn.argtypes = [ctypes.c_int64, ctypes.c_int64, ctypes.POINTER(ctypes.c_int64)]
        fn.restype = ctypes.c_int32

    sum_status, total = checked_i64_call(lib.sum_to_n, 5)
    high_status, high = checked_i64_call(lib.choose, 10, 3)
    low_status, low = checked_i64_call(lib.choose, 1, 3)
    overflow_status, _ = checked_i64_call(lib.condition_overflow, 9223372036854775807, 1)
    if (
        sum_status != CK_OK
        or total != 10
        or high_status != CK_OK
        or high != 10
        or low_status != CK_OK
        or low != 3
        or overflow_status != CK_ERR_OVERFLOW
    ):
        raise AssertionError("c-control-checked mismatch")
    return f"c-control-checked:sum={total};choose={high},{low};overflow={overflow_status}"


def run_c_logical_checked(library_path: str) -> str:
    lib = ctypes.CDLL(library_path)
    lib.and_short_circuit.argtypes = [ctypes.c_int64, ctypes.c_int64, ctypes.POINTER(ctypes.c_bool)]
    lib.and_short_circuit.restype = ctypes.c_int32
    lib.or_short_circuit.argtypes = [ctypes.c_int64, ctypes.c_int64, ctypes.POINTER(ctypes.c_bool)]
    lib.or_short_circuit.restype = ctypes.c_int32
    lib.and_rhs_error.argtypes = [ctypes.c_bool, ctypes.c_int64, ctypes.c_int64, ctypes.POINTER(ctypes.c_bool)]
    lib.and_rhs_error.restype = ctypes.c_int32

    cases = [
        checked_bool_call(lib.and_short_circuit, 0, 10),
        checked_bool_call(lib.and_short_circuit, 2, 10),
        checked_bool_call(lib.or_short_circuit, 0, 10),
        checked_bool_call(lib.or_short_circuit, 2, 10),
        checked_bool_call(lib.and_rhs_error, False, 10, 0),
    ]
    div_zero_status, _ = checked_bool_call(lib.and_rhs_error, True, 10, 0)
    expected = [
        (CK_OK, False),
        (CK_OK, True),
        (CK_OK, True),
        (CK_OK, True),
        (CK_OK, False),
    ]
    if cases != expected or div_zero_status != CK_ERR_DIV_BY_ZERO:
        raise AssertionError(f"c-logical-checked mismatch cases={cases} div_zero={div_zero_status}")
    encoded = ",".join(str(int(value)) for _, value in cases)
    return f"c-logical-checked:out={encoded};div_zero={div_zero_status}"


def run_c_calls_checked(library_path: str) -> str:
    lib = ctypes.CDLL(library_path)
    lib.calc.argtypes = [ctypes.c_int64, ctypes.c_int64, ctypes.POINTER(ctypes.c_int64)]
    lib.calc.restype = ctypes.c_int32
    lib.calc_overflow.argtypes = [ctypes.c_int64, ctypes.c_int64, ctypes.POINTER(ctypes.c_int64)]
    lib.calc_overflow.restype = ctypes.c_int32

    status, result = checked_i64_call(lib.calc, 1, 2)
    overflow_status, _ = checked_i64_call(lib.calc_overflow, 9223372036854775807, 1)
    if status != CK_OK or result != 6 or overflow_status != CK_ERR_OVERFLOW:
        raise AssertionError(f"c-calls-checked mismatch status={status} result={result} overflow={overflow_status}")
    return f"c-calls-checked:calc={result};overflow={overflow_status}"


def expected_total(price: int, quantity: int, discount: int, tax_rate_ppm: int) -> int:
    after_discount = price * quantity - discount
    tax = after_discount * tax_rate_ppm // 1000000
    return after_discount + tax


def run_c_perf_pricing_helpers(library_path: str, label: str) -> str:
    class Item(ctypes.Structure):
        _fields_ = [
            ("price", ctypes.c_int64),
            ("qty", ctypes.c_int64),
            ("discount", ctypes.c_int64),
            ("tax_rate_ppm", ctypes.c_int64),
        ]

    lib = ctypes.CDLL(library_path)
    lib.calc_items.argtypes = [
        ctypes.POINTER(Item),
        ctypes.c_int32,
        ctypes.POINTER(ctypes.c_int64),
    ]
    lib.calc_items.restype = ctypes.c_int32

    rows = [
        (10000, 2, 1000, 82500),
        (2500, 4, 0, 100000),
        (1200, 5, 500, 100000),
        (999, 3, 100, 62500),
    ]
    items = (Item * len(rows))(*(Item(*row) for row in rows))
    out = (ctypes.c_int64 * len(rows))()
    status = lib.calc_items(items, ctypes.c_int32(len(rows)), out)
    actual = list(out)
    expected = [expected_total(*row) for row in rows]
    if status != 0 or actual != expected:
        raise AssertionError(f"{label} mismatch status={status} actual={actual} expected={expected}")
    return f"{label}:status={status};out={','.join(str(value) for value in actual)}"


def run_c_perf_pricing_soa(library_path: str) -> str:
    lib = ctypes.CDLL(library_path)
    lib.pricing_soa.argtypes = [
        ctypes.POINTER(ctypes.c_int64),
        ctypes.POINTER(ctypes.c_int64),
        ctypes.POINTER(ctypes.c_int64),
        ctypes.POINTER(ctypes.c_int64),
        ctypes.POINTER(ctypes.c_int64),
        ctypes.c_int32,
    ]
    lib.pricing_soa.restype = ctypes.c_int32

    rows = [
        (10000, 2, 1000, 82500),
        (2500, 4, 0, 100000),
        (1200, 5, 500, 100000),
        (999, 3, 100, 62500),
    ]
    length = len(rows)
    prices = (ctypes.c_int64 * length)(*(row[0] for row in rows))
    quantities = (ctypes.c_int64 * length)(*(row[1] for row in rows))
    discounts = (ctypes.c_int64 * length)(*(row[2] for row in rows))
    tax_rates = (ctypes.c_int64 * length)(*(row[3] for row in rows))
    out = (ctypes.c_int64 * length)()
    status = lib.pricing_soa(prices, quantities, discounts, tax_rates, out, ctypes.c_int32(length))
    actual = list(out)
    expected = [expected_total(*row) for row in rows]
    if status != 0 or actual != expected:
        raise AssertionError(f"c-perf-pricing-soa-o3 mismatch status={status} actual={actual} expected={expected}")
    return f"c-perf-pricing-soa-o3:status={status};out={','.join(str(value) for value in actual)}"


def run_c_perf_f64_kernels(library_path: str) -> str:
    lib = ctypes.CDLL(library_path)
    double_ptr = ctypes.POINTER(ctypes.c_double)
    lib.axpy_f64.argtypes = [ctypes.c_double, double_ptr, double_ptr, ctypes.c_int32]
    lib.axpy_f64.restype = ctypes.c_double
    lib.dot_f64.argtypes = [double_ptr, double_ptr, ctypes.c_int32]
    lib.dot_f64.restype = ctypes.c_double
    lib.sum_f64.argtypes = [double_ptr, ctypes.c_int32]
    lib.sum_f64.restype = ctypes.c_double
    lib.scale_f64.argtypes = [ctypes.c_double, double_ptr, ctypes.c_int32]
    lib.scale_f64.restype = ctypes.c_double

    x_input = [1.0, -2.0, 3.5, 4.25]
    y_input = [0.5, 8.0, -1.5, 2.25]
    length = len(x_input)
    x = (ctypes.c_double * length)(*x_input)
    y = (ctypes.c_double * length)(*y_input)
    axpy_checksum = lib.axpy_f64(ctypes.c_double(1.5), x, y, ctypes.c_int32(length))
    axpy_actual = list(y)
    axpy_expected = [1.5 * value + y_input[index] for index, value in enumerate(x_input)]
    axpy_expected_checksum = sum(axpy_expected)
    if not close(axpy_checksum, axpy_expected_checksum) or any(
        not close(actual, expected) for actual, expected in zip(axpy_actual, axpy_expected)
    ):
        raise AssertionError(f"c-perf-f64-kernels-o3 axpy mismatch checksum={axpy_checksum} out={axpy_actual}")

    x = (ctypes.c_double * length)(*x_input)
    y = (ctypes.c_double * length)(*y_input)
    dot_actual = lib.dot_f64(x, y, ctypes.c_int32(length))
    dot_expected = sum(value * y_input[index] for index, value in enumerate(x_input))
    if not close(dot_actual, dot_expected):
        raise AssertionError(f"c-perf-f64-kernels-o3 dot mismatch actual={dot_actual} expected={dot_expected}")

    sum_actual = lib.sum_f64(x, ctypes.c_int32(length))
    sum_expected = sum(x_input)
    if not close(sum_actual, sum_expected):
        raise AssertionError(f"c-perf-f64-kernels-o3 sum mismatch actual={sum_actual} expected={sum_expected}")

    scale_input = [0.25, -1.5, 2.0, 10.0]
    scale = (ctypes.c_double * length)(*scale_input)
    scale_checksum = lib.scale_f64(ctypes.c_double(-2.0), scale, ctypes.c_int32(length))
    scale_actual = list(scale)
    scale_expected = [-2.0 * value for value in scale_input]
    scale_expected_checksum = sum(scale_expected)
    if not close(scale_checksum, scale_expected_checksum) or any(
        not close(actual, expected) for actual, expected in zip(scale_actual, scale_expected)
    ):
        raise AssertionError(f"c-perf-f64-kernels-o3 scale mismatch checksum={scale_checksum} out={scale_actual}")

    return (
        f"c-perf-f64-kernels-o3:axpy={format_float(axpy_checksum)};"
        f"dot={format_float(dot_actual)};sum={format_float(sum_actual)};"
        f"scale={format_float(scale_checksum)};"
        f"axpyOut={','.join(format_float(value) for value in axpy_actual)};"
        f"scaleOut={','.join(format_float(value) for value in scale_actual)}"
    )


RUNNERS = {
    "c-scalar": run_c_scalar,
    "c-casts": run_c_casts,
    "c-scalar-checked": run_c_scalar_checked,
    "c-control-checked": run_c_control_checked,
    "c-logical-checked": run_c_logical_checked,
    "c-calls-checked": run_c_calls_checked,
    "c-perf-pricing-helpers-o0": lambda path: run_c_perf_pricing_helpers(path, "c-perf-pricing-helpers-o0"),
    "c-perf-pricing-helpers-o2": lambda path: run_c_perf_pricing_helpers(path, "c-perf-pricing-helpers-o2"),
    "c-perf-pricing-soa-o3": run_c_perf_pricing_soa,
    "c-perf-f64-kernels-o3": run_c_perf_f64_kernels,
}


def main() -> None:
    case_name, library_path = sys.argv[1:3]
    runner = RUNNERS.get(case_name)
    if runner is None:
        raise RuntimeError(f"unknown C dynamic case: {case_name}")
    print(runner(library_path))


if __name__ == "__main__":
    main()
"#
}
