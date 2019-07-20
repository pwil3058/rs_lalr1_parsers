use std::fmt::Debug;

use regex::Regex;

use crate::error::LexanError;

#[derive(Debug, Default)]
pub struct RegexMatcher<H: Copy + Debug> {
    lexemes: Vec<(H, Regex)>,
}

impl<H: Copy + Ord + Debug> RegexMatcher<H> {
    pub fn new<'a>(lexeme_patterns: &[(H, &'a str)]) -> Result<RegexMatcher<H>, LexanError<'a, H>> {
        let mut handles = vec![];
        let mut patterns = vec![];
        let mut lexemes = vec![];
        for (handle, pattern) in lexeme_patterns.iter() {
            if pattern.len() == 0 {
                return Err(LexanError::EmptyPattern(*handle));
            };
            match handles.binary_search(handle) {
                Ok(_) => return Err(LexanError::DuplicateHandle(*handle)),
                Err(index) => handles.insert(index, *handle),
            }
            match patterns.binary_search(pattern) {
                Ok(_) => return Err(LexanError::DuplicatePattern(pattern)),
                Err(index) => patterns.insert(index, *pattern),
            }
            lexemes.push((*handle, Regex::new(pattern)?));
        }
        Ok(Self { lexemes })
    }

    /// Returns the longest regular expression matches at start of `text`.
    pub fn longest_matches(&self, text: &str) -> (Vec<H>, usize) {
        let mut matches = vec![];
        let mut largest_end = 0;
        for (handle, regex) in self.lexemes.iter() {
            if let Some(m) = regex.find(text) {
                if m.start() == 0 {
                    if m.end() == largest_end {
                        matches.push(*handle);
                    } else if m.end() > largest_end {
                        largest_end = m.end();
                        matches = vec![*handle];
                    }
                }
            }
        }
        (matches, largest_end)
    }

    /// Returns `true` if we match the start of the text
    pub fn matches(&self, text: &str) -> bool {
        for (_, regex) in self.lexemes.iter() {
            if let Some(m) = regex.find(text) {
                if m.start() == 0 {
                    return true;
                }
            }
        }
        false
    }
}
