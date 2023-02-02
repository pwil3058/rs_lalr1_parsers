use std::convert::From;
use std::fmt::Formatter;
use std::{error, fmt};

#[derive(Debug, PartialEq)]
pub enum LexanError<'a, T> {
    DuplicateHandle(T),
    DuplicatePattern(&'a str),
    EmptyPattern(Option<T>),
    RegexError(regex::Error),
}

impl<'a, T> From<regex::Error> for LexanError<'a, T> {
    fn from(error: regex::Error) -> Self {
        LexanError::RegexError(error)
    }
}

impl<'a, T: fmt::Debug> fmt::Display for LexanError<'a, T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::DuplicateHandle(t) => write!(f, "{t:?}: Duplicate handle"),
            Self::DuplicatePattern(s) => write!(f, "{s:?}: Duplicate pattern"),
            Self::EmptyPattern(p) => write!(f, "{p:?}: Empty pattern"),
            Self::RegexError(re) => write!(f, "Regex Error: {re:?}"),
        }
    }
}

impl<'a, T: fmt::Debug> error::Error for LexanError<'a, T> {}
