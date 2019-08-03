use std::fmt::Debug;

use crate::error::LexanError;
use crate::matcher::{LiteralMatcher, RegexMatcher, SkipMatcher};

#[derive(Default, Debug)]
pub struct Lexicon<T>
where
    T: Copy + PartialEq + Debug,
{
    literal_matcher: LiteralMatcher<T>,
    regex_matcher: RegexMatcher<T>,
    skip_matcher: SkipMatcher,
    end_marker: T,
}

impl<T> Lexicon<T>
where
    T: Copy + Eq + Debug + Ord,
{
    pub fn new<'a>(
        literal_lexemes: &[(T, &'a str)],
        regex_lexemes: &[(T, &'a str)],
        skip_regex_strs: &[&'a str],
        end_marker: T,
    ) -> Result<Self, LexanError<'a, T>> {
        let mut tags = vec![end_marker];
        let mut patterns = vec![];
        for (tag, pattern) in literal_lexemes.iter().chain(regex_lexemes.iter()) {
            match tags.binary_search(tag) {
                Ok(_) => return Err(LexanError::DuplicateHandle(*tag)),
                Err(index) => tags.insert(index, *tag),
            }
            match patterns.binary_search(pattern) {
                Ok(_) => return Err(LexanError::DuplicatePattern(pattern)),
                Err(index) => patterns.insert(index, pattern),
            }
        }
        for regex in skip_regex_strs.iter() {
            match patterns.binary_search(regex) {
                Ok(_) => return Err(LexanError::DuplicatePattern(regex)),
                Err(index) => patterns.insert(index, regex),
            }
        }
        let literal_matcher = LiteralMatcher::new(literal_lexemes)?;
        let regex_matcher = RegexMatcher::new(regex_lexemes)?;
        let skip_matcher = SkipMatcher::new(skip_regex_strs)?;
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

    #[derive(PartialEq, Eq, Clone, Copy, Hash, Debug, PartialOrd, Ord)]
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
            assert_eq!(err, LexanError::DuplicateHandle(If));
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
            assert_eq!(err, LexanError::DuplicateHandle(Action));
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
            assert_eq!(err, LexanError::DuplicateHandle(When));
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
            assert_eq!(err, LexanError::DuplicateHandle(Action));
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
            assert_eq!(err, LexanError::DuplicatePattern("if"));
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
            assert_eq!(err, LexanError::DuplicatePattern("when"));
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
            assert_eq!(err, LexanError::DuplicatePattern("(\"\\S+\")"));
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
            assert_eq!(err, LexanError::DuplicatePattern("(\"\\S+\")"));
        } else {
            assert!(false)
        }
    }
}
