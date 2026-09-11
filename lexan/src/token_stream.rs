// Copyright (c) 2026 Peter Williams <pwil3058@bigpond.net.au> <pwil3058@gmail.com>.

use std::{
    fmt::{Debug, Display},
    sync::Arc,
};

use crate::lexicon::Lexicon;
use crate::token::{Error, Location, Token, Tokens};

struct BasicTokenStream<T>
where
    T: Debug + Display + Copy + Eq + Ord,
{
    tokens: Tokens<T>,
    front: Option<Result<Token<T>, Error<T>>>,
}

impl<T> BasicTokenStream<T>
where
    T: Debug + Display + Copy + Eq + Ord,
{
    pub fn new(lexicon: &Arc<Lexicon<T>>, text: &str, label: &str) -> Self {
        let mut bts = Self {
            tokens: Tokens::new(lexicon, text, label),
            front: None,
        };
        bts.advance();
        bts
    }

    fn front(&self) -> Option<Result<Token<T>, Error<T>>> {
        self.front.clone()
    }

    fn is_empty(&self) -> bool {
        self.front.is_none()
    }

    fn advance(&mut self) {
        self.front = self.tokens.next();
    }

    fn location(&self) -> Location {
        self.tokens.location()
    }
}

pub struct TokenStream<T>
where
    T: Debug + Display + Copy + Eq + Ord,
{
    lexicon: Arc<Lexicon<T>>,
    token_stream_stack: Vec<BasicTokenStream<T>>,
    front: Result<Token<T>, Error<T>>,
}

impl<T> TokenStream<T>
where
    T: Debug + Display + Copy + Eq + Ord,
{
    pub(crate) fn new(lexicon: &Arc<Lexicon<T>>, text: &str, label: &str) -> Self {
        let mut stream = Self {
            lexicon: Arc::clone(lexicon),
            token_stream_stack: vec![],
            front: Err(Error::AdvancedWhenEmpty(Location::default())),
        };
        stream.inject(text, label);
        stream
    }

    pub fn is_empty(&self) -> bool {
        self.token_stream_stack.len() == 0
    }

    pub fn front(&self) -> Result<Token<T>, Error<T>> {
        self.front.clone()
    }

    pub fn inject(&mut self, text: &str, label: &str) {
        let token_stream = BasicTokenStream::new(&self.lexicon, text, label);
        if !token_stream.is_empty() {
            self.front = token_stream.front().unwrap();
            self.token_stream_stack.push(token_stream);
        }
    }

    pub fn advance(&mut self) {
        let mut i = self.token_stream_stack.len();
        if i > 0 {
            self.token_stream_stack[i - 1].advance();
            let mut popped = None;
            while i > 0 && self.token_stream_stack[i - 1].is_empty() {
                popped = self.token_stream_stack.pop();
                i -= 1;
            }
            self.front = if i > 0 {
                self.token_stream_stack[i - 1].front().unwrap()
            } else {
                let end_location = popped.unwrap().location();
                Ok(Token {
                    tag: self.lexicon.end_marker(),
                    lexeme: String::new(),
                    location: end_location,
                })
            }
        } else {
            let location = match &self.front {
                Ok(token) => token.location(),
                Err(err) => match err {
                    Error::UnexpectedText(_, location) => location,
                    Error::AmbiguousMatches(_, _, location) => location,
                    Error::AdvancedWhenEmpty(location) => location,
                },
            };
            self.front = Err(Error::AdvancedWhenEmpty(location.clone()))
        }
    }

    pub fn front_advance(&mut self) -> Result<Token<T>, Error<T>> {
        let front = self.front.clone();
        self.advance();
        front
    }

    pub fn advance_front(&mut self) -> Result<Token<T>, Error<T>> {
        self.advance();
        self.front.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lexicon::Lexicon;
    use std::fmt;

    #[test]
    fn token_stream_basics() {
        #[derive(PartialEq, Eq, Clone, Copy, Hash, Debug, PartialOrd, Ord)]
        enum Handle {
            If,
            When,
            Ident,
            End,
        }

        impl Display for Handle {
            fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
                use Handle::*;
                match self {
                    If => write!(f, "\"if\""),
                    When => write!(f, "\"when\""),
                    Ident => write!(f, "Ident"),
                    End => write!(f, "End"),
                }
            }
        }

        use Handle::*;
        let lexicon = Lexicon::new(
            &[(If, "if"), (When, "when")],
            &[(Ident, "[a-zA-Z]+[\\w_]*")],
            &[r"(/\*(.|[\n\r])*?\*/)", r"(//[^\n\r]*)", r"(\s+)"],
            End,
        );
        let lexicon = Arc::new(lexicon.unwrap());
        let text = "      ";
        let label = "label";
        let mut token_stream = TokenStream::new(&lexicon, text, label);
        assert!(token_stream.is_empty());
        assert!(token_stream.front().is_err());
        let text = " if nothing happens 9 ";
        let label = "another";
        token_stream.inject(text, label);
        assert!(!token_stream.is_empty());
        let token = Token {
            tag: If,
            lexeme: "if".to_string(),
            location: Location {
                index: 1,
                line_number: 1,
                offset: 2,
                label: "another".to_string(),
            },
        };
        assert_eq!((token_stream.front().clone()).unwrap(), token);
        assert_eq!((token_stream.front().clone()).unwrap(), token);
        token_stream.advance();
        let token = Token {
            tag: Ident,
            lexeme: "nothing".to_string(),
            location: Location {
                index: 4,
                line_number: 1,
                offset: 5,
                label: "another".to_string(),
            },
        };
        assert_eq!((token_stream.front().clone()).unwrap(), token);
        let text = "just";
        let label = "more";
        token_stream.inject(text, label);
        let token = Token {
            tag: Ident,
            lexeme: "just".to_string(),
            location: Location {
                index: 0,
                line_number: 1,
                offset: 1,
                label: "more".to_string(),
            },
        };
        assert_eq!((token_stream.front().clone()).unwrap(), token);
        token_stream.advance();
        let token = Token {
            tag: Ident,
            lexeme: "nothing".to_string(),
            location: Location {
                index: 4,
                line_number: 1,
                offset: 5,
                label: "another".to_string(),
            },
        };
        assert_eq!((token_stream.front().clone()).unwrap(), token);
        token_stream.advance();
        assert!(token_stream.front().is_ok());
        token_stream.advance();
        assert!(token_stream.front().is_err());
        token_stream.advance();
        let token = Token {
            tag: End,
            lexeme: "".to_string(),
            location: Location {
                index: 22,
                line_number: 1,
                offset: 23,
                label: "another".to_string(),
            },
        };
        assert_eq!(token_stream.front().unwrap(), token);
        assert!(token_stream.advance_front().is_err());
    }
}
