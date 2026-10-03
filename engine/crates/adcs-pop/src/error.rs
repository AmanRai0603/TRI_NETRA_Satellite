//! The propagator's error at its outer layers (loading data, building the force model,
//! propagating): what kind of failure it is, and the message. The model modules keep their own
//! precise errors (drag, atmosphere, SPK, frames, integration); they arrive here as `Run`, or as
//! `Data` when a data file is the cause.
use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PopError {
    /// a data file the propagator reads is missing, unreadable or does not parse
    Data(String),
    /// an option or model the port does not have, or an input it cannot use
    Unsupported(String),
    /// the propagation failed (a model refused its inputs, a non-finite state)
    Run(String),
}

impl PopError {
    pub fn message(&self) -> &str { match self { PopError::Data(m) | PopError::Unsupported(m) | PopError::Run(m) => m } }
}

impl fmt::Display for PopError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result { f.write_str(self.message()) }
}

impl std::error::Error for PopError {}

pub type PopResult<T> = Result<T, PopError>;
