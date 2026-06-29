use std::process::Command;

#[test]
fn npm_package_surface_should_expose_ckc_bin_and_wasm_arena_helper() {
    if !node_available() || !npm_available() {
        return;
    }

    let output = Command::new("node")
        .arg("--input-type=module")
        .arg("--eval")
        .arg(package_surface_smoke_script())
        .env("CKC_BIN", env!("CARGO_BIN_EXE_ckc"))
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .output()
        .expect("run package surface smoke script");

    assert!(
        output.status.success(),
        "package smoke failed\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn npm_package_wasm_arena_should_cover_typescript_boundary_behavior() {
    if !node_available() {
        return;
    }

    let output = Command::new("node")
        .arg("--input-type=module")
        .arg("--eval")
        .arg(package_wasm_arena_boundary_script())
        .env("CKC_BIN", env!("CARGO_BIN_EXE_ckc"))
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .output()
        .expect("run WASM arena boundary script");

    assert!(
        output.status.success(),
        "WASM arena boundary smoke failed\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

fn node_available() -> bool {
    Command::new("node")
        .arg("--version")
        .output()
        .is_ok_and(|output| output.status.success())
}

fn npm_available() -> bool {
    Command::new("npm")
        .arg("--version")
        .output()
        .is_ok_and(|output| output.status.success())
}

fn package_surface_smoke_script() -> &'static str {
    r#"
import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { chmodSync, existsSync, mkdirSync, mkdtempSync, readdirSync, readFileSync, rmSync, statSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join, resolve } from "node:path";
import { pathToFileURL } from "node:url";

const root = process.cwd();
const pkg = JSON.parse(readFileSync(join(root, "package.json"), "utf8"));
const cargoToml = readFileSync(join(root, "Cargo.toml"), "utf8");

assert.equal(pkg.name, "calckernel");
assert.equal(pkg.version, "0.8.0");
assert.match(cargoToml, /\nversion = "0\.8\.0"\n/);
assert.equal(pkg.type, "module");
assert.deepEqual(Object.keys(pkg.bin), ["ckc"]);
assert.equal(pkg.bin.ckc, "./npm/ckc.js");
assert.equal(pkg.exports["."].import, "./npm/index.js");
assert.equal(pkg.exports["."].types, "./npm/index.d.ts");
assert.equal(pkg.scripts["build:npm-matrix"], "node scripts/build-npm-binary-matrix.mjs");
assert(!Object.keys(pkg.bin).some((name) => name !== "ckc"));

const publicApi = await import(pathToFileURL(join(root, "npm/index.js")));
assert.equal(typeof publicApi.SourceFile, "function");
assert.equal(typeof publicApi.lex, "function");
assert.equal(typeof publicApi.parse, "function");
assert.equal(typeof publicApi.check, "function");
assert.equal(typeof publicApi.getExprType, "function");
assert.equal(typeof publicApi.getLetType, "function");
assert.equal(typeof publicApi.getStructInfo, "function");
assert.equal(typeof publicApi.getFieldInfo, "function");
assert.equal(typeof publicApi.getFunctionInfo, "function");
assert.equal(typeof publicApi.Scope, "function");
assert.equal(typeof publicApi.SymbolTable, "function");
assert.equal(typeof publicApi.emitCHeader, "function");
assert.equal(typeof publicApi.emitCSource, "function");
assert.equal(typeof publicApi.emitCFiles, "function");
assert.equal(typeof publicApi.buildSharedLibrary, "function");
assert.equal(typeof publicApi.sharedLibraryOutputPath, "function");
assert.equal(typeof publicApi.formatDiagnostic, "function");
assert.equal(typeof publicApi.formatDiagnostics, "function");
assert.equal(typeof publicApi.TokenKind, "object");
assert.equal(typeof publicApi.CKWasmArena, "function");
assert.equal(typeof publicApi.createCKWasmArena, "function");
assert.equal(publicApi.TokenKind.Eof, "Eof");
assert.equal(publicApi.TokenKind.Identifier, "Identifier");
assert.equal(publicApi.TokenKind.F64, "F64");

const diagnosticSource = new publicApi.SourceFile("diag.ck", "return @;\n");
assert.equal(diagnosticSource.fileName, "diag.ck");
assert.equal(diagnosticSource.text, "return @;\n");
const diagnostic = {
  code: "CK0001",
  severity: "error",
  message: "Unexpected character '@'.",
  fileName: "diag.ck",
  line: 1,
  column: 8,
  span: {
    start: { offset: 7, line: 1, column: 8 },
    end: { offset: 8, line: 1, column: 9 }
  }
};
assert.equal(
  publicApi.formatDiagnostic(diagnosticSource, diagnostic),
  "diag.ck:1:8: error CK0001: Unexpected character '@'.\nreturn @;\n       ^\n"
);
assert.equal(publicApi.formatDiagnostics(diagnosticSource, [diagnostic]), publicApi.formatDiagnostic(diagnosticSource, diagnostic));

const tsIndexPath = "/Users/lynn/code/CalcKernel/dist/src/index.js";
assert(existsSync(tsIndexPath), `${tsIndexPath} is required for package API oracle coverage`);
const tsApi = await import(pathToFileURL(tsIndexPath));
for (const [fileName, text] of [
  ["surface.ck", "export fn add(a: i32, b: i32) -> i32 { return a + b; }\n"],
  ["bad_float.ck", "let x: f64 = 1e+;"],
  ["unicode.ck", "🙂 let x: i32 = 1;"]
]) {
  const actual = normalizeLexResult(publicApi.lex(new publicApi.SourceFile(fileName, text)));
  const expected = normalizeLexResult(tsApi.lex(new tsApi.SourceFile(fileName, text)));
  assert.deepEqual(actual, expected, `lex mismatch for ${fileName}`);
}
for (const [fileName, text] of [
  [
    "parse_surface.ck",
    `
      struct Item {
        price: i64;
        qty: i32;
      }

      export fn calc(items: ptr<Item>, len: i32) -> i64 {
        let i: i32 = 0;
        while i < len {
          i = i + 1;
        }
        return items[0].price;
      }
    `
  ],
  ["parse_error.ck", "export fn bad() -> i32 { return ; }\n"]
]) {
  const actual = publicApi.parse(new publicApi.SourceFile(fileName, text));
  const expected = tsApi.parse(new tsApi.SourceFile(fileName, text));
  assert.deepEqual(actual, expected, `parse mismatch for ${fileName}`);
}
for (const [fileName, text] of [
  [
    "check_surface.ck",
    `
      struct Item {
        price: i64;
        qty: i32;
      }

      export fn add(a: i64, b: i64) -> i64 {
        return a + b;
      }

      export fn calc(item: ptr<Item>) -> i64 {
        let subtotal: i64 = item[0].price + add(1, 2);
        return subtotal;
      }
    `
  ],
  [
    "check_errors.ck",
    `
      struct Item { price: i64; }
      struct Item { qty: i64; }

      export fn f(a: i32, a: i32) -> i32 {
        let x: Missing = 0;
        let x: i32 = y;
        return x;
      }

      export fn f() -> i32 {
        return 0;
      }
    `
  ]
]) {
  const actual = normalizeCheckResult(publicApi, publicApi.check(new publicApi.SourceFile(fileName, text)));
  const expected = normalizeCheckResult(tsApi, tsApi.check(new tsApi.SourceFile(fileName, text)));
  assert.deepEqual(actual, expected, `check mismatch for ${fileName}`);
}

const helperChecked = publicApi.check(new publicApi.SourceFile(
  "helper.ck",
  "struct Item { price: i64; } export fn calc(item: ptr<Item>) -> i64 { let subtotal: i64 = item[0].price; return subtotal; }"
)).checkedProgram;
const helperFunction = publicApi.getFunctionInfo(helperChecked, "calc");
const helperLet = helperFunction.declaration.body.statements[0];
const helperInitializer = helperLet.initializer;
assert.deepEqual(publicApi.getStructInfo(helperChecked, "Item").fields.map((field) => field.name), ["price"]);
assert.deepEqual(publicApi.getFieldInfo(helperChecked, "Item", "price").type, { kind: "primitive", name: "i64" });
assert.deepEqual(helperFunction.returnType, { kind: "primitive", name: "i64" });
assert.deepEqual(publicApi.getLetType(helperChecked, helperLet), { kind: "primitive", name: "i64" });
assert.deepEqual(publicApi.getExprType(helperChecked, helperInitializer), { kind: "primitive", name: "i64" });

const cBackendSource = `
  struct Item {
    price: i64;
    qty: i64;
  }

  export fn add(a: i64, b: i64) -> i64 {
    return a + b;
  }

  export fn calc(item: ptr<Item>) -> i64 {
    return item[0].price + add(1, 2);
  }
`;
const publicCChecked = publicApi.check(new publicApi.SourceFile("c_backend.ck", cBackendSource));
const tsCChecked = tsApi.check(new tsApi.SourceFile("c_backend.ck", cBackendSource));
assert.equal(publicApi.emitCHeader(publicCChecked), tsApi.emitCHeader(tsCChecked));
assert.equal(
  publicApi.emitCHeader(publicCChecked, { overflowMode: "checked" }),
  tsApi.emitCHeader(tsCChecked, { overflowMode: "checked" })
);
assert.equal(
  publicApi.emitCSource(publicCChecked, { headerFileName: "calc.h" }),
  tsApi.emitCSource(tsCChecked, { headerFileName: "calc.h" })
);
assert.equal(
  publicApi.emitCSource(publicCChecked, { headerFileName: "calc_checked.h", overflowMode: "checked", optLevel: 0 }),
  tsApi.emitCSource(tsCChecked, { headerFileName: "calc_checked.h", overflowMode: "checked", optLevel: 0 })
);
assert.throws(
  () => publicApi.emitCHeader(publicApi.check(new publicApi.SourceFile("bad_c.ck", "export fn bad() -> i32 { return missing; }"))),
  /Cannot emit C for a program with diagnostics\./
);

const cApiRoot = mkdtempSync(join(tmpdir(), "rust-calckernel-c-api-"));
try {
  const cFile = join(cApiRoot, "out", "calc.c");
  const headerFile = join(cApiRoot, "out", "calc.h");
  publicApi.emitCFiles(publicCChecked, {
    cFile,
    headerFile,
    headerFileName: "calc.h"
  });
  assert.equal(readFileSync(headerFile, "utf8"), tsApi.emitCHeader(tsCChecked));
  assert.equal(readFileSync(cFile, "utf8"), tsApi.emitCSource(tsCChecked, { headerFileName: "calc.h" }));

  assert.equal(publicApi.sharedLibraryOutputPath("build/calc", "darwin"), "build/calc.dylib");
  assert.equal(publicApi.sharedLibraryOutputPath("build/calc", "win32"), "build/calc.dll");
  assert.equal(publicApi.sharedLibraryOutputPath("build/calc", "linux"), "build/calc.so");
  assert.equal(publicApi.sharedLibraryOutputPath("build/calc.dylib", "linux"), "build/calc.dylib");

  const buildResult = publicApi.buildSharedLibrary(publicCChecked, {
    cFile: join(cApiRoot, "build", "calc.c"),
    headerFile: join(cApiRoot, "build", "calc.h"),
    headerFileName: "calc.h",
    outputPath: join(cApiRoot, "build", "calc"),
    platform: "linux",
    runCommand: () => ({ status: null, stdout: "", stderr: "", error: { code: "ENOENT" } })
  });
  assert.deepEqual(buildResult, {
    ok: false,
    outputPath: join(cApiRoot, "build", "calc.so"),
    message: "clang was not found. Install clang and make sure it is available on PATH."
  });
  assert.equal(readFileSync(join(cApiRoot, "build", "calc.h"), "utf8"), tsApi.emitCHeader(tsCChecked));
  assert.equal(readFileSync(join(cApiRoot, "build", "calc.c"), "utf8"), tsApi.emitCSource(tsCChecked, { headerFileName: "calc.h" }));
} finally {
  rmSync(cApiRoot, { recursive: true, force: true });
}

const platformApi = await import(pathToFileURL(join(root, "npm/platform.js")));
assert.deepEqual(
  platformApi.SUPPORTED_CKC_BINARY_TARGETS.map((target) => target.name),
  ["darwin-arm64", "darwin-x64", "linux-arm64", "linux-x64", "win32-arm64", "win32-x64"]
);
assert.equal(platformApi.binaryNameForTarget("darwin-arm64"), "ckc-darwin-arm64");
assert.equal(platformApi.binaryNameForTarget("linux-x64"), "ckc-linux-x64");
assert.equal(platformApi.binaryNameForTarget("win32-x64"), "ckc-win32-x64.exe");
assert.equal(platformApi.targetFromPlatformArch("darwin", "arm64").rustTarget, "aarch64-apple-darwin");
assert.equal(platformApi.targetFromPlatformArch("linux", "x64").rustTarget, "x86_64-unknown-linux-gnu");
assert.equal(platformApi.targetFromPlatformArch("win32", "arm64").rustTarget, "aarch64-pc-windows-msvc");
assert.equal(platformApi.currentPlatformBinaryName(), platformApi.binaryNameForTarget(platformApi.currentTargetName()));

function normalizeLexResult(result) {
  return {
    tokens: result.tokens.map((token) => ({
      kind: token.kind,
      text: token.text,
      line: token.line,
      column: token.column,
      start: token.start,
      end: token.end
    })),
    diagnostics: result.diagnostics.map((diagnostic) => ({
      code: diagnostic.code,
      severity: diagnostic.severity,
      message: diagnostic.message,
      fileName: diagnostic.fileName,
      line: diagnostic.line,
      column: diagnostic.column,
      span: diagnostic.span
    }))
  };
}

function normalizeCheckResult(api, result) {
  const checkedProgram = result.checkedProgram;
  return {
    ast: result.ast,
    typedAst: {
      program: result.typedAst.program,
      expressionTypes: normalizeTypeMap(result.typedAst.expressionTypes)
    },
    checkedProgram: {
      ast: checkedProgram.ast,
      symbols: normalizeSymbols(checkedProgram.symbols),
      types: normalizeTypeMap(checkedProgram.types),
      localTypes: normalizeLetTypeMap(checkedProgram.localTypes),
      structs: checkedProgram.structs.map((struct) => ({
        name: struct.name,
        declaration: struct.declaration,
        fields: struct.fields.map((field) => ({
          name: field.name,
          type: field.type,
          declaration: field.declaration
        })),
        fieldMap: [...struct.fieldMap.entries()].map(([name, field]) => ({
          name,
          field: { name: field.name, type: field.type, declaration: field.declaration }
        }))
      })),
      functions: checkedProgram.functions.map((func) => ({
        name: func.name,
        exported: func.exported,
        declaration: func.declaration,
        params: func.params.map((param) => ({
          name: param.name,
          type: param.type,
          declaration: param.declaration
        })),
        returnType: func.returnType
      })),
      structMap: [...checkedProgram.structMap.keys()],
      functionMap: [...checkedProgram.functionMap.keys()],
      helpers: {
        item: api.getStructInfo(checkedProgram, "Item")?.fields.map((field) => [field.name, field.type]),
        price: api.getFieldInfo(checkedProgram, "Item", "price")?.type,
        add: api.getFunctionInfo(checkedProgram, "add")?.returnType,
        calc: api.getFunctionInfo(checkedProgram, "calc")?.returnType
      }
    },
    diagnostics: result.diagnostics,
    symbols: normalizeSymbols(result.symbols)
  };
}

function normalizeTypeMap(map) {
  return [...map.entries()].map(([expression, type]) => ({
    expression: {
      kind: expression.kind,
      span: expression.span,
      name: expression.name,
      text: expression.text,
      value: expression.value,
      operator: expression.operator
    },
    type
  }));
}

function normalizeLetTypeMap(map) {
  return [...map.entries()].map(([statement, type]) => ({
    statement: {
      kind: statement.kind,
      name: statement.name?.name,
      span: statement.span
    },
    type
  }));
}

function normalizeSymbols(symbols) {
  return {
    structs: [...symbols.structs.entries()].map(([name, symbol]) => ({
      name,
      declaration: symbol.declaration,
      fields: [...symbol.fields.entries()]
    })),
    functions: [...symbols.functions.entries()].map(([name, symbol]) => ({
      name,
      declaration: symbol.declaration,
      params: symbol.params,
      returnType: symbol.returnType
    }))
  };
}

const cargoTargetRoot = mkdtempSync(join(tmpdir(), "rust-calckernel-cargo-targets-"));
const stagedBinaryRoot = mkdtempSync(join(tmpdir(), "rust-calckernel-binaries-"));
try {
  const expectedTargets = platformApi.SUPPORTED_CKC_BINARY_TARGETS;
  const expectedBinaryFiles = expectedTargets.map((target) =>
    platformApi.binaryNameForTarget(target.name)
  ).sort();
  for (const target of expectedTargets) {
    const releaseDir = join(cargoTargetRoot, target.rustTarget, "release");
    mkdirSync(releaseDir, { recursive: true });
    const binaryPath = join(releaseDir, target.platform === "win32" ? "ckc.exe" : "ckc");
    writeFileSync(binaryPath, binaryStubForTarget(target));
    if (target.platform !== "win32") chmodSync(binaryPath, 0o755);
  }
  const incrementalX64 = spawnSync(process.execPath, [
    "scripts/build-npm-binary-matrix.mjs",
    "--skip-build",
    "--target",
    "linux-x64",
    "--out",
    stagedBinaryRoot,
    "--cargo-target-dir",
    cargoTargetRoot
  ], {
    cwd: root,
    encoding: "utf8"
  });
  assert.equal(incrementalX64.status, 0, incrementalX64.stderr || incrementalX64.stdout);
  const incrementalArm64 = spawnSync(process.execPath, [
    "scripts/build-npm-binary-matrix.mjs",
    "--skip-build",
    "--target",
    "linux-arm64",
    "--out",
    stagedBinaryRoot,
    "--cargo-target-dir",
    cargoTargetRoot
  ], {
    cwd: root,
    encoding: "utf8"
  });
  assert.equal(incrementalArm64.status, 0, incrementalArm64.stderr || incrementalArm64.stdout);
  assert(existsSync(join(stagedBinaryRoot, "ckc-linux-x64")), "incremental staging must preserve prior target");
  assert(existsSync(join(stagedBinaryRoot, "ckc-linux-arm64")), "incremental staging must add new target");

  const incompleteMatrix = spawnSync(process.execPath, [
    "scripts/build-npm-binary-matrix.mjs",
    "--skip-build",
    "--expect-complete",
    "--target",
    "linux-x64",
    "--out",
    stagedBinaryRoot,
    "--cargo-target-dir",
    cargoTargetRoot
  ], {
    cwd: root,
    encoding: "utf8"
  });
  assert.notEqual(incompleteMatrix.status, 0, incompleteMatrix.stdout);
  assert.match(incompleteMatrix.stderr, /missing staged targets/);
  for (const targetName of platformApi.supportedTargetNames().filter((name) => !["linux-x64", "linux-arm64"].includes(name))) {
    assert.match(incompleteMatrix.stderr, new RegExp(targetName));
    assert.match(incompleteMatrix.stderr, new RegExp(platformApi.binaryNameForTarget(targetName).replace(".", "\\.")));
  }

  const stageMatrix = spawnSync(process.execPath, [
    "scripts/build-npm-binary-matrix.mjs",
    "--clean",
    "--skip-build",
    "--expect-complete",
    "--out",
    stagedBinaryRoot,
    "--cargo-target-dir",
    cargoTargetRoot
  ], {
    cwd: root,
    encoding: "utf8"
  });
  assert.equal(stageMatrix.status, 0, stageMatrix.stderr || stageMatrix.stdout);
  const stageManifest = JSON.parse(stageMatrix.stdout);
  assert.equal(stageManifest.outputDir, stagedBinaryRoot);
  assert.deepEqual(stageManifest.targets.map((target) => target.name).sort(), platformApi.supportedTargetNames().sort());
  for (const target of expectedTargets) {
    const stagedPath = join(stagedBinaryRoot, platformApi.binaryNameForTarget(target.name));
    assert.deepEqual(readFileSync(stagedPath), binaryStubForTarget(target));
    assert.equal((statSync(stagedPath).mode & 0o100) !== 0, target.platform !== "win32");
    const stagedTarget = stageManifest.targets.find((entry) => entry.name === target.name);
    assert(stagedTarget, `${target.name} missing from staging manifest`);
    assert.equal(stagedTarget.binaryPath, stagedPath);
    assert.equal(stagedTarget.rustTarget, target.rustTarget);
    assert.equal(stagedTarget.sizeBytes, binaryStubForTarget(target).length);
    assert.match(stagedTarget.sha256, /^[0-9a-f]{64}$/);
  }

  const prepareMatrix = spawnSync(process.execPath, ["scripts/prepare-npm-package.mjs"], {
    cwd: root,
    env: { ...process.env, CKC_NPM_BINARIES_DIR: stagedBinaryRoot },
    encoding: "utf8"
  });
  assert.equal(prepareMatrix.status, 0, prepareMatrix.stderr || prepareMatrix.stdout);
  assert.deepEqual(readdirSync(join(root, "npm", "bin")).sort(), expectedBinaryFiles);

  const matrixPack = spawnSync("npm", ["pack", "--dry-run", "--json", "--ignore-scripts"], {
    cwd: root,
    encoding: "utf8"
  });
  assert.equal(matrixPack.status, 0, matrixPack.stderr || matrixPack.stdout);
  const matrixFiles = JSON.parse(matrixPack.stdout)[0].files.map((file) => file.path);
  for (const fileName of expectedBinaryFiles) {
    assert(matrixFiles.includes(`npm/bin/${fileName}`), `${fileName} missing from matrix pack`);
  }

  const matrixPackRoot = mkdtempSync(join(tmpdir(), "rust-calckernel-matrix-pack-"));
  try {
    const matrixReleasePack = spawnSync("npm", ["pack", "--json", "--pack-destination", matrixPackRoot, "--ignore-scripts"], {
      cwd: root,
      encoding: "utf8"
    });
    assert.equal(matrixReleasePack.status, 0, matrixReleasePack.stderr || matrixReleasePack.stdout);
    const matrixRelease = JSON.parse(matrixReleasePack.stdout)[0];
    const matrixTarball = resolve(matrixPackRoot, matrixRelease.filename);

    const verifyRelease = spawnSync(process.execPath, ["scripts/verify-npm-release.mjs", matrixTarball], {
      cwd: root,
      encoding: "utf8"
    });
    assert.equal(verifyRelease.status, 0, verifyRelease.stderr || verifyRelease.stdout);
    const manifest = JSON.parse(verifyRelease.stdout);
    assert.equal(manifest.packageName, "calckernel");
    assert.equal(manifest.packageVersion, "0.8.0");
    assert.match(manifest.tarballSha256, /^[0-9a-f]{64}$/);
    assert.deepEqual(manifest.packageMetadata, {
      type: "module",
      main: "./npm/index.js",
      types: "./npm/index.d.ts",
      exports: {
        ".": {
          types: "./npm/index.d.ts",
          import: "./npm/index.js"
        }
      },
      bin: {
        ckc: "./npm/ckc.js"
      },
      dependencyFields: {}
    });
    assert.deepEqual(manifest.fileSurface.packageJsonFiles, [
      "npm",
      "README.md",
      "docs/npm-release.md",
      "docs/architecture-review.md",
      "docs/zh-CN/architecture-review.md"
    ]);
    assert.deepEqual(manifest.fileSurface.requiredFiles, [
      "package/package.json",
      "package/npm/ckc.js",
      "package/npm/platform.js",
      "package/npm/index.js",
      "package/npm/index.d.ts",
      "package/docs/npm-release.md",
      "package/docs/architecture-review.md",
      "package/docs/zh-CN/architecture-review.md",
      "package/README.md"
    ]);
    assert.deepEqual(manifest.fileSurface.forbiddenPrefixes, [
      "package/docs/superpowers/",
      "package/src/",
      "package/target/"
    ]);
    assert.deepEqual(
      manifest.fileSurface.allowedEntries.toSorted(),
      [
        ...manifest.fileSurface.requiredFiles,
        ...expectedBinaryFiles.map((fileName) => `package/npm/bin/${fileName}`)
      ].toSorted()
    );
    assert.deepEqual(manifest.targets.map((target) => target.name).sort(), platformApi.supportedTargetNames().sort());
    for (const target of manifest.targets) {
      const expectedTarget = platformApi.targetByName(target.name);
      assert.match(target.sha256, /^[0-9a-f]{64}$/);
      assert.equal(target.binaryPath, `package/npm/bin/${platformApi.binaryNameForTarget(target.name)}`);
      assert.equal(target.sizeBytes, binaryStubForTarget(expectedTarget).length);
      assert.equal(target.binaryFormat, expectedBinaryFormat(expectedTarget));
      assert.equal(target.binaryArchitecture, expectedTarget.arch);
      assert.equal(typeof target.fileMode, "string");
      assert.equal(target.fileMode[3], expectedTarget.platform === "win32" ? "-" : "x");
    }

    const mutatedBinaryRoot = mkdtempSync(join(tmpdir(), "rust-calckernel-mutated-binary-pack-"));
    try {
      const unpack = spawnSync("tar", ["-xzf", matrixTarball, "-C", mutatedBinaryRoot], {
        cwd: root,
        encoding: "utf8"
      });
      assert.equal(unpack.status, 0, unpack.stderr || unpack.stdout);
      writeFileSync(
        join(mutatedBinaryRoot, "package", "npm", "bin", "ckc-linux-x64"),
        "not a linux executable\n"
      );
      const mutatedTarball = join(mutatedBinaryRoot, "calckernel-mutated-binary.tgz");
      const repack = spawnSync("tar", ["-czf", mutatedTarball, "-C", mutatedBinaryRoot, "package"], {
        cwd: root,
        encoding: "utf8"
      });
      assert.equal(repack.status, 0, repack.stderr || repack.stdout);
      const verifyMutated = spawnSync(process.execPath, ["scripts/verify-npm-release.mjs", mutatedTarball], {
        cwd: root,
        encoding: "utf8"
      });
      assert.notEqual(verifyMutated.status, 0, verifyMutated.stdout);
      assert.match(verifyMutated.stderr, /ckc-linux-x64.*expected ELF executable/);
    } finally {
      rmSync(mutatedBinaryRoot, { recursive: true, force: true });
    }

    const mutatedArchRoot = mkdtempSync(join(tmpdir(), "rust-calckernel-mutated-arch-pack-"));
    try {
      const unpack = spawnSync("tar", ["-xzf", matrixTarball, "-C", mutatedArchRoot], {
        cwd: root,
        encoding: "utf8"
      });
      assert.equal(unpack.status, 0, unpack.stderr || unpack.stdout);
      writeFileSync(
        join(mutatedArchRoot, "package", "npm", "bin", "ckc-linux-x64"),
        binaryStubForTarget(platformApi.targetByName("linux-arm64"))
      );
      const mutatedTarball = join(mutatedArchRoot, "calckernel-mutated-arch.tgz");
      const repack = spawnSync("tar", ["-czf", mutatedTarball, "-C", mutatedArchRoot, "package"], {
        cwd: root,
        encoding: "utf8"
      });
      assert.equal(repack.status, 0, repack.stderr || repack.stdout);
      const verifyMutated = spawnSync(process.execPath, ["scripts/verify-npm-release.mjs", mutatedTarball], {
        cwd: root,
        encoding: "utf8"
      });
      assert.notEqual(verifyMutated.status, 0, verifyMutated.stdout);
      assert.match(verifyMutated.stderr, /ckc-linux-x64.*expected x64.*found arm64/);
    } finally {
      rmSync(mutatedArchRoot, { recursive: true, force: true });
    }

    const mutatedModeRoot = mkdtempSync(join(tmpdir(), "rust-calckernel-mutated-mode-pack-"));
    try {
      const unpack = spawnSync("tar", ["-xzf", matrixTarball, "-C", mutatedModeRoot], {
        cwd: root,
        encoding: "utf8"
      });
      assert.equal(unpack.status, 0, unpack.stderr || unpack.stdout);
      chmodSync(join(mutatedModeRoot, "package", "npm", "bin", "ckc-linux-x64"), 0o644);
      const mutatedTarball = join(mutatedModeRoot, "calckernel-mutated-mode.tgz");
      const repack = spawnSync("tar", ["-czf", mutatedTarball, "-C", mutatedModeRoot, "package"], {
        cwd: root,
        encoding: "utf8"
      });
      assert.equal(repack.status, 0, repack.stderr || repack.stdout);
      const verifyMutated = spawnSync(process.execPath, ["scripts/verify-npm-release.mjs", mutatedTarball], {
        cwd: root,
        encoding: "utf8"
      });
      assert.notEqual(verifyMutated.status, 0, verifyMutated.stdout);
      assert.match(verifyMutated.stderr, /ckc-linux-x64.*expected executable mode/);
    } finally {
      rmSync(mutatedModeRoot, { recursive: true, force: true });
    }

    const mutatedRoot = mkdtempSync(join(tmpdir(), "rust-calckernel-mutated-pack-"));
    try {
      const unpack = spawnSync("tar", ["-xzf", matrixTarball, "-C", mutatedRoot], {
        cwd: root,
        encoding: "utf8"
      });
      assert.equal(unpack.status, 0, unpack.stderr || unpack.stdout);
      writeFileSync(join(mutatedRoot, "package", "unexpected.txt"), "unexpected release file\n");
      const mutatedTarball = join(mutatedRoot, "calckernel-mutated.tgz");
      const repack = spawnSync("tar", ["-czf", mutatedTarball, "-C", mutatedRoot, "package"], {
        cwd: root,
        encoding: "utf8"
      });
      assert.equal(repack.status, 0, repack.stderr || repack.stdout);
      const verifyMutated = spawnSync(process.execPath, ["scripts/verify-npm-release.mjs", mutatedTarball], {
        cwd: root,
        encoding: "utf8"
      });
      assert.notEqual(verifyMutated.status, 0, verifyMutated.stdout);
      assert.match(verifyMutated.stderr, /unexpected file package\/unexpected\.txt/);
    } finally {
      rmSync(mutatedRoot, { recursive: true, force: true });
    }

    const mutatedPackageJsonRoot = mkdtempSync(join(tmpdir(), "rust-calckernel-mutated-package-json-"));
    try {
      const unpack = spawnSync("tar", ["-xzf", matrixTarball, "-C", mutatedPackageJsonRoot], {
        cwd: root,
        encoding: "utf8"
      });
      assert.equal(unpack.status, 0, unpack.stderr || unpack.stdout);
      const packageJsonPath = join(mutatedPackageJsonRoot, "package", "package.json");
      const mutatedPackageJson = JSON.parse(readFileSync(packageJsonPath, "utf8"));
      mutatedPackageJson.dependencies = { wabt: "1.0.0" };
      writeFileSync(packageJsonPath, `${JSON.stringify(mutatedPackageJson, null, 2)}\n`);
      const mutatedTarball = join(mutatedPackageJsonRoot, "calckernel-mutated-package-json.tgz");
      const repack = spawnSync("tar", ["-czf", mutatedTarball, "-C", mutatedPackageJsonRoot, "package"], {
        cwd: root,
        encoding: "utf8"
      });
      assert.equal(repack.status, 0, repack.stderr || repack.stdout);
      const verifyMutated = spawnSync(process.execPath, ["scripts/verify-npm-release.mjs", mutatedTarball], {
        cwd: root,
        encoding: "utf8"
      });
      assert.notEqual(verifyMutated.status, 0, verifyMutated.stdout);
      assert.match(verifyMutated.stderr, /package\/package\.json must not declare dependencies/);
    } finally {
      rmSync(mutatedPackageJsonRoot, { recursive: true, force: true });
    }
  } finally {
    rmSync(matrixPackRoot, { recursive: true, force: true });
  }
} finally {
  spawnSync(process.execPath, ["scripts/cleanup-npm-package.mjs"], { cwd: root, encoding: "utf8" });
  rmSync(stagedBinaryRoot, { recursive: true, force: true });
  rmSync(cargoTargetRoot, { recursive: true, force: true });
}

function binaryStubForTarget(target) {
  const bytes = Buffer.alloc(4096);
  if (target.platform === "darwin") {
    bytes.set([0xcf, 0xfa, 0xed, 0xfe], 0);
    bytes.writeUInt32LE(target.arch === "arm64" ? 0x0100000c : 0x01000007, 4);
  } else if (target.platform === "linux") {
    bytes.set([0x7f, 0x45, 0x4c, 0x46], 0);
    bytes[4] = 2;
    bytes[5] = 1;
    bytes[6] = 1;
    bytes.writeUInt16LE(target.arch === "arm64" ? 183 : 62, 18);
  } else if (target.platform === "win32") {
    bytes.write("MZ", 0, "ascii");
    bytes.writeUInt32LE(0x80, 0x3c);
    bytes.write("PE\0\0", 0x80, "ascii");
    bytes.writeUInt16LE(target.arch === "arm64" ? 0xaa64 : 0x8664, 0x84);
  } else {
    throw new Error(`unsupported test target ${target.name}`);
  }
  return bytes;
}

function expectedBinaryFormat(target) {
  if (target.platform === "darwin") return "Mach-O";
  if (target.platform === "linux") return "ELF";
  if (target.platform === "win32") return "PE";
  throw new Error(`unsupported test target ${target.name}`);
}

const memory = new WebAssembly.Memory({ initial: 1 });
const arena = publicApi.createCKWasmArena({
  memory,
  __ck_heap_base: { value: 64 }
});
const copied = arena.copyInF64(new Float64Array([1.25, -2.5, 3.75]));
assert.equal(copied.ptr, 64);
assert.deepEqual(Array.from(copied.view), [1.25, -2.5, 3.75]);
assert.deepEqual(Array.from(arena.copyOutF64(copied.ptr, 3)), [1.25, -2.5, 3.75]);

const help = spawnSync(process.execPath, [join(root, pkg.bin.ckc), "--help"], {
  cwd: root,
  env: { ...process.env, CKC_BIN: process.env.CKC_BIN },
  encoding: "utf8"
});
assert.equal(help.status, 0, help.stderr || help.stdout);
assert.match(help.stdout, /Usage:\n\s+ckc check <file>/);

const pack = spawnSync("npm", ["pack", "--dry-run", "--json", "--ignore-scripts"], {
  cwd: root,
  encoding: "utf8"
});
assert.equal(pack.status, 0, pack.stderr || pack.stdout);
const packed = JSON.parse(pack.stdout)[0];
const files = packed.files.map((file) => file.path);
assert(files.includes("package.json"));
assert(files.includes("npm/index.js"));
assert(files.includes("npm/index.d.ts"));
assert(files.includes("npm/ckc.js"));
assert(files.includes("npm/platform.js"));
assert(files.includes("README.md"));
assert(files.includes("docs/npm-release.md"));
assert(files.includes("docs/architecture-review.md"));
assert(files.includes("docs/zh-CN/architecture-review.md"));
assert(!files.some((file) => file.startsWith("docs/superpowers/")));
assert(!files.some((file) => file.startsWith("target/")));
assert(!files.some((file) => file.startsWith("src/")));

const tmpRoot = mkdtempSync(join(tmpdir(), "rust-calckernel-package-"));
try {
  const releasePack = spawnSync("npm", ["pack", "--json", "--pack-destination", tmpRoot], {
    cwd: root,
    encoding: "utf8"
  });
  assert.equal(releasePack.status, 0, releasePack.stderr || releasePack.stdout);
  const release = JSON.parse(releasePack.stdout)[0];
  const releaseFiles = release.files.map((file) => file.path);
  assert(releaseFiles.some((file) => /^npm\/bin\/ckc-[^/]+$/.test(file)), releaseFiles.join("\n"));

  const consumer = join(tmpRoot, "consumer");
  mkdirSync(consumer);
  writeFileSync(
    join(consumer, "package.json"),
    JSON.stringify({
      type: "module",
      dependencies: {
        calckernel: `file:${join(tmpRoot, release.filename)}`
      }
    })
  );

  const install = spawnSync("npm", ["install", "--ignore-scripts"], {
    cwd: consumer,
    encoding: "utf8"
  });
  assert.equal(install.status, 0, install.stderr || install.stdout);

  const installedEnv = { ...process.env };
  delete installedEnv.CKC_BIN;
  const installedBin = join(
    consumer,
    "node_modules",
    ".bin",
    process.platform === "win32" ? "ckc.cmd" : "ckc"
  );
  const installedHelp = spawnSync(installedBin, ["--help"], {
    cwd: consumer,
    env: installedEnv,
    encoding: "utf8"
  });
  assert.equal(installedHelp.status, 0, installedHelp.stderr || installedHelp.stdout);
  assert.match(installedHelp.stdout, /Usage:\n\s+ckc check <file>/);

  const installedApi = spawnSync(
    process.execPath,
    [
      "--input-type=module",
      "--eval",
      "import assert from 'node:assert/strict'; import { createCKWasmArena } from 'calckernel'; assert.equal(typeof createCKWasmArena, 'function');"
    ],
    {
      cwd: consumer,
      env: installedEnv,
      encoding: "utf8"
    }
  );
  assert.equal(installedApi.status, 0, installedApi.stderr || installedApi.stdout);
} finally {
  rmSync(tmpRoot, { recursive: true, force: true });
}
"#
}

fn package_wasm_arena_boundary_script() -> &'static str {
    r#"
import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { mkdirSync, mkdtempSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { pathToFileURL } from "node:url";

const root = process.cwd();
const { CKWasmArena, createCKWasmArena } = await import(pathToFileURL(join(root, "npm/index.js")));

function assertThrowsMatch(fn, pattern) {
  let thrown = undefined;
  try {
    fn();
  } catch (error) {
    thrown = error;
  }
  assert(thrown, `expected ${pattern} to be thrown`);
  assert.match(thrown instanceof Error ? thrown.message : String(thrown), pattern);
}

function close(actual, expected) {
  return Math.abs(actual - expected) < 0.0000001;
}

function runCkc(cwd, args) {
  const output = spawnSync(process.execPath, [join(root, "npm/ckc.js"), ...args], {
    cwd,
    env: { ...process.env, CKC_BIN: process.env.CKC_BIN },
    encoding: "utf8"
  });
  assert.equal(output.status, 0, output.stderr || output.stdout);
  assert.equal(output.stderr, "");
  return output.stdout;
}

assert.equal(typeof CKWasmArena, "function");
assert.equal(typeof createCKWasmArena, "function");

const memory = new WebAssembly.Memory({ initial: 1 });
const arena = new CKWasmArena(memory, { heapBase: 0 });
assert.equal(arena.memory, memory);
assertThrowsMatch(
  () => new CKWasmArena({ buffer: new ArrayBuffer(8), grow: () => 0 }),
  /CKWasmArena\.constructor.*WebAssembly\.Memory.*Pass instance\.exports\.memory/s
);
assertThrowsMatch(
  () => new CKWasmArena(memory, { heapBase: -1 }),
  /CKWasmArena\.constructor.*heapBase.*non-negative safe integer/s
);
assertThrowsMatch(
  () => arena.allocBytes(1, 0),
  /CKWasmArena\.allocBytes.*align.*positive safe integer/s
);
assertThrowsMatch(
  () => arena.allocBytes(-1, 1),
  /CKWasmArena\.allocBytes.*bytes.*non-negative safe integer/s
);
assertThrowsMatch(
  () => arena.allocF64(-1),
  /CKWasmArena\.allocF64.*length.*non-negative safe integer/s
);
assertThrowsMatch(
  () => arena.allocF64(Number.MAX_SAFE_INTEGER),
  /CKWasmArena\.allocF64.*byte length.*safe integer/s
);
assertThrowsMatch(
  () => arena.viewF64(1, 1),
  /CKWasmArena\.viewF64.*ptr.*8-byte aligned/s
);
assertThrowsMatch(
  () => arena.viewF64(0, -1),
  /CKWasmArena\.viewF64.*length.*non-negative safe integer/s
);
assertThrowsMatch(
  () => new CKWasmArena(new WebAssembly.Memory({ initial: 1 })).allocBytes(1, 1),
  /CKWasmArena\.allocBytes.*heapBase/s
);

const limitedArena = new CKWasmArena(new WebAssembly.Memory({ initial: 1, maximum: 1 }), { heapBase: 0 });
assertThrowsMatch(
  () => limitedArena.ensureBytes(70_000),
  /CKWasmArena\.ensureBytes.*memory\.grow failed.*Pre-grow/s
);
assertThrowsMatch(
  () => limitedArena.viewF64(65_536, 1),
  /CKWasmArena\.viewF64.*memory\.grow failed.*Pre-grow/s
);

assertThrowsMatch(
  () => arena.copyInF64(new Int32Array([1])),
  /CKWasmArena\.copyInF64.*Float64Array.*Pass a Float64Array/s
);
assertThrowsMatch(
  () => arena.copyInI32(new Float64Array([1])),
  /CKWasmArena\.copyInI32.*Int32Array.*Pass an Int32Array/s
);
assertThrowsMatch(
  () => arena.copyInU32(new Int32Array([1])),
  /CKWasmArena\.copyInU32.*Uint32Array.*Pass a Uint32Array/s
);

const alignedArena = new CKWasmArena(new WebAssembly.Memory({ initial: 1 }), { heapBase: 3 });
assert.equal(alignedArena.allocBytes(1, 1), 3);
assert.equal(alignedArena.allocI32(1) % 4, 0);
assert.equal(alignedArena.allocU32(1) % 4, 0);
assert.equal(alignedArena.allocF64(1) % 8, 0);
assert.equal(alignedArena.allocI64(1) % 8, 0);
assert.equal(alignedArena.allocU64(1) % 8, 0);

const copyArena = new CKWasmArena(new WebAssembly.Memory({ initial: 1 }), { heapBase: 64 });
const copiedF64 = copyArena.copyInF64(new Float64Array([1.25, -2.5, 3.75]));
assert.equal(copiedF64.ptr % 8, 0);
assert.deepEqual(Array.from(copiedF64.view), [1.25, -2.5, 3.75]);
const copiedI32 = copyArena.copyInI32(new Int32Array([-7, 0, 42]));
assert.equal(copiedI32.ptr % 4, 0);
assert.deepEqual(Array.from(copiedI32.view), [-7, 0, 42]);
const copiedU32 = copyArena.copyInU32(new Uint32Array([0, 42, 0xffff_ffff]));
assert.equal(copiedU32.ptr % 4, 0);
assert.deepEqual(Array.from(copiedU32.view), [0, 42, 0xffff_ffff]);

const outArena = new CKWasmArena(new WebAssembly.Memory({ initial: 1 }), { heapBase: 128 });
const outPtr = outArena.allocF64(2);
const outView = outArena.viewF64(outPtr, 2);
outView.set([10.5, 20.25]);
const outCopy = outArena.copyOutF64(outPtr, 2);
assert.deepEqual(Array.from(outCopy), [10.5, 20.25]);
outView[0] = 99.0;
assert.equal(outCopy[0], 10.5);
assert.notEqual(outCopy.buffer, outView.buffer);

const growMemory = new WebAssembly.Memory({ initial: 1 });
const growArena = new CKWasmArena(growMemory, { heapBase: 0 });
const growPtr = growArena.allocF64(1);
const beforeGrow = growArena.viewF64(growPtr, 1);
beforeGrow[0] = 42.5;
const beforeBuffer = beforeGrow.buffer;
growArena.ensureBytes(70_000);
growArena.refreshViewsIfNeeded();
const afterGrow = growArena.viewF64(growPtr, 1);
assert.equal(afterGrow.buffer, growMemory.buffer);
assert.notEqual(afterGrow.buffer, beforeBuffer);
assert.equal(afterGrow[0], 42.5);

const heapArena = CKWasmArena.fromExports({
  memory: new WebAssembly.Memory({ initial: 1 }),
  __heap_base: { value: 16 },
  __ck_heap_base: { value: 64 }
});
assert.equal(heapArena.allocBytes(1, 1), 64);
assert.equal(
  createCKWasmArena({
    exports: {
      memory: new WebAssembly.Memory({ initial: 1 }),
      __heap_base: { value: 16 },
      __ck_heap_base: { value: 64 }
    }
  }).allocBytes(1, 1),
  64
);
assert.equal(
  createCKWasmArena(
    {
      memory: new WebAssembly.Memory({ initial: 1 }),
      __ck_heap_base: { value: 64 }
    },
    { heapBase: 32 }
  ).allocBytes(1, 1),
  32
);
assertThrowsMatch(
  () => createCKWasmArena({}),
  /createCKWasmArena.*exports\.memory.*Pass a WebAssembly\.Instance/s
);
assertThrowsMatch(
  () => createCKWasmArena({ memory: new WebAssembly.Memory({ initial: 1 }) }),
  /createCKWasmArena.*heapBase.*Pass \{ heapBase \}.*__ck_heap_base/s
);

const tmpRoot = mkdtempSync(join(tmpdir(), "rust-calckernel-wasm-arena-"));
try {
  mkdirSync(join(tmpRoot, "build"), { recursive: true });
  writeFileSync(
    join(tmpRoot, "arena_read.ck"),
    `
      export fn read_f64(values: ptr<f64>, i: i32) -> f64 {
        return values[i];
      }
    `
  );
  assert.equal(
    runCkc(tmpRoot, ["emit-wasm", "arena_read.ck", "--out", "build/arena_read.wasm"]),
    "OK: emitted WASM build/arena_read.wasm\n"
  );
  const { instance: readInstance } = await WebAssembly.instantiate(readFileSync(join(tmpRoot, "build/arena_read.wasm")));
  const readF64 = readInstance.exports.read_f64;
  assert.equal(CKWasmArena.heapBaseFromExports(readInstance.exports), 0);
  const generatedArena = createCKWasmArena(readInstance);
  const generatedPtr = generatedArena.allocF64(3);
  assert.equal(generatedPtr, 0);
  generatedArena.viewF64(generatedPtr, 3).set([1.5, 2.25, 3.75]);
  assert(close(readF64(generatedPtr, 0), 1.5));
  assert(close(readF64(generatedPtr, 1), 2.25));
  assert(close(readF64(generatedPtr, 2), 3.75));

  writeFileSync(
    join(tmpRoot, "arena_f64_kernels.ck"),
    `
      export fn sum_f64(x: ptr<f64>, len: i32) -> f64 {
        let i: i32 = 0;
        let checksum: f64 = 0.0;

        while i < len {
          checksum = checksum + x[i];
          i = i + 1;
        }

        return checksum;
      }

      export fn axpy_f64(a: f64, x: ptr<f64>, y: ptr<f64>, len: i32) -> f64 {
        let i: i32 = 0;
        let checksum: f64 = 0.0;

        while i < len {
          let value: f64 = a * x[i] + y[i];
          y[i] = value;
          checksum = checksum + value;
          i = i + 1;
        }

        return checksum;
      }
    `
  );
  assert.equal(
    runCkc(tmpRoot, ["emit-wasm", "arena_f64_kernels.ck", "--out", "build/arena_f64_kernels.wasm", "-O3"]),
    "OK: emitted WASM build/arena_f64_kernels.wasm\n"
  );
  const { instance: kernelsInstance } = await WebAssembly.instantiate(
    readFileSync(join(tmpRoot, "build/arena_f64_kernels.wasm"))
  );
  const kernelsMemory = kernelsInstance.exports.memory;
  const sumF64 = kernelsInstance.exports.sum_f64;
  const axpyF64 = kernelsInstance.exports.axpy_f64;
  const kernelsArena = new CKWasmArena(kernelsMemory, { heapBase: 128 });
  const xPtr = kernelsArena.allocF64(4);
  const yPtr = kernelsArena.allocF64(4);
  const xView = kernelsArena.viewF64(xPtr, 4);
  const yView = kernelsArena.viewF64(yPtr, 4);

  xView.set([1.25, -2.5, 3.75, 4.5]);
  assert(close(sumF64(xPtr, xView.length), 7.0));
  xView.set([Number.POSITIVE_INFINITY, 1.0, 2.0, 3.0]);
  assert.equal(sumF64(xPtr, xView.length), Number.POSITIVE_INFINITY);
  xView.set([Number.NaN, 1.0, 2.0, 3.0]);
  assert(Number.isNaN(sumF64(xPtr, xView.length)));

  xView.set([1.0, 2.0, 3.0, 4.0]);
  yView.set([0.5, -1.0, 10.0, 20.0]);
  assert(close(axpyF64(2.0, xPtr, yPtr, xView.length), 49.5));
  assert.deepEqual(Array.from(yView), [2.5, 3.0, 16.0, 28.0]);
  assert.equal(yView.buffer, kernelsMemory.buffer);
  const yCopy = kernelsArena.copyOutF64(yPtr, yView.length);
  assert.notEqual(yCopy.buffer, kernelsMemory.buffer);
  assert.deepEqual(Array.from(yCopy), [2.5, 3.0, 16.0, 28.0]);
  yView[0] = 99.0;
  assert.equal(yCopy[0], 2.5);

  xView.set([-0.0, Number.POSITIVE_INFINITY, Number.NaN, 1.0]);
  yView.set([-0.0, 1.0, 2.0, Number.NEGATIVE_INFINITY]);
  axpyF64(1.0, xPtr, yPtr, xView.length);
  assert(Object.is(yView[0], -0.0));
  assert.equal(yView[1], Number.POSITIVE_INFINITY);
  assert(Number.isNaN(yView[2]));
  assert.equal(yView[3], Number.NEGATIVE_INFINITY);
} finally {
  rmSync(tmpRoot, { recursive: true, force: true });
}
"#
}
