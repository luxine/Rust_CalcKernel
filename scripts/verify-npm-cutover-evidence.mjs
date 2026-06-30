#!/usr/bin/env node
import { existsSync, readFileSync } from "node:fs";
import { basename, resolve } from "node:path";
import { SUPPORTED_CKC_BINARY_TARGETS, binaryNameForTarget, supportedTargetNames } from "../npm/platform.js";

const EXPECTED_PACKAGE_DESCRIPTION = "A small CK / CalcKernel integer-computation DSL compiler with C, WASM, and LLVM backends.";
const EXPECTED_PACKAGE_KEYWORDS = ["calckernel", "ck", "compiler", "dsl", "c", "wasm", "llvm"];
const EXPECTED_PACKAGE_LICENSE = "MIT";
const EXPECTED_PACKAGE_ENGINES = { node: ">=20" };
const RELEASE_WORKFLOW = "npm release artifact";
const PLATFORM_SIGNOFF_JOB = "platform-signoff";
const PUBLISH_JOB = "publish-npm";
const PUBLISH_RUNNER_OS = "Linux";
const PUBLISH_RUNNER_ARCH = "X64";
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
  consumerInstallScripts: [],
  packageManager: null,
  scriptNames: EXPECTED_PACKAGE_SCRIPT_NAMES
});
const EXPECTED_PACKAGE_JSON_FILES = Object.freeze([
  "npm",
  "README.md",
  "README.zh-CN.md",
  "docs/npm-release.md",
  "docs/architecture-review.md",
  "docs/zh-CN/architecture-review.md"
]);
const REQUIRED_FILES = Object.freeze([
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
]);
const FORBIDDEN_PREFIXES = Object.freeze([
  "package/docs/superpowers/",
  "package/src/",
  "package/target/"
]);
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
  sourceGitSha: manifest.sourceGitSha,
  sourceRepository: manifest.sourceRepository,
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
  publishProvenance: publishResult.publishProvenance,
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
  if (!isGitSha(value.sourceGitSha)) {
    fail(`release manifest sourceGitSha must be a 40-character lowercase hex commit SHA, found ${JSON.stringify(value.sourceGitSha)}`);
  }
  if (!isSourceRepository(value.sourceRepository)) {
    fail(
      `release manifest sourceRepository must be "local" or a GitHub owner/repository value, ` +
        `found ${JSON.stringify(value.sourceRepository)}`
    );
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
  validateReleaseManifestFileSurface(value.fileSurface);
  validateReleaseManifestTargets(value.targets, "release manifest targets");
}

function validateReleaseSignoff(value, manifest) {
  expectEqual(value.status, "ok", "release sign-off status");
  expectEqual(value.package, manifest.packageName, "release sign-off package");
  expectEqual(value.packageVersion, manifest.packageVersion, "release sign-off packageVersion");
  expectEqual(value.tarball, manifest.tarball, "release sign-off tarball");
  expectEqual(value.tarballSha256, manifest.tarballSha256, "release sign-off tarballSha256");
  expectEqual(value.sourceGitSha, manifest.sourceGitSha, "release sign-off sourceGitSha");
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

  const manifestTargetShaByName = new Map(
    Array.isArray(manifest.targets)
      ? manifest.targets.map((target) => [target.name, target.sha256])
      : []
  );
  for (const target of value.signedTargets ?? []) {
    if (target.sha256 !== manifestTargetShaByName.get(target.name)) {
      fail(
        `release sign-off signedTargets ${target.name} sha256 must match release manifest target sha256`
      );
    }
    if (target.githubSha !== manifest.sourceGitSha) {
      fail(`release sign-off signedTargets ${target.name} githubSha must match release manifest sourceGitSha`);
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
  expectEqual(value.sourceGitSha, manifest.sourceGitSha, "release sign-off summary sourceGitSha");
  expectEqual(value.sourceGitSha, signoff.sourceGitSha, "release sign-off summary sourceGitSha");
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
  expectEqual(value.sourceGitSha, manifest.sourceGitSha, "publish artifact sourceGitSha");
  expectEqual(value.sourceRepository, manifest.sourceRepository, "publish artifact sourceRepository");
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
  expectEqual(value.sourceGitSha, manifest.sourceGitSha, "publish result sourceGitSha");
  expectEqual(value.sourceRepository, manifest.sourceRepository, "publish result sourceRepository");
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
  validatePublishProvenance(
    value.publishProvenance,
    "publish result publishProvenance",
    manifest.sourceGitSha,
    manifest.sourceRepository
  );
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
      && target?.ciProvider === expected[index]?.ciProvider
      && target?.githubRunId === expected[index]?.githubRunId
      && target?.githubRunAttempt === expected[index]?.githubRunAttempt
      && target?.githubSha === expected[index]?.githubSha
      && target?.githubWorkflow === expected[index]?.githubWorkflow
      && target?.githubJob === expected[index]?.githubJob
      && target?.runnerOs === expected[index]?.runnerOs
      && target?.runnerArch === expected[index]?.runnerArch
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

function validateReleaseManifestFileSurface(actual) {
  if (!actual || typeof actual !== "object" || Array.isArray(actual)) {
    fail("release manifest fileSurface is missing");
    return;
  }
  expectJson(
    actual.packageJsonFiles,
    EXPECTED_PACKAGE_JSON_FILES,
    "release manifest fileSurface.packageJsonFiles"
  );
  expectJson(actual.requiredFiles, REQUIRED_FILES, "release manifest fileSurface.requiredFiles");
  expectJson(
    actual.forbiddenPrefixes,
    FORBIDDEN_PREFIXES,
    "release manifest fileSurface.forbiddenPrefixes"
  );
  expectJson(
    actual.allowedEntries,
    expectedAllowedEntries(),
    "release manifest fileSurface.allowedEntries"
  );
}

function validateReleaseManifestTargets(actual, label) {
  const expectedTargets = supportedTargetNames();
  if (!Array.isArray(actual)) {
    fail(`${label} must be an array`);
    return;
  }
  const actualNames = actual.map((target) => target?.name);
  if (!sameStringArray(actualNames, expectedTargets)) {
    fail(`${label} names must be ${JSON.stringify(expectedTargets)}, found ${JSON.stringify(actualNames)}`);
  }
  for (const [index, target] of actual.entries()) {
    const expectedTarget = SUPPORTED_CKC_BINARY_TARGETS[index];
    if (!expectedTarget) {
      fail(`${label} unexpected target at index ${index}`);
      continue;
    }
    const targetName = target?.name ?? "unknown";
    const targetLabel = `${label} ${targetName}`;
    expectEqual(target?.rustTarget, expectedTarget.rustTarget, `${targetLabel} rustTarget`);
    expectEqual(
      target?.binaryPath,
      `package/npm/bin/${binaryNameForTarget(expectedTarget.name)}`,
      `${targetLabel} binaryPath`
    );
    if (typeof target?.fileMode !== "string" || target.fileMode.length === 0) {
      fail(`${targetLabel} fileMode is missing`);
    } else if (expectedTarget.platform !== "win32" && !hasOwnerExecuteBit(target.fileMode)) {
      fail(`${targetLabel} fileMode must be executable, found ${JSON.stringify(target.fileMode)}`);
    }
    expectEqual(target?.binaryFormat, expectedBinaryFormat(expectedTarget), `${targetLabel} binaryFormat`);
    expectEqual(target?.binaryArchitecture, expectedTarget.arch, `${targetLabel} binaryArchitecture`);
    if (!Number.isSafeInteger(target?.sizeBytes) || target.sizeBytes <= 0) {
      fail(`${targetLabel} sizeBytes must be a positive integer`);
    }
    if (!isSha256(target?.sha256)) {
      fail(`${targetLabel} sha256 is invalid`);
    }
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

function expectedAllowedEntries() {
  return [
    ...REQUIRED_FILES,
    ...SUPPORTED_CKC_BINARY_TARGETS.map((target) => `package/npm/bin/${binaryNameForTarget(target.name)}`)
  ].sort();
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
  return undefined;
}

function hasOwnerExecuteBit(mode) {
  return mode.length >= 4 && (mode[3] === "x" || mode[3] === "s");
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

function validatePublishProvenance(actual, label, sourceGitSha, sourceRepository) {
  if (!actual || typeof actual !== "object" || Array.isArray(actual)) {
    fail(`${label} is missing`);
    return;
  }
  if (actual.ciProvider !== "github-actions") {
    fail(`${label} ciProvider must be "github-actions"`);
  }
  requireDigits(actual.githubRunId, `${label} githubRunId`);
  requireDigits(actual.githubRunAttempt, `${label} githubRunAttempt`);
  if (typeof actual.githubSha !== "string" || !/^[0-9a-f]{40}$/.test(actual.githubSha)) {
    fail(`${label} githubSha must be a 40-character lowercase hex commit SHA`);
  } else if (actual.githubSha !== sourceGitSha) {
    fail(`${label} githubSha must match release manifest sourceGitSha`);
  }
  if (!isGitHubRepository(actual.githubRepository)) {
    fail(`${label} githubRepository must be a GitHub owner/repository value`);
  } else if (actual.githubRepository !== sourceRepository) {
    fail(`${label} githubRepository must match release manifest sourceRepository`);
  }
  if (actual.githubWorkflow !== RELEASE_WORKFLOW) {
    fail(`${label} githubWorkflow must be ${JSON.stringify(RELEASE_WORKFLOW)}`);
  }
  if (actual.githubJob !== PUBLISH_JOB) {
    fail(`${label} githubJob must be ${JSON.stringify(PUBLISH_JOB)}`);
  }
  if (actual.runnerOs !== PUBLISH_RUNNER_OS) {
    fail(`${label} runnerOs must be ${JSON.stringify(PUBLISH_RUNNER_OS)}`);
  }
  if (actual.runnerArch !== PUBLISH_RUNNER_ARCH) {
    fail(`${label} runnerArch must be ${JSON.stringify(PUBLISH_RUNNER_ARCH)}`);
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

function isGitSha(value) {
  return typeof value === "string" && /^[0-9a-f]{40}$/.test(value);
}

function isSourceRepository(value) {
  return value === "local" || isGitHubRepository(value);
}

function isGitHubRepository(value) {
  return typeof value === "string" && /^[A-Za-z0-9_.-]+\/[A-Za-z0-9_.-]+$/.test(value);
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
