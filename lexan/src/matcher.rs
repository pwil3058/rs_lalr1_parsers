use std::{cmp::Eq, collections::HashMap, fmt::Debug};

use regex::Regex;

use crate::error::LexanError;

#[derive(Debug, Default)]
struct LiteralMatcherNode<T: PartialEq + Debug + Copy> {
    tag: Option<T>,
    length: usize,
    tails: HashMap<u8, LiteralMatcherNode<T>>,
}

impl<T: PartialEq + Debug + Copy> LiteralMatcherNode<T> {
    fn new(tag: T, string: &str, s_index: usize) -> LiteralMatcherNode<T> {
        debug_assert!(string.len() > 0);
        let mut t = HashMap::<u8, LiteralMatcherNode<T>>::new();
        if string.len() == s_index {
            LiteralMatcherNode {
                tag: Some(tag),
                length: string.len(),
                tails: t,
            }
        } else {
            let key = string.as_bytes()[s_index];
            t.insert(key, LiteralMatcherNode::<T>::new(tag, string, s_index + 1));
            LiteralMatcherNode {
                tag: None,
                length: s_index,
                tails: t,
            }
        }
    }

    fn add<'a>(
        &mut self,
        tag: T,
        string: &'a str,
        s_index: usize,
    ) -> Result<(), LexanError<'a, T>> {
        debug_assert!(string.len() > 0);
        if string.len() == s_index {
            if self.tag.is_some() {
                return Err(LexanError::DuplicatePattern(string));
            }
            self.tag = Some(tag);
            self.length = string.len();
        } else {
            let key = string.as_bytes()[s_index];
            // Couldn't do this with match because of ownership issues with "tails"
            if self.tails.contains_key(&key) {
                self.tails
                    .get_mut(&key)
                    .unwrap()
                    .add(tag, string, s_index + 1)?;
            } else {
                self.tails
                    .insert(key, LiteralMatcherNode::<T>::new(tag, string, s_index + 1));
            }
        }
        Ok(())
    }
}

#[derive(Debug, Default)]
pub(crate) struct LiteralMatcher<T: PartialEq + Debug + Copy> {
    lexemes: HashMap<u8, LiteralMatcherNode<T>>,
}

impl<T: Eq + Debug + Copy + Ord> LiteralMatcher<T> {
    pub fn new<'a>(lexemes: &[(T, &'a str)]) -> Result<LiteralMatcher<T>, LexanError<'a, T>> {
        let mut lexes = HashMap::<u8, LiteralMatcherNode<T>>::new();
        for &(tag, pattern) in lexemes.iter() {
            // make sure that tags are unique and strings are not empty
            if pattern.len() == 0 {
                return Err(LexanError::EmptyPattern(Some(tag)));
            }

            let key = pattern.as_bytes()[0];
            if lexes.contains_key(&key) {
                lexes.get_mut(&key).unwrap().add(tag, pattern, 1)?;
            } else {
                lexes.insert(key, LiteralMatcherNode::<T>::new(tag, pattern, 1));
            }
        }
        Ok(LiteralMatcher { lexemes: lexes })
    }

    pub fn longest_match(&self, string: &str) -> Option<(T, usize)> {
        let mut rval: Option<(T, usize)> = None;
        let mut lexemes = &self.lexemes;
        for key in string.as_bytes().iter() {
            match lexemes.get(&key) {
                None => break,
                Some(node) => {
                    if let Some(tag) = node.tag {
                        rval = Some((tag, node.length));
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
                    if node.tag.is_some() {
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
pub(crate) struct RegexMatcher<T: Copy + Debug> {
    lexemes: Vec<(T, Regex)>,
}

impl<T: Copy + Ord + Debug> RegexMatcher<T> {
    pub fn new<'a>(lexeme_patterns: &[(T, &'a str)]) -> Result<RegexMatcher<T>, LexanError<'a, T>> {
        let mut lexemes = vec![];
        for (tag, pattern) in lexeme_patterns.iter() {
            if pattern.len() == 0 {
                return Err(LexanError::EmptyPattern(Some(*tag)));
            };
            let mut anchored_pattern = "\\A".to_string();
            anchored_pattern.push_str(pattern);
            lexemes.push((*tag, Regex::new(&anchored_pattern)?));
        }
        Ok(Self { lexemes })
    }

    /// Returns the longest regular expression matches at start of `text`.
    pub fn longest_matches(&self, text: &str) -> (Vec<T>, usize) {
        let mut matches = vec![];
        let mut largest_end = 0;
        for (tag, regex) in self.lexemes.iter() {
            if let Some(m) = regex.find(text) {
                if m.end() == largest_end {
                    matches.push(*tag);
                } else if m.end() > largest_end {
                    largest_end = m.end();
                    matches = vec![*tag];
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
    pub fn new<'a, T>(regex_strs: &[&'a str]) -> Result<Self, LexanError<'a, T>> {
        let mut regexes = vec![];
        for regex_str in regex_strs.iter() {
            if regex_str.len() == 0 {
                return Err(LexanError::EmptyPattern(None));
            };
            let mut anchored_pattern = "\\A".to_string();
            anchored_pattern.push_str(regex_str);
            regexes.push(Regex::new(&anchored_pattern)?);
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
