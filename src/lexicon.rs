use std::{
    //cmp::Eq,
    //collections::{HashMap, HashSet},
    fmt::Debug,
    hash::Hash,
};

use regex::Regex;

use crate::LiteralMatcher;

#[derive(Default)]
pub struct Lexicon<H>
where
    H: Copy + PartialEq + Debug + Hash + Default
{
    literal_matcher: LiteralMatcher<H>,
    regex_leximes: Vec<(H, Regex)>,
    skip_regex_list: Vec<Regex>,
}

impl<H> Lexicon<H>
where
    H: Copy + Eq + Debug + Hash + Default
{
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
        let mut matches = vec![];
        let mut largest_end = 0;
        for (handle, regex) in self.regex_leximes.iter() {
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
        (matches, largest_end + 1)
    }

    /// Returns the distance in bytes to the next valid content in `text`
    pub fn distance_to_next_valid_byte(&self, text: &str) -> usize {
        for index in 0..text.len() {
            if self.literal_matcher.longest_match(&text[index..]).is_some() {
                return index;
            }
            for (_, regex) in self.regex_leximes.iter() {
                if let Some(m) = regex.find_at(text, index) {
                    if m.start() == index {
                        return index;
                    }
                }
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
