#!/usr/bin/env node
import { existsSync, readFileSync } from "node:fs";
import { basename, resolve } from "node:path";

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

expectEqual(publish.name, manifest.packageName, "publish package name");
expectEqual(publish.version, manifest.packageVersion, "publish package version");
expectEqual(publish.id, `${manifest.packageName}@${manifest.packageVersion}`, "publish id");
expectEqual(publish.filename, manifest.tarball, "publish tarball filename");
expectEqual(registry.status, "ok", "registry replacement status");
expectEqual(registry.package, manifest.packageName, "registry package name");
expectEqual(registry.version, manifest.packageVersion, "registry package version");
expectRegistryTarball(registry.tarball, manifest.packageName, manifest.tarball);
expectEmptyArray(registry.consumerInstallScripts, "registry consumerInstallScripts");

if (!isSha512Integrity(publish.integrity)) {
  fail(`publish integrity must be a sha512 npm integrity string, found ${JSON.stringify(publish.integrity)}`);
}
if (!isSha512Integrity(registry.integrity)) {
  fail(`registry integrity must be a sha512 npm integrity string, found ${JSON.stringify(registry.integrity)}`);
}
if (publish.integrity && registry.integrity && publish.integrity !== registry.integrity) {
  fail(
    `registry integrity must match npm publish integrity: ` +
      `expected ${publish.integrity}, found ${registry.integrity}`
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
  version: manifest.packageVersion,
  tarball: manifest.tarball,
  registryStatus: registry.status,
  registryTarball: registry.tarball,
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

function expectEqual(actual, expected, label) {
  if (actual !== expected) {
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

function fail(message) {
  failures.push(message);
}

function failImmediate(message) {
  console.error(`verify-npm-publish-result: ${message}`);
  process.exit(1);
}
