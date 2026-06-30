#!/usr/bin/env node
import { createHash } from "node:crypto";
import { existsSync, readFileSync, statSync } from "node:fs";
import { basename, join, resolve } from "node:path";
import { SUPPORTED_CKC_BINARY_TARGETS, binaryNameForTarget, supportedTargetNames } from "../npm/platform.js";

const EXPECTED_PACKAGE_METADATA = Object.freeze({
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

const [manifestArg, distDirArg] = process.argv.slice(2);

if (!manifestArg || !distDirArg || manifestArg === "--help" || manifestArg === "-h") {
  console.error("Usage: node scripts/verify-npm-publish-artifact.mjs <release-manifest.json> <dist-dir>");
  process.exit(manifestArg ? 0 : 1);
}

const manifestPath = resolve(manifestArg);
const distDir = resolve(distDirArg);
if (!existsSync(manifestPath)) {
  fail(`Release manifest does not exist: ${manifestPath}`);
}
if (!existsSync(distDir) || !statSync(distDir).isDirectory()) {
  fail(`Distribution directory does not exist: ${distDir}`);
}

const manifest = readJson(manifestPath);
if (manifest.packageName !== "calckernel") {
  fail(`release manifest packageName must be "calckernel", found ${JSON.stringify(manifest.packageName)}`);
}
if (!manifest.packageVersion) {
  fail("release manifest is missing packageVersion");
}
if (!manifest.tarball || basename(manifest.tarball) !== manifest.tarball) {
  fail(`release manifest tarball must be a tarball filename, found ${JSON.stringify(manifest.tarball)}`);
}
if (!isSha256(manifest.tarballSha256)) {
  fail(`release manifest tarballSha256 is invalid: ${JSON.stringify(manifest.tarballSha256)}`);
}
validateReleaseManifestEvidence(manifest);

const tarballPath = join(distDir, manifest.tarball);
if (!existsSync(tarballPath) || !statSync(tarballPath).isFile()) {
  fail(`Publish tarball does not exist: ${tarballPath}`);
}

const tarballBytes = readFileSync(tarballPath);
const tarballSha256 = sha256(tarballBytes);
if (tarballSha256 !== manifest.tarballSha256) {
  fail(
    `publish tarballSha256 does not match release manifest: ` +
      `expected ${manifest.tarballSha256}, found ${tarballSha256}`
  );
}

console.log(JSON.stringify({
  status: "ok",
  package: manifest.packageName,
  packageVersion: manifest.packageVersion,
  tarball: manifest.tarball,
  tarballPath,
  tarballSha256
}, null, 2));

function readJson(path) {
  try {
    return JSON.parse(readFileSync(path, "utf8"));
  } catch (error) {
    fail(`Unable to read release manifest: ${error.message}`);
  }
}

function validateReleaseManifestEvidence(manifest) {
  if (!manifest.packageMetadata || typeof manifest.packageMetadata !== "object") {
    fail("release manifest packageMetadata is missing");
  } else {
    expectJson(manifest.packageMetadata, EXPECTED_PACKAGE_METADATA, "release manifest packageMetadata");
  }

  if (!manifest.fileSurface || typeof manifest.fileSurface !== "object") {
    fail("release manifest fileSurface is missing");
  } else {
    expectJson(
      manifest.fileSurface.packageJsonFiles,
      EXPECTED_PACKAGE_JSON_FILES,
      "release manifest fileSurface.packageJsonFiles"
    );
    expectJson(manifest.fileSurface.requiredFiles, REQUIRED_FILES, "release manifest fileSurface.requiredFiles");
    expectJson(
      manifest.fileSurface.forbiddenPrefixes,
      FORBIDDEN_PREFIXES,
      "release manifest fileSurface.forbiddenPrefixes"
    );
    expectJson(
      manifest.fileSurface.allowedEntries,
      expectedAllowedEntries(),
      "release manifest fileSurface.allowedEntries"
    );
  }

  requireArray(manifest.targets, "release manifest targets");
  const expectedTargets = supportedTargetNames();
  const actualTargets = Array.isArray(manifest.targets)
    ? manifest.targets.map((target) => target.name)
    : [];
  if (!sameStringArray(actualTargets, expectedTargets)) {
    fail(`release manifest targets must be ${JSON.stringify(expectedTargets)}, found ${JSON.stringify(actualTargets)}`);
  }
  if (Array.isArray(manifest.targets)) {
    for (const [index, target] of manifest.targets.entries()) {
      const expectedTarget = SUPPORTED_CKC_BINARY_TARGETS[index];
      const label = `${target.name ?? "unknown"} release manifest`;
      expectEqual(target.rustTarget, expectedTarget.rustTarget, `${label} rustTarget`);
      expectEqual(
        target.binaryPath,
        `package/npm/bin/${binaryNameForTarget(expectedTarget.name)}`,
        `${label} binaryPath`
      );
      if (typeof target.fileMode !== "string" || target.fileMode.length === 0) {
        fail(`${label} fileMode is missing`);
      }
      if (typeof target.binaryFormat !== "string" || target.binaryFormat.length === 0) {
        fail(`${label} binaryFormat is missing`);
      }
      if (typeof target.binaryArchitecture !== "string" || target.binaryArchitecture.length === 0) {
        fail(`${label} binaryArchitecture is missing`);
      }
      if (!Number.isSafeInteger(target.sizeBytes) || target.sizeBytes <= 0) {
        fail(`${label} sizeBytes must be a positive integer`);
      }
      if (!isSha256(target.sha256)) {
        fail(`${label} binary sha256 is invalid`);
      }
    }
  }
}

function sha256(bytes) {
  return createHash("sha256").update(bytes).digest("hex");
}

function requireArray(value, label) {
  if (!Array.isArray(value) || value.length === 0) {
    fail(`${label} must be a non-empty array`);
  }
}

function expectedAllowedEntries() {
  return [
    ...REQUIRED_FILES,
    ...SUPPORTED_CKC_BINARY_TARGETS.map((target) => `package/npm/bin/${binaryNameForTarget(target.name)}`)
  ].sort();
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

function isSha256(value) {
  return typeof value === "string" && /^[0-9a-f]{64}$/.test(value);
}

function fail(message) {
  console.error(`verify-npm-publish-artifact: ${message}`);
  process.exit(1);
}
