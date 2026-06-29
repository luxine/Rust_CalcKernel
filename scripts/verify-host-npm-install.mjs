#!/usr/bin/env node
import { spawnSync } from "node:child_process";
import { createHash } from "node:crypto";
import { existsSync, mkdirSync, mkdtempSync, readFileSync, rmSync, statSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { basename, dirname, join, resolve } from "node:path";
import { fileURLToPath } from "node:url";
import { currentTarget } from "../npm/platform.js";

const root = resolve(dirname(fileURLToPath(import.meta.url)), "..");
const options = parseArgs(process.argv.slice(2));
const keepTemp = options.keepTemp || process.env.CKC_KEEP_HOST_NPM_SMOKE === "1";

const tmpRoot = mkdtempSync(join(tmpdir(), "rust-calckernel-host-install-"));

try {
  const tarball = options.tarballPath ?? npmPack(tmpRoot);
  const target = currentTarget();
  const consumer = join(tmpRoot, "consumer");
  mkdirSync(consumer);

  run("npm", ["init", "-y"], { cwd: consumer });
  run("npm", ["install", "--ignore-scripts", tarball], { cwd: consumer });

  const installedEnv = { ...process.env };
  delete installedEnv.CKC_BIN;

  const installedBin = join(
    consumer,
    "node_modules",
    ".bin",
    process.platform === "win32" ? "ckc.cmd" : "ckc"
  );

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

  if (commandAvailable("clang")) {
    const args = ["build-llvm", "smoke.ck", "--kind", "object", "-o", "build/smoke.o"];
    run(installedBin, args, { cwd: smokeRoot, env: installedEnv });
    completedCommands.push(["ckc", ...args].join(" "));
    requireNonEmpty(join(buildRoot, "smoke.o"));
  }

  for (const file of ["smoke.mir", "smoke.c", "smoke.h", "smoke.wat", "smoke.wasm", "smoke.ll"]) {
    requireNonEmpty(join(buildRoot, file));
  }

  writeFileSync(join(consumer, "api-smoke.mjs"), apiSmokeScript());
  run(process.execPath, ["api-smoke.mjs"], { cwd: consumer, env: installedEnv });
  const typeSmoke = runTypeSmoke(consumer, installedEnv);

  console.log(JSON.stringify({
    package: "calckernel",
    tarball: basename(tarball),
    tarballSha256: sha256(readFileSync(tarball)),
    targetName: target.name,
    platform: target.platform,
    arch: target.arch,
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
    ckcBinOverride: "unset"
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

function runTypeSmoke(consumer, env) {
  const tsc = resolveTsc();
  if (!tsc) {
    return "skipped: tsc not found";
  }

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

function resolveTsc() {
  if (process.env.TSC_BIN && existsSync(process.env.TSC_BIN)) {
    return process.env.TSC_BIN;
  }

  const candidates = [
    join(root, "node_modules", ".bin", process.platform === "win32" ? "tsc.cmd" : "tsc"),
    join("/Users/lynn/code/CalcKernel", "node_modules", ".bin", process.platform === "win32" ? "tsc.cmd" : "tsc")
  ];

  return candidates.find((candidate) => existsSync(candidate));
}

function requireNonEmpty(path) {
  if (!existsSync(path)) {
    fail(`Expected output file does not exist: ${path}`);
  }
  if (statSync(path).size === 0) {
    fail(`Expected output file is empty: ${path}`);
  }
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
