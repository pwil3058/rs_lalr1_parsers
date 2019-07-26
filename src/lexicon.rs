use std::fmt::Debug;

use crate::error::LexanError;
use crate::matcher::{LiteralMatcher, RegexMatcher, SkipMatcher};

#[derive(Default)]
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
        let mut literals = vec![];
        let mut regexes = vec![];
        for (handle, literal) in literal_lexemes.iter() {
            if literal.len() == 0 {
                return Err(LexanError::EmptyPattern(*handle));
            }
            match handles.binary_search(handle) {
                Ok(_) => return Err(LexanError::DuplicateHandle(*handle)),
                Err(index) => handles.insert(index, *handle),
            }
            match literals.binary_search(literal) {
                Ok(_) => return Err(LexanError::DuplicatePattern(literal)),
                Err(index) => literals.insert(index, literal),
            }
        }
        for (handle, regex) in regex_lexemes.iter() {
            if regex.len() == 0 {
                return Err(LexanError::EmptyPattern(*handle));
            }
            match handles.binary_search(handle) {
                Ok(_) => return Err(LexanError::DuplicateHandle(*handle)),
                Err(index) => handles.insert(index, *handle),
            }
            match regexes.binary_search(regex) {
                Ok(_) => return Err(LexanError::DuplicatePattern(regex)),
                Err(index) => regexes.insert(index, regex),
            }
        }
        for regex in skip_regex_strs.iter() {
            match regexes.binary_search(regex) {
                Ok(_) => return Err(LexanError::DuplicatePattern(regex)),
                Err(index) => regexes.insert(index, regex),
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
    fn streamer_basic() {
        use self::Handle::*;
        let lexicon = Rc::new(Lexicon::<Handle>::new(
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
        )
        .unwrap());
        let mut token_stream = TokenStream::new(
            &lexicon,
            "if iffy\n \"quoted\" \"if\" \n9 $ \tname &{ one \n two &} and so ?{on?}",
            "raw text",
        );
        match token_stream.next().unwrap() {
            Ok(token) => {
                assert_eq!(*token.handle(), If);
                assert_eq!(token.matched_text(), "if");
                assert_eq!(format!("{}", token.location()), "\"raw text\":1:1");
            }
            _ => assert!(false),
        };
        match token_stream.next().unwrap() {
            Ok(token) => {
                assert_eq!(*token.handle(), Ident);
                assert_eq!(token.matched_text(), "iffy");
                assert_eq!(format!("{}", token.location()), "\"raw text\":1:4");
            }
            _ => assert!(false),
        };
        match token_stream.next().unwrap() {
            Ok(token) => {
                assert_eq!(*token.handle(), Literal);
                assert_eq!(token.matched_text(), "\"quoted\"");
                assert_eq!(format!("{}", token.location()), "\"raw text\":2:2");
            }
            _ => assert!(false),
        };
        match token_stream.next().unwrap() {
            Ok(token) => {
                assert_eq!(*token.handle(), Literal);
                assert_eq!(token.matched_text(), "\"if\"");
                assert_eq!(format!("{}", token.location()), "\"raw text\":2:11");
            }
            _ => assert!(false),
        };
        match token_stream.next().unwrap() {
            Err(err) => match err {
                Error::UnexpectedText(text, location) => {
                    assert_eq!(text, "9");
                    assert_eq!(format!("{}", location), "\"raw text\":3:1");
                }
                _ => assert!(false),
            },
            _ => assert!(false),
        };
        match token_stream.next().unwrap() {
            Err(err) => match err {
                Error::UnexpectedText(text, location) => {
                    assert_eq!(text, "$");
                    assert_eq!(format!("{}", location), "\"raw text\":3:3");
                }
                _ => assert!(false),
            },
            _ => assert!(false),
        };
        match token_stream.next().unwrap() {
            Ok(token) => {
                assert_eq!(*token.handle(), Ident);
                assert_eq!(token.matched_text(), "name");
                assert_eq!(format!("{}", token.location()), "\"raw text\":3:6");
            }
            _ => assert!(false),
        };
        match token_stream.next().unwrap() {
            Ok(token) => {
                assert_eq!(*token.handle(), Btextl);
                assert_eq!(token.matched_text(), "&{ one \n two &}");
                assert_eq!(format!("{}", token.location()), "\"raw text\":3:11");
            }
            _ => assert!(false),
        };
        match token_stream.next().unwrap() {
            Ok(token) => {
                assert_eq!(*token.handle(), Ident);
                assert_eq!(token.matched_text(), "and");
                assert_eq!(format!("{}", token.location()), "\"raw text\":4:9");
            }
            _ => assert!(false),
        };
        match token_stream.next().unwrap() {
            Ok(token) => {
                assert_eq!(*token.handle(), Ident);
                assert_eq!(token.matched_text(), "so");
                assert_eq!(format!("{}", token.location()), "\"raw text\":4:13");
            }
            _ => assert!(false),
        };
        match token_stream.next().unwrap() {
            Ok(token) => {
                assert_eq!(*token.handle(), Pred);
                assert_eq!(token.matched_text(), "?{on?}");
                assert_eq!(format!("{}", token.location()), "\"raw text\":4:16");
            }
            _ => assert!(false),
        };
        assert!(token_stream.next().is_none());
    }

    #[test]
    fn streamer_injectable() {
        use self::Handle::*;
        let lexicon = Rc::new(Lexicon::<Handle>::new(
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
        )
        .unwrap());
        let mut token_stream = InjectableTokenStream::new(
            &lexicon,
            "if iffy\n \"quoted\" \"if\" \n9 $ \tname &{ one \n two &} and so ?{on?}",
            "raw text",
        );
        match token_stream.next().unwrap() {
            Ok(token) => {
                assert_eq!(*token.handle(), If);
                assert_eq!(token.matched_text(), "if");
                assert_eq!(format!("{}", token.location()), "\"raw text\":1:1");
            }
            _ => assert!(false),
        };
        match token_stream.next().unwrap() {
            Ok(token) => {
                assert_eq!(*token.handle(), Ident);
                assert_eq!(token.matched_text(), "iffy");
                assert_eq!(format!("{}", token.location()), "\"raw text\":1:4");
            }
            _ => assert!(false),
        };
        match token_stream.next().unwrap() {
            Ok(token) => {
                assert_eq!(*token.handle(), Literal);
                assert_eq!(token.matched_text(), "\"quoted\"");
                assert_eq!(format!("{}", token.location()), "\"raw text\":2:2");
            }
            _ => assert!(false),
        };
        match token_stream.next().unwrap() {
            Ok(token) => {
                assert_eq!(*token.handle(), Literal);
                assert_eq!(token.matched_text(), "\"if\"");
                assert_eq!(format!("{}", token.location()), "\"raw text\":2:11");
            }
            _ => assert!(false),
        };
        token_stream.inject("if one \"name\"", "\"injected text\"");
        match token_stream.next().unwrap() {
            Ok(token) => {
                assert_eq!(*token.handle(), If);
                assert_eq!(token.matched_text(), "if");
                assert_eq!(format!("{}", token.location()), "\"\"injected text\"\":1:1");
            }
            _ => assert!(false),
        };
        match token_stream.next().unwrap() {
            Ok(token) => {
                assert_eq!(*token.handle(), Ident);
                assert_eq!(token.matched_text(), "one");
                assert_eq!(format!("{}", token.location()), "\"\"injected text\"\":1:4");
            }
            _ => assert!(false),
        };
        token_stream.inject("  two", "another text");
        match token_stream.next().unwrap() {
            Ok(token) => {
                assert_eq!(*token.handle(), Ident);
                assert_eq!(token.matched_text(), "two");
                assert_eq!(format!("{}", token.location()), "\"another text\":1:3");
            }
            _ => assert!(false),
        };
        match token_stream.next().unwrap() {
            Ok(token) => {
                assert_eq!(*token.handle(), Literal);
                assert_eq!(token.matched_text(), "\"name\"");
                assert_eq!(format!("{}", token.location()), "\"\"injected text\"\":1:8");
            }
            _ => assert!(false),
        };
        token_stream.inject("   three", "yet another text");
        match token_stream.next().unwrap() {
            Ok(token) => {
                assert_eq!(*token.handle(), Ident);
                assert_eq!(token.matched_text(), "three");
                assert_eq!(format!("{}", token.location()), "\"yet another text\":1:4");
            }
            _ => assert!(false),
        };
        match token_stream.next().unwrap() {
            Err(err) => match err {
                Error::UnexpectedText(text, location) => {
                    assert_eq!(text, "9");
                    assert_eq!(format!("{}", location), "\"raw text\":3:1");
                }
                _ => assert!(false),
            },
            _ => assert!(false),
        };
        match token_stream.next().unwrap() {
            Err(err) => match err {
                Error::UnexpectedText(text, location) => {
                    assert_eq!(text, "$");
                    assert_eq!(format!("{}", location), "\"raw text\":3:3");
                }
                _ => assert!(false),
            },
            _ => assert!(false),
        };
        match token_stream.next().unwrap() {
            Ok(token) => {
                assert_eq!(*token.handle(), Ident);
                assert_eq!(token.matched_text(), "name");
                assert_eq!(format!("{}", token.location()), "\"raw text\":3:6");
            }
            _ => assert!(false),
        };
        match token_stream.next().unwrap() {
            Ok(token) => {
                assert_eq!(*token.handle(), Btextl);
                assert_eq!(token.matched_text(), "&{ one \n two &}");
                assert_eq!(format!("{}", token.location()), "\"raw text\":3:11");
            }
            _ => assert!(false),
        };
        match token_stream.next().unwrap() {
            Ok(token) => {
                assert_eq!(*token.handle(), Ident);
                assert_eq!(token.matched_text(), "and");
                assert_eq!(format!("{}", token.location()), "\"raw text\":4:9");
            }
            _ => assert!(false),
        };
        match token_stream.next().unwrap() {
            Ok(token) => {
                assert_eq!(*token.handle(), Ident);
                assert_eq!(token.matched_text(), "so");
                assert_eq!(format!("{}", token.location()), "\"raw text\":4:13");
            }
            _ => assert!(false),
        };
        match token_stream.next().unwrap() {
            Ok(token) => {
                assert_eq!(*token.handle(), Pred);
                assert_eq!(token.matched_text(), "?{on?}");
                assert_eq!(format!("{}", token.location()), "\"raw text\":4:16");
            }
            _ => assert!(false),
        };
        assert!(token_stream.next().is_none());
    }
}
