# IK to CK Migration Guide

This guide documents the historical rename from IntKernel / IK naming to
CalcKernel / CK naming. In the Rust repository, the user-facing compiler is the
native `ckc` executable.

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

## Compatibility Policy

The Rust compiler does not keep `ikc`, `.ik`, or `IK_` compatibility aliases.
Users should migrate source files, command invocations, generated C headers,
and FFI bindings to CK naming in one migration window.

## User Migration Steps

1. Rename source files from `.ik` to `.ck`.
2. Update scripts and documentation from `ikc` to `ckc`.
3. Update project references from `IntKernel` to `CalcKernel`.
4. Update short project references from `IK` to `CK`.
5. Regenerate C headers and update FFI code from `IK_*` names to `CK_*`.
6. Run the native CLI smoke checks:

```sh
cargo build --release --locked
./target/release/ckc --help
./target/release/ckc check examples/scalar.ck
```

## C ABI Notes

Generated checked C status numeric values are preserved while the public names
change:

| Name | Value |
| --- | ---: |
| `CK_OK` | `0` |
| `CK_ERR_OVERFLOW` | `1` |
| `CK_ERR_DIV_BY_ZERO` | `2` |
| `CK_ERR_NULL_POINTER` | `3` |

No `IK_` typedefs or macros are emitted as compatibility aliases. Rebuild
generated headers and update C, Python ctypes, Rust FFI, Go, C#, or other host
bindings to use the `CK_` names.
