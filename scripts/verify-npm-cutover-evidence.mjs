#!/usr/bin/env node
import { existsSync, readFileSync } from "node:fs";
import { basename, resolve } from "node:path";
import { supportedTargetNames } from "../npm/platform.js";

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
  backendRuntimeSmokes: signoff.backendRuntimeSmokes,
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
  validateSignedTargets(value.targets, "release manifest targets");
}

function validateReleaseSignoff(value, manifest) {
  expectEqual(value.status, "ok", "release sign-off status");
  expectEqual(value.package, manifest.packageName, "release sign-off package");
  expectEqual(value.packageVersion, manifest.packageVersion, "release sign-off packageVersion");
  expectEqual(value.tarball, manifest.tarball, "release sign-off tarball");
  expectEqual(value.tarballSha256, manifest.tarballSha256, "release sign-off tarballSha256");
  expectEqual(value.sourceFallback, "disabled", "release sign-off sourceFallback");

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
}

function validatePublishArtifact(value, manifest) {
  expectEqual(value.status, "ok", "publish artifact status");
  expectEqual(value.package, manifest.packageName, "publish artifact package");
  expectEqual(value.packageVersion, manifest.packageVersion, "publish artifact packageVersion");
  expectEqual(value.tarball, manifest.tarball, "publish artifact tarball");
  expectEqual(value.tarballSha256, manifest.tarballSha256, "publish artifact tarballSha256");
}

function validatePublishResult(value, manifest) {
  expectEqual(value.status, "ok", "publish result status");
  expectEqual(value.package, manifest.packageName, "publish result package");
  expectEqual(value.packageVersion, manifest.packageVersion, "publish result packageVersion");
  expectEqual(value.version, manifest.packageVersion, "publish result version");
  expectEqual(value.tarball, manifest.tarball, "publish result tarball");
  expectEqual(value.registryStatus, "ok", "publish result registryStatus");
  if (typeof value.registryTarball !== "string" || !value.registryTarball.endsWith(`/${manifest.packageName}/-/${manifest.tarball}`)) {
    fail(
      `publish result registryTarball must end with /${manifest.packageName}/-/${manifest.tarball}, ` +
        `found ${JSON.stringify(value.registryTarball)}`
    );
  }
  if (!isSha512Integrity(value.integrity)) {
    fail(`publish result integrity must be a sha512 npm integrity string, found ${JSON.stringify(value.integrity)}`);
  }
  if (!isSha1(value.shasum)) {
    fail(`publish result shasum must be a sha1 hex string, found ${JSON.stringify(value.shasum)}`);
  }
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
    && actual.length === expected.length
    && actual.every((value, index) => value === expected[index]);
}

function sameSignedTargets(actual, expected) {
  return Array.isArray(actual)
    && Array.isArray(expected)
    && actual.length === expected.length
    && actual.every((target, index) => (
      target?.name === expected[index]?.name
      && target?.sha256 === expected[index]?.sha256
    ));
}

function expectEmptyArray(actual, label) {
  if (!Array.isArray(actual) || actual.length !== 0) {
    fail(`${label} must be an empty array, found ${JSON.stringify(actual)}`);
  }
}

function validateSignedTargets(actual, label) {
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

function validateBackendRuntimeSmokes(actual, label) {
  const expected = backendRuntimeSmokes();
  if (!sameStringArray(actual, expected)) {
    fail(`${label} must be ${JSON.stringify(expected)}, found ${JSON.stringify(actual)}`);
  }
}

function backendRuntimeSmokes() {
  return [
    "node smoke-c-runtime.mjs",
    "node smoke-wasm-runtime.mjs",
    "node smoke-llvm-object-runtime.mjs"
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
