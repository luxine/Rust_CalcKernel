#!/usr/bin/env node
import { createHash } from "node:crypto";
import { existsSync, readFileSync, statSync } from "node:fs";
import { basename, join, resolve } from "node:path";

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

function sha256(bytes) {
  return createHash("sha256").update(bytes).digest("hex");
}

function isSha256(value) {
  return typeof value === "string" && /^[0-9a-f]{64}$/.test(value);
}

function fail(message) {
  console.error(`verify-npm-publish-artifact: ${message}`);
  process.exit(1);
}
