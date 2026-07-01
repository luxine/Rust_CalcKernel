#!/usr/bin/env node
import { existsSync, readFileSync } from "node:fs";
import { basename, resolve } from "node:path";

const EXPECTED_PACKAGE_DESCRIPTION = "A small CK / CalcKernel integer-computation DSL compiler with C, WASM, and LLVM backends.";
const EXPECTED_PACKAGE_KEYWORDS = ["calckernel", "ck", "compiler", "dsl", "c", "wasm", "llvm"];
const EXPECTED_PACKAGE_REPOSITORY = {
  type: "git",
  url: "https://github.com/luxine/Rust_CalcKernel"
};
const EXPECTED_REGISTRY_REPOSITORY = {
  type: "git",
  url: "git+https://github.com/luxine/Rust_CalcKernel.git"
};
const EXPECTED_PACKAGE_LICENSE = "MIT";
const EXPECTED_PACKAGE_ENGINES = { node: ">=20" };
const EXPECTED_PACKAGE_SCRIPT_NAMES = Object.freeze([
  "audit:release-workflow",
  "audit:typescript-test-surface",
  "build",
  "build:npm-matrix",
  "ckc",
  "postpack",
  "prepack",
  "test",
  "verify:cutover-evidence",
  "verify:declaration-parity",
  "verify:host-npm-install",
  "verify:npm-release",
  "verify:public-api-parity",
  "verify:publish-artifact",
  "verify:publish-result",
  "verify:registry-replacement",
  "verify:release-signoff",
  "verify:release-signoff-summary",
  "verify:typescript-oracle"
]);
const EXPECTED_PACKAGE_METADATA = Object.freeze({
  description: EXPECTED_PACKAGE_DESCRIPTION,
  keywords: EXPECTED_PACKAGE_KEYWORDS,
  repository: EXPECTED_PACKAGE_REPOSITORY,
  license: EXPECTED_PACKAGE_LICENSE,
  engines: EXPECTED_PACKAGE_ENGINES,
  type: "module",
  main: "./npm/index.js",
  types: "./npm/index.d.ts",
  exports: {
    ".": {
      types: "./npm/index.d.ts",
      import: "./npm/index.js"
    }
  },
  bin: { ckc: "./npm/ckc.js" },
  dependencyFields: {},
  consumerInstallScripts: [],
  packageManager: null,
  scriptNames: EXPECTED_PACKAGE_SCRIPT_NAMES
});
const RELEASE_WORKFLOW = "npm release artifact";
const PUBLISH_JOB = "publish-npm";
const PUBLISH_RUNNER_OS = "Linux";
const PUBLISH_RUNNER_ARCH = "X64";
const [manifestArg, publishArg, registryArg] = process.argv.slice(2);

if (!manifestArg || !publishArg || !registryArg || manifestArg === "--help" || manifestArg === "-h") {
  console.error(
    "Usage: node scripts/verify-npm-publish-result.mjs " +
      "<release-manifest.json> <npm-publish.json> <npm-registry-replacement.json>"
  );
  process.exit(manifestArg ? 0 : 1);
}

const manifestPath = resolve(manifestArg);
const publishPath = resolve(publishArg);
const registryPath = resolve(registryArg);
const manifest = readJsonFile(manifestPath, "release manifest");
const publish = normalizePublishResult(readJsonFile(publishPath, "npm publish result"));
const registry = readJsonFile(registryPath, "npm registry replacement result");
const failures = [];

validateManifest(manifest);
const publishProvenance = collectPublishProvenance();
if (publishProvenance.ciProvider === "github-actions") {
  expectEqual(
    publishProvenance.githubSha,
    manifest.sourceGitSha,
    "publish provenance githubSha from release manifest sourceGitSha"
  );
  expectEqual(
    publishProvenance.githubRepository,
    manifest.sourceRepository,
    "publish provenance githubRepository from release manifest sourceRepository"
  );
}
expectEqual(publish.name, manifest.packageName, "publish package name");
expectEqual(publish.version, manifest.packageVersion, "publish package version");
expectEqual(publish.id, `${manifest.packageName}@${manifest.packageVersion}`, "publish id");
expectEqual(publish.filename, manifest.tarball, "publish tarball filename");
expectEqual(registry.status, "ok", "registry replacement status");
expectEqual(registry.package, manifest.packageName, "registry package name");
expectEqual(registry.packageVersion, manifest.packageVersion, "registry packageVersion");
expectEqual(registry.version, manifest.packageVersion, "registry package version");
expectRegistryTarball(registry.tarball, manifest.packageName, manifest.tarball);
expectEmptyArray(registry.consumerInstallScripts, "registry consumerInstallScripts");
expectEqual(registry.description, EXPECTED_PACKAGE_DESCRIPTION, "registry description");
expectJson(registry.keywords, EXPECTED_PACKAGE_KEYWORDS, "registry keywords");
expectPackageRepository(registry.repository, "registry repository");
expectEqual(registry.license, EXPECTED_PACKAGE_LICENSE, "registry license");
expectJson(registry.engines, EXPECTED_PACKAGE_ENGINES, "registry engines");
expectEqual(
  registry.description,
  manifest.packageMetadata?.description,
  "registry description from release manifest packageMetadata"
);
expectJson(
  registry.keywords,
  manifest.packageMetadata?.keywords,
  "registry keywords from release manifest packageMetadata"
);
expectEquivalentPackageRepository(
  registry.repository,
  manifest.packageMetadata?.repository,
  "registry repository from release manifest packageMetadata"
);
expectEqual(
  registry.license,
  manifest.packageMetadata?.license,
  "registry license from release manifest packageMetadata"
);
expectJson(
  registry.engines,
  manifest.packageMetadata?.engines,
  "registry engines from release manifest packageMetadata"
);

if (!isSha512Integrity(publish.integrity)) {
  fail(`publish integrity must be a sha512 npm integrity string, found ${JSON.stringify(publish.integrity)}`);
}
if (!isSha512Integrity(registry.integrity)) {
  fail(`registry integrity must be a sha512 npm integrity string, found ${JSON.stringify(registry.integrity)}`);
}
if (!isSha1(publish.shasum)) {
  fail(`publish shasum must be a sha1 hex string, found ${JSON.stringify(publish.shasum)}`);
}
if (!isSha1(registry.shasum)) {
  fail(`registry shasum must be a sha1 hex string, found ${JSON.stringify(registry.shasum)}`);
}
if (publish.integrity && registry.integrity && publish.integrity !== registry.integrity) {
  fail(
    `registry integrity must match npm publish integrity: ` +
      `expected ${publish.integrity}, found ${registry.integrity}`
  );
}
if (publish.shasum && registry.shasum && publish.shasum !== registry.shasum) {
  fail(
    `registry shasum must match npm publish shasum: ` +
      `expected ${publish.shasum}, found ${registry.shasum}`
  );
}

if (failures.length > 0) {
  for (const failure of failures) {
    console.error(failure);
  }
  process.exit(1);
}

console.log(JSON.stringify({
  status: "ok",
  package: manifest.packageName,
  packageVersion: manifest.packageVersion,
  version: manifest.packageVersion,
  tarball: manifest.tarball,
  sourceGitSha: manifest.sourceGitSha,
  sourceRepository: manifest.sourceRepository,
  publishPackage: publish.name,
  publishVersion: publish.version,
  publishId: publish.id,
  publishFilename: publish.filename,
  publishShasum: publish.shasum,
  publishIntegrity: publish.integrity,
  registryStatus: registry.status,
  registryTarball: registry.tarball,
  shasum: registry.shasum,
  description: registry.description,
  keywords: registry.keywords,
  repository: registry.repository,
  license: registry.license,
  engines: registry.engines,
  consumerInstallScripts: registry.consumerInstallScripts,
  integrity: registry.integrity,
  publishProvenance
}, null, 2));

function readJsonFile(path, label) {
  if (!existsSync(path)) {
    failImmediate(`${label} does not exist: ${path}`);
  }
  try {
    return JSON.parse(readFileSync(path, "utf8"));
  } catch (error) {
    failImmediate(`Unable to read ${label}: ${error.message}`);
  }
}

function normalizePublishResult(result) {
  if (Array.isArray(result)) {
    if (result.length !== 1) {
      failImmediate(`npm publish result must contain exactly one package, found ${result.length}`);
    }
    return result[0];
  }
  return result;
}

function validateManifest(manifest) {
  expectEqual(manifest.packageName, "calckernel", "release manifest packageName");
  if (!manifest.packageVersion) {
    fail("release manifest is missing packageVersion");
  }
  if (!manifest.tarball || basename(manifest.tarball) !== manifest.tarball) {
    fail(`release manifest tarball must be a tarball filename, found ${JSON.stringify(manifest.tarball)}`);
  }
  if (!isSha256(manifest.tarballSha256)) {
    fail(`release manifest tarballSha256 is invalid: ${JSON.stringify(manifest.tarballSha256)}`);
  }
  if (!isGitSha(manifest.sourceGitSha)) {
    fail(`release manifest sourceGitSha must be a 40-character lowercase hex commit SHA, found ${JSON.stringify(manifest.sourceGitSha)}`);
  }
  if (!isSourceRepository(manifest.sourceRepository)) {
    fail(
      `release manifest sourceRepository must be "local" or a GitHub owner/repository value, ` +
        `found ${JSON.stringify(manifest.sourceRepository)}`
    );
  }
  if (
    !manifest.packageMetadata
    || typeof manifest.packageMetadata !== "object"
    || Array.isArray(manifest.packageMetadata)
  ) {
    fail("release manifest packageMetadata is missing");
  } else {
    expectJson(manifest.packageMetadata, EXPECTED_PACKAGE_METADATA, "release manifest packageMetadata");
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

function expectPackageRepository(actual, label) {
  if (!isExpectedPackageRepository(actual)) {
    fail(
      `${label} must be ${JSON.stringify(EXPECTED_PACKAGE_REPOSITORY)} ` +
        `or ${JSON.stringify(EXPECTED_REGISTRY_REPOSITORY)}, found ${JSON.stringify(actual)}`
    );
  }
}

function expectEquivalentPackageRepository(actual, expected, label) {
  if (JSON.stringify(actual) === JSON.stringify(expected)) {
    return;
  }
  if (isExpectedPackageRepository(actual) && isExpectedPackageRepository(expected)) {
    return;
  }
  fail(`${label} must be equivalent to ${JSON.stringify(expected)}, found ${JSON.stringify(actual)}`);
}

function isExpectedPackageRepository(value) {
  return JSON.stringify(value) === JSON.stringify(EXPECTED_PACKAGE_REPOSITORY)
    || JSON.stringify(value) === JSON.stringify(EXPECTED_REGISTRY_REPOSITORY);
}

function expectRegistryTarball(tarball, packageName, tarballFile) {
  const expectedSuffix = `/${packageName}/-/${basename(tarballFile)}`;
  if (typeof tarball !== "string" || !tarball.endsWith(expectedSuffix)) {
    fail(`registry tarball must end with ${expectedSuffix}, found ${JSON.stringify(tarball)}`);
  }
}

function expectEmptyArray(actual, label) {
  if (!Array.isArray(actual) || actual.length !== 0) {
    fail(`${label} must be an empty array, found ${JSON.stringify(actual)}`);
  }
}

function collectPublishProvenance() {
  if (process.env.GITHUB_ACTIONS !== "true") {
    return {
      ciProvider: "local",
      githubRunId: "",
      githubRunAttempt: "",
      githubSha: "",
      githubRepository: "",
      githubWorkflow: "",
      githubJob: "",
      runnerOs: localRunnerOs(),
      runnerArch: localRunnerArch()
    };
  }

  const githubRunId = requireGithubEnv("GITHUB_RUN_ID", "githubRunId");
  const githubRunAttempt = requireGithubEnv("GITHUB_RUN_ATTEMPT", "githubRunAttempt");
  const githubSha = requireGithubEnv("GITHUB_SHA", "githubSha");
  const githubRepository = requireGithubEnv("GITHUB_REPOSITORY", "githubRepository");
  const githubWorkflow = requireGithubEnv("GITHUB_WORKFLOW", "githubWorkflow");
  const githubJob = requireGithubEnv("GITHUB_JOB", "githubJob");
  const runnerOs = requireGithubEnv("RUNNER_OS", "runnerOs");
  const runnerArch = requireGithubEnv("RUNNER_ARCH", "runnerArch");

  if (!/^\d+$/.test(githubRunId)) {
    fail("githubRunId must be a non-empty decimal string");
  }
  if (!/^\d+$/.test(githubRunAttempt)) {
    fail("githubRunAttempt must be a non-empty decimal string");
  }
  if (!/^[0-9a-f]{40}$/.test(githubSha)) {
    fail("githubSha must be a 40-character lowercase hex commit SHA");
  }
  if (!isGitHubRepository(githubRepository)) {
    fail(`githubRepository must be a GitHub owner/repository value, found ${JSON.stringify(githubRepository)}`);
  }
  if (githubWorkflow !== RELEASE_WORKFLOW) {
    fail(`githubWorkflow must be ${JSON.stringify(RELEASE_WORKFLOW)}, found ${JSON.stringify(githubWorkflow)}`);
  }
  if (githubJob !== PUBLISH_JOB) {
    fail(`githubJob must be ${JSON.stringify(PUBLISH_JOB)}, found ${JSON.stringify(githubJob)}`);
  }
  if (runnerOs !== PUBLISH_RUNNER_OS) {
    fail(`runnerOs must be ${JSON.stringify(PUBLISH_RUNNER_OS)}, found ${JSON.stringify(runnerOs)}`);
  }
  if (runnerArch !== PUBLISH_RUNNER_ARCH) {
    fail(`runnerArch must be ${JSON.stringify(PUBLISH_RUNNER_ARCH)}, found ${JSON.stringify(runnerArch)}`);
  }

  return {
    ciProvider: "github-actions",
    githubRunId,
    githubRunAttempt,
    githubSha,
    githubRepository,
    githubWorkflow,
    githubJob,
    runnerOs,
    runnerArch
  };
}

function requireGithubEnv(envName, evidenceName) {
  const value = process.env[envName];
  if (typeof value !== "string" || value.length === 0) {
    fail(`${evidenceName} is required when GITHUB_ACTIONS=true`);
  }
  return value;
}

function localRunnerOs() {
  switch (process.platform) {
    case "darwin":
      return "macOS";
    case "linux":
      return "Linux";
    case "win32":
      return "Windows";
    default:
      return process.platform;
  }
}

function localRunnerArch() {
  switch (process.arch) {
    case "arm64":
      return "ARM64";
    case "x64":
      return "X64";
    default:
      return process.arch;
  }
}

function isSha512Integrity(value) {
  return typeof value === "string" && /^sha512-[A-Za-z0-9+/]{86}==$/.test(value);
}

function isSha1(value) {
  return typeof value === "string" && /^[0-9a-f]{40}$/.test(value);
}

function isSha256(value) {
  return typeof value === "string" && /^[0-9a-f]{64}$/.test(value);
}

function isGitSha(value) {
  return typeof value === "string" && /^[0-9a-f]{40}$/.test(value);
}

function isSourceRepository(value) {
  return value === "local" || isGitHubRepository(value);
}

function isGitHubRepository(value) {
  return typeof value === "string" && /^[A-Za-z0-9_.-]+\/[A-Za-z0-9_.-]+$/.test(value);
}

function fail(message) {
  failures.push(message);
}

function failImmediate(message) {
  console.error(`verify-npm-publish-result: ${message}`);
  process.exit(1);
}
