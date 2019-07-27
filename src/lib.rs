extern crate regex;

use std::fmt::Debug;
use std::rc::Rc;

mod analyzer;
mod error;
mod lexicon;
mod matcher;

pub use analyzer::{Error, InjectableTokenStream, Location, Token, TokenStream};
use lexicon::Lexicon;

pub struct LexicalAnalyzer<H>
where
    H: Ord + Copy + PartialEq + Debug,
{
    lexicon: Rc<Lexicon<H>>,
}

impl<H> LexicalAnalyzer<H>
where
    H: Ord + Copy + PartialEq + Debug,
{
    pub fn new<'a>(
        literal_lexemes: &[(H, &'a str)],
        regex_lexemes: &[(H, &'a str)],
        skip_regex_strs: &[&'a str],
    ) -> Self {
        let lexicon = match Lexicon::new(literal_lexemes, regex_lexemes, skip_regex_strs) {
            Ok(lexicon) => Rc::new(lexicon),
            Err(err) => panic!("Fatal Error: {:?}", err),
        };
        Self { lexicon }
    }

    pub fn token_stream<'a>(&self, text: &'a str, label: &'a str) -> TokenStream<'a, H> {
        TokenStream::new(&self.lexicon, text, label)
    }

    pub fn injectable_token_stream<'a>(
        &self,
        text: &'a str,
        label: &'a str,
    ) -> InjectableTokenStream<'a, H> {
        InjectableTokenStream::new(&self.lexicon, text, label)
    }
}

#[cfg(test)]
mod tests {
    use crate::analyzer::Error;

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
    fn lexical_analyser() {
        use Handle::*;

        let lexan = super::LexicalAnalyzer::new(
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

        let mut token_stream = lexan.token_stream(
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

        let mut injectable_token_stream = lexan.injectable_token_stream(
            "if iffy\n \"quoted\" \"if\" \n9 $ \tname &{ one \n two &} and so ?{on?}",
            "raw text",
        );

        match injectable_token_stream.next().unwrap() {
            Ok(token) => {
                assert_eq!(*token.handle(), If);
                assert_eq!(token.matched_text(), "if");
                assert_eq!(format!("{}", token.location()), "\"raw text\":1:1");
            }
            _ => assert!(false),
        };
        match injectable_token_stream.next().unwrap() {
            Ok(token) => {
                assert_eq!(*token.handle(), Ident);
                assert_eq!(token.matched_text(), "iffy");
                assert_eq!(format!("{}", token.location()), "\"raw text\":1:4");
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

        match injectable_token_stream.next().unwrap() {
            Ok(token) => {
                assert_eq!(*token.handle(), Literal);
                assert_eq!(token.matched_text(), "\"quoted\"");
                assert_eq!(format!("{}", token.location()), "\"raw text\":2:2");
            }
            _ => assert!(false),
        };
        match injectable_token_stream.next().unwrap() {
            Ok(token) => {
                assert_eq!(*token.handle(), Literal);
                assert_eq!(token.matched_text(), "\"if\"");
                assert_eq!(format!("{}", token.location()), "\"raw text\":2:11");
            }
            _ => assert!(false),
        };
        injectable_token_stream.inject("if one \"name\"", "\"injected text\"");
        match injectable_token_stream.next().unwrap() {
            Ok(token) => {
                assert_eq!(*token.handle(), If);
                assert_eq!(token.matched_text(), "if");
                assert_eq!(format!("{}", token.location()), "\"\"injected text\"\":1:1");
            }
            _ => assert!(false),
        };
        match injectable_token_stream.next().unwrap() {
            Ok(token) => {
                assert_eq!(*token.handle(), Ident);
                assert_eq!(token.matched_text(), "one");
                assert_eq!(format!("{}", token.location()), "\"\"injected text\"\":1:4");
            }
            _ => assert!(false),
        };
        injectable_token_stream.inject("  two", "another text");
        match injectable_token_stream.next().unwrap() {
            Ok(token) => {
                assert_eq!(*token.handle(), Ident);
                assert_eq!(token.matched_text(), "two");
                assert_eq!(format!("{}", token.location()), "\"another text\":1:3");
            }
            _ => assert!(false),
        };
        match injectable_token_stream.next().unwrap() {
            Ok(token) => {
                assert_eq!(*token.handle(), Literal);
                assert_eq!(token.matched_text(), "\"name\"");
                assert_eq!(format!("{}", token.location()), "\"\"injected text\"\":1:8");
            }
            _ => assert!(false),
        };
        injectable_token_stream.inject("   three", "yet another text");
        match injectable_token_stream.next().unwrap() {
            Ok(token) => {
                assert_eq!(*token.handle(), Ident);
                assert_eq!(token.matched_text(), "three");
                assert_eq!(format!("{}", token.location()), "\"yet another text\":1:4");
            }
            _ => assert!(false),
        };
        match injectable_token_stream.next().unwrap() {
            Err(err) => match err {
                Error::UnexpectedText(text, location) => {
                    assert_eq!(text, "9");
                    assert_eq!(format!("{}", location), "\"raw text\":3:1");
                }
                _ => assert!(false),
            },
            _ => assert!(false),
        };
        match injectable_token_stream.next().unwrap() {
            Err(err) => match err {
                Error::UnexpectedText(text, location) => {
                    assert_eq!(text, "$");
                    assert_eq!(format!("{}", location), "\"raw text\":3:3");
                }
                _ => assert!(false),
            },
            _ => assert!(false),
        };
        match injectable_token_stream.next().unwrap() {
            Ok(token) => {
                assert_eq!(*token.handle(), Ident);
                assert_eq!(token.matched_text(), "name");
                assert_eq!(format!("{}", token.location()), "\"raw text\":3:6");
            }
            _ => assert!(false),
        };
        match injectable_token_stream.next().unwrap() {
            Ok(token) => {
                assert_eq!(*token.handle(), Btextl);
                assert_eq!(token.matched_text(), "&{ one \n two &}");
                assert_eq!(format!("{}", token.location()), "\"raw text\":3:11");
            }
            _ => assert!(false),
        };
        match injectable_token_stream.next().unwrap() {
            Ok(token) => {
                assert_eq!(*token.handle(), Ident);
                assert_eq!(token.matched_text(), "and");
                assert_eq!(format!("{}", token.location()), "\"raw text\":4:9");
            }
            _ => assert!(false),
        };
        match injectable_token_stream.next().unwrap() {
            Ok(token) => {
                assert_eq!(*token.handle(), Ident);
                assert_eq!(token.matched_text(), "so");
                assert_eq!(format!("{}", token.location()), "\"raw text\":4:13");
            }
            _ => assert!(false),
        };
        match injectable_token_stream.next().unwrap() {
            Ok(token) => {
                assert_eq!(*token.handle(), Pred);
                assert_eq!(token.matched_text(), "?{on?}");
                assert_eq!(format!("{}", token.location()), "\"raw text\":4:16");
            }
            _ => assert!(false),
        };
        assert!(injectable_token_stream.next().is_none());
    }
}
