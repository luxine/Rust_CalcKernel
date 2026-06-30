#!/usr/bin/env node
import { existsSync, readFileSync } from "node:fs";
import { basename, resolve } from "node:path";
import { SUPPORTED_CKC_BINARY_TARGETS, binaryNameForTarget, supportedTargetNames } from "../npm/platform.js";

const EXPECTED_PACKAGE_DESCRIPTION = "A small CK / CalcKernel integer-computation DSL compiler with C, WASM, and LLVM backends.";
const EXPECTED_PACKAGE_KEYWORDS = ["calckernel", "ck", "compiler", "dsl", "c", "wasm", "llvm"];
const EXPECTED_PACKAGE_LICENSE = "MIT";
const EXPECTED_PACKAGE_ENGINES = { node: ">=20" };
const EXPECTED_PACKAGE_METADATA = Object.freeze({
  description: EXPECTED_PACKAGE_DESCRIPTION,
  keywords: EXPECTED_PACKAGE_KEYWORDS,
  license: EXPECTED_PACKAGE_LICENSE,
  engines: EXPECTED_PACKAGE_ENGINES,
  type: "module",
  main: "./npm/index.js",
  types: "./npm/index.d.ts",
  exports: {
    ".": {
      types: "./npm/index.d.ts",
      import: "./npm/index.js"
    }
  },
  bin: { ckc: "./npm/ckc.js" },
  dependencyFields: {},
  consumerInstallScripts: []
});
const [manifestArg, signoffArg, signoffSummaryArg, publishArtifactArg, publishResultArg] = process.argv.slice(2);

if (
  !manifestArg
  || !signoffArg
  || !signoffSummaryArg
  || !publishArtifactArg
  || !publishResultArg
  || manifestArg === "--help"
  || manifestArg === "-h"
) {
  console.error(
    "Usage: node scripts/verify-npm-cutover-evidence.mjs " +
      "<release-manifest.json> <release-signoff.json> " +
      "<release-signoff-summary.json> " +
      "<npm-publish-artifact.json> <npm-publish-result.json>"
  );
  process.exit(manifestArg ? 0 : 1);
}

const manifestPath = resolve(manifestArg);
const signoffPath = resolve(signoffArg);
const signoffSummaryPath = resolve(signoffSummaryArg);
const publishArtifactPath = resolve(publishArtifactArg);
const publishResultPath = resolve(publishResultArg);
const manifest = readJsonFile(manifestPath, "release manifest");
const signoff = readJsonFile(signoffPath, "release sign-off");
const signoffSummary = readJsonFile(signoffSummaryPath, "release sign-off summary");
const publishArtifact = readJsonFile(publishArtifactPath, "npm publish artifact verifier output");
const publishResult = readJsonFile(publishResultPath, "npm publish result verifier output");
const failures = [];

validateManifest(manifest);
validateReleaseSignoff(signoff, manifest);
validateReleaseSignoffSummary(signoffSummary, manifest, signoff);
validatePublishArtifact(publishArtifact, manifest);
validatePublishResult(publishResult, manifest);

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
  publishArtifactTarballPath: publishArtifact.tarballPath,
  publishPackage: publishResult.publishPackage,
  publishVersion: publishResult.publishVersion,
  publishId: publishResult.publishId,
  publishFilename: publishResult.publishFilename,
  publishShasum: publishResult.publishShasum,
  publishIntegrity: publishResult.publishIntegrity,
  registryStatus: publishResult.registryStatus,
  registryTarball: publishResult.registryTarball,
  shasum: publishResult.shasum,
  description: publishResult.description,
  keywords: publishResult.keywords,
  license: publishResult.license,
  engines: publishResult.engines,
  consumerInstallScripts: publishResult.consumerInstallScripts,
  integrity: publishResult.integrity,
  evidence: {
    manifest: manifestPath,
    releaseSignoff: signoffPath,
    releaseSignoffSummary: signoffSummaryPath,
    publishArtifact: publishArtifactPath,
    publishResult: publishResultPath
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
  if (
    !value.packageMetadata
    || typeof value.packageMetadata !== "object"
    || Array.isArray(value.packageMetadata)
  ) {
    fail("release manifest packageMetadata is missing");
  } else {
    expectJson(value.packageMetadata, EXPECTED_PACKAGE_METADATA, "release manifest packageMetadata");
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

function validateReleaseSignoffSummary(value, manifest, signoff) {
  expectEqual(value.status, "ok", "release sign-off summary status");
  expectEqual(value.package, manifest.packageName, "release sign-off summary package");
  expectEqual(value.packageVersion, manifest.packageVersion, "release sign-off summary packageVersion");
  expectEqual(value.version, manifest.packageVersion, "release sign-off summary version");
  expectEqual(value.tarball, manifest.tarball, "release sign-off summary tarball");
  expectEqual(value.tarballSha256, manifest.tarballSha256, "release sign-off summary tarballSha256");
  expectEqual(value.sourceFallback, "disabled", "release sign-off summary sourceFallback");
  expectEqual(value.sourceFallback, signoff.sourceFallback, "release sign-off summary sourceFallback");
  expectEqual(value.ckcBinOverride, "unset", "release sign-off summary ckcBinOverride");
  expectEqual(value.ckcBinOverride, signoff.ckcBinOverride, "release sign-off summary ckcBinOverride");
  expectEqual(value.typeSmoke, "passed", "release sign-off summary typeSmoke");
  expectEqual(value.typeSmoke, signoff.typeSmoke, "release sign-off summary typeSmoke");
  validateCommands(value.commands, "release sign-off summary commands");
  validateApiSymbols(value.apiSymbols, "release sign-off summary apiSymbols");

  const expectedTargets = supportedTargetNames();
  if (value.targetCount !== expectedTargets.length) {
    fail(`release sign-off summary targetCount must be ${expectedTargets.length}, found ${JSON.stringify(value.targetCount)}`);
  }
  if (!sameStringArray(value.targets, expectedTargets)) {
    fail(`release sign-off summary targets must be ${JSON.stringify(expectedTargets)}, found ${JSON.stringify(value.targets)}`);
  }
  validateSignedTargets(value.signedTargets, "release sign-off summary signedTargets");
  validateBackendRuntimeSmokes(
    value.backendRuntimeSmokes,
    "release sign-off summary backendRuntimeSmokes"
  );

  if (!sameSignedTargets(value.signedTargets, signoff.signedTargets)) {
    fail("release sign-off summary signedTargets must match release sign-off signedTargets");
  }
  if (!sameStringArray(value.backendRuntimeSmokes, signoff.backendRuntimeSmokes)) {
    fail("release sign-off summary backendRuntimeSmokes must match release sign-off backendRuntimeSmokes");
  }
  if (!sameStringArray(value.commands, signoff.commands)) {
    fail("release sign-off summary commands must match release sign-off commands");
  }
  if (!sameStringArray(value.apiSymbols, signoff.apiSymbols)) {
    fail("release sign-off summary apiSymbols must match release sign-off apiSymbols");
  }
}

function validatePublishArtifact(value, manifest) {
  expectEqual(value.status, "ok", "publish artifact status");
  expectEqual(value.package, manifest.packageName, "publish artifact package");
  expectEqual(value.packageVersion, manifest.packageVersion, "publish artifact packageVersion");
  expectEqual(value.tarball, manifest.tarball, "publish artifact tarball");
  expectEqual(value.tarballSha256, manifest.tarballSha256, "publish artifact tarballSha256");
  if (typeof value.tarballPath !== "string" || value.tarballPath.length === 0) {
    fail("publish artifact tarballPath is missing");
  } else if (basename(value.tarballPath) !== manifest.tarball) {
    fail(
      `publish artifact tarballPath must end with ${manifest.tarball}, ` +
        `found ${JSON.stringify(value.tarballPath)}`
    );
  }
}

function validatePublishResult(value, manifest) {
  expectEqual(value.status, "ok", "publish result status");
  expectEqual(value.package, manifest.packageName, "publish result package");
  expectEqual(value.packageVersion, manifest.packageVersion, "publish result packageVersion");
  expectEqual(value.version, manifest.packageVersion, "publish result version");
  expectEqual(value.tarball, manifest.tarball, "publish result tarball");
  expectEqual(value.publishPackage, manifest.packageName, "publish result publishPackage");
  expectEqual(value.publishVersion, manifest.packageVersion, "publish result publishVersion");
  expectEqual(
    value.publishId,
    `${manifest.packageName}@${manifest.packageVersion}`,
    "publish result publishId"
  );
  expectEqual(value.publishFilename, manifest.tarball, "publish result publishFilename");
  expectEqual(value.registryStatus, "ok", "publish result registryStatus");
  if (typeof value.registryTarball !== "string" || !value.registryTarball.endsWith(`/${manifest.packageName}/-/${manifest.tarball}`)) {
    fail(
      `publish result registryTarball must end with /${manifest.packageName}/-/${manifest.tarball}, ` +
        `found ${JSON.stringify(value.registryTarball)}`
    );
  }
  if (!isSha512Integrity(value.publishIntegrity)) {
    fail(`publish result publishIntegrity must be a sha512 npm integrity string, found ${JSON.stringify(value.publishIntegrity)}`);
  }
  if (!isSha512Integrity(value.integrity)) {
    fail(`publish result integrity must be a sha512 npm integrity string, found ${JSON.stringify(value.integrity)}`);
  }
  if (!isSha1(value.publishShasum)) {
    fail(`publish result publishShasum must be a sha1 hex string, found ${JSON.stringify(value.publishShasum)}`);
  }
  if (!isSha1(value.shasum)) {
    fail(`publish result shasum must be a sha1 hex string, found ${JSON.stringify(value.shasum)}`);
  }
  expectEqual(value.publishIntegrity, value.integrity, "publish result publishIntegrity");
  expectEqual(value.publishShasum, value.shasum, "publish result publishShasum");
  expectEqual(value.description, EXPECTED_PACKAGE_DESCRIPTION, "publish result description");
  expectJson(value.keywords, EXPECTED_PACKAGE_KEYWORDS, "publish result keywords");
  expectEqual(value.license, EXPECTED_PACKAGE_LICENSE, "publish result license");
  expectJson(value.engines, EXPECTED_PACKAGE_ENGINES, "publish result engines");
  expectEqual(
    value.description,
    manifest.packageMetadata?.description,
    "publish result description from release manifest packageMetadata"
  );
  expectJson(
    value.keywords,
    manifest.packageMetadata?.keywords,
    "publish result keywords from release manifest packageMetadata"
  );
  expectEqual(
    value.license,
    manifest.packageMetadata?.license,
    "publish result license from release manifest packageMetadata"
  );
  expectJson(
    value.engines,
    manifest.packageMetadata?.engines,
    "publish result engines from release manifest packageMetadata"
  );
  expectEmptyArray(value.consumerInstallScripts, "publish result consumerInstallScripts");
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

function expectJson(actual, expected, label) {
  if (JSON.stringify(actual) !== JSON.stringify(expected)) {
    fail(`${label} must be ${JSON.stringify(expected)}, found ${JSON.stringify(actual)}`);
  }
}

function sameStringArray(actual, expected) {
  return Array.isArray(actual)
    && Array.isArray(expected)
    && actual.length === expected.length
    && actual.every((value, index) => value === expected[index]);
}

function sameSignedTargets(actual, expected) {
  return Array.isArray(actual)
    && Array.isArray(expected)
    && actual.length === expected.length
    && actual.every((target, index) => (
      target?.name === expected[index]?.name
      && target?.platform === expected[index]?.platform
      && target?.arch === expected[index]?.arch
      && target?.sha256 === expected[index]?.sha256
      && target?.nodeVersion === expected[index]?.nodeVersion
      && target?.npmVersion === expected[index]?.npmVersion
      && target?.installedBin === expected[index]?.installedBin
      && target?.packagedBinary === expected[index]?.packagedBinary
      && target?.packagedBinarySha256 === expected[index]?.packagedBinarySha256
    ));
}

function expectEmptyArray(actual, label) {
  if (!Array.isArray(actual) || actual.length !== 0) {
    fail(`${label} must be an empty array, found ${JSON.stringify(actual)}`);
  }
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
    validateSignedTargetBinaryEvidence(target, expectedTarget, label);
  }
}

function validateSignedTargetRuntimeEnvironment(actual, expectedTarget, label) {
  requireNonEmptyString(actual?.nodeVersion, `${label} ${expectedTarget.name} nodeVersion`);
  requireNonEmptyString(actual?.npmVersion, `${label} ${expectedTarget.name} npmVersion`);
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

function isSha256(value) {
  return typeof value === "string" && /^[0-9a-f]{64}$/.test(value);
}

function isSha512Integrity(value) {
  return typeof value === "string" && /^sha512-[A-Za-z0-9+/]{86}==$/.test(value);
}

function isSha1(value) {
  return typeof value === "string" && /^[0-9a-f]{40}$/.test(value);
}

function fail(message) {
  failures.push(message);
}

function failImmediate(message) {
  console.error(`verify-npm-cutover-evidence: ${message}`);
  process.exit(1);
}
