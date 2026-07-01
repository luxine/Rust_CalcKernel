# Rust CalcKernel Native Architecture Review

This document describes the current `native ckc` architecture in
`/Users/lynn/code/Rust_CalcKernel`.

No npm. No scripting-language wrapper. No declaration parity surface. The Rust
repository ships a native command-line compiler and keeps the old source
checkout read-only as behavior reference material.

## Product Boundary

The release product is:

- `Cargo.toml` package `calckernel`
- binary target `ckc`
- Rust library modules used by the CLI and tests
- native archives built by `.github/workflows/native-release.yml`

The release product is not:

- a registry package wrapper
- a root runtime API for another language
- a platform dispatch script
- a declaration compatibility layer

## Pipeline

```text
.ck source
  -> SourceFile
  -> lexer
  -> parser / AST
  -> type checker / CheckedProgram
  -> MIR lowering
  -> MIR optimization pipeline
  -> MIR validator
  -> C backend | WAT/WASM backend | LLVM backend
  -> src/main.rs native ckc CLI
```

## Rust Module Boundaries

| Area | Rust source | Responsibility |
| --- | --- | --- |
| Source and diagnostics | `src/source.rs`, `src/diagnostics.rs` | File text, spans, line/column mapping, diagnostic formatting |
| Lexer | `src/lexer/mod.rs` | Tokens, lexer recovery, lexer diagnostics |
| Parser | `src/parser.rs` | AST, statements, expressions, parser diagnostics |
| Type checker | `src/typeck.rs` | Scopes, symbols, type validation, checked program metadata |
| MIR | `src/mir/mod.rs` | MIR data structures, lowering, validation, printing |
| Optimizer | `src/opt/mod.rs` | O0-O3 pass pipeline and safety boundaries |
| Backends | `src/backend/mod.rs` | C, WAT/WASM, and LLVM emission |
| CLI | `src/main.rs` | Argument parsing, file IO, command execution, stdout/stderr, exit codes |

## Native CLI Commands

`src/main.rs` owns all user-facing command behavior:

- `ckc check`
- `ckc emit-mir`
- `ckc emit-c`
- `ckc emit-wat`
- `ckc emit-wasm`
- `ckc emit-llvm`
- `ckc build`
- `ckc build-llvm`

The CLI uses structured Rust compiler APIs internally. It handles path errors,
atomic output writes, backend option validation, external `clang` calls, and
process exit codes at the command boundary.

## Compatibility Policy

Native-only does not loosen language compatibility. Tests still compare against
the old compiler where useful for:

- diagnostics
- stdout and stderr
- exit codes
- MIR text
- generated C/H, WAT/WASM, and LLVM output
- runtime behavior of emitted artifacts

Those tests protect compiler behavior only. They do not protect a wrapper API,
package metadata, or publication workflow from the old implementation.

## Release Architecture

The native release workflow:

1. Runs `cargo fmt --check`.
2. Runs `cargo clippy --all-targets --all-features --locked -- -D warnings`.
3. Runs `cargo test --locked`.
4. Builds `ckc` with `cargo build --release --locked`.
5. Smokes each produced binary with `ckc --help` and `ckc check`.
6. Packages one executable per archive.
7. Writes `SHA256` checksums.
8. Uploads archives and checksums, optionally attaching them to a
   `GitHub Release`.

This is the stable release surface for `native ckc`.
