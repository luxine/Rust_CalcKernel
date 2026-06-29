#!/usr/bin/env node
import { existsSync, realpathSync } from "node:fs";
import { dirname, join, resolve } from "node:path";
import { fileURLToPath } from "node:url";
import { spawnSync } from "node:child_process";
import { currentPlatformBinaryName, supportedTargetNames } from "./platform.js";

const here = dirname(realpathSync(fileURLToPath(import.meta.url)));
const root = resolve(here, "..");
const exeName = process.platform === "win32" ? "ckc.exe" : "ckc";

function candidateBinaries() {
  let packagedBinary;
  try {
    packagedBinary = join(here, "bin", currentPlatformBinaryName());
  } catch {
    packagedBinary = undefined;
  }
  return [
    process.env.CKC_BIN,
    packagedBinary,
    join(root, "target", "release", exeName),
    join(root, "target", "debug", exeName)
  ].filter(Boolean);
}

function resolveBinary() {
  for (const candidate of candidateBinaries()) {
    if (existsSync(candidate)) {
      return candidate;
    }
  }
  const searched = candidateBinaries().join("\n  ");
  throw new Error(
    `Unable to find the Rust ckc binary.\n` +
      `Set CKC_BIN to a built ckc executable or run "cargo build --release".\n` +
      `Supported packaged targets: ${supportedTargetNames().join(", ")}.\n` +
      `Searched:\n  ${searched}`
  );
}

let binary;
try {
  binary = resolveBinary();
} catch (error) {
  console.error(error instanceof Error ? error.message : String(error));
  process.exit(1);
}

const result = spawnSync(binary, process.argv.slice(2), { stdio: "inherit" });
if (result.error) {
  console.error(result.error.message);
  process.exit(1);
}
if (result.signal) {
  process.kill(process.pid, result.signal);
}
process.exit(result.status ?? 1);
