#!/usr/bin/env node
import { existsSync, readFileSync } from "node:fs";
import { dirname, join, resolve } from "node:path";
import { fileURLToPath } from "node:url";

const root = resolve(dirname(fileURLToPath(import.meta.url)), "..");
const tsRoot = process.env.CALCKERNEL_TS_ROOT ?? "/Users/lynn/code/CalcKernel";
const failures = [];
const EXPECTED_PACKAGE_SCRIPT_NAMES = Object.freeze([
  "audit:release-workflow",
  "audit:typescript-test-surface",
  "build",
  "build:npm-matrix",
  "ckc",
  "postpack",
  "prepack",
  "test",
  "verify:cutover-evidence",
  "verify:declaration-parity",
  "verify:host-npm-install",
  "verify:npm-release",
  "verify:public-api-parity",
  "verify:publish-artifact",
  "verify:publish-result",
  "verify:registry-replacement",
  "verify:release-signoff",
  "verify:release-signoff-summary",
  "verify:typescript-oracle"
]);

const packageJson = readJson(join(root, "package.json"));
expectEqual(packageJson.name, "calckernel", "Rust package name");
expectEqual(
  packageJson.description,
  "A small CK / CalcKernel integer-computation DSL compiler with C, WASM, and LLVM backends.",
  "Rust package description"
);
expectJson(packageJson.keywords, ["calckernel", "ck", "compiler", "dsl", "c", "wasm", "llvm"], "Rust package keywords");
expectEqual(packageJson.license, "MIT", "Rust package license");
expectJson(packageJson.engines, { node: ">=20" }, "Rust package engines");
expectEqual(packageJson.type, "module", "Rust package type");
expectEqual(packageJson.main, "./npm/index.js", "Rust package main");
expectEqual(packageJson.types, "./npm/index.d.ts", "Rust package types");
expectJson(packageJson.exports, { ".": { types: "./npm/index.d.ts", import: "./npm/index.js" } }, "Rust package exports");
expectJson(packageJson.bin, { ckc: "./npm/ckc.js" }, "Rust package bin");
expectJson(
  packageJson.files,
  [
    "npm",
    "README.md",
    "README.zh-CN.md",
    "docs/npm-release.md",
    "docs/architecture-review.md",
    "docs/zh-CN/architecture-review.md"
  ],
  "Rust package files"
);
expectNoDependencyFields(packageJson, "Rust package");
expectNoPackageManager(packageJson, "Rust package");
expectPackageScriptNames(packageJson, EXPECTED_PACKAGE_SCRIPT_NAMES, "Rust package");
expectEqual(packageJson.scripts?.["build:npm-matrix"], "node scripts/build-npm-binary-matrix.mjs", "binary matrix script");
expectEqual(packageJson.scripts?.["audit:release-workflow"], "node scripts/audit-npm-release-workflow.mjs", "release workflow audit script");
expectEqual(packageJson.scripts?.["audit:typescript-test-surface"], "node scripts/audit-typescript-test-surface.mjs", "TypeScript test surface audit script");
expectEqual(packageJson.scripts?.["verify:host-npm-install"], "node scripts/verify-host-npm-install.mjs", "host install verifier script");
expectEqual(packageJson.scripts?.["verify:npm-release"], "node scripts/verify-npm-release.mjs", "release verifier script");
expectEqual(packageJson.scripts?.["verify:declaration-parity"], "node scripts/verify-declaration-parity.mjs", "declaration parity verifier script");
expectEqual(packageJson.scripts?.["verify:public-api-parity"], "node scripts/verify-public-api-parity.mjs", "public API parity verifier script");
expectEqual(packageJson.scripts?.["verify:publish-artifact"], "node scripts/verify-npm-publish-artifact.mjs", "publish artifact verifier script");
expectEqual(packageJson.scripts?.["verify:publish-result"], "node scripts/verify-npm-publish-result.mjs", "publish result verifier script");
expectEqual(packageJson.scripts?.["verify:cutover-evidence"], "node scripts/verify-npm-cutover-evidence.mjs", "cutover evidence verifier script");
expectEqual(packageJson.scripts?.["verify:registry-replacement"], "node scripts/verify-npm-registry-replacement.mjs", "registry replacement verifier script");
expectEqual(packageJson.scripts?.["verify:release-signoff"], "node scripts/verify-npm-release-signoff.mjs", "release sign-off verifier script");
expectEqual(packageJson.scripts?.["verify:release-signoff-summary"], "node scripts/verify-npm-release-signoff-summary.mjs", "release sign-off summary verifier script");
expectEqual(packageJson.scripts?.["verify:typescript-oracle"], "node scripts/verify-typescript-oracle.mjs", "TypeScript oracle verifier script");

for (const path of [
  "npm/ckc.js",
  "npm/index.js",
  "npm/index.d.ts",
  "npm/platform.js",
  "scripts/build-npm-binary-matrix.mjs",
  "scripts/audit-npm-release-workflow.mjs",
  "scripts/audit-typescript-test-surface.mjs",
  "scripts/audit-typescript-oracle-fixtures.mjs",
  "scripts/verify-host-npm-install.mjs",
  "scripts/verify-npm-release.mjs",
  "scripts/verify-declaration-parity.mjs",
  "scripts/verify-public-api-parity.mjs",
  "scripts/verify-npm-publish-artifact.mjs",
  "scripts/verify-npm-publish-result.mjs",
  "scripts/verify-npm-cutover-evidence.mjs",
  "scripts/verify-npm-registry-replacement.mjs",
  "scripts/verify-npm-release-signoff.mjs",
  "scripts/verify-npm-release-signoff-summary.mjs",
  "scripts/verify-typescript-oracle.mjs",
  "docs/typescript-test-surface.json",
  "docs/npm-release.md",
  "README.md",
  "README.zh-CN.md",
  "docs/architecture-review.md",
  "docs/zh-CN/architecture-review.md"
]) {
  expectExists(join(root, path), `Rust replacement file ${path}`);
}

const npmRelease = readFileSync(join(root, "docs/npm-release.md"), "utf8");
const readme = readFileSync(join(root, "README.md"), "utf8");
const zhReadme = readFileSync(join(root, "README.zh-CN.md"), "utf8");
expectIncludes(readme, "README.zh-CN.md", "README language link");
expectIncludes(zhReadme, "README.md", "Chinese README language link");
expectIncludes(npmRelease, "CKC_NPM_BINARIES_DIR", "npm release docs");
expectIncludes(npmRelease, "--expect-complete", "npm release docs");
expectIncludes(npmRelease, "verify:npm-release", "npm release docs");
expectIncludes(npmRelease, "verify:declaration-parity", "npm release docs");
expectIncludes(npmRelease, "verify:public-api-parity", "npm release docs");
expectIncludes(npmRelease, "verify:publish-artifact", "npm release docs");
expectIncludes(npmRelease, "verify:publish-result", "npm release docs");
expectIncludes(npmRelease, "verify:cutover-evidence", "npm release docs");
expectIncludes(npmRelease, "verify:registry-replacement", "npm release docs");
expectIncludes(npmRelease, "registry replacement status", "npm release docs");
expectIncludes(npmRelease, "registry tarball URL", "npm release docs");
expectIncludes(npmRelease, "release-manifest/release-manifest.json", "npm release docs");
expectIncludes(npmRelease, "release/release-signoff.json", "npm release docs");
expectIncludes(npmRelease, "npm-publish.json", "npm release docs");
expectIncludes(npmRelease, "npm-registry-replacement.json", "npm release docs");
expectIncludes(npmRelease, "release-signoff-summary.json", "npm release docs");
expectIncludes(npmRelease, "npm-publish-artifact.json", "npm release docs");
expectIncludes(npmRelease, "npm-publish-result.json", "npm release docs");
expectIncludes(npmRelease, "sha512 npm integrity", "npm release docs");
expectIncludes(npmRelease, "sha1 shasum", "npm release docs");
expectIncludes(npmRelease, "consumer install lifecycle scripts", "npm release docs");
expectIncludes(npmRelease, "scriptNames", "npm release docs");
expectIncludes(npmRelease, "packageManager", "npm release docs");
expectIncludes(npmRelease, "publishArtifactTarballPath", "npm release docs");
expectIncludes(npmRelease, "publishId", "npm release docs");
expectIncludes(npmRelease, "publishFilename", "npm release docs");
expectIncludes(npmRelease, "publishShasum", "npm release docs");
expectIncludes(npmRelease, "publishIntegrity", "npm release docs");
expectIncludes(npmRelease, "publishProvenance", "npm release docs");
expectIncludes(npmRelease, "publish-npm", "npm release docs");
expectIncludes(npmRelease, "fileSurface", "npm release docs");
expectIncludes(npmRelease, "required/forbidden/allowed file lists", "npm release docs");
expectIncludes(npmRelease, "target Rust triples", "npm release docs");
expectIncludes(npmRelease, "binary paths", "npm release docs");
expectIncludes(npmRelease, "file modes", "npm release docs");
expectIncludes(npmRelease, "binary formats", "npm release docs");
expectIncludes(npmRelease, "architectures", "npm release docs");
expectIncludes(npmRelease, "sizes", "npm release docs");
expectIncludes(npmRelease, "SHA256 values", "npm release docs");
expectIncludes(npmRelease, "verify:host-npm-install", "npm release docs");
expectIncludes(npmRelease, "verify:release-signoff", "npm release docs");
expectIncludes(npmRelease, "verify:release-signoff-summary", "npm release docs");
expectIncludes(npmRelease, "verify:typescript-oracle", "npm release docs");
expectIncludes(npmRelease, "TypeScript declaration smoke", "npm release docs");
expectIncludes(npmRelease, "declaration export kind", "npm release docs");
expectIncludes(npmRelease, "runtime export kind", "npm release docs");
expectIncludes(npmRelease, "typeSmoke", "npm release docs");
expectIncludes(npmRelease, "typescript@^5.8.0", "npm release docs");
expectIncludes(npmRelease, "packagedBinarySha256", "npm release docs");
expectIncludes(npmRelease, "installedBin", "npm release docs");
expectIncludes(npmRelease, "packagedBinary", "npm release docs");
expectIncludes(npmRelease, "signed target binary SHA256", "npm release docs");
expectIncludes(npmRelease, "platform / arch", "npm release docs");
expectIncludes(npmRelease, "nodeVersion", "npm release docs");
expectIncludes(npmRelease, "npmVersion", "npm release docs");
expectIncludes(npmRelease, "ciProvider", "npm release docs");
expectIncludes(npmRelease, "githubRunId", "npm release docs");
expectIncludes(npmRelease, "githubRunAttempt", "npm release docs");
expectIncludes(npmRelease, "githubSha", "npm release docs");
expectIncludes(npmRelease, "githubWorkflow", "npm release docs");
expectIncludes(npmRelease, "githubJob", "npm release docs");
expectIncludes(npmRelease, "npm release artifact", "npm release docs");
expectIncludes(npmRelease, "platform-signoff", "npm release docs");
expectIncludes(npmRelease, "runnerOs", "npm release docs");
expectIncludes(npmRelease, "runnerArch", "npm release docs");
expectIncludes(npmRelease, "ckcBinOverride", "npm release docs");
expectIncludes(npmRelease, "commands", "npm release docs");
expectIncludes(npmRelease, "apiSymbols", "npm release docs");
expectIncludes(npmRelease, "sourceFallback", "npm release docs");
expectIncludes(npmRelease, "backend runtime smoke", "npm release docs");
expectIncludes(npmRelease, "backendRuntimeSmokes", "npm release docs");
expectIncludes(npmRelease, "node smoke-c-runtime.mjs", "npm release docs");
expectIncludes(npmRelease, "node smoke-wasm-runtime.mjs", "npm release docs");
expectIncludes(npmRelease, "node smoke-llvm-object-runtime.mjs", "npm release docs");
expectIncludes(npmRelease, "audit:release-workflow", "npm release docs");
expectIncludes(npmRelease, "npm-cutover-evidence.json", "npm release docs");
expectIncludes(npmRelease, "publish=true", "npm release docs");
expectIncludes(npmRelease, "NPM_TOKEN", "npm release docs");

const architectureReview = readFileSync(join(root, "docs/architecture-review.md"), "utf8");
expectIncludes(architectureReview, "TypeScript checkout remains the compatibility oracle", "architecture review");
expectIncludes(architectureReview, "without requiring edits to the TypeScript checkout", "architecture review");
expectIncludes(architectureReview, "tests/fixtures", "architecture review");
expectIncludes(architectureReview, "f64 edge fixture C dynamic-library runtime parity", "architecture review");
expectIncludes(architectureReview, "f64-array C dynamic-library runtime parity", "architecture review");
expectIncludes(architectureReview, "f64-axpy/f64-sum/pricing-SoA C dynamic-library runtime parity", "architecture review");
expectIncludes(architectureReview, "WASM scalar/calls/control-flow/memory/short-circuit C dynamic-library runtime parity", "architecture review");
expectIncludes(architectureReview, "LLVM scalar/calls/control-flow/memory/short-circuit/bool C dynamic-library runtime parity", "architecture review");
expectIncludes(architectureReview, "dijkstra C dynamic-library runtime parity", "architecture review");
expectIncludes(architectureReview, "f64 edge fixture WASM runtime parity", "architecture review");
expectIncludes(architectureReview, "dijkstra WASM runtime parity", "architecture review");
expectIncludes(architectureReview, "dijkstra LLVM object/dynamic runtime parity", "architecture review");
expectIncludes(architectureReview, "f64-array LLVM object/dynamic runtime parity", "architecture review");
expectIncludes(architectureReview, "f64-axpy/f64-sum/pricing-SoA LLVM object/dynamic runtime parity", "architecture review");

const zhArchitectureReview = readFileSync(join(root, "docs/zh-CN/architecture-review.md"), "utf8");
expectIncludes(zhArchitectureReview, "TypeScript checkout 继续作为 compatibility oracle", "Chinese architecture review");
expectIncludes(zhArchitectureReview, "不要求修改 TypeScript checkout", "Chinese architecture review");
expectIncludes(zhArchitectureReview, "tests/fixtures", "Chinese architecture review");
expectIncludes(zhArchitectureReview, "f64 edge fixture C dynamic-library runtime parity", "Chinese architecture review");
expectIncludes(zhArchitectureReview, "f64-array C dynamic-library runtime parity", "Chinese architecture review");
expectIncludes(zhArchitectureReview, "f64-axpy/f64-sum/pricing-SoA C dynamic-library runtime parity", "Chinese architecture review");
expectIncludes(zhArchitectureReview, "WASM scalar/calls/control-flow/memory/short-circuit C dynamic-library runtime parity", "Chinese architecture review");
expectIncludes(zhArchitectureReview, "LLVM scalar/calls/control-flow/memory/short-circuit/bool C dynamic-library runtime parity", "Chinese architecture review");
expectIncludes(zhArchitectureReview, "dijkstra C dynamic-library runtime parity", "Chinese architecture review");
expectIncludes(zhArchitectureReview, "f64 edge fixture WASM runtime parity", "Chinese architecture review");
expectIncludes(zhArchitectureReview, "dijkstra WASM runtime parity", "Chinese architecture review");
expectIncludes(zhArchitectureReview, "dijkstra LLVM object/dynamic runtime parity", "Chinese architecture review");
expectIncludes(zhArchitectureReview, "f64-array LLVM object/dynamic runtime parity", "Chinese architecture review");
expectIncludes(zhArchitectureReview, "f64-axpy/f64-sum/pricing-SoA LLVM object/dynamic runtime parity", "Chinese architecture review");

if (existsSync(join(tsRoot, "package.json"))) {
  const tsPackageJson = readJson(join(tsRoot, "package.json"));
  expectEqual(tsPackageJson.name, "calckernel", "TypeScript oracle package name");
  expectEqual(tsPackageJson.main, "./dist/src/index.js", "TypeScript oracle main");
  expectJson(tsPackageJson.bin, { ckc: "./dist/src/cli.js" }, "TypeScript oracle bin");
  expectEqual(tsPackageJson.dependencies?.wabt, "^1.0.39", "TypeScript oracle wabt dependency");
}

if (failures.length > 0) {
  for (const failure of failures) {
    console.error(failure);
  }
  process.exit(1);
}

console.log(JSON.stringify({
  status: "ok",
  rustPackage: "calckernel",
  replacementRoot: root,
  typescriptOracleRoot: tsRoot,
  typescriptCheckoutMutationRequired: false
}, null, 2));

function readJson(path) {
  return JSON.parse(readFileSync(path, "utf8"));
}

function expectExists(path, label) {
  if (!existsSync(path)) {
    failures.push(`${label} is missing: ${path}`);
  }
}

function expectIncludes(text, expected, label) {
  if (!text.includes(expected)) {
    failures.push(`${label} must include ${expected}`);
  }
}

function expectEqual(actual, expected, label) {
  if (actual !== expected) {
    failures.push(`${label} must be ${JSON.stringify(expected)}, found ${JSON.stringify(actual)}`);
  }
}

function expectJson(actual, expected, label) {
  if (JSON.stringify(actual) !== JSON.stringify(expected)) {
    failures.push(`${label} must be ${JSON.stringify(expected)}, found ${JSON.stringify(actual)}`);
  }
}

function expectNoDependencyFields(packageJson, label) {
  for (const field of [
    "dependencies",
    "devDependencies",
    "optionalDependencies",
    "peerDependencies",
    "bundledDependencies",
    "bundleDependencies"
  ]) {
    const value = packageJson[field];
    if (value && (Array.isArray(value) ? value.length > 0 : Object.keys(value).length > 0)) {
      failures.push(`${label} must not declare ${field}`);
    }
  }
}

function expectNoPackageManager(packageJson, label) {
  if (Object.hasOwn(packageJson, "packageManager")) {
    failures.push(`${label} must not declare packageManager`);
  }
}

function expectPackageScriptNames(packageJson, expected, label) {
  const scripts = packageJson.scripts;
  if (!scripts || typeof scripts !== "object" || Array.isArray(scripts)) {
    failures.push(`${label} scripts must be an object, found ${JSON.stringify(scripts)}`);
    return;
  }
  expectJson(Object.keys(scripts).sort(), expected, `${label} scriptNames`);
}
