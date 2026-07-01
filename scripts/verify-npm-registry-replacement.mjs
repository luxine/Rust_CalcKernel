#!/usr/bin/env node
import { spawnSync } from "node:child_process";
import { existsSync, readFileSync } from "node:fs";
import { dirname, join, resolve } from "node:path";
import { fileURLToPath } from "node:url";

const root = resolve(dirname(fileURLToPath(import.meta.url)), "..");
const EXPECTED_PACKAGE_DESCRIPTION = "A small CK / CalcKernel integer-computation DSL compiler with C, WASM, and LLVM backends.";
const EXPECTED_PACKAGE_KEYWORDS = ["calckernel", "ck", "compiler", "dsl", "c", "wasm", "llvm"];
const EXPECTED_PACKAGE_REPOSITORY = {
  type: "git",
  url: "https://github.com/luxine/Rust_CalcKernel"
};
const EXPECTED_REGISTRY_REPOSITORY = {
  type: "git",
  url: "git+https://github.com/luxine/Rust_CalcKernel.git"
};
const EXPECTED_PACKAGE_LICENSE = "MIT";
const EXPECTED_PACKAGE_ENGINES = { node: ">=20" };
const EXPECTED_PACKAGE_BIN = { ckc: "./npm/ckc.js" };
const EXPECTED_REGISTRY_BIN = { ckc: "npm/ckc.js" };
const REGISTRY_RETRY_ATTEMPTS = positiveIntegerEnv("CK_NPM_REGISTRY_RETRY_ATTEMPTS", 12);
const REGISTRY_RETRY_DELAY_MS = positiveIntegerEnv("CK_NPM_REGISTRY_RETRY_DELAY_MS", 5000);
const options = parseArgs(process.argv.slice(2));
const packageJson = JSON.parse(readFileSync(join(root, "package.json"), "utf8"));
const version = options.version ?? packageJson.version;
const metadata = options.metadataFile
  ? readMetadataFile(options.metadataFile)
  : readRegistryMetadata(version);
const failures = [];

expectEqual(metadata.name, "calckernel", "name");
expectEqual(metadata.version, version, "version");
expectEqual(metadata.description, EXPECTED_PACKAGE_DESCRIPTION, "description");
expectJson(metadata.keywords, EXPECTED_PACKAGE_KEYWORDS, "keywords");
expectJsonOneOf(metadata.repository, [EXPECTED_PACKAGE_REPOSITORY, EXPECTED_REGISTRY_REPOSITORY], "repository");
expectEqual(metadata.license, EXPECTED_PACKAGE_LICENSE, "license");
expectJson(metadata.engines, EXPECTED_PACKAGE_ENGINES, "engines");
expectEqual(metadata.type, "module", "type");
expectEqual(metadata.main, "./npm/index.js", "main");
expectEqual(metadata.types, "./npm/index.d.ts", "types");
expectJson(metadata.exports, { ".": { types: "./npm/index.d.ts", import: "./npm/index.js" } }, "exports");
expectJsonOneOf(metadata.bin, [EXPECTED_PACKAGE_BIN, EXPECTED_REGISTRY_BIN], "bin");
expectNoDependencyFields(metadata);
expectNoConsumerInstallScripts(metadata);
expectTarball(metadata.dist?.tarball, version);
if (!isSha1(metadata.dist?.shasum)) {
  fail(`dist.shasum must be a sha1 hex string, found ${JSON.stringify(metadata.dist?.shasum)}`);
}
if (!isSha512Integrity(metadata.dist?.integrity)) {
  fail(`dist.integrity must be a sha512 npm integrity string, found ${JSON.stringify(metadata.dist?.integrity)}`);
}

if (failures.length > 0) {
  for (const failure of failures) {
    console.error(failure);
  }
  process.exit(1);
}

console.log(JSON.stringify({
  status: "ok",
  package: metadata.name,
  packageVersion: version,
  version,
  tarball: metadata.dist.tarball,
  shasum: metadata.dist.shasum,
  integrity: metadata.dist.integrity,
  description: metadata.description,
  keywords: metadata.keywords,
  repository: metadata.repository,
  license: metadata.license,
  engines: metadata.engines,
  consumerInstallScripts: [],
  bin: metadata.bin,
  main: metadata.main,
  types: metadata.types
}, null, 2));

function parseArgs(args) {
  const parsed = {
    metadataFile: undefined,
    version: undefined
  };
  const positional = [];

  for (let index = 0; index < args.length; index += 1) {
    const arg = args[index];
    if (arg === "--help" || arg === "-h") {
      printUsage();
      process.exit(0);
    }
    if (arg === "--metadata-file") {
      parsed.metadataFile = resolve(requireValue(args, ++index, arg));
      continue;
    }
    if (arg === "--version") {
      parsed.version = requireValue(args, ++index, arg);
      continue;
    }
    if (arg.startsWith("-")) {
      failImmediate(`Unknown option ${arg}`);
    }
    positional.push(arg);
  }

  if (positional.length > 1) {
    failImmediate("Expected at most one version argument");
  }
  if (positional[0] && parsed.version) {
    failImmediate("Pass version either positionally or with --version, not both");
  }
  parsed.version ??= positional[0];
  if (parsed.metadataFile && !existsSync(parsed.metadataFile)) {
    failImmediate(`Metadata file does not exist: ${parsed.metadataFile}`);
  }

  return parsed;
}

function requireValue(args, index, flag) {
  const value = args[index];
  if (!value || value.startsWith("-")) {
    failImmediate(`${flag} requires a value`);
  }
  return value;
}

function readMetadataFile(path) {
  return JSON.parse(readFileSync(path, "utf8"));
}

function readRegistryMetadata(version) {
  let lastResult;
  for (let attempt = 1; attempt <= REGISTRY_RETRY_ATTEMPTS; attempt += 1) {
    const result = spawnSync("npm", ["view", `calckernel@${version}`, "--json"], {
      encoding: "utf8"
    });
    if (result.error) {
      failImmediate(`npm view failed to start: ${result.error.message}`);
    }
    if (result.status === 0) {
      return JSON.parse(result.stdout);
    }
    lastResult = result;
    if (attempt < REGISTRY_RETRY_ATTEMPTS) {
      sleep(REGISTRY_RETRY_DELAY_MS);
    }
  }
  failImmediate(
    `npm view calckernel@${version} failed with status ${lastResult.status}\n` +
      `stdout:\n${lastResult.stdout}\n` +
      `stderr:\n${lastResult.stderr}`
  );
}

function expectEqual(actual, expected, label) {
  if (actual !== expected) {
    fail(`${label} must be ${expected}, found ${JSON.stringify(actual)}`);
  }
}

function expectJson(actual, expected, label) {
  if (JSON.stringify(actual) !== JSON.stringify(expected)) {
    fail(`${label} must be ${JSON.stringify(expected)}, found ${JSON.stringify(actual)}`);
  }
}

function expectJsonOneOf(actual, expectedValues, label) {
  if (!expectedValues.some((expected) => JSON.stringify(actual) === JSON.stringify(expected))) {
    fail(`${label} must be one of ${JSON.stringify(expectedValues)}, found ${JSON.stringify(actual)}`);
  }
}

function sleep(ms) {
  Atomics.wait(new Int32Array(new SharedArrayBuffer(4)), 0, 0, ms);
}

function positiveIntegerEnv(name, fallback) {
  const raw = process.env[name];
  if (raw === undefined || raw === "") {
    return fallback;
  }
  const value = Number.parseInt(raw, 10);
  if (!Number.isInteger(value) || value <= 0) {
    failImmediate(`${name} must be a positive integer, found ${JSON.stringify(raw)}`);
  }
  return value;
}

function expectNoDependencyFields(metadata) {
  for (const field of [
    "dependencies",
    "devDependencies",
    "optionalDependencies",
    "peerDependencies",
    "bundledDependencies",
    "bundleDependencies"
  ]) {
    const value = metadata[field];
    if (value && (Array.isArray(value) ? value.length > 0 : Object.keys(value).length > 0)) {
      fail(`${field} must be empty or absent`);
    }
  }
}

function expectNoConsumerInstallScripts(metadata) {
  const scripts = metadata.scripts;
  if (!scripts) {
    return;
  }
  if (typeof scripts !== "object" || Array.isArray(scripts)) {
    fail(`scripts must be an object when present, found ${JSON.stringify(scripts)}`);
    return;
  }
  for (const scriptName of ["preinstall", "install", "postinstall"]) {
    if (Object.hasOwn(scripts, scriptName)) {
      fail(`consumer install lifecycle script ${scriptName} must be absent`);
    }
  }
}

function expectTarball(tarball, version) {
  const expectedSuffix = `/calckernel/-/calckernel-${version}.tgz`;
  if (typeof tarball !== "string" || !tarball.endsWith(expectedSuffix)) {
    fail(`dist.tarball must end with ${expectedSuffix}, found ${JSON.stringify(tarball)}`);
  }
}

function isSha512Integrity(value) {
  return typeof value === "string" && /^sha512-[A-Za-z0-9+/]{86}==$/.test(value);
}

function isSha1(value) {
  return typeof value === "string" && /^[0-9a-f]{40}$/.test(value);
}

function printUsage() {
  console.log("Usage: node scripts/verify-npm-registry-replacement.mjs [--metadata-file file] [version]");
}

function fail(message) {
  failures.push(message);
}

function failImmediate(message) {
  console.error(`verify-npm-registry-replacement: ${message}`);
  process.exit(1);
}
