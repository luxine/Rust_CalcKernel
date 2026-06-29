#!/usr/bin/env node
import { spawnSync } from "node:child_process";
import { existsSync, readFileSync, readdirSync, statSync } from "node:fs";
import { join, resolve } from "node:path";

const tsRoot = resolve(process.env.CALCKERNEL_TS_ROOT ?? "/Users/lynn/code/CalcKernel");
const failures = [];

expectExists(tsRoot, "TypeScript oracle root");

const packageJsonPath = join(tsRoot, "package.json");
expectExists(packageJsonPath, "TypeScript oracle package.json");
const packageJson = existsSync(packageJsonPath) ? readJson(packageJsonPath) : {};
expectEqual(packageJson.name, "calckernel", "TypeScript oracle package name");
expectEqual(packageJson.main, "./dist/src/index.js", "TypeScript oracle main");
expectJson(packageJson.bin, { ckc: "./dist/src/cli.js" }, "TypeScript oracle bin");
expectEqual(packageJson.dependencies?.wabt, "^1.0.39", "TypeScript oracle wabt dependency");

const cliPath = join(tsRoot, "dist", "src", "cli.js");
expectExists(cliPath, "TypeScript oracle dist/src/cli.js");

const fixtureRoots = [
  "examples",
  "bench/perf/fixtures",
  "tests/fixtures"
];
for (const fixtureRoot of fixtureRoots) {
  expectExists(join(tsRoot, fixtureRoot), `TypeScript oracle fixture root ${fixtureRoot}`);
}

let cliStatus = "not-run";
let cliHelpFirstLine = "";
if (failures.length === 0) {
  const output = spawnSync(process.execPath, [cliPath, "--help"], {
    cwd: tsRoot,
    encoding: "utf8"
  });
  if (output.error) {
    fail(`TypeScript oracle CLI failed to start: ${output.error.message}`);
  } else if (output.status !== 0) {
    fail(
      `TypeScript oracle CLI --help failed with status ${output.status}\n` +
        `stdout:\n${output.stdout}\nstderr:\n${output.stderr}`
    );
  } else {
    const help = output.stdout;
    if (!help.includes("ckc check <file>")) {
      fail("TypeScript oracle CLI --help must include ckc check <file>");
    }
    if (!help.includes("ckc build-llvm <file>")) {
      fail("TypeScript oracle CLI --help must include ckc build-llvm <file>");
    }
    cliStatus = "ok";
    cliHelpFirstLine = help.split(/\r?\n/, 1)[0] ?? "";
  }
}

const fixtureCounts = Object.fromEntries(
  fixtureRoots.map((fixtureRoot) => [fixtureRoot, countCkFiles(join(tsRoot, fixtureRoot))])
);

if (failures.length > 0) {
  for (const failure of failures) {
    console.error(failure);
  }
  process.exit(1);
}

console.log(JSON.stringify({
  status: "ok",
  typescriptOracleRoot: tsRoot,
  cliPath,
  cliStatus,
  cliHelpFirstLine,
  package: {
    name: packageJson.name,
    main: packageJson.main,
    bin: packageJson.bin
  },
  fixtureCounts
}, null, 2));

function readJson(path) {
  try {
    return JSON.parse(readFileSync(path, "utf8"));
  } catch (error) {
    fail(`Unable to read JSON ${path}: ${error.message}`);
    return {};
  }
}

function expectExists(path, label) {
  if (!existsSync(path)) {
    fail(`${label} is missing: ${path}`);
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

function countCkFiles(dir) {
  if (!existsSync(dir)) {
    return 0;
  }

  let count = 0;
  for (const entry of readdirSync(dir)) {
    const path = join(dir, entry);
    const stats = statSync(path);
    if (stats.isDirectory()) {
      count += countCkFiles(path);
    } else if (stats.isFile() && path.endsWith(".ck")) {
      count += 1;
    }
  }
  return count;
}

function fail(message) {
  failures.push(message);
}
