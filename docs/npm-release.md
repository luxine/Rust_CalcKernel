# npm Release and Migration Checklist

This document defines the npm packaging contract for the Rust `ckc` replacement.
It is the release-side checklist for replacing the TypeScript package while
keeping the public package name, CLI name, and JavaScript helper API stable.

## Package Contract

- Package name stays `calckernel`.
- The only npm binary remains `ckc`.
- The package root exports TypeScript-compatible `SourceFile`, `TokenKind`,
  `lex`, `parse`, `check`, type-checker helpers, `Scope`, `SymbolTable`,
  C backend helpers, `formatDiagnostic`, `formatDiagnostics`, `CKWasmArena`,
  and `createCKWasmArena`.
- Published packages do not compile native code during consumer install.
- A published release tarball contains the full supported binary matrix under
  `npm/bin/`.
- The package file surface is intentionally narrow: `README.md`,
  `docs/npm-release.md`, `docs/architecture-review.md`,
  `docs/zh-CN/architecture-review.md`, `package.json`, and the `npm/`
  runtime files.
- Published packages must not include internal `docs/superpowers/` planning or
  specification notes.
- A local smoke tarball may contain only the current host binary, but that is
  not sufficient for npm publication.
- `CKC_BIN` remains an escape hatch for source checkouts and local debugging.

The runtime lookup order in `npm/ckc.js` is:

1. `CKC_BIN`
2. `npm/bin/ckc-${platform}-${arch}[.exe]`
3. local source checkout fallback: `target/release/ckc`
4. local source checkout fallback: `target/debug/ckc`

## Supported Binary Matrix

| npm target | Rust target | packaged binary |
| --- | --- | --- |
| `darwin-arm64` | `aarch64-apple-darwin` | `npm/bin/ckc-darwin-arm64` |
| `darwin-x64` | `x86_64-apple-darwin` | `npm/bin/ckc-darwin-x64` |
| `linux-arm64` | `aarch64-unknown-linux-gnu` | `npm/bin/ckc-linux-arm64` |
| `linux-x64` | `x86_64-unknown-linux-gnu` | `npm/bin/ckc-linux-x64` |
| `win32-arm64` | `aarch64-pc-windows-msvc` | `npm/bin/ckc-win32-arm64.exe` |
| `win32-x64` | `x86_64-pc-windows-msvc` | `npm/bin/ckc-win32-x64.exe` |

`npm/platform.js` is the authoritative matrix used by the CLI wrapper, prepack
script, and smoke tests. Do not hand-code target names in release automation.

## Building Target Artifacts

For local host smoke testing:

```sh
cargo build --release
npm run verify:host-npm-install
```

`verify:host-npm-install` packs the current host binary, installs the tarball
into a temporary consumer project with `CKC_BIN` unset, runs the installed `ckc`
through the CLI backend commands, imports the package root API, and runs a
TypeScript declaration smoke against `npm/index.d.ts` when `tsc` is available.
Its JSON output includes the npm target name, platform, architecture, tarball
filename, tarball SHA256, command list, API symbols, TypeScript smoke status,
and `ckcBinOverride: "unset"` so it can be archived as a platform sign-off.
To verify an already generated tarball instead of repacking, pass its path:

```sh
npm run verify:host-npm-install -- /path/to/calckernel-0.8.0.tgz
```

For a release, build each Rust target in the appropriate runner and stage the
binary matrix with the exact packaged filenames from `npm/platform.js`:

```sh
npm run build:npm-matrix -- --target linux-x64
```

When a runner cannot cross-compile, run the same command per supported target on
the appropriate host and merge the produced `build/npm-binaries` files. The
script prints a JSON manifest containing each target name, Rust target, staged
binary path, file mode, size, and SHA256. Add `--expect-complete` on the final
staging pass to fail before packing if any supported `npm/platform.js` target is
missing from the staging directory. To stage already-built Cargo target
directories without rebuilding, use:

```sh
npm run build:npm-matrix -- --clean --skip-build --expect-complete --cargo-target-dir target --out build/npm-binaries
```

`build:npm-matrix` stages incrementally by default so separate platform runners
can append to the same staging directory. Use `--clean` only when starting a new
matrix from an intentionally empty output directory. `--expect-complete` reports
missing target names and packaged binary filenames, so run it before `npm pack`
once the merged staging directory is supposed to be complete.

After all six files are staged, pack once:

```sh
CKC_NPM_BINARIES_DIR=build/npm-binaries npm pack --json
npm run verify:npm-release -- calckernel-0.8.0.tgz > release-manifest.json
```

## Automated Release Workflow

The checked-in GitHub Actions workflow `.github/workflows/npm-release.yml` is
the executable release path for producing a formal multi-platform artifact. It
can be started with `workflow_dispatch` and runs these stages:

1. Verify release scripts with `cargo fmt --check`, `cargo clippy`, package
   release tests, `audit-rust-replacement-readiness`, and
   `audit-npm-release-workflow`.
2. Build the six npm targets from `npm/platform.js` on their matching runners
   and upload one binary artifact per target.
3. Download all binaries into `build/npm-binaries`, pack once with
   `CKC_NPM_BINARIES_DIR`, and write `release-manifest.json` with
   `verify:npm-release`.
4. Download the same tarball on every target platform, run
   `verify:host-npm-install` with `CKC_BIN` unset, and upload
   `signoffs/<npm-target>.json`.
5. Download all signoffs and run `verify:release-signoff` to prove that every
   supported platform installed the same tarball SHA256.
6. When publication is intentionally approved, rerun or dispatch the workflow
   with `publish=true`. The `publish-npm` job requires the protected
   `npm-production` environment, `secrets.NPM_TOKEN`, and npm provenance
   (`npm publish --provenance --access public`) before the signed-off tarball is
   uploaded to the npm registry.

`npm run audit:release-workflow` validates that this workflow still contains
the required jobs, target matrix entries, runners, artifact flow, and release
verification and gated publish commands. The default `publish=false` mode only
produces release artifacts and sign-off evidence; it does not publish.

## Per-Artifact Sign-Off

Run these checks before a release tarball is approved:

```sh
cargo test
cargo clippy --all-targets --all-features --locked -- -D warnings
npm run build:npm-matrix -- --target <npm-target>
npm run build:npm-matrix -- --skip-build --expect-complete --cargo-target-dir target --out build/npm-binaries
CKC_NPM_BINARIES_DIR=build/npm-binaries npm pack --json
npm run verify:npm-release -- calckernel-0.8.0.tgz > release-manifest.json
npm run verify:host-npm-install -- calckernel-0.8.0.tgz > signoffs/<npm-target>.json
npm run verify:release-signoff -- release-manifest.json signoffs
npm pack --dry-run --json --ignore-scripts
```

Then install the generated tarball in a fresh consumer project on each supported
platform with scripts disabled and verify the packaged binary path, not a local
`CKC_BIN` override:

```sh
mkdir -p /tmp/ckc-consumer
cd /tmp/ckc-consumer
npm init -y
npm install --ignore-scripts /path/to/calckernel-0.8.0.tgz
unset CKC_BIN
./node_modules/.bin/ckc --help
node --input-type=module --eval "import { SourceFile, TokenKind, lex, parse, check, getFunctionInfo, emitCHeader, emitCSource, createCKWasmArena } from 'calckernel'; console.log(typeof SourceFile, TokenKind.Eof, typeof lex, typeof parse, typeof check, typeof getFunctionInfo, typeof emitCHeader, typeof emitCSource, typeof createCKWasmArena)"
```

`build:npm-matrix --expect-complete` rejects incomplete staging directories
before `npm pack`. `verify:npm-release` rejects any file outside the release file surface and
prints a JSON manifest containing the tarball filename, tarball SHA256,
Rust package metadata (`type`, `main`, `types`, `exports`, `bin`, and empty
dependency fields), `package.json` `files` whitelist, allowed tarball entries,
required package files, forbidden internal prefixes, every packaged binary file mode,
architecture, format, size, and SHA256. It also rejects staged target binaries
that do not look like their expected executable format and architecture, or
macOS/Linux entries that are not executable: Mach-O for macOS, ELF for Linux,
PE for Windows, and `arm64` / `x64` matching the npm target name.
Save that manifest as `release-manifest.json`. On each supported target
platform, run `npm run verify:host-npm-install -- <tarball>` against the same
tarball with `CKC_BIN` unset and save stdout as `signoffs/<npm-target>.json`.
Then run `npm run verify:release-signoff -- release-manifest.json signoffs`.
The sign-off verifier rejects missing or duplicate targets, unsupported target
names, mismatched tarball SHA256s, `CKC_BIN` overrides, missing backend smoke
commands, missing public API symbols, and TypeScript declaration smoke failures.
Record the release manifest and the final sign-off verifier output in the
release notes.

## TypeScript Package Migration

The migration is intentionally in-place:

- Consumers keep installing `calckernel`.
- CLI scripts keep invoking `ckc`.
- Existing commands such as `check`, `emit-c`, `emit-wasm`, `emit-llvm`,
  `build`, and `build-llvm` remain the compatibility surface.
- JavaScript callers can import root `SourceFile`, `TokenKind`, `lex`, `parse`,
  `check`, type-checker helpers, C backend helpers, diagnostic formatters,
  `CKWasmArena`, and `createCKWasmArena` without depending on internal
  TypeScript paths.
- Consumer installs no longer need TypeScript sources, `dist/`, or `wabt`.

Cutover is complete only after the release tarball contains every supported
binary, each target platform has passed `verify:host-npm-install`, and the
Rust package's own release, sign-off, and compatibility oracle verifiers pass.
Actual registry replacement requires the workflow's gated `publish=true` path
to publish the signed-off tarball with `NPM_TOKEN` and npm provenance.
The TypeScript checkout remains read-only source material during the rewrite;
this package does not require changes to the original TypeScript repository.

## 中文说明

这个包的迁移方式是原地替换：包名仍是 `calckernel`，命令仍是 `ckc`，
用户安装时不运行 native build。正式发布的 tarball 必须携带完整平台矩阵
的 Rust 二进制，文件名由 `npm/platform.js` 统一定义。只携带当前平台
二进制的 tarball 只能用于本地 smoke，不能作为 npm 正式发布产物。

发布时必须按上面的矩阵逐个平台构建 binary，集中放入 staging 目录，然后
用 `CKC_NPM_BINARIES_DIR` 打一个主包。在每个目标平台 fresh install 同一个
tarball，并在没有 `CKC_BIN` 的情况下运行 `node_modules/.bin/ckc --help`。
只有 tarball hash、全部二进制 hash、各平台 CLI smoke、root API import 和
TypeScript declaration smoke 都签核后，才可以把现有 TypeScript `ckc` 包视为
已被 Rust 包替换。
TypeScript checkout 在重写期间保持只读 oracle；Rust 包的发布签核不要求修改
原 TypeScript 仓库。
`npm run verify:npm-release -- <tarball>` 会输出可归档的 JSON manifest，
用于确认正式 tarball 携带完整矩阵、严格文件面、每个二进制的 file mode、
architecture、格式、大小和 SHA256，并会拒绝文件面之外的额外文件、macOS/Linux
二进制不可执行、格式不像目标平台 executable 或架构与 npm target 不匹配的随包文件。
`npm run verify:host-npm-install` 用于本机 fresh install smoke：它会临时安装
当前 host tarball、清空 `CKC_BIN`，并验证 CLI backend 命令、package root API
和 TypeScript declaration smoke。
如果要验证已经生成的 tarball，使用
`npm run verify:host-npm-install -- /path/to/calckernel-0.8.0.tgz`。
每个平台都应把该命令 stdout 保存为 `signoffs/<npm-target>.json`，再用
`npm run verify:release-signoff -- release-manifest.json signoffs` 汇总验收。
该 verifier 会拒绝缺失或重复平台、tarball SHA256 不匹配、`CKC_BIN` override、
backend smoke 命令缺失、公开 API symbol 缺失和 TypeScript declaration smoke
未通过的签核文件。
真正替换 npm registry 上的包时，必须显式用 `publish=true` 触发 workflow 的
`publish-npm` job；该 job 需要受保护的 `npm-production` environment、
`NPM_TOKEN`，并用 `npm publish --provenance --access public` 发布已经签核的
同一个 tarball。默认 `publish=false` 只生成 artifact 和 sign-off evidence，
不会发布。
