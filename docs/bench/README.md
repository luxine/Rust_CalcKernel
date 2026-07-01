# Native Benchmark Suite

[Simplified Chinese](README.zh-CN.md)

The Rust repository carries a native Cargo benchmark suite. It does not use the
old TypeScript-era process runner, Node.js, npm, pnpm, or hyperfine.

Run the benchmark harness with:

```sh
cargo bench --bench ckc_perf
```

For a short local smoke run:

```sh
cargo bench --bench ckc_perf -- --quick
```

The harness reads cases from `bench/perf/cases/native-cases.tsv`, loads CK
sources from `bench/perf/fixtures/` and `examples/`, then measures:

- frontend check
- MIR lowering at O0
- MIR optimization at O3
- C backend emission at O3
- WAT and WASM backend emission at O3
- LLVM IR backend emission at O3

It writes:

- `build/perf/latest.summary.json`
- `build/perf/latest.summary.md`

The migrated CK benchmark source fixtures under `bench/perf/fixtures/` are:

- `pricing_helpers.ck`
- `pricing_soa.ck`
- `f64_kernels.ck`

Current correctness and generated-artifact coverage is handled by Rust tests:

```sh
cargo test --locked
cargo test --test c_backend_test --locked
cargo test --test wasm_backend_test --locked
cargo test --test llvm_backend_test --locked
```

For manual artifact checks, build `native ckc` and emit artifacts from the
migrated fixtures:

```sh
cargo build --release --locked
./target/release/ckc emit-c bench/perf/fixtures/pricing_soa.ck --out build/pricing_soa.c
./target/release/ckc emit-wasm bench/perf/fixtures/f64_kernels.ck --out build/f64_kernels.wasm
./target/release/ckc emit-llvm bench/perf/fixtures/pricing_helpers.ck --out build/pricing_helpers.ll
```

Historical local performance snapshots copied from the source project are kept
under `docs/bench/docs/` for context only. They are not CI thresholds and should
not be treated as cross-machine performance claims.
