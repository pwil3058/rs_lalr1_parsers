use std::fmt::Debug;

use regex::Regex;

use crate::analyzer::{InjectableTokenStream, TokenStream};
use crate::error::LexanError;
use crate::matcher::RegexMatcher;
use crate::LiteralMatcher;

#[derive(Default)]
pub struct Lexicon<H>
where
    H: Copy + PartialEq + Debug,
{
    literal_matcher: LiteralMatcher<H>,
    regex_matcher: RegexMatcher<H>,
    skip_regexes: Vec<Regex>,
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
        let literal_matcher = LiteralMatcher::new(literal_lexemes)?;
        let regex_matcher = RegexMatcher::new(regex_lexemes)?;
        let mut skip_regexes = vec![];
        for skip_regex_str in skip_regex_strs.iter() {
            skip_regexes.push(Regex::new(skip_regex_str)?);
        }
        Ok(Self {
            literal_matcher,
            regex_matcher,
            skip_regexes,
        })
    }

    /// Returns number of skippable bytes at start of `text`.
    pub fn skippable_count(&self, text: &str) -> usize {
        let mut index = 0;
        while index < text.len() {
            let mut skips = 0;
            for skip_regex in self.skip_regexes.iter() {
                if let Some(m) = skip_regex.find_at(text, index) {
                    if m.start() == index {
                        index = m.end();
                        skips += 1;
                    }
                }
            }
            if skips == 0 {
                break;
            }
        }
        index
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
            for regex in self.skip_regexes.iter() {
                if let Some(m) = regex.find_at(text, index) {
                    if m.start() == index {
                        return index;
                    }
                }
            }
        }
        text.len()
    }

    pub fn token_stream<'a>(&'a self, text: &'a str, label: &'a str) -> TokenStream<'a, H> {
        TokenStream::new(self, text, label)
    }

    pub fn injectable_token_stream<'a>(
        &'a self,
        text: &'a str,
        label: &'a str,
    ) -> InjectableTokenStream<'a, H> {
        InjectableTokenStream::new(self, text, label)
    }
}

#[cfg(test)]
mod tests {
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
        let lexicon = Lexicon::<Handle>::new(
            &[(If, "if"), (When, "when")],
            &[
                (Ident, "^[a-zA-Z]+[\\w_]*"),
                (Btextl, r"^&\{(.|[\n\r])*&\}"),
                (Pred, r"^\?\{(.|[\n\r])*\?\}"),
                (Literal, "^(\"\\S+\")"),
                (Action, r"^(!\{(.|[\n\r])*?!\})"),
                (Predicate, r"^(\?\((.|[\n\r])*?\?\))"),
                (Code, r"^(%\{(.|[\n\r])*?%\})"),
            ],
            &[r"^(/\*(.|[\n\r])*?\*/)", r"^(//[^\n\r]*)", r"^(\s+)"],
        )
        .unwrap();
        let mut token_stream = lexicon.token_stream(
            "if iffy\n \"quoted\" \"if\" \n9 $ \tname &{ one \n two &} and so ?{on?}",
            "raw text",
        );
        match token_stream.next().unwrap() {
            Token::Valid(handle, text, locn) => {
                assert_eq!(handle, If);
                assert_eq!(text, "if");
                assert_eq!(format!("{}", locn), "raw text:1(1)");
            }
            _ => assert!(false),
        };
        match token_stream.next().unwrap() {
            Token::Valid(handle, text, locn) => {
                assert_eq!(handle, Ident);
                assert_eq!(text, "iffy");
                assert_eq!(format!("{}", locn), "raw text:1(4)");
            }
            _ => assert!(false),
        };
        match token_stream.next().unwrap() {
            Token::Valid(handle, text, locn) => {
                assert_eq!(handle, Literal);
                assert_eq!(text, "\"quoted\"");
                assert_eq!(format!("{}", locn), "raw text:2(2)");
            }
            _ => assert!(false),
        };
        match token_stream.next().unwrap() {
            Token::Valid(handle, text, locn) => {
                assert_eq!(handle, Literal);
                assert_eq!(text, "\"if\"");
                assert_eq!(format!("{}", locn), "raw text:2(11)");
            }
            _ => assert!(false),
        };
        match token_stream.next().unwrap() {
            Token::UnexpectedText(text, locn) => {
                assert_eq!(text, "9 $ \t");
                assert_eq!(format!("{}", locn), "raw text:3(1)");
            }
            _ => assert!(false),
        };
        match token_stream.next().unwrap() {
            Token::Valid(handle, text, locn) => {
                assert_eq!(handle, Ident);
                assert_eq!(text, "name");
                assert_eq!(format!("{}", locn), "raw text:3(6)");
            }
            _ => assert!(false),
        };
        match token_stream.next().unwrap() {
            Token::Valid(handle, text, locn) => {
                assert_eq!(handle, Btextl);
                assert_eq!(text, "&{ one \n two &}");
                assert_eq!(format!("{}", locn), "raw text:3(11)");
            }
            _ => assert!(false),
        };
        match token_stream.next().unwrap() {
            Token::Valid(handle, text, locn) => {
                assert_eq!(handle, Ident);
                assert_eq!(text, "and");
                assert_eq!(format!("{}", locn), "raw text:4(9)");
            }
            _ => assert!(false),
        };
        match token_stream.next().unwrap() {
            Token::Valid(handle, text, locn) => {
                assert_eq!(handle, Ident);
                assert_eq!(text, "so");
                assert_eq!(format!("{}", locn), "raw text:4(13)");
            }
            _ => assert!(false),
        };
        match token_stream.next().unwrap() {
            Token::Valid(handle, text, locn) => {
                assert_eq!(handle, Pred);
                assert_eq!(text, "?{on?}");
                assert_eq!(format!("{}", locn), "raw text:4(16)");
            }
            _ => assert!(false),
        };
        assert!(token_stream.next().is_none());
    }

    #[test]
    fn streamer_injectable() {
        use self::Handle::*;
        let lexicon = Lexicon::<Handle>::new(
            &[(If, "if"), (When, "when")],
            &[
                (Ident, "^[a-zA-Z]+[\\w_]*"),
                (Btextl, r"^&\{(.|[\n\r])*&\}"),
                (Pred, r"^\?\{(.|[\n\r])*\?\}"),
                (Literal, "^(\"\\S+\")"),
                (Action, r"^(!\{(.|[\n\r])*?!\})"),
                (Predicate, r"^(\?\((.|[\n\r])*?\?\))"),
                (Code, r"^(%\{(.|[\n\r])*?%\})"),
            ],
            &[r"^(/\*(.|[\n\r])*?\*/)", r"^(//[^\n\r]*)", r"^(\s+)"],
        )
        .unwrap();
        let mut token_stream = lexicon.injectable_token_stream(
            "if iffy\n \"quoted\" \"if\" \n9 $ \tname &{ one \n two &} and so ?{on?}",
            "raw text",
        );
        match token_stream.next().unwrap() {
            Token::Valid(handle, text, locn) => {
                assert_eq!(handle, If);
                assert_eq!(text, "if");
                assert_eq!(format!("{}", locn), "raw text:1(1)");
            }
            _ => assert!(false),
        };
        match token_stream.next().unwrap() {
            Token::Valid(handle, text, locn) => {
                assert_eq!(handle, Ident);
                assert_eq!(text, "iffy");
                assert_eq!(format!("{}", locn), "raw text:1(4)");
            }
            _ => assert!(false),
        };
        match token_stream.next().unwrap() {
            Token::Valid(handle, text, locn) => {
                assert_eq!(handle, Literal);
                assert_eq!(text, "\"quoted\"");
                assert_eq!(format!("{}", locn), "raw text:2(2)");
            }
            _ => assert!(false),
        };
        match token_stream.next().unwrap() {
            Token::Valid(handle, text, locn) => {
                assert_eq!(handle, Literal);
                assert_eq!(text, "\"if\"");
                assert_eq!(format!("{}", locn), "raw text:2(11)");
            }
            _ => assert!(false),
        };
        token_stream.inject("if one \"name\"", "injected text");
        match token_stream.next().unwrap() {
            Token::Valid(handle, text, locn) => {
                assert_eq!(handle, If);
                assert_eq!(text, "if");
                assert_eq!(format!("{}", locn), "injected text:1(1)");
            }
            _ => assert!(false),
        };
        match token_stream.next().unwrap() {
            Token::Valid(handle, text, locn) => {
                assert_eq!(handle, Ident);
                assert_eq!(text, "one");
                assert_eq!(format!("{}", locn), "injected text:1(4)");
            }
            _ => assert!(false),
        };
        token_stream.inject("  two", "another text");
        match token_stream.next().unwrap() {
            Token::Valid(handle, text, locn) => {
                assert_eq!(handle, Ident);
                assert_eq!(text, "two");
                assert_eq!(format!("{}", locn), "another text:1(3)");
            }
            _ => assert!(false),
        };
        match token_stream.next().unwrap() {
            Token::Valid(handle, text, locn) => {
                assert_eq!(handle, Literal);
                assert_eq!(text, "\"name\"");
                assert_eq!(format!("{}", locn), "injected text:1(8)");
            }
            _ => assert!(false),
        };
        token_stream.inject("   three", "yet another text");
        match token_stream.next().unwrap() {
            Token::Valid(handle, text, locn) => {
                assert_eq!(handle, Ident);
                assert_eq!(text, "three");
                assert_eq!(format!("{}", locn), "yet another text:1(4)");
            }
            _ => assert!(false),
        };
        match token_stream.next().unwrap() {
            Token::UnexpectedText(text, locn) => {
                assert_eq!(text, "9 $ \t");
                assert_eq!(format!("{}", locn), "raw text:3(1)");
            }
            _ => assert!(false),
        };
        match token_stream.next().unwrap() {
            Token::Valid(handle, text, locn) => {
                assert_eq!(handle, Ident);
                assert_eq!(text, "name");
                assert_eq!(format!("{}", locn), "raw text:3(6)");
            }
            _ => assert!(false),
        };
        match token_stream.next().unwrap() {
            Token::Valid(handle, text, locn) => {
                assert_eq!(handle, Btextl);
                assert_eq!(text, "&{ one \n two &}");
                assert_eq!(format!("{}", locn), "raw text:3(11)");
            }
            _ => assert!(false),
        };
        match token_stream.next().unwrap() {
            Token::Valid(handle, text, locn) => {
                assert_eq!(handle, Ident);
                assert_eq!(text, "and");
                assert_eq!(format!("{}", locn), "raw text:4(9)");
            }
            _ => assert!(false),
        };
        match token_stream.next().unwrap() {
            Token::Valid(handle, text, locn) => {
                assert_eq!(handle, Ident);
                assert_eq!(text, "so");
                assert_eq!(format!("{}", locn), "raw text:4(13)");
            }
            _ => assert!(false),
        };
        match token_stream.next().unwrap() {
            Token::Valid(handle, text, locn) => {
                assert_eq!(handle, Pred);
                assert_eq!(text, "?{on?}");
                assert_eq!(format!("{}", locn), "raw text:4(16)");
            }
            _ => assert!(false),
        };
        assert!(token_stream.next().is_none());
    }
}
