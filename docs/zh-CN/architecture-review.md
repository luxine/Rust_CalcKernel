# Rust CalcKernel 原生架构审查

本文描述 `/Users/lynn/code/Rust_CalcKernel` 当前的 `native ckc` 架构。

No npm。没有脚本语言包装层。没有 declaration parity surface。Rust 仓库发布的是
原生命令行编译器，旧 source checkout 只作为只读行为参考材料。

## 产品边界

发布产品是：

- `Cargo.toml` package `calckernel`
- binary target `ckc`
- CLI 和测试使用的 Rust library modules
- `.github/workflows/native-release.yml` 构建出的原生归档

发布产品不是：

- registry package wrapper
- 其他语言的 root runtime API
- platform dispatch script
- declaration compatibility layer

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

## Rust 模块边界

| 区域 | Rust source | 职责 |
| --- | --- | --- |
| Source 和 diagnostics | `src/source.rs`, `src/diagnostics.rs` | 文件文本、span、line/column 映射、diagnostic formatting |
| Lexer | `src/lexer/mod.rs` | token、lexer recovery、lexer diagnostics |
| Parser | `src/parser.rs` | AST、statement、expression、parser diagnostics |
| Type checker | `src/typeck.rs` | scope、symbol、type validation、checked program metadata |
| MIR | `src/mir/mod.rs` | MIR 数据结构、lowering、validation、printing |
| Optimizer | `src/opt/mod.rs` | O0-O3 pass pipeline 和 safety boundary |
| Backends | `src/backend/mod.rs` | C、WAT/WASM 和 LLVM emission |
| CLI | `src/main.rs` | 参数解析、文件 IO、command execution、stdout/stderr、exit code |

## 原生 CLI 命令

`src/main.rs` 负责所有用户可见命令行为：

- `ckc check`
- `ckc emit-mir`
- `ckc emit-c`
- `ckc emit-wat`
- `ckc emit-wasm`
- `ckc emit-llvm`
- `ckc build`
- `ckc build-llvm`

CLI 内部调用结构化 Rust compiler API。路径错误、atomic output write、backend option
validation、外部 `clang` 调用和 process exit code 都在 command boundary 处理。

## 兼容策略

native-only 不代表语言行为可以漂移。测试仍在有价值的地方对照旧编译器：

- diagnostics
- stdout 和 stderr
- exit code
- MIR text
- 生成的 C/H、WAT/WASM 和 LLVM output
- 生成 artifact 的 runtime behavior

这些测试只保护 compiler behavior，不保护旧实现的 wrapper API、package metadata 或
publication workflow。

## 发布架构

native release workflow：

1. 运行 `cargo fmt --check`。
2. 运行 `cargo clippy --all-targets --all-features --locked -- -D warnings`。
3. 运行 `cargo test --locked`。
4. 使用 `cargo build --release --locked` 构建 `ckc`。
5. 用 `ckc --help` 和 `ckc check` 冒烟测试每个二进制。
6. 每个 archive 只打包一个 executable。
7. 写出 `SHA256` checksum。
8. 上传 archive 和 checksum，并可附加到 `GitHub Release`。

这是 `native ckc` 的稳定发布表面。
