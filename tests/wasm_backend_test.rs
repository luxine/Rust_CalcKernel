use calckernel::{
    MirPassContext, MirPassOverflowMode, MirPassTargetBackend, SourceFile,
    build_mir_optimization_pipeline, check, emit_wasm_module, emit_wat_module, lower_to_mir,
    run_mir_pass_pipeline,
};
use std::{
    fs,
    path::PathBuf,
    process::Command,
    time::{SystemTime, UNIX_EPOCH},
};

fn emit_wat(source_text: &str, opt_level: u8) -> String {
    let checked = check(&SourceFile::new("test.ck", source_text));
    assert_eq!(checked.diagnostics, []);
    let mir = lower_to_mir(&checked.checked_program).expect("MIR lowering should succeed");
    let pipeline = build_mir_optimization_pipeline(opt_level);
    let optimized = run_mir_pass_pipeline(
        mir,
        &pipeline,
        &MirPassContext {
            opt_level,
            overflow_mode: MirPassOverflowMode::Unchecked,
            target_backend: MirPassTargetBackend::Wasm,
            debug: Default::default(),
        },
    );
    assert_eq!(optimized.validation_errors, []);
    emit_wat_module(&optimized.module)
}

#[test]
fn wat_backend_should_emit_scalar_memory_cast_and_dispatcher_text() {
    let wat = emit_wat(
        r#"
      struct Item {
        price: i64;
        qty: i64;
      }

      export fn add_i64(a: i64, b: i64) -> i64 {
        return a + b;
      }

      export fn sum_to_n(n: i64) -> i64 {
        let i: i64 = 0;
        let sum: i64 = 0;
        while i < n {
          sum = sum + i;
          i = i + 1;
        }
        return sum;
      }

      export fn calc(items: ptr<Item>, out: ptr<i64>) -> i32 {
        out[0] = items[0].price * items[0].qty;
        return 0;
      }

      export fn as_f64(a: i32, b: u32) -> f64 {
        return i32_to_f64(a) + u32_to_f64(b);
      }
    "#,
        1,
    );

    assert!(wat.contains("(memory (export \"memory\") 1)"));
    assert!(wat.contains("(global (export \"__ck_heap_base\") i32 (i32.const 0))"));
    assert!(wat.contains("(func $add_i64 (export \"add_i64\")"));
    assert!(wat.contains("i64.add"));
    assert!(wat.contains("(local $ik_bb i32)"));
    assert!(wat.contains("loop $ik_dispatch"));
    assert!(wat.contains("i64.load offset=0 align=8"));
    assert!(wat.contains("i64.store offset=0 align=8"));
    assert!(wat.contains("f64.convert_i32_s"));
    assert!(wat.contains("f64.convert_i32_u"));
}

#[test]
fn wasm_backend_should_compile_wat_to_wasm_bytes() {
    let checked = check(&SourceFile::new(
        "test.ck",
        "export fn add_i64(a: i64, b: i64) -> i64 { return a + b; }",
    ));
    assert_eq!(checked.diagnostics, []);
    let mir = lower_to_mir(&checked.checked_program).expect("MIR lowering should succeed");

    let bytes = emit_wasm_module(&mir).expect("WAT should compile to WASM");

    assert_eq!(&bytes[..4], b"\0asm");
    assert_eq!(&bytes[4..8], &[1, 0, 0, 0]);
}

#[test]
fn wat_backend_should_match_typescript_oracle_for_official_examples() {
    let Some(ts_cli) = typescript_cli() else {
        return;
    };
    let examples = [
        "examples/scalar.ck",
        "examples/explicit_casts.ck",
        "examples/pricing.ck",
        "examples/dijkstra.ck",
        "examples/scalar_checked.ck",
        "examples/scalar_control_checked.ck",
        "examples/scalar_calls_checked.ck",
        "examples/scalar_logical_checked.ck",
        "examples/llvm_scalar.ck",
        "examples/llvm_calls.ck",
        "examples/llvm_memory.ck",
        "examples/llvm_control_flow.ck",
        "examples/llvm_short_circuit.ck",
        "examples/llvm_bool.ck",
        "examples/wasm_scalar.ck",
        "examples/wasm_calls.ck",
        "examples/wasm_memory.ck",
        "examples/wasm_control_flow.ck",
        "examples/wasm_short_circuit.ck",
        "examples/node-wasm-f64-array/f64_array.ck",
        "examples/wasm/f64-axpy/axpy.ck",
        "examples/wasm/f64-sum/sum.ck",
        "examples/wasm/pricing-soa/pricing_soa.ck",
        "bench/perf/fixtures/pricing_helpers.ck",
        "bench/perf/fixtures/pricing_soa.ck",
        "bench/perf/fixtures/f64_kernels.ck",
    ];

    for example in examples {
        let source_path = PathBuf::from("/Users/lynn/code/CalcKernel").join(example);
        let source_text = std::fs::read_to_string(&source_path).expect("read TS WAT example");
        let rust_wat = emit_wat(&source_text, 0);
        let ts_output = Command::new("node")
            .arg(&ts_cli)
            .arg("emit-wat")
            .arg(&source_path)
            .output()
            .expect("run TypeScript ckc emit-wat");

        assert!(
            ts_output.status.success(),
            "{example} stderr:\n{}",
            String::from_utf8_lossy(&ts_output.stderr)
        );
        assert_eq!(
            rust_wat,
            String::from_utf8(ts_output.stdout).expect("TS WAT should be UTF-8"),
            "{example}"
        );
    }
}

#[test]
fn wasm_cli_should_match_typescript_oracle_for_official_example_bytes() {
    let Some(ts_cli) = typescript_cli() else {
        return;
    };
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system time")
        .as_nanos();
    let dir = std::env::temp_dir().join(format!("rust_calckernel_wasm_oracle_{unique}"));
    fs::create_dir_all(&dir).expect("create temp dir");
    let examples = [
        "examples/scalar.ck",
        "examples/explicit_casts.ck",
        "examples/pricing.ck",
        "examples/dijkstra.ck",
        "examples/scalar_checked.ck",
        "examples/scalar_control_checked.ck",
        "examples/scalar_calls_checked.ck",
        "examples/scalar_logical_checked.ck",
        "examples/llvm_scalar.ck",
        "examples/llvm_calls.ck",
        "examples/llvm_memory.ck",
        "examples/llvm_control_flow.ck",
        "examples/llvm_short_circuit.ck",
        "examples/llvm_bool.ck",
        "examples/wasm_scalar.ck",
        "examples/wasm_calls.ck",
        "examples/wasm_memory.ck",
        "examples/wasm_control_flow.ck",
        "examples/wasm_short_circuit.ck",
        "examples/node-wasm-f64-array/f64_array.ck",
        "examples/wasm/f64-axpy/axpy.ck",
        "examples/wasm/f64-sum/sum.ck",
        "examples/wasm/pricing-soa/pricing_soa.ck",
        "bench/perf/fixtures/pricing_helpers.ck",
        "bench/perf/fixtures/pricing_soa.ck",
        "bench/perf/fixtures/f64_kernels.ck",
    ];

    for (index, example) in examples.iter().enumerate() {
        let source_path = PathBuf::from("/Users/lynn/code/CalcKernel").join(example);
        let wasm_path = dir.join(format!("example_{index}.wasm"));

        let ts_output = Command::new("node")
            .arg(&ts_cli)
            .arg("emit-wasm")
            .arg("--out")
            .arg(&wasm_path)
            .arg(&source_path)
            .output()
            .expect("run TypeScript ckc emit-wasm");
        assert!(
            ts_output.status.success(),
            "{example} TS stderr:\n{}",
            String::from_utf8_lossy(&ts_output.stderr)
        );
        let ts_bytes = fs::read(&wasm_path).expect("read TS wasm");

        let rust_output = Command::new(env!("CARGO_BIN_EXE_ckc"))
            .arg("emit-wasm")
            .arg("--out")
            .arg(&wasm_path)
            .arg(&source_path)
            .output()
            .expect("run Rust ckc emit-wasm");
        assert!(
            rust_output.status.success(),
            "{example} Rust stderr:\n{}",
            String::from_utf8_lossy(&rust_output.stderr)
        );
        let rust_bytes = fs::read(&wasm_path).expect("read Rust wasm");

        assert_eq!(
            String::from_utf8(rust_output.stdout).expect("Rust stdout should be UTF-8"),
            String::from_utf8(ts_output.stdout).expect("TS stdout should be UTF-8"),
            "{example} stdout"
        );
        assert_eq!(rust_bytes, ts_bytes, "{example} wasm bytes");
    }
}

#[test]
fn wasm_cli_should_match_typescript_oracle_for_official_interop_runtime_behavior() {
    let Some(ts_cli) = typescript_cli() else {
        return;
    };
    if !node_available() {
        return;
    }
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system time")
        .as_nanos();
    let dir = std::env::temp_dir().join(format!("rust_calckernel_wasm_runtime_{unique}"));
    fs::create_dir_all(&dir).expect("create temp dir");
    let runner = dir.join("run_wasm_case.mjs");
    fs::write(&runner, wasm_runtime_runner()).expect("write WASM runtime runner");
    let cases = [
        ("wasm-scalar", "examples/wasm_scalar.ck"),
        ("wasm-calls", "examples/wasm_calls.ck"),
        ("wasm-control-flow", "examples/wasm_control_flow.ck"),
        ("wasm-memory", "examples/wasm_memory.ck"),
        ("wasm-short-circuit", "examples/wasm_short_circuit.ck"),
        ("pricing-aos", "examples/pricing.ck"),
        ("f64-array", "examples/node-wasm-f64-array/f64_array.ck"),
        ("f64-axpy", "examples/wasm/f64-axpy/axpy.ck"),
        ("f64-sum", "examples/wasm/f64-sum/sum.ck"),
        ("pricing-soa", "examples/wasm/pricing-soa/pricing_soa.ck"),
    ];

    for (case_name, example) in cases {
        let source_path = PathBuf::from("/Users/lynn/code/CalcKernel").join(example);
        let ts_wasm = dir.join(format!("{case_name}.ts.wasm"));
        let rust_wasm = dir.join(format!("{case_name}.rust.wasm"));

        let ts_emit = Command::new("node")
            .arg(&ts_cli)
            .arg("emit-wasm")
            .arg("-O3")
            .arg("--out")
            .arg(&ts_wasm)
            .arg(&source_path)
            .output()
            .expect("run TypeScript ckc emit-wasm");
        assert!(
            ts_emit.status.success(),
            "{case_name} TS emit stderr:\n{}",
            String::from_utf8_lossy(&ts_emit.stderr)
        );

        let rust_emit = Command::new(env!("CARGO_BIN_EXE_ckc"))
            .arg("emit-wasm")
            .arg("-O3")
            .arg("--out")
            .arg(&rust_wasm)
            .arg(&source_path)
            .output()
            .expect("run Rust ckc emit-wasm");
        assert!(
            rust_emit.status.success(),
            "{case_name} Rust emit stderr:\n{}",
            String::from_utf8_lossy(&rust_emit.stderr)
        );

        let ts_run = Command::new("node")
            .arg(&runner)
            .arg(case_name)
            .arg(&ts_wasm)
            .output()
            .expect("run TS WASM runtime case");
        assert!(
            ts_run.status.success(),
            "{case_name} TS runtime stderr:\n{}",
            String::from_utf8_lossy(&ts_run.stderr)
        );

        let rust_run = Command::new("node")
            .arg(&runner)
            .arg(case_name)
            .arg(&rust_wasm)
            .output()
            .expect("run Rust WASM runtime case");
        assert!(
            rust_run.status.success(),
            "{case_name} Rust runtime stderr:\n{}",
            String::from_utf8_lossy(&rust_run.stderr)
        );

        assert_eq!(
            String::from_utf8(rust_run.stdout).expect("Rust runtime stdout should be UTF-8"),
            String::from_utf8(ts_run.stdout).expect("TS runtime stdout should be UTF-8"),
            "{case_name} runtime stdout"
        );
        assert_eq!(
            String::from_utf8(rust_run.stderr).expect("Rust runtime stderr should be UTF-8"),
            String::from_utf8(ts_run.stderr).expect("TS runtime stderr should be UTF-8"),
            "{case_name} runtime stderr"
        );
    }
}

#[test]
fn wasm_cli_should_match_typescript_oracle_for_perf_fixture_runtime_behavior() {
    let Some(ts_cli) = typescript_cli() else {
        return;
    };
    if !node_available() {
        return;
    }
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system time")
        .as_nanos();
    let dir = std::env::temp_dir().join(format!("rust_calckernel_perf_wasm_runtime_{unique}"));
    fs::create_dir_all(&dir).expect("create temp dir");
    let runner = dir.join("run_wasm_case.mjs");
    fs::write(&runner, wasm_runtime_runner()).expect("write WASM runtime runner");
    let cases = [
        (
            "bench-pricing-helpers",
            "bench/perf/fixtures/pricing_helpers.ck",
        ),
        ("bench-pricing-soa", "bench/perf/fixtures/pricing_soa.ck"),
        ("bench-f64-kernels", "bench/perf/fixtures/f64_kernels.ck"),
    ];

    for (case_name, fixture) in cases {
        let source_path = PathBuf::from("/Users/lynn/code/CalcKernel").join(fixture);
        let ts_wasm = dir.join(format!("{case_name}.ts.wasm"));
        let rust_wasm = dir.join(format!("{case_name}.rust.wasm"));

        let ts_emit = Command::new("node")
            .arg(&ts_cli)
            .arg("emit-wasm")
            .arg("-O3")
            .arg("--out")
            .arg(&ts_wasm)
            .arg(&source_path)
            .output()
            .expect("run TypeScript ckc emit-wasm");
        assert!(
            ts_emit.status.success(),
            "{case_name} TS emit stderr:\n{}",
            String::from_utf8_lossy(&ts_emit.stderr)
        );

        let rust_emit = Command::new(env!("CARGO_BIN_EXE_ckc"))
            .arg("emit-wasm")
            .arg("-O3")
            .arg("--out")
            .arg(&rust_wasm)
            .arg(&source_path)
            .output()
            .expect("run Rust ckc emit-wasm");
        assert!(
            rust_emit.status.success(),
            "{case_name} Rust emit stderr:\n{}",
            String::from_utf8_lossy(&rust_emit.stderr)
        );

        let ts_run = Command::new("node")
            .arg(&runner)
            .arg(case_name)
            .arg(&ts_wasm)
            .output()
            .expect("run TS WASM runtime case");
        assert!(
            ts_run.status.success(),
            "{case_name} TS runtime stderr:\n{}",
            String::from_utf8_lossy(&ts_run.stderr)
        );

        let rust_run = Command::new("node")
            .arg(&runner)
            .arg(case_name)
            .arg(&rust_wasm)
            .output()
            .expect("run Rust WASM runtime case");
        assert!(
            rust_run.status.success(),
            "{case_name} Rust runtime stderr:\n{}",
            String::from_utf8_lossy(&rust_run.stderr)
        );

        assert_eq!(
            String::from_utf8(rust_run.stdout).expect("Rust runtime stdout should be UTF-8"),
            String::from_utf8(ts_run.stdout).expect("TS runtime stdout should be UTF-8"),
            "{case_name} runtime stdout"
        );
        assert_eq!(
            String::from_utf8(rust_run.stderr).expect("Rust runtime stderr should be UTF-8"),
            String::from_utf8(ts_run.stderr).expect("TS runtime stderr should be UTF-8"),
            "{case_name} runtime stderr"
        );
    }
}

fn typescript_cli() -> Option<PathBuf> {
    let root = std::env::var_os("CALCKERNEL_TS_ROOT")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("/Users/lynn/code/CalcKernel"));
    let cli = root.join("dist/src/cli.js");
    cli.exists().then_some(cli)
}

fn node_available() -> bool {
    Command::new("node")
        .arg("--version")
        .output()
        .is_ok_and(|output| output.status.success())
}

fn wasm_runtime_runner() -> &'static str {
    r#"
import { readFileSync } from "node:fs";

const [caseName, wasmPath] = process.argv.slice(2);
const bytes = readFileSync(wasmPath);
const { instance } = await WebAssembly.instantiate(bytes);
const { memory } = instance.exports;
if (!(memory instanceof WebAssembly.Memory)) {
  throw new Error("generated module did not export memory");
}

function close(actual, expected) {
  return Math.abs(actual - expected) < 0.0000001;
}

function expectedTotal(price, quantity, discount, taxRatePpm) {
  const subtotal = price * quantity;
  const afterDiscount = subtotal - discount;
  const tax = (afterDiscount * taxRatePpm) / 1000000n;
  return afterDiscount + tax;
}

function runWasmScalar() {
  const addI32 = instance.exports.add_i32;
  const addI64 = instance.exports.add_i64;
  const lessI64 = instance.exports.less_i64;
  const divU64 = instance.exports.div_u64;
  if (
    typeof addI32 !== "function" ||
    typeof addI64 !== "function" ||
    typeof lessI64 !== "function" ||
    typeof divU64 !== "function"
  ) {
    throw new Error("generated scalar WASM did not export the expected functions");
  }
  const values = {
    addI32: addI32(1, 2),
    addI64: addI64(1n, 2n),
    lessI64: lessI64(1n, 2n),
    divU64: divU64(10n, 2n)
  };
  if (values.addI32 !== 3 || values.addI64 !== 3n || values.lessI64 !== 1 || values.divU64 !== 5n) {
    throw new Error(`wasm-scalar mismatch ${JSON.stringify(values, (_, value) => typeof value === "bigint" ? value.toString() : value)}`);
  }
  return `wasm-scalar:add_i32=${values.addI32};add_i64=${values.addI64};less_i64=${values.lessI64};div_u64=${values.divU64}`;
}

function runWasmCalls() {
  const calc = instance.exports.calc;
  if (typeof calc !== "function") {
    throw new Error("generated call WASM did not export calc");
  }
  if (instance.exports.add_i64 !== undefined || instance.exports.double_i64 !== undefined) {
    throw new Error("generated call WASM exported non-export helper functions");
  }
  const result = calc(1n, 2n);
  if (result !== 6n) {
    throw new Error(`wasm-calls mismatch result=${result}`);
  }
  return `wasm-calls:calc=${result};helpers=private`;
}

function runWasmControlFlow() {
  const maxI32 = instance.exports.max_i32;
  const sumToN = instance.exports.sum_to_n;
  if (typeof maxI32 !== "function" || typeof sumToN !== "function") {
    throw new Error("generated control-flow WASM did not export the expected functions");
  }
  const high = maxI32(10, 3);
  const low = maxI32(1, 3);
  const sum = sumToN(5n);
  if (high !== 10 || low !== 3 || sum !== 10n) {
    throw new Error(`wasm-control-flow mismatch high=${high} low=${low} sum=${sum}`);
  }
  return `wasm-control-flow:max=${high},${low};sum=${sum}`;
}

function writeItem(view, offset, fields) {
  view.setBigInt64(offset + 0, fields.price, true);
  view.setBigInt64(offset + 8, fields.qty, true);
  view.setBigInt64(offset + 16, fields.discount, true);
  view.setBigInt64(offset + 24, fields.taxRatePpm, true);
}

function runWasmMemory() {
  const firstPrice = instance.exports.first_price;
  const getPrice = instance.exports.get_price;
  const writeI64 = instance.exports.write_i64;
  if (typeof firstPrice !== "function" || typeof getPrice !== "function" || typeof writeI64 !== "function") {
    throw new Error("generated memory WASM did not export the expected functions");
  }
  const view = new DataView(memory.buffer);
  writeItem(view, 0, { price: 1234n, qty: 2n, discount: 3n, taxRatePpm: 4n });
  const first = firstPrice(0);
  const base = 128;
  const itemSize = 32;
  writeItem(view, base, { price: 11n, qty: 0n, discount: 0n, taxRatePpm: 0n });
  writeItem(view, base + itemSize, { price: 222n, qty: 0n, discount: 0n, taxRatePpm: 0n });
  const indexed = getPrice(base, 1);
  const outOffset = 512;
  const status = writeI64(outOffset, 123n);
  const stored = view.getBigInt64(outOffset, true);
  if (first !== 1234n || indexed !== 222n || status !== 0 || stored !== 123n) {
    throw new Error(`wasm-memory mismatch first=${first} indexed=${indexed} status=${status} stored=${stored}`);
  }
  return `wasm-memory:first=${first};indexed=${indexed};status=${status};stored=${stored}`;
}

function runWasmShortCircuit() {
  const andShortCircuit = instance.exports.and_short_circuit;
  const orShortCircuit = instance.exports.or_short_circuit;
  if (typeof andShortCircuit !== "function" || typeof orShortCircuit !== "function") {
    throw new Error("generated short-circuit WASM did not export the expected functions");
  }
  const values = [
    andShortCircuit(0n, 10n),
    andShortCircuit(2n, 10n),
    orShortCircuit(0n, 10n),
    orShortCircuit(2n, 10n)
  ];
  const expected = [0, 1, 1, 1];
  if (values.some((value, index) => value !== expected[index])) {
    throw new Error(`wasm-short-circuit mismatch actual=${values.join(",")} expected=${expected.join(",")}`);
  }
  return `wasm-short-circuit:out=${values.join(",")}`;
}

function runPricingAos() {
  const calcItems = instance.exports.calc_items;
  if (typeof calcItems !== "function") {
    throw new Error("generated WASM did not export calc_items");
  }
  const view = new DataView(memory.buffer);
  const itemsOffset = 0;
  const outOffset = 4096;
  const itemSize = 32;
  const items = [
    { price: 10000n, qty: 2n, discount: 1000n, taxRatePpm: 82500n },
    { price: 2500n, qty: 4n, discount: 0n, taxRatePpm: 100000n },
    { price: 1200n, qty: 5n, discount: 500n, taxRatePpm: 100000n }
  ];
  for (let index = 0; index < items.length; index += 1) {
    const base = itemsOffset + index * itemSize;
    const item = items[index];
    view.setBigInt64(base + 0, item.price, true);
    view.setBigInt64(base + 8, item.qty, true);
    view.setBigInt64(base + 16, item.discount, true);
    view.setBigInt64(base + 24, item.taxRatePpm, true);
  }
  const status = calcItems(itemsOffset, items.length, outOffset);
  const actual = items.map((_, index) => view.getBigInt64(outOffset + index * 8, true));
  const expected = items.map((item) => expectedTotal(item.price, item.qty, item.discount, item.taxRatePpm));
  if (status !== 0 || actual.some((value, index) => value !== expected[index])) {
    throw new Error(`pricing-aos mismatch status=${status} actual=${actual.join(",")} expected=${expected.join(",")}`);
  }
  return `pricing-aos:status=${status};out=${actual.join(",")}`;
}

function runF64Array() {
  const axpyF64 = instance.exports.axpy_f64;
  if (typeof axpyF64 !== "function") {
    throw new Error("generated WASM did not export axpy_f64");
  }
  const values = new Float64Array(memory.buffer);
  const len = 4;
  const factor = 1.25;
  const xOffset = 0;
  const yOffset = 8 * 8;
  const xIndex = xOffset / 8;
  const yIndex = yOffset / 8;
  values.set([1.0, 2.0, 3.0, 4.0], xIndex);
  values.set([0.5, 1.25, 1.25, 2.0], yIndex);
  const checksum = axpyF64(factor, xOffset, yOffset, len);
  const actual = Array.from(values.subarray(yIndex, yIndex + len));
  const expected = [1.75, 3.75, 5.0, 7.0];
  const expectedChecksum = expected.reduce((sum, value) => sum + value, 0.0);
  if (!close(checksum, expectedChecksum) || actual.some((value, index) => !close(value, expected[index]))) {
    throw new Error(`f64-array mismatch checksum=${checksum} actual=${actual.join(",")}`);
  }
  return `f64-array:checksum=${checksum};out=${actual.join(",")}`;
}

function runF64Axpy() {
  const axpyF64 = instance.exports.axpy_f64;
  if (typeof axpyF64 !== "function") {
    throw new Error("generated WASM did not export axpy_f64");
  }
  const values = new Float64Array(memory.buffer);
  const xInput = [1.0, 2.0, 3.0, 4.0];
  const yInput = [0.5, -1.0, 10.0, 20.0];
  const xIndex = 0;
  const yIndex = 8;
  values.set(xInput, xIndex);
  values.set(yInput, yIndex);
  const checksum = axpyF64(2.0, xIndex * 8, yIndex * 8, xInput.length);
  const actual = Array.from(values.subarray(yIndex, yIndex + xInput.length));
  const expected = xInput.map((value, index) => 2.0 * value + yInput[index]);
  const expectedChecksum = expected.reduce((sum, value) => sum + value, 0.0);
  if (!close(checksum, expectedChecksum) || actual.some((value, index) => !close(value, expected[index]))) {
    throw new Error(`f64-axpy mismatch checksum=${checksum} actual=${actual.join(",")}`);
  }
  return `f64-axpy:checksum=${checksum};out=${actual.join(",")}`;
}

function runF64Sum() {
  const sumF64 = instance.exports.sum_f64;
  if (typeof sumF64 !== "function") {
    throw new Error("generated WASM did not export sum_f64");
  }
  const input = [1.25, -2.5, 3.75, 4.5, 10.0];
  new Float64Array(memory.buffer).set(input, 0);
  const actual = sumF64(0, input.length);
  const expected = input.reduce((sum, value) => sum + value, 0.0);
  if (!close(actual, expected)) {
    throw new Error(`f64-sum mismatch actual=${actual} expected=${expected}`);
  }
  return `f64-sum:result=${actual};inputLength=${input.length}`;
}

function runPricingSoa() {
  const pricingSoA = instance.exports.pricing_soa;
  if (typeof pricingSoA !== "function") {
    throw new Error("generated WASM did not export pricing_soa");
  }
  const rows = [
    { price: 10000n, quantity: 2n, discount: 1000n, taxRatePpm: 82500n },
    { price: 2500n, quantity: 4n, discount: 0n, taxRatePpm: 100000n },
    { price: 1200n, quantity: 5n, discount: 500n, taxRatePpm: 100000n },
    { price: 999n, quantity: 3n, discount: 100n, taxRatePpm: 62500n }
  ];
  const values = new BigInt64Array(memory.buffer);
  const len = rows.length;
  const pricesIndex = 0;
  const quantitiesIndex = pricesIndex + len;
  const discountsIndex = quantitiesIndex + len;
  const taxRatesIndex = discountsIndex + len;
  const outIndex = taxRatesIndex + len;
  for (let index = 0; index < len; index += 1) {
    values[pricesIndex + index] = rows[index].price;
    values[quantitiesIndex + index] = rows[index].quantity;
    values[discountsIndex + index] = rows[index].discount;
    values[taxRatesIndex + index] = rows[index].taxRatePpm;
  }
  const status = pricingSoA(pricesIndex * 8, quantitiesIndex * 8, discountsIndex * 8, taxRatesIndex * 8, outIndex * 8, len);
  const actual = Array.from(values.subarray(outIndex, outIndex + len));
  const expected = rows.map((row) => expectedTotal(row.price, row.quantity, row.discount, row.taxRatePpm));
  if (status !== 0 || actual.some((value, index) => value !== expected[index])) {
    throw new Error(`pricing-soa mismatch status=${status} actual=${actual.join(",")} expected=${expected.join(",")}`);
  }
  return `pricing-soa:status=${status};out=${actual.join(",")}`;
}

function runBenchPricingHelpers() {
  if (
    instance.exports.item_subtotal !== undefined ||
    instance.exports.apply_discount !== undefined ||
    instance.exports.calc_tax !== undefined
  ) {
    throw new Error("generated benchmark helper WASM exported private helper functions");
  }
  return runPricingAos().replace("pricing-aos:", "bench-pricing-helpers:");
}

function runBenchPricingSoa() {
  return runPricingSoa().replace("pricing-soa:", "bench-pricing-soa:");
}

function runBenchF64Kernels() {
  const axpyF64 = instance.exports.axpy_f64;
  const dotF64 = instance.exports.dot_f64;
  const sumF64 = instance.exports.sum_f64;
  const scaleF64 = instance.exports.scale_f64;
  if (
    typeof axpyF64 !== "function" ||
    typeof dotF64 !== "function" ||
    typeof sumF64 !== "function" ||
    typeof scaleF64 !== "function"
  ) {
    throw new Error("generated WASM did not export the expected f64 kernels");
  }
  const values = new Float64Array(memory.buffer);
  const len = 4;
  const xInput = [1.0, -2.0, 3.5, 4.25];
  const yInput = [0.5, 8.0, -1.5, 2.25];
  const xIndex = 0;
  const yIndex = 16;
  const scaleIndex = 32;
  values.set(xInput, xIndex);
  values.set(yInput, yIndex);
  const axpyChecksum = axpyF64(1.5, xIndex * 8, yIndex * 8, len);
  const axpyActual = Array.from(values.subarray(yIndex, yIndex + len));
  const axpyExpected = xInput.map((value, index) => 1.5 * value + yInput[index]);
  const axpyExpectedChecksum = axpyExpected.reduce((sum, value) => sum + value, 0.0);
  if (
    !close(axpyChecksum, axpyExpectedChecksum) ||
    axpyActual.some((value, index) => !close(value, axpyExpected[index]))
  ) {
    throw new Error(`bench-f64-kernels axpy mismatch checksum=${axpyChecksum} out=${axpyActual.join(",")}`);
  }

  values.set(xInput, xIndex);
  values.set(yInput, yIndex);
  const dotActual = dotF64(xIndex * 8, yIndex * 8, len);
  const dotExpected = xInput.reduce((sum, value, index) => sum + value * yInput[index], 0.0);
  if (!close(dotActual, dotExpected)) {
    throw new Error(`bench-f64-kernels dot mismatch actual=${dotActual} expected=${dotExpected}`);
  }

  const sumActual = sumF64(xIndex * 8, len);
  const sumExpected = xInput.reduce((sum, value) => sum + value, 0.0);
  if (!close(sumActual, sumExpected)) {
    throw new Error(`bench-f64-kernels sum mismatch actual=${sumActual} expected=${sumExpected}`);
  }

  const scaleInput = [0.25, -1.5, 2.0, 10.0];
  values.set(scaleInput, scaleIndex);
  const scaleChecksum = scaleF64(-2.0, scaleIndex * 8, len);
  const scaleActual = Array.from(values.subarray(scaleIndex, scaleIndex + len));
  const scaleExpected = scaleInput.map((value) => -2.0 * value);
  const scaleExpectedChecksum = scaleExpected.reduce((sum, value) => sum + value, 0.0);
  if (
    !close(scaleChecksum, scaleExpectedChecksum) ||
    scaleActual.some((value, index) => !close(value, scaleExpected[index]))
  ) {
    throw new Error(`bench-f64-kernels scale mismatch checksum=${scaleChecksum} out=${scaleActual.join(",")}`);
  }

  return `bench-f64-kernels:axpy=${axpyChecksum};dot=${dotActual};sum=${sumActual};scale=${scaleChecksum};axpyOut=${axpyActual.join(",")};scaleOut=${scaleActual.join(",")}`;
}

const runners = {
  "wasm-scalar": runWasmScalar,
  "wasm-calls": runWasmCalls,
  "wasm-control-flow": runWasmControlFlow,
  "wasm-memory": runWasmMemory,
  "wasm-short-circuit": runWasmShortCircuit,
  "pricing-aos": runPricingAos,
  "f64-array": runF64Array,
  "f64-axpy": runF64Axpy,
  "f64-sum": runF64Sum,
  "pricing-soa": runPricingSoa,
  "bench-pricing-helpers": runBenchPricingHelpers,
  "bench-pricing-soa": runBenchPricingSoa,
  "bench-f64-kernels": runBenchF64Kernels
};

const runner = runners[caseName];
if (!runner) {
  throw new Error(`unknown case: ${caseName}`);
}
console.log(runner());
"#
}
