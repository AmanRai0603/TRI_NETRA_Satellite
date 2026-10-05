//! The two kinds of failure: a problem in the source (found by the lexer, the parser or the
//! checker, at a file, line and column), and a run that stops (an index outside its array, a
//! division of ints by zero, a bit operation out of its range).
//! Owner: Agastya. Copyright (c) 2026 Agastya. All rights reserved.

use std::fmt;
use std::rc::Rc;

/// Where in the sources: the file as it was named to `compile`, the line and column from 1
/// (columns count UTF-16 code units, as the JavaScript does). A position the checker does not
/// know is line 0, column 0.
#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub struct Pos {
    pub file: Option<Rc<str>>,
    pub line: u32,
    pub col: u32,
}

/// Which stage found a problem. `Internal` is a source on which the JavaScript checker itself
/// fails (it throws a TypeError rather than naming a problem); the Rust names it and stops there.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ErrorKind {
    Lex,
    Parse,
    Check,
    Internal,
}

/// A problem in the source, as `design/js/pcode.js` reports it (its `PcodeError`).
#[derive(Clone, Debug, PartialEq)]
pub struct PcodeError {
    pub kind: ErrorKind,
    pub message: String,
    pub pos: Pos,
}

impl PcodeError {
    pub(crate) fn new(kind: ErrorKind, message: impl Into<String>, pos: &Pos) -> Self {
        PcodeError { kind, message: message.into(), pos: pos.clone() }
    }
    /// `file:line:col`, as the JavaScript's `where()`.
    pub fn location(&self) -> String {
        format!("{}:{}:{}", self.pos.file.as_deref().unwrap_or("<input>"), self.pos.line, self.pos.col)
    }
}

impl fmt::Display for PcodeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", self.location(), self.message)
    }
}

impl std::error::Error for PcodeError {}

/// A run that stops, with the JavaScript's message (its `RunError`).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RunError(pub String);

impl fmt::Display for RunError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl std::error::Error for RunError {}
