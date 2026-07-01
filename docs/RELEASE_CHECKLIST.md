# Native CKC Release Checklist

Use this checklist for releasing the Rust-built `native ckc` CLI.

## Source State

- Confirm the git worktree is clean before building release artifacts.
- Confirm `Cargo.toml` contains the intended version.
- Confirm `README.md`, `README.zh-CN.md`, `docs/native-release.md`, and
  `docs/zh-CN/native-release.md` describe the same release boundary.
- Confirm the release tag, if any, matches the Cargo version.

## Local Gate

Run:

```sh
cargo fmt --check
cargo clippy --all-targets --all-features --locked -- -D warnings
cargo test --locked
cargo build --release --locked
./target/release/ckc --help
./target/release/ckc check examples/scalar.ck
./target/release/ckc emit-mir examples/scalar.ck -O3
```

For backend smoke coverage, also run at least one emitted artifact path:

```sh
./target/release/ckc emit-c examples/pricing.ck --out build/pricing.c --header build/pricing.h
./target/release/ckc emit-wasm examples/wasm_scalar.ck --out build/scalar.wasm
./target/release/ckc emit-llvm examples/llvm_scalar.ck --out build/scalar.ll
```

## Artifact Gate

- Build release binaries on the target platform runners.
- Run `ckc --help` from each produced binary.
- Run `ckc check <smoke-file.ck>` from each produced binary.
- Package each binary as a single-platform archive.
- Generate a `SHA256` checksum for each archive.
- Upload the archive and checksum as workflow artifacts.

Expected archive names:

- `ckc-darwin-arm64.tar.gz`
- `ckc-darwin-x64.tar.gz`
- `ckc-linux-arm64.tar.gz`
- `ckc-linux-x64.tar.gz`
- `ckc-win32-arm64.zip`
- `ckc-win32-x64.zip`

## Release Publication

- For tag releases, attach all archives and checksum files to the matching
  `GitHub Release`.
- Verify every uploaded file can be downloaded.
- Verify at least one downloaded archive with:

```sh
shasum -a 256 -c ckc-linux-x64.tar.gz.sha256
```

- Unpack the downloaded archive and run:

```sh
./ckc --help
./ckc check smoke.ck
```

## Signoff

The release is ready only when:

- The strict Rust gate is green.
- The native release workflow is green.
- Every supported platform produced a binary archive.
- Every archive has a matching `SHA256` checksum.
- README and docs describe `native ckc` as the shipped product.
