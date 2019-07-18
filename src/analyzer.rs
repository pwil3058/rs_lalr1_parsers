use std::fmt;

use crate::lexicon::LexiconIfce;

/// Data for use in user friendly lexical analysis error messages
#[derive(Debug, Clone)]
pub struct Location {
    /// Index of this location within the string
    index: usize,
    /// Human friendly line number of this location
    line_number: usize,
    /// Human friendly offset of this location within its line
    offset: usize,
    /// A label describing the source of the string in which this location occurs
    label: String,
}

impl Location {
    fn new(label: &str) -> Self {
        Self {
            index: 0,
            line_number: 1,
            offset: 0,
            label: label.to_string(),
        }
    }
}

impl fmt::Display for Location {
    fn fmt(&self, dest: &mut fmt::Formatter) -> fmt::Result {
        if self.label.len() > 0 {
            write!(dest, "{}:{}({})", self.label, self.line_number, self.offset)
        } else {
            write!(dest, "{}({})", self.line_number, self.offset)
        }
    }
}

#[derive(Debug)]
pub enum Token<'a, H: fmt::Debug> {
    Valid(H, &'a str, Location),
    UnexpectedText(&'a str, Location),
    AmbiguousMatches(Vec<H>, &'a str, Location),
}

pub struct TokenStream<'a, L, H>
where
    L: LexiconIfce<H>,
    H: fmt::Debug + Copy + PartialEq,
{
    lexicon: L,
    text: &'a str,
    index_location: Location,
    end_handle: Option<H>,
}

impl<'a, L, H> TokenStream<'a, L, H>
where
    L: LexiconIfce<H>,
    H: fmt::Debug + Copy + PartialEq,
{
    fn incr_index_location(&mut self, length: usize) {
        let next_index = self.index_location.index + length;
        let slice = &self.text[self.index_location.index..next_index];
        let mut i = 0;
        while i < length {
            if let Some(eol_i) = slice[i..].find("\r\n") {
                self.index_location.line_number += 1;
                self.index_location.offset = 1;
                i += eol_i + 2;
            } else if let Some(eol_i) = slice[i..].find("\n") {
                self.index_location.line_number += 1;
                self.index_location.offset = 1;
                i += eol_i + 1;
            } else {
                self.index_location.offset += length - i;
                i = length;
            };
        }
        self.index_location.index = next_index;
    }
}

impl<'a, L, H> Iterator for TokenStream<'a, L, H>
where
    L: LexiconIfce<H>,
    H: fmt::Debug + Copy + PartialEq,
{
    type Item = Token<'a, H>;

    fn next(&mut self) -> Option<Self::Item> {
        let text = &self.text[self.index_location.index..];
        self.incr_index_location(self.lexicon.skippable_count(text));
        if self.index_location.index >= self.text.len() {
            return None;
        }

        let current_location = self.index_location.clone();
        let text = &self.text[current_location.index..];
        let o_llm = self.lexicon.longest_literal_match(text);
        let lrems = self.lexicon.longest_regex_matches(text);

        if let Some(llm) = o_llm {
            if lrems.0.len() > 1 && lrems.1 > llm.1 {
                self.incr_index_location(lrems.1);
                Some(Token::AmbiguousMatches(lrems.0, &text[..lrems.1], current_location))
            } else if lrems.0.len() == 1 && lrems.1 > llm.1 {
                self.incr_index_location(lrems.1);
                Some(Token::Valid(lrems.0[0], &text[..lrems.1], current_location))
            } else {
                self.incr_index_location(llm.1);
                Some(Token::Valid(llm.0, &text[..llm.1], current_location))
            }
        } else if lrems.0.len() == 1 {
            self.incr_index_location(lrems.1);
            Some(Token::Valid(lrems.0[0], &text[..lrems.1], current_location))
        } else if lrems.0.len() > 1 {
            self.incr_index_location(lrems.1);
            Some(Token::AmbiguousMatches(lrems.0, &text[..lrems.1], current_location))
        } else {
            let distance = self.lexicon.distance_to_next_valid_byte(text);
            self.incr_index_location(distance);
            Some(Token::UnexpectedText(&text[..distance], current_location))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct Lexicon {}

    impl LexiconIfce<u32> for Lexicon {
        /// Returns number of skippable bytes at start of `text`.
        fn skippable_count(&self, text: &str) -> usize {
            0
        }
        /// Returns the longest literal match at start of `text`.
        fn longest_literal_match(&self, text: &str) -> Option<(u32, usize)> {
            None
        }
        /// Returns the longest regular expression match at start of `text`.
        fn longest_regex_matches(&self, text: &str) -> (Vec<u32>, usize) {
            (vec![], 0)
        }
        /// Returns the distance in bytes to the next valid content in `text`
        fn distance_to_next_valid_byte(&self, text: &str) -> usize {
            0
        }
    }

    #[test]
    fn format_location() {
        let location = Location {
            index: 10,
            line_number: 10,
            offset: 15,
            label: "whatever".to_string(),
        };
        assert_eq!(format!("{}", location), "whatever:10(15)");
        let location = Location {
            index: 100,
            line_number: 9,
            offset: 23,
            label: String::new(),
        };
        assert_eq!(format!("{}", location), "9(23)");
    }

    #[test]
    fn incr_index_location() {
        let mut token_stream = TokenStream {
            lexicon: Lexicon {},
            text: &"String\nwith a new line in it".to_string(),
            index_location: Location::new("whatever"),
            end_handle: None,
        };
        token_stream.incr_index_location(11);
        println!("{:?}", token_stream.index_location);
        assert_eq!(token_stream.index_location.index, 11);
        assert_eq!(token_stream.index_location.line_number, 2);
        assert_eq!(token_stream.index_location.offset, 5);
    }
}
