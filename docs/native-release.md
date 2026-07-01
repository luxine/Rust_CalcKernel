# Native CKC Release

This repository ships `native ckc`, a Rust-built command-line compiler. The
release artifact is a native executable for each supported platform, not a
language package wrapper.

## Local Gate

Run the strict local checks before creating release artifacts:

```sh
cargo fmt --check
cargo clippy --all-targets --all-features --locked -- -D warnings
cargo test --locked
cargo build --release --locked
./target/release/ckc --help
```

Use the release binary for smoke checks:

```sh
tmp_ck="$(mktemp "${TMPDIR:-/tmp}/ckc-smoke.XXXXXX.ck")"
cat > "$tmp_ck" <<'CK'
export fn add(a: i32, b: i32) -> i32 {
  return a + b;
}
CK
./target/release/ckc check "$tmp_ck"
./target/release/ckc emit-mir "$tmp_ck" -O3
```

## Artifacts

The release workflow builds native `ckc` executables for macOS, Linux, and
Windows runners. Each artifact archive includes exactly one executable and a
matching `SHA256` checksum file.

Expected archive names:

- `ckc-darwin-arm64.tar.gz`
- `ckc-darwin-x64.tar.gz`
- `ckc-linux-arm64.tar.gz`
- `ckc-linux-x64.tar.gz`
- `ckc-win32-arm64.zip`
- `ckc-win32-x64.zip`

## Signoff

Each platform runner must execute the built binary with:

```sh
ckc --help
ckc check <smoke-file.ck>
```

The runner then packages the executable, writes the `SHA256` checksum, and
uploads both files as workflow artifacts. Tagged builds may attach the same
files to a `GitHub Release`.

## Manual Verification

After downloading an archive, verify the checksum first:

```sh
shasum -a 256 -c ckc-linux-x64.tar.gz.sha256
```

Then unpack and run:

```sh
./ckc --help
./ckc check smoke.ck
```
