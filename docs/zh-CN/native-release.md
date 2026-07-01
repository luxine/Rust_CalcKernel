# Native CKC 发布

本仓库发布 `native ckc`，也就是由 Rust 构建的命令行编译器。发布产物是每个
目标平台上的原生可执行文件，不包含脚本包装层。

## 本地门禁

创建发布产物前先运行严格检查：

```sh
cargo fmt --check
cargo clippy --all-targets --all-features --locked -- -D warnings
cargo test --locked
cargo build --release --locked
./target/release/ckc --help
```

用 release 二进制做冒烟检查：

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

## 发布产物

发布工作流会在 macOS、Linux 和 Windows runner 上构建原生 `ckc` 可执行文件。
每个归档只包含一个可执行文件，并配套生成 `SHA256` 校验文件。

预期归档名称：

- `ckc-darwin-arm64.tar.gz`
- `ckc-darwin-x64.tar.gz`
- `ckc-linux-arm64.tar.gz`
- `ckc-linux-x64.tar.gz`
- `ckc-win32-arm64.zip`
- `ckc-win32-x64.zip`

## 签核

每个目标平台 runner 都必须用构建出的二进制执行：

```sh
ckc --help
ckc check <smoke-file.ck>
```

之后 runner 会打包可执行文件，写出 `SHA256` 校验文件，并上传为工作流产物。
带标签的构建可以把同一组文件附加到 `GitHub Release`。

## 手动验证

下载归档后先验证校验和：

```sh
shasum -a 256 -c ckc-linux-x64.tar.gz.sha256
```

再解包运行：

```sh
./ckc --help
./ckc check smoke.ck
```
