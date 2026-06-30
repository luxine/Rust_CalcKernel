# CalcKernel Architecture Review and Rust Rewrite Map

This document is the working architecture review for replacing
`/Users/lynn/code/CalcKernel` with the Rust implementation in
`/Users/lynn/code/Rust_CalcKernel`.

## Project Summary

CalcKernel is a small compiler for deterministic `.ck` compute kernels. The
language is intentionally narrow: no IO, strings, runtime, allocator, modules,
owned arrays, or bounds-checking runtime. Host programs own memory and call the
generated C, WASM, or LLVM-backed artifacts.

The compiler is useful when business logic such as pricing, numeric kernels,
array transforms, or graph-like calculations must be written once and embedded
in different hosts.

## Original TypeScript Architecture

The TypeScript project is organized as a compiler pipeline plus package/CLI
surface:

```text
.ck source
  -> SourceFile
  -> lexer
  -> parser / AST
  -> type checker / CheckedProgram
  -> MIR lowering
  -> MIR optimizer
  -> MIR validator
  -> C backend | WASM backend | LLVM backend
  -> ckc CLI / npm package API
```

The important TypeScript ownership boundaries are:

| Area | TypeScript source |
| --- | --- |
| CLI and package root | `src/cli.ts`, `src/index.ts` |
| Source and diagnostics | `src/source/*` |
| Lexer | `src/lexer/*` |
| Parser and AST | `src/parser/*` |
| Type checker and symbols | `src/typeck/*` |
| MIR | `src/mir/*` |
| Optimizer | `src/opt/*`, `src/optimization/*` |
| C backend | `src/backend/c/*` |
| WASM backend | `src/backend/wasm/*`, `src/wasm/*` |
| LLVM backend | `src/backend/llvm/*` |

The TypeScript checkout remains the compatibility oracle while the Rust package
is being verified. Rust tests run the TypeScript CLI and package API directly
when `/Users/lynn/code/CalcKernel/dist/src` is available.

## Rust Rewrite Architecture

The Rust project keeps the same compiler shape instead of redesigning the
language:

```text
src/
  source.rs       SourceFile, spans, line/column lookup
  diagnostics.rs  diagnostic codes and formatter
  lexer/mod.rs    tokenization and UTF-16-compatible public offsets
  parser.rs       AST and recursive-descent parser
  typeck.rs       symbols, checked program, type metadata helpers
  mir/mod.rs      MIR model, lowering, validation, printer
  opt/mod.rs      O0-O3 MIR optimization pipeline
  backend/mod.rs  C, WAT/WASM, and LLVM emitters
  main.rs         ckc CLI, file IO, clang integration, exit behavior

npm/
  ckc.js          Node bin wrapper that selects packaged Rust binary
  index.js        TypeScript-compatible root JS API shim
  index.d.ts      TypeScript declarations for the root API
  platform.js     supported npm binary matrix
```

The library owns pure compiler transformations. The binary owns side effects:
argument parsing, filesystem reads/writes, stdout/stderr, exit codes, and
external `clang` calls. The npm wrapper owns platform binary selection and the
published JavaScript compatibility surface.

## TS to Rust Rewrite Map

| Required rewrite area | Rust implementation | Current verification shape |
| --- | --- | --- |
| Lexer/parser | `src/lexer/mod.rs`, `src/parser.rs` | Unit tests plus package API parity for tokens, diagnostics, AST, and UTF-16 public offsets |
| Frontend + type checker | `src/typeck.rs` | Checker tests plus root API parity for checked program, symbol lookup, helper metadata, and diagnostics |
| Frontend + MIR + optimizer | `src/mir/mod.rs`, `src/opt/mod.rs` | MIR O0-O3 TS oracle output across official examples and perf fixtures |
| C backend | `src/backend/mod.rs`, C emission sections | C/header output parity, dijkstra C dynamic-library runtime parity, f64-array C dynamic-library runtime parity, f64-axpy/f64-sum/pricing-SoA C dynamic-library runtime parity, WASM scalar/calls/control-flow/memory/short-circuit C dynamic-library runtime parity, LLVM scalar/calls/control-flow/memory/short-circuit/bool C dynamic-library runtime parity, f64 edge fixture C dynamic-library runtime parity, C dynamic-library runtime parity, checked/unchecked ABI checks |
| WASM backend | `src/backend/mod.rs`, WAT/WASM sections | WAT text, WASM bytes, dijkstra WASM runtime parity, f64 edge fixture WASM runtime parity, Node runtime interop, package WASM helper interop |
| LLVM backend | `src/backend/mod.rs`, LLVM sections | LLVM IR parity, dijkstra LLVM object/dynamic runtime parity, f64-array LLVM object/dynamic runtime parity, f64-axpy/f64-sum/pricing-SoA LLVM object/dynamic runtime parity, object/dynamic-library runtime parity, target/clang behavior parity |
| CLI replacement | `src/main.rs`, `npm/ckc.js` | stdout/stderr/exit-code parity, error precedence, output write failures, fresh npm install smoke |
| npm package replacement | `package.json`, `npm/*`, `scripts/*` | strict file-surface verifier, binary matrix staging, target executable format and architecture checks, TypeScript declaration smoke |

## Behavioral Contracts to Preserve

- The package and command stay `calckernel` and `ckc`.
- `.ck` is the only accepted source extension; legacy `.ik` inputs are rejected.
- Diagnostics keep the TypeScript format:
  `file:line:column: error CKxxxx: message`, source line, and caret marker.
- Public token offsets and diagnostic columns preserve TypeScript-style UTF-16
  code-unit semantics for compatibility with existing JavaScript callers.
- `-O0` through `-O3` preserve the TypeScript MIR pass pipeline and conservative
  safety boundaries around checked arithmetic and strict `f64`.
- C checked and unchecked ABIs remain distinct and match TypeScript output.
- WASM uses exported linear memory and host-owned pointers. `i64` / `u64`
  values remain JavaScript `BigInt`.
- LLVM emits textual IR and delegates object/dynamic-library builds to `clang`.
- User-visible CLI behavior includes argument error precedence, ignored unknown
  long flags where TypeScript ignores them, deferred semantic flag validation,
  Node-like file read errors, and atomic output write behavior.

## Replacement Gates

The Rust implementation should be considered a replacement candidate only when
these gates are true in the current checkout:

1. `npm run verify:typescript-oracle` passes, proving the read-only TypeScript
   oracle checkout and `dist/src/cli.js` are present before local parity tests
   are trusted.
2. `cargo test` passes with TypeScript oracle tests enabled.
3. `cargo fmt --check` and
   `cargo clippy --all-targets --all-features --locked -- -D warnings` pass.
4. The TypeScript oracle fixture coverage audit passes, proving every current
   `examples`, `bench/perf/fixtures`, and `tests/fixtures` `.ck` input is
   present in MIR, C, WASM, and LLVM backend oracle tests.
5. `npm run verify:host-npm-install` passes with `CKC_BIN` unset,
   `sourceFallback: "disabled"`, and `typeSmoke: "passed"`; the host verifier
   prepares `typescript@^5.8.0` in the temporary consumer when no `tsc` is
   already available, so release sign-offs do not depend on the developer-local
   TypeScript checkout or a source checkout fallback.
6. A formal release tarball is built with all binaries from `npm/platform.js`
   staged by `npm run build:npm-matrix`, checked with
   `build:npm-matrix --expect-complete` or
   `build:npm-matrix --verify-staged --expect-complete`, and packed through
   `CKC_NPM_BINARIES_DIR`.
7. `npm run verify:npm-release -- <tarball>` passes and records tarball SHA256,
   Rust package metadata, `consumerInstallScripts: []`, binary file mode,
   binary architecture, binary format, binary size, binary SHA256s, and strict
   file-surface manifest data.
8. Each supported target platform fresh-installs the same tarball with scripts
   disabled and runs packaged `node_modules/.bin/ckc`, not a local checkout
   fallback, and its TypeScript declaration smoke passes.
9. `npm run verify:release-signoff -- release-manifest.json signoffs` passes
   against the saved `verify:host-npm-install` JSON from every supported
   platform and confirms all sign-offs used the same package version and
   tarball SHA256 with `sourceFallback: "disabled"`.
10. `npm run audit:release-workflow` passes, proving the checked-in
   `workflow_dispatch` release workflow checks out and builds the read-only
   TypeScript oracle through `typescript_oracle_repository` /
   `typescript_oracle_ref`, sets `CALCKERNEL_TS_ROOT` for parity tests, then
   builds, packs, platform-smokes, and final-signs the six-target npm matrix.
11. `npm run verify:publish-artifact -- <release-manifest.json> <dist-dir>`
   passes in the publish job before `npm publish`, proving the tarball SHA256
   still matches the signed-off release manifest.
12. Registry replacement is executed only through the workflow's gated
   `publish=true` path, which requires `NPM_TOKEN`, the `npm-production`
   environment, and `npm publish --provenance --access public` after sign-off.
13. `npm run verify:registry-replacement -- <version>` passes after publish,
   proving the npm registry metadata points at Rust `npm/` entrypoints and not
   stale TypeScript `dist/` entrypoints.
14. The Rust replacement package carries its own release checklist and
   verification scripts without requiring edits to the TypeScript checkout.

## Current Boundary

The Rust implementation has broad oracle coverage and an automated gate for
current TypeScript `.ck` fixture coverage, but the goal is not complete until
the formal multi-platform release artifact is signed off and the existing
TypeScript `ckc` publication path is actually replaced by the Rust package.
The TypeScript checkout should remain available as the oracle until that
cutover is complete; it is treated as read-only source material during this
rewrite.
