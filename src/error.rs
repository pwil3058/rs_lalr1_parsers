use std::convert::From;

use regex;

#[derive(Debug, PartialEq)]
pub enum LexanError<'a, T> {
    DuplicateHandle(T),
    DuplicatePattern(&'a str),
    EmptyPattern(Option<T>),
    RegexError(regex::Error),
    UnanchoredRegex(&'a str),
}

impl<'a, T> From<regex::Error> for LexanError<'a, T> {
    fn from(error: regex::Error) -> Self {
        LexanError::RegexError(error)
    }
}
