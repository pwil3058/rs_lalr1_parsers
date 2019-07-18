use std::{
    //cmp::Eq,
    //collections::{HashMap, HashSet},
    fmt::Debug,
    hash::Hash,
};

use regex::Regex;

use crate::LiteralMatcher;

pub trait LexiconIfce<H>
where
    H: Copy + PartialEq,
{
    /// Returns number of skippable bytes at start of `text`.
    fn skippable_count(&self, text: &str) -> usize;
    /// Returns the longest literal match at start of `text`.
    fn longest_literal_match(&self, text: &str) -> Option<(H, usize)>;
    /// Returns the longest regular expression match at start of `text`.
    fn longest_regex_matches(&self, text: &str) -> (Vec<H>, usize);
    /// Returns the distance in bytes to the next valid content in `text`
    fn distance_to_next_valid_byte(&self, text: &str) -> usize
    ;
}

pub struct Lexicon<H>
where
    H: Copy + PartialEq + Debug + Hash
{
    literal_matcher: LiteralMatcher<H>,
    regex_leximes: Vec<(H, Regex)>,
    skip_regex_list: Vec<Regex>,
}

impl<H> LexiconIfce<H> for Lexicon<H>
where
    H: Copy + PartialEq + Debug + Hash
{
    /// Returns number of skippable bytes at start of `text`.
    fn skippable_count(&self, text: &str) -> usize {
        let mut index = 0;
        while index < text.len() {
            let mut skips = 0;
            for skip_regex in self.skip_regex_list.iter() {
                if let Some(cap) = skip_regex.captures(&text[index..]) {
                    index += cap.len();
                    skips += 1;
                }
            }
            if skips == 0 {
                 break;
            }
        }
        index
    }
    /// Returns the longest literal match at start of `text`.
    fn longest_literal_match(&self, text: &str) -> Option<(H, usize)> {
        None
    }
    /// Returns the longest regular expression match at start of `text`.
    fn longest_regex_matches(&self, text: &str) -> (Vec<H>, usize) {
        (vec![], 0)
    }
    /// Returns the distance in bytes to the next valid content in `text`
    fn distance_to_next_valid_byte(&self, text: &str) -> usize {
        0
    }
}
