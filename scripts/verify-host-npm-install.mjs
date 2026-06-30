#!/usr/bin/env node
import { spawnSync } from "node:child_process";
import { createHash } from "node:crypto";
import { existsSync, mkdirSync, mkdtempSync, readFileSync, rmSync, statSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { basename, dirname, join, resolve } from "node:path";
import { fileURLToPath } from "node:url";
import { currentPlatformBinaryName, currentTarget } from "../npm/platform.js";

const root = resolve(dirname(fileURLToPath(import.meta.url)), "..");
const TYPESCRIPT_COMPILER_PACKAGE = "typescript@^5.8.0";
const C_BUILD_RUNTIME_COMMAND = "ckc build smoke.ck -o build/smoke-c";
const C_RUNTIME_COMMAND = "node smoke-c-runtime.mjs";
const WASM_RUNTIME_COMMAND = "node smoke-wasm-runtime.mjs";
const LLVM_OBJECT_RUNTIME_COMMAND = "node smoke-llvm-object-runtime.mjs";
const options = parseArgs(process.argv.slice(2));
const keepTemp = options.keepTemp || process.env.CKC_KEEP_HOST_NPM_SMOKE === "1";

const tmpRoot = mkdtempSync(join(tmpdir(), "rust-calckernel-host-install-"));

try {
  const target = currentTarget();
  const ciProvenance = collectCiProvenance(target);
  const tarball = options.tarballPath ?? npmPack(tmpRoot);
  const npmVersion = readNpmVersion();
  const consumer = join(tmpRoot, "consumer");
  mkdirSync(consumer);

  run("npm", ["init", "-y"], { cwd: consumer });
  run("npm", ["install", "--ignore-scripts", tarball], { cwd: consumer });

  const installedEnv = { ...process.env };
  delete installedEnv.CKC_BIN;
  installedEnv.CKC_DISABLE_SOURCE_FALLBACK = "1";

  const installedBin = join(
    consumer,
    "node_modules",
    ".bin",
    process.platform === "win32" ? "ckc.cmd" : "ckc"
  );
  const packageRoot = join(consumer, "node_modules", "calckernel");
  const installedPackageJson = JSON.parse(readFileSync(join(packageRoot, "package.json"), "utf8"));
  if (installedPackageJson.name !== "calckernel") {
    fail(`Installed package name must be calckernel, found ${JSON.stringify(installedPackageJson.name)}`);
  }
  if (!installedPackageJson.version) {
    fail("Installed package is missing package.json version");
  }
  const packagedBinary = join(packageRoot, "npm", "bin", currentPlatformBinaryName());
  requireNonEmpty(packagedBinary);
  const packagedBinaryBytes = readFileSync(packagedBinary);

  const smokeRoot = join(consumer, "smoke");
  const buildRoot = join(smokeRoot, "build");
  mkdirSync(buildRoot, { recursive: true });
  writeFileSync(join(smokeRoot, "smoke.ck"), smokeSource());

  const commands = [
    ["--help"],
    ["check", "smoke.ck"],
    ["emit-mir", "smoke.ck", "-o", "build/smoke.mir"],
    ["emit-c", "smoke.ck", "-o", "build/smoke.c"],
    ["emit-wat", "smoke.ck", "-o", "build/smoke.wat"],
    ["emit-wasm", "smoke.ck", "-o", "build/smoke.wasm"],
    ["emit-llvm", "smoke.ck", "-o", "build/smoke.ll"]
  ];

  const completedCommands = [];
  for (const args of commands) {
    run(installedBin, args, { cwd: smokeRoot, env: installedEnv });
    completedCommands.push(["ckc", ...args].join(" "));
  }

  requireClangForRuntimeSmokes();

  const buildArgs = ["build", "smoke.ck", "-o", "build/smoke-c"];
  run(installedBin, buildArgs, { cwd: smokeRoot, env: installedEnv });
  completedCommands.push(C_BUILD_RUNTIME_COMMAND);
  requireNonEmpty(join(buildRoot, sharedLibraryName("smoke-c")));

  writeFileSync(join(smokeRoot, "smoke-c-runtime.mjs"), cRuntimeSmokeScript());
  writeFileSync(join(smokeRoot, "smoke-wasm-runtime.mjs"), wasmRuntimeSmokeScript());
  writeFileSync(join(smokeRoot, "smoke-llvm-object-runtime.mjs"), llvmObjectRuntimeSmokeScript());

  run(process.execPath, ["smoke-c-runtime.mjs"], { cwd: smokeRoot, env: installedEnv });
  completedCommands.push(C_RUNTIME_COMMAND);
  run(process.execPath, ["smoke-wasm-runtime.mjs"], { cwd: smokeRoot, env: installedEnv });
  completedCommands.push(WASM_RUNTIME_COMMAND);

  const buildLlvmArgs = ["build-llvm", "smoke.ck", "--kind", "object", "-o", "build/smoke.o"];
  run(installedBin, buildLlvmArgs, { cwd: smokeRoot, env: installedEnv });
  completedCommands.push(["ckc", ...buildLlvmArgs].join(" "));
  requireNonEmpty(join(buildRoot, "smoke.o"));
  run(process.execPath, ["smoke-llvm-object-runtime.mjs"], { cwd: smokeRoot, env: installedEnv });
  completedCommands.push(LLVM_OBJECT_RUNTIME_COMMAND);

  for (const file of ["smoke.mir", "smoke.c", "smoke.h", "smoke.wat", "smoke.wasm", "smoke.ll"]) {
    requireNonEmpty(join(buildRoot, file));
  }

  writeFileSync(join(consumer, "api-smoke.mjs"), apiSmokeScript());
  run(process.execPath, ["api-smoke.mjs"], { cwd: consumer, env: installedEnv });
  const tsc = ensureTypeScriptCompiler(consumer, installedEnv);
  const typeSmoke = runTypeSmoke(consumer, installedEnv, tsc);

  console.log(JSON.stringify({
    package: "calckernel",
    packageVersion: installedPackageJson.version,
    tarball: basename(tarball),
    tarballSha256: sha256(readFileSync(tarball)),
    targetName: target.name,
    platform: target.platform,
    arch: target.arch,
    nodeVersion: process.version,
    npmVersion,
    ...ciProvenance,
    installedBin,
    packageRoot,
    packagedBinary,
    packagedBinarySha256: sha256(packagedBinaryBytes),
    commands: completedCommands,
    apiSymbols: [
      "SourceFile",
      "TokenKind",
      "lex",
      "parse",
      "check",
      "getFunctionInfo",
      "emitCHeader",
      "emitCSource",
      "CKWasmArena",
      "createCKWasmArena"
    ],
    typeSmoke,
    ckcBinOverride: "unset",
    sourceFallback: "disabled"
  }, null, 2));
} finally {
  if (keepTemp) {
    console.error(`verify-host-npm-install: kept temp directory ${tmpRoot}`);
  } else {
    rmSync(tmpRoot, { recursive: true, force: true });
  }
}

function parseArgs(args) {
  const positional = [];
  let keepTempArg = false;

  for (const arg of args) {
    if (arg === "--help" || arg === "-h") {
      console.log("Usage: node scripts/verify-host-npm-install.mjs [--keep-temp] [calckernel-version.tgz]");
      process.exit(0);
    }
    if (arg === "--keep-temp") {
      keepTempArg = true;
      continue;
    }
    if (arg.startsWith("-")) {
      fail(`Unknown option: ${arg}`);
    }
    positional.push(arg);
  }

  if (positional.length > 1) {
    fail("Expected at most one tarball path argument");
  }

  const tarballPath = positional[0] ? resolve(positional[0]) : undefined;
  if (tarballPath && !existsSync(tarballPath)) {
    fail(`Tarball does not exist: ${tarballPath}`);
  }

  return { keepTemp: keepTempArg, tarballPath };
}

function npmPack(packDestination) {
  const output = run("npm", ["pack", "--json", "--pack-destination", packDestination], { cwd: root });
  const packs = JSON.parse(output.stdout);
  if (!Array.isArray(packs) || packs.length !== 1) {
    fail(`Expected npm pack to return one package entry, got: ${output.stdout}`);
  }
  const packed = packs[0];
  const tarball = resolve(packDestination, packed.filename);
  if (!existsSync(tarball)) {
    fail(`npm pack reported ${packed.filename}, but the tarball does not exist`);
  }
  return tarball;
}

function commandAvailable(command) {
  const output = spawnSync(command, ["--version"], { encoding: "utf8" });
  return output.status === 0;
}

function readNpmVersion() {
  return run("npm", ["--version"], { cwd: root }).stdout.trim();
}

function collectCiProvenance(target) {
  if (process.env.GITHUB_ACTIONS !== "true") {
    return {
      ciProvider: "local",
      githubRunId: "",
      githubRunAttempt: "",
      githubSha: "",
      githubWorkflow: "",
      githubJob: "",
      runnerOs: localRunnerOs(),
      runnerArch: localRunnerArch()
    };
  }

  const runnerOs = requireGithubEnv("RUNNER_OS", "runnerOs");
  const runnerArch = requireGithubEnv("RUNNER_ARCH", "runnerArch");
  const expectedRunnerOs = runnerOsForTarget(target);
  const expectedRunnerArch = runnerArchForTarget(target);

  if (runnerOs !== expectedRunnerOs) {
    fail(`${target.name} runnerOs must be ${expectedRunnerOs}, found ${JSON.stringify(runnerOs)}`);
  }
  if (runnerArch !== expectedRunnerArch) {
    fail(`${target.name} runnerArch must be ${expectedRunnerArch}, found ${JSON.stringify(runnerArch)}`);
  }

  const githubRunId = requireGithubEnv("GITHUB_RUN_ID", "githubRunId");
  const githubRunAttempt = requireGithubEnv("GITHUB_RUN_ATTEMPT", "githubRunAttempt");
  const githubSha = requireGithubEnv("GITHUB_SHA", "githubSha");
  if (!/^\d+$/.test(githubRunId)) {
    fail(`githubRunId must be a non-empty decimal string`);
  }
  if (!/^\d+$/.test(githubRunAttempt)) {
    fail(`githubRunAttempt must be a non-empty decimal string`);
  }
  if (!/^[0-9a-f]{40}$/.test(githubSha)) {
    fail(`githubSha must be a 40-character lowercase hex commit SHA`);
  }

  return {
    ciProvider: "github-actions",
    githubRunId,
    githubRunAttempt,
    githubSha,
    githubWorkflow: requireGithubEnv("GITHUB_WORKFLOW", "githubWorkflow"),
    githubJob: requireGithubEnv("GITHUB_JOB", "githubJob"),
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

function runnerOsForTarget(target) {
  switch (target.platform) {
    case "darwin":
      return "macOS";
    case "linux":
      return "Linux";
    case "win32":
      return "Windows";
    default:
      fail(`${target.name} runnerOs cannot be inferred for platform ${target.platform}`);
      return undefined;
  }
}

function runnerArchForTarget(target) {
  switch (target.arch) {
    case "arm64":
      return "ARM64";
    case "x64":
      return "X64";
    default:
      fail(`${target.name} runnerArch cannot be inferred for arch ${target.arch}`);
      return undefined;
  }
}

function requireClangForRuntimeSmokes() {
  if (!commandAvailable("clang")) {
    fail("clang is required for C and LLVM release runtime smokes");
  }
}

function runTypeSmoke(consumer, env, tsc) {
  writeFileSync(join(consumer, "tsconfig.json"), JSON.stringify({
    compilerOptions: {
      target: "ES2022",
      module: "NodeNext",
      moduleResolution: "NodeNext",
      lib: ["ES2022", "DOM"],
      strict: true,
      noEmit: true,
      skipLibCheck: false
    },
    include: ["type-smoke.mts"]
  }, null, 2));
  writeFileSync(join(consumer, "type-smoke.mts"), typeSmokeSource());
  run(tsc, ["--project", "tsconfig.json"], { cwd: consumer, env });
  return "passed";
}

function ensureTypeScriptCompiler(consumer, env) {
  const existing = resolveTsc(consumer);
  if (existing) {
    return existing;
  }

  run("npm", ["install", "--ignore-scripts", "--no-audit", "--fund=false", "--save-dev", TYPESCRIPT_COMPILER_PACKAGE], {
    cwd: consumer,
    env
  });

  const installed = consumerTscPath(consumer);
  if (!existsSync(installed)) {
    fail(`TypeScript compiler install did not create ${installed}`);
  }
  return installed;
}

function resolveTsc(consumer) {
  if (process.env.TSC_BIN && existsSync(process.env.TSC_BIN)) {
    return process.env.TSC_BIN;
  }

  const candidates = [
    consumerTscPath(consumer),
    join(root, "node_modules", ".bin", process.platform === "win32" ? "tsc.cmd" : "tsc")
  ];

  return candidates.find((candidate) => existsSync(candidate));
}

function consumerTscPath(consumer) {
  return join(consumer, "node_modules", ".bin", process.platform === "win32" ? "tsc.cmd" : "tsc");
}

function requireNonEmpty(path) {
  if (!existsSync(path)) {
    fail(`Expected output file does not exist: ${path}`);
  }
  if (statSync(path).size === 0) {
    fail(`Expected output file is empty: ${path}`);
  }
}

function sharedLibraryName(name) {
  if (process.platform === "darwin") {
    return `${name}.dylib`;
  }
  if (process.platform === "win32") {
    return `${name}.dll`;
  }
  return `${name}.so`;
}

function sha256(bytes) {
  return createHash("sha256").update(bytes).digest("hex");
}

function run(command, args, options = {}) {
  const output = spawnSync(command, args, {
    cwd: options.cwd,
    env: options.env ?? process.env,
    encoding: "utf8"
  });

  if (output.error) {
    fail(`${command} ${args.join(" ")} failed to start: ${output.error.message}`);
  }
  if (output.status !== 0) {
    fail(
      `${command} ${args.join(" ")} failed with status ${output.status}\n` +
        `stdout:\n${output.stdout}\n` +
        `stderr:\n${output.stderr}`
    );
  }
  return output;
}

function smokeSource() {
  return `struct Item {
  price: f64;
}

export fn compute(items: ptr<Item>, values: ptr<f64>) -> f64 {
  let value: f64 = items[0].price + values[0];
  if value > 0.0 {
    return -value;
  }
  return value;
}
`;
}

function cRuntimeSmokeScript() {
  return `import { spawnSync } from "node:child_process";
import { writeFileSync } from "node:fs";
import { join } from "node:path";

writeFileSync(join("build", "smoke-c-runtime.c"), ${JSON.stringify(cRuntimeHarnessSource())});
const exe = join("build", process.platform === "win32" ? "smoke-c-runtime.exe" : "smoke-c-runtime");
run("clang", [join("build", "smoke.c"), join("build", "smoke-c-runtime.c"), "-o", exe]);
run(exe, []);

function run(command, args) {
  const output = spawnSync(command, args, { encoding: "utf8" });
  if (output.error) {
    throw new Error(\`\${command} \${args.join(" ")} failed to start: \${output.error.message}\`);
  }
  if (output.status !== 0) {
    throw new Error(\`\${command} \${args.join(" ")} failed with status \${output.status}\\nstdout:\\n\${output.stdout}\\nstderr:\\n\${output.stderr}\`);
  }
}
`;
}

function wasmRuntimeSmokeScript() {
  return `import { readFileSync } from "node:fs";

const bytes = readFileSync("build/smoke.wasm");
const { instance } = await WebAssembly.instantiate(bytes, {});
const { compute, memory, __ck_heap_base } = instance.exports;
if (typeof compute !== "function") {
  throw new Error("wasm export compute is missing");
}
if (!(memory instanceof WebAssembly.Memory)) {
  throw new Error("wasm export memory is missing");
}
const heapBase = Number(__ck_heap_base?.value ?? __ck_heap_base ?? 64);
const view = new DataView(memory.buffer);
view.setFloat64(heapBase, 2.5, true);
view.setFloat64(heapBase + 8, 1.25, true);
const actual = compute(heapBase, heapBase + 8);
if (actual !== -3.75) {
  throw new Error(\`wasm runtime smoke expected -3.75, got \${actual}\`);
}
`;
}

function llvmObjectRuntimeSmokeScript() {
  return `import { spawnSync } from "node:child_process";
import { writeFileSync } from "node:fs";
import { join } from "node:path";

const object = join("build", "smoke.o");
writeFileSync(join("build", "smoke-llvm-object-runtime.c"), ${JSON.stringify(cRuntimeHarnessSource())});
const exe = join("build", process.platform === "win32" ? "smoke-llvm-object-runtime.exe" : "smoke-llvm-object-runtime");
run("clang", [object, join("build", "smoke-llvm-object-runtime.c"), "-o", exe]);
run(exe, []);

function run(command, args) {
  const output = spawnSync(command, args, { encoding: "utf8" });
  if (output.error) {
    throw new Error(\`\${command} \${args.join(" ")} failed to start: \${output.error.message}\`);
  }
  if (output.status !== 0) {
    throw new Error(\`\${command} \${args.join(" ")} failed with status \${output.status}\\nstdout:\\n\${output.stdout}\\nstderr:\\n\${output.stderr}\`);
  }
}
`;
}

function cRuntimeHarnessSource() {
  return `typedef struct {
  double price;
} Item;

double compute(Item* items, double* values);

int main(void) {
  Item items[1] = {{2.5}};
  double values[1] = {1.25};
  double actual = compute(items, values);
  return actual == -3.75 ? 0 : 1;
}
`;
}

function apiSmokeScript() {
  return `import assert from "node:assert/strict";
import {
  CKWasmArena,
  SourceFile,
  TokenKind,
  check,
  createCKWasmArena,
  emitCHeader,
  emitCSource,
  getFunctionInfo,
  lex,
  parse
} from "calckernel";

const source = new SourceFile("api.ck", ${JSON.stringify(smokeSource())});
const lexed = lex(source);
assert.equal(lexed.diagnostics.length, 0);
assert.equal(lexed.tokens[0].kind, TokenKind.Struct);
assert(lexed.tokens.some((token) => token.kind === TokenKind.F64));

const parsed = parse(source);
assert.equal(parsed.diagnostics.length, 0);

const checked = check(source);
assert.equal(checked.diagnostics.length, 0);
assert(getFunctionInfo(checked.checkedProgram, "compute"));
assert.match(emitCHeader(checked), /CK_API/);
assert.match(emitCSource(checked, { headerFileName: "api.h" }), /compute/);

assert.equal(typeof CKWasmArena, "function");
assert.equal(typeof createCKWasmArena, "function");
const arena = createCKWasmArena({
  memory: new WebAssembly.Memory({ initial: 1 }),
  __ck_heap_base: { value: 64 }
});
const copied = arena.copyInF64(new Float64Array([1.25, -2.5]));
assert.deepEqual(Array.from(copied.view), [1.25, -2.5]);
assert.deepEqual(Array.from(arena.copyOutF64(copied.ptr, 2)), [1.25, -2.5]);
`;
}

function typeSmokeSource() {
  return `import {
  CKWasmArena,
  SourceFile,
  TokenKind,
  Scope,
  SymbolTable,
  check,
  createCKWasmArena,
  emitCFiles,
  emitCHeader,
  emitCSource,
  formatDiagnostics,
  getExprType,
  getFieldInfo,
  getFunctionInfo,
  getLetType,
  getStructInfo,
  lex,
  parse,
  sharedLibraryOutputPath,
  type BuildSharedLibraryResult,
  type BuildSharedLibraryOptions,
  type CKWasmArenaCopy,
  type CKWasmArenaOptions,
  type CKWasmGlobal,
  type CKWasmInstanceLike,
  type CKWasmMemory,
  type CKHostPlatform,
  type CKSystemError,
  type CalcKernelType,
  type CheckedProgram,
  type CheckResult,
  type CommandResult,
  type CommandRunner,
  type Diagnostic,
  type DiagnosticCode,
  type EmitCFilesOptions,
  type EmitCSourceOptions,
  type FunctionParamInfo,
  type FunctionInfo,
  type FunctionSymbol,
  type LexResult,
  type LetTypeMap,
  type ParseResult,
  type PrimitiveTypeName,
  type SourcePosition,
  type SourceSpan,
  type StructFieldInfo,
  type StructInfo,
  type StructSymbol,
  type Token,
  type TypeMap,
  type TypedAst,
  type VariableSymbol,
  type AstNode,
  type AssignmentStatement,
  type BinaryExpression,
  type BlockStatement,
  type BoolLiteral,
  type CallExpression,
  type Declaration,
  type ErrorExpression,
  type ErrorStatement,
  type ErrorTypeNode,
  type Expression,
  type FieldExpression,
  type FloatLiteral,
  type FunctionDeclaration,
  type FunctionParam,
  type IdentifierExpression,
  type IdentifierNode,
  type IfStatement,
  type IndexExpression,
  type IntegerLiteral,
  type LetStatement,
  type NamedTypeNode,
  type ParenthesizedExpression,
  type PointerTypeNode,
  type PrimitiveTypeNode,
  type Program,
  type ReturnStatement,
  type Statement,
  type StructDeclaration,
  type StructField,
  type TypeNode,
  type UnaryExpression,
  type WhileStatement,
} from "calckernel";

const source = new SourceFile("typed.ck", ${JSON.stringify(smokeSource())});
const token: Token = lex(source).tokens[0];
const lexedResult: LexResult = lex(source);
const parsedResult: ParseResult = parse(source);
const tokenKind: typeof TokenKind.Struct = TokenKind.Struct;
if (token.kind !== tokenKind) {
  throw new Error("unexpected token");
}

const checked = check(source);
const checkedResult: CheckResult = checked;
const checkedProgram: CheckedProgram = checked.checkedProgram;
const diagnosticsText: string = formatDiagnostics(source, checked.diagnostics);
const functionInfo: FunctionInfo | undefined = getFunctionInfo(checked.checkedProgram, "compute");
const structInfo: StructInfo | undefined = getStructInfo(checked.checkedProgram, "Item");
const firstFieldType: CalcKernelType | undefined = getFieldInfo(checked.checkedProgram, "Item", "price")?.type;
const firstStatement = functionInfo?.declaration.body.statements[0];
if (firstStatement?.kind === "LetStatement") {
  const letType: CalcKernelType | undefined = getLetType(checked.checkedProgram, firstStatement);
  const exprType: CalcKernelType | undefined = getExprType(checked.checkedProgram, firstStatement.initializer);
  void [letType, exprType];
}

const symbols: SymbolTable = checked.symbols;
const typedAst: TypedAst = checked.typedAst;
const typeMap: TypeMap = checked.checkedProgram.types;
const letTypeMap: LetTypeMap = checked.checkedProgram.localTypes;
const scope = new Scope();
scope.declare({ name: "x", type: { kind: "primitive", name: "f64" } });
const lookedUp = scope.lookup("x");

const header: string = emitCHeader(checked);
const cSource: string = emitCSource(checked, { headerFileName: "typed.h" });
const emitFilesOptions: EmitCFilesOptions = {
  cFile: "typed.c",
  headerFile: "typed.h",
  headerFileName: "typed.h",
  writeDebug(text: string) {
    void text;
  }
};
const emitSourceOptions: EmitCSourceOptions = { headerFileName: "typed.h" };
emitCFiles(checked, emitFilesOptions);
const libraryPath: string = sharedLibraryOutputPath("build/typed", "linux");
const commandResult: CommandResult = { status: 0, stdout: "", stderr: "" };
const commandRunner: CommandRunner = () => commandResult;
const buildOptions: BuildSharedLibraryOptions = {
  ...emitFilesOptions,
  outputPath: libraryPath,
  platform: "linux",
  runCommand: commandRunner
};
const buildResult: BuildSharedLibraryResult = { ok: true, outputPath: libraryPath };

const instanceLike: CKWasmInstanceLike = {
  exports: {
    memory: new WebAssembly.Memory({ initial: 1 }),
    __ck_heap_base: { value: 64 }
  }
};
const arena: CKWasmArena = createCKWasmArena(instanceLike);
const copy: CKWasmArenaCopy<Float64Array> = arena.copyInF64(new Float64Array([1]));
const wasmOptions: CKWasmArenaOptions = { heapBase: 64 };
const wasmMemory: CKWasmMemory = arena.memory;
const wasmGlobal: CKWasmGlobal = { value: 64 };

const sourcePosition: SourcePosition = { offset: 0, line: 1, column: 1 };
const sourceSpan: SourceSpan = { start: sourcePosition, end: sourcePosition };
const diagnosticCode: DiagnosticCode = "CK0001";
const diagnostic: Diagnostic = {
  code: diagnosticCode,
  severity: "error",
  message: "x",
  fileName: "typed.ck",
  line: 1,
  column: 1,
  span: sourceSpan
};
const platform: CKHostPlatform = "linux";
const systemError: CKSystemError = new Error("x");
const primitiveName: PrimitiveTypeName = "f64";

type RootAstExports = [
  AstNode,
  IdentifierNode,
  Program,
  Declaration,
  StructDeclaration,
  StructField,
  FunctionDeclaration,
  FunctionParam,
  TypeNode,
  PrimitiveTypeNode,
  PointerTypeNode,
  NamedTypeNode,
  ErrorTypeNode,
  Statement,
  BlockStatement,
  LetStatement,
  AssignmentStatement,
  ReturnStatement,
  IfStatement,
  WhileStatement,
  ErrorStatement,
  Expression,
  IdentifierExpression,
  IntegerLiteral,
  FloatLiteral,
  BoolLiteral,
  UnaryExpression,
  BinaryExpression,
  CallExpression,
  FieldExpression,
  IndexExpression,
  ParenthesizedExpression,
  ErrorExpression,
  FunctionParamInfo,
  FunctionSymbol,
  StructFieldInfo,
  StructSymbol,
  VariableSymbol
];
const rootAstExportCount: number = 38 satisfies RootAstExports["length"];

void [
  lexedResult,
  parsedResult,
  checkedResult,
  checkedProgram,
  diagnosticsText,
  structInfo,
  firstFieldType,
  symbols,
  typedAst,
  typeMap,
  letTypeMap,
  lookedUp,
  header,
  cSource,
  emitSourceOptions,
  buildOptions,
  commandResult,
  buildResult,
  copy,
  wasmOptions,
  wasmMemory,
  wasmGlobal,
  diagnostic,
  platform,
  systemError,
  primitiveName,
  rootAstExportCount
];
`;
}

function fail(message) {
  console.error(`verify-host-npm-install: ${message}`);
  process.exit(1);
}
