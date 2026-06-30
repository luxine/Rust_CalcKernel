# Rust CalcKernel

[Simplified Chinese](README.zh-CN.md)

Rust CalcKernel is the Rust replacement for the TypeScript `ckc` compiler in
`/Users/lynn/code/CalcKernel`. The goal is not to redesign CK / CalcKernel, but
to preserve the existing language, diagnostics, CLI behavior, generated output,
and package surface while moving the implementation to Rust.

## Project Shape

CK is a small DSL for pure computation kernels. It is intended for deterministic
logic such as pricing, array processing, graph algorithms, and numerical
kernels that need embeddable C, WASM, or LLVM outputs.

This repository provides:

- `calckernel` library APIs for lexing, parsing, type checking, MIR,
  optimization passes, and backend emission.
- `ckc` binary behavior aligned with the TypeScript CLI.
- C backend output with C/H generation, unchecked and checked overflow ABI
  modes, and optional `clang` shared-library builds.
- WASM backend output as WAT or WASM bytes.
- LLVM backend output as LLVM IR, dynamic libraries, or object files through
  `clang`.

## Architecture

```text
.ck source
  |
  v
lexer -> parser -> type checker
  |
  v
MIR lowering -> MIR optimizer
  |
  +--> C emitter -> clang -> shared library
  +--> WAT emitter -> wasm bytes
  +--> LLVM IR emitter -> clang -> shared library / object
```

Primary source entry points:

- `src/lexer/mod.rs`: tokenization, source positions, and lexer diagnostics.
- `src/parser.rs`: AST, statements, expressions, and parser diagnostics.
- `src/typeck.rs`: symbol tables, scopes, type checking, and TypeScript-style
  metadata lookup helpers.
- `src/mir/mod.rs`: MIR data structures, lowering, validation, and printing.
- `src/opt/mod.rs`: O0-O3 pass pipeline and MIR optimization passes.
- `src/backend/mod.rs`: C, WAT/WASM, and LLVM backends.
- `src/main.rs`: `ckc` CLI argument parsing, file IO, `clang` calls, and
  TypeScript-compatible CLI messages.

## Usage

```sh
cargo run -- check /Users/lynn/code/CalcKernel/examples/scalar.ck
cargo run -- emit-mir /Users/lynn/code/CalcKernel/examples/scalar.ck -O3
cargo run -- emit-c /Users/lynn/code/CalcKernel/examples/pricing.ck --out /tmp/pricing.c
cargo run -- emit-wasm /Users/lynn/code/CalcKernel/examples/wasm_scalar.ck --out /tmp/scalar.wasm
cargo run -- emit-llvm /Users/lynn/code/CalcKernel/examples/llvm_scalar.ck --target ck-test-target
```

Build the replacement `ckc` binary:

```sh
cargo build --release
./target/release/ckc --help
```

The npm release and TypeScript package migration matrix live in
`docs/npm-release.md`. The architecture review and TypeScript-to-Rust module
mapping live in `docs/architecture-review.md` and
`docs/zh-CN/architecture-review.md`.

## Compatibility Verification

The test suite calls the read-only TypeScript oracle at
`/Users/lynn/code/CalcKernel/dist/src/cli.js` and compares Rust `ckc` stdout,
stderr, exit codes, and generated files. Run `npm run verify:typescript-oracle`
first to confirm that the oracle checkout and built CLI are available.

Current coverage includes:

- `check`, `emit-mir`, `emit-c`, `emit-wat`, `emit-wasm`, `emit-llvm`,
  `build`, and `build-llvm`.
- Lexer, parser, and type checker diagnostics.
- MIR O0-O3 output across official examples, pricing kernels, checked scalar
  examples, WASM and LLVM examples, f64-array examples, and TypeScript
  performance fixtures.
- C/header output, checked and unchecked C runtime behavior, `clang`
  invocation behavior, and Python `ctypes` dynamic-library hosts.
- WAT/WASM output, deterministic WASM byte behavior, f64 interop helpers, and
  Node host runtime comparisons.
- LLVM IR, default target behavior, object and dynamic-library runtime interop,
  and f64 edge behavior.
- npm package surface, root JavaScript and TypeScript APIs, `ckc` bin behavior,
  platform binary matrix staging, formal release tarball verification, strict
  file-surface checks, consumer install behavior, and cutover evidence scripts.
- Error behavior for invalid flags, usage errors, missing inputs, directory
  inputs, invalid UTF-8 replacement decoding, Unicode diagnostic positions,
  write failures, parent directory creation errors, unknown commands, unknown
  flags, and semantic flag precedence.
- TypeScript oracle fixture coverage and TypeScript test-surface audits so the
  Rust suite tracks the current oracle inputs and original test files.

Run the main local gate:

```sh
npm run verify:typescript-oracle
npm run audit:typescript-test-surface
cargo fmt --check
cargo test
cargo clippy --all-targets --all-features --locked -- -D warnings
```

## Current Boundary

This is not a final cutover claim. The Rust implementation has broad
TypeScript-oracle coverage and release automation, but the full replacement is
complete only after the formal multi-platform npm artifact is signed off on the
real target platforms and the existing TypeScript `ckc` publication path is
actually replaced by the Rust package.

The TypeScript checkout remains read-only source material and the compatibility
oracle until that cutover is complete.
