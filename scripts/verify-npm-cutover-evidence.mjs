#!/usr/bin/env node
import { existsSync, readFileSync } from "node:fs";
import { basename, resolve } from "node:path";
import { supportedTargetNames } from "../npm/platform.js";

const [manifestArg, signoffArg, publishArtifactArg, publishResultArg] = process.argv.slice(2);

if (
  !manifestArg
  || !signoffArg
  || !publishArtifactArg
  || !publishResultArg
  || manifestArg === "--help"
  || manifestArg === "-h"
) {
  console.error(
    "Usage: node scripts/verify-npm-cutover-evidence.mjs " +
      "<release-manifest.json> <release-signoff.json> " +
      "<npm-publish-artifact.json> <npm-publish-result.json>"
  );
  process.exit(manifestArg ? 0 : 1);
}

const manifestPath = resolve(manifestArg);
const signoffPath = resolve(signoffArg);
const publishArtifactPath = resolve(publishArtifactArg);
const publishResultPath = resolve(publishResultArg);
const manifest = readJsonFile(manifestPath, "release manifest");
const signoff = readJsonFile(signoffPath, "release sign-off");
const publishArtifact = readJsonFile(publishArtifactPath, "npm publish artifact verifier output");
const publishResult = readJsonFile(publishResultPath, "npm publish result verifier output");
const failures = [];

validateManifest(manifest);
validateReleaseSignoff(signoff, manifest);
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
  version: manifest.packageVersion,
  tarball: manifest.tarball,
  tarballSha256: manifest.tarballSha256,
  targetCount: signoff.targetCount,
  targets: signoff.targets,
  registryStatus: publishResult.registryStatus,
  registryTarball: publishResult.registryTarball,
  consumerInstallScripts: publishResult.consumerInstallScripts,
  integrity: publishResult.integrity,
  evidence: {
    manifest: manifestPath,
    releaseSignoff: signoffPath,
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
}

function validateReleaseSignoff(value, manifest) {
  expectEqual(value.status, "ok", "release sign-off status");
  expectEqual(value.package, manifest.packageName, "release sign-off package");
  expectEqual(value.packageVersion, manifest.packageVersion, "release sign-off packageVersion");
  expectEqual(value.tarball, manifest.tarball, "release sign-off tarball");
  expectEqual(value.tarballSha256, manifest.tarballSha256, "release sign-off tarballSha256");

  const expectedTargets = supportedTargetNames();
  if (value.targetCount !== expectedTargets.length) {
    fail(`release sign-off targetCount must be ${expectedTargets.length}, found ${JSON.stringify(value.targetCount)}`);
  }
  if (!sameStringArray(value.targets, expectedTargets)) {
    fail(`release sign-off targets must be ${JSON.stringify(expectedTargets)}, found ${JSON.stringify(value.targets)}`);
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

function sameStringArray(actual, expected) {
  return Array.isArray(actual)
    && actual.length === expected.length
    && actual.every((value, index) => value === expected[index]);
}

function expectEmptyArray(actual, label) {
  if (!Array.isArray(actual) || actual.length !== 0) {
    fail(`${label} must be an empty array, found ${JSON.stringify(actual)}`);
  }
}

function isSha256(value) {
  return typeof value === "string" && /^[0-9a-f]{64}$/.test(value);
}

function isSha512Integrity(value) {
  return typeof value === "string" && /^sha512-[A-Za-z0-9+/]{86}==$/.test(value);
}

function fail(message) {
  failures.push(message);
}

function failImmediate(message) {
  console.error(`verify-npm-cutover-evidence: ${message}`);
  process.exit(1);
}
