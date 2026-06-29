# Rust CalcKernel Milestone 1 Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build the Rust project foundation and implement source spans, diagnostics, and lexer behavior compatible with the TypeScript CalcKernel oracle.

**Architecture:** The crate exposes pure library APIs for `SourceFile`, diagnostics, tokens, and lexing. The binary is a thin shell that will later host the `ckc` CLI. Tests are written first and compare stable user-visible behavior, especially token kind/text/position and diagnostic formatting.

**Tech Stack:** Rust 1.90, Cargo, `thiserror` for typed errors, `assert_cmd` and `predicates` for future CLI tests.

---

## File Structure

- Create `/Users/lynn/code/Rust_CalcKernel/Cargo.toml` for package metadata, library, binary, and dev dependencies.
- Create `/Users/lynn/code/Rust_CalcKernel/.gitignore` for Rust build outputs and temporary generated artifacts.
- Create `/Users/lynn/code/Rust_CalcKernel/src/lib.rs` as the public library module root.
- Create `/Users/lynn/code/Rust_CalcKernel/src/main.rs` as the future `ckc` binary entrypoint.
- Create `/Users/lynn/code/Rust_CalcKernel/src/source.rs` for `SourceFile`, `SourcePosition`, and `SourceSpan`.
- Create `/Users/lynn/code/Rust_CalcKernel/src/diagnostics.rs` for diagnostic codes and formatting.
- Create `/Users/lynn/code/Rust_CalcKernel/src/lexer/mod.rs` for token kinds, tokens, and `lex`.
- Create `/Users/lynn/code/Rust_CalcKernel/tests/lexer_test.rs` for lexer and diagnostic oracle tests.

## Task 1: Bootstrap Project Files

**Files:**
- Create: `/Users/lynn/code/Rust_CalcKernel/Cargo.toml`
- Create: `/Users/lynn/code/Rust_CalcKernel/.gitignore`
- Create: `/Users/lynn/code/Rust_CalcKernel/src/lib.rs`
- Create: `/Users/lynn/code/Rust_CalcKernel/src/main.rs`

- [ ] **Step 1: Write failing crate smoke test**

Create `/Users/lynn/code/Rust_CalcKernel/tests/lexer_test.rs` with:

```rust
use calckernel::{lex, SourceFile, TokenKind};

#[test]
fn lex_should_emit_eof_for_empty_source() {
    let source = SourceFile::new("empty.ck", "");
    let result = lex(&source);

    assert_eq!(result.tokens.len(), 1);
    assert_eq!(result.tokens[0].kind, TokenKind::Eof);
}
```

- [ ] **Step 2: Run test to verify it fails**

Run:

```sh
cd /Users/lynn/code/Rust_CalcKernel
cargo test lex_should_emit_eof_for_empty_source
```

Expected: fail because the crate and exported symbols do not exist yet.

- [ ] **Step 3: Add minimal Cargo project and exports**

Create the package, module root, and placeholder binary. The library should
compile, but the lexer implementation can be minimal enough to satisfy only the
smoke test.

- [ ] **Step 4: Run smoke test to verify it passes**

Run:

```sh
cd /Users/lynn/code/Rust_CalcKernel
cargo test lex_should_emit_eof_for_empty_source
```

Expected: pass.

## Task 2: Source and Diagnostic Formatting

**Files:**
- Modify: `/Users/lynn/code/Rust_CalcKernel/src/source.rs`
- Modify: `/Users/lynn/code/Rust_CalcKernel/src/diagnostics.rs`
- Modify: `/Users/lynn/code/Rust_CalcKernel/src/lib.rs`
- Test: `/Users/lynn/code/Rust_CalcKernel/tests/lexer_test.rs`

- [ ] **Step 1: Write failing diagnostic formatting test**

Append to `/Users/lynn/code/Rust_CalcKernel/tests/lexer_test.rs`:

```rust
use calckernel::{format_diagnostic, Diagnostic, DiagnosticCode, SourcePosition, SourceSpan};

#[test]
fn format_diagnostic_should_match_typescript_caret_output() {
    let source = SourceFile::new("test.ck", "export fn bad() -> i32 {\n  return @;\n}\n");
    let diagnostic = Diagnostic::error(
        DiagnosticCode::Ck0001,
        "Unexpected character '@'.",
        "test.ck",
        SourceSpan {
            start: SourcePosition { offset: 34, line: 2, column: 10 },
            end: SourcePosition { offset: 35, line: 2, column: 11 },
        },
    );

    assert_eq!(
        format_diagnostic(&source, &diagnostic),
        "test.ck:2:10: error CK0001: Unexpected character '@'.\n  return @;\n         ^\n"
    );
}
```

- [ ] **Step 2: Run test to verify it fails**

Run:

```sh
cd /Users/lynn/code/Rust_CalcKernel
cargo test format_diagnostic_should_match_typescript_caret_output
```

Expected: fail because diagnostic formatting is missing.

- [ ] **Step 3: Implement source and diagnostic formatting**

Implement exact line lookup, marker width, diagnostic code display, and public
exports.

- [ ] **Step 4: Run diagnostic test to verify it passes**

Run:

```sh
cd /Users/lynn/code/Rust_CalcKernel
cargo test format_diagnostic_should_match_typescript_caret_output
```

Expected: pass.

## Task 3: Lexer Tokens and Positions

**Files:**
- Modify: `/Users/lynn/code/Rust_CalcKernel/src/lexer/mod.rs`
- Test: `/Users/lynn/code/Rust_CalcKernel/tests/lexer_test.rs`

- [ ] **Step 1: Write failing token stream test**

Append:

```rust
#[test]
fn lex_should_emit_keywords_identifiers_numbers_and_punctuation() {
    let source = SourceFile::new("test.ck", "export fn add(a: i64, b: i64) -> i64 { return a + b; }\n");
    let result = lex(&source);
    let kinds: Vec<TokenKind> = result.tokens.iter().map(|token| token.kind).collect();

    assert_eq!(
        kinds,
        vec![
            TokenKind::Export,
            TokenKind::Fn,
            TokenKind::Identifier,
            TokenKind::LeftParen,
            TokenKind::Identifier,
            TokenKind::Colon,
            TokenKind::I64,
            TokenKind::Comma,
            TokenKind::Identifier,
            TokenKind::Colon,
            TokenKind::I64,
            TokenKind::RightParen,
            TokenKind::Arrow,
            TokenKind::I64,
            TokenKind::LeftBrace,
            TokenKind::Return,
            TokenKind::Identifier,
            TokenKind::Plus,
            TokenKind::Identifier,
            TokenKind::Semicolon,
            TokenKind::RightBrace,
            TokenKind::Eof,
        ]
    );
    assert_eq!(result.tokens[0].text, "export");
    assert_eq!(result.tokens[0].line, 1);
    assert_eq!(result.tokens[0].column, 1);
}
```

- [ ] **Step 2: Run test to verify it fails**

Run:

```sh
cd /Users/lynn/code/Rust_CalcKernel
cargo test lex_should_emit_keywords_identifiers_numbers_and_punctuation
```

Expected: fail because only the smoke lexer exists.

- [ ] **Step 3: Implement token kinds and scanner**

Implement all TS token kinds, keywords, whitespace skipping, `//` line comments,
single and two-character operators, integer literals, float literals, malformed
float diagnostics, and unexpected-character diagnostics.

- [ ] **Step 4: Run token stream test to verify it passes**

Run:

```sh
cd /Users/lynn/code/Rust_CalcKernel
cargo test lex_should_emit_keywords_identifiers_numbers_and_punctuation
```

Expected: pass.

## Task 4: Lexer Error Compatibility

**Files:**
- Modify: `/Users/lynn/code/Rust_CalcKernel/src/lexer/mod.rs`
- Test: `/Users/lynn/code/Rust_CalcKernel/tests/lexer_test.rs`

- [ ] **Step 1: Write failing lexer diagnostics tests**

Append:

```rust
#[test]
fn lex_should_report_unexpected_character_with_ck0001() {
    let source = SourceFile::new("test.ck", "export fn bad() -> i32 {\n  return @;\n}\n");
    let result = lex(&source);

    assert_eq!(result.diagnostics.len(), 1);
    assert_eq!(result.diagnostics[0].code, DiagnosticCode::Ck0001);
    assert_eq!(result.diagnostics[0].message, "Unexpected character '@'.");
    assert_eq!(format_diagnostic(&source, &result.diagnostics[0]), "test.ck:2:10: error CK0001: Unexpected character '@'.\n  return @;\n         ^\n");
}

#[test]
fn lex_should_report_malformed_float_with_ck0001() {
    let source = SourceFile::new("test.ck", "export fn bad() -> f64 {\n  return 1.;\n}\n");
    let result = lex(&source);

    assert_eq!(result.diagnostics.len(), 1);
    assert_eq!(result.diagnostics[0].code, DiagnosticCode::Ck0001);
    assert_eq!(result.diagnostics[0].message, "Malformed float literal '1.'.");
}
```

- [ ] **Step 2: Run tests to verify they fail**

Run:

```sh
cd /Users/lynn/code/Rust_CalcKernel
cargo test lex_should_report
```

Expected: fail until diagnostics match the oracle.

- [ ] **Step 3: Implement recovery-compatible lexer errors**

Ensure bad characters and malformed floats report diagnostics and lexing
continues to EOF.

- [ ] **Step 4: Run lexer diagnostics tests**

Run:

```sh
cd /Users/lynn/code/Rust_CalcKernel
cargo test lex_should_report
```

Expected: pass.

## Task 5: Milestone Verification

**Files:**
- All files created in this milestone.

- [ ] **Step 1: Run full Rust checks**

Run:

```sh
cd /Users/lynn/code/Rust_CalcKernel
cargo test
cargo fmt --check
cargo clippy --all-targets --all-features --locked -- -D warnings
```

Expected: all pass.

- [ ] **Step 2: Compare against TypeScript oracle health**

Run:

```sh
cd /Users/lynn/code/CalcKernel
pnpm test -- --runInBand
```

Expected: 60 test files and 426 tests pass.

- [ ] **Step 3: Record remaining scope**

The next plan must cover parser and AST compatibility before implementing type
checking, MIR, optimizers, or backends.
