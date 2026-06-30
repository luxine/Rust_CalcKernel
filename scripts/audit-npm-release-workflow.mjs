#!/usr/bin/env node
import { existsSync, readFileSync } from "node:fs";
import { dirname, join, resolve } from "node:path";
import { fileURLToPath } from "node:url";
import { SUPPORTED_CKC_BINARY_TARGETS, binaryNameForTarget } from "../npm/platform.js";

const root = resolve(dirname(fileURLToPath(import.meta.url)), "..");
const workflowPath = process.env.CKC_NPM_RELEASE_WORKFLOW
  ? resolve(process.env.CKC_NPM_RELEASE_WORKFLOW)
  : join(root, ".github", "workflows", "npm-release.yml");
const failures = [];

if (!existsSync(workflowPath)) {
  fail(`npm release workflow is missing: ${workflowPath}`);
} else {
  const workflow = readFileSync(workflowPath, "utf8");
  const buildBinaryJob = workflowSection(workflow, "build-binary:", "pack-release:");
  const platformSignoffJob = workflowSection(workflow, "platform-signoff:", "finalize-signoff:");
  expectIncludes(workflow, "workflow_dispatch:", "workflow trigger");
  expectIncludes(workflow, "verify-release-scripts:", "source/package verifier job");
  expectIncludes(workflow, "build-binary:", "binary matrix job");
  expectIncludes(workflow, "pack-release:", "release packing job");
  expectIncludes(workflow, "platform-signoff:", "platform sign-off job");
  expectIncludes(workflow, "finalize-signoff:", "final sign-off job");
  expectIncludes(workflow, "publish-npm:", "npm publish job");
  expectIncludes(workflow, "publish:", "publish workflow input");
  expectIncludes(workflow, "type: boolean", "boolean publish input");
  expectIncludes(workflow, "default: false", "publish default");
  expectIncludes(workflow, "typescript_oracle_repository:", "TypeScript oracle repository input");
  expectIncludes(workflow, "default: \"luxine/CalcKernel\"", "TypeScript oracle default repository");
  expectIncludes(workflow, "typescript_oracle_ref:", "TypeScript oracle ref input");
  expectIncludes(workflow, "CALCKERNEL_TS_ROOT: ${{ github.workspace }}/typescript-oracle", "TypeScript oracle root env");
  expectIncludes(workflow, "repository: ${{ inputs.typescript_oracle_repository }}", "TypeScript oracle checkout repository");
  expectIncludes(workflow, "ref: ${{ inputs.typescript_oracle_ref }}", "TypeScript oracle checkout ref");
  expectIncludes(workflow, "path: typescript-oracle", "TypeScript oracle checkout path");
  expectIncludes(workflow, "corepack enable", "TypeScript oracle package manager setup");
  expectIncludes(workflow, "pnpm install --frozen-lockfile", "TypeScript oracle dependency install");
  expectIncludes(workflow, "pnpm build", "TypeScript oracle build");
  expectIncludes(workflow, "npm run verify:typescript-oracle", "TypeScript oracle readiness gate");
  expectIncludes(workflow, "if: ${{ inputs.publish }}", "publish job guard");
  expectIncludes(workflow, "environment: npm-production", "publish environment");
  expectIncludes(workflow, "id-token: write", "npm provenance token permission");
  expectIncludes(workflow, "registry-url: \"https://registry.npmjs.org\"", "npm registry URL");
  expectIncludes(workflow, "NODE_AUTH_TOKEN: ${{ secrets.NPM_TOKEN }}", "npm token secret");
  expectIncludes(workflow, "name: release-manifest", "publish release manifest artifact");
  expectIncludes(workflow, "path: release-manifest", "publish release manifest path");
  expectIncludes(workflow, "npm run verify:publish-artifact -- release-manifest/release-manifest.json dist", "pre-publish tarball manifest verifier command");
  expectIncludes(workflow, "npm-publish-artifact.json", "pre-publish tarball verifier artifact");
  expectIncludes(workflow, "JSON.parse(require('fs').readFileSync('release-manifest/release-manifest.json', 'utf8')).tarball", "manifest-derived publish tarball");
  expectNotIncludes(workflow, "TARBALL=\"$(ls dist/*.tgz | head -n 1)\"\n          npm publish", "publish job ls tarball selection");
  expectIncludes(workflow, "npm publish \"${TARBALL}\" --provenance --access public --json > npm-publish.json", "npm publish command");
  expectIncludes(workflow, "npm run verify:registry-replacement", "post-publish registry verifier command");
  expectIncludes(
    workflow,
    "npm run verify:registry-replacement -- \"$(node -p \"JSON.parse(require('fs').readFileSync('release-manifest/release-manifest.json', 'utf8')).packageVersion\")\" > npm-registry-replacement.json",
    "post-publish registry verifier manifest version command"
  );
  expectNotIncludes(
    workflow,
    "npm run verify:registry-replacement -- \"$(node -p \"require('./package.json').version\")\"",
    "post-publish registry verifier package.json version"
  );
  expectIncludes(workflow, "npm-registry-replacement.json", "post-publish registry verifier artifact");
  expectIncludes(workflow, "--test npm_publish_result_test", "publish result verifier test gate");
  expectIncludes(workflow, "--test npm_cutover_evidence_test", "cutover evidence verifier test gate");
  expectIncludes(workflow, "--test npm_release_signoff_summary_test", "release signoff summary verifier test gate");
  expectIncludes(
    workflow,
    "npm run verify:release-signoff-summary -- release-manifest/release-manifest.json release/release-signoff.json > release-signoff-summary.json",
    "pre-publish release signoff summary verifier command"
  );
  expectIncludes(workflow, "release-signoff-summary.json", "pre-publish release signoff summary artifact");
  expectOrder(
    workflow,
    "npm run verify:release-signoff-summary -- release-manifest/release-manifest.json release/release-signoff.json > release-signoff-summary.json",
    "npm publish \"${TARBALL}\" --provenance --access public --json > npm-publish.json",
    "release signoff summary verification before npm publish"
  );
  expectIncludes(
    workflow,
    "npm run verify:publish-result -- release-manifest/release-manifest.json npm-publish.json npm-registry-replacement.json > npm-publish-result.json",
    "post-publish result verifier command"
  );
  expectIncludes(workflow, "npm-publish-result.json", "post-publish result verifier artifact");
  expectIncludes(
    workflow,
    "npm run verify:cutover-evidence -- release-manifest/release-manifest.json release/release-signoff.json release-signoff-summary.json npm-publish-artifact.json npm-publish-result.json > npm-cutover-evidence.json",
    "final cutover evidence verifier command"
  );
  expectIncludes(workflow, "npm-cutover-evidence.json", "final cutover evidence verifier artifact");
  expectIncludes(workflow, "name: npm-publish", "npm publish artifact");
  expectIncludes(workflow, "cargo fmt --check", "format gate");
  expectIncludes(workflow, "cargo clippy --all-targets --all-features --locked -- -D warnings", "clippy gate");
  expectIncludes(workflow, "- run: cargo test\n", "full Rust test suite gate");
  expectIncludes(workflow, "--test typescript_test_surface_audit_test", "TypeScript test surface audit test gate");
  expectIncludes(workflow, "--test npm_declaration_parity_test", "declaration parity test gate");
  expectIncludes(workflow, "--test npm_public_api_parity_test", "public API parity test gate");
  expectIncludes(workflow, "--test npm_publish_artifact_test", "publish artifact verifier test gate");
  expectIncludes(workflow, "--test npm_registry_replacement_test", "registry replacement verifier test gate");
  expectIncludes(workflow, "node scripts/audit-typescript-test-surface.mjs", "TypeScript test surface audit command");
  expectIncludes(workflow, "node scripts/verify-declaration-parity.mjs", "declaration parity verifier command");
  expectIncludes(workflow, "node scripts/verify-public-api-parity.mjs", "public API parity verifier command");
  expectIncludes(workflow, "scripts/audit-rust-replacement-readiness.mjs", "readiness audit");
  expectIncludes(workflow, "scripts/audit-npm-release-workflow.mjs", "workflow self-audit");
  expectIncludes(workflow, "npm run build:npm-matrix -- --target", "matrix build command");
  expectIncludes(workflow, "npm run build:npm-matrix -- --verify-staged --expect-complete --out build/npm-binaries", "staged binary matrix verifier command");
  expectIncludes(workflow, "CKC_NPM_BINARIES_DIR=build/npm-binaries", "matrix pack environment");
  expectIncludes(workflow, "npm run verify:npm-release", "release verifier command");
  expectIncludes(workflow, "npm run verify:host-npm-install", "host install verifier command");
  expectIncludes(platformSignoffJob, "name: release-manifest", "platform signoff release manifest artifact");
  expectIncludes(platformSignoffJob, "path: release-manifest", "platform signoff release manifest path");
  expectIncludes(
    platformSignoffJob,
    "JSON.parse(require('fs').readFileSync('release-manifest/release-manifest.json', 'utf8')).tarball",
    "platform signoff manifest-derived tarball"
  );
  expectNotIncludes(
    platformSignoffJob,
    "TARBALL=\"$(ls dist/*.tgz | head -n 1)\"",
    "platform signoff ls tarball selection"
  );
  expectIncludes(workflow, "npm run verify:release-signoff", "release sign-off command");
  expectIncludes(workflow, "release-manifest.json", "release manifest artifact");
  expectIncludes(workflow, "signoffs/${{ matrix.target }}.json", "target sign-off output");
  expectIncludes(workflow, "name: signoff-${{ matrix.target }}", "target sign-off artifact");
  expectIncludes(workflow, "actions/upload-artifact@v4", "artifact upload");
  expectIncludes(workflow, "actions/download-artifact@v4", "artifact download");

  for (const target of SUPPORTED_CKC_BINARY_TARGETS) {
    expectIncludes(workflow, `target: ${target.name}`, `${target.name} matrix entry`);
    expectIncludes(workflow, `rust-target: ${target.rustTarget}`, `${target.name} rust target`);
    expectIncludes(workflow, `binary: ${binaryNameForTarget(target.name)}`, `${target.name} binary artifact`);
    expectIncludes(
      buildBinaryJob,
      targetMatrixEntry(target),
      `${target.name} build-binary target/runner binding`
    );
    expectIncludes(
      platformSignoffJob,
      targetMatrixEntry(target),
      `${target.name} platform-signoff target/runner binding`
    );
  }

  for (const runner of [
    "ubuntu-24.04",
    "ubuntu-24.04-arm",
    "macos-15",
    "macos-15-intel",
    "windows-2025",
    "windows-11-arm"
  ]) {
    expectIncludes(workflow, `runner: ${runner}`, `${runner} runner`);
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
  workflow: workflowPath,
  publishJob: true,
  targets: SUPPORTED_CKC_BINARY_TARGETS.map((target) => target.name)
}, null, 2));

function expectIncludes(text, expected, label) {
  if (!text.includes(expected)) {
    fail(`${label} must include ${expected}`);
  }
}

function expectNotIncludes(text, expected, label) {
  if (text.includes(expected)) {
    fail(`${label} must not include ${expected}`);
  }
}

function workflowSection(workflow, start, end) {
  const startIndex = workflow.indexOf(start);
  if (startIndex < 0) {
    fail(`workflow must include ${start}`);
    return "";
  }
  const endIndex = workflow.indexOf(end, startIndex);
  if (endIndex < 0) {
    fail(`workflow must include ${end} after ${start}`);
    return "";
  }
  return workflow.slice(startIndex, endIndex);
}

function expectOrder(text, before, after, label) {
  const beforeIndex = text.indexOf(before);
  if (beforeIndex < 0) {
    fail(`${label} must include ${before}`);
    return;
  }
  const afterIndex = text.indexOf(after);
  if (afterIndex < 0) {
    fail(`${label} must include ${after}`);
    return;
  }
  if (beforeIndex >= afterIndex) {
    fail(`${label} must place ${before} before ${after}`);
  }
}

function targetMatrixEntry(target) {
  return [
    `          - target: ${target.name}`,
    `            rust-target: ${target.rustTarget}`,
    `            runner: ${runnerForTarget(target)}`,
    `            binary: ${binaryNameForTarget(target.name)}`
  ].join("\n");
}

function runnerForTarget(target) {
  switch (target.name) {
    case "darwin-arm64":
      return "macos-15";
    case "darwin-x64":
      return "macos-15-intel";
    case "linux-arm64":
      return "ubuntu-24.04-arm";
    case "linux-x64":
      return "ubuntu-24.04";
    case "win32-arm64":
      return "windows-11-arm";
    case "win32-x64":
      return "windows-2025";
    default:
      fail(`${target.name} has no configured release runner`);
      return "";
  }
}

function fail(message) {
  failures.push(message);
}
