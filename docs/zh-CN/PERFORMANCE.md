# 性能

[English](../PERFORMANCE.md)

这个 Rust-native 仓库把性能工作和 correctness 分开。严格 correctness gate 是：

```sh
cargo test --locked
```

性能阈值不属于普通测试，因为结果依赖硬件、compiler version、OS scheduling、
电源状态和系统负载。

## Native Benchmark Harness

本机性能测量使用 native Cargo benchmark harness：

```sh
cargo bench --bench ckc_perf
```

修改 benchmark 或 compiler 后，可以先跑 quick smoke：

```sh
cargo bench --bench ckc_perf -- --quick
```

harness 会根据 `bench/perf/cases/native-cases.tsv` 测量 frontend check、MIR
lowering、O3 MIR optimization，以及 C/WAT/WASM/LLVM emission。结果写到：

- `build/perf/latest.summary.json`
- `build/perf/latest.summary.md`

可以在 `--` 后使用 `--case <name>` 或 `--task <name>` 缩小范围，例如：

```sh
cargo bench --bench ckc_perf -- --quick --case pricing --task emit-c-o3
```

## 已迁移 Fixtures

迁移后的 benchmark source fixtures 位于 `bench/perf/fixtures/`：

- `pricing_helpers.ck`
- `pricing_soa.ck`
- `f64_kernels.ck`

这些 fixtures 通过 Rust backend tests 和 TypeScript-oracle fixture coverage audit
获得覆盖。它们适合手工性能工作，也用于保护生成的 C/WASM/LLVM output behavior。

## 手工 Artifact 检查

构建 release binary：

```sh
cargo build --release --locked
```

生成代表性 artifacts：

```sh
./target/release/ckc emit-c bench/perf/fixtures/pricing_soa.ck --out build/pricing_soa.c
./target/release/ckc emit-wasm bench/perf/fixtures/f64_kernels.ck --out build/f64_kernels.wasm
./target/release/ckc emit-llvm bench/perf/fixtures/pricing_helpers.ck --out build/pricing_helpers.ll
```

如需继续检查 generated C，可以用 host C compiler 编译：

```sh
clang -O3 -c build/pricing_soa.c -o build/pricing_soa.o
```

## 历史快照

从 source project 搬来的历史本机性能快照保留在 `docs/bench/docs/` 下。它们只提供
历史上下文，不是 `native ckc` release criteria。

评估性能变更时应重新测量，并报告 machine、OS、compiler、command、workload size，
以及结果属于 native C、WASM、LLVM 还是 host-language baseline path。
