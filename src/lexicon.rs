use std::fmt::Debug;

use regex::Regex;

use crate::analyzer::TokenStream;
use crate::error::LexanError;
use crate::matcher::RegexMatcher;
use crate::LiteralMatcher;

#[derive(Default)]
pub struct Lexicon<H>
where
    H: Copy + PartialEq + Debug,
{
    literal_matcher: LiteralMatcher<H>,
    regex_matcher: RegexMatcher<H>,
    skip_regexes: Vec<Regex>,
}

impl<H> Lexicon<H>
where
    H: Copy + Eq + Debug + Ord,
{
    pub fn new<'a>(
        literal_lexemes: &[(H, &'a str)],
        regex_lexemes: &[(H, &'a str)],
        skip_regex_strs: &[&'a str],
    ) -> Result<Self, LexanError<'a, H>> {
        let literal_matcher = LiteralMatcher::new(literal_lexemes)?;
        let regex_matcher = RegexMatcher::new(regex_lexemes)?;
        let mut skip_regexes = vec![];
        for skip_regex_str in skip_regex_strs.iter() {
            skip_regexes.push(Regex::new(skip_regex_str)?);
        }
        Ok(Self {
            literal_matcher,
            regex_matcher,
            skip_regexes,
        })
    }

    /// Returns number of skippable bytes at start of `text`.
    pub fn skippable_count(&self, text: &str) -> usize {
        let mut index = 0;
        while index < text.len() {
            let mut skips = 0;
            for skip_regex in self.skip_regexes.iter() {
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
            for regex in self.skip_regexes.iter() {
                if let Some(m) = regex.find_at(text, index) {
                    if m.start() == index {
                        return index;
                    }
                }
            }
        }
        text.len()
    }

    pub fn token_stream<'a>(&'a self, text: &'a str, label: &'a str) -> TokenStream<'a, H> {
        TokenStream::new(self, text, label)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(PartialEq, Eq, Clone, Copy, Hash, Debug, PartialOrd, Ord)]
    enum Handle {
        If,
        When,
        Ident,
        Btextl,
        Pred,
        Literal,
        Action,
        Predicate,
        Code,
        Morse,
    }

    #[test]
    fn streamer_basic() {
        let lexicon = Lexicon::<Handle>::new(
            &[],
            &[],
            &[],
        );
    }
}
