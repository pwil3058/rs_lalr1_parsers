extern crate regex;

pub use std::fmt::{Debug, Display};
use std::sync::Arc;

mod analyzer;
mod error;
mod lexicon;
mod matcher;

pub use analyzer::{Error, Location, Token, TokenStream};
use lexicon::Lexicon;

pub struct LexicalAnalyzer<T>
where
    T: Ord + Copy + PartialEq + Debug + Display,
{
    lexicon: Arc<Lexicon<T>>,
}

impl<T> LexicalAnalyzer<T>
where
    T: Ord + Copy + PartialEq + Debug + Display,
{
    pub fn new<'a>(
        literal_lexemes: &[(T, &'a str)],
        regex_lexemes: &[(T, &'a str)],
        skip_regex_strs: &[&'a str],
        end_marker: T,
    ) -> Self {
        let lexicon = match Lexicon::new(literal_lexemes, regex_lexemes, skip_regex_strs, end_marker) {
            Ok(lexicon) => Arc::new(lexicon),
            Err(err) => panic!("Fatal Error: {:?}", err),
        };
        Self { lexicon }
    }

    pub fn token_stream(&self, text: String, label: String) -> TokenStream<T> {
        TokenStream::new(&self.lexicon, text, label)
    }
}

#[cfg(test)]
mod tests {
    use crate::analyzer::Error;

    #[derive(PartialEq, Eq, Clone, Copy, Hash, PartialOrd, Ord, Debug)]
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
        End,
    }

    impl std::fmt::Display for Handle {
        fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
            use Handle::*;
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
    fn lexical_analyser() {
        use Handle::*;

        let lexan = super::LexicalAnalyzer::new(
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

        let mut token_stream = lexan.token_stream(
            "if iffy\n \"quoted\" \"if\" \n9 $ \tname &{ one \n two &} and so ?{on?}".to_string(),
            "raw text".to_string(),
        );

        match token_stream.front_advance() {
            Ok(token) => {
                assert_eq!(*token.tag(), If);
                assert_eq!(token.lexeme(), "if");
                assert_eq!(format!("{}", token.location()), "\"raw text\":1:1");
            }
            _ => assert!(false),
        };
        match token_stream.front_advance() {
            Ok(token) => {
                assert_eq!(*token.tag(), Ident);
                assert_eq!(token.lexeme(), "iffy");
                assert_eq!(format!("{}", token.location()), "\"raw text\":1:4");
            }
            _ => assert!(false),
        };
        match token_stream.front_advance() {
            Ok(token) => {
                assert_eq!(*token.tag(), Literal);
                assert_eq!(token.lexeme(), "\"quoted\"");
                assert_eq!(format!("{}", token.location()), "\"raw text\":2:2");
            }
            _ => assert!(false),
        };
        match token_stream.front_advance() {
            Ok(token) => {
                assert_eq!(*token.tag(), Literal);
                assert_eq!(token.lexeme(), "\"if\"");
                assert_eq!(format!("{}", token.location()), "\"raw text\":2:11");
            }
            _ => assert!(false),
        };
        match token_stream.front_advance() {
            Err(err) => match err {
                Error::UnexpectedText(text, location) => {
                    assert_eq!(text, "9");
                    assert_eq!(format!("{}", location), "\"raw text\":3:1");
                }
                _ => assert!(false),
            },
            _ => assert!(false),
        };
        match token_stream.front_advance() {
            Err(err) => match err {
                Error::UnexpectedText(text, location) => {
                    assert_eq!(text, "$");
                    assert_eq!(format!("{}", location), "\"raw text\":3:3");
                }
                _ => assert!(false),
            },
            _ => assert!(false),
        };
        match token_stream.front_advance() {
            Ok(token) => {
                assert_eq!(*token.tag(), Ident);
                assert_eq!(token.lexeme(), "name");
                assert_eq!(format!("{}", token.location()), "\"raw text\":3:6");
            }
            _ => assert!(false),
        };
        match token_stream.front_advance() {
            Ok(token) => {
                assert_eq!(*token.tag(), Btextl);
                assert_eq!(token.lexeme(), "&{ one \n two &}");
                assert_eq!(format!("{}", token.location()), "\"raw text\":3:11");
            }
            _ => assert!(false),
        };
        match token_stream.front_advance() {
            Ok(token) => {
                assert_eq!(*token.tag(), Ident);
                assert_eq!(token.lexeme(), "and");
                assert_eq!(format!("{}", token.location()), "\"raw text\":4:9");
            }
            _ => assert!(false),
        };
        match token_stream.front_advance() {
            Ok(token) => {
                assert_eq!(*token.tag(), Ident);
                assert_eq!(token.lexeme(), "so");
                assert_eq!(format!("{}", token.location()), "\"raw text\":4:13");
            }
            _ => assert!(false),
        };

        let mut second_token_stream = lexan.token_stream(
            "if iffy\n \"quoted\" \"if\" \n9 $ \tname &{ one \n two &} and so ?{on?}".to_string(),
            "raw text".to_string(),
        );

        match second_token_stream.front_advance() {
            Ok(token) => {
                assert_eq!(*token.tag(), If);
                assert_eq!(token.lexeme(), "if");
                assert_eq!(format!("{}", token.location()), "\"raw text\":1:1");
            }
            _ => assert!(false),
        };
        match second_token_stream.front_advance() {
            Ok(token) => {
                assert_eq!(*token.tag(), Ident);
                assert_eq!(token.lexeme(), "iffy");
                assert_eq!(format!("{}", token.location()), "\"raw text\":1:4");
            }
            _ => assert!(false),
        };

        match token_stream.front_advance() {
            Ok(token) => {
                assert_eq!(*token.tag(), Pred);
                assert_eq!(token.lexeme(), "?{on?}");
                assert_eq!(format!("{}", token.location()), "\"raw text\":4:16");
            }
            _ => assert!(false),
        };

        match token_stream.front_advance() {
            Ok(token) => {
                assert_eq!(*token.tag(), End);
                assert_eq!(token.lexeme(), "");
                assert_eq!(format!("{}", token.location()), "\"raw text\":4:22");
            }
            _ => assert!(false),
        };

        match second_token_stream.front_advance() {
            Ok(token) => {
                assert_eq!(*token.tag(), Literal);
                assert_eq!(token.lexeme(), "\"quoted\"");
                assert_eq!(format!("{}", token.location()), "\"raw text\":2:2");
            }
            _ => assert!(false),
        };
        match second_token_stream.front_advance() {
            Ok(token) => {
                assert_eq!(*token.tag(), Literal);
                assert_eq!(token.lexeme(), "\"if\"");
                assert_eq!(format!("{}", token.location()), "\"raw text\":2:11");
            }
            _ => assert!(false),
        };
        second_token_stream.inject("if one \"name\"".to_string(), "\"injected text\"".to_string());
        match second_token_stream.front_advance() {
            Ok(token) => {
                assert_eq!(*token.tag(), If);
                assert_eq!(token.lexeme(), "if");
                assert_eq!(format!("{}", token.location()), "\"\"injected text\"\":1:1");
            }
            _ => assert!(false),
        };
        match second_token_stream.front_advance() {
            Ok(token) => {
                assert_eq!(*token.tag(), Ident);
                assert_eq!(token.lexeme(), "one");
                assert_eq!(format!("{}", token.location()), "\"\"injected text\"\":1:4");
            }
            _ => assert!(false),
        };
        second_token_stream.inject("  two".to_string(), "another text".to_string());
        match second_token_stream.front_advance() {
            Ok(token) => {
                assert_eq!(*token.tag(), Ident);
                assert_eq!(token.lexeme(), "two");
                assert_eq!(format!("{}", token.location()), "\"another text\":1:3");
            }
            _ => assert!(false),
        };
        match second_token_stream.front_advance() {
            Ok(token) => {
                assert_eq!(*token.tag(), Literal);
                assert_eq!(token.lexeme(), "\"name\"");
                assert_eq!(format!("{}", token.location()), "\"\"injected text\"\":1:8");
            }
            _ => assert!(false),
        };
        second_token_stream.inject("   three".to_string(), "yet another text".to_string());
        match second_token_stream.front_advance() {
            Ok(token) => {
                assert_eq!(*token.tag(), Ident);
                assert_eq!(token.lexeme(), "three");
                assert_eq!(format!("{}", token.location()), "\"yet another text\":1:4");
            }
            _ => assert!(false),
        };
        match second_token_stream.front_advance() {
            Err(err) => match err {
                Error::UnexpectedText(text, location) => {
                    assert_eq!(text, "9");
                    assert_eq!(format!("{}", location), "\"raw text\":3:1");
                }
                _ => assert!(false),
            },
            _ => assert!(false),
        };
        match second_token_stream.front_advance() {
            Err(err) => match err {
                Error::UnexpectedText(text, location) => {
                    assert_eq!(text, "$");
                    assert_eq!(format!("{}", location), "\"raw text\":3:3");
                }
                _ => assert!(false),
            },
            _ => assert!(false),
        };
        match second_token_stream.front_advance() {
            Ok(token) => {
                assert_eq!(*token.tag(), Ident);
                assert_eq!(token.lexeme(), "name");
                assert_eq!(format!("{}", token.location()), "\"raw text\":3:6");
            }
            _ => assert!(false),
        };
        match second_token_stream.front_advance() {
            Ok(token) => {
                assert_eq!(*token.tag(), Btextl);
                assert_eq!(token.lexeme(), "&{ one \n two &}");
                assert_eq!(format!("{}", token.location()), "\"raw text\":3:11");
            }
            _ => assert!(false),
        };
        match second_token_stream.front_advance() {
            Ok(token) => {
                assert_eq!(*token.tag(), Ident);
                assert_eq!(token.lexeme(), "and");
                assert_eq!(format!("{}", token.location()), "\"raw text\":4:9");
            }
            _ => assert!(false),
        };
        match second_token_stream.front_advance() {
            Ok(token) => {
                assert_eq!(*token.tag(), Ident);
                assert_eq!(token.lexeme(), "so");
                assert_eq!(format!("{}", token.location()), "\"raw text\":4:13");
            }
            _ => assert!(false),
        };
        match second_token_stream.front_advance() {
            Ok(token) => {
                assert_eq!(*token.tag(), Pred);
                assert_eq!(token.lexeme(), "?{on?}");
                assert_eq!(format!("{}", token.location()), "\"raw text\":4:16");
            }
            _ => assert!(false),
        };
        match second_token_stream.front_advance() {
            Ok(token) => {
                assert_eq!(*token.tag(), End);
                assert_eq!(token.lexeme(), "");
                assert_eq!(format!("{}", token.location()), "\"raw text\":4:22");
            }
            _ => assert!(false),
        };
        assert!(second_token_stream.advance_front().is_err());
    }
}
