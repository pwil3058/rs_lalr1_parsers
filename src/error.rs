use std::convert::From;

use regex;

#[derive(Debug)]
pub enum LexanError<'a, H> {
    AmbiguousMatch(&'a str, Vec<H>),
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
