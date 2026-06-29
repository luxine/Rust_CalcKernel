export interface SourcePosition {
  offset: number;
  line: number;
  column: number;
}

export interface SourceSpan {
  start: SourcePosition;
  end: SourcePosition;
}

export declare class SourceFile {
  readonly fileName: string;
  readonly text: string;

  constructor(fileName: string, text: string);
}

export type DiagnosticSeverity = "error";
export type DiagnosticCode =
  | "CK0001"
  | "CK1001"
  | "CK2001"
  | "CK2002"
  | "CK2003"
  | "CK2004"
  | "CK2005"
  | "CK2006"
  | "CK2007"
  | "CK2008";

export interface Diagnostic {
  code: DiagnosticCode;
  severity: DiagnosticSeverity;
  message: string;
  fileName: string;
  line: number;
  column: number;
  span: SourceSpan;
}

export declare function formatDiagnostic(sourceFile: SourceFile, diagnostic: Diagnostic): string;
export declare function formatDiagnostics(sourceFile: SourceFile, diagnostics: Diagnostic[]): string;

export declare const TokenKind: Readonly<{
  Eof: "Eof";
  Identifier: "Identifier";
  Integer: "Integer";
  Float: "Float";
  Struct: "Struct";
  Export: "Export";
  Fn: "Fn";
  Let: "Let";
  Return: "Return";
  If: "If";
  Else: "Else";
  While: "While";
  True: "True";
  False: "False";
  I32: "I32";
  I64: "I64";
  U32: "U32";
  U64: "U64";
  F64: "F64";
  Bool: "Bool";
  Ptr: "Ptr";
  LeftParen: "LeftParen";
  RightParen: "RightParen";
  LeftBrace: "LeftBrace";
  RightBrace: "RightBrace";
  LeftBracket: "LeftBracket";
  RightBracket: "RightBracket";
  Comma: "Comma";
  Colon: "Colon";
  Semicolon: "Semicolon";
  Dot: "Dot";
  Arrow: "Arrow";
  Plus: "Plus";
  Minus: "Minus";
  Star: "Star";
  Slash: "Slash";
  Percent: "Percent";
  Equal: "Equal";
  EqualEqual: "EqualEqual";
  Bang: "Bang";
  BangEqual: "BangEqual";
  Less: "Less";
  LessEqual: "LessEqual";
  Greater: "Greater";
  GreaterEqual: "GreaterEqual";
  AmpAmp: "AmpAmp";
  PipePipe: "PipePipe";
}>;
export type TokenKind = (typeof TokenKind)[keyof typeof TokenKind];

export interface Token {
  kind: TokenKind;
  text: string;
  line: number;
  column: number;
  start: number;
  end: number;
}

export interface LexResult {
  tokens: Token[];
  diagnostics: Diagnostic[];
}

export declare function lex(source: SourceFile): LexResult;

export interface AstNode {
  kind: string;
  span: SourceSpan;
}

export interface IdentifierNode extends AstNode {
  kind: "Identifier";
  name: string;
}

export interface Program extends AstNode {
  kind: "Program";
  declarations: Declaration[];
}

export type Declaration = StructDeclaration | FunctionDeclaration;

export interface StructDeclaration extends AstNode {
  kind: "StructDeclaration";
  name: IdentifierNode;
  fields: StructField[];
}

export interface StructField extends AstNode {
  kind: "StructField";
  name: IdentifierNode;
  type: TypeNode;
}

export interface FunctionDeclaration extends AstNode {
  kind: "FunctionDeclaration";
  exported: boolean;
  name: IdentifierNode;
  params: FunctionParam[];
  returnType: TypeNode;
  body: BlockStatement;
}

export interface FunctionParam extends AstNode {
  kind: "FunctionParam";
  name: IdentifierNode;
  type: TypeNode;
}

export type TypeNode = PrimitiveTypeNode | PointerTypeNode | NamedTypeNode | ErrorTypeNode;

export interface PrimitiveTypeNode extends AstNode {
  kind: "PrimitiveType";
  name: "i32" | "i64" | "u32" | "u64" | "f64" | "bool";
}

export interface PointerTypeNode extends AstNode {
  kind: "PointerType";
  elementType: TypeNode;
}

export interface NamedTypeNode extends AstNode {
  kind: "NamedType";
  name: IdentifierNode;
}

export interface ErrorTypeNode extends AstNode {
  kind: "ErrorType";
}

export type Statement =
  | BlockStatement
  | LetStatement
  | AssignmentStatement
  | ReturnStatement
  | IfStatement
  | WhileStatement
  | ErrorStatement;

export interface BlockStatement extends AstNode {
  kind: "BlockStatement";
  statements: Statement[];
}

export interface LetStatement extends AstNode {
  kind: "LetStatement";
  name: IdentifierNode;
  type: TypeNode;
  initializer: Expression;
}

export interface AssignmentStatement extends AstNode {
  kind: "AssignmentStatement";
  target: Expression;
  value: Expression;
}

export interface ReturnStatement extends AstNode {
  kind: "ReturnStatement";
  value: Expression;
}

export interface IfStatement extends AstNode {
  kind: "IfStatement";
  condition: Expression;
  thenBlock: BlockStatement;
  elseBlock: BlockStatement | null;
}

export interface WhileStatement extends AstNode {
  kind: "WhileStatement";
  condition: Expression;
  body: BlockStatement;
}

export interface ErrorStatement extends AstNode {
  kind: "ErrorStatement";
}

export type Expression =
  | IdentifierExpression
  | IntegerLiteral
  | FloatLiteral
  | BoolLiteral
  | UnaryExpression
  | BinaryExpression
  | CallExpression
  | FieldExpression
  | IndexExpression
  | ParenthesizedExpression
  | ErrorExpression;

export interface IdentifierExpression extends AstNode {
  kind: "IdentifierExpression";
  name: string;
}

export interface IntegerLiteral extends AstNode {
  kind: "IntegerLiteral";
  text: string;
}

export interface FloatLiteral extends AstNode {
  kind: "FloatLiteral";
  text: string;
}

export interface BoolLiteral extends AstNode {
  kind: "BoolLiteral";
  value: boolean;
}

export interface UnaryExpression extends AstNode {
  kind: "UnaryExpression";
  operator: "!" | "-";
  operand: Expression;
}

export interface BinaryExpression extends AstNode {
  kind: "BinaryExpression";
  operator: string;
  left: Expression;
  right: Expression;
}

export interface CallExpression extends AstNode {
  kind: "CallExpression";
  callee: Expression;
  args: Expression[];
}

export interface FieldExpression extends AstNode {
  kind: "FieldExpression";
  object: Expression;
  field: IdentifierNode;
}

export interface IndexExpression extends AstNode {
  kind: "IndexExpression";
  object: Expression;
  index: Expression;
}

export interface ParenthesizedExpression extends AstNode {
  kind: "ParenthesizedExpression";
  expression: Expression;
}

export interface ErrorExpression extends AstNode {
  kind: "ErrorExpression";
}

export interface ParseResult {
  ast: Program;
  diagnostics: Diagnostic[];
}

export declare function parse(source: SourceFile): ParseResult;

export type PrimitiveTypeName = "i32" | "i64" | "u32" | "u64" | "f64" | "bool";

export type CalcKernelType =
  | { kind: "primitive"; name: PrimitiveTypeName }
  | { kind: "pointer"; elementType: CalcKernelType }
  | { kind: "struct"; name: string }
  | { kind: "integerLiteral" }
  | { kind: "unknown" };

export interface StructSymbol {
  name: string;
  declaration: StructDeclaration;
  fields: Map<string, CalcKernelType>;
}

export interface FunctionSymbol {
  name: string;
  declaration: FunctionDeclaration;
  params: CalcKernelType[];
  returnType: CalcKernelType;
}

export interface VariableSymbol {
  name: string;
  type: CalcKernelType;
}

export declare class SymbolTable {
  readonly structs: Map<string, StructSymbol>;
  readonly functions: Map<string, FunctionSymbol>;
}

export declare class Scope {
  readonly parent: Scope | null;

  constructor(parent?: Scope | null);

  declare(variable: VariableSymbol): boolean;
  lookup(name: string): VariableSymbol | null;
}

export interface TypedAst {
  program: Program;
  expressionTypes: Map<Expression, CalcKernelType>;
}

export type TypeMap = Map<Expression, CalcKernelType>;
export type LetTypeMap = Map<LetStatement, CalcKernelType>;

export interface StructFieldInfo {
  name: string;
  type: CalcKernelType;
  declaration: StructField;
}

export interface StructInfo {
  name: string;
  declaration: StructDeclaration;
  fields: StructFieldInfo[];
  fieldMap: Map<string, StructFieldInfo>;
}

export interface FunctionParamInfo {
  name: string;
  type: CalcKernelType;
  declaration: FunctionParam;
}

export interface FunctionInfo {
  name: string;
  exported: boolean;
  declaration: FunctionDeclaration;
  params: FunctionParamInfo[];
  returnType: CalcKernelType;
}

export interface CheckedProgram {
  ast: Program;
  symbols: SymbolTable;
  types: TypeMap;
  localTypes: LetTypeMap;
  structs: StructInfo[];
  functions: FunctionInfo[];
  structMap: Map<string, StructInfo>;
  functionMap: Map<string, FunctionInfo>;
}

export interface CheckResult {
  ast: Program;
  typedAst: TypedAst;
  checkedProgram: CheckedProgram;
  diagnostics: Diagnostic[];
  symbols: SymbolTable;
}

export declare function check(source: SourceFile): CheckResult;
export declare function getExprType(checkedProgram: CheckedProgram, expression: Expression): CalcKernelType | undefined;
export declare function getLetType(checkedProgram: CheckedProgram, statement: LetStatement): CalcKernelType | undefined;
export declare function getStructInfo(checkedProgram: CheckedProgram, name: string): StructInfo | undefined;
export declare function getFieldInfo(checkedProgram: CheckedProgram, structName: string, fieldName: string): StructFieldInfo | undefined;
export declare function getFunctionInfo(checkedProgram: CheckedProgram, name: string): FunctionInfo | undefined;

export type OverflowMode = "unchecked" | "checked";
export type OptimizationLevel = 0 | 1 | 2 | 3;

export interface OptimizationOptions {
  optLevel?: OptimizationLevel;
}

export interface CCodegenOptions extends OptimizationOptions {
  overflowMode?: OverflowMode;
}

export interface MirPassDebugFlags {
  printPassPipeline?: boolean;
  printMirBeforeOpt?: boolean;
  printMirAfterOpt?: boolean;
}

export interface EmitCFilesOptions extends CCodegenOptions {
  cFile: string;
  headerFile: string;
  headerFileName: string;
  mirDebug?: MirPassDebugFlags;
  writeDebug?: (text: string) => void;
}

export interface EmitCSourceOptions extends CCodegenOptions {
  headerFileName: string;
  mirDebug?: MirPassDebugFlags;
  writeDebug?: (text: string) => void;
}

export type CKHostPlatform =
  | "aix"
  | "android"
  | "cygwin"
  | "darwin"
  | "freebsd"
  | "haiku"
  | "linux"
  | "netbsd"
  | "openbsd"
  | "sunos"
  | "win32"
  | (string & {});

export type BuildPlatform = CKHostPlatform;

export interface CKSystemError extends Error {
  code?: string;
  errno?: number;
  syscall?: string;
  path?: string;
}

export interface CommandResult {
  status: number | null;
  stdout: string;
  stderr: string;
  error?: CKSystemError;
}

export type CommandRunner = (command: string, args: string[]) => CommandResult;

export interface BuildSharedLibraryOptions extends EmitCFilesOptions {
  outputPath: string;
  platform: BuildPlatform;
  runCommand: CommandRunner;
}

export interface BuildSharedLibraryResult {
  ok: boolean;
  outputPath: string;
  message?: string;
}

export declare function emitCHeader(checked: CheckResult, options?: CCodegenOptions): string;
export declare function emitCSource(checked: CheckResult, options: EmitCSourceOptions): string;
export declare function emitCFiles(checked: CheckResult, options: EmitCFilesOptions): void;
export declare function buildSharedLibrary(checked: CheckResult, options: BuildSharedLibraryOptions): BuildSharedLibraryResult;
export declare function sharedLibraryOutputPath(outputPath: string, platform: BuildPlatform): string;

export interface CKWasmArenaOptions {
  heapBase?: number;
}

export interface CKWasmMemory {
  readonly buffer: ArrayBuffer;
  grow(delta: number): number;
}

export interface CKWasmGlobal {
  value: number | bigint;
}

export interface CKWasmArenaCopy<T extends ArrayBufferView> {
  ptr: number;
  view: T;
}

export interface CKWasmInstanceLike {
  exports: Record<string, unknown>;
}

export declare function createCKWasmArena(
  instanceOrExports: CKWasmInstanceLike | Record<string, unknown>,
  options?: CKWasmArenaOptions
): CKWasmArena;

export declare class CKWasmArena {
  readonly memory: CKWasmMemory;

  constructor(memory: CKWasmMemory, options?: CKWasmArenaOptions);

  static fromExports(exports: Record<string, unknown>, options?: CKWasmArenaOptions): CKWasmArena;
  static heapBaseFromExports(exports: Record<string, unknown>): number | undefined;

  ensureBytes(bytes: number): void;
  refreshViewsIfNeeded(): void;
  allocBytes(bytes: number, align: number): number;
  allocF64(length: number): number;
  allocI32(length: number): number;
  allocU32(length: number): number;
  allocI64(length: number): number;
  allocU64(length: number): number;
  viewF64(ptr: number, length: number): Float64Array;
  viewI32(ptr: number, length: number): Int32Array;
  viewU32(ptr: number, length: number): Uint32Array;
  viewI64(ptr: number, length: number): BigInt64Array;
  viewU64(ptr: number, length: number): BigUint64Array;
  copyInF64(src: Float64Array): CKWasmArenaCopy<Float64Array>;
  copyInI32(src: Int32Array): CKWasmArenaCopy<Int32Array>;
  copyInU32(src: Uint32Array): CKWasmArenaCopy<Uint32Array>;
  copyOutF64(ptr: number, length: number): Float64Array;
}
