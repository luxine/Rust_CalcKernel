import { chmodSync, copyFileSync, existsSync, mkdirSync, rmSync } from "node:fs";
import { dirname, join, resolve } from "node:path";
import { fileURLToPath } from "node:url";
import {
  SUPPORTED_CKC_BINARY_TARGETS,
  binaryNameForTarget,
  currentTargetName,
  targetByName
} from "../npm/platform.js";

const root = resolve(dirname(fileURLToPath(import.meta.url)), "..");
const exeName = process.platform === "win32" ? "ckc.exe" : "ckc";
const binDir = join(root, "npm", "bin");

rmSync(binDir, { recursive: true, force: true });
mkdirSync(binDir, { recursive: true });

if (process.env.CKC_NPM_BINARIES_DIR) {
  copyAllTargetBinaries(process.env.CKC_NPM_BINARIES_DIR);
} else {
  const targetName = process.env.CKC_NPM_TARGET ?? currentTargetName();
  const target = targetByName(targetName);
  const source = process.env.CKC_NPM_BINARY ?? join(root, "target", "release", exeName);
  copyTargetBinary(target, source);
}

function copyAllTargetBinaries(sourceDir) {
  for (const target of SUPPORTED_CKC_BINARY_TARGETS) {
    const fileName = binaryNameForTarget(target.name);
    copyTargetBinary(target, join(sourceDir, fileName));
  }
}

function copyTargetBinary(target, source) {
  if (!existsSync(source)) {
    throw new Error(`Missing ${target.name} release binary at ${source}`);
  }

  const output = join(binDir, binaryNameForTarget(target.name));
  copyFileSync(source, output);
  if (target.platform !== "win32") {
    chmodSync(output, 0o755);
  }
}
