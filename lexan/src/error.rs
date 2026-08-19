// Copyright (c) 2026 Peter Williams <pwil3058@bigpond.net.au> <pwil3058@gmail.com>.

use std::fmt::Display;

use thiserror::Error;

#[derive(Debug, PartialEq, Error)]
pub enum LexanError<'a, T: Display> {
    #[error("{0}: duplicate handle.")]
    DuplicateHandle(T),
    #[error("{0}: duplicate regex pattern")]
    DuplicatePattern(&'a str),
    #[error("{0}: empty regex pattern")]
    EmptyPattern(Option<T>),
    #[error("{0}: illegal token")]
    RegexError(#[from] regex::Error),
}
