#!/usr/bin/env node
import { existsSync, readdirSync, readFileSync, statSync } from "node:fs";
import { resolve, join } from "node:path";
import { SUPPORTED_CKC_BINARY_TARGETS, binaryNameForTarget, supportedTargetNames } from "../npm/platform.js";

const RELEASE_WORKFLOW = "npm release artifact";
const PLATFORM_SIGNOFF_JOB = "platform-signoff";

const [manifestArg, signoffDirArg] = process.argv.slice(2);
if (!manifestArg || !signoffDirArg || manifestArg === "--help" || manifestArg === "-h") {
  console.error("Usage: node scripts/verify-npm-release-signoff.mjs <release-manifest.json> <signoff-dir>");
  process.exit(manifestArg ? 0 : 1);
}

const manifestPath = resolve(manifestArg);
const signoffDir = resolve(signoffDirArg);
if (!existsSync(manifestPath)) {
  fail(`Release manifest does not exist: ${manifestPath}`);
}
if (!existsSync(signoffDir) || !statSync(signoffDir).isDirectory()) {
  fail(`Sign-off directory does not exist: ${signoffDir}`);
}

const manifest = readJson(manifestPath, "release manifest");
validateManifest(manifest);
const signoffs = readSignoffs(signoffDir);
const manifestTargetNames = new Set(manifest.targets.map((target) => target.name));
const manifestTargetsByName = new Map(manifest.targets.map((target) => [target.name, target]));
const signoffsByTarget = new Map();

for (const signoff of signoffs) {
  if (!signoff.targetName) {
    fail("sign-off is missing targetName");
  }
  if (!supportedTargetNames().includes(signoff.targetName)) {
    fail(`unsupported platform sign-off for ${signoff.targetName}`);
  }
  if (signoffsByTarget.has(signoff.targetName)) {
    fail(`duplicate platform sign-off for ${signoff.targetName}`);
  }
  signoffsByTarget.set(signoff.targetName, signoff);
}

const verifiedTargets = [];
const signedTargets = [];
for (const target of SUPPORTED_CKC_BINARY_TARGETS) {
  if (!manifestTargetNames.has(target.name)) {
    fail(`release manifest is missing target ${target.name}`);
  }
  const signoff = signoffsByTarget.get(target.name);
  if (!signoff) {
    fail(`missing platform sign-off for ${target.name}`);
  }
  const manifestTarget = manifestTargetsByName.get(target.name);
  validateSignoff(signoff, target, manifest, manifestTarget);
  verifiedTargets.push(target.name);
  signedTargets.push({
    name: target.name,
    platform: target.platform,
    arch: target.arch,
    sha256: manifestTarget.sha256,
    nodeVersion: signoff.nodeVersion,
    npmVersion: signoff.npmVersion,
    ciProvider: signoff.ciProvider,
    githubRunId: signoff.githubRunId,
    githubRunAttempt: signoff.githubRunAttempt,
    githubSha: signoff.githubSha,
    githubRepository: signoff.githubRepository,
    githubWorkflow: signoff.githubWorkflow,
    githubJob: signoff.githubJob,
    runnerOs: signoff.runnerOs,
    runnerArch: signoff.runnerArch,
    installedBin: signoff.installedBin,
    packagedBinary: signoff.packagedBinary,
    packagedBinarySha256: signoff.packagedBinarySha256
  });
}

console.log(JSON.stringify({
  status: "ok",
  package: manifest.packageName,
  packageVersion: manifest.packageVersion,
  tarball: manifest.tarball,
  tarballSha256: manifest.tarballSha256,
  sourceGitSha: manifest.sourceGitSha,
  sourceRepository: manifest.sourceRepository,
  targetCount: verifiedTargets.length,
  targets: verifiedTargets,
  signedTargets,
  sourceFallback: "disabled",
  ckcBinOverride: "unset",
  commands: requiredCommands(),
  apiSymbols: requiredApiSymbols(),
  typeSmoke: "passed",
  backendRuntimeSmokes: backendRuntimeSmokes()
}, null, 2));

function validateManifest(manifest) {
  if (manifest.packageName !== "calckernel") {
    fail(`release manifest packageName must be "calckernel", found ${JSON.stringify(manifest.packageName)}`);
  }
  if (!manifest.packageVersion) {
    fail("release manifest is missing packageVersion");
  }
  if (!manifest.tarball) {
    fail("release manifest is missing tarball");
  }
  if (!isSha256(manifest.tarballSha256)) {
    fail(`release manifest tarballSha256 is invalid: ${JSON.stringify(manifest.tarballSha256)}`);
  }
  if (!isGitSha(manifest.sourceGitSha)) {
    fail(`release manifest sourceGitSha must be a 40-character lowercase hex commit SHA, found ${JSON.stringify(manifest.sourceGitSha)}`);
  }
  if (!isSourceRepository(manifest.sourceRepository)) {
    fail(
      `release manifest sourceRepository must be "local" or a GitHub owner/repository value, ` +
        `found ${JSON.stringify(manifest.sourceRepository)}`
    );
  }
  if (!Array.isArray(manifest.targets)) {
    fail("release manifest targets must be an array");
  }
  const actualTargets = manifest.targets.map((target) => target?.name);
  const expectedTargets = supportedTargetNames();
  if (JSON.stringify(actualTargets) !== JSON.stringify(expectedTargets)) {
    fail(
      `release manifest targets must be ${JSON.stringify(expectedTargets)}, ` +
        `found ${JSON.stringify(actualTargets)}`
    );
  }
}

function validateSignoff(signoff, target, manifest, manifestTarget) {
  if (signoff.package !== "calckernel") {
    fail(`${target.name} sign-off package must be "calckernel"`);
  }
  if (signoff.packageVersion !== manifest.packageVersion) {
    fail(`${target.name} sign-off packageVersion must be ${manifest.packageVersion}`);
  }
  if (signoff.tarball !== manifest.tarball) {
    fail(`${target.name} sign-off tarball must be ${manifest.tarball}`);
  }
  if (signoff.tarballSha256 !== manifest.tarballSha256) {
    fail(`${target.name} sign-off tarballSha256 does not match release manifest`);
  }
  if (signoff.platform !== target.platform) {
    fail(`${target.name} sign-off platform must be ${target.platform}`);
  }
  if (signoff.arch !== target.arch) {
    fail(`${target.name} sign-off arch must be ${target.arch}`);
  }
  if (signoff.ckcBinOverride !== "unset") {
    fail(`${target.name} sign-off must run with CKC_BIN unset`);
  }
  if (signoff.sourceFallback !== "disabled") {
    fail(`${target.name} source fallback must be disabled`);
  }
  validateRuntimeEnvironmentEvidence(signoff, target);
  validateCiProvenance(signoff, target);
  if (signoff.githubSha !== manifest.sourceGitSha) {
    fail(`${target.name} githubSha must match release manifest sourceGitSha`);
  }
  if (signoff.githubRepository !== manifest.sourceRepository) {
    fail(`${target.name} githubRepository must match release manifest sourceRepository`);
  }
  validateBinaryEvidence(signoff, target, manifestTarget);
  if (signoff.typeSmoke !== "passed") {
    fail(`${target.name} sign-off must pass TypeScript declaration smoke`);
  }
  requireIncludes(signoff.commands, requiredCommands(), `${target.name} commands`);
  requireIncludes(signoff.apiSymbols, requiredApiSymbols(), `${target.name} apiSymbols`);
}

function validateRuntimeEnvironmentEvidence(signoff, target) {
  requireNonEmptyString(signoff.nodeVersion, `${target.name} nodeVersion`);
  requireNonEmptyString(signoff.npmVersion, `${target.name} npmVersion`);
}

function validateCiProvenance(signoff, target) {
  if (signoff.ciProvider !== "github-actions") {
    fail(`${target.name} ciProvider must be "github-actions"`);
  }
  requireDigits(signoff.githubRunId, `${target.name} githubRunId`);
  requireDigits(signoff.githubRunAttempt, `${target.name} githubRunAttempt`);
  if (typeof signoff.githubSha !== "string" || !/^[0-9a-f]{40}$/.test(signoff.githubSha)) {
    fail(`${target.name} githubSha must be a 40-character lowercase hex commit SHA`);
  }
  if (!isGitHubRepository(signoff.githubRepository)) {
    fail(`${target.name} githubRepository must be a GitHub owner/repository value`);
  }
  if (signoff.githubWorkflow !== RELEASE_WORKFLOW) {
    fail(`${target.name} githubWorkflow must be ${JSON.stringify(RELEASE_WORKFLOW)}`);
  }
  if (signoff.githubJob !== PLATFORM_SIGNOFF_JOB) {
    fail(`${target.name} githubJob must be ${JSON.stringify(PLATFORM_SIGNOFF_JOB)}`);
  }

  const expectedRunnerOs = runnerOsForTarget(target);
  const expectedRunnerArch = runnerArchForTarget(target);
  if (signoff.runnerOs !== expectedRunnerOs) {
    fail(`${target.name} runnerOs must be ${expectedRunnerOs}`);
  }
  if (signoff.runnerArch !== expectedRunnerArch) {
    fail(`${target.name} runnerArch must be ${expectedRunnerArch}`);
  }
}

function validateBinaryEvidence(signoff, target, manifestTarget) {
  const installedBinName = target.platform === "win32" ? "ckc.cmd" : "ckc";
  requirePathSuffix(
    signoff.installedBin,
    `node_modules/.bin/${installedBinName}`,
    `${target.name} installedBin`
  );
  requirePathSuffix(
    signoff.packagedBinary,
    `node_modules/calckernel/npm/bin/${binaryNameForTarget(target.name)}`,
    `${target.name} packagedBinary`
  );
  if (!isSha256(manifestTarget?.sha256)) {
    fail(`${target.name} release manifest target sha256 is invalid`);
  }
  if (signoff.packagedBinarySha256 !== manifestTarget.sha256) {
    fail(
      `${target.name} sign-off packagedBinarySha256 does not match release manifest target sha256`
    );
  }
}

function readSignoffs(dir) {
  return readdirSync(dir)
    .filter((entry) => entry.endsWith(".json"))
    .sort()
    .map((entry) => readJson(join(dir, entry), `sign-off ${entry}`));
}

function readJson(path, label) {
  try {
    return JSON.parse(readFileSync(path, "utf8"));
  } catch (error) {
    fail(`Unable to read ${label}: ${error.message}`);
  }
}

function requireIncludes(actual, expected, label) {
  if (!Array.isArray(actual)) {
    fail(`${label} must be an array`);
  }
  for (const value of expected) {
    if (!actual.includes(value)) {
      fail(`${label} is missing ${value}`);
    }
  }
}

function requireNonEmptyString(actual, label) {
  if (typeof actual !== "string" || actual.length === 0) {
    fail(`${label} is missing`);
  }
}

function requireDigits(actual, label) {
  if (typeof actual !== "string" || !/^\d+$/.test(actual)) {
    fail(`${label} must be a non-empty decimal string`);
  }
}

function runnerOsForTarget(target) {
  switch (target.platform) {
    case "darwin":
      return "macOS";
    case "linux":
      return "Linux";
    case "win32":
      return "Windows";
    default:
      fail(`${target.name} runnerOs cannot be inferred for platform ${target.platform}`);
      return undefined;
  }
}

function runnerArchForTarget(target) {
  switch (target.arch) {
    case "arm64":
      return "ARM64";
    case "x64":
      return "X64";
    default:
      fail(`${target.name} runnerArch cannot be inferred for arch ${target.arch}`);
      return undefined;
  }
}

function requirePathSuffix(actual, expectedSuffix, label) {
  if (typeof actual !== "string" || actual.length === 0) {
    fail(`${label} is missing`);
  }
  const normalizedActual = actual.replace(/\\/g, "/");
  if (!normalizedActual.endsWith(expectedSuffix)) {
    fail(`${label} must end with ${expectedSuffix}, found ${JSON.stringify(actual)}`);
  }
}

function requiredCommands() {
  return [
    "ckc --help",
    "ckc check smoke.ck",
    "ckc emit-mir smoke.ck -o build/smoke.mir",
    "ckc emit-c smoke.ck -o build/smoke.c",
    "ckc emit-wat smoke.ck -o build/smoke.wat",
    "ckc emit-wasm smoke.ck -o build/smoke.wasm",
    "ckc emit-llvm smoke.ck -o build/smoke.ll",
    "ckc build smoke.ck -o build/smoke-c",
    ...backendRuntimeSmokes().slice(0, 2),
    "ckc build-llvm smoke.ck --kind object -o build/smoke.o",
    backendRuntimeSmokes()[2]
  ];
}

function backendRuntimeSmokes() {
  return [
    "node smoke-c-runtime.mjs",
    "node smoke-wasm-runtime.mjs",
    "node smoke-llvm-object-runtime.mjs"
  ];
}

function requiredApiSymbols() {
  return [
    "SourceFile",
    "TokenKind",
    "lex",
    "parse",
    "check",
    "getFunctionInfo",
    "emitCHeader",
    "emitCSource",
    "CKWasmArena",
    "createCKWasmArena"
  ];
}

function isSha256(value) {
  return typeof value === "string" && /^[0-9a-f]{64}$/.test(value);
}

function isGitSha(value) {
  return typeof value === "string" && /^[0-9a-f]{40}$/.test(value);
}

function isSourceRepository(value) {
  return value === "local" || isGitHubRepository(value);
}

function isGitHubRepository(value) {
  return typeof value === "string" && /^[A-Za-z0-9_.-]+\/[A-Za-z0-9_.-]+$/.test(value);
}

function fail(message) {
  console.error(`verify-npm-release-signoff: ${message}`);
  process.exit(1);
}
