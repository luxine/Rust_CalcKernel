# IK 到 CK 迁移指南

本文记录从 IntKernel / IK 命名迁移到 CalcKernel / CK 命名的历史规则。在 Rust
仓库中，用户可见编译器是原生 `ckc` 可执行文件。

## Mapping

```text
IK -> CK
IntKernel -> CalcKernel
ikc -> ckc
.ik -> .ck
IK_API -> CK_API
IK_BUILD_DLL -> CK_BUILD_DLL
IK_Status -> CK_Status
IK_OK -> CK_OK
IK_ERR_OVERFLOW -> CK_ERR_OVERFLOW
IK_ERR_DIV_BY_ZERO -> CK_ERR_DIV_BY_ZERO
IK_ERR_NULL_POINTER -> CK_ERR_NULL_POINTER
```

## 兼容策略

Rust 编译器不保留 `ikc`、`.ik` 或 `IK_` compatibility aliases。用户应在同一个迁移
窗口中把源码、命令、生成的 C header 和 FFI binding 迁移到 CK naming。

## 用户迁移步骤

1. 将源码文件从 `.ik` 重命名为 `.ck`。
2. 将脚本和文档中的 `ikc` 更新为 `ckc`。
3. 将项目名 reference 从 `IntKernel` 更新为 `CalcKernel`。
4. 将短项目名 reference 从 `IK` 更新为 `CK`。
5. 重新生成 C header，并将 FFI code 中的 `IK_*` 名称更新为 `CK_*`。
6. 运行原生 CLI 冒烟检查：

```sh
cargo build --release --locked
./target/release/ckc --help
./target/release/ckc check examples/scalar.ck
```

## C ABI 说明

Generated checked C status 的 numeric value 保持不变，只改变公开名称：

| Name | Value |
| --- | ---: |
| `CK_OK` | `0` |
| `CK_ERR_OVERFLOW` | `1` |
| `CK_ERR_DIV_BY_ZERO` | `2` |
| `CK_ERR_NULL_POINTER` | `3` |

不会生成 `IK_` typedef 或 macro 作为 compatibility alias。需要重新生成 header，
并将 C、Python ctypes、Rust FFI、Go、C# 或其他 host binding 更新到 `CK_` 名称。
