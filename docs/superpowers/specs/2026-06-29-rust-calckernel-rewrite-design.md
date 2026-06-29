# Rust CalcKernel Rewrite Design

## Goal

Rewrite `/Users/lynn/code/CalcKernel` as a Rust implementation in
`/Users/lynn/code/Rust_CalcKernel`, replacing the TypeScript `ckc` with a Rust
`ckc` that preserves source-language behavior, diagnostics, CLI behavior,
generated outputs, and backend ABI contracts.

## Source Project Summary

The TypeScript project is a CK / CalcKernel compiler and package. It compiles
`.ck` pure computation kernels into C, WAT/WASM, and LLVM IR. It is intentionally
narrow: no IO, strings, modules, runtime, dynamic allocation, bounds checks, or
owned arrays. Host code owns all memory.

The current compiler pipeline is:

```text
.ck source
  -> SourceFile
  -> lexer
  -> parser / AST
  -> type checker / CheckedProgram
  -> MIR lowering
  -> MIR optimization pipeline
  -> MIR validator
  -> C backend | WASM backend | LLVM backend
  -> ckc CLI outputs and build commands
```

The TypeScript checkout is the oracle for compatibility. At the start of this
rewrite, `pnpm test -- --runInBand` in `/Users/lynn/code/CalcKernel` passes 60
test files and 426 tests. Existing uncommitted documentation edits in the
TypeScript checkout must not be touched by the Rust rewrite.

## Compatibility Contract

The Rust compiler must preserve these user-visible contracts:

- Language name and CLI identity: CK / CalcKernel, `.ck` source files, `ckc`
  command. No `tk`, `tkc`, or `.tk` aliases.
- Lexer tokens, source positions, line and column behavior, and lexing recovery.
- Parser AST shape and parse diagnostics for the V0 language.
- Type checker strictness, including integer literal materialization, `f64`
  strictness, explicit `i32_to_f64` and `u32_to_f64` builtins, pointer indexing,
  struct field access, and definite return checks.
- Diagnostic data and formatting:
  `file:line:column: error CKxxxx: message`, source line, and caret range.
- MIR structure, printed MIR format, validation semantics, and pass behavior.
- Optimization levels `-O0` through `-O3`, including pass order and safety
  boundaries around checked arithmetic, short-circuiting, and strict `f64`.
- C backend output, header output, unchecked ABI, checked ABI, and build command
  behavior.
- WASM backend output and ABI: WAT generation, `.wasm` assembly, exported
  memory, deterministic WASM layout, `BigInt` for `i64` / `u64`, and `Number`
  for `f64`.
- LLVM backend output and build behavior: textual `.ll`, opaque pointers,
  target triple handling, unchecked-only backend, and external `clang` builds.
- CLI commands, flags, defaults, unsupported-mode errors, stdout/stderr text,
  exit codes, output file creation, and atomic writes where currently used.

## Rust Architecture

The Rust project is a library plus binary:

```text
src/
  lib.rs
  main.rs
  source.rs
  diagnostics.rs
  lexer/mod.rs
  parser.rs
  typeck.rs
  mir/mod.rs
  opt/mod.rs
  backend/mod.rs
tests/
  c_backend_test.rs
  checker_test.rs
  cli_test.rs
  lexer_test.rs
  llvm_backend_test.rs
  mir_test.rs
  optimizer_test.rs
  parser_test.rs
  wasm_backend_test.rs
```

The library owns compiler data structures and pure transformations. The binary
owns argument parsing, file IO, process execution, stdout/stderr, and exit
codes. Library functions return structured results and never terminate the
process.

Error handling uses typed errors and diagnostics. Public fallible operations
return `Result<T, E>` or a result struct carrying diagnostics. Production code
must avoid `unwrap()` and `expect()` except where failure is statically
impossible and documented.

## Implementation Strategy

This rewrite is phased. Each phase must produce working, testable software and
must compare against the TypeScript oracle before moving to the next layer.

1. Bootstrap Rust project and implement source spans, diagnostics, and lexer.
2. Implement parser and AST with parser diagnostics.
3. Implement frontend and type checker.
4. Implement MIR types, lowering, printer, and validator.
5. Implement MIR optimization pass manager and passes.
6. Implement C backend and checked/unchecked C ABI.
7. Implement WASM backend and WAT-to-WASM path.
8. Implement LLVM backend and `build-llvm`.
9. Replace TS `ckc` behavior with Rust `ckc`, including package/bin smoke.

Later phases may use smaller plans per subsystem. A broad all-at-once plan is
not appropriate because lexer/parser, type checking, MIR, optimizers, and three
backends each have independent oracle surfaces and failure modes.

## Verification Strategy

Every implementation phase follows test-first development:

```sh
cargo test
cargo fmt --check
cargo clippy --all-targets --all-features --locked -- -D warnings
```

Backend phases add target-specific checks:

```sh
clang -std=c11 -O3 -Wall -Wextra -Werror generated.c harness.c -o harness
ckc emit-wat input.ck --out output.wat
ckc emit-wasm input.ck --out output.wasm
ckc emit-llvm input.ck --out output.ll
ckc build-llvm input.ck --out output --kind dynamic
```

Completion requires full current-state evidence that the Rust implementation
satisfies the original objective. Passing only lexer/parser tests, or only local
unit tests, is not completion.

## Current Compatibility Evidence

The Rust checkout now includes direct TypeScript-oracle CLI tests in
`tests/cli_test.rs`. When `/Users/lynn/code/CalcKernel/dist/src/cli.js` exists
or `CALCKERNEL_TS_ROOT` points at the TypeScript checkout, those tests run both
implementations with the same arguments and compare exit code, stdout, and
stderr exactly.

Current oracle-covered CLI surfaces:

- `--help`, `check`, and `emit-mir -O3` with MIR pass/debug printing.
- `emit-mir` stdout for the official scalar/pricing/dijkstra, checked scalar,
  LLVM, WASM, and f64-array examples at O0, O1, O2, and O3.
- Successful `emit-c`, `emit-wat`, `emit-wasm`, and `emit-llvm` CLI stdout and
  stderr behavior, including `--out` messages and `emit-c` failure cleanup when
  the requested header path cannot be created.
- Lexer, parser, and type-checker diagnostics through `ckc check`.
- Usage errors: no command, unknown command, missing input, too many inputs,
  missing `--out`, missing long/short flag values, TS-compatible handling for
  unknown flags, deferred semantic validation for unused flags, command
  argument error precedence, missing input-file `ENOENT` text, directory-input
  `EISDIR` text, and Node-compatible replacement decoding for invalid UTF-8
  source bytes.
- Diagnostics for Unicode source text, including TypeScript-compatible UTF-16
  code unit columns for non-BMP unknown characters and multi-line caret marker
  widths on lines containing CJK or emoji text.
- Public lexer token and diagnostic offsets after non-BMP characters, matching
  the TypeScript API's UTF-16 code unit offset semantics while preserving Rust
  byte offsets internally for source slicing.
- Direct and atomic output write errors, including TS-compatible direct
  `open` failures for `emit-mir` and atomic temp-file `rename` failures for
  WAT/LLVM/WASM/C-style output paths.
- Parent directory creation failures for output paths, including TS-compatible
  `EEXIST` and `ENOTDIR` `mkdir` diagnostics across direct and atomic output
  paths.
- Public error cases: checked WASM/LLVM rejection, invalid optimization level,
  invalid overflow mode, and legacy `.ik` extension rejection.
- `build-llvm` missing-clang diagnostics, including the TS-compatible
  `emit-llvm` fallback hint.
- `build` stdout/stderr/status, generated C/header files, and the
  TS-compatible `-DCK_BUILD_DLL` clang define for unchecked and checked C
  shared-library builds.
- npm package surface for `calckernel`, including the single `ckc` bin mapping,
  Cargo/npm `0.8.0` version alignment, Node wrapper dispatch to the Rust
  binary, root `SourceFile`, `TokenKind`, `lex`, `parse`, `check`, type-checker
  helpers, `Scope`, `SymbolTable`, C backend helpers, diagnostic formatter,
  `CKWasmArena`, and `createCKWasmArena` exports, TypeScript declarations,
  `lex` token/diagnostic parity, `parse` AST/diagnostic parity, `check`
  checked-program/helper/diagnostic parity, and root C backend API parity
  against the TypeScript package oracle, shared
  `npm/platform.js` target matrix,
  `docs/npm-release.md` release/migration checklist, all-target
  `CKC_NPM_BINARIES_DIR` staging coverage, formal tarball manifest/SHA256
  verification, TypeScript test-surface migration mapping for every current
  `tests/**/*.test.ts` file in the oracle checkout, and
  `npm pack --dry-run --ignore-scripts` file coverage.
- npm release workflow portability, including `workflow_dispatch` inputs for
  the TypeScript oracle repository/ref, checkout/build of that oracle before
  Rust tests, `CALCKERNEL_TS_ROOT` propagation to oracle-dependent tests and
  parity scripts, and an audit that rejects direct local fixture-path joins in
  Rust oracle tests.
- npm `CKWasmArena` helper boundary behavior, including TS-compatible repair
  hints for invalid memory, heap base, allocation, alignment, pointer, length,
  memory.grow, and typed-array input errors; heap-base precedence; view refresh
  after memory growth; JS-owned `copyOutF64`; and generated Rust WASM f64 read,
  sum, axpy, NaN, Infinity, and negative-zero interop through the package root
  helper.
- npm release-install smoke coverage for a real `npm pack` tarball, including
  producer-side `prepack` binary embedding, `postpack` cleanup, consumer-side
  `npm install --ignore-scripts`, `node_modules/.bin/ckc --help` without
  `CKC_BIN`, and installed root API import.
- Type-checker lookup helpers and `Scope`/`SymbolTable`-style symbol metadata
  for expression, let, struct, field, function, and variable lookup.

Current oracle-covered backend generated outputs:

- WAT output and WASM bytes for the official scalar/pricing/dijkstra/checked
  scalar/WASM/f64-array/f64-axpy/f64-sum/pricing-SoA examples, plus the TS
  `bench/perf/fixtures` pricing helpers, pricing SoA, and f64 kernels,
  including TS-compatible omission of the debug `name` custom section.
- WASM runtime interop through Node for the official scalar, calls,
  control-flow, memory, and short-circuit examples, plus pricing AoS,
  f64-array, f64-axpy, f64-sum, pricing SoA, and the TS `bench/perf/fixtures`
  pricing helpers, pricing SoA, and f64 kernels, comparing TypeScript-emitted
  and Rust-emitted module behavior.
- C/header output for `examples/scalar.ck`, `examples/explicit_casts.ck`,
  `examples/pricing.ck`, `examples/dijkstra.ck`, `examples/scalar_checked.ck`,
  `examples/scalar_control_checked.ck`, `examples/scalar_calls_checked.ck`, and
  `examples/scalar_logical_checked.ck`, plus the f64-array WASM example and TS
  `bench/perf/fixtures` pricing helpers at O0/O2, pricing SoA at O3, and f64
  kernels at O3.
- C dynamic-library runtime interop through Python `ctypes` for unchecked
  scalar/casts, checked scalar/control-flow/logical/calls, and unchecked plus
  checked pricing examples, plus TS `bench/perf/fixtures` pricing helpers at
  O0/O2, pricing SoA at O3, and f64 kernels at O3, comparing TypeScript-built
  and Rust-built library behavior.
- LLVM output with explicit target for the official scalar/pricing/dijkstra,
  checked scalar, LLVM, f64-array examples, and
  `tests/fixtures/f64_edges.ck`, including TS-compatible unordered f64
  not-equal (`fcmp une`) semantics for NaN, plus the TS `bench/perf/fixtures`
  f64 kernels at O3.
- LLVM CLI default target detection for `examples/scalar.ck`.
- `build-llvm --kind object` generated `.ll` parity for `examples/scalar.ck`,
  including the TS behavior that it does not infer a default target triple.
- LLVM object runtime interop for the official scalar, calls, control-flow,
  memory, short-circuit, bool, f64 edge fixture, TS perf f64 kernels, and
  pricing examples, linking TypeScript-built and Rust-built objects with the
  same C harnesses and comparing runtime output.
- LLVM dynamic-library runtime interop for the official scalar, calls,
  control-flow, memory, short-circuit, bool, f64 edge fixture, TS perf f64
  kernels, and pricing examples, loading the TypeScript-built and Rust-built
  shared libraries with Python `ctypes` hosts and comparing runtime output.

The latest verified Rust gate is:

```sh
cargo fmt --check
cargo test
cargo clippy --all-targets --all-features --locked -- -D warnings
```

At this point the suite has 138 Rust tests. This is substantial coverage but not
completion of the full replacement objective; broader generated-output/runtime
parity fixtures plus real target-platform npm binary sign-off runs are still
required.
