# WASM Interop

[Simplified Chinese](zh-CN/wasm-interop.md)

This document explains how to use WASM artifacts emitted by `native ckc`.

## Generate a Module

```sh
cargo build --release --locked
./target/release/ckc emit-wasm examples/wasm_scalar.ck --out build/scalar.wasm
./target/release/ckc emit-wat examples/wasm_scalar.ck --out build/scalar.wat
```

`emit-wasm` produces a WebAssembly binary. It requires a WebAssembly runtime
supplied by the host environment.

## Memory Model

Generated modules use caller-owned memory. The host chooses offsets in exported
linear memory, writes input buffers, calls CK functions, and reads output
buffers. CK does not ship an allocator or runtime.

Use these host-side rules:

- keep `ptr<T>` values as byte offsets
- respect the layout in `docs/WASM_ABI.md`
- use little-endian reads and writes
- prefer typed-array views for homogeneous arrays
- use `DataView` for mixed-width structs and byte-level tests
- recreate views after `memory.grow`

## Verification

```sh
cargo test --test wasm_backend_test --locked
```

The test suite checks WAT text, WASM bytes, ABI layout, f64 behavior, fixture
coverage, and runtime behavior for generated artifacts.
