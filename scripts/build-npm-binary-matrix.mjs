#!/usr/bin/env node
import { spawnSync } from "node:child_process";
import { createHash } from "node:crypto";
import { chmodSync, copyFileSync, existsSync, mkdirSync, readFileSync, rmSync, statSync } from "node:fs";
import { dirname, join, resolve } from "node:path";
import { fileURLToPath } from "node:url";
import {
  SUPPORTED_CKC_BINARY_TARGETS,
  binaryNameForTarget,
  supportedTargetNames,
  targetByName
} from "../npm/platform.js";

const root = resolve(dirname(fileURLToPath(import.meta.url)), "..");
const args = parseArgs(process.argv.slice(2));
const outputDir = resolve(args.out ?? join(root, "build", "npm-binaries"));
const cargoTargetDir = resolve(args.cargoTargetDir ?? join(root, "target"));
const selectedTargets = args.targets.length > 0
  ? args.targets.map((name) => targetByName(name))
  : SUPPORTED_CKC_BINARY_TARGETS;

if (args.clean) {
  rmSync(outputDir, { recursive: true, force: true });
}
mkdirSync(outputDir, { recursive: true });

const targets = selectedTargets.map((target) => {
  if (!args.skipBuild) {
    runCargoBuild(args.cargo, target);
  }

  const sourcePath = join(cargoTargetDir, target.rustTarget, "release", target.platform === "win32" ? "ckc.exe" : "ckc");
  if (!existsSync(sourcePath)) {
    fail(`Missing ${target.name} binary at ${sourcePath}`);
  }

  const binaryPath = join(outputDir, binaryNameForTarget(target.name));
  copyFileSync(sourcePath, binaryPath);
  if (target.platform !== "win32") {
    chmodSync(binaryPath, 0o755);
  }

  const binaryBytes = readFileSync(binaryPath);
  return {
    name: target.name,
    rustTarget: target.rustTarget,
    sourcePath,
    binaryPath,
    fileMode: modeString(statSync(binaryPath).mode),
    sizeBytes: binaryBytes.length,
    sha256: sha256(binaryBytes)
  };
});

if (args.expectComplete) {
  const missingTargets = missingStagedTargets(outputDir);
  if (missingTargets.length > 0) {
    fail(
      `missing staged targets in ${outputDir}: ` +
        missingTargets.map((target) => `${target.name} (${binaryNameForTarget(target.name)})`).join(", ")
    );
  }
}

console.log(JSON.stringify({ outputDir, targets }, null, 2));

function parseArgs(argv) {
  const parsed = {
    cargo: "cargo",
    cargoTargetDir: undefined,
    clean: false,
    expectComplete: false,
    out: undefined,
    skipBuild: false,
    targets: []
  };

  for (let index = 0; index < argv.length; index += 1) {
    const arg = argv[index];
    if (arg === "--help" || arg === "-h") {
      printUsage();
      process.exit(0);
    }
    if (arg === "--skip-build") {
      parsed.skipBuild = true;
      continue;
    }
    if (arg === "--clean") {
      parsed.clean = true;
      continue;
    }
    if (arg === "--expect-complete") {
      parsed.expectComplete = true;
      continue;
    }
    if (arg === "--cargo") {
      parsed.cargo = requireValue(argv, ++index, arg);
      continue;
    }
    if (arg === "--cargo-target-dir") {
      parsed.cargoTargetDir = requireValue(argv, ++index, arg);
      continue;
    }
    if (arg === "--out") {
      parsed.out = requireValue(argv, ++index, arg);
      continue;
    }
    if (arg === "--target") {
      parsed.targets.push(requireValue(argv, ++index, arg));
      continue;
    }
    fail(`Unknown argument ${arg}`);
  }

  return parsed;
}

function requireValue(argv, index, flag) {
  const value = argv[index];
  if (!value || value.startsWith("--")) {
    fail(`${flag} requires a value`);
  }
  return value;
}

function runCargoBuild(cargo, target) {
  const result = spawnSync(cargo, [
    "build",
    "--release",
    "--bin",
    "ckc",
    "--target",
    target.rustTarget
  ], {
    cwd: root,
    stdio: "inherit"
  });

  if (result.error) {
    fail(`Unable to build ${target.name}: ${result.error.message}`);
  }
  if (result.status !== 0) {
    fail(`cargo build failed for ${target.name} with status ${result.status ?? "unknown"}`);
  }
}

function modeString(mode) {
  return `0${(mode & 0o777).toString(8)}`;
}

function sha256(bytes) {
  return createHash("sha256").update(bytes).digest("hex");
}

function missingStagedTargets(stagedDir) {
  return SUPPORTED_CKC_BINARY_TARGETS.filter((target) => {
    return !existsSync(join(stagedDir, binaryNameForTarget(target.name)));
  });
}

function printUsage() {
  console.log(
    [
      "Usage: node scripts/build-npm-binary-matrix.mjs [options]",
      "",
      "Options:",
      "  --out <dir>                Directory for npm-named binaries. Default: build/npm-binaries.",
      "  --target <name>            Build only one npm target. Can be repeated.",
      "  --cargo <cmd>              Cargo executable. Default: cargo.",
      "  --cargo-target-dir <dir>   Cargo target directory. Default: target.",
      "  --clean                    Remove the output directory before staging.",
      "  --expect-complete          Fail unless the output directory contains every supported target.",
      "  --skip-build               Stage existing target binaries without running cargo build.",
      "",
      `Supported targets: ${supportedTargetNames().join(", ")}`
    ].join("\n")
  );
}

function fail(message) {
  console.error(`build-npm-binary-matrix: ${message}`);
  process.exit(1);
}
