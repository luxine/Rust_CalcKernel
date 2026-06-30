# Rust CalcKernel

[English](README.md)

Rust CalcKernel 是 `/Users/lynn/code/CalcKernel` TypeScript `ckc` 编译器的
Rust 横向重写。目标不是重新设计语言，而是在 Rust 中复刻现有 CK / CalcKernel
的语法、类型检查、MIR、优化、C / WASM / LLVM 后端、CLI 行为、错误输出和 npm
包发布面。

## 项目定位

CK 是一个面向纯计算 kernel 的小型 DSL，适合把定价、数组处理、图算法和数值
kernel 等确定性逻辑编译成可嵌入宿主程序的产物。

当前 Rust 仓库提供：

- `calckernel` library：lexer、parser、type checker、MIR、优化 pass、后端
  emitter。
- `ckc` binary：与 TS `ckc` 对齐的命令行入口。
- C backend：输出 C/H，支持 unchecked 和 checked overflow ABI，并可调用
  `clang` 构建动态库。
- WASM backend：输出 WAT 或 WASM bytes。
- LLVM backend：输出 LLVM IR，或调用 `clang` 构建动态库 / object。

## 架构

```text
.ck source
  |
  v
lexer -> parser -> type checker
  |
  v
MIR lowering -> MIR optimizer
  |
  +--> C emitter -> clang -> shared library
  +--> WAT emitter -> wasm bytes
  +--> LLVM IR emitter -> clang -> shared library / object
```

主要源码入口：

- `src/lexer/mod.rs`：tokenization、位置追踪、lexer diagnostics。
- `src/parser.rs`：AST、语句/表达式解析、parser diagnostics。
- `src/typeck.rs`：符号表、Scope、类型推导/检查、TS 风格 metadata lookup
  helper。
- `src/mir/mod.rs`：MIR 数据结构、lowering、validator、printer。
- `src/opt/mod.rs`：O0-O3 pass pipeline 和 MIR 优化 pass。
- `src/backend/mod.rs`：C、WAT/WASM、LLVM 三个后端。
- `src/main.rs`：`ckc` CLI 参数解析、文件 IO、clang 调用、TS 兼容文案。

## 使用

```sh
cargo run -- check /Users/lynn/code/CalcKernel/examples/scalar.ck
cargo run -- emit-mir /Users/lynn/code/CalcKernel/examples/scalar.ck -O3
cargo run -- emit-c /Users/lynn/code/CalcKernel/examples/pricing.ck --out /tmp/pricing.c
cargo run -- emit-wasm /Users/lynn/code/CalcKernel/examples/wasm_scalar.ck --out /tmp/scalar.wasm
cargo run -- emit-llvm /Users/lynn/code/CalcKernel/examples/llvm_scalar.ck --target ck-test-target
```

构建替代 `ckc` 二进制：

```sh
cargo build --release
./target/release/ckc --help
```

npm 发布和 TS 包迁移矩阵见 `docs/npm-release.md`。架构审查和 TypeScript 到
Rust 的模块映射见 `docs/architecture-review.md` /
`docs/zh-CN/architecture-review.md`。

## 兼容性验证

当前测试会直接调用只读 TS oracle
`/Users/lynn/code/CalcKernel/dist/src/cli.js`，对比 Rust `ckc` 的 stdout、
stderr、退出码和生成文件。先运行 `npm run verify:typescript-oracle`，确认
TS oracle checkout 和已构建 CLI 存在。

已覆盖的主要横向面：

- `check`、`emit-mir`、`emit-c`、`emit-wat`、`emit-wasm`、`emit-llvm`、
  `build`、`build-llvm`。
- lexer、parser、type checker diagnostics。
- MIR O0-O3 输出，覆盖官方样例、pricing kernel、checked scalar、WASM/LLVM
  样例、f64-array 样例和 TypeScript performance fixtures。
- C/header 输出、checked/unchecked C runtime 行为、`clang` 调用行为，以及
  Python `ctypes` 动态库宿主运行对比。
- WAT/WASM 输出、确定性 WASM bytes、f64 interop helper 和 Node 宿主运行对比。
- LLVM IR、默认 target 行为、object/dynamic-library runtime interop 和 f64
  edge 行为。
- npm package surface、root JavaScript/TypeScript API、`ckc` bin 行为、平台
  二进制矩阵 staging、正式 release tarball 校验、严格文件面检查、consumer
  install 行为和 cutover evidence scripts。
- invalid flag、usage errors、缺失输入、目录输入、非法 UTF-8 replacement 解码、
  Unicode 诊断位置、写入失败、父目录创建错误、未知 command/flag 和语义 flag
  优先级行为。
- TypeScript oracle fixture coverage audit 和 TypeScript test-surface audit，
  确认 Rust 测试持续覆盖当前 oracle 输入和原始测试文件。

运行主要本地门禁：

```sh
npm run verify:typescript-oracle
npm run audit:typescript-test-surface
npm run verify:declaration-parity
npm run verify:public-api-parity
npm run audit:release-workflow
node scripts/audit-rust-replacement-readiness.mjs
cargo fmt --check
cargo test
cargo clippy --all-targets --all-features --locked -- -D warnings
```

## 当前边界

这不是最终 cutover 完成声明。Rust 实现已经具备较广的 TypeScript oracle 覆盖和
release 自动化；但只有正式多平台 npm artifact 在真实目标平台完成签核，并且现有
TypeScript `ckc` 发布路径实际替换为 Rust package 后，完整替换才算完成。

在 cutover 完成前，TypeScript checkout 继续作为只读 source material 和
compatibility oracle。
