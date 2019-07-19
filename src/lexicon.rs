use std::{
    //cmp::Eq,
    collections::HashSet,
    fmt::Debug,
    hash::Hash,
};

use regex::Regex;

use crate::error::LexanError;
use crate::matcher::RegexMatcher;
use crate::LiteralMatcher;

#[derive(Default)]
pub struct Lexicon<H>
where
    H: Copy + PartialEq + Debug + Hash + Default,
{
    literal_matcher: LiteralMatcher<H>,
    regex_matcher: RegexMatcher<H>,
    skip_regex_list: Vec<Regex>,
}

impl<H> Lexicon<H>
where
    H: Copy + Eq + Debug + Hash + Default + Ord,
{
    pub fn new<'a>(
        literal_lexemes: &[(H, &'a str)],
        regex_lexeme_strs: &[(H, &'a str)],
    ) -> Result<Self, LexanError<'a, H>> {
        let mut handles: HashSet<H> = literal_lexemes.iter().map(|x| x.0).collect();
        let _literal_matcher = LiteralMatcher::new(literal_lexemes)?;
        let regex_lexemes: Vec<(H, Regex)> = Vec::new();
        for (handle, regex_str) in regex_lexeme_strs.iter() {
            if !handles.insert(*handle) {
                return Err(LexanError::DuplicateHandle(*handle));
            }
        }
        Ok(Self::default())
    }

    /// Returns number of skippable bytes at start of `text`.
    pub fn skippable_count(&self, text: &str) -> usize {
        let mut index = 0;
        while index < text.len() {
            let mut skips = 0;
            for skip_regex in self.skip_regex_list.iter() {
                if let Some(m) = skip_regex.find_at(text, index) {
                    if m.start() == index {
                        index = m.end() + 1;
                        skips += 1;
                    }
                }
            }
            if skips == 0 {
                break;
            }
        }
        index
    }

    /// Returns the longest literal match at start of `text`.
    pub fn longest_literal_match(&self, text: &str) -> Option<(H, usize)> {
        self.literal_matcher.longest_match(text)
    }

    /// Returns the longest regular expression match at start of `text`.
    pub fn longest_regex_matches(&self, text: &str) -> (Vec<H>, usize) {
        self.regex_matcher.longest_matches(text)
    }

    /// Returns the distance in bytes to the next valid content in `text`
    pub fn distance_to_next_valid_byte(&self, text: &str) -> usize {
        for index in 0..text.len() {
            if self.literal_matcher.matches(&text[index..]) {
                return index;
            }
            if self.regex_matcher.matches(&text[index..]) {
                return index;
            }
            for regex in self.skip_regex_list.iter() {
                if let Some(m) = regex.find_at(text, index) {
                    if m.start() == index {
                        return index;
                    }
                }
            }
        }
        text.len()
    }
}
