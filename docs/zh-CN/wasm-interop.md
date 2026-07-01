# WASM Interop

[English](../wasm-interop.md)

本文说明如何使用 `native ckc` 生成的 WASM artifact。

## 生成 Module

```sh
cargo build --release --locked
./target/release/ckc emit-wasm examples/wasm_scalar.ck --out build/scalar.wasm
./target/release/ckc emit-wat examples/wasm_scalar.ck --out build/scalar.wat
```

`emit-wasm` 生成 WebAssembly binary。运行该 binary 需要 host environment 提供
WebAssembly runtime。

## Memory Model

生成的 module 使用 caller-owned memory。host 在 exported linear memory 中选择
offset，写入 input buffer，调用 CK function，再读取 output buffer。CK 不提供
allocator 或 runtime。

host 侧规则：

- `ptr<T>` value 是 byte offset
- 遵守 `docs/WASM_ABI.md` 中的 layout
- 使用 little-endian read/write
- homogeneous array 优先使用 typed-array view
- mixed-width struct 和 byte-level test 使用 `DataView`
- `memory.grow` 后重新创建 view

## 验证

```sh
cargo test --test wasm_backend_test --locked
```

测试套件检查 WAT text、WASM bytes、ABI layout、f64 behavior、fixture coverage，
以及生成 artifact 的 runtime behavior。
