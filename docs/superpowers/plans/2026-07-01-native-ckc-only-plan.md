# Native CKC Only Implementation Plan

> Implementation gate: review the design in
> `docs/superpowers/specs/2026-07-01-native-ckc-only-design.md` before editing
> product files. The TypeScript checkout is read-only.

## Current Baseline

- Initial gate before this plan: `cargo fmt --check` and `cargo test --locked`
  passed.
- Initial repository surface still contained npm package metadata, npm wrappers,
  npm release scripts, npm workflow automation, npm parity tests, and npm
  release docs.
- Current completion state: the repository is native-only and the final local
  gate has passed.

## Phase 1: Native Release Surface Tests

Files:

- Modify: `tests/git_repository_test.rs` or add `tests/release_surface_test.rs`

Tasks:

- [x] Add a failing no-npm-surface test that rejects `package.json`, `npm/`,
      npm publish workflow files, and npm-specific release scripts.
- [x] Add a failing native release doc test for `docs/native-release.md` and
      `docs/zh-CN/native-release.md`.
- [x] Add a failing native workflow audit test for
      `.github/workflows/native-release.yml`.
- [x] Run focused tests and confirm they fail for the current npm surface.

## Phase 2: Remove npm and JavaScript Compatibility Layer

Files:

- Delete: `package.json`
- Delete: `npm/`
- Delete: npm-specific `scripts/*.mjs`
- Delete or replace: `.github/workflows/npm-release.yml`
- Delete: npm-specific tests under `tests/`

Tasks:

- [x] Remove npm package metadata and wrappers.
- [x] Remove JS API and TypeScript declaration parity checks.
- [x] Remove npm registry, publish, signoff, cutover, and install-smoke tests.
- [x] Keep compiler behavior oracle tests that compare CLI and backend output.
- [x] Run `cargo test --locked` and fix imports/test lists.

## Phase 3: Native Release Workflow and Guards

Files:

- Add: `.github/workflows/native-release.yml`
- Add or modify: native release surface tests
- Add: native release documentation

Tasks:

- [x] Build release binaries for supported target triples.
- [x] Smoke the native binary on each runner.
- [x] Archive binaries and SHA256 checksums.
- [x] Optionally publish artifacts to GitHub Releases for tags or explicit
      workflow dispatch.
- [x] Add tests that audit the workflow for native release commands and absence
      of npm publish behavior.

## Phase 4: Documentation Migration

Files:

- Add/modify: `docs/*.md`
- Add/modify: `docs/zh-CN/*.md`
- Modify: `README.md`
- Modify: `README.zh-CN.md`

Tasks:

- [x] Migrate TypeScript docs into Rust-native wording.
- [x] Replace npm release guidance with native release guidance.
- [x] Remove JavaScript API/package install language.
- [x] Preserve CK language, MIR, optimizer, backend, ABI, and CLI output docs.
- [x] Keep English and Simplified Chinese docs aligned.
- [x] Add doc tests for required links and bilingual coverage.

## Phase 5: Final Native CLI Verification

Commands:

```sh
cargo fmt --check
cargo clippy --all-targets --all-features --locked -- -D warnings
cargo test --locked
cargo build --release --locked
./target/release/ckc --help
./target/release/ckc check examples/scalar.ck
```

Tasks:

- [x] Run the full local gate.
- [x] Verify `git status` contains only intended Rust repository changes.
- [x] Confirm `/Users/lynn/code/CalcKernel` was not modified.
- [x] Report remaining distance to full native release readiness.
