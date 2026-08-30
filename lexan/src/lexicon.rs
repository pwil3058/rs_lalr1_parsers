// Copyright (c) 2026 Peter Williams <pwil3058@bigpond.net.au> <pwil3058@gmail.com>.

pub use std::fmt::{Debug, Display};

use crate::matcher::{LiteralMatcher, RegexMatcher, SkipMatcher};

use thiserror::Error;

#[derive(Debug, PartialEq, Error, Clone)]
pub enum Error<T: Display + Copy + Debug + Eq> {
    #[error("{0}: duplicate handle.")]
    DuplicateHandle(T),
    #[error("{0}: duplicate literal or regex pattern")]
    DuplicatePattern(String),
    #[error("{0:?}: empty literal or regex pattern")]
    EmptyPattern(Option<T>),
    #[error("{0}: invalid regex pattern")]
    RegexError(#[from] regex::Error),
}

#[derive(Default)]
pub struct Lexicon<T>
where
    T: Copy + PartialEq + Debug + Display,
{
    literal_matcher: LiteralMatcher<T>,
    regex_matcher: RegexMatcher<T>,
    skip_matcher: SkipMatcher,
    end_marker: T,
}

impl<T> Lexicon<T>
where
    T: Copy + Eq + Debug + Display + Ord,
{
    pub fn new(
        literal_lexemes: &[(T, &str)],
        regex_lexemes: &[(T, &str)],
        skip_regexes: &[&str],
        end_marker: T,
    ) -> Result<Self, Error<T>> {
        let mut tags = vec![end_marker];
        let mut patterns = vec![];
        for (tag, pattern) in literal_lexemes.iter().chain(regex_lexemes.iter()) {
            match tags.binary_search(tag) {
                Ok(_) => return Err(Error::DuplicateHandle(*tag)),
                Err(index) => tags.insert(index, *tag),
            }
            match patterns.binary_search(pattern) {
                Ok(_) => return Err(Error::DuplicatePattern(pattern.to_string())),
                Err(index) => patterns.insert(index, pattern),
            }
        }
        for regex in skip_regexes.iter() {
            match patterns.binary_search(regex) {
                Ok(_) => return Err(Error::DuplicatePattern(regex.to_string())),
                Err(index) => patterns.insert(index, regex),
            }
        }
        let literal_matcher = LiteralMatcher::new(literal_lexemes)?;
        let regex_matcher = RegexMatcher::new(regex_lexemes)?;
        let skip_matcher = SkipMatcher::new(skip_regexes)?;
        Ok(Self {
            literal_matcher,
            regex_matcher,
            skip_matcher,
            end_marker,
        })
    }

    /// Returns the end marker for this Lexicon
    pub fn end_marker(&self) -> T {
        self.end_marker
    }

    /// Returns number of skippable bytes at start of `text`.
    pub fn skippable_count(&self, text: &str) -> usize {
        self.skip_matcher.skippable_count(text)
    }

    /// Returns the longest literal match at start of `text`.
    pub fn longest_literal_match(&self, text: &str) -> Option<(T, usize)> {
        self.literal_matcher.longest_match(text)
    }

    /// Returns the longest regular expression match at start of `text`.
    pub fn longest_regex_matches(&self, text: &str) -> (Vec<T>, usize) {
        self.regex_matcher.longest_matches(text)
    }

    /// Returns the distance in bytes to the next valid content in `text`
    pub fn distance_to_next_valid_byte(&self, text: &str) -> usize {
        for index in 0..text.len() {
            if self.literal_matcher.matches(&text[index..]) {
                return index;
            }
            if self.regex_matcher.matches(&text[index..]) {
                return index;
            }
            if self.skip_matcher.matches(&text[index..]) {
                return index;
            }
        }
        text.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(PartialEq, Eq, Clone, Copy, Hash, PartialOrd, Ord, Debug)]
    enum Tag {
        If,
        When,
        Ident,
        Btextl,
        Pred,
        Literal,
        Action,
        Predicate,
        Code,
        End,
    }

    impl std::fmt::Display for Tag {
        fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
            use Tag::*;
            match self {
                If => write!(f, "\"if\""),
                When => write!(f, "\"when\""),
                Ident => write!(f, "Ident"),
                Btextl => write!(f, "Btextl"),
                Pred => write!(f, "Pred"),
                Literal => write!(f, "Literal"),
                Action => write!(f, "Action"),
                Predicate => write!(f, "Predicate"),
                Code => write!(f, "Code"),
                End => write!(f, "End"),
            }
        }
    }

    #[test]
    fn lexicon_ok() {
        use self::Tag::*;
        let lexicon = Lexicon::<Tag>::new(
            &[(If, "if"), (When, "when")],
            &[
                (Ident, "[a-zA-Z]+[\\w_]*"),
                (Btextl, r"&\{(.|[\n\r])*&\}"),
                (Pred, r"\?\{(.|[\n\r])*\?\}"),
                (Literal, "(\"\\S+\")"),
                (Action, r"(!\{(.|[\n\r])*?!\})"),
                (Predicate, r"(\?\((.|[\n\r])*?\?\))"),
                (Code, r"(%\{(.|[\n\r])*?%\})"),
            ],
            &[r"(/\*(.|[\n\r])*?\*/)", r"(//[^\n\r]*)", r"(\s+)"],
            End,
        );
        assert!(lexicon.is_ok());
    }

    #[test]
    fn lexicon_fail() {
        use self::Tag::*;
        let lexicon = Lexicon::<Tag>::new(
            &[(If, "if"), (If, "when")],
            &[
                (Ident, "[a-zA-Z]+[\\w_]*"),
                (Btextl, r"&\{(.|[\n\r])*&\}"),
                (Pred, r"\?\{(.|[\n\r])*\?\}"),
                (Literal, "(\"\\S+\")"),
                (Action, r"(!\{(.|[\n\r])*?!\})"),
                (Predicate, r"(\?\((.|[\n\r])*?\?\))"),
                (Code, r"(%\{(.|[\n\r])*?%\})"),
            ],
            &[r"(/\*(.|[\n\r])*?\*/)", r"(//[^\n\r]*)", r"(\s+)"],
            End,
        );
        if let Err(err) = lexicon {
            assert_eq!(err, Error::DuplicateHandle(If));
        } else {
            assert!(false)
        }

        let lexicon = Lexicon::<Tag>::new(
            &[(If, "if"), (When, "when")],
            &[
                (Action, "[a-zA-Z]+[\\w_]*"),
                (Btextl, r"&\{(.|[\n\r])*&\}"),
                (Pred, r"\?\{(.|[\n\r])*\?\}"),
                (Literal, "(\"\\S+\")"),
                (Action, r"(!\{(.|[\n\r])*?!\})"),
                (Predicate, r"(\?\((.|[\n\r])*?\?\))"),
                (Code, r"(%\{(.|[\n\r])*?%\})"),
            ],
            &[r"(/\*(.|[\n\r])*?\*/)", r"(//[^\n\r]*)", r"(\s+)"],
            End,
        );
        if let Err(err) = lexicon {
            assert_eq!(err, Error::DuplicateHandle(Action));
        } else {
            assert!(false)
        }

        let lexicon = Lexicon::<Tag>::new(
            &[(If, "if"), (When, "when")],
            &[
                (Ident, "[a-zA-Z]+[\\w_]*"),
                (Btextl, r"&\{(.|[\n\r])*&\}"),
                (Pred, r"\?\{(.|[\n\r])*\?\}"),
                (Literal, "(\"\\S+\")"),
                (When, r"(!\{(.|[\n\r])*?!\})"),
                (Predicate, r"(\?\((.|[\n\r])*?\?\))"),
                (Code, r"(%\{(.|[\n\r])*?%\})"),
            ],
            &[r"(/\*(.|[\n\r])*?\*/)", r"(//[^\n\r]*)", r"(\s+)"],
            End,
        );
        if let Err(err) = lexicon {
            assert_eq!(err, Error::DuplicateHandle(When));
        } else {
            assert!(false)
        }

        let lexicon = Lexicon::<Tag>::new(
            &[(If, "if"), (When, "when")],
            &[
                (Ident, "[a-zA-Z]+[\\w_]*"),
                (Btextl, r"&\{(.|[\n\r])*&\}"),
                (Pred, r"\?\{(.|[\n\r])*\?\}"),
                (Literal, "(\"\\S+\")"),
                (Action, r"(!\{(.|[\n\r])*?!\})"),
                (Predicate, r"(\?\((.|[\n\r])*?\?\))"),
                (Code, r"(%\{(.|[\n\r])*?%\})"),
            ],
            &[r"(/\*(.|[\n\r])*?\*/)", r"(//[^\n\r]*)", r"(\s+)"],
            Action,
        );
        if let Err(err) = lexicon {
            assert_eq!(err, Error::DuplicateHandle(Action));
        } else {
            assert!(false)
        }

        let lexicon = Lexicon::<Tag>::new(
            &[(If, "if"), (When, "if")],
            &[
                (Ident, "[a-zA-Z]+[\\w_]*"),
                (Btextl, r"&\{(.|[\n\r])*&\}"),
                (Pred, r"\?\{(.|[\n\r])*\?\}"),
                (Literal, "(\"\\S+\")"),
                (Action, r"(!\{(.|[\n\r])*?!\})"),
                (Predicate, r"(\?\((.|[\n\r])*?\?\))"),
                (Code, r"(%\{(.|[\n\r])*?%\})"),
            ],
            &[r"(/\*(.|[\n\r])*?\*/)", r"(//[^\n\r]*)", r"(\s+)"],
            End,
        );
        if let Err(err) = lexicon {
            assert_eq!(err, Error::DuplicatePattern("if".to_string()));
        } else {
            assert!(false)
        }

        let lexicon = Lexicon::<Tag>::new(
            &[(If, "if"), (When, "when")],
            &[
                (Ident, "[a-zA-Z]+[\\w_]*"),
                (Btextl, r"&\{(.|[\n\r])*&\}"),
                (Pred, r"\?\{(.|[\n\r])*\?\}"),
                (Literal, "when"),
                (Action, r"(!\{(.|[\n\r])*?!\})"),
                (Predicate, r"(\?\((.|[\n\r])*?\?\))"),
                (Code, r"(%\{(.|[\n\r])*?%\})"),
            ],
            &[r"(/\*(.|[\n\r])*?\*/)", r"(//[^\n\r]*)", r"(\s+)"],
            End,
        );
        if let Err(err) = lexicon {
            assert_eq!(err, Error::DuplicatePattern("when".to_string()));
        } else {
            assert!(false)
        }

        let lexicon = Lexicon::<Tag>::new(
            &[(If, "if"), (When, "when")],
            &[
                (Ident, "(\"\\S+\")"),
                (Btextl, r"&\{(.|[\n\r])*&\}"),
                (Pred, r"\?\{(.|[\n\r])*\?\}"),
                (Literal, "(\"\\S+\")"),
                (Action, r"(!\{(.|[\n\r])*?!\})"),
                (Predicate, r"(\?\((.|[\n\r])*?\?\))"),
                (Code, r"(%\{(.|[\n\r])*?%\})"),
            ],
            &[r"(/\*(.|[\n\r])*?\*/)", r"(//[^\n\r]*)", r"(\s+)"],
            End,
        );
        if let Err(err) = lexicon {
            assert_eq!(err, Error::DuplicatePattern("(\"\\S+\")".to_string()));
        } else {
            assert!(false)
        }

        let lexicon = Lexicon::<Tag>::new(
            &[(If, "if"), (When, "when")],
            &[
                (Ident, "[a-zA-Z]+[\\w_]*"),
                (Btextl, r"&\{(.|[\n\r])*&\}"),
                (Pred, r"\?\{(.|[\n\r])*\?\}"),
                (Literal, "(\"\\S+\")"),
                (Action, r"(!\{(.|[\n\r])*?!\})"),
                (Predicate, r"(\?\((.|[\n\r])*?\?\))"),
                (Code, r"(%\{(.|[\n\r])*?%\})"),
            ],
            &[r"(/\*(.|[\n\r])*?\*/)", r"(//[^\n\r]*)", "(\"\\S+\")"],
            End,
        );
        if let Err(err) = lexicon {
            assert_eq!(err, Error::DuplicatePattern("(\"\\S+\")".to_string()));
        } else {
            assert!(false)
        }
    }
}
