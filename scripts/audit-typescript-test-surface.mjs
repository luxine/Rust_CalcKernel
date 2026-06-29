#!/usr/bin/env node
import { existsSync, readFileSync, readdirSync, statSync } from "node:fs";
import { dirname, join, relative, resolve, sep } from "node:path";
import { fileURLToPath } from "node:url";

const root = resolve(dirname(fileURLToPath(import.meta.url)), "..");
const options = parseArgs(process.argv.slice(2));
const rustRoot = options.rustRoot ?? root;
const typescriptRoot =
  options.typescriptRoot ?? process.env.CALCKERNEL_TS_ROOT ?? "/Users/lynn/code/CalcKernel";
const mappingPath = options.mapping ?? join(rustRoot, "docs", "typescript-test-surface.json");
const failures = [];

const typescriptTests = listTypeScriptTests(typescriptRoot);
const mapping = readMapping(mappingPath);
const mappedTests = new Map();
const rustEvidenceFiles = new Set();

for (const [index, entry] of mapping.entries()) {
  const label = `mapping entry ${index + 1}`;
  if (!entry || typeof entry !== "object" || Array.isArray(entry)) {
    fail(`${label} must be an object`);
    continue;
  }

  const typescriptTest = normalizeRelativePath(entry.typescriptTest);
  if (!typescriptTest) {
    fail(`${label} is missing typescriptTest`);
    continue;
  }
  if (mappedTests.has(typescriptTest)) {
    fail(`${typescriptTest} has duplicate migration mappings`);
  }
  mappedTests.set(typescriptTest, entry);

  if (!typescriptTests.includes(typescriptTest)) {
    fail(`${typescriptTest} is not present in the TypeScript oracle test surface`);
  }

  if (!Array.isArray(entry.rustTests) || entry.rustTests.length === 0) {
    fail(`${typescriptTest} must list at least one Rust test file`);
  } else {
    for (const rustTest of entry.rustTests) {
      const rustPath = normalizeRelativePath(rustTest);
      if (!rustPath) {
        fail(`${typescriptTest} has an invalid Rust test path`);
        continue;
      }
      rustEvidenceFiles.add(rustPath);
      if (!existsSync(join(rustRoot, rustPath))) {
        fail(`${typescriptTest} maps to missing Rust test file ${rustPath}`);
      }
    }
  }

  if (typeof entry.coverage !== "string" || entry.coverage.trim() === "") {
    fail(`${typescriptTest} must describe migrated coverage`);
  }
}

for (const typescriptTest of typescriptTests) {
  if (!mappedTests.has(typescriptTest)) {
    fail(`missing migration mapping for ${typescriptTest}`);
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
  typescriptRoot,
  rustRoot,
  mapping: mappingPath,
  typescriptTestCount: typescriptTests.length,
  mappedTestCount: mappedTests.size,
  rustEvidenceFileCount: rustEvidenceFiles.size,
  rustEvidenceFiles: Array.from(rustEvidenceFiles).sort()
}, null, 2));

function parseArgs(args) {
  const parsed = {
    mapping: undefined,
    rustRoot: undefined,
    typescriptRoot: undefined
  };

  for (let index = 0; index < args.length; index += 1) {
    const arg = args[index];
    if (arg === "--help" || arg === "-h") {
      printUsage();
      process.exit(0);
    }
    if (arg === "--mapping") {
      parsed.mapping = resolve(requireValue(args, ++index, arg));
      continue;
    }
    if (arg === "--rust-root") {
      parsed.rustRoot = resolve(requireValue(args, ++index, arg));
      continue;
    }
    if (arg === "--typescript-root") {
      parsed.typescriptRoot = resolve(requireValue(args, ++index, arg));
      continue;
    }
    failImmediate(`Unknown option ${arg}`);
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

function readMapping(path) {
  if (!existsSync(path)) {
    failImmediate(`mapping file is missing: ${path}`);
  }

  const parsed = JSON.parse(readFileSync(path, "utf8"));
  if (!Array.isArray(parsed)) {
    failImmediate(`mapping file must contain a JSON array: ${path}`);
  }
  return parsed;
}

function listTypeScriptTests(tsRoot) {
  const testsRoot = join(tsRoot, "tests");
  if (!existsSync(testsRoot)) {
    failImmediate(`TypeScript tests directory is missing: ${testsRoot}`);
  }

  const tests = [];
  walk(testsRoot, (path) => {
    if (path.endsWith(".test.ts")) {
      tests.push(toPosix(relative(tsRoot, path)));
    }
  });
  return tests.sort();
}

function walk(dir, visitFile) {
  for (const entry of readdirSync(dir).sort()) {
    const path = join(dir, entry);
    const stat = statSync(path);
    if (stat.isDirectory()) {
      walk(path, visitFile);
      continue;
    }
    if (stat.isFile()) {
      visitFile(path);
    }
  }
}

function normalizeRelativePath(path) {
  if (typeof path !== "string" || path.trim() === "") {
    return undefined;
  }
  const normalized = toPosix(path);
  if (normalized.startsWith("/") || normalized.includes("../") || normalized === "..") {
    return undefined;
  }
  return normalized;
}

function toPosix(path) {
  return path.split(sep).join("/");
}

function printUsage() {
  console.log(
    "Usage: node scripts/audit-typescript-test-surface.mjs " +
      "[--typescript-root dir] [--rust-root dir] [--mapping file]"
  );
}

function fail(message) {
  failures.push(message);
}

function failImmediate(message) {
  console.error(`audit-typescript-test-surface: ${message}`);
  process.exit(1);
}
