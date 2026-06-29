#!/usr/bin/env node
import { existsSync } from "node:fs";
import { dirname, join, resolve } from "node:path";
import { pathToFileURL } from "node:url";
import { fileURLToPath } from "node:url";

const root = resolve(dirname(fileURLToPath(import.meta.url)), "..");
const options = parseArgs(process.argv.slice(2));
const rustIndex = options.rustIndex ?? join(root, "npm", "index.js");
const typescriptRoot = process.env.CALCKERNEL_TS_ROOT ?? "/Users/lynn/code/CalcKernel";
const typescriptIndex = options.typescriptIndex ?? join(typescriptRoot, "dist", "src", "index.js");
const failures = [];

const rustExports = await readRuntimeExports(rustIndex, "Rust package root");
const typescriptExports = await readRuntimeExports(typescriptIndex, "TypeScript oracle package root");
const extraRustExports = rustExports.filter((name) => !typescriptExports.includes(name));
const missingRustExports = typescriptExports.filter((name) => !rustExports.includes(name));

if (extraRustExports.length > 0) {
  fail(`extra Rust exports: ${extraRustExports.join(", ")}`);
}
if (missingRustExports.length > 0) {
  fail(`missing Rust exports: ${missingRustExports.join(", ")}`);
}

if (failures.length > 0) {
  for (const failure of failures) {
    console.error(failure);
  }
  process.exit(1);
}

console.log(JSON.stringify({
  status: "ok",
  rustIndex,
  typescriptIndex,
  exportCount: rustExports.length,
  exports: rustExports
}, null, 2));

function parseArgs(args) {
  const parsed = {
    rustIndex: undefined,
    typescriptIndex: undefined
  };

  for (let index = 0; index < args.length; index += 1) {
    const arg = args[index];
    if (arg === "--help" || arg === "-h") {
      printUsage();
      process.exit(0);
    }
    if (arg === "--rust-index") {
      parsed.rustIndex = resolve(requireValue(args, ++index, arg));
      continue;
    }
    if (arg === "--typescript-index") {
      parsed.typescriptIndex = resolve(requireValue(args, ++index, arg));
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

async function readRuntimeExports(path, label) {
  if (!existsSync(path)) {
    failImmediate(`${label} does not exist: ${path}`);
  }
  try {
    const module = await import(pathToFileURL(path));
    return Object.keys(module).sort();
  } catch (error) {
    failImmediate(`Unable to import ${label}: ${error.message}`);
  }
}

function printUsage() {
  console.log(
    "Usage: node scripts/verify-public-api-parity.mjs " +
      "[--rust-index file] [--typescript-index file]"
  );
}

function fail(message) {
  failures.push(message);
}

function failImmediate(message) {
  console.error(`verify-public-api-parity: ${message}`);
  process.exit(1);
}
