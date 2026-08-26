// Copyright (c) 2026 Peter Williams <pwil3058@bigpond.net.au> <pwil3058@gmail.com>.

use thiserror::Error;

pub mod production;
pub mod state;
pub mod symbol;

#[derive(Debug, Error)]
pub enum GrammarError {
    #[error("Too many errors: {0}")]
    TooManyErrors(u32),
    #[error("{0} undefined symbols")]
    UndefinedSymbols(u32),
    #[error("Unexpected Shift/Reduce conflicts: {0} {1} {2}")]
    UnexpectedSRConflicts(u32, u32, String),
    #[error("Unexpected Reduce/Reduce conflicts: {0} {1} {2}")]
    UnexpectedRRConflicts(u32, u32, String),
}

#[cfg(test)]
mod tests;
