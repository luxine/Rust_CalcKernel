# Performance

[Simplified Chinese](zh-CN/PERFORMANCE.md)

This Rust-native repository keeps performance work separate from correctness.
The strict correctness gate is:

```sh
cargo test --locked
```

Performance thresholds are not part of ordinary tests because results depend on
hardware, compiler version, operating system scheduling, power state, and system
load.

## Native Benchmark Harness

Run local performance measurements with the native Cargo benchmark harness:

```sh
cargo bench --bench ckc_perf
```

Use the quick mode for a smoke check after benchmark or compiler changes:

```sh
cargo bench --bench ckc_perf -- --quick
```

The harness measures frontend checking, MIR lowering, O3 MIR optimization, and
C/WAT/WASM/LLVM emission over the manifest in
`bench/perf/cases/native-cases.tsv`. It writes fresh local results to:

- `build/perf/latest.summary.json`
- `build/perf/latest.summary.md`

Use `--case <name>` or `--task <name>` after the `--` separator to narrow a run,
for example:

```sh
cargo bench --bench ckc_perf -- --quick --case pricing --task emit-c-o3
```

## Migrated Fixtures

The migrated benchmark source fixtures live under `bench/perf/fixtures/`:

- `pricing_helpers.ck`
- `pricing_soa.ck`
- `f64_kernels.ck`

These fixtures are covered by Rust backend tests through the TypeScript-oracle
fixture coverage audit. They are useful for manual performance work and for
guarding generated C/WASM/LLVM output behavior.

## Manual Artifact Checks

Build the release binary:

```sh
cargo build --release --locked
```

Emit representative artifacts:

```sh
./target/release/ckc emit-c bench/perf/fixtures/pricing_soa.ck --out build/pricing_soa.c
./target/release/ckc emit-wasm bench/perf/fixtures/f64_kernels.ck --out build/f64_kernels.wasm
./target/release/ckc emit-llvm bench/perf/fixtures/pricing_helpers.ck --out build/pricing_helpers.ll
```

Compile generated C with the host C compiler when needed:

```sh
clang -O3 -c build/pricing_soa.c -o build/pricing_soa.o
```

## Historical Snapshots

Historical local performance snapshots copied from the source project are kept
under `docs/bench/docs/`. They provide context for prior measurements, but they
are not release criteria for `native ckc`.

Use fresh local measurements when evaluating a performance change. Always report
the machine, OS, compiler, command, workload size, and whether the result is a
native C, WASM, LLVM, or host-language baseline path.
