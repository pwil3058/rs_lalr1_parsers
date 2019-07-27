use std::fmt::Debug;

use crate::error::LexanError;
use crate::matcher::{LiteralMatcher, RegexMatcher, SkipMatcher};

#[derive(Default, Debug)]
pub struct Lexicon<H>
where
    H: Copy + PartialEq + Debug,
{
    literal_matcher: LiteralMatcher<H>,
    regex_matcher: RegexMatcher<H>,
    skip_matcher: SkipMatcher,
}

impl<H> Lexicon<H>
where
    H: Copy + Eq + Debug + Ord,
{
    pub fn new<'a>(
        literal_lexemes: &[(H, &'a str)],
        regex_lexemes: &[(H, &'a str)],
        skip_regex_strs: &[&'a str],
    ) -> Result<Self, LexanError<'a, H>> {
        let mut handles = vec![];
        let mut patterns = vec![];
        for (handle, pattern) in literal_lexemes.iter().chain(regex_lexemes.iter()) {
            match handles.binary_search(handle) {
                Ok(_) => return Err(LexanError::DuplicateHandle(*handle)),
                Err(index) => handles.insert(index, *handle),
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
        })
    }

    /// Returns number of skippable bytes at start of `text`.
    pub fn skippable_count(&self, text: &str) -> usize {
        self.skip_matcher.skippable_count(text)
    }

    /// Returns the longest literal match at start of `text`.
    pub fn longest_literal_match(&self, text: &str) -> Option<(H, usize)> {
        self.literal_matcher.longest_match(text)
    }

    /// Returns the longest regular expression match at start of `text`.
    pub fn longest_regex_matches(&self, text: &str) -> (Vec<H>, usize) {
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
    use std::rc::Rc;

    use super::*;
    use crate::analyzer::*;

    #[derive(PartialEq, Eq, Clone, Copy, Hash, Debug, PartialOrd, Ord)]
    enum Handle {
        If,
        When,
        Ident,
        Btextl,
        Pred,
        Literal,
        Action,
        Predicate,
        Code,
    }

    #[test]
    fn lexicon_ok() {
        use self::Handle::*;
        let lexicon = Lexicon::<Handle>::new(
            &[(If, "if"), (When, "when")],
            &[
                (Ident, "\\A[a-zA-Z]+[\\w_]*"),
                (Btextl, r"\A&\{(.|[\n\r])*&\}"),
                (Pred, r"\A\?\{(.|[\n\r])*\?\}"),
                (Literal, "\\A(\"\\S+\")"),
                (Action, r"\A(!\{(.|[\n\r])*?!\})"),
                (Predicate, r"\A(\?\((.|[\n\r])*?\?\))"),
                (Code, r"\A(%\{(.|[\n\r])*?%\})"),
            ],
            &[r"\A(/\*(.|[\n\r])*?\*/)", r"\A(//[^\n\r]*)", r"\A(\s+)"],
        );
        assert!(lexicon.is_ok());
    }

    #[test]
    fn lexicon_fail() {
        use self::Handle::*;
        let lexicon = Lexicon::<Handle>::new(
            &[(If, "if"), (If, "when")],
            &[
                (Ident, "\\A[a-zA-Z]+[\\w_]*"),
                (Btextl, r"\A&\{(.|[\n\r])*&\}"),
                (Pred, r"\A\?\{(.|[\n\r])*\?\}"),
                (Literal, "\\A(\"\\S+\")"),
                (Action, r"\A(!\{(.|[\n\r])*?!\})"),
                (Predicate, r"\A(\?\((.|[\n\r])*?\?\))"),
                (Code, r"\A(%\{(.|[\n\r])*?%\})"),
            ],
            &[r"\A(/\*(.|[\n\r])*?\*/)", r"\A(//[^\n\r]*)", r"\A(\s+)"],
        );
        if let Err(err) = lexicon {
            assert_eq!(err, LexanError::DuplicateHandle(If));
        } else {
            assert!(false)
        }

        let lexicon = Lexicon::<Handle>::new(
            &[(If, "if"), (When, "when")],
            &[
                (Action, "\\A[a-zA-Z]+[\\w_]*"),
                (Btextl, r"\A&\{(.|[\n\r])*&\}"),
                (Pred, r"\A\?\{(.|[\n\r])*\?\}"),
                (Literal, "\\A(\"\\S+\")"),
                (Action, r"\A(!\{(.|[\n\r])*?!\})"),
                (Predicate, r"\A(\?\((.|[\n\r])*?\?\))"),
                (Code, r"\A(%\{(.|[\n\r])*?%\})"),
            ],
            &[r"\A(/\*(.|[\n\r])*?\*/)", r"\A(//[^\n\r]*)", r"\A(\s+)"],
        );
        if let Err(err) = lexicon {
            assert_eq!(err, LexanError::DuplicateHandle(Action));
        } else {
            assert!(false)
        }

        let lexicon = Lexicon::<Handle>::new(
            &[(If, "if"), (When, "when")],
            &[
                (Ident, "\\A[a-zA-Z]+[\\w_]*"),
                (Btextl, r"\A&\{(.|[\n\r])*&\}"),
                (Pred, r"\A\?\{(.|[\n\r])*\?\}"),
                (Literal, "\\A(\"\\S+\")"),
                (When, r"\A(!\{(.|[\n\r])*?!\})"),
                (Predicate, r"\A(\?\((.|[\n\r])*?\?\))"),
                (Code, r"\A(%\{(.|[\n\r])*?%\})"),
            ],
            &[r"\A(/\*(.|[\n\r])*?\*/)", r"\A(//[^\n\r]*)", r"\A(\s+)"],
        );
        if let Err(err) = lexicon {
            assert_eq!(err, LexanError::DuplicateHandle(When));
        } else {
            assert!(false)
        }

        let lexicon = Lexicon::<Handle>::new(
            &[(If, "if"), (When, "if")],
            &[
                (Ident, "\\A[a-zA-Z]+[\\w_]*"),
                (Btextl, r"\A&\{(.|[\n\r])*&\}"),
                (Pred, r"\A\?\{(.|[\n\r])*\?\}"),
                (Literal, "\\A(\"\\S+\")"),
                (Action, r"\A(!\{(.|[\n\r])*?!\})"),
                (Predicate, r"\A(\?\((.|[\n\r])*?\?\))"),
                (Code, r"\A(%\{(.|[\n\r])*?%\})"),
            ],
            &[r"\A(/\*(.|[\n\r])*?\*/)", r"\A(//[^\n\r]*)", r"\A(\s+)"],
        );
        if let Err(err) = lexicon {
            assert_eq!(err, LexanError::DuplicatePattern("if"));
        } else {
            assert!(false)
        }

        let lexicon = Lexicon::<Handle>::new(
            &[(If, "if"), (When, "when")],
            &[
                (Ident, "\\A[a-zA-Z]+[\\w_]*"),
                (Btextl, r"\A&\{(.|[\n\r])*&\}"),
                (Pred, r"\A\?\{(.|[\n\r])*\?\}"),
                (Literal, "when"),
                (Action, r"\A(!\{(.|[\n\r])*?!\})"),
                (Predicate, r"\A(\?\((.|[\n\r])*?\?\))"),
                (Code, r"\A(%\{(.|[\n\r])*?%\})"),
            ],
            &[r"\A(/\*(.|[\n\r])*?\*/)", r"\A(//[^\n\r]*)", r"\A(\s+)"],
        );
        if let Err(err) = lexicon {
            assert_eq!(err, LexanError::DuplicatePattern("when"));
        } else {
            assert!(false)
        }

        let lexicon = Lexicon::<Handle>::new(
            &[(If, "if"), (When, "when")],
            &[
                (Ident, "\\A(\"\\S+\")"),
                (Btextl, r"\A&\{(.|[\n\r])*&\}"),
                (Pred, r"\A\?\{(.|[\n\r])*\?\}"),
                (Literal, "\\A(\"\\S+\")"),
                (Action, r"\A(!\{(.|[\n\r])*?!\})"),
                (Predicate, r"\A(\?\((.|[\n\r])*?\?\))"),
                (Code, r"\A(%\{(.|[\n\r])*?%\})"),
            ],
            &[r"\A(/\*(.|[\n\r])*?\*/)", r"\A(//[^\n\r]*)", r"\A(\s+)"],
        );
        if let Err(err) = lexicon {
            assert_eq!(err, LexanError::DuplicatePattern("\\A(\"\\S+\")"));
        } else {
            assert!(false)
        }

        let lexicon = Lexicon::<Handle>::new(
            &[(If, "if"), (When, "when")],
            &[
                (Ident, "\\A[a-zA-Z]+[\\w_]*"),
                (Btextl, r"\A&\{(.|[\n\r])*&\}"),
                (Pred, r"\A\?\{(.|[\n\r])*\?\}"),
                (Literal, "\\A(\"\\S+\")"),
                (Action, r"\A(!\{(.|[\n\r])*?!\})"),
                (Predicate, r"\A(\?\((.|[\n\r])*?\?\))"),
                (Code, r"\A(%\{(.|[\n\r])*?%\})"),
            ],
            &[
                r"\A(/\*(.|[\n\r])*?\*/)",
                r"\A(//[^\n\r]*)",
                "\\A(\"\\S+\")",
            ],
        );
        if let Err(err) = lexicon {
            assert_eq!(err, LexanError::DuplicatePattern("\\A(\"\\S+\")"));
        } else {
            assert!(false)
        }

        let lexicon = Lexicon::<Handle>::new(
            &[(If, "if"), (When, "when")],
            &[
                (Ident, "\\A[a-zA-Z]+[\\w_]*"),
                (Btextl, r"\A&\{(.|[\n\r])*&\}"),
                (Pred, r"\A\?\{(.|[\n\r])*\?\}"),
                (Literal, "\\A(\"\\S+\")"),
                (Action, r"\A(!\{(.|[\n\r])*?!\})"),
                (Predicate, r"(\?\((.|[\n\r])*?\?\))"),
                (Code, r"\A(%\{(.|[\n\r])*?%\})"),
            ],
            &[r"\A(/\*(.|[\n\r])*?\*/)", r"\A(//[^\n\r]*)", r"\A(\s+)"],
        );
        if let Err(err) = lexicon {
            assert_eq!(err, LexanError::UnanchoredRegex(r"(\?\((.|[\n\r])*?\?\))"));
        } else {
            assert!(false)
        }
    }
}
