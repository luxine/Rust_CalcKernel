# Native CKC 发布检查清单

本清单用于发布 Rust 构建的 `native ckc` CLI。

## 源码状态

- 构建发布产物前确认 git worktree 干净。
- 确认 `Cargo.toml` 包含目标版本。
- 确认 `README.md`、`README.zh-CN.md`、`docs/native-release.md` 和
  `docs/zh-CN/native-release.md` 描述的是同一个发布边界。
- 如果使用 release tag，确认 tag 与 Cargo version 一致。

## 本地门禁

运行：

```sh
cargo fmt --check
cargo clippy --all-targets --all-features --locked -- -D warnings
cargo test --locked
cargo build --release --locked
./target/release/ckc --help
./target/release/ckc check examples/scalar.ck
./target/release/ckc emit-mir examples/scalar.ck -O3
```

backend 冒烟检查至少覆盖一个生成产物路径：

```sh
./target/release/ckc emit-c examples/pricing.ck --out build/pricing.c --header build/pricing.h
./target/release/ckc emit-wasm examples/wasm_scalar.ck --out build/scalar.wasm
./target/release/ckc emit-llvm examples/llvm_scalar.ck --out build/scalar.ll
```

## 产物门禁

- 在目标平台 runner 上构建 release binary。
- 用每个产出的 binary 运行 `ckc --help`。
- 用每个产出的 binary 运行 `ckc check <smoke-file.ck>`。
- 将每个平台的 binary 打成单平台归档。
- 为每个归档生成 `SHA256` checksum。
- 将归档和 checksum 上传为 workflow artifact。

预期归档名称：

- `ckc-darwin-arm64.tar.gz`
- `ckc-darwin-x64.tar.gz`
- `ckc-linux-arm64.tar.gz`
- `ckc-linux-x64.tar.gz`
- `ckc-win32-arm64.zip`
- `ckc-win32-x64.zip`

## 发布

- tag release 将所有归档和 checksum 文件附加到对应的 `GitHub Release`。
- 确认每个上传文件都可以下载。
- 至少下载一个归档并验证：

```sh
shasum -a 256 -c ckc-linux-x64.tar.gz.sha256
```

- 解包下载的归档并运行：

```sh
./ckc --help
./ckc check smoke.ck
```

## 签核

只有满足以下条件才可以认为 release ready：

- 严格 Rust 门禁为 green。
- native release workflow 为 green。
- 每个支持平台都产出了 binary archive。
- 每个 archive 都有匹配的 `SHA256` checksum。
- README 和 docs 都明确 `native ckc` 是发布产品。
