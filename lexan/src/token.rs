// Copyright (c) 2026 Peter Williams <pwil3058@bigpond.net.au> <pwil3058@gmail.com>.

use std::fmt::{self, Display, Formatter};
use std::sync::Arc;

use thiserror::Error;

use crate::lexicon::Lexicon;

/// Data for use in user friendly lexical analysis error messages
#[derive(Debug, Clone, PartialEq, Eq, Default, PartialOrd, Ord)]
pub struct Location {
    /// A label describing the source of the string in which this location occurs
    pub label: String,
    /// Current position in the parsed string
    pub index: usize,
    /// Human friendly line number of this location
    pub line_number: usize,
    /// Human friendly offset of this location within its line
    pub offset: usize,
}

impl Location {
    pub(crate) fn new(label: &str) -> Self {
        Self {
            index: 0,
            line_number: 1,
            offset: 1,
            label: label.to_string(),
        }
    }

    pub fn index(&self) -> usize {
        self.index
    }

    pub fn line_number(&self) -> usize {
        self.line_number
    }

    pub fn offset(&self) -> usize {
        self.offset
    }

    pub fn label(&self) -> &String {
        &self.label
    }

    pub(crate) fn step_past(&mut self, slice: &str) {
        let next_index = self.index + slice.len();
        let mut i = 0;
        while i < slice.len() {
            if let Some(eol_i) = slice[i..].find("\r\n") {
                self.line_number += 1;
                self.offset = 1;
                i += eol_i + 2;
            } else if let Some(eol_i) = slice[i..].find('\n') {
                self.line_number += 1;
                self.offset = 1;
                i += eol_i + 1;
            } else {
                self.offset += slice.len() - i;
                i = slice.len();
            };
        }
        self.index = next_index;
    }
}

impl Display for Location {
    fn fmt(&self, dest: &mut Formatter) -> fmt::Result {
        if !self.label.is_empty() {
            if self.label.contains(' ') || self.label.contains('\t') {
                write!(
                    dest,
                    "\"{}\":{}:{}:{}",
                    self.label, self.index, self.line_number, self.offset
                )
            } else {
                write!(
                    dest,
                    "{}:{}:{}:{}",
                    self.label, self.index, self.line_number, self.offset
                )
            }
        } else {
            write!(dest, "{}:{}:{}", self.index, self.line_number, self.offset)
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Token<T: Display + Copy + Eq> {
    pub tag: T,
    pub lexeme: String,
    pub location: Location,
}

impl<T: Display + Copy + Eq> Display for Token<T> {
    fn fmt(&self, dest: &mut fmt::Formatter) -> fmt::Result {
        let string = format!("{}({}) at {}", self.tag, self.lexeme, self.location);
        write!(dest, "{}", string)
    }
}

impl<T: Display + Copy + Eq> Token<T> {
    pub fn tag(&self) -> &T {
        &self.tag
    }

    pub fn lexeme(&self) -> &str {
        &self.lexeme
    }

    pub fn location(&self) -> &Location {
        &self.location
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct List<T: Display + Copy>(Box<[T]>);

impl<T: Display + Copy> FromIterator<T> for List<T> {
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        Self(iter.into_iter().collect::<Vec<T>>().into_boxed_slice())
    }
}

impl<T: Display + Copy> Display for List<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut result = "[".to_string();
        for (i, item) in self.0.iter().enumerate() {
            if i > 0 {
                result.push_str(", ")
            };
            result.push_str(&item.to_string());
        }
        result.push(']');
        write!(f, "{}", result)
    }
}

#[derive(Clone, Debug, PartialEq, Error)]
pub enum Error<T: Display + Copy + fmt::Debug + Eq> {
    #[error("Unexpected text {0} at  {1}")]
    UnexpectedText(String, Location),
    #[error("Ambiguous matches {0} {1} at  {2}")]
    AmbiguousMatches(List<T>, String, Location),
    #[error("Advanced when empty at {0}")]
    AdvancedWhenEmpty(Location),
}

#[derive(Debug)]
pub struct Tokens<T>
where
    T: fmt::Debug + Display + Copy + Eq + Ord,
{
    lexicon: Arc<Lexicon<T>>,
    text: String,
    location: Location,
}

impl<T> Tokens<T>
where
    T: fmt::Debug + Display + Copy + Eq + Ord,
{
    pub(crate) fn new(lexicon: &Arc<Lexicon<T>>, text: &str, label: &str) -> Self {
        Self {
            lexicon: Arc::clone(lexicon),
            text: text.to_string(),
            location: Location::new(label),
        }
    }

    fn incr_location(&mut self, incr: usize) {
        let slice = &self.text[self.location.index..self.location.index + incr];
        self.location.step_past(slice);
    }

    pub fn location(&self) -> Location {
        self.location.clone()
    }

    pub fn is_empty(&self) -> bool {
        let skippable_count = self
            .lexicon
            .skippable_count(&self.text[self.location.index..]);
        self.location.index + skippable_count >= self.text.len()
    }
}

impl<T> Iterator for Tokens<T>
where
    T: fmt::Debug + Display + Copy + Eq + Ord,
{
    type Item = Result<Token<T>, Error<T>>;

    fn next(&mut self) -> Option<Self::Item> {
        let skippable_count = self
            .lexicon
            .skippable_count(&self.text[self.location.index..]);
        self.incr_location(skippable_count);
        if self.location.index >= self.text.len() {
            return None;
        }
        let start = self.location.index;
        let current_location = self.location();
        let longest_regex_matches = self
            .lexicon
            .longest_regex_matches(&self.text[self.location.index..]);
        if let Some(longest_literal_match) = self
            .lexicon
            .longest_literal_match(&self.text[self.location.index..])
        {
            if longest_literal_match.1 >= longest_regex_matches.1 {
                self.incr_location(longest_literal_match.1);
                Some(Ok(Token {
                    tag: longest_literal_match.0,
                    lexeme: (self.text[start..self.location.index]).to_string(),
                    location: current_location,
                }))
            } else if longest_regex_matches.0.len() == 1 {
                self.incr_location(longest_regex_matches.1);
                Some(Ok(Token {
                    tag: longest_regex_matches.0[0],
                    lexeme: (self.text[start..self.location.index]).to_string(),
                    location: current_location,
                }))
            } else {
                self.incr_location(longest_regex_matches.1);
                Some(Err(Error::AmbiguousMatches(
                    List::from_iter(longest_regex_matches.0),
                    (self.text[start..self.location.index]).to_string(),
                    current_location,
                )))
            }
        } else if longest_regex_matches.0.len() == 1 {
            self.incr_location(longest_regex_matches.1);
            Some(Ok(Token {
                tag: longest_regex_matches.0[0],
                lexeme: (self.text[start..self.location.index]).to_string(),
                location: current_location,
            }))
        } else if longest_regex_matches.0.len() > 1 {
            self.incr_location(longest_regex_matches.1);
            Some(Err(Error::AmbiguousMatches(
                List::from_iter(longest_regex_matches.0),
                (self.text[start..self.location.index]).to_string(),
                current_location,
            )))
        } else {
            let distance = self
                .lexicon
                .distance_to_next_valid_byte(&self.text[self.location.index..]);
            self.incr_location(distance);
            Some(Err(Error::UnexpectedText(
                (self.text[start..self.location.index]).to_string(),
                current_location,
            )))
        }
    }
}

#[cfg(test)]
pub mod tests {
    use super::{List, Location, Token, Tokens};
    use crate::lexicon::Lexicon;
    use std::fmt;
    use std::fmt::{Display, Formatter};
    use std::sync::Arc;

    #[test]
    fn format_location() {
        let location = Location {
            index: 120,
            line_number: 10,
            offset: 15,
            label: "whatever".to_string(),
        };
        assert_eq!(format!("{location}"), "whatever:120:10:15");
        let location = Location {
            index: 120,
            line_number: 9,
            offset: 23,
            label: "".to_string(),
        };
        assert_eq!(format!("{location}"), "120:9:23");
    }

    #[test]
    fn step_past() {
        let mut location = Location::new("whatever");
        let text = "String\nwith some new lines\n in it".to_string();
        location.step_past(&text[..11]);
        assert_eq!(location.index, 11);
        assert_eq!(location.line_number, 2);
        assert_eq!(location.offset, 5);
        location.step_past(&text[11..]);
        assert_eq!(location.index, text.len());
        assert_eq!(location.line_number, 3);
        assert_eq!(location.offset, 7);
    }

    #[test]
    fn list() {
        let vec = vec!["a", "b", "c", "d"];
        let list = List::from_iter(vec);
        assert_eq!("[a, b, c, d]", format!("{}", list));
    }

    #[test]
    fn tokens() {
        #[derive(PartialEq, Eq, Clone, Copy, Hash, Debug, PartialOrd, Ord)]
        enum Handle {
            Equals,
            If,
            When,
            Ident,
            End,
        }

        impl Display for Handle {
            fn fmt(&self, f: &mut Formatter) -> fmt::Result {
                use Handle::*;
                match self {
                    Equals => write!(f, "\"==\""),
                    If => write!(f, "\"if\""),
                    When => write!(f, "\"when\""),
                    Ident => write!(f, "Ident"),
                    End => write!(f, "End"),
                }
            }
        }
        use Handle::*;
        let lexicon = Lexicon::new(
            &[(If, "if"), (When, "when"), (Equals, "==")],
            &[(Ident, "[a-zA-Z]+[\\w_]*")],
            &[r"(/\*(.|[\n\r])*?\*/)", r"(//[^\n\r]*)", r"(\s+)"],
            End,
        );
        let lexicon = Arc::new(lexicon.unwrap());
        let text = "      ";
        let label = "label";
        let mut tokens = Tokens::new(&lexicon, text, label);
        assert!(tokens.next().is_none());
        let text = "   if A == B do\n something + ";
        let mut tokens = Tokens::new(&lexicon, text, label);
        let token = Token {
            tag: If,
            lexeme: "if".to_string(),
            location: Location {
                index: 3,
                line_number: 1,
                offset: 4,
                label: "label".to_string(),
            },
        };
        assert_eq!(tokens.next().unwrap(), Ok(token));
        assert!(tokens.next().is_some());
        assert!(tokens.next().is_some());
        assert!(tokens.next().is_some());
        assert!(tokens.next().is_some());
        let token = Token {
            tag: Ident,
            lexeme: "something".to_string(),
            location: Location {
                index: 17,
                line_number: 2,
                offset: 2,
                label: "label".to_string(),
            },
        };
        assert_eq!(tokens.next().unwrap(), Ok(token));
        assert!(tokens.next().unwrap().is_err());
        assert!(tokens.next().is_none());
    }
}
