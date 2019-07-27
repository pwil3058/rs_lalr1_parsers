use std::{cmp::Eq, collections::HashMap, fmt::Debug};

use regex::Regex;

use crate::error::LexanError;

#[derive(Debug, Default)]
struct LiteralMatcherNode<H: PartialEq + Debug + Copy> {
    handle: Option<H>,
    length: usize,
    tails: HashMap<u8, LiteralMatcherNode<H>>,
}

impl<H: PartialEq + Debug + Copy> LiteralMatcherNode<H> {
    fn new(handle: H, string: &str, s_index: usize) -> LiteralMatcherNode<H> {
        debug_assert!(string.len() > 0);
        let mut t = HashMap::<u8, LiteralMatcherNode<H>>::new();
        if string.len() == s_index {
            LiteralMatcherNode {
                handle: Some(handle),
                length: string.len(),
                tails: t,
            }
        } else {
            let key = string.as_bytes()[s_index];
            t.insert(
                key,
                LiteralMatcherNode::<H>::new(handle, string, s_index + 1),
            );
            LiteralMatcherNode {
                handle: None,
                length: s_index,
                tails: t,
            }
        }
    }

    fn add<'a>(
        &mut self,
        handle: H,
        string: &'a str,
        s_index: usize,
    ) -> Result<(), LexanError<'a, H>> {
        debug_assert!(string.len() > 0);
        if string.len() == s_index {
            if self.handle.is_some() {
                return Err(LexanError::DuplicatePattern(string));
            }
            self.handle = Some(handle);
            self.length = string.len();
        } else {
            let key = string.as_bytes()[s_index];
            // Couldn't do this with match because of ownership issues with "tails"
            if self.tails.contains_key(&key) {
                self.tails
                    .get_mut(&key)
                    .unwrap()
                    .add(handle, string, s_index + 1)?;
            } else {
                self.tails.insert(
                    key,
                    LiteralMatcherNode::<H>::new(handle, string, s_index + 1),
                );
            }
        }
        Ok(())
    }
}

#[derive(Debug, Default)]
pub(crate) struct LiteralMatcher<H: PartialEq + Debug + Copy> {
    lexemes: HashMap<u8, LiteralMatcherNode<H>>,
}

impl<H: Eq + Debug + Copy + Ord> LiteralMatcher<H> {
    pub fn new<'a>(lexemes: &[(H, &'a str)]) -> Result<LiteralMatcher<H>, LexanError<'a, H>> {
        let mut lexes = HashMap::<u8, LiteralMatcherNode<H>>::new();
        for &(handle, pattern) in lexemes.iter() {
            // make sure that handles are unique and strings are not empty
            if pattern.len() == 0 {
                return Err(LexanError::EmptyPattern(Some(handle)));
            }

            let key = pattern.as_bytes()[0];
            if lexes.contains_key(&key) {
                lexes.get_mut(&key).unwrap().add(handle, pattern, 1)?;
            } else {
                lexes.insert(key, LiteralMatcherNode::<H>::new(handle, pattern, 1));
            }
        }
        Ok(LiteralMatcher { lexemes: lexes })
    }

    pub fn longest_match(&self, string: &str) -> Option<(H, usize)> {
        let mut rval: Option<(H, usize)> = None;
        let mut lexemes = &self.lexemes;
        for key in string.as_bytes().iter() {
            match lexemes.get(&key) {
                None => break,
                Some(node) => {
                    if let Some(handle) = node.handle {
                        rval = Some((handle, node.length));
                    }
                    lexemes = &node.tails;
                }
            }
        }
        rval
    }

    pub fn matches(&self, string: &str) -> bool {
        let mut lexemes = &self.lexemes;
        for key in string.as_bytes().iter() {
            match lexemes.get(&key) {
                None => break,
                Some(node) => {
                    if node.handle.is_some() {
                        return true;
                    }
                    lexemes = &node.tails;
                }
            }
        }
        false
    }
}

#[derive(Debug, Default)]
pub(crate) struct RegexMatcher<H: Copy + Debug> {
    lexemes: Vec<(H, Regex)>,
}

impl<H: Copy + Ord + Debug> RegexMatcher<H> {
    pub fn new<'a>(lexeme_patterns: &[(H, &'a str)]) -> Result<RegexMatcher<H>, LexanError<'a, H>> {
        let mut lexemes = vec![];
        for (handle, pattern) in lexeme_patterns.iter() {
            if !pattern.starts_with("\\A") {
                return Err(LexanError::UnanchoredRegex(pattern));
            };
            if pattern.len() <= "\\A".len() {
                return Err(LexanError::EmptyPattern(Some(*handle)));
            };
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
pub(crate) struct SkipMatcher {
    regexes: Vec<Regex>,
}

impl SkipMatcher {
    pub fn new<'a, H>(regex_strs: &[&'a str]) -> Result<Self, LexanError<'a, H>> {
        let mut regexes = vec![];
        for regex_str in regex_strs.iter() {
            if !regex_str.starts_with("\\A") {
                return Err(LexanError::UnanchoredRegex(regex_str));
            };
            if regex_str.len() <= "\\A".len() {
                return Err(LexanError::EmptyPattern(None));
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

#[cfg(test)]
mod tests {
    #[test]
    fn literal_matcher() {
        let lm = super::LiteralMatcher::new(&[
            (0, "test"),
            (1, "whatever"),
            (2, "anything"),
            (3, "anything at all"),
        ])
        .unwrap();
        assert!(lm.longest_match("something").is_none());
        assert_eq!(lm.longest_match("anything at all something"), Some((3, 15)));
        assert_eq!(
            lm.longest_match(&"anything at all whatever something"[16..]),
            Some((1, 8))
        );
    }
}
