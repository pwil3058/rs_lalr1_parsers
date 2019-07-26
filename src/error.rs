use std::convert::From;

use regex;

#[derive(Debug, PartialEq)]
pub enum LexanError<'a, H> {
    DuplicateHandle(H),
    DuplicatePattern(&'a str),
    EmptyPattern(H),
    RegexError(regex::Error),
    UnanchoredRegex(&'a str),
}

impl<'a, H> From<regex::Error> for LexanError<'a, H> {
    fn from(error: regex::Error) -> Self {
        LexanError::RegexError(error)
    }
}
