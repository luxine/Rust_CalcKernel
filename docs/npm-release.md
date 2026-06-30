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
- Published packages do not compile native code during consumer install, and
  registry metadata must not contain consumer install lifecycle scripts.
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
TypeScript declaration smoke against `npm/index.d.ts`. If no compiler is
already available through `TSC_BIN` or local `node_modules`, the verifier
installs `typescript@^5.8.0` in that temporary consumer before running the
smoke. Its JSON output includes the npm target name, platform, architecture,
tarball filename, tarball SHA256, installed `node_modules/.bin/ckc` path,
packaged `node_modules/calckernel/npm/bin/ckc-<target>` Rust binary path,
`packagedBinarySha256` for that Rust binary, command list, API symbols,
TypeScript smoke status, and
`ckcBinOverride: "unset"` so it can be archived as a platform sign-off.
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
npm run build:npm-matrix -- --verify-staged --expect-complete --out build/npm-binaries
CKC_NPM_BINARIES_DIR=build/npm-binaries npm pack --json
npm run verify:npm-release -- calckernel-0.8.0.tgz > release-manifest.json
```

## Automated Release Workflow

The checked-in GitHub Actions workflow `.github/workflows/npm-release.yml` is
the executable release path for producing a formal multi-platform artifact. It
can be started with `workflow_dispatch`. The dispatch inputs include the npm
`publish` switch plus `typescript_oracle_repository` and `typescript_oracle_ref`
for the read-only TypeScript compatibility oracle. The default oracle repository
is `luxine/CalcKernel`; if that repository is private or cross-org, configure
`TYPESCRIPT_ORACLE_TOKEN` with read access.

The workflow runs these stages:

1. Checkout and build the TypeScript oracle with `pnpm install --frozen-lockfile`
   and `pnpm build`, set `CALCKERNEL_TS_ROOT` to that checkout, and run
   `verify:typescript-oracle`.
2. Verify release scripts with `cargo fmt --check`, `cargo clippy`, full
   `cargo test`, package release tests including the registry replacement
   verifier test, `audit-rust-replacement-readiness`,
   `audit:typescript-test-surface`, `verify:declaration-parity`,
   `verify:public-api-parity`, and `audit-npm-release-workflow`.
3. Build the six npm targets from `npm/platform.js` on their matching runners
   and upload one binary artifact per target.
4. Download all binaries into `build/npm-binaries`, pack once with
   `CKC_NPM_BINARIES_DIR`, but first run
   `build:npm-matrix --verify-staged --expect-complete` against the downloaded
   directory. Then write `release-manifest.json` with `verify:npm-release`.
5. Download the release manifest and the same tarball on every target platform,
   derive the tarball filename from `release-manifest.json`, and run
   `verify:host-npm-install` with `CKC_BIN` unset; the verifier prepares
   `typescript@^5.8.0` in the temporary consumer when needed so the TypeScript
   declaration smoke must pass, then uploads `signoffs/<npm-target>.json`.
6. Download all signoffs and run `verify:release-signoff` to prove that every
   supported platform installed the same tarball SHA256 and that each platform's
   packaged Rust binary SHA256 matches the corresponding target entry in
   `release-manifest.json`.
7. When publication is intentionally approved, rerun or dispatch the workflow
   with `publish=true`. The `publish-npm` job requires the protected
   `npm-production` environment, `secrets.NPM_TOKEN`, and npm provenance
   (`npm publish --provenance --access public`). Before publishing, it runs
   `verify:release-signoff-summary` against `release-manifest.json` and
   `release-signoff.json` so publication cannot start from a missing or
   mismatched six-platform sign-off summary. It also runs
   `verify:publish-artifact` against `release-manifest.json` and `dist/` to
   prove the tarball SHA256 still matches the signed-off release manifest and
   that the manifest carries the formal release verifier metadata, file surface,
   and target matrix evidence. After publish, it runs `verify:registry-replacement`
   for the `packageVersion`
   recorded in `release-manifest.json` against npm registry metadata to confirm
   the published package exposes the Rust package `main`, `types`, `exports`,
   and `ckc` bin paths rather than stale TypeScript `dist/` paths, and that the
   registry metadata has no consumer install lifecycle scripts. It then runs
   `verify:publish-result` to bind `release-manifest.json`,
   `npm publish --json`, and the registry verifier output to the same package,
   version, tarball filename, npm integrity, sha1 shasum, and successful
   registry replacement status, including the registry tarball URL. The publish
   and registry integrity values must both be sha512 npm integrity strings and
   must match; the publish and registry shasum values must both be sha1 shasum
   strings and must match. Finally, it runs `verify:cutover-evidence` to bind
   the release manifest, six-platform sign-off summary including signed target
   binary SHA256 values, pre-publish artifact verifier output, and post-publish
   result verifier output into one final evidence JSON.

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
npm run build:npm-matrix -- --verify-staged --expect-complete --out build/npm-binaries
CKC_NPM_BINARIES_DIR=build/npm-binaries npm pack --json
npm run verify:npm-release -- calckernel-0.8.0.tgz > release-manifest.json
npm run verify:host-npm-install -- calckernel-0.8.0.tgz > signoffs/<npm-target>.json
npm run verify:release-signoff -- release-manifest.json signoffs > release-signoff.json
npm run verify:release-signoff-summary -- release-manifest.json release-signoff.json
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
commands, missing `build-llvm --kind object` smoke evidence, missing public API
symbols, missing installed/package binary path evidence, mismatched
`packagedBinarySha256` values, and TypeScript declaration smoke failures.
`verify:host-npm-install` must report
`typeSmoke: "passed"` on every sign-off target; skipped declaration smokes are
not acceptable release evidence.
Record the release manifest and the final sign-off verifier output in the
release notes. After publication, also archive `npm-cutover-evidence.json`;
it proves the signed tarball, signed target binary SHA256 values, platform
sign-offs, pre-publish artifact check, and registry publish result all refer to
the same replacement package version.
Before `npm publish`, run
`npm run verify:release-signoff-summary -- release-manifest.json release-signoff.json`
so the publish step is gated by the same release manifest and six-platform
sign-off summary that the final cutover evidence will later bind to registry
metadata.

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

Before using local oracle parity as replacement evidence, run:

```sh
npm run verify:typescript-oracle
npm run audit:typescript-test-surface
npm run verify:declaration-parity
npm run verify:public-api-parity
```

This confirms the read-only TypeScript oracle checkout exists, its
`dist/src/cli.js` is built and runnable, the fixture roots used by the Rust
oracle tests are present, every TypeScript oracle test file has an explicit
Rust migration mapping in `docs/typescript-test-surface.json`, the Rust package
root declaration exports match the resolved TypeScript declaration exports from
`dist/src/index.d.ts`, and the Rust package root runtime exports exactly the
same public JavaScript API names as `dist/src/index.js`.

Cutover is complete only after the release tarball contains every supported
binary, each target platform has passed `verify:host-npm-install`, and the
Rust package's own release, sign-off, and compatibility oracle verifiers pass.
Actual registry replacement requires the workflow's gated `publish=true` path
to publish the signed-off tarball with `NPM_TOKEN` and npm provenance. That job
must first pass
`npm run verify:release-signoff-summary -- release-manifest.json release-signoff.json`
and `npm run verify:publish-artifact -- release-manifest.json dist`, or the
equivalent workflow artifact paths, then pass
`npm run verify:registry-replacement -- "$(node -p "JSON.parse(require('fs').readFileSync('release-manifest.json', 'utf8')).packageVersion")"`
after publication, and then
pass `npm run verify:publish-result -- release-manifest.json npm-publish.json
npm-registry-replacement.json` so the manifest, publish result, registry
replacement status, registry metadata, and absence of consumer install lifecycle
scripts all prove the same npm artifact.
The publish, registry, and final cutover evidence must carry the same
sha512 npm integrity value and sha1 shasum, and final cutover evidence must
report the registry tarball URL.
The final downloaded evidence set
should also pass `npm run verify:cutover-evidence -- release-manifest.json
release-signoff.json npm-publish-artifact.json npm-publish-result.json`.
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
和 TypeScript declaration smoke。它输出的 `packagedBinarySha256` 必须与
`release-manifest.json` 中对应 target 的 binary SHA256 一致。
如果要验证已经生成的 tarball，使用
`npm run verify:host-npm-install -- /path/to/calckernel-0.8.0.tgz`。
使用本地 TypeScript oracle parity 作为替换证据前，先运行
`npm run verify:typescript-oracle` 和 `npm run audit:typescript-test-surface`；
它们会只读确认 TypeScript checkout 存在、`dist/src/cli.js` 已构建且可启动，
确认 Rust oracle tests 依赖的 fixture 目录存在，并确认 TypeScript oracle
的每个测试文件都有 Rust 迁移映射。
每个平台都应把该命令 stdout 保存为 `signoffs/<npm-target>.json`，再用
`npm run verify:release-signoff -- release-manifest.json signoffs` 汇总验收。
该 verifier 会拒绝缺失或重复平台、tarball SHA256 不匹配、`CKC_BIN` override、
backend smoke 命令缺失、`build-llvm --kind object` smoke evidence 缺失、公开
API symbol 缺失、`packagedBinarySha256` 与 release manifest target SHA256
不一致，以及 TypeScript declaration smoke 未通过的签核文件。最终
cutover evidence 必须归档 signed target binary SHA256 值。
真正替换 npm registry 上的包时，必须显式用 `publish=true` 触发 workflow 的
`publish-npm` job；该 job 需要受保护的 `npm-production` environment、
`NPM_TOKEN`，并用 `npm publish --provenance --access public` 发布已经签核的
同一个 tarball。发布前必须先运行 `verify:release-signoff-summary`，确认
`release-signoff.json` 和 `release-manifest.json` 指向同一个包、版本、tarball、
SHA256 和六个平台；随后运行 `verify:publish-artifact`，用 `release-manifest.json`
校验 `dist/` 中即将发布的 tarball SHA256 仍然匹配已签核 manifest，并确认
manifest 带有正式 release verifier 产生的 metadata、文件面和目标矩阵证据。
默认 `publish=false` 只生成 artifact 和 sign-off evidence，不会发布。
workflow 在发布前会先运行 registry replacement verifier 的测试，避免
`publish=true` 之后才发现 registry metadata 检查脚本本身失效。
发布后 workflow 会从 `release-manifest.json` 读取 `packageVersion` 并运行
`npm run verify:registry-replacement -- <manifest packageVersion>`，从 npm
registry metadata 验证已发布包暴露的是 Rust package 的 `main`、`types`、
`exports` 和 `ckc` bin 路径，而不是旧 TypeScript `dist/` 路径，并确认
registry metadata 不含 consumer install lifecycle scripts。
`verify:publish-result` 会同时要求 registry replacement status 为 `ok`，
并把 npm publish JSON、registry metadata 和 release manifest 绑定到同一个
package、version、tarball、registry tarball URL、integrity 和 shasum。
publish、registry 和最终 cutover evidence 中的 integrity 必须是同一个
sha512 npm integrity 字符串，shasum 必须是同一个 sha1 shasum 字符串。
