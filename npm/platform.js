export const SUPPORTED_CKC_BINARY_TARGETS = Object.freeze([
  Object.freeze({
    name: "darwin-arm64",
    platform: "darwin",
    arch: "arm64",
    rustTarget: "aarch64-apple-darwin",
    exeSuffix: ""
  }),
  Object.freeze({
    name: "darwin-x64",
    platform: "darwin",
    arch: "x64",
    rustTarget: "x86_64-apple-darwin",
    exeSuffix: ""
  }),
  Object.freeze({
    name: "linux-arm64",
    platform: "linux",
    arch: "arm64",
    rustTarget: "aarch64-unknown-linux-gnu",
    exeSuffix: ""
  }),
  Object.freeze({
    name: "linux-x64",
    platform: "linux",
    arch: "x64",
    rustTarget: "x86_64-unknown-linux-gnu",
    exeSuffix: ""
  }),
  Object.freeze({
    name: "win32-arm64",
    platform: "win32",
    arch: "arm64",
    rustTarget: "aarch64-pc-windows-msvc",
    exeSuffix: ".exe"
  }),
  Object.freeze({
    name: "win32-x64",
    platform: "win32",
    arch: "x64",
    rustTarget: "x86_64-pc-windows-msvc",
    exeSuffix: ".exe"
  })
]);

const targetsByName = new Map(SUPPORTED_CKC_BINARY_TARGETS.map((target) => [target.name, target]));
const targetsByPlatformArch = new Map(
  SUPPORTED_CKC_BINARY_TARGETS.map((target) => [`${target.platform}:${target.arch}`, target])
);

export function normalizeArch(arch) {
  return arch === "x64" ? "x64" : arch;
}

export function supportedTargetNames() {
  return SUPPORTED_CKC_BINARY_TARGETS.map((target) => target.name);
}

export function targetByName(name) {
  const target = targetsByName.get(name);
  if (!target) {
    throw new Error(`Unsupported CKC target "${name}". Supported targets: ${supportedTargetNames().join(", ")}`);
  }
  return target;
}

export function targetFromPlatformArch(platform, arch) {
  const normalizedArch = normalizeArch(arch);
  const target = targetsByPlatformArch.get(`${platform}:${normalizedArch}`);
  if (!target) {
    throw new Error(
      `Unsupported CKC platform/arch "${platform}-${normalizedArch}". ` +
        `Supported targets: ${supportedTargetNames().join(", ")}`
    );
  }
  return target;
}

export function currentTarget() {
  return targetFromPlatformArch(process.platform, process.arch);
}

export function currentTargetName() {
  return currentTarget().name;
}

export function binaryNameForTarget(name) {
  const target = targetByName(name);
  return `ckc-${target.name}${target.exeSuffix}`;
}

export function currentPlatformBinaryName() {
  return binaryNameForTarget(currentTargetName());
}
