use std::{
    fmt::{self, Debug},
    rc::Rc,
};

use crate::lexicon::Lexicon;

/// Data for use in user friendly lexical analysis error messages
#[derive(Debug, Clone, Copy)]
pub struct Location<'a> {
    /// Human friendly line number of this location
    line_number: usize,
    /// Human friendly offset of this location within its line
    offset: usize,
    /// A label describing the source of the string in which this location occurs
    label: &'a str,
}

impl<'a> Location<'a> {
    fn new(label: &'a str) -> Self {
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

    pub fn label(&self) -> &'a str {
        self.label
    }
}

impl<'a> fmt::Display for Location<'a> {
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
pub enum Error<'a, T: Debug + Copy> {
    UnexpectedText(&'a str, Location<'a>),
    AmbiguousMatches(Vec<T>, &'a str, Location<'a>),
}

impl<'a, T: Debug + Copy> fmt::Display for Error<'a, T> {
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

impl<'a, T: Debug + Copy> std::error::Error for Error<'a, T> {}

#[derive(Debug, Clone, Copy)]
pub struct Token<'a, T: Debug + Copy + Eq> {
    tag: T,
    matched_text: &'a str,
    location: Location<'a>,
}

impl<'a, T: Debug + Copy + Eq> Token<'a, T> {
    pub fn tag<'h>(&'h self) -> &'h T {
        &self.tag
    }

    pub fn matched_text(&'a self) -> &'a str {
        &self.matched_text
    }

    pub fn location(&'a self) -> &'a Location<'a> {
        &self.location
    }
}

pub struct TokenStream<'a, T>
where
    T: Debug + Copy + Eq,
{
    lexicon: Rc<Lexicon<T>>,
    text: &'a str,
    index: usize,
    location: Location<'a>,
}

impl<'a, T> TokenStream<'a, T>
where
    T: Debug + Copy + Eq,
{
    pub fn new(lexicon: &Rc<Lexicon<T>>, text: &'a str, label: &'a str) -> Self {
        let location = Location::new(label);
        Self {
            lexicon: Rc::clone(lexicon),
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
}

impl<'a, T> Iterator for TokenStream<'a, T>
where
    T: Debug + Copy + Eq + Ord,
{
    type Item = Result<Token<'a, T>, Error<'a, T>>;

    fn next(&mut self) -> Option<Self::Item> {
        let text = &self.text[self.index..];
        self.incr_index_and_location(self.lexicon.skippable_count(text));
        if self.index >= self.text.len() {
            return None;
        }

        let current_location = self.location;
        let text = &self.text[self.index..];
        let o_llm = self.lexicon.longest_literal_match(text);
        let lrems = self.lexicon.longest_regex_matches(text);

        if let Some(llm) = o_llm {
            if lrems.0.len() > 1 && lrems.1 > llm.1 {
                self.incr_index_and_location(lrems.1);
                Some(Err(Error::AmbiguousMatches(
                    lrems.0,
                    &text[..lrems.1],
                    current_location,
                )))
            } else if lrems.0.len() == 1 && lrems.1 > llm.1 {
                self.incr_index_and_location(lrems.1);
                Some(Ok(Token {
                    tag: lrems.0[0],
                    matched_text: &text[..lrems.1],
                    location: current_location,
                }))
            } else {
                self.incr_index_and_location(llm.1);
                Some(Ok(Token {
                    tag: llm.0,
                    matched_text: &text[..llm.1],
                    location: current_location,
                }))
            }
        } else if lrems.0.len() == 1 {
            self.incr_index_and_location(lrems.1);
            Some(Ok(Token {
                tag: lrems.0[0],
                matched_text: &text[..lrems.1],
                location: current_location,
            }))
        } else if lrems.0.len() > 1 {
            self.incr_index_and_location(lrems.1);
            Some(Err(Error::AmbiguousMatches(
                lrems.0,
                &text[..lrems.1],
                current_location,
            )))
        } else {
            let distance = self.lexicon.distance_to_next_valid_byte(text);
            self.incr_index_and_location(distance);
            Some(Err(Error::UnexpectedText(
                &text[..distance],
                current_location,
            )))
        }
    }
}

pub struct InjectableTokenStream<'a, T>
where
    T: Debug + Copy + Eq,
{
    lexicon: Rc<Lexicon<T>>,
    token_stream_stack: Vec<TokenStream<'a, T>>,
}

impl<'a, T> InjectableTokenStream<'a, T>
where
    T: Debug + Copy + Eq + Ord,
{
    pub fn new(lexicon: &Rc<Lexicon<T>>, text: &'a str, label: &'a str) -> Self {
        let mut stream = Self {
            lexicon: Rc::clone(lexicon),
            token_stream_stack: vec![],
        };
        stream.inject(text, label);
        stream
    }

    pub fn inject(&mut self, text: &'a str, label: &'a str) {
        let token_stream = TokenStream::new(&self.lexicon, text, label);
        self.token_stream_stack.push(token_stream);
    }
}

impl<'a, T> Iterator for InjectableTokenStream<'a, T>
where
    T: Debug + Copy + Eq + Ord,
{
    type Item = Result<Token<'a, T>, Error<'a, T>>;

    fn next(&mut self) -> Option<Self::Item> {
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
            label: &"whatever",
        };
        assert_eq!(format!("{}", location), "whatever:10:15");
        let location = Location {
            line_number: 9,
            offset: 23,
            label: &"",
        };
        assert_eq!(format!("{}", location), "9:23");
    }

    #[test]
    fn incr_index_and_location() {
        let lexicon = Rc::new(Lexicon::<u32>::new(&[], &[], &[]).unwrap());
        let mut token_stream = TokenStream {
            lexicon: lexicon,
            text: &"String\nwith a new line in it".to_string(),
            location: Location::new("whatever"),
            index: 0,
        };
        token_stream.incr_index_and_location(11);
        println!("{:?}", token_stream.location);
        assert_eq!(token_stream.index, 11);
        assert_eq!(token_stream.location.line_number, 2);
        assert_eq!(token_stream.location.offset, 5);
    }
}
