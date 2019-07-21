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
            if !pattern.starts_with("\\A") {
                return Err(LexanError::UnanchoredRegex(pattern));
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
                if m.end() == largest_end {
                    matches.push(*handle);
                } else if m.end() > largest_end {
                    largest_end = m.end();
                    matches = vec![*handle];
                }
            }
        }
        (matches, largest_end)
    }

    /// Returns `true` if we match the start of the text
    pub fn matches(&self, text: &str) -> bool {
        for (_, regex) in self.lexemes.iter() {
            if regex.find(text).is_some() {
                return true;
            }
        }
        false
    }
}

#[derive(Debug, Default)]
pub struct SkipMatcher {
    regexes: Vec<Regex>,
}

impl SkipMatcher {
    pub fn new<'a, H>(regex_strs: &[&'a str]) -> Result<Self, LexanError<'a, H>> {
        let mut regexes = vec![];
        for regex_str in regex_strs.iter() {
            if !regex_str.starts_with("\\A") {
                return Err(LexanError::UnanchoredRegex(regex_str));
            };
            regexes.push(Regex::new(regex_str)?);
        }
        Ok(Self { regexes })
    }

    /// Returns number of skippable bytes at start of `text`.
    pub fn skippable_count(&self, text: &str) -> usize {
        let mut index = 0;
        'outer: while index < text.len() {
            for regex in self.regexes.iter() {
                if let Some(m) = regex.find(&text[index..]) {
                    index += m.end();
                    continue 'outer;
                }
            }
            break;
        }
        index
    }

    pub fn matches(&self, text: &str) -> bool {
        for regex in self.regexes.iter() {
            if regex.find(text).is_some() {
                return true;
            }
        }
        false
    }
}
