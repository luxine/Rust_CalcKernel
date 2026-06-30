#!/usr/bin/env node
import { existsSync } from "node:fs";
import { createRequire } from "node:module";
import { dirname, join, resolve } from "node:path";
import { fileURLToPath } from "node:url";

const require = createRequire(import.meta.url);
const root = resolve(dirname(fileURLToPath(import.meta.url)), "..");
const tsRoot = process.env.CALCKERNEL_TS_ROOT ?? "/Users/lynn/code/CalcKernel";
const options = parseArgs(process.argv.slice(2));
const rustDts = options.rustDts ?? join(root, "npm", "index.d.ts");
const typescriptDts = options.typescriptDts ?? join(tsRoot, "dist", "src", "index.d.ts");
const ts = loadTypescript(options.typescriptModule);
const failures = [];

const rustSurface = readDeclarationExportSurface(rustDts, "Rust declaration file");
const typescriptSurface = readDeclarationExportSurface(typescriptDts, "TypeScript oracle declaration file");
const rustExports = rustSurface.map((entry) => entry.name);
const typescriptExports = typescriptSurface.map((entry) => entry.name);
const extraRustExports = rustExports.filter((name) => !typescriptExports.includes(name));
const missingRustExports = typescriptExports.filter((name) => !rustExports.includes(name));
const rustDeclarationKinds = new Map(rustSurface.map((entry) => [entry.name, entry.kind]));
const typescriptDeclarationKinds = new Map(typescriptSurface.map((entry) => [entry.name, entry.kind]));

if (extraRustExports.length > 0) {
  fail(`extra Rust declaration exports: ${extraRustExports.join(", ")}`);
}
if (missingRustExports.length > 0) {
  fail(`missing Rust declaration exports: ${missingRustExports.join(", ")}`);
}
for (const name of rustExports.filter((exportName) => typescriptExports.includes(exportName))) {
  const rustKind = rustDeclarationKinds.get(name);
  const typescriptKind = typescriptDeclarationKinds.get(name);
  if (rustKind !== typescriptKind) {
    fail(`declaration kind mismatch for ${name}: Rust ${rustKind}, TypeScript ${typescriptKind}`);
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
  rustDts,
  typescriptDts,
  exportCount: rustExports.length,
  exports: rustExports,
  declarationKinds: Object.fromEntries(rustSurface.map((entry) => [entry.name, entry.kind]))
}, null, 2));

function parseArgs(args) {
  const parsed = {
    rustDts: undefined,
    typescriptDts: undefined,
    typescriptModule: undefined
  };

  for (let index = 0; index < args.length; index += 1) {
    const arg = args[index];
    if (arg === "--help" || arg === "-h") {
      printUsage();
      process.exit(0);
    }
    if (arg === "--rust-dts") {
      parsed.rustDts = resolve(requireValue(args, ++index, arg));
      continue;
    }
    if (arg === "--typescript-dts") {
      parsed.typescriptDts = resolve(requireValue(args, ++index, arg));
      continue;
    }
    if (arg === "--typescript-module") {
      parsed.typescriptModule = resolve(requireValue(args, ++index, arg));
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

function loadTypescript(explicitPath) {
  const candidates = [
    explicitPath,
    join(root, "node_modules", "typescript", "lib", "typescript.js"),
    join(tsRoot, "node_modules", "typescript", "lib", "typescript.js")
  ].filter(Boolean);

  for (const candidate of candidates) {
    if (existsSync(candidate)) {
      return require(candidate);
    }
  }

  try {
    return require("typescript");
  } catch {
    failImmediate(
      "Unable to locate TypeScript compiler. Install TypeScript or set --typescript-module."
    );
  }
}

function readDeclarationExportSurface(path, label) {
  if (!existsSync(path)) {
    failImmediate(`${label} does not exist: ${path}`);
  }

  const program = ts.createProgram([path], {
    allowJs: false,
    module: ts.ModuleKind.NodeNext,
    moduleResolution: ts.ModuleResolutionKind.NodeNext,
    noEmit: true,
    skipLibCheck: true,
    strict: false,
    target: ts.ScriptTarget.ES2022
  });
  const diagnostics = ts.getPreEmitDiagnostics(program);
  if (diagnostics.length > 0) {
    failImmediate(`${label} has TypeScript diagnostics:\n${formatDiagnostics(diagnostics)}`);
  }

  const sourceFile = program.getSourceFile(path);
  if (!sourceFile) {
    failImmediate(`${label} was not loaded by TypeScript: ${path}`);
  }
  const checker = program.getTypeChecker();
  const moduleSymbol = checker.getSymbolAtLocation(sourceFile) ?? sourceFile.symbol;
  if (!moduleSymbol) {
    failImmediate(`${label} is not an external module: ${path}`);
  }

  return checker
    .getExportsOfModule(moduleSymbol)
    .map((symbol) => ({
      name: symbol.getName(),
      kind: declarationExportKind(symbol, checker, label)
    }))
    .sort((left, right) => left.name.localeCompare(right.name));
}

function declarationExportKind(symbol, checker, label) {
  const resolvedSymbol = symbol.flags & ts.SymbolFlags.Alias
    ? checker.getAliasedSymbol(symbol)
    : symbol;
  const declarations = resolvedSymbol.getDeclarations() ?? [];
  if (declarations.length === 0) {
    failImmediate(`${label} export ${symbol.getName()} has no declarations`);
  }
  return declarations
    .map((declaration) => ts.SyntaxKind[declaration.kind])
    .sort()
    .join("+");
}

function formatDiagnostics(diagnostics) {
  return diagnostics
    .map((diagnostic) => {
      const message = ts.flattenDiagnosticMessageText(diagnostic.messageText, "\n");
      if (!diagnostic.file || diagnostic.start === undefined) {
        return message;
      }
      const { line, character } = diagnostic.file.getLineAndCharacterOfPosition(diagnostic.start);
      return `${diagnostic.file.fileName}:${line + 1}:${character + 1}: ${message}`;
    })
    .join("\n");
}

function printUsage() {
  console.log(
    "Usage: node scripts/verify-declaration-parity.mjs " +
      "[--rust-dts file] [--typescript-dts file] [--typescript-module file]"
  );
}

function fail(message) {
  failures.push(message);
}

function failImmediate(message) {
  console.error(`verify-declaration-parity: ${message}`);
  process.exit(1);
}
