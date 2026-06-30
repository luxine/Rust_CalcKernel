#!/usr/bin/env node
import { existsSync, readFileSync } from "node:fs";
import { basename, resolve } from "node:path";

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
const [manifestArg, publishArg, registryArg] = process.argv.slice(2);

if (!manifestArg || !publishArg || !registryArg || manifestArg === "--help" || manifestArg === "-h") {
  console.error(
    "Usage: node scripts/verify-npm-publish-result.mjs " +
      "<release-manifest.json> <npm-publish.json> <npm-registry-replacement.json>"
  );
  process.exit(manifestArg ? 0 : 1);
}

const manifestPath = resolve(manifestArg);
const publishPath = resolve(publishArg);
const registryPath = resolve(registryArg);
const manifest = readJsonFile(manifestPath, "release manifest");
const publish = normalizePublishResult(readJsonFile(publishPath, "npm publish result"));
const registry = readJsonFile(registryPath, "npm registry replacement result");
const failures = [];

validateManifest(manifest);
expectEqual(publish.name, manifest.packageName, "publish package name");
expectEqual(publish.version, manifest.packageVersion, "publish package version");
expectEqual(publish.id, `${manifest.packageName}@${manifest.packageVersion}`, "publish id");
expectEqual(publish.filename, manifest.tarball, "publish tarball filename");
expectEqual(registry.status, "ok", "registry replacement status");
expectEqual(registry.package, manifest.packageName, "registry package name");
expectEqual(registry.packageVersion, manifest.packageVersion, "registry packageVersion");
expectEqual(registry.version, manifest.packageVersion, "registry package version");
expectRegistryTarball(registry.tarball, manifest.packageName, manifest.tarball);
expectEmptyArray(registry.consumerInstallScripts, "registry consumerInstallScripts");
expectEqual(registry.description, EXPECTED_PACKAGE_DESCRIPTION, "registry description");
expectJson(registry.keywords, EXPECTED_PACKAGE_KEYWORDS, "registry keywords");
expectEqual(registry.license, EXPECTED_PACKAGE_LICENSE, "registry license");
expectJson(registry.engines, EXPECTED_PACKAGE_ENGINES, "registry engines");
expectEqual(
  registry.description,
  manifest.packageMetadata?.description,
  "registry description from release manifest packageMetadata"
);
expectJson(
  registry.keywords,
  manifest.packageMetadata?.keywords,
  "registry keywords from release manifest packageMetadata"
);
expectEqual(
  registry.license,
  manifest.packageMetadata?.license,
  "registry license from release manifest packageMetadata"
);
expectJson(
  registry.engines,
  manifest.packageMetadata?.engines,
  "registry engines from release manifest packageMetadata"
);

if (!isSha512Integrity(publish.integrity)) {
  fail(`publish integrity must be a sha512 npm integrity string, found ${JSON.stringify(publish.integrity)}`);
}
if (!isSha512Integrity(registry.integrity)) {
  fail(`registry integrity must be a sha512 npm integrity string, found ${JSON.stringify(registry.integrity)}`);
}
if (!isSha1(publish.shasum)) {
  fail(`publish shasum must be a sha1 hex string, found ${JSON.stringify(publish.shasum)}`);
}
if (!isSha1(registry.shasum)) {
  fail(`registry shasum must be a sha1 hex string, found ${JSON.stringify(registry.shasum)}`);
}
if (publish.integrity && registry.integrity && publish.integrity !== registry.integrity) {
  fail(
    `registry integrity must match npm publish integrity: ` +
      `expected ${publish.integrity}, found ${registry.integrity}`
  );
}
if (publish.shasum && registry.shasum && publish.shasum !== registry.shasum) {
  fail(
    `registry shasum must match npm publish shasum: ` +
      `expected ${publish.shasum}, found ${registry.shasum}`
  );
}

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
  publishPackage: publish.name,
  publishVersion: publish.version,
  publishId: publish.id,
  publishFilename: publish.filename,
  publishShasum: publish.shasum,
  publishIntegrity: publish.integrity,
  registryStatus: registry.status,
  registryTarball: registry.tarball,
  shasum: registry.shasum,
  description: registry.description,
  keywords: registry.keywords,
  license: registry.license,
  engines: registry.engines,
  consumerInstallScripts: registry.consumerInstallScripts,
  integrity: registry.integrity
}, null, 2));

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

function normalizePublishResult(result) {
  if (Array.isArray(result)) {
    if (result.length !== 1) {
      failImmediate(`npm publish result must contain exactly one package, found ${result.length}`);
    }
    return result[0];
  }
  return result;
}

function validateManifest(manifest) {
  expectEqual(manifest.packageName, "calckernel", "release manifest packageName");
  if (!manifest.packageVersion) {
    fail("release manifest is missing packageVersion");
  }
  if (!manifest.tarball || basename(manifest.tarball) !== manifest.tarball) {
    fail(`release manifest tarball must be a tarball filename, found ${JSON.stringify(manifest.tarball)}`);
  }
  if (!isSha256(manifest.tarballSha256)) {
    fail(`release manifest tarballSha256 is invalid: ${JSON.stringify(manifest.tarballSha256)}`);
  }
  if (
    !manifest.packageMetadata
    || typeof manifest.packageMetadata !== "object"
    || Array.isArray(manifest.packageMetadata)
  ) {
    fail("release manifest packageMetadata is missing");
  } else {
    expectJson(manifest.packageMetadata, EXPECTED_PACKAGE_METADATA, "release manifest packageMetadata");
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

function expectRegistryTarball(tarball, packageName, tarballFile) {
  const expectedSuffix = `/${packageName}/-/${basename(tarballFile)}`;
  if (typeof tarball !== "string" || !tarball.endsWith(expectedSuffix)) {
    fail(`registry tarball must end with ${expectedSuffix}, found ${JSON.stringify(tarball)}`);
  }
}

function expectEmptyArray(actual, label) {
  if (!Array.isArray(actual) || actual.length !== 0) {
    fail(`${label} must be an empty array, found ${JSON.stringify(actual)}`);
  }
}

function isSha512Integrity(value) {
  return typeof value === "string" && /^sha512-[A-Za-z0-9+/]{86}==$/.test(value);
}

function isSha1(value) {
  return typeof value === "string" && /^[0-9a-f]{40}$/.test(value);
}

function isSha256(value) {
  return typeof value === "string" && /^[0-9a-f]{64}$/.test(value);
}

function fail(message) {
  failures.push(message);
}

function failImmediate(message) {
  console.error(`verify-npm-publish-result: ${message}`);
  process.exit(1);
}
