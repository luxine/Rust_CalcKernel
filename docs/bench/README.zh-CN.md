# Native Benchmark Suite

[English](README.md)

Rust 仓库现在带有 native Cargo benchmark suite。它不使用 TypeScript 时代的 process
runner，也不依赖 Node.js、npm、pnpm 或 hyperfine。

运行完整 benchmark harness：

```sh
cargo bench --bench ckc_perf
```

短 smoke run：

```sh
cargo bench --bench ckc_perf -- --quick
```

harness 从 `bench/perf/cases/native-cases.tsv` 读取 cases，从
`bench/perf/fixtures/` 和 `examples/` 加载 CK source，然后测量：

- frontend check
- O0 MIR lowering
- O3 MIR optimization
- O3 C backend emission
- O3 WAT/WASM backend emission
- O3 LLVM IR backend emission

它会写出：

- `build/perf/latest.summary.json`
- `build/perf/latest.summary.md`

迁移后的 CK benchmark source fixtures 位于 `bench/perf/fixtures/`：

- `pricing_helpers.ck`
- `pricing_soa.ck`
- `f64_kernels.ck`

当前 correctness 和 generated-artifact 覆盖由 Rust tests 负责：

```sh
cargo test --locked
cargo test --test c_backend_test --locked
cargo test --test wasm_backend_test --locked
cargo test --test llvm_backend_test --locked
```

手工 artifact 检查可以先构建 `native ckc`，再用迁移后的 fixtures 生成 artifact：

```sh
cargo build --release --locked
./target/release/ckc emit-c bench/perf/fixtures/pricing_soa.ck --out build/pricing_soa.c
./target/release/ckc emit-wasm bench/perf/fixtures/f64_kernels.ck --out build/f64_kernels.wasm
./target/release/ckc emit-llvm bench/perf/fixtures/pricing_helpers.ck --out build/pricing_helpers.ll
```

从 source project 搬来的历史本机性能快照保留在 `docs/bench/docs/` 下，仅作为上下文。
它们不是 CI threshold，也不应被当成跨机器性能结论。
