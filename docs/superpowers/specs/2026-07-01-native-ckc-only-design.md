# Native CKC Only Design

## Goal

Turn `/Users/lynn/code/Rust_CalcKernel` into the native Rust `ckc` product.
The repository should no longer ship, test, or document an npm package, a
JavaScript API, or a TypeScript package replacement layer. The TypeScript
checkout at `/Users/lynn/code/CalcKernel` remains read-only source material for
language behavior, diagnostics, examples, and documentation migration.

## Product Boundary

The shipped product is the native `ckc` command-line tool built from Rust.

In scope:

- Rust lexer, parser, type checker, MIR, optimizer, and C/WASM/LLVM backends.
- Native `ckc` CLI commands and behavior:
  `check`, `emit-mir`, `emit-c`, `emit-wat`, `emit-wasm`, `emit-llvm`, `build`,
  and `build-llvm`.
- Rust crate internals needed by the CLI and tests.
- Native binary release artifacts for macOS, Linux, and Windows target triples.
- Bilingual documentation maintained for the Rust-native project.
- Strict behavior, diagnostic, generated-output, and release-surface tests.

Out of scope:

- `package.json`, npm package metadata, npm tarballs, and npm registry publish.
- `npm/` JavaScript wrappers, platform dispatch, and JS public API exports.
- TypeScript declaration parity and JavaScript runtime API compatibility.
- npm token usage, npm provenance, npm install smokes, and npm cutover evidence.
- Browser/Node wrapper examples that exist only to exercise the old package
  surface.

## Compatibility Policy

Native-only does not mean language behavior may drift. The Rust compiler still
preserves the TypeScript compiler's source-language behavior, diagnostics,
stdout/stderr text, exit codes, MIR text, and generated backend outputs unless a
future design explicitly changes the CK language.

The old TypeScript repository should be treated as an oracle in tests where it
is useful, but not as a package release dependency. Oracle tests should validate
compiler behavior only, not npm or JavaScript API shape.

## Repository Shape

Target repository surface:

```text
Cargo.toml
Cargo.lock
README.md
README.zh-CN.md
src/
tests/
docs/
examples/
.github/workflows/
```

Expected removals:

- `/package.json`
- `/npm/`
- npm package and publish scripts under `/scripts/`
- npm-specific Rust tests under `/tests/`
- `.github/workflows/npm-release.yml`
- npm release documentation

Expected replacements:

- `docs/native-release.md`
- `docs/zh-CN/native-release.md`
- `.github/workflows/native-release.yml`
- Native release manifest/checksum verification scripts or Rust tests.
- Tests that fail if npm/JS release files are reintroduced.

## Documentation Migration

Migrate the TypeScript docs as Rust-native documentation, not as a literal copy.

Keep and rewrite:

- `LANGUAGE_SPEC.md`
- `COMPILER_ARCHITECTURE.md`
- `MIR.md`
- `OPTIMIZATION.md`
- `ABI.md`
- `WASM_ABI.md`
- `LLVM_BACKEND.md`
- `CHECKED_ARITHMETIC.md`
- `ckc-outputs.md`
- `PERFORMANCE.md`
- `RELEASE_CHECKLIST.md`
- `ROADMAP.md`
- `MIGRATION.md`
- `MIGRATION_IK_TO_CK.md`
- `NAMING_CONVENTIONS.md`

Convert release docs:

- Replace npm release instructions with native artifact build, signoff,
  checksum, and GitHub Release steps.
- Remove npm token, npm provenance, `npm pack`, and JS declaration/API parity
  requirements.

Handle examples:

- Keep `.ck` examples and native host examples such as C/Python/native WASM
  runners when they validate emitted artifacts.
- Rewrite Node-only examples into native CLI artifact examples only when they
  still teach the CK/WASM/LLVM ABI.
- Do not migrate browser/package-wrapper examples as product documentation.

Every user-facing doc that has an English version should have a matching
Simplified Chinese version.

## Test Strategy

The new strict local gate should be:

```sh
cargo fmt --check
cargo clippy --all-targets --all-features --locked -- -D warnings
cargo test --locked
target/release/ckc --help
target/release/ckc check examples/scalar.ck
```

Retain and strengthen:

- Lexer/parser/type-checker tests.
- MIR and optimizer tests.
- C/WASM/LLVM backend output and runtime tests.
- CLI behavior and diagnostic tests.
- TypeScript-oracle behavior tests that compare native compiler behavior.

Remove:

- npm package smoke tests.
- npm public API parity tests.
- TypeScript declaration parity tests.
- npm registry/publish/signoff/cutover tests.
- npm release workflow audits.

Add:

- Native release artifact surface tests.
- Native workflow audit tests.
- No-npm-surface guard tests.
- Documentation link and bilingual coverage tests.

## Release Strategy

The native release workflow should:

1. Run the strict Rust gate.
2. Build release `ckc` binaries for supported macOS, Linux, and Windows targets.
3. Smoke each binary on its runner with `--help`, `check`, and at least one
   backend emission command.
4. Package binaries as `.tar.gz` or `.zip` artifacts with SHA256 checksums.
5. Upload artifacts to GitHub Actions and optionally attach them to a GitHub
   Release when a tag is pushed or a workflow input enables publishing.

No npm token is required or valid for this release path.

## Acceptance Criteria

Implementation is not complete until all are true:

- No npm or JavaScript compatibility layer remains in the Rust repository.
- README and docs describe the Rust-native CLI product, not an npm package.
- Migrated docs exist in English and Simplified Chinese where applicable.
- Strict local gates pass.
- Native release workflow and tests protect the artifact surface.
- Original `/Users/lynn/code/CalcKernel` has not been modified.
