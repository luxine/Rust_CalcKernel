# Rust_CalcKernel 架构审查报告

审查日期：2026-07-01

范围：`/Users/lynn/code/Rust_CalcKernel` 当前 `native ckc` 架构。旧 TypeScript
项目 `/Users/lynn/code/CalcKernel` 仅作为只读行为参考，不修改。

本文件是 AI 分析报告，按 `AGENTS.md` 放在 `Ai_repository/`，不作为用户发布文档。

## 总体结论

项目作为聚焦的原生编译器重写，架构方向成立。它有清晰的编译流水线、很小的依赖面、
覆盖面很强的兼容测试、明确的原生发布边界，并且在 backend emission 前有 MIR
validation gate。这些都是好的工程选择。

当前主要弱点不是正确性测试，而是维护压力。几个模块已经变成较大的职责聚合：

- `src/backend/mod.rs`：同时包含 C、WAT/WASM、LLVM、ABI helper、layout 逻辑和
  backend-specific optimization。
- `src/opt/mod.rs`：pass manager 和所有 optimization pass 都在一个文件里。
- `src/mir/mod.rs`：IR data model、lowering、printer、validator 都在一个文件里。
- `src/main.rs`：argument parsing、command routing、file IO、clang execution、
  output formatting、exit behavior 都在一个文件里。

以当前规模看仍可接受，但如果编译器继续增长，这会是最先需要处理的工程风险。

## 审查证据

- `Cargo.toml`：单 package `calckernel`，binary target `ckc`，benchmark target
  `ckc_perf`，运行依赖只有 `thiserror` 和 `wat`。
- `src/lib.rs`：public crate facade，导出各编译层。
- `src/main.rs`：原生 CLI command boundary 和 process behavior。
- `src/lexer/mod.rs`、`src/parser.rs`、`src/typeck.rs`、`src/mir/mod.rs`、
  `src/opt/mod.rs`、`src/backend/mod.rs`：核心编译层。
- `tests/*.rs`：lexer、parser、checker、MIR、optimizer、C、WASM、LLVM、CLI、
  docs、release-surface、TypeScript-oracle 测试。
- `.github/workflows/native-release.yml`：原生发布 gate 和 artifact build workflow。
- 当前验证：
  - `cargo clippy --all-targets --all-features --locked -- -D warnings` 通过。
  - `cargo fmt --check` 通过。
  - `cargo test --locked` 通过。
  - `cargo test --locked --test docs_surface_test` 通过。

## 当前架构

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

## 符合最佳实践的地方

### 产品边界清楚

仓库现在描述并测试的是 native-only 产品：

- `Cargo.toml` package `calckernel`
- binary target `ckc`
- CLI 和测试使用的 Rust library modules
- `.github/workflows/native-release.yml` 构建出的原生归档

边界明确：没有脚本语言包装层，没有 declaration parity surface，也没有旧 registry
workflow。这降低了发布口径的混乱。

### 编译流水线分层合理

流水线符合常规编译器结构：

- source 和 diagnostics
- lexer
- parser / AST
- type checker
- typed `CheckedProgram`
- MIR lowering
- MIR optimization
- MIR validation
- backend emission
- CLI command boundary

这让语义分析和代码生成分离得比较清楚。backend 消费 MIR，而不是重新解释 AST 语义。

### 兼容测试很强

测试覆盖：

- diagnostics 和 public error behavior
- stdout/stderr 和 exit code
- MIR output
- 生成的 C/H、WAT/WASM、LLVM IR
- 生成 native 和 WASM artifact 的 runtime behavior
- release 和 documentation surface 约束
- TypeScript oracle availability 和 portability

对于兼容性重写来说，这是正确的工程重心。

### MIR Validation 是真实安全门

`validate_mir_module` 会验证 function name、block label、terminator、
branch target、value type、call signature 和 load/store place。pass manager
在每个 optimization pass 后运行 validation。这样 optimizer 或 backend 改动造成的
非法 IR 能更早暴露。

### 优化策略保守

optimizer 在 checked overflow mode 下避免不安全折叠，并且对 f64 行为有显式 guard。
这是编译器工程的正确顺序：先守住语义，再基于测量优化热路径。

### Runtime 和依赖面很小

当前 runtime 架构很轻：

- 不捆绑 allocator
- 不依赖 WASI
- 不嵌入 LLVM
- `wat` 只用于把 WAT assemble 成 bytes
- `clang` 作为外部原生工具调用

生产 `src/` 扫描没有发现 `unsafe`。

### 发布 gate 具体

workflow 和文档对齐了一组有用的 gate：

- `cargo fmt --check`
- `cargo clippy --all-targets --all-features --locked -- -D warnings`
- `cargo test --locked`
- `cargo build --release --locked`
- binary smoke checks
- archive 和 `SHA256` generation

这是原生二进制交付的良好基线。

## 问题和风险

### P1：Backend 模块职责过宽

`src/backend/mod.rs` 在一个模块里负责所有 C、WASM 和 LLVM emission，同时还包含
layout calculation、checked C helper、WASM structured-loop lowering、LLVM register
helper、C hot-path proof、name escaping 和低层字符串 formatting。

风险：

- backend-specific invariant 很难隔离。
- reviewer 需要一次理解三个 target。
- 小改动容易误触无关 backend 行为。
- helper 中的 panic 在大文件里更难审计。

建议拆分：

```text
src/backend/mod.rs        public facade and shared options
src/backend/c.rs          C/H emission and checked C helpers
src/backend/wasm.rs       WAT/WASM emission and WASM layout
src/backend/llvm.rs       LLVM IR emission
src/backend/layout.rs     truly shared struct layout helpers
```

### P1：MIR 和 Optimizer 文件混合多个职责

`src/mir/mod.rs` 同时包含 IR definitions、lowering、pretty-printing 和 validation。
`src/opt/mod.rs` 同时包含 pass definitions、pass manager 和所有 pass implementation。

风险：

- 核心 IR contract 和 transformation code 混在一起。
- validator change 与 lowering change 不容易在 review 中分离。
- pass-specific state 和 generic pass-manager logic 共用一个命名空间。
- 后续如果加入 SSA 或更多 backend-specific lowering，文件压力会继续上升。

建议拆分：

```text
src/mir/ir.rs
src/mir/lower.rs
src/mir/print.rs
src/mir/validate.rs
src/opt/pipeline.rs
src/opt/passes/constant_folding.rs
src/opt/passes/copy_propagation.rs
src/opt/passes/dce.rs
src/opt/passes/cfg.rs
src/opt/passes/cse.rs
src/opt/passes/loops.rs
```

### P1：Public API 暴露了过多内部实现面

`src/lib.rs` 用 wildcard exports 重新导出了整个模块。这会让许多内部 compiler structure
意外成为 public crate surface。

风险：

- 内部 helper 后续重命名更困难。
- 调用方可以构造非法 MIR 并直接调用 public emitter。
- backend precondition 在 facade boundary 不清楚。
- 即使预期产品是 `ckc` binary，也会产生额外兼容压力。

建议：

- 大部分 phase internals 改成 `pub(crate)`。
- 导出较小的稳定 facade，例如 `check`、`compile_to_mir`、`emit_c`、
  `emit_wasm`、`emit_llvm`。
- 只有明确支持外部使用的详细 IR type 才保持 public。
- 对任何接受 raw MIR 的 public emitter 写清 precondition。

### P1：Backend public function 对非法 MIR 可能 panic

生产 backend code 中存在 assertion-style panic path，例如未知 struct field、非法 scalar
type、非法 assignment target。经过 validation 后，这些作为内部断言可以理解；但由于
backend 当前被公开导出，外部调用方可以绕开 validation。

风险：

- 调用方构造的非法 MIR 可能直接 crash process，而不是返回 error。
- CLI 路径大多受 validation 保护，但 library caller 的边界不清楚。
- panic boundary 没有作为 API precondition 明确记录。

建议：

- 引入 typed backend errors，例如 `BackendError`。
- 在 public emitter entry point 内部验证 MIR，或只暴露 checked wrapper API。
- assertion-style panic 只留在 validation 之后的 private helper 中。

### P2：CLI Boundary 需要更薄

`src/main.rs` 目前有效，但职责偏宽：argument parsing、command dispatch、file IO、
atomic write、clang call、target triple detection、diagnostic formatting、exit-code
behavior 都集中在这里。

风险：

- command behavior 测试很强，但实现改动 review 成本偏高。
- IO/process execution 和 compiler command semantics 混在一起。
- 增加新 flag 或 target 会继续扩大 `main.rs`。

建议拆分：

```text
src/cli/args.rs
src/cli/commands.rs
src/cli/io.rs
src/cli/clang.rs
src/main.rs       thin process::exit wrapper
```

### P2：各层 Error Type 不一致

项目有较好的 diagnostics 和 `MirLowerError`，但 CLI/backend 路径大量使用
`Result<_, String>`。这对 CLI parity 很实用，但内部 contract 偏弱。

建议：

- 用户可见字符串继续保留在 CLI boundary。
- library layer 使用 typed errors。
- 只在 command boundary 把 typed errors 转成兼容字符串。

### P2：Toolchain Version 没有固定

项目使用 Rust edition 2024，CI 安装 stable Rust，但 `Cargo.toml` 没有声明
`rust-version`，也没有可见的 `rust-toolchain.toml`。

影响：

- edition 2024 让 toolchain drift 更相关。
- local 与 CI 行为会随 stable 版本推进而漂移。
- release reproducibility 不够强。

建议：

- 在 `Cargo.toml` 增加 `rust-version`。
- 如果项目需要固定本地工具链，增加 `rust-toolchain.toml`。
- 在 release docs 中写清 supported MSRV。

### P2：文档有 phase-history 噪声

部分 architecture docs 仍像开发阶段记录，包含 phase number 和旧迁移期表述。它们对历史
有帮助，但作为当前架构文档会降低可读性。

建议：

- 当前架构文档改成 state-based，而不是 phase-based。
- 如果需要保留历史，把迁移过程放到单独 migration record。
- 文档重点写 current invariant、owner、verification command。

### P3：Release Workflow Publish Step 可以更明确

release workflow 对 artifact build 和 smoke check 做得不错。publish step 在
tag-triggered 或 manual `publish` 为 true 时运行，然后上传到 `GITHUB_REF_NAME`
指向的 release。

风险：

- 如果在 branch 上手动运行且 `publish=true`，`GITHUB_REF_NAME` 可能是 branch name，
  不是 version tag。
- 目标 GitHub Release 可能不存在。

建议：

- manual publishing 要求显式 tag input。
- ref 不是 `refs/tags/v*` 时提前失败。
- 可选：在受控步骤中创建 draft release。

### P3：Supply-Chain Check 较少

依赖很少，这是优点。不过当前没有可见的 license/advisory gate，例如 `cargo deny`。

建议：

- 增加 `cargo deny check advisories licenses bans sources`。
- 对 dependency exception 做显式记录。

## 建议优先级

1. 先拆 backend modules。这是维护收益最高、语义风险最低的机械性重构。
2. 拆 MIR 和 optimizer modules，不改变 public behavior。
3. 收窄 crate facade，把内部实现细节改成 `pub(crate)`。
4. 引入 typed backend errors，或只暴露经过 validation 的 public wrapper API。
5. 拆分 CLI implementation：args、commands、IO、clang helpers。
6. 固定并记录支持的 Rust toolchain。
7. 清理 architecture docs，让它描述当前状态，而不是 phase history。
8. 增加可选 supply-chain policy checks。

## 结论

项目当前满足一个编译器重写最重要的工程实践：产品边界清楚、流水线分层、测试以行为为
中心、optimization 后有 validation、local checks 严格，并且有原生 release automation。

它还没有达到长期大型编译器代码库的模块化程度。下一步质量提升应该优先做模块化和
API 边界收紧，而不是继续堆行为。现有测试已经覆盖了关键用户可见行为，因此这些重构是
现实可控的。
