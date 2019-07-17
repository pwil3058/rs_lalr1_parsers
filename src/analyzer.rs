use std::fmt;

use crate::lexicon::LexiconIfce;

/// Data for use in user friendly lexical analysis error messages
#[derive(Debug)]
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
pub struct Token<'a, H>
where
    H: fmt::Debug,
{
    handle: H,
    matched_text: &'a str,
    location: Location,
}

pub struct TokenStream<'a, L, H>
where
    L: LexiconIfce<H>,
    H: fmt::Debug + Copy + PartialEq,
{
    lexicon: L, // Temporay for testing
    text: String,
    index_location: Location,
    current_match: Option<Token<'a, H>>,
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

#[cfg(test)]
mod tests {
    use super::*;

    struct Lexicon {}

    impl LexiconIfce<u32> for Lexicon {
        /// Returns number of skippable bytes at start of `text`.
        fn skippable_count(&self, text: &str) -> usize {0}
        /// Returns the longest literal match at start of `text`.
        fn longest_literal_match(&self, text: &str) -> Option<(u32, usize)> { None }
        /// Returns the longest regular expression match at start of `text`.
        fn longest_regex_match(&self, text: &str) -> Option<(u32, usize)> { None }
        /// Returns the distance in bytes to the next valid content in `text`
        fn distance_to_next_valid_byte(&self, text: &str) -> Option<usize> { None }
    }

    #[test]
    fn format_location() {
        let location = Location{ index: 10, line_number: 10, offset: 15, label: "whatever".to_string()};
        assert_eq!(format!("{}", location), "whatever:10(15)");
        let location = Location{ index: 100, line_number: 9, offset: 23, label: String::new()};
        assert_eq!(format!("{}", location), "9(23)");
    }

    #[test]
    fn incr_index_location() {
        let mut token_stream = TokenStream {
            lexicon: Lexicon{},
            text: "String\nwith a new line in it".to_string(),
            index_location: Location::new("whatever"),
            current_match: None,
            end_handle: None,
        };
        token_stream.incr_index_location(11);
        println!("{:?}", token_stream.index_location);
        assert_eq!(token_stream.index_location.index, 11);
        assert_eq!(token_stream.index_location.line_number, 2);
        assert_eq!(token_stream.index_location.offset, 5);
    }
}