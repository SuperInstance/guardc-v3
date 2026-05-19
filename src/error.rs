use std::fmt;

#[derive(Debug, Clone)]
pub struct Span {
    pub start: usize,
    pub end: usize,
    pub line: usize,
    pub col: usize,
}

impl Span {
    pub fn new(start: usize, end: usize, line: usize, col: usize) -> Self {
        Self { start, end, line, col }
    }
    pub fn zero() -> Self {
        Self { start: 0, end: 0, line: 1, col: 1 }
    }
}

#[derive(Debug, Clone)]
pub enum ErrorKind {
    LexError(String),
    ParseError(String),
    TypeError(String),
    CodegenError(String),
    ProofError(String),
}

#[derive(Debug, Clone)]
pub struct CompileError {
    pub kind: ErrorKind,
    pub span: Span,
    pub message: String,
}

impl CompileError {
    pub fn lex(msg: impl Into<String>, span: Span) -> Self {
        Self { kind: ErrorKind::LexError(msg.into()), span, message: String::new() }
    }
    pub fn parse(msg: impl Into<String>, span: Span) -> Self {
        Self { kind: ErrorKind::ParseError(msg.into()), span, message: String::new() }
    }
    pub fn type_err(msg: impl Into<String>, span: Span) -> Self {
        Self { kind: ErrorKind::TypeError(msg.into()), span, message: String::new() }
    }
    pub fn codegen(msg: impl Into<String>, span: Span) -> Self {
        Self { kind: ErrorKind::CodegenError(msg.into()), span, message: String::new() }
    }
}

impl fmt::Display for CompileError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "error at {}:{}: {}", self.span.line, self.span.col, match &self.kind {
            ErrorKind::LexError(m) | ErrorKind::ParseError(m) |
            ErrorKind::TypeError(m) | ErrorKind::CodegenError(m) |
            ErrorKind::ProofError(m) => m,
        })
    }
}

impl std::error::Error for CompileError {}

pub type Result<T> = std::result::Result<T, CompileError>;
