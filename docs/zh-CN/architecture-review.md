# CalcKernel 架构审查与 Rust 重写映射

本文是将 `/Users/lynn/code/CalcKernel` 替换为
`/Users/lynn/code/Rust_CalcKernel` Rust 实现的工作架构审查。

## 项目概览

CalcKernel 是一个面向确定性 `.ck` 计算 kernel 的小型编译器。语言边界刻意
保持很窄：没有 IO、字符串、运行时、allocator、模块、owned array，也没有
运行时 bounds check。宿主程序拥有内存，并调用生成的 C、WASM 或 LLVM 产物。

这个项目适合把定价、数值 kernel、数组处理、图算法等业务逻辑写一次，然后嵌入
不同宿主环境。

## 原 TypeScript 架构

TypeScript 项目由编译器 pipeline、CLI 和 npm package surface 组成：

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

主要 TypeScript 责任边界如下：

| 区域 | TypeScript source |
| --- | --- |
| CLI 和 package root | `src/cli.ts`, `src/index.ts` |
| Source 与 diagnostics | `src/source/*` |
| Lexer | `src/lexer/*` |
| Parser 与 AST | `src/parser/*` |
| Type checker 与 symbols | `src/typeck/*` |
| MIR | `src/mir/*` |
| Optimizer | `src/opt/*`, `src/optimization/*` |
| C backend | `src/backend/c/*` |
| WASM backend | `src/backend/wasm/*`, `src/wasm/*` |
| LLVM backend | `src/backend/llvm/*` |

在 Rust 包完成发布签核前，TypeScript checkout 仍作为 compatibility oracle。
当 `/Users/lynn/code/CalcKernel/dist/src` 存在时，Rust 测试会直接调用
TypeScript CLI 和 package API 做横向对比。

## Rust 重写架构

Rust 项目保持同一条编译链路，不重新设计语言：

```text
src/
  source.rs       SourceFile、span、line/column 查询
  diagnostics.rs  diagnostic code 与 formatter
  lexer/mod.rs    tokenization 和 UTF-16-compatible public offsets
  parser.rs       AST 与 recursive-descent parser
  typeck.rs       symbols、checked program、type metadata helpers
  mir/mod.rs      MIR model、lowering、validation、printer
  opt/mod.rs      O0-O3 MIR optimization pipeline
  backend/mod.rs  C、WAT/WASM、LLVM emitters
  main.rs         ckc CLI、file IO、clang integration、exit behavior

npm/
  ckc.js          选择随包 Rust binary 的 Node bin wrapper
  index.js        TypeScript-compatible root JS API shim
  index.d.ts      root API 的 TypeScript declarations
  platform.js     supported npm binary matrix
```

Library 负责纯编译转换。Binary 负责副作用：参数解析、文件读写、stdout/stderr、
exit code 和外部 `clang` 调用。npm wrapper 负责平台二进制选择和发布后的
JavaScript compatibility surface。

## TS 到 Rust 的重写映射

| 必须重写的区域 | Rust 实现 | 当前验证形态 |
| --- | --- | --- |
| Lexer/parser | `src/lexer/mod.rs`, `src/parser.rs` | 单元测试，以及 token、diagnostic、AST、UTF-16 public offset 的 package API parity |
| Frontend + type checker | `src/typeck.rs` | Checker tests，以及 checked program、symbol lookup、helper metadata、diagnostics 的 root API parity |
| Frontend + MIR + optimizer | `src/mir/mod.rs`, `src/opt/mod.rs` | 官方 examples 和 perf fixtures 的 MIR O0-O3 TS oracle 输出对比 |
| C backend | `src/backend/mod.rs` 的 C emission 部分 | C/header 输出 parity、dijkstra C dynamic-library runtime parity、f64-array C dynamic-library runtime parity、f64-axpy/f64-sum/pricing-SoA C dynamic-library runtime parity、WASM scalar/calls/control-flow/memory/short-circuit C dynamic-library runtime parity、f64 edge fixture C dynamic-library runtime parity、C dynamic-library runtime parity、checked/unchecked ABI checks |
| WASM backend | `src/backend/mod.rs` 的 WAT/WASM 部分 | WAT text、WASM bytes、dijkstra WASM runtime parity、f64 edge fixture WASM runtime parity、Node runtime interop、package WASM helper interop |
| LLVM backend | `src/backend/mod.rs` 的 LLVM 部分 | LLVM IR parity、dijkstra LLVM object/dynamic runtime parity、f64-array LLVM object/dynamic runtime parity、f64-axpy/f64-sum/pricing-SoA LLVM object/dynamic runtime parity、object/dynamic-library runtime parity、target/clang behavior parity |
| CLI replacement | `src/main.rs`, `npm/ckc.js` | stdout/stderr/exit-code parity、error precedence、output write failures、fresh npm install smoke |
| npm package replacement | `package.json`, `npm/*`, `scripts/*` | strict file-surface verifier、binary matrix staging、target executable format 与 architecture checks、TypeScript declaration smoke |

## 必须保持的行为合同

- package 和命令保持 `calckernel` / `ckc`。
- `.ck` 是唯一接受的源码扩展名；legacy `.ik` 输入必须被拒绝。
- diagnostic 格式保持 TypeScript 风格：
  `file:line:column: error CKxxxx: message`、source line 和 caret marker。
- public token offset 和 diagnostic column 继续使用 TypeScript-compatible
  UTF-16 code unit 语义，以兼容现有 JavaScript 调用方。
- `-O0` 到 `-O3` 保持 TypeScript MIR pass pipeline，以及 checked arithmetic
  和 strict `f64` 周围的保守安全边界。
- C checked ABI 和 unchecked ABI 保持分离，并与 TypeScript 输出匹配。
- WASM 使用 exported linear memory 和 host-owned pointers。`i64` / `u64`
  在 JavaScript 侧仍是 `BigInt`。
- LLVM 输出 textual IR，并把 object/dynamic-library build 委托给 `clang`。
- CLI 用户可见行为包括参数错误优先级、TypeScript 忽略的 unknown long flags、
  deferred semantic flag validation、Node-like file read errors，以及 atomic
  output write behavior。

## 替换签核门槛

只有当前 checkout 满足以下条件时，Rust 实现才应被视为 replacement candidate：

1. `npm run verify:typescript-oracle` 通过，证明只读 TypeScript oracle
   checkout 和 `dist/src/cli.js` 存在，然后才能信任本地 parity tests。
2. `cargo test` 在 TypeScript oracle tests 启用时通过。
3. `cargo fmt --check` 和
   `cargo clippy --all-targets --all-features --locked -- -D warnings` 通过。
4. TypeScript oracle fixture coverage audit 通过，证明当前 `examples`、
   `bench/perf/fixtures` 和 `tests/fixtures` 的 `.ck` 输入都已经进入
   MIR、C、WASM、LLVM backend oracle 测试。
5. `npm run verify:host-npm-install` 在 `CKC_BIN` unset、
   `sourceFallback: "disabled"` 且 `typeSmoke: "passed"` 的情况下通过；如果没有
   现成 `tsc`，host verifier 会在临时 consumer 中准备 `typescript@^5.8.0`，因此
   release sign-off 不依赖开发机本地 TypeScript checkout，也不依赖 source checkout
   fallback。
6. 正式 release tarball 使用 `npm run build:npm-matrix` staging
   `npm/platform.js` 里的全部二进制，通过
   `build:npm-matrix --expect-complete` 或
   `build:npm-matrix --verify-staged --expect-complete` 检查完整性，并通过
   `CKC_NPM_BINARIES_DIR` 打包。
7. `npm run verify:npm-release -- <tarball>` 通过，并记录 tarball SHA256、
   Rust package metadata、`consumerInstallScripts: []`、每个 binary 的 file mode、
   architecture、格式、大小、SHA256 和 strict file-surface manifest 数据。
8. 每个支持平台都 fresh-install 同一个 tarball，关闭 install scripts，并运行随包的
   `node_modules/.bin/ckc`，不能依赖本地 checkout fallback，并且 TypeScript
   declaration smoke 必须通过。
9. `npm run verify:release-signoff -- release-manifest.json signoffs` 对每个
   支持平台保存的 `verify:host-npm-install` JSON 通过，并确认所有签核使用同一个
   package version 和 tarball SHA256，且 `sourceFallback: "disabled"`。
10. `npm run audit:release-workflow` 通过，证明 checked-in
   `workflow_dispatch` release workflow 会通过 `typescript_oracle_repository` /
   `typescript_oracle_ref` checkout 并构建只读 TypeScript oracle，为 parity
   tests 设置 `CALCKERNEL_TS_ROOT`，然后构建、打包、平台 smoke，并最终签核
   六目标 npm matrix。
11. publish job 在 `npm publish` 前必须通过
   `npm run verify:publish-artifact -- <release-manifest.json> <dist-dir>`，
   证明即将发布的 tarball SHA256 仍然匹配已签核的 release manifest。
12. registry 替换只能通过 workflow 里 gated `publish=true` 路径执行；它要求
   `NPM_TOKEN`、`npm-production` environment，并在 sign-off 后使用
   `npm publish --provenance --access public`。
13. 发布后 `npm run verify:registry-replacement -- <version>` 通过，证明 npm
   registry metadata 指向 Rust `npm/` entrypoints，而不是旧 TypeScript `dist/`
   entrypoints。
14. Rust replacement package 自带 release checklist 和 verification scripts，
   不要求修改 TypeScript checkout。

## 当前边界

Rust 实现已经有较广的 oracle 覆盖，并且有自动门禁确认当前 TypeScript `.ck`
fixture 已进入横向 backend oracle 测试；但只有正式多平台 release artifact 完成签核、
并且现有 TypeScript `ckc` 发布路径实际替换为 Rust package 后，整个目标才算完成。
在 cutover 完成前，TypeScript checkout 继续作为 compatibility oracle，并在本次
重写过程中按只读 source material 处理。
