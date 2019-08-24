pub use std::{
    fmt::{self, Debug, Display},
    sync::Arc,
};

use crate::lexicon::Lexicon;

/// Data for use in user friendly lexical analysis error messages
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Location {
    /// Human friendly line number of this location
    line_number: usize,
    /// Human friendly offset of this location within its line
    offset: usize,
    /// A label describing the source of the string in which this location occurs
    label: String,
}

impl Location {
    fn new(label: String) -> Self {
        Self {
            line_number: 1,
            offset: 1,
            label: label,
        }
    }

    pub fn line_number(&self) -> usize {
        self.line_number
    }

    pub fn offset(&self) -> usize {
        self.offset
    }

    pub fn label<'a>(&'a self) -> &'a String {
        &self.label
    }
}

impl fmt::Display for Location {
    fn fmt(&self, dest: &mut fmt::Formatter) -> fmt::Result {
        if self.label.len() > 0 {
            if self.label.contains(' ') || self.label.contains('\t') {
                write!(
                    dest,
                    "\"{}\":{}:{}",
                    self.label, self.line_number, self.offset
                )
            } else {
                write!(dest, "{}:{}:{}", self.label, self.line_number, self.offset)
            }
        } else {
            write!(dest, "{}:{}", self.line_number, self.offset)
        }
    }
}

#[derive(Clone, Debug)]
pub enum Error<T: Display + Copy> {
    UnexpectedText(String, Location),
    AmbiguousMatches(Vec<T>, String, Location),
    AdvancedWhenEmpty(Location),
}

impl<T: Display + Copy> Error<T> {
    pub fn is_unexpected_text(&self) -> bool {
        match self {
            Error::UnexpectedText(_, _) => true,
            _ => false,
        }
    }

    pub fn is_ambiguous_match(&self) -> bool {
        match self {
            Error::AmbiguousMatches(_, _, _) => true,
            _ => false,
        }
    }

    pub fn is_advance_when_empty(&self) -> bool {
        match self {
            Error::AdvancedWhenEmpty(_) => true,
            _ => false,
        }
    }
}

impl<T: Debug + Display + Copy> fmt::Display for Error<T> {
    fn fmt(&self, dest: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Error::UnexpectedText(text, location) => {
                write!(dest, "Enexpected text \"{}\" at: {}.", text, location)
            }
            Error::AmbiguousMatches(tags, text, location) => write!(
                dest,
                "Ambiguous matches {:#?} \"{}\" at: {}.",
                tags, text, location
            ),
            Error::AdvancedWhenEmpty(location) => write!(
                dest,
                "Advanced past end of text at: {}.",
                location,
            ),
        }
    }
}

impl<T: Debug + Display + Copy> std::error::Error for Error<T> {}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Token<T: Display + Copy + Eq> {
    tag: T,
    lexeme: String,
    location: Location,
}

impl<T: Display + Copy + Eq> Token<T> {
    pub fn tag<'a>(&'a self) -> &'a T {
        &self.tag
    }

    pub fn lexeme<'a>(&'a self) -> &'a String {
        &self.lexeme
    }

    pub fn location<'a>(&'a self) -> &'a Location {
        &self.location
    }
}

struct BasicTokenStream<T>
where
    T: Debug + Display + Copy + Eq + Ord,
{
    lexicon: Arc<Lexicon<T>>,
    text: String,
    index: usize,
    location: Location,
    front: Option<Result<Token<T>, Error<T>>>,
}

impl<T> BasicTokenStream<T>
where
    T: Debug + Display + Copy + Eq + Ord,
{
    pub fn new(lexicon: &Arc<Lexicon<T>>, text: String, label: String) -> Self {
        let location = Location::new(label);
        let mut bts = Self {
            lexicon: Arc::clone(lexicon),
            text,
            location,
            index: 0,
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
        self.front = self.next();
    }

    fn location(&self) -> Location {
        self.location.clone()
    }

    fn incr_index_and_location(&mut self, length: usize) {
        let next_index = self.index + length;
        let slice = &self.text[self.index..next_index];
        let mut i = 0;
        while i < length {
            if let Some(eol_i) = slice[i..].find("\r\n") {
                self.location.line_number += 1;
                self.location.offset = 1;
                i += eol_i + 2;
            } else if let Some(eol_i) = slice[i..].find("\n") {
                self.location.line_number += 1;
                self.location.offset = 1;
                i += eol_i + 1;
            } else {
                self.location.offset += length - i;
                i = length;
            };
        }
        self.index = next_index;
    }

    fn next(&mut self) -> Option<Result<Token<T>, Error<T>>> {
        self.incr_index_and_location(self.lexicon.skippable_count(&self.text[self.index..]));
        if self.index >= self.text.len() {
            return None;
        }

        let current_location = self.location();
        let start = self.index;
        let o_llm = self.lexicon.longest_literal_match(&self.text[self.index..]);
        let lrems = self.lexicon.longest_regex_matches(&self.text[self.index..]);

        if let Some(llm) = o_llm {
            if lrems.0.len() > 1 && lrems.1 > llm.1 {
                self.incr_index_and_location(lrems.1);
                Some(Err(Error::AmbiguousMatches(
                    lrems.0,
                    (&self.text[start..self.index]).to_string(),
                    current_location,
                )))
            } else if lrems.0.len() == 1 && lrems.1 > llm.1 {
                self.incr_index_and_location(lrems.1);
                Some(Ok(Token {
                    tag: lrems.0[0],
                    lexeme: (&self.text[start..self.index]).to_string(),
                    location: current_location,
                }))
            } else {
                self.incr_index_and_location(llm.1);
                Some(Ok(Token {
                    tag: llm.0,
                    lexeme: (&self.text[start..self.index]).to_string(),
                    location: current_location,
                }))
            }
        } else if lrems.0.len() == 1 {
            self.incr_index_and_location(lrems.1);
            Some(Ok(Token {
                tag: lrems.0[0],
                lexeme: (&self.text[start..self.index]).to_string(),
                location: current_location,
            }))
        } else if lrems.0.len() > 1 {
            self.incr_index_and_location(lrems.1);
            Some(Err(Error::AmbiguousMatches(
                lrems.0,
                (&self.text[start..self.index]).to_string(),
                current_location,
            )))
        } else {
            let distance = self.lexicon.distance_to_next_valid_byte(&self.text[self.index..]);
            self.incr_index_and_location(distance);
            Some(Err(Error::UnexpectedText(
                (&self.text[start..self.index]).to_string(),
                current_location,
            )))
        }
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

impl<'a, T> TokenStream<T>
where
    T: Debug + Display + Copy + Eq + Ord,
{
    pub fn new(lexicon: &Arc<Lexicon<T>>, text: String, label: String) -> Self {
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

    pub fn inject(&mut self, text: String, label: String) {
        let token_stream = BasicTokenStream::new(&self.lexicon, text, label);
        if !token_stream.is_empty() {
            self.front = token_stream.front().unwrap();
            self.token_stream_stack.push(token_stream);
        }
    }

    pub fn advance(&mut self) {
        let mut i = self.token_stream_stack.len();
        if i > 0 {
            self.token_stream_stack[i-1].advance();
            let mut popped = None;
            while i > 0 && self.token_stream_stack[i-1].is_empty() {
                popped = self.token_stream_stack.pop();
                i -= 1;
            }
            self.front = if i > 0 {
                self.token_stream_stack[i-1].front().unwrap()
            } else {
                let end_location = popped.unwrap().location();
                Ok(Token{
                    tag: self.lexicon.end_marker(),
                    lexeme: String::new(),
                    location: end_location
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

    #[test]
    fn format_location() {
        let location = Location {
            line_number: 10,
            offset: 15,
            label: "whatever".to_string(),
        };
        assert_eq!(format!("{}", location), "whatever:10:15");
        let location = Location {
            line_number: 9,
            offset: 23,
            label: "".to_string(),
        };
        assert_eq!(format!("{}", location), "9:23");
    }

    #[test]
    fn incr_index_and_location() {
        let lexicon = Arc::new(Lexicon::<u32>::new(&[], &[], &[], 0).unwrap());
        let mut token_stream = BasicTokenStream {
            lexicon: lexicon,
            text: "String\nwith a new line in it".to_string(),
            location: Location::new("whatever".to_string()),
            index: 0,
            front: None,
        };
        token_stream.incr_index_and_location(11);
        println!("{:?}", token_stream.location);
        assert_eq!(token_stream.index, 11);
        assert_eq!(token_stream.location.line_number, 2);
        assert_eq!(token_stream.location.offset, 5);
    }

    #[test]
    fn token_stream_basics() {
        #[derive(PartialEq, Eq, Clone, Copy, Hash, Debug, PartialOrd, Ord)]
        enum Handle {
            If,
            When,
            Ident,
            End,
        }

        impl std::fmt::Display for Handle {
            fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
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
            &[
                (Ident, "[a-zA-Z]+[\\w_]*"),
            ],
            &[r"(/\*(.|[\n\r])*?\*/)", r"(//[^\n\r]*)", r"(\s+)"],
            End,
        );
        let lexicon = Arc::new(lexicon.unwrap());
        let text = "      ".to_string();
        let label = "label".to_string();
        let mut token_stream = TokenStream::new(&lexicon, text, label);
        assert!(token_stream.is_empty());
        assert!(token_stream.front().is_err());
        let text = " if nothing happens 9 ".to_string();
        let label = "another".to_string();
        token_stream.inject(text, label);
        assert!(!token_stream.is_empty());
        let token = Token {
            tag: If,
            lexeme: "if".to_string(),
            location: Location { line_number: 1, offset: 2, label: "another".to_string() },
        };
        assert_eq!((token_stream.front().clone()).unwrap(), token.clone());
        assert_eq!((token_stream.front().clone()).unwrap(), token.clone());
        token_stream.advance();
        let token = Token {
            tag: Ident,
            lexeme: "nothing".to_string(),
            location: Location { line_number: 1, offset: 5, label: "another".to_string() },
        };
        assert_eq!((token_stream.front().clone()).unwrap(), token.clone());
        let text = "just".to_string();
        let label = "more".to_string();
        token_stream.inject(text, label);
        let token = Token {
            tag: Ident,
            lexeme: "just".to_string(),
            location: Location { line_number: 1, offset: 1, label: "more".to_string() },
        };
        assert_eq!((token_stream.front().clone()).unwrap(), token.clone());
        token_stream.advance();
        let token = Token {
            tag: Ident,
            lexeme: "nothing".to_string(),
            location: Location { line_number: 1, offset: 5, label: "another".to_string() },
        };
        assert_eq!((token_stream.front().clone()).unwrap(), token.clone());
        token_stream.advance();
        assert!(token_stream.front().clone().is_ok());
        token_stream.advance();
        assert!(token_stream.front().clone().is_err());
        token_stream.advance();
        let token = Token {
            tag: End,
            lexeme: "".to_string(),
            location: Location { line_number: 1, offset: 23, label: "another".to_string() },
        };
        assert_eq!(token_stream.front().clone().unwrap(), token);
        assert!(token_stream.advance_front().is_err());
    }
}
