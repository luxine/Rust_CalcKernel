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

const declarationProgram = createDeclarationProgram([rustDts, typescriptDts]);
const checker = declarationProgram.getTypeChecker();
const rustSurface = readDeclarationExportSurface(declarationProgram, checker, rustDts, "Rust declaration file");
const typescriptSurface = readDeclarationExportSurface(
  declarationProgram,
  checker,
  typescriptDts,
  "TypeScript oracle declaration file"
);
const rustExports = rustSurface.map((entry) => entry.name);
const typescriptExports = typescriptSurface.map((entry) => entry.name);
const extraRustExports = rustExports.filter((name) => !typescriptExports.includes(name));
const missingRustExports = typescriptExports.filter((name) => !rustExports.includes(name));
const rustDeclarationKinds = new Map(rustSurface.map((entry) => [entry.name, entry.kind]));
const typescriptDeclarationKinds = new Map(typescriptSurface.map((entry) => [entry.name, entry.kind]));
const rustFunctionSignatures = new Map(
  rustSurface
    .filter((entry) => entry.functionInfo)
    .map((entry) => [entry.name, entry.functionInfo])
);
const typescriptFunctionSignatures = new Map(
  typescriptSurface
    .filter((entry) => entry.functionInfo)
    .map((entry) => [entry.name, entry.functionInfo])
);
const rustClassMembers = new Map(
  rustSurface
    .filter((entry) => entry.classMemberInfo)
    .map((entry) => [entry.name, entry.classMemberInfo])
);
const typescriptClassMembers = new Map(
  typescriptSurface
    .filter((entry) => entry.classMemberInfo)
    .map((entry) => [entry.name, entry.classMemberInfo])
);

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
  const rustFunction = rustFunctionSignatures.get(name);
  const typescriptFunction = typescriptFunctionSignatures.get(name);
  if (
    rustFunction
    && typescriptFunction
    && !functionSignaturesAreCompatible(rustFunction, typescriptFunction, checker)
  ) {
    fail(
      `function signature mismatch for ${name}: ` +
      `Rust ${JSON.stringify(rustFunction.signatures)}, TypeScript ${JSON.stringify(typescriptFunction.signatures)}`
    );
  }
  const rustClass = rustClassMembers.get(name);
  const typescriptClass = typescriptClassMembers.get(name);
  if (rustClass && typescriptClass && !sameJson(rustClass.members, typescriptClass.members)) {
    fail(
      `declaration member mismatch for ${name}: ` +
        `Rust ${JSON.stringify(rustClass.members)}, TypeScript ${JSON.stringify(typescriptClass.members)}`
    );
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
  declarationKinds: Object.fromEntries(rustSurface.map((entry) => [entry.name, entry.kind])),
  functionSignatures: Object.fromEntries(
    [...rustFunctionSignatures.entries()].map(([name, info]) => [name, info.signatures])
  ),
  classMembers: Object.fromEntries(
    [...rustClassMembers.entries()].map(([name, info]) => [name, info.members])
  )
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

function createDeclarationProgram(paths) {
  const program = ts.createProgram(paths, {
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
    failImmediate(`declaration parity inputs have TypeScript diagnostics:\n${formatDiagnostics(diagnostics)}`);
  }
  return program;
}

function readDeclarationExportSurface(program, checker, path, label) {
  if (!existsSync(path)) {
    failImmediate(`${label} does not exist: ${path}`);
  }

  const sourceFile = program.getSourceFile(path);
  if (!sourceFile) {
    failImmediate(`${label} was not loaded by TypeScript: ${path}`);
  }
  const moduleSymbol = checker.getSymbolAtLocation(sourceFile) ?? sourceFile.symbol;
  if (!moduleSymbol) {
    failImmediate(`${label} is not an external module: ${path}`);
  }

  return checker
    .getExportsOfModule(moduleSymbol)
    .map((symbol) => ({
      name: symbol.getName(),
      kind: declarationExportKind(symbol, checker, label),
      functionInfo: declarationFunctionInfo(symbol, checker),
      classMemberInfo: declarationClassMemberInfo(symbol, checker)
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

function declarationFunctionInfo(symbol, checker) {
  const resolvedSymbol = resolveAliasedSymbol(symbol, checker);
  const declarations = resolvedSymbol.getDeclarations() ?? [];
  if (!declarations.some((declaration) => declaration.kind === ts.SyntaxKind.FunctionDeclaration)) {
    return null;
  }
  const signatureAnchor = declarations[0];
  const type = checker.getTypeOfSymbolAtLocation(resolvedSymbol, signatureAnchor);
  const signatures = checker
    .getSignaturesOfType(type, ts.SignatureKind.Call)
    .map((signature) => checker.signatureToString(signature, signatureAnchor, ts.TypeFormatFlags.NoTruncation))
    .sort();
  return { type, signatures };
}

function declarationClassMemberInfo(symbol, checker) {
  const resolvedSymbol = resolveAliasedSymbol(symbol, checker);
  const classDeclarations = (resolvedSymbol.getDeclarations() ?? []).filter(ts.isClassDeclaration);
  if (classDeclarations.length === 0) {
    return null;
  }
  const printer = ts.createPrinter({ removeComments: true });
  const members = classDeclarations
    .flatMap((declaration) => declaration.members.map((member) => classMemberText(member, declaration, printer)))
    .sort();
  return { members };
}

function classMemberText(member, declaration, printer) {
  return printer
    .printNode(ts.EmitHint.Unspecified, member, declaration.getSourceFile())
    .replace(/\s+/g, " ")
    .trim();
}

function functionSignaturesAreCompatible(rustFunction, typescriptFunction, checker) {
  if (sameJson(rustFunction.signatures, typescriptFunction.signatures)) {
    return true;
  }
  return checker.isTypeAssignableTo(rustFunction.type, typescriptFunction.type)
    && checker.isTypeAssignableTo(typescriptFunction.type, rustFunction.type);
}

function resolveAliasedSymbol(symbol, checker) {
  return symbol.flags & ts.SymbolFlags.Alias
    ? checker.getAliasedSymbol(symbol)
    : symbol;
}

function sameJson(left, right) {
  return JSON.stringify(left) === JSON.stringify(right);
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
