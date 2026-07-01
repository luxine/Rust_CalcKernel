#!/usr/bin/env node
import { spawnSync } from "node:child_process";
import { createHash } from "node:crypto";
import { existsSync, readFileSync } from "node:fs";
import { basename, dirname, resolve } from "node:path";
import { fileURLToPath } from "node:url";
import { SUPPORTED_CKC_BINARY_TARGETS, binaryNameForTarget } from "../npm/platform.js";

const EXPECTED_PACKAGE_DESCRIPTION = "A small CK / CalcKernel integer-computation DSL compiler with C, WASM, and LLVM backends.";
const EXPECTED_PACKAGE_KEYWORDS = ["calckernel", "ck", "compiler", "dsl", "c", "wasm", "llvm"];
const EXPECTED_PACKAGE_LICENSE = "MIT";
const EXPECTED_PACKAGE_ENGINES = { node: ">=20" };
const TAR_MAX_BUFFER_BYTES = 64 * 1024 * 1024;
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
const tarballArg = process.argv[2];
const sourceRoot = resolve(dirname(fileURLToPath(import.meta.url)), "..");

if (!tarballArg || tarballArg === "--help" || tarballArg === "-h") {
  console.error("Usage: node scripts/verify-npm-release.mjs <calckernel-version.tgz>");
  process.exit(tarballArg ? 0 : 1);
}

const tarballPath = resolve(tarballArg);
if (!existsSync(tarballPath)) {
  fail(`Release tarball does not exist: ${tarballPath}`);
}

const tarballDetails = listTarballDetails(tarballPath);
const entries = tarballDetails.map((detail) => detail.entry);
const entrySet = new Set(entries);
const entryDetails = new Map(tarballDetails.map((detail) => [detail.entry, detail]));
const requiredFiles = [
  "package/package.json",
  "package/npm/ckc.js",
  "package/npm/platform.js",
  "package/npm/index.js",
  "package/npm/index.d.ts",
  "package/docs/npm-release.md",
  "package/docs/architecture-review.md",
  "package/docs/zh-CN/architecture-review.md",
  "package/README.md",
  "package/README.zh-CN.md"
];
const expectedPackageJsonFiles = [
  "npm",
  "README.md",
  "README.zh-CN.md",
  "docs/npm-release.md",
  "docs/architecture-review.md",
  "docs/zh-CN/architecture-review.md"
];

for (const file of requiredFiles) {
  requireEntry(entrySet, file);
}

const forbiddenPrefixes = [
  "package/docs/superpowers/",
  "package/src/",
  "package/target/"
];

for (const entry of entries) {
  for (const prefix of forbiddenPrefixes) {
    if (entry.startsWith(prefix)) {
      fail(`Release tarball must not include ${entry}`);
    }
  }
}

const packageJson = JSON.parse(extractTarballEntry(tarballPath, "package/package.json").toString("utf8"));
if (packageJson.name !== "calckernel") {
  fail(`Expected package name "calckernel", found "${packageJson.name}"`);
}
if (!packageJson.version) {
  fail("package/package.json is missing version");
}
if (!sameStringArray(packageJson.files, expectedPackageJsonFiles)) {
  fail(
    `package/package.json files must be ${JSON.stringify(expectedPackageJsonFiles)}, found ${JSON.stringify(packageJson.files)}`
  );
}
const packageMetadata = validatePackageMetadata(packageJson);

const targets = SUPPORTED_CKC_BINARY_TARGETS.map((target) => {
  const binaryPath = `package/npm/bin/${binaryNameForTarget(target.name)}`;
  requireEntry(entrySet, binaryPath);
  const fileMode = readTargetFileMode(entryDetails, target, binaryPath);
  const binaryBytes = extractTarballEntry(tarballPath, binaryPath);
  const binaryFormat = expectedBinaryFormat(target);
  validateBinaryFormat(target, binaryPath, binaryBytes, binaryFormat);
  const binaryArchitecture = readBinaryArchitecture(target, binaryPath, binaryBytes, binaryFormat);
  return {
    name: target.name,
    rustTarget: target.rustTarget,
    binaryPath,
    fileMode,
    binaryFormat,
    binaryArchitecture,
    sizeBytes: binaryBytes.length,
    sha256: sha256(binaryBytes)
  };
});
const allowedEntries = [
  ...requiredFiles,
  ...targets.map((target) => target.binaryPath)
].sort();
const allowedEntrySet = new Set(allowedEntries);
for (const entry of entries) {
  if (entry.endsWith("/")) {
    continue;
  }
  if (!allowedEntrySet.has(entry)) {
    fail(`Release tarball includes unexpected file ${entry}`);
  }
}

const manifest = {
  packageName: packageJson.name,
  packageVersion: packageJson.version,
  packageMetadata,
  tarball: basename(tarballPath),
  tarballSha256: sha256(readFileSync(tarballPath)),
  sourceGitSha: readSourceGitSha(),
  sourceRepository: readSourceRepository(),
  requiredFiles,
  fileSurface: {
    packageJsonFiles: packageJson.files,
    requiredFiles,
    forbiddenPrefixes,
    allowedEntries
  },
  targets
};

console.log(JSON.stringify(manifest, null, 2));

function listTarballDetails(tarball) {
  const output = runTar(["-tvzf", tarball], "list tarball details");
  return output
    .toString("utf8")
    .split(/\r?\n/)
    .map((line) => line.trim())
    .filter(Boolean)
    .map(parseTarballDetail);
}

function extractTarballEntry(tarball, entry) {
  return runTar(["-xOf", tarball, entry], `extract ${entry}`);
}

function requireEntry(entrySet, entry) {
  if (!entrySet.has(entry)) {
    fail(`Release tarball is missing ${entry}`);
  }
}

function parseTarballDetail(line) {
  const parts = line.split(/\s+/);
  return {
    mode: parts[0] ?? "",
    entry: parts.at(-1) ?? ""
  };
}

function readTargetFileMode(entryDetails, target, binaryPath) {
  const detail = entryDetails.get(binaryPath);
  if (!detail) {
    fail(`${binaryPath} for ${target.name} is invalid: missing tar entry mode`);
  }
  if (target.platform !== "win32" && !hasOwnerExecuteBit(detail.mode)) {
    fail(`${binaryPath} for ${target.name} is invalid: expected executable mode, found ${detail?.mode ?? "unknown"}`);
  }
  return detail.mode;
}

function hasOwnerExecuteBit(mode) {
  return mode.length >= 4 && (mode[3] === "x" || mode[3] === "s");
}

function runTar(args, action) {
  const output = spawnSync("tar", args, { maxBuffer: TAR_MAX_BUFFER_BYTES });
  if (output.error) {
    fail(`Unable to ${action}: ${output.error.message}`);
  }
  if (output.status !== 0) {
    const stderr = output.stderr.toString("utf8").trim();
    const stdout = output.stdout.toString("utf8").trim();
    fail(`Unable to ${action}${stderr || stdout ? `: ${stderr || stdout}` : ""}`);
  }
  return output.stdout;
}

function sha256(bytes) {
  return createHash("sha256").update(bytes).digest("hex");
}

function sameStringArray(actual, expected) {
  return Array.isArray(actual)
    && actual.length === expected.length
    && actual.every((value, index) => value === expected[index]);
}

function readSourceGitSha() {
  const githubSha = process.env.GITHUB_SHA;
  if (githubSha) {
    if (!isGitSha(githubSha)) {
      fail(`GITHUB_SHA must be a 40-character lowercase hex commit SHA, found ${JSON.stringify(githubSha)}`);
    }
    return githubSha;
  }

  requireCleanGitWorktree();

  const output = spawnSync("git", ["rev-parse", "HEAD"], { cwd: sourceRoot, encoding: "utf8" });
  if (output.error) {
    fail(`Unable to read source git SHA: ${output.error.message}`);
  }
  if (output.status !== 0) {
    const stderr = output.stderr.trim();
    const stdout = output.stdout.trim();
    fail(`Unable to read source git SHA${stderr || stdout ? `: ${stderr || stdout}` : ""}`);
  }
  const sourceGitSha = output.stdout.trim();
  if (!isGitSha(sourceGitSha)) {
    fail(`git rev-parse HEAD returned invalid source git SHA: ${JSON.stringify(sourceGitSha)}`);
  }
  return sourceGitSha;
}

function readSourceRepository() {
  const githubRepository = process.env.GITHUB_REPOSITORY;
  if (githubRepository) {
    validateGitHubRepository(githubRepository, "GITHUB_REPOSITORY");
    return githubRepository;
  }
  if (process.env.GITHUB_SHA || process.env.GITHUB_ACTIONS === "true") {
    fail("GITHUB_REPOSITORY is required when creating a release manifest from GitHub source");
  }
  return "local";
}

function requireCleanGitWorktree() {
  const output = spawnSync("git", ["status", "--porcelain"], { cwd: sourceRoot, encoding: "utf8" });
  if (output.error) {
    fail(`Unable to verify clean source git worktree: ${output.error.message}`);
  }
  if (output.status !== 0) {
    const stderr = output.stderr.trim();
    const stdout = output.stdout.trim();
    fail(`Unable to verify clean source git worktree${stderr || stdout ? `: ${stderr || stdout}` : ""}`);
  }
  if (output.stdout.trim().length > 0) {
    fail("source git worktree must be clean before creating release manifest");
  }
}

function isGitSha(value) {
  return typeof value === "string" && /^[0-9a-f]{40}$/.test(value);
}

function validateGitHubRepository(value, label) {
  if (!/^[A-Za-z0-9_.-]+\/[A-Za-z0-9_.-]+$/.test(value)) {
    fail(`${label} must be a GitHub owner/repository value, found ${JSON.stringify(value)}`);
  }
}

function validatePackageMetadata(packageJson) {
  if (packageJson.description !== EXPECTED_PACKAGE_DESCRIPTION) {
    fail(
      `package/package.json description must be ${JSON.stringify(EXPECTED_PACKAGE_DESCRIPTION)}, ` +
        `found ${JSON.stringify(packageJson.description)}`
    );
  }
  if (!sameJson(packageJson.keywords, EXPECTED_PACKAGE_KEYWORDS)) {
    fail(
      `package/package.json keywords must be ${JSON.stringify(EXPECTED_PACKAGE_KEYWORDS)}, ` +
        `found ${JSON.stringify(packageJson.keywords)}`
    );
  }
  if (packageJson.license !== EXPECTED_PACKAGE_LICENSE) {
    fail(`package/package.json license must be ${EXPECTED_PACKAGE_LICENSE}, found ${JSON.stringify(packageJson.license)}`);
  }
  if (!sameJson(packageJson.engines, EXPECTED_PACKAGE_ENGINES)) {
    fail(
      `package/package.json engines must be ${JSON.stringify(EXPECTED_PACKAGE_ENGINES)}, ` +
        `found ${JSON.stringify(packageJson.engines)}`
    );
  }
  if (packageJson.type !== "module") {
    fail(`package/package.json type must be "module", found ${JSON.stringify(packageJson.type)}`);
  }
  if (packageJson.main !== "./npm/index.js") {
    fail(`package/package.json main must be "./npm/index.js", found ${JSON.stringify(packageJson.main)}`);
  }
  if (packageJson.types !== "./npm/index.d.ts") {
    fail(`package/package.json types must be "./npm/index.d.ts", found ${JSON.stringify(packageJson.types)}`);
  }

  const expectedExports = {
    ".": {
      types: "./npm/index.d.ts",
      import: "./npm/index.js"
    }
  };
  if (!sameJson(packageJson.exports, expectedExports)) {
    fail(`package/package.json exports must be ${JSON.stringify(expectedExports)}, found ${JSON.stringify(packageJson.exports)}`);
  }

  const expectedBin = { ckc: "./npm/ckc.js" };
  if (!sameJson(packageJson.bin, expectedBin)) {
    fail(`package/package.json bin must be ${JSON.stringify(expectedBin)}, found ${JSON.stringify(packageJson.bin)}`);
  }

  const dependencyFields = {};
  for (const field of [
    "dependencies",
    "devDependencies",
    "optionalDependencies",
    "peerDependencies",
    "bundledDependencies",
    "bundleDependencies"
  ]) {
    if (isNonEmptyDependencyField(packageJson[field])) {
      fail(`package/package.json must not declare ${field}`);
    }
  }
  if (Object.hasOwn(packageJson, "packageManager")) {
    fail("package/package.json packageManager must be absent");
  }
  const consumerInstallScripts = readConsumerInstallScripts(packageJson);
  const scriptNames = readPackageScriptNames(packageJson);

  return {
    description: EXPECTED_PACKAGE_DESCRIPTION,
    keywords: EXPECTED_PACKAGE_KEYWORDS,
    license: EXPECTED_PACKAGE_LICENSE,
    engines: EXPECTED_PACKAGE_ENGINES,
    type: packageJson.type,
    main: packageJson.main,
    types: packageJson.types,
    exports: expectedExports,
    bin: expectedBin,
    dependencyFields,
    consumerInstallScripts,
    packageManager: null,
    scriptNames
  };
}

function readPackageScriptNames(packageJson) {
  const scripts = packageJson.scripts;
  if (!scripts || typeof scripts !== "object" || Array.isArray(scripts)) {
    fail(`package/package.json scripts must be an object, found ${JSON.stringify(scripts)}`);
  }
  const scriptNames = Object.keys(scripts).sort();
  if (!sameStringArray(scriptNames, EXPECTED_PACKAGE_SCRIPT_NAMES)) {
    fail(
      `package/package.json scriptNames must be ${JSON.stringify(EXPECTED_PACKAGE_SCRIPT_NAMES)}, ` +
        `found ${JSON.stringify(scriptNames)}`
    );
  }
  return scriptNames;
}

function readConsumerInstallScripts(packageJson) {
  const scripts = packageJson.scripts;
  if (scripts === undefined || scripts === null) {
    return [];
  }
  if (typeof scripts !== "object" || Array.isArray(scripts)) {
    fail(`package/package.json scripts must be an object when present, found ${JSON.stringify(scripts)}`);
  }
  const consumerInstallScripts = ["preinstall", "install", "postinstall"].filter((scriptName) =>
    Object.hasOwn(scripts, scriptName)
  );
  if (consumerInstallScripts.length > 0) {
    fail(`package/package.json consumer install lifecycle scripts must be absent: ${consumerInstallScripts.join(", ")}`);
  }
  return consumerInstallScripts;
}

function sameJson(actual, expected) {
  return JSON.stringify(actual) === JSON.stringify(expected);
}

function isNonEmptyDependencyField(value) {
  if (value === undefined || value === null) {
    return false;
  }
  if (Array.isArray(value)) {
    return value.length > 0;
  }
  if (typeof value === "object") {
    return Object.keys(value).length > 0;
  }
  return true;
}

function expectedBinaryFormat(target) {
  if (target.platform === "darwin") {
    return "Mach-O";
  }
  if (target.platform === "linux") {
    return "ELF";
  }
  if (target.platform === "win32") {
    return "PE";
  }
  fail(`Unsupported release binary platform for ${target.name}: ${target.platform}`);
}

function validateBinaryFormat(target, binaryPath, bytes, format) {
  if (format === "Mach-O" && !isMachO(bytes)) {
    fail(`${binaryPath} for ${target.name} is invalid: expected Mach-O executable`);
  }
  if (format === "ELF" && !hasPrefix(bytes, [0x7f, 0x45, 0x4c, 0x46])) {
    fail(`${binaryPath} for ${target.name} is invalid: expected ELF executable`);
  }
  if (format === "PE" && !isPE(bytes)) {
    fail(`${binaryPath} for ${target.name} is invalid: expected PE executable`);
  }
}

function readBinaryArchitecture(target, binaryPath, bytes, format) {
  const actual = detectBinaryArchitecture(bytes, format);
  if (actual !== target.arch) {
    fail(`${binaryPath} for ${target.name} is invalid: expected ${target.arch} architecture, found ${actual}`);
  }
  return actual;
}

function detectBinaryArchitecture(bytes, format) {
  if (format === "Mach-O") {
    return machOArchitecture(bytes);
  }
  if (format === "ELF") {
    return elfArchitecture(bytes);
  }
  if (format === "PE") {
    return peArchitecture(bytes);
  }
  return "unknown";
}

function machOArchitecture(bytes) {
  if (bytes.length < 8) {
    return "unknown";
  }
  let cpuType;
  if (hasPrefix(bytes, [0xcf, 0xfa, 0xed, 0xfe]) || hasPrefix(bytes, [0xce, 0xfa, 0xed, 0xfe])) {
    cpuType = bytes.readUInt32LE(4);
  } else if (hasPrefix(bytes, [0xfe, 0xed, 0xfa, 0xcf]) || hasPrefix(bytes, [0xfe, 0xed, 0xfa, 0xce])) {
    cpuType = bytes.readUInt32BE(4);
  } else {
    return "unknown";
  }
  return architectureFromMachOCpuType(cpuType);
}

function elfArchitecture(bytes) {
  if (bytes.length < 20) {
    return "unknown";
  }
  const dataEncoding = bytes[5];
  let machine;
  if (dataEncoding === 1) {
    machine = bytes.readUInt16LE(18);
  } else if (dataEncoding === 2) {
    machine = bytes.readUInt16BE(18);
  } else {
    return "unknown";
  }
  return architectureFromElfMachine(machine);
}

function peArchitecture(bytes) {
  if (bytes.length < 0x40) {
    return "unknown";
  }
  const peOffset = bytes.readUInt32LE(0x3c);
  if (bytes.length < peOffset + 6 || !hasPrefix(bytes.subarray(peOffset), [0x50, 0x45, 0x00, 0x00])) {
    return "unknown";
  }
  return architectureFromPeMachine(bytes.readUInt16LE(peOffset + 4));
}

function architectureFromMachOCpuType(cpuType) {
  if (cpuType === 0x0100000c) {
    return "arm64";
  }
  if (cpuType === 0x01000007) {
    return "x64";
  }
  return `unknown(${cpuType})`;
}

function architectureFromElfMachine(machine) {
  if (machine === 183) {
    return "arm64";
  }
  if (machine === 62) {
    return "x64";
  }
  return `unknown(${machine})`;
}

function architectureFromPeMachine(machine) {
  if (machine === 0xaa64) {
    return "arm64";
  }
  if (machine === 0x8664) {
    return "x64";
  }
  return `unknown(${machine})`;
}

function isMachO(bytes) {
  return [
    [0xfe, 0xed, 0xfa, 0xce],
    [0xce, 0xfa, 0xed, 0xfe],
    [0xfe, 0xed, 0xfa, 0xcf],
    [0xcf, 0xfa, 0xed, 0xfe],
    [0xca, 0xfe, 0xba, 0xbe],
    [0xbe, 0xba, 0xfe, 0xca],
    [0xca, 0xfe, 0xba, 0xbf],
    [0xbf, 0xba, 0xfe, 0xca]
  ].some((magic) => hasPrefix(bytes, magic));
}

function hasPrefix(bytes, prefix) {
  return bytes.length >= prefix.length && prefix.every((byte, index) => bytes[index] === byte);
}

function isPE(bytes) {
  if (!hasPrefix(bytes, [0x4d, 0x5a]) || bytes.length < 0x40) {
    return false;
  }
  const peOffset = bytes.readUInt32LE(0x3c);
  return bytes.length >= peOffset + 6 && hasPrefix(bytes.subarray(peOffset), [0x50, 0x45, 0x00, 0x00]);
}

function fail(message) {
  console.error(`verify-npm-release: ${message}`);
  process.exit(1);
}
