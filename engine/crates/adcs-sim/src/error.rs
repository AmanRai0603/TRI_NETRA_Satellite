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

/// The orbit propagator's error: a data file it could not use is the file's, an option it does not
/// have is refused, a failed propagation is the run's.
impl From<adcs_pop::PopError> for Error {
    fn from(e: adcs_pop::PopError) -> Self {
        match e {
            adcs_pop::PopError::Data(m) => Error { kind: Kind::Io, msg: m },
            adcs_pop::PopError::Unsupported(m) => Error::refused(m),
            adcs_pop::PopError::Run(m) => Error::run(m),
        }
    }
}

/// The flight-software layer's error: a configuration the flight software refused is the
/// caller's, a link or OBC failure is the run's.
impl From<adcs_fsw_abi::FswError> for Error {
    fn from(e: adcs_fsw_abi::FswError) -> Self {
        match e {
            adcs_fsw_abi::FswError::Refused(m) => Error::refused(m),
            adcs_fsw_abi::FswError::Link(m) => Error::run(m),
        }
    }
}

/// For callers that report errors as text.
impl From<Error> for String {
    fn from(e: Error) -> Self { e.msg }
}
