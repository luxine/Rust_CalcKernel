# Rust CalcKernel

Rust CalcKernel 是 `/Users/lynn/code/CalcKernel` TypeScript `ckc` 编译器的 Rust 横向重写。目标不是重新设计语言，而是在 Rust 中复刻现有 CK / CalcKernel 的语法、类型检查、MIR、优化和 C / WASM / LLVM 后端，并逐步替换现有 TS CLI 的行为与错误输出。

## 项目定位

CK 是一个面向纯计算 kernel 的小型 DSL，适合把定价、数组处理、图算法等确定性逻辑编译成可嵌入宿主程序的产物。

当前 Rust 仓库提供：

- `calckernel` library：lexer、parser、type checker、MIR、优化 pass、后端 emitter。
- `ckc` binary：与 TS `ckc` 对齐的命令行入口。
- C backend：输出 C/H，支持 unchecked 和 checked overflow ABI，并可调用 `clang` 构建动态库。
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
- `src/typeck.rs`：符号表、Scope、类型推导/检查、TS 风格 metadata lookup helper。
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

npm 发布和 TS 包迁移矩阵见 `docs/npm-release.md`。架构审查和
TypeScript 到 Rust 的模块映射见 `docs/architecture-review.md` /
`docs/zh-CN/architecture-review.md`。

## 兼容性验证

当前测试会直接调用 TS oracle `/Users/lynn/code/CalcKernel/dist/src/cli.js`，对比 Rust `ckc` 的 stdout、stderr、退出码和生成文件。
先运行 `npm run verify:typescript-oracle`，确认只读 TS oracle checkout 和
`dist/src/cli.js` 存在且可启动，避免本地 parity 证据来自被跳过的 oracle 测试。

已覆盖的主要横向面：

- `check`、`emit-mir`、`emit-c`、`emit-wat`、`emit-wasm`、`emit-llvm`、`build`、`build-llvm`。
- lexer、parser、type checker diagnostics。
- MIR O0-O3 输出，覆盖官方 scalar、pricing、dijkstra、checked scalar、LLVM、WASM、f64-array 样例，以及 TS perf fixture 的 pricing helpers、pricing SoA、f64 kernels。
- C/header 输出，覆盖官方 scalar/pricing/dijkstra、checked scalar、LLVM/WASM 命名样例、`examples/wasm/**` f64/pricing interop 样例，`emit-c` header 写入失败时不留下 C 半成品、`build` 生成文件和 TS 兼容的 clang `-DCK_BUILD_DLL` 参数，TS perf fixture 的 O0/O2/O3 C 输出，以及 unchecked scalar/casts、checked scalar/control-flow/logical/calls、pricing unchecked/checked、perf pricing/f64 和 `tests/fixtures/f64_edges.ck` 动态库的 Python `ctypes` 宿主调用结果。
- WAT 输出和 WASM bytes，包含官方 f64-array、f64-axpy、f64-sum、pricing SoA interop 样例，TS `bench/perf/fixtures` 的 pricing helpers、pricing SoA、f64 kernels，以及 TS 兼容的无 `name` custom section 行为。
- WASM runtime interop，覆盖官方 scalar、calls、control-flow、memory、short-circuit，以及 pricing AoS、f64-array、f64-axpy、f64-sum、pricing SoA、`tests/fixtures/f64_edges.ck` 和 TS perf fixture 的 Node 宿主调用结果。
- LLVM IR 显式 target 输出，覆盖官方 scalar/pricing/dijkstra、checked scalar、LLVM/WASM 命名样例、`examples/wasm/**` f64/pricing interop 样例和 TS perf fixture；`emit-llvm` 默认 native target 探测、`build-llvm` 不默认探测 target 的 TS 行为，`tests/fixtures/f64_edges.ck` 的 NaN/Infinity/-0/f64 memory edge parity，TS perf f64 kernels 的 O3 IR，以及官方 scalar/calls/control-flow/memory/short-circuit/bool、f64 edges、perf f64 kernels 与 pricing 的 LLVM object/dynamic-library 宿主运行结果。
- npm package surface，覆盖 `calckernel`/Cargo `0.8.0` version alignment、唯一 `ckc` bin、Node wrapper 调用 Rust `ckc`、root `SourceFile`/`TokenKind`/`lex`/`parse`/`check`/type-checker helpers/`Scope`/`SymbolTable`/C backend API/diagnostic formatter/`CKWasmArena/createCKWasmArena` export、`lex` 与 TS oracle 的 token/diagnostic parity、`parse` 与 TS oracle 的 AST/diagnostic parity、`check` 与 TS oracle 的 checked-program/helper/diagnostic parity、root C header/source/files/build helper 与 TS oracle parity、平台二进制矩阵、`build:npm-matrix` 全矩阵构建/staging、`build:npm-matrix --expect-complete` 全矩阵 staging 门禁、正式 tarball package metadata、strict file-surface manifest、binary file mode/architecture/format/size/SHA256 校验和额外文件拒绝、TS 兼容的 `CKWasmArena` 错误/heap/memory.grow/typed-array 边界、生成 WASM f64 helper interop、`npm pack --dry-run --ignore-scripts` 文件面，以及真实 tarball fresh-install 后通过 `node_modules/.bin/ckc` 启动随包 Rust 二进制。
- checked WASM/LLVM rejection、invalid flag、usage errors、legacy `.ik` rejection、缺失输入文件 `ENOENT`、目录输入 `EISDIR`、非法 UTF-8 replacement 解码、UTF-16 code unit 风格的 Unicode 诊断位置/marker/public lexer token offset、direct/atomic 输出写入错误，以及父目录创建 `EEXIST`/`ENOTDIR` 错误、TS 兼容的未知 command/flag、延迟语义 flag 校验和参数错误优先级行为。
- TS oracle fixture coverage audit：只读枚举 TS checkout 的 `examples`、`bench/perf/fixtures`、`tests/fixtures` `.ck` 输入，要求现有 examples/perf fixtures 在 MIR/C/WASM/LLVM backend oracle 测试中横向覆盖。
- TS oracle test-surface audit：只读枚举 TS checkout 的 `tests/**/*.test.ts`，要求每个原始测试文件在 `docs/typescript-test-surface.json` 中有显式 Rust 迁移映射。

运行验证：

```sh
npm run verify:typescript-oracle
npm run audit:typescript-test-surface
cargo fmt --check
cargo test
cargo clippy --all-targets --all-features --locked -- -D warnings
```

## 当前边界

这不是最终完成声明。当前 Rust 实现已经有较强的 TS oracle 覆盖，会先验证
TS oracle CLI 可用，并且会自动审计 TS checkout 现有 `.ck` fixture 是否进入
Rust 横向 backend oracle 测试，以及 TS oracle 的每个 `.test.ts` 文件是否有
显式 Rust 迁移映射；完整替换还需要继续签核：

- npm 多平台二进制产物在真实目标平台的实际签核、`packageVersion` 与
  `sourceFallback: "disabled"` 的随包二进制验收、`verify:release-signoff`
  汇总验收和版本号发布面。
- 最终发布切换必须由 Rust package 自身的 release checklist、release verifier、sign-off verifier 和
  `workflow_dispatch` release workflow 证明；实际 npm registry 替换需要显式
  `publish=true`、`NPM_TOKEN`、npm provenance 发布路径，以及
  `verify:registry-replacement` 的 registry metadata 验证；
  TypeScript checkout 继续按只读 compatibility oracle 使用，不要求修改原项目。

详细阶段证据见 `docs/superpowers/specs/2026-06-29-rust-calckernel-rewrite-design.md`。
