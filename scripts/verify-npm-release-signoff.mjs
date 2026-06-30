#!/usr/bin/env node
import { existsSync, readdirSync, readFileSync, statSync } from "node:fs";
import { resolve, join } from "node:path";
import { SUPPORTED_CKC_BINARY_TARGETS, binaryNameForTarget, supportedTargetNames } from "../npm/platform.js";

const [manifestArg, signoffDirArg] = process.argv.slice(2);
if (!manifestArg || !signoffDirArg || manifestArg === "--help" || manifestArg === "-h") {
  console.error("Usage: node scripts/verify-npm-release-signoff.mjs <release-manifest.json> <signoff-dir>");
  process.exit(manifestArg ? 0 : 1);
}

const manifestPath = resolve(manifestArg);
const signoffDir = resolve(signoffDirArg);
if (!existsSync(manifestPath)) {
  fail(`Release manifest does not exist: ${manifestPath}`);
}
if (!existsSync(signoffDir) || !statSync(signoffDir).isDirectory()) {
  fail(`Sign-off directory does not exist: ${signoffDir}`);
}

const manifest = readJson(manifestPath, "release manifest");
validateManifest(manifest);
const signoffs = readSignoffs(signoffDir);
const manifestTargetNames = new Set(manifest.targets.map((target) => target.name));
const manifestTargetsByName = new Map(manifest.targets.map((target) => [target.name, target]));
const signoffsByTarget = new Map();

for (const signoff of signoffs) {
  if (!signoff.targetName) {
    fail("sign-off is missing targetName");
  }
  if (!supportedTargetNames().includes(signoff.targetName)) {
    fail(`unsupported platform sign-off for ${signoff.targetName}`);
  }
  if (signoffsByTarget.has(signoff.targetName)) {
    fail(`duplicate platform sign-off for ${signoff.targetName}`);
  }
  signoffsByTarget.set(signoff.targetName, signoff);
}

const verifiedTargets = [];
const signedTargets = [];
for (const target of SUPPORTED_CKC_BINARY_TARGETS) {
  if (!manifestTargetNames.has(target.name)) {
    fail(`release manifest is missing target ${target.name}`);
  }
  const signoff = signoffsByTarget.get(target.name);
  if (!signoff) {
    fail(`missing platform sign-off for ${target.name}`);
  }
  const manifestTarget = manifestTargetsByName.get(target.name);
  validateSignoff(signoff, target, manifest, manifestTarget);
  verifiedTargets.push(target.name);
  signedTargets.push({ name: target.name, sha256: manifestTarget.sha256 });
}

console.log(JSON.stringify({
  status: "ok",
  package: manifest.packageName,
  packageVersion: manifest.packageVersion,
  tarball: manifest.tarball,
  tarballSha256: manifest.tarballSha256,
  targetCount: verifiedTargets.length,
  targets: verifiedTargets,
  signedTargets
}, null, 2));

function validateManifest(manifest) {
  if (manifest.packageName !== "calckernel") {
    fail(`release manifest packageName must be "calckernel", found ${JSON.stringify(manifest.packageName)}`);
  }
  if (!manifest.packageVersion) {
    fail("release manifest is missing packageVersion");
  }
  if (!manifest.tarball) {
    fail("release manifest is missing tarball");
  }
  if (!isSha256(manifest.tarballSha256)) {
    fail(`release manifest tarballSha256 is invalid: ${JSON.stringify(manifest.tarballSha256)}`);
  }
  if (!Array.isArray(manifest.targets)) {
    fail("release manifest targets must be an array");
  }
}

function validateSignoff(signoff, target, manifest, manifestTarget) {
  if (signoff.package !== "calckernel") {
    fail(`${target.name} sign-off package must be "calckernel"`);
  }
  if (signoff.tarball !== manifest.tarball) {
    fail(`${target.name} sign-off tarball must be ${manifest.tarball}`);
  }
  if (signoff.tarballSha256 !== manifest.tarballSha256) {
    fail(`${target.name} sign-off tarballSha256 does not match release manifest`);
  }
  if (signoff.platform && signoff.platform !== target.platform) {
    fail(`${target.name} sign-off platform must be ${target.platform}`);
  }
  if (signoff.arch && signoff.arch !== target.arch) {
    fail(`${target.name} sign-off arch must be ${target.arch}`);
  }
  if (signoff.ckcBinOverride !== "unset") {
    fail(`${target.name} sign-off must run with CKC_BIN unset`);
  }
  if (signoff.sourceFallback !== "disabled") {
    fail(`${target.name} source fallback must be disabled`);
  }
  validateBinaryEvidence(signoff, target, manifestTarget);
  if (signoff.typeSmoke !== "passed") {
    fail(`${target.name} sign-off must pass TypeScript declaration smoke`);
  }
  requireIncludes(signoff.commands, requiredCommands(), `${target.name} commands`);
  requireIncludes(signoff.apiSymbols, requiredApiSymbols(), `${target.name} apiSymbols`);
}

function validateBinaryEvidence(signoff, target, manifestTarget) {
  const installedBinName = target.platform === "win32" ? "ckc.cmd" : "ckc";
  requirePathSuffix(
    signoff.installedBin,
    `node_modules/.bin/${installedBinName}`,
    `${target.name} installedBin`
  );
  requirePathSuffix(
    signoff.packagedBinary,
    `node_modules/calckernel/npm/bin/${binaryNameForTarget(target.name)}`,
    `${target.name} packagedBinary`
  );
  if (!isSha256(manifestTarget?.sha256)) {
    fail(`${target.name} release manifest target sha256 is invalid`);
  }
  if (signoff.packagedBinarySha256 !== manifestTarget.sha256) {
    fail(
      `${target.name} sign-off packagedBinarySha256 does not match release manifest target sha256`
    );
  }
}

function readSignoffs(dir) {
  return readdirSync(dir)
    .filter((entry) => entry.endsWith(".json"))
    .sort()
    .map((entry) => readJson(join(dir, entry), `sign-off ${entry}`));
}

function readJson(path, label) {
  try {
    return JSON.parse(readFileSync(path, "utf8"));
  } catch (error) {
    fail(`Unable to read ${label}: ${error.message}`);
  }
}

function requireIncludes(actual, expected, label) {
  if (!Array.isArray(actual)) {
    fail(`${label} must be an array`);
  }
  for (const value of expected) {
    if (!actual.includes(value)) {
      fail(`${label} is missing ${value}`);
    }
  }
}

function requirePathSuffix(actual, expectedSuffix, label) {
  if (typeof actual !== "string" || actual.length === 0) {
    fail(`${label} is missing`);
  }
  const normalizedActual = actual.replace(/\\/g, "/");
  if (!normalizedActual.endsWith(expectedSuffix)) {
    fail(`${label} must end with ${expectedSuffix}, found ${JSON.stringify(actual)}`);
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
    "ckc build-llvm smoke.ck --kind object -o build/smoke.o"
  ];
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

function fail(message) {
  console.error(`verify-npm-release-signoff: ${message}`);
  process.exit(1);
}
