use std::convert::From;

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
