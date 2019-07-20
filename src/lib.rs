extern crate regex;

use std::{cmp::Eq, collections::HashMap, fmt::Debug};

pub mod analyzer;
pub mod error;
pub mod lexicon;
pub mod matcher;

use crate::error::LexanError;

#[derive(Clone, Copy, PartialEq, Debug)]
struct MatchData<H: PartialEq + Debug + Copy> {
    handle: H,
    length: usize,
}

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
pub struct LiteralMatcher<H: PartialEq + Debug + Copy> {
    lexemes: HashMap<u8, LiteralMatcherNode<H>>,
}

impl<H: Eq + Debug + Copy + Ord> LiteralMatcher<H> {
    pub fn new<'a>(lexemes: &[(H, &'a str)]) -> Result<LiteralMatcher<H>, LexanError<'a, H>> {
        let mut handles = vec![];
        let mut lexes = HashMap::<u8, LiteralMatcherNode<H>>::new();
        for &(handle, pattern) in lexemes.iter() {
            // make sure that handles are unique and strings are not empty
            if pattern.len() == 0 {
                return Err(LexanError::EmptyPattern(handle));
            }
            match handles.binary_search(&handle) {
                Ok(_) => return Err(LexanError::DuplicateHandle(handle)),
                Err(index) => handles.insert(index, handle),
            };

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

#[cfg(test)]
mod tests {
    #[test]
    fn literal_matcher() {
        let lm = crate::LiteralMatcher::new(&[
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
