#!/usr/bin/env node
import { existsSync, readFileSync } from "node:fs";
import { basename, resolve } from "node:path";
import { supportedTargetNames } from "../npm/platform.js";

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

  const manifestTargetShaByName = new Map(manifest.targets.map((target) => [target.name, target.sha256]));
  for (const target of value.signedTargets ?? []) {
    if (target.sha256 !== manifestTargetShaByName.get(target.name)) {
      fail(
        `release sign-off signedTargets ${target.name} sha256 must match release manifest target sha256`
      );
    }
  }
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
