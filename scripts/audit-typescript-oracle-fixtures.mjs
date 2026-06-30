#!/usr/bin/env node
import { existsSync, readdirSync, readFileSync, statSync } from "node:fs";
import { dirname, join, relative, resolve, sep } from "node:path";
import { fileURLToPath } from "node:url";

const root = resolve(dirname(fileURLToPath(import.meta.url)), "..");
const tsRoot = process.env.CALCKERNEL_TS_ROOT ?? "/Users/lynn/code/CalcKernel";
const fixtureRoots = ["examples", "bench/perf/fixtures", "tests/fixtures"];
const backendCoverage = [
  ["MIR", "tests/mir_test.rs"],
  ["C", "tests/c_backend_test.rs"],
  ["WASM", "tests/wasm_backend_test.rs"],
  ["LLVM", "tests/llvm_backend_test.rs"]
];
const failures = [];

if (!existsSync(tsRoot)) {
  fail(`TypeScript oracle root is missing: ${tsRoot}`);
}

const fixtures = fixtureRoots.flatMap((fixtureRoot) => listCkFiles(join(tsRoot, fixtureRoot)));
const generatedOutputFixtures = fixtures;
const auxiliaryFixtures = fixtures.filter((fixture) => !generatedOutputFixtures.includes(fixture));

const backendContents = new Map(
  backendCoverage.map(([label, path]) => [label, readRustFile(path)])
);
const allRustTests = listFiles(join(root, "tests"))
  .filter((path) => path.endsWith(".rs"))
  .map((path) => readFileSync(path, "utf8"))
  .join("\n");

for (const fixture of generatedOutputFixtures) {
  for (const [label] of backendCoverage) {
    if (!backendContents.get(label).includes(fixture)) {
      fail(`${fixture} is missing from ${label} backend oracle coverage`);
    }
  }
}

for (const fixture of fixtures) {
  if (!allRustTests.includes(fixture)) {
    fail(`${fixture} is not referenced by any Rust oracle test`);
  }
}

if (failures.length > 0) {
  for (const failure of failures) {
    console.error(failure);
  }
  process.exit(1);
}

console.log(JSON.stringify({
  status: "ok",
  typescriptOracleRoot: tsRoot,
  fixtureRoots,
  fixtureCount: fixtures.length,
  generatedOutputFixtureCount: generatedOutputFixtures.length,
  auxiliaryFixtureCount: auxiliaryFixtures.length,
  backendCoverage: Object.fromEntries(
    backendCoverage.map(([label, path]) => [label, path])
  ),
  generatedOutputFixtures,
  auxiliaryFixtures
}, null, 2));

function readRustFile(path) {
  const absolute = join(root, path);
  if (!existsSync(absolute)) {
    fail(`Rust test file is missing: ${path}`);
    return "";
  }
  return readFileSync(absolute, "utf8");
}

function listCkFiles(dir) {
  if (!existsSync(dir)) {
    fail(`TypeScript fixture directory is missing: ${dir}`);
    return [];
  }
  return listFiles(dir)
    .filter((path) => path.endsWith(".ck"))
    .map((path) => normalizeRelative(tsRoot, path))
    .sort();
}

function listFiles(dir) {
  const files = [];
  for (const entry of readdirSync(dir)) {
    const path = join(dir, entry);
    const stats = statSync(path);
    if (stats.isDirectory()) {
      files.push(...listFiles(path));
    } else if (stats.isFile()) {
      files.push(path);
    }
  }
  return files;
}

function normalizeRelative(base, path) {
  return relative(base, path).split(sep).join("/");
}

function fail(message) {
  failures.push(message);
}
