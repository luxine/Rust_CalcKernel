import { spawnSync } from "node:child_process";
import { existsSync, mkdirSync, mkdtempSync, realpathSync, readFileSync, renameSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { dirname, join, resolve } from "node:path";
import { fileURLToPath } from "node:url";
import { currentPlatformBinaryName, supportedTargetNames } from "./platform.js";

const WASM_PAGE_BYTES = 64 * 1024;

export class SourceFile {
  constructor(fileName, text) {
    this.fileName = fileName;
    this.text = text;
  }
}

export const TokenKind = Object.freeze({
  Eof: "Eof",
  Identifier: "Identifier",
  Integer: "Integer",
  Float: "Float",
  Struct: "Struct",
  Export: "Export",
  Fn: "Fn",
  Let: "Let",
  Return: "Return",
  If: "If",
  Else: "Else",
  While: "While",
  True: "True",
  False: "False",
  I32: "I32",
  I64: "I64",
  U32: "U32",
  U64: "U64",
  F64: "F64",
  Bool: "Bool",
  Ptr: "Ptr",
  LeftParen: "LeftParen",
  RightParen: "RightParen",
  LeftBrace: "LeftBrace",
  RightBrace: "RightBrace",
  LeftBracket: "LeftBracket",
  RightBracket: "RightBracket",
  Comma: "Comma",
  Colon: "Colon",
  Semicolon: "Semicolon",
  Dot: "Dot",
  Arrow: "Arrow",
  Plus: "Plus",
  Minus: "Minus",
  Star: "Star",
  Slash: "Slash",
  Percent: "Percent",
  Equal: "Equal",
  EqualEqual: "EqualEqual",
  Bang: "Bang",
  BangEqual: "BangEqual",
  Less: "Less",
  LessEqual: "LessEqual",
  Greater: "Greater",
  GreaterEqual: "GreaterEqual",
  AmpAmp: "AmpAmp",
  PipePipe: "PipePipe"
});

const KEYWORDS = new Map([
  ["struct", TokenKind.Struct],
  ["export", TokenKind.Export],
  ["fn", TokenKind.Fn],
  ["let", TokenKind.Let],
  ["return", TokenKind.Return],
  ["if", TokenKind.If],
  ["else", TokenKind.Else],
  ["while", TokenKind.While],
  ["true", TokenKind.True],
  ["false", TokenKind.False],
  ["i32", TokenKind.I32],
  ["i64", TokenKind.I64],
  ["u32", TokenKind.U32],
  ["u64", TokenKind.U64],
  ["f64", TokenKind.F64],
  ["bool", TokenKind.Bool],
  ["ptr", TokenKind.Ptr]
]);

export function lex(source) {
  return new Lexer(source).lex();
}

class Lexer {
  constructor(source) {
    this.source = source;
    this.tokens = [];
    this.diagnostics = [];
    this.offset = 0;
    this.line = 1;
    this.column = 1;
  }

  lex() {
    while (!this.isAtEnd()) {
      this.scanToken();
    }

    const position = this.position();
    this.tokens.push({
      kind: TokenKind.Eof,
      text: "",
      line: position.line,
      column: position.column,
      start: position.offset,
      end: position.offset
    });

    return {
      tokens: this.tokens,
      diagnostics: this.diagnostics
    };
  }

  scanToken() {
    const char = this.peek();

    if (isWhitespace(char)) {
      this.advance();
      return;
    }

    if (char === "/" && this.peekNext() === "/") {
      this.skipLineComment();
      return;
    }

    if (isIdentifierStart(char)) {
      this.scanIdentifierOrKeyword();
      return;
    }

    if (isDigit(char)) {
      this.scanNumber();
      return;
    }

    const start = this.position();

    switch (char) {
      case "(":
        this.advance();
        this.addToken(TokenKind.LeftParen, start);
        return;
      case ")":
        this.advance();
        this.addToken(TokenKind.RightParen, start);
        return;
      case "{":
        this.advance();
        this.addToken(TokenKind.LeftBrace, start);
        return;
      case "}":
        this.advance();
        this.addToken(TokenKind.RightBrace, start);
        return;
      case "[":
        this.advance();
        this.addToken(TokenKind.LeftBracket, start);
        return;
      case "]":
        this.advance();
        this.addToken(TokenKind.RightBracket, start);
        return;
      case ",":
        this.advance();
        this.addToken(TokenKind.Comma, start);
        return;
      case ":":
        this.advance();
        this.addToken(TokenKind.Colon, start);
        return;
      case ";":
        this.advance();
        this.addToken(TokenKind.Semicolon, start);
        return;
      case ".":
        if (isDigit(this.peekNext())) {
          this.scanMalformedFloatStartingWithDot();
          return;
        }
        this.advance();
        this.addToken(TokenKind.Dot, start);
        return;
      case "+":
        this.advance();
        this.addToken(TokenKind.Plus, start);
        return;
      case "-":
        this.advance();
        this.addToken(this.match(">") ? TokenKind.Arrow : TokenKind.Minus, start);
        return;
      case "*":
        this.advance();
        this.addToken(TokenKind.Star, start);
        return;
      case "/":
        this.advance();
        this.addToken(TokenKind.Slash, start);
        return;
      case "%":
        this.advance();
        this.addToken(TokenKind.Percent, start);
        return;
      case "=":
        this.advance();
        this.addToken(this.match("=") ? TokenKind.EqualEqual : TokenKind.Equal, start);
        return;
      case "!":
        this.advance();
        this.addToken(this.match("=") ? TokenKind.BangEqual : TokenKind.Bang, start);
        return;
      case "<":
        this.advance();
        this.addToken(this.match("=") ? TokenKind.LessEqual : TokenKind.Less, start);
        return;
      case ">":
        this.advance();
        this.addToken(this.match("=") ? TokenKind.GreaterEqual : TokenKind.Greater, start);
        return;
      case "&":
        this.advance();
        if (this.match("&")) {
          this.addToken(TokenKind.AmpAmp, start);
        } else {
          this.reportUnexpected(start, char);
        }
        return;
      case "|":
        this.advance();
        if (this.match("|")) {
          this.addToken(TokenKind.PipePipe, start);
        } else {
          this.reportUnexpected(start, char);
        }
        return;
      default:
        this.advance();
        this.reportUnexpected(start, char);
    }
  }

  scanIdentifierOrKeyword() {
    const start = this.position();
    this.advance();

    while (!this.isAtEnd() && isIdentifierPart(this.peek())) {
      this.advance();
    }

    const text = this.source.text.slice(start.offset, this.offset);
    this.addToken(KEYWORDS.get(text) ?? TokenKind.Identifier, start);
  }

  scanNumber() {
    const start = this.position();
    this.advance();

    while (!this.isAtEnd() && isDigit(this.peek())) {
      this.advance();
    }

    let isFloat = false;

    if (this.peek() === ".") {
      isFloat = true;
      this.advance();
      if (!isDigit(this.peek())) {
        this.reportMalformedFloat(start);
        return;
      }

      while (!this.isAtEnd() && isDigit(this.peek())) {
        this.advance();
      }
    }

    if (isExponentStart(this.peek())) {
      isFloat = true;
      this.advance();
      if (this.peek() === "+" || this.peek() === "-") {
        this.advance();
      }

      if (!isDigit(this.peek())) {
        this.reportMalformedFloat(start);
        return;
      }

      while (!this.isAtEnd() && isDigit(this.peek())) {
        this.advance();
      }
    }

    this.addToken(isFloat ? TokenKind.Float : TokenKind.Integer, start);
  }

  scanMalformedFloatStartingWithDot() {
    const start = this.position();
    this.advance();

    while (!this.isAtEnd() && isDigit(this.peek())) {
      this.advance();
    }

    this.reportMalformedFloat(start);
  }

  skipLineComment() {
    while (!this.isAtEnd() && this.peek() !== "\n") {
      this.advance();
    }
  }

  addToken(kind, start) {
    this.tokens.push({
      kind,
      text: this.source.text.slice(start.offset, this.offset),
      line: start.line,
      column: start.column,
      start: start.offset,
      end: this.offset
    });
  }

  reportUnexpected(start, char) {
    this.diagnostics.push(
      errorAt(this.source, { start, end: this.position() }, "CK0001", `Unexpected character '${char}'.`)
    );
  }

  reportMalformedFloat(start) {
    const text = this.source.text.slice(start.offset, this.offset);
    this.diagnostics.push(errorAt(this.source, { start, end: this.position() }, "CK0001", `Malformed float literal '${text}'.`));
  }

  match(expected) {
    if (this.isAtEnd() || this.peek() !== expected) {
      return false;
    }

    this.advance();
    return true;
  }

  advance() {
    const char = this.source.text[this.offset] ?? "";
    this.offset += 1;

    if (char === "\n") {
      this.line += 1;
      this.column = 1;
    } else {
      this.column += 1;
    }

    return char;
  }

  peek() {
    return this.source.text[this.offset] ?? "";
  }

  peekNext() {
    return this.source.text[this.offset + 1] ?? "";
  }

  isAtEnd() {
    return this.offset >= this.source.text.length;
  }

  position() {
    return {
      offset: this.offset,
      line: this.line,
      column: this.column
    };
  }
}

function errorAt(source, span, code, message) {
  return {
    code,
    severity: "error",
    message,
    fileName: source.fileName,
    line: span.start.line,
    column: span.start.column,
    span
  };
}

function isWhitespace(char) {
  return char === " " || char === "\r" || char === "\t" || char === "\n";
}

function isDigit(char) {
  return char >= "0" && char <= "9";
}

function isExponentStart(char) {
  return char === "e" || char === "E";
}

function isIdentifierStart(char) {
  return (char >= "A" && char <= "Z") || (char >= "a" && char <= "z") || char === "_";
}

function isIdentifierPart(char) {
  return isIdentifierStart(char) || isDigit(char);
}

export function parse(source) {
  const lexResult = lex(source);
  return new Parser(source, lexResult.tokens, [...lexResult.diagnostics]).parse();
}

class Parser {
  constructor(source, tokens, diagnostics) {
    this.source = source;
    this.tokens = tokens;
    this.diagnostics = diagnostics;
    this.index = 0;
  }

  parse() {
    const start = this.positionFromToken(this.current());
    const declarations = [];

    while (!this.check(TokenKind.Eof)) {
      const declaration = this.parseDeclaration();
      if (declaration) {
        declarations.push(declaration);
      }
    }

    const end = this.positionFromToken(this.current());
    return {
      ast: {
        kind: "Program",
        declarations,
        span: { start, end }
      },
      diagnostics: this.diagnostics
    };
  }

  parseDeclaration() {
    if (this.check(TokenKind.Struct)) {
      return this.parseStructDeclaration();
    }

    if (this.check(TokenKind.Export) || this.check(TokenKind.Fn)) {
      return this.parseFunctionDeclaration();
    }

    this.error(this.current(), "Expected declaration.");
    this.advance();
    return null;
  }

  parseStructDeclaration() {
    const structToken = this.consume(TokenKind.Struct, "Expected 'struct'.");
    const name = this.parseIdentifier("Expected struct name.");
    this.consume(TokenKind.LeftBrace, "Expected '{' after struct name.");

    const fields = [];
    while (!this.check(TokenKind.RightBrace) && !this.check(TokenKind.Eof)) {
      const fieldStart = this.current();
      const fieldName = this.parseIdentifier("Expected field name.");
      this.consume(TokenKind.Colon, "Expected ':' after field name.");
      const fieldType = this.parseType();
      const semicolon = this.consume(TokenKind.Semicolon, "Expected ';' after struct field.");
      fields.push({
        kind: "StructField",
        name: fieldName,
        type: fieldType,
        span: this.spanBetweenTokens(fieldStart, semicolon)
      });
    }

    const end = this.consume(TokenKind.RightBrace, "Expected '}' after struct fields.");
    return {
      kind: "StructDeclaration",
      name,
      fields,
      span: this.spanBetweenTokens(structToken, end)
    };
  }

  parseFunctionDeclaration() {
    const startToken = this.match(TokenKind.Export) ? this.previous() : this.current();
    const exported = startToken.kind === TokenKind.Export;
    this.consume(TokenKind.Fn, "Expected 'fn' after 'export'.");
    const name = this.parseIdentifier("Expected function name.");
    this.consume(TokenKind.LeftParen, "Expected '(' after function name.");

    const params = [];
    if (!this.check(TokenKind.RightParen)) {
      do {
        params.push(this.parseFunctionParam());
      } while (this.match(TokenKind.Comma));
    }

    this.consume(TokenKind.RightParen, "Expected ')' after parameters.");
    this.consume(TokenKind.Arrow, "Expected '->' before return type.");
    const returnType = this.parseType();
    const body = this.parseBlockStatement();

    return {
      kind: "FunctionDeclaration",
      exported,
      name,
      params,
      returnType,
      body,
      span: this.spanFromPositions(this.positionFromToken(startToken), body.span.end)
    };
  }

  parseFunctionParam() {
    const start = this.current();
    const name = this.parseIdentifier("Expected parameter name.");
    this.consume(TokenKind.Colon, "Expected ':' after parameter name.");
    const type = this.parseType();
    return {
      kind: "FunctionParam",
      name,
      type,
      span: this.spanFromPositions(this.positionFromToken(start), type.span.end)
    };
  }

  parseType() {
    const token = this.current();
    switch (token.kind) {
      case TokenKind.I32:
      case TokenKind.I64:
      case TokenKind.U32:
      case TokenKind.U64:
      case TokenKind.F64:
      case TokenKind.Bool:
        this.advance();
        return {
          kind: "PrimitiveType",
          name: token.text,
          span: this.spanFromToken(token)
        };
      case TokenKind.Identifier: {
        const name = this.parseIdentifier("Expected type name.");
        return {
          kind: "NamedType",
          name,
          span: name.span
        };
      }
      case TokenKind.Ptr: {
        const ptrToken = this.advance();
        this.consume(TokenKind.Less, "Expected '<' after 'ptr'.");
        const elementType = this.parseType();
        const greater = this.consume(TokenKind.Greater, "Expected '>' after pointer type.");
        return {
          kind: "PointerType",
          elementType,
          span: this.spanBetweenTokens(ptrToken, greater)
        };
      }
      default:
        this.error(token, "Expected type.");
        this.advance();
        return {
          kind: "ErrorType",
          span: this.spanFromToken(token)
        };
    }
  }

  parseBlockStatement() {
    const leftBrace = this.consume(TokenKind.LeftBrace, "Expected '{' before block.");
    const statements = [];

    while (!this.check(TokenKind.RightBrace) && !this.check(TokenKind.Eof)) {
      statements.push(this.parseStatement());
    }

    const rightBrace = this.consume(TokenKind.RightBrace, "Expected '}' after block.");
    return {
      kind: "BlockStatement",
      statements,
      span: this.spanBetweenTokens(leftBrace, rightBrace)
    };
  }

  parseStatement() {
    if (this.check(TokenKind.LeftBrace)) {
      return this.parseBlockStatement();
    }
    if (this.check(TokenKind.Let)) {
      return this.parseLetStatement();
    }
    if (this.check(TokenKind.Return)) {
      return this.parseReturnStatement();
    }
    if (this.check(TokenKind.If)) {
      return this.parseIfStatement();
    }
    if (this.check(TokenKind.While)) {
      return this.parseWhileStatement();
    }

    return this.parseAssignmentStatement();
  }

  parseLetStatement() {
    const letToken = this.consume(TokenKind.Let, "Expected 'let'.");
    const name = this.parseIdentifier("Expected local name.");
    this.consume(TokenKind.Colon, "Expected ':' after local name.");
    const type = this.parseType();
    this.consume(TokenKind.Equal, "Expected '=' after local type.");
    const initializer = this.parseExpression();
    const semicolon = this.consume(TokenKind.Semicolon, "Expected ';' after let statement.");

    return {
      kind: "LetStatement",
      name,
      type,
      initializer,
      span: this.spanBetweenTokens(letToken, semicolon)
    };
  }

  parseAssignmentStatement() {
    const start = this.current();
    const target = this.parseExpression();

    if (!this.match(TokenKind.Equal)) {
      this.error(this.current(), "Expected '=' in assignment statement.");
      this.synchronizeStatement();
      return {
        kind: "ErrorStatement",
        span: this.spanFromToken(start)
      };
    }

    const value = this.parseExpression();
    const semicolon = this.consume(TokenKind.Semicolon, "Expected ';' after assignment statement.");
    return {
      kind: "AssignmentStatement",
      target,
      value,
      span: this.spanFromPositions(target.span.start, this.endPositionFromToken(semicolon))
    };
  }

  parseReturnStatement() {
    const returnToken = this.consume(TokenKind.Return, "Expected 'return'.");
    const value = this.parseExpression();
    const semicolon = this.consume(TokenKind.Semicolon, "Expected ';' after return statement.");
    return {
      kind: "ReturnStatement",
      value,
      span: this.spanBetweenTokens(returnToken, semicolon)
    };
  }

  parseIfStatement() {
    const ifToken = this.consume(TokenKind.If, "Expected 'if'.");
    const condition = this.parseExpression();
    const thenBlock = this.parseBlockStatement();
    const elseBlock = this.match(TokenKind.Else) ? this.parseBlockStatement() : null;
    return {
      kind: "IfStatement",
      condition,
      thenBlock,
      elseBlock,
      span: this.spanFromPositions(this.positionFromToken(ifToken), (elseBlock ?? thenBlock).span.end)
    };
  }

  parseWhileStatement() {
    const whileToken = this.consume(TokenKind.While, "Expected 'while'.");
    const condition = this.parseExpression();
    const body = this.parseBlockStatement();
    return {
      kind: "WhileStatement",
      condition,
      body,
      span: this.spanFromPositions(this.positionFromToken(whileToken), body.span.end)
    };
  }

  parseExpression(minPrecedence = 1) {
    let left = this.parseUnaryExpression();

    while (true) {
      const operator = this.current();
      const precedence = binaryPrecedence(operator.kind);
      if (precedence < minPrecedence) {
        break;
      }

      this.advance();
      const right = this.parseExpression(precedence + 1);
      left = {
        kind: "BinaryExpression",
        operator: operator.text,
        left,
        right,
        span: this.spanFromPositions(left.span.start, right.span.end)
      };
    }

    return left;
  }

  parseUnaryExpression() {
    if (this.check(TokenKind.Bang) || this.check(TokenKind.Minus)) {
      const operator = this.advance();
      const operand = this.parseExpression(7);
      return {
        kind: "UnaryExpression",
        operator: operator.text,
        operand,
        span: this.spanFromPositions(this.positionFromToken(operator), operand.span.end)
      };
    }

    return this.parsePostfixExpression(this.parsePrimaryExpression());
  }

  parsePostfixExpression(base) {
    let expression = base;

    while (true) {
      if (this.match(TokenKind.LeftParen)) {
        const args = [];
        if (!this.check(TokenKind.RightParen)) {
          do {
            args.push(this.parseExpression());
          } while (this.match(TokenKind.Comma));
        }
        const rightParen = this.consume(TokenKind.RightParen, "Expected ')' after arguments.");
        expression = {
          kind: "CallExpression",
          callee: expression,
          args,
          span: this.spanFromPositions(expression.span.start, this.endPositionFromToken(rightParen))
        };
        continue;
      }

      if (this.match(TokenKind.Dot)) {
        const field = this.parseIdentifier("Expected field name after '.'.");
        expression = {
          kind: "FieldExpression",
          object: expression,
          field,
          span: this.spanFromPositions(expression.span.start, field.span.end)
        };
        continue;
      }

      if (this.match(TokenKind.LeftBracket)) {
        const index = this.parseExpression();
        const rightBracket = this.consume(TokenKind.RightBracket, "Expected ']' after index expression.");
        expression = {
          kind: "IndexExpression",
          object: expression,
          index,
          span: this.spanFromPositions(expression.span.start, this.endPositionFromToken(rightBracket))
        };
        continue;
      }

      return expression;
    }
  }

  parsePrimaryExpression() {
    const token = this.current();

    if (this.match(TokenKind.Integer)) {
      return {
        kind: "IntegerLiteral",
        text: token.text,
        span: this.spanFromToken(token)
      };
    }

    if (this.match(TokenKind.Float)) {
      return {
        kind: "FloatLiteral",
        text: token.text,
        span: this.spanFromToken(token)
      };
    }

    if (this.match(TokenKind.True) || this.match(TokenKind.False)) {
      return {
        kind: "BoolLiteral",
        value: token.kind === TokenKind.True,
        span: this.spanFromToken(token)
      };
    }

    if (this.match(TokenKind.Identifier)) {
      return {
        kind: "IdentifierExpression",
        name: token.text,
        span: this.spanFromToken(token)
      };
    }

    if (this.match(TokenKind.LeftParen)) {
      const expression = this.parseExpression();
      const rightParen = this.consume(TokenKind.RightParen, "Expected ')' after expression.");
      return {
        kind: "ParenthesizedExpression",
        expression,
        span: this.spanFromPositions(this.positionFromToken(token), this.endPositionFromToken(rightParen))
      };
    }

    this.error(token, "Expected expression.");
    this.advance();
    return {
      kind: "ErrorExpression",
      span: this.spanFromToken(token)
    };
  }

  parseIdentifier(message) {
    const token = this.consume(TokenKind.Identifier, message);
    return {
      kind: "Identifier",
      name: token.text,
      span: this.spanFromToken(token)
    };
  }

  match(kind) {
    if (!this.check(kind)) {
      return false;
    }
    this.advance();
    return true;
  }

  consume(kind, message) {
    if (this.check(kind)) {
      return this.advance();
    }

    const token = this.current();
    this.error(token, message);
    return {
      kind,
      text: "",
      line: token.line,
      column: token.column,
      start: token.start,
      end: token.start
    };
  }

  check(kind) {
    return this.current().kind === kind;
  }

  advance() {
    const token = this.current();
    if (!this.check(TokenKind.Eof)) {
      this.index += 1;
    }
    return token;
  }

  previous() {
    return this.tokens[Math.max(0, this.index - 1)] ?? this.current();
  }

  current() {
    return this.tokens[this.index] ?? this.tokens[this.tokens.length - 1];
  }

  error(token, message) {
    this.diagnostics.push(errorAt(this.source, this.spanFromToken(token), "CK1001", message));
  }

  synchronizeStatement() {
    while (!this.check(TokenKind.Eof)) {
      if (this.match(TokenKind.Semicolon)) {
        return;
      }
      if (this.check(TokenKind.RightBrace)) {
        return;
      }
      this.advance();
    }
  }

  spanFromToken(token) {
    return {
      start: this.positionFromToken(token),
      end: this.endPositionFromToken(token)
    };
  }

  spanBetweenTokens(start, end) {
    return {
      start: this.positionFromToken(start),
      end: this.endPositionFromToken(end)
    };
  }

  spanFromPositions(start, end) {
    return { start, end };
  }

  positionFromToken(token) {
    return {
      offset: token.start,
      line: token.line,
      column: token.column
    };
  }

  endPositionFromToken(token) {
    return {
      offset: token.end,
      line: token.line,
      column: token.column + token.text.length
    };
  }
}

function binaryPrecedence(kind) {
  switch (kind) {
    case TokenKind.PipePipe:
      return 1;
    case TokenKind.AmpAmp:
      return 2;
    case TokenKind.EqualEqual:
    case TokenKind.BangEqual:
      return 3;
    case TokenKind.Less:
    case TokenKind.LessEqual:
    case TokenKind.Greater:
    case TokenKind.GreaterEqual:
      return 4;
    case TokenKind.Plus:
    case TokenKind.Minus:
      return 5;
    case TokenKind.Star:
    case TokenKind.Slash:
    case TokenKind.Percent:
      return 6;
    default:
      return 0;
  }
}

const unknownType = Object.freeze({ kind: "unknown" });
const integerLiteralType = Object.freeze({ kind: "integerLiteral" });
const integerPrimitiveNames = Object.freeze(["i32", "i64", "u32", "u64"]);
const indexIntegerPrimitiveNames = Object.freeze(["i32", "u32"]);
const floatPrimitiveNames = Object.freeze(["f64"]);

function primitiveType(name) {
  return { kind: "primitive", name };
}

function pointerType(elementType) {
  return { kind: "pointer", elementType };
}

function structType(name) {
  return { kind: "struct", name };
}

function isUnknown(type) {
  return type.kind === "unknown";
}

function isBool(type) {
  return type.kind === "primitive" && type.name === "bool";
}

function isIntegerPrimitiveName(name) {
  return integerPrimitiveNames.includes(name);
}

function isFloatPrimitiveName(name) {
  return floatPrimitiveNames.includes(name);
}

function isIntegerPrimitive(type) {
  return type.kind === "primitive" && isIntegerPrimitiveName(type.name);
}

function isFloatType(type) {
  return type.kind === "primitive" && isFloatPrimitiveName(type.name);
}

function isInteger(type) {
  return type.kind === "integerLiteral" || isIntegerPrimitive(type);
}

function isNumericType(type) {
  return isInteger(type) || isFloatType(type);
}

function isIndexInteger(type) {
  return type.kind === "integerLiteral" || (type.kind === "primitive" && indexIntegerPrimitiveNames.includes(type.name));
}

function sameType(left, right) {
  if (left.kind === "unknown" || right.kind === "unknown") {
    return true;
  }

  if (left.kind === "integerLiteral" && isInteger(right)) {
    return true;
  }

  if (right.kind === "integerLiteral" && isInteger(left)) {
    return true;
  }

  if (left.kind !== right.kind) {
    return false;
  }

  switch (left.kind) {
    case "primitive":
      return right.kind === "primitive" && left.name === right.name;
    case "pointer":
      return right.kind === "pointer" && sameType(left.elementType, right.elementType);
    case "struct":
      return right.kind === "struct" && left.name === right.name;
    case "integerLiteral":
      return true;
    default:
      return false;
  }
}

function canAssign(target, value) {
  return sameType(target, value);
}

function materializeIntegerLiteral(type, fallback = primitiveType("i32")) {
  return type.kind === "integerLiteral" ? fallback : type;
}

function typeToString(type) {
  switch (type.kind) {
    case "primitive":
      return type.name;
    case "pointer":
      return `ptr<${typeToString(type.elementType)}>`;
    case "struct":
      return type.name;
    case "integerLiteral":
      return "i32";
    case "unknown":
      return "unknown";
    default:
      return "unknown";
  }
}

export class SymbolTable {
  constructor() {
    this.structs = new Map();
    this.functions = new Map();
  }
}

export class Scope {
  constructor(parent = null) {
    this.parent = parent;
    this.variables = new Map();
  }

  declare(variable) {
    if (this.variables.has(variable.name)) {
      return false;
    }

    this.variables.set(variable.name, variable);
    return true;
  }

  lookup(name) {
    return this.variables.get(name) ?? this.parent?.lookup(name) ?? null;
  }
}

const compilerBuiltins = new Map(
  [
    {
      name: "i32_to_f64",
      params: [primitiveType("i32")],
      returnType: primitiveType("f64")
    },
    {
      name: "u32_to_f64",
      params: [primitiveType("u32")],
      returnType: primitiveType("f64")
    }
  ].map((builtin) => [builtin.name, builtin])
);

export function check(source) {
  const parseResult = parse(source);
  const checker = new Checker(source, parseResult.ast, [...parseResult.diagnostics]);
  return checker.check();
}

class Checker {
  constructor(source, program, diagnostics) {
    this.source = source;
    this.program = program;
    this.diagnostics = diagnostics;
    this.symbols = new SymbolTable();
    this.expressionTypes = new Map();
    this.localTypes = new Map();
  }

  check() {
    this.collectStructNames();
    this.collectStructFields();
    this.collectFunctionSignatures();
    this.checkFunctionBodies();

    const typedAst = {
      program: this.program,
      expressionTypes: this.expressionTypes
    };

    return {
      ast: this.program,
      typedAst,
      checkedProgram: createCheckedProgram(this.program, this.symbols, this.expressionTypes, this.localTypes),
      diagnostics: this.diagnostics,
      symbols: this.symbols
    };
  }

  collectStructNames() {
    for (const declaration of this.program.declarations) {
      if (declaration.kind !== "StructDeclaration") {
        continue;
      }

      const name = declaration.name.name;
      if (this.symbols.structs.has(name)) {
        this.error(declaration.name.span, `Duplicate struct '${name}'.`);
        continue;
      }

      this.symbols.structs.set(name, {
        name,
        declaration,
        fields: new Map()
      });
    }
  }

  collectStructFields() {
    for (const declaration of this.program.declarations) {
      if (declaration.kind !== "StructDeclaration") {
        continue;
      }

      const symbol = this.symbols.structs.get(declaration.name.name);
      if (!symbol || symbol.declaration !== declaration) {
        continue;
      }

      for (const field of declaration.fields) {
        if (symbol.fields.has(field.name.name)) {
          this.error(field.name.span, `Duplicate field '${field.name.name}' in struct '${symbol.name}'.`);
          continue;
        }

        symbol.fields.set(field.name.name, this.resolveType(field.type));
      }
    }
  }

  collectFunctionSignatures() {
    for (const declaration of this.program.declarations) {
      if (declaration.kind !== "FunctionDeclaration") {
        continue;
      }

      const name = declaration.name.name;
      if (compilerBuiltins.has(name)) {
        this.error(declaration.name.span, `Cannot define reserved compiler builtin '${name}'.`);
        continue;
      }

      if (this.symbols.functions.has(name)) {
        this.error(declaration.name.span, `Duplicate function '${name}'.`);
        continue;
      }

      this.symbols.functions.set(name, {
        name,
        declaration,
        params: declaration.params.map((param) => this.resolveType(param.type)),
        returnType: this.resolveType(declaration.returnType)
      });
    }
  }

  checkFunctionBodies() {
    for (const declaration of this.program.declarations) {
      if (declaration.kind !== "FunctionDeclaration") {
        continue;
      }

      const functionSymbol = this.symbols.functions.get(declaration.name.name);
      if (!functionSymbol || functionSymbol.declaration !== declaration) {
        continue;
      }

      this.checkFunctionBody(declaration, functionSymbol);
    }
  }

  checkFunctionBody(declaration, functionSymbol) {
    const scope = new Scope();

    declaration.params.forEach((param, index) => {
      const name = param.name.name;
      const type = functionSymbol.params[index] ?? unknownType;
      if (!scope.declare({ name, type })) {
        this.error(param.name.span, `Duplicate variable '${name}'.`);
      }
    });

    this.checkBlock(declaration.body, scope, functionSymbol.returnType, false);
    if (!this.blockDefinitelyReturns(declaration.body)) {
      this.error(declaration.body.span, `Missing return in function '${declaration.name.name}'.`);
    }
  }

  checkBlock(block, parentScope, returnType, createScope) {
    const scope = createScope ? new Scope(parentScope) : parentScope;

    for (const statement of block.statements) {
      this.checkStatement(statement, scope, returnType);
    }
  }

  checkStatement(statement, scope, returnType) {
    switch (statement.kind) {
      case "BlockStatement":
        this.checkBlock(statement, scope, returnType, true);
        return;
      case "LetStatement":
        this.checkLetStatement(statement, scope);
        return;
      case "AssignmentStatement":
        this.checkAssignmentStatement(statement, scope);
        return;
      case "ReturnStatement":
        this.checkReturnStatement(statement, scope, returnType);
        return;
      case "IfStatement":
        this.checkIfStatement(statement, scope, returnType);
        return;
      case "WhileStatement":
        this.checkWhileStatement(statement, scope, returnType);
        return;
      case "ErrorStatement":
        return;
      default:
        return;
    }
  }

  checkLetStatement(statement, scope) {
    const declaredType = this.resolveType(statement.type);
    this.localTypes.set(statement, declaredType);

    if (!scope.declare({ name: statement.name.name, type: declaredType })) {
      this.error(statement.name.span, `Duplicate variable '${statement.name.name}'.`);
    }

    const initializerType = this.checkExpression(statement.initializer, scope, declaredType);
    if (!isUnknown(declaredType) && !isUnknown(initializerType) && !canAssign(declaredType, initializerType)) {
      this.error(
        statement.initializer.span,
        `Cannot initialize '${statement.name.name}': expected ${typeToString(declaredType)} but got ${typeToString(initializerType)}.`
      );
    }
  }

  checkAssignmentStatement(statement, scope) {
    if (!this.isAssignableExpression(statement.target)) {
      this.error(statement.target.span, "Invalid assignment target.");
    }

    const targetType = this.checkExpression(statement.target, scope);
    const valueType = this.checkExpression(statement.value, scope, targetType);

    if (!isUnknown(targetType) && !isUnknown(valueType) && !canAssign(targetType, valueType)) {
      this.error(statement.value.span, `Cannot assign ${typeToString(valueType)} to ${typeToString(targetType)}.`);
    }
  }

  checkReturnStatement(statement, scope, returnType) {
    const valueType = this.checkExpression(statement.value, scope, returnType);
    if (!isUnknown(returnType) && !isUnknown(valueType) && !canAssign(returnType, valueType)) {
      this.error(statement.value.span, `Return type mismatch: expected ${typeToString(returnType)} but got ${typeToString(valueType)}.`);
    }
  }

  checkIfStatement(statement, scope, returnType) {
    const conditionType = materializeIntegerLiteral(this.checkExpression(statement.condition, scope));
    if (!isUnknown(conditionType) && !isBool(conditionType)) {
      this.error(statement.condition.span, `If condition must be bool, got ${typeToString(conditionType)}.`);
    }

    this.checkBlock(statement.thenBlock, scope, returnType, true);
    if (statement.elseBlock) {
      this.checkBlock(statement.elseBlock, scope, returnType, true);
    }
  }

  checkWhileStatement(statement, scope, returnType) {
    const conditionType = materializeIntegerLiteral(this.checkExpression(statement.condition, scope));
    if (!isUnknown(conditionType) && !isBool(conditionType)) {
      this.error(statement.condition.span, `While condition must be bool, got ${typeToString(conditionType)}.`);
    }

    this.checkBlock(statement.body, scope, returnType, true);
  }

  checkExpression(expression, scope, expectedType) {
    switch (expression.kind) {
      case "IdentifierExpression":
        return this.checkIdentifierExpression(expression, scope);
      case "IntegerLiteral":
        return this.checkIntegerLiteral(expression, expectedType);
      case "FloatLiteral":
        return this.checkFloatLiteral(expression);
      case "BoolLiteral":
        return this.recordExpressionType(expression, { kind: "primitive", name: "bool" });
      case "UnaryExpression":
        return this.checkUnaryExpression(expression, scope, expectedType);
      case "BinaryExpression":
        return this.checkBinaryExpression(expression, scope, expectedType);
      case "CallExpression":
        return this.checkCallExpression(expression, scope);
      case "FieldExpression":
        return this.checkFieldExpression(expression, scope);
      case "IndexExpression":
        return this.checkIndexExpression(expression, scope);
      case "ParenthesizedExpression":
        return this.checkParenthesizedExpression(expression, scope, expectedType);
      case "ErrorExpression":
        return this.recordExpressionType(expression, unknownType);
      default:
        return this.recordExpressionType(expression, unknownType);
    }
  }

  checkIdentifierExpression(expression, scope) {
    const symbol = scope.lookup(expression.name);
    if (!symbol) {
      this.error(expression.span, `Unknown variable '${expression.name}'.`);
      return this.recordExpressionType(expression, unknownType);
    }

    return this.recordExpressionType(expression, symbol.type);
  }

  checkIntegerLiteral(expression, expectedType) {
    const type = expectedType && isInteger(expectedType) ? expectedType : integerLiteralType;
    return this.recordExpressionType(expression, type);
  }

  checkFloatLiteral(expression) {
    return this.recordExpressionType(expression, primitiveType("f64"));
  }

  checkUnaryExpression(expression, scope, expectedType) {
    if (expression.operator === "!") {
      const operandType = materializeIntegerLiteral(this.checkExpression(expression.operand, scope));
      if (!isUnknown(operandType) && !isBool(operandType)) {
        this.error(expression.operand.span, `Unary operator '!' requires bool operand, got ${typeToString(operandType)}.`);
      }

      return this.recordExpressionType(expression, primitiveType("bool"));
    }

    const fallback = integerLiteralFallback(expectedType);
    const operandType = materializeIntegerLiteral(this.checkExpression(expression.operand, scope, fallback), fallback);
    if (!isUnknown(operandType) && !isNumericType(operandType)) {
      this.error(expression.operand.span, `Unary operator '-' requires integer operand, got ${typeToString(operandType)}.`);
      return this.recordExpressionType(expression, unknownType);
    }

    return this.recordExpressionType(expression, materializeIntegerLiteral(operandType, fallback));
  }

  checkBinaryExpression(expression, scope, expectedType) {
    if (isArithmeticOperator(expression.operator)) {
      return this.checkArithmeticExpression(expression, scope, expectedType);
    }

    if (isComparisonOperator(expression.operator)) {
      return this.checkComparisonExpression(expression, scope);
    }

    if (expression.operator === "&&" || expression.operator === "||") {
      const leftType = materializeIntegerLiteral(this.checkExpression(expression.left, scope));
      const rightType = materializeIntegerLiteral(this.checkExpression(expression.right, scope));
      if (!isUnknown(leftType) && !isBool(leftType)) {
        this.error(expression.left.span, `Logical operator '${expression.operator}' requires bool operands.`);
      }
      if (!isUnknown(rightType) && !isBool(rightType)) {
        this.error(expression.right.span, `Logical operator '${expression.operator}' requires bool operands.`);
      }
      return this.recordExpressionType(expression, primitiveType("bool"));
    }

    return this.recordExpressionType(expression, unknownType);
  }

  checkArithmeticExpression(expression, scope, expectedType) {
    const leftRaw = this.checkExpression(expression.left, scope);
    const rightRaw = this.checkExpression(expression.right, scope);
    const fallback = integerLiteralFallback(expectedType);
    const leftType = materializeIntegerLiteral(leftRaw, rightRaw.kind === "integerLiteral" ? fallback : integerLiteralFallback(rightRaw));
    const rightType = materializeIntegerLiteral(rightRaw, integerLiteralFallback(leftType));

    if (expression.operator === "%" && (isFloatType(leftType) || isFloatType(rightType))) {
      this.error(expression.span, "Arithmetic operator '%' does not support f64 operands.");
      return this.recordExpressionType(expression, unknownType);
    }

    if (!isUnknown(leftType) && !isUnknown(rightType) && (!isNumericType(leftType) || !isNumericType(rightType) || !sameType(leftType, rightType))) {
      this.error(expression.span, `Arithmetic operator '${expression.operator}' requires integer operands of the same type.`);
      return this.recordExpressionType(expression, unknownType);
    }

    this.expressionTypes.set(expression.left, leftType);
    this.expressionTypes.set(expression.right, rightType);
    return this.recordExpressionType(expression, materializeIntegerLiteral(leftType, fallback));
  }

  checkComparisonExpression(expression, scope) {
    const leftRaw = this.checkExpression(expression.left, scope);
    const rightRaw = this.checkExpression(expression.right, scope);
    const leftType = materializeIntegerLiteral(leftRaw, rightRaw.kind === "integerLiteral" ? primitiveType("i32") : integerLiteralFallback(rightRaw));
    const rightType = materializeIntegerLiteral(rightRaw, integerLiteralFallback(leftType));

    const valid =
      expression.operator === "==" || expression.operator === "!="
        ? sameType(leftType, rightType)
        : isNumericType(leftType) && isNumericType(rightType) && sameType(leftType, rightType);

    if (!isUnknown(leftType) && !isUnknown(rightType) && !valid) {
      this.error(expression.span, `Comparison operator '${expression.operator}' requires compatible operands.`);
    }

    this.expressionTypes.set(expression.left, leftType);
    this.expressionTypes.set(expression.right, rightType);
    return this.recordExpressionType(expression, primitiveType("bool"));
  }

  checkCallExpression(expression, scope) {
    if (expression.callee.kind !== "IdentifierExpression") {
      this.error(expression.callee.span, "Can only call functions by name.");
      for (const arg of expression.args) {
        this.checkExpression(arg, scope);
      }
      return this.recordExpressionType(expression, unknownType);
    }

    const builtin = compilerBuiltins.get(expression.callee.name);
    if (builtin) {
      return this.checkCompilerBuiltinCall(expression, scope, builtin);
    }

    const functionSymbol = this.symbols.functions.get(expression.callee.name);
    if (!functionSymbol) {
      this.error(expression.callee.span, `Unknown function '${expression.callee.name}'.`);
      for (const arg of expression.args) {
        this.checkExpression(arg, scope);
      }
      return this.recordExpressionType(expression, unknownType);
    }

    this.recordExpressionType(expression.callee, functionSymbol.returnType);

    if (expression.args.length !== functionSymbol.params.length) {
      this.error(
        expression.span,
        `Function '${functionSymbol.name}' expects ${functionSymbol.params.length} argument${functionSymbol.params.length === 1 ? "" : "s"} but got ${expression.args.length}.`
      );
    }

    expression.args.forEach((arg, index) => {
      const expected = functionSymbol.params[index];
      const argType = this.checkExpression(arg, scope, expected);
      if (expected && !isUnknown(expected) && !isUnknown(argType) && !canAssign(expected, argType)) {
        this.error(arg.span, `Argument ${index + 1} of function '${functionSymbol.name}' expects ${typeToString(expected)} but got ${typeToString(argType)}.`);
      }
    });

    return this.recordExpressionType(expression, functionSymbol.returnType);
  }

  checkCompilerBuiltinCall(expression, scope, builtin) {
    this.recordExpressionType(expression.callee, builtin.returnType);

    if (expression.args.length !== builtin.params.length) {
      this.error(
        expression.span,
        `Compiler builtin '${builtin.name}' expects ${builtin.params.length} argument${builtin.params.length === 1 ? "" : "s"} but got ${expression.args.length}.`
      );
    }

    expression.args.forEach((arg, index) => {
      const expected = builtin.params[index];
      const argType = this.checkExpression(arg, scope, expected);
      if (expected && !isUnknown(expected) && !isUnknown(argType) && !canAssign(expected, argType)) {
        this.error(arg.span, `Argument ${index + 1} of compiler builtin '${builtin.name}' expects ${typeToString(expected)} but got ${typeToString(argType)}.`);
      }
    });

    return this.recordExpressionType(expression, builtin.returnType);
  }

  checkFieldExpression(expression, scope) {
    const objectType = this.checkExpression(expression.object, scope);
    if (objectType.kind !== "struct") {
      if (!isUnknown(objectType)) {
        this.error(expression.object.span, `Field access requires struct value, got ${typeToString(objectType)}.`);
      }
      return this.recordExpressionType(expression, unknownType);
    }

    const structSymbol = this.symbols.structs.get(objectType.name);
    const fieldType = structSymbol?.fields.get(expression.field.name);
    if (!fieldType) {
      this.error(expression.field.span, `Struct '${objectType.name}' has no field '${expression.field.name}'.`);
      return this.recordExpressionType(expression, unknownType);
    }

    return this.recordExpressionType(expression, fieldType);
  }

  checkIndexExpression(expression, scope) {
    const objectType = this.checkExpression(expression.object, scope);
    const indexType = materializeIntegerLiteral(this.checkExpression(expression.index, scope), primitiveType("i32"));

    if (!isUnknown(indexType) && !isIndexInteger(indexType)) {
      this.error(expression.index.span, `Index expression requires i32 or u32 index, got ${typeToString(indexType)}.`);
    }

    if (objectType.kind !== "pointer") {
      if (!isUnknown(objectType)) {
        this.error(expression.object.span, `Index access requires pointer value, got ${typeToString(objectType)}.`);
      }
      return this.recordExpressionType(expression, unknownType);
    }

    return this.recordExpressionType(expression, objectType.elementType);
  }

  checkParenthesizedExpression(expression, scope, expectedType) {
    const type = this.checkExpression(expression.expression, scope, expectedType);
    return this.recordExpressionType(expression, type);
  }

  resolveType(typeNode) {
    switch (typeNode.kind) {
      case "PrimitiveType":
        return primitiveType(typeNode.name);
      case "PointerType":
        return pointerType(this.resolveType(typeNode.elementType));
      case "NamedType": {
        const name = typeNode.name.name;
        if (!this.symbols.structs.has(name)) {
          this.error(typeNode.name.span, `Unknown type '${name}'.`);
          return unknownType;
        }
        return structType(name);
      }
      case "ErrorType":
        return unknownType;
      default:
        return unknownType;
    }
  }

  recordExpressionType(expression, type) {
    this.expressionTypes.set(expression, type);
    return type;
  }

  blockDefinitelyReturns(block) {
    const lastStatement = block.statements.at(-1);
    if (!lastStatement) {
      return false;
    }

    return this.statementDefinitelyReturns(lastStatement);
  }

  statementDefinitelyReturns(statement) {
    switch (statement.kind) {
      case "ReturnStatement":
        return true;
      case "BlockStatement":
        return this.blockDefinitelyReturns(statement);
      case "IfStatement":
        return Boolean(
          statement.elseBlock &&
            this.blockDefinitelyReturns(statement.thenBlock) &&
            this.blockDefinitelyReturns(statement.elseBlock)
        );
      case "LetStatement":
      case "AssignmentStatement":
      case "WhileStatement":
      case "ErrorStatement":
        return false;
      default:
        return false;
    }
  }

  isAssignableExpression(expression) {
    return expression.kind === "IdentifierExpression" || expression.kind === "FieldExpression" || expression.kind === "IndexExpression";
  }

  error(span, message) {
    this.diagnostics.push(errorAt(this.source, span, checkerDiagnosticCode(message), message));
  }
}

export function getExprType(checkedProgram, expression) {
  return checkedProgram.types.get(expression);
}

export function getLetType(checkedProgram, statement) {
  return checkedProgram.localTypes.get(statement);
}

export function getStructInfo(checkedProgram, name) {
  return checkedProgram.structMap.get(name);
}

export function getFieldInfo(checkedProgram, structName, fieldName) {
  return checkedProgram.structMap.get(structName)?.fieldMap.get(fieldName);
}

export function getFunctionInfo(checkedProgram, name) {
  return checkedProgram.functionMap.get(name);
}

function createCheckedProgram(ast, symbols, expressionTypes, localTypes) {
  const structs = [...symbols.structs.values()].map(toStructInfo);
  const functions = [...symbols.functions.values()].map(toFunctionInfo);

  return {
    ast,
    symbols,
    types: new Map(expressionTypes),
    localTypes: new Map(localTypes),
    structs,
    functions,
    structMap: new Map(structs.map((struct) => [struct.name, struct])),
    functionMap: new Map(functions.map((func) => [func.name, func]))
  };
}

function toStructInfo(symbol) {
  const fields = [...symbol.fields.entries()].map(([name, type]) => {
    const declaration = symbol.declaration.fields.find((field) => field.name.name === name);
    if (!declaration) {
      throw new Error(`Checker invariant violation: missing declaration for field '${symbol.name}.${name}'.`);
    }
    return { name, type, declaration };
  });

  return {
    name: symbol.name,
    declaration: symbol.declaration,
    fields,
    fieldMap: new Map(fields.map((field) => [field.name, field]))
  };
}

function toFunctionInfo(symbol) {
  const params = symbol.declaration.params.map((param, index) => ({
    name: param.name.name,
    type: symbol.params[index] ?? unknownType,
    declaration: param
  }));

  return {
    name: symbol.name,
    exported: symbol.declaration.exported,
    declaration: symbol.declaration,
    params,
    returnType: symbol.returnType
  };
}

export function emitCHeader(checked, options = {}) {
  assertCanEmitC(checked);
  const overflowMode = resolveCOverflowMode(options);
  const lines = ["#pragma once", "", "#include <stdint.h>", "#include <stdbool.h>"];

  if (overflowMode === "checked") {
    lines.push("#include <stddef.h>");
  }

  lines.push(
    "",
    "#if defined(_WIN32) || defined(__CYGWIN__)",
    "  #ifdef CK_BUILD_DLL",
    "    #define CK_API __declspec(dllexport)",
    "  #else",
    "    #define CK_API __declspec(dllimport)",
    "  #endif",
    "#else",
    "  #define CK_API __attribute__((visibility(\"default\")))",
    "#endif"
  );

  if (overflowMode === "checked") {
    lines.push(
      "",
      "typedef int32_t CK_Status;",
      "",
      "#define CK_OK ((CK_Status)0)",
      "#define CK_ERR_OVERFLOW ((CK_Status)1)",
      "#define CK_ERR_DIV_BY_ZERO ((CK_Status)2)",
      "#define CK_ERR_NULL_POINTER ((CK_Status)3)"
    );
  }

  lines.push("", "#ifdef __cplusplus", "extern \"C\" {", "#endif");

  for (const declaration of checked.ast.declarations) {
    if (declaration.kind === "StructDeclaration") {
      lines.push("", emitCStructTypedef(declaration));
    }
  }

  for (const declaration of checked.ast.declarations) {
    if (declaration.kind === "FunctionDeclaration" && declaration.exported) {
      const signature = overflowMode === "checked" ? emitCheckedCFunctionSignature(declaration) : emitCFunctionSignature(declaration);
      lines.push("", `CK_API ${signature};`);
    }
  }

  lines.push("", "#ifdef __cplusplus", "}", "#endif");
  return `${lines.join("\n")}\n`;
}

export function emitCSource(checked, options) {
  return emitRustCSource(checked, options);
}

export function emitCFiles(checked, options) {
  const headerText = emitCHeader(checked, { overflowMode: options.overflowMode });
  const sourceText = emitCSource(checked, {
    headerFileName: options.headerFileName,
    overflowMode: options.overflowMode,
    optLevel: options.optLevel,
    mirDebug: options.mirDebug,
    writeDebug: options.writeDebug
  });

  mkdirSync(dirname(options.cFile), { recursive: true });
  mkdirSync(dirname(options.headerFile), { recursive: true });
  writeFileAtomic(options.headerFile, headerText);
  writeFileAtomic(options.cFile, sourceText);
}

export function buildSharedLibrary(checked, options) {
  emitCFiles(checked, options);

  const clangProbe = options.runCommand("clang", ["--version"]);
  if (isMissingCommand(clangProbe)) {
    return {
      ok: false,
      outputPath: sharedLibraryOutputPath(options.outputPath, options.platform),
      message: "clang was not found. Install clang and make sure it is available on PATH."
    };
  }

  if (clangProbe.status !== 0) {
    return {
      ok: false,
      outputPath: sharedLibraryOutputPath(options.outputPath, options.platform),
      message: clangProbe.stderr || "Unable to run clang --version."
    };
  }

  const outputPath = sharedLibraryOutputPath(options.outputPath, options.platform);
  mkdirSync(dirname(outputPath), { recursive: true });
  const result = options.runCommand("clang", clangArgs(options.cFile, outputPath, options.platform));

  if (isMissingCommand(result)) {
    return {
      ok: false,
      outputPath,
      message: "clang was not found. Install clang and make sure it is available on PATH."
    };
  }

  if (result.status !== 0) {
    return {
      ok: false,
      outputPath,
      message: result.stderr || `clang failed with exit code ${result.status ?? "unknown"}.`
    };
  }

  return { ok: true, outputPath };
}

export function sharedLibraryOutputPath(outputPath, platform) {
  if (/\.(so|dylib|dll)$/i.test(outputPath)) {
    return outputPath;
  }

  switch (platform) {
    case "darwin":
      return `${outputPath}.dylib`;
    case "win32":
      return `${outputPath}.dll`;
    default:
      return `${outputPath}.so`;
  }
}

function assertCanEmitC(checked) {
  if (checked.diagnostics.length > 0) {
    throw new Error("Cannot emit C for a program with diagnostics.");
  }
}

function resolveCOverflowMode(options = {}) {
  return options.overflowMode ?? "unchecked";
}

function resolveCOptLevel(options = {}) {
  return options.optLevel ?? 1;
}

function emitCFunctionSignature(functionDeclaration) {
  const returnType = emitCType(functionDeclaration.returnType);
  const params = functionDeclaration.params.map((param) => `${emitCType(param.type)} ${param.name.name}`).join(", ");
  return `${returnType} ${functionDeclaration.name.name}(${params})`;
}

function emitCheckedCFunctionSignature(functionDeclaration) {
  const params = functionDeclaration.params.map((param) => `${emitCType(param.type)} ${param.name.name}`);
  params.push(`${emitCType(functionDeclaration.returnType)}* ck_return`);
  return `CK_Status ${functionDeclaration.name.name}(${params.join(", ")})`;
}

function emitCType(type) {
  switch (type.kind) {
    case "PrimitiveType":
      return emitCPrimitiveType(type.name);
    case "PointerType":
      return `${emitCType(type.elementType)}*`;
    case "NamedType":
      return type.name.name;
    case "ErrorType":
      throw new Error("Cannot emit C for unresolved type.");
    default:
      throw new Error("Cannot emit C for unresolved type.");
  }
}

function emitCPrimitiveType(name) {
  switch (name) {
    case "i32":
      return "int32_t";
    case "i64":
      return "int64_t";
    case "u32":
      return "uint32_t";
    case "u64":
      return "uint64_t";
    case "f64":
      return "double";
    case "bool":
      return "bool";
    default:
      throw new Error(`Unsupported C primitive type '${name}'.`);
  }
}

function emitCStructTypedef(structDeclaration) {
  const lines = [`typedef struct ${structDeclaration.name.name} {`];
  for (const field of structDeclaration.fields) {
    lines.push(`  ${emitCType(field.type)} ${field.name.name};`);
  }
  lines.push(`} ${structDeclaration.name.name};`);
  return lines.join("\n");
}

function emitRustCSource(checked, options = {}) {
  assertCanEmitC(checked);

  const tempRoot = mkdtempSync(join(tmpdir(), "calckernel-c-api-"));
  try {
    const sourceFile = join(tempRoot, "input.ck");
    const cFile = join(tempRoot, "out.c");
    const headerFile = join(tempRoot, options.headerFileName ?? "out.h");
    writeFileSync(sourceFile, printProgram(checked.ast));

    const args = ["emit-c", sourceFile, "--out", cFile, "--header", headerFile, "--overflow", resolveCOverflowMode(options), "--opt-level", String(resolveCOptLevel(options))];
    if (options.mirDebug?.printPassPipeline) {
      args.push("--print-pass-pipeline");
    }
    if (options.mirDebug?.printMirBeforeOpt) {
      args.push("--print-mir-before-opt");
    }
    if (options.mirDebug?.printMirAfterOpt) {
      args.push("--print-mir-after-opt");
    }

    const result = spawnSync(resolveCkcBinary(), args, { encoding: "utf8" });
    if (options.writeDebug && result.stderr) {
      options.writeDebug(result.stderr);
    }
    if (result.error) {
      throw result.error;
    }
    if (result.status !== 0) {
      throw new Error(result.stderr || result.stdout || `ckc emit-c failed with exit code ${result.status ?? "unknown"}.`);
    }

    return readFileSync(cFile, "utf8");
  } finally {
    rmSync(tempRoot, { recursive: true, force: true });
  }
}

function printProgram(program) {
  return `${program.declarations.map(printDeclaration).join("\n\n")}\n`;
}

function printDeclaration(declaration) {
  switch (declaration.kind) {
    case "StructDeclaration":
      return `struct ${declaration.name.name} {\n${declaration.fields.map((field) => `  ${field.name.name}: ${printType(field.type)};`).join("\n")}\n}`;
    case "FunctionDeclaration": {
      const prefix = declaration.exported ? "export " : "";
      const params = declaration.params.map((param) => `${param.name.name}: ${printType(param.type)}`).join(", ");
      return `${prefix}fn ${declaration.name.name}(${params}) -> ${printType(declaration.returnType)} ${printBlock(declaration.body, 0)}`;
    }
    default:
      throw new Error(`Cannot print declaration kind '${declaration.kind}'.`);
  }
}

function printType(type) {
  switch (type.kind) {
    case "PrimitiveType":
      return type.name;
    case "PointerType":
      return `ptr<${printType(type.elementType)}>`;
    case "NamedType":
      return type.name.name;
    case "ErrorType":
      throw new Error("Cannot print unresolved type.");
    default:
      throw new Error(`Cannot print type kind '${type.kind}'.`);
  }
}

function printBlock(block, indent) {
  const pad = " ".repeat(indent);
  const bodyPad = " ".repeat(indent + 2);
  if (block.statements.length === 0) {
    return "{\n" + pad + "}";
  }
  return `{\n${block.statements.map((statement) => `${bodyPad}${printStatement(statement, indent + 2)}`).join("\n")}\n${pad}}`;
}

function printStatement(statement, indent) {
  switch (statement.kind) {
    case "BlockStatement":
      return printBlock(statement, indent);
    case "LetStatement":
      return `let ${statement.name.name}: ${printType(statement.type)} = ${printExpression(statement.initializer)};`;
    case "AssignmentStatement":
      return `${printExpression(statement.target)} = ${printExpression(statement.value)};`;
    case "ReturnStatement":
      return `return ${printExpression(statement.value)};`;
    case "IfStatement":
      return `if ${printExpression(statement.condition)} ${printBlock(statement.thenBlock, indent)}${statement.elseBlock ? ` else ${printBlock(statement.elseBlock, indent)}` : ""}`;
    case "WhileStatement":
      return `while ${printExpression(statement.condition)} ${printBlock(statement.body, indent)}`;
    case "ErrorStatement":
      throw new Error("Cannot print error statement.");
    default:
      throw new Error(`Cannot print statement kind '${statement.kind}'.`);
  }
}

function printExpression(expression) {
  switch (expression.kind) {
    case "IdentifierExpression":
      return expression.name;
    case "IntegerLiteral":
    case "FloatLiteral":
      return expression.text;
    case "BoolLiteral":
      return expression.value ? "true" : "false";
    case "UnaryExpression":
      return `(${expression.operator}${printExpression(expression.operand)})`;
    case "BinaryExpression":
      return `(${printExpression(expression.left)} ${expression.operator} ${printExpression(expression.right)})`;
    case "CallExpression":
      return `${printExpression(expression.callee)}(${expression.args.map(printExpression).join(", ")})`;
    case "FieldExpression":
      return `${printPostfixObject(expression.object)}.${expression.field.name}`;
    case "IndexExpression":
      return `${printPostfixObject(expression.object)}[${printExpression(expression.index)}]`;
    case "ParenthesizedExpression":
      return `(${printExpression(expression.expression)})`;
    case "ErrorExpression":
      throw new Error("Cannot print error expression.");
    default:
      throw new Error(`Cannot print expression kind '${expression.kind}'.`);
  }
}

function printPostfixObject(expression) {
  switch (expression.kind) {
    case "IdentifierExpression":
    case "IntegerLiteral":
    case "FloatLiteral":
    case "BoolLiteral":
    case "CallExpression":
    case "FieldExpression":
    case "IndexExpression":
    case "ParenthesizedExpression":
      return printExpression(expression);
    default:
      return `(${printExpression(expression)})`;
  }
}

const npmIndexPath = realpathSync(fileURLToPath(import.meta.url));
const npmRoot = dirname(npmIndexPath);
const packageRoot = resolve(npmRoot, "..");
const sourceCheckoutExeName = process.platform === "win32" ? "ckc.exe" : "ckc";

function candidateCkcBinaries() {
  let packagedBinary;
  try {
    packagedBinary = join(npmRoot, "bin", currentPlatformBinaryName());
  } catch {
    packagedBinary = undefined;
  }

  return [
    process.env.CKC_BIN,
    packagedBinary,
    join(packageRoot, "target", "release", sourceCheckoutExeName),
    join(packageRoot, "target", "debug", sourceCheckoutExeName)
  ].filter(Boolean);
}

function resolveCkcBinary() {
  for (const candidate of candidateCkcBinaries()) {
    if (existsSync(candidate)) {
      return candidate;
    }
  }

  const searched = candidateCkcBinaries().join("\n  ");
  throw new Error(
    `Unable to find the Rust ckc binary.\n` +
      `Set CKC_BIN to a built ckc executable or run "cargo build --release".\n` +
      `Supported packaged targets: ${supportedTargetNames().join(", ")}.\n` +
      `Searched:\n  ${searched}`
  );
}

function clangArgs(cFile, outputPath, platform) {
  const flags = ["-std=c11", "-O3", "-Wall", "-Wextra", "-Werror", "-DCK_BUILD_DLL", "-shared"];
  if (platform === "win32") {
    return [...flags, cFile, "-o", outputPath];
  }

  return [...flags, "-fPIC", cFile, "-o", outputPath];
}

function isMissingCommand(result) {
  return result.error?.code === "ENOENT";
}

let tempFileCounter = 0;

function writeFileAtomic(path, contents) {
  tempFileCounter += 1;
  const tempPath = `${path}.tmp-${process.pid}-${tempFileCounter}`;

  try {
    writeFileSync(tempPath, contents);
    renameSync(tempPath, path);
  } catch (error) {
    rmSync(tempPath, { force: true });
    throw error;
  }
}

function checkerDiagnosticCode(message) {
  if (message.startsWith("Unknown variable")) {
    return "CK2001";
  }
  if (message.startsWith("Unknown function")) {
    return "CK2002";
  }
  if (message.startsWith("Unknown type")) {
    return "CK2003";
  }
  if (message.startsWith("Duplicate")) {
    return "CK2005";
  }
  if (message.startsWith("If condition") || message.startsWith("While condition")) {
    return "CK2006";
  }
  if (message.startsWith("Invalid assignment target")) {
    return "CK2007";
  }
  if (message.startsWith("Missing return")) {
    return "CK2008";
  }

  return "CK2004";
}

function integerLiteralFallback(type) {
  return type?.kind === "primitive" && isInteger(type) ? type : primitiveType("i32");
}

function isArithmeticOperator(operator) {
  return operator === "+" || operator === "-" || operator === "*" || operator === "/" || operator === "%";
}

function isComparisonOperator(operator) {
  return operator === "==" || operator === "!=" || operator === "<" || operator === "<=" || operator === ">" || operator === ">=";
}

export function formatDiagnostic(sourceFile, diagnostic) {
  const sourceLine = sourceFile.text.split(/\r?\n/)[diagnostic.line - 1] ?? "";
  const markerWidth = diagnosticMarkerWidth(diagnostic, sourceLine);
  const caret = `${" ".repeat(Math.max(0, diagnostic.column - 1))}${"^".repeat(markerWidth)}`;
  return `${diagnostic.fileName}:${diagnostic.line}:${diagnostic.column}: ${diagnostic.severity} ${diagnostic.code}: ${diagnostic.message}\n${sourceLine}\n${caret}\n`;
}

export function formatDiagnostics(sourceFile, diagnostics) {
  return diagnostics.map((diagnostic) => formatDiagnostic(sourceFile, diagnostic)).join("");
}

function diagnosticMarkerWidth(diagnostic, sourceLine) {
  if (diagnostic.span.start.line === diagnostic.span.end.line) {
    return Math.max(1, diagnostic.span.end.column - diagnostic.span.start.column);
  }

  return Math.max(1, sourceLine.length - diagnostic.column + 1);
}

export function createCKWasmArena(instanceOrExports, options = {}) {
  const api = "createCKWasmArena";
  const exports = exportsFromInstanceOrExports(instanceOrExports, api);
  const memory = exports.memory;
  if (!isWasmMemory(memory)) {
    throw arenaError(
      api,
      "exports.memory must be a WebAssembly.Memory instance",
      "Pass a WebAssembly.Instance, instance.exports, or an exports object containing the exported memory."
    );
  }

  const heapBase = options.heapBase ?? CKWasmArena.heapBaseFromExports(exports);
  if (heapBase === undefined) {
    throw arenaError(
      api,
      "heapBase is missing; no options.heapBase, __ck_heap_base, or __heap_base value was found",
      "Pass { heapBase } explicitly or use CK / CalcKernel WASM output that exports __ck_heap_base."
    );
  }

  return new CKWasmArena(memory, { heapBase });
}

export class CKWasmArena {
  constructor(memory, options = {}) {
    const api = "CKWasmArena.constructor";
    if (!isWasmMemory(memory)) {
      throw arenaError(
        api,
        "memory must be a WebAssembly.Memory instance",
        "Pass instance.exports.memory or create a new WebAssembly.Memory({ initial })."
      );
    }

    this.memory = memory;
    this.buffer = memory.buffer;
    this.nextOffset =
      options.heapBase === undefined
        ? undefined
        : checkedNonNegativeInteger(
            options.heapBase,
            "heapBase",
            api,
            "Pass a non-negative safe integer heapBase option, or omit it and export __ck_heap_base / __heap_base."
          );
  }

  static fromExports(exports, options = {}) {
    const api = "CKWasmArena.fromExports";
    const memory = exports.memory;
    if (!isWasmMemory(memory)) {
      throw arenaError(
        api,
        "exports.memory must be a WebAssembly.Memory instance",
        "Export memory from the WASM module or pass a WebAssembly.Memory directly to the constructor."
      );
    }

    const heapBase = options.heapBase ?? CKWasmArena.heapBaseFromExports(exports);
    return new CKWasmArena(memory, { heapBase });
  }

  static heapBaseFromExports(exports) {
    const api = "CKWasmArena.heapBaseFromExports";
    const candidate = exports.__ck_heap_base ?? exports.__heap_base;
    if (candidate === undefined) {
      return undefined;
    }
    return checkedNonNegativeInteger(
      exportedHeapBaseValue(candidate, api),
      "__ck_heap_base/__heap_base",
      api,
      "Export a non-negative safe integer heap base, or pass { heapBase } explicitly."
    );
  }

  ensureBytes(bytes) {
    this.ensureBytesForApi(bytes, "CKWasmArena.ensureBytes");
  }

  refreshViewsIfNeeded() {
    if (this.buffer !== this.memory.buffer) {
      this.buffer = this.memory.buffer;
    }
  }

  allocBytes(bytes, align) {
    return this.allocBytesForApi(bytes, align, "CKWasmArena.allocBytes");
  }

  allocF64(length) {
    return this.allocTyped(length, Float64Array.BYTES_PER_ELEMENT, "CKWasmArena.allocF64");
  }

  allocI32(length) {
    return this.allocTyped(length, Int32Array.BYTES_PER_ELEMENT, "CKWasmArena.allocI32");
  }

  allocU32(length) {
    return this.allocTyped(length, Uint32Array.BYTES_PER_ELEMENT, "CKWasmArena.allocU32");
  }

  allocI64(length) {
    return this.allocTyped(length, BigInt64Array.BYTES_PER_ELEMENT, "CKWasmArena.allocI64");
  }

  allocU64(length) {
    return this.allocTyped(length, BigUint64Array.BYTES_PER_ELEMENT, "CKWasmArena.allocU64");
  }

  viewF64(ptr, length) {
    return this.viewTyped(Float64Array, ptr, length, Float64Array.BYTES_PER_ELEMENT, "CKWasmArena.viewF64");
  }

  viewI32(ptr, length) {
    return this.viewTyped(Int32Array, ptr, length, Int32Array.BYTES_PER_ELEMENT, "CKWasmArena.viewI32");
  }

  viewU32(ptr, length) {
    return this.viewTyped(Uint32Array, ptr, length, Uint32Array.BYTES_PER_ELEMENT, "CKWasmArena.viewU32");
  }

  viewI64(ptr, length) {
    return this.viewTyped(BigInt64Array, ptr, length, BigInt64Array.BYTES_PER_ELEMENT, "CKWasmArena.viewI64");
  }

  viewU64(ptr, length) {
    return this.viewTyped(BigUint64Array, ptr, length, BigUint64Array.BYTES_PER_ELEMENT, "CKWasmArena.viewU64");
  }

  copyInF64(src) {
    assertTypedArray(src, Float64Array, "Float64Array", "CKWasmArena.copyInF64");
    const ptr = this.allocTyped(src.length, Float64Array.BYTES_PER_ELEMENT, "CKWasmArena.copyInF64");
    const view = this.viewTyped(Float64Array, ptr, src.length, Float64Array.BYTES_PER_ELEMENT, "CKWasmArena.copyInF64");
    view.set(src);
    return { ptr, view };
  }

  copyInI32(src) {
    assertTypedArray(src, Int32Array, "Int32Array", "CKWasmArena.copyInI32");
    const ptr = this.allocTyped(src.length, Int32Array.BYTES_PER_ELEMENT, "CKWasmArena.copyInI32");
    const view = this.viewTyped(Int32Array, ptr, src.length, Int32Array.BYTES_PER_ELEMENT, "CKWasmArena.copyInI32");
    view.set(src);
    return { ptr, view };
  }

  copyInU32(src) {
    assertTypedArray(src, Uint32Array, "Uint32Array", "CKWasmArena.copyInU32");
    const ptr = this.allocTyped(src.length, Uint32Array.BYTES_PER_ELEMENT, "CKWasmArena.copyInU32");
    const view = this.viewTyped(Uint32Array, ptr, src.length, Uint32Array.BYTES_PER_ELEMENT, "CKWasmArena.copyInU32");
    view.set(src);
    return { ptr, view };
  }

  copyOutF64(ptr, length) {
    return new Float64Array(
      this.viewTyped(Float64Array, ptr, length, Float64Array.BYTES_PER_ELEMENT, "CKWasmArena.copyOutF64")
    );
  }

  allocTyped(length, bytesPerElement, api) {
    const itemCount = checkedNonNegativeInteger(length, "length", api, "Pass a non-negative safe integer element count.");
    const bytes = checkedByteLength(itemCount, bytesPerElement, api);
    return this.allocBytesForApi(bytes, bytesPerElement, api);
  }

  allocBytesForApi(bytes, align, api) {
    if (this.nextOffset === undefined) {
      throw arenaError(
        api,
        "allocation requires a heapBase option or exported __ck_heap_base / __heap_base",
        "Pass { heapBase } when constructing the arena, or export a heap base from the WASM module."
      );
    }
    const byteCount = checkedNonNegativeInteger(bytes, "bytes", api, "Pass a non-negative safe integer byte count.");
    const alignment = checkedPositiveInteger(
      align,
      "align",
      api,
      "Pass a positive safe integer alignment such as 4 for i32/u32 or 8 for f64/i64/u64."
    );
    const ptr = alignTo(this.nextOffset, alignment);
    const end = checkedByteEnd(ptr, byteCount, api);
    this.ensureBytesForApi(end, api);
    this.nextOffset = end;
    return ptr;
  }

  viewTyped(ctor, ptr, length, bytesPerElement, api) {
    const byteOffset = checkedNonNegativeInteger(
      ptr,
      "ptr",
      api,
      "Pass the byte offset returned by alloc*/copyIn*, or another non-negative WASM memory byte offset."
    );
    if (byteOffset % bytesPerElement !== 0) {
      throw arenaError(
        api,
        `ptr must be ${bytesPerElement}-byte aligned; got ${byteOffset}`,
        "Use the matching alloc*/copyIn* method or align the pointer before creating the view."
      );
    }
    const itemCount = checkedNonNegativeInteger(length, "length", api, "Pass a non-negative safe integer element count.");
    const byteLength = checkedByteLength(itemCount, bytesPerElement, api);
    const requiredBytes = checkedByteEnd(byteOffset, byteLength, api);
    this.ensureBytesForApi(requiredBytes, api);
    this.refreshViewsIfNeeded();
    try {
      return new ctor(this.buffer, byteOffset, itemCount);
    } catch (error) {
      throw arenaError(
        api,
        `typed array view is out of bounds for ptr=${byteOffset}, length=${itemCount}`,
        `Ensure memory is large enough or call ensureBytes(${requiredBytes}) before creating the view. Cause: ${errorMessage(error)}`
      );
    }
  }

  ensureBytesForApi(bytes, api) {
    const requiredBytes = checkedNonNegativeInteger(bytes, "bytes", api, "Pass a non-negative safe integer byte count.");
    this.refreshViewsIfNeeded();
    if (requiredBytes <= this.buffer.byteLength) {
      return;
    }

    const currentPages = Math.ceil(this.buffer.byteLength / WASM_PAGE_BYTES);
    const requiredPages = Math.ceil(requiredBytes / WASM_PAGE_BYTES);
    const pagesToGrow = requiredPages - currentPages;
    try {
      this.memory.grow(pagesToGrow);
    } catch (error) {
      throw arenaError(
        api,
        `memory.grow failed while growing from ${currentPages} to ${requiredPages} WASM pages for ${requiredBytes} bytes`,
        `Pre-grow memory, increase the memory maximum, or pass smaller buffers. Cause: ${errorMessage(error)}`
      );
    }

    this.refreshViewsIfNeeded();
    if (requiredBytes > this.buffer.byteLength) {
      throw arenaError(
        api,
        `memory.grow completed but memory is still too small for ${requiredBytes} bytes`,
        "Pre-grow memory with enough pages or increase the WebAssembly.Memory maximum."
      );
    }
  }
}

function isWasmMemory(value) {
  const MemoryCtor = globalThis.WebAssembly?.Memory;
  return typeof MemoryCtor === "function" && value instanceof MemoryCtor;
}

function exportsFromInstanceOrExports(value, api) {
  if (typeof value !== "object" || value === null) {
    throw arenaError(
      api,
      "instanceOrExports must be a WebAssembly.Instance-like object or an exports object",
      "Pass a WebAssembly.Instance, instance.exports, or an exports object containing memory."
    );
  }
  const maybeExports = value.exports;
  if (maybeExports !== undefined) {
    if (typeof maybeExports !== "object" || maybeExports === null) {
      throw arenaError(api, "instance.exports must be an object", "Pass a WebAssembly.Instance or its instance.exports object.");
    }
    return maybeExports;
  }
  return value;
}

function isWasmGlobal(value) {
  return typeof value === "object" && value !== null && "value" in value;
}

function exportedHeapBaseValue(value, api) {
  const raw = isWasmGlobal(value) ? value.value : value;
  if (typeof raw === "bigint") {
    if (raw > BigInt(Number.MAX_SAFE_INTEGER)) {
      throw arenaError(
        api,
        "__ck_heap_base/__heap_base is larger than Number.MAX_SAFE_INTEGER",
        "Export a smaller heap base or pass a safe integer { heapBase } explicitly."
      );
    }
    return Number(raw);
  }
  if (typeof raw !== "number") {
    throw arenaError(
      api,
      "__ck_heap_base/__heap_base must be a number or WebAssembly.Global value",
      "Export a numeric heap base or pass { heapBase } explicitly."
    );
  }
  return raw;
}

function checkedNonNegativeInteger(value, name, api, fix) {
  if (!Number.isSafeInteger(value) || value < 0) {
    throw arenaError(api, `${name} must be a non-negative safe integer; got ${String(value)}`, fix);
  }
  return value;
}

function checkedPositiveInteger(value, name, api, fix) {
  if (!Number.isSafeInteger(value) || value <= 0) {
    throw arenaError(api, `${name} must be a positive safe integer; got ${String(value)}`, fix);
  }
  return value;
}

function checkedByteLength(length, bytesPerElement, api) {
  const bytes = length * bytesPerElement;
  if (!Number.isSafeInteger(bytes)) {
    throw arenaError(
      api,
      `byte length must be a safe integer; length=${length}, bytesPerElement=${bytesPerElement}`,
      "Pass a smaller element count or split the buffer into multiple allocations."
    );
  }
  return bytes;
}

function checkedByteEnd(ptr, byteLength, api) {
  const end = ptr + byteLength;
  if (!Number.isSafeInteger(end)) {
    throw arenaError(
      api,
      `ptr + byte length must be a safe integer; ptr=${ptr}, byteLength=${byteLength}`,
      "Pass a smaller pointer/length pair or split the buffer into multiple views."
    );
  }
  return end;
}

function alignTo(value, align) {
  return Math.ceil(value / align) * align;
}

function assertTypedArray(value, ctor, expected, api) {
  if (!(value instanceof ctor)) {
    const article = expected === "Int32Array" ? "an" : "a";
    throw arenaError(api, `src must be ${article} ${expected}`, `Pass ${article} ${expected} backed by the data you want to copy.`);
  }
}

function arenaError(api, reason, fix) {
  return new Error(`${api}: ${reason}. ${fix}`);
}

function errorMessage(error) {
  return error instanceof Error ? error.message : String(error);
}
