#!/usr/bin/env node
import { existsSync, readFileSync } from "node:fs";
import { basename, resolve } from "node:path";
import { SUPPORTED_CKC_BINARY_TARGETS, binaryNameForTarget, supportedTargetNames } from "../npm/platform.js";

const RELEASE_WORKFLOW = "npm release artifact";
const PLATFORM_SIGNOFF_JOB = "platform-signoff";

const [manifestArg, signoffArg] = process.argv.slice(2);

if (!manifestArg || !signoffArg || manifestArg === "--help" || manifestArg === "-h") {
  console.error(
    "Usage: node scripts/verify-npm-release-signoff-summary.mjs " +
      "<release-manifest.json> <release-signoff.json>"
  );
  process.exit(manifestArg ? 0 : 1);
}

const manifestPath = resolve(manifestArg);
const signoffPath = resolve(signoffArg);
const manifest = readJsonFile(manifestPath, "release manifest");
const signoff = readJsonFile(signoffPath, "release sign-off");
const failures = [];

validateManifest(manifest);
validateReleaseSignoff(signoff, manifest);

if (failures.length > 0) {
  for (const failure of failures) {
    console.error(failure);
  }
  process.exit(1);
}

console.log(JSON.stringify({
  status: "ok",
  package: manifest.packageName,
  packageVersion: manifest.packageVersion,
  version: manifest.packageVersion,
  tarball: manifest.tarball,
  tarballSha256: manifest.tarballSha256,
  targetCount: signoff.targetCount,
  targets: signoff.targets,
  signedTargets: signoff.signedTargets,
  sourceFallback: signoff.sourceFallback,
  ckcBinOverride: signoff.ckcBinOverride,
  commands: signoff.commands,
  apiSymbols: signoff.apiSymbols,
  typeSmoke: signoff.typeSmoke,
  backendRuntimeSmokes: signoff.backendRuntimeSmokes,
  evidence: {
    manifest: manifestPath,
    releaseSignoff: signoffPath
  }
}, null, 2));

function validateManifest(value) {
  expectEqual(value.packageName, "calckernel", "release manifest packageName");
  if (!value.packageVersion) {
    fail("release manifest is missing packageVersion");
  }
  if (!value.tarball || basename(value.tarball) !== value.tarball) {
    fail(`release manifest tarball must be a filename, found ${JSON.stringify(value.tarball)}`);
  }
  if (!isSha256(value.tarballSha256)) {
    fail(`release manifest tarballSha256 is invalid: ${JSON.stringify(value.tarballSha256)}`);
  }
  validateManifestTargets(value.targets, "release manifest targets");
}

function validateReleaseSignoff(value, manifest) {
  expectEqual(value.status, "ok", "release sign-off status");
  expectEqual(value.package, manifest.packageName, "release sign-off package");
  expectEqual(value.packageVersion, manifest.packageVersion, "release sign-off packageVersion");
  expectEqual(value.tarball, manifest.tarball, "release sign-off tarball");
  expectEqual(value.tarballSha256, manifest.tarballSha256, "release sign-off tarballSha256");
  expectEqual(value.sourceFallback, "disabled", "release sign-off sourceFallback");
  expectEqual(value.ckcBinOverride, "unset", "release sign-off ckcBinOverride");
  expectEqual(value.typeSmoke, "passed", "release sign-off typeSmoke");
  validateCommands(value.commands, "release sign-off commands");
  validateApiSymbols(value.apiSymbols, "release sign-off apiSymbols");

  const expectedTargets = supportedTargetNames();
  if (value.targetCount !== expectedTargets.length) {
    fail(`release sign-off targetCount must be ${expectedTargets.length}, found ${JSON.stringify(value.targetCount)}`);
  }
  if (!sameStringArray(value.targets, expectedTargets)) {
    fail(`release sign-off targets must be ${JSON.stringify(expectedTargets)}, found ${JSON.stringify(value.targets)}`);
  }
  validateSignedTargets(value.signedTargets, "release sign-off signedTargets");
  validateBackendRuntimeSmokes(value.backendRuntimeSmokes, "release sign-off backendRuntimeSmokes");

  const manifestTargetShaByName = new Map(manifest.targets.map((target) => [target.name, target.sha256]));
  for (const target of value.signedTargets ?? []) {
    if (target.sha256 !== manifestTargetShaByName.get(target.name)) {
      fail(
        `release sign-off signedTargets ${target.name} sha256 must match release manifest target sha256`
      );
    }
  }
}

function validateBackendRuntimeSmokes(actual, label) {
  const expected = backendRuntimeSmokes();
  if (!sameStringArray(actual, expected)) {
    fail(`${label} must be ${JSON.stringify(expected)}, found ${JSON.stringify(actual)}`);
  }
}

function validateCommands(actual, label) {
  const expected = requiredCommands();
  if (!sameStringArray(actual, expected)) {
    fail(`${label} must be ${JSON.stringify(expected)}, found ${JSON.stringify(actual)}`);
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

function validateApiSymbols(actual, label) {
  const expected = requiredApiSymbols();
  if (!sameStringArray(actual, expected)) {
    fail(`${label} must be ${JSON.stringify(expected)}, found ${JSON.stringify(actual)}`);
  }
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

function readJsonFile(path, label) {
  if (!existsSync(path)) {
    failImmediate(`${label} does not exist: ${path}`);
  }
  try {
    return JSON.parse(readFileSync(path, "utf8"));
  } catch (error) {
    failImmediate(`Unable to read ${label}: ${error.message}`);
  }
}

function expectEqual(actual, expected, label) {
  if (actual !== expected) {
    fail(`${label} must be ${JSON.stringify(expected)}, found ${JSON.stringify(actual)}`);
  }
}

function sameStringArray(actual, expected) {
  return Array.isArray(actual)
    && actual.length === expected.length
    && actual.every((value, index) => value === expected[index]);
}

function validateManifestTargets(actual, label) {
  const expectedTargets = supportedTargetNames();
  if (!Array.isArray(actual)) {
    fail(`${label} must be an array`);
    return;
  }
  const actualNames = actual.map((target) => target?.name);
  if (!sameStringArray(actualNames, expectedTargets)) {
    fail(`${label} names must be ${JSON.stringify(expectedTargets)}, found ${JSON.stringify(actualNames)}`);
  }
  for (const target of actual) {
    if (!isSha256(target?.sha256)) {
      fail(`${label} ${target?.name ?? "unknown"} sha256 is invalid`);
    }
  }
}

function validateSignedTargets(actual, label) {
  validateManifestTargets(actual, label);
  if (!Array.isArray(actual) || actual.length !== SUPPORTED_CKC_BINARY_TARGETS.length) {
    return;
  }
  for (const [index, target] of (actual ?? []).entries()) {
    const expectedTarget = SUPPORTED_CKC_BINARY_TARGETS[index];
    if (target?.platform !== expectedTarget.platform) {
      fail(`${label} ${expectedTarget.name} platform must be ${expectedTarget.platform}`);
    }
    if (target?.arch !== expectedTarget.arch) {
      fail(`${label} ${expectedTarget.name} arch must be ${expectedTarget.arch}`);
    }
    validateSignedTargetRuntimeEnvironment(target, expectedTarget, label);
    validateSignedTargetCiProvenance(target, expectedTarget, label);
    validateSignedTargetBinaryEvidence(target, expectedTarget, label);
  }
}

function validateSignedTargetRuntimeEnvironment(actual, expectedTarget, label) {
  requireNonEmptyString(actual?.nodeVersion, `${label} ${expectedTarget.name} nodeVersion`);
  requireNonEmptyString(actual?.npmVersion, `${label} ${expectedTarget.name} npmVersion`);
}

function validateSignedTargetCiProvenance(actual, expectedTarget, label) {
  if (actual?.ciProvider !== "github-actions") {
    fail(`${label} ${expectedTarget.name} ciProvider must be "github-actions"`);
  }
  requireDigits(actual?.githubRunId, `${label} ${expectedTarget.name} githubRunId`);
  requireDigits(actual?.githubRunAttempt, `${label} ${expectedTarget.name} githubRunAttempt`);
  if (typeof actual?.githubSha !== "string" || !/^[0-9a-f]{40}$/.test(actual.githubSha)) {
    fail(`${label} ${expectedTarget.name} githubSha must be a 40-character lowercase hex commit SHA`);
  }
  if (actual?.githubWorkflow !== RELEASE_WORKFLOW) {
    fail(`${label} ${expectedTarget.name} githubWorkflow must be ${JSON.stringify(RELEASE_WORKFLOW)}`);
  }
  if (actual?.githubJob !== PLATFORM_SIGNOFF_JOB) {
    fail(`${label} ${expectedTarget.name} githubJob must be ${JSON.stringify(PLATFORM_SIGNOFF_JOB)}`);
  }

  const expectedRunnerOs = runnerOsForTarget(expectedTarget);
  const expectedRunnerArch = runnerArchForTarget(expectedTarget);
  if (actual?.runnerOs !== expectedRunnerOs) {
    fail(`${label} ${expectedTarget.name} runnerOs must be ${expectedRunnerOs}`);
  }
  if (actual?.runnerArch !== expectedRunnerArch) {
    fail(`${label} ${expectedTarget.name} runnerArch must be ${expectedRunnerArch}`);
  }
}

function validateSignedTargetBinaryEvidence(actual, expectedTarget, label) {
  const installedBinName = expectedTarget.platform === "win32" ? "ckc.cmd" : "ckc";
  requirePathSuffix(
    actual?.installedBin,
    `node_modules/.bin/${installedBinName}`,
    `${label} ${expectedTarget.name} installedBin`
  );
  requirePathSuffix(
    actual?.packagedBinary,
    `node_modules/calckernel/npm/bin/${binaryNameForTarget(expectedTarget.name)}`,
    `${label} ${expectedTarget.name} packagedBinary`
  );
  if (actual?.packagedBinarySha256 !== actual?.sha256) {
    fail(`${label} ${expectedTarget.name} packagedBinarySha256 must match sha256`);
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
    return;
  }
  const normalizedActual = actual.replace(/\\/g, "/");
  if (!normalizedActual.endsWith(expectedSuffix)) {
    fail(`${label} must end with ${expectedSuffix}, found ${JSON.stringify(actual)}`);
  }
}

function isSha256(value) {
  return typeof value === "string" && /^[0-9a-f]{64}$/.test(value);
}

function fail(message) {
  failures.push(message);
}

function failImmediate(message) {
  console.error(`verify-npm-release-signoff-summary: ${message}`);
  process.exit(1);
}
