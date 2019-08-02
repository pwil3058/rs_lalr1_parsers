use std::{
    fmt::{self, Debug},
    sync::Arc,
};

use crate::lexicon::Lexicon;

/// Data for use in user friendly lexical analysis error messages
#[derive(Debug, Clone)]
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

    pub fn label<'a>(&'a self) -> &'a str {
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

#[derive(Debug, Clone)]
pub enum Error<T: Debug + Copy> {
    UnexpectedText(String, Location),
    AmbiguousMatches(Vec<T>, String, Location),
}

impl<T: Debug + Copy> Error<T> {
    pub fn is_unexpected_text(&self) -> bool {
        match self {
            Error::UnexpectedText(_, _) => true,
            Error::AmbiguousMatches(_, _, _) => false,
        }
    }

    pub fn is_ambiguous_match(&self) -> bool {
        match self {
            Error::UnexpectedText(_, _) => false,
            Error::AmbiguousMatches(_, _, _) => true,
        }
    }
}

impl<T: Debug + Copy> fmt::Display for Error<T> {
    fn fmt(&self, dest: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Error::UnexpectedText(text, location) => {
                write!(dest, "Enexpected text \"{}\" at: {}", text, location)
            }
            Error::AmbiguousMatches(tags, text, location) => write!(
                dest,
                "Ambiguous matches {:?} \"{}\" at: {}",
                tags, text, location
            ),
        }
    }
}

impl<T: Debug + Copy> std::error::Error for Error<T> {}

#[derive(Debug, Clone)]
pub struct Token<T: Debug + Copy + Eq> {
    tag: T,
    lexeme: String,
    location: Location,
}

impl<T: Debug + Copy + Eq> Token<T> {
    pub fn tag<'a>(&'a self) -> &'a T {
        &self.tag
    }

    pub fn lexeme<'a>(&'a self) -> &'a str {
        &self.lexeme
    }

    pub fn location<'a>(&'a self) -> &'a Location {
        &self.location
    }
}

struct BasicTokenStream<T>
where
    T: Debug + Copy + Eq + Ord,
{
    lexicon: Arc<Lexicon<T>>,
    text: String,
    index: usize,
    location: Location,
}

impl<T> BasicTokenStream<T>
where
    T: Debug + Copy + Eq + Ord,
{
    pub fn new(lexicon: &Arc<Lexicon<T>>, text: String, label: String) -> Self {
        let location = Location::new(label);
        Self {
            lexicon: Arc::clone(lexicon),
            text,
            location,
            index: 0,
        }
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
        //let text = &self.text[self.index..];
        self.incr_index_and_location(self.lexicon.skippable_count(&self.text[self.index..]));
        if self.index >= self.text.len() {
            return None;
        }

        let current_location = self.location.clone();
        //let text = &self.text[self.index..];
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
    T: Debug + Copy + Eq + Ord,
{
    lexicon: Arc<Lexicon<T>>,
    token_stream_stack: Vec<BasicTokenStream<T>>,
}

impl<'a, T> TokenStream<T>
where
    T: Debug + Copy + Eq + Ord,
{
    pub fn new(lexicon: &Arc<Lexicon<T>>, text: String, label: String) -> Self {
        let mut stream = Self {
            lexicon: Arc::clone(lexicon),
            token_stream_stack: vec![],
        };
        stream.inject(text, label);
        stream
    }

    pub fn inject(&mut self, text: String, label: String) {
        let token_stream = BasicTokenStream::new(&self.lexicon, text, label);
        self.token_stream_stack.push(token_stream);
    }

    pub fn next(&mut self) -> Option<Result<Token<T>, Error<T>>> {
        loop {
            if let Some(token_stream) = self.token_stream_stack.last_mut() {
                if let Some(token) = token_stream.next() {
                    return Some(token);
                } else {
                    self.token_stream_stack.pop();
                }
            } else {
                return None;
            }
        }
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
        let lexicon = Arc::new(Lexicon::<u32>::new(&[], &[], &[]).unwrap());
        let mut token_stream = BasicTokenStream {
            lexicon: lexicon,
            text: "String\nwith a new line in it".to_string(),
            location: Location::new("whatever".to_string()),
            index: 0,
        };
        token_stream.incr_index_and_location(11);
        println!("{:?}", token_stream.location);
        assert_eq!(token_stream.index, 11);
        assert_eq!(token_stream.location.line_number, 2);
        assert_eq!(token_stream.location.offset, 5);
    }
}
