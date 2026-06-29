//! Rust implementation of the CK / CalcKernel compiler.

mod backend;
mod diagnostics;
mod lexer;
mod mir;
mod opt;
mod parser;
mod source;
mod typeck;

pub use backend::*;
pub use diagnostics::{
    Diagnostic, DiagnosticCode, DiagnosticSeverity, format_diagnostic, format_diagnostics,
};
pub use lexer::{LexResult, Token, TokenKind, lex};
pub use mir::*;
pub use opt::*;
pub use parser::*;
pub use source::{SourceFile, SourcePosition, SourceSpan};
pub use typeck::*;
