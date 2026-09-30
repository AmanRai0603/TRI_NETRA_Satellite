//! The engine's error: what kind of failure it is, and the message a person reads.
//!
//! The kind says whose it is to fix. `Refused` is the caller's: an input the engine cannot
//! fly or a request it will not do, named with what it takes (`adcs` exits 2, the app
//! answers 400). `Io` and `Malformed` are the files': one could not be read or written, or
//! was read and is not what it should be. `Run` is the run's own: the simulation or the
//! flight software failed. The message is the same text whatever the kind.
//! Owner: Agastya. Copyright (c) 2026 Agastya. All rights reserved.
use std::fmt;
use std::path::Path;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Kind { Refused, Io, Malformed, Run }

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Error { pub kind: Kind, msg: String }

pub type Result<T> = std::result::Result<T, Error>;

impl Error {
    pub fn refused(msg: impl Into<String>) -> Self { Error { kind: Kind::Refused, msg: msg.into() } }
    pub fn malformed(msg: impl Into<String>) -> Self { Error { kind: Kind::Malformed, msg: msg.into() } }
    pub fn run(msg: impl Into<String>) -> Self { Error { kind: Kind::Run, msg: msg.into() } }
    /// A file that could not be read or written: `<path>: <why>`.
    pub fn io(path: &Path, e: impl fmt::Display) -> Self { Error { kind: Kind::Io, msg: format!("{}: {e}", path.display()) } }
    pub fn message(&self) -> &str { &self.msg }
    /// The exit status a command line gives it: 2 for a refused input, 1 for a failure.
    pub fn exit_code(&self) -> u8 { if self.kind == Kind::Refused { 2 } else { 1 } }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result { f.write_str(&self.msg) }
}

impl std::error::Error for Error {}

/// A message from a crate that reports errors as text (the orbit propagator, the flight
/// software link): a failure of the run.
impl From<String> for Error {
    fn from(s: String) -> Self { Error::run(s) }
}

/// For callers that report errors as text.
impl From<Error> for String {
    fn from(e: Error) -> Self { e.msg }
}
