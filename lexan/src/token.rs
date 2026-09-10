// Copyright (c) 2026 Peter Williams <pwil3058@bigpond.net.au> <pwil3058@gmail.com>.

use std::fmt;

/// Data for use in user friendly lexical analysis error messages
#[derive(Debug, Clone, PartialEq, Eq, Default, PartialOrd, Ord)]
pub struct Location {
    /// A label describing the source of the string in which this location occurs
    label: String,
    /// Current position in the parsed string
    index: usize,
    /// Human friendly line number of this location
    line_number: usize,
    /// Human friendly offset of this location within its line
    offset: usize,
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

impl fmt::Display for Location {
    fn fmt(&self, dest: &mut fmt::Formatter) -> fmt::Result {
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

#[cfg(test)]
pub mod tests {
    use super::Location;

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
}
